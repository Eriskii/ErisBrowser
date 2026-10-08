//! Closed Number TypedArray intrinsic graph.
//! The caller owns an unexposed bootstrap Runtime; no author callback runs here.
use super::*;
use std::collections::btree_map::Entry;

pub(in crate::script) const METADATA_OBJECTS: usize = 31;
pub(in crate::script) const GLOBAL_COUNT: usize = 10;
pub(in crate::script) const GLOBAL_NAME_MAX: usize = {
    let mut maximum = 0;
    let mut index = 0;
    while index < Kind::ALL.len() {
        let length = Kind::ALL[index].name().len();
        if length > maximum {
            maximum = length;
        }
        index += 1;
    }
    maximum
};

// The indices are the accepted Kind order, not the global staging order.
// These are bounded stack arrays; there is no temporary sorting Vec.
const LITERALS: [&str; 28] = [
    "length",
    "name",
    "prototype",
    "BYTES_PER_ELEMENT",
    "constructor",
    "buffer",
    "byteLength",
    "byteOffset",
    "keys",
    "values",
    "entries",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "TypedArray",
    "get buffer",
    "get byteLength",
    "get byteOffset",
    "get length",
    "get [Symbol.toStringTag]",
    "get [Symbol.species]",
];

// New-site logical accounting. The 31 maps start empty, contain <=10 entries,
// and are filled in strictly increasing PropertyKey order. No split or suffix
// movement occurs. 130 examined-prefix comparisons cost718 in total.
const MAP_WORK: usize = 31 * 16 + 92 * 18 + 718;
const FIXED_WORK: usize = 192;

fn leaf_bytes() -> usize {
    16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}

fn getter(value: Value) -> Property {
    Property {
        value: PropertyValue::Accessor {
            get: value,
            set: Value::Undefined,
        },
        enumerable: false,
        configurable: true,
    }
}

impl Runtime {
    /// Only called on the raw private realm, after native discovery and self,
    /// immediately before the existing joined DOM global publication.
    pub(in crate::script) fn initialize_typed_array_intrinsics(&mut self) -> Result<()> {
        // All fixed-array assembly, map entry/order operations and eventual
        // local/map handle retirement are admitted before constructing them.
        // State storage itself is charged once by BootstrapBuilder, not here.
        self.work(MAP_WORK + FIXED_WORK)?;
        self.charge(METADATA_OBJECTS * leaf_bytes())?;
        if self.typed_arrays.intrinsic.is_some() || self.typed_arrays.prototype.is_some() {
            return Err(ScriptError::resource(
                "TypedArray intrinsics initialized twice",
            ));
        }
        self.work(30)?; // Object.prototype search22 plus setup8.
        let object = self
            .prototypes
            .get("Object")
            .copied()
            .ok_or_else(|| ScriptError::resource("TypedArray Object.prototype missing"))?;
        self.work(75)?; // Three well-known searches:30 +15 +30.
        let species = self.well_known_key("species");
        let iterator = self.well_known_key("iterator");
        let tag = self.well_known_key("toStringTag");
        // Explicit fallible construction avoids a hidden Vec or a second pool.
        let pool = [
            self.dom_proto_text(LITERALS[0])?,
            self.dom_proto_text(LITERALS[1])?,
            self.dom_proto_text(LITERALS[2])?,
            self.dom_proto_text(LITERALS[3])?,
            self.dom_proto_text(LITERALS[4])?,
            self.dom_proto_text(LITERALS[5])?,
            self.dom_proto_text(LITERALS[6])?,
            self.dom_proto_text(LITERALS[7])?,
            self.dom_proto_text(LITERALS[8])?,
            self.dom_proto_text(LITERALS[9])?,
            self.dom_proto_text(LITERALS[10])?,
            self.dom_proto_text(LITERALS[11])?,
            self.dom_proto_text(LITERALS[12])?,
            self.dom_proto_text(LITERALS[13])?,
            self.dom_proto_text(LITERALS[14])?,
            self.dom_proto_text(LITERALS[15])?,
            self.dom_proto_text(LITERALS[16])?,
            self.dom_proto_text(LITERALS[17])?,
            self.dom_proto_text(LITERALS[18])?,
            self.dom_proto_text(LITERALS[19])?,
            self.dom_proto_text(LITERALS[20])?,
            self.dom_proto_text(LITERALS[21])?,
            self.dom_proto_text(LITERALS[22])?,
            self.dom_proto_text(LITERALS[23])?,
            self.dom_proto_text(LITERALS[24])?,
            self.dom_proto_text(LITERALS[25])?,
            self.dom_proto_text(LITERALS[26])?,
            self.dom_proto_text(LITERALS[27])?,
        ];
        let shared_prototype = self.dom_proto_object(Some(Value::Object(object)), 10)?;
        let shared_bag =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 4)?;
        let shared = self.typed_array_intrinsic_native("TypedArray", shared_bag)?;

        let buffer = self.typed_array_intrinsic_method("TypedArray.getBuffer", &pool, 22)?;
        let byte_length =
            self.typed_array_intrinsic_method("TypedArray.getByteLength", &pool, 23)?;
        let byte_offset =
            self.typed_array_intrinsic_method("TypedArray.getByteOffset", &pool, 24)?;
        let length = self.typed_array_intrinsic_method("TypedArray.getLength", &pool, 25)?;
        let tag_getter = self.typed_array_intrinsic_method("TypedArray.getTag", &pool, 26)?;
        let species_getter =
            self.typed_array_intrinsic_method("TypedArray.getSpecies", &pool, 27)?;
        let keys = self.typed_array_intrinsic_method("TypedArray.keys", &pool, 8)?;
        let values = self.typed_array_intrinsic_method("TypedArray.values", &pool, 9)?;
        let entries = self.typed_array_intrinsic_method("TypedArray.entries", &pool, 10)?;

