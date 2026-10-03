//! Bootstrap-only Node constant publication. Mutable bags and creation order
//! remain ordinary; only repeated tree insertion is replaced by sorted builds.
use super::*;

#[cfg(test)]
mod tests;

const COUNT: usize = 18;
const PROTOTYPE_OLD: usize = 7;
const CONSTRUCTOR_OLD: usize = 3;
const PROTOTYPE_FINAL: usize = PROTOTYPE_OLD + COUNT;
const CONSTRUCTOR_FINAL: usize = CONSTRUCTOR_OLD + COUNT;
const MAX_KEY_UNITS: usize = 41;
const SORTED: [usize; COUNT] = [1, 3, 7, 10, 8, 16, 15, 12, 14, 17, 13, 9, 0, 5, 4, 11, 6, 2];

// Rust 1.88/1.98: three (N-1)-comparison passes at full UTF-16 cost.
// N=25 has height one and leaf overflows at items 12/24: initial leaf,
// new root and second leaf, then a third leaf. N=21 still needs three nodes.
// Right-border repair allocates nothing; full sort scratch remains separate.
const fn map_work(old: usize, count: usize, overflows: usize, nodes: usize) -> usize {
    3 * (count - 1) * (1 + MAX_KEY_UNITS)
        + 64
        + 4 * (old + 1)
        + 6 * count
        + 8 * count
        + 16 * overflows
        + 64
        + 8 * nodes
        + 32
}
const fn owner_work(old: usize) -> usize {
    32 + 4 * (old + 1) + 2 * old * (1 + MAX_KEY_UNITS)
}
const MAP_WORK: usize = map_work(PROTOTYPE_OLD, PROTOTYPE_FINAL, 2, 4)
    + map_work(CONSTRUCTOR_OLD, CONSTRUCTOR_FINAL, 1, 3);
const OWNER_WORK: usize = owner_work(PROTOTYPE_OLD) + owner_work(CONSTRUCTOR_OLD);

fn invalid() -> ScriptError {
    ScriptError::resource("invalid Node constant bootstrap staging")
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(invalid)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(invalid)
}

enum ExpectedKey<'a> {
    Text(&'static str),
    Tag(&'a PropertyKey),
}
fn matches_key(key: &PropertyKey, expected: &ExpectedKey<'_>) -> bool {
    match (key, expected) {
        (PropertyKey::String(text), ExpectedKey::Text(name)) => {
            text.len() == name.len() && text.units().iter().copied().eq(name.bytes().map(u16::from))
        }
        (PropertyKey::Symbol(_), ExpectedKey::Tag(tag)) => key == *tag,
        _ => false,
    }
}
fn owner_matches(bag: &ScriptObject, prototype: bool, tag: &PropertyKey) -> bool {
    let old = if prototype {
        PROTOTYPE_OLD
    } else {
        CONSTRUCTOR_OLD
    };
    let capacity = if prototype {
        PROTOTYPE_FINAL + 1
    } else {
        CONSTRUCTOR_FINAL
    };
    if bag.values.len() != old || bag.order.len() != old || bag.order.capacity() < capacity {
        return false;
    }
    let sorted: &[ExpectedKey<'_>] = if prototype {
        &[
            ExpectedKey::Text("contains"),
            ExpectedKey::Text("hasChildNodes"),
            ExpectedKey::Text("isSameNode"),
            ExpectedKey::Text("nodeValue"),
            ExpectedKey::Text("normalize"),
            ExpectedKey::Text("textContent"),
            ExpectedKey::Tag(tag),
        ]
    } else {
        &[
            ExpectedKey::Text("length"),
            ExpectedKey::Text("name"),
            ExpectedKey::Text("prototype"),
        ]
    };
    let order: &[ExpectedKey<'_>] = if prototype {
        &[
            ExpectedKey::Tag(tag),
            ExpectedKey::Text("nodeValue"),
            ExpectedKey::Text("textContent"),
            ExpectedKey::Text("hasChildNodes"),
            ExpectedKey::Text("normalize"),
            ExpectedKey::Text("isSameNode"),
            ExpectedKey::Text("contains"),
        ]
    } else {
        &[
            ExpectedKey::Text("length"),
            ExpectedKey::Text("name"),
            ExpectedKey::Text("prototype"),
        ]
    };
    bag.values.keys().zip(sorted).all(|(key, expected)| {
        key.as_string()
            .is_none_or(|text| text.len() <= MAX_KEY_UNITS)
            && matches_key(key, expected)
    }) && bag.order.iter().zip(order).all(|(key, expected)| {
        key.as_string()
            .is_none_or(|text| text.len() <= MAX_KEY_UNITS)
            && matches_key(key, expected)
    })
}

fn completed_map(
    old: BTreeMap<PropertyKey, Property>,
    mut entries: Vec<(PropertyKey, Property)>,
) -> Result<BTreeMap<PropertyKey, Property>> {
    for entry in old {
        entries.push(entry);
    }
    if entries.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(invalid());
    }
    // Preserve the direct, unadvanced IntoIter: both pinned Vec specializations
    // reuse this admitted buffer. Sort scratch and fresh nodes are charged too.
    Ok(entries.into_iter().collect())
}

