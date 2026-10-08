//! Closed Number TypedArray intrinsic graph.
//! The caller owns an unexposed bootstrap Runtime; no author callback runs here.
use super::*;
use std::collections::btree_map::Entry;

#[cfg(test)]
mod views_tests;

pub(in crate::script) const METADATA_OBJECTS: usize = 40;
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
const LITERALS: [&str; 38] = [
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
    "at",
    "includes",
    "indexOf",
    "lastIndexOf",
    "fill",
    "reverse",
    "toReversed",
];

// The original ten-key shared owner (416 work) is replaced by the bulk-built
// twenty-key owner below. Thirty old small leaves retain their exact fees;
// the nine additional length/name leaves each cost58. No old entry tariff is reduced.
const MAP_WORK: usize = 31 * 16 + 92 * 18 + 718 - 416 + 9 * 58;
const FIXED_WORK: usize = 192;
// Includes nineteen application-order, nineteen stable-sort and nineteen
// dedup comparisons, each bounded by4+2*11. The tree still has three nodes.
// Each appended method adds6 staging,8 tree writes,4 order and3*26 comparisons.
// Retain the earlier64-work right-border allowance even though20 sorted entries
// fill11/root1/right8 directly and require no payload repair.
const SHARED_OWNER_WORK: usize = 1370 + 384 + 96 + 96 + 96;
const VIEW_FIXED_WORK: usize = 64;
const SEARCH_FIXED_WORK: usize = 64;
// Method invocation/store/retirement6, pooled handle lifetime2, parent/ID2,
// and fixed literal/count/order-index control6. Payloads and maps pay separately.
const FILL_FIXED_WORK: usize = 16;
const REVERSE_FIXED_WORK: usize = 16;
const TO_REVERSED_FIXED_WORK: usize = 16;

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
        self.work(
            MAP_WORK
                + FIXED_WORK
                + SHARED_OWNER_WORK
                + VIEW_FIXED_WORK
                + SEARCH_FIXED_WORK
                + FILL_FIXED_WORK
                + REVERSE_FIXED_WORK
                + TO_REVERSED_FIXED_WORK,
        )?;
        //39 fresh small leaves plus three nodes for the shared owner. Pay the
        // complete20-pair staging Vec and conservative48-pair sort scratch.
        self.charge(
            (METADATA_OBJECTS + 2) * leaf_bytes()
                + (20 + 48) * std::mem::size_of::<(PropertyKey, Property)>(),
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
            self.dom_proto_text(LITERALS[31])?,
            self.dom_proto_text(LITERALS[32])?,
            self.dom_proto_text(LITERALS[33])?,
            self.dom_proto_text(LITERALS[34])?,
            self.dom_proto_text(LITERALS[35])?,
            self.dom_proto_text(LITERALS[36])?,
            self.dom_proto_text(LITERALS[37])?,
        ];
        let shared_prototype = self.dom_proto_object(Some(Value::Object(object)), 20)?;
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
        // Append search bags after all33 historical bags. Preserve every old
        // creation-order entry, then append the four search properties.
        let at = self.typed_array_intrinsic_method_length("TypedArray.at", &pool, 31, 1)?;
        let includes =
            self.typed_array_intrinsic_method_length("TypedArray.includes", &pool, 32, 1)?;
        let index_of =
            self.typed_array_intrinsic_method_length("TypedArray.indexOf", &pool, 33, 1)?;
        let last_index_of =
            self.typed_array_intrinsic_method_length("TypedArray.lastIndexOf", &pool, 34, 1)?;
        // Preserve all37 old bags and append one shared fill native.
        let fill = self.typed_array_intrinsic_method_length("TypedArray.fill", &pool, 35, 1)?;
        // Append reverse after all38 existing bags without shifting old IDs.
        let reverse =
            self.typed_array_intrinsic_method_length("TypedArray.reverse", &pool, 36, 0)?;
        // Append after all39 old bags; preserve every existing native identity.
        let to_reversed =
            self.typed_array_intrinsic_method_length("TypedArray.toReversed", &pool, 37, 0)?;
        // Storage order is strings then the two symbols.
        self.typed_array_fill_shared(
            shared_prototype,
            [
                (
                    pool[31].clone().into(),
                    Property::data(at, true, false, true),
                ),
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
                    pool[35].clone().into(),
                    Property::data(fill, true, false, true),
                ),
                (
                    pool[32].clone().into(),
                    Property::data(includes, true, false, true),
                ),
                (
                    pool[33].clone().into(),
                    Property::data(index_of, true, false, true),
                ),
                (
                    pool[29].clone().into(),
                    Property::data(join, true, false, true),
                ),
                (
                    pool[8].clone().into(),
                    Property::data(keys, true, false, true),
                ),
                (
                    pool[34].clone().into(),
                    Property::data(last_index_of, true, false, true),
                ),
                (pool[0].clone().into(), getter(length)),
                (
                    pool[36].clone().into(),
                    Property::data(reverse, true, false, true),
                ),
                (
                    pool[28].clone().into(),
                    Property::data(subarray, true, false, true),
                ),
                (
                    pool[37].clone().into(),
                    Property::data(to_reversed, true, false, true),
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

    // Private to this closed installation. All39 small leaves and their100
    // entries are prepaid. The twenty-key shared owner uses its separate
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
        pool: &[JsString; 38],
        display: usize,
    ) -> Result<Value> {
        self.typed_array_intrinsic_method_length(full, pool, display, 0)
    }

    fn typed_array_intrinsic_method_length(
        &mut self,
        full: &str,
        pool: &[JsString; 38],
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
        entries: [(PropertyKey, Property); 20],
    ) -> Result<()> {
        let bag = &mut self.objects[owner];
        if !bag.values.is_empty()
            || !bag.order.is_empty()
            || bag.order.capacity() < 20
            || entries.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(ScriptError::resource(
                "unexpected TypedArray shared metadata shape",
            ));
        }
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(20)
            .map_err(|_| ScriptError::resource("TypedArray shared map allocation failed"))?;
        for index in [
            4, 1, 2, 3, 12, 19, 10, 17, 5, 18, 14, 9, 16, 0, 7, 8, 11, 6, 13, 15,
        ] {
            bag.order.push(entries[index].0.clone());
        }
        staged.extend(entries);
        // Strictly ascending20-element input:19 sort comparisons and19 dedup
        // checks; bulk construction allocates three nodes (11/root1/right8).
        // The existing64-work right-border allowance remains conservative.
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
