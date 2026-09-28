//! Ordinary activations and defaults scheduled without recursive Rust invocation.
use super::{
    Document, ExprFrame, Frame as Job, ListOwner, Output, Phase as ExprPhase, Result, Runtime,
    code, enter_frame, push, statements, value,
};
use crate::script::{
    DeclarationKind, Flow, JsString, Native, NumberPredicate, ParameterStorage, ScriptError, Value,
};
use std::collections::BTreeSet;
use std::rc::Rc;

pub(in crate::script) struct Frame {
    owns_call: bool,
    phase: Phase,
}
enum Phase {
    Start {
        function: Value,
        arguments: Vec<Value>,
        receiver: Value,
        native_guarded: bool,
    },
    Default {
        activation: Activation,
        index: usize,
    },
    Body,
    Forward,
    Vacant,
}
struct Activation {
    code: code::FunctionRef,
    env: usize,
    arguments: Vec<Value>,
    arguments_binding: bool,
    parameter_expressions: bool,
}
impl Frame {
    pub(super) fn new(
        function: Value,
        arguments: Vec<Value>,
        receiver: Value,
        preentered: bool,
    ) -> Self {
        Self {
            owns_call: !preentered,
            phase: Phase::Start {
                function,
                arguments,
                receiver,
                native_guarded: preentered,
            },
        }
    }
    pub(super) fn owns_call(&self) -> bool {
        self.owns_call
    }
}
fn schedule(
    runtime: &mut Runtime,
    mut frame: Frame,
    phase: Phase,
    next: Job,
) -> Result<Option<Output>> {
    frame.phase = phase;
    push(runtime, Job::Call(frame))?;
    enter_frame(runtime, next)?;
    Ok(None)
}
pub(super) fn step(
    runtime: &mut Runtime,
    mut frame: Frame,
    output: Option<Output>,
    doc: &mut Document,
) -> Result<Option<Output>> {
    match std::mem::replace(&mut frame.phase, Phase::Vacant) {
        Phase::Start {
            function,
            arguments,
            receiver,
            native_guarded,
        } => match function {
            Value::Function(id) => setup(runtime, frame, id, arguments, receiver, doc),
            Value::Native(native) => {
                invoke_native(runtime, native, arguments, receiver, native_guarded, doc)
            }
            _ => Err(ScriptError::type_error("value is not callable")),
        },
        Phase::Default { activation, index } => {
            let value = value(output);
            let parameter = &activation.code.params[index];
            let initializer = parameter.initializer.expect("default initializer");
            if activation.code.unit.anonymous(initializer) {
                runtime.charge(parameter.name.len().saturating_mul(4))?;
                runtime.set_function_name(
                    &value,
                    &JsString::from(parameter.name.as_str()),
                    None,
                )?;
            }
            initialize_parameter(runtime, &activation, index, value);
            parameters(runtime, frame, activation, index + 1, doc)
        }
        Phase::Body => match output {
            Some(Output::Flow(Flow::Return(value))) => Ok(Some(Output::Value(value))),
            Some(Output::Flow(Flow::Normal(_))) => Ok(Some(Output::Value(Value::Undefined))),
            Some(Output::Flow(_)) => Err(ScriptError::new("loop control outside loop")),
            _ => unreachable!("function body completion"),
        },
        Phase::Forward => Ok(output),
        Phase::Vacant => unreachable!("uninitialized invocation continuation"),
    }
}
fn invoke_native(
    runtime: &mut Runtime,
    native: Rc<Native>,
    arguments: Vec<Value>,
    receiver: Value,
    already_guarded: bool,
    doc: &mut Document,
) -> Result<Option<Output>> {
    if !already_guarded {
        runtime.enter_stack(4)?;
    }
    let result = if NumberPredicate::from_name(&native.name).is_some() {
        runtime.native_call(&native, arguments, doc)
    } else if native.name.contains('.') {
        runtime.native_call(
            &Native {
                name: native.name.clone(),
                receiver,
            },
            arguments,
            doc,
        )
    } else {
        runtime.native_call(&native, arguments, doc)
    };
    if !already_guarded {
        runtime.stack_units -= 4;
    }
    result.map(|value| Some(Output::Value(value)))
}
fn setup(
    runtime: &mut Runtime,
    frame: Frame,
    id: usize,
    arguments: Vec<Value>,
    receiver: Value,
    doc: &mut Document,
) -> Result<Option<Output>> {
    // Keep the conservative legacy activation allowance while sharing all code.
    // Argument forwarding below reserves one checked buffer instead of cloning a
    // bound vector and then growing it during recursive forwarding.
    let parameter_count = runtime.functions[id].code.params.len();
    runtime.work(1 + parameter_count)?;
    let code = &runtime.functions[id].code;
    let text_bytes = code.params.iter().fold(0usize, |size, parameter| {
        size.saturating_add(parameter.name.len())
    });
    let name_bytes = code.name.as_ref().map_or(0, String::len);
    let bound_bytes = runtime.functions[id].bound.as_ref().map_or(0, |bound| {
        bound
            .arguments
            .len()
            .saturating_mul(std::mem::size_of::<Value>())
    });
    runtime.work(1 + text_bytes.saturating_add(name_bytes) / 8)?;
    runtime.charge(
        128usize
            .saturating_add(parameter_count.saturating_mul(std::mem::size_of::<ParameterStorage>()))
            .saturating_add(text_bytes)
            .saturating_add(name_bytes)
            .saturating_add(bound_bytes),
    )?;
    if runtime.functions[id].bound.is_some() {
        return forward(runtime, frame, id, arguments);
    }
    let code = runtime.functions[id].code.clone();
    let env = runtime.environment(runtime.functions[id].environment)?;
    runtime.environments[env].function_scope = true;
    runtime.environments[env].strict = code.strict;
    if !code.arrow {
        let receiver = if code.strict {
            receiver
        } else if matches!(receiver, Value::Undefined | Value::Null) {
            Value::Window
        } else {
            runtime.coerce_object(receiver)?
        };
        runtime.environments[env].this_binding = Some(receiver);
    }
    let parameter_expressions = code.has_parameter_expressions();
    // Every formal exists before the first initializer; expression parameters
    // remain in their TDZ until initialized in source order.
    for parameter in &code.params {
        if !runtime.environments[env]
            .bindings
            .contains_key(&parameter.name)
        {
            runtime.define(env, &parameter.name, Value::Undefined, true)?;
            runtime.environments[env]
                .bindings
                .get_mut(&parameter.name)
                .unwrap()
                .initialized = !parameter_expressions;
        }
    }
    let shadows_arguments = code.params.iter().any(|p| p.name == "arguments")
        || !parameter_expressions && code.body.iter().any(|s| matches!(code.unit.stmt(*s), code::Stmt::Function(name, _) if name == "arguments")
            || matches!(code.unit.stmt(*s), code::Stmt::Var(bindings, kind) if *kind != DeclarationKind::Var && bindings.iter().any(|(name, _)| name == "arguments")));
    let arguments_binding = !code.arrow && !shadows_arguments;
    if arguments_binding {
        let unmapped = code.strict || !code.has_simple_parameters();
        let args = runtime.arguments_object(&arguments, unmapped, Value::Function(id))?;
        if !unmapped {
            let Value::Object(id) = args else {
                unreachable!()
            };
            let mut seen = BTreeSet::new();
            for (index, parameter) in code.params.iter().enumerate().rev() {
                let name = &parameter.name;
                if seen.insert(name) && index < arguments.len() {
                    runtime.charge(96 + name.len())?;
                    runtime.objects[id]
                        .parameter_map
                        .insert(index.to_string().into(), (env, name.clone()));
                }
            }
        }
        runtime.define(env, "arguments", args, true)?;
    }
    parameters(
        runtime,
        frame,
        Activation {
            code,
            env,
            arguments,
            arguments_binding,
            parameter_expressions,
        },
        0,
        doc,
    )
}
fn forward(
    runtime: &mut Runtime,
    frame: Frame,
    id: usize,
    arguments: Vec<Value>,
) -> Result<Option<Output>> {
    let bound = runtime.functions[id].bound.as_ref().unwrap();
    let length = bound.arguments.len().saturating_add(arguments.len());
    let target = bound.target.clone();
    let receiver = bound.receiver.clone();
    runtime.work(length)?;
    runtime.charge(length.saturating_mul(std::mem::size_of::<Value>()))?;
    let mut combined = Vec::new();
    combined
        .try_reserve_exact(length)
        .map_err(|_| ScriptError::resource("bound arguments allocation failed"))?;
    combined.extend(
        runtime.functions[id]
            .bound
            .as_ref()
            .unwrap()
            .arguments
            .iter()
            .cloned(),
    );
    combined.extend(arguments);
    let next = Job::Call(Frame::new(target, combined, receiver, false));
    schedule(runtime, frame, Phase::Forward, next)
}
fn initialize_parameter(
    runtime: &mut Runtime,
    activation: &Activation,
    index: usize,
    value: Value,
) {
    let binding = runtime.environments[activation.env]
        .bindings
        .get_mut(&activation.code.params[index].name)
        .unwrap();
    binding.value = value;
    binding.initialized = true;
}
fn parameters(
    runtime: &mut Runtime,
    frame: Frame,
    activation: Activation,
    mut index: usize,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let code = activation.code.clone();
    while let Some(parameter) = code.params.get(index) {
        runtime.tick()?;
        let value = if parameter.rest {
            runtime
                .rest_arguments(&activation.arguments[index.min(activation.arguments.len())..])?
        } else {
            activation
                .arguments
                .get(index)
                .cloned()
                .unwrap_or(Value::Undefined)
        };
        if matches!(value, Value::Undefined)
            && let Some(initializer) = parameter.initializer
        {
            let next = Job::Expression(ExprFrame {
                unit: code.unit.clone(),
                expression: initializer,
                env: activation.env,
                reference: false,
                phase: ExprPhase::Start,
            });
            return schedule(runtime, frame, Phase::Default { activation, index }, next);
        }
        initialize_parameter(runtime, &activation, index, value);
        index += 1;
    }
    body(runtime, frame, activation, doc)
}
fn body(
    runtime: &mut Runtime,
    frame: Frame,
    activation: Activation,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let Activation {
        code,
        env,
        arguments,
        arguments_binding,
        parameter_expressions,
    } = activation;
    // Parameter and arguments bindings retain their own values. The evaluation
    // vector is no longer needed once defaults/rest initialization has finished.
    drop(arguments);
    let body_env = if parameter_expressions {
        // Default closures retain the parameter environment; body declarations
        // never become visible to those closures retroactively.
        let child = runtime.environment(env)?;
        runtime.environments[child].function_scope = true;
        runtime.hoist_vars(&code.unit, &code.body, child)?;
        for parameter in &code.params {
            runtime.tick()?;
            if runtime.environments[child]
                .bindings
                .contains_key(&parameter.name)
            {
                let value = runtime.binding_value(env, &parameter.name, doc)?;
                runtime.environments[child]
                    .bindings
                    .get_mut(&parameter.name)
                    .unwrap()
                    .value = value;
            }
        }
        if arguments_binding
            && runtime.environments[child]
                .bindings
                .contains_key("arguments")
        {
            let value = runtime.binding_value(env, "arguments", doc)?;
            runtime.environments[child]
                .bindings
                .get_mut("arguments")
                .unwrap()
                .value = value;
        }
        child
    } else {
        env
    };
    let next = Job::Statement(statements::Frame::list(
        &code.unit,
        ListOwner::Function(code.id()),
        body_env,
    ));
    schedule(runtime, frame, Phase::Body, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::Parameter;
    use crate::script::parser::Parser;
    use crate::script::{
        Expr, FunctionCode, MAX_CALLS, MAX_HEAP, MAX_STACK_UNITS, MAX_STEPS, Stmt,
    };

    fn clean(runtime: &Runtime) {
        assert_eq!(
            (
                runtime.frames.len(),
                runtime.eval_depth,
                runtime.stack_units,
                runtime.calls
            ),
            (0, 0, 0, 0)
        );
    }
    fn run_unit(runtime: &mut Runtime, unit: &Rc<code::Unit>, doc: &mut Document) -> Result<Value> {
        match super::super::evaluate_statements(runtime, unit, ListOwner::Program, 1, doc)? {
            Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
            _ => panic!("fixture left abrupt control flow"),
        }
    }
    fn source(depth: usize, defaults: bool) -> String {
        if defaults {
            format!("function f(n,x=n?f(n-1):0){{return x+1;}}f({});", depth - 1)
        } else {
            format!(
                "function f(n){{if(n===0)return 0;return f(n-1)+1;}}f({});",
                depth - 1
            )
        }
    }

    #[test]
    fn ordinary_and_default_calls_reach_shared_logical_limit_in_both_modes() {
        for strict in [false, true] {
            for defaults in [false, true] {
                for depth in [1, 13, 14, 15, 16, 31, 32, 33, 40] {
                    let mut runtime = Runtime::new();
                    let mut doc = Document::parse("");
                    let source = source(depth, defaults);
                    let result = if strict {
                        runtime.execute_strict(&source, &mut doc)
                    } else {
                        runtime.execute(&source, &mut doc)
                    };
                    if depth <= MAX_CALLS {
                        assert_eq!(
                            result.unwrap(),
                            Value::Number((depth - usize::from(!defaults)) as f64)
                        );
                    } else {
                        let error = result.unwrap_err();
                        assert!(error.is_resource_limit());
                        assert_eq!(error.message, "script call stack limit exceeded");
                    }
                    clean(&runtime);
                }
            }
        }
    }

    fn recursive_unit(depth: usize, defaults: bool, strict: bool) -> Rc<code::Unit> {
        let number = |v| Expr::Literal(Value::Number(v));
        let binary =
            |left, op: &str, right| Expr::BinaryChain(Box::new(left), vec![(op.into(), right)]);
        let recurse = || {
            Expr::Call(
                Box::new(Expr::Ident("f".into())),
                vec![binary(Expr::Ident("n".into()), "-", number(1.0))],
            )
        };
        let mut params = vec![Parameter::simple("n".into())];
        let body = if defaults {
            params.push(Parameter {
                name: "x".into(),
                initializer: Some(Rc::new(Expr::Conditional(
                    Box::new(Expr::Ident("n".into())),
                    Box::new(recurse()),
                    Box::new(number(0.0)),
                ))),
                rest: false,
            });
            vec![Stmt::Return(Some(binary(
                Expr::Ident("x".into()),
                "+",
                number(1.0),
            )))]
        } else {
            vec![
                Stmt::If(
                    binary(Expr::Ident("n".into()), "===", number(0.0)),
                    Box::new(Stmt::Return(Some(number(0.0)))),
                    None,
                ),
                Stmt::Return(Some(binary(recurse(), "+", number(1.0)))),
            ]
        };
        code::test_unit(&[
            Stmt::Function(
                "f".into(),
                FunctionCode {
                    params,
                    body: Rc::new(body),
                    name: Some("f".into()),
                    arrow: false,
                    self_name: false,
                    constructable: true,
                    strict,
                },
            ),
            Stmt::Expr(Expr::Call(
                Box::new(Expr::Ident("f".into())),
                vec![number((depth - 1) as f64)],
            )),
        ])
    }
    #[test]
    fn thirty_two_activations_execute_on_small_native_stack_without_parsing_there() {
        for defaults in [false, true] {
            Runtime::parse_only(&source(32, defaults)).unwrap();
        }
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                for strict in [false, true] {
                    for defaults in [false, true] {
                        for depth in [32, 33] {
                            let unit = recursive_unit(depth, defaults, strict);
                            let weak = Rc::downgrade(&unit);
                            let mut runtime = Runtime::new();
                            let mut doc = Document::parse("");
                            let result = run_unit(&mut runtime, &unit, &mut doc);
                            if depth == 32 {
                                assert_eq!(
                                    result.unwrap(),
                                    Value::Number(if defaults { 32.0 } else { 31.0 })
                                );
                            } else {
                                assert_eq!(
                                    result.unwrap_err().message,
                                    "script call stack limit exceeded"
                                );
                            }
                            clean(&runtime);
                            drop(runtime);
                            drop(unit);
                            assert!(weak.upgrade().is_none());
                        }
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn all_activation_work_cuts_and_storage_refusals_restore_call_ownership() {
        let fixtures = [
            (
                "function f(a,b=a+1){return b;}",
                "f(2);",
                Value::Number(3.0),
            ),
            (
                "function f(a=1,...r){return a+r.length;}",
                "f(undefined,2,3);",
                Value::Number(3.0),
            ),
            (
                "function f(a){arguments[0]=8;return a;}",
                "f(2);",
                Value::Number(8.0),
            ),
            (
                "function f(a=1){arguments[0]=8;return a;}",
                "f(2);",
                Value::Number(2.0),
            ),
            (
                "function f(a=()=>a){return a()===a;}",
                "f();",
                Value::Bool(true),
            ),
            (
                "var marker={};function fail(){throw marker;}function f(a=fail(),b=(marker.hit=true)){return 2;}",
                "var ok=false;try{f();}catch(e){ok=e===marker;}ok&&marker.hit===undefined;",
                Value::Bool(true),
            ),
            (
                "function f(a,b){return this.k+a+b;}var g=f.bind({k:3},1);",
                "g(2);",
                Value::Number(6.0),
            ),
            (
                "function C(x=2){this.x=x;}",
                "new C(3).x;",
                Value::Number(3.0),
            ),
        ];
        for (setup, source, expected) in fixtures {
            let unit = Parser::program(source).unwrap();
            let mut doc = Document::parse("");
            let prepare = |doc: &mut Document| {
                let mut runtime = Runtime::new();
                runtime.execute(setup, doc).unwrap();
                runtime.steps = MAX_STEPS;
                runtime
            };
            let mut measured = prepare(&mut doc);
            let initial = measured.allocated;
            assert_eq!(run_unit(&mut measured, &unit, &mut doc).unwrap(), expected);
            let work = MAX_STEPS - measured.steps;
            let storage = measured.allocated - initial;
            assert!(work > 0 && storage > 0);
            clean(&measured);
            for allowance in 0..=work {
                let mut runtime = prepare(&mut doc);
                runtime.steps = allowance;
                let result = run_unit(&mut runtime, &unit, &mut doc);
                if allowance == work {
                    assert_eq!(result.unwrap(), expected, "{source}: {allowance}");
                } else {
                    assert!(
                        result.unwrap_err().is_resource_limit(),
                        "{source}: {allowance}"
                    );
                }
                clean(&runtime);
            }
            for allowance in [
                0,
                1,
                31,
                32,
                127,
                128,
                255,
                256,
                storage / 2,
                storage - 1,
                storage,
            ] {
                let mut runtime = prepare(&mut doc);
                runtime.allocated = MAX_HEAP - allowance;
                let result = run_unit(&mut runtime, &unit, &mut doc);
                if allowance >= storage {
                    assert_eq!(result.unwrap(), expected, "{source}: {allowance}");
                } else {
                    assert!(
                        result.unwrap_err().is_resource_limit(),
                        "{source}: {allowance}"
                    );
                }
                clean(&runtime);
            }
            let weak = Rc::downgrade(&unit);
            drop(unit);
            assert!(
                weak.upgrade().is_none(),
                "completed frames retained invocation code"
            );
        }
    }

    #[test]
    fn bound_wrappers_and_preentered_bridges_share_exact_call_counts() {
        for native in [false, true] {
            for wrappers in [0, 30, 31, 32] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                let target = if native { "Math.max" } else { "f" };
                runtime.execute(&format!("function f(x){{return x;}}var g={target};for(var i=0;i<{wrappers};i++)g=g.bind(null);"), &mut doc).unwrap();
                let function = runtime.lookup(0, "g").unwrap().1;
                for bridge in [false, true] {
                    runtime.steps = MAX_STEPS;
                    let result = if bridge {
                        runtime.call(
                            function.clone(),
                            vec![Value::Number(7.0)],
                            Value::Undefined,
                            &mut doc,
                        )
                    } else {
                        runtime.execute("g(7);", &mut doc)
                    };
                    if wrappers < MAX_CALLS {
                        assert_eq!(result.unwrap(), Value::Number(7.0));
                    } else {
                        assert_eq!(
                            result.unwrap_err().message,
                            "script call stack limit exceeded"
                        );
                    }
                    clean(&runtime);
                }
            }
        }
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        runtime
            .execute("function f(){return 7;}", &mut doc)
            .unwrap();
        let f = runtime.lookup(0, "f").unwrap().1;
        runtime.stack_units = MAX_STACK_UNITS;
        assert_eq!(
            runtime.execute("f();", &mut doc).unwrap(),
            Value::Number(7.0)
        );
        assert_eq!(runtime.stack_units, MAX_STACK_UNITS);
        assert!(
            runtime
                .call(f, vec![], Value::Undefined, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            runtime
                .execute("Math.max(1,2);", &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (
                runtime.calls,
                runtime.eval_depth,
                runtime.frames.len(),
                runtime.stack_units
            ),
            (0, 0, 0, MAX_STACK_UNITS)
        );
        runtime.stack_units = MAX_STACK_UNITS - 4;
        assert_eq!(
            runtime
                .call(
                    Runtime::native("Math.max", Value::Math),
                    vec![Value::Number(7.0)],
                    Value::Math,
                    &mut doc
                )
                .unwrap(),
            Value::Number(7.0)
        );
        assert_eq!(runtime.stack_units, MAX_STACK_UNITS - 4);
        runtime.stack_units = 0;
        clean(&runtime);
    }

    #[test]
    fn native_getters_constructors_json_and_coercions_retain_uncatchable_guards() {
        let sources = [
            "var o={get x(){return o.x;}};o.x;",
            "var o={valueOf(){return +o;}};+o;",
            "function C(){new C();}new C();",
            "function f(){return f.apply(null,[]);}f();",
            "function f(k,v){return JSON.stringify(v,f);}JSON.stringify(1,f);",
        ];
        for body in sources {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            let source = format!(
                "var caught=false,finished=false;try{{{body}}}catch(e){{caught=true;}}finally{{finished=true;}}"
            );
            let error = runtime.execute(&source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit());
            assert_eq!(
                error.message, "combined script nesting limit exceeded",
                "{body}"
            );
            assert_eq!(runtime.lookup(0, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.lookup(0, "finished").unwrap().1, Value::Bool(false));
            clean(&runtime);
            assert_eq!(runtime.json_depth, 0);
        }
    }

    #[test]
    fn defaults_tdz_closures_and_cross_unit_finally_preserve_identity() {
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var marker={},log='';function foreign(x){log+='f';return x;}function fail(){throw marker;}", &mut doc).unwrap();
            let source = r#"
                function f(a=()=>x,x=foreign(3)){var x=8;return [a,x];}
                var r=f(),tdz=false;try{(function(a=b,b=1){} )();}catch(e){tdz=e instanceof ReferenceError;}
                function g(a=fail(),b=(log+='bad')){log+='body';}
                var same=false;try{g();}catch(e){same=e===marker;}finally{log+='z';}
                function h(n){try{if(n===0)throw marker;return h(n-1);}finally{log+='h';}}
                var deep=false;try{h(31);}catch(e){deep=e===marker;}
                r[0]()===3&&r[1]===8&&tdz&&same&&deep;
            "#;
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert_eq!(result.unwrap(), Value::Bool(true));
            assert_eq!(
                runtime.lookup(0, "log").unwrap().1.to_string(),
                format!("fz{}", "h".repeat(32))
            );
            clean(&runtime);
        }
    }

    #[test]
    fn frame_storage_refusal_preserves_outer_frames_and_unentered_call_count() {
        let unit = Parser::program("held;").unwrap();
        let code::Stmt::Expr(id) = unit.stmt(unit.body[0]) else {
            unreachable!()
        };
        let id = *id;
        let weak = Rc::downgrade(&unit);
        let mut runtime = Runtime::new();
        runtime.calls = 7; // Simulate a reentrant caller whose count we do not own.
        runtime.steps = MAX_STEPS;
        let before = runtime.allocated;
        let mut last_accepted = before;
        loop {
            let pending = Job::Expression(ExprFrame {
                unit: unit.clone(),
                expression: id,
                env: 0,
                reference: true,
                phase: ExprPhase::Start,
            });
            match push(&mut runtime, pending) {
                Ok(()) => last_accepted = runtime.allocated,
                Err(error) => {
                    assert!(error.is_resource_limit());
                    break;
                }
            }
        }
        assert_eq!(runtime.frames.len(), runtime.frames.capacity());
        assert!(runtime.frames.capacity() > 193);
        assert!(runtime.frames.capacity() < super::super::MAX_FRAMES);
        // The cumulative ledger records refused charges too. Retained capacity
        // and every successful charge must stay below the allowance.
        assert!(last_accepted > before && last_accepted <= MAX_HEAP);
        assert!(runtime.allocated > MAX_HEAP);
        assert!(runtime.frames.capacity() * std::mem::size_of::<Job>() < MAX_HEAP);
        let retained = runtime.frames.len();
        let error = enter_frame(
            &mut runtime,
            Job::Call(Frame::new(
                Value::Undefined,
                vec![],
                Value::Undefined,
                false,
            )),
        )
        .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.calls, 7);
        assert_eq!(runtime.frames.len(), retained);
        assert_eq!((runtime.eval_depth, runtime.stack_units), (0, 0));
        runtime.frames.clear();
        runtime.calls = 0;
        drop(unit);
        assert!(weak.upgrade().is_none());
        clean(&runtime);
    }
}
