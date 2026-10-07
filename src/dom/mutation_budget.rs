//! Shared checked DOM budget primitives. Existing replacement charges are unchanged.
use super::DomDataError;
use std::mem::size_of;

pub(crate) struct DomMutationBudget {
    pub steps: usize,
    pub allocated: usize,
    pub heap_limit: usize,
}

impl DomMutationBudget {
    pub(super) fn work(&mut self, count: usize) -> Result<(), DomDataError> {
        if count > self.steps {
            self.steps = 0;
            return Err(DomDataError::LimitExceeded);
        }
        self.steps -= count;
        Ok(())
    }

    pub(super) fn charge(&mut self, bytes: usize) -> Result<(), DomDataError> {
        self.allocated = self
            .allocated
            .checked_add(bytes)
            .ok_or(DomDataError::LimitExceeded)?;
        if self.allocated > self.heap_limit {
            return Err(DomDataError::LimitExceeded);
        }
        Ok(())
    }
}

pub(super) fn add(left: usize, right: usize) -> Result<usize, DomDataError> {
    left.checked_add(right).ok_or(DomDataError::LimitExceeded)
}

pub(super) fn mul(left: usize, right: usize) -> Result<usize, DomDataError> {
    left.checked_mul(right).ok_or(DomDataError::LimitExceeded)
}

// The same BTree fanout bound used by script's own-property/native bags. Full
// UTF-8 key bytes, not a fractional character allowance, bound comparisons.
pub(super) fn tree(count: usize) -> Result<(usize, usize), DomDataError> {
    if count == 0 {
        return Ok((0, 0));
    }
    let mut capacity = add(count, 1)? / 2;
    let mut height = 1;
    while capacity >= 6 {
        capacity /= 6;
        height += 1;
    }
    Ok((count.min(mul(11, height)?), height))
}

pub(super) fn search(count: usize, key_bytes: usize) -> Result<usize, DomDataError> {
    mul(tree(count)?.0, add(2, key_bytes)?)
}

pub(super) fn movement(count: usize) -> Result<usize, DomDataError> {
    if count == 0 {
        Ok(0)
    } else {
        mul(64, add(tree(count)?.1, 1)?)
    }
}

pub(super) fn map_nodes<K, V>(count: usize) -> Result<usize, DomDataError> {
    let bytes = add(
        mul(16, add(size_of::<K>(), size_of::<V>())?)?,
        add(mul(32, size_of::<usize>())?, 64)?,
    )?;
    mul(add(tree(count)?.1, 1)?, bytes)
}

pub(super) fn push<T>(
    values: &mut Vec<T>,
    value: T,
    maximum: usize,
    budget: &mut DomMutationBudget,
) -> Result<(), DomDataError> {
    if values.len() >= maximum {
        return Err(DomDataError::LimitExceeded);
    }
    if values.len() == values.capacity() {
        let next = add(values.len(), 1)?
            .max(mul(values.capacity(), 2)?.min(maximum))
            .min(maximum);
        budget.work(add(1, mul(values.len(), 2)?)?)?;
        budget.charge(mul(next, size_of::<T>())?)?;
        values
            .try_reserve_exact(next - values.len())
            .map_err(|_| DomDataError::AllocationFailed)?;
    }
    budget.work(1)?;
    values.push(value);
    Ok(())
}
