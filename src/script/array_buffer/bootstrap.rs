//! Prepaid, closed ArrayBuffer metadata installation.
//! Registry publication is sorted independently of JavaScript creation order.
use super::*;
use std::collections::btree_map::Entry;
use std::ops::Bound::{Excluded, Included};

#[cfg(test)]
mod tests;

const INSTALL_WORK: usize = 8_150;
const GUARD_AND_LOOKUP_WORK: usize = 128 + 174;
const GAP_WORK: usize = 256;
const REGISTRY_BEFORE: usize = 178;
const REGISTRY_ORDER: [usize; 10] = [1, 4, 2, 3, 0, 5, 6, 9, 7, 8];
const TEXT: [&str; 17] = [
    "name",
    "length",
    "isView",
    "get byteLength",
    "get maxByteLength",
    "get resizable",
    "get detached",
    "resize",
    "slice",
    "transfer",
    "transferToFixedLength",
    "get [Symbol.species]",
    "byteLength",
    "maxByteLength",
    "resizable",
    "detached",
    "ArrayBuffer",
];

struct Method {
    full: &'static str,
    display: usize,
    // None denotes the original well-known species symbol.
    key: Option<usize>,
    length: usize,
    constructor: bool,
    getter: bool,
}

const METHODS: [Method; 10] = [
    Method {
        full: "ArrayBuffer.isView",
        display: 2,
        key: Some(2),
        length: 1,
        constructor: true,
        getter: false,
    },
    Method {
        full: "ArrayBuffer.getByteLength",
        display: 3,
        key: Some(12),
        length: 0,
        constructor: false,
        getter: true,
    },
    Method {
        full: "ArrayBuffer.getMaxByteLength",
        display: 4,
        key: Some(13),
        length: 0,
        constructor: false,
        getter: true,
    },
    Method {
        full: "ArrayBuffer.getResizable",
        display: 5,
        key: Some(14),
        length: 0,
        constructor: false,
        getter: true,
    },
    Method {
        full: "ArrayBuffer.getDetached",
        display: 6,
        key: Some(15),
        length: 0,
        constructor: false,
        getter: true,
    },
    Method {
        full: "ArrayBuffer.resize",
        display: 7,
        key: Some(7),
        length: 1,
        constructor: false,
        getter: false,
    },
    Method {
        full: "ArrayBuffer.slice",
        display: 8,
        key: Some(8),
        length: 2,
        constructor: false,
        getter: false,
    },
    Method {
        full: "ArrayBuffer.transfer",
        display: 9,
        key: Some(9),
        length: 0,
        constructor: false,
        getter: false,
    },
    Method {
        full: "ArrayBuffer.transferToFixedLength",
        display: 10,
        key: Some(10),
        length: 0,
        constructor: false,
        getter: false,
    },
    Method {
        full: "ArrayBuffer.species",
        display: 11,
        key: None,
        length: 0,
        constructor: true,
        getter: true,
    },
];

fn invariant() -> ScriptError {
    ScriptError::resource("ArrayBuffer metadata bootstrap invariant violated")
}

fn node_bytes<K, V>() -> Option<usize> {
    std::mem::size_of::<K>()
        .checked_add(std::mem::size_of::<V>())?
        .checked_mul(16)?
        .checked_add(32usize.checked_mul(std::mem::size_of::<usize>())?)?
        .checked_add(64)
}

fn install_bytes() -> Result<usize> {
    let base = 72usize
        .checked_add(std::mem::size_of::<Option<AbortSlot>>())
        .and_then(|n| n.checked_add(std::mem::size_of::<Option<f64>>()))
        .ok_or_else(invariant)?;
    let terms = [
        Some(2_281), // All pooled text and both copies of the native names.
        base.checked_mul(10),
        node_bytes::<PropertyKey, Property>().and_then(|n| n.checked_mul(10)),
        std::mem::size_of::<PropertyKey>().checked_mul(72),
        std::mem::size_of::<Native>()
            .checked_add(32)
            .and_then(|n| n.checked_mul(11)),
        node_bytes::<String, usize>().and_then(|n| n.checked_mul(3)),
    ];
    terms.into_iter().try_fold(0usize, |sum, term| {
        sum.checked_add(term.ok_or_else(invariant)?)
            .ok_or_else(invariant)
    })
}

