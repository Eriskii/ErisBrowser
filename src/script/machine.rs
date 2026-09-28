//! Resumable expression and reference evaluation. Ordinary calls and statements
//! still use guarded native bridges until their continuation stages are added.
use super::{
    Document, JsString, MAX_DEPTH, MAX_STACK_UNITS, PropertyDescriptor, Reference, Result, Runtime,
    ScriptError, TrackedGlobal, Value, code, js_object, to_i32,
};
use std::collections::BTreeSet;
use std::rc::Rc;

// Each guarded expression can suspend at most one unticked reference job.
// Remaining native entry paths still consume their existing stack units.
const MAX_FRAMES: usize = 2 * MAX_STACK_UNITS;
const INITIAL_FRAMES: usize = 8;

pub(super) struct Frame {
    unit: Rc<code::Unit>,
    expression: code::ExprId,
    env: usize,
    reference: bool,
    phase: Phase,
}
enum Phase {
    Start,
    ReferenceObject,
    ReferenceKey(Value),
    Unary,
    Delete,
    BinaryHead,
    BinaryRight {
        index: usize,
        left: Value,
    },
    Conditional,
    Identity,
    Sequence(usize),
    Array {
        index: usize,
        values: Vec<Value>,
        holes: BTreeSet<usize>,
    },
    ObjectKey {
        index: usize,
        object: Value,
    },
    ObjectValue {
        index: usize,
        object: Value,
        key: JsString,
    },
    ObjectPrototype {
        index: usize,
        object: Value,
    },
    Template {
        index: usize,
        output: Vec<u16>,
    },
    Member,
    AssignReference,
    AssignValue {
        reference: Reference,
        old: Option<Value>,
        logical: bool,
    },
    Update,
    Callee,
    MemberCallee,
    Arguments {
        index: usize,
        function: Value,
        receiver: Value,
        arguments: Vec<Value>,
    },
}
enum Output {
    Value(Value),
    Reference(Reference),
}
fn value(output: Option<Output>) -> Value {
    match output {
        Some(Output::Value(value)) => value,
        _ => unreachable!("value continuation"),
    }
}
fn reference(output: Option<Output>) -> Reference {
    match output {
        Some(Output::Reference(reference)) => reference,
        _ => unreachable!("reference continuation"),
    }
}

