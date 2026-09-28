//! HTML byte decoding and bounded encoding declarations. Codec tables and
//! replacement behavior come from encoding_rs; selection follows HTML/Encoding.
//! https://html.spec.whatwg.org/multipage/parsing.html#determining-the-character-encoding
use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252, X_USER_DEFINED};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
pub(crate) struct HtmlEncoding {
    pub encoding: &'static Encoding,
    pub certain: bool,
}

pub(crate) fn html_encoding(bytes: &[u8], content_type: &str) -> HtmlEncoding {
    if let Some((encoding, _)) = Encoding::for_bom(bytes) {
        return HtmlEncoding {
            encoding,
            certain: true,
        };
    }
    if let Some(encoding) = transport_encoding(content_type) {
        return HtmlEncoding {
            encoding,
            certain: true,
        };
    }
    HtmlEncoding {
        encoding: prescan(&bytes[..bytes.len().min(1024)]).unwrap_or(WINDOWS_1252),
        certain: false,
    }
}

pub(crate) fn decode(bytes: &[u8], encoding: &'static Encoding) -> String {
    encoding.decode(bytes).0.into_owned()
}

/// MIME parameter parsing respects quoted semicolons/escapes and the first
/// charset parameter. An unknown first label cannot be replaced by a duplicate.
pub(crate) fn transport_encoding(content_type: &str) -> Option<&'static Encoding> {
    let bytes = content_type
        .trim_matches(|c: char| c.is_ascii() && http_space(c as u8))
        .as_bytes();
    let essence_end = bytes.iter().position(|&b| b == b';').unwrap_or(bytes.len());
    let mut essence = &bytes[..essence_end];
    while essence.last().is_some_and(|&b| http_space(b)) {
        essence = &essence[..essence.len() - 1];
    }
    let slash = essence.iter().position(|&b| b == b'/')?;
    if slash == 0
        || slash + 1 == essence.len()
        || !essence[..slash]
            .iter()
            .chain(&essence[slash + 1..])
            .all(|&b| http_token(b))
    {
        return None;
    }
    let mut at = bytes.iter().position(|&b| b == b';')? + 1;
    while at < bytes.len() {
        while bytes.get(at).is_some_and(|&b| http_space(b) || b == b';') {
            at += 1;
        }
        let start = at;
        while bytes.get(at).is_some_and(|&b| b != b'=' && b != b';') {
            at += 1;
        }
        if bytes.get(at) != Some(&b'=') {
            continue;
        }
        let name = &bytes[start..at];
        let wanted = name.eq_ignore_ascii_case(b"charset");
        at += 1;
        // Spaces after '=' are part of an unquoted parameter and trimmed below.
        let quoted = bytes.get(at) == Some(&b'"');
        let mut value = Vec::new();
        if quoted {
            at += 1;
            while let Some(&b) = bytes.get(at) {
                at += 1;
                if b == b'"' {
                    break;
                }
                let b = if b == b'\\' && at < bytes.len() {
                    at += 1;
                    bytes[at - 1]
                } else {
                    b
                };
                if wanted {
                    value.push(b);
                }
            }
            while bytes.get(at).is_some_and(|&b| b != b';') {
                at += 1;
            }
        } else {
            let start = at;
            while bytes.get(at).is_some_and(|&b| b != b';') {
                at += 1;
            }
            if wanted {
                let mut slice = &bytes[start..at];
                while slice.last().is_some_and(|&b| http_space(b)) {
                    slice = &slice[..slice.len() - 1];
                }
                if slice.is_empty() {
                    continue;
                }
                value.extend_from_slice(slice);
            }
        }
        if wanted {
            // Invalid MIME parameter values do not occupy the parameter map.
            // Valid but unrecognized labels do, preventing duplicate overrides.
            if String::from_utf8_lossy(&value)
                .chars()
                .all(|c| matches!(c, '\t' | ' '..='~' | '\u{80}'..='\u{ff}'))
            {
                return Encoding::for_label(&value);
            }
        }
    }
    None
}
fn http_token(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

pub(crate) fn css_encoding(
    bytes: &[u8],
    content_type: &str,
    environment: &'static Encoding,
) -> &'static Encoding {
    Encoding::for_bom(bytes)
        .map(|value| value.0)
        .or_else(|| transport_encoding(content_type))
        .or_else(|| {
            let bytes = bytes
                .get(..bytes.len().min(1024))?
                .strip_prefix(b"@charset \"")?;
            let end = bytes.iter().position(|&b| b == b'"')?;
            if bytes.get(end + 1) != Some(&b';') || !bytes[..end].is_ascii() {
                return None;
            }
            Encoding::for_label(&bytes[..end]).map(|encoding| {
                if encoding == UTF_16LE || encoding == UTF_16BE {
                    UTF_8
                } else {
                    encoding
                }
            })
        })
        .unwrap_or(environment)
}

