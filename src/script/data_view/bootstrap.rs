//! Closed, fully admitted DataView metadata installation with direct native bags.
use super::*;
use std::collections::btree_map::Entry;
use std::num::NonZeroUsize;

#[cfg(test)]
mod tests;

const INSTALL_WORK: usize = 4_553;
const GUARD_AND_LOOKUP_WORK: usize = 128 + 108;
const REGISTRY_BEFORE: usize = 188;
const OWNER_ORDER: [usize; 23] = [
    0, 1, 2, 21, 15, 17, 19, 7, 11, 3, 9, 13, 5, 16, 18, 20, 8, 12, 4, 10, 14, 6, 22,
];
const TEXT: [&str; 27] = [
    "name",
    "length",
    "get buffer",
    "get byteLength",
    "get byteOffset",
    "getInt8",
    "setInt8",
    "getUint8",
    "setUint8",
    "getInt16",
    "setInt16",
    "getUint16",
    "setUint16",
    "getInt32",
    "setInt32",
    "getUint32",
    "setUint32",
    "getFloat16",
    "setFloat16",
    "getFloat32",
    "setFloat32",
    "getFloat64",
    "setFloat64",
    "buffer",
    "byteLength",
    "byteOffset",
    "DataView",
];

struct Method {
    full: &'static str,
    display: usize,
    key: usize,
    length: usize,
    getter: bool,
}

const METHODS: [Method; 21] = [
    Method {
        full: "DataView.getBuffer",
        display: 2,
        key: 23,
        length: 0,
        getter: true,
    },
    Method {
        full: "DataView.getByteLength",
        display: 3,
        key: 24,
        length: 0,
        getter: true,
    },
    Method {
        full: "DataView.getByteOffset",
        display: 4,
        key: 25,
        length: 0,
        getter: true,
    },
    Method {
        full: "DataView.getInt8",
        display: 5,
        key: 5,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setInt8",
        display: 6,
        key: 6,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getUint8",
        display: 7,
        key: 7,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setUint8",
        display: 8,
        key: 8,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getInt16",
        display: 9,
        key: 9,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setInt16",
        display: 10,
        key: 10,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getUint16",
        display: 11,
        key: 11,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setUint16",
        display: 12,
        key: 12,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getInt32",
        display: 13,
        key: 13,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setInt32",
        display: 14,
        key: 14,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getUint32",
        display: 15,
        key: 15,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setUint32",
        display: 16,
        key: 16,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getFloat16",
        display: 17,
        key: 17,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setFloat16",
        display: 18,
        key: 18,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getFloat32",
        display: 19,
        key: 19,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setFloat32",
        display: 20,
        key: 20,
        length: 2,
        getter: false,
    },
    Method {
        full: "DataView.getFloat64",
        display: 21,
        key: 21,
        length: 1,
        getter: false,
    },
    Method {
        full: "DataView.setFloat64",
        display: 22,
        key: 22,
        length: 2,
        getter: false,
    },
];

fn invariant() -> ScriptError {
    ScriptError::resource("DataView metadata bootstrap invariant violated")
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
        Some(2_688), // Twenty-seven pooled UTF-16 strings, including Vec/Rc copies.
        base.checked_mul(21),
        node_bytes::<PropertyKey, Property>().and_then(|n| n.checked_mul(24)),
        std::mem::size_of::<PropertyKey>().checked_mul(65),
        std::mem::size_of::<Native>()
            .checked_add(32)
            .and_then(|n| n.checked_mul(21))
            .and_then(|n| n.checked_add(382)),
        std::mem::size_of::<(PropertyKey, Property)>().checked_mul(71),
    ];
    terms.into_iter().try_fold(0usize, |sum, term| {
        sum.checked_add(term.ok_or_else(invariant)?)
            .ok_or_else(invariant)
    })
}

fn literal(key: &PropertyKey, expected: &str) -> bool {
    key.as_string().is_some_and(|key| {
        key.len() == expected.len()
            && key
                .units()
                .iter()
                .copied()
                .eq(expected.bytes().map(u16::from))
    })
}

// All payload construction follows the complete fixed work and heap admission.
fn text_prepaid(text: &str) -> Result<JsString> {
    let mut units = Vec::new();
    units
        .try_reserve_exact(text.len())
        .map_err(|_| ScriptError::resource("DataView metadata text allocation failed"))?;
    units.extend(text.bytes().map(u16::from));
    Ok(units.into())
}

