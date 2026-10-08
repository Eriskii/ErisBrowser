//! Bounded string-key snapshots and for-in's reached visited-name operations.
//! Live descriptor/Get/binding lookup accounting is separate from this module.
use super::*;

#[cfg(test)]
mod for_in_tests;
#[cfg(test)]
mod ordinary_snapshot_tests;
#[cfg(test)]
mod singleton_tests;
#[cfg(test)]
mod tests;

// The first name of each length needs no separately allocated search tree.
// A tree is published only after a second distinct, live name is admitted.
#[cfg_attr(test, derive(Clone, Debug, PartialEq, Eq))]
enum NameBucket {
    Single(JsString),
    Tree(BTreeMap<JsString, ()>),
}

// Equality requires equal UTF-16 lengths. The outer tree compares only fixed-
// width lengths; full text comparisons occur within the reached length bucket.
// This set's ordering never controls enumeration order.
#[derive(Default)]
#[cfg_attr(test, derive(Clone, Debug, PartialEq, Eq))]
pub(super) struct VisitedNames {
    buckets: BTreeMap<usize, NameBucket>,
}

type ForInReader = fn(&mut Runtime, &Value, &JsString) -> Result<Option<Property>>;

// A snapshot's object and its descriptor route cannot be replaced separately.
// Only a successful no-record lookup can choose the ordinary route. Object IDs
// are never reused, and TypedArray constructors brand a fresh shell before any
// callback can expose it. This caches no descriptor, prototype or buffer state.
pub(super) struct ForInObject {
    value: Value,
    reader: ForInReader,
}

impl ForInObject {
    pub(super) fn value(&self) -> &Value {
        &self.value
    }
}

const VISITED_NAME_BYTES: usize =
    16 * std::mem::size_of::<JsString>() + 32 * std::mem::size_of::<usize>() + 64;
const VISITED_BUCKET_BYTES: usize = 16
    * (std::mem::size_of::<usize>() + std::mem::size_of::<NameBucket>())
    + 32 * std::mem::size_of::<usize>()
    + 64;

// Stored nonnumeric names are unique: ScriptObject::insert_property appends
// only absent names, and remove updates both the map and creation-order list.
// Only virtual numeric keys/length can overlap stored keys. Integer ordinals
// preserve creation order without comparing or copying retained UTF-16 names.
struct OwnKey {
    kind: u8,
    index: u32,
    ordinal: usize,
    text: JsString,
}

impl OwnKey {
    fn rank(&self) -> (u8, u32, usize) {
        (self.kind, self.index, self.ordinal)
    }
}

fn push_key(keys: &mut Vec<OwnKey>, text: JsString) {
    let (kind, index) = json_array_index(&text).map_or((1, 0), |index| (0, index));
    keys.push(OwnKey {
        kind,
        index,
        ordinal: keys.len(),
        text,
    });
}

// Rust 1.88/1.98 B-trees have B=6: at most 11 comparisons per node.
// Height h needs at least 2*6^h-1 keys. This loop is bounded by usize bits.
// The whole tree cannot compare more keys than it contains on one search.
fn tree_bound(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut nodes = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        nodes += 1;
    }
    (count.min(nodes.saturating_mul(11)), nodes)
}

fn overflow() -> ScriptError {
    ScriptError::resource("own-key allocation size overflow")
}