pub(crate) fn script_encoding(
    bytes: &[u8],
    content_type: &str,
    charset: Option<&str>,
    environment: &'static Encoding,
) -> &'static Encoding {
    Encoding::for_bom(bytes)
        .map(|value| value.0)
        .or_else(|| transport_encoding(content_type))
        .or_else(|| charset.and_then(|label| Encoding::for_label(label.as_bytes())))
        .unwrap_or(environment)
}
fn http_space(b: u8) -> bool {
    matches!(b, b'\t' | b'\n' | b'\r' | b' ')
}
fn space(b: u8) -> bool {
    http_space(b) || b == 0x0c
}

/// Actual HTML meta processing (after tokenization), unlike the byte prescan,
/// has already decoded character references in attribute values.
pub(crate) fn meta_encoding(
    charset: Option<&str>,
    http_equiv: Option<&str>,
    content: Option<&str>,
) -> Option<&'static Encoding> {
    let encoding = charset
        .and_then(|label| Encoding::for_label(label.as_bytes()))
        .or_else(|| {
            http_equiv
                .is_some_and(|s| s.eq_ignore_ascii_case("content-type"))
                .then(|| content.and_then(|value| extract_meta_encoding(value.as_bytes())))
                .flatten()
        })?;
    Some(normalize_meta(encoding))
}
fn normalize_meta(encoding: &'static Encoding) -> &'static Encoding {
    if encoding == UTF_16LE || encoding == UTF_16BE {
        UTF_8
    } else if encoding == X_USER_DEFINED {
        WINDOWS_1252
    } else {
        encoding
    }
}
fn extract_meta_encoding(value: &[u8]) -> Option<&'static Encoding> {
    let mut at = 0;
    while at + 7 <= value.len() {
        if !value[at..at + 7].eq_ignore_ascii_case(b"charset") {
            at += 1;
            continue;
        }
        at += 7;
        while value.get(at).is_some_and(|&b| space(b)) {
            at += 1;
        }
        if value.get(at) != Some(&b'=') {
            continue;
        }
        at += 1;
        while value.get(at).is_some_and(|&b| space(b)) {
            at += 1;
        }
        let first = *value.get(at)?;
        if first == b'\'' || first == b'"' {
            let start = at + 1;
            let length = value[start..].iter().position(|&b| b == first)?;
            return Encoding::for_label(&value[start..start + length]);
        }
        let start = at;
        while value.get(at).is_some_and(|&b| !space(b) && b != b';') {
            at += 1;
        }
        return Encoding::for_label(&value[start..at]);
    }
    None
}

