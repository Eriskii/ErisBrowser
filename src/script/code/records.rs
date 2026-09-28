//! Bounded record pages. Appending syntax never relocates existing records.
use super::*;

const PAGE: usize = 128;
const PAGES: usize = MAX_TOKENS.div_ceil(PAGE);

#[derive(Debug)]
pub(super) struct Records<T> {
    pages: Vec<Vec<T>>,
    length: usize,
}
impl<T> Records<T> {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            length: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn reserve(&mut self, needed: usize, budget: &mut regexp::Budget) -> Result<()> {
        if needed > MAX_TOKENS {
            return Err(ScriptError::resource(
                "executable record count limit exceeded",
            ));
        }
        // A singleton intrinsic is normally immutable. Still keep this storage
        // type safe to extend: its short first page must grow before append.
        if let Some(first) = self.pages.first_mut()
            && needed > first.capacity()
            && first.capacity() < PAGE
        {
            budget.work(first.len() + 1).map_err(regexp_error)?;
            compile_allocate(budget, PAGE * std::mem::size_of::<T>() + 32)?;
            first
                .try_reserve_exact(PAGE - first.len())
                .map_err(|_| ScriptError::resource("executable record allocation failed"))?;
        }
        while self.pages.len() < needed.div_ceil(PAGE) {
            budget.work(1).map_err(regexp_error)?;
            if self.pages.len() == self.pages.capacity() {
                let capacity = self.pages.capacity().saturating_mul(2).clamp(4, PAGES);
                budget.work(self.pages.len() + 1).map_err(regexp_error)?;
                compile_allocate(budget, capacity * std::mem::size_of::<Vec<T>>() + 32)?;
                self.pages
                    .try_reserve_exact(capacity - self.pages.len())
                    .map_err(|_| ScriptError::resource("executable page allocation failed"))?;
            }
            compile_allocate(budget, PAGE * std::mem::size_of::<T>() + 32)?;
            let mut page = Vec::new();
            page.try_reserve_exact(PAGE)
                .map_err(|_| ScriptError::resource("executable record allocation failed"))?;
            self.pages.push(page);
        }
        Ok(())
    }
    pub fn append(&mut self, value: T, budget: &mut regexp::Budget) -> Result<()> {
        budget.work(1).map_err(regexp_error)?;
        self.reserve(self.length.saturating_add(1), budget)?;
        let page = &mut self.pages[self.length / PAGE];
        assert!(page.len() < page.capacity());
        page.push(value);
        self.length += 1;
        Ok(())
    }
    // The built-in empty function is immutable and never extended. Avoid a full
    // parser page for this one-record intrinsic; no source reaches this path.
    pub fn singleton(value: T) -> Result<Self> {
        let mut page = Vec::new();
        page.try_reserve_exact(1)
            .map_err(|_| ScriptError::resource("empty executable allocation failed"))?;
        page.push(value);
        let mut pages = Vec::new();
        pages
            .try_reserve_exact(1)
            .map_err(|_| ScriptError::resource("empty executable page allocation failed"))?;
        pages.push(page);
        Ok(Self { pages, length: 1 })
    }
    #[cfg(test)]
    pub fn test_push(&mut self, value: T) {
        self.append(
            value,
            &mut regexp::Budget {
                steps: usize::MAX,
                allocated: 0,
                heap_limit: usize::MAX,
                stack_limit: 16,
            },
        )
        .unwrap();
    }
}
impl<T> std::ops::Index<usize> for Records<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        assert!(index < self.length);
        &self.pages[index / PAGE][index % PAGE]
    }
}
impl<T> std::ops::IndexMut<usize> for Records<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.length);
        &mut self.pages[index / PAGE][index % PAGE]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn budget() -> regexp::Budget {
        regexp::Budget {
            steps: crate::script::MAX_STEPS,
            allocated: 0,
            heap_limit: crate::script::MAX_HEAP,
            stack_limit: 16,
        }
    }
    #[test]
    fn record_pages_preserve_indices_and_precharge_directory_growth() {
        let mut records = Records::new();
        let mut ledger = budget();
        records.append(0usize, &mut ledger).unwrap();
        let first = &records[0] as *const usize;
        for i in 1..1_025 {
            records.append(i, &mut ledger).unwrap();
        }
        assert_eq!(first, &records[0] as *const usize);
        for i in 0..1_025 {
            assert_eq!(records[i], i);
        }
        records[128] = 0;
        assert_eq!(records[128], 0);
        assert_eq!(records.pages.len(), 9);
        assert!(ledger.allocated >= 9 * PAGE * std::mem::size_of::<usize>());
        // Work grows with appends and page descriptors, not copied records.
        assert!(crate::script::MAX_STEPS - ledger.steps < 1_100);
        assert!(
            records
                .reserve(MAX_TOKENS + 1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
    }
    #[test]
    fn record_refusals_leave_retained_values_and_charges_intact() {
        let marker = Rc::new(1usize);
        let weak = Rc::downgrade(&marker);
        let mut records = Records::new();
        let mut ledger = budget();
        records.append(marker, &mut ledger).unwrap();
        let allocation = ledger.allocated;
        for _ in 1..PAGE {
            records.append(Rc::new(0), &mut ledger).unwrap();
        }
        ledger.heap_limit = allocation;
        assert!(
            records
                .append(Rc::new(2), &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(records.len(), PAGE);
        assert_eq!(records.pages.len(), 1);
        assert!(ledger.allocated > allocation);
        assert!(weak.upgrade().is_some());
        drop(records);
        assert!(weak.upgrade().is_none());
        let mut records = Records::<usize>::new();
        ledger = budget();
        ledger.steps = 0;
        assert!(
            records
                .append(1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (records.len(), records.pages.capacity(), ledger.allocated),
            (0, 0, 0)
        );
        let mut singleton = Records::singleton(7usize).unwrap();
        ledger = budget();
        ledger.heap_limit = 0;
        assert!(
            singleton
                .append(8, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(singleton.len(), 1);
        assert_eq!(singleton[0], 7);
        ledger = budget();
        singleton.append(8, &mut ledger).unwrap();
        assert_eq!(singleton[1], 8);
        assert_eq!(singleton.pages[0].capacity(), PAGE);
    }
}