impl Runtime {
    fn own_key_buffer<T>(&mut self, count: usize) -> Result<Vec<T>> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(overflow)?;
        self.charge(bytes)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("own-key buffer allocation failed"))?;
        Ok(result)
    }

    pub(super) fn own_key_values(&mut self, count: usize) -> Result<Vec<Value>> {
        self.own_key_buffer(count)
    }

    pub(super) fn own_key_descriptors(
        &mut self,
        count: usize,
    ) -> Result<Vec<(PropertyKey, PropertyDescriptor)>> {
        self.own_key_buffer(count)
    }

    pub(super) fn own_keys(&mut self, receiver: &Value) -> Result<Vec<JsString>> {
        if let Some(length) = self.typed_array_own_length(receiver)? {
            return self.typed_array_own_strings(receiver, length);
        }
        self.ordinary_own_keys(receiver)
    }

    pub(super) fn for_in_snapshot(&mut self, value: Value) -> Result<(ForInObject, Vec<JsString>)> {
        let length = self.typed_array_own_length(&value)?;
        // Select the route once, using the result of the original snapshot
        // search. Some(0) is still a branded view and always uses live checks.
        self.work(4)?;
        let ordinary =
            length.is_none() && matches!(value, Value::Object(_)) && self.typed_array_has_records();
        let reader: ForInReader = if ordinary {
            // Admit proof construction, moved identity and eventual retirement.
            self.work(8)?;
            Self::for_in_proven_ordinary_property
        } else {
            Self::for_in_own_property
        };
        let keys = match length {
            Some(length) => self.typed_array_own_strings(&value, length)?,
            None => self.ordinary_own_keys(&value)?,
        };
        Ok((ForInObject { value, reader }, keys))
    }

    fn ordinary_own_keys(&mut self, receiver: &Value) -> Result<Vec<JsString>> {
        if receiver == &Value::Window {
            return self.window_own_keys();
        }
        if matches!(receiver, Value::Document) {
            return Err(ScriptError::unsupported(
                "Document complete own-key enumeration is not implemented",
            ));
        }
        let dom = dom_own_properties::host(receiver).is_some();
        let object = if dom {
            self.dom_own_object(receiver)?
        } else {
            self.property_object(receiver)
        };
        if object.is_none() && js_object(receiver) && !dom {
            return Err(ScriptError::unsupported(
                "host own-property enumeration is not implemented",
            ));
        }
        let dense = match receiver {
            Value::Array(id) => self.arrays[*id].len(),
            _ => 0,
        };
        let text = match receiver {
            Value::String(text) => Some(text.len()),
            Value::Object(id) => match &self.objects[*id].boxed {
                Some(Value::String(text)) => Some(text.len()),
                _ => None,
            },
            _ => None,
        };
        let virtual_length = matches!(receiver, Value::Array(_)) || text.is_some();
        let order_len = object.map_or(0, |id| self.objects[id].order.len());
        // A snapshot with no virtual keys and no possible Array-index strings
        // already has its final order. Prove this afresh from the retained
        // creation-order keys, never from cached descriptors or prototypes.
        // Small orders retain the generic builder: the extra proof/copy setup
        // is worthwhile only beyond a fixed, source-derived minimum size.
        self.work(4)?;
        if !virtual_length
            && order_len >= 8
            && let Some(id) = object
            && let Some(keys) = self.own_key_stored_strings(id)?
        {
            return Ok(keys);
        }
        // Counting retained strings reads only tags, never string contents.
        self.work(order_len.saturating_add(1))?;
        let stored = object.map_or(0, |id| {
            self.objects[id]
                .order
                .iter()
                .filter(|key| matches!(key, PropertyKey::String(_)))
                .count()
        });
        let virtual_indices = dense.checked_add(text.unwrap_or(0)).ok_or_else(overflow)?;
        let bound = virtual_indices
            .checked_add(usize::from(virtual_length))
            .and_then(|count| count.checked_add(stored))
            .ok_or_else(overflow)?;
        let hole_work = match receiver {
            Value::Array(id) => tree_bound(self.array_holes[*id].len()).0,
            _ => 0,
        };
        let digits = virtual_indices
            .saturating_sub(1)
            .checked_ilog10()
            .map_or(1, |n| n as usize + 1);
        // Bound decimal formatting, UTF-16 encoding, Rc payload copying and
        // numeric parsing by the actual virtual-index range. Retained classification
        // examines at most ten units; virtual length examines six. Hole-tree
        // comparisons are integers. No logical gaps run.
        self.work(
            virtual_indices
                .saturating_mul(4 * (digits + 1) + hole_work)
                .saturating_add(order_len.saturating_mul(12))
                .saturating_add(bound.saturating_mul(2))
                .saturating_add(usize::from(virtual_length) * 14),
        )?;
        let buffers = bound
            .checked_mul(std::mem::size_of::<OwnKey>() + std::mem::size_of::<JsString>())
            .ok_or_else(overflow)?;
        // Formatting String, UTF-16 Vec and Rc payload/header are all covered
        // before either vector or virtual text is allocated. Retained keys
        // only clone Rc handles and have no text-length-dependent allocation.
        let virtual_bytes = virtual_indices
            .checked_mul(64 + digits * 6)
            .and_then(|bytes| bytes.checked_add(usize::from(virtual_length) * 64))
            .ok_or_else(overflow)?;
        self.charge(buffers.checked_add(virtual_bytes).ok_or_else(overflow)?)?;
        let mut keys = Vec::new();
        keys.try_reserve_exact(bound)
            .map_err(|_| ScriptError::resource("own-key entry allocation failed"))?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(bound)
            .map_err(|_| ScriptError::resource("own-key output allocation failed"))?;
        for index in 0..virtual_indices {
            if let Value::Array(id) = receiver
                && self.array_holes[*id].contains(&index)
            {
                continue;
            }
            push_key(&mut keys, JsString::from(index.to_string()));
        }
        if virtual_length {
            push_key(&mut keys, "length".into());
        }
        if let Some(id) = object {
            for key in &self.objects[id].order {
                let Some(key) = key.as_string() else {
                    continue;
                };
                if virtual_length && key.units() == [108, 101, 110, 103, 116, 104] {
                    continue;
                }
                push_key(&mut keys, key.clone());
            }
        }
        self.own_key_sort(&mut keys)?;
        let mut previous_index = None;
        for key in keys {
            if key.kind == 0 {
                if previous_index == Some(key.index) {
                    continue;
                }
                previous_index = Some(key.index);
            }
            result.push(key.text);
        }
        Ok(result)
    }

    fn own_key_stored_strings(&mut self, object: usize) -> Result<Option<Vec<JsString>>> {
        self.work(4)?;
        let order_len = self.objects[object].order.len();
        let mut stored = 0usize;
        for ordinal in 0..order_len {
            // Tag/count, first-unit access, digit bounds and loop control.
            // Empty, non-ASCII and nonscalar strings cannot be Array indices.
            self.work(6)?;
            let Some(key) = self.objects[object].order[ordinal].as_string() else {
                continue;
            };
            if key
                .units()
                .first()
                .is_some_and(|unit| (48..=57).contains(unit))
            {
                return Ok(None);
            }
            stored = stored.checked_add(1).ok_or_else(overflow)?;
        }
        // Pay the checked size/reserve, second tag/iteration pass and retained
        // handle copy/move/retirement before allocating the sole result vector.
        let work = order_len
            .checked_mul(4)
            .and_then(|work| work.checked_add(stored.checked_mul(2)?))
            .and_then(|work| work.checked_add(8))
            .ok_or_else(overflow)?;
        self.work(work)?;
        self.charge(
            stored
                .checked_mul(std::mem::size_of::<JsString>())
                .ok_or_else(overflow)?,
        )?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(stored)
            .map_err(|_| ScriptError::resource("own-key string allocation failed"))?;
        for key in &self.objects[object].order {
            if let Some(key) = key.as_string() {
                result.push(key.clone());
            }
        }
        Ok(Some(result))
    }

    fn typed_array_own_strings(
        &mut self,
        receiver: &Value,
        length: usize,
    ) -> Result<Vec<JsString>> {
        // A real view's live dense keys precede its ordinary creation-order
        // strings. Keep usize indices throughout: this is not Array's u32
        // index classifier, and no virtual length property is synthesized.
        self.work(8)?;
        let Value::Object(id) = receiver else {
            unreachable!("TypedArray brand belongs to an ordinary object")
        };
        let order_len = self.objects[*id].order.len();
        self.work(order_len.checked_add(1).ok_or_else(overflow)?)?;
        let stored = self.objects[*id]
            .order
            .iter()
            .filter(|key| matches!(key, PropertyKey::String(_)))
            .count();
        let bound = length.checked_add(stored).ok_or_else(overflow)?;
        let digits = length
            .saturating_sub(1)
            .checked_ilog10()
            .map_or(1, |n| n as usize + 1);
        // Retain the established virtual decimal/String/UTF-16/Rc envelope.
        // Stored keys clone handles; each reached canonical classification is
        // additionally paid by the core helper below. No sort or second key
        // vector is required for this separate dense/ordinary order.
        let work = length
            .checked_mul(4 * (digits + 1))
            .and_then(|work| work.checked_add(order_len.checked_mul(4)?))
            .and_then(|work| work.checked_add(bound.checked_mul(2)?))
            .ok_or_else(overflow)?;
        self.work(work)?;
        let bytes = length
            .checked_mul(64 + digits * 6)
            .and_then(|bytes| {
                bytes.checked_add(bound.checked_mul(std::mem::size_of::<JsString>())?)
            })
            .ok_or_else(overflow)?;
        self.charge(bytes)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(bound)
            .map_err(|_| ScriptError::resource("TypedArray own-key allocation failed"))?;
        for index in 0..length {
            result.push(JsString::from(index.to_string()));
        }
        for ordinal in 0..order_len {
            let Some(key) = self.objects[*id].order[ordinal].as_string().cloned() else {
                continue;
            };
            // Canonical numeric keys never belong to the ordinary stored-key
            // suffix, even after resize/detachment or in a malformed private
            // fixture. This also prevents a virtual/stored duplicate index.
            if matches!(
                self.typed_array_own_property(receiver, &key, false)?,
                typed_array::Exotic::Ordinary
            ) {
                result.push(key);
            }
        }
        Ok(result)
    }

    fn own_key_less(&mut self, left: &OwnKey, right: &OwnKey) -> Result<bool> {
        self.work(3)?;
        Ok(left.rank() < right.rank())
    }

    fn own_key_sift(&mut self, keys: &mut [OwnKey], mut root: usize, end: usize) -> Result<()> {
        loop {
            self.tick()?;
            let Some(mut child) = root
                .checked_mul(2)
                .and_then(|index| index.checked_add(1))
                .filter(|index| *index < end)
            else {
                return Ok(());
            };
            if child + 1 < end && self.own_key_less(&keys[child], &keys[child + 1])? {
                child += 1;
            }
            if !self.own_key_less(&keys[root], &keys[child])? {
                return Ok(());
            }
            self.tick()?;
            keys.swap(root, child);
            root = child;
        }
    }

    fn own_key_sort(&mut self, keys: &mut [OwnKey]) -> Result<()> {
        let mut ordered = true;
        for pair in keys.windows(2) {
            if self.own_key_less(&pair[1], &pair[0])? {
                ordered = false;
                break;
            }
        }
        if ordered {
            return Ok(());
        }
        // Fallible in-place heapsort: pay for each reached fixed-width
        // comparison/move, not an estimated UTF-16 comparison or sort scratch.
        let count = keys.len();
        for root in (0..count / 2).rev() {
            self.own_key_sift(keys, root, count)?;
        }
        for end in (1..count).rev() {
            self.tick()?;
            keys.swap(0, end);
            self.own_key_sift(keys, 0, end)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn for_in_visit(
        &mut self,
        visited: &mut VisitedNames,
        object: &Value,
        key: &JsString,
    ) -> Result<Option<Property>> {
        self.for_in_visit_using(visited, object, key, Self::for_in_own_property)
    }

    pub(super) fn for_in_visit_snapshot(
        &mut self,
        visited: &mut VisitedNames,
        object: &ForInObject,
        key: &JsString,
    ) -> Result<Option<Property>> {
        self.for_in_visit_using(visited, &object.value, key, object.reader)
    }

    fn for_in_visit_using(
        &mut self,
        visited: &mut VisitedNames,
        object: &Value,
        key: &JsString,
        reader: ForInReader,
    ) -> Result<Option<Property>> {
        use std::collections::btree_map::Entry;

        // Reading slice length does not scan or copy the UTF-16 payload.
        self.tick()?;
        let length = key.len();
        let (comparisons, nodes) = tree_bound(visited.buckets.len());
        self.work(1usize.saturating_add(nodes).saturating_add(comparisons))?;
        // entry() does not allocate on pinned Rust 1.88/1.98, even for an empty
        // tree. Cached locations cross only a nontrapping own-descriptor read,
        // never author code. Missing descriptors must leave no empty bucket.
        // Tree insertion's 24 units per level bound fixed-size key/value-entry moves
        // and edge/backlink updates, not individual machine-word copies. A map
        // header moves without traversing its contents. One node allowance per
        // tree insertion/bucket covers cumulative append-only splits; the first
        // inline name is covered by its outer-node allowance. That allowance
        // uses the full enum size. BTreeMap's infallible allocator
        // remains a prepaid boundary. Hidden properties are inserted too.
        match visited.buckets.entry(length) {
            Entry::Occupied(mut bucket) => {
                self.work(2)?;
                match bucket.get_mut() {
                    NameBucket::Single(first) => {
                        self.work(length.saturating_add(1))?;
                        if first == key {
                            return Ok(None);
                        }
                        let Some(property) = reader(self, object, key)? else {
                            return Ok(None);
                        };
                        // Build an empty root and insert into its one-key leaf:
                        // 25 + 25 bounded entry moves, 3 + length for the second
                        // search, and 4 for retained handles/publication. This
                        // fresh two-key leaf cannot split or traverse a parent.
                        // The general tree insertion schedule below is unchanged.
                        self.work(length.saturating_add(57))?;
                        self.charge(VISITED_NAME_BYTES)?;
                        let mut names = BTreeMap::new();
                        names.insert(first.clone(), ());
                        names.insert(key.clone(), ());
                        *bucket.get_mut() = NameBucket::Tree(names);
                        Ok(Some(property))
                    }
                    NameBucket::Tree(names) => {
                        let (comparisons, nodes) = tree_bound(names.len());
                        self.work(
                            1usize.saturating_add(nodes).saturating_add(
                                comparisons.saturating_mul(length.saturating_add(1)),
                            ),
                        )?;
                        let Entry::Vacant(entry) = names.entry(key.clone()) else {
                            return Ok(None);
                        };
                        let Some(property) = reader(self, object, key)? else {
                            return Ok(None);
                        };
                        self.work(
                            1usize.saturating_add(nodes.saturating_add(1).saturating_mul(24)),
                        )?;
                        self.charge(VISITED_NAME_BYTES)?;
                        entry.insert(());
                        Ok(Some(property))
                    }
                }
            }
            Entry::Vacant(entry) => {
                let Some(property) = reader(self, object, key)? else {
                    return Ok(None);
                };
                // Retain the name directly in the outer node. Pay the cached
                // insertion and four fixed-size handle operations before any
                // tree mutation; there is no inner root allocation/insertion.
                self.work(4usize.saturating_add(
                    1usize.saturating_add(nodes.saturating_add(1).saturating_mul(24)),
                ))?;
                self.charge(VISITED_BUCKET_BYTES)?;
                entry.insert(NameBucket::Single(key.clone()));
                Ok(Some(property))
            }
        }
    }

    fn for_in_own_property(&mut self, object: &Value, key: &JsString) -> Result<Option<Property>> {
        match self.typed_array_own_property(object, key, false)? {
            typed_array::Exotic::Handled(property) => Ok(property),
            typed_array::Exotic::Ordinary => self.read_ordinary_own_property(object, key),
        }
    }

    fn for_in_proven_ordinary_property(
        &mut self,
        object: &Value,
        key: &JsString,
    ) -> Result<Option<Property>> {
        // Reached only after visited-name rejection; admit the proof use before
        // the live descriptor read or any insertion into the visited set.
        self.work(4)?;
        self.read_ordinary_own_property(object, key)
    }
}