fn prescan(bytes: &[u8]) -> Option<&'static Encoding> {
    if bytes.starts_with(b"<\0?\0x\0") {
        return Some(UTF_16LE);
    }
    if bytes.starts_with(b"\0<\0?\0x") {
        return Some(UTF_16BE);
    }
    prescan_meta(bytes).or_else(|| xml_declaration(bytes))
}
fn prescan_meta(bytes: &[u8]) -> Option<&'static Encoding> {
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at..].starts_with(b"<!--") {
            // '-->' may share its dashes with the opening sequence: <!-->.
            let end = bytes[at + 2..].windows(3).position(|w| w == b"-->")?;
            at += 2 + end + 3;
            continue;
        }
        if bytes
            .get(at..at + 5)
            .is_some_and(|s| s.eq_ignore_ascii_case(b"<meta"))
            && bytes.get(at + 5).is_some_and(|&b| space(b) || b == b'/')
        {
            at += 5;
            let mut seen = BTreeSet::new();
            let mut pragma = false;
            let mut need_pragma = None;
            let mut charset = None;
            let mut charset_seen = false;
            while let Some((name, value)) = attribute(bytes, &mut at).ok()? {
                if !seen.insert(name.clone()) {
                    continue;
                }
                match name.as_slice() {
                    b"http-equiv" => pragma = value == b"content-type",
                    b"content" if !charset_seen => {
                        if let Some(encoding) = extract_meta_encoding(&value) {
                            charset = Some(encoding);
                            charset_seen = true;
                            need_pragma = Some(true);
                        }
                    }
                    b"charset" => {
                        charset = Encoding::for_label(&value);
                        charset_seen = true;
                        need_pragma = Some(false);
                    }
                    _ => {}
                }
            }
            if (need_pragma == Some(false) || need_pragma == Some(true) && pragma)
                && let Some(encoding) = charset
            {
                return Some(normalize_meta(encoding));
            }
        } else if bytes[at] == b'<' {
            let name = at + 1 + usize::from(bytes.get(at + 1) == Some(&b'/'));
            if bytes.get(name).is_some_and(u8::is_ascii_alphabetic) {
                at = name;
                while bytes.get(at).is_some_and(|&b| !space(b) && b != b'>') {
                    at += 1;
                }
                while attribute(bytes, &mut at).ok()?.is_some() {}
            } else if bytes.get(at + 1).is_some_and(|b| b"!/?".contains(b)) {
                at += bytes[at..].iter().position(|&b| b == b'>')?;
            }
        }
        at += 1;
    }
    None
}

