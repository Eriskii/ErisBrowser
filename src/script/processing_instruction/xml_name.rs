//! Pure XML 1.0 Fifth Edition Name validation for a later PI increment.
//!
//! Primary source: W3C Recommendation, 26 November 2008, productions [4], [4a], [5]:
//! https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NameStartChar
//! https://www.w3.org/TR/2008/REC-xml-20081126/#NT-Name
//!
//! This validates Name, not QName, NCName or the XML serialization PITarget
//! production [17]. Colons and case variants of "xml" are valid Names. It does
//! not validate PI data, perform normalization, convert to UTF-8, or publish nodes.
//! Isolated UTF-16 surrogates and scalars above U+EFFFF are rejected.

/// Checked linear precharge, to be paid before xml_name_utf16 inspects input.
/// The fixed coefficient covers at most 30 start-range comparisons, nine extra
/// continuation-range comparisons, and bounded UTF-16 decoding/loop decisions
/// per scalar. Scalar count never exceeds UTF-16 unit count. Early rejection
/// does not authorize skipping precharge; None means the bound overflowed.
/// No allocation charge is needed for this validator itself.
pub(super) fn xml_name_work(units: usize) -> Option<usize> {
    units.checked_mul(64)?.checked_add(8)
}

/// Allocation-free O(units.len()) predicate. The caller owns work admission.
/// A JsString caller can borrow its existing units() slice directly.
pub(super) fn xml_name_utf16(units: &[u16]) -> bool {
    let mut scalars = char::decode_utf16(units.iter().copied());
    let Some(Ok(first)) = scalars.next() else {
        return false;
    };
    if !xml_name_start(u32::from(first)) {
        return false;
    }
    for scalar in scalars {
        let Ok(scalar) = scalar else {
            return false;
        };
        if !xml_name_char(u32::from(scalar)) {
            return false;
        }
    }
    true
}

fn xml_name_start(scalar: u32) -> bool {
    matches!(
        scalar,
        0x003a
            | 0x0041..=0x005a
            | 0x005f
            | 0x0061..=0x007a
            | 0x00c0..=0x00d6
            | 0x00d8..=0x00f6
            | 0x00f8..=0x02ff
            | 0x0370..=0x037d
            | 0x037f..=0x1fff
            | 0x200c..=0x200d
            | 0x2070..=0x218f
            | 0x2c00..=0x2fef
            | 0x3001..=0xd7ff
            | 0xf900..=0xfdcf
            | 0xfdf0..=0xfffd
            | 0x10000..=0xeffff
    )
}

