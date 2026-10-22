//! Inherited point-targeting eligibility; independent of paint and focus.
use super::keyword_value;
use crate::selectors::Error;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PointerEvents {
    #[default]
    Auto = 0,
    None = 1,
}

const KEYWORDS: [&str; 7] = [
    "auto",
    "none",
    "inherit",
    "initial",
    "unset",
    "revert",
    "revert-layer",
];

pub(crate) fn valid(value: &str, work: &mut usize) -> Result<bool, Error> {
    Ok(keyword_value::keyword(value, &KEYWORDS, work)?.is_some())
}

pub(super) fn parse(value: &str, work: &mut usize) -> Result<Option<PointerEvents>, Error> {
    Ok(match keyword_value::keyword(value, &KEYWORDS, work)? {
        Some("auto") => Some(PointerEvents::Auto),
        Some("none") => Some(PointerEvents::None),
        _ => None,
    })
}
