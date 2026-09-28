//! Bounded token pages avoid repeatedly relocating the complete token prefix.
use super::{MAX_TOKENS, Result, ScriptError, Token, compile_allocate, regexp, regexp_error};

const PAGE: usize = 128;
const LIMIT: usize = MAX_TOKENS + 2;
const PAGES: usize = LIMIT.div_ceil(PAGE);

#[derive(Debug, Default)]
pub(super) struct Tokens {
    pages: Vec<Vec<Token>>,
    length: usize,
}
impl Tokens {
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn get(&self, index: usize) -> Option<&Token> {
        if index < self.length {
            self.pages[index / PAGE].get(index % PAGE)
        } else {
            None
        }
    }
    #[cfg(test)]
    pub fn iter(&self) -> impl Iterator<Item = &Token> {
        self.pages.iter().flatten()
    }
    pub fn range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = &Token> {
        range.map(|index| &self[index])
    }
    pub fn reserve(&mut self, additional: usize, budget: &mut regexp::Budget) -> Result<()> {
        let needed = self.length.saturating_add(additional);
        if needed > LIMIT {
            return Err(ScriptError::resource("script token limit exceeded"));
        }
        let pages = needed.div_ceil(PAGE);
        while self.pages.len() < pages {
            budget.work(1).map_err(regexp_error)?;
            if self.pages.len() == self.pages.capacity() {
                let capacity = self.pages.capacity().saturating_mul(2).clamp(4, PAGES);
                budget.work(self.pages.len() + 1).map_err(regexp_error)?;
                compile_allocate(budget, capacity * std::mem::size_of::<Vec<Token>>() + 32)?;
                self.pages
                    .try_reserve_exact(capacity - self.pages.len())
                    .map_err(|_| ScriptError::resource("script token page allocation failed"))?;
            }
            let capacity = PAGE.min(LIMIT - self.pages.len() * PAGE);
            compile_allocate(budget, capacity * std::mem::size_of::<Token>() + 32)?;
            let mut page = Vec::new();
            page.try_reserve_exact(capacity)
                .map_err(|_| ScriptError::resource("script token allocation failed"))?;
            self.pages.push(page);
        }
        Ok(())
    }
    pub fn push(&mut self, token: Token) {
        let page = &mut self.pages[self.length / PAGE];
        assert!(page.len() < page.capacity());
        page.push(token);
        self.length += 1;
    }
    pub fn truncate(&mut self, length: usize) {
        if length >= self.length {
            return;
        }
        self.pages.truncate(length.div_ceil(PAGE));
        if let Some(last) = self.pages.last_mut() {
            last.truncate(if length.is_multiple_of(PAGE) {
                PAGE
            } else {
                length % PAGE
            });
        }
        self.length = length;
    }
}
impl std::ops::Index<usize> for Tokens {
    type Output = Token;
    fn index(&self, index: usize) -> &Token {
        self.get(index).expect("token index in bounds")
    }
}
impl std::ops::IndexMut<usize> for Tokens {
    fn index_mut(&mut self, index: usize) -> &mut Token {
        assert!(index < self.length);
        &mut self.pages[index / PAGE][index % PAGE]
    }
}
impl IntoIterator for Tokens {
    type Item = Token;
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Vec<Token>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.pages.into_iter().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{MAX_HEAP, MAX_STEPS, TokenKind};
    fn budget() -> regexp::Budget {
        regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        }
    }
    fn token(offset: usize) -> Token {
        Token {
            kind: TokenKind::End,
            offset,
            line_break_before: false,
            string_literal: false,
            use_strict: false,
            legacy_literal: false,
        }
    }
    #[test]
    fn pages_keep_existing_records_stable_and_iterate_across_boundaries() {
        let mut tokens = Tokens::default();
        let mut ledger = budget();
        tokens.reserve(1, &mut ledger).unwrap();
        tokens.push(token(0));
        let first = &tokens[0] as *const Token;
        for i in 1..1_025 {
            tokens.reserve(1, &mut ledger).unwrap();
            tokens.push(token(i));
        }
        assert_eq!(first, &tokens[0] as *const Token);
        assert_eq!(
            tokens.range(125..260).map(|t| t.offset).collect::<Vec<_>>(),
            (125..260).collect::<Vec<_>>()
        );
        tokens[128].legacy_literal = true;
        assert!(tokens.get(128).unwrap().legacy_literal);
        assert!(tokens.get(1_025).is_none());
        assert!(tokens.get(usize::MAX).is_none());
        assert_eq!(
            tokens.into_iter().map(|t| t.offset).collect::<Vec<_>>(),
            (0..1_025).collect::<Vec<_>>()
        );
    }
    #[test]
    fn truncation_reuses_prefix_pages_without_refunding_cumulative_storage() {
        for keep in [0, 1, 127, 128, 129, 255, 256, 257] {
            let mut tokens = Tokens::default();
            let mut ledger = budget();
            tokens.reserve(385, &mut ledger).unwrap();
            for i in 0..385 {
                tokens.push(token(i));
            }
            let allocated = ledger.allocated;
            tokens.truncate(keep);
            assert_eq!(tokens.len(), keep);
            assert_eq!(ledger.allocated, allocated);
            tokens.reserve(260, &mut ledger).unwrap();
            for i in 0..260 {
                tokens.push(token(1_000 + i));
            }
            let expected = (0..keep).chain(1_000..1_260).collect::<Vec<_>>();
            assert_eq!(
                tokens.into_iter().map(|t| t.offset).collect::<Vec<_>>(),
                expected
            );
        }
    }
    #[test]
    fn page_growth_refuses_work_storage_and_count_before_retaining_tokens() {
        let mut tokens = Tokens::default();
        let mut ledger = budget();
        ledger.steps = 0;
        assert!(
            tokens
                .reserve(1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (tokens.len(), tokens.pages.capacity(), ledger.allocated),
            (0, 0, 0)
        );
        let mut ledger = budget();
        ledger.heap_limit = 4 * std::mem::size_of::<Vec<Token>>() + 31;
        assert!(
            tokens
                .reserve(1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((tokens.len(), tokens.pages.capacity()), (0, 0));
        let mut ledger = budget();
        ledger.heap_limit =
            4 * std::mem::size_of::<Vec<Token>>() + 32 + PAGE * std::mem::size_of::<Token>() + 31;
        assert!(
            tokens
                .reserve(1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(tokens.pages.is_empty());
        assert_eq!(tokens.len(), 0);
        let mut tokens = Tokens::default();
        let mut ledger = budget();
        tokens.reserve(LIMIT, &mut ledger).unwrap();
        assert_eq!(tokens.pages.len(), PAGES);
        assert_eq!(tokens.pages.last().unwrap().capacity(), 2);
        for i in 0..LIMIT {
            tokens.push(token(i));
        }
        let before = (ledger.steps, ledger.allocated);
        assert!(
            tokens
                .reserve(1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((ledger.steps, ledger.allocated), before);
        assert_eq!(tokens.len(), LIMIT);
        tokens.truncate(LIMIT - 1);
        tokens.reserve(1, &mut ledger).unwrap();
        tokens.push(token(99));
        assert_eq!(tokens[LIMIT - 1].offset, 99);
    }
}
