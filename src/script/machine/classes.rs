//! Source-ordered class definition with flat heritage/computed-key continuations.
use super::{
    Document, ExprFrame, Frame as Job, Output, Phase as ExprPhase, PropertyDescriptor, PropertyKey,
    Result, Runtime, ScriptError, Value, code, enter_frame, js_object, push, value,
};
use std::rc::Rc;

pub(in crate::script) struct Frame {
    unit: Rc<code::Unit>,
    class_id: code::ClassId,
    outer_environment: usize,
    phase: Phase,
}
enum Phase {
    Start {
        inferred_name: Option<PropertyKey>,
    },
    Heritage {
        class_environment: usize,
        inferred_name: Option<PropertyKey>,
    },
    ComputedKey {
        class_environment: usize,
        constructor: usize,
        prototype: usize,
        element_index: usize,
    },
    Vacant,
}
impl Frame {
    pub(in crate::script) fn new(
        unit: Rc<code::Unit>,
        class_id: code::ClassId,
        outer_environment: usize,
        inferred_name: Option<PropertyKey>,
    ) -> Self {
        Self {
            unit,
            class_id,
            outer_environment,
            phase: Phase::Start { inferred_name },
        }
    }
}

fn child(
    runtime: &mut Runtime,
    mut frame: Frame,
    phase: Phase,
    expression: code::ExprId,
    environment: usize,
) -> Result<Option<Output>> {
    let next = ExprFrame {
        unit: frame.unit.clone(),
        expression,
        env: environment,
        reference: false,
        phase: ExprPhase::Start,
    };
    frame.phase = phase;
    push(runtime, Job::Class(frame))?;
    enter_frame(runtime, Job::Expression(next))?;
    Ok(None)
}

pub(super) fn step(
    runtime: &mut Runtime,
    mut frame: Frame,
    output: Option<Output>,
    doc: &mut Document,
) -> Result<Option<Output>> {
    match std::mem::replace(&mut frame.phase, Phase::Vacant) {
        Phase::Start { inferred_name } => {
            // Six bounded transitions and six retained input/output roles.
            runtime.work(12)?;
            let unit = frame.unit.clone();
            let class = unit.class(frame.class_id);
            let name = match &class.name {
                Some(name) => runtime.class_identifier_name_hint(name)?,
                None => match inferred_name {
                    Some(name) => name,
                    None => runtime.class_identifier_name_hint("")?,
                },
            };
            let environment =
                runtime.class_environment(frame.outer_environment, class.name.as_deref())?;
            if let Some(heritage) = class.heritage {
                child(
                    runtime,
                    frame,
                    Phase::Heritage {
                        class_environment: environment,
                        inferred_name: Some(name),
                    },
                    heritage,
                    environment,
                )
            } else {
                runtime.work(
                    crate::script::promises::intrinsic_tree(runtime.prototypes.len()).0
                        * (1 + "Object".len() / 8),
                )?;
                let prototype = runtime
                    .prototypes
                    .get("Object")
                    .copied()
                    .filter(|id| *id < runtime.objects.len())
                    .ok_or_else(|| ScriptError::resource("class Object prototype is missing"))?;
                let constructor_parent = Value::Function(runtime.function_prototype);
                create(
                    runtime,
                    frame,
                    environment,
                    name,
                    constructor_parent,
                    Value::Object(prototype),
                )
            }
        }
        Phase::Heritage {
            class_environment,
            inferred_name,
        } => {
            let heritage = value(output);
            let name = inferred_name.expect("prepared class display name");
            if heritage == Value::Null {
                let constructor_parent = Value::Function(runtime.function_prototype);
                return create(
                    runtime,
                    frame,
                    class_environment,
                    name,
                    constructor_parent,
                    Value::Null,
                );
            }
            // Keep the heritage value for both the observable prototype Get
            // and the constructor's internal prototype. Each retain is paid
            // only when its stage is reached.
            runtime.work(1)?;
            if !runtime.is_constructor(heritage.clone())? {
                return Err(ScriptError::type_error(
                    "class heritage is not a constructor",
                ));
            }
            runtime.work(1)?;
            let prototype = runtime.get(heritage.clone(), "prototype", doc)?;
            if prototype != Value::Null && !js_object(&prototype) {
                return Err(ScriptError::type_error(
                    "class heritage prototype is not an object or null",
                ));
            }
            create(runtime, frame, class_environment, name, heritage, prototype)
        }
        Phase::ComputedKey {
            class_environment,
            constructor,
            prototype,
            element_index,
        } => {
            let key = runtime.property_key(value(output), doc)?;
            define_element(
                runtime,
                &frame,
                class_environment,
                constructor,
                prototype,
                element_index,
                key,
            )?;
            elements(
                runtime,
                frame,
                class_environment,
                constructor,
                prototype,
                element_index + 1,
            )
        }
        Phase::Vacant => unreachable!("class continuation phase"),
    }
}

