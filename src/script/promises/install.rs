//! Direct Promise intrinsic installation under the frozen startup allowance.
use super::*;
use std::collections::btree_map::Entry;
use std::mem::size_of;
use std::num::NonZeroUsize;

const GLOBAL: &str = "Promise";

// Same pinned B=6 comparison/insertion envelope used by other installers.
pub(in crate::script) fn tree(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut height = 1;
    while capacity >= 6 {
        capacity /= 6;
        height += 1;
    }
    (count.min(11 * height), height)
}

impl Runtime {
    pub(in crate::script) fn promise_native(
        &mut self,
        properties: usize,
        name: &str,
        receiver: Value,
    ) -> Result<Value> {
        self.work(8 + name.len())?;
        self.charge(size_of::<Native>() + 32 + name.len())?;
        let mut owned = String::new();
        owned
            .try_reserve_exact(name.len())
            .map_err(|_| ScriptError::resource("Promise native name allocation failed"))?;
        owned.push_str(name);
        let properties = NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("Promise metadata ID is zero"))?;
        Ok(Value::Native(Rc::new(Native {
            name: owned,
            receiver,
            properties: Some(properties),
        })))
    }

    fn promise_intrinsic_method(
        &mut self,
        internal: &str,
        display: JsString,
        arity: usize,
        keys: [&JsString; 2],
    ) -> Result<Value> {
        let bag = self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        // These two immutable-key clones were prepaid by the installer body.
        self.dom_proto_property(
            bag,
            keys[0].clone().into(),
            Property::data(Value::Number(arity as f64), false, false, true),
        )?;
        self.dom_proto_property(
            bag,
            keys[1].clone().into(),
            Property::data(Value::String(display), false, false, true),
        )?;
        self.promise_native(bag, internal, Value::Undefined)
    }

    #[cfg(test)]
    pub(in crate::script) fn install_promise_intrinsics(&mut self) -> Result<()> {
        // Preserve standalone guard-before-allocation ordering and all fourteen
        // temporary handle releases, including the two keys created here.
        self.work(23)?;
        self.check_promise_install_state()?;
        let keys = [self.dom_proto_text("length")?, self.dom_proto_text("name")?];
        self.install_promise_intrinsics_body(keys)
    }

    pub(in crate::script) fn install_promise_intrinsics_with_keys(
        &mut self,
        keys: [JsString; 2],
    ) -> Result<()> {
        // The producer prepaid both supplied handles' releases, including
        // refusal of this debit. Pay guards, publication and twelve pool releases.
        self.work(21)?;
        self.check_promise_install_state()?;
        self.install_promise_intrinsics_body(keys)
    }

    pub(in crate::script) fn install_promise_any_intrinsic(&mut self) -> Result<()> {
        // Eight bounded guards/control roles and four temporary releases.
        // This late installation preserves every existing metadata object ID.
        self.work(12)?;
        let intrinsics = self
            .promises
            .intrinsics
            .as_ref()
            .ok_or_else(|| ScriptError::resource("Promise intrinsics missing"))?;
        let Value::Native(constructor) = &intrinsics.constructor else {
            return Err(ScriptError::resource("invalid Promise constructor"));
        };
        let owner = constructor
            .properties
            .map(NonZeroUsize::get)
            .filter(|id| *id < self.objects.len())
            .ok_or_else(|| ScriptError::resource("Promise metadata missing"))?;
        let bag = &self.objects[owner];
        if bag.values.len() != 9 || bag.order.len() != 9 || bag.order.capacity() < bag.order.len() {
            return Err(ScriptError::resource("invalid Promise metadata shape"));
        }
        let length = self.dom_proto_text("length")?;
        let name = self.dom_proto_text("name")?;
        let any = self.dom_proto_text("any")?;
        // Two metadata key copies and the display-name copy.
        self.work(3)?;
        let method =
            self.promise_intrinsic_method("Promise.any", any.clone(), 1, [&length, &name])?;
        // Existing insertion pays physical order growth and duplicate checks.
        self.dom_proto_property(owner, any.into(), Property::data(method, true, false, true))
    }

    pub(in crate::script) fn install_promise_with_resolvers_intrinsic(&mut self) -> Result<()> {
        // Eight bounded guards/control roles and four temporary releases.
        // Append after Any so all earlier intrinsic object IDs stay stable.
        self.work(12)?;
        let intrinsics = self
            .promises
            .intrinsics
            .as_ref()
            .ok_or_else(|| ScriptError::resource("Promise intrinsics missing"))?;
        let Value::Native(constructor) = &intrinsics.constructor else {
            return Err(ScriptError::resource("invalid Promise constructor"));
        };
        let owner = constructor
            .properties
            .map(NonZeroUsize::get)
            .filter(|id| *id < self.objects.len())
            .ok_or_else(|| ScriptError::resource("Promise metadata missing"))?;
        let bag = &self.objects[owner];
        if bag.values.len() != 10 || bag.order.len() != 10 || bag.order.capacity() < bag.order.len()
        {
            return Err(ScriptError::resource("invalid Promise metadata shape"));
        }
        let length = self.dom_proto_text("length")?;
        let name = self.dom_proto_text("name")?;
        let key = self.dom_proto_text("withResolvers")?;
        // Two metadata key copies and the display-name copy.
        self.work(3)?;
        let method = self.promise_intrinsic_method(
            "Promise.withResolvers",
            key.clone(),
            0,
            [&length, &name],
        )?;
        self.dom_proto_property(owner, key.into(), Property::data(method, true, false, true))
    }

    fn check_promise_install_state(&self) -> Result<()> {
        if !self.promises.promises.records.is_empty()
            || !self.promises.gates.records.is_empty()
            || !self.promises.callbacks.records.is_empty()
            || !self.promises.captures.records.is_empty()
            || !self.promises.jobs.records.is_empty()
            || !self.promises.finally_captures.records.is_empty()
            || !self.promises.all_captures.records.is_empty()
            || self.promises.intrinsics.is_some()
        {
            return Err(ScriptError::resource("Promise state already initialized"));
        }
        Ok(())
    }

    fn install_promise_intrinsics_body(&mut self, keys: [JsString; 2]) -> Result<()> {
        let [length, name] = keys;
        let pool = [
            length,
            name,
            self.dom_proto_text("Promise")?,
            self.dom_proto_text("then")?,
            self.dom_proto_text("resolve")?,
            self.dom_proto_text("reject")?,
            self.dom_proto_text("prototype")?,
            self.dom_proto_text("constructor")?,
            self.dom_proto_text("get [Symbol.species]")?,
            self.dom_proto_text("catch")?,
            self.dom_proto_text("finally")?,
            self.dom_proto_text("all")?,
            self.dom_proto_text("race")?,
            self.dom_proto_text("allSettled")?,
        ];
        // 30 string keys, 10 display names, 1 prototype tag, 2 constructor handles.
        self.work(43)?;
        let (symbol_comparisons, _) = tree(self.well_known_symbol_count());
        self.work(symbol_comparisons * (1 + "species".len() / 8))?;
        let species = self.well_known_key("species");
        self.work(symbol_comparisons * (1 + "toStringTag".len() / 8))?;
        let tag = self.well_known_key("toStringTag");
        let (prototype_comparisons, _) = tree(self.prototypes.len());
        self.work(prototype_comparisons * (1 + "Object".len() / 8))?;
        let object_prototype = self
            .prototypes
            .get("Object")
            .copied()
            .ok_or_else(|| ScriptError::resource("Promise Object prototype missing"))?;

        let constructor_bag =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 9)?;
        let prototype = self.dom_proto_object(Some(Value::Object(object_prototype)), 5)?;
        self.dom_proto_property(
            constructor_bag,
            pool[0].clone().into(),
            Property::data(Value::Number(1.0), false, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[1].clone().into(),
            Property::data(Value::String(pool[2].clone()), false, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[6].clone().into(),
            Property::data(Value::Object(prototype), false, false, false),
        )?;
        let constructor = self.promise_native(constructor_bag, GLOBAL, Value::Window)?;
        let keys = [&pool[0], &pool[1]];
        let then = self.promise_intrinsic_method("Promise.then", pool[3].clone(), 2, keys)?;
        let resolve = self.promise_intrinsic_method("Promise.resolve", pool[4].clone(), 1, keys)?;
        let reject = self.promise_intrinsic_method("Promise.reject", pool[5].clone(), 1, keys)?;
        let species_getter =
            self.promise_intrinsic_method("Promise.getSpecies", pool[8].clone(), 0, keys)?;
        let catch = self.promise_intrinsic_method("Promise.catch", pool[9].clone(), 1, keys)?;
        let finally =
            self.promise_intrinsic_method("Promise.finally", pool[10].clone(), 1, keys)?;
        let all = self.promise_intrinsic_method("Promise.all", pool[11].clone(), 1, keys)?;
        let race = self.promise_intrinsic_method("Promise.race", pool[12].clone(), 1, keys)?;
        let all_settled =
            self.promise_intrinsic_method("Promise.allSettled", pool[13].clone(), 1, keys)?;
        self.dom_proto_property(
            constructor_bag,
            pool[4].clone().into(),
            Property::data(resolve, true, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[5].clone().into(),
            Property::data(reject, true, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[11].clone().into(),
            Property::data(all, true, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[12].clone().into(),
            Property::data(race, true, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[13].clone().into(),
            Property::data(all_settled, true, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            species,
            Property {
                value: PropertyValue::Accessor {
                    get: species_getter,
                    set: Value::Undefined,
                },
                enumerable: false,
                configurable: true,
            },
        )?;
        self.dom_proto_property(
            prototype,
            pool[7].clone().into(),
            Property::data(constructor.clone(), true, false, true),
        )?;
        self.dom_proto_property(
            prototype,
            pool[3].clone().into(),
            Property::data(then, true, false, true),
        )?;
        self.dom_proto_property(
            prototype,
            pool[9].clone().into(),
            Property::data(catch, true, false, true),
        )?;
        self.dom_proto_property(
            prototype,
            pool[10].clone().into(),
            Property::data(finally, true, false, true),
        )?;
        self.dom_proto_property(
            prototype,
            tag,
            Property::data(Value::String(pool[2].clone()), false, false, true),
        )?;

        let (comparisons, height) = tree(self.environments[0].bindings.len());
        let levels = height + 1;
        self.work(8 + comparisons * (1 + GLOBAL.len() / 8) + 28 * levels + GLOBAL.len())?;
        let block =
            16 * (size_of::<String>() + size_of::<Binding>()) + 32 * size_of::<usize>() + 64;
        self.charge(32 + GLOBAL.len() + levels * block)?;
        let next_order = self
            .next_global_order
            .checked_add(1)
            .ok_or_else(|| ScriptError::resource("Promise global order overflow"))?;
        let mut key = String::new();
        key.try_reserve_exact(GLOBAL.len())
            .map_err(|_| ScriptError::resource("Promise global key allocation failed"))?;
        key.push_str(GLOBAL);
        let Entry::Vacant(entry) = self.environments[0].bindings.entry(key) else {
            return Err(ScriptError::resource("duplicate Promise global"));
        };
        entry.insert(Binding {
            value: constructor.clone(),
            accessor: None,
            mutable: true,
            initialized: true,
            strict_immutable: false,
            global_property: true,
            global_order: self.next_global_order,
            enumerable: false,
            deletable: true,
        });
        self.next_global_order = next_order;
        self.promises.intrinsics = Some(Intrinsics {
            constructor,
            prototype,
        });
        Ok(())
    }
}
