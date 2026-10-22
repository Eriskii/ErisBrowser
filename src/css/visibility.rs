//! Inherited visibility for boxes that retain their layout contribution.
use super::keyword_value;
use crate::selectors::Error;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Visible = 0,
    Hidden = 1,
}

const KEYWORDS: [&str; 7] = [
    "visible",
    "hidden",
    "inherit",
    "initial",
    "unset",
    "revert",
    "revert-layer",
];

pub(crate) fn valid(value: &str, work: &mut usize) -> Result<bool, Error> {
    Ok(keyword_value::keyword(value, &KEYWORDS, work)?.is_some())
}

pub(super) fn parse(value: &str, work: &mut usize) -> Result<Option<Visibility>, Error> {
    Ok(match keyword_value::keyword(value, &KEYWORDS, work)? {
        Some("visible") => Some(Visibility::Visible),
        Some("hidden") => Some(Visibility::Hidden),
        _ => None,
    })
}