impl Runtime {
    pub(super) fn install_node_constants(
        &mut self,
        prototype: usize,
        properties: usize,
        tag: &PropertyKey,
    ) -> Result<()> {
        self.node_constants_with_order(prototype, properties, tag, &SORTED)
    }

    fn node_constants_with_order(
        &mut self,
        prototype: usize,
        properties: usize,
        tag: &PropertyKey,
        sorted: &[usize],
    ) -> Result<()> {
        self.work(32)?;
        if NODE_CONSTANTS.len() != COUNT || sorted.len() != COUNT {
            return Err(invalid());
        }
        self.work(4 * COUNT)?;
        let mut key_bytes = 0;
        for ((name, _), index) in NODE_CONSTANTS.iter().zip(sorted) {
            if name.len() > MAX_KEY_UNITS || *index >= COUNT {
                return Err(invalid());
            }
            key_bytes = add(key_bytes, name.len())?;
        }
        self.work(key_bytes)?;
        if NODE_CONSTANTS.iter().any(|(name, _)| !name.is_ascii()) {
            return Err(invalid());
        }
        self.work(OWNER_WORK)?;
        if prototype == properties
            || !matches!(tag, PropertyKey::Symbol(_))
            || !self
                .objects
                .get(prototype)
                .is_some_and(|bag| owner_matches(bag, true, tag))
            || !self
                .objects
                .get(properties)
                .is_some_and(|bag| owner_matches(bag, false, tag))
        {
            return Err(invalid());
        }

        self.work(32)?; // checked planning, independent of string comparisons
        let pair = std::mem::size_of::<(PropertyKey, Property)>();
        let node = add(
            mul(
                16,
                add(
                    std::mem::size_of::<PropertyKey>(),
                    std::mem::size_of::<Property>(),
                )?,
            )?,
            add(mul(32, std::mem::size_of::<usize>())?, 64)?,
        )?;
        let payload = mul(2, add(mul(COUNT, 256)?, mul(4, key_bytes)?)?)?;
        let scratch_and_nodes = add(mul(2 * 48, pair)?, mul(4 + 3, node)?)?;
        // Separate key stores, two order-vector writes and final publication.
        // Initial order blocks/old leaves stay charged. No refunds or growth.
        self.work(MAP_WORK + 2 * COUNT + 2 * 4 * COUNT + 8)?;
        self.charge(add(payload, scratch_and_nodes)?)?;

        let mut keys = self.dom_proto_vector::<PropertyKey>(COUNT)?;
        for (name, _) in NODE_CONSTANTS {
            keys.push(self.dom_proto_text(name)?.into());
        }
        let mut prototype_entries = self.dom_proto_vector(PROTOTYPE_FINAL)?;
        let mut constructor_entries = self.dom_proto_vector(CONSTRUCTOR_FINAL)?;
        for index in sorted {
            let value = NODE_CONSTANTS[*index].1;
            for entries in [&mut prototype_entries, &mut constructor_entries] {
                entries.push((
                    keys[*index].clone(),
                    Property::data(Value::Number(value as f64), false, true, false),
                ));
            }
        }

        // All quota checks and fallible reserves have finished. An ordering
        // invariant failure from here discards this private initializer; it
        // does not promise reusable-map rollback or publish a usable Runtime.
        let old_prototype = std::mem::take(&mut self.objects[prototype].values);
        let old_constructor = std::mem::take(&mut self.objects[properties].values);
        let prototype_values = completed_map(old_prototype, prototype_entries)?;
        let constructor_values = completed_map(old_constructor, constructor_entries)?;
        for key in keys {
            self.objects[prototype].order.push(key.clone());
            self.objects[properties].order.push(key);
        }
        self.objects[prototype].values = prototype_values;
        self.objects[properties].values = constructor_values;
        Ok(())
    }
}
