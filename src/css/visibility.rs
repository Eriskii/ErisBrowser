//! Inherited visibility for boxes that retain their layout contribution.
use crate::selectors::{self, Error, Kind, Token, spend};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Visible = 0,
    Hidden = 1,
}

// Supports scanners prepay the token scan. Final grammar validation below
// checks decoded spelling; this predicate only permits escaped identifiers.
pub(super) fn escape_allowed(token: Token<'_>) -> bool {
    token.kind == Kind::Ident && !token.raw.contains('\0')
}

fn keyword(value: &str, work: &mut usize) -> Result<Option<&'static str>, Error> {
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
    Ok([
        "visible",
        "hidden",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ]
    .into_iter()
    .find(|name| token.is_name(name)))
}

pub(crate) fn valid(value: &str, work: &mut usize) -> Result<bool, Error> {
    Ok(keyword(value, work)?.is_some())
}

pub(super) fn parse(value: &str, work: &mut usize) -> Result<Option<Visibility>, Error> {
    Ok(match keyword(value, work)? {
        Some("visible") => Some(Visibility::Visible),
        Some("hidden") => Some(Visibility::Hidden),
        _ => None,
    })
}