        // Storage order is sorted; observable creation order is length,name,
        // prototype,@@species. Shared constructor length is0, concrete is3.
        self.typed_array_fill_intrinsic(
            shared_bag,
            [
                (
                    pool[0].clone().into(),
                    Property::data(Value::Number(0.0), false, false, true),
                ),
                (
                    pool[1].clone().into(),
                    Property::data(Value::String(pool[21].clone()), false, false, true),
                ),
                (
                    pool[2].clone().into(),
                    Property::data(Value::Object(shared_prototype), false, false, false),
                ),
                (species, getter(species_getter)),
            ],
            [0, 1, 2, 3],
        )?;
        // Sorted strings then symbols (iterator identity5 < toStringTag13).
        // Creation order: constructor, buffer, byteLength, byteOffset, length,
        // @@toStringTag, keys, values, entries, @@iterator.
        self.typed_array_fill_intrinsic(
            shared_prototype,
            [
                (pool[5].clone().into(), getter(buffer)),
                (pool[6].clone().into(), getter(byte_length)),
                (pool[7].clone().into(), getter(byte_offset)),
                (
                    pool[4].clone().into(),
                    Property::data(shared.clone(), true, false, true),
                ),
                (
                    pool[10].clone().into(),
                    Property::data(entries, true, false, true),
                ),
                (
                    pool[8].clone().into(),
                    Property::data(keys, true, false, true),
                ),
                (pool[0].clone().into(), getter(length)),
                (
                    pool[9].clone().into(),
                    Property::data(values.clone(), true, false, true),
                ),
                (iterator, Property::data(values, true, false, true)),
                (tag, getter(tag_getter)),
            ],
            [3, 0, 1, 2, 6, 9, 5, 7, 4, 8],
        )?;

        let mut constructors: [Option<Value>; 10] = Default::default();
        let mut prototypes = [None; 10];
        for kind in Kind::ALL {
            let index = kind.index();
            let prototype = self.dom_proto_object(Some(Value::Object(shared_prototype)), 2)?;
            let bag = self.dom_proto_object(Some(shared.clone()), 4)?;
            let constructor = self.typed_array_intrinsic_native(kind.name(), bag)?;
            let width = Value::Number(kind.width() as f64);
            self.typed_array_fill_intrinsic(
                bag,
                [
                    (
                        pool[3].clone().into(),
                        Property::data(width.clone(), false, false, false),
                    ),
                    (
                        pool[0].clone().into(),
                        Property::data(Value::Number(3.0), false, false, true),
                    ),
                    (
                        pool[1].clone().into(),
                        Property::data(Value::String(pool[11 + index].clone()), false, false, true),
                    ),
                    (
                        pool[2].clone().into(),
                        Property::data(Value::Object(prototype), false, false, false),
                    ),
                ],
                [1, 2, 3, 0],
            )?;
            self.typed_array_fill_intrinsic(
                prototype,
                [
                    (
                        pool[3].clone().into(),
                        Property::data(width, false, false, false),
                    ),
                    (
                        pool[4].clone().into(),
                        Property::data(constructor.clone(), true, false, true),
                    ),
                ],
                [1, 0],
            )?;
            constructors[index] = Some(constructor);
            prototypes[index] = Some(prototype);
        }
        // No fallible operation follows these private saved-handle writes.
        // Whole finish_bootstrap can still fail later and discard the realm.
        self.typed_arrays.intrinsic = Some(shared);
        self.typed_arrays.prototype = Some(shared_prototype);
        self.typed_arrays.constructors = constructors;
        self.typed_arrays.prototypes = prototypes;
        Ok(())
    }

    // Private to this closed installation. The caller prepays all31 leaves and
    // all92 fixed entries; only fresh bags and literal order permutations reach
    // this helper. The separate order Vec already has its exact capacity.
    fn typed_array_fill_intrinsic<const N: usize>(
        &mut self,
        owner: usize,
        entries: [(PropertyKey, Property); N],
        creation_order: [usize; N],
    ) -> Result<()> {
        let bag = &mut self.objects[owner];
        for index in creation_order {
            bag.order.push(entries[index].0.clone());
        }
        for (key, property) in entries {
            let Entry::Vacant(entry) = bag.values.entry(key) else {
                return Err(ScriptError::resource(
                    "duplicate TypedArray intrinsic member",
                ));
            };
            entry.insert(property);
        }
        Ok(())
    }

    fn typed_array_intrinsic_method(
        &mut self,
        full: &str,
        pool: &[JsString; 28],
        display: usize,
    ) -> Result<Value> {
        let bag = self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        self.typed_array_fill_intrinsic(
            bag,
            [
                (
                    pool[0].clone().into(),
                    Property::data(Value::Number(0.0), false, false, true),
                ),
                (
                    pool[1].clone().into(),
                    Property::data(Value::String(pool[display].clone()), false, false, true),
                ),
            ],
            [0, 1],
        )?;
        self.typed_array_intrinsic_native(full, bag)
    }

    fn typed_array_intrinsic_native(&mut self, full: &str, bag: usize) -> Result<Value> {
        self.work(8 + full.len())?;
        self.charge(std::mem::size_of::<Native>() + 32 + full.len())?;
        let properties = std::num::NonZeroUsize::new(bag)
            .ok_or_else(|| ScriptError::resource("TypedArray intrinsic bag ID is zero"))?;
        let mut name = String::new();
        name.try_reserve_exact(full.len())
            .map_err(|_| ScriptError::resource("TypedArray native name allocation failed"))?;
        name.push_str(full);
        Ok(Value::Native(Rc::new(Native {
            name,
            properties: Some(properties),
            receiver: Value::Undefined,
        })))
    }
}
