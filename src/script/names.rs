//! Fallible, borrowed declaration-name validation within the compile ledger.

use super::{MAX_TOKENS, Result, ScriptError, compile_allocate, regexp, regexp_error};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Entry<'a> {
    pub text: &'a str,
    pub block_function: bool,
    order: usize,
}

#[derive(Default, Debug)]
pub(super) struct Names<'a> {
    entries: Vec<Entry<'a>>,
}

#[derive(Debug)]
pub(super) struct SortedNames<'a> {
    entries: Vec<Entry<'a>>,
}

fn compare(left: &str, right: &str, budget: &mut regexp::Budget) -> Result<Ordering> {
    budget
        .work(1 + left.len().min(right.len()) / 8)
        .map_err(regexp_error)?;
    Ok(left.cmp(right))
}

impl<'a> Names<'a> {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(
        &mut self,
        text: &'a str,
        block_function: bool,
        budget: &mut regexp::Budget,
    ) -> Result<()> {
        budget.work(1).map_err(regexp_error)?;
        if self.entries.len() == MAX_TOKENS {
            return Err(ScriptError::resource(
                "declaration name count limit exceeded",
            ));
        }
        if self.entries.len() == self.entries.capacity() {
            let capacity = self
                .entries
                .capacity()
                .saturating_mul(2)
                .clamp(8, MAX_TOKENS);
            budget.work(self.entries.len() + 1).map_err(regexp_error)?;
            compile_allocate(budget, capacity * std::mem::size_of::<Entry<'a>>() + 32)?;
            self.entries
                .try_reserve_exact(capacity - self.entries.len())
                .map_err(|_| ScriptError::resource("declaration name allocation failed"))?;
        }
        self.entries.push(Entry {
            text,
            block_function,
            order: self.entries.len(),
        });
        Ok(())
    }

    pub fn finish(mut self, budget: &mut regexp::Budget) -> Result<SortedNames<'a>> {
        let length = self.entries.len();
        if length < 2 {
            return Ok(SortedNames {
                entries: self.entries,
            });
        }
        // Reserve once, before copying any records. Strings remain borrowed.
        budget.work(length + 1).map_err(regexp_error)?;
        compile_allocate(budget, length * std::mem::size_of::<Entry<'a>>() + 32)?;
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("declaration sort allocation failed"))?;
        scratch.extend_from_slice(&self.entries);
        let mut width = 1usize;
        while width < length {
            let stride = width
                .checked_mul(2)
                .ok_or_else(|| ScriptError::resource("declaration sort width overflow"))?;
            for start in (0..length).step_by(stride) {
                let middle = start.saturating_add(width).min(length);
                let end = start.saturating_add(stride).min(length);
                let (mut left, mut right) = (start, middle);
                for output in &mut scratch[start..end] {
                    budget.work(1).map_err(regexp_error)?;
                    let take_left = right == end
                        || left < middle
                            && compare(self.entries[left].text, self.entries[right].text, budget)?
                                != Ordering::Greater;
                    let index = if take_left {
                        let index = left;
                        left += 1;
                        index
                    } else {
                        let index = right;
                        right += 1;
                        index
                    };
                    *output = self.entries[index];
                }
            }
            std::mem::swap(&mut self.entries, &mut scratch);
            width = stride;
        }
        Ok(SortedNames {
            entries: self.entries,
        })
    }
}

impl<'a> SortedNames<'a> {
    pub fn first_duplicate(&self, budget: &mut regexp::Budget) -> Result<Option<Entry<'a>>> {
        let mut duplicate: Option<Entry<'a>> = None;
        for pair in self.entries.windows(2) {
            if compare(pair[0].text, pair[1].text, budget)? == Ordering::Equal
                && duplicate.is_none_or(|found| pair[1].order < found.order)
            {
                // Stable ordering makes the right entry a repeated occurrence.
                // Select the earliest repetition in source order, not key order.
                duplicate = Some(pair[1]);
            }
        }
        Ok(duplicate)
    }

    pub fn contains(&self, name: &str, budget: &mut regexp::Budget) -> Result<bool> {
        let (mut start, mut end) = (0, self.entries.len());
        while start < end {
            let middle = start + (end - start) / 2;
            match compare(self.entries[middle].text, name, budget)? {
                Ordering::Less => start = middle + 1,
                Ordering::Greater => end = middle,
                Ordering::Equal => return Ok(true),
            }
        }
        Ok(false)
    }

    pub fn first_intersection(
        &self,
        other: &Self,
        budget: &mut regexp::Budget,
    ) -> Result<Option<&'a str>> {
        let (mut left, mut right) = (0, 0);
        while left < self.entries.len() && right < other.entries.len() {
            let name = self.entries[left].text;
            match compare(name, other.entries[right].text, budget)? {
                Ordering::Less => left += 1,
                Ordering::Greater => right += 1,
                Ordering::Equal => return Ok(Some(name)),
            }
        }
        Ok(None)
    }
}