fn xml_name_char(scalar: u32) -> bool {
    xml_name_start(scalar)
        || matches!(
            scalar,
            0x002d | 0x002e | 0x0030..=0x0039 | 0x00b7 | 0x0300..=0x036f | 0x203f..=0x2040
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_name_ascii_syntax_is_name_not_qname_or_pi_serialization() {
        for units in [
            &[0x3a][..],               // :
            &[0x61, 0x3a, 0x3a, 0x62], // a::b
            &[0x5f, 0x30],             // _0
            &[0x78, 0x6d, 0x6c],       // xml
            &[0x58, 0x4d, 0x4c],       // XML
            &[0x78, 0x4d, 0x6c],       // xMl
            &[0x61, 0x2d, 0x2e, 0x39], // a-.9
        ] {
            assert!(xml_name_utf16(units), "{units:x?}");
        }
        for units in [
            &[][..],
            &[0x30],
            &[0x2d],
            &[0x2e],
            &[0x20, 0x61],
            &[0x61, 0x20],
            &[0x61, 0x2f],
            &[0x61, 0x3f],
            &[0x61, 0x3e],
            &[0x61, 0x0],
            &[0x61, 0x9],
            &[0x61, 0xa],
            &[0x61, 0xd],
        ] {
            assert!(!xml_name_utf16(units), "{units:x?}");
        }
    }

    #[test]
    fn xml_name_bmp_literal_start_boundaries_and_holes() {
        // Independently literal boundary expectations, not derived by walking
        // the production predicates or their range table.
        for (unit, allowed) in [
            (0x0039, false),
            (0x003a, true),
            (0x003b, false),
            (0x0040, false),
            (0x0041, true),
            (0x005a, true),
            (0x005b, false),
            (0x005e, false),
            (0x005f, true),
            (0x0060, false),
            (0x0061, true),
            (0x007a, true),
            (0x007b, false),
            (0x00bf, false),
            (0x00c0, true),
            (0x00d6, true),
            (0x00d7, false),
            (0x00d8, true),
            (0x00f6, true),
            (0x00f7, false),
            (0x00f8, true),
            (0x02ff, true),
            (0x0300, false),
            (0x036f, false),
            (0x0370, true),
            (0x037d, true),
            (0x037e, false),
            (0x037f, true),
            (0x1fff, true),
            (0x2000, false),
            (0x200b, false),
            (0x200c, true),
            (0x200d, true),
            (0x200e, false),
            (0x206f, false),
            (0x2070, true),
            (0x218f, true),
            (0x2190, false),
            (0x2bff, false),
            (0x2c00, true),
            (0x2fef, true),
            (0x2ff0, false),
            (0x3000, false),
            (0x3001, true),
            (0xd7ff, true),
            (0xd800, false),
            (0xdbff, false),
            (0xdc00, false),
            (0xdfff, false),
            (0xe000, false),
            (0xf8ff, false),
            (0xf900, true),
            (0xfdcf, true),
            (0xfdd0, false),
            (0xfdef, false),
            (0xfdf0, true),
            (0xfffd, true),
            (0xfffe, false),
            (0xffff, false),
        ] {
            assert_eq!(xml_name_utf16(&[unit]), allowed, "U+{unit:04X}");
        }
    }

    #[test]
    fn xml_name_continuation_only_literals_and_adjacent_gaps() {
        for unit in [
            0x002d, 0x002e, 0x0030, 0x0039, 0x00b7, 0x0300, 0x036f, 0x203f, 0x2040,
        ] {
            assert!(!xml_name_utf16(&[unit]), "initial U+{unit:04X}");
            assert!(xml_name_utf16(&[0x61, unit]), "continuation U+{unit:04X}");
        }
        for unit in [0x002c, 0x002f, 0x00b6, 0x00b8, 0x037e, 0x203e, 0x2041] {
            assert!(!xml_name_utf16(&[0x61, unit]), "U+{unit:04X}");
        }
        // Adjacent 0x02ff and 0x0370 are already NameStartChar, so must not be
        // accidentally rejected as holes next to the combining-mark interval.
        assert!(xml_name_utf16(&[0x61, 0x02ff, 0x0370]));
    }

    #[test]
    fn xml_name_supplementary_literal_limit_and_noncharacters() {
        for (pair, allowed) in [
            ([0xd800, 0xdc00], true),  // U+10000
            ([0xd83f, 0xdfff], true),  // U+1FFFF: discouraged, not excluded
            ([0xdb7f, 0xdffe], true),  // U+EFFFE
            ([0xdb7f, 0xdfff], true),  // U+EFFFF: final admitted scalar
            ([0xdb80, 0xdc00], false), // U+F0000
            ([0xdbff, 0xdfff], false), // U+10FFFF
        ] {
            assert_eq!(xml_name_utf16(&pair), allowed, "{pair:x?}");
            assert_eq!(xml_name_utf16(&[0x61, pair[0], pair[1]]), allowed);
        }
    }

    #[test]
    fn xml_name_malformed_utf16_is_never_lossily_replaced() {
        for units in [
            &[0xd800][..],
            &[0xdc00],
            &[0xd800, 0x61],
            &[0xd800, 0xd800],
            &[0xdc00, 0xd800],
            &[0x61, 0xd800],
            &[0x61, 0xdc00],
            &[0xd800, 0xdc00, 0xdc00],
        ] {
            assert!(!xml_name_utf16(units), "{units:x?}");
        }
        // Replacement U+FFFD is itself admitted; rejecting a malformed unit
        // above therefore proves that lossy decoding is not equivalent.
        assert!(xml_name_utf16(&[0xfffd]));
    }

    #[test]
    fn xml_name_work_preflight_is_checked_without_reading_input() {
        assert_eq!(xml_name_work(0), Some(8));
        assert_eq!(xml_name_work(1), Some(72));
        assert_eq!(xml_name_work(2), Some(136));
        assert_eq!(xml_name_work(4096), Some(262_152));
        let last = (usize::MAX - 8) / 64;
        assert!(xml_name_work(last).is_some());
        assert_eq!(xml_name_work(last + 1), None);
        assert_eq!(xml_name_work(usize::MAX), None);
    }
}