fn bag_prepaid(
    function_prototype: usize,
    pool: &[JsString; 27],
    row: &Method,
) -> Result<ScriptObject> {
    let mut bag = ScriptObject {
        prototype: Some(Value::Function(function_prototype)),
        ..ScriptObject::default()
    };
    bag.order
        .try_reserve_exact(2)
        .map_err(|_| ScriptError::resource("DataView metadata order allocation failed"))?;
    bag.order.push(pool[0].clone().into());
    bag.order.push(pool[1].clone().into());
    // Ascending leaf insertion has no suffix shifts; creation order is separate.
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

fn native_prepaid(full: &str, id: usize) -> Result<Value> {
    let properties = Some(NonZeroUsize::new(id).ok_or_else(invariant)?);
    let mut name = String::new();
    name.try_reserve_exact(full.len())
        .map_err(|_| ScriptError::resource("DataView metadata name allocation failed"))?;
    name.push_str(full);
    Ok(Value::Native(Rc::new(Native {
        name,
        properties,
        receiver: Value::Undefined,
    })))
}

impl Runtime {
    pub(in crate::script) fn install_data_view_intrinsics(&mut self) -> Result<()> {
        self.work(GUARD_AND_LOOKUP_WORK)?;
        if self.native_properties.len() != REGISTRY_BEFORE
            || self.prototypes.len() != 25
            || self.data_views.prototype.is_some()
            || !self.data_views.records.is_empty()
            || self.functions.get(self.function_prototype).is_none()
            || self
                .objects
                .len()
                .checked_add(21)
                .is_none_or(|n| n > self.objects.capacity())
        {
            return Err(invariant());
        }
        let prototype = *self.prototypes.get("DataView").ok_or_else(invariant)?;
        let owner = self.objects.get(prototype).ok_or_else(invariant)?;
        if owner.values.len() != 1
            || owner.order.len() != 1
            || owner.order.capacity() != 4
            || !literal(&owner.order[0], "constructor")
        {
            return Err(invariant());
        }
        let (constructor_key, constructor) =
            owner.values.first_key_value().ok_or_else(invariant)?;
        if !literal(constructor_key, "constructor")
            || constructor.enumerable
            || !constructor.configurable
        {
            return Err(invariant());
        }
        let PropertyValue::Data {
            value: Value::Native(native),
            writable: true,
        } = &constructor.value
        else {
            return Err(invariant());
        };
        if native.name != "DataView"
            || native.receiver != Value::Window
            || native.properties.is_some()
        {
            return Err(invariant());
        }
        let constructor_key = constructor_key.clone();
        let constructor = constructor.clone();
        self.work(INSTALL_WORK - GUARD_AND_LOOKUP_WORK)?;
        self.charge(install_bytes()?)?;

        let mut order = Vec::new();
        order
            .try_reserve_exact(23)
            .map_err(|_| ScriptError::resource("DataView owner order allocation failed"))?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(23)
            .map_err(|_| ScriptError::resource("DataView owner staging allocation failed"))?;
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
            text_prepaid(TEXT[17])?,
            text_prepaid(TEXT[18])?,
            text_prepaid(TEXT[19])?,
            text_prepaid(TEXT[20])?,
            text_prepaid(TEXT[21])?,
            text_prepaid(TEXT[22])?,
            text_prepaid(TEXT[23])?,
            text_prepaid(TEXT[24])?,
            text_prepaid(TEXT[25])?,
            text_prepaid(TEXT[26])?,
        ];
        let tag = self.well_known_key("toStringTag");
        let mut properties: [Option<(PropertyKey, Property)>; 23] = std::array::from_fn(|_| None);
        order.push(constructor_key.clone());
        properties[21] = Some((constructor_key, constructor));
        for (ordinal, row) in METHODS.iter().enumerate() {
            let bag = bag_prepaid(self.function_prototype, &pool, row)?;
            let id = self.objects.len();
            self.objects.push(bag);
            let function = native_prepaid(row.full, id)?;
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
            let key: PropertyKey = pool[row.key].clone().into();
            order.push(key.clone());
            properties[ordinal] = Some((key, property));
        }
        order.push(tag.clone());
        properties[22] = Some((
            tag,
            Property::data(Value::String(pool[26].clone()), false, false, true),
        ));
        for ordinal in OWNER_ORDER {
            entries.push(properties[ordinal].take().ok_or_else(invariant)?);
        }
        // This literal sorted run pays 22 Ord and 22 separate Eq comparisons.
        // The unadvanced Vec iterator retains the existing collect specialization.
        // Three B-tree nodes form full 11-key leaves under a one-key root.
        let values = entries.into_iter().collect();
        self.objects[prototype].values = values;
        self.objects[prototype].order = order;
        self.data_views.prototype = Some(prototype);
        Ok(())
    }
}
