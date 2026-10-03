//! Bootstrap-only Node constant publication. Mutable bags and creation order
//! remain ordinary; only repeated tree insertion is replaced by sorted builds.
use super::*;

#[cfg(test)]
mod tests;

const COUNT: usize = 18;
const OLD_COUNT: usize = 3;
const FINAL_COUNT: usize = OLD_COUNT + COUNT;
const MAX_KEY_UNITS: usize = 41;
const SORTED: [usize; COUNT] = [1, 3, 7, 10, 8, 16, 15, 12, 14, 17, 13, 9, 0, 5, 4, 11, 6, 2];

// Rust 1.88/1.98: 20 actual-key comparisons each for validation, ascending
// stable-sort detection and dedup. Full UTF-16 comparison work, not byte / 8.
// Structure: setup; old traversal; entry transfers; bulk writes; overflow;
// right-border repair; three node headers; root/iterator transitions.
const MAP_WORK: usize = 3 * (FINAL_COUNT - 1) * (1 + MAX_KEY_UNITS)
    + 64
    + 4 * (OLD_COUNT + 1)
    + 6 * FINAL_COUNT
    + 8 * FINAL_COUNT
    + 16
    + 64
    + 8 * 3
    + 32;
const OWNER_WORK: usize = 2 * (32 + 4 * (OLD_COUNT + 1) + 6 * (1 + MAX_KEY_UNITS));

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
    if bag.values.len() != OLD_COUNT
        || bag.order.len() != OLD_COUNT
        || bag.order.capacity() < FINAL_COUNT + usize::from(prototype)
    {
        return false;
    }
    let sorted = if prototype {
        [
            ExpectedKey::Text("nodeValue"),
            ExpectedKey::Text("textContent"),
            ExpectedKey::Tag(tag),
        ]
    } else {
        [
            ExpectedKey::Text("length"),
            ExpectedKey::Text("name"),
            ExpectedKey::Text("prototype"),
        ]
    };
    let order = if prototype {
        [
            ExpectedKey::Tag(tag),
            ExpectedKey::Text("nodeValue"),
            ExpectedKey::Text("textContent"),
        ]
    } else {
        [
            ExpectedKey::Text("length"),
            ExpectedKey::Text("name"),
            ExpectedKey::Text("prototype"),
        ]
    };
    bag.values.keys().zip(&sorted).all(|(key, expected)| {
        key.as_string()
            .is_none_or(|text| text.len() <= MAX_KEY_UNITS)
            && matches_key(key, expected)
    }) && bag.order.iter().zip(&order).all(|(key, expected)| {
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
        let scratch_and_nodes = add(mul(2 * 48, pair)?, mul(2 * 3, node)?)?;
        // Separate key stores, two order-vector writes and final publication.
        // Initial order blocks/old leaves stay charged. No refunds or growth.
        self.work(2 * MAP_WORK + 2 * COUNT + 2 * 4 * COUNT + 8)?;
        self.charge(add(payload, scratch_and_nodes)?)?;

        let mut keys = self.dom_proto_vector::<PropertyKey>(COUNT)?;
        for (name, _) in NODE_CONSTANTS {
            keys.push(self.dom_proto_text(name)?.into());
        }
        let mut prototype_entries = self.dom_proto_vector(FINAL_COUNT)?;
        let mut constructor_entries = self.dom_proto_vector(FINAL_COUNT)?;
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
