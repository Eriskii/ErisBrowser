//! Class-only admission, factories and persistent constructor ownership.
use super::*;
use std::mem::size_of;

fn invalid() -> ScriptError {
    ScriptError::resource("invalid class runtime state")
}
fn add(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right).ok_or_else(invalid)
}
fn mul(left: usize, right: usize) -> Result<usize> {
    left.checked_mul(right).ok_or_else(invalid)
}
fn lookup_work(count: usize, length: usize) -> Result<usize> {
    mul(promises::intrinsic_tree(count).0, add(1, length)?)
}

impl Runtime {
    pub(super) fn class_name_hint(&mut self, source: &PropertyKey) -> Result<PropertyKey> {
        // Classification, one handle copy, its cleanup, and the return handoff.
        self.work(4)?;
        Ok(source.clone())
    }

    pub(super) fn class_identifier_name_hint(&mut self, source: &str) -> Result<PropertyKey> {
        self.work(add(8, source.len())?)?;
        if source.len() > MAX_SOURCE {
            return Err(invalid());
        }
        let units = source.encode_utf16().count();
        if units > MAX_STRING {
            return Err(invalid());
        }
        self.work(mul(2, units)?)?;
        self.charge(add(64, mul(4, units)?)?)?;
        let mut text = Vec::new();
        text.try_reserve_exact(units)
            .map_err(|_| ScriptError::resource("class name allocation failed"))?;
        text.extend(source.encode_utf16());
        Ok(PropertyKey::String(text.into()))
    }

    pub(super) fn class_environment(
        &mut self,
        parent: usize,
        inner_name: Option<&str>,
    ) -> Result<usize> {
        self.work(8)?;
        if parent >= self.environments.len() {
            return Err(invalid());
        }
        let plan = promises::growth_plan::<Environment>(
            self.environments.len(),
            self.environments.capacity(),
            1,
        )?;
        self.work(plan.work)?;
        self.charge(plan.bytes)?;
        if let Some(target) = plan.target {
            self.environments
                .try_reserve_exact(target - self.environments.len())
                .map_err(|_| ScriptError::resource("class environment allocation failed"))?;
        }
        // Keep the old row allowance and its selected representation delta.
        let environment = self.environment(parent)?;
        self.environments[environment].strict = true;
        if let Some(name) = inner_name {
            self.work(name.len())?;
            self.define(environment, name, Value::Undefined, false)?;
            let binding = self.environments[environment]
                .bindings
                .get_mut(name)
                .unwrap();
            binding.initialized = false;
            binding.strict_immutable = true;
        }
        Ok(environment)
    }

    fn class_reserve_objects(&mut self, plan: promises::GrowthPlan) -> Result<()> {
        if let Some(target) = plan.target {
            self.objects
                .try_reserve_exact(target - self.objects.len())
                .map_err(|_| ScriptError::resource("class object arena allocation failed"))?;
        }
        Ok(())
    }

    fn class_reserve_functions(&mut self, plan: promises::GrowthPlan) -> Result<()> {
        if let Some(target) = plan.target {
            self.functions
                .try_reserve_exact(target - self.functions.len())
                .map_err(|_| ScriptError::resource("class function arena allocation failed"))?;
        }
        Ok(())
    }

    pub(super) fn class_prototype_value(&mut self, parent: Value) -> Result<Value> {
        // Six bounded stages and the incoming parent/result owners.
        self.work(8)?;
        if parent != Value::Null && !js_object(&parent) {
            return Err(invalid());
        }
        let plan = promises::object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
        self.work(plan.work)?;
        self.charge(plan.bytes)?;
        self.class_reserve_objects(plan)?;
        self.dom_proto_object(Some(parent), 1).map(Value::Object)
    }