pub(super) fn initialize(runtime: &mut Runtime) -> Result<()> {
    runtime.charge(INITIAL_FRAMES * std::mem::size_of::<Frame>() + 32)?;
    runtime
        .frames
        .try_reserve_exact(INITIAL_FRAMES)
        .map_err(|_| ScriptError::resource("expression frame allocation failed"))
}
fn push(runtime: &mut Runtime, frame: Frame) -> Result<()> {
    if runtime.frames.len() == MAX_FRAMES {
        return Err(ScriptError::resource("expression frame limit exceeded"));
    }
    if runtime.frames.len() == runtime.frames.capacity() {
        let capacity = runtime
            .frames
            .capacity()
            .saturating_mul(2)
            .clamp(INITIAL_FRAMES, MAX_FRAMES);
        runtime.work(runtime.frames.len() + 1)?;
        runtime.charge(capacity * std::mem::size_of::<Frame>() + 32)?;
        runtime
            .frames
            .try_reserve_exact(capacity - runtime.frames.len())
            .map_err(|_| ScriptError::resource("expression frame allocation failed"))?;
    }
    runtime.frames.push(frame);
    Ok(())
}
fn enter(runtime: &mut Runtime, frame: Frame) -> Result<()> {
    if !frame.reference {
        if runtime.eval_depth >= MAX_DEPTH {
            return Err(ScriptError::resource(
                "expression evaluation nesting limit exceeded",
            ));
        }
        runtime.enter_stack(1)?;
        runtime.eval_depth += 1;
    }
    push(runtime, frame)
}
fn child(
    runtime: &mut Runtime,
    parent: Frame,
    expression: code::ExprId,
    reference: bool,
) -> Result<Option<Output>> {
    let next = Frame {
        unit: parent.unit.clone(),
        expression,
        env: parent.env,
        reference,
        phase: Phase::Start,
    };
    push(runtime, parent)?;
    enter(runtime, next)?;
    Ok(None)
}
fn eval_child(
    runtime: &mut Runtime,
    mut frame: Frame,
    phase: Phase,
    expression: code::ExprId,
) -> Result<Option<Output>> {
    frame.phase = phase;
    child(runtime, frame, expression, false)
}
fn reference_child(
    runtime: &mut Runtime,
    mut frame: Frame,
    phase: Phase,
    expression: code::ExprId,
) -> Result<Option<Output>> {
    frame.phase = phase;
    child(runtime, frame, expression, true)
}
pub(super) fn evaluate(
    runtime: &mut Runtime,
    unit: &Rc<code::Unit>,
    expression: code::ExprId,
    env: usize,
    doc: &mut Document,
) -> Result<Value> {
    match drive(
        runtime,
        Frame {
            unit: unit.clone(),
            expression,
            env,
            reference: false,
            phase: Phase::Start,
        },
        doc,
    )? {
        Output::Value(value) => Ok(value),
        _ => unreachable!("expression root"),
    }
}
pub(super) fn evaluate_reference(
    runtime: &mut Runtime,
    unit: &Rc<code::Unit>,
    expression: code::ExprId,
    env: usize,
    doc: &mut Document,
) -> Result<Reference> {
    match drive(
        runtime,
        Frame {
            unit: unit.clone(),
            expression,
            env,
            reference: true,
            phase: Phase::Start,
        },
        doc,
    )? {
        Output::Reference(value) => Ok(value),
        _ => unreachable!("reference root"),
    }
}
fn drive(runtime: &mut Runtime, root: Frame, doc: &mut Document) -> Result<Output> {
    let base = runtime.frames.len();
    let depth = runtime.eval_depth;
    let stack = runtime.stack_units;
    let result = (|| {
        enter(runtime, root)?;
        let mut output = None;
        while runtime.frames.len() > base {
            let frame = runtime.frames.pop().unwrap();
            let entered = !frame.reference;
            output = step(runtime, frame, output.take(), doc)?;
            if output.is_some() && entered {
                runtime.eval_depth -= 1;
                runtime.stack_units -= 1;
            }
        }
        Ok(output.expect("completed expression drive"))
    })();
    // Cleanup is bounded and cannot need another allocation or author callback.
    runtime.frames.truncate(base);
    runtime.eval_depth = depth;
    runtime.stack_units = stack;
    result
}
fn step(
    runtime: &mut Runtime,
    mut frame: Frame,
    output: Option<Output>,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let id = frame.expression;
    let env = frame.env;
    let phase = std::mem::replace(&mut frame.phase, Phase::Start);
    match phase {
        Phase::Start => start(runtime, frame, doc),
        Phase::ReferenceObject => {
            let code::Expr::Member(_, key) = unit.expr(id) else {
                unreachable!()
            };
            eval_child(runtime, frame, Phase::ReferenceKey(value(output)), *key)
        }
        Phase::ReferenceKey(object) => Ok(Some(Output::Reference(Reference::Property(
            object,
            value(output),
            runtime.environments[env].strict,
        )))),
        Phase::Unary => {
            unary(runtime, &unit, id, value(output), doc).map(|v| Some(Output::Value(v)))
        }
        Phase::Delete => {
            let Reference::Property(object, key, strict) = reference(output) else {
                unreachable!()
            };
            let key = runtime.reference_key(&object, key, doc)?;
            let deleted = runtime.delete_property(object, &key)?;
            if !deleted && strict {
                return Err(ScriptError::type_error(
                    "cannot delete a non-configurable property",
                ));
            }
            Ok(Some(Output::Value(Value::Bool(deleted))))
        }
        Phase::BinaryHead => binary_next(runtime, frame, 0, value(output)),
        Phase::BinaryRight { index, left } => {
            let code::Expr::BinaryChain(_, operations) = unit.expr(id) else {
                unreachable!()
            };
            let op = &operations[index].0;
            let right = value(output);
            let value = if matches!(op.as_str(), "&&" | "||" | "??") {
                right
            } else {
                runtime.binary_value(op, left, right, doc)?
            };
            binary_next(runtime, frame, index + 1, value)
        }
        Phase::Conditional => {
            let code::Expr::Conditional(_, yes, no) = unit.expr(id) else {
                unreachable!()
            };
            let next = if value(output).truthy() { *yes } else { *no };
            eval_child(runtime, frame, Phase::Identity, next)
        }
        Phase::Identity => Ok(output),
        Phase::Sequence(index) => {
            let code::Expr::Sequence(items) = unit.expr(id) else {
                unreachable!()
            };
            if let Some(next) = items.get(index + 1) {
                eval_child(runtime, frame, Phase::Sequence(index + 1), *next)
            } else {
                Ok(output)
            }
        }
        Phase::Array {
            index,
            mut values,
            holes,
        } => {
            values.push(value(output));
            array_next(runtime, frame, index + 1, values, holes)
        }
        Phase::ObjectKey { index, object } => {
            let key = runtime.string_hint(value(output), doc)?;
            object_entry(runtime, frame, index, object, key)
        }
        Phase::ObjectValue { index, object, key } => {
            let code::Expr::Object(items) = unit.expr(id) else {
                unreachable!()
            };
            let code::ObjectEntry::Data(expression) = &items[index].1 else {
                unreachable!()
            };
            let value = value(output);
            if unit.anonymous(*expression) {
                runtime.set_function_name(&value, &key, None)?;
            }
            runtime.define_own(
                &object,
                &key,
                PropertyDescriptor::data_property(value, true, true, true),
            )?;
            object_next(runtime, frame, index + 1, object)
        }
        Phase::ObjectPrototype { index, object } => {
            let prototype = value(output);
            if js_object(&prototype) || prototype == Value::Null {
                runtime.set_object_prototype(&object, prototype)?;
            }
            object_next(runtime, frame, index + 1, object)
        }
        Phase::Template {
            index,
            output: mut text,
        } => {
            let code::Expr::Template(_, tail) = unit.expr(id) else {
                unreachable!()
            };
            let cooked = runtime.string_hint(value(output), doc)?;
            runtime.append_template_text(&mut text, &cooked)?;
            runtime.append_template_text(&mut text, &tail[index].1)?;
            if let Some((next, _)) = tail.get(index + 1) {
                eval_child(
                    runtime,
                    frame,
                    Phase::Template {
                        index: index + 1,
                        output: text,
                    },
                    *next,
                )
            } else {
                runtime.string(text).map(|v| Some(Output::Value(v)))
            }
        }
        Phase::Member => {
            let mut reference = reference(output);
            runtime
                .read_reference(&mut reference, doc)
                .map(|v| Some(Output::Value(v)))
        }
        Phase::AssignReference => assign_reference(runtime, frame, reference(output), doc),
        Phase::AssignValue {
            reference,
            old,
            logical,
        } => {
            let code::Expr::Assign(op, left, right) = unit.expr(id) else {
                unreachable!()
            };
            let mut value = value(output);
            if logical
                && let code::Expr::Ident(name) = unit.expr(*left)
                && unit.anonymous(*right)
            {
                runtime.work(1 + name.len() / 8)?;
                runtime.charge(64 + name.len().saturating_mul(4))?;
                runtime.set_function_name(&value, &name.as_str().into(), None)?;
            }
            if let Some(old) = old {
                value = runtime.binary_value(&op[..op.len() - 1], old, value, doc)?;
            }
            runtime.write_reference(reference, value.clone(), doc)?;
            Ok(Some(Output::Value(value)))
        }
        Phase::Update => {
            let code::Expr::Update(_, delta, prefix) = unit.expr(id) else {
                unreachable!()
            };
            let mut reference = reference(output);
            let previous = runtime.read_reference(&mut reference, doc)?;
            if let Value::String(text) = &previous {
                runtime.work(1 + text.len() / 8)?;
            }
            let old = runtime.number_value(previous, doc)?;
            let value = Value::Number(old + delta);
            runtime.write_reference(reference, value.clone(), doc)?;
            Ok(Some(Output::Value(if *prefix {
                value
            } else {
                Value::Number(old)
            })))
        }
        Phase::Callee => arguments_start(runtime, frame, value(output), Value::Undefined, doc),
        Phase::MemberCallee => {
            let mut reference = reference(output);
            let Reference::Property(receiver, _, _) = &reference else {
                unreachable!()
            };
            let receiver = receiver.clone();
            let function = runtime.read_reference(&mut reference, doc)?;
            arguments_start(runtime, frame, function, receiver, doc)
        }
        Phase::Arguments {
            index,
            function,
            receiver,
            mut arguments,
        } => {
            arguments.push(value(output));
            arguments_next(
                runtime,
                frame,
                index + 1,
                function,
                receiver,
                arguments,
                doc,
            )
        }
    }
}

