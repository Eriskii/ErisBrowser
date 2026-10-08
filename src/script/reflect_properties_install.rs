//! Closed Reflect metadata extension, installed only in a private bootstrap realm.
use super::*;
use std::collections::btree_map::Entry;

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 9;
const METHODS: [(&str, &str, usize); 9] = [
    ("has", "Reflect.has", 2),
    ("get", "Reflect.get", 2),
    ("set", "Reflect.set", 3),
    ("deleteProperty", "Reflect.deleteProperty", 2),
    (
        "getOwnPropertyDescriptor",
        "Reflect.getOwnPropertyDescriptor",
        2,
    ),
    ("getPrototypeOf", "Reflect.getPrototypeOf", 1),
    ("setPrototypeOf", "Reflect.setPrototypeOf", 2),
    ("isExtensible", "Reflect.isExtensible", 1),
    ("preventExtensions", "Reflect.preventExtensions", 1),
];

pub(super) fn is_method(name: &str) -> bool {
    matches!(
        name,
        "Reflect.has"
            | "Reflect.get"
            | "Reflect.set"
            | "Reflect.deleteProperty"
            | "Reflect.getOwnPropertyDescriptor"
            | "Reflect.getPrototypeOf"
            | "Reflect.setPrototypeOf"
            | "Reflect.isExtensible"
            | "Reflect.preventExtensions"
    )
}

fn leaf_bytes() -> usize {
    16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}

impl Runtime {
    pub(super) fn initialize_reflect_property_intrinsics(&mut self) -> Result<()> {
        // New-site logical schedule: nine sorted two-entry leaves522;
        // old map/order guard306; sorted14-entry owner map1395; control108.
        // The owner map bulk builder allocates three nodes and repairs its
        // right border (three pairs), covered by the included64 repair work.
        // An additional30 pays the real @@toStringTag identity lookup.
        self.work(522 + 306 + 1395 + 108 + 30)?;
        let mut capacity = self.environments[0].bindings.len().saturating_add(1) / 2;
        let mut height = 1usize;
        while capacity >= 6 {
            capacity /= 6;
            height += 1;
        }
        self.work(8 + self.environments[0].bindings.len().min(11 * height) * 8)?;
        let Some(Binding {
            value: Value::Object(owner),
            ..
        }) = self.environments[0].bindings.get("Reflect")
        else {
            return Err(ScriptError::resource("Reflect owner missing"));
        };
        let owner = *owner;
        let tag = self.well_known_key("toStringTag");
        let bag = &self.objects[owner];
        if bag.values.len() != 5 || bag.order.len() != 5 {
            return Err(ScriptError::resource("unexpected Reflect metadata shape"));
        }
        let keys: [&PropertyKey; 5] = {
            let mut keys = bag.values.keys();
            std::array::from_fn(|_| keys.next().unwrap())
        };
        for (key, literal) in
            keys[..4]
                .iter()
                .zip(["apply", "construct", "defineProperty", "ownKeys"])
        {
            if !key.as_string().is_some_and(|key| {
                key.units()
                    .iter()
                    .copied()
                    .eq(literal.bytes().map(u16::from))
            }) {
                return Err(ScriptError::resource("unexpected Reflect metadata key"));
            }
        }
        if keys[4] != &tag
            || [3, 0, 1, 2, 4]
                .into_iter()
                .zip(&bag.order)
                .any(|(index, key)| keys[index] != key)
        {
            return Err(ScriptError::resource("unexpected Reflect creation order"));
        }
        // Exact order/staging capacities plus a conservative48-pair sort
        // scratch envelope. The direct, unadvanced Vec IntoIter below permits
        // allocation reuse by BTreeMap::from_iter on the pinned Rust versions.
        self.charge(
            12 * leaf_bytes()
                + 14 * std::mem::size_of::<PropertyKey>()
                + (14 + 48) * std::mem::size_of::<(PropertyKey, Property)>(),
        )?;
        let mut order = Vec::new();
        order
            .try_reserve_exact(14)
            .map_err(|_| ScriptError::resource("Reflect order allocation failed"))?;
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(14)
            .map_err(|_| ScriptError::resource("Reflect map staging allocation failed"))?;
        order.extend(self.objects[owner].order.iter().cloned());
        let pool = [
            self.dom_proto_text("length")?,
            self.dom_proto_text("name")?,
            self.dom_proto_text("has")?,
            self.dom_proto_text("get")?,
            self.dom_proto_text("set")?,
            self.dom_proto_text("deleteProperty")?,
            self.dom_proto_text("getOwnPropertyDescriptor")?,
            self.dom_proto_text("getPrototypeOf")?,
            self.dom_proto_text("setPrototypeOf")?,
            self.dom_proto_text("isExtensible")?,
            self.dom_proto_text("preventExtensions")?,
        ];
        let mut methods: [Option<(PropertyKey, Property)>; 9] = Default::default();
        for (index, (_, full, length)) in METHODS.into_iter().enumerate() {
            let id = self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
            let bag = &mut self.objects[id];
            for (key, property) in [
                (
                    PropertyKey::from(pool[0].clone()),
                    Property::data(Value::Number(length as f64), false, false, true),
                ),
                (
                    PropertyKey::from(pool[1].clone()),
                    Property::data(Value::String(pool[index + 2].clone()), false, false, true),
                ),
            ] {
                bag.order.push(key.clone());
                let Entry::Vacant(entry) = bag.values.entry(key) else {
                    return Err(ScriptError::resource("duplicate Reflect function metadata"));
                };
                entry.insert(property);
            }
            self.work(8 + full.len())?;
            self.charge(std::mem::size_of::<Native>() + 32 + full.len())?;
            let mut name = String::new();
            name.try_reserve_exact(full.len())
                .map_err(|_| ScriptError::resource("Reflect native name allocation failed"))?;
            name.push_str(full);
            let properties = std::num::NonZeroUsize::new(id)
                .ok_or_else(|| ScriptError::resource("Reflect metadata ID is zero"))?;
            let function = Value::Native(Rc::new(Native {
                name,
                properties: Some(properties),
                receiver: Value::Undefined,
            }));
            let key = PropertyKey::from(pool[index + 2].clone());
            order.push(key.clone());
            methods[index] = Some((key, Property::data(function, true, false, true)));
        }
        // All admissions precede consumption of the old map. This initializer
        // has no author callbacks and failure discards the unexposed realm.
        let mut old = std::mem::take(&mut self.objects[owner].values).into_iter();
        staged.push(old.next().unwrap()); // apply
        staged.push(old.next().unwrap()); // construct
        staged.push(old.next().unwrap()); // defineProperty
        for index in [3, 1, 4, 5, 0, 7] {
            staged.push(methods[index].take().unwrap());
        }
        staged.push(old.next().unwrap()); // ownKeys
        for index in [8, 2, 6] {
            staged.push(methods[index].take().unwrap());
        }
        staged.push(old.next().unwrap()); // @@toStringTag
        if old.next().is_some() || staged.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
            return Err(ScriptError::resource("Reflect map is not uniquely sorted"));
        }
        self.objects[owner].values = staged.into_iter().collect();
        self.objects[owner].order = order;
        Ok(())
    }

    pub(super) fn reflect_property_call_preflight(&mut self, name: &str) -> Result<()> {
        // Paid before the machine copies the temporary native call name.
        self.work(4 + name.len())?;
        self.charge(32 + name.len())
    }
}