    pub(super) fn class_constructor_value(
        &mut self,
        code: &code::FunctionRef,
        class_env: usize,
        constructor_parent: Value,
        prototype: Value,
        display_name: &PropertyKey,
    ) -> Result<Value> {
        // Fourteen bounded stages and six parent/prototype/key owner roles.
        self.work(20)?;
        if code.class_kind.is_none()
            || !code.constructable
            || !code.strict
            || code.arrow
            || class_env >= self.environments.len()
            || !matches!(prototype, Value::Object(id) if id < self.objects.len())
            || !js_object(&constructor_parent)
        {
            return Err(invalid());
        }
        self.charge_function_code_copy(code)?;
        let mut functions =
            promises::growth_plan::<Function>(self.functions.len(), self.functions.capacity(), 1)?;
        functions.bytes = add(functions.bytes, size_of::<Function>())?;
        let objects = promises::object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
        self.work(functions.work)?;
        self.charge(functions.bytes)?;
        self.work(objects.work)?;
        self.charge(objects.bytes)?;
        self.class_reserve_functions(functions)?;
        self.class_reserve_objects(objects)?;
        let properties = self.dom_proto_object(Some(constructor_parent), 3)?;
        let length = PropertyKey::String(self.dom_proto_text("length")?);
        let name = PropertyKey::String(self.dom_proto_text("name")?);
        let prototype_key = PropertyKey::String(self.dom_proto_text("prototype")?);
        let constructor_key = PropertyKey::String(self.dom_proto_text("constructor")?);
        let Value::Object(prototype_id) = prototype else {
            unreachable!()
        };
        let id = self.functions.len();
        self.functions.push(Function {
            code: code.clone(),
            environment: class_env,
            properties,
            bound: None,
            home_object: Some(prototype_id),
        });
        let constructor = Value::Function(id);
        self.class_define_property(
            &constructor,
            &length,
            PropertyDescriptor::data_property(
                Value::Number(code.length() as f64),
                false,
                false,
                true,
            ),
        )?;
        // The inherited code-metadata allowance retains the empty name value.
        self.class_define_property(
            &constructor,
            &name,
            PropertyDescriptor::data_property(
                Value::String(JsString::default()),
                false,
                false,
                true,
            ),
        )?;
        self.class_define_property(
            &constructor,
            &prototype_key,
            PropertyDescriptor::data_property(Value::Object(prototype_id), false, false, false),
        )?;
        self.set_key_function_name(&constructor, display_name, None)?;
        self.class_define_property(
            &Value::Object(prototype_id),
            &constructor_key,
            PropertyDescriptor::data_property(Value::Function(id), true, false, true),
        )?;
        Ok(constructor)
    }

    pub(super) fn class_method_value(
        &mut self,
        code: &code::FunctionRef,
        class_env: usize,
        key: &PropertyKey,
        prefix: Option<&str>,
    ) -> Result<Value> {
        self.work(10)?;
        if !code.strict
            || code.constructable
            || code.arrow
            || code.class_kind.is_some()
            || code.self_name
            || class_env >= self.environments.len()
        {
            return Err(invalid());
        }
        let mut functions =
            promises::growth_plan::<Function>(self.functions.len(), self.functions.capacity(), 1)?;
        functions.bytes = add(functions.bytes, size_of::<Function>())?;
        let objects = promises::object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
        self.work(functions.work)?;
        self.charge(functions.bytes)?;
        self.work(objects.work)?;
        self.charge(objects.bytes)?;
        self.class_reserve_functions(functions)?;
        self.class_reserve_objects(objects)?;
        let method = self.function_value_prepaid_home(code, class_env)?;
        self.set_key_function_name(&method, key, prefix)?;
        Ok(method)
    }

    pub(super) fn class_define_property(
        &mut self,
        target: &Value,
        key: &PropertyKey,
        descriptor: PropertyDescriptor,
    ) -> Result<()> {
        self.work(8)?;
        let owner = match target {
            Value::Object(id) => *id,
            Value::Function(id) => self.functions.get(*id).ok_or_else(invalid)?.properties,
            _ => return Err(invalid()),
        };
        let bag = self.objects.get(owner).ok_or_else(invalid)?;
        if bag.order.len() != bag.values.len() || bag.order.len() > bag.order.capacity() {
            return Err(invalid());
        }
        let count = bag.values.len();
        self.work(lookup_work(
            count,
            key.as_string().map_or(0, JsString::len),
        )?)?;
        if !self.objects[owner].values.contains_key(key) {
            let order = &self.objects[owner].order;
            let plan = promises::growth_plan::<PropertyKey>(order.len(), order.capacity(), 1)?;
            self.work(plan.work)?;
            self.charge(plan.bytes)?;
            if let Some(target) = plan.target {
                let order = &mut self.objects[owner].order;
                order
                    .try_reserve_exact(target - order.len())
                    .map_err(|_| ScriptError::resource("class property order allocation failed"))?;
            }
        }
        if !self.define_own_key(target, key, descriptor)? {
            return Err(ScriptError::type_error("class property cannot be defined"));
        }
        Ok(())
    }

