//! Shared bounded grammar for two-value inherited keyword properties.
use crate::selectors::{self, Error, Kind, Token, spend};

// Supports scanners prepay the token scan. Final grammar validation below
// checks decoded spelling; this predicate only permits escaped identifiers.
pub(super) fn escape_allowed(token: Token<'_>) -> bool {
    token.kind == Kind::Ident && !token.raw.contains('\0')
}

pub(super) fn keyword(
    value: &str,
    keywords: &[&'static str; 7],
    work: &mut usize,
) -> Result<Option<&'static str>, Error> {
    if value.len() > 4096 {
        return Err(Error::Limit);
    }
    // Prepay the significant-token pass and at most seven decoded comparisons.
    spend(work, value.len().saturating_mul(8) + 1)?;
    let tokens = selectors::tokens(value, work)?;
    let mut significant = tokens.iter().filter(|token| token.kind != Kind::Whitespace);
    let Some(token) = significant.next() else {
        return Ok(None);
    };
    if significant.next().is_some() || !escape_allowed(*token) {
        return Ok(None);
    }
    Ok(keywords.iter().copied().find(|name| token.is_name(name)))
}
