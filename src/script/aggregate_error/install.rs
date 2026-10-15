//! Late AggregateError metadata installation with prepaid map publication.
use super::*;
use crate::script::promises::intrinsic_tree as tree;
use std::collections::btree_map::Entry;
use std::mem::size_of;

#[cfg(test)]
mod tests;

const GLOBAL: &str = "AggregateError";

fn search(count: usize, length: usize) -> usize {
    tree(count).0.saturating_mul(1 + length / 8)
}

fn node_bytes<K, V>() -> usize {
    16 * (size_of::<K>() + size_of::<V>()) + 32 * size_of::<usize>() + 64
}

fn invariant() -> ScriptError {
    ScriptError::resource("AggregateError installer state is invalid")
}

fn owned_name(runtime: &mut Runtime) -> Result<String> {
    runtime.work(1 + GLOBAL.len())?;
    runtime.charge(GLOBAL.len())?;
    let mut name = String::new();
    name.try_reserve_exact(GLOBAL.len())
        .map_err(|_| ScriptError::resource("AggregateError map key allocation failed"))?;
    name.push_str(GLOBAL);
    Ok(name)
}

impl Runtime {
    pub(in crate::script) fn install_aggregate_error_intrinsics(&mut self) -> Result<()> {
        // Eight prerequisite checks and eleven local cleanup roles: seven text
        // handles, saved/new constructors and two owned publication keys. The
        // initial arena reservation already admitted both metadata object slots.
        self.work(19)?;
        self.work(search(self.environments[0].bindings.len(), "Error".len()))?;
        // Pay the retained copy before borrowing the map, so one paid lookup
        // supplies the checks and copy without a borrow across mutable work.
        self.work(1)?;
        let error_constructor = {
            let binding = self.environments[0]
                .bindings
                .get("Error")
                .ok_or_else(invariant)?;
            if binding.accessor.is_some() {
                return Err(invariant());
            }
            let Value::Native(constructor) = &binding.value else {
                return Err(invariant());
            };
            Value::Native(constructor.clone())
        };
        self.work(search(self.prototypes.len(), "Error".len()))?;
        let error_prototype = self
            .prototypes
            .get("Error")
            .copied()
            .ok_or_else(invariant)?;
        if error_prototype >= self.objects.len() {
            return Err(invariant());
        }
        self.work(search(self.prototypes.len(), GLOBAL.len()))?;
        if self.prototypes.contains_key(GLOBAL) {
            return Err(invariant());
        }
        self.work(search(self.native_properties.len(), GLOBAL.len()))?;
        if self.native_properties.contains_key(GLOBAL) {
            return Err(invariant());
        }
        self.work(search(self.environments[0].bindings.len(), GLOBAL.len()))?;
        if self.environments[0].bindings.contains_key(GLOBAL) {
            return Err(invariant());
        }

        let pool = [
            self.dom_proto_text("length")?,
            self.dom_proto_text("name")?,
            self.dom_proto_text("prototype")?,
            self.dom_proto_text("constructor")?,
            self.dom_proto_text("message")?,
            self.dom_proto_text(GLOBAL)?,
            self.dom_proto_text("")?,
        ];
        // Nine immutable text copies and the constructor's back-reference.
        self.work(10)?;
        let prototype = self.dom_proto_object(Some(Value::Object(error_prototype)), 3)?;
        let constructor_bag = self.dom_proto_object(Some(error_constructor), 3)?;
        self.dom_proto_property(
            constructor_bag,
            pool[0].clone().into(),
            Property::data(Value::Number(2.0), false, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[1].clone().into(),
            Property::data(Value::String(pool[5].clone()), false, false, true),
        )?;
        self.dom_proto_property(
            constructor_bag,
            pool[2].clone().into(),
            Property::data(Value::Object(prototype), false, false, false),
        )?;
        let constructor = self.promise_native(constructor_bag, GLOBAL, Value::Window)?;
        self.dom_proto_property(
            prototype,
            pool[3].clone().into(),
            Property::data(constructor.clone(), true, false, true),
        )?;
        self.dom_proto_property(
            prototype,
            pool[1].clone().into(),
            Property::data(Value::String(pool[5].clone()), true, false, true),
        )?;
        self.dom_proto_property(
            prototype,
            pool[4].clone().into(),
            Property::data(Value::String(pool[6].clone()), true, false, true),
        )?;

        let (comparisons, height) = tree(self.prototypes.len());
        let levels = height + 1;
        self.work(8 + comparisons * (1 + GLOBAL.len() / 8) + 28 * levels)?;
        self.charge(levels * node_bytes::<&'static str, usize>())?;
        let Entry::Vacant(entry) = self.prototypes.entry(GLOBAL) else {
            return Err(invariant());
        };
        entry.insert(prototype);

        let key = owned_name(self)?;
        let (comparisons, height) = tree(self.native_properties.len());
        let levels = height + 1;
        self.work(8 + comparisons * (1 + GLOBAL.len() / 8) + 28 * levels)?;
        self.charge(32 + levels * node_bytes::<String, usize>())?;
        let Entry::Vacant(entry) = self.native_properties.entry(key) else {
            return Err(invariant());
        };
        entry.insert(constructor_bag);

        let key = owned_name(self)?;
        let (comparisons, height) = tree(self.environments[0].bindings.len());
        let levels = height + 1;
        self.work(8 + comparisons * (1 + GLOBAL.len() / 8) + 28 * levels)?;
        self.charge(32 + levels * node_bytes::<String, Binding>())?;
        let next_order = self
            .next_global_order
            .checked_add(1)
            .ok_or_else(invariant)?;
        let Entry::Vacant(entry) = self.environments[0].bindings.entry(key) else {
            return Err(invariant());
        };
        entry.insert(Binding {
            value: constructor,
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
        Ok(())
    }
}