pub(super) fn named_error(
    prefix: &str,
    name: &str,
    budget: &mut regexp::Budget,
) -> Result<ScriptError> {
    let length = prefix
        .len()
        .checked_add(name.len())
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| ScriptError::resource("declaration diagnostic length overflow"))?;
    budget.work(1 + length / 8).map_err(regexp_error)?;
    compile_allocate(budget, length.saturating_add(64))?;
    let mut message = String::new();
    message
        .try_reserve_exact(length)
        .map_err(|_| ScriptError::resource("declaration diagnostic allocation failed"))?;
    message.push_str(prefix);
    message.push_str(name);
    message.push('\'');
    Ok(ScriptError::at(message, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{MAX_HEAP, MAX_STEPS};
    use std::collections::{BTreeMap, BTreeSet};

    fn budget() -> regexp::Budget {
        regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        }
    }

    fn sorted<'a>(input: &[(&'a str, bool)], ledger: &mut regexp::Budget) -> SortedNames<'a> {
        let mut names = Names::default();
        for &(name, block_function) in input {
            names.push(name, block_function, ledger).unwrap();
        }
        names.finish(ledger).unwrap()
    }

    #[test]
    fn merge_matches_independent_stable_order_duplicate_and_set_oracles() {
        let mut random = 0x47f2_1526u32;
        let pool = (0..23)
            .map(|i| {
                format!(
                    "{}{:02}",
                    ["a", "π", "名字", "prefix_"].get(i % 4).unwrap(),
                    i
                )
            })
            .collect::<Vec<_>>();
        for length in 0..128 {
            let mut input = Vec::new();
            let mut expected = Vec::new();
            let mut first = BTreeMap::new();
            let mut duplicate = None;
            for order in 0..length {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                let text = pool[random as usize % pool.len()].as_str();
                let block_function = random & 1 == 1;
                input.push((text, block_function));
                let entry = Entry {
                    text,
                    block_function,
                    order,
                };
                if first.insert(text, order).is_some() && duplicate.is_none() {
                    duplicate = Some(entry);
                }
                expected.push(entry);
            }
            expected.sort_by_key(|entry| entry.text);
            let mut ledger = budget();
            let actual = sorted(&input, &mut ledger);
            assert_eq!(actual.entries, expected, "length={length}");
            assert_eq!(actual.first_duplicate(&mut ledger).unwrap(), duplicate);
            for name in pool.iter().map(String::as_str).chain(["absent"]) {
                assert_eq!(
                    actual.contains(name, &mut ledger).unwrap(),
                    first.contains_key(name)
                );
            }
            let right = sorted(
                &[("π01", false), ("prefix_03", false), ("absent", false)],
                &mut ledger,
            );
            let a: BTreeSet<_> = input.iter().map(|&(name, _)| name).collect();
            let b: BTreeSet<_> = right.entries.iter().map(|entry| entry.text).collect();
            assert_eq!(
                actual.first_intersection(&right, &mut ledger).unwrap(),
                a.intersection(&b).next().copied()
            );
            for entry in &actual.entries {
                assert!(pool.iter().any(|name| name.as_ptr() == entry.text.as_ptr()));
            }
        }
    }

    #[test]
    fn collection_precharges_work_and_growth_before_mutation() {
        for steps in [0, 1] {
            let mut ledger = budget();
            ledger.steps = steps;
            let mut names = Names::default();
            assert!(
                names
                    .push("a", false, &mut ledger)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert!(names.entries.is_empty());
            assert_eq!((names.entries.capacity(), ledger.allocated), (0, 0));
        }
        let mut ledger = budget();
        ledger.heap_limit = 8 * std::mem::size_of::<Entry<'_>>() + 31;
        let mut names = Names::default();
        assert!(
            names
                .push("a", false, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(names.entries.capacity(), 0);
        assert_eq!(ledger.allocated, ledger.heap_limit + 1);
        let mut ledger = budget();
        for _ in 0..8 {
            names.push("a", false, &mut ledger).unwrap();
        }
        let allocated = ledger.allocated;
        ledger.steps = 9;
        assert!(
            names
                .push("b", true, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (
                names.entries.len(),
                names.entries.capacity(),
                ledger.allocated
            ),
            (8, 8, allocated)
        );
        ledger.steps = 10;
        ledger.heap_limit = allocated + 16 * std::mem::size_of::<Entry<'_>>() + 31;
        assert!(
            names
                .push("b", true, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((names.entries.len(), names.entries.capacity()), (8, 8));
        assert!(
            names
                .entries
                .iter()
                .all(|entry| entry.text == "a" && !entry.block_function)
        );
    }

    #[test]
    fn sorting_scratch_and_comparison_failures_use_the_same_ledger() {
        let make = || {
            let mut names = Names::default();
            let mut ledger = budget();
            names.push("z", false, &mut ledger).unwrap();
            names.push("a", true, &mut ledger).unwrap();
            (names, ledger)
        };
        let (names, mut ledger) = make();
        let allocated = ledger.allocated;
        ledger.steps = 2;
        assert!(names.finish(&mut ledger).unwrap_err().is_resource_limit());
        assert_eq!(ledger.allocated, allocated);
        let (names, mut ledger) = make();
        ledger.heap_limit = ledger.allocated + 2 * std::mem::size_of::<Entry<'_>>() + 31;
        assert!(names.finish(&mut ledger).unwrap_err().is_resource_limit());
        assert_eq!(ledger.allocated, ledger.heap_limit + 1);
        let (names, mut ledger) = make();
        ledger.steps = 4; // scratch copy, then first output; comparison cannot begin.
        assert!(names.finish(&mut ledger).unwrap_err().is_resource_limit());
        assert_eq!(ledger.steps, 0);
        let (names, mut ledger) = make();
        let initial = ledger.steps;
        let names = names.finish(&mut ledger).unwrap();
        assert_eq!(initial - ledger.steps, 6);
        assert_eq!(names.entries[0].text, "a");
        assert_eq!(names.entries[1].text, "z");
    }

    #[test]
    fn membership_duplicates_intersection_and_diagnostics_precharge_string_work() {
        let name = "x".repeat(1024);
        let mut ledger = budget();
        let left = sorted(&[(&name, false), (&name, true)], &mut ledger);
        let right = sorted(&[(&name, false)], &mut ledger);
        let allocated = ledger.allocated;
        for operation in 0..3 {
            ledger.steps = 128;
            let error = match operation {
                0 => left.contains(&name, &mut ledger).unwrap_err(),
                1 => left.first_duplicate(&mut ledger).unwrap_err(),
                _ => left.first_intersection(&right, &mut ledger).unwrap_err(),
            };
            assert!(error.is_resource_limit());
            assert_eq!((ledger.steps, ledger.allocated), (0, allocated));
        }
        ledger.steps = 129;
        assert!(left.contains(&name, &mut ledger).unwrap());
        assert_eq!(ledger.steps, 0);
        let prefix = "duplicate lexical binding '";
        let length = prefix.len() + name.len() + 1;
        ledger.steps = length / 8;
        assert!(
            named_error(prefix, &name, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(ledger.allocated, allocated);
        ledger.steps = MAX_STEPS;
        ledger.heap_limit = allocated + length + 63;
        assert!(
            named_error(prefix, &name, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        let mut ledger = budget();
        let error = named_error(prefix, &name, &mut ledger).unwrap();
        assert_eq!(error.message, format!("{prefix}{name}'"));
        assert_eq!(error.offset, Some(0));
        assert!(error.is_parse_error());
    }

    #[test]
    fn count_limit_and_trivial_lists_need_no_sort_allocation() {
        let mut ledger = budget();
        ledger.steps = 0;
        ledger.heap_limit = 0;
        let empty = Names::default().finish(&mut ledger).unwrap();
        assert!(empty.entries.is_empty());
        let mut ledger = budget();
        let mut names = Names::default();
        names.push("one", false, &mut ledger).unwrap();
        let allocated = ledger.allocated;
        ledger.steps = 0;
        ledger.heap_limit = allocated;
        assert_eq!(names.finish(&mut ledger).unwrap().entries.len(), 1);
        assert_eq!(ledger.allocated, allocated);
        let mut names = Names {
            entries: vec![
                Entry {
                    text: "x",
                    block_function: false,
                    order: 0
                };
                MAX_TOKENS
            ],
        };
        let mut ledger = budget();
        assert!(
            names
                .push("over", false, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((names.entries.len(), ledger.allocated), (MAX_TOKENS, 0));
    }
}
