//! Branded, bounded synchronous Array/String iterators. No author-visible slots.
use super::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArrayKind {
    Values,
    Keys,
    Entries,
}
#[derive(Clone, Debug)]
enum IteratorState {
    Array {
        object: Option<Value>,
        index: u64,
        kind: ArrayKind,
    },
    String {
        text: Option<JsString>,
        index: usize,
    },
}
#[derive(Default)]
pub(super) struct State {
    entries: BTreeMap<usize, IteratorState>,
    prototypes: Option<(usize, usize)>,
    array_values: Option<Value>,
    iterator_key: Option<PropertyKey>,
    result_keys: Option<[JsString; 2]>,
    length_key: Option<JsString>,
}

const OBJECT_BYTES: usize =
    72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>();
const RESULT_BYTES: usize = OBJECT_BYTES + 2 * 256 + (5 + 4) * 4;
const MAX_LENGTH: u64 = 9_007_199_254_740_991;

fn tree_levels(entries: usize) -> usize {
    entries.checked_ilog2().map_or(0, |n| n as usize + 1)
}

impl Runtime {
    pub(super) fn install_iterator_intrinsics(&mut self) -> Result<()> {
        // Fixed keys and symbol references are saved once; next() never scans or
        // allocates a fresh UTF-16 "length"/"value"/"done" key.
        self.work(64)?;
        self.charge(512)?;
        let iterator_key = self.well_known_key("iterator");
        let tag_key = self.well_known_key("toStringTag");
        self.iterators.result_keys = Some(["value".into(), "done".into()]);
        self.iterators.length_key = Some("length".into());
        self.iterators.iterator_key = Some(iterator_key.clone());
        let Value::Object(common) = self.object_ordered([])? else {
            unreachable!()
        };
        let Value::Object(array) = self.object_ordered([])? else {
            unreachable!()
        };
        let Value::Object(string) = self.object_ordered([])? else {
            unreachable!()
        };
        self.objects[array].prototype = Some(Value::Object(common));
        self.objects[string].prototype = Some(Value::Object(common));
        self.iterators.prototypes = Some((array, string));
        for (full, name, owner, key) in [
            (
                "Iterator.self",
                "[Symbol.iterator]",
                common,
                iterator_key.clone(),
            ),
            (
                "Iterator.array.next",
                "next",
                array,
                PropertyKey::from("next"),
            ),
            (
                "Iterator.string.next",
                "next",
                string,
                PropertyKey::from("next"),
            ),
            (
                "Iterator.string.create",
                "[Symbol.iterator]",
                self.prototypes["String"],
                iterator_key.clone(),
            ),
        ] {
            let function = self.iterator_intrinsic(full, name)?;
            self.iterator_install_property(owner, key, function, true)?;
        }
        for (full, name) in [
            ("Iterator.array.values", "values"),
            ("Iterator.array.keys", "keys"),
            ("Iterator.array.entries", "entries"),
        ] {
            let function = self.iterator_intrinsic(full, name)?;
            let owner = self.prototypes["Array"];
            self.iterator_install_property(owner, name.into(), function.clone(), true)?;
            if name == "values" {
                self.iterator_install_property(
                    owner,
                    iterator_key.clone(),
                    function.clone(),
                    true,
                )?;
                self.iterators.array_values = Some(function);
            }
        }
        for (owner, name) in [(array, "Array Iterator"), (string, "String Iterator")] {
            self.charge(64 + name.len() * 2)?;
            self.iterator_install_property(
                owner,
                tag_key.clone(),
                Value::String(name.into()),
                false,
            )?;
        }
        Ok(())
    }

    fn iterator_intrinsic(&mut self, full: &str, name: &str) -> Result<Value> {
        self.work(1 + full.len() + name.len())?;
        // Native Rc/name, registry node and temporary metadata keys precede
        // intrinsic_function's separately charged ordinary function properties.
        self.charge(1024 + full.len() * 2 + name.len() * 4)?;
        self.intrinsic_function(full, name, 0)
    }
    fn iterator_install_property(
        &mut self,
        owner: usize,
        key: PropertyKey,
        value: Value,
        writable: bool,
    ) -> Result<()> {
        self.work(1 + key.byte_len() + 8 * tree_levels(self.objects[owner].values.len()))?;
        self.charge(256 + key.byte_len() * 2)?;
        self.objects[owner].insert_property(key, Property::data(value, writable, false, true));
        Ok(())
    }
    pub(super) fn install_arguments_iterator(&mut self, object: &Value) -> Result<()> {
        let key = self
            .iterators
            .iterator_key
            .clone()
            .ok_or_else(|| ScriptError::resource("iterator intrinsics unavailable"))?;
        let value = self
            .iterators
            .array_values
            .clone()
            .ok_or_else(|| ScriptError::resource("iterator intrinsics unavailable"))?;
        let Value::Object(id) = object else {
            return Err(ScriptError::type_error("arguments object required"));
        };
        self.iterator_install_property(*id, key, value, true)
    }

