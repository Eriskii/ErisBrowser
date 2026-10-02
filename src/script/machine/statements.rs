//! Statement/list continuations sharing the expression driver and its unwind boundary.
use super::{Document, ExprFrame, Frame as Job, Output, Phase as ExprPhase, Result, Runtime, code};
use super::{enter_frame, push, reference, value};
use crate::script::{
    BINDING_BYTES, DeclarationKind, Flow, JsString, MAX_DEPTH, ScriptError, Value, js_object,
    own_keys::VisitedNames,
};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(in crate::script) enum ListOwner {
    Program,
    Function(code::FunctionId),
    Block(code::StmtId),
    Catch(code::StmtId),
}
impl ListOwner {
    fn body(self, unit: &code::Unit) -> &[code::StmtId] {
        match self {
            Self::Program => &unit.body,
            Self::Function(id) => &unit.function(id).body,
            Self::Block(id) => match unit.stmt(id) {
                code::Stmt::Block(body) => body,
                _ => unreachable!("block list"),
            },
            Self::Catch(id) => match unit.stmt(id) {
                code::Stmt::Try(_, Some(handler), _) => &handler.body,
                _ => unreachable!("catch list"),
            },
        }
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Statement(code::StmtId, Option<usize>),
    List(ListOwner),
}
pub(in crate::script) struct Frame {
    unit: Rc<code::Unit>,
    env: usize,
    kind: Kind,
    phase: Phase,
}
enum Phase {
    Start,
    ListNext {
        index: usize,
        last: Option<Value>,
    },
    Identity,
    Expression,
    Return,
    Throw,
    Label,
    If,
    UpdateEmpty,
    Declare(usize),
    WhileTest(Value),
    DoTest(Value),
    LoopBody(Value),
    ForInit(usize),
    ForTest {
        child: usize,
        last: Value,
    },
    ForBody {
        child: usize,
        last: Value,
    },
    ForUpdate {
        child: usize,
        last: Value,
    },
    SwitchValue,
    SwitchTest {
        child: usize,
        value: Value,
        index: usize,
        default: Option<usize>,
    },
    SwitchBody {
        child: usize,
        case: usize,
        index: usize,
        last: Value,
    },
    TryBody,
    TryCatch,
    TryFinally(Result<Flow>),
    ForInValue,
    ForInTarget {
        state: ForInState,
        key: Value,
    },
    ForInBody(ForInState),
    ForOfValue,
    ForOfIterator,
    ForOfNext(ForOfState),
    ForOfTarget {
        state: ForOfState,
        value: Value,
    },
    ForOfBody(ForOfState),
    ForOfClose(Result<Flow>),
}
struct ForInState {
    object: Value,
    keys: std::vec::IntoIter<JsString>,
    visited: VisitedNames,
    depth: usize,
    last: Value,
}
struct ForOfState {
    iterator: Value,
    next: Value,
    last: Value,
}
impl Frame {
    pub(super) fn list(unit: &Rc<code::Unit>, owner: ListOwner, env: usize) -> Self {
        Self {
            unit: unit.clone(),
            env,
            kind: Kind::List(owner),
            phase: Phase::Start,
        }
    }
    fn statement(
        unit: &Rc<code::Unit>,
        id: code::StmtId,
        env: usize,
        label: Option<usize>,
    ) -> Self {
        Self {
            unit: unit.clone(),
            env,
            kind: Kind::Statement(id, label),
            phase: Phase::Start,
        }
    }
    fn id(&self) -> code::StmtId {
        match self.kind {
            Kind::Statement(id, _) => id,
            _ => unreachable!("statement ID"),
        }
    }
    fn label(&self) -> Option<usize> {
        match self.kind {
            Kind::Statement(_, label) => label,
            _ => None,
        }
    }
}
fn flow(output: Option<Output>) -> Flow {
    match output {
        Some(Output::Flow(flow)) => flow,
        _ => unreachable!("statement continuation"),
    }
}
fn done(flow: Flow) -> Result<Option<Output>> {
    Ok(Some(Output::Flow(flow)))
}
fn normal(value: Value) -> Result<Option<Output>> {
    done(Flow::Normal(Some(value)))
}
fn schedule(
    runtime: &mut Runtime,
    mut frame: Frame,
    phase: Phase,
    next: Job,
) -> Result<Option<Output>> {
    frame.phase = phase;
    push(runtime, Job::Statement(frame))?;
    enter_frame(runtime, next)?;
    Ok(None)
}
fn expression(
    runtime: &mut Runtime,
    frame: Frame,
    phase: Phase,
    id: code::ExprId,
    env: usize,
) -> Result<Option<Output>> {
    let next = Job::Expression(ExprFrame {
        unit: frame.unit.clone(),
        expression: id,
        env,
        reference: false,
        phase: ExprPhase::Start,
    });
    schedule(runtime, frame, phase, next)
}
fn statement(
    runtime: &mut Runtime,
    frame: Frame,
    phase: Phase,
    id: code::StmtId,
    env: usize,
    label: Option<usize>,
) -> Result<Option<Output>> {
    let next = Job::Statement(Frame::statement(&frame.unit, id, env, label));
    schedule(runtime, frame, phase, next)
}
fn list(
    runtime: &mut Runtime,
    frame: Frame,
    phase: Phase,
    owner: ListOwner,
    env: usize,
) -> Result<Option<Output>> {
    let next = Job::Statement(Frame::list(&frame.unit, owner, env));
    schedule(runtime, frame, phase, next)
}

pub(super) fn step(
    runtime: &mut Runtime,
    mut frame: Frame,
    output: Option<Result<Output>>,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let phase = std::mem::replace(&mut frame.phase, Phase::Start);
    // Try and iterator-close boundaries intercept ordinary exceptions. Host
    // stops never reach them: the driver restores its frame base immediately.
    let output = match phase {
        Phase::TryBody => {
            return try_body(
                runtime,
                frame,
                output.expect("try body result").map(|v| flow(Some(v))),
                doc,
            );
        }
        Phase::TryCatch => {
            return finish_try(
                runtime,
                frame,
                output.expect("catch result").map(|v| flow(Some(v))),
            );
        }
        Phase::ForOfTarget { state, value } => {
            let result = output
                .expect("for-of target result")
                .and_then(|output| runtime.write_reference(reference(Some(output)), value, doc));
            return match result {
                Ok(()) => {
                    let env = frame.env;
                    for_of_body(runtime, frame, state, env)
                }
                Err(error) => for_of_close(runtime, frame, state, Err(error), doc),
            };
        }
        Phase::ForOfBody(mut state) => {
            let completion = match output.expect("for-of body result") {
                Ok(output) => match flow(Some(output)).loop_step(frame.label(), &mut state.last) {
                    Ok(()) => return for_of_next(runtime, frame, state),
                    Err(abrupt) => Ok(abrupt),
                },
                Err(error) => Err(error),
            };
            return for_of_close(runtime, frame, state, completion, doc);
        }
        Phase::ForOfClose(pending) => {
            return for_of_closed(
                pending,
                output
                    .expect("iterator return result")
                    .map(|out| value(Some(out))),
            );
        }
        _ => output.transpose()?,
    };
    let unit = frame.unit.clone();
    let env = frame.env;
    match phase {
        Phase::Start => start(runtime, frame, doc),
        Phase::ListNext { index, last } => match flow(output).update_empty(last.as_ref()) {
            Flow::Normal(last) => list_next(runtime, frame, index + 1, last),
            abrupt => done(abrupt),
        },
        Phase::Identity => Ok(output),
        Phase::Expression => normal(value(output)),
        Phase::Return => done(Flow::Return(value(output))),
        Phase::Throw => {
            let value = value(output);
            let name = runtime.thrown_name(&value, doc)?;
            let intrinsic = runtime.thrown_intrinsic_name(&value, doc)?;
            let mut error = runtime.thrown_error(value)?;
            error.thrown_name = name;
            error.intrinsic_name = intrinsic;
            Err(error)
        }
        Phase::Label => {
            let code::Stmt::Label(target, _) = unit.stmt(frame.id()) else {
                unreachable!()
            };
            done(match flow(output) {
                Flow::Break(Some(found), value) if found == *target => Flow::Normal(value),
                flow => flow,
            })
        }
        Phase::If => {
            let code::Stmt::If(_, yes, no) = unit.stmt(frame.id()) else {
                unreachable!()
            };
            if let Some(next) = if value(output).truthy() {
                Some(*yes)
            } else {
                *no
            } {
                statement(runtime, frame, Phase::UpdateEmpty, next, env, None)
            } else {
                normal(Value::Undefined)
            }
        }
        Phase::UpdateEmpty => done(flow(output).update_empty(Some(&Value::Undefined))),
        Phase::Declare(index) => {
            declaration_value(runtime, &frame, index, value(output), doc)?;
            declaration_next(runtime, frame, index + 1, doc)
        }
        Phase::WhileTest(last) | Phase::DoTest(last) => {
            if !value(output).truthy() {
                return normal(last);
            }
            let (code::Stmt::While(_, body) | code::Stmt::DoWhile(_, body)) = unit.stmt(frame.id())
            else {
                unreachable!()
            };
            runtime.tick()?;
            statement(runtime, frame, Phase::LoopBody(last), *body, env, None)
        }
        Phase::LoopBody(mut last) => {
            if let Err(flow) = flow(output).loop_step(frame.label(), &mut last) {
                return done(flow.consume_break());
            }
            let (condition, phase) = match unit.stmt(frame.id()) {
                code::Stmt::While(condition, _) => (*condition, Phase::WhileTest(last)),
                code::Stmt::DoWhile(condition, _) => (*condition, Phase::DoTest(last)),
                _ => unreachable!(),
            };
            expression(runtime, frame, phase, condition, env)
        }
        Phase::ForInit(child) => {
            let _ = flow(output);
            let child = iteration_environment(runtime, &frame, child)?;
            for_test(runtime, frame, child, Value::Undefined)
        }
        Phase::ForTest { child, last } => {
            if !value(output).truthy() {
                return normal(last);
            }
            for_body(runtime, frame, child, last)
        }
        Phase::ForBody { child, mut last } => {
            if let Err(flow) = flow(output).loop_step(frame.label(), &mut last) {
                return done(flow.consume_break());
            }
            let child = iteration_environment(runtime, &frame, child)?;
            let code::Stmt::For(_, _, update, _) = unit.stmt(frame.id()) else {
                unreachable!()
            };
            if let Some(update) = update {
                expression(
                    runtime,
                    frame,
                    Phase::ForUpdate { child, last },
                    *update,
                    child,
                )
            } else {
                for_test(runtime, frame, child, last)
            }
        }
        Phase::ForUpdate { child, last } => {
            let _ = value(output);
            for_test(runtime, frame, child, last)
        }
        Phase::SwitchValue => switch_start(runtime, frame, value(output)),
        Phase::SwitchTest {
            child,
            value: target,
            index,
            default,
        } => {
            let matched = runtime.binary_value("===", target.clone(), value(output), doc)?
                == Value::Bool(true);
            if matched {
                switch_body(runtime, frame, child, index, 0, Value::Undefined)
            } else {
                switch_test(runtime, frame, child, target, index + 1, default)
            }
        }
        Phase::SwitchBody {
            child,
            case,
            index,
            mut last,
        } => {
            match flow(output) {
                Flow::Normal(Some(value)) => last = value,
                Flow::Normal(None) => {}
                abrupt => return done(abrupt.update_empty(Some(&last)).consume_break()),
            }
            switch_body(runtime, frame, child, case, index + 1, last)
        }
        Phase::TryFinally(pending) => match flow(output) {
            Flow::Normal(_) => {
                pending.and_then(|flow| done(flow.update_empty(Some(&Value::Undefined))))
            }
            abrupt => done(abrupt.update_empty(Some(&Value::Undefined))),
        },
        Phase::ForOfValue => for_of_start(runtime, frame, value(output), doc),
        Phase::ForOfIterator => {
            let iterator = value(output);
            if !js_object(&iterator) {
                return Err(ScriptError::type_error(
                    "iterator method must return an object",
                ));
            }
            let next = for_of_get(runtime, &iterator, "next", doc)?;
            for_of_next(
                runtime,
                frame,
                ForOfState {
                    iterator,
                    next,
                    last: Value::Undefined,
                },
            )
        }
        Phase::ForOfNext(state) => {
            let result = value(output);
            if !js_object(&result) {
                return Err(ScriptError::type_error(
                    "iterator next must return an object",
                ));
            }
            if for_of_get(runtime, &result, "done", doc)?.truthy() {
                return normal(state.last);
            }
            let next_value = for_of_get(runtime, &result, "value", doc)?;
            for_of_assign(runtime, frame, state, next_value, doc)
        }
        Phase::ForInValue => for_in_start(runtime, frame, value(output), doc),
        Phase::ForInTarget { state, key } => {
            runtime.write_reference(reference(output), key, doc)?;
            for_in_body(runtime, frame, state, env)
        }
        Phase::ForInBody(mut state) => {
            if let Err(flow) = flow(output).loop_step(frame.label(), &mut state.last) {
                return done(flow.consume_break());
            }
            for_in_next(runtime, frame, state, doc)
        }
        Phase::TryBody
        | Phase::TryCatch
        | Phase::ForOfTarget { .. }
        | Phase::ForOfBody(_)
        | Phase::ForOfClose(_) => unreachable!(),
    }
}
fn start(runtime: &mut Runtime, frame: Frame, doc: &mut Document) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let env = frame.env;
    let id = match frame.kind {
        Kind::List(owner) => {
            runtime.instantiate_statements(&unit, owner.body(&unit), env)?;
            return list_next(runtime, frame, 0, None);
        }
        Kind::Statement(id, _) => id,
    };
    runtime.tick()?;
    match unit.stmt(id) {
        code::Stmt::Empty | code::Stmt::Function(..) => done(Flow::Normal(None)),
        code::Stmt::Label(target, body) => {
            statement(runtime, frame, Phase::Label, *body, env, Some(*target))
        }
        code::Stmt::Expr(expr) => expression(runtime, frame, Phase::Expression, *expr, env),
        code::Stmt::Var(..) => declaration_next(runtime, frame, 0, doc),
        code::Stmt::Block(_) => {
            let child = runtime.environment(env)?;
            list(runtime, frame, Phase::Identity, ListOwner::Block(id), child)
        }
        code::Stmt::If(condition, _, _) => expression(runtime, frame, Phase::If, *condition, env),
        code::Stmt::While(condition, _) => expression(
            runtime,
            frame,
            Phase::WhileTest(Value::Undefined),
            *condition,
            env,
        ),
        code::Stmt::DoWhile(_, body) => {
            runtime.tick()?;
            statement(
                runtime,
                frame,
                Phase::LoopBody(Value::Undefined),
                *body,
                env,
                None,
            )
        }
        code::Stmt::For(init, _, _, _) => {
            let child = runtime.environment(env)?;
            if let Some(init) = init {
                runtime.instantiate_lexical(&unit, std::slice::from_ref(init), child)?;
                statement(runtime, frame, Phase::ForInit(child), *init, child, None)
            } else {
                for_test(runtime, frame, child, Value::Undefined)
            }
        }
        code::Stmt::ForIn(binding, expr, _) | code::Stmt::ForOf(binding, expr, _) => {
            let expression_env = if let code::ForBinding::Declaration(name, kind) = binding
                && *kind != DeclarationKind::Var
            {
                let child = runtime.environment(env)?;
                runtime.define(
                    child,
                    name,
                    Value::Undefined,
                    *kind != DeclarationKind::Const,
                )?;
                runtime.environments[child]
                    .bindings
                    .get_mut(name)
                    .unwrap()
                    .initialized = false;
                child
            } else {
                env
            };
            let phase = if matches!(unit.stmt(id), code::Stmt::ForOf(..)) {
                Phase::ForOfValue
            } else {
                Phase::ForInValue
            };
            expression(runtime, frame, phase, *expr, expression_env)
        }
        code::Stmt::Switch(expr, _) => expression(runtime, frame, Phase::SwitchValue, *expr, env),
        code::Stmt::Return(Some(expr)) => expression(runtime, frame, Phase::Return, *expr, env),
        code::Stmt::Return(None) => done(Flow::Return(Value::Undefined)),
        code::Stmt::Throw(expr) => expression(runtime, frame, Phase::Throw, *expr, env),
        code::Stmt::Try(body, _, _) => statement(runtime, frame, Phase::TryBody, *body, env, None),
        code::Stmt::Break(target) => done(Flow::Break(*target, None)),
        code::Stmt::Continue(target) => done(Flow::Continue(*target, None)),
    }
}
fn list_next(
    runtime: &mut Runtime,
    frame: Frame,
    index: usize,
    last: Option<Value>,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let Kind::List(owner) = frame.kind else {
        unreachable!()
    };
    let env = frame.env;
    match owner.body(&unit).get(index) {
        Some(next) => statement(
            runtime,
            frame,
            Phase::ListNext { index, last },
            *next,
            env,
            None,
        ),
        None => done(Flow::Normal(last)),
    }
}
fn declaration_value(
    runtime: &mut Runtime,
    frame: &Frame,
    index: usize,
    value: Value,
    doc: &mut Document,
) -> Result<()> {
    let code::Stmt::Var(bindings, kind) = frame.unit.stmt(frame.id()) else {
        unreachable!()
    };
    let name = &bindings[index].0;
    let env = frame.env;
    let owner = if *kind == DeclarationKind::Var {
        runtime.var_scope(env)
    } else {
        env
    };
    if *kind == DeclarationKind::Var {
        let target = runtime.lookup(env, name).map_or(owner, |(owner, _)| owner);
        runtime.write_name(
            Some(target),
            name,
            runtime.environments[env].strict,
            value,
            doc,
        )?;
    } else if let Some(binding) = runtime.environments[owner].bindings.get_mut(name)
        && !binding.initialized
    {
        binding.value = value;
        binding.initialized = true;
    } else {
        runtime.define(owner, name, value, *kind != DeclarationKind::Const)?;
        runtime.environments[owner]
            .bindings
            .get_mut(name)
            .unwrap()
            .global_property = false;
    }
    Ok(())
}
fn declaration_next(
    runtime: &mut Runtime,
    frame: Frame,
    mut index: usize,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::Var(bindings, kind) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    while let Some((_, initializer)) = bindings.get(index) {
        if let Some(initializer) = initializer {
            let env = frame.env;
            return expression(runtime, frame, Phase::Declare(index), *initializer, env);
        }
        if *kind != DeclarationKind::Var {
            declaration_value(runtime, &frame, index, Value::Undefined, doc)?;
        }
        index += 1;
    }
    done(Flow::Normal(None))
}
fn iteration_environment(runtime: &mut Runtime, frame: &Frame, previous: usize) -> Result<usize> {
    let code::Stmt::For(Some(init), _, _, _) = frame.unit.stmt(frame.id()) else {
        return Ok(previous);
    };
    let code::Stmt::Var(bindings, DeclarationKind::Let) = frame.unit.stmt(*init) else {
        return Ok(previous);
    };
    if bindings.is_empty() {
        return Ok(previous);
    }
    let next = runtime.environment(frame.env)?;
    for (name, _) in bindings {
        runtime.charge(BINDING_BYTES + name.len())?;
        let binding = runtime.environments[previous].bindings[name].clone();
        runtime.environments[next]
            .bindings
            .insert(name.clone(), binding);
    }
    Ok(next)
}
fn for_test(
    runtime: &mut Runtime,
    frame: Frame,
    child: usize,
    last: Value,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::For(_, condition, _, _) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    runtime.tick()?;
    if let Some(condition) = condition {
        expression(
            runtime,
            frame,
            Phase::ForTest { child, last },
            *condition,
            child,
        )
    } else {
        for_body(runtime, frame, child, last)
    }
}
fn for_body(
    runtime: &mut Runtime,
    frame: Frame,
    child: usize,
    last: Value,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::For(_, _, _, body) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    statement(
        runtime,
        frame,
        Phase::ForBody { child, last },
        *body,
        child,
        None,
    )
}
fn switch_start(runtime: &mut Runtime, frame: Frame, value: Value) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::Switch(_, cases) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    let child = runtime.environment(frame.env)?;
    for (_, body) in cases {
        runtime.instantiate_lexical(&unit, body, child)?;
        for statement in body {
            if let code::Stmt::Function(name, code) = unit.stmt(*statement) {
                let function =
                    runtime.function_value(&code::FunctionRef::new(&unit, *code), child)?;
                runtime.define(child, name, function, true)?;
            }
        }
    }
    switch_test(runtime, frame, child, value, 0, None)
}
fn switch_test(
    runtime: &mut Runtime,
    frame: Frame,
    child: usize,
    value: Value,
    mut index: usize,
    mut default: Option<usize>,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::Switch(_, cases) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    while let Some((condition, _)) = cases.get(index) {
        runtime.tick()?;
        if let Some(condition) = condition {
            return expression(
                runtime,
                frame,
                Phase::SwitchTest {
                    child,
                    value,
                    index,
                    default,
                },
                *condition,
                child,
            );
        }
        default = Some(index);
        index += 1;
    }
    if let Some(default) = default {
        switch_body(runtime, frame, child, default, 0, Value::Undefined)
    } else {
        normal(Value::Undefined)
    }
}
fn switch_body(
    runtime: &mut Runtime,
    frame: Frame,
    child: usize,
    mut case: usize,
    mut index: usize,
    last: Value,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::Switch(_, cases) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    while let Some((_, body)) = cases.get(case) {
        if let Some(next) = body.get(index) {
            return statement(
                runtime,
                frame,
                Phase::SwitchBody {
                    child,
                    case,
                    index,
                    last,
                },
                *next,
                child,
                None,
            );
        }
        case += 1;
        index = 0;
    }
    normal(last)
}
fn try_body(
    runtime: &mut Runtime,
    frame: Frame,
    completion: Result<Flow>,
    _doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let id = frame.id();
    let code::Stmt::Try(_, handler, _) = unit.stmt(id) else {
        unreachable!()
    };
    let completion = match completion {
        Err(error) => {
            if let Some(handler) = handler {
                let child = runtime.environment(frame.env)?;
                if let Some(binding) = &handler.binding {
                    let value = runtime.exception_value(error)?;
                    runtime.define(child, binding, value, true)?;
                }
                return list(runtime, frame, Phase::TryCatch, ListOwner::Catch(id), child);
            }
            Err(error)
        }
        completion => completion,
    };
    finish_try(runtime, frame, completion)
}
fn finish_try(
    runtime: &mut Runtime,
    frame: Frame,
    completion: Result<Flow>,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::Try(_, _, finalizer) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    let env = frame.env;
    if let Some(finalizer) = finalizer {
        statement(
            runtime,
            frame,
            Phase::TryFinally(completion),
            *finalizer,
            env,
            None,
        )
    } else {
        completion.and_then(|flow| done(flow.update_empty(Some(&Value::Undefined))))
    }
}
fn for_in_start(
    runtime: &mut Runtime,
    frame: Frame,
    value: Value,
    doc: &Document,
) -> Result<Option<Output>> {
    if matches!(value, Value::Null | Value::Undefined) {
        return normal(Value::Undefined);
    }
    let object = runtime.coerce_object(value)?;
    let keys = runtime.own_keys(&object)?.into_iter();
    let state = ForInState {
        object,
        keys,
        visited: VisitedNames::default(),
        depth: 1,
        last: Value::Undefined,
    };
    for_in_next(runtime, frame, state, doc)
}
fn for_in_next(
    runtime: &mut Runtime,
    frame: Frame,
    mut state: ForInState,
    doc: &Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::ForIn(binding, _, _) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    let env = frame.env;
    loop {
        for key in state.keys.by_ref() {
            let Some(property) = runtime.for_in_visit(&mut state.visited, &state.object, &key)?
            else {
                continue;
            };
            if !property.enumerable {
                continue;
            }
            let value = runtime.string(key)?;
            let scope = match binding {
                code::ForBinding::Declaration(name, DeclarationKind::Var) => {
                    let owner = runtime.var_scope(env);
                    runtime.define(owner, name, value, true)?;
                    env
                }
                code::ForBinding::Declaration(name, kind) => {
                    let child = runtime.environment(env)?;
                    runtime.define(child, name, value, *kind != DeclarationKind::Const)?;
                    child
                }
                code::ForBinding::Target(target) => {
                    let next = Job::Expression(ExprFrame {
                        unit: unit.clone(),
                        expression: *target,
                        env,
                        reference: true,
                        phase: ExprPhase::Start,
                    });
                    return schedule(
                        runtime,
                        frame,
                        Phase::ForInTarget { state, key: value },
                        next,
                    );
                }
            };
            return for_in_body(runtime, frame, state, scope);
        }
        let Some(next) = runtime.prototype_of_in(&state.object, doc)? else {
            return normal(state.last);
        };
        if state.depth >= MAX_DEPTH {
            return Err(ScriptError::resource("for-in prototype depth exceeded"));
        }
        state.depth += 1;
        state.keys = runtime.own_keys(&next)?.into_iter();
        state.object = next;
    }
}
fn for_in_body(
    runtime: &mut Runtime,
    frame: Frame,
    state: ForInState,
    scope: usize,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::ForIn(_, _, body) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    statement(runtime, frame, Phase::ForInBody(state), *body, scope, None)
}

