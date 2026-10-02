//! Exact DOM code units with one canonical owned payload.
//!
//! Scalar parser input retains its UTF-8 buffer. Only strings containing an
//! unmatched surrogate use UTF-16 storage. Projection to replacement characters
//! is explicit and never updates the stored value.
use super::DomDataError;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Storage {
    Scalar(String),
    Units(Vec<u16>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct DomString(Storage);

impl std::fmt::Debug for DomString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            // Keep the established scalar DOM dump format. Exact units remain
            // explicit diagnostics, without a replacement-character projection.
            Storage::Scalar(value) => std::fmt::Debug::fmt(value, formatter),
            Storage::Units(value) => formatter.debug_tuple("UTF16").field(value).finish(),
        }
    }
}

#[derive(Clone)]
pub struct DomUnits<'a>(Units<'a>);

#[derive(Clone)]
pub struct DomScalars<'a>(Scalars<'a>);

#[derive(Clone)]
enum Scalars<'a> {
    Scalar(std::str::Chars<'a>),
    Exact(std::char::DecodeUtf16<std::iter::Copied<std::slice::Iter<'a, u16>>>),
}

impl Iterator for DomScalars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match &mut self.0 {
            Scalars::Scalar(values) => values.next(),
            Scalars::Exact(values) => values
                .next()
                .map(|value| value.unwrap_or(char::REPLACEMENT_CHARACTER)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            Scalars::Scalar(values) => values.size_hint(),
            Scalars::Exact(values) => values.size_hint(),
        }
    }
}

#[derive(Clone)]
enum Units<'a> {
    Scalar(std::str::EncodeUtf16<'a>),
    Exact(std::iter::Copied<std::slice::Iter<'a, u16>>),
}

impl Iterator for DomUnits<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match &mut self.0 {
            Units::Scalar(units) => units.next(),
            Units::Exact(units) => units.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            Units::Scalar(units) => units.size_hint(),
            Units::Exact(units) => units.size_hint(),
        }
    }
}

impl Default for DomString {
    fn default() -> Self {
        Self(Storage::Scalar(String::new()))
    }
}

impl From<String> for DomString {
    fn from(value: String) -> Self {
        Self(Storage::Scalar(value))
    }
}

impl From<&str> for DomString {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl PartialEq<str> for DomString {
    fn eq(&self, other: &str) -> bool {
        self.scalar() == Some(other)
    }
}

impl PartialEq<&str> for DomString {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl DomString {
    pub fn scalar(&self) -> Option<&str> {
        match &self.0 {
            Storage::Scalar(value) => Some(value),
            Storage::Units(_) => None,
        }
    }

    pub fn raw_units(&self) -> Option<&[u16]> {
        match &self.0 {
            Storage::Scalar(_) => None,
            Storage::Units(value) => Some(value),
        }
    }

    pub fn units(&self) -> DomUnits<'_> {
        DomUnits(match &self.0 {
            Storage::Scalar(value) => Units::Scalar(value.encode_utf16()),
            Storage::Units(value) => Units::Exact(value.iter().copied()),
        })
    }

    pub fn scalar_values(&self) -> DomScalars<'_> {
        DomScalars(match &self.0 {
            Storage::Scalar(value) => Scalars::Scalar(value.chars()),
            Storage::Units(value) => Scalars::Exact(char::decode_utf16(value.iter().copied())),
        })
    }