fn create(
    runtime: &mut Runtime,
    frame: Frame,
    environment: usize,
    name: PropertyKey,
    constructor_parent: Value,
    prototype_parent: Value,
) -> Result<Option<Output>> {
    let prototype = runtime.class_prototype_value(prototype_parent)?;
    let Value::Object(prototype_id) = prototype else {
        unreachable!()
    };
    let class = frame.unit.class(frame.class_id);
    let code = code::FunctionRef::new(&frame.unit, class.constructor);
    let constructor = runtime.class_constructor_value(
        &code,
        environment,
        constructor_parent,
        prototype,
        &name,
    )?;
    let Value::Function(constructor_id) = constructor else {
        unreachable!()
    };
    elements(runtime, frame, environment, constructor_id, prototype_id, 0)
}

fn elements(
    runtime: &mut Runtime,
    frame: Frame,
    environment: usize,
    constructor: usize,
    prototype: usize,
    mut index: usize,
) -> Result<Option<Output>> {
    let unit = frame.unit.clone();
    while let Some(element) = unit.class(frame.class_id).elements.get(index) {
        // This admission precedes both computed evaluation and literal copies.
        runtime.work(8)?;
        match &element.name {
            code::PropertyName::Literal(name) => {
                runtime.work(1)?;
                let key = PropertyKey::String(name.clone());
                define_element(
                    runtime,
                    &frame,
                    environment,
                    constructor,
                    prototype,
                    index,
                    key,
                )?;
            }
            code::PropertyName::Computed(expression) => {
                return child(
                    runtime,
                    frame,
                    Phase::ComputedKey {
                        class_environment: environment,
                        constructor,
                        prototype,
                        element_index: index,
                    },
                    *expression,
                    environment,
                );
            }
        }
        index += 1;
    }
    let result = Value::Function(constructor);
    runtime.class_initialize_inner_name(
        environment,
        unit.class(frame.class_id).name.as_deref(),
        &result,
    )?;
    Ok(Some(Output::Value(result)))
}

fn define_element(
    runtime: &mut Runtime,
    frame: &Frame,
    environment: usize,
    constructor: usize,
    prototype: usize,
    index: usize,
    key: PropertyKey,
) -> Result<()> {
    let element = &frame.unit.class(frame.class_id).elements[index];
    let prefix = match element.kind {
        code::ClassElementKind::Method => None,
        code::ClassElementKind::Getter => Some("get"),
        code::ClassElementKind::Setter => Some("set"),
    };
    let method = runtime.class_method_value(
        &code::FunctionRef::new(&frame.unit, element.function),
        environment,
        &key,
        prefix,
    )?;
    let mut descriptor = PropertyDescriptor {
        enumerable: Some(false),
        configurable: Some(true),
        ..PropertyDescriptor::default()
    };
    match element.kind {
        code::ClassElementKind::Method => {
            descriptor.value = Some(method);
            descriptor.writable = Some(true);
        }
        code::ClassElementKind::Getter => descriptor.get = Some(method),
        code::ClassElementKind::Setter => descriptor.set = Some(method),
    }
    let target = if element.is_static {
        Value::Function(constructor)
    } else {
        Value::Object(prototype)
    };
    runtime.class_define_property(&target, &key, descriptor)
}