// These protocol calls suspend in the shared driver. Only property getter
// bridges use the runtime's existing bounded callback bridge.
fn for_of_get(
    runtime: &mut Runtime,
    object: &Value,
    name: &str,
    doc: &mut Document,
) -> Result<Value> {
    // These fixed protocol keys are at most six ASCII units. The charged
    // ordinary walker preserves the original accessor receiver and declines
    // host-only prototype edges when reached, without speculative traversal.
    runtime.work(1 + name.len())?;
    runtime.charge(64)?;
    let key = JsString::from(name);
    runtime.reduce_get(object, &key, doc)
}
fn for_of_call(
    runtime: &mut Runtime,
    frame: Frame,
    phase: Phase,
    method: Value,
    receiver: Value,
) -> Result<Option<Output>> {
    schedule(
        runtime,
        frame,
        phase,
        Job::Call(super::calls::Frame::new(
            method,
            Vec::new(),
            receiver,
            false,
        )),
    )
}
fn for_of_start(
    runtime: &mut Runtime,
    frame: Frame,
    source: Value,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let method =
        runtime.get_property_key(source.clone(), &runtime.well_known_key("iterator"), doc)?;
    if !matches!(method, Value::Function(_) | Value::Native(_)) {
        return Err(ScriptError::type_error("value is not iterable"));
    }
    for_of_call(runtime, frame, Phase::ForOfIterator, method, source)
}
fn for_of_next(runtime: &mut Runtime, frame: Frame, state: ForOfState) -> Result<Option<Output>> {
    runtime.tick()?;
    let method = state.next.clone();
    let receiver = state.iterator.clone();
    for_of_call(runtime, frame, Phase::ForOfNext(state), method, receiver)
}
fn for_of_assign(
    runtime: &mut Runtime,
    frame: Frame,
    state: ForOfState,
    value: Value,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::ForOf(binding, _, _) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    let env = frame.env;
    let scope = match binding {
        code::ForBinding::Declaration(name, DeclarationKind::Var) => {
            // Resolve in the current lexical environment, including a catch
            // parameter; hoisting does not dictate the assignment reference.
            let result = runtime
                .resolve_binding_in(env, name, doc)
                .and_then(|owner| {
                    runtime.write_name(owner, name, runtime.environments[env].strict, value, doc)
                });
            if let Err(error) = result {
                return for_of_close(runtime, frame, state, Err(error), doc);
            }
            env
        }
        code::ForBinding::Declaration(name, kind) => {
            let child = runtime.environment(env)?;
            runtime.define(child, name, value, *kind != DeclarationKind::Const)?;
            child
        }
        code::ForBinding::Target(target) => {
            let next = Job::Expression(ExprFrame {
                unit: unit.clone(),
                expression: *target,
                env,
                reference: true,
                phase: ExprPhase::Start,
            });
            return schedule(runtime, frame, Phase::ForOfTarget { state, value }, next);
        }
    };
    for_of_body(runtime, frame, state, scope)
}
fn for_of_body(
    runtime: &mut Runtime,
    frame: Frame,
    state: ForOfState,
    scope: usize,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Stmt::ForOf(_, _, body) = unit.stmt(frame.id()) else {
        unreachable!()
    };
    statement(runtime, frame, Phase::ForOfBody(state), *body, scope, None)
}
fn for_of_close(
    runtime: &mut Runtime,
    frame: Frame,
    state: ForOfState,
    pending: Result<Flow>,
    doc: &mut Document,
) -> Result<Option<Output>> {
    if let Err(error) = &pending
        && (error.is_resource_limit() || error.is_unsupported())
    {
        return pending.and_then(done);
    }
    let method = match for_of_get(runtime, &state.iterator, "return", doc) {
        Ok(Value::Undefined | Value::Null) => {
            return pending.and_then(|flow| done(flow.consume_break()));
        }
        Ok(method) => method,
        Err(error) => return for_of_closed(pending, Err(error)),
    };
    if !matches!(method, Value::Function(_) | Value::Native(_)) {
        return for_of_closed(
            pending,
            Err(ScriptError::type_error("iterator return is not callable")),
        );
    }
    for_of_call(
        runtime,
        frame,
        Phase::ForOfClose(pending),
        method,
        state.iterator,
    )
}
fn for_of_closed(pending: Result<Flow>, close: Result<Value>) -> Result<Option<Output>> {
    // The host's terminal limits remain uncatchable even while an ordinary
    // author exception is pending. Ordinary close errors lose to that throw.
    if let Err(error) = &close
        && (error.is_resource_limit() || error.is_unsupported())
    {
        return close.map(|_| None);
    }
    let flow = pending?;
    if !js_object(&close?) {
        return Err(ScriptError::type_error(
            "iterator return must return an object",
        ));
    }
    done(flow.consume_break())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::parser::Parser;
    use crate::script::{Expr, MAX_HEAP, MAX_STACK_UNITS, MAX_STEPS, Stmt};

    fn compile(source: &str) -> Rc<code::Unit> {
        Parser::program(source).unwrap()
    }
    fn execute(runtime: &mut Runtime, unit: &Rc<code::Unit>, doc: &mut Document) -> Result<Value> {
        match super::super::evaluate_statements(runtime, unit, ListOwner::Program, 1, doc)? {
            Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
            _ => panic!("fixture left abrupt control flow"),
        }
    }
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

    #[test]
    fn every_statement_work_cut_restores_frames_and_preserves_host_termination() {
        let sources = [
            "var x=0;{let y=2;if(y)x=y;}x;",
            "var i=0,x=0;while(i<3){i++;if(i===2)continue;x+=i;}x;",
            "var i=0;do{i++;if(i===2)break;}while(i<3);i;",
            "var x=0;for(let i=0;i<3;i++){x+=i;}x;",
            "var x='';for(var k in {a:1,b:2}){x+=k;}x;",
            "var x=0;switch(2){default:x=1;case 1:x=2;break;case 2:x=3;case 3:x+=4;}x;",
            "var x=0;try{throw 7;}catch(e){x=e;}finally{x++;}x;",
            "var x=0;outer:for(var i=0;i<3;i++){try{if(i===1)continue outer;x++;}finally{x+=2;}}x;",
        ];
        for source in sources {
            let unit = compile(source);
            let mut doc = Document::parse("");
            let mut measured = Runtime::new();
            measured.steps = MAX_STEPS;
            let expected = execute(&mut measured, &unit, &mut doc).unwrap();
            let work = MAX_STEPS - measured.steps;
            assert!(work > 0);
            clean(&measured);
            for allowance in 0..=work {
                let mut runtime = Runtime::new();
                runtime.steps = allowance;
                let result = execute(&mut runtime, &unit, &mut doc);
                if allowance == work {
                    assert_eq!(result.unwrap(), expected, "{source}");
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
                "completed frames retained code: {source}"
            );
        }
    }

    #[test]
    fn statement_storage_refusal_unwinds_pending_lists_loops_and_finally() {
        let sources = [
            "{let x=1;{let y=2;x+y;}}",
            "var x=0;for(let i=0;i<3;i++){x+=i;}x;",
            "var x='';for(var k in {a:1,b:2})x+=k;x;",
            "try{throw {x:7};}catch(e){e.x;}finally{[1,2,3];}",
            "switch(3){case 1:0;break;case 3:{let x=8;x;}default:9;}",
        ];
        for source in sources {
            let unit = compile(source);
            let mut doc = Document::parse("");
            let mut measured = Runtime::new();
            let initial = measured.allocated;
            measured.steps = MAX_STEPS;
            let expected = execute(&mut measured, &unit, &mut doc).unwrap();
            let needed = measured.allocated - initial;
            assert!(needed > 0);
            let cuts = [
                0,
                1,
                31,
                32,
                127,
                128,
                255,
                256,
                needed / 2,
                needed - 1,
                needed,
            ];
            for allowance in cuts {
                let mut runtime = Runtime::new();
                runtime.steps = MAX_STEPS;
                runtime.allocated = MAX_HEAP - allowance;
                let result = execute(&mut runtime, &unit, &mut doc);
                if allowance >= needed {
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
            assert!(weak.upgrade().is_none());
        }
        for source in [
            "try{while(true){}}catch(e){console.log('caught');}finally{console.log('finally');}",
            "try{throw 1;}catch(e){for(;;){}}finally{console.log('finally');}",
            "try{for(var k in document){}}catch(e){console.log('caught');}finally{console.log('finally');}",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            let error = runtime.execute(source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit() || error.is_unsupported());
            assert!(runtime.console.is_empty());
            clean(&runtime);
        }
        // Window enumeration is now supported, so its original case completes
        // normally and executes finally. The unsupported Document case above
        // still checks that an uncatchable host stop bypasses both handlers.
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        runtime.execute("try{for(var k in globalThis){}}catch(e){console.log('caught');}finally{console.log('finally');}", &mut doc).unwrap();
        assert_eq!(runtime.console, ["finally"]);
        clean(&runtime);
    }

    #[test]
    fn reentrant_exceptions_cross_only_their_own_driver_boundary() {
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var first={},second={},log='';function foreign(v){try{if(v)throw first;return 5;}finally{log+='f';}}function inner(){try{throw second;}catch(e){return e;}}", &mut doc).unwrap();
            let source = r#"
                var o={get x(){log+='g';return foreign(true);}},caught=false;
                try{for(let i=0;i<2;i++){switch(i){case 0:[1,o.x];break;default:log+='bad';}}}
                catch(e){caught=e===first;log+='c';}
                finally{log+='z';}
                function keep(){try{return first;}finally{inner();log+='k';}}
                function override(){try{throw first;}finally{return inner();}}
                var a=keep(),b=override(),c=foreign(false);
                caught&&a===first&&b===second&&c===5&&log==='gfczkf';
            "#;
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert_eq!(result.unwrap(), Value::Bool(true));
            clean(&runtime);
            assert_eq!(
                runtime
                    .execute(
                        "try{try{throw first;}finally{throw second;}}catch(e){e===second;}",
                        &mut doc
                    )
                    .unwrap(),
                Value::Bool(true)
            );
            clean(&runtime);
        }
    }

    #[test]
    fn suspended_iteration_retains_closures_and_live_for_in_state() {
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime
                .execute("var log='';function read(v){log+=v;return v;}", &mut doc)
                .unwrap();
            let source = r#"
                var keep=[];outer:for(let i=0;i<3;i++){try{keep.push(function(){return i;});continue outer;}finally{read(i);}}
                var proto={b:2,c:3},obj={a:1,b:4},names='';Object.setPrototypeOf(obj,proto);
                var target={set key(v){names+=v;read(v);if(v==='a'){delete obj.b;delete proto.c;proto.d=5;}}};
                for(target.key in obj){if(names==='ab'){break;}}
                var per=[];for(let k in {x:1,y:2}){per.push(function(){return k;});}
                keep[0]()===0&&keep[1]()===1&&keep[2]()===2&&names==='ab'&&log==='012ab'&&per[0]()==='x'&&per[1]()==='y';
            "#;
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert_eq!(result.unwrap(), Value::Bool(true));
            clean(&runtime);
        }
    }

    #[test]
    fn statement_chains_use_small_native_stack_with_independent_traversal_refusal() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                // These are directly constructed executable fixtures. The parser's
                // independent depth limit is neither exercised nor changed here.
                for (blocks, succeeds) in [
                    (80, true),
                    (MAX_STACK_UNITS - 2, true),
                    (MAX_STACK_UNITS - 1, true),
                    (MAX_STACK_UNITS, true),
                    (MAX_STACK_UNITS + 1, false),
                ] {
                    let mut statement = Stmt::Expr(Expr::Literal(Value::Number(7.0)));
                    for _ in 0..blocks {
                        statement = Stmt::Block(vec![statement]);
                    }
                    let unit = code::test_unit(&[statement]);
                    runtime.steps = MAX_STEPS;
                    let result = execute(&mut runtime, &unit, &mut doc);
                    if succeeds {
                        assert_eq!(result.unwrap(), Value::Number(7.0));
                    } else {
                        let error = result.unwrap_err();
                        assert!(error.is_resource_limit());
                        assert_eq!(error.message, "statement traversal depth exceeded");
                    }
                    clean(&runtime);
                    let weak = Rc::downgrade(&unit);
                    drop(unit);
                    assert!(weak.upgrade().is_none());
                }
                assert!(runtime.frames.capacity() >= 193);
                assert!(runtime.frames.capacity() <= super::super::MAX_FRAMES);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod for_of_tests;
