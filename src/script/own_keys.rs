//! Bounded string-key snapshots and for-in's reached visited-name operations.
//! Live descriptor/Get/binding lookup accounting is separate from this module.
use super::*;

#[cfg(test)]
mod tests;

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
        if receiver == &Value::Window {
            return self.window_own_keys();
        }
        let object = self.property_object(receiver);
        if object.is_none() && js_object(receiver) {
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

    pub(super) fn for_in_visit(
        &mut self,
        visited: &mut BTreeMap<JsString, ()>,
        object: &Value,
        key: &JsString,
    ) -> Result<Option<Property>> {
        let (comparisons, nodes) = tree_bound(visited.len());
        self.work(
            1usize
                .saturating_add(nodes)
                .saturating_add(comparisons.saturating_mul(key.len().saturating_add(1))),
        )?;
        // One actual tree search. Rust 1.88/1.98 entry() does not allocate,
        // including an empty map; vacant insertion below owns node allocation.
        // The cached location is held across a nontrapping own-descriptor read,
        // never across author code. Occupied names skip that read entirely.
        let std::collections::btree_map::Entry::Vacant(entry) = visited.entry(key.clone()) else {
            return Ok(None);
        };
        let Some(property) = self.own_property(object, key) else {
            return Ok(None);
        };
        // A cached vacant insertion splits/moves handles and edges without
        // repeating the text search. Missing properties never enter visited;
        // present nonenumerable properties must still shadow inherited names.
        self.work(1usize.saturating_add(nodes.saturating_add(1).saturating_mul(24)))?;
        // One conservative node allowance per inserted key covers cumulative
        // splits: this map starts empty and never removes keys, and every
        // allocated node retains at least one key. Unit values occupy no bytes;
        // both supported node layouts fit 11 handles, 13 pointers and metadata.
        // BTreeMap has no fallible reserve; this is a prepaid storage bound.
        self.charge(16 * std::mem::size_of::<JsString>() + 32 * std::mem::size_of::<usize>() + 64)?;
        entry.insert(());
        Ok(Some(property))
    }
}