    fn iterator_preflight(&self, bytes: usize) -> Result<()> {
        if bytes > MAX_HEAP.saturating_sub(self.allocated) {
            Err(ScriptError::resource("script allocation limit exceeded"))
        } else {
            Ok(())
        }
    }
    fn iterator_lookup_work(&mut self) -> Result<()> {
        self.work(1 + 4 * tree_levels(self.iterators.entries.len()))
    }
    fn iterator_create(&mut self, state: IteratorState, prototype: usize) -> Result<Value> {
        let levels = tree_levels(self.iterators.entries.len());
        self.work(1 + 8 * (levels + 1))?;
        // A conservative whole B-tree node (11 key/value slots, links/header)
        // for each potentially split level plus new root, before insertion.
        let node_bytes =
            128 + 11 * (std::mem::size_of::<usize>() + std::mem::size_of::<IteratorState>());
        let map_bytes = node_bytes.saturating_mul(levels + 2);
        self.iterator_preflight(map_bytes.saturating_add(OBJECT_BYTES))?;
        self.charge(map_bytes)?;
        self.objects
            .try_reserve(1)
            .map_err(|_| ScriptError::resource("iterator object allocation failed"))?;
        let value = self.object_ordered([])?;
        let Value::Object(id) = value else {
            unreachable!()
        };
        self.objects[id].prototype = Some(Value::Object(prototype));
        self.iterators.entries.insert(id, state);
        Ok(value)
    }
    fn iterator_snapshot(&mut self, receiver: Value) -> Result<(usize, IteratorState)> {
        self.iterator_lookup_work()?;
        let Value::Object(id) = receiver else {
            return Err(ScriptError::type_error("incompatible iterator receiver"));
        };
        let state = self
            .iterators
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ScriptError::type_error("incompatible iterator receiver"))?;
        Ok((id, state))
    }
    fn iterator_result(&mut self, value: Value, done: bool) -> Result<Value> {
        self.work(2)?;
        self.iterator_result_reserved(value, done)
    }
    fn iterator_result_reserved(&mut self, value: Value, done: bool) -> Result<Value> {
        self.iterator_preflight(RESULT_BYTES)?;
        let [value_key, done_key] = self
            .iterators
            .result_keys
            .clone()
            .ok_or_else(|| ScriptError::resource("iterator intrinsics unavailable"))?;
        self.object_ordered([(value_key, value), (done_key, Value::Bool(done))])
    }
    pub(super) fn iterator_native(
        &mut self,
        method: &str,
        receiver: Value,
        _args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        match method {
            "self" => Ok(receiver),
            "array.values" | "array.keys" | "array.entries" => {
                let object = self.coerce_object(receiver)?;
                let kind = match method {
                    "array.keys" => ArrayKind::Keys,
                    "array.entries" => ArrayKind::Entries,
                    _ => ArrayKind::Values,
                };
                let (prototype, _) = self
                    .iterators
                    .prototypes
                    .ok_or_else(|| ScriptError::resource("iterator intrinsics unavailable"))?;
                self.iterator_create(
                    IteratorState::Array {
                        object: Some(object),
                        index: 0,
                        kind,
                    },
                    prototype,
                )
            }
            "array.next" => self.array_iterator_next(receiver, doc),
            "string.create" => {
                if matches!(receiver, Value::Null | Value::Undefined) {
                    return Err(ScriptError::type_error(
                        "string iterator receiver is nullish",
                    ));
                }
                let text = self.string_hint(receiver, doc)?;
                if text.len() > MAX_STRING {
                    return Err(ScriptError::resource("script string limit exceeded"));
                }
                let (_, prototype) = self
                    .iterators
                    .prototypes
                    .ok_or_else(|| ScriptError::resource("iterator intrinsics unavailable"))?;
                self.iterator_create(
                    IteratorState::String {
                        text: Some(text),
                        index: 0,
                    },
                    prototype,
                )
            }
            "string.next" => self.string_iterator_next(receiver),
            _ => Err(ScriptError::unsupported("unknown iterator intrinsic")),
        }
    }

    fn array_iterator_next(&mut self, receiver: Value, doc: &mut Document) -> Result<Value> {
        let (id, snapshot) = self.iterator_snapshot(receiver)?;
        let IteratorState::Array {
            object,
            index,
            kind,
        } = snapshot
        else {
            return Err(ScriptError::type_error("receiver is not an Array iterator"));
        };
        let Some(object) = object else {
            return self.iterator_result(Value::Undefined, true);
        };
        let key = self
            .iterators
            .length_key
            .clone()
            .ok_or_else(|| ScriptError::resource("iterator intrinsics unavailable"))?;
        let length = self.reduce_get(&object, &key, doc)?;
        let length = integer_or_infinity(self.number_value(length, doc)?)
            .clamp(0.0, MAX_LENGTH as f64) as u64;
        // Author length getters/conversion can reenter this same iterator. Only
        // write the named slot below; never restore our saved object/kind/state.
        self.iterator_lookup_work()?;
        let Some(IteratorState::Array {
            object: live_object,
            index: live_index,
            ..
        }) = self.iterators.entries.get_mut(&id)
        else {
            return Err(ScriptError::type_error("iterator state missing"));
        };
        if index >= length {
            *live_object = None;
            return self.iterator_result(Value::Undefined, true);
        }
        *live_index = index + 1;
        let value = if kind == ArrayKind::Keys {
            Value::Number(index as f64)
        } else {
            let key = self.reduce_index_key(index)?;
            let value = self.reduce_get(&object, &key, doc)?;
            if kind == ArrayKind::Values {
                value
            } else {
                // Pair construction and the result's two-key work are both
                // prepaid before any post-Get allocation.
                self.work(4)?;
                let staging = 32 + 2 * std::mem::size_of::<Value>();
                // Covers staging + array() retained storage + ordinary array
                // property bag + the later result object, before the pair Vec.
                self.iterator_preflight(staging * 2 + OBJECT_BYTES + RESULT_BYTES)?;
                self.charge(staging)?;
                let mut pair = Vec::new();
                pair.try_reserve_exact(2)
                    .map_err(|_| ScriptError::resource("iterator entry allocation failed"))?;
                pair.extend([Value::Number(index as f64), value]);
                let pair = self.array(pair)?;
                return self.iterator_result_reserved(pair, false);
            }
        };
        self.iterator_result(value, false)
    }
    fn string_iterator_next(&mut self, receiver: Value) -> Result<Value> {
        let (id, snapshot) = self.iterator_snapshot(receiver)?;
        let IteratorState::String { text, index } = snapshot else {
            return Err(ScriptError::type_error("receiver is not a String iterator"));
        };
        let Some(text) = text else {
            return self.iterator_result(Value::Undefined, true);
        };
        if index >= text.len() {
            self.iterator_lookup_work()?;
            let Some(IteratorState::String { text, .. }) = self.iterators.entries.get_mut(&id)
            else {
                return Err(ScriptError::type_error("iterator state missing"));
            };
            *text = None;
            return self.iterator_result(Value::Undefined, true);
        }
        // Two code-unit probes plus result-key work, before yielded storage.
        self.work(4)?;
        let first = text.units()[index];
        let count = if (0xd800..=0xdbff).contains(&first)
            && text
                .units()
                .get(index + 1)
                .is_some_and(|second| (0xdc00..=0xdfff).contains(second))
        {
            2
        } else {
            1
        };
        let text_bytes = 32 + count * 2;
        self.iterator_preflight(text_bytes + RESULT_BYTES)?;
        self.iterator_lookup_work()?;
        self.charge(text_bytes)?;
        let Some(IteratorState::String {
            index: live_index, ..
        }) = self.iterators.entries.get_mut(&id)
        else {
            return Err(ScriptError::type_error("iterator state missing"));
        };
        *live_index = index + count;
        let value = Value::String(JsString::from(&text.units()[index..index + count]));
        self.iterator_result_reserved(value, false)
    }
}
