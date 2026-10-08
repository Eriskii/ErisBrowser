//! Closed Number TypedArray intrinsic graph.
//! The caller owns an unexposed bootstrap Runtime; no author callback runs here.
use super::*;
use std::collections::btree_map::Entry;

#[cfg(test)]
mod views_tests;

pub(in crate::script) const METADATA_OBJECTS: usize = 33;
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
const LITERALS: [&str; 31] = [
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
    "subarray",
    "join",
    "toString",
];

// The original ten-key shared owner (416 work) is replaced by the bulk-built
// thirteen-key owner below. Thirty old small leaves retain their exact fees;
// the two new length/name leaves each cost58. No old entry tariff is reduced.
const MAP_WORK: usize = 31 * 16 + 92 * 18 + 718 - 416 + 2 * 58;
const FIXED_WORK: usize = 192;
// Includes twelve application-order, twelve stable-sort and twelve dedup
// comparisons, each bounded by4+2*11. The shared tree has three nodes.
const SHARED_OWNER_WORK: usize = 1370;
const VIEW_FIXED_WORK: usize = 64;

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
        self.work(MAP_WORK + FIXED_WORK + SHARED_OWNER_WORK + VIEW_FIXED_WORK)?;
        //32 fresh small leaves plus three nodes for the shared owner. Pay the
        // complete13-pair staging Vec and conservative48-pair sort scratch.
        self.charge(
            (METADATA_OBJECTS + 2) * leaf_bytes()
                + (13 + 48) * std::mem::size_of::<(PropertyKey, Property)>(),
        )?;
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
            self.dom_proto_text(LITERALS[28])?,
            self.dom_proto_text(LITERALS[29])?,
            self.dom_proto_text(LITERALS[30])?,
        ];
        let shared_prototype = self.dom_proto_object(Some(Value::Object(object)), 13)?;
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
        // Append the two new bags after all31 historical bags; the old TypedArray
        // constructor/prototype/getter IDs remain unchanged.
        let subarray =
            self.typed_array_intrinsic_method_length("TypedArray.subarray", &pool, 28, 2)?;
        let join = self.typed_array_intrinsic_method_length("TypedArray.join", &pool, 29, 1)?;
        let to_string = self.typed_array_to_string_alias(&pool[30])?;
        // Preserve the old ten-key creation-order subsequence, then append
        // subarray,join,toString. Storage order is strings then the two symbols.
        self.typed_array_fill_shared(
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
                    pool[29].clone().into(),
                    Property::data(join, true, false, true),
                ),
                (
                    pool[8].clone().into(),
                    Property::data(keys, true, false, true),
                ),
                (pool[0].clone().into(), getter(length)),
                (
                    pool[28].clone().into(),
                    Property::data(subarray, true, false, true),
                ),
                (
                    pool[30].clone().into(),
                    Property::data(to_string, true, false, true),
                ),
                (
                    pool[9].clone().into(),
                    Property::data(values.clone(), true, false, true),
                ),
                (iterator, Property::data(values, true, false, true)),
                (tag, getter(tag_getter)),
            ],
        )?;
        // No fallible operation follows these private saved-handle writes.
        // Whole finish_bootstrap can still fail later and discard the realm.
        self.typed_arrays.intrinsic = Some(shared);
        self.typed_arrays.prototype = Some(shared_prototype);
        self.typed_arrays.constructors = constructors;
        self.typed_arrays.prototypes = prototypes;
        Ok(())
    }

    // Private to this closed installation. All32 small leaves and their86
    // entries are prepaid. The thirteen-key shared owner uses its separate
    // bulk builder. Each order Vec already has its exact final capacity.
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
        pool: &[JsString; 31],
        display: usize,
    ) -> Result<Value> {
        self.typed_array_intrinsic_method_length(full, pool, display, 0)
    }

    fn typed_array_intrinsic_method_length(
        &mut self,
        full: &str,
        pool: &[JsString; 31],
        display: usize,
        length: usize,
    ) -> Result<Value> {
        let bag = self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        self.typed_array_fill_intrinsic(
            bag,
            [
                (
                    pool[0].clone().into(),
                    Property::data(Value::Number(length as f64), false, false, true),
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

    fn typed_array_to_string_alias(&mut self, name: &JsString) -> Result<Value> {
        // The saved Array prototype is an arena handle. This closed bootstrap
        // lookup cannot invoke authors or follow a mutable global constructor.
        // At most128 private keys imply at most33 B-tree comparisons; each
        // compares at most eight UTF-16 units (4+2*8 work). The remaining44
        // covers handle checks, native-name validation and Rc retirement.
        self.work(16)?;
        let failure = || ScriptError::resource("TypedArray Array.toString alias missing");
        let array = self.array_prototype.ok_or_else(failure)?;
        let owner = *self.array_properties.get(array).ok_or_else(failure)?;
        if self
            .objects
            .get(owner)
            .is_none_or(|bag| bag.values.len() > 128)
        {
            return Err(failure());
        }
        self.work(688)?;
        let bag = &self.objects[owner];
        let Some(Property {
            value:
                PropertyValue::Data {
                    value: Value::Native(native),
                    ..
                },
            ..
        }) = bag.values.get(&PropertyKey::String(name.clone()))
        else {
            return Err(failure());
        };
        if native.name != "Array.toString" {
            return Err(failure());
        }
        Ok(Value::Native(native.clone()))
    }

    fn typed_array_fill_shared(
        &mut self,
        owner: usize,
        entries: [(PropertyKey, Property); 13],
    ) -> Result<()> {
        let bag = &mut self.objects[owner];
        if !bag.values.is_empty()
            || !bag.order.is_empty()
            || bag.order.capacity() < 13
            || entries.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(ScriptError::resource(
                "unexpected TypedArray shared metadata shape",
            ));
        }
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(13)
            .map_err(|_| ScriptError::resource("TypedArray shared map allocation failed"))?;
        for index in [3, 0, 1, 2, 7, 12, 6, 10, 4, 11, 8, 5, 9] {
            bag.order.push(entries[index].0.clone());
        }
        staged.extend(entries);
        // Strictly ascending13-element input:12 sort comparisons and12 dedup
        // checks; bulk construction allocates three nodes. Its right-border
        // repair moves four pairs, covered by the separate64-work envelope.
        bag.values = staged.into_iter().collect();
        Ok(())
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
