//! Strict, bounded CSS `url()` decoding and pure URL resolution.
//!
//! This utility does not admit a CSS property, apply a fetch policy, or fetch a
//! resource. Authored tokens remain the caller's serialization source. Unlike
//! the HTML URL helper, resolution uses the URL parser's default UTF-8 encoding.

use super::SourceBase;
use crate::selectors::{self, Kind, Token};
use std::mem::size_of;

const MAX_RAW_BYTES: usize = 4096;
// The existing resource URL ceiling; this is a size bound, not fetch authority.
const MAX_URL_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedUrl {
    text: String,
}

impl DecodedUrl {
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolvedUrl {
    Absolute(::url::Url),
    /// The decoded text, including its leading `#`, without base resolution.
    LocalFragment(String),
    Empty,
    Unresolved(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlError {
    Limit,
}

/// Cumulative admission for all calls sharing this budget. Refused work or
/// storage exhausts the corresponding allowance; earlier debits are retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UrlBudget {
    pub work: usize,
    pub bytes: usize,
}

impl UrlBudget {
    fn work(&mut self, cost: usize) -> Result<(), UrlError> {
        selectors::spend(&mut self.work, cost).map_err(|_| UrlError::Limit)
    }

    fn bytes(&mut self, cost: usize) -> Result<(), UrlError> {
        if cost > self.bytes {
            self.bytes = 0;
            return Err(UrlError::Limit);
        }
        self.bytes -= cost;
        Ok(())
    }
}

struct Cursor<'a> {
    source: &'a str,
    at: usize,
}

impl<'a> Cursor<'a> {
    // Deliberately strict about EOF comments, unlike tokenizer recovery.
    fn trivia(&mut self) -> bool {
        while self.at < self.source.len() {
            let rest = &self.source[self.at..];
            if let Some(comment) = rest.strip_prefix("/*") {
                let Some(end) = comment.find("*/") else {
                    return false;
                };
                self.at += end + 4;
            } else if rest.chars().next().is_some_and(selectors::space) {
                self.at += 1;
            } else {
                break;
            }
        }
        true
    }

    fn token(&mut self) -> Option<Token<'a>> {
        if self.at == self.source.len() {
            return None;
        }
        let start = self.at;
        let (kind, count) = selectors::token(&self.source[start..]);
        self.at += count;
        Some(Token {
            kind,
            raw: &self.source[start..self.at],
            start,
            end: self.at,
        })
    }
}

