//! SameValue on existing runtime values, with bounded borrowed comparisons.
use super::*;

#[cfg(test)]
mod tests;

// Pinned B=6 tree bound, as used by the scoped ArrayBuffer/DataView installers.
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
    (count.min(11usize.saturating_mul(nodes)), nodes)
}

fn search_work(count: usize, units: usize) -> usize {
    tree_bound(count).0.saturating_mul(1 + units / 8)
}

fn insertion_work(count: usize) -> usize {
    // <=12 fixed-size entry moves +12 edge/backlink updates +4 setup units
    // per possible level, including a new root. These are not byte counts.
    tree_bound(count).1.saturating_add(1).saturating_mul(28)
}

fn node_bytes<K, V>() -> usize {
    16 * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}

fn insertion_bytes<K, V>(count: usize) -> Result<usize> {
    tree_bound(count)
        .1
        .saturating_add(1)
        .checked_mul(node_bytes::<K, V>())
        .ok_or_else(|| ScriptError::resource("Object.is intrinsic storage overflow"))
}

impl Runtime {
    pub(super) fn install_object_is_intrinsic(&mut self) -> Result<()> {
        let registry = self.native_properties.len();
        self.work(search_work(registry, 6))?;
        let owner = self.native_properties["Object"];
        let properties = self.objects[owner].values.len();
        self.work(
            64usize
                .saturating_add(2 * search_work(registry, 9))
                .saturating_add(insertion_work(registry))
                .saturating_add(search_work(self.prototypes.len(), 6))
                .saturating_add(2 * search_work(0, 4))
                .saturating_add(2 * search_work(1, 6))
                .saturating_add(search_work(2, 4))
                .saturating_add(search_work(2, 6))
                // Fresh leaf: three entry moves, four setup operations, two
                // order appends and six descriptor flag assignments.
                .saturating_add(15)
                .saturating_add(2 * search_work(properties, 2))
                .saturating_add(insertion_work(properties))
                .saturating_add(1),
        )?;
        // Generic intrinsic_function keeps its ordinary base and 552 property
        // bytes. Pay fixed payloads, actual-type nodes and the implicit fresh
        // Vec growth separately. Pinned RawVec starts this element type at 4.
        self.charge(
            1536 + 8 * (9 + 2)
                + node_bytes::<PropertyKey, Property>()
                + 4 * std::mem::size_of::<PropertyKey>(),
        )?;
        self.charge(insertion_bytes::<String, usize>(registry)?)?;
        self.charge(insertion_bytes::<PropertyKey, Property>(properties)?)?;
        self.object_is_reserve_order(owner)?;
        // One-time literal installation. Keep the existing generic path,
        // including its two searches and final descriptor rewrites.
        let function = self.intrinsic_function("Object.is", "is", 2)?;
        self.objects[owner].insert_hidden("is".into(), function);
        Ok(())
    }

    fn object_is_reserve_order(&mut self, owner: usize) -> Result<()> {
        let order = &self.objects[owner].order;
        if order.len() < order.capacity() {
            return Ok(());
        }
        let length = order.len();
        let capacity = order
            .capacity()
            .checked_mul(2)
            .and_then(|grown| length.checked_add(1).map(|needed| grown.max(needed).max(4)))
            .ok_or_else(|| ScriptError::resource("Object.is intrinsic order overflow"))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<PropertyKey>())
            .ok_or_else(|| ScriptError::resource("Object.is intrinsic order overflow"))?;
        self.work(1usize.saturating_add(length.saturating_mul(2)))?;
        // Cumulative requested storage, without refunding the old buffer.
        // Allocator rounding remains separate from this logical prepayment.
        self.charge(bytes)?;
        self.objects[owner]
            .order
            .try_reserve_exact(capacity - length)
            .map_err(|_| ScriptError::resource("Object.is intrinsic order allocation failed"))
    }

    pub(super) fn object_is(&mut self, args: &[Value]) -> Result<Value> {
        self.object_is_values(
            args.first().unwrap_or(&Value::Undefined),
            args.get(1).unwrap_or(&Value::Undefined),
        )
        .map(Value::Bool)
    }

    pub(super) fn object_is_values(&mut self, mut left: &Value, mut right: &Value) -> Result<bool> {
        self.work(4)?;
        let mut receiver_key = false;
        loop {
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    return Ok(if receiver_key {
                        // Implementation identity keys retain existing equality;
                        // they are not recursively new SameValue operands.
                        a == b
                    } else {
                        a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
                    });
                }
                (Value::String(a), Value::String(b)) => {
                    if a.len() != b.len() {
                        return Ok(false);
                    }
                    self.work(2 + a.len() / 8 + b.len() / 8)?;
                    return Ok(a == b);
                }
                (Value::Native(a), Value::Native(b)) => {
                    self.tick()?;
                    // Native is PartialEq, not Eq: this is an explicit scoped
                    // reflexivity guarantee, not an inherited Rc shortcut.
                    if !receiver_key && Rc::ptr_eq(a, b) {
                        return Ok(true);
                    }
                    if a.name.len() != b.name.len() {
                        return Ok(false);
                    }
                    self.work(2 + a.name.len() / 8 + b.name.len() / 8)?;
                    if a.name != b.name {
                        return Ok(false);
                    }
                    left = &a.receiver;
                    right = &b.receiver;
                    receiver_key = true;
                }
                // All remaining same-kind values are scalar identities. String
                // and Native payload comparisons are handled above; mismatched
                // variants neither inspect properties nor traverse payloads.
                _ => return Ok(left == right),
            }
        }
    }
}