/// Err means the scan ended inside an attribute; None means a complete tag end.
type ByteAttribute = (Vec<u8>, Vec<u8>);
fn attribute(bytes: &[u8], at: &mut usize) -> Result<Option<ByteAttribute>, ()> {
    while bytes.get(*at).is_some_and(|&b| space(b) || b == b'/') {
        *at += 1;
    }
    if *bytes.get(*at).ok_or(())? == b'>' {
        return Ok(None);
    }
    let mut name = Vec::new();
    loop {
        let b = *bytes.get(*at).ok_or(())?;
        if b == b'=' && !name.is_empty() {
            *at += 1;
            break;
        }
        if b == b'/' || b == b'>' {
            return Ok(Some((name, Vec::new())));
        }
        if space(b) {
            while bytes.get(*at).is_some_and(|&b| space(b)) {
                *at += 1;
            }
            if *bytes.get(*at).ok_or(())? != b'=' {
                return Ok(Some((name, Vec::new())));
            }
            *at += 1;
            break;
        }
        name.push(b.to_ascii_lowercase());
        *at += 1;
    }
    while bytes.get(*at).is_some_and(|&b| space(b)) {
        *at += 1;
    }
    let first = *bytes.get(*at).ok_or(())?;
    let mut value = Vec::new();
    if first == b'\'' || first == b'"' {
        *at += 1;
        loop {
            let b = *bytes.get(*at).ok_or(())?;
            *at += 1;
            if b == first {
                return Ok(Some((name, value)));
            }
            value.push(b.to_ascii_lowercase());
        }
    }
    if first == b'>' {
        return Ok(Some((name, value)));
    }
    loop {
        let b = *bytes.get(*at).ok_or(())?;
        if space(b) || b == b'>' {
            return Ok(Some((name, value)));
        }
        value.push(b.to_ascii_lowercase());
        *at += 1;
    }
}
fn xml_declaration(bytes: &[u8]) -> Option<&'static Encoding> {
    if !bytes.starts_with(b"<?xml") {
        return None;
    }
    let end = bytes.iter().position(|&b| b == b'>')?;
    let declaration = &bytes[..end];
    let mut at = declaration.windows(8).position(|w| w == b"encoding")? + 8;
    while declaration.get(at).is_some_and(|&b| b <= 0x20) {
        at += 1;
    }
    if declaration.get(at) != Some(&b'=') {
        return None;
    }
    at += 1;
    while declaration.get(at).is_some_and(|&b| b <= 0x20) {
        at += 1;
    }
    let quote = *declaration.get(at)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    at += 1;
    let length = declaration[at..].iter().position(|&b| b == quote)?;
    let label = &declaration[at..at + length];
    if label.iter().any(|&b| b <= 0x20) {
        return None;
    }
    let encoding = Encoding::for_label(label)?;
    Some(if encoding == UTF_16LE || encoding == UTF_16BE {
        UTF_8
    } else {
        encoding
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use encoding_rs::{SHIFT_JIS, WINDOWS_1251};

    #[test]
    fn bom_transport_and_html_prescan_have_ordered_precedence() {
        let bytes = b"\xef\xbb\xbf<meta charset=windows-1251><p>\xc3\xa9";
        let selected = html_encoding(bytes, "text/html; charset=shift_jis");
        assert_eq!(selected.encoding, UTF_8);
        assert!(selected.certain);
        assert!(decode(bytes, selected.encoding).ends_with('é'));
        let selected = html_encoding(&bytes[3..], "text/html; charset=shift_jis");
        assert_eq!(selected.encoding, SHIFT_JIS);
        assert!(selected.certain);
        let selected = html_encoding(&bytes[3..], "text/html; charset=nonsense");
        assert_eq!(selected.encoding, WINDOWS_1251);
        assert!(!selected.certain);
        assert_eq!(
            html_encoding(b"<p>\xe9", "text/html").encoding,
            WINDOWS_1252
        );
    }

    #[test]
    fn mime_quoted_parameters_duplicates_and_invalid_tokens() {
        for header in [
            "text/html; a=\"not;charset=shift_jis\"; charset=\"utf-8\"",
            "text/html; charset=\"utf\\-8\"trailing",
            "text/html; charset=; charset=utf-8",
            "text/html; charset =shift_jis; charset=utf-8",
            "text/html; charset=\"bad\rlabel\"; charset=utf-8",
        ] {
            assert_eq!(transport_encoding(header), Some(UTF_8), "{header:?}");
        }
        for header in [
            "text/html; charset=unknown; charset=utf-8",
            "text/html; charset=\"\"; charset=utf-8",
            "text/html; charset= 'utf-8'",
            "text/html; charset= \"utf-8\"",
            "text; charset=utf-8",
            "text /html; charset=utf-8",
            "text/; charset=utf-8",
        ] {
            assert_eq!(transport_encoding(header), None, "{header:?}");
        }
        assert_eq!(
            transport_encoding(&format!("text/html; charset=\"{}utf-8\"", " ".repeat(500))),
            Some(UTF_8)
        );
    }

    #[test]
    fn prescan_skips_comments_and_quoted_tag_attributes() {
        for prefix in [
            "<!-- <meta charset=shift_jis> -->",
            "<div data-x='<meta charset=shift_jis>'>",
            "<metacharset=shift_jis>",
        ] {
            assert_eq!(
                html_encoding(format!("{prefix}<META CHARSET='utf-8'>").as_bytes(), "").encoding,
                UTF_8
            );
        }
        assert_eq!(
            html_encoding(b"<!--><meta/charset=utf-8>", "").encoding,
            UTF_8
        );
        // The prescan deliberately does not run the HTML tokenizer's RAWTEXT states.
        assert_eq!(
            html_encoding(b"<script>'<meta charset=utf-8>'</script>", "").encoding,
            UTF_8
        );
    }

    #[test]
    fn meta_pragma_attribute_order_and_normalization() {
        for source in [
            "<meta content='text/html;charset=shift_jis' http-equiv=content-type>",
            "<meta http-equiv=Content-Type content='charset = shift_jis'>",
            "<meta charset=shift_jis charset=utf-8>",
            "<meta content='charset=utf-8' charset=shift_jis>",
        ] {
            assert_eq!(
                html_encoding(source.as_bytes(), "").encoding,
                SHIFT_JIS,
                "{source}"
            );
        }
        for source in [
            "<meta content='charset=shift_jis'>",
            "<meta charset=unknown content='charset=shift_jis' http-equiv=content-type>",
            "<meta charset=x-user-defined>",
        ] {
            assert_eq!(
                html_encoding(source.as_bytes(), "").encoding,
                WINDOWS_1252,
                "{source}"
            );
        }
        assert_eq!(html_encoding(b"<meta charset=utf-16>", "").encoding, UTF_8);
        assert_eq!(
            html_encoding(b"<meta charset=utf&#45;8>", "").encoding,
            WINDOWS_1252
        );
        assert_eq!(meta_encoding(Some("utf-8"), None, None), Some(UTF_8));
        // Actual token processing can fall back after an invalid charset label;
        // the byte prescan instead remembers its failed charset declaration.
        assert_eq!(
            meta_encoding(
                Some("unknown"),
                Some("content-type"),
                Some("charset=shift_jis")
            ),
            Some(SHIFT_JIS)
        );
        assert_eq!(
            meta_encoding(
                Some("utf-8"),
                Some("content-type"),
                Some("charset=shift_jis")
            ),
            Some(UTF_8)
        );
    }

    #[test]
    fn prescan_is_bounded_and_xml_fallback_is_distinct() {
        let mut bytes = vec![b' '; 1024];
        bytes.extend_from_slice(b"<meta charset=utf-8>");
        assert_eq!(html_encoding(&bytes, "").encoding, WINDOWS_1252);
        assert_eq!(
            html_encoding(b"<?xml version='1.0' encoding='shift_jis'?>", "").encoding,
            SHIFT_JIS
        );
        assert_eq!(
            html_encoding(b"<?xml encoding='utf-16'?>", "").encoding,
            UTF_8
        );
        assert_eq!(
            html_encoding(b"<?xml encoding='x-user-defined'?>", "").encoding,
            X_USER_DEFINED
        );
        assert_eq!(
            html_encoding(b"<?XML encoding='utf-8'?>", "").encoding,
            WINDOWS_1252
        );
        assert_eq!(html_encoding(b"<\0?\0x\0m\0l\0", "").encoding, UTF_16LE);
    }

    #[test]
    fn stylesheet_byte_declarations_are_exact_and_inherit_environment() {
        for source in [b"@charset \"utf-8\";".as_slice(), b"@charset \"utf-16le\";"] {
            assert_eq!(css_encoding(source, "text/css", WINDOWS_1252), UTF_8);
            assert_eq!(
                css_encoding(source, "text/css;charset=shift_jis", WINDOWS_1252),
                SHIFT_JIS
            );
        }
        for source in [
            " @charset \"utf-8\";",
            "@CHARSET \"utf-8\";",
            "@charset 'utf-8';",
            "@charset  \"utf-8\";",
            "@charset \"utf-8\" ",
        ] {
            assert_eq!(
                css_encoding(source.as_bytes(), "text/css", WINDOWS_1252),
                WINDOWS_1252
            );
        }
        let bytes = format!("@charset \"{}utf-8\";", " ".repeat(1024));
        assert_eq!(
            css_encoding(bytes.as_bytes(), "text/css", WINDOWS_1252),
            WINDOWS_1252
        );
        assert_eq!(
            css_encoding(b"\xef\xbb\xbf", "text/css;charset=shift_jis", WINDOWS_1252),
            UTF_8
        );
    }

    #[test]
    fn classic_script_charset_is_a_fallback_after_bom_and_transport() {
        assert_eq!(
            script_encoding(b"", "text/javascript", Some("shift_jis"), WINDOWS_1252),
            SHIFT_JIS
        );
        assert_eq!(
            script_encoding(
                b"",
                "text/javascript;charset=utf-8",
                Some("shift_jis"),
                WINDOWS_1252
            ),
            UTF_8
        );
        assert_eq!(
            script_encoding(
                b"\xff\xfe",
                "text/javascript;charset=utf-8",
                Some("shift_jis"),
                WINDOWS_1252
            ),
            UTF_16LE
        );
        assert_eq!(
            script_encoding(b"", "", Some("unknown"), WINDOWS_1252),
            WINDOWS_1252
        );
    }
}