enum Body<'a> {
    Quoted(&'a str),
    Unquoted(&'a str),
}

/// Decode exactly one complete quoted or unquoted `url()` value. `None` is
/// ordinary invalid/unsupported syntax; strict EOF refusal is intentional.
pub fn decode(value: &str, budget: &mut UrlBudget) -> Result<Option<DecodedUrl>, UrlError> {
    let len = value.len();
    if len > MAX_RAW_BYTES {
        return Err(UrlError::Limit);
    }
    // Existing tokenizer tariff, without allocating its optional token Vec.
    budget.work(len.saturating_mul(2).saturating_add(1))?;
    // Independently covers trivia, name recheck, strict closure and escape scans.
    budget.work(len.saturating_mul(8).saturating_add(32))?;
    let mut cursor = Cursor {
        source: value,
        at: 0,
    };
    if !cursor.trivia() {
        return Ok(None);
    }
    let Some(first) = cursor.token() else {
        return Ok(None);
    };
    let body = match first.kind {
        Kind::Url => {
            let (name_end, is_url) = selectors::ident_sequence(first.raw);
            if !is_url || !first.raw[name_end..].starts_with('(') {
                return Ok(None);
            }
            Body::Unquoted(&first.raw[name_end + 1..])
        }
        Kind::Function if first.is_name("url") => {
            if !cursor.trivia() {
                return Ok(None);
            }
            let Some(string) = cursor.token().filter(|token| token.kind == Kind::String) else {
                return Ok(None);
            };
            if !cursor.trivia()
                || !cursor
                    .token()
                    .is_some_and(|token| token.kind == Kind::Close(')'))
            {
                return Ok(None);
            }
            Body::Quoted(string.raw)
        }
        _ => return Ok(None),
    };
    if !cursor.trivia() || cursor.at != value.len() {
        return Ok(None);
    }
    let capacity = len.saturating_mul(3);
    budget.bytes(capacity.saturating_add(size_of::<String>()))?;
    let mut text = String::new();
    text.try_reserve_exact(capacity)
        .map_err(|_| UrlError::Limit)?;
    let valid = match body {
        Body::Quoted(raw) => quoted(raw, &mut text),
        Body::Unquoted(raw) => unquoted(raw, &mut text),
    };
    Ok(valid.then_some(DecodedUrl { text }))
}

fn replacement(ch: char) -> char {
    if ch == '\0' { '\u{fffd}' } else { ch }
}

// `at` follows a backslash. No EOF repair or unquoted escaped newline.
fn escaped(raw: &str, at: &mut usize) -> Option<char> {
    let ch = raw[*at..].chars().next()?;
    if matches!(ch, '\n' | '\r' | '\x0c') {
        return None;
    }
    if !ch.is_ascii_hexdigit() {
        *at += ch.len_utf8();
        return Some(replacement(ch));
    }
    let mut code = 0;
    for _ in 0..6 {
        let Some(byte) = raw
            .as_bytes()
            .get(*at)
            .filter(|byte| byte.is_ascii_hexdigit())
        else {
            break;
        };
        code = code * 16 + char::from(*byte).to_digit(16).unwrap();
        *at += 1;
    }
    if raw[*at..].starts_with("\r\n") {
        *at += 2;
    } else if raw[*at..].chars().next().is_some_and(selectors::space) {
        *at += 1;
    }
    Some(char::from_u32(code).map_or('\u{fffd}', replacement))
}

fn quoted(raw: &str, out: &mut String) -> bool {
    let quote = char::from(raw.as_bytes()[0]);
    let mut at = 1;
    while let Some(ch) = raw[at..].chars().next() {
        at += ch.len_utf8();
        if ch == quote {
            return at == raw.len();
        }
        if matches!(ch, '\n' | '\r' | '\x0c') {
            return false;
        }
        if ch == '\\' {
            if raw[at..].starts_with("\r\n") {
                at += 2;
            } else if raw[at..].starts_with(['\n', '\r', '\x0c']) {
                at += 1;
            } else if let Some(decoded) = escaped(raw, &mut at) {
                out.push(decoded);
            } else {
                return false;
            }
        } else {
            out.push(replacement(ch));
        }
    }
    false
}

fn unquoted(raw: &str, out: &mut String) -> bool {
    let mut at = 0;
    while raw[at..].chars().next().is_some_and(selectors::space) {
        at += 1;
    }
    while let Some(ch) = raw[at..].chars().next() {
        at += ch.len_utf8();
        if ch == ')' {
            return at == raw.len();
        }
        if selectors::space(ch) {
            while raw[at..].chars().next().is_some_and(selectors::space) {
                at += 1;
            }
            return &raw[at..] == ")";
        }
        if ch == '\\' {
            let Some(decoded) = escaped(raw, &mut at) else {
                return false;
            };
            out.push(decoded);
        } else if matches!(ch, '\'' | '"' | '(' | '\x01'..='\x08' | '\x0b' | '\x0e'..='\x1f' | '\x7f')
        {
            return false;
        } else {
            // Comments here are URL data, not tokenizer trivia.
            out.push(replacement(ch));
        }
    }
    false
}

fn copy_text(text: &str) -> Result<String, UrlError> {
    let mut copy = String::new();
    copy.try_reserve_exact(text.len())
        .map_err(|_| UrlError::Limit)?;
    copy.push_str(text);
    Ok(copy)
}

/// Resolve decoded text against its explicit source context. This does not use
/// the HTML helper's legacy query encoding or perform any resource operation.
pub fn resolve(
    value: &DecodedUrl,
    base: &SourceBase,
    current_document_base: &::url::Url,
    budget: &mut UrlBudget,
) -> Result<ResolvedUrl, UrlError> {
    budget.work(8)?;
    let text = value.as_str();
    if text.is_empty() {
        return Ok(ResolvedUrl::Empty);
    }
    if text.starts_with('#') {
        budget.work(text.len().saturating_mul(4).saturating_add(8))?;
        budget.bytes(text.len().saturating_add(size_of::<String>()))?;
        return copy_text(text).map(ResolvedUrl::LocalFragment);
    }
    let base = match base {
        SourceBase::Document => current_document_base,
        SourceBase::Stylesheet(url) => url.as_ref(),
    };
    if base.as_str().len() > MAX_URL_BYTES || text.len() > MAX_URL_BYTES {
        return Err(UrlError::Limit);
    }
    let input = base
        .as_str()
        .len()
        .saturating_add(text.len())
        .saturating_add(1);
    let decoded = text.len().saturating_add(1);
    // Pinned URL parsing copies base prefixes and percent-encodes new input.
    // IDNA/punycode includes bounded nested scalar scans, hence the quadratic
    // allowance in decoded input only. The base is already a parsed URL.
    budget.work(
        input
            .saturating_mul(32)
            .saturating_add(decoded.saturating_mul(decoded).saturating_mul(16)),
    )?;
    // Conservative output/scratch admission, not allocator instrumentation.
    // The dependency's allocation is infallible; fixed input/result ceilings
    // and this prepaid allowance remain distinct from URL parse failure.
    budget.bytes(
        input
            .saturating_mul(64)
            .saturating_add(4096)
            .saturating_add(size_of::<::url::Url>()),
    )?;
    match ::url::Url::options().base_url(Some(base)).parse(text) {
        Ok(url) if url.as_str().len() <= MAX_URL_BYTES => Ok(ResolvedUrl::Absolute(url)),
        Ok(_) => Err(UrlError::Limit),
        Err(_) => copy_text(text).map(ResolvedUrl::Unresolved),
    }
}

#[cfg(test)]
mod tests;