fn start(runtime: &mut Runtime, frame: Frame, doc: &mut Document) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let id = frame.expression;
    let env = frame.env;
    if frame.reference {
        return match unit.expr(id) {
            code::Expr::Ident(name) => {
                let owner = runtime.resolve_binding(env, name)?;
                Ok(Some(Output::Reference(Reference::CodeName {
                    unit: unit.clone(),
                    expression: id,
                    owner,
                    strict: runtime.environments[env].strict,
                })))
            }
            code::Expr::Member(object, _) => {
                eval_child(runtime, frame, Phase::ReferenceObject, *object)
            }
            _ => Err(ScriptError::type_error("invalid assignment target")),
        };
    }
    runtime.tick()?;
    match unit.expr(id) {
        code::Expr::Literal(v) => Ok(Some(Output::Value(v.clone()))),
        code::Expr::RegExp(v) => runtime
            .regexp_object(v.clone())
            .map(|v| Some(Output::Value(v))),
        code::Expr::Ident(name) => {
            let owner = runtime
                .resolve_binding(env, name)?
                .ok_or_else(|| ScriptError::reference(format!("'{name}' is not defined")))?;
            runtime
                .binding_value(owner, name, doc)
                .map(|v| Some(Output::Value(v)))
        }
        code::Expr::Unary(op, expression) => {
            if op == "delete" {
                match unit.expr(*expression) {
                    code::Expr::Member(..) => {
                        return reference_child(runtime, frame, Phase::Delete, *expression);
                    }
                    code::Expr::Ident(name) => {
                        return delete_name(runtime, env, name).map(|v| Some(Output::Value(v)));
                    }
                    _ => {}
                }
            }
            if op == "typeof"
                && let code::Expr::Ident(name) = unit.expr(*expression)
                && runtime.resolve_binding(env, name)?.is_none()
            {
                return runtime.string("undefined").map(|v| Some(Output::Value(v)));
            }
            eval_child(runtime, frame, Phase::Unary, *expression)
        }
        code::Expr::BinaryChain(head, _) => eval_child(runtime, frame, Phase::BinaryHead, *head),
        code::Expr::Conditional(condition, _, _) => {
            eval_child(runtime, frame, Phase::Conditional, *condition)
        }
        code::Expr::Sequence(items) => match items.first() {
            Some(first) => eval_child(runtime, frame, Phase::Sequence(0), *first),
            None => Ok(Some(Output::Value(Value::Undefined))),
        },
        code::Expr::Array(items) => {
            // Prepay the final array's value storage before constructing it.
            let values = reserve_values(runtime, items.len(), true)?;
            runtime.charge(
                items
                    .iter()
                    .filter(|item| item.is_none())
                    .count()
                    .saturating_mul(32),
            )?;
            array_next(runtime, frame, 0, values, BTreeSet::new())
        }
        code::Expr::Object(_) => {
            let object = runtime.object_ordered([])?;
            object_next(runtime, frame, 0, object)
        }
        code::Expr::Template(head, tail) => {
            let mut output = Vec::new();
            runtime.append_template_text(&mut output, head)?;
            match tail.first() {
                Some((first, _)) => {
                    eval_child(runtime, frame, Phase::Template { index: 0, output }, *first)
                }
                None => runtime.string(output).map(|v| Some(Output::Value(v))),
            }
        }
        code::Expr::Member(..) => reference_child(runtime, frame, Phase::Member, id),
        code::Expr::Assign(_, left, _) => {
            reference_child(runtime, frame, Phase::AssignReference, *left)
        }
        code::Expr::Update(target, _, _) => reference_child(runtime, frame, Phase::Update, *target),
        code::Expr::Call(callee, _) => {
            if matches!(unit.expr(*callee), code::Expr::Member(..)) {
                reference_child(runtime, frame, Phase::MemberCallee, *callee)
            } else {
                eval_child(runtime, frame, Phase::Callee, *callee)
            }
        }
        code::Expr::New(callee, _) => eval_child(runtime, frame, Phase::Callee, *callee),
        code::Expr::Function(id) => runtime
            .function_value(&code::FunctionRef::new(&unit, *id), env)
            .map(|v| Some(Output::Value(v))),
    }
}
fn binary_next(
    runtime: &mut Runtime,
    frame: Frame,
    mut index: usize,
    left: Value,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Expr::BinaryChain(_, operations) = unit.expr(frame.expression) else {
        unreachable!()
    };
    while let Some((op, right)) = operations.get(index) {
        runtime.tick()?;
        if op == "&&" && !left.truthy()
            || op == "||" && left.truthy()
            || op == "??" && !matches!(left, Value::Undefined | Value::Null)
        {
            index += 1;
            continue;
        }
        return eval_child(runtime, frame, Phase::BinaryRight { index, left }, *right);
    }
    Ok(Some(Output::Value(left)))
}
fn unary(
    runtime: &mut Runtime,
    unit: &code::Unit,
    id: code::ExprId,
    value: Value,
    doc: &mut Document,
) -> Result<Value> {
    let code::Expr::Unary(op, _) = unit.expr(id) else {
        unreachable!()
    };
    if op == "delete" {
        return Ok(Value::Bool(true));
    }
    if let Value::String(text) = &value {
        runtime.work(1 + text.len() / 8)?;
    }
    match op.as_str() {
        "!" => Ok(Value::Bool(!value.truthy())),
        "-" => Ok(Value::Number(-runtime.number_value(value, doc)?)),
        "+" => Ok(Value::Number(runtime.number_value(value, doc)?)),
        "~" => Ok(Value::Number(
            (!to_i32(runtime.number_value(value, doc)?)) as f64,
        )),
        "void" => Ok(Value::Undefined),
        "typeof" => runtime.string(match value {
            Value::Undefined => "undefined",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Function(_) | Value::Native(_) => "function",
            _ => "object",
        }),
        _ => Err(ScriptError::new("unknown unary operator")),
    }
}
fn delete_name(runtime: &mut Runtime, env: usize, name: &str) -> Result<Value> {
    if name == "this" {
        return Ok(Value::Bool(true));
    }
    if let Some(owner) = runtime.resolve_binding(env, name)? {
        if owner == 0
            && let Some(kind) = TrackedGlobal::from_name(name)
        {
            let key = runtime.global_key(kind);
            return runtime
                .delete_property(Value::Window, &key)
                .map(Value::Bool);
        }
        let removable = runtime.environments[owner].bindings[name].deletable;
        if removable {
            runtime.environments[owner].bindings.remove(name);
        }
        Ok(Value::Bool(removable))
    } else {
        Ok(Value::Bool(true))
    }
}
fn assign_reference(
    runtime: &mut Runtime,
    frame: Frame,
    mut reference: Reference,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Expr::Assign(op, _, right) = unit.expr(frame.expression) else {
        unreachable!()
    };
    let logical = matches!(op.as_str(), "&&=" | "||=" | "??=");
    let old = if op != "=" {
        Some(runtime.read_reference(&mut reference, doc)?)
    } else {
        None
    };
    if logical {
        let value = old.as_ref().unwrap();
        if op == "&&=" && !value.truthy()
            || op == "||=" && value.truthy()
            || op == "??=" && !matches!(value, Value::Undefined | Value::Null)
        {
            return Ok(Some(Output::Value(old.unwrap())));
        }
    }
    eval_child(
        runtime,
        frame,
        Phase::AssignValue {
            reference,
            old: if logical { None } else { old },
            logical,
        },
        *right,
    )
}
fn reserve_values(runtime: &mut Runtime, length: usize, array: bool) -> Result<Vec<Value>> {
    runtime.work(length)?;
    if array || length != 0 {
        runtime.charge(32 + length.saturating_mul(std::mem::size_of::<Value>()))?;
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(|_| ScriptError::resource("expression values allocation failed"))?;
    Ok(values)
}
fn array_next(
    runtime: &mut Runtime,
    frame: Frame,
    mut index: usize,
    mut values: Vec<Value>,
    mut holes: BTreeSet<usize>,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Expr::Array(items) = unit.expr(frame.expression) else {
        unreachable!()
    };
    while let Some(item) = items.get(index) {
        if let Some(expression) = item {
            return eval_child(
                runtime,
                frame,
                Phase::Array {
                    index,
                    values,
                    holes,
                },
                *expression,
            );
        }
        holes.insert(index);
        values.push(Value::Undefined);
        index += 1;
    }
    let array = runtime.array_reserved(values)?;
    let Value::Array(id) = array else {
        unreachable!()
    };
    runtime.array_holes[id] = holes;
    Ok(Some(Output::Value(array)))
}
fn object_next(
    runtime: &mut Runtime,
    frame: Frame,
    index: usize,
    object: Value,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let code::Expr::Object(items) = unit.expr(frame.expression) else {
        unreachable!()
    };
    let Some((key, entry)) = items.get(index) else {
        return Ok(Some(Output::Value(object)));
    };
    if let code::ObjectEntry::Prototype(expression) = entry {
        return eval_child(
            runtime,
            frame,
            Phase::ObjectPrototype { index, object },
            *expression,
        );
    }
    match key {
        code::PropertyName::Literal(key) => {
            object_entry(runtime, frame, index, object, key.clone())
        }
        code::PropertyName::Computed(expression) => eval_child(
            runtime,
            frame,
            Phase::ObjectKey { index, object },
            *expression,
        ),
    }
}
fn object_entry(
    runtime: &mut Runtime,
    frame: Frame,
    mut index: usize,
    object: Value,
    mut key: JsString,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let env = frame.env;
    let code::Expr::Object(items) = unit.expr(frame.expression) else {
        unreachable!()
    };
    loop {
        let mut desc = PropertyDescriptor {
            enumerable: Some(true),
            configurable: Some(true),
            ..PropertyDescriptor::default()
        };
        match &items[index].1 {
            code::ObjectEntry::Data(expression) => {
                return eval_child(
                    runtime,
                    frame,
                    Phase::ObjectValue { index, object, key },
                    *expression,
                );
            }
            code::ObjectEntry::Method(id) => {
                let function = runtime.function_value(&code::FunctionRef::new(&unit, *id), env)?;
                runtime.set_function_name(&function, &key, None)?;
                desc.value = Some(function);
                desc.writable = Some(true);
            }
            code::ObjectEntry::Accessor(id, setter) => {
                let function = runtime.function_value(&code::FunctionRef::new(&unit, *id), env)?;
                runtime.set_function_name(
                    &function,
                    &key,
                    Some(if *setter { "set" } else { "get" }),
                )?;
                if *setter {
                    desc.set = Some(function);
                } else {
                    desc.get = Some(function);
                }
            }
            code::ObjectEntry::Prototype(_) => {
                unreachable!("prototype entry bypasses key conversion")
            }
        }
        runtime.define_own(&object, &key, desc)?;
        index += 1;
        let Some((next, entry)) = items.get(index) else {
            return Ok(Some(Output::Value(object)));
        };
        if let code::ObjectEntry::Prototype(expression) = entry {
            return eval_child(
                runtime,
                frame,
                Phase::ObjectPrototype { index, object },
                *expression,
            );
        }
        match next {
            code::PropertyName::Literal(next) => key = next.clone(),
            code::PropertyName::Computed(expression) => {
                return eval_child(
                    runtime,
                    frame,
                    Phase::ObjectKey { index, object },
                    *expression,
                );
            }
        }
    }
}
fn arguments_start(
    runtime: &mut Runtime,
    frame: Frame,
    function: Value,
    receiver: Value,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let (code::Expr::Call(_, items) | code::Expr::New(_, items)) = unit.expr(frame.expression)
    else {
        unreachable!()
    };
    let arguments = reserve_values(runtime, items.len(), false)?;
    arguments_next(runtime, frame, 0, function, receiver, arguments, doc)
}
fn arguments_next(
    runtime: &mut Runtime,
    frame: Frame,
    index: usize,
    function: Value,
    receiver: Value,
    arguments: Vec<Value>,
    doc: &mut Document,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    let (code::Expr::Call(_, items) | code::Expr::New(_, items)) = unit.expr(frame.expression)
    else {
        unreachable!()
    };
    if let Some(expression) = items.get(index) {
        return eval_child(
            runtime,
            frame,
            Phase::Arguments {
                index,
                function,
                receiver,
                arguments,
            },
            *expression,
        );
    }
    let value = if matches!(unit.expr(frame.expression), code::Expr::New(..)) {
        runtime.construct(function, arguments, doc)?
    } else {
        runtime.call(function, arguments, receiver, doc)?
    };
    Ok(Some(Output::Value(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Expr, MAX_HEAP, MAX_STEPS, Parser, Stmt};

    fn expression(source: &str) -> (Rc<code::Unit>, code::ExprId) {
        let unit = code::compile(Parser::program(source).unwrap()).unwrap();
        let code::Stmt::Expr(id) = unit.stmt(unit.body[0]) else {
            panic!("expression fixture");
        };
        let id = *id;
        (unit, id)
    }
    fn unary_unit(depth: usize) -> (Rc<code::Unit>, code::ExprId) {
        let mut expression = Expr::Literal(Value::Bool(true));
        for _ in 0..depth {
            expression = Expr::Unary("!".into(), Box::new(expression));
        }
        let unit = code::test_unit(&[Stmt::Expr(expression)]);
        let code::Stmt::Expr(id) = unit.stmt(unit.body[0]) else {
            unreachable!()
        };
        let id = *id;
        (unit, id)
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
    fn frame_growth_is_precharged_reused_and_does_not_retain_completed_units() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let (unit, id) = unary_unit(20);
        let weak = Rc::downgrade(&unit);
        let initial = runtime.allocated;
        assert_eq!(
            runtime.eval(&unit, &id, 0, &mut document).unwrap(),
            Value::Bool(true)
        );
        clean(&runtime);
        assert!(runtime.frames.capacity() >= 21);
        let allocated = runtime.allocated;
        assert!(allocated > initial);
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime.eval(&unit, &id, 0, &mut document).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(runtime.allocated, allocated);
        clean(&runtime);
        drop(unit);
        assert!(weak.upgrade().is_none());

        let mut runtime = Runtime::new();
        let initial = runtime.allocated;
        let (unit, id) = unary_unit(20);
        let weak = Rc::downgrade(&unit);
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .eval(&unit, &id, 0, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        clean(&runtime);
        assert_eq!(runtime.frames.capacity(), INITIAL_FRAMES);
        drop(unit);
        assert!(weak.upgrade().is_none());
        runtime.allocated = initial;
        runtime.steps = MAX_STEPS;
        let (unit, id) = expression("1+2;");
        assert_eq!(
            runtime.eval(&unit, &id, 0, &mut document).unwrap(),
            Value::Number(3.0)
        );
        clean(&runtime);
    }

    #[test]
    fn every_work_cut_unwinds_only_its_frames_and_releases_owned_payloads() {
        let sources = [
            "1+(2*(3+4));",
            "[1,,2+3];",
            "({a:1+2,b:4});",
            "`a${1+2}b${3+4}`;",
            "true?(1,2+3):4;",
            "({a:1})['a'];",
            "Number(1+2);",
            "Math.max(1,2,3);",
        ];
        for source in sources {
            let (unit, id) = expression(source);
            let mut measured = Runtime::new();
            let mut document = Document::parse("");
            measured.steps = MAX_STEPS;
            measured.eval(&unit, &id, 0, &mut document).unwrap();
            let work = MAX_STEPS - measured.steps;
            assert!(work > 0);
            for allowance in 0..=work {
                let mut runtime = Runtime::new();
                runtime.steps = allowance;
                let result = runtime.eval(&unit, &id, 0, &mut document);
                if allowance == work {
                    assert!(result.is_ok(), "{source}: {allowance}");
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
    }

    #[test]
    fn callback_reentry_preserves_outer_values_references_and_abrupt_identity() {
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            runtime.execute("var log='',marker={};function other(x){log+='r';return x+2;}function fail(){throw marker;}",&mut document).unwrap();
            let source = r#"
                var o={get x(){log+='g';return other(3);},set x(v){log+='s'+v;}},key={toString(){log+='k';return 'x';}};
                var result=[1+o.x,`${o.x}`,(o[key]+=other(1)),({[other('a')]:o.x})];
                var seen=false;try{[1,o.x,fail(),(log+='bad')];}catch(e){seen=e===marker;}
                var method={get m(){log+='m';return function(a,b){log+='c';return this===method&&a===3&&b===4;};}};
                var call=method.m(other(1),other(2));
                result[0]===6&&result[1]==='5'&&result[2]===8&&result[3].a2===5&&seen&&call&&log.indexOf('bad')===-1;
            "#;
            let result = if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            };
            assert_eq!(result.unwrap(), Value::Bool(true));
            clean(&runtime);
            assert_eq!(
                runtime.lookup(0, "log").unwrap().1.to_string(),
                "grgrkgrrs8rgrgrmrrc"
            );
            let error=runtime.execute("try{[1,{get x(){while(true){}}}.x];}catch(e){log+='caught';}finally{log+='finally';}",&mut document).unwrap_err();
            assert!(error.is_resource_limit());
            clean(&runtime);
            assert!(
                !runtime
                    .lookup(0, "log")
                    .unwrap()
                    .1
                    .to_string()
                    .contains("caught")
            );
            assert!(
                !runtime
                    .lookup(0, "log")
                    .unwrap()
                    .1
                    .to_string()
                    .contains("finally")
            );
        }
    }

    #[test]
    fn identifier_references_retain_code_without_copying_names_or_losing_recreation_rules() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime.execute("var held=0;", &mut document).unwrap();
        let (unit, id) = expression("held;");
        let weak = Rc::downgrade(&unit);
        runtime.allocated = MAX_HEAP;
        let before = runtime.allocated;
        let mut reference = runtime.reference(&unit, &id, 0, &mut document).unwrap();
        assert_eq!(
            runtime
                .read_reference(&mut reference, &mut document)
                .unwrap(),
            Value::Number(0.0)
        );
        runtime
            .write_reference(reference, Value::Number(3.0), &mut document)
            .unwrap();
        assert_eq!(runtime.allocated, before);
        clean(&runtime);
        let reference = runtime.reference(&unit, &id, 0, &mut document).unwrap();
        drop(unit);
        assert!(weak.upgrade().is_some());
        drop(reference);
        assert!(weak.upgrade().is_none());
        let mut runtime = Runtime::new();
        assert_eq!(
            runtime
                .execute("fresh=2;fresh+=(delete fresh,3);fresh;", &mut document)
                .unwrap(),
            Value::Number(5.0)
        );
        assert_eq!(runtime.execute_strict("var hit=false;try{fresh+=(delete globalThis.fresh,4);}catch(e){hit=e instanceof ReferenceError;}hit;",&mut document).unwrap(),Value::Bool(true));
        clean(&runtime);
    }

    #[test]
    fn array_and_argument_precharges_precede_children_and_array_arenas_stay_aligned() {
        let mut document = Document::parse("");
        for source in ["[touch()];", "target(touch());"] {
            let mut runtime = Runtime::new();
            runtime
                .execute(
                    "var hit=0;function touch(){hit++;return 1;}function target(x){return x;}",
                    &mut document,
                )
                .unwrap();
            let (unit, id) = expression(source);
            runtime.allocated = MAX_HEAP;
            assert!(
                runtime
                    .eval(&unit, &id, 0, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(0, "hit").unwrap().1, Value::Number(0.0));
            clean(&runtime);
        }
        let mut runtime = Runtime::new();
        let (unit, id) = expression("[];");
        let before = (
            runtime.arrays.len(),
            runtime.array_properties.len(),
            runtime.array_holes.len(),
        );
        runtime.allocated = MAX_HEAP - 32;
        assert!(
            runtime
                .eval(&unit, &id, 0, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (
                runtime.arrays.len(),
                runtime.array_properties.len(),
                runtime.array_holes.len()
            ),
            before
        );
        clean(&runtime);
    }

    #[test]
    fn expression_execution_uses_small_native_stack_and_keeps_existing_depth_guards() {
        // Parsing is verified on the normal test stack. Build equivalent code
        // directly inside the small-stack thread, isolating execution from the
        // still-recursive parser. This is not deeper call or parser acceptance.
        Runtime::parse_only(&format!("{}true;", "!".repeat(80))).unwrap();
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                let (unit, id) = unary_unit(80);
                assert_eq!(
                    runtime.eval(&unit, &id, 0, &mut document).unwrap(),
                    Value::Bool(true)
                );
                clean(&runtime);
                let (unit, id) = unary_unit(MAX_DEPTH - 1);
                assert_eq!(
                    runtime.eval(&unit, &id, 0, &mut document).unwrap(),
                    Value::Bool(false)
                );
                clean(&runtime);
                let (unit, id) = unary_unit(MAX_DEPTH);
                assert!(
                    runtime
                        .eval(&unit, &id, 0, &mut document)
                        .unwrap_err()
                        .is_resource_limit()
                );
                clean(&runtime);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