    pub(super) fn class_initialize_inner_name(
        &mut self,
        environment: usize,
        name: Option<&str>,
        constructor: &Value,
    ) -> Result<()> {
        self.work(6)?;
        let row = self.environments.get(environment).ok_or_else(invalid)?;
        if let Some(name) = name {
            let work = add(lookup_work(row.bindings.len(), name.len())?, 1)?;
            self.work(work)?;
            let binding = self.environments[environment]
                .bindings
                .get_mut(name)
                .ok_or_else(invalid)?;
            if binding.mutable || !binding.strict_immutable || binding.initialized {
                return Err(invalid());
            }
            binding.value = constructor.clone();
            binding.initialized = true;
        }
        Ok(())
    }

    pub(super) fn this_environment(&mut self, mut environment: usize) -> Result<usize> {
        loop {
            self.tick()?;
            let row = self.environments.get(environment).ok_or_else(invalid)?;
            if !matches!(row.this_binding, ThisBinding::Lexical) {
                return Ok(environment);
            }
            environment = row.parent.ok_or_else(invalid)?;
        }
    }

    pub(super) fn class_this_value(&self, owner: usize) -> Result<Value> {
        match &self
            .environments
            .get(owner)
            .ok_or_else(invalid)?
            .this_binding
        {
            ThisBinding::Initialized(value) => Ok(value.clone()),
            ThisBinding::Uninitialized => Err(ScriptError::reference("this is not initialized")),
            ThisBinding::Lexical => Err(invalid()),
        }
    }

    pub(super) fn class_super_target(
        &mut self,
        environment: usize,
    ) -> Result<(usize, Value, Value)> {
        self.work(12)?;
        // Two captures and the later bind/result copy; all precede argument effects.
        self.work(3)?;
        let owner = self.this_environment(environment)?;
        let row = &self.environments[owner];
        let id = row.active_function.ok_or_else(invalid)?;
        let target = row.new_target_binding.as_ref().ok_or_else(invalid)?;
        let function = self.functions.get(id).ok_or_else(invalid)?;
        if !matches!(
            function.code.class_kind,
            Some(code::ClassKind::ExplicitDerived | code::ClassKind::DefaultDerived)
        ) {
            return Err(invalid());
        }
        let parent = self
            .objects
            .get(function.properties)
            .ok_or_else(invalid)?
            .prototype
            .clone()
            .unwrap_or(Value::Null);
        Ok((owner, target.clone(), parent))
    }

    pub(super) fn class_bind_this(&mut self, owner: usize, constructed: Value) -> Result<Value> {
        if !js_object(&constructed) {
            return Err(invalid());
        }
        let row = self.environments.get_mut(owner).ok_or_else(invalid)?;
        match row.this_binding {
            ThisBinding::Initialized(_) => Err(ScriptError::reference(
                "super constructor already initialized this",
            )),
            ThisBinding::Uninitialized => {
                // Copy/cleanup and the transition were prepaid by class_super_target.
                row.this_binding = ThisBinding::Initialized(constructed.clone());
                Ok(constructed)
            }
            ThisBinding::Lexical => Err(invalid()),
        }
    }

    pub(super) fn class_validate_superclass(&mut self, superclass: &Value) -> Result<()> {
        // Keep a separate captured superclass for Construct. Reject bound
        // nonconstructors before the generic bound dispatcher copies arguments.
        self.work(1)?;
        if !self.is_constructor(superclass.clone())? {
            return Err(ScriptError::type_error("superclass is not a constructor"));
        }
        Ok(())
    }

    pub(super) fn construct_class(
        &mut self,
        id: usize,
        arguments: Vec<Value>,
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(16)?;
        self.work(2)?;
        if self
            .functions
            .get(id)
            .is_none_or(|f| f.code.class_kind.is_none())
            || arguments.len() > 65536
        {
            return Err(invalid());
        }
        self.tick()?;
        if self.calls >= MAX_CALLS {
            return Err(ScriptError::resource("script call stack limit exceeded"));
        }
        self.enter_stack(4)?;
        self.calls += 1;
        let result = machine::invoke_class_preentered(self, id, arguments, new_target, doc);
        self.calls -= 1;
        self.stack_units -= 4;
        result
    }
}
#[cfg(test)]
mod quota_tests;
#[cfg(test)]
mod super_validation_tests;