fn exact_map_keys(object: &ScriptObject, expected: &[&str]) -> bool {
    object.values.len() == expected.len()
        && object.values.keys().zip(expected).all(|(key, text)| {
            key.as_string().is_some_and(|key| {
                key.len() == text.len()
                    && key.units().iter().copied().eq(text.bytes().map(u16::from))
            })
        })
}

// The caller prepays the entire fixed plan before any of these constructors.
// These mirror the existing typed text/object/native helpers without debiting
// twice. Every growable payload is reserved before it is filled.
fn text_prepaid(text: &str) -> Result<JsString> {
    let mut units = Vec::new();
    units
        .try_reserve_exact(text.len())
        .map_err(|_| ScriptError::resource("ArrayBuffer metadata text allocation failed"))?;
    units.extend(text.bytes().map(u16::from));
    Ok(units.into())
}

fn string_prepaid(text: &str) -> Result<String> {
    let mut name = String::new();
    name.try_reserve_exact(text.len())
        .map_err(|_| ScriptError::resource("ArrayBuffer metadata name allocation failed"))?;
    name.push_str(text);
    Ok(name)
}

fn native_prepaid(full: &str, receiver: Value) -> Result<Value> {
    Ok(Value::Native(Rc::new(Native {
        name: string_prepaid(full)?,
        properties: None,
        receiver,
    })))
}

fn bag_prepaid(
    function_prototype: usize,
    pool: &[JsString; 17],
    row: &Method,
) -> Result<ScriptObject> {
    let mut bag = ScriptObject {
        prototype: Some(Value::Function(function_prototype)),
        ..ScriptObject::default()
    };
    bag.order
        .try_reserve_exact(4)
        .map_err(|_| ScriptError::resource("ArrayBuffer metadata order allocation failed"))?;
    // Preserve the original creation order, while avoiding map suffix shifts.
    bag.order.push(pool[0].clone().into());
    bag.order.push(pool[1].clone().into());
    for (key, value) in [
        (pool[1].clone().into(), Value::Number(row.length as f64)),
        (
            pool[0].clone().into(),
            Value::String(pool[row.display].clone()),
        ),
    ] {
        let Entry::Vacant(entry) = bag.values.entry(key) else {
            return Err(invariant());
        };
        entry.insert(Property::data(value, false, false, true));
    }
    Ok(bag)
}

fn owner_insert_prepaid(
    owner: &mut ScriptObject,
    key: PropertyKey,
    property: Property,
) -> Result<()> {
    if owner.order.len() == owner.order.capacity() {
        let capacity = owner
            .order
            .capacity()
            .checked_mul(2)
            .ok_or_else(invariant)?;
        let additional = capacity
            .checked_sub(owner.order.len())
            .ok_or_else(invariant)?;
        owner
            .order
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("ArrayBuffer owner order allocation failed"))?;
    }
    let Entry::Vacant(entry) = owner.values.entry(key.clone()) else {
        return Err(invariant());
    };
    owner.order.push(key);
    entry.insert(property);
    Ok(())
}

