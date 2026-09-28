//! ECMAScript strings are sequences of UTF-16 code units, including surrogates.
//! Runtime entry points account for and limit their storage. UTF-8 conversion
//! is explicit: scalar strings round-trip, while host display/DOM boundaries
//! replace unpaired surrogates with U+FFFD.

use std::{fmt, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsString(Rc<[u16]>);

impl JsString {
    pub fn units(&self) -> &[u16] {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn byte_len(&self) -> usize {
        self.len().saturating_mul(2)
    }
    pub fn to_utf8(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(&self.0)
    }
    pub fn to_utf8_lossy(&self) -> String {
        String::from_utf16_lossy(&self.0)
    }
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self::from(&self.0[start..end])
    }
    pub fn trimmed_units(&self) -> &[u16] {
        let start = self
            .0
            .iter()
            .position(|u| !is_js_whitespace(*u))
            .unwrap_or(self.len());
        let end = self
            .0
            .iter()
            .rposition(|u| !is_js_whitespace(*u))
            .map(|i| i + 1)
            .unwrap_or(start);
        &self.0[start..end]
    }
    pub fn number(&self) -> f64 {
        let units = self.trimmed_units();
        if units.is_empty() {
            return 0.0;
        }
        if units.iter().any(|unit| *unit > 0x7f) {
            return f64::NAN;
        }
        let text: String = units.iter().map(|u| char::from(*u as u8)).collect();
        match text.as_str() {
            "Infinity" | "+Infinity" => return f64::INFINITY,
            "-Infinity" => return f64::NEG_INFINITY,
            _ => {}
        }
        let bytes = text.as_bytes();
        if bytes.len() > 2 && bytes[0] == b'0' {
            let shift = match bytes[1] {
                b'x' | b'X' => 4,
                b'o' | b'O' => 3,
                b'b' | b'B' => 1,
                _ => 0,
            };
            if shift != 0 {
                return radix_number(&bytes[2..], shift);
            }
        }
        let mut i = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
        let before = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        let mut digits = i - before;
        if bytes.get(i) == Some(&b'.') {
            i += 1;
            let start = i;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            digits += i - start;
        }
        if digits == 0 {
            return f64::NAN;
        }
        if matches!(bytes.get(i), Some(b'e' | b'E')) {
            i += 1;
            if matches!(bytes.get(i), Some(b'+' | b'-')) {
                i += 1;
            }
            let start = i;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            if start == i {
                return f64::NAN;
            }
        }
        if i != bytes.len() {
            return f64::NAN;
        }
        text.parse().unwrap_or(f64::NAN)
    }
}

impl From<&str> for JsString {
    fn from(text: &str) -> Self {
        Self(text.encode_utf16().collect::<Vec<_>>().into())
    }
}
impl From<String> for JsString {
    fn from(text: String) -> Self {
        Self::from(text.as_str())
    }
}
impl From<&String> for JsString {
    fn from(text: &String) -> Self {
        Self::from(text.as_str())
    }
}
impl From<Vec<u16>> for JsString {
    fn from(units: Vec<u16>) -> Self {
        Self(units.into())
    }
}
impl From<&[u16]> for JsString {
    fn from(units: &[u16]) -> Self {
        Self(Rc::from(units))
    }
}
impl From<&JsString> for JsString {
    fn from(text: &JsString) -> Self {
        text.clone()
    }
}
impl fmt::Display for JsString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for scalar in char::decode_utf16(self.0.iter().copied()) {
            write!(f, "{}", scalar.unwrap_or(char::REPLACEMENT_CHARACTER))?;
        }
        Ok(())
    }
}

pub(crate) fn is_js_whitespace(unit: u16) -> bool {
    matches!(unit,0x0009..=0x000d|0x0020|0x00a0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}

pub(crate) fn radix_number(digits: &[u8], shift: usize) -> f64 {
    let radix = 1u32 << shift;
    let mut first = None;
    for (i, byte) in digits.iter().enumerate() {
        let Some(digit) = (*byte as char).to_digit(radix) else {
            return f64::NAN;
        };
        if digit != 0 && first.is_none() {
            first = Some((i, digit));
        }
    }
    let Some((start, first)) = first else {
        return 0.0;
    };
    let first_bits = (32 - first.leading_zeros()) as usize;
    let bit_len = first_bits + (digits.len() - start - 1) * shift;
    if bit_len > 1024 {
        return f64::INFINITY;
    }
    let mut mantissa = 0u64;
    let mut seen = 0usize;
    let mut guard = false;
    let mut sticky = false;
    for (i, byte) in digits[start..].iter().enumerate() {
        let digit = (*byte as char).to_digit(radix).unwrap();
        for bit in (0..if i == 0 { first_bits } else { shift }).rev() {
            let one = digit & (1 << bit) != 0;
            if seen < 53 {
                mantissa = (mantissa << 1) | u64::from(one);
            } else if seen == 53 {
                guard = one;
            } else {
                sticky |= one;
            }
            seen += 1;
        }
    }
    if bit_len <= 53 {
        return mantissa as f64;
    }
    if guard && (sticky || mantissa & 1 == 1) {
        mantissa += 1;
    }
    mantissa as f64 * 2f64.powi((bit_len - 53) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn code_units_preserve_surrogates_and_utf8_boundaries_are_explicit() {
        let text = JsString::from(vec![0xd800, 0xd83e, 0xdd80, 0xdc00]);
        assert_eq!(text.len(), 4);
        assert!(text.to_utf8().is_err());
        assert_eq!(text.to_utf8_lossy(), "�🦀�");
        assert_eq!(JsString::from("🦀").units(), [0xd83e, 0xdd80]);
        assert!(JsString::from("🦀") < JsString::from("\u{e000}"));
    }
    #[test]
    fn string_numeric_grammar_and_radix_rounding_are_explicit() {
        for (text, want) in [
            ("\u{feff} 12\u{a0}", 12.0),
            (".5", 0.5),
            ("+1.", 1.0),
            ("0x20000000000001", 9007199254740992.0),
            ("0x20000000000003", 9007199254740996.0),
            ("0b101", 5.0),
            ("0o77", 63.0),
        ] {
            assert_eq!(JsString::from(text).number(), want, "{text}");
        }
        for text in [
            "inf", "infinity", "+0x1", "-0b1", "0x", "0b2", "1e+", "\u{85}1",
        ] {
            assert!(JsString::from(text).number().is_nan(), "{text}");
        }
        assert!(JsString::from(vec![0xd800]).number().is_nan());
        assert!(JsString::from("-0").number().is_sign_negative());
    }
}