    /// Payload bytes, not an allocator-capacity or projected-output estimate.
    pub fn stored_bytes(&self) -> usize {
        match &self.0 {
            Storage::Scalar(value) => value.len(),
            // A valid Vec<u16> allocation already fits in isize::MAX bytes.
            Storage::Units(value) => value.len() * std::mem::size_of::<u16>(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Storage::Scalar(value) => value.is_empty(),
            Storage::Units(value) => value.is_empty(),
        }
    }

    /// The caller prepays both the classification scan and any scalar copy.
    /// Nonscalar input is retained without a payload allocation or copy.
    pub fn from_units_owned(units: Vec<u16>) -> Result<Self, DomDataError> {
        let mut bytes = 0usize;
        let mut nonscalar = false;
        for value in char::decode_utf16(units.iter().copied()) {
            match value {
                Ok(value) => {
                    bytes = bytes
                        .checked_add(value.len_utf8())
                        .ok_or(DomDataError::LimitExceeded)?;
                }
                Err(_) => nonscalar = true,
            }
        }
        if nonscalar {
            return Ok(Self(Storage::Units(units)));
        }
        let mut scalar = String::new();
        scalar
            .try_reserve_exact(bytes)
            .map_err(|_| DomDataError::AllocationFailed)?;
        for value in char::decode_utf16(units.iter().copied()) {
            scalar.push(value.unwrap());
        }
        Ok(scalar.into())
    }

    /// Wire admission checks bytes/canonicality before allocation; this second
    /// allocation-free validation protects the representation's sole invariant.
    pub(crate) fn from_nonscalar_units(units: Vec<u16>) -> Result<Self, DomDataError> {
        if char::decode_utf16(units.iter().copied()).any(|value| value.is_err()) {
            Ok(Self(Storage::Units(units)))
        } else {
            Err(DomDataError::InvalidData)
        }
    }

    /// Explicit replacement projection; the original unit payload is retained.
    pub fn try_to_scalar_string(&self, maximum: usize) -> Result<String, DomDataError> {
        scalar_from_units(self.units(), maximum, true, false)
    }

    pub(super) fn scalar_mut(&mut self) -> Option<&mut String> {
        match &mut self.0 {
            Storage::Scalar(value) => Some(value),
            Storage::Units(_) => None,
        }
    }
}

/// Count/admit before reserving, then emit from a repeatable exact unit stream.
/// Keeping decoding outside individual nodes permits pairs across boundaries.
pub(super) fn scalar_from_units<I>(
    units: I,
    maximum: usize,
    replace: bool,
    truncate: bool,
) -> Result<String, DomDataError>
where
    I: Iterator<Item = u16> + Clone,
{
    let scalar = |value: Result<char, std::char::DecodeUtf16Error>| match value {
        Ok(value) => Ok(value),
        Err(_) if replace => Ok(char::REPLACEMENT_CHARACTER),
        Err(_) => Err(DomDataError::InvalidData),
    };
    let mut bytes = 0usize;
    let mut count = 0usize;
    for value in char::decode_utf16(units.clone()) {
        let value = scalar(value)?;
        let next = bytes
            .checked_add(value.len_utf8())
            .ok_or(DomDataError::LimitExceeded)?;
        if next > maximum {
            if truncate {
                break;
            }
            return Err(DomDataError::LimitExceeded);
        }
        bytes = next;
        count += 1;
    }
    let mut output = String::new();
    output
        .try_reserve_exact(bytes)
        .map_err(|_| DomDataError::AllocationFailed)?;
    for value in char::decode_utf16(units).take(count) {
        output.push(scalar(value)?);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_scalar_and_nonscalar_buffers_move_without_duplicate_storage() {
        let scalar = "a\0𝄞".to_owned();
        let pointer = scalar.as_ptr();
        let text = DomString::from(scalar);
        assert_eq!(text.scalar().unwrap().as_ptr(), pointer);
        assert_eq!(text.stored_bytes(), 6);
        assert_eq!(text.units().collect::<Vec<_>>(), [97, 0, 0xd834, 0xdd1e]);
        let units = vec![0xd800, 97, 0xdc00];
        let pointer = units.as_ptr();
        let text = DomString::from_units_owned(units).unwrap();
        assert_eq!(text.raw_units().unwrap().as_ptr(), pointer);
        assert_eq!(text.stored_bytes(), 6);
        assert!(text.scalar().is_none());
        assert_eq!(text.try_to_scalar_string(7).unwrap(), "�a�");
        assert_eq!(
            text.try_to_scalar_string(6),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(text.raw_units().unwrap(), [0xd800, 97, 0xdc00]);
    }

    #[test]
    fn canonical_units_accept_every_isolated_class_and_normalize_pairs() {
        for unit in 0xd800..=0xdfff {
            let text = DomString::from_units_owned(vec![unit]).unwrap();
            assert_eq!(text.raw_units().unwrap(), [unit]);
        }
        for units in [vec![], vec![0], vec![0xd834, 0xdd1e], vec![97, 0x20ac]] {
            assert_eq!(
                DomString::from_nonscalar_units(units.clone()),
                Err(DomDataError::InvalidData)
            );
            let normalized = DomString::from_units_owned(units.clone()).unwrap();
            assert!(normalized.scalar().is_some());
            assert_eq!(normalized.units().collect::<Vec<_>>(), units);
        }
    }

    #[test]
    fn aggregate_decoder_pairs_across_stream_boundaries_and_admits_output() {
        let high = DomString::from_units_owned(vec![0xd834]).unwrap();
        let low = DomString::from_units_owned(vec![0xdd1e]).unwrap();
        let units = high.units().chain(low.units());
        assert_eq!(
            scalar_from_units(units.clone(), 4, false, false).unwrap(),
            "𝄞"
        );
        assert_eq!(
            scalar_from_units(units, 3, false, false),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(
            scalar_from_units(high.units(), 4, false, false),
            Err(DomDataError::InvalidData)
        );
        assert_eq!(scalar_from_units(high.units(), 2, true, true).unwrap(), "");
        assert_eq!(scalar_from_units(high.units(), 3, true, true).unwrap(), "�");
    }
}