impl Runtime {
    pub(in crate::script) fn install_array_buffer_intrinsics(&mut self) -> Result<()> {
        self.work(GUARD_AND_LOOKUP_WORK)?;
        if self.native_properties.len() != REGISTRY_BEFORE
            || self.prototypes.len() != 25
            || self.array_buffers.intrinsic.is_some()
            || self.array_buffers.prototype.is_some()
            || !self.array_buffers.records.is_empty()
            || self.functions.get(self.function_prototype).is_none()
            || self
                .objects
                .len()
                .checked_add(10)
                .is_none_or(|n| n > self.objects.capacity())
        {
            return Err(invariant());
        }
        let constructor = *self
            .native_properties
            .get("ArrayBuffer")
            .ok_or_else(invariant)?;
        let prototype = *self.prototypes.get("ArrayBuffer").ok_or_else(invariant)?;
        let ctor = self.objects.get(constructor).ok_or_else(invariant)?;
        let proto = self.objects.get(prototype).ok_or_else(invariant)?;
        if constructor == prototype
            || ctor.order.len() != 3
            || proto.order.len() != 1
            || ctor.order.capacity() != 4
            || proto.order.capacity() != 4
            || !exact_map_keys(ctor, &["length", "name", "prototype"])
            || !exact_map_keys(proto, &["constructor"])
        {
            return Err(invariant());
        }
        self.work(GAP_WORK)?;
        if self
            .native_properties
            .range::<str, _>((Included("ArrayBuffer."), Excluded("ArrayBuffer/")))
            .next()
            .is_some()
        {
            return Err(invariant());
        }
        self.work(INSTALL_WORK - GUARD_AND_LOOKUP_WORK - GAP_WORK)?;
        self.charge(install_bytes()?)?;

        let pool = [
            text_prepaid(TEXT[0])?,
            text_prepaid(TEXT[1])?,
            text_prepaid(TEXT[2])?,
            text_prepaid(TEXT[3])?,
            text_prepaid(TEXT[4])?,
            text_prepaid(TEXT[5])?,
            text_prepaid(TEXT[6])?,
            text_prepaid(TEXT[7])?,
            text_prepaid(TEXT[8])?,
            text_prepaid(TEXT[9])?,
            text_prepaid(TEXT[10])?,
            text_prepaid(TEXT[11])?,
            text_prepaid(TEXT[12])?,
            text_prepaid(TEXT[13])?,
            text_prepaid(TEXT[14])?,
            text_prepaid(TEXT[15])?,
            text_prepaid(TEXT[16])?,
        ];
        let saved = native_prepaid("ArrayBuffer", Value::Window)?;
        let species = self.well_known_key("species");
        let tag = self.well_known_key("toStringTag");
        let mut registry: [Option<(String, usize)>; 10] = std::array::from_fn(|_| None);
        for (ordinal, row) in METHODS.iter().enumerate() {
            let bag = bag_prepaid(self.function_prototype, &pool, row)?;
            let id = self.objects.len();
            self.objects.push(bag);
            let function = native_prepaid(row.full, Value::Undefined)?;
            registry[ordinal] = Some((string_prepaid(row.full)?, id));
            let property = if row.getter {
                Property {
                    value: PropertyValue::Accessor {
                        get: function,
                        set: Value::Undefined,
                    },
                    enumerable: false,
                    configurable: true,
                }
            } else {
                Property::data(function, true, false, true)
            };
            let key = match row.key {
                Some(key) => pool[key].clone().into(),
                None => species.clone(),
            };
            let owner = if row.constructor {
                constructor
            } else {
                prototype
            };
            owner_insert_prepaid(&mut self.objects[owner], key, property)?;
        }
        // The prepaid empty gap and this static ascending permutation imply
        // <=2 leaf splits and <=1 parent split. At178..188 entries a full
        // level-two root is impossible (minimum431 entries). Search visits are
        // paid separately:13 reached insertion levels and3 typed node blocks.
        for ordinal in REGISTRY_ORDER {
            let (key, id) = registry[ordinal].take().ok_or_else(invariant)?;
            let Entry::Vacant(entry) = self.native_properties.entry(key) else {
                return Err(invariant());
            };
            entry.insert(id);
        }
        owner_insert_prepaid(
            &mut self.objects[prototype],
            tag,
            Property::data(Value::String(pool[16].clone()), false, false, true),
        )?;
        self.array_buffers.intrinsic = Some(saved);
        self.array_buffers.prototype = Some(prototype);
        Ok(())
    }
}
