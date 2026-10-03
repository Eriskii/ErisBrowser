//! A deliberately small, capability-limited JavaScript interpreter.
//!
//! This is a custom language implementation, not an ECMAScript conformance claim.
//! Every entry point enforces execution, nesting, source, and allocation limits.
//! Scripts have DOM access but no filesystem, network, process, or host-eval access.
//! Strings and ordinary property keys preserve UTF-16 code units. Represented
//! DOM reads/clones retain exact character data; presentation projection is
//! separate. Legacy DOM writes remain scalar/lossy until their own migration.
//! JavaScript HTML serialization explicitly refuses a nonscalar final string.

use crate::date_host::DateHost;
use crate::dom::{Document, Namespace, NodeId, NodeKind};
use crate::js_identifier::{IDENTIFIER_LOOKUP_WORK, is_identifier_part, is_identifier_start};
use crate::js_string::{JsString, is_js_whitespace, radix_number};
use crate::js_uri;
use crate::regexp::{self, RegExp};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

#[cfg(test)]
mod bootstrap_tests;
mod code;
mod construction;
mod data_view;
mod date_builtins;
mod document_title;
mod dom_bindings;
mod dom_data;
mod dom_own_properties;
mod dom_prototypes;
mod iterators;
mod machine;
mod names;
mod node_connected;
mod node_data;
mod node_equality;
mod node_normalize;
mod node_position;
mod node_predicates;
mod node_root;
mod object_integrity;
mod object_is;
mod own_keys;
mod parser;
mod processing_instruction;
mod property_keys;
mod regexp_builtins;
mod string_builtins;
mod symbols;
mod text_operations;
mod window;
use symbols::PropertyKey;
pub use symbols::Symbol;
#[cfg(test)]
mod parser_legacy;
#[cfg(test)]
use parser_legacy::{ActiveLabel, Parser};
mod array_buffer;
mod array_builtins;
mod array_concat;
mod array_from;
mod array_methods;
mod array_properties;
mod array_splice;
mod tokens;

const MAX_SOURCE: usize = 256 * 1024;
const MAX_TOKENS: usize = 32_768;
const MAX_DEPTH: usize = 96;
// Native callbacks share this budget with user functions: one JavaScript call
// can retain several Rust interpreter/JSON frames, including in debug builds.
const MAX_CALLS: usize = 32;
const MAX_STACK_UNITS: usize = 96;
const MAX_STEPS: usize = 100_000;
const MAX_HEAP: usize = 8 * 1024 * 1024;
const MAX_STRING: usize = 256 * 1024;
const MAX_NODES: usize = 100_000;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(JsString),
    Symbol(Symbol),
    Array(usize),
    Object(usize),
    Function(usize),
    Node(NodeId),
    Document,
    Window,
    Console,
    Math,
    Json,
    Style(NodeId),
    ClassList(NodeId),
    Native(Rc<Native>),
}

#[derive(Clone, Debug)]
pub struct Native {
    name: String,
    receiver: Value,
    // A private eagerly allocated property bag, independent of callable
    // identity. Legacy native functions continue using the name registry.
    properties: Option<std::num::NonZeroUsize>,
}
const NATIVE_METADATA_BYTES: usize = std::mem::size_of::<Option<std::num::NonZeroUsize>>();

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.receiver == other.receiver
    }
}

#[derive(Clone, Copy)]
enum NumberPredicate {
    Finite,
    Nan,
    Integer,
    SafeInteger,
}

impl NumberPredicate {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "Number.isFinite" => Self::Finite,
            "Number.isNaN" => Self::Nan,
            "Number.isInteger" => Self::Integer,
            "Number.isSafeInteger" => Self::SafeInteger,
            _ => return None,
        })
    }

    fn test(self, value: Option<&Value>) -> bool {
        let Some(Value::Number(number)) = value else {
            return false;
        };
        match self {
            Self::Finite => number.is_finite(),
            Self::Nan => number.is_nan(),
            Self::Integer => number.is_finite() && number.trunc() == *number,
            Self::SafeInteger => {
                number.is_finite()
                    && number.trunc() == *number
                    && number.abs() <= 9_007_199_254_740_991.0
            }
        }
    }
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    fn truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Bool(b) => *b,
            Self::Number(n) => *n != 0.0 && !n.is_nan(),
            Self::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    fn number(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Null => 0.0,
            Self::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Self::String(s) => s.number(),
            _ => f64::NAN,
        }
    }

    fn js_string(&self) -> JsString {
        match self {
            Self::String(text) => text.clone(),
            Self::Number(number) => JsString::from(json_number(*number)),
            _ => JsString::from(self.to_string()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undefined => write!(f, "undefined"),
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) if n.is_nan() => write!(f, "NaN"),
            Self::Number(n) if *n == f64::INFINITY => write!(f, "Infinity"),
            Self::Number(n) if *n == f64::NEG_INFINITY => write!(f, "-Infinity"),
            Self::Number(n) if *n == 0.0 => write!(f, "0"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Symbol(symbol) => {
                write!(f, "Symbol(")?;
                if let Some(description) = symbol.description() {
                    write!(f, "{description}")?;
                }
                write!(f, ")")
            }
            Self::Function(_) | Self::Native(_) => write!(f, "[function]"),
            Self::Array(_) => write!(f, "[object Array]"),
            Self::Node(_) => write!(f, "[object Element]"),
            _ => write!(f, "[object Object]"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptError {
    pub message: String,
    pub offset: Option<usize>,
    kind: ErrorKind,
    thrown_name: Option<String>,
    intrinsic_name: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
enum ErrorKind {
    Runtime(&'static str),
    Resource,
    Unsupported,
    Thrown(Value),
}

impl ScriptError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("Error"),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn at(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset: Some(offset),
            kind: ErrorKind::Runtime("SyntaxError"),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn resource(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Resource,
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn reference(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("ReferenceError"),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn type_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("TypeError"),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn syntax(message: impl Into<String>) -> Self {
        let mut error = Self::new(message);
        error.kind = ErrorKind::Runtime("SyntaxError");
        error
    }
    fn range_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("RangeError"),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn thrown(value: Value) -> Self {
        Self {
            message: format!("uncaught exception: {value}"),
            offset: None,
            kind: ErrorKind::Thrown(value),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    /// Resource failures terminate script execution and cannot enter catch/finally.
    pub fn is_resource_limit(&self) -> bool {
        matches!(self.kind, ErrorKind::Resource)
    }
    pub fn name(&self) -> &str {
        match &self.kind {
            ErrorKind::Runtime(name) => name,
            ErrorKind::Resource => "ResourceLimit",
            ErrorKind::Unsupported => "UnsupportedFeature",
            ErrorKind::Thrown(_) => self.thrown_name.as_deref().unwrap_or("ThrownValue"),
        }
    }
    /// Canonical engine error identity, independent of writable diagnostic names.
    /// Plain objects and user constructors cannot impersonate an intrinsic error.
    pub fn intrinsic_error_name(&self) -> Option<&'static str> {
        match self.kind {
            ErrorKind::Runtime(name) => intrinsic_error_type(name),
            ErrorKind::Thrown(_) => self.intrinsic_name,
            ErrorKind::Resource | ErrorKind::Unsupported => None,
        }
    }
    pub fn is_parse_error(&self) -> bool {
        self.offset.is_some() && matches!(self.kind, ErrorKind::Runtime("SyntaxError"))
    }
    pub fn is_unsupported(&self) -> bool {
        matches!(self.kind, ErrorKind::Unsupported)
    }
    fn unsupported(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Unsupported,
            thrown_name: None,
            intrinsic_name: None,
        }
    }
}
impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "{} at byte {}", self.message, offset)
        } else {
            write!(f, "{}", self.message)
        }
    }
}
impl std::error::Error for ScriptError {}
type Result<T> = std::result::Result<T, ScriptError>;
fn uri_error(error: js_uri::Error) -> ScriptError {
    match error {
        js_uri::Error::Malformed => {
            let mut error = ScriptError::new("malformed URI encoding");
            error.kind = ErrorKind::Runtime("URIError");
            error
        }
        js_uri::Error::OutputLimit => ScriptError::resource("script string limit exceeded"),
    }
}
fn regexp_error(error: regexp::Error) -> ScriptError {
    match error {
        regexp::Error::Syntax(message) => ScriptError::syntax(message),
        regexp::Error::Unsupported(message) => ScriptError::unsupported(message),
        regexp::Error::Resource(message) => ScriptError::resource(message),
    }
}

#[derive(Clone, Debug)]
struct IdentifierToken {
    value: Rc<str>,
    escaped: bool,
}
#[derive(Clone, Debug)]
enum TokenKind {
    Word(IdentifierToken),
    Number(f64),
    String(JsString),
    Symbol(String),
    RegExp(Rc<RegExp>),
    TemplateStart,
    Invalid(Rc<ScriptError>),
    End,
}
#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    offset: usize,
    line_break_before: bool,
    string_literal: bool,
    use_strict: bool,
    legacy_literal: bool,
}

// Scan one quoted string or cooked template component. The delimiter at
// `start` is an opening quote/backtick or the substitution-closing brace.
fn quoted_text(
    source: &str,
    start: usize,
    quote: char,
    mut budget: Option<&mut regexp::Budget>,
) -> Result<(JsString, usize, bool, bool)> {
    if let Some(budget) = budget.as_deref_mut() {
        compile_allocate(budget, 32)?;
    }
    let mut pos = start + 1;
    let mut legacy_literal = false;
    let mut interpolation = false;
    let mut value = Vec::<u16>::new();
    let mut closed = false;
    while pos < source.len() {
        if let Some(budget) = budget.as_deref_mut() {
            budget.work(1).map_err(regexp_error)?;
            compile_allocate(budget, 8)?;
        }
        if value.len() > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        let c = source[pos..].chars().next().unwrap();
        pos += c.len_utf8();
        if c == quote {
            closed = true;
            break;
        }
        if quote == '`' && c == '$' && source[pos..].starts_with('{') {
            pos += 1;
            interpolation = true;
            break;
        }
        if value.capacity().saturating_sub(value.len()) < 2 {
            let capacity = value.capacity().saturating_mul(2).max(4);
            if let Some(budget) = budget.as_deref_mut() {
                compile_allocate(budget, capacity.saturating_mul(2).saturating_add(32))?;
            }
            value
                .try_reserve_exact(capacity - value.len())
                .map_err(|_| ScriptError::resource("quoted token allocation failed"))?;
        }
        if c == '\\' {
            let e = source[pos..]
                .chars()
                .next()
                .ok_or_else(|| ScriptError::at("unterminated escape", pos))?;
            pos += e.len_utf8();
            match e {
                'n' => value.push(10),
                'r' => value.push(13),
                't' => value.push(9),
                'b' => value.push(8),
                'f' => value.push(12),
                'v' => value.push(11),
                '0'..='7' => {
                    let mut n = e as u16 - '0' as u16;
                    let mut count = 1;
                    while count < if e <= '3' { 3 } else { 2 }
                        && source
                            .as_bytes()
                            .get(pos)
                            .is_some_and(|b| (b'0'..=b'7').contains(b))
                    {
                        n = n * 8 + u16::from(source.as_bytes()[pos] - b'0');
                        pos += 1;
                        count += 1;
                    }
                    legacy_literal |= e != '0'
                        || count > 1
                        || source.as_bytes().get(pos).is_some_and(u8::is_ascii_digit);
                    value.push(n);
                }
                '8' | '9' => {
                    legacy_literal = true;
                    value.push(e as u16);
                }
                '\n' | '\u{2028}' | '\u{2029}' => {}
                '\r' => {
                    if source.as_bytes().get(pos) == Some(&b'\n') {
                        pos += 1;
                    }
                }
                'u' | 'x' => {
                    if e == 'u' && source.as_bytes().get(pos) == Some(&b'{') {
                        pos += 1;
                        let start = pos;
                        while source
                            .as_bytes()
                            .get(pos)
                            .is_some_and(u8::is_ascii_hexdigit)
                        {
                            pos += 1;
                        }
                        if start == pos || source.as_bytes().get(pos) != Some(&b'}') {
                            return Err(ScriptError::at(
                                "invalid Unicode code point escape",
                                start,
                            ));
                        }
                        let point = u32::from_str_radix(&source[start..pos], 16).map_err(|_| {
                            ScriptError::at("invalid Unicode code point escape", start)
                        })?;
                        if point > 0x10ffff {
                            return Err(ScriptError::at("Unicode code point exceeds range", start));
                        }
                        if point <= 0xffff {
                            value.push(point as u16);
                        } else {
                            let point = point - 0x10000;
                            value.push(0xd800 + (point >> 10) as u16);
                            value.push(0xdc00 + (point & 0x3ff) as u16);
                        }
                        pos += 1;
                        continue;
                    }
                    let len = if e == 'u' { 4 } else { 2 };
                    let end = pos
                        .checked_add(len)
                        .filter(|end| *end <= source.len())
                        .ok_or_else(|| ScriptError::at("incomplete character escape", pos))?;
                    let hex = source
                        .get(pos..end)
                        .ok_or_else(|| ScriptError::at("invalid character escape", pos))?;
                    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                        return Err(ScriptError::at("invalid character escape", pos));
                    }
                    let n = u32::from_str_radix(hex, 16)
                        .map_err(|_| ScriptError::at("invalid character escape", pos))?;
                    value.push(n as u16);
                    pos = end;
                }
                _ => {
                    let mut units = [0; 2];
                    value.extend_from_slice(e.encode_utf16(&mut units));
                }
            }
        } else {
            if matches!(c, '\n' | '\r') && quote != '`' {
                return Err(ScriptError::at("newline in string", pos));
            }
            if quote == '`' && c == '\r' {
                if source.as_bytes().get(pos) == Some(&b'\n') {
                    pos += 1;
                }
                value.push(10);
            } else {
                let mut units = [0; 2];
                value.extend_from_slice(c.encode_utf16(&mut units));
            }
        }
    }
    if !closed && !interpolation {
        return Err(ScriptError::at("unterminated string", start));
    }
    if quote == '`' && legacy_literal {
        return Err(ScriptError::at(
            "legacy escapes are forbidden in template literals",
            start,
        ));
    }
    if let Some(budget) = budget {
        budget.work((pos - start) / 8 + 1).map_err(regexp_error)?;
        compile_allocate(budget, value.len().saturating_mul(2).saturating_add(32))?;
    }
    Ok((JsString::from(value), pos, legacy_literal, interpolation))
}

fn compile_allocate(budget: &mut regexp::Budget, bytes: usize) -> Result<()> {
    budget.allocated = budget.allocated.saturating_add(bytes);
    if budget.allocated > budget.heap_limit {
        return Err(ScriptError::resource(
            "script compile allocation limit exceeded",
        ));
    }
    Ok(())
}

fn reserve_tokens(
    tokens: &mut tokens::Tokens,
    additional: usize,
    budget: &mut regexp::Budget,
) -> Result<()> {
    tokens.reserve(additional, budget)
}

fn identifier_escape(
    source: &str,
    start: usize,
    budget: &mut regexp::Budget,
) -> Result<(char, usize)> {
    let bytes = source.as_bytes();
    if bytes.get(start + 1) != Some(&b'u') {
        return Err(ScriptError::at(
            "identifier escape must use Unicode syntax",
            start,
        ));
    }
    let mut at = start + 2;
    let braced = bytes.get(at) == Some(&b'{');
    if braced {
        at += 1;
    }
    let digits = at;
    let mut value = 0u32;
    while braced || at - digits < 4 {
        if (at - digits) & 7 == 0 {
            budget.work(1).map_err(regexp_error)?;
        }
        let Some(byte) = bytes.get(at).copied() else {
            break;
        };
        let digit = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'a'..=b'f' => u32::from(byte - b'a' + 10),
            b'A'..=b'F' => u32::from(byte - b'A' + 10),
            _ => break,
        };
        value = value
            .checked_mul(16)
            .and_then(|value| value.checked_add(digit))
            .filter(|value| *value <= 0x10ffff)
            .ok_or_else(|| ScriptError::at("identifier escape exceeds Unicode range", start))?;
        at += 1;
    }
    if braced {
        if at == digits || bytes.get(at) != Some(&b'}') {
            return Err(ScriptError::at("invalid braced identifier escape", start));
        }
        at += 1;
    } else if at - digits != 4 {
        return Err(ScriptError::at("invalid identifier escape", start));
    }
    let value = char::from_u32(value)
        .ok_or_else(|| ScriptError::at("surrogate is not an identifier code point", start))?;
    Ok((value, at))
}

enum IdentifierFirst {
    ValidatedRaw(char),
    Escape,
}

fn identifier_token(
    source: &str,
    start: usize,
    first: IdentifierFirst,
    budget: &mut regexp::Budget,
) -> Result<(IdentifierToken, usize)> {
    // The lexer already queried the raw first scalar. Reuse that result;
    // escaped first scalars still need decoding and positional validation.
    let mut length = match first {
        IdentifierFirst::ValidatedRaw(value) => value.len_utf8(),
        IdentifierFirst::Escape => 0,
    };
    let mut at = start + length;
    let mut escaped = false;
    while let Some(raw) = source[at..].chars().next() {
        let (value, end) = if raw == '\\' {
            escaped = true;
            identifier_escape(source, at, budget)?
        } else {
            (raw, at + raw.len_utf8())
        };
        if !value.is_ascii() {
            budget.work(IDENTIFIER_LOOKUP_WORK).map_err(regexp_error)?;
        }
        let valid = if at == start {
            is_identifier_start(value)
        } else {
            is_identifier_part(value)
        };
        if !valid {
            if raw == '\\' || at == start {
                return Err(ScriptError::at("invalid identifier code point", at));
            }
            break;
        }
        length += value.len_utf8();
        at = end;
    }
    compile_allocate(
        budget,
        length
            .saturating_mul(if escaped { 2 } else { 1 })
            .saturating_add(32),
    )?;
    budget.work(1 + (at - start) / 8).map_err(regexp_error)?;
    let value = if !escaped {
        Rc::from(&source[start..at])
    } else {
        let mut output = String::new();
        output
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("identifier allocation failed"))?;
        let mut cursor = start;
        while cursor < at {
            let raw = source[cursor..].chars().next().unwrap();
            if raw == '\\' {
                let (value, end) = identifier_escape(source, cursor, budget)?;
                output.push(value);
                cursor = end;
            } else {
                output.push(raw);
                cursor += raw.len_utf8();
            }
        }
        Rc::from(output)
    };
    Ok((IdentifierToken { value, escaped }, at))
}

fn lex(source: &str, budget: &mut regexp::Budget) -> Result<tokens::Tokens> {
    if source.len() > MAX_SOURCE {
        return Err(ScriptError::resource("script source limit exceeded"));
    }
    // Precharge bounded ASCII scans, including comments and numeric parsing.
    // Non-ASCII table searches and escape decoding charge separately.
    budget.work(1 + source.len() / 4).map_err(regexp_error)?;
    let mut tokens = tokens::Tokens::default();
    let mut pos = 0;
    let mut line_break_before = false;
    // The parser supplies the RegExp lexical goal at a primary expression.
    // Keep provisional division-goal errors as tokens: quotes and comments
    // inside a yet-unrecognized pattern must not reject the whole script.
    let result = (|| -> Result<()> {
        while pos < source.len() {
            let ch = source[pos..].chars().next().unwrap();
            if u16::try_from(u32::from(ch)).is_ok_and(is_js_whitespace) {
                line_break_before |= matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}');
                pos += ch.len_utf8();
                continue;
            }
            if source[pos..].starts_with("//") {
                pos += source[pos..]
                    .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                    .unwrap_or(source.len() - pos);
                continue;
            }
            if source[pos..].starts_with("/*") {
                let end = source[pos + 2..]
                    .find("*/")
                    .ok_or_else(|| ScriptError::at("unterminated comment", pos))?;
                line_break_before |=
                    source[pos..pos + end + 4].contains(['\n', '\r', '\u{2028}', '\u{2029}']);
                pos += end + 4;
                continue;
            }
            let start = pos;
            let mut legacy_literal = false;
            let kind = if ch == '`' {
                // A parser-selected lexical goal resumes after this marker.
                // Provisional division-goal scanning must not guess where an
                // interpolated expression or its following template text ends.
                pos += 1;
                TokenKind::TemplateStart
            } else if ch == '\'' || ch == '"' {
                let (value, end, legacy, _) = quoted_text(source, start, ch, Some(budget))?;
                pos = end;
                legacy_literal = legacy;
                TokenKind::String(value)
            } else if ch.is_ascii_digit()
                || (ch == '.'
                    && source
                        .as_bytes()
                        .get(pos + 1)
                        .is_some_and(u8::is_ascii_digit))
            {
                if source[pos..].starts_with("0x") || source[pos..].starts_with("0X") {
                    pos += 2;
                    let digits = pos;
                    while source
                        .as_bytes()
                        .get(pos)
                        .is_some_and(u8::is_ascii_hexdigit)
                    {
                        pos += 1;
                    }
                    let n = u64::from_str_radix(&source[digits..pos], 16)
                        .map_err(|_| ScriptError::at("invalid hexadecimal number", start))?;
                    TokenKind::Number(n as f64)
                } else {
                    while source.as_bytes().get(pos).is_some_and(u8::is_ascii_digit) {
                        pos += 1;
                    }
                    if source.as_bytes().get(pos) == Some(&b'.') {
                        pos += 1;
                        while source.as_bytes().get(pos).is_some_and(u8::is_ascii_digit) {
                            pos += 1;
                        }
                    }
                    if matches!(source.as_bytes().get(pos), Some(b'e' | b'E')) {
                        pos += 1;
                        if matches!(source.as_bytes().get(pos), Some(b'+' | b'-')) {
                            pos += 1;
                        }
                        while source.as_bytes().get(pos).is_some_and(u8::is_ascii_digit) {
                            pos += 1;
                        }
                    }
                    let raw = &source[start..pos];
                    legacy_literal =
                        raw.len() > 1 && raw.starts_with('0') && raw.as_bytes()[1].is_ascii_digit();
                    TokenKind::Number(
                        if legacy_literal && raw.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
                            raw.bytes().fold(0.0, |n, b| n * 8.0 + f64::from(b - b'0'))
                        } else {
                            raw.parse()
                                .map_err(|_| ScriptError::at("invalid number", start))?
                        },
                    )
                }
            } else if ch == '\\' || {
                if !ch.is_ascii() {
                    budget.work(IDENTIFIER_LOOKUP_WORK).map_err(regexp_error)?;
                }
                is_identifier_start(ch)
            } {
                let first = if ch == '\\' {
                    IdentifierFirst::Escape
                } else {
                    IdentifierFirst::ValidatedRaw(ch)
                };
                let (identifier, end) = identifier_token(source, start, first, budget)?;
                pos = end;
                TokenKind::Word(identifier)
            } else {
                let operator = [
                    ">>>=", "===", "!==", ">>>", "**=", "<<=", ">>=", "=>", "==", "!=", "<=", ">=",
                    "&&=", "||=", "??=", "&&", "||", "??", "++", "--", "+=", "-=", "*=", "/=",
                    "%=", "&=", "^=", "|=", "**", "<<", ">>",
                ]
                .into_iter()
                .find(|op| source[pos..].starts_with(op));
                if let Some(op) = operator {
                    compile_allocate(budget, 32)?;
                    pos += op.len();
                    TokenKind::Symbol(op.to_owned())
                } else if "{}[]().,;:?+-*/%<>=!~&|^".contains(ch) {
                    compile_allocate(budget, 32)?;
                    pos += 1;
                    TokenKind::Symbol(ch.to_string())
                } else {
                    return Err(ScriptError::at(
                        format!("unsupported character {ch:?}"),
                        pos,
                    ));
                }
            };
            if matches!(kind, TokenKind::Number(_))
                && let Some(next) = source[pos..].chars().next()
            {
                if !next.is_ascii() {
                    budget.work(IDENTIFIER_LOOKUP_WORK).map_err(regexp_error)?;
                }
                if next == '\\' || next.is_ascii_digit() || is_identifier_start(next) {
                    return Err(ScriptError::at(
                        "identifier immediately follows numeric literal",
                        pos,
                    ));
                }
            }
            reserve_tokens(&mut tokens, 1, budget)?;
            tokens.push(Token {
                kind,
                offset: start,
                line_break_before,
                string_literal: matches!(ch, '\'' | '"'),
                use_strict: matches!(&source[start..pos], "\"use strict\"" | "'use strict'"),
                legacy_literal,
            });
            line_break_before = false;
            if ch == '`' {
                break;
            }
            if tokens.len() > MAX_TOKENS {
                return Err(ScriptError::resource("script token limit exceeded"));
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        if error.is_resource_limit() {
            return Err(error);
        }
        reserve_tokens(&mut tokens, 1, budget)?;
        compile_allocate(
            budget,
            std::mem::size_of::<ScriptError>() + 2 * std::mem::size_of::<usize>(),
        )?;
        tokens.push(Token {
            offset: error.offset.unwrap_or(pos),
            kind: TokenKind::Invalid(Rc::new(error)),
            line_break_before,
            string_literal: false,
            use_strict: false,
            legacy_literal: false,
        });
    }
    reserve_tokens(&mut tokens, 1, budget)?;
    tokens.push(Token {
        kind: TokenKind::End,
        offset: source.len(),
        line_break_before,
        string_literal: false,
        use_strict: false,
        legacy_literal: false,
    });
    Ok(tokens)
}

// Preserve the existing conservative per-activation metadata allowance while
// production parameters use flat expression IDs. No owning syntax is retained.
struct ParameterStorage {
    _name: String,
    _initializer: Option<Rc<()>>,
    _rest: bool,
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct Parameter {
    name: String,
    // Initializers, like bodies, are immutable shared syntax. Closure/call
    // copies must never recursively clone an initializer's retained AST.
    initializer: Option<Rc<Expr>>,
    rest: bool,
}
#[cfg(test)]
impl Parameter {
    fn simple(name: String) -> Self {
        Self {
            name,
            initializer: None,
            rest: false,
        }
    }
    fn is_simple(&self) -> bool {
        !self.rest && self.initializer.is_none()
    }
}
#[cfg(test)]
#[derive(Clone, Debug)]
struct FunctionCode {
    params: Vec<Parameter>,
    body: Rc<Vec<Stmt>>,
    name: Option<String>,
    arrow: bool,
    self_name: bool,
    constructable: bool,
    strict: bool,
}
#[cfg(test)]
#[derive(Debug)]
struct Program {
    body: Vec<Stmt>,
    strict: bool,
    compiled_storage: usize,
    remaining_work: usize,
}
#[cfg(test)]
#[derive(Clone, Debug)]
enum Expr {
    Literal(Value),
    RegExp(Rc<RegExp>),
    Ident(String),
    NewTarget,
    Array(Vec<Option<Expr>>),
    Object(Vec<(PropertyName, ObjectEntry)>),
    Unary(String, Box<Expr>),
    BinaryChain(Box<Expr>, Vec<(String, Expr)>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>, Box<Expr>),
    Update(Box<Expr>, f64, bool),
    Member(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    New(Box<Expr>, Vec<Expr>),
    Function(FunctionCode),
    Sequence(Vec<Expr>),
    Template(JsString, Vec<(Expr, JsString)>),
}
#[cfg(test)]
#[derive(Clone, Debug)]
enum PropertyName {
    Literal(JsString, bool), // The boolean preserves IdentifierName syntax for shorthand/accessors.
    Computed(Box<Expr>),
}
#[cfg(test)]
#[derive(Clone, Debug)]
enum ObjectEntry {
    Data(Expr),
    Method(FunctionCode),
    Accessor(FunctionCode, bool),
    Prototype(Expr),
}
#[cfg(test)]
#[derive(Clone, Debug)]
enum Stmt {
    Empty,
    Label(usize, Box<Stmt>),
    Expr(Expr),
    Var(Vec<(String, Option<Expr>)>, DeclarationKind),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    DoWhile(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    ForIn(ForBinding, Expr, Box<Stmt>),
    ForOf(ForBinding, Expr, Box<Stmt>),
    Switch(Expr, Vec<(Option<Expr>, Vec<Stmt>)>),
    Function(String, FunctionCode),
    Return(Option<Expr>),
    Throw(Expr),
    Try(Box<Stmt>, Option<CatchClause>, Option<Box<Stmt>>),
    Break(Option<usize>),
    Continue(Option<usize>),
}
#[cfg(test)]
impl Stmt {
    fn is_lexical_declaration(&self, block_functions: bool) -> bool {
        matches!(
            self,
            Self::Var(_, DeclarationKind::Let | DeclarationKind::Const)
        ) || block_functions && matches!(self, Self::Function(..))
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
enum ForBinding {
    Declaration(String, DeclarationKind),
    Target(Expr),
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct CatchClause {
    binding: Option<String>,
    body: Vec<Stmt>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum DeclarationKind {
    Var,
    Let,
    Const,
}

// Borrow statement children in source order without visiting expression or
// function bodies. One cursor represents one ancestor, independent of width.
#[cfg(test)]
#[derive(Clone, Copy)]
enum StatementChildren<'a> {
    Empty,
    Branches {
        first: Option<&'a Stmt>,
        middle: &'a [Stmt],
        last: Option<&'a Stmt>,
    },
    Cases {
        cases: &'a [(Option<Expr>, Vec<Stmt>)],
        body: &'a [Stmt],
    },
}

#[cfg(test)]
impl<'a> StatementChildren<'a> {
    fn of(statement: &'a Stmt) -> Self {
        let (first, middle, last) = match statement {
            Stmt::Block(body) => (None, body.as_slice(), None),
            Stmt::If(_, yes, no) => (Some(&**yes), &[][..], no.as_deref()),
            Stmt::Label(_, body) | Stmt::While(_, body) | Stmt::DoWhile(_, body) => {
                (Some(&**body), &[][..], None)
            }
            Stmt::For(init, _, _, body) => (init.as_deref(), &[][..], Some(&**body)),
            Stmt::ForIn(_, _, body) | Stmt::ForOf(_, _, body) => (Some(&**body), &[][..], None),
            Stmt::Try(body, handler, finalizer) => (
                Some(&**body),
                handler.as_ref().map_or(&[][..], |handler| &handler.body),
                finalizer.as_deref(),
            ),
            Stmt::Switch(_, cases) if !cases.is_empty() => {
                return Self::Cases { cases, body: &[] };
            }
            _ => return Self::Empty,
        };
        if first.is_none() && middle.is_empty() && last.is_none() {
            Self::Empty
        } else {
            Self::Branches {
                first,
                middle,
                last,
            }
        }
    }

    fn next(&mut self, work: &mut impl FnMut() -> Result<()>) -> Result<Option<&'a Stmt>> {
        work()?;
        match self {
            Self::Empty => Ok(None),
            Self::Branches {
                first,
                middle,
                last,
            } => {
                if let Some(first) = first.take() {
                    return Ok(Some(first));
                }
                if let Some((statement, rest)) = middle.split_first() {
                    *middle = rest;
                    return Ok(Some(statement));
                }
                Ok(last.take())
            }
            Self::Cases { cases, body } => loop {
                if let Some((statement, rest)) = body.split_first() {
                    *body = rest;
                    return Ok(Some(statement));
                }
                let Some(((_, statements), rest)) = cases.split_first() else {
                    return Ok(None);
                };
                // Empty case lists still consume work before advancing.
                work()?;
                *cases = rest;
                *body = statements;
            },
        }
    }
}

#[cfg(test)]
struct StatementWalk<'a, I> {
    roots: I,
    ancestors: [StatementChildren<'a>; MAX_DEPTH],
    depth: usize,
    done: bool,
}

#[cfg(test)]
impl<'a, I: Iterator<Item = &'a Stmt>> StatementWalk<'a, I> {
    fn new(roots: I) -> Self {
        Self {
            roots,
            ancestors: [StatementChildren::Empty; MAX_DEPTH],
            depth: 0,
            done: false,
        }
    }

    fn next(&mut self, mut work: impl FnMut() -> Result<()>) -> Result<Option<&'a Stmt>> {
        if self.done {
            return Ok(None);
        }
        let result = self.advance(&mut work);
        if !matches!(result, Ok(Some(_))) {
            self.done = true;
        }
        result
    }

    fn advance(&mut self, work: &mut impl FnMut() -> Result<()>) -> Result<Option<&'a Stmt>> {
        loop {
            let statement = if self.depth == 0 {
                work()?;
                let Some(root) = self.roots.next() else {
                    return Ok(None);
                };
                root
            } else if let Some(child) = self.ancestors[self.depth - 1].next(work)? {
                child
            } else {
                self.depth -= 1;
                continue;
            };
            let children = StatementChildren::of(statement);
            if !matches!(children, StatementChildren::Empty) {
                if self.depth == self.ancestors.len() {
                    return Err(ScriptError::resource("statement traversal depth exceeded"));
                }
                self.ancestors[self.depth] = children;
                self.depth += 1;
            }
            return Ok(Some(statement));
        }
    }
}

#[derive(Clone, Copy)]
enum ReduceDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrackedGlobal {
    WindowSelf,
    GlobalThis,
    Undefined,
    Nan,
    Infinity,
}
impl TrackedGlobal {
    const ALL: [Self; 5] = [
        Self::WindowSelf,
        Self::GlobalThis,
        Self::Undefined,
        Self::Nan,
        Self::Infinity,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::WindowSelf => "self",
            Self::GlobalThis => "globalThis",
            Self::Undefined => "undefined",
            Self::Nan => "NaN",
            Self::Infinity => "Infinity",
        }
    }
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "self" => Some(Self::WindowSelf),
            "globalThis" => Some(Self::GlobalThis),
            "undefined" => Some(Self::Undefined),
            "NaN" => Some(Self::Nan),
            "Infinity" => Some(Self::Infinity),
            _ => None,
        }
    }
    fn from_key(key: &JsString) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| {
            let name = kind.name();
            key.len() == name.len() && key.units().iter().copied().eq(name.bytes().map(u16::from))
        })
    }
}

#[derive(Clone)]
struct Binding {
    value: Value,
    // Global accessors share one authoritative binding record with data properties.
    // The data value is Undefined while this pair is present. Lexical bindings
    // never have an accessor, even when they shadow the same global name.
    accessor: Option<Rc<BindingAccessor>>,
    mutable: bool,
    initialized: bool,
    strict_immutable: bool,
    global_property: bool,
    // Creation order for object-environment properties; unused by lexical bindings.
    global_order: u64,
    enumerable: bool,
    deletable: bool,
}
struct BindingAccessor {
    get: Value,
    set: Value,
}
const BINDING_BYTES: usize =
    128 + std::mem::size_of::<Option<Rc<BindingAccessor>>>() + std::mem::size_of::<u64>();
impl Binding {
    fn property(&self) -> Property {
        Property {
            value: match &self.accessor {
                Some(accessor) => PropertyValue::Accessor {
                    get: accessor.get.clone(),
                    set: accessor.set.clone(),
                },
                None => PropertyValue::Data {
                    value: self.value.clone(),
                    writable: self.mutable,
                },
            },
            enumerable: self.enumerable,
            configurable: self.deletable,
        }
    }
}
struct Environment {
    bindings: BTreeMap<String, Binding>,
    // Execution receiver, never an author-visible property or lexical name.
    this_binding: Option<Value>,
    new_target_binding: Option<Value>,
    parent: Option<usize>,
    function_scope: bool,
    strict: bool,
}
#[derive(Clone)]
struct Function {
    code: code::FunctionRef,
    environment: usize,
    properties: usize,
    bound: Option<BoundFunction>,
}
#[derive(Clone)]
struct BoundFunction {
    target: Value,
    receiver: Value,
    arguments: Vec<Value>,
}
enum Flow {
    // None is the specification's empty completion, distinct from undefined.
    Normal(Option<Value>),
    Return(Value),
    Break(Option<usize>, Option<Value>),
    Continue(Option<usize>, Option<Value>),
}
impl Flow {
    fn update_empty(mut self, previous: Option<&Value>) -> Self {
        match &mut self {
            Self::Normal(value) | Self::Break(_, value) | Self::Continue(_, value)
                if value.is_none() =>
            {
                *value = previous.cloned();
            }
            _ => {}
        }
        self
    }

    // LoopContinues followed by the iteration-value update. A departing jump
    // retains its own value, or takes the last value produced by this loop.
    fn loop_step(self, label: Option<usize>, last: &mut Value) -> std::result::Result<(), Self> {
        let value = match self {
            Self::Normal(value) | Self::Continue(None, value) => value,
            Self::Continue(target, value) if target == label => value,
            flow => return Err(flow.update_empty(Some(last))),
        };
        if let Some(value) = value {
            *last = value;
        }
        Ok(())
    }

    fn consume_break(self) -> Self {
        match self {
            Self::Break(None, value) => Self::Normal(Some(value.unwrap_or(Value::Undefined))),
            flow => flow,
        }
    }
}
enum Reference {
    CodeName {
        unit: Rc<code::Unit>,
        expression: code::ExprId,
        owner: Option<usize>,
        strict: bool,
    },
    // Computed names stay uncoerced until GetValue/PutValue. A successful read
    // replaces the name with a string or symbol so compound assignments convert once.
    Property(Value, Value, bool),
}

#[derive(Clone, Debug)]
enum PropertyValue {
    Data { value: Value, writable: bool },
    Accessor { get: Value, set: Value },
}
#[derive(Clone, Debug)]
struct Property {
    value: PropertyValue,
    enumerable: bool,
    configurable: bool,
}
impl Property {
    fn data(value: Value, writable: bool, enumerable: bool, configurable: bool) -> Self {
        Self {
            value: PropertyValue::Data { value, writable },
            enumerable,
            configurable,
        }
    }
}
#[derive(Default)]
struct PropertyDescriptor {
    value: Option<Value>,
    writable: Option<bool>,
    get: Option<Value>,
    set: Option<Value>,
    enumerable: Option<bool>,
    configurable: Option<bool>,
}
impl PropertyDescriptor {
    fn data_property(value: Value, writable: bool, enumerable: bool, configurable: bool) -> Self {
        Self {
            value: Some(value),
            writable: Some(writable),
            enumerable: Some(enumerable),
            configurable: Some(configurable),
            ..Self::default()
        }
    }
    fn accessor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }
    fn data(&self) -> bool {
        self.value.is_some() || self.writable.is_some()
    }
}
#[derive(Default)]
struct ScriptObject {
    values: BTreeMap<PropertyKey, Property>,
    order: Vec<PropertyKey>,
    prototype: Option<Value>,
    boxed: Option<Value>,
    non_extensible: bool,
    intrinsic_error: Option<&'static str>,
    parameter_map: BTreeMap<JsString, (usize, String)>,
    arguments: bool,
    regexp: Option<Rc<RegExp>>,
    date_value: Option<f64>,
    event: Option<usize>,
    event_target: bool,
    abort: Option<AbortSlot>,
    namespace: Option<&'static str>,
}
impl ScriptObject {
    fn get(&self, key: impl Into<PropertyKey>) -> Option<&Value> {
        self.values
            .get(&key.into())
            .and_then(|property| match &property.value {
                PropertyValue::Data { value, .. } => Some(value),
                PropertyValue::Accessor { .. } => None,
            })
    }
    fn contains_key(&self, key: impl Into<PropertyKey>) -> bool {
        self.values.contains_key(&key.into())
    }
    fn insert(&mut self, key: JsString, value: Value) {
        self.insert_property(key.into(), Property::data(value, true, true, true));
    }
    fn insert_property(&mut self, key: PropertyKey, property: Property) {
        if !self.values.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.values.insert(key, property);
    }
    fn remove(&mut self, key: &PropertyKey) {
        self.values.remove(key);
        self.order.retain(|item| item != key);
    }
    fn insert_hidden(&mut self, key: JsString, value: Value) {
        self.insert_property(key.into(), Property::data(value, true, false, true));
    }
    fn attributes(&mut self, key: &str, writable: bool, enumerable: bool, configurable: bool) {
        if let Some(property) = self.values.get_mut(&PropertyKey::from(key)) {
            property.enumerable = enumerable;
            property.configurable = configurable;
            if let PropertyValue::Data { writable: old, .. } = &mut property.value {
                *old = writable;
            }
        }
    }
}

struct JsonRecord {
    value: Value,
    source: Option<JsString>,
    children: BTreeMap<JsString, JsonRecord>,
}
struct JsonReader<'a> {
    source: &'a [u16],
    at: usize,
    tokens: usize,
    record: bool,
}
struct JsonWriter {
    replacer: Option<Value>,
    properties: Option<Vec<JsString>>,
    gap: JsString,
    stack: Vec<Value>,
    output: Vec<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EventTarget {
    Window,
    Document,
    Node(NodeId),
    Object(usize),
}
impl EventTarget {
    fn value(self) -> Value {
        match self {
            Self::Window => Value::Window,
            Self::Document => Value::Document,
            Self::Node(id) => Value::Node(id),
            Self::Object(id) => Value::Object(id),
        }
    }
}
#[derive(Clone, Copy)]
enum AbortSlot {
    Controller(usize),
    Signal(usize),
}
struct AbortState {
    reason: Value,
    listeners: Vec<usize>,
}
struct ToggleState {
    old_state: JsString,
    new_state: JsString,
    source: Value,
}
struct EventState {
    event_type: JsString,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    detail: Value,
    custom: bool,
    toggle: Option<ToggleState>,
    target: Value,
    current_target: Value,
    phase: u8,
    stopped: bool,
    immediate: bool,
    canceled: bool,
    passive: bool,
    dispatching: bool,
    initialized: bool,
    trusted: bool,
    timestamp: f64,
    path: Vec<EventTarget>,
}
#[derive(Clone)]
struct EventListener {
    callback: Value,
    capture: bool,
    once: bool,
    passive: bool,
    removed: bool,
    handler: bool,
}
struct EventHandler {
    listener: Option<usize>,
    attribute: Option<String>,
    assigned: bool,
}

pub struct Runtime {
    date_host: DateHost,
    environments: Vec<Environment>,
    functions: Vec<Function>,
    arrays: Vec<Vec<Value>>,
    array_properties: Vec<usize>,
    array_holes: Vec<BTreeSet<usize>>,
    array_lengths: Vec<array_properties::ArrayLength>,
    active_array_joins: Vec<Value>,
    array_prototype: Option<usize>,
    objects: Vec<ScriptObject>,
    prototypes: BTreeMap<&'static str, usize>,
    native_properties: BTreeMap<String, usize>,
    symbols: symbols::State,
    iterators: iterators::State,
    array_buffers: array_buffer::State,
    data_views: data_view::State,
    dom_prototypes: dom_prototypes::State,
    host_symbol_objects: BTreeMap<property_keys::HostKey, usize>,
    function_prototype: usize,
    events: Vec<EventState>,
    abort_signals: Vec<AbortState>,
    listeners: Vec<EventListener>,
    event_listeners: BTreeMap<(EventTarget, JsString), Vec<usize>>,
    event_handlers: BTreeMap<(EventTarget, JsString), EventHandler>,
    readiness_fired: bool,
    started: std::time::Instant,
    steps: usize,
    allocated: usize,
    calls: usize,
    // Diagnostic active-expression count; iterative JavaScript uses frame storage.
    eval_depth: usize,
    frames: Vec<machine::Frame>,
    json_depth: usize,
    stack_units: usize,
    pub console: Vec<String>,
    pub last_default_prevented: bool,
    tracked_global_keys: [JsString; 5],
    next_global_order: u64,
    global_non_scalar: BTreeMap<JsString, window::GlobalProperty>,
}

/// Uses the panicking convenience constructor [`Runtime::new`].
impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
impl Runtime {
    /// Construct a realm, panicking if bounded initialization fails.
    /// Embedders that need to report initialization failure should use
    /// [`Runtime::try_new`]. This constructor does not access the filesystem.
    pub fn new() -> Self {
        Self::try_new().expect("runtime bootstrap failed")
    }

    /// Construct a realm, returning checked initialization failures unchanged.
    /// This constructor does not access the filesystem. Local Date operations
    /// require an explicitly supplied host; UTC operations remain available.
    /// Successful construction starts a fresh execution work budget;
    /// initialization allocations remain charged to the realm.
    pub fn try_new() -> Result<Self> {
        let mut runtime = Self::uninitialized().finish_bootstrap()?;
        runtime.steps = MAX_STEPS;
        Ok(runtime)
    }

    fn uninitialized() -> Self {
        let mut bindings = BTreeMap::new();
        for (order, (name, value)) in [
            ("undefined", Value::Undefined),
            ("NaN", Value::Number(f64::NAN)),
            ("Infinity", Value::Number(f64::INFINITY)),
            ("document", Value::Document),
            ("window", Value::Window),
            ("globalThis", Value::Window),
            ("console", Value::Console),
            ("Math", Value::Math),
            ("JSON", Value::Json),
        ]
        .into_iter()
        .enumerate()
        {
            bindings.insert(
                name.to_owned(),
                Binding {
                    value,
                    accessor: None,
                    mutable: matches!(name, "globalThis" | "Math" | "JSON"),
                    initialized: true,
                    strict_immutable: false,
                    global_property: true,
                    global_order: order as u64,
                    enumerable: !matches!(
                        name,
                        "globalThis" | "undefined" | "NaN" | "Infinity" | "Math" | "JSON"
                    ),
                    deletable: matches!(name, "globalThis" | "Math" | "JSON"),
                },
            );
        }
        let native_names = [
            "String",
            "Number",
            "Boolean",
            "Date",
            "Symbol",
            "parseInt",
            "parseFloat",
            "encodeURI",
            "encodeURIComponent",
            "decodeURI",
            "decodeURIComponent",
            "isNaN",
            "isFinite",
            "Object",
            "Array",
            "ArrayBuffer",
            "DataView",
            "Function",
            "RegExp",
            "Event",
            "CustomEvent",
            "ToggleEvent",
            "EventTarget",
            "DOMException",
            "AbortController",
            "AbortSignal",
            "Error",
            "TypeError",
            "SyntaxError",
            "ReferenceError",
            "RangeError",
            "EvalError",
            "URIError",
            "eval",
        ];
        for name in native_names {
            let global_order = bindings.len() as u64;
            bindings.insert(
                name.to_owned(),
                Binding {
                    value: Self::native(name, Value::Window),
                    accessor: None,
                    mutable: true,
                    initialized: true,
                    strict_immutable: false,
                    global_property: true,
                    global_order,
                    enumerable: false,
                    deletable: true,
                },
            );
        }
        let initial_binding_bytes = bindings.len()
            * (std::mem::size_of::<Option<Rc<BindingAccessor>>>() + std::mem::size_of::<u64>());
        let next_global_order = bindings.len() as u64;
        Self {
            date_host: DateHost::unconfigured(),
            environments: vec![
                Environment {
                    bindings,
                    this_binding: Some(Value::Window),
                    new_target_binding: None,
                    parent: None,
                    function_scope: true,
                    strict: false,
                },
                Environment {
                    bindings: BTreeMap::new(),
                    this_binding: None,
                    new_target_binding: None,
                    parent: Some(0),
                    function_scope: false,
                    strict: false,
                },
            ],
            functions: Vec::new(),
            arrays: Vec::new(),
            array_properties: Vec::new(),
            array_holes: Vec::new(),
            array_lengths: Vec::new(),
            active_array_joins: Vec::new(),
            array_prototype: None,
            objects: Vec::new(),
            prototypes: BTreeMap::new(),
            native_properties: BTreeMap::new(),
            symbols: symbols::State::default(),
            iterators: iterators::State::default(),
            array_buffers: array_buffer::State::default(),
            data_views: data_view::State::default(),
            dom_prototypes: dom_prototypes::State::default(),
            host_symbol_objects: BTreeMap::new(),
            function_prototype: 0,
            events: Vec::new(),
            abort_signals: Vec::new(),
            listeners: Vec::new(),
            event_listeners: BTreeMap::new(),
            event_handlers: BTreeMap::new(),
            readiness_fired: false,
            started: std::time::Instant::now(),
            steps: MAX_STEPS,
            allocated: 2048
                + std::mem::size_of::<DateHost>()
                + std::mem::size_of::<iterators::State>()
                + std::mem::size_of::<array_buffer::State>()
                + std::mem::size_of::<data_view::State>()
                + std::mem::size_of::<dom_prototypes::State>()
                + 4 * std::mem::size_of::<Option<Value>>()
                + initial_binding_bytes
                + native_names.len() * NATIVE_METADATA_BYTES
                + TrackedGlobal::ALL
                    .iter()
                    .map(|kind| 64 + kind.name().len() * 6)
                    .sum::<usize>(),
            calls: 0,
            eval_depth: 0,
            frames: Vec::new(),
            json_depth: 0,
            stack_units: 0,
            console: Vec::new(),
            last_default_prevented: false,
            tracked_global_keys: TrackedGlobal::ALL.map(|kind| kind.name().into()),
            next_global_order,
            global_non_scalar: BTreeMap::new(),
        }
    }

    fn finish_bootstrap(mut self) -> Result<Self> {
        self.reserve_bootstrap_objects()?;
        #[cfg(test)]
        let bootstrap_object_capacity = self.objects.capacity();
        machine::initialize(&mut self)?;
        self.initialize_intrinsics()?;
        #[cfg(test)]
        assert_eq!(self.objects.capacity(), bootstrap_object_capacity);
        Ok(self)
    }

    /// Construct a realm with an explicitly supplied clock and local timezone,
    /// panicking if bounded initialization fails. Use [`Runtime::try_with_date_host`]
    /// when the caller needs to report initialization failure.
    pub fn with_date_host(date_host: DateHost) -> Self {
        Self::try_with_date_host(date_host).expect("runtime bootstrap failed")
    }

    /// Construct a realm with an explicitly supplied clock and local timezone,
    /// returning checked initialization failures unchanged. The supplied host is
    /// installed after bootstrap; initialization does not read its clock or zone.
    pub fn try_with_date_host(date_host: DateHost) -> Result<Self> {
        let mut runtime = Self::try_new()?;
        runtime.date_host = date_host;
        Ok(runtime)
    }

    pub fn parse_only(source: &str) -> Result<()> {
        parser::Parser::program(source).map(|_| ())
    }
    /// Parse a script using a caller-supplied strict parse goal without altering
    /// its source bytes (used by the unchanged Test262 mode adapter).
    pub fn parse_only_strict(source: &str) -> Result<()> {
        parser::Parser::program_context(source, false, true).map(|_| ())
    }

    fn initialize_intrinsics(&mut self) -> Result<()> {
        for name in [
            "Object",
            "Function",
            "Array",
            "ArrayBuffer",
            "DataView",
            "String",
            "Number",
            "Boolean",
            "Date",
            "Symbol",
            "RegExp",
            "Error",
            "TypeError",
            "SyntaxError",
            "ReferenceError",
            "RangeError",
            "EvalError",
            "URIError",
            "Event",
            "CustomEvent",
            "ToggleEvent",
            "EventTarget",
            "DOMException",
            "AbortController",
            "AbortSignal",
        ] {
            let Value::Object(id) = self.object_ordered([])? else {
                unreachable!()
            };
            if matches!(
                name,
                "TypeError"
                    | "SyntaxError"
                    | "ReferenceError"
                    | "RangeError"
                    | "EvalError"
                    | "URIError"
            ) {
                self.objects[id].prototype = Some(Value::Object(self.prototypes["Error"]));
            }
            self.prototypes.insert(name, id);
        }
        self.function_prototype = self.functions.len();
        self.objects[self.prototypes["CustomEvent"]].prototype =
            Some(Value::Object(self.prototypes["Event"]));
        self.objects[self.prototypes["ToggleEvent"]].prototype =
            Some(Value::Object(self.prototypes["Event"]));
        self.objects[self.prototypes["DOMException"]].prototype =
            Some(Value::Object(self.prototypes["Error"]));
        self.objects[self.prototypes["AbortSignal"]].prototype =
            Some(Value::Object(self.prototypes["EventTarget"]));
        self.charge(
            std::mem::size_of::<code::Unit>() + std::mem::size_of::<code::Function>() + 64,
        )?;
        self.functions.push(Function {
            code: code::FunctionRef::empty()?,
            environment: 0,
            properties: self.prototypes["Function"],
            bound: None,
        });
        self.objects[self.prototypes["Function"]]
            .insert_hidden("name".into(), Value::String(JsString::default()));
        self.objects[self.prototypes["Function"]]
            .insert_hidden("length".into(), Value::Number(0.0));
        let thrower = self.alloc_native("ThrowTypeError", Value::Undefined)?;
        for name in ["caller", "arguments"] {
            self.objects[self.prototypes["Function"]].insert_property(
                name.into(),
                Property {
                    value: PropertyValue::Accessor {
                        get: thrower.clone(),
                        set: thrower.clone(),
                    },
                    enumerable: false,
                    configurable: true,
                },
            );
        }
        self.initialize_dom_parent_bindings()?;
        let Value::Array(array_prototype) = self.array(Vec::new())? else {
            unreachable!()
        };
        self.array_properties[array_prototype] = self.prototypes["Array"];
        self.array_prototype = Some(array_prototype);
        for name in [
            "Object",
            "Function",
            "Array",
            "ArrayBuffer",
            "DataView",
            "String",
            "Number",
            "Boolean",
            "Date",
            "Symbol",
            "RegExp",
            "Error",
            "TypeError",
            "SyntaxError",
            "ReferenceError",
            "RangeError",
            "EvalError",
            "URIError",
            "Event",
            "CustomEvent",
            "ToggleEvent",
            "EventTarget",
            "DOMException",
            "AbortController",
            "AbortSignal",
        ] {
            let constructor = self.alloc_native(name, Value::Window)?;
            let prototype = self.prototypes[name];
            let Value::Object(properties) = self.object_ordered([
                (
                    "length".into(),
                    Value::Number(match name {
                        "RegExp" => 2.0,
                        "Date" => 7.0,
                        "EventTarget" | "DOMException" | "AbortController" | "AbortSignal" => 0.0,
                        _ => 1.0,
                    }),
                ),
                ("name".into(), Value::String(name.into())),
                (
                    "prototype".into(),
                    if name == "Function" {
                        Value::Function(self.function_prototype)
                    } else if name == "Array" {
                        Value::Array(array_prototype)
                    } else {
                        Value::Object(prototype)
                    },
                ),
            ])?
            else {
                unreachable!()
            };
            self.objects[properties].prototype = Some(Value::Function(self.function_prototype));
            self.objects[properties].attributes("name", false, false, true);
            self.objects[properties].attributes("length", false, false, true);
            self.objects[properties].attributes("prototype", false, false, false);
            self.native_properties.insert(name.into(), properties);
            self.objects[prototype].insert_hidden("constructor".into(), constructor);
            if name.ends_with("Error") {
                self.objects[prototype].insert_hidden("name".into(), Value::String(name.into()));
                self.objects[prototype]
                    .insert_hidden("message".into(), Value::String(JsString::default()));
            }
        }
        for (prototype, key, method) in [
            ("Object", "toString", "Object.toString"),
            ("Object", "valueOf", "Object.valueOf"),
            ("Object", "hasOwnProperty", "Object.hasOwnProperty"),
            ("Object", "isPrototypeOf", "Object.isPrototypeOf"),
            (
                "Object",
                "propertyIsEnumerable",
                "Object.propertyIsEnumerable",
            ),
            ("Function", "call", "Function.call"),
            ("Function", "apply", "Function.apply"),
            ("Function", "bind", "Function.bind"),
            ("Error", "toString", "Error.toString"),
            ("Array", "map", "Array.map"),
            ("Array", "join", "Array.join"),
            ("Array", "push", "Array.push"),
            ("Array", "toString", "Array.toString"),
            ("String", "toString", "String.toString"),
            ("String", "valueOf", "String.valueOf"),
            ("Number", "toString", "Number.toString"),
            ("Number", "valueOf", "Number.valueOf"),
            ("Boolean", "toString", "Boolean.toString"),
            ("Boolean", "valueOf", "Boolean.valueOf"),
        ] {
            let id = self.prototypes[prototype];
            let value = self.alloc_native(method, Value::Undefined)?;
            self.objects[id].insert_hidden(key.into(), value);
        }
        let properties = self.native_properties["Object"];
        for method in [
            "create",
            "getPrototypeOf",
            "setPrototypeOf",
            "keys",
            "values",
            "defineProperty",
            "defineProperties",
            "getOwnPropertyDescriptor",
            "getOwnPropertyNames",
            "preventExtensions",
            "isExtensible",
        ] {
            let value = self.alloc_native(&format!("Object.{method}"), Value::Undefined)?;
            self.objects[properties].insert_hidden(method.into(), value);
        }
        let properties = self.native_properties["Number"];
        for (key, value) in [
            ("NaN", f64::NAN),
            ("POSITIVE_INFINITY", f64::INFINITY),
            ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
        ] {
            self.objects[properties].insert_hidden(key.into(), Value::Number(value));
            self.objects[properties].attributes(key, false, false, false);
        }
        self.objects[self.prototypes["Function"]].attributes("name", false, false, true);
        self.objects[self.prototypes["Function"]].attributes("length", false, false, true);
        self.objects[self.prototypes["String"]].boxed = Some(Value::String(JsString::default()));
        self.objects[self.prototypes["Number"]].boxed = Some(Value::Number(0.0));
        self.objects[self.prototypes["Boolean"]].boxed = Some(Value::Bool(false));
        for (name, key, length) in [
            ("String", "concat", 1),
            ("String", "charAt", 1),
            ("String", "charCodeAt", 1),
            ("String", "codePointAt", 1),
            ("String", "slice", 2),
            ("String", "substring", 2),
            ("String", "includes", 1),
            ("String", "startsWith", 1),
            ("String", "endsWith", 1),
            ("String", "indexOf", 1),
            ("String", "lastIndexOf", 1),
            ("String", "split", 2),
            ("String", "match", 1),
            ("String", "search", 1),
            ("String", "replace", 2),
            ("RegExp", "exec", 1),
            ("RegExp", "test", 1),
            ("RegExp", "toString", 0),
            ("String", "trim", 0),
            ("String", "toUpperCase", 0),
            ("String", "toLowerCase", 0),
            ("Array", "pop", 0),
            ("Array", "shift", 0),
            ("Array", "unshift", 1),
            ("Array", "forEach", 1),
            ("Array", "every", 1),
            ("Array", "some", 1),
            ("Array", "find", 1),
            ("Array", "findIndex", 1),
            ("Array", "findLast", 1),
            ("Array", "findLastIndex", 1),
            ("Array", "filter", 1),
            ("Array", "includes", 1),
            ("Array", "indexOf", 1),
            ("Array", "lastIndexOf", 1),
            ("Array", "slice", 2),
            ("Array", "reverse", 0),
            ("Array", "sort", 1),
            ("Array", "reduce", 1),
            ("Array", "reduceRight", 1),
            ("Number", "toString", 1),
        ] {
            let full = format!("{name}.{key}");
            let value = self.intrinsic_function(&full, key, length)?;
            self.objects[self.prototypes[name]].insert_hidden(key.into(), value);
        }
        for key in [
            "source",
            "flags",
            "global",
            "ignoreCase",
            "multiline",
            "dotAll",
            "sticky",
            "hasIndices",
            "unicode",
            "unicodeSets",
        ] {
            let get =
                self.intrinsic_function(&format!("RegExp.get.{key}"), &format!("get {key}"), 0)?;
            self.objects[self.prototypes["RegExp"]].insert_property(
                key.into(),
                Property {
                    value: PropertyValue::Accessor {
                        get,
                        set: Value::Undefined,
                    },
                    enumerable: false,
                    configurable: true,
                },
            );
        }
        for (owner, key, length) in [
            ("String", "fromCharCode", 1),
            ("String", "fromCodePoint", 1),
            ("Array", "isArray", 1),
            ("Object", "seal", 1),
            ("Object", "freeze", 1),
            ("Object", "isSealed", 1),
            ("Object", "isFrozen", 1),
        ] {
            let full = format!("{owner}.{key}");
            let value = self.intrinsic_function(&full, key, length)?;
            self.objects[self.native_properties[owner]].insert_hidden(key.into(), value);
        }
        self.install_array_from()?;
        self.install_array_splice()?;
        self.install_array_concat()?;
        self.initialize_number_statics()?;
        for name in [
            "isFinite",
            "isNaN",
            "encodeURI",
            "encodeURIComponent",
            "decodeURI",
            "decodeURIComponent",
        ] {
            // Cover literal metadata/native strings and registry storage before
            // intrinsic_function creates its separately charged property bag.
            self.work(1 + name.len())?;
            self.charge(1024 + name.len() * 6)?;
            let function = self.intrinsic_function(name, name, 1)?;
            let binding = self.environments[0].bindings.get_mut(name).unwrap();
            binding.value = function;
            binding.enumerable = false;
        }
        for (name, length) in [("parseInt", 2), ("parseFloat", 1)] {
            self.work(1 + name.len())?;
            // Native metadata, registry strings and Number's additional key.
            self.charge(1280 + name.len() * 10)?;
            let function = self.intrinsic_function(name, name, length)?;
            let binding = self.environments[0].bindings.get_mut(name).unwrap();
            binding.value = function.clone();
            binding.enumerable = false;
            self.objects[self.native_properties["Number"]].insert_hidden(name.into(), function);
        }
        for name in ["JSON", "Math"] {
            let Value::Object(id) = self.object_ordered([])? else {
                unreachable!()
            };
            self.native_properties.insert(name.into(), id);
        }
        for (key, length) in [("parse", 2), ("stringify", 3)] {
            let function = self.intrinsic_function(&format!("JSON.{key}"), key, length)?;
            self.objects[self.native_properties["JSON"]].insert_hidden(key.into(), function);
        }
        for (key, length) in [
            ("abs", 1),
            ("floor", 1),
            ("ceil", 1),
            ("round", 1),
            ("trunc", 1),
            ("sqrt", 1),
            ("pow", 2),
            ("sin", 1),
            ("cos", 1),
            ("tan", 1),
            ("log", 1),
            ("exp", 1),
            ("sign", 1),
            ("min", 2),
            ("max", 2),
        ] {
            let function = self.intrinsic_function(&format!("Math.{key}"), key, length)?;
            self.objects[self.native_properties["Math"]].insert_hidden(key.into(), function);
        }
        for (key, value) in [("PI", std::f64::consts::PI), ("E", std::f64::consts::E)] {
            self.objects[self.native_properties["Math"]].insert_property(
                key.into(),
                Property::data(Value::Number(value), false, false, false),
            );
        }
        self.initialize_events()?;
        self.initialize_date()?;
        for (name, length) in [
            ("getPropertyValue", 1),
            ("getPropertyPriority", 1),
            ("setProperty", 2),
            ("removeProperty", 1),
            ("item", 1),
        ] {
            self.intrinsic_function(&format!("CSSStyleDeclaration.{name}"), name, length)?;
        }
        let supports = self.intrinsic_function("CSS.supports", "supports", 1)?;
        let namespace = self.object_ordered([("supports".into(), supports)])?;
        if let Value::Object(id) = namespace {
            self.objects[id].namespace = Some("CSS");
        }
        self.charge(BINDING_BYTES)?;
        let global_order = self.global_creation_order("CSS")?;
        self.environments[0].bindings.insert(
            "CSS".into(),
            Binding {
                value: namespace,
                accessor: None,
                mutable: true,
                initialized: true,
                strict_immutable: false,
                global_property: true,
                global_order,
                enumerable: false,
                deletable: true,
            },
        );
        // Give every stored intrinsic method a stable ordinary property bag.
        let mut methods = Vec::new();
        for object in &self.objects {
            for property in object.values.values() {
                if let PropertyValue::Data {
                    value: Value::Native(native),
                    ..
                } = &property.value
                    && native.name.contains('.')
                    && !self.native_properties.contains_key(&native.name)
                {
                    methods.push(native.name.clone());
                }
            }
        }
        for full in methods {
            let key = full.rsplit('.').next().unwrap();
            let length = match key {
                "call" | "bind" => 1,
                "apply"
                | "create"
                | "setPrototypeOf"
                | "defineProperties"
                | "getOwnPropertyDescriptor" => 2,
                "defineProperty" => 3,
                "toString" | "valueOf" => 0,
                _ => 1,
            };
            self.intrinsic_function(&full, key, length)?;
        }
        let get = self.intrinsic_function("Window.get.self", "get self", 0)?;
        let set = self.intrinsic_function("Window.set.self", "set self", 1)?;
        let key = self.global_key(TrackedGlobal::WindowSelf);
        self.define_own(
            &Value::Window,
            &key,
            PropertyDescriptor {
                get: Some(get),
                set: Some(set),
                enumerable: Some(true),
                configurable: Some(true),
                ..PropertyDescriptor::default()
            },
        )?;
        self.initialize_dom_prototypes()?;
        Ok(())
    }

    fn initialize_number_statics(&mut self) -> Result<()> {
        let properties = self.native_properties["Number"];
        for (key, value) in [
            ("EPSILON", f64::EPSILON),
            ("MAX_SAFE_INTEGER", 9_007_199_254_740_991.0),
            ("MIN_SAFE_INTEGER", -9_007_199_254_740_991.0),
            ("MAX_VALUE", f64::MAX),
            // ECMAScript MIN_VALUE is the smallest positive subnormal here.
            ("MIN_VALUE", f64::from_bits(1)),
        ] {
            self.work(1 + key.len())?;
            self.charge(288 + key.len() * 4)?;
            self.objects[properties].insert_property(
                key.into(),
                Property::data(Value::Number(value), false, false, false),
            );
        }
        for (key, full) in [
            ("isFinite", "Number.isFinite"),
            ("isNaN", "Number.isNaN"),
            ("isInteger", "Number.isInteger"),
            ("isSafeInteger", "Number.isSafeInteger"),
        ] {
            // Before construction, cover literal UTF-16 keys/name, the Native
            // record and its string, and registry/constructor property entries.
            // intrinsic_function separately charges its ordinary property bag.
            self.work(1 + full.len() + key.len())?;
            self.charge(1024 + full.len() * 2 + key.len() * 4)?;
            let value = self.intrinsic_function(full, key, 1)?;
            self.objects[properties].insert_hidden(key.into(), value);
        }
        Ok(())
    }

    fn intrinsic_function(&mut self, full: &str, name: &str, length: usize) -> Result<Value> {
        // Admit the returned native's metadata slot before publishing its bag.
        // The raw factory below consumes this debit without a late failure.
        self.charge(NATIVE_METADATA_BYTES)?;
        if !self.native_properties.contains_key(full) {
            let Value::Object(id) = self.object_ordered([
                ("name".into(), Value::String(name.into())),
                ("length".into(), Value::Number(length as f64)),
            ])?
            else {
                unreachable!()
            };
            self.objects[id].prototype = Some(Value::Function(self.function_prototype));
            self.objects[id].attributes("name", false, false, true);
            self.objects[id].attributes("length", false, false, true);
            self.native_properties.insert(full.into(), id);
        }
        Ok(Self::native(full, Value::Undefined))
    }

    pub fn execute(&mut self, source: &str, document: &mut Document) -> Result<Value> {
        let program = parser::Parser::program(source)?;
        self.execute_program(source, program, document)
    }
    pub fn execute_strict(&mut self, source: &str, document: &mut Document) -> Result<Value> {
        let program = parser::Parser::program_context(source, false, true)?;
        self.execute_program(source, program, document)
    }
    fn execute_program(
        &mut self,
        source: &str,
        program: Rc<code::Unit>,
        document: &mut Document,
    ) -> Result<Value> {
        self.charge(source.len().saturating_mul(3))?;
        self.charge(program.compiled_storage)?;
        self.steps = MAX_STEPS;
        let saved = self.environments[1].strict;
        self.environments[1].strict = program.strict;
        let completion =
            machine::evaluate_statements(self, &program, machine::ListOwner::Program, 1, document);
        self.environments[1].strict = saved;
        match completion? {
            Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
            Flow::Return(_) => Err(ScriptError::new("return outside function")),
            _ => Err(ScriptError::new("loop control outside loop")),
        }
    }

    fn initialize_events(&mut self) -> Result<()> {
        self.objects[self.native_properties["CustomEvent"]].prototype =
            Some(self.alloc_native("Event", Value::Window)?);
        self.objects[self.native_properties["ToggleEvent"]].prototype =
            Some(self.alloc_native("Event", Value::Window)?);
        self.objects[self.native_properties["AbortSignal"]].prototype =
            Some(self.alloc_native("EventTarget", Value::Window)?);
        for (owner, key, length) in [
            ("EventTarget", "addEventListener", 2),
            ("EventTarget", "removeEventListener", 2),
            ("EventTarget", "dispatchEvent", 1),
            ("Event", "preventDefault", 0),
            ("Event", "stopPropagation", 0),
            ("Event", "stopImmediatePropagation", 0),
            ("Event", "composedPath", 0),
            ("Event", "initEvent", 1),
            ("CustomEvent", "initCustomEvent", 1),
            ("AbortController", "abort", 0),
            ("AbortSignal", "throwIfAborted", 0),
        ] {
            let value = self.intrinsic_function(&format!("{owner}.{key}"), key, length)?;
            self.objects[self.prototypes[owner]].insert(key.into(), value);
        }
        let abort = self.intrinsic_function("AbortSignal.static.abort", "abort", 0)?;
        self.objects[self.native_properties["AbortSignal"]].insert("abort".into(), abort);
        for (owner, key) in [
            ("AbortController", "signal"),
            ("AbortSignal", "aborted"),
            ("AbortSignal", "reason"),
            ("AbortSignal", "onabort"),
        ] {
            let get =
                self.intrinsic_function(&format!("{owner}.get.{key}"), &format!("get {key}"), 0)?;
            let set = if key == "onabort" {
                self.intrinsic_function("AbortSignal.set.onabort", "set onabort", 1)?
            } else {
                Value::Undefined
            };
            self.objects[self.prototypes[owner]].insert_property(
                key.into(),
                Property {
                    value: PropertyValue::Accessor { get, set },
                    enumerable: true,
                    configurable: true,
                },
            );
        }
        for key in [
            "type",
            "target",
            "srcElement",
            "currentTarget",
            "eventPhase",
            "bubbles",
            "cancelable",
            "composed",
            "defaultPrevented",
            "timeStamp",
            "cancelBubble",
            "returnValue",
            "detail",
            "oldState",
            "newState",
            "source",
        ] {
            let get =
                self.intrinsic_function(&format!("Event.get.{key}"), &format!("get {key}"), 0)?;
            let set = if matches!(key, "cancelBubble" | "returnValue") {
                self.intrinsic_function(&format!("Event.set.{key}"), &format!("set {key}"), 1)?
            } else {
                Value::Undefined
            };
            let owner = if key == "detail" {
                "CustomEvent"
            } else if matches!(key, "oldState" | "newState" | "source") {
                "ToggleEvent"
            } else {
                "Event"
            };
            self.objects[self.prototypes[owner]].insert_property(
                key.into(),
                Property {
                    value: PropertyValue::Accessor { get, set },
                    enumerable: true,
                    configurable: true,
                },
            );
        }
        self.intrinsic_function("Event.get.isTrusted", "get isTrusted", 0)?;
        for (key, number) in [
            ("NONE", 0.0),
            ("CAPTURING_PHASE", 1.0),
            ("AT_TARGET", 2.0),
            ("BUBBLING_PHASE", 3.0),
        ] {
            for id in [self.prototypes["Event"], self.native_properties["Event"]] {
                self.objects[id].insert_property(
                    key.into(),
                    Property::data(Value::Number(number), false, true, false),
                );
            }
        }
        for id in [
            self.prototypes["DOMException"],
            self.native_properties["DOMException"],
        ] {
            self.objects[id].insert_property(
                "INVALID_STATE_ERR".into(),
                Property::data(Value::Number(11.0), false, true, false),
            );
            self.objects[id].insert_property(
                "ABORT_ERR".into(),
                Property::data(Value::Number(20.0), false, true, false),
            );
        }
        self.initialize_symbols()?;
        self.install_array_buffer_intrinsics()?;
        self.install_data_view_intrinsics()?;
        self.install_object_is_intrinsic()?;
        self.install_iterator_intrinsics()?;
        self.initialize_dom_bindings()
    }
    fn abort_signal_index(&self, value: &Value) -> Result<usize> {
        if let Value::Object(id) = value
            && let Some(AbortSlot::Signal(index)) = self.objects[*id].abort
        {
            return Ok(index);
        }
        Err(ScriptError::type_error("receiver is not an AbortSignal"))
    }
    fn abort_signal_object(&mut self, reason: Value) -> Result<Value> {
        self.charge(std::mem::size_of::<AbortState>())?;
        let value = self.object_ordered([])?;
        let Value::Object(id) = value else {
            unreachable!()
        };
        self.objects[id].abort = Some(AbortSlot::Signal(self.abort_signals.len()));
        self.objects[id].event_target = true;
        self.objects[id].prototype = Some(Value::Object(self.prototypes["AbortSignal"]));
        self.abort_signals.push(AbortState {
            reason,
            listeners: Vec::new(),
        });
        Ok(value)
    }
    fn abort_reason(&mut self, reason: Value) -> Result<Value> {
        if reason == Value::Undefined {
            self.dom_exception("AbortError".into(), "The operation was aborted.".into())
        } else {
            Ok(reason)
        }
    }
    fn abort_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        if name == "AbortSignal.static.abort" {
            let reason = self.abort_reason(args.first().cloned().unwrap_or(Value::Undefined))?;
            return self.abort_signal_object(reason);
        }
        if name.starts_with("AbortController.") {
            let Value::Object(controller) = receiver else {
                return Err(ScriptError::type_error(
                    "receiver is not an AbortController",
                ));
            };
            let Some(AbortSlot::Controller(signal)) = self.objects[controller].abort else {
                return Err(ScriptError::type_error(
                    "receiver is not an AbortController",
                ));
            };
            if name == "AbortController.get.signal" {
                return Ok(Value::Object(signal));
            }
            let index = self.abort_signal_index(&Value::Object(signal))?;
            if self.abort_signals[index].reason != Value::Undefined {
                return Ok(Value::Undefined);
            }
            // No user callbacks run until all removals have completed. Reserve
            // their entire work before making observable state changes, so an
            // exhausted quota cannot leave a partially detached observer list.
            self.work(self.abort_signals[index].listeners.len())?;
            let reason = self.abort_reason(args.first().cloned().unwrap_or(Value::Undefined))?;
            let event =
                self.event_object("abort".into(), false, false, false, Value::Null, false)?;
            let event_index = self.event_index(&event)?;
            self.abort_signals[index].reason = reason;
            let listeners = std::mem::take(&mut self.abort_signals[index].listeners);
            for listener in listeners {
                self.listeners[listener].removed = true;
            }
            self.events[event_index].trusted = true;
            self.dispatch_event_object(EventTarget::Object(signal), event, None, doc)?;
            return Ok(Value::Undefined);
        }
        let index = self.abort_signal_index(&receiver)?;
        match name {
            "AbortSignal.get.aborted" => Ok(Value::Bool(
                self.abort_signals[index].reason != Value::Undefined,
            )),
            "AbortSignal.get.reason" => Ok(self.abort_signals[index].reason.clone()),
            "AbortSignal.throwIfAborted" => {
                let reason = self.abort_signals[index].reason.clone();
                if reason == Value::Undefined {
                    Ok(Value::Undefined)
                } else {
                    Err(self.thrown_error(reason)?)
                }
            }
            "AbortSignal.get.onabort" => {
                let target = self.event_target(&receiver, doc)?;
                self.event_listener_lookup_work(&"abort".into())?;
                self.event_handler_callback(target, &"abort".into())
            }
            "AbortSignal.set.onabort" => {
                let target = self.event_target(&receiver, doc)?;
                self.event_listener_lookup_work(&"abort".into())?;
                self.set_event_handler(
                    target,
                    "abort".into(),
                    args.first().cloned().unwrap_or(Value::Undefined),
                    true,
                )?;
                Ok(Value::Undefined)
            }
            _ => unreachable!("unknown AbortSignal native operation"),
        }
    }
    fn event_index(&self, value: &Value) -> Result<usize> {
        if let Value::Object(id) = value
            && let Some(index) = self.objects[*id].event
        {
            return Ok(index);
        }
        Err(ScriptError::type_error("receiver is not an Event"))
    }
    fn event_target(&self, value: &Value, doc: &Document) -> Result<EventTarget> {
        match value {
            Value::Window => Ok(EventTarget::Window),
            Value::Document => Ok(EventTarget::Document),
            Value::Node(id) if *id == doc.root => Ok(EventTarget::Document),
            Value::Node(id) if *id < doc.nodes.len() => Ok(EventTarget::Node(*id)),
            Value::Object(id) if self.objects[*id].event_target => Ok(EventTarget::Object(*id)),
            _ => Err(ScriptError::type_error("receiver is not an EventTarget")),
        }
    }
    fn event_object(
        &mut self,
        event_type: JsString,
        bubbles: bool,
        cancelable: bool,
        composed: bool,
        detail: Value,
        custom: bool,
    ) -> Result<Value> {
        self.work(1 + event_type.len())?;
        self.charge(std::mem::size_of::<EventState>() + 192)?;
        let value = self.object_ordered([])?;
        let Value::Object(id) = value else {
            unreachable!()
        };
        self.objects[id].prototype = Some(Value::Object(
            self.prototypes[if custom { "CustomEvent" } else { "Event" }],
        ));
        self.objects[id].event = Some(self.events.len());
        let trusted_getter = self.alloc_native("Event.get.isTrusted", Value::Undefined)?;
        self.objects[id].insert_property(
            "isTrusted".into(),
            Property {
                value: PropertyValue::Accessor {
                    get: trusted_getter,
                    set: Value::Undefined,
                },
                enumerable: true,
                configurable: false,
            },
        );
        self.events.push(EventState {
            event_type,
            bubbles,
            cancelable,
            composed,
            detail,
            custom,
            toggle: None,
            target: Value::Null,
            current_target: Value::Null,
            phase: 0,
            stopped: false,
            immediate: false,
            canceled: false,
            passive: false,
            dispatching: false,
            initialized: true,
            trusted: false,
            timestamp: self.started.elapsed().as_millis() as f64,
            path: Vec::new(),
        });
        Ok(value)
    }
    fn event_construct(&mut self, name: &str, args: &[Value], doc: &mut Document) -> Result<Value> {
        if name == "AbortSignal" {
            return Err(ScriptError::type_error(
                "AbortSignal has no public constructor",
            ));
        }
        if name == "AbortController" {
            let signal = self.abort_signal_object(Value::Undefined)?;
            let value = self.object_ordered([])?;
            let (Value::Object(signal), Value::Object(id)) = (signal, &value) else {
                unreachable!()
            };
            self.objects[*id].abort = Some(AbortSlot::Controller(signal));
            self.objects[*id].prototype = Some(Value::Object(self.prototypes["AbortController"]));
            return Ok(value);
        }
        if name == "EventTarget" {
            let value = self.object_ordered([])?;
            let Value::Object(id) = value else {
                unreachable!()
            };
            self.objects[id].event_target = true;
            self.objects[id].prototype = Some(Value::Object(self.prototypes["EventTarget"]));
            return Ok(value);
        }
        if name == "DOMException" {
            let message = self.json_text(
                args.first()
                    .filter(|value| **value != Value::Undefined)
                    .cloned()
                    .unwrap_or(Value::String("".into())),
                doc,
                &mut Vec::new(),
            )?;
            let name = self.json_text(
                args.get(1)
                    .filter(|value| **value != Value::Undefined)
                    .cloned()
                    .unwrap_or(Value::String("Error".into())),
                doc,
                &mut Vec::new(),
            )?;
            return self.dom_exception(name, message);
        }
        let Some(kind) = args.first() else {
            return Err(ScriptError::type_error("Event requires a type"));
        };
        let kind = if name == "ToggleEvent" {
            self.string_hint(kind.clone(), doc)?
        } else {
            self.json_text(kind.clone(), doc, &mut Vec::new())?
        };
        let init = args.get(1).cloned().unwrap_or(Value::Undefined);
        if !matches!(init, Value::Null | Value::Undefined) && !js_object(&init) {
            return Err(ScriptError::type_error(
                "event initialization dictionary must be an object",
            ));
        }
        let mut flags = [false; 3];
        for (i, key) in ["bubbles", "cancelable", "composed"]
            .into_iter()
            .enumerate()
        {
            if js_object(&init) {
                flags[i] = self.get(init.clone(), key, doc)?.truthy();
            }
        }
        if name == "ToggleEvent" {
            // Web IDL converts dictionary members in lexicographic order,
            // after converting all inherited dictionary members.
            let mut states = [JsString::default(), JsString::default()];
            let mut source = Value::Null;
            if js_object(&init) {
                for (index, key) in ["newState", "oldState"].into_iter().enumerate() {
                    let value = self.get(init.clone(), key, doc)?;
                    if value != Value::Undefined {
                        states[index] = self.string_hint(value, doc)?;
                    }
                }
                source = match self.get(init, "source", doc)? {
                    Value::Null | Value::Undefined => Value::Null,
                    Value::Node(node)
                        if matches!(
                            doc.nodes.get(node).map(|node| &node.kind),
                            Some(NodeKind::Element(_))
                        ) =>
                    {
                        Value::Node(node)
                    }
                    _ => {
                        return Err(ScriptError::type_error(
                            "ToggleEvent source must be an Element or null",
                        ));
                    }
                };
            }
            let [new_state, old_state] = states;
            let event =
                self.event_object(kind, flags[0], flags[1], flags[2], Value::Null, false)?;
            self.attach_toggle_state(&event, old_state, new_state, source)?;
            return Ok(event);
        }
        let custom = name == "CustomEvent";
        let detail = if custom && js_object(&init) {
            match self.get(init, "detail", doc)? {
                Value::Undefined => Value::Null,
                value => value,
            }
        } else {
            Value::Null
        };
        self.event_object(kind, flags[0], flags[1], flags[2], detail, custom)
    }
    fn attach_toggle_state(
        &mut self,
        event: &Value,
        old_state: JsString,
        new_state: JsString,
        source: Value,
    ) -> Result<()> {
        let id = self.event_index(event)?;
        let Value::Object(object) = event else {
            unreachable!()
        };
        self.charge(old_state.byte_len().saturating_add(new_state.byte_len()))?;
        self.objects[*object].prototype = Some(Value::Object(self.prototypes["ToggleEvent"]));
        self.events[id].toggle = Some(ToggleState {
            old_state,
            new_state,
            source,
        });
        Ok(())
    }
    fn dom_exception(&mut self, name: JsString, message: JsString) -> Result<Value> {
        let code = match name.to_utf8().as_deref() {
            Ok("IndexSizeError") => 1.0,
            Ok("InvalidStateError") => 11.0,
            Ok("NotSupportedError") => 9.0,
            Ok("SyntaxError") => 12.0,
            Ok("InvalidCharacterError") => 5.0,
            Ok("NotFoundError") => 8.0,
            Ok("HierarchyRequestError") => 3.0,
            Ok("AbortError") => 20.0,
            _ => 0.0,
        };
        let value = self.object_ordered([
            ("name".into(), Value::String(name)),
            ("message".into(), Value::String(message)),
            ("code".into(), Value::Number(code)),
        ])?;
        let Value::Object(id) = value else {
            unreachable!()
        };
        self.objects[id].prototype = Some(Value::Object(self.prototypes["DOMException"]));
        for key in ["name", "message", "code"] {
            self.objects[id].attributes(key, false, false, true);
        }
        Ok(value)
    }
    fn event_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let id = self.event_index(&receiver)?;
        if let Some(key) = name.strip_prefix("get.") {
            let state = &self.events[id];
            return Ok(match key {
                "type" => Value::String(state.event_type.clone()),
                "target" | "srcElement" => state.target.clone(),
                "currentTarget" => state.current_target.clone(),
                "eventPhase" => Value::Number(state.phase.into()),
                "bubbles" => Value::Bool(state.bubbles),
                "cancelable" => Value::Bool(state.cancelable),
                "composed" => Value::Bool(state.composed),
                "defaultPrevented" => Value::Bool(state.canceled),
                "cancelBubble" => Value::Bool(state.stopped),
                "returnValue" => Value::Bool(!state.canceled),
                "isTrusted" => Value::Bool(state.trusted),
                "timeStamp" => Value::Number(state.timestamp),
                "detail" if state.custom => state.detail.clone(),
                "oldState" if state.toggle.is_some() => {
                    Value::String(state.toggle.as_ref().unwrap().old_state.clone())
                }
                "newState" if state.toggle.is_some() => {
                    Value::String(state.toggle.as_ref().unwrap().new_state.clone())
                }
                "source" if state.toggle.is_some() => state.toggle.as_ref().unwrap().source.clone(),
                _ => {
                    return Err(ScriptError::type_error(
                        "event getter receiver is incompatible",
                    ));
                }
            });
        }
        let arg = |n| args.get(n).cloned().unwrap_or(Value::Undefined);
        match name {
            "preventDefault" | "set.returnValue" => {
                if name == "preventDefault" || !arg(0).truthy() {
                    let state = &mut self.events[id];
                    if state.cancelable && !state.passive {
                        state.canceled = true;
                    }
                }
            }
            "stopPropagation" => self.events[id].stopped = true,
            "set.cancelBubble" => {
                if arg(0).truthy() {
                    self.events[id].stopped = true;
                }
            }
            "stopImmediatePropagation" => {
                self.events[id].stopped = true;
                self.events[id].immediate = true;
            }
            "composedPath" => {
                self.work(self.events[id].path.len())?;
                self.charge(self.events[id].path.len() * std::mem::size_of::<Value>())?;
                return self.array(
                    self.events[id]
                        .path
                        .iter()
                        .map(|target| target.value())
                        .collect(),
                );
            }
            "initEvent" | "initCustomEvent" => {
                if args.is_empty() {
                    return Err(ScriptError::type_error("initEvent requires a type"));
                }
                if name == "initCustomEvent" && !self.events[id].custom {
                    return Err(ScriptError::type_error("receiver is not a CustomEvent"));
                }
                let event_type = self.json_text(arg(0), doc, &mut Vec::new())?;
                if !self.events[id].dispatching {
                    let state = &mut self.events[id];
                    state.event_type = event_type;
                    state.bubbles = arg(1).truthy();
                    state.cancelable = arg(2).truthy();
                    state.initialized = true;
                    state.stopped = false;
                    state.immediate = false;
                    state.canceled = false;
                    state.trusted = false;
                    state.target = Value::Null;
                    if name == "initCustomEvent" {
                        state.detail = match arg(3) {
                            Value::Undefined => Value::Null,
                            value => value,
                        };
                    }
                }
            }
            _ => return Err(ScriptError::unsupported("event method is not implemented")),
        }
        Ok(Value::Undefined)
    }
    fn add_listener(
        &mut self,
        target: EventTarget,
        kind: JsString,
        listener: EventListener,
    ) -> Result<usize> {
        self.charge(256 + kind.byte_len())?;
        let id = self.listeners.len();
        self.listeners.push(listener);
        self.event_listeners
            .entry((target, kind))
            .or_default()
            .push(id);
        Ok(id)
    }
    fn event_listener_lookup_work(&mut self, kind: &JsString) -> Result<()> {
        let comparisons = self.event_listeners.len().saturating_add(1).ilog2() as usize + 2;
        self.work(
            (kind.len() + 1)
                .saturating_mul(comparisons)
                .saturating_mul(4),
        )
    }
    fn event_target_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let target = self.event_target(&receiver, doc)?;
        if name == "dispatchEvent" {
            let event = args
                .first()
                .cloned()
                .ok_or_else(|| ScriptError::type_error("dispatchEvent requires an event"))?;
            let id = self.event_index(&event)?;
            if self.events[id].dispatching || !self.events[id].initialized {
                let value = self.dom_exception(
                    "InvalidStateError".into(),
                    "event is already being dispatched or is uninitialized".into(),
                )?;
                return Err(self.thrown_error(value)?);
            }
            self.events[id].trusted = false;
            return self
                .dispatch_event_object(target, event, None, doc)
                .map(Value::Bool);
        }
        if args.len() < 2 {
            return Err(ScriptError::type_error(
                "event listener operation requires type and callback",
            ));
        }
        let kind = self.json_text(args[0].clone(), doc, &mut Vec::new())?;
        let callback = args[1].clone();
        if !matches!(callback, Value::Null | Value::Undefined) && !js_object(&callback) {
            return Err(ScriptError::type_error(
                "event listener must be an object or null",
            ));
        }
        let options = args.get(2).cloned().unwrap_or(Value::Undefined);
        let dictionary = js_object(&options);
        let capture = if dictionary {
            self.get(options.clone(), "capture", doc)?.truthy()
        } else {
            options.truthy()
        };
        let mut once = false;
        let mut passive = None;
        let mut signal_index = None;
        if dictionary && name == "addEventListener" {
            once = self.get(options.clone(), "once", doc)?.truthy();
            let value = self.get(options.clone(), "passive", doc)?;
            if value != Value::Undefined {
                passive = Some(value.truthy());
            }
            let signal = self.get(options, "signal", doc)?;
            if signal != Value::Undefined {
                signal_index = Some(self.abort_signal_index(&signal)?);
            }
        }
        if signal_index.is_some_and(|id| self.abort_signals[id].reason != Value::Undefined) {
            return Ok(Value::Undefined);
        }
        if matches!(callback, Value::Null | Value::Undefined) {
            return Ok(Value::Undefined);
        }
        self.event_listener_lookup_work(&kind)?;
        self.sync_event_handler(target, &kind, doc)?;
        let key = (target, kind.clone());
        let count = self.event_listeners.get(&key).map_or(0, Vec::len);
        self.work(count + kind.len())?;
        let existing = self.event_listeners.get(&key).and_then(|ids| {
            ids.iter().copied().find(|id| {
                let item = &self.listeners[*id];
                !item.removed
                    && !item.handler
                    && item.capture == capture
                    && item.callback == callback
            })
        });
        if name == "removeEventListener" {
            if let Some(id) = existing {
                self.listeners[id].removed = true;
            }
        } else if existing.is_none() {
            let passive = match passive {
                Some(value) => value,
                None if ["touchstart", "touchmove", "wheel", "mousewheel"]
                    .iter()
                    .any(|name| kind.units().iter().copied().eq(name.encode_utf16())) =>
                {
                    match target {
                        EventTarget::Window | EventTarget::Document => true,
                        EventTarget::Node(id) => {
                            // These helpers scan document/HTML children. Charge
                            // before scanning even for a detached wheel target.
                            self.work(doc.nodes.len().saturating_mul(2))?;
                            Some(id) == document_element(doc)
                                || Some(id) == html_document_child(doc, "body")
                        }
                        _ => false,
                    }
                }
                None => false,
            };
            if signal_index.is_some() {
                self.charge(std::mem::size_of::<usize>())?;
            }
            let listener = self.add_listener(
                target,
                kind,
                EventListener {
                    callback,
                    capture,
                    once,
                    passive,
                    removed: false,
                    handler: false,
                },
            )?;
            if let Some(signal) = signal_index {
                self.abort_signals[signal].listeners.push(listener);
            }
        }
        Ok(Value::Undefined)
    }
    fn set_event_handler(
        &mut self,
        target: EventTarget,
        kind: JsString,
        value: Value,
        assigned: bool,
    ) -> Result<()> {
        let key = (target, kind.clone());
        let old = self.event_handlers.get(&key).and_then(|h| h.listener);
        // HTML EventHandler conversion treats primitive assignments as null.
        let active = js_object(&value);
        let listener = if active {
            if let Some(id) = old.filter(|id| !self.listeners[*id].removed) {
                self.listeners[id].callback = value;
                Some(id)
            } else {
                Some(self.add_listener(
                    target,
                    kind,
                    EventListener {
                        callback: value,
                        capture: false,
                        once: false,
                        passive: false,
                        removed: false,
                        handler: true,
                    },
                )?)
            }
        } else {
            if let Some(id) = old {
                self.listeners[id].removed = true;
            }
            None
        };
        if !self.event_handlers.contains_key(&key) {
            self.charge(192 + key.1.byte_len())?;
        }
        let entry = self.event_handlers.entry(key).or_insert(EventHandler {
            listener: None,
            attribute: None,
            assigned: false,
        });
        entry.listener = listener;
        entry.assigned = assigned;
        Ok(())
    }
    fn sync_event_handler(
        &mut self,
        target: EventTarget,
        kind: &JsString,
        doc: &Document,
    ) -> Result<()> {
        let EventTarget::Node(node) = target else {
            return Ok(());
        };
        // Every supported handler name is short ASCII. Arbitrary long event
        // types never require allocating or scanning an HTML attribute name.
        if kind.len() > 16 {
            return Ok(());
        }
        let Ok(kind_text) = kind.to_utf8() else {
            return Ok(());
        };
        if !event_handler_name(&format!("on{kind_text}")) {
            return Ok(());
        }
        let key = (target, kind.clone());
        if self.event_handlers.get(&key).is_some_and(|h| h.assigned) {
            return Ok(());
        }
        let source = doc.attr(node, &format!("on{kind_text}"));
        // Repeated IDL reads and dispatches revisit unchanged inline handlers.
        // String equality can scan the entire cached source, even when no
        // handler is recompiled. Charge that work before comparing its bytes.
        let comparison_bytes = source
            .zip(
                self.event_handlers
                    .get(&key)
                    .and_then(|h| h.attribute.as_deref()),
            )
            .filter(|(current, cached)| current.len() == cached.len())
            .map_or(0, |(current, _)| current.len());
        self.work(comparison_bytes)?;
        if source
            == self
                .event_handlers
                .get(&key)
                .and_then(|h| h.attribute.as_deref())
        {
            return Ok(());
        }
        if let Some(source) = source {
            self.work(source.len())?;
            self.charge(source.len() + 192 + kind.byte_len())?;
            let old = self.event_handlers.get(&key).and_then(|h| h.listener);
            let listener = if let Some(id) = old.filter(|id| !self.listeners[*id].removed) {
                self.listeners[id].callback = Value::Undefined;
                id
            } else {
                self.add_listener(
                    target,
                    kind.clone(),
                    EventListener {
                        callback: Value::Undefined,
                        capture: false,
                        once: false,
                        passive: false,
                        removed: false,
                        handler: true,
                    },
                )?
            };
            self.event_handlers.insert(
                key,
                EventHandler {
                    listener: Some(listener),
                    attribute: Some(source.into()),
                    assigned: false,
                },
            );
        } else {
            self.set_event_handler(target, kind.clone(), Value::Null, false)?;
            self.event_handlers.get_mut(&key).unwrap().attribute = None;
        }
        Ok(())
    }
    fn event_attribute_changed(&mut self, node: NodeId, name: &str, doc: &Document) -> Result<()> {
        let name = if doc.namespace(node) == Some(Namespace::Html) {
            name.to_ascii_lowercase()
        } else {
            name.into()
        };
        if !event_handler_name(&name) {
            return Ok(());
        }
        let kind = JsString::from(&name[2..]);
        let target = EventTarget::Node(node);
        if doc.attr(node, &name).is_none() {
            self.set_event_handler(target, kind.clone(), Value::Null, false)?;
            self.event_handlers
                .get_mut(&(target, kind))
                .unwrap()
                .attribute = None;
            return Ok(());
        }
        if let Some(handler) = self.event_handlers.get_mut(&(target, kind.clone())) {
            handler.assigned = false;
            // Force source replacement even when an attribute is set to the
            // same text after its corresponding IDL handler was reassigned.
            handler.attribute = None;
        }
        self.sync_event_handler(target, &kind, doc)
    }
    fn event_handler_callback(&mut self, target: EventTarget, kind: &JsString) -> Result<Value> {
        let key = (target, kind.clone());
        let Some(listener) = self
            .event_handlers
            .get(&key)
            .and_then(|h| h.listener)
            .filter(|id| !self.listeners[*id].removed)
        else {
            return Ok(Value::Null);
        };
        if self.listeners[listener].callback != Value::Undefined {
            return Ok(self.listeners[listener].callback.clone());
        }
        let source_len = self.event_handlers[&key]
            .attribute
            .as_ref()
            .map_or(0, String::len);
        self.charge(source_len)?;
        let source = self.event_handlers[&key]
            .attribute
            .clone()
            .unwrap_or_default();
        let result = (|| {
            self.work(source.len())?;
            self.charge(source.len().saturating_mul(4))?;
            let code = parser::Parser::handler(&source, format!("on{kind}"))?;
            self.charge(code.unit.compiled_storage)?;
            self.function_value(&code, 1)
        })();
        match result {
            Ok(callback) => {
                self.listeners[listener].callback = callback.clone();
                Ok(callback)
            }
            Err(error) => {
                self.listeners[listener].removed = true;
                self.event_handlers.get_mut(&key).unwrap().listener = None;
                self.report_event_error(error)?;
                Ok(Value::Null)
            }
        }
    }
    fn report_event_error(&mut self, error: ScriptError) -> Result<()> {
        if error.is_resource_limit() || error.is_unsupported() {
            return Err(error);
        }
        let message = format!("Uncaught event listener exception: {error}");
        self.work(message.len())?;
        self.charge(message.len() + 32)?;
        if self.console.len() < 1024 {
            self.console.push(message);
        }
        Ok(())
    }
    fn invoke_event_listeners(
        &mut self,
        target: EventTarget,
        event: &Value,
        id: usize,
        capture: bool,
        doc: &mut Document,
    ) -> Result<()> {
        if self.events[id].stopped {
            return Ok(());
        }
        self.events[id].current_target = target.value();
        let kind = self.events[id].event_type.clone();
        self.event_listener_lookup_work(&kind)?;
        if let Err(error) = self.sync_event_handler(target, &kind, doc) {
            self.report_event_error(error)?;
        }
        let key = (target, kind);
        let count = self.event_listeners.get(&key).map_or(0, Vec::len);
        self.work(count)?;
        self.charge(count * std::mem::size_of::<usize>())?;
        let snapshot = self.event_listeners.get(&key).cloned().unwrap_or_default();
        for listener_id in snapshot {
            self.tick()?;
            let listener = self.listeners[listener_id].clone();
            if listener.removed || listener.capture != capture {
                continue;
            }
            if listener.once {
                self.listeners[listener_id].removed = true;
            }
            self.events[id].passive = listener.passive;
            let result = (|| {
                if listener.handler {
                    let callback =
                        self.event_handler_callback(target, &self.events[id].event_type.clone())?;
                    if !json_callable(&callback) {
                        return Ok(Value::Undefined);
                    }
                    self.call(callback, vec![event.clone()], target.value(), doc)
                } else if json_callable(&listener.callback) {
                    self.call(listener.callback, vec![event.clone()], target.value(), doc)
                } else {
                    let method = self.get(listener.callback.clone(), "handleEvent", doc)?;
                    self.call(method, vec![event.clone()], listener.callback, doc)
                }
            })();
            self.events[id].passive = false;
            match result {
                Ok(Value::Bool(false)) if listener.handler && self.events[id].cancelable => {
                    self.events[id].canceled = true
                }
                Ok(_) => {}
                Err(error) => self.report_event_error(error)?,
            }
            if self.events[id].immediate {
                break;
            }
        }
        Ok(())
    }
    fn dispatch_event_object(
        &mut self,
        target: EventTarget,
        event: Value,
        override_target: Option<Value>,
        doc: &mut Document,
    ) -> Result<bool> {
        let id = self.event_index(&event)?;
        self.enter_stack(4)?;
        let result = (|| {
            let mut path = Vec::new();
            let mut cursor = Some(target);
            while let Some(target) = cursor {
                self.tick()?;
                if path.len() >= 258 {
                    return Err(ScriptError::resource("event path limit exceeded"));
                }
                self.charge(std::mem::size_of::<EventTarget>())?;
                path.push(target);
                cursor = match target {
                    EventTarget::Node(node) => doc.nodes[node].parent.map(|parent| {
                        if parent == doc.root {
                            EventTarget::Document
                        } else {
                            EventTarget::Node(parent)
                        }
                    }),
                    EventTarget::Document
                        if self.events[id].event_type != JsString::from("load") =>
                    {
                        Some(EventTarget::Window)
                    }
                    _ => None,
                };
            }
            self.events[id].dispatching = true;
            self.events[id].target = override_target.unwrap_or_else(|| target.value());
            self.events[id].path = path;
            for index in (0..self.events[id].path.len()).rev() {
                self.events[id].phase = if index == 0 { 2 } else { 1 };
                self.invoke_event_listeners(self.events[id].path[index], &event, id, true, doc)?;
            }
            for index in 0..self.events[id].path.len() {
                if index > 0 && !self.events[id].bubbles {
                    continue;
                }
                self.events[id].phase = if index == 0 { 2 } else { 3 };
                self.invoke_event_listeners(self.events[id].path[index], &event, id, false, doc)?;
            }
            Ok(!self.events[id].canceled)
        })();
        let state = &mut self.events[id];
        state.dispatching = false;
        state.passive = false;
        state.stopped = false;
        state.immediate = false;
        state.phase = 0;
        state.current_target = Value::Null;
        state.path.clear();
        self.stack_units -= 4;
        result
    }
    pub fn dispatch_dom_content_loaded(&mut self, document: &mut Document) -> Result<()> {
        if self.readiness_fired {
            return Ok(());
        }
        self.readiness_fired = true;
        self.steps = MAX_STEPS;
        let event = self.event_object(
            "DOMContentLoaded".into(),
            true,
            false,
            false,
            Value::Null,
            false,
        )?;
        let id = self.event_index(&event)?;
        self.events[id].trusted = true;
        self.dispatch_event_object(EventTarget::Document, event, None, document)?;
        let event = self.event_object("load".into(), false, false, false, Value::Null, false)?;
        let id = self.event_index(&event)?;
        self.events[id].trusted = true;
        self.dispatch_event_object(EventTarget::Window, event, Some(Value::Document), document)?;
        Ok(())
    }
    pub fn dispatch_click(&mut self, target: NodeId, document: &mut Document) -> Result<()> {
        self.dispatch_event(target, "click", document)
    }
    /// Host task checkpoint: coalesced details notifications share one budget.
    /// Tasks beyond the checkpoint limit remain queued for a later checkpoint.
    pub fn dispatch_details_toggles(&mut self, document: &mut Document) -> Result<()> {
        if document.details_toggle_running() {
            return Err(ScriptError::type_error(
                "details checkpoint is already running",
            ));
        }
        self.steps = MAX_STEPS;
        for _ in 0..64 {
            let Some(task) = document.peek_details_toggle() else {
                break;
            };
            // Preflight the event and path before consuming the queued task.
            // Once dispatch starts, ordinary listener errors are reported and
            // quota errors terminate the task without replaying its callbacks.
            let mut cursor = Some(task.event.node);
            let mut path_length = 2usize; // Document/window upper bound.
            while let Some(node) = cursor {
                self.tick()?;
                if path_length >= 258 {
                    return Err(ScriptError::resource("event path limit exceeded"));
                }
                path_length += 1;
                cursor = document.nodes.get(node).and_then(|node| node.parent);
            }
            // Reserve the bounded ordered-map operations for begin and finish.
            // Cleanup must run even if callbacks spend the remaining budget.
            self.work(128 + path_length)?;
            self.charge(path_length.saturating_mul(std::mem::size_of::<EventTarget>()))?;
            let event =
                self.event_object("toggle".into(), false, false, false, Value::Null, false)?;
            self.attach_toggle_state(
                &event,
                if task.event.old_open {
                    "open"
                } else {
                    "closed"
                }
                .into(),
                if task.event.new_open {
                    "open"
                } else {
                    "closed"
                }
                .into(),
                Value::Null,
            )?;
            let id = self.event_index(&event)?;
            self.events[id].trusted = true;
            let task = document
                .begin_details_toggle(task.id)
                .ok_or_else(|| ScriptError::new("details task changed during preflight"))?;
            let result = self.dispatch_event_object(
                EventTarget::Node(task.event.node),
                event,
                None,
                document,
            );
            document.finish_details_toggle(task.id);
            result?;
        }
        Ok(())
    }
    pub fn dispatch_event(
        &mut self,
        target: NodeId,
        event_type: &str,
        document: &mut Document,
    ) -> Result<()> {
        if event_type.len() > 128 {
            return Err(ScriptError::resource("event name limit exceeded"));
        }
        let target = self.event_target(&Value::Node(target), document)?;
        self.steps = MAX_STEPS;
        self.last_default_prevented = false;
        let cancelable = matches!(
            event_type,
            "click"
                | "submit"
                | "keydown"
                | "keyup"
                | "beforeinput"
                | "contextmenu"
                | "wheel"
                | "mousedown"
                | "mouseup"
        );
        let bubbles = !matches!(
            event_type,
            "focus" | "blur" | "load" | "unload" | "mouseenter" | "mouseleave"
        );
        let event = self.event_object(
            event_type.into(),
            bubbles,
            cancelable,
            false,
            Value::Null,
            false,
        )?;
        let id = self.event_index(&event)?;
        self.events[id].trusted = true;
        let result = self.dispatch_event_object(target, event, None, document);
        self.last_default_prevented = self.events[id].canceled;
        result.map(|_| ())
    }

    fn tick(&mut self) -> Result<()> {
        if self.steps == 0 {
            return Err(ScriptError::resource("script instruction limit exceeded"));
        }
        self.steps -= 1;
        Ok(())
    }
    fn work(&mut self, units: usize) -> Result<()> {
        if units > self.steps {
            self.steps = 0;
            return Err(ScriptError::resource("script instruction limit exceeded"));
        }
        self.steps -= units;
        Ok(())
    }
    fn charge(&mut self, bytes: usize) -> Result<()> {
        self.allocated = self
            .allocated
            .checked_add(bytes)
            .ok_or_else(|| ScriptError::resource("script allocation limit exceeded"))?;
        if self.allocated > MAX_HEAP {
            return Err(ScriptError::resource("script allocation limit exceeded"));
        }
        Ok(())
    }
    fn string(&mut self, text: impl Into<JsString>) -> Result<Value> {
        let text = text.into();
        if text.len() > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.charge(text.byte_len() + 24)?;
        Ok(Value::String(text))
    }
    fn array(&mut self, values: Vec<Value>) -> Result<Value> {
        self.charge(32 + values.len() * std::mem::size_of::<Value>())?;
        self.array_reserved(values)
    }
    fn array_reserved(&mut self, values: Vec<Value>) -> Result<Value> {
        let id = self.arrays.len();
        let Value::Object(properties) = self.object_ordered([])? else {
            unreachable!()
        };
        self.objects[properties].prototype = self
            .array_prototype
            .map(Value::Array)
            .or_else(|| self.prototypes.get("Array").copied().map(Value::Object));
        self.array_lengths.push(array_properties::ArrayLength {
            value: values.len() as u32,
            writable: true,
        });
        self.arrays.push(values);
        self.array_properties.push(properties);
        self.array_holes.push(BTreeSet::new());
        Ok(Value::Array(id))
    }
    fn rest_arguments(&mut self, arguments: &[Value]) -> Result<Value> {
        if arguments.len() > 65_536 {
            return Err(ScriptError::resource(
                "rest argument array length limit exceeded",
            ));
        }
        self.work(arguments.len().saturating_add(1))?;
        let copied_bytes = arguments.len().saturating_mul(std::mem::size_of::<Value>());
        // Check both the copy and array()'s retained storage/ordinary property
        // record before allocating. No getters/iterators run between this
        // preflight and array construction, and failure leaves its arenas intact.
        let required = copied_bytes
            .saturating_mul(2)
            .saturating_add(32 + 72 + std::mem::size_of::<Option<AbortSlot>>());
        if required > MAX_HEAP.saturating_sub(self.allocated) {
            return Err(ScriptError::resource("script allocation limit exceeded"));
        }
        self.charge(copied_bytes)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(arguments.len())
            .map_err(|_| ScriptError::resource("rest argument array allocation failed"))?;
        values.extend_from_slice(arguments);
        self.array(values)
    }
    fn arguments_object(&mut self, values: &[Value], strict: bool, callee: Value) -> Result<Value> {
        let object = self.object_ordered(
            values
                .iter()
                .enumerate()
                .map(|(i, value)| (i.to_string().into(), value.clone())),
        )?;
        let Value::Object(id) = object else {
            unreachable!()
        };
        self.objects[id].arguments = true;
        self.charge(320)?;
        self.objects[id].insert_hidden("length".into(), Value::Number(values.len() as f64));
        if strict {
            let thrower = self.alloc_native("ThrowTypeError", Value::Undefined)?;
            self.objects[id].insert_property(
                "callee".into(),
                Property {
                    value: PropertyValue::Accessor {
                        get: thrower.clone(),
                        set: thrower,
                    },
                    enumerable: false,
                    configurable: false,
                },
            );
        } else {
            self.objects[id].insert_hidden("callee".into(), callee);
        }
        self.install_arguments_iterator(&object)?;
        Ok(object)
    }
    fn object(&mut self, values: BTreeMap<String, Value>) -> Result<Value> {
        self.object_ordered(values.into_iter().map(|(key, value)| (key.into(), value)))
    }
    fn object_ordered(
        &mut self,
        values: impl IntoIterator<Item = (JsString, Value)>,
    ) -> Result<Value> {
        self.charge(
            72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>(),
        )?;
        let mut object = ScriptObject {
            prototype: self.prototypes.get("Object").copied().map(Value::Object),
            ..ScriptObject::default()
        };
        for (key, value) in values {
            self.charge(256 + key.byte_len().saturating_mul(2))?;
            object.insert(key, value);
        }
        let id = self.objects.len();
        self.objects.push(object);
        Ok(Value::Object(id))
    }
    fn environment(&mut self, parent: usize) -> Result<usize> {
        self.charge(128 + 2 * std::mem::size_of::<Option<Value>>())?;
        let id = self.environments.len();
        self.environments.push(Environment {
            bindings: BTreeMap::new(),
            this_binding: None,
            new_target_binding: None,
            parent: Some(parent),
            function_scope: false,
            strict: self.environments[parent].strict,
        });
        Ok(id)
    }
    fn define(&mut self, env: usize, name: &str, value: Value, mutable: bool) -> Result<()> {
        if !self.environments[env].bindings.contains_key(name) {
            self.charge(name.len() + BINDING_BYTES)?;
        } else if !self.environments[env].bindings[name].mutable {
            return Err(ScriptError::new(format!(
                "cannot redeclare constant '{name}'"
            )));
        }
        let global_order = if env == 0 {
            self.global_creation_order(name)?
        } else {
            0
        };
        self.environments[env].bindings.insert(
            name.into(),
            Binding {
                value,
                accessor: None,
                mutable,
                initialized: true,
                strict_immutable: !mutable,
                global_property: env == 0,
                global_order,
                enumerable: true,
                deletable: false,
            },
        );
        Ok(())
    }
    fn lookup(&self, mut env: usize, name: &str) -> Option<(usize, Value)> {
        loop {
            if let Some(binding) = self.environments[env].bindings.get(name) {
                return Some((env, binding.value.clone()));
            }
            env = self.environments[env].parent?;
        }
    }
    fn global_key(&self, kind: TrackedGlobal) -> JsString {
        self.tracked_global_keys[kind as usize].clone()
    }
    fn resolve_binding_in(
        &mut self,
        env: usize,
        name: &str,
        doc: &Document,
    ) -> Result<Option<usize>> {
        if name == "this" {
            let mut cursor = env;
            loop {
                self.tick()?;
                if self.environments[cursor].this_binding.is_some() {
                    return Ok(Some(cursor));
                }
                let Some(parent) = self.environments[cursor].parent else {
                    return Ok(None);
                };
                cursor = parent;
            }
        }
        if let Some((owner, _)) = self.lookup(env, name) {
            return Ok(Some(owner));
        }
        let key = self.global_name_key(name)?;
        if self.find_property_in(&Value::Window, &key, doc)?.is_some() {
            return Ok(Some(0));
        }
        Ok(None)
    }
    fn binding_value(&mut self, env: usize, name: &str, doc: &mut Document) -> Result<Value> {
        if name == "this" {
            return self.environments[env]
                .this_binding
                .clone()
                .ok_or_else(|| ScriptError::reference("missing execution receiver"));
        }
        if env == 0 {
            // Ordinary own data bindings need neither prototype traversal nor
            // a property-key conversion. Accessors/absence use the live object.
            if let Some(binding) = self.environments[0].bindings.get(name)
                && binding.accessor.is_none()
            {
                return Ok(binding.value.clone());
            }
            let key = self.global_name_key(name)?;
            return self
                .lookup_property(&Value::Window, &key, doc)?
                .ok_or_else(|| ScriptError::reference(format!("'{name}' is not defined")));
        }
        let binding = self.environments[env]
            .bindings
            .get(name)
            .ok_or_else(|| ScriptError::reference(format!("'{name}' is not defined")))?;
        if !binding.initialized {
            return Err(ScriptError::reference(format!(
                "cannot access '{name}' before initialization"
            )));
        }
        Ok(binding.value.clone())
    }
    fn validate_lexical(
        &self,
        unit: &Rc<code::Unit>,
        body: &[code::StmtId],
        env: usize,
    ) -> Result<()> {
        for statement in body {
            if let code::Stmt::Var(bindings, kind) = unit.stmt(*statement)
                && *kind != DeclarationKind::Var
            {
                for (name, _) in bindings {
                    if self.environments[env].bindings.contains_key(name)
                        || env == 1
                            && self.environments[0]
                                .bindings
                                .get(name)
                                .is_some_and(|b| !b.deletable)
                    {
                        return Err(ScriptError::syntax(format!(
                            "duplicate lexical binding '{name}'"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
    fn instantiate_lexical(
        &mut self,
        unit: &Rc<code::Unit>,
        body: &[code::StmtId],
        env: usize,
    ) -> Result<()> {
        self.validate_lexical(unit, body, env)?;
        for statement in body {
            if let code::Stmt::Var(bindings, kind) = unit.stmt(*statement)
                && *kind != DeclarationKind::Var
            {
                for (name, _) in bindings {
                    self.charge(name.len() + BINDING_BYTES)?;
                    self.environments[env].bindings.insert(
                        name.clone(),
                        Binding {
                            value: Value::Undefined,
                            accessor: None,
                            mutable: *kind != DeclarationKind::Const,
                            initialized: false,
                            strict_immutable: *kind == DeclarationKind::Const,
                            global_property: false,
                            global_order: 0,
                            enumerable: true,
                            deletable: false,
                        },
                    );
                }
            }
        }
        Ok(())
    }
    fn var_scope(&self, mut env: usize) -> usize {
        while !self.environments[env].function_scope {
            env = self.environments[env].parent.unwrap_or(0);
        }
        env
    }
    fn hoist_vars(
        &mut self,
        unit: &Rc<code::Unit>,
        body: &[code::StmtId],
        env: usize,
    ) -> Result<()> {
        self.hoist_vars_mode(unit, body, env, true)
    }
    fn hoist_vars_mode(
        &mut self,
        unit: &Rc<code::Unit>,
        body: &[code::StmtId],
        env: usize,
        insert: bool,
    ) -> Result<()> {
        let owner = self.var_scope(env);
        let mut walk = code::StatementWalk::new(unit, body.iter());
        while let Some(statement) = walk.next(|| self.tick())? {
            match unit.stmt(*statement) {
                code::Stmt::Var(bindings, DeclarationKind::Var) => {
                    for (name, _) in bindings {
                        self.hoist_name(name, owner, insert)?;
                    }
                }
                code::Stmt::ForIn(
                    code::ForBinding::Declaration(name, DeclarationKind::Var),
                    _,
                    _,
                )
                | code::Stmt::ForOf(
                    code::ForBinding::Declaration(name, DeclarationKind::Var),
                    _,
                    _,
                ) => {
                    self.hoist_name(name, owner, insert)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn hoist_name(&mut self, name: &str, owner: usize, insert: bool) -> Result<()> {
        if owner == 0 && self.environments[1].bindings.contains_key(name) {
            return Err(ScriptError::syntax(format!(
                "global lexical binding conflicts with var '{name}'"
            )));
        }
        if insert && !self.environments[owner].bindings.contains_key(name) {
            self.define(owner, name, Value::Undefined, true)?;
        }
        Ok(())
    }
    fn function_value(&mut self, code: &code::FunctionRef, environment: usize) -> Result<Value> {
        self.charge_function_code_copy(code)?;
        let id = self.functions.len();
        let Value::Object(properties) = self.object_ordered([
            ("length".into(), Value::Number(code.length() as f64)),
            (
                "name".into(),
                Value::String(code.name.clone().unwrap_or_default().into()),
            ),
        ])?
        else {
            unreachable!()
        };
        self.objects[properties].prototype = Some(Value::Function(self.function_prototype));
        self.objects[properties].attributes("name", false, false, true);
        self.objects[properties].attributes("length", false, false, true);
        self.functions.push(Function {
            code: code.clone(),
            environment,
            properties,
            bound: None,
        });
        if code.self_name
            && let Some(name) = &code.name
        {
            let scope = self.environment(environment)?;
            self.define(scope, name, Value::Function(id), false)?;
            self.environments[scope]
                .bindings
                .get_mut(name)
                .unwrap()
                .strict_immutable = false;
            self.functions[id].environment = scope;
        }
        if code.constructable {
            let prototype = self.object_ordered([("constructor".into(), Value::Function(id))])?;
            if let Value::Object(id) = prototype {
                self.objects[id].attributes("constructor", true, false, true);
            }
            self.objects[properties].insert_hidden("prototype".into(), prototype);
            self.objects[properties].attributes("prototype", true, false, false);
        }
        Ok(Value::Function(id))
    }
    fn charge_function_code_copy(&mut self, code: &code::FunctionRef) -> Result<()> {
        // Preserve the conservative legacy metadata allowance for each closure.
        // Flat code is shared; this scan and allowance do not clone its syntax.
        self.work(1 + code.params.len())?;
        let text_bytes = code.params.iter().fold(0usize, |size, parameter| {
            size.saturating_add(parameter.name.len())
        });
        let name_bytes = code.name.as_ref().map_or(0, String::len);
        self.work(1 + text_bytes.saturating_add(name_bytes) / 8)?;
        self.charge(
            128usize
                .saturating_add(
                    code.params
                        .len()
                        .saturating_mul(std::mem::size_of::<ParameterStorage>()),
                )
                .saturating_add(text_bytes)
                .saturating_add(name_bytes.saturating_mul(4)),
        )
    }
    fn set_function_name(
        &mut self,
        function: &Value,
        key: &JsString,
        prefix: Option<&str>,
    ) -> Result<()> {
        let name = if let Some(prefix) = prefix {
            let length = key.len().saturating_add(prefix.len() + 1);
            if length > MAX_STRING {
                return Err(ScriptError::resource("function name string limit exceeded"));
            }
            self.work(1 + length / 8)?;
            self.charge(length.saturating_mul(4))?;
            let mut units = Vec::with_capacity(length);
            units.extend(prefix.encode_utf16());
            units.push(u16::from(b' '));
            units.extend_from_slice(key.units());
            JsString::from(units)
        } else {
            key.clone()
        };
        self.define_own(
            function,
            &"name".into(),
            PropertyDescriptor {
                value: Some(Value::String(name)),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
                ..PropertyDescriptor::default()
            },
        )?;
        Ok(())
    }
    // Preserve existing legacy allocation allowances while explicitly paying
    // for the new optional direct-metadata slot before each reached Rc factory.
    fn alloc_native(&mut self, name: &str, receiver: Value) -> Result<Value> {
        self.charge(NATIVE_METADATA_BYTES)?;
        Ok(Self::native(name, receiver))
    }
    fn native(name: &str, receiver: Value) -> Value {
        Value::Native(Rc::new(Native {
            properties: None,
            name: name.into(),
            receiver,
        }))
    }

    fn instantiate_statements(
        &mut self,
        unit: &Rc<code::Unit>,
        body: &[code::StmtId],
        env: usize,
    ) -> Result<()> {
        if env == 1 {
            return self.instantiate_global_statements(unit, body);
        }
        self.instantiate_lexical(unit, body, env)?;
        self.hoist_vars(unit, body, env)?;
        for statement in body {
            if let code::Stmt::Function(name, code) = unit.stmt(*statement) {
                let function = self.function_value(&code::FunctionRef::new(unit, *code), env)?;
                self.define(env, name, function, true)?;
            }
        }
        Ok(())
    }
    fn exception_value(&mut self, error: ScriptError) -> Result<Value> {
        match error {
            ScriptError {
                kind: ErrorKind::Thrown(value),
                ..
            } => Ok(value),
            ScriptError {
                kind: ErrorKind::Runtime(name),
                message,
                ..
            } => {
                let message = self.string(message)?;
                self.error_object(name, message)
            }
            error => Err(error),
        }
    }
    fn error_object(&mut self, name: &str, message: Value) -> Result<Value> {
        let value = self.object_ordered([])?;
        let Value::Object(id) = value else {
            unreachable!()
        };
        self.objects[id].intrinsic_error = intrinsic_error_type(name);
        if message != Value::Undefined {
            self.charge(160)?;
            self.objects[id].insert_hidden("message".into(), message);
        }
        self.objects[id].prototype = Some(Value::Object(
            self.prototypes
                .get(name)
                .copied()
                .unwrap_or(self.prototypes["Error"]),
        ));
        Ok(value)
    }
    fn thrown_error(&mut self, value: Value) -> Result<ScriptError> {
        // Value's diagnostic formatter never reads author properties. A UTF-16
        // code unit needs at most three UTF-8 bytes, including lone-surrogate
        // replacement; the constant also covers the prefix and numeric output.
        let units = match &value {
            Value::String(text) => text.len(),
            Value::Symbol(symbol) => symbol.description().map_or(0, JsString::len),
            _ => 0,
        };
        self.work(1 + units / 8)?;
        self.charge(units.saturating_mul(3).saturating_add(512))?;
        Ok(ScriptError::thrown(value))
    }
    fn thrown_name(&mut self, value: &Value, doc: &mut Document) -> Result<Option<String>> {
        if !js_object(value) {
            return Ok(None);
        }
        let constructor = self.get(value.clone(), "constructor", doc)?;
        if json_callable(&constructor) {
            let name = self.get(constructor, "name", doc)?;
            if let Value::String(name) = name {
                self.work(1 + name.len() / 16)?;
                self.charge(name.len().saturating_mul(3).saturating_add(24))?;
                return Ok(Some(name.to_utf8_lossy()));
            }
        }
        Ok(None)
    }
    fn thrown_intrinsic_name(
        &mut self,
        value: &Value,
        doc: &mut Document,
    ) -> Result<Option<&'static str>> {
        let Value::Object(id) = value else {
            return Ok(None);
        };
        let Some(name) = self.objects[*id].intrinsic_error else {
            return Ok(None);
        };
        let constructor = self.get(value.clone(), "constructor", doc)?;
        Ok((constructor == self.alloc_native(name, Value::Window)?).then_some(name))
    }
    #[cfg(test)]
    fn eval(
        &mut self,
        unit: &Rc<code::Unit>,
        expression: &code::ExprId,
        env: usize,
        doc: &mut Document,
    ) -> Result<Value> {
        machine::evaluate(self, unit, *expression, env, doc)
    }
    fn number_hint_primitive(&mut self, value: Value, doc: &mut Document) -> Result<Value> {
        self.primitive_with_hint(value, "default", doc)
    }
    fn primitive_with_hint(
        &mut self,
        value: Value,
        hint: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        if !js_object(&value) {
            return Ok(value);
        }
        if let Some(primitive) = self.exotic_primitive(&value, hint, doc)? {
            return Ok(primitive);
        }
        // Ordinary objects use the number hint for default-hint conversion.
        // Read the fallback only after the first callable has returned.
        for key in ["valueOf", "toString"] {
            self.charge(64)?;
            let method = self.get(value.clone(), key, doc)?;
            if json_callable(&method) {
                let primitive = self.call(method, Vec::new(), value.clone(), doc)?;
                if !js_object(&primitive) {
                    return Ok(primitive);
                }
            }
        }
        Err(ScriptError::type_error(
            "object cannot be converted to a primitive",
        ))
    }
    fn addition_text(&mut self, primitive: Value) -> Result<JsString> {
        match primitive {
            Value::String(text) => Ok(text),
            Value::Number(_) => {
                // json_number can format a full decimal intermediate before
                // selecting ECMAScript notation; cover its bounded scratch.
                self.work(128)?;
                self.charge(1024)?;
                Ok(primitive.js_string())
            }
            Value::Undefined | Value::Null | Value::Bool(_) => {
                self.work(4)?;
                self.charge(128)?;
                Ok(primitive.js_string())
            }
            Value::Symbol(_) => Err(ScriptError::type_error(
                "cannot convert a symbol to a string",
            )),
            _ => unreachable!("addition converts both operands before formatting"),
        }
    }
    fn addition_value(&mut self, left: Value, right: Value, doc: &mut Document) -> Result<Value> {
        let left = self.number_hint_primitive(left, doc)?;
        let right = self.number_hint_primitive(right, doc)?;
        if !matches!(left, Value::String(_)) && !matches!(right, Value::String(_)) {
            let left = self.primitive_number_value(left)?;
            let right = self.primitive_number_value(right)?;
            return Ok(Value::Number(left + right));
        }
        let left = self.addition_text(left)?;
        let right = self.addition_text(right)?;
        let length = left.len().saturating_add(right.len());
        if length > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.work(1 + length.saturating_mul(2) / 8)?;
        // The exact-sized Vec and its conversion to an Rc slice can coexist.
        // Charge both before allocation, including the retained result header.
        self.charge(64 + length.saturating_mul(4))?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("addition string allocation failed"))?;
        units.extend_from_slice(left.units());
        units.extend_from_slice(right.units());
        Ok(Value::String(JsString::from(units)))
    }
    fn relational_value(
        &mut self,
        op: &str,
        left: Value,
        right: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        // All four operators convert in source order. Reversing the comparison
        // for > and <= must not reverse getters or calls in ToPrimitive.
        let left = self.primitive_with_hint(left, "number", doc)?;
        let right = self.primitive_with_hint(right, "number", doc)?;
        for value in [&left, &right] {
            if let Value::String(text) = value {
                self.work(1 + text.len() / 8)?;
            }
        }
        let (x, y) = if matches!(op, ">" | "<=") {
            (right, left)
        } else {
            (left, right)
        };
        let less = if let (Value::String(a), Value::String(b)) = (&x, &y) {
            Some(a < b)
        } else {
            let a = self.primitive_number_value(x)?;
            let b = self.primitive_number_value(y)?;
            a.partial_cmp(&b).map(|ordering| ordering == Ordering::Less)
        };
        // An unordered numeric result is false for every relational operator.
        Ok(Value::Bool(if matches!(op, "<=" | ">=") {
            less == Some(false)
        } else {
            less == Some(true)
        }))
    }
    fn loosely_equal(
        &mut self,
        mut left: Value,
        mut right: Value,
        doc: &mut Document,
    ) -> Result<bool> {
        // Each continuation removes an object or Boolean conversion. Keep the
        // abstract algorithm iterative; author callbacks use the shared budget.
        loop {
            match (&left, &right) {
                (Value::String(a), Value::String(b)) => {
                    self.work(1 + a.len() / 8)?;
                    self.work(1 + b.len() / 8)?;
                    return Ok(a == b);
                }
                (Value::Undefined, Value::Undefined)
                | (Value::Null, Value::Null)
                | (Value::Bool(_), Value::Bool(_))
                | (Value::Symbol(_), Value::Symbol(_))
                | (Value::Number(_), Value::Number(_)) => return Ok(left == right),
                (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => {
                    return Ok(true);
                }
                (Value::Number(number), Value::String(_)) => {
                    return Ok(*number == self.primitive_number_value(right)?);
                }
                (Value::String(_), Value::Number(number)) => {
                    return Ok(*number == self.primitive_number_value(left)?);
                }
                (Value::Bool(value), _) => {
                    self.tick()?;
                    left = Value::Number(f64::from(*value));
                }
                (_, Value::Bool(value)) => {
                    self.tick()?;
                    right = Value::Number(f64::from(*value));
                }
                (Value::String(_) | Value::Number(_) | Value::Symbol(_), object)
                    if js_object(object) =>
                {
                    right = self.number_hint_primitive(right, doc)?;
                }
                (object, Value::String(_) | Value::Number(_) | Value::Symbol(_))
                    if js_object(object) =>
                {
                    left = self.number_hint_primitive(left, doc)?;
                }
                (a, b) if js_object(a) && js_object(b) => return Ok(left == right),
                _ => return Ok(false),
            }
        }
    }
    fn binary_value(
        &mut self,
        op: &str,
        left: Value,
        right: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        if op == "+" {
            return self.addition_value(left, right, doc);
        }
        if matches!(op, "<" | ">" | "<=" | ">=") {
            return self.relational_value(op, left, right, doc);
        }
        if matches!(op, "==" | "!=") {
            let equal = self.loosely_equal(right, left, doc)?;
            return Ok(Value::Bool(if op == "!=" { !equal } else { equal }));
        }
        for value in [&left, &right] {
            if let Value::String(text) = value {
                self.work(1 + text.len() / 8)?;
            }
        }
        if op == "instanceof" {
            return self.instance_of(left, right, false, doc).map(Value::Bool);
        }
        if op == "in" {
            if !js_object(&right) {
                return Err(ScriptError::type_error(
                    "in right-hand side is not an object",
                ));
            }
            let key = self.property_key(left, doc)?;
            if let Value::Style(id) = right
                && let PropertyKey::String(key) = &key
            {
                return self.style_has(id, key, doc).map(Value::Bool);
            }
            return Ok(Value::Bool(
                self.find_property_key_in(&right, &key, doc)?.is_some(),
            ));
        }
        if matches!(op, "===" | "!==") {
            let equal = left == right;
            return Ok(Value::Bool(if op == "!==" { !equal } else { equal }));
        }
        let a = self.number_value(left, doc)?;
        let b = self.number_value(right, doc)?;
        Ok(match op {
            "-" => Value::Number(a - b),
            "*" => Value::Number(a * b),
            "/" => Value::Number(a / b),
            "%" => Value::Number(a % b),
            "**" => Value::Number(a.powf(b)),
            "&" => Value::Number((to_i32(a) & to_i32(b)) as f64),
            "|" => Value::Number((to_i32(a) | to_i32(b)) as f64),
            "^" => Value::Number((to_i32(a) ^ to_i32(b)) as f64),
            "<<" => Value::Number(to_i32(a).wrapping_shl(to_i32(b) as u32 & 31) as f64),
            ">>" => Value::Number(to_i32(a).wrapping_shr(to_i32(b) as u32 & 31) as f64),
            ">>>" => Value::Number((to_i32(a) as u32).wrapping_shr(to_i32(b) as u32 & 31) as f64),
            _ => return Err(ScriptError::new(format!("unsupported operator '{op}'"))),
        })
    }
    #[cfg(test)]
    fn reference(
        &mut self,
        unit: &Rc<code::Unit>,
        expression: &code::ExprId,
        env: usize,
        doc: &mut Document,
    ) -> Result<Reference> {
        machine::evaluate_reference(self, unit, *expression, env, doc)
    }
    fn reference_key(
        &mut self,
        base: &Value,
        value: Value,
        doc: &mut Document,
    ) -> Result<PropertyKey> {
        // ToObject's only observable primitive effect here is rejecting null
        // and undefined. Keep the original primitive receiver for accessors.
        if matches!(base, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot access property of null or undefined",
            ));
        }
        self.property_key(value, doc)
    }
    fn read_reference(&mut self, reference: &mut Reference, doc: &mut Document) -> Result<Value> {
        match reference {
            Reference::CodeName {
                unit,
                expression,
                owner,
                ..
            } => {
                let code::Expr::Ident(name) = unit.expr(*expression) else {
                    unreachable!("identifier reference")
                };
                match owner {
                    Some(env) => self.binding_value(*env, name, doc),
                    None => Err(ScriptError::reference(format!("'{name}' is not defined"))),
                }
            }
            Reference::Property(object, name, _) => {
                let key = self.reference_key(object, name.clone(), doc)?;
                *name = key.value();
                self.get_property_key(object.clone(), &key, doc)
            }
        }
    }
    fn write_reference(
        &mut self,
        reference: Reference,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        match reference {
            Reference::CodeName {
                unit,
                expression,
                owner,
                strict,
            } => {
                let code::Expr::Ident(name) = unit.expr(expression) else {
                    unreachable!("identifier reference")
                };
                self.write_name(owner, name, strict, value, doc)
            }
            Reference::Property(object, key, strict) => {
                let key = self.reference_key(&object, key, doc)?;
                self.set_property_key(object, &key, value, strict, doc)
            }
        }
    }
    fn write_name(
        &mut self,
        owner: Option<usize>,
        name: &str,
        strict: bool,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        if let Some(env) = owner {
            if env == 0 {
                if let Some(binding) = self.environments[0].bindings.get_mut(name)
                    && binding.accessor.is_none()
                {
                    if !binding.mutable {
                        return Self::failed_write(strict);
                    }
                    binding.value = value;
                    return Ok(());
                }
                let key = self.global_name_key(name)?;
                if strict && self.find_property_in(&Value::Window, &key, doc)?.is_none() {
                    return Err(ScriptError::reference(format!("'{name}' is not defined")));
                }
                return self.set_key_strict(Value::Window, &key, value, strict, doc);
            }
            // Declarative bindings preserve TDZ and immutable-binding rules.
            if let Some(binding) = self.environments[env].bindings.get_mut(name) {
                if !binding.initialized {
                    return Err(ScriptError::reference(format!(
                        "cannot assign to '{name}' before initialization"
                    )));
                }
                if !binding.mutable {
                    return if strict || binding.strict_immutable {
                        Err(ScriptError::type_error(format!(
                            "cannot assign to constant '{name}'"
                        )))
                    } else {
                        Ok(())
                    };
                }
                binding.value = value;
                return Ok(());
            }
            return Err(ScriptError::reference(format!("'{name}' is not defined")));
        }
        if strict {
            return Err(ScriptError::reference(format!("'{name}' is not defined")));
        }
        let key = self.global_name_key(name)?;
        self.set_key_strict(Value::Window, &key, value, false, doc)
    }

    fn call(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.call_with_new_target(function, arguments, receiver, Value::Undefined, doc)
    }
    fn call_with_new_target(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
        receiver: Value,
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if self.calls >= MAX_CALLS {
            return Err(ScriptError::resource("script call stack limit exceeded"));
        }
        self.enter_stack(4)?;
        self.calls += 1;
        let result =
            machine::invoke_preentered(self, function, arguments, receiver, new_target, doc);
        self.calls -= 1;
        self.stack_units -= 4;
        result
    }
    fn property_object(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Object(id) => Some(*id),
            Value::Function(id) => Some(self.functions[*id].properties),
            Value::Array(id) => Some(self.array_properties[*id]),
            Value::Native(native) => native
                .properties
                .map(std::num::NonZeroUsize::get)
                .or_else(|| self.native_properties.get(&native.name).copied()),
            Value::Json => self.native_properties.get("JSON").copied(),
            Value::Math => self.native_properties.get("Math").copied(),
            _ => None,
        }
    }
    fn prototype_of(&self, value: &Value) -> Option<Value> {
        if let Some(id) = self.property_object(value) {
            return self.objects[id].prototype.clone();
        }
        if matches!(value, Value::Native(_)) {
            return Some(Value::Function(self.function_prototype));
        }
        let name = match value {
            Value::Node(_) | Value::Document | Value::Window => "EventTarget",
            Value::String(_) => "String",
            Value::Number(_) => "Number",
            Value::Bool(_) => "Boolean",
            Value::Symbol(_) => "Symbol",
            Value::Null | Value::Undefined => return None,
            _ => "Object",
        };
        self.prototypes.get(name).copied().map(Value::Object)
    }
    fn object_is_prototype_of_in(
        &mut self,
        receiver: Value,
        mut value: Value,
        doc: &Document,
    ) -> Result<Value> {
        self.tick()?;
        // Unlike most Object methods, a primitive argument returns before ToObject(this).
        if !js_object(&value) {
            return Ok(Value::Bool(false));
        }
        let object = self.coerce_object(receiver)?;
        for _ in 0..MAX_DEPTH {
            self.tick()?;
            let Some(prototype) = self.prototype_of_in(&value, doc)? else {
                return Ok(Value::Bool(false));
            };
            if prototype == object {
                return Ok(Value::Bool(true));
            }
            value = prototype;
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }
    fn set_object_prototype_in(
        &mut self,
        object: &Value,
        prototype: Value,
        doc: &Document,
    ) -> Result<()> {
        let id = self.property_object(object).ok_or_else(|| {
            ScriptError::unsupported("host object prototype mutation is unsupported")
        })?;
        let next = if prototype == Value::Null {
            None
        } else {
            Some(prototype)
        };
        if self.objects[id].non_extensible && self.objects[id].prototype != next {
            return Err(ScriptError::type_error(
                "non-extensible object prototype cannot change",
            ));
        }
        let mut cursor = next.clone();
        for depth in 0..=MAX_DEPTH {
            self.tick()?;
            let Some(value) = cursor else {
                break;
            };
            if &value == object {
                return Err(ScriptError::type_error("cyclic object prototype"));
            }
            if depth == MAX_DEPTH {
                return Err(ScriptError::resource("prototype chain limit exceeded"));
            }
            cursor = self.prototype_of_in(&value, doc)?;
        }
        self.objects[id].prototype = next;
        Ok(())
    }
    fn own_property(&self, receiver: &Value, key: &JsString) -> Option<Property> {
        if matches!(receiver, Value::Window) {
            if let Some(kind) = TrackedGlobal::from_key(key) {
                return self.environments[0]
                    .bindings
                    .get(kind.name())
                    .map(Binding::property);
            }
            return match key.to_utf8() {
                Ok(key) => self.environments[0]
                    .bindings
                    .get(&key)
                    .filter(|b| b.global_property)
                    .map(Binding::property),
                Err(_) => self.global_non_scalar.get(key).map(|p| p.property.clone()),
            };
        }
        if let Some(id) = self.property_object(receiver)
            && let Some(property) = self.objects[id].values.get(&key.into())
        {
            let mut property = property.clone();
            if let Some((env, name)) = self.objects[id].parameter_map.get(key)
                && let PropertyValue::Data { value, .. } = &mut property.value
            {
                *value = self.environments[*env].bindings[name].value.clone();
            }
            return Some(property);
        }
        if let Value::Array(id) = receiver {
            if key.units() == [108, 101, 110, 103, 116, 104] {
                return Some(Property::data(
                    Value::Number(self.array_lengths[*id].value as f64),
                    self.array_lengths[*id].writable,
                    false,
                    false,
                ));
            }
            if let Some(index) = json_array_index(key).map(|index| index as usize)
                && !self.array_holes[*id].contains(&index)
                && let Some(value) = self.arrays[*id].get(index)
            {
                return Some(Property::data(value.clone(), true, true, true));
            }
        }
        let text = match receiver {
            Value::String(text) => Some(text),
            Value::Object(id) => match &self.objects[*id].boxed {
                Some(Value::String(text)) => Some(text),
                _ => None,
            },
            _ => None,
        };
        if let Some(text) = text {
            if key.units() == [108, 101, 110, 103, 116, 104] {
                return Some(Property::data(
                    Value::Number(text.len() as f64),
                    false,
                    false,
                    false,
                ));
            }
            if let Some(index) = json_array_index(key).map(|index| index as usize)
                && let Some(unit) = text.units().get(index)
            {
                return Some(Property::data(
                    Value::String(vec![*unit].into()),
                    false,
                    true,
                    false,
                ));
            }
        }
        None
    }
    #[cfg(test)]
    fn resolve_binding(&mut self, env: usize, name: &str) -> Result<Option<usize>> {
        self.resolve_binding_in(env, name, &Document::parse(""))
    }
    #[cfg(test)]
    fn reduce_property(&mut self, receiver: &Value, key: &JsString) -> Result<Option<Property>> {
        self.reduce_property_in(receiver, key, &Document::parse(""))
    }
    #[cfg(test)]
    fn object_is_prototype_of(&mut self, receiver: Value, value: Value) -> Result<Value> {
        self.object_is_prototype_of_in(receiver, value, &Document::parse(""))
    }
    #[cfg(test)]
    fn set_object_prototype(&mut self, object: &Value, prototype: Value) -> Result<()> {
        self.set_object_prototype_in(object, prototype, &Document::parse(""))
    }

    fn find_property_in(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &Document,
    ) -> Result<Option<Property>> {
        let mut cursor = Some(receiver.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else {
                return Ok(None);
            };
            self.work(1 + key.len() / 16)?;
            if value == Value::Window {
                self.window_lookup_budget(key)?;
            }
            if let Some(property) = self.read_own_property(&value, key)? {
                return Ok(Some(property));
            }
            cursor = self.prototype_of_in(&value, doc)?;
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }
    fn lookup_property(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        let Some(property) = self.find_property_in(receiver, key, doc)? else {
            return Ok(None);
        };
        Ok(Some(match property.value {
            PropertyValue::Data { value, .. } => value,
            PropertyValue::Accessor {
                get: Value::Undefined,
                ..
            } => Value::Undefined,
            PropertyValue::Accessor { get, .. } => {
                self.call(get, Vec::new(), receiver.clone(), doc)?
            }
        }))
    }
    fn property_descriptor(
        &mut self,
        object: Value,
        doc: &mut Document,
    ) -> Result<PropertyDescriptor> {
        if !js_object(&object) {
            return Err(ScriptError::type_error(
                "property descriptor must be an object",
            ));
        }
        let mut desc = PropertyDescriptor::default();
        for key in [
            "enumerable",
            "configurable",
            "value",
            "writable",
            "get",
            "set",
        ] {
            if self.find_property_in(&object, &key.into(), doc)?.is_none() {
                continue;
            }
            let value = self.get(object.clone(), key, doc)?;
            match key {
                "enumerable" => desc.enumerable = Some(value.truthy()),
                "configurable" => desc.configurable = Some(value.truthy()),
                "value" => desc.value = Some(value),
                "writable" => desc.writable = Some(value.truthy()),
                "get" | "set" => {
                    if value != Value::Undefined && !json_callable(&value) {
                        return Err(ScriptError::type_error(
                            "accessor must be callable or undefined",
                        ));
                    }
                    if key == "get" {
                        desc.get = Some(value);
                    } else {
                        desc.set = Some(value);
                    }
                }
                _ => unreachable!(),
            }
        }
        if desc.accessor() && desc.data() {
            return Err(ScriptError::type_error(
                "descriptor cannot contain data and accessors",
            ));
        }
        Ok(desc)
    }
    fn define_own(
        &mut self,
        receiver: &Value,
        key: &JsString,
        desc: PropertyDescriptor,
    ) -> Result<bool> {
        self.define_own_key(receiver, &key.into(), desc)
    }
    fn define_own_key(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
        desc: PropertyDescriptor,
    ) -> Result<bool> {
        if let Value::Array(id) = receiver
            && let Some(key) = key.as_string()
        {
            if key == &JsString::from("length") {
                return Err(ScriptError::unsupported(
                    "array length definition requires observable conversion",
                ));
            }
            if let Some(index) = json_array_index(key) {
                if index >= self.array_lengths[*id].value && !self.array_lengths[*id].writable {
                    return Ok(false);
                }
                if !self.ordinary_define_own_key(receiver, &key.into(), desc)? {
                    return Ok(false);
                }
                self.array_lengths[*id].value = self.array_lengths[*id].value.max(index + 1);
                return Ok(true);
            }
        }
        self.ordinary_define_own_key(receiver, key, desc)
    }
    fn ordinary_define_own_key(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
        desc: PropertyDescriptor,
    ) -> Result<bool> {
        if dom_own_properties::host(receiver).is_some() {
            return self.dom_define_own(receiver, key, desc);
        }
        self.work(1 + key.byte_len() / 2 / 8)?;
        let window_key = if receiver == &Value::Window {
            key.as_string()
        } else {
            None
        };
        let object_id = if matches!(key, PropertyKey::Symbol(_)) {
            self.ensure_symbol_property_object(receiver)?
        } else {
            self.property_object(receiver)
        };
        if object_id.is_none() && window_key.is_none() {
            return Err(ScriptError::unsupported(
                "host property definition is not implemented",
            ));
        }
        if let Some(key) = window_key {
            self.window_lookup_budget(key)?;
        }
        let current = self.own_property_key(receiver, key);
        if let Some(current) = &current {
            if !current.configurable {
                if desc.configurable == Some(true)
                    || desc
                        .enumerable
                        .is_some_and(|value| value != current.enumerable)
                {
                    return Ok(false);
                }
                match &current.value {
                    PropertyValue::Data { value, writable } => {
                        if desc.accessor()
                            || !*writable
                                && (desc.writable == Some(true)
                                    || desc
                                        .value
                                        .as_ref()
                                        .is_some_and(|next| !json_same_value(value, next)))
                        {
                            return Ok(false);
                        }
                    }
                    PropertyValue::Accessor { get, set } => {
                        if desc.data()
                            || desc
                                .get
                                .as_ref()
                                .is_some_and(|value| !json_same_value(value, get))
                            || desc
                                .set
                                .as_ref()
                                .is_some_and(|value| !json_same_value(value, set))
                        {
                            return Ok(false);
                        }
                    }
                }
            }
        } else if object_id.is_some_and(|id| self.objects[id].non_extensible) {
            return Ok(false);
        }
        let mapped_value = desc.value.clone();
        let sever_mapping = desc.accessor() || desc.writable == Some(false);
        let mut property =
            current.unwrap_or_else(|| Property::data(Value::Undefined, false, false, false));
        if desc.accessor() && matches!(property.value, PropertyValue::Data { .. }) {
            property.value = PropertyValue::Accessor {
                get: Value::Undefined,
                set: Value::Undefined,
            };
        } else if desc.data() && matches!(property.value, PropertyValue::Accessor { .. }) {
            property.value = PropertyValue::Data {
                value: Value::Undefined,
                writable: false,
            };
        }
        if let Some(value) = desc.enumerable {
            property.enumerable = value;
        }
        if let Some(value) = desc.configurable {
            property.configurable = value;
        }
        match &mut property.value {
            PropertyValue::Data { value, writable } => {
                if let Some(next) = desc.value {
                    *value = next;
                }
                if let Some(next) = desc.writable {
                    *writable = next;
                }
            }
            PropertyValue::Accessor { get, set } => {
                if let Some(next) = desc.get {
                    *get = next;
                }
                if let Some(next) = desc.set {
                    *set = next;
                }
            }
        }
        if let Some(key) = window_key {
            self.store_window_property(key, property)?;
            return Ok(true);
        }
        let id = object_id.unwrap();
        if let Value::Array(array_id) = receiver
            && let Some(string) = key.as_string()
            && self.store_array_property(*array_id, string, &property)?
        {
            return Ok(true);
        }
        if !self.objects[id].contains_key(key) {
            self.charge(256 + key.byte_len().saturating_mul(2))?;
        }
        self.objects[id].insert_property(key.clone(), property);
        // Updating the property table does not change parameter_map. Borrow
        // its binding name after the update instead of allocating a copy.
        if let Some(key) = key.as_string()
            && let Some((env, name)) = self.objects[id].parameter_map.get(key)
        {
            if let Some(value) = mapped_value {
                self.environments[*env]
                    .bindings
                    .get_mut(name)
                    .unwrap()
                    .value = value;
            }
            if sever_mapping {
                self.objects[id].parameter_map.remove(key);
            }
        }
        Ok(true)
    }
    fn store_global_property(&mut self, name: &str, property: Property) -> Result<()> {
        let new = !self.environments[0].bindings.contains_key(name);
        let accessor_bytes = if matches!(property.value, PropertyValue::Accessor { .. }) {
            16 + std::mem::size_of::<BindingAccessor>()
        } else {
            0
        };
        self.charge(accessor_bytes + if new { BINDING_BYTES + name.len() } else { 0 })?;
        let global_order = self.global_creation_order(name)?;
        let (value, mutable, accessor) = match property.value {
            PropertyValue::Data { value, writable } => (value, writable, None),
            PropertyValue::Accessor { get, set } => (
                Value::Undefined,
                true,
                Some(Rc::new(BindingAccessor { get, set })),
            ),
        };
        let binding = Binding {
            value,
            accessor,
            mutable,
            initialized: true,
            strict_immutable: false,
            global_property: true,
            global_order,
            enumerable: property.enumerable,
            deletable: property.configurable,
        };
        if let Some(previous) = self.environments[0].bindings.get_mut(name) {
            *previous = binding;
        } else {
            self.environments[0].bindings.insert(name.into(), binding);
        }
        Ok(())
    }
    fn define_properties(
        &mut self,
        object: Value,
        properties: Value,
        doc: &mut Document,
    ) -> Result<()> {
        let properties = self.coerce_object(properties)?;
        let keys = self.own_property_keys(&properties)?;
        let mut descriptors = self.own_key_descriptors(keys.len())?;
        for key in keys {
            if !self
                .read_own_property_key(&properties, &key)?
                .is_some_and(|p| p.enumerable)
            {
                continue;
            }
            let value = self.get_property_key(properties.clone(), &key, doc)?;
            let desc = self.property_descriptor(value, doc)?;
            self.charge(256 + key.byte_len())?;
            descriptors.push((key, desc));
        }
        for (key, desc) in descriptors {
            if !self.define_property_key(&object, &key, desc, doc)? {
                return Err(ScriptError::type_error("incompatible property definition"));
            }
        }
        Ok(())
    }
    fn delete_property(&mut self, receiver: Value, key: &JsString) -> Result<bool> {
        if dom_own_properties::host(&receiver).is_some() {
            return self.dom_delete_own(&receiver, &PropertyKey::String(key.clone()));
        }
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot delete property of null or undefined",
            ));
        }
        self.work(1 + key.len() / 8)?;
        if receiver == Value::Window {
            self.window_lookup_budget(key)?;
            if key.to_utf8().is_ok_and(|name| event_handler_name(&name)) {
                return Err(ScriptError::unsupported(
                    "Window event-handler property deletion is not implemented",
                ));
            }
        }
        let Some(property) = self.own_property(&receiver, key) else {
            if receiver == Value::Window {
                return Ok(true);
            }
            if self.property_object(&receiver).is_none() && js_object(&receiver) {
                return Err(ScriptError::unsupported(
                    "host property deletion is not implemented",
                ));
            }
            return Ok(true);
        };
        if !property.configurable {
            return Ok(false);
        }
        if matches!(receiver, Value::Window) {
            if let Some(kind) = TrackedGlobal::from_key(key) {
                self.environments[0].bindings.remove(kind.name());
            } else if let Ok(key) = key.to_utf8() {
                self.environments[0].bindings.remove(&key);
            } else {
                self.global_non_scalar.remove(key);
            }
            return Ok(true);
        }
        if let Value::Array(id) = receiver
            && let Some(index) = json_array_index(key)
            && (index as usize) < self.arrays[id].len()
        {
            self.charge(32)?;
            self.array_holes[id].insert(index as usize);
            self.arrays[id][index as usize] = Value::Undefined;
        }
        if let Some(id) = self.property_object(&receiver) {
            self.work(1 + self.objects[id].order.len() / 8)?;
            self.objects[id].remove(&key.into());
            self.objects[id].parameter_map.remove(key);
        }
        Ok(true)
    }
    fn coerce_object(&mut self, value: Value) -> Result<Value> {
        if js_object(&value) {
            return Ok(value);
        }
        if matches!(value, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot convert null or undefined to object",
            ));
        }
        let prototype = self.prototype_of(&value);
        let result = self.object_ordered([])?;
        let Value::Object(id) = result else {
            unreachable!()
        };
        self.objects[id].prototype = prototype;
        self.objects[id].boxed = Some(value);
        Ok(result)
    }
    fn reduce_property_in(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &Document,
    ) -> Result<Option<Property>> {
        // Bounded short-key comparisons still depend on the stored tree's
        // size. Empty trees need no comparison allowance. Ordinary key work
        // uses eight-unit chunks, as the existing per-edge charge does.
        fn lookup_work(entries: usize, units: usize) -> usize {
            let levels = entries.checked_ilog2().map_or(0, |n| n as usize + 1);
            (1 + units / 8).saturating_mul(4 * levels)
        }
        let mut cursor = Some(receiver.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else {
                return Ok(None);
            };
            self.work(1 + key.len() / 8)?;
            let native_name = match &value {
                Value::Native(native) if native.properties.is_none() => Some(native.name.as_str()),
                Value::Json => Some("JSON"),
                Value::Math => Some("Math"),
                _ => None,
            };
            if let Some(name) = native_name {
                // Resolution here, in own_property and (if absent) in
                // prototype_of can each consult this intrinsic-name tree.
                self.work(lookup_work(self.native_properties.len(), name.len()).saturating_mul(3))?;
            }
            let Some(object) = self.property_object(&value) else {
                return Err(ScriptError::unsupported(
                    "host prototype lookup during reduction is not implemented",
                ));
            };
            let properties = &self.objects[object];
            let table_work = lookup_work(properties.values.len(), key.len())
                .saturating_add(
                    lookup_work(properties.parameter_map.len(), key.len()).saturating_mul(2),
                )
                .saturating_add(match &value {
                    Value::Array(id) => lookup_work(self.array_holes[*id].len(), 1),
                    _ => 0,
                });
            // Covers both this mapping lookup and the later own-property
            // lookup. No property value or mapped binding has been read yet.
            self.work(table_work)?;
            if let Some((env, name)) = self.objects[object].parameter_map.get(key) {
                let levels = 1 + self.environments[*env]
                    .bindings
                    .len()
                    .checked_ilog2()
                    .unwrap_or(0) as usize;
                // The binding's retained name can be much longer than "0".
                // Borrow it and charge before own_property snapshots its value.
                self.work((1 + name.len()).saturating_mul(4 * levels))?;
            }
            // Keys are either "length" or at most sixteen ASCII digits.
            // Retain the conservative per-edge storage estimate. Array index
            // decoding itself is allocation-free; a boxed-string edge can
            // still create a one-unit value during either presence or Get.
            self.charge(128)?;
            if let Some(property) = self.own_property(&value, key) {
                return Ok(Some(property));
            }
            cursor = self.prototype_of_in(&value, doc)?;
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }
    fn reduce_get(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &mut Document,
    ) -> Result<Value> {
        let Some(property) = self.reduce_property_in(receiver, key, doc)? else {
            return Ok(Value::Undefined);
        };
        match property.value {
            PropertyValue::Data { value, .. } => Ok(value),
            PropertyValue::Accessor {
                get: Value::Undefined,
                ..
            } => Ok(Value::Undefined),
            PropertyValue::Accessor { get, .. } => {
                self.call(get, Vec::new(), receiver.clone(), doc)
            }
        }
    }
    fn reduce_index_key(&mut self, mut index: u64) -> Result<JsString> {
        if index > 9_007_199_254_740_991 {
            return Err(ScriptError::resource(
                "reduction index exceeds safe integer range",
            ));
        }
        // Precharge the actual decimal conversion, not sixteen digits for
        // every small index. The range check above bounds this to 1..=16.
        let count = index.checked_ilog10().unwrap_or(0) as usize + 1;
        self.work(1 + count)?;
        self.charge(64)?;
        let mut digits = [0u16; 16];
        let start = digits.len() - count;
        for digit in digits[start..].iter_mut().rev() {
            *digit = u16::from(b'0') + (index % 10) as u16;
            index /= 10;
        }
        Ok(JsString::from(&digits[start..]))
    }
    fn array_reduce(
        &mut self,
        receiver: Value,
        callback: Value,
        mut accumulator: Option<Value>,
        direction: ReduceDirection,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let object = self.coerce_object(receiver)?;
        if self.property_object(&object).is_none() {
            return Err(ScriptError::unsupported(
                "host array-like reduction is not implemented",
            ));
        }
        self.charge(64)?;
        let length = self.reduce_get(&object, &JsString::from("length"), doc)?;
        let length = integer_or_infinity(self.number_value(length, doc)?)
            .clamp(0.0, 9_007_199_254_740_991.0) as u64;
        // Length access/conversion precedes validation, even for an empty
        // input. Initial presence must remain distinct from an undefined value.
        if !json_callable(&callback) {
            return Err(ScriptError::type_error(
                "reduction callback must be callable",
            ));
        }
        // No collection is allocated from this logical length. A huge sparse
        // range consumes the same shared work budget one visited index at a time.
        for visited in 0..length {
            self.tick()?;
            let index = match direction {
                ReduceDirection::Left => visited,
                ReduceDirection::Right => length - visited - 1,
            };
            let key = self.reduce_index_key(index)?;
            if self.reduce_property_in(&object, &key, doc)?.is_none() {
                continue;
            }
            let value = self.reduce_get(&object, &key, doc)?;
            let Some(previous) = accumulator.take() else {
                accumulator = Some(value);
                continue;
            };
            let bytes = std::mem::size_of::<Value>()
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(32))
                .ok_or_else(|| {
                    ScriptError::resource("reduction callback allocation limit exceeded")
                })?;
            self.charge(bytes)?;
            let mut arguments = Vec::new();
            arguments
                .try_reserve_exact(4)
                .map_err(|_| ScriptError::resource("reduction callback allocation failed"))?;
            arguments.extend([previous, value, Value::Number(index as f64), object.clone()]);
            accumulator = Some(self.call(callback.clone(), arguments, Value::Undefined, doc)?);
        }
        accumulator
            .ok_or_else(|| ScriptError::type_error("empty reduction requires an initial value"))
    }
    fn array_sort(
        &mut self,
        receiver: Value,
        comparator: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        // Validation precedes ToObject and every receiver/length property read.
        if comparator != Value::Undefined && !json_callable(&comparator) {
            return Err(ScriptError::type_error(
                "sort comparator must be callable or undefined",
            ));
        }
        let object = self.coerce_object(receiver)?;
        if self.property_object(&object).is_none() {
            return Err(ScriptError::unsupported(
                "host array-like sort is not implemented",
            ));
        }
        let length = self.get(object.clone(), "length", doc)?;
        let length = integer_or_infinity(self.number_value(length, doc)?)
            .clamp(0.0, 9_007_199_254_740_991.0);
        if length > 65_536.0 {
            return Err(ScriptError::resource("array-like length limit exceeded"));
        }
        let length = length as usize;
        let mut items = Vec::new();
        if length != 0 {
            let bytes = length
                .checked_mul(std::mem::size_of::<Value>())
                .and_then(|bytes| bytes.checked_add(32))
                .ok_or_else(|| {
                    ScriptError::resource("sort collection allocation limit exceeded")
                })?;
            self.charge(bytes)?;
            items
                .try_reserve_exact(length)
                .map_err(|_| ScriptError::resource("sort collection allocation failed"))?;
        }
        // Collect through live properties. Getter effects can change later
        // presence/values, but the saved length remains the traversal boundary.
        for index in 0..length {
            let key = self.sort_index_key(index)?;
            if self.find_property_in(&object, &key, doc)?.is_some() {
                items.push(self.get_key(object.clone(), &key, doc)?);
            }
        }
        let order = if items.len() > 1 {
            Some(self.sort_order(&items, &comparator, doc)?)
        } else {
            None
        };
        // No intrinsic receiver writes occur until all collection/comparison
        // succeeds. Commit is deliberately sequential, not a transaction:
        // a later setter/delete failure preserves prior successful effects.
        for index in 0..items.len() {
            let source = order.as_ref().map_or(index, |order| order[index]);
            let key = self.sort_index_key(index)?;
            self.set_key_strict(object.clone(), &key, items[source].clone(), true, doc)?;
        }
        for index in items.len()..length {
            let key = self.sort_index_key(index)?;
            if !self.delete_property(object.clone(), &key)? {
                return Err(ScriptError::type_error("sort cannot delete property"));
            }
        }
        Ok(object)
    }
    fn sort_index_key(&mut self, mut index: usize) -> Result<JsString> {
        if index >= 65_536 {
            return Err(ScriptError::resource("sort index limit exceeded"));
        }
        // Format directly into bounded UTF-16 scratch: no decimal String/Vec.
        // Cover all five digit operations and the one small retained Rc slice.
        self.work(6)?;
        self.charge(64)?;
        let mut digits = [0u16; 5];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = u16::from(b'0') + (index % 10) as u16;
            index /= 10;
            if index == 0 {
                break;
            }
        }
        Ok(JsString::from(&digits[start..]))
    }
    fn sort_compare(
        &mut self,
        left: &Value,
        right: &Value,
        comparator: &Value,
        doc: &mut Document,
    ) -> Result<Ordering> {
        self.tick()?;
        match (left, right) {
            (Value::Undefined, Value::Undefined) => return Ok(Ordering::Equal),
            (Value::Undefined, _) => return Ok(Ordering::Greater),
            (_, Value::Undefined) => return Ok(Ordering::Less),
            _ => {}
        }
        if comparator != &Value::Undefined {
            self.charge(32 + 2 * std::mem::size_of::<Value>())?;
            let mut arguments = Vec::new();
            arguments
                .try_reserve_exact(2)
                .map_err(|_| ScriptError::resource("sort callback allocation failed"))?;
            arguments.push(left.clone());
            arguments.push(right.clone());
            let result = self.call(comparator.clone(), arguments, Value::Undefined, doc)?;
            let number = self.number_value(result, doc)?;
            return Ok(if number < 0.0 {
                Ordering::Less
            } else if number > 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            });
        }
        // Conversion order is observable; never precompute/cache author hooks.
        let left = self.string_hint(left.clone(), doc)?;
        let right = self.string_hint(right.clone(), doc)?;
        self.work(1 + left.len().saturating_add(right.len()) / 8)?;
        Ok(left.units().cmp(right.units()))
    }
    fn sort_order(
        &mut self,
        items: &[Value],
        comparator: &Value,
        doc: &mut Document,
    ) -> Result<Vec<usize>> {
        let count = items.len();
        if count > 65_536 {
            return Err(ScriptError::resource("sort item limit exceeded"));
        }
        let bytes = count
            .checked_mul(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(64))
            .ok_or_else(|| ScriptError::resource("sort index allocation limit exceeded"))?;
        self.work(count.saturating_mul(2).saturating_add(1))?;
        self.charge(bytes)?;
        let mut order = Vec::new();
        let mut scratch = Vec::new();
        order
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("sort index allocation failed"))?;
        scratch
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("sort merge allocation failed"))?;
        order.extend(0..count);
        scratch.resize(count, 0);
        let mut width = 1usize;
        while width < count {
            let stride = width
                .checked_mul(2)
                .ok_or_else(|| ScriptError::resource("sort merge width overflow"))?;
            let mut begin = 0;
            while begin < count {
                let middle = begin.checked_add(width).unwrap_or(count).min(count);
                let end = begin.checked_add(stride).unwrap_or(count).min(count);
                let (mut left, mut right) = (begin, middle);
                for destination in &mut scratch[begin..end] {
                    self.tick()?;
                    // Each iteration advances one cursor, regardless of whether
                    // the author comparator defines a consistent ordering.
                    if left < middle
                        && (right == end
                            || self.sort_compare(
                                &items[order[left]],
                                &items[order[right]],
                                comparator,
                                doc,
                            )? != Ordering::Greater)
                    {
                        *destination = order[left];
                        left += 1;
                    } else {
                        *destination = order[right];
                        right += 1;
                    }
                }
                begin = end;
            }
            std::mem::swap(&mut order, &mut scratch);
            width = stride;
        }
        Ok(order)
    }
    fn array_reverse(&mut self, receiver: Value, doc: &mut Document) -> Result<Value> {
        let object = self.coerce_object(receiver)?;
        if self.property_object(&object).is_none() {
            return Err(ScriptError::unsupported(
                "host array-like reverse is not implemented",
            ));
        }
        let length = self.get(object.clone(), "length", doc)?;
        let length = integer_or_infinity(self.number_value(length, doc)?)
            .clamp(0.0, 9_007_199_254_740_991.0);
        if length > 65_536.0 {
            return Err(ScriptError::resource("array-like length limit exceeded"));
        }
        let length = length as usize;
        for lower in 0..length / 2 {
            self.tick()?;
            // Both index strings have at most five UTF-16 units under the cap.
            self.charge(96)?;
            let upper_key = JsString::from((length - lower - 1).to_string());
            let lower_key = JsString::from(lower.to_string());
            let lower_value = if self.find_property_in(&object, &lower_key, doc)?.is_some() {
                Some(self.get_key(object.clone(), &lower_key, doc)?)
            } else {
                None
            };
            let upper_value = if self.find_property_in(&object, &upper_key, doc)?.is_some() {
                Some(self.get_key(object.clone(), &upper_key, doc)?)
            } else {
                None
            };
            match (lower_value, upper_value) {
                (Some(lower), Some(upper)) => {
                    self.set_key_strict(object.clone(), &lower_key, upper, true, doc)?;
                    self.set_key_strict(object.clone(), &upper_key, lower, true, doc)?;
                }
                (None, Some(upper)) => {
                    self.set_key_strict(object.clone(), &lower_key, upper, true, doc)?;
                    if !self.delete_property(object.clone(), &upper_key)? {
                        return Err(ScriptError::type_error("reverse cannot delete property"));
                    }
                }
                (Some(lower), None) => {
                    if !self.delete_property(object.clone(), &lower_key)? {
                        return Err(ScriptError::type_error("reverse cannot delete property"));
                    }
                    self.set_key_strict(object.clone(), &upper_key, lower, true, doc)?;
                }
                (None, None) => {}
            }
        }
        Ok(object)
    }
    fn number_to_string(&mut self, number: f64, radix: Value, doc: &mut Document) -> Result<Value> {
        let radix = if radix == Value::Undefined {
            10.0
        } else {
            integer_or_infinity(self.number_value(radix, doc)?)
        };
        if !(2.0..=36.0).contains(&radix) {
            return Err(ScriptError::range_error(
                "number radix must be between 2 and 36",
            ));
        }
        if radix == 10.0 || !number.is_finite() || number == 0.0 {
            return self.string(Value::Number(number).js_string());
        }
        if number.fract() != 0.0 || number.abs() > 9_007_199_254_740_991.0 {
            return Err(ScriptError::unsupported(
                "nondecimal formatting requires a finite safe integer",
            ));
        }
        let radix = radix as u64;
        let mut magnitude = number.abs() as u64;
        let mut units = [0u16; 54];
        let mut start = units.len();
        while magnitude != 0 {
            self.tick()?;
            let digit = (magnitude % radix) as u16;
            start -= 1;
            units[start] = if digit < 10 {
                u16::from(b'0') + digit
            } else {
                u16::from(b'a') + digit - 10
            };
            magnitude /= radix;
        }
        if number < 0.0 {
            start -= 1;
            units[start] = u16::from(b'-');
        }
        self.charge((units.len() - start).saturating_mul(2) + 24)?;
        Ok(Value::String(JsString::from(units[start..].to_vec())))
    }
    fn primitive_number_value(&mut self, value: Value) -> Result<f64> {
        if matches!(value, Value::Symbol(_)) {
            return Err(ScriptError::type_error(
                "cannot convert a symbol to a number",
            ));
        }
        if let Value::String(text) = &value {
            // Trimming, ASCII validation/copy and decimal/radix parsing make
            // bounded passes over the code units. Cover temporary ASCII
            // storage before JsString::number constructs it.
            self.work(1 + text.len().saturating_mul(6) / 8)?;
            self.charge(32 + text.len().saturating_mul(2))?;
        }
        Ok(value.number())
    }
    fn number_value(&mut self, value: Value, doc: &mut Document) -> Result<f64> {
        if !js_object(&value) {
            return self.primitive_number_value(value);
        }
        if let Some(primitive) = self.exotic_primitive(&value, "number", doc)? {
            return self.primitive_number_value(primitive);
        }
        for key in ["valueOf", "toString"] {
            self.charge(64)?;
            let method = self.get(value.clone(), key, doc)?;
            if json_callable(&method) {
                let primitive = self.call(method, Vec::new(), value.clone(), doc)?;
                if !js_object(&primitive) {
                    return self.primitive_number_value(primitive);
                }
            }
        }
        Err(ScriptError::type_error(
            "object cannot be converted to a number",
        ))
    }

    fn get_key(&mut self, receiver: Value, key: &JsString, doc: &mut Document) -> Result<Value> {
        self.work(1 + key.len() / 8)?;
        if let Value::Style(id) = receiver {
            self.work(key.len().saturating_add(1))?;
            self.charge(24 + key.len().saturating_mul(6))?;
            return match key.to_utf8() {
                Ok(key) => self.style_get(id, &key, doc),
                Err(_) => Ok(self
                    .lookup_property(&Value::Style(id), key, doc)?
                    .unwrap_or(Value::Undefined)),
            };
        }
        if let Some(value) = self.lookup_property(&receiver, key, doc)? {
            return Ok(value);
        }
        match key.to_utf8() {
            Ok(key) => self.get(receiver, &key, doc),
            Err(_) if matches!(receiver, Value::Null | Value::Undefined) => Err(
                ScriptError::type_error("cannot read property of null or undefined"),
            ),
            Err(_) => Ok(Value::Undefined),
        }
    }
    fn set_key_strict(
        &mut self,
        receiver: Value,
        key: &JsString,
        value: Value,
        strict: bool,
        doc: &mut Document,
    ) -> Result<()> {
        self.work(1 + key.len() / 8)?;
        if dom_own_properties::host(&receiver).is_some()
            && self.dom_set_ordinary(&receiver, key, &value, strict, doc)?
        {
            return Ok(());
        }
        if let Value::Style(id) = receiver {
            self.work(key.len().saturating_add(1))?;
            self.charge(24 + key.len().saturating_mul(6))?;
            let key = key.to_utf8().map_err(|_| {
                ScriptError::unsupported("non-scalar style expando properties are not implemented")
            })?;
            return self.style_set(id, &key, value, strict, doc);
        }
        if matches!(receiver, Value::Node(_) | Value::Document | Value::Window)
            && let Ok(name) = key.to_utf8()
            && event_handler_name(&name)
        {
            let target = self.event_target(&receiver, doc)?;
            self.sync_event_handler(target, &JsString::from(&name[2..]), doc)?;
            return self.set_event_handler(target, JsString::from(&name[2..]), value, true);
        }
        if matches!(receiver, Value::Document)
            && [
                "characterSet",
                "charset",
                "inputEncoding",
                "compatMode",
                "URL",
                "documentURI",
                "baseURI",
            ]
            .iter()
            .any(|name| key == &JsString::from(*name))
        {
            return Self::failed_write(strict);
        }
        if let Some(property) = self.find_property_in(&receiver, key, doc)? {
            match property.value {
                PropertyValue::Accessor {
                    set: Value::Undefined,
                    ..
                }
                | PropertyValue::Data {
                    writable: false, ..
                } => return Self::failed_write(strict),
                PropertyValue::Accessor { set, .. } => {
                    self.call(set, vec![value], receiver, doc)?;
                    return Ok(());
                }
                _ => {}
            }
        }
        if let Some(id) = self.property_object(&receiver) {
            let indexed = matches!(receiver, Value::Array(_))
                && (key == &JsString::from("length") || json_array_index(key).is_some());
            if indexed {
                let desc = if self.own_property(&receiver, key).is_some() {
                    PropertyDescriptor {
                        value: Some(value),
                        ..PropertyDescriptor::default()
                    }
                } else {
                    PropertyDescriptor::data_property(value, true, true, true)
                };
                return if self.define_property_key(&receiver, &key.into(), desc, doc)? {
                    Ok(())
                } else {
                    Self::failed_write(strict)
                };
            } else {
                if let Some(property) = self.objects[id].values.get_mut(&key.into()) {
                    if let PropertyValue::Data { value: old, .. } = &mut property.value {
                        *old = value.clone();
                    }
                    if let Some((env, name)) = self.objects[id].parameter_map.get(key) {
                        self.environments[*env]
                            .bindings
                            .get_mut(name)
                            .unwrap()
                            .value = value;
                    }
                    return Ok(());
                }
                if self.objects[id].non_extensible {
                    return Self::failed_write(strict);
                }
                self.charge(256 + key.byte_len().saturating_mul(2))?;
                self.objects[id].insert(key.clone(), value);
                return Ok(());
            }
        } else if !js_object(&receiver) && !matches!(receiver, Value::Null | Value::Undefined) {
            return Self::failed_write(strict);
        }
        if receiver == Value::Window {
            return self.window_write_data(key, value);
        }
        let key = key
            .to_utf8()
            .map_err(|_| ScriptError::type_error("non-scalar host property name is unsupported"))?;
        self.set(receiver, &key, value, doc)
    }
    fn failed_write(strict: bool) -> Result<()> {
        if strict {
            Err(ScriptError::type_error("property cannot be assigned"))
        } else {
            Ok(())
        }
    }
    fn get(&mut self, receiver: Value, key: &str, doc: &mut Document) -> Result<Value> {
        if dom_own_properties::host(&receiver).is_some()
            && let Some(value) = self.dom_own_get_utf8(&receiver, key, doc)?
        {
            return Ok(value);
        }
        let absent_dom_attribute = dom_own_properties::host(&receiver).is_some()
            && self.dom_attribute_get(&receiver, key, doc)?;
        if let Value::Style(id) = receiver {
            return self.style_get(id, key, doc);
        }
        if !absent_dom_attribute
            && matches!(receiver, Value::Node(_) | Value::Document | Value::Window)
            && event_handler_name(key)
        {
            let target = self.event_target(&receiver, doc)?;
            let kind = JsString::from(&key[2..]);
            self.sync_event_handler(target, &kind, doc)?;
            return self.event_handler_callback(target, &kind);
        }
        if let Some(value) = self.lookup_property(&receiver, &key.into(), doc)? {
            return Ok(value);
        }
        if absent_dom_attribute
            || self.property_object(&receiver).is_some()
            || matches!(
                receiver,
                Value::String(_) | Value::Number(_) | Value::Bool(_)
            )
        {
            return Ok(Value::Undefined);
        }
        if matches!(receiver, Value::Document)
            && !matches!(
                key,
                "URL"
                    | "documentURI"
                    | "baseURI"
                    | "characterSet"
                    | "charset"
                    | "inputEncoding"
                    | "compatMode"
            )
            || matches!(key, "innerText" | "innerHTML" | "outerHTML")
        {
            self.work(1 + doc.nodes.len() / 8)?;
        }
        match &receiver {
            Value::Undefined | Value::Null => {
                return Err(ScriptError::type_error(format!(
                    "cannot read property '{key}' of {receiver}"
                )));
            }
            Value::Object(id) => {
                if let Some(value) = self.objects[*id].get(key) {
                    return Ok(value.clone());
                }
            }
            Value::String(text) => {
                if key == "length" {
                    return Ok(Value::Number(text.len() as f64));
                }
                if let Ok(index) = key.parse::<usize>()
                    && index.to_string() == key
                {
                    return match text.units().get(index) {
                        Some(unit) => self.string(vec![*unit]),
                        None => Ok(Value::Undefined),
                    };
                }
                if [
                    "toUpperCase",
                    "toLowerCase",
                    "trim",
                    "includes",
                    "startsWith",
                    "endsWith",
                    "indexOf",
                    "slice",
                    "substring",
                    "split",
                    "charAt",
                    "charCodeAt",
                    "codePointAt",
                    "toString",
                ]
                .contains(&key)
                {
                    return self.alloc_native(key, receiver);
                }
            }
            Value::Native(native)
                if native.name == "String" && matches!(key, "fromCharCode" | "fromCodePoint") =>
            {
                return self.alloc_native(key, receiver);
            }
            Value::Number(_) | Value::Bool(_) if key == "toString" => {
                return self.alloc_native(key, receiver);
            }
            Value::Console if ["log", "warn", "error", "info", "debug"].contains(&key) => {
                return self.alloc_native(key, receiver);
            }
            Value::Json if ["parse", "stringify"].contains(&key) => {
                return self.alloc_native(key, receiver);
            }
            Value::Math => {
                if key == "PI" {
                    return Ok(Value::Number(std::f64::consts::PI));
                }
                if key == "E" {
                    return Ok(Value::Number(std::f64::consts::E));
                }
                if [
                    "abs", "floor", "ceil", "round", "trunc", "sqrt", "pow", "min", "max", "sin",
                    "cos", "tan", "log", "exp", "sign",
                ]
                .contains(&key)
                {
                    return self.alloc_native(key, receiver);
                }
            }
            Value::Document => match key {
                "defaultView" => return Ok(Value::Window),
                "URL" | "documentURI" => return self.string(doc.url().as_str()),
                "baseURI" => {
                    return self.string(doc.base_url().as_str());
                }
                "characterSet" | "charset" | "inputEncoding" => {
                    return self.string(doc.character_set());
                }
                "compatMode" => {
                    return self.string(if doc.mode() == crate::dom::DocumentMode::Quirks {
                        "BackCompat"
                    } else {
                        "CSS1Compat"
                    });
                }
                "body" | "head" => {
                    return Ok(html_document_child(doc, key)
                        .map(Value::Node)
                        .unwrap_or(Value::Null));
                }
                "documentElement" => {
                    return Ok(document_element(doc)
                        .map(Value::Node)
                        .unwrap_or(Value::Null));
                }
                "readyState" => return self.string("complete"),
                "getElementById"
                | "getElementsByTagName"
                | "getElementsByClassName"
                | "createElement"
                | "createTextNode"
                | "createDocumentFragment" => return self.dom_method(key, false),
                "createEvent" => return self.alloc_native(key, receiver),
                _ => {}
            },
            Value::Node(id) => {
                let id = *id;
                if id >= doc.nodes.len() {
                    return Err(ScriptError::new("invalid DOM node"));
                }
                match key {
                    "open"
                        if doc.namespace(id) == Some(Namespace::Html)
                            && doc.tag(id) == Some("details") =>
                    {
                        return Ok(Value::Bool(doc.attr(id, "open").is_some()));
                    }
                    "name"
                        if doc.namespace(id) == Some(Namespace::Html)
                            && doc.tag(id) == Some("details") =>
                    {
                        return self.string(doc.attr(id, "name").unwrap_or(""));
                    }
                    "content" if doc.template_contents(id).is_some() => {
                        return Ok(Value::Node(doc.template_contents(id).unwrap()));
                    }
                    "innerText" => {
                        return Ok(Value::String(self.dom_text_units(id, doc)?.into()));
                    }
                    "innerHTML" => return self.dom_html(id, true, doc),
                    "outerHTML" => return self.dom_html(id, false, doc),
                    "value" if doc.tag(id) == Some("textarea") => {
                        self.work(1 + doc.nodes.len() / 8)?;
                        let mut text = self.dom_text_units(id, doc)?;
                        self.work(1 + 2 * text.len())?;
                        let mut read = 0;
                        let mut write = 0;
                        while read < text.len() {
                            let unit = text[read];
                            read += 1;
                            text[write] = if unit == 13 {
                                if text.get(read) == Some(&10) {
                                    read += 1;
                                }
                                10
                            } else {
                                unit
                            };
                            write += 1;
                        }
                        text.truncate(write);
                        return Ok(Value::String(text.into()));
                    }
                    "id" | "title" | "value" | "href" | "src" | "type" => {
                        return self.string(doc.attr(id, key).unwrap_or(""));
                    }
                    "className" => return self.string(doc.attr(id, "class").unwrap_or("")),
                    "tagName" | "nodeName" => {
                        return self.string(match &doc.nodes[id].kind {
                            NodeKind::DocumentFragment { .. } => "#document-fragment".into(),
                            NodeKind::Document => "#document".into(),
                            NodeKind::Text(_) => "#text".into(),
                            NodeKind::Comment(_) => "#comment".into(),
                            NodeKind::ProcessingInstruction { target, .. } => target.clone(),
                            NodeKind::Doctype(doctype) => doctype.name.clone(),
                            NodeKind::Element(element) if element.namespace == Namespace::Html => {
                                element.tag.to_ascii_uppercase()
                            }
                            NodeKind::Element(element) => element.tag.clone(),
                        });
                    }
                    "namespaceURI" => {
                        return match doc.namespace(id) {
                            Some(namespace) => self.string(namespace.uri()),
                            None => Ok(Value::Null),
                        };
                    }
                    "nodeType" => {
                        return Ok(Value::Number(match doc.nodes[id].kind {
                            NodeKind::DocumentFragment { .. } => 11.0,
                            NodeKind::Document => 9.0,
                            NodeKind::Element(_) => 1.0,
                            NodeKind::Text(_) => 3.0,
                            NodeKind::Comment(_) => 8.0,
                            NodeKind::ProcessingInstruction { .. } => 7.0,
                            NodeKind::Doctype(_) => 10.0,
                        }));
                    }
                    "parentNode" | "parentElement" => {
                        return Ok(doc.nodes[id]
                            .parent
                            .filter(|parent| key == "parentNode" || doc.tag(*parent).is_some())
                            .map(|parent| {
                                if parent == doc.root {
                                    Value::Document
                                } else {
                                    Value::Node(parent)
                                }
                            })
                            .unwrap_or(Value::Null));
                    }
                    "firstChild" => {
                        return Ok(doc.nodes[id]
                            .children
                            .first()
                            .copied()
                            .map(Value::Node)
                            .unwrap_or(Value::Null));
                    }
                    "lastChild" => {
                        return Ok(doc.nodes[id]
                            .children
                            .last()
                            .copied()
                            .map(Value::Node)
                            .unwrap_or(Value::Null));
                    }
                    "children" | "childNodes" => {
                        let children = doc.nodes[id]
                            .children
                            .iter()
                            .filter(|child| key == "childNodes" || doc.tag(**child).is_some())
                            .copied()
                            .map(Value::Node)
                            .collect();
                        return self.array(children);
                    }
                    "style" => return Ok(Value::Style(id)),
                    "classList" => return Ok(Value::ClassList(id)),
                    "checked" | "disabled" | "hidden" => {
                        return Ok(Value::Bool(doc.attr(id, key).is_some()));
                    }
                    "appendChild" | "removeChild" | "remove" | "setAttribute" | "getAttribute"
                    | "hasAttribute" | "removeAttribute" => return self.dom_method(key, false),
                    "cloneNode" => return self.dom_method(key, false),
                    _ => {}
                }
            }
            Value::ClassList(id) => {
                if ["add", "remove", "toggle", "contains"].contains(&key) {
                    return self.dom_method(key, true);
                }
                if key == "length" {
                    return self.class_list_length(*id, doc);
                }
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    // Console display is diagnostic formatting, separate from DOM conversion.
    // Charge an author-sized Symbol description before formatting it.
    fn display_value(&mut self, value: &Value) -> Result<String> {
        if let Value::Symbol(symbol) = value {
            let length = symbol
                .description()
                .map_or(0, JsString::len)
                .saturating_add(8);
            self.work(1 + length / 8)?;
            self.charge(length.saturating_mul(3) + 32)?;
        }
        Ok(value.to_string())
    }

    fn set(&mut self, receiver: Value, key: &str, value: Value, doc: &mut Document) -> Result<()> {
        // Host writes may decode strings or parse array lengths. Charge their
        // linear work even when the string itself is an existing shared value.
        if let Value::String(text) = &value {
            self.work(1 + text.len() / 16)?;
        }
        match receiver {
            Value::Object(id) => {
                if !self.objects[id].contains_key(key) {
                    self.charge(key.len().saturating_mul(2) + 160)?;
                }
                self.objects[id].insert(key.into(), value);
            }
            Value::Array(_) => {
                self.set_key_strict(receiver, &key.into(), value, false, doc)?;
            }
            Value::Window => {
                let key = self.global_name_key(key)?;
                self.set_key_strict(Value::Window, &key, value, false, doc)?;
            }
            Value::Node(id) => {
                if id >= doc.nodes.len() {
                    return Err(ScriptError::new("invalid DOM node"));
                }
                if doc.namespace(id) == Some(Namespace::Html) && doc.tag(id) == Some("details") {
                    if key == "open" {
                        self.charge_details_attribute(id, key, 0, doc)?;
                        if value.truthy() {
                            doc.set_attr(id, "open", "");
                        } else {
                            doc.remove_attr(id, "open");
                        }
                        return Ok(());
                    }
                    if key == "name" {
                        let name = self.string_hint(value, doc)?;
                        self.work(1 + name.len() / 16)?;
                        self.charge(name.len().saturating_mul(3))?;
                        let text = name.to_utf8_lossy();
                        self.charge_details_attribute(id, key, text.len(), doc)?;
                        doc.set_attr(id, "name", &text);
                        return Ok(());
                    }
                }
                if matches!(key, "checked" | "disabled" | "hidden") {
                    if value.truthy() {
                        doc.set_attr(id, key, "");
                    } else {
                        doc.remove_attr(id, key);
                    }
                    return Ok(());
                }
                let text = if matches!(key, "innerText" | "innerHTML") && value == Value::Null
                    || key == "value" && doc.tag(id) == Some("textarea") && value == Value::Null
                {
                    String::new()
                } else {
                    self.dom_string(value, doc)?
                };
                self.charge(text.len())?;
                match key {
                    "innerText" => {
                        self.ensure_dom_capacity(doc, 1)?;
                        self.charge_dom_clear(id, doc)?;
                        doc.set_text_content(id, &text);
                    }
                    "innerHTML" => self.set_inner_html(id, &text, doc)?,
                    "value" if doc.tag(id) == Some("textarea") => {
                        // Until separate raw/default/dirty values exist, use
                        // the same text storage as native editing and forms.
                        self.ensure_dom_capacity(doc, 1)?;
                        self.work(1 + text.len() / 16)?;
                        self.charge(text.len().saturating_mul(2))?;
                        let text = text.replace("\r\n", "\n").replace('\r', "\n");
                        self.charge_dom_clear(id, doc)?;
                        doc.set_text_content(id, &text);
                    }
                    "className" => doc.set_attr(id, "class", &text),
                    "id" | "title" | "value" | "href" | "src" | "type" => {
                        let work = doc.base_attribute_work(id, key);
                        self.work(work.saturating_add(if work > 0 { text.len() } else { 0 }))?;
                        doc.set_attr(id, key, &text)
                    }
                    _ => {
                        return Err(ScriptError::new(format!(
                            "unsupported DOM property '{key}'"
                        )));
                    }
                }
            }
            Value::Style(id) => return self.style_set(id, key, value, false, doc),
            _ => {
                return Err(ScriptError::type_error(format!(
                    "cannot set property '{key}'"
                )));
            }
        }
        Ok(())
    }

    fn ensure_dom_capacity(&mut self, doc: &Document, count: usize) -> Result<()> {
        if doc.nodes.len().saturating_add(count) > MAX_NODES {
            return Err(ScriptError::resource("DOM node limit exceeded"));
        }
        self.charge(count.saturating_mul(128))
    }

    // Exact output avoids an intermediate display String. The planning walk
    // counts only selected text; unrelated retained payloads are never scanned.
    fn dom_text_units(&mut self, id: NodeId, doc: &Document) -> Result<Vec<u16>> {
        let count = doc.nodes.len();
        // The shared allocation-free classifier checks at most two nodes.
        // Empty and single-Text containers, like CharacterData leaves, need
        // no arena-sized traversal scratch for unrelated retained nodes.
        self.work(16)?;
        match doc
            .text_content_shape(id)
            .map_err(processing_instruction::dom_data_error)?
        {
            crate::dom::TextContentShape::Empty => {
                self.charge(64)?;
                return Ok(Vec::new());
            }
            crate::dom::TextContentShape::Single(text) => {
                self.work(1 + text.stored_bytes())?;
                let length = text.units().count();
                if length > MAX_STRING {
                    return Err(ScriptError::resource("script string limit exceeded"));
                }
                self.work(1 + text.stored_bytes() + 2 * length)?;
                self.charge(64 + 4 * length)?;
                let mut output = Vec::new();
                output
                    .try_reserve_exact(length)
                    .map_err(|_| ScriptError::resource("DOM read allocation failed"))?;
                output.extend(text.units());
                return Ok(output);
            }
            crate::dom::TextContentShape::Tree => {}
        }
        if count > MAX_NODES {
            return Err(ScriptError::resource("DOM read traversal limit exceeded"));
        }
        let stack_bytes = count
            .checked_mul(std::mem::size_of::<NodeId>())
            .ok_or_else(|| ScriptError::resource("DOM read scratch overflow"))?;
        self.charge(24 + stack_bytes)?;
        let mut pending = Vec::new();
        pending
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("DOM read scratch allocation failed"))?;
        pending.push(id);
        let mut visits = 0usize;
        let mut units = 0usize;
        let mut source_bytes = 0usize;
        while let Some(next) = pending.pop() {
            self.tick()?;
            visits += 1;
            if visits > count {
                return Err(ScriptError::resource("DOM read traversal limit exceeded"));
            }
            let node = doc
                .nodes
                .get(next)
                .ok_or_else(|| ScriptError::type_error("invalid DOM read node"))?;
            let text = match &node.kind {
                NodeKind::ProcessingInstruction { data, .. } if next == id => Some(data),
                NodeKind::Text(text) | NodeKind::Comment(text)
                    if next == id || matches!(node.kind, NodeKind::Text(_)) =>
                {
                    Some(text)
                }
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_) => {
                    self.work(node.children.len())?;
                    if node.children.len() > count.saturating_sub(pending.len()) {
                        return Err(ScriptError::resource("DOM read traversal limit exceeded"));
                    }
                    pending.extend(node.children.iter().rev().copied());
                    None
                }
                _ => None,
            };
            if let Some(text) = text {
                self.work(1 + text.stored_bytes())?;
                source_bytes = source_bytes
                    .checked_add(text.stored_bytes())
                    .ok_or_else(|| ScriptError::resource("DOM read size overflow"))?;
                units = units
                    .checked_add(text.units().count())
                    .filter(|length| *length <= MAX_STRING)
                    .ok_or_else(|| ScriptError::resource("script string limit exceeded"))?;
            }
        }
        drop(pending);
        // Core allocates a NodeId stack, a reference list and initialized seen
        // bytes. Output Vec and its eventual Rc copy are both admitted here.
        let scratch = count
            .checked_mul(
                std::mem::size_of::<NodeId>()
                    + std::mem::size_of::<&crate::dom::DomString>()
                    + std::mem::size_of::<bool>(),
            )
            .and_then(|bytes| bytes.checked_add(128 + 4 * units))
            .ok_or_else(|| ScriptError::resource("DOM read scratch overflow"))?;
        self.charge(scratch)?;
        self.work(
            count
                .saturating_add(2 * source_bytes)
                .saturating_add(2 * units),
        )?;
        doc.text_content_units_bounded(id, MAX_STRING, &mut self.steps)
            .map_err(processing_instruction::dom_data_error)
    }

    fn dom_reserve_node_growth(&mut self, doc: &mut Document, count: usize) -> Result<()> {
        let required = doc
            .nodes
            .len()
            .checked_add(count)
            .filter(|required| *required <= MAX_NODES)
            .ok_or_else(|| ScriptError::resource("DOM node limit exceeded"))?;
        if count > doc.nodes.capacity().saturating_sub(doc.nodes.len()) {
            self.work(1 + 2 * doc.nodes.len())?;
            self.charge(
                required
                    .checked_mul(std::mem::size_of::<crate::dom::Node>())
                    .ok_or_else(|| ScriptError::resource("DOM node storage overflow"))?,
            )?;
            doc.nodes
                .try_reserve_exact(count)
                .map_err(|_| ScriptError::resource("DOM node allocation failed"))?;
        }
        Ok(())
    }

    fn dom_html(&mut self, id: NodeId, children: bool, doc: &Document) -> Result<Value> {
        let mut plan = ScriptHtml::new(None);
        if children {
            serialize_children_at(self, doc, id, 0, &mut plan)?;
        } else {
            serialize_node(self, doc, id, 0, &mut plan)?;
        }
        plan.finish()?;
        let (bytes, units) = (plan.bytes, plan.units);
        self.charge(bytes + 4 * units + 64)?;
        self.work(2 * bytes + 2 * units)?;
        let mut output = String::new();
        output
            .try_reserve_exact(bytes)
            .map_err(|_| ScriptError::resource("HTML serialization allocation failed"))?;
        let mut sink = ScriptHtml::new(Some(&mut output));
        if children {
            serialize_children_at(self, doc, id, 0, &mut sink)?;
        } else {
            serialize_node(self, doc, id, 0, &mut sink)?;
        }
        sink.finish()?;
        if sink.bytes != bytes || sink.units != units {
            return Err(ScriptError::resource(
                "HTML serialization changed during copy",
            ));
        }
        let mut encoded = Vec::new();
        encoded
            .try_reserve_exact(units)
            .map_err(|_| ScriptError::resource("HTML result allocation failed"))?;
        encoded.extend(output.encode_utf16());
        Ok(Value::String(encoded.into()))
    }

    // append_child validates the entire host-inclusive subtree, walks the
    // destination's host-inclusive ancestors, and removes the old sibling
    // entry. Account for all that work before it changes either child list.
    fn charge_dom_append(&mut self, parent: NodeId, child: NodeId, doc: &Document) -> Result<()> {
        self.charge_dom_append_admission(parent, child, doc)
            .map(|_| ())
    }

    fn charge_dom_append_admission(
        &mut self,
        parent: NodeId,
        child: NodeId,
        doc: &Document,
    ) -> Result<bool> {
        self.tick()?;
        if parent >= doc.nodes.len()
            || child >= doc.nodes.len()
            || child == doc.root
            || parent == child
            || !matches!(
                doc.nodes[parent].kind,
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
            )
            || matches!(doc.nodes[child].kind, NodeKind::Doctype(_))
                && !matches!(doc.nodes[parent].kind, NodeKind::Document)
        {
            return Ok(false);
        }
        let mut cursor = Some(parent);
        let mut ancestors = 0;
        while let Some(id) = cursor {
            self.tick()?;
            if id == child || ancestors >= crate::dom::MAX_DEPTH {
                return Ok(false);
            }
            ancestors += 1;
            cursor = doc.nodes[id].parent.or(match doc.nodes[id].kind {
                NodeKind::DocumentFragment { host } => host,
                _ => None,
            });
        }
        self.charge(32)?;
        let mut pending = vec![(child, 1usize)];
        let mut visited = 0;
        let mut contains_base = false;
        while let Some((id, depth)) = pending.pop() {
            self.tick()?;
            visited += 1;
            if ancestors + depth > crate::dom::MAX_DEPTH || visited > MAX_NODES {
                return Ok(false);
            }
            let children = &doc.nodes[id].children;
            contains_base |= matches!(&doc.nodes[id].kind, NodeKind::Element(element)
                if element.namespace == Namespace::Html && element.tag == "base" && element.attrs.contains_key("href"));
            let contents = doc.template_contents(id);
            let edges = children.len() + usize::from(contents.is_some());
            self.work(edges)?;
            // Both this preflight and the DOM validation allocate a traversal
            // stack. Charge before growing either one.
            self.charge(edges.saturating_mul(32))?;
            pending.extend(children.iter().map(|id| (*id, depth + 1)));
            if let Some(contents) = contents {
                pending.push((contents, depth + 1));
            }
        }
        let inserted = if matches!(doc.nodes[child].kind, NodeKind::DocumentFragment { .. }) {
            let count = doc.nodes[child].children.len();
            self.work(count)?;
            count
        } else {
            if let Some(old) = doc.nodes[child].parent {
                self.work(doc.nodes[old].children.len())?;
            }
            1
        };
        self.work(doc.base_tree_change_work(parent, child, contains_base))?;
        self.work(doc.details_tree_change_work(parent, child))?;
        self.charge(inserted.saturating_mul(2 * std::mem::size_of::<NodeId>()))?;
        Ok(true)
    }

    fn charge_dom_remove(&mut self, parent: NodeId, doc: &Document) -> Result<()> {
        self.work(
            doc.nodes[parent]
                .children
                .len()
                .saturating_add(doc.details_tree_change_work(parent, parent)),
        )
    }
    fn charge_dom_clear(&mut self, parent: NodeId, doc: &Document) -> Result<()> {
        self.work(
            doc.nodes[parent]
                .children
                .len()
                .saturating_add(doc.base_clear_work(parent))
                .saturating_add(doc.details_tree_change_work(parent, parent)),
        )
    }
    fn charge_details_attribute(
        &mut self,
        id: NodeId,
        key: &str,
        incoming: usize,
        doc: &Document,
    ) -> Result<()> {
        let work = doc.details_attribute_work(id, key);
        if work > 0 {
            self.work(work.saturating_add(incoming))?;
            // Queue/group bookkeeping is retained by the DOM rather than the
            // script heap, but script-triggered growth still consumes quota.
            self.charge(256)?;
        }
        Ok(())
    }

    fn set_inner_html(&mut self, id: NodeId, source: &str, doc: &mut Document) -> Result<()> {
        if source.len() > MAX_SOURCE {
            return Err(ScriptError::resource("HTML fragment source limit exceeded"));
        }
        let fragment = doc
            .parse_fragment(id, source)
            .map_err(ScriptError::unsupported)?;
        if fragment.retained_bytes()
            > crate::dom::MAX_DOM_BYTES.saturating_sub(doc.retained_bytes())
        {
            return Err(ScriptError::resource("DOM fragment storage limit exceeded"));
        }
        let mut pending = vec![(fragment.root, 0usize)];
        let mut details_count = 0usize;
        let mut details_name_bytes = 0usize;
        while let Some((node, depth)) = pending.pop() {
            self.tick()?;
            if depth > 96 {
                return Err(ScriptError::resource("DOM fragment nesting limit exceeded"));
            }
            self.charge(fragment.nodes[node].children.len().saturating_mul(16))?;
            pending.extend(
                fragment.nodes[node]
                    .children
                    .iter()
                    .map(|child| (*child, depth + 1)),
            );
            if let Some(contents) = fragment.template_contents(node) {
                pending.push((contents, depth));
            }
            if fragment.namespace(node) == Some(Namespace::Html)
                && fragment.tag(node) == Some("details")
            {
                details_count += 1;
                details_name_bytes = details_name_bytes
                    .saturating_add(fragment.attr(node, "name").map_or(0, str::len));
            }
        }
        self.work(
            doc.details_bulk_change_work(
                fragment.nodes.len().saturating_add(1),
                details_name_bytes,
            ),
        )?;
        self.charge(
            details_count
                .saturating_mul(256)
                .saturating_add(details_name_bytes.saturating_mul(4)),
        )?;
        self.ensure_dom_capacity(doc, fragment.nodes.len().saturating_add(1))?;
        self.dom_reserve_node_growth(doc, fragment.nodes.len().saturating_add(1))?;
        let id = doc.template_contents(id).unwrap_or(id);
        let staging = doc.create_document_fragment();
        for child in fragment.nodes[fragment.root].children.clone() {
            import_node(self, doc, staging, &fragment, child, 0)?;
        }
        self.charge_dom_clear(id, doc)?;
        self.charge_dom_append(id, staging, doc)?;
        doc.clear_children(id);
        doc.append_child(id, staging);
        Ok(())
    }
    fn clone_dom_node(&mut self, source: NodeId, deep: bool, doc: &mut Document) -> Result<NodeId> {
        if source >= doc.nodes.len() {
            return Err(ScriptError::type_error("invalid DOM clone source"));
        }
        let mut pending = vec![source];
        let mut new_nodes = 0usize;
        let mut details_count = 0usize;
        let mut details_name_bytes = 0usize;
        self.charge(std::mem::size_of::<NodeId>())?;
        while let Some(node) = pending.pop() {
            self.tick()?;
            new_nodes += 1;
            if doc.namespace(node) == Some(Namespace::Html) && doc.tag(node) == Some("details") {
                details_count += 1;
                details_name_bytes =
                    details_name_bytes.saturating_add(doc.attr(node, "name").map_or(0, str::len));
            }
            if deep {
                let edges = doc.nodes[node].children.len()
                    + usize::from(doc.template_contents(node).is_some());
                self.charge(edges.saturating_mul(2 * std::mem::size_of::<NodeId>()))?;
                pending.extend(doc.nodes[node].children.iter().copied());
                if let Some(contents) = doc.template_contents(node) {
                    pending.push(contents);
                }
            } else {
                new_nodes += usize::from(doc.template_contents(node).is_some());
            }
        }
        self.work(doc.details_bulk_change_work(new_nodes, details_name_bytes))?;
        self.charge(
            details_count
                .saturating_mul(256)
                .saturating_add(details_name_bytes.saturating_mul(4)),
        )?;
        let root = self.clone_dom_shallow(source, doc)?;
        if !deep {
            return Ok(root);
        }
        let mut pending = vec![(source, root, 0usize)];
        self.charge(24)?;
        while let Some((old, new, depth)) = pending.pop() {
            if depth >= 96 {
                return Err(ScriptError::resource("DOM clone nesting limit exceeded"));
            }
            self.tick()?;
            self.charge(doc.nodes[old].children.len().saturating_mul(32))?;
            let children = doc.nodes[old].children.clone();
            for child in children {
                let copy = self.clone_dom_shallow(child, doc)?;
                doc.append_child(new, copy);
                pending.push((child, copy, depth + 1));
            }
            if let Some(old_content) = doc.template_contents(old)
                && let Some(new_content) = doc.template_contents(new)
            {
                self.charge(24)?;
                pending.push((old_content, new_content, depth + 1));
            }
        }
        Ok(root)
    }
    fn clone_dom_shallow(&mut self, source: NodeId, doc: &mut Document) -> Result<NodeId> {
        let node = doc
            .nodes
            .get(source)
            .ok_or_else(|| ScriptError::type_error("invalid DOM clone source"))?;
        let bytes = match &node.kind {
            NodeKind::Element(element) => {
                element.tag.len()
                    + element
                        .attrs
                        .iter()
                        .map(|(k, v)| k.len() + v.len() + 96)
                        .sum::<usize>()
            }
            NodeKind::Text(text) | NodeKind::Comment(text) => text.stored_bytes(),
            NodeKind::ProcessingInstruction { target, data } => target.len() + data.stored_bytes(),
            NodeKind::Doctype(value) => {
                value.name.len()
                    + value.public_id.as_ref().map_or(0, String::len)
                    + value.system_id.as_ref().map_or(0, String::len)
            }
            NodeKind::DocumentFragment { .. } => 0,
            NodeKind::Document => {
                return Err(ScriptError::unsupported(
                    "Document cloning is not implemented",
                ));
            }
        };
        let count = if doc.template_contents(source).is_some() {
            2
        } else {
            1
        };
        if bytes > crate::dom::MAX_DOM_BYTES.saturating_sub(doc.retained_bytes()) {
            return Err(ScriptError::resource("DOM clone storage limit exceeded"));
        }
        self.work(1 + bytes / 16)?;
        self.charge(bytes.saturating_mul(2))?;
        self.ensure_dom_capacity(doc, count)?;
        self.dom_reserve_node_growth(doc, count)?;
        let kind = doc.nodes[source].kind.clone();
        let id = match kind {
            NodeKind::Document => unreachable!(),
            NodeKind::DocumentFragment { .. } => doc.create_document_fragment(),
            NodeKind::Text(text) => doc
                .create_text_node_owned(text)
                .map_err(processing_instruction::dom_data_error)?,
            NodeKind::Comment(text) => doc
                .create_comment_owned(text)
                .map_err(processing_instruction::dom_data_error)?,
            NodeKind::ProcessingInstruction { target, data } => doc
                .create_processing_instruction_owned(target, data)
                .map_err(processing_instruction::dom_data_error)?,
            NodeKind::Doctype(value) => doc.create_doctype(value),
            NodeKind::Element(element) => {
                let id = doc.create_element_ns(element.namespace, &element.tag);
                for (key, value) in element.attrs {
                    if let Some(namespace) = element.attr_namespaces.get(&key) {
                        doc.set_attr_ns(id, *namespace, &key, &value);
                    } else {
                        doc.set_attr(id, &key, &value);
                    }
                }
                id
            }
        };
        if id == doc.root {
            return Err(ScriptError::resource("DOM clone storage limit exceeded"));
        }
        Ok(id)
    }

    // Native callbacks, constructors, event dispatch and JSON can recurse into
    // one another. Their retained Rust frames share this weighted guard. Fully
    // iterative JavaScript is bounded by continuation storage and logical calls.
    fn enter_stack(&mut self, units: usize) -> Result<()> {
        if self.stack_units.saturating_add(units) > MAX_STACK_UNITS {
            return Err(ScriptError::resource(
                "combined script nesting limit exceeded",
            ));
        }
        self.stack_units += units;
        Ok(())
    }

    fn json_enter(&mut self) -> Result<()> {
        self.tick()?;
        if self.json_depth >= MAX_DEPTH {
            return Err(ScriptError::resource("JSON nesting limit exceeded"));
        }
        self.enter_stack(1)?;
        self.json_depth += 1;
        Ok(())
    }

    fn json_keys(&mut self, id: usize) -> Result<Vec<JsString>> {
        let object = &self.objects[id];
        let count = object.order.len();
        let bytes = object
            .order
            .iter()
            .map(PropertyKey::byte_len)
            .sum::<usize>();
        self.work(1 + count.saturating_mul(1 + count.checked_ilog2().unwrap_or(0) as usize) / 8)?;
        self.charge(bytes.saturating_add(count.saturating_mul(32)))?;
        let mut keys: Vec<_> = self.objects[id]
            .order
            .iter()
            .filter(|key| {
                self.objects[id]
                    .values
                    .get(*key)
                    .is_some_and(|p| p.enumerable)
            })
            .filter_map(PropertyKey::as_string)
            .cloned()
            .collect();
        keys.sort_by_key(|key| {
            json_array_index(key)
                .map(|index| (false, index))
                .unwrap_or((true, 0))
        });
        Ok(keys)
    }

    fn json_parse(&mut self, input: Value, reviver: Value, doc: &mut Document) -> Result<Value> {
        let text = self.json_text(input, doc, &mut Vec::new())?;
        if text.len() > MAX_STRING {
            return Err(ScriptError::resource("JSON source limit exceeded"));
        }
        self.work(1 + text.len() / 16)?;
        self.charge(text.len().saturating_mul(2))?;
        let revive = json_callable(&reviver);
        let mut reader = JsonReader {
            source: text.units(),
            at: 0,
            tokens: 0,
            record: revive,
        };
        let (value, record) = reader.value(self)?;
        reader.whitespace();
        if reader.at != text.len() {
            return Err(reader.error("unexpected text after JSON value"));
        }
        if !revive {
            return Ok(value);
        }
        let holder = self.object_ordered([(JsString::default(), value)])?;
        self.json_revive(holder, &JsString::default(), &reviver, record.as_ref(), doc)
    }

    fn append_template_text(&mut self, output: &mut Vec<u16>, text: &JsString) -> Result<()> {
        if output.len().saturating_add(text.len()) > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.work(1 + text.len() / 8)?;
        self.charge(text.len().saturating_mul(4))?;
        output.extend_from_slice(text.units());
        Ok(())
    }
    fn string_primitive(&mut self, value: Value, doc: &mut Document) -> Result<Value> {
        let mut primitive = value.clone();
        if let Some(primitive) = self.exotic_primitive(&value, "string", doc)? {
            return Ok(primitive);
        }
        if js_object(&value) {
            for key in ["toString", "valueOf"] {
                self.charge(64)?;
                let method = self.get(value.clone(), key, doc)?;
                if json_callable(&method) {
                    primitive = self.call(method, Vec::new(), value.clone(), doc)?;
                    if json_primitive(&primitive) {
                        break;
                    }
                }
            }
            if !json_primitive(&primitive) {
                return Err(ScriptError::type_error(
                    "value cannot be converted to a primitive string",
                ));
            }
        }
        Ok(primitive)
    }
    fn string_hint(&mut self, value: Value, doc: &mut Document) -> Result<JsString> {
        let primitive = self.string_primitive(value, doc)?;
        if matches!(primitive, Value::Symbol(_)) {
            return Err(ScriptError::type_error(
                "cannot convert a symbol to a string",
            ));
        }
        if !matches!(primitive, Value::String(_)) {
            self.charge(1024)?;
        }
        Ok(primitive.js_string())
    }
    fn json_text(
        &mut self,
        value: Value,
        doc: &mut Document,
        arrays: &mut Vec<usize>,
    ) -> Result<JsString> {
        self.json_enter()?;
        let result = (|| {
            if let Some(primitive) = self.exotic_primitive(&value, "string", doc)? {
                return self.json_text(primitive, doc, arrays);
            }
            match value {
                Value::Symbol(_) => Err(ScriptError::type_error(
                    "cannot convert a symbol to a string",
                )),
                Value::String(text) => {
                    self.charge(text.byte_len())?;
                    Ok(text)
                }
                Value::Number(number) => Ok(json_number(number).into()),
                Value::Object(id) => {
                    for key in ["toString", "valueOf"] {
                        let convert = self.get(Value::Object(id), key, doc)?;
                        if json_callable(&convert) {
                            let converted =
                                self.call(convert, Vec::new(), Value::Object(id), doc)?;
                            if json_primitive(&converted) {
                                return self.json_text(converted, doc, arrays);
                            }
                        }
                    }
                    Err(ScriptError::type_error(
                        "JSON input cannot be converted to a primitive string",
                    ))
                }
                Value::Array(id) => {
                    if arrays.contains(&id) {
                        return Ok(JsString::default());
                    }
                    arrays.push(id);
                    let mut text = Vec::<u16>::new();
                    let length = self.array_lengths[id].value as usize;
                    for index in 0..length {
                        self.tick()?;
                        let key = self.reduce_index_key(index as u64)?;
                        let value = self.get_key(Value::Array(id), &key, doc)?;
                        let part = if matches!(value, Value::Null | Value::Undefined) {
                            JsString::default()
                        } else {
                            self.json_text(value, doc, arrays)?
                        };
                        if text
                            .len()
                            .saturating_add(part.len())
                            .saturating_add(usize::from(index > 0))
                            > MAX_STRING
                        {
                            return Err(ScriptError::resource("JSON source limit exceeded"));
                        }
                        self.charge(part.byte_len() + usize::from(index > 0) * 2)?;
                        if index > 0 {
                            text.push(44);
                        }
                        text.extend_from_slice(part.units());
                    }
                    arrays.pop();
                    Ok(text.into())
                }
                _ => Ok(value.js_string()),
            }
        })();
        self.json_depth -= 1;
        self.stack_units -= 1;
        result
    }

    fn json_revive(
        &mut self,
        holder: Value,
        key: &JsString,
        reviver: &Value,
        record: Option<&JsonRecord>,
        doc: &mut Document,
    ) -> Result<Value> {
        self.json_enter()?;
        let result = (|| {
            let value = self.get_key(holder.clone(), key, doc)?;
            let record = record.filter(|record| json_same_value(&record.value, &value));
            let context = if let Some(source) = record.and_then(|record| record.source.as_ref()) {
                let source = self.string(source.clone())?;
                self.object_ordered([("source".into(), source)])?
            } else {
                self.object(BTreeMap::new())?
            };
            let keys = match value {
                Value::Object(id) => self.json_keys(id)?,
                Value::Array(id) => {
                    let len = self.array_lengths[id].value as usize;
                    self.charge(len.saturating_mul(32))?;
                    (0..len).map(|i| JsString::from(i.to_string())).collect()
                }
                _ => Vec::new(),
            };
            for key in keys {
                let child_record = record.and_then(|record| record.children.get(&key));
                let child = self.json_revive(value.clone(), &key, reviver, child_record, doc)?;
                if child == Value::Undefined {
                    self.delete_property(value.clone(), &key)?;
                } else {
                    self.define_property_key(
                        &value,
                        &PropertyKey::String(key),
                        PropertyDescriptor {
                            value: Some(child),
                            writable: Some(true),
                            enumerable: Some(true),
                            configurable: Some(true),
                            ..PropertyDescriptor::default()
                        },
                        doc,
                    )?;
                }
            }
            let key = self.string(key)?;
            self.call(reviver.clone(), vec![key, value, context], holder, doc)
        })();
        self.json_depth -= 1;
        self.stack_units -= 1;
        result
    }

    fn json_stringify(
        &mut self,
        value: Value,
        replacer: Value,
        space: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let properties = if let Value::Array(id) = replacer {
            let len = self.array_lengths[id].value as usize;
            let mut seen = std::collections::BTreeSet::new();
            let mut keys = Vec::new();
            for index in 0..len {
                self.tick()?;
                let item = self.get(Value::Array(id), &index.to_string(), doc)?;
                let key = match item {
                    Value::String(text) => Some(text),
                    Value::Number(number) => Some(JsString::from(json_number(number))),
                    Value::Object(id)
                        if matches!(
                            self.objects[id].boxed,
                            Some(Value::String(_) | Value::Number(_))
                        ) =>
                    {
                        Some(self.json_text(Value::Object(id), doc, &mut Vec::new())?)
                    }
                    _ => None,
                };
                if let Some(key) = key
                    && !seen.contains(&key)
                {
                    self.charge(96 + key.len().saturating_mul(2))?;
                    seen.insert(key.clone());
                    keys.push(key);
                }
            }
            Some(keys)
        } else {
            None
        };
        let space = if let Value::Object(id) = space {
            match self.objects[id].boxed {
                Some(Value::Number(_)) => Value::Number(self.number_value(Value::Object(id), doc)?),
                Some(Value::String(_)) => {
                    Value::String(self.json_text(Value::Object(id), doc, &mut Vec::new())?)
                }
                _ => Value::Object(id),
            }
        } else {
            space
        };
        let gap = match space {
            Value::Number(n) => JsString::from(" ".repeat(n.clamp(0.0, 10.0) as usize)),
            Value::String(text) => text.slice(0, text.len().min(10)),
            _ => JsString::default(),
        };
        let mut writer = JsonWriter {
            replacer: json_callable(&replacer).then_some(replacer),
            properties,
            gap,
            stack: Vec::new(),
            output: Vec::new(),
        };
        let holder = self.object_ordered([(JsString::default(), value)])?;
        let value = self.json_prepare(holder, &JsString::default(), &writer.replacer, doc)?;
        if self.json_emit(value, &mut writer, 0, doc)? {
            self.string(writer.output)
        } else {
            Ok(Value::Undefined)
        }
    }

    fn json_prepare(
        &mut self,
        holder: Value,
        key: &JsString,
        replacer: &Option<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let mut value = self.get_key(holder.clone(), key, doc)?;
        if !json_primitive(&value) {
            let convert = self.get(value.clone(), "toJSON", doc)?;
            if json_callable(&convert) {
                let key = self.string(key)?;
                value = self.call(convert, vec![key], value, doc)?;
            }
        }
        if let Some(replacer) = replacer {
            let key = self.string(key)?;
            value = self.call(replacer.clone(), vec![key, value], holder, doc)?;
        }
        if let Value::Object(id) = value {
            match self.objects[id].boxed.clone() {
                Some(Value::String(_)) => {
                    value =
                        Value::String(self.json_text(Value::Object(id), doc, &mut Vec::new())?)
                }
                Some(Value::Number(_)) => {
                    value = Value::Number(self.number_value(Value::Object(id), doc)?)
                }
                Some(value @ Value::Bool(_)) => return Ok(value),
                _ => {}
            }
        }
        Ok(value)
    }

    fn json_append(&mut self, writer: &mut JsonWriter, text: &str) -> Result<()> {
        let units: Vec<u16> = text.encode_utf16().collect();
        self.json_append_units(writer, &units)
    }
    fn json_append_units(&mut self, writer: &mut JsonWriter, units: &[u16]) -> Result<()> {
        if writer.output.len().saturating_add(units.len()) > MAX_STRING {
            return Err(ScriptError::resource("JSON output string limit exceeded"));
        }
        self.work(1 + units.len() / 8)?;
        self.charge(units.len().saturating_mul(2))?;
        writer.output.extend_from_slice(units);
        Ok(())
    }
    fn json_quote(&mut self, writer: &mut JsonWriter, text: &JsString) -> Result<()> {
        self.json_append(writer, "\"")?;
        let mut index = 0;
        while let Some(&unit) = text.units().get(index) {
            match unit {
                34 => self.json_append(writer, "\\\"")?,
                92 => self.json_append(writer, "\\\\")?,
                8 => self.json_append(writer, "\\b")?,
                12 => self.json_append(writer, "\\f")?,
                10 => self.json_append(writer, "\\n")?,
                13 => self.json_append(writer, "\\r")?,
                9 => self.json_append(writer, "\\t")?,
                0xd800..=0xdbff
                    if text
                        .units()
                        .get(index + 1)
                        .is_some_and(|low| (0xdc00..=0xdfff).contains(low)) =>
                {
                    self.json_append_units(writer, &text.units()[index..index + 2])?;
                    index += 1;
                }
                0..=31 | 0xd800..=0xdfff => self.json_append(writer, &format!("\\u{unit:04x}"))?,
                _ => self.json_append_units(writer, &[unit])?,
            }
            index += 1;
        }
        self.json_append(writer, "\"")
    }
    fn json_indent(&mut self, writer: &mut JsonWriter, depth: usize) -> Result<()> {
        if !writer.gap.is_empty() {
            self.json_append(writer, "\n")?;
            let gap = writer.gap.clone();
            for _ in 0..depth {
                self.json_append_units(writer, gap.units())?;
            }
        }
        Ok(())
    }
    fn json_emit(
        &mut self,
        value: Value,
        writer: &mut JsonWriter,
        depth: usize,
        doc: &mut Document,
    ) -> Result<bool> {
        self.json_enter()?;
        let result = self.json_emit_inner(value, writer, depth, doc);
        self.json_depth -= 1;
        self.stack_units -= 1;
        result
    }
    fn json_emit_inner(
        &mut self,
        value: Value,
        writer: &mut JsonWriter,
        depth: usize,
        doc: &mut Document,
    ) -> Result<bool> {
        match &value {
            Value::Undefined | Value::Symbol(_) | Value::Function(_) | Value::Native(_) => {
                return Ok(false);
            }
            Value::Null => self.json_append(writer, "null")?,
            Value::Bool(value) => {
                self.json_append(writer, if *value { "true" } else { "false" })?
            }
            Value::Number(number) => self.json_append(
                writer,
                &if number.is_finite() {
                    json_number(*number)
                } else {
                    "null".into()
                },
            )?,
            Value::String(text) => self.json_quote(writer, text)?,
            Value::Array(_) | Value::Object(_) | Value::Json | Value::Math => {
                if writer.stack.contains(&value) {
                    return Err(ScriptError::type_error(
                        "cyclic value cannot be serialized as JSON",
                    ));
                }
                writer.stack.push(value.clone());
                let array = matches!(value, Value::Array(_));
                self.json_append(writer, if array { "[" } else { "{" })?;
                let keys = if let Value::Array(id) = value {
                    let len = self.array_lengths[id].value as usize;
                    self.charge(len.saturating_mul(32))?;
                    (0..len)
                        .map(|i| JsString::from(i.to_string()))
                        .collect::<Vec<_>>()
                } else if let Some(properties) = &writer.properties {
                    self.charge(properties.iter().map(|key| key.len() + 32).sum())?;
                    properties.clone()
                } else if let Value::Object(id) = value {
                    self.json_keys(id)?
                } else {
                    Vec::new()
                };
                let mut count = 0;
                for key in keys {
                    let child = self.json_prepare(value.clone(), &key, &writer.replacer, doc)?;
                    let omitted = matches!(
                        child,
                        Value::Undefined | Value::Symbol(_) | Value::Function(_) | Value::Native(_)
                    );
                    if omitted && !array {
                        continue;
                    }
                    if count > 0 {
                        self.json_append(writer, ",")?;
                    }
                    self.json_indent(writer, depth + 1)?;
                    if !array {
                        self.json_quote(writer, &key)?;
                        self.json_append(writer, ":")?;
                        if !writer.gap.is_empty() {
                            self.json_append(writer, " ")?;
                        }
                    }
                    if omitted {
                        self.json_append(writer, "null")?;
                    } else {
                        self.json_emit(child, writer, depth + 1, doc)?;
                    }
                    count += 1;
                }
                if count > 0 {
                    self.json_indent(writer, depth)?;
                }
                self.json_append(writer, if array { "]" } else { "}" })?;
                writer.stack.pop();
            }
            _ => {
                return Err(ScriptError::type_error(
                    "JSON serialization of host objects is unsupported",
                ));
            }
        }
        Ok(true)
    }

    fn string_find(&mut self, text: &[u16], needle: &[u16], start: usize) -> Result<Option<usize>> {
        if needle.is_empty() {
            return Ok(Some(start.min(text.len())));
        }
        if needle.len() > text.len().saturating_sub(start) {
            return Ok(None);
        }
        for index in start..=text.len() - needle.len() {
            self.work(1 + needle.len() / 8)?;
            if text[index..index + needle.len()] == *needle {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn string_case(&mut self, text: &JsString, upper: bool) -> Result<Value> {
        self.charge(text.byte_len().saturating_mul(3))?;
        let mut output = Vec::new();
        let mut segment = String::new();
        for scalar in char::decode_utf16(text.units().iter().copied()) {
            match scalar {
                Ok(scalar) => segment.push(scalar),
                Err(error) => {
                    let mapped = if upper {
                        segment.to_uppercase()
                    } else {
                        segment.to_lowercase()
                    };
                    output.extend(mapped.encode_utf16());
                    segment.clear();
                    output.push(error.unpaired_surrogate());
                }
            }
            if output.len() > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
        }
        let mapped = if upper {
            segment.to_uppercase()
        } else {
            segment.to_lowercase()
        };
        output.extend(mapped.encode_utf16());
        self.string(output)
    }

    fn regexp_slot(&self, value: &Value) -> Option<Rc<RegExp>> {
        if let Value::Object(id) = value {
            self.objects[*id].regexp.clone()
        } else {
            None
        }
    }
    fn regexp_object(&mut self, pattern: Rc<RegExp>) -> Result<Value> {
        let value = self.object_ordered([])?;
        let Value::Object(id) = value else {
            unreachable!()
        };
        self.objects[id].prototype = Some(Value::Object(self.prototypes["RegExp"]));
        self.objects[id].regexp = Some(pattern);
        self.objects[id].insert_property(
            "lastIndex".into(),
            Property::data(Value::Number(0.0), true, false, false),
        );
        Ok(value)
    }
    fn regexp_budget(&self) -> regexp::Budget {
        regexp::Budget {
            steps: self.steps,
            allocated: 0,
            heap_limit: MAX_HEAP.saturating_sub(self.allocated),
            stack_limit: MAX_STACK_UNITS.saturating_sub(self.stack_units) / 4,
        }
    }
    fn regexp_find(
        &mut self,
        pattern: &RegExp,
        text: &JsString,
        start: usize,
        sticky: bool,
    ) -> Result<Option<regexp::Match>> {
        let mut budget = self.regexp_budget();
        let result = pattern.find(text.units(), start, sticky, &mut budget);
        self.steps = budget.steps;
        self.allocated = self.allocated.saturating_add(budget.allocated);
        result.map_err(regexp_error)
    }
    fn regexp_last_index(&mut self, value: Value, index: usize, doc: &mut Document) -> Result<()> {
        self.write_reference(
            Reference::Property(value, Value::String("lastIndex".into()), true),
            Value::Number(index as f64),
            doc,
        )
    }
    fn regexp_builtin_exec(
        &mut self,
        receiver: Value,
        text: JsString,
        doc: &mut Document,
    ) -> Result<Value> {
        let pattern = self
            .regexp_slot(&receiver)
            .ok_or_else(|| ScriptError::type_error("RegExp receiver lacks pattern slots"))?;
        let index = self.get(receiver.clone(), "lastIndex", doc)?;
        let index =
            integer_or_infinity(self.number_value(index, doc)?).clamp(0.0, 9007199254740991.0);
        let start = if pattern.flags.global || pattern.flags.sticky {
            index as usize
        } else {
            0
        };
        let result = self.regexp_find(&pattern, &text, start, pattern.flags.sticky)?;
        let Some(found) = result else {
            if pattern.flags.global || pattern.flags.sticky {
                self.regexp_last_index(receiver, 0, doc)?;
            }
            return Ok(Value::Null);
        };
        if pattern.flags.global || pattern.flags.sticky {
            self.regexp_last_index(receiver, found.span().1, doc)?;
        }
        self.regexp_match_array(&pattern, &text, &found)
    }
    fn regexp_match_array(
        &mut self,
        pattern: &RegExp,
        text: &JsString,
        found: &regexp::Match,
    ) -> Result<Value> {
        self.charge(pattern.names.len() * std::mem::size_of::<(&JsString, &usize)>())?;
        self.work(
            pattern.names.len() * (pattern.names.len().checked_ilog2().unwrap_or(0) as usize + 1),
        )?;
        let mut names: Vec<_> = pattern.names.iter().collect();
        names.sort_unstable_by_key(|(_, index)| **index);
        let mut captures = Vec::new();
        for span in &found.captures {
            captures.push(match span {
                Some((start, end)) => self.string(&text.units()[*start..*end])?,
                None => Value::Undefined,
            });
        }
        let mut groups = Value::Undefined;
        if !pattern.names.is_empty() {
            groups = self.object_ordered(
                names
                    .iter()
                    .map(|(key, index)| ((*key).clone(), captures[**index].clone())),
            )?;
            if let Value::Object(id) = groups {
                self.objects[id].prototype = None;
            }
        }
        let result = self.array(captures)?;
        let Value::Array(id) = result else {
            unreachable!()
        };
        let properties = self.array_properties[id];
        self.objects[properties].insert("index".into(), Value::Number(found.span().0 as f64));
        self.objects[properties].insert("input".into(), Value::String(text.clone()));
        self.objects[properties].insert("groups".into(), groups);
        if pattern.flags.indices {
            let mut indices = Vec::new();
            for span in &found.captures {
                indices.push(match span {
                    Some((start, end)) => self.array(vec![
                        Value::Number(*start as f64),
                        Value::Number(*end as f64),
                    ])?,
                    None => Value::Undefined,
                });
            }
            let mut groups = Value::Undefined;
            if !pattern.names.is_empty() {
                groups = self.object_ordered(
                    names
                        .iter()
                        .map(|(key, index)| ((*key).clone(), indices[**index].clone())),
                )?;
                if let Value::Object(id) = groups {
                    self.objects[id].prototype = None;
                }
            }
            let indices = self.array(indices)?;
            let Value::Array(index_id) = indices else {
                unreachable!()
            };
            self.objects[self.array_properties[index_id]].insert("groups".into(), groups);
            self.objects[properties].insert("indices".into(), indices);
        }
        Ok(result)
    }
    fn regexp_exec(
        &mut self,
        receiver: Value,
        text: &JsString,
        doc: &mut Document,
    ) -> Result<Value> {
        let exec = self.get(receiver.clone(), "exec", doc)?;
        if json_callable(&exec) {
            self.charge(std::mem::size_of::<Value>())?;
            let result = self.call(exec, vec![Value::String(text.clone())], receiver, doc)?;
            if result != Value::Null && !js_object(&result) {
                return Err(ScriptError::type_error(
                    "RegExp exec must return an object or null",
                ));
            }
            Ok(result)
        } else {
            self.regexp_builtin_exec(receiver, text.clone(), doc)
        }
    }
    fn regexp_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        if name == "species" {
            return Ok(receiver);
        }
        if name == "symbolMatch" {
            return self.regexp_symbol_match(
                receiver,
                args.first().cloned().unwrap_or(Value::Undefined),
                doc,
            );
        }
        if name == "symbolSearch" {
            return self.regexp_symbol_search(
                receiver,
                args.first().cloned().unwrap_or(Value::Undefined),
                doc,
            );
        }
        if name == "symbolSplit" {
            return self.regexp_symbol_split(
                receiver,
                args.first().cloned().unwrap_or(Value::Undefined),
                args.get(1).cloned().unwrap_or(Value::Undefined),
                doc,
            );
        }
        if let Some(key) = name.strip_prefix("get.") {
            if key == "flags" {
                if !js_object(&receiver) {
                    return Err(ScriptError::type_error(
                        "RegExp flags receiver must be an object",
                    ));
                }
                let mut flags = String::new();
                for (key, flag) in [
                    ("hasIndices", 'd'),
                    ("global", 'g'),
                    ("ignoreCase", 'i'),
                    ("multiline", 'm'),
                    ("dotAll", 's'),
                    ("unicode", 'u'),
                    ("unicodeSets", 'v'),
                    ("sticky", 'y'),
                ] {
                    if self.get(receiver.clone(), key, doc)?.truthy() {
                        flags.push(flag);
                    }
                }
                return self.string(flags);
            }
            let pattern = self.regexp_slot(&receiver);
            if pattern.is_none() && receiver == Value::Object(self.prototypes["RegExp"]) {
                return if key == "source" {
                    self.string("(?:)")
                } else {
                    Ok(Value::Undefined)
                };
            }
            let pattern = pattern.ok_or_else(|| {
                ScriptError::type_error("RegExp getter receiver lacks pattern slots")
            })?;
            if key == "source" {
                self.work(pattern.source.len() + 1)?;
                self.charge(pattern.source.byte_len().saturating_mul(6))?;
                return self.string(pattern.escaped_source());
            }
            return Ok(Value::Bool(match key {
                "global" => pattern.flags.global,
                "ignoreCase" => pattern.flags.ignore_case,
                "multiline" => pattern.flags.multiline,
                "dotAll" => pattern.flags.dot_all,
                "sticky" => pattern.flags.sticky,
                "hasIndices" => pattern.flags.indices,
                "unicode" | "unicodeSets" => false,
                _ => unreachable!(),
            }));
        }
        if name == "toString" {
            if !js_object(&receiver) {
                return Err(ScriptError::type_error(
                    "RegExp toString receiver must be an object",
                ));
            }
            let source = self.get(receiver.clone(), "source", doc)?;
            let source = self.json_text(source, doc, &mut Vec::new())?;
            let flags = self.get(receiver, "flags", doc)?;
            let flags = self.json_text(flags, doc, &mut Vec::new())?;
            let len = source.len().saturating_add(flags.len()).saturating_add(2);
            if len > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
            self.charge(len * 2)?;
            let mut result = vec![47];
            result.extend_from_slice(source.units());
            result.push(47);
            result.extend_from_slice(flags.units());
            return self.string(result);
        }
        if !js_object(&receiver) || name == "exec" && self.regexp_slot(&receiver).is_none() {
            return Err(ScriptError::type_error(
                "RegExp method receiver is incompatible",
            ));
        }
        let text = self.json_text(
            args.first().cloned().unwrap_or(Value::Undefined),
            doc,
            &mut Vec::new(),
        )?;
        if name == "exec" {
            self.regexp_builtin_exec(receiver, text, doc)
        } else if name == "test" {
            Ok(Value::Bool(
                self.regexp_exec(receiver, &text, doc)? != Value::Null,
            ))
        } else {
            Err(ScriptError::unsupported("RegExp method is not implemented"))
        }
    }

    fn advance_regexp(
        &mut self,
        receiver: Value,
        text: &JsString,
        unicode: bool,
        doc: &mut Document,
    ) -> Result<()> {
        let index = self.get(receiver.clone(), "lastIndex", doc)?;
        let index = integer_or_infinity(self.number_value(index, doc)?)
            .clamp(0.0, 9007199254740991.0) as usize;
        let pair = unicode
            && text
                .units()
                .get(index)
                .is_some_and(|u| (0xd800..=0xdbff).contains(u))
            && text
                .units()
                .get(index.saturating_add(1))
                .is_some_and(|u| (0xdc00..=0xdfff).contains(u));
        self.regexp_last_index(
            receiver,
            index.saturating_add(if pair { 2 } else { 1 }),
            doc,
        )
    }
    fn string_replace(
        &mut self,
        text: JsString,
        receiver: Value,
        argument: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let regex = self.regexp_slot(&receiver);
        let plain_needle = if regex.is_none() {
            Some(self.json_text(receiver.clone(), doc, &mut Vec::new())?)
        } else {
            None
        };
        let replacement = if json_callable(&argument) {
            None
        } else {
            Some(self.json_text(argument.clone(), doc, &mut Vec::new())?)
        };
        let mut results = Vec::new();
        if regex.is_some() {
            let (global, unicode) = self.regexp_iteration_flags(receiver.clone(), doc)?;
            if global {
                self.regexp_last_index(receiver.clone(), 0, doc)?;
            }
            loop {
                self.tick()?;
                let result = self.regexp_exec(receiver.clone(), &text, doc)?;
                if result == Value::Null {
                    break;
                }
                self.charge(std::mem::size_of::<Value>())?;
                results.push(result.clone());
                if !global {
                    break;
                }
                let matched = self.get(result, "0", doc)?;
                if self.json_text(matched, doc, &mut Vec::new())?.is_empty() {
                    self.advance_regexp(receiver.clone(), &text, unicode, doc)?;
                }
            }
        } else {
            let needle = plain_needle.unwrap();
            if let Some(index) = self.string_find(text.units(), needle.units(), 0)? {
                let result = self.array(vec![Value::String(needle)])?;
                if let Value::Array(id) = result {
                    self.objects[self.array_properties[id]]
                        .insert("index".into(), Value::Number(index as f64));
                }
                results.push(result);
            }
        }
        let mut output = Vec::new();
        let mut next = 0;
        for result in results {
            let length = self.get(result.clone(), "length", doc)?;
            let length =
                integer_or_infinity(self.number_value(length, doc)?).clamp(0.0, 65537.0) as usize;
            if length > 65536 {
                return Err(ScriptError::resource(
                    "RegExp capture result length limit exceeded",
                ));
            }
            let matched = self.get(result.clone(), "0", doc)?;
            let matched = self.json_text(matched, doc, &mut Vec::new())?;
            let position = self.get(result.clone(), "index", doc)?;
            let position = integer_or_infinity(self.number_value(position, doc)?)
                .clamp(0.0, text.len() as f64) as usize;
            let mut captures = Vec::new();
            self.charge(length.saturating_mul(std::mem::size_of::<Value>()))?;
            for index in 1..length {
                let capture = self.get(result.clone(), &index.to_string(), doc)?;
                captures.push(if capture == Value::Undefined {
                    Value::Undefined
                } else {
                    Value::String(self.json_text(capture, doc, &mut Vec::new())?)
                });
            }
            let groups = self.get(result, "groups", doc)?;
            let replacement = if let Some(replacement) = &replacement {
                self.regexp_substitution(
                    replacement,
                    (&matched, position),
                    &text,
                    &captures,
                    groups,
                    doc,
                )?
            } else {
                let mut args = vec![Value::String(matched.clone())];
                args.extend(captures);
                args.push(Value::Number(position as f64));
                args.push(Value::String(text.clone()));
                if groups != Value::Undefined {
                    args.push(groups);
                }
                let result = self.call(argument.clone(), args, Value::Undefined, doc)?;
                self.json_text(result, doc, &mut Vec::new())?
            };
            if position >= next {
                self.regexp_output(&mut output, &text.units()[next..position])?;
                self.regexp_output(&mut output, replacement.units())?;
                next = position.saturating_add(matched.len()).min(text.len());
            }
        }
        self.regexp_output(&mut output, &text.units()[next..])?;
        self.string(output)
    }
    fn regexp_iteration_flags(
        &mut self,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<(bool, bool)> {
        let flags = self.get(receiver, "flags", doc)?;
        let flags = self.string_hint(flags, doc)?;
        self.work(flags.len().saturating_mul(3))?;
        Ok((
            flags.units().contains(&u16::from(b'g')),
            flags.units().contains(&u16::from(b'u')) || flags.units().contains(&u16::from(b'v')),
        ))
    }
    fn regexp_output(&mut self, output: &mut Vec<u16>, addition: &[u16]) -> Result<()> {
        if output.len().saturating_add(addition.len()) > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.work(1 + addition.len() / 8)?;
        self.charge(addition.len().saturating_mul(4))?;
        output.extend_from_slice(addition);
        Ok(())
    }
    fn regexp_substitution(
        &mut self,
        replacement: &JsString,
        match_at: (&JsString, usize),
        text: &JsString,
        captures: &[Value],
        groups: Value,
        doc: &mut Document,
    ) -> Result<JsString> {
        let (matched, position) = match_at;
        let groups = if groups != Value::Undefined {
            self.coerce_object(groups)?
        } else {
            groups
        };
        let mut output = Vec::new();
        let units = replacement.units();
        let mut at = 0;
        while at < units.len() {
            self.tick()?;
            if units[at] != 36 || at + 1 == units.len() {
                self.regexp_output(&mut output, &units[at..at + 1])?;
                at += 1;
                continue;
            }
            let next = units[at + 1];
            let mut consumed = 2;
            match next {
                36 => self.regexp_output(&mut output, &[36])?,
                38 => self.regexp_output(&mut output, matched.units())?,
                96 => self.regexp_output(&mut output, &text.units()[..position])?,
                39 => self.regexp_output(
                    &mut output,
                    &text.units()[position.saturating_add(matched.len()).min(text.len())..],
                )?,
                48..=57 => {
                    let mut index = (next - 48) as usize;
                    if let Some(second @ 48..=57) = units.get(at + 2) {
                        let combined = index * 10 + (*second - 48) as usize;
                        if combined > 0 && combined <= captures.len() {
                            index = combined;
                            consumed = 3;
                        }
                    }
                    if index > 0 && index <= captures.len() {
                        if let Value::String(capture) = &captures[index - 1] {
                            self.regexp_output(&mut output, capture.units())?;
                        }
                    } else {
                        self.regexp_output(&mut output, &[36])?;
                        consumed = 1;
                    }
                }
                60 if groups != Value::Undefined => {
                    self.work(units.len() - at - 2)?;
                    let end = units[at + 2..]
                        .iter()
                        .position(|unit| *unit == 62)
                        .map(|index| at + 2 + index);
                    if let Some(end) = end {
                        let key = JsString::from(&units[at + 2..end]);
                        let value = self.get_key(groups.clone(), &key, doc)?;
                        if value != Value::Undefined {
                            let value = self.json_text(value, doc, &mut Vec::new())?;
                            self.regexp_output(&mut output, value.units())?;
                        }
                        consumed = end - at + 1;
                    } else {
                        self.regexp_output(&mut output, &[36])?;
                        consumed = 1;
                    }
                }
                _ => {
                    self.regexp_output(&mut output, &[36])?;
                    consumed = 1;
                }
            }
            at += consumed;
        }
        Ok(output.into())
    }
    fn style_property_name(&mut self, name: &str) -> Result<Option<String>> {
        self.work(name.len().saturating_add(1))?;
        if name.len() > 256 || name.starts_with("--") {
            return Ok(None);
        }
        self.charge(64 + name.len().saturating_mul(4))?;
        let property = if name == "cssFloat" {
            "float".to_owned()
        } else {
            css_name(name)
        };
        Ok(crate::cssom::recognized_property(&property).then_some(property))
    }

    fn style_source(&mut self, text: &JsString) -> Result<String> {
        self.work(text.len().saturating_add(1))?;
        if text.len() > crate::cssom::MAX_INLINE_BYTES {
            return Err(ScriptError::resource("inline style string limit exceeded"));
        }
        self.charge(24 + text.len().saturating_mul(6))?;
        // The CSSOMString boundary consistently uses USVString, as CSS.supports
        // does. JavaScript keys and strings themselves remain UTF-16.
        let text = text.to_utf8_lossy();
        if text.len() > crate::cssom::MAX_INLINE_BYTES {
            return Err(ScriptError::resource("inline style string limit exceeded"));
        }
        Ok(text)
    }

    fn style_parse(
        &mut self,
        id: NodeId,
        extra_bytes: usize,
        doc: &Document,
    ) -> Result<crate::cssom::InlineStyle> {
        let source = doc.attr(id, "style").unwrap_or("");
        self.charge(crate::cssom::scratch_bytes(
            source.len().saturating_add(extra_bytes),
        ))?;
        crate::cssom::InlineStyle::parse(source, &mut self.steps).map_err(style_limit)
    }

    fn style_commit(
        &mut self,
        id: NodeId,
        style: &crate::cssom::InlineStyle,
        doc: &mut Document,
    ) -> Result<()> {
        let source = style.serialize(&mut self.steps).map_err(style_limit)?;
        self.work(source.len().saturating_add(1))?;
        self.charge(source.len().saturating_add(24))?;
        doc.set_attr(id, "style", &source);
        Ok(())
    }

    fn style_get(&mut self, id: NodeId, key: &str, doc: &mut Document) -> Result<Value> {
        self.work(key.len().saturating_add(1))?;
        if let Some(method) = style_method(key) {
            self.charge(
                std::mem::size_of::<Native>() + 2 * std::mem::size_of::<usize>() + method.len(),
            )?;
            return Ok(Self::native(method, Value::Undefined));
        }
        if key == "parentRule" {
            return Ok(Value::Null);
        }
        if matches!(key, "cssText" | "length") || style_index(key).is_some() {
            let style = self.style_parse(id, key.len(), doc)?;
            if key == "length" {
                return Ok(Value::Number(style.len() as f64));
            }
            if let Some(index) = style_index(key) {
                return if index < style.len() {
                    self.string(style.item(index))
                } else {
                    Ok(Value::Undefined)
                };
            }
            let source = style.serialize(&mut self.steps).map_err(style_limit)?;
            return self.string(source);
        }
        if let Some(property) = self.style_property_name(key)? {
            let style = self.style_parse(id, property.len(), doc)?;
            let value = style
                .get_property_value(&property, &mut self.steps)
                .map_err(style_limit)?;
            return self.string(value);
        }
        self.charge(24 + key.len().saturating_mul(2))?;
        Ok(self
            .lookup_property(&Value::Style(id), &key.into(), doc)?
            .unwrap_or(Value::Undefined))
    }

    fn style_has(&mut self, id: NodeId, key: &JsString, doc: &mut Document) -> Result<bool> {
        self.work(key.len().saturating_add(1))?;
        self.charge(24 + key.len().saturating_mul(6))?;
        if let Ok(name) = key.to_utf8() {
            if style_method(&name).is_some()
                || matches!(name.as_str(), "cssText" | "length" | "parentRule")
                || self.style_property_name(&name)?.is_some()
            {
                return Ok(true);
            }
            if let Some(index) = style_index(&name) {
                return Ok(index < self.style_parse(id, name.len(), doc)?.len());
            }
        }
        Ok(self
            .find_property_in(&Value::Style(id), key, doc)?
            .is_some())
    }

    fn style_set(
        &mut self,
        id: NodeId,
        key: &str,
        value: Value,
        strict: bool,
        doc: &mut Document,
    ) -> Result<()> {
        self.work(key.len().saturating_add(1))?;
        if matches!(key, "length" | "parentRule") || style_index(key).is_some() {
            return Self::failed_write(strict);
        }
        if key == "cssText" {
            let value = self.string_hint(value, doc)?;
            let source = self.style_source(&value)?;
            self.charge(crate::cssom::scratch_bytes(source.len()))?;
            let style =
                crate::cssom::InlineStyle::parse(&source, &mut self.steps).map_err(style_limit)?;
            return self.style_commit(id, &style, doc);
        }
        let Some(property) = self.style_property_name(key)? else {
            return Err(ScriptError::unsupported(
                "inline style expando or method replacement is not implemented",
            ));
        };
        let value = if value == Value::Null {
            JsString::default()
        } else {
            self.string_hint(value, doc)?
        };
        let value = self.style_source(&value)?;
        // Author conversion may have changed this node's style. Parse only now.
        let mut style = self.style_parse(id, property.len().saturating_add(value.len()), doc)?;
        if style
            .set_property(&property, &value, "", &mut self.steps)
            .map_err(style_limit)?
        {
            self.style_commit(id, &style, doc)?;
        }
        Ok(())
    }

    fn style_native(
        &mut self,
        method: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let Value::Style(id) = receiver else {
            return Err(ScriptError::type_error(
                "CSSStyleDeclaration receiver required",
            ));
        };
        let required = if method == "setProperty" { 2 } else { 1 };
        if args.len() < required {
            return Err(ScriptError::type_error(
                "not enough CSSStyleDeclaration arguments",
            ));
        }
        if method == "item" {
            let index = to_i32(self.number_value(args[0].clone(), doc)?) as u32 as usize;
            let style = self.style_parse(id, 0, doc)?;
            return self.string(style.item(index));
        }
        let name = self.string_hint(args[0].clone(), doc)?;
        // Web IDL conversion completes left-to-right before the CSS operation,
        // including value/priority for an unsupported or malformed property.
        let mut value = JsString::default();
        let mut priority = JsString::default();
        if method == "setProperty" {
            if args[1] != Value::Null {
                value = self.string_hint(args[1].clone(), doc)?;
            }
            if let Some(argument) = args.get(2)
                && !matches!(argument, Value::Undefined | Value::Null)
            {
                priority = self.string_hint(argument.clone(), doc)?;
            }
        }
        let name = self.style_source(&name)?;
        let value = self.style_source(&value)?;
        let priority = self.style_source(&priority)?;
        let extra = name
            .len()
            .saturating_add(value.len())
            .saturating_add(priority.len());
        let mut style = self.style_parse(id, extra, doc)?;
        match method {
            "getPropertyValue" => {
                let value = style
                    .get_property_value(&name, &mut self.steps)
                    .map_err(style_limit)?;
                self.string(value)
            }
            "getPropertyPriority" => {
                let value = style
                    .get_property_priority(&name, &mut self.steps)
                    .map_err(style_limit)?;
                self.string(value)
            }
            "removeProperty" => {
                let previous_length = style.len();
                let previous = style
                    .remove_property(&name, &mut self.steps)
                    .map_err(style_limit)?;
                // Allocate the result before the one final DOM mutation.
                let result = self.string(previous)?;
                if style.len() != previous_length {
                    self.style_commit(id, &style, doc)?;
                }
                Ok(result)
            }
            "setProperty" => {
                if style
                    .set_property(&name, &value, &priority, &mut self.steps)
                    .map_err(style_limit)?
                {
                    self.style_commit(id, &style, doc)?;
                }
                Ok(Value::Undefined)
            }
            _ => Err(ScriptError::unsupported(
                "unknown CSSStyleDeclaration operation",
            )),
        }
    }

    fn css_supports(&mut self, args: &[Value], doc: &mut Document) -> Result<Value> {
        let Some(first) = args.first() else {
            return Err(ScriptError::type_error(
                "CSS.supports requires at least one argument",
            ));
        };
        // Web IDL selects the overload by argument count, then converts its
        // arguments left to right. An invalid property must not skip the value
        // conversion, and additional arguments are not converted by this API.
        let first = self.string_hint(first.clone(), doc)?;
        let second = args
            .get(1)
            .map(|value| self.string_hint(value.clone(), doc))
            .transpose()?;
        let first = self.cssom_source(&first)?;
        let second = second
            .as_ref()
            .map(|text| self.cssom_source(text))
            .transpose()?;
        let (Some(first), second) = (first, second) else {
            return Ok(Value::Bool(false));
        };
        if matches!(second, Some(None)) {
            return Ok(Value::Bool(false));
        }
        let second = second.flatten();
        let bytes = first
            .len()
            .saturating_add(second.as_ref().map_or(0, String::len));
        if bytes > crate::css::MAX_SUPPORTS_BYTES {
            return Ok(Value::Bool(false));
        }
        // Covers token-vector growth, normalized strings and nested selector
        // scratch, proportional to this input rather than the maximum query.
        self.charge(bytes.saturating_mul(256).saturating_add(4096))?;
        let supported = if let Some(second) = second {
            crate::css::supports_declaration_with_budget(&first, &second, &mut self.steps)
        } else {
            crate::css::supports_matches_with_budget(&first, true, &mut self.steps)
        };
        if self.steps == 0 {
            return Err(ScriptError::resource("script instruction limit exceeded"));
        }
        Ok(Value::Bool(supported))
    }

    fn cssom_source(&mut self, text: &JsString) -> Result<Option<String>> {
        // CSSOM permits either DOMString or USVString. This boundary chooses
        // USVString: paired surrogates survive, isolated ones become U+FFFD.
        // All author conversions have already completed before size rejection.
        self.work(text.len().saturating_add(1))?;
        if text.len() > crate::css::MAX_SUPPORTS_BYTES {
            return Ok(None);
        }
        self.charge(text.len().saturating_mul(3).saturating_add(24))?;
        let source = text.to_utf8_lossy();
        Ok((source.len() <= crate::css::MAX_SUPPORTS_BYTES).then_some(source))
    }

    fn uri_value(&mut self, text: &JsString, mode: js_uri::Mode) -> Result<Value> {
        // Two linear traversals validate/size, then fill the exact result.
        // The visitor has bounded stack scratch and no heap allocation.
        self.work(1 + text.len().saturating_mul(2))?;
        let mut length = 0usize;
        js_uri::visit(text.units(), mode, |units| {
            length = length.saturating_add(units.len());
            if length > MAX_STRING {
                Err(js_uri::Error::OutputLimit)
            } else {
                Ok(())
            }
        })
        .map_err(uri_error)?;
        self.work(1 + length.saturating_mul(2) / 8)?;
        self.charge(64 + length.saturating_mul(4))?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("URI result allocation failed"))?;
        js_uri::visit(text.units(), mode, |units| {
            output.extend_from_slice(units);
            Ok(())
        })
        .map_err(uri_error)?;
        Ok(Value::String(JsString::from(output)))
    }

    fn native_call(
        &mut self,
        native: &Native,
        args: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        if dom_prototypes::is_interface_name(&native.name) {
            self.dom_interface_exists(&native.name)?;
            return Err(ScriptError::type_error(
                "DOM interface constructor requires new",
            ));
        }
        if let Some(method) = native.name.strip_prefix(text_operations::PREFIX) {
            return self.text_operations_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native.name.strip_prefix(document_title::PREFIX) {
            return self.document_title_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native.name.strip_prefix(node_data::PREFIX) {
            return self.node_data_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native.name.strip_prefix(processing_instruction::PREFIX) {
            return self.pi_native(method, native.receiver.clone(), &args, doc);
        }
        if native.name == "Object.is" {
            return self.object_is(&args);
        }
        if native.name == "DataView" {
            return Err(ScriptError::type_error("DataView requires new"));
        }
        if let Some(method) = native.name.strip_prefix("DataView.") {
            return self.data_view_native(method, native.receiver.clone(), &args, doc);
        }
        if native.name == "ArrayBuffer" {
            return Err(ScriptError::type_error("ArrayBuffer requires new"));
        }
        if let Some(method) = native.name.strip_prefix("ArrayBuffer.") {
            return self.array_buffer_native(method, native.receiver.clone(), &args, doc);
        }
        if native.name == "Array.from" {
            return self.array_from(native.receiver.clone(), &args, doc);
        }
        if native.name == "Array.splice" {
            return self.array_splice(native.receiver.clone(), &args, doc);
        }
        if native.name == "Array.concat" {
            return self.array_concat(native.receiver.clone(), &args, doc);
        }
        if native.name == "Date" {
            let now = self.date_now()?;
            return self.date_format(now, "toString");
        }
        if let Some(method) = native.name.strip_prefix("Iterator.") {
            return self.iterator_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native.name.strip_prefix("Date.") {
            return self.date_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(operation) = object_integrity::IntegrityOperation::from_name(&native.name) {
            return self.object_integrity(
                args.first().cloned().unwrap_or(Value::Undefined),
                operation,
                doc,
            );
        }
        if let Some(method) = native.name.strip_prefix("DOM.") {
            return self.dom_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native.name.strip_prefix("DOMTokenList.") {
            return self.token_list_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(mode) = js_uri::Mode::from_name(&native.name) {
            let text = self.string_hint(args.first().cloned().unwrap_or(Value::Undefined), doc)?;
            return self.uri_value(&text, mode);
        }
        if let Some(predicate) = NumberPredicate::from_name(&native.name) {
            self.work(4)?;
            // No coercion, argument-content scan, receiver lookup or allocation.
            return Ok(Value::Bool(predicate.test(args.first())));
        }
        if matches!(native.name.as_str(), "Number" | "isFinite" | "isNaN") {
            self.tick()?;
            let number = if native.name == "Number" && args.is_empty() {
                0.0
            } else {
                self.number_value(args.first().cloned().unwrap_or(Value::Undefined), doc)?
            };
            return Ok(match native.name.as_str() {
                "Number" => Value::Number(number),
                "isFinite" => Value::Bool(number.is_finite()),
                _ => Value::Bool(number.is_nan()),
            });
        }
        if matches!(native.name.as_str(), "parseInt" | "parseFloat") {
            self.tick()?;
            let text = self.string_hint(args.first().cloned().unwrap_or(Value::Undefined), doc)?;
            let radix = if native.name == "parseInt" {
                to_i32(self.number_value(args.get(1).cloned().unwrap_or(Value::Undefined), doc)?)
            } else {
                0
            };
            if native.name == "parseInt" && radix != 0 && !(2..=36).contains(&radix) {
                return Ok(Value::Number(f64::NAN));
            }
            // UTF-16 decoding, UTF-8 capacity growth, whitespace/prefix scans
            // and numeric parsing are bounded by the converted first string.
            // Non-ASCII scalars and replacement characters terminate the same
            // ASCII numeric prefix, including an isolated input surrogate.
            self.work(1 + text.len())?;
            self.charge(32 + text.len().saturating_mul(6))?;
            let text = text.to_utf8_lossy();
            return Ok(Value::Number(if native.name == "parseInt" {
                parse_int(&text, f64::from(radix))
            } else {
                parse_float(&text)
            }));
        }
        if let Some(method) = native.name.strip_prefix("Array.")
            && matches!(
                method,
                "push"
                    | "unshift"
                    | "pop"
                    | "shift"
                    | "join"
                    | "includes"
                    | "indexOf"
                    | "slice"
                    | "forEach"
                    | "every"
                    | "some"
                    | "find"
                    | "findIndex"
                    | "findLast"
                    | "findLastIndex"
                    | "map"
                    | "filter"
            )
        {
            return self.array_method(native.receiver.clone(), method, args, doc);
        }
        if native.name == "Array.lastIndexOf" {
            return self.array_last_index_of(
                native.receiver.clone(),
                args.first().cloned().unwrap_or(Value::Undefined),
                args.get(1).cloned(),
                doc,
            );
        }
        if matches!(native.name.as_str(), "Array.reduce" | "Array.reduceRight") {
            return self.array_reduce(
                native.receiver.clone(),
                args.first().cloned().unwrap_or(Value::Undefined),
                args.get(1).cloned(),
                if native.name == "Array.reduce" {
                    ReduceDirection::Left
                } else {
                    ReduceDirection::Right
                },
                doc,
            );
        }
        if native.name == "Array.sort" {
            return self.array_sort(
                native.receiver.clone(),
                args.first().cloned().unwrap_or(Value::Undefined),
                doc,
            );
        }
        if matches!(native.name.as_str(), "Window.get.self" | "Window.set.self") {
            self.tick()?;
            if !matches!(
                native.receiver,
                Value::Window | Value::Null | Value::Undefined
            ) {
                return Err(ScriptError::type_error("self accessor requires a Window"));
            }
            if native.name == "Window.get.self" {
                return Ok(Value::Window);
            }
            let value = args.first().cloned().unwrap_or(Value::Undefined);
            let key = self.global_key(TrackedGlobal::WindowSelf);
            if !self.define_own(
                &Value::Window,
                &key,
                PropertyDescriptor::data_property(value, true, true, true),
            )? {
                return Err(ScriptError::type_error("cannot replace Window.self"));
            }
            return Ok(Value::Undefined);
        }
        if native.name == "Object.isPrototypeOf" {
            // Only identities and prototype links are examined, not argument contents.
            return self.object_is_prototype_of_in(
                native.receiver.clone(),
                args.first().cloned().unwrap_or(Value::Undefined),
                doc,
            );
        }
        if let Some(method) = native.name.strip_prefix("CSSStyleDeclaration.") {
            return self.style_native(method, native.receiver.clone(), &args, doc);
        }
        if native.name == "CSS.supports" {
            return self.css_supports(&args, doc);
        }
        if native.name.starts_with("AbortController.") || native.name.starts_with("AbortSignal.") {
            return self.abort_native(&native.name, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native
            .name
            .strip_prefix("Event.")
            .or_else(|| native.name.strip_prefix("CustomEvent."))
            .or_else(|| native.name.strip_prefix("ToggleEvent."))
        {
            return self.event_native(method, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = native.name.strip_prefix("EventTarget.") {
            return self.event_target_native(method, native.receiver.clone(), &args, doc);
        }
        if matches!(
            native.name.as_str(),
            "Event"
                | "CustomEvent"
                | "ToggleEvent"
                | "EventTarget"
                | "DOMException"
                | "AbortController"
                | "AbortSignal"
        ) {
            return Err(ScriptError::type_error("DOM constructor requires new"));
        }
        if native.name == "createEvent" {
            if native.receiver != Value::Document {
                return Err(ScriptError::type_error("createEvent requires a Document"));
            }
            let Some(kind) = args.first() else {
                return Err(ScriptError::type_error("createEvent requires an interface"));
            };
            let kind = self
                .json_text(kind.clone(), doc, &mut Vec::new())?
                .to_utf8_lossy()
                .to_ascii_lowercase();
            if !matches!(
                kind.as_str(),
                "event" | "events" | "htmlevents" | "customevent"
            ) {
                let value = self.dom_exception(
                    "NotSupportedError".into(),
                    "event interface is not supported".into(),
                )?;
                return Err(self.thrown_error(value)?);
            }
            let event = self.event_object(
                "".into(),
                false,
                false,
                false,
                Value::Null,
                kind == "customevent",
            )?;
            let id = self.event_index(&event)?;
            self.events[id].initialized = false;
            return Ok(event);
        }
        if matches!(native.name.as_str(), "String.match" | "String.search") {
            return self.string_match_search(
                native.name.strip_prefix("String.").unwrap(),
                native.receiver.clone(),
                args.first().cloned().unwrap_or(Value::Undefined),
                doc,
            );
        }
        if native.name == "String.concat" {
            return self.string_concat(native.receiver.clone(), &args, doc);
        }
        if native.name == "String.split"
            && let Some(result) = self.string_split_hook(native.receiver.clone(), &args, doc)?
        {
            return Ok(result);
        }
        let normalized;
        let native = if let Some(method) = native.name.strip_prefix("String.")
            && !matches!(method, "toString" | "valueOf")
        {
            let receiver = if matches!(method, "fromCharCode" | "fromCodePoint") {
                self.alloc_native("String", Value::Window)?
            } else {
                if matches!(native.receiver, Value::Null | Value::Undefined) {
                    return Err(ScriptError::type_error(
                        "String method receiver is null or undefined",
                    ));
                }
                Value::String(self.string_hint(native.receiver.clone(), doc)?)
            };
            normalized = Native {
                properties: native.properties,
                name: method.into(),
                receiver,
            };
            &normalized
        } else if let Some(method) = native.name.strip_prefix("Array.")
            && !matches!(method, "isArray" | "toString" | "reverse")
        {
            if !matches!(native.receiver, Value::Array(_)) {
                return Err(ScriptError::unsupported(
                    "generic Array method receivers are not implemented",
                ));
            }
            normalized = Native {
                properties: native.properties,
                name: method.into(),
                receiver: native.receiver.clone(),
            };
            &normalized
        } else if let Some(method) = native.name.strip_prefix("JSON.") {
            normalized = Native {
                properties: native.properties,
                name: method.into(),
                receiver: Value::Json,
            };
            &normalized
        } else if let Some(method) = native.name.strip_prefix("Math.") {
            normalized = Native {
                properties: native.properties,
                name: method.into(),
                receiver: Value::Math,
            };
            &normalized
        } else {
            native
        };
        let name = native.name.as_str();
        if name == "Symbol" || name.starts_with("Symbol.") {
            return self.symbol_native(name, native.receiver.clone(), &args, doc);
        }
        if let Some(method) = name.strip_prefix("RegExp.") {
            return self.regexp_native(method, native.receiver.clone(), &args, doc);
        }
        if name == "RegExp" && native.receiver == Value::Window {
            return self.regexp_constructor(
                args.first().cloned().unwrap_or(Value::Undefined),
                args.get(1).cloned().unwrap_or(Value::Undefined),
                None,
                doc,
            );
        }
        if name == "ThrowTypeError" {
            return Err(ScriptError::type_error(
                "restricted function or arguments property",
            ));
        }
        if name == "eval" {
            return Err(ScriptError::unsupported("dynamic eval is not implemented"));
        }
        let mut units = 1usize;
        for value in args.iter().chain(std::iter::once(&native.receiver)) {
            units = units.saturating_add(match value {
                Value::String(text) => text.len() / 16,
                Value::Array(id) => self.arrays[*id].len() / 8,
                _ => 0,
            });
        }
        self.work(units)?;
        let arg = |index: usize| args.get(index).cloned().unwrap_or(Value::Undefined);
        match name {
            "Array.reverse" => return self.array_reverse(native.receiver.clone(), doc),
            "Array.toString" => {
                let object = self.coerce_object(native.receiver.clone())?;
                let join = self.get(object.clone(), "join", doc)?;
                return if json_callable(&join) {
                    self.call(join, Vec::new(), object, doc)
                } else {
                    self.native_call(
                        &Native {
                            properties: None,
                            name: "Object.toString".into(),
                            receiver: object,
                        },
                        Vec::new(),
                        doc,
                    )
                };
            }
            "Array.isArray" => return Ok(Value::Bool(matches!(arg(0), Value::Array(_)))),
            "Function.bind" => {
                let target = native.receiver.clone();
                if !json_callable(&target) {
                    return Err(ScriptError::type_error("bind receiver is not callable"));
                }
                let length = self.get(target.clone(), "length", doc)?;
                let length = if let Value::Number(number) = length {
                    integer_or_infinity(number).max(0.0)
                } else {
                    0.0
                };
                let name = self.get(target.clone(), "name", doc)?;
                let mut units = JsString::from("bound ").units().to_vec();
                if let Value::String(text) = name {
                    units.extend_from_slice(text.units());
                }
                let name = self.string(units)?;
                let bound_args: Vec<_> = args.iter().skip(1).cloned().collect();
                self.charge(
                    192 + bound_args
                        .len()
                        .saturating_mul(std::mem::size_of::<Value>()),
                )?;
                let Value::Object(properties) = self.object_ordered([
                    ("name".into(), name),
                    (
                        "length".into(),
                        Value::Number((length - bound_args.len() as f64).max(0.0)),
                    ),
                ])?
                else {
                    unreachable!()
                };
                self.objects[properties].prototype = self.prototype_of(&target);
                self.objects[properties].attributes("name", false, false, true);
                self.objects[properties].attributes("length", false, false, true);
                let id = self.functions.len();
                self.functions.push(Function {
                    code: self.functions[self.function_prototype].code.clone(),
                    environment: 0,
                    properties,
                    bound: Some(BoundFunction {
                        target,
                        receiver: arg(0),
                        arguments: bound_args,
                    }),
                });
                return Ok(Value::Function(id));
            }
            "Reflect.apply" => {
                let target = arg(0);
                if !json_callable(&target) {
                    return Err(ScriptError::type_error(
                        "Reflect.apply target is not callable",
                    ));
                }
                let arguments = self.argument_list(arg(2), doc)?;
                return self.call(target, arguments, arg(1), doc);
            }
            "Reflect.construct" => {
                let target = arg(0);
                if !self.is_constructor(target.clone())? {
                    return Err(ScriptError::type_error(
                        "Reflect.construct target is not a constructor",
                    ));
                }
                let new_target = if args.len() < 3 {
                    target.clone()
                } else {
                    arg(2)
                };
                if !self.is_constructor(new_target.clone())? {
                    return Err(ScriptError::type_error(
                        "Reflect.construct newTarget is not a constructor",
                    ));
                }
                let arguments = self.argument_list(arg(1), doc)?;
                return self.construct_with_target(target, arguments, new_target, doc);
            }
            "Function.hasInstance" => {
                return self
                    .instance_of(arg(0), native.receiver.clone(), true, doc)
                    .map(Value::Bool);
            }
            "Function.call" | "Function.apply" => {
                if !json_callable(&native.receiver) {
                    return Err(ScriptError::type_error("function receiver is not callable"));
                }
                let receiver = arg(0);
                let parameters = if name == "Function.call" {
                    args.iter().skip(1).cloned().collect()
                } else if matches!(arg(1), Value::Null | Value::Undefined) {
                    Vec::new()
                } else {
                    self.argument_list(arg(1), doc)?
                };
                return self.call(native.receiver.clone(), parameters, receiver, doc);
            }
            "Object.toString" => {
                if !matches!(native.receiver, Value::Null | Value::Undefined) {
                    let key = self.well_known_key("toStringTag");
                    if let Value::String(tag) =
                        self.get_property_key(native.receiver.clone(), &key, doc)?
                    {
                        let length = tag.len().saturating_add(9);
                        if length > MAX_STRING {
                            return Err(ScriptError::resource("object tag string limit exceeded"));
                        }
                        self.work(1 + length / 8)?;
                        self.charge(length.saturating_mul(4) + 32)?;
                        let mut units = Vec::new();
                        units
                            .try_reserve_exact(length)
                            .map_err(|_| ScriptError::resource("object tag allocation failed"))?;
                        units.extend("[object ".encode_utf16());
                        units.extend_from_slice(tag.units());
                        units.push(u16::from(b']'));
                        return Ok(Value::String(units.into()));
                    }
                }
                let tag = match &native.receiver {
                    Value::Undefined => "Undefined",
                    Value::Null => "Null",
                    Value::Array(_) => "Array",
                    Value::Function(_) | Value::Native(_) => "Function",
                    Value::String(_) => "String",
                    Value::Number(_) => "Number",
                    Value::Bool(_) => "Boolean",
                    Value::Object(id) if self.objects[*id].arguments => "Arguments",
                    Value::Object(id) if self.objects[*id].regexp.is_some() => "RegExp",
                    Value::Object(id) if self.objects[*id].date_value.is_some() => "Date",
                    Value::Object(id) if self.objects[*id].event.is_some() => {
                        let state = &self.events[self.objects[*id].event.unwrap()];
                        if state.toggle.is_some() {
                            "ToggleEvent"
                        } else if state.custom {
                            "CustomEvent"
                        } else {
                            "Event"
                        }
                    }
                    Value::Object(id)
                        if matches!(self.objects[*id].abort, Some(AbortSlot::Controller(_))) =>
                    {
                        "AbortController"
                    }
                    Value::Object(id)
                        if matches!(self.objects[*id].abort, Some(AbortSlot::Signal(_))) =>
                    {
                        "AbortSignal"
                    }
                    Value::Object(id) if self.objects[*id].event_target => "EventTarget",
                    Value::Object(id) if self.objects[*id].namespace.is_some() => {
                        self.objects[*id].namespace.unwrap()
                    }
                    Value::Object(id) => match self.objects[*id].boxed {
                        Some(Value::String(_)) => "String",
                        Some(Value::Number(_)) => "Number",
                        Some(Value::Bool(_)) => "Boolean",
                        _ => "Object",
                    },
                    _ => "Object",
                };
                return self.string(format!("[object {tag}]"));
            }
            "Object.valueOf" => return self.coerce_object(native.receiver.clone()),
            "Object.hasOwnProperty" | "Object.propertyIsEnumerable" => {
                let key = self.property_key(arg(0), doc)?;
                let object = self.coerce_object(native.receiver.clone())?;
                if self.property_object(&object).is_none()
                    && object != Value::Window
                    && dom_own_properties::host(&object).is_none()
                    && matches!(key, PropertyKey::String(_))
                {
                    return Err(ScriptError::unsupported(
                        "host own-property reflection is not implemented",
                    ));
                }
                let property = if object == Value::Window
                    && let Some(key) = key.as_string()
                {
                    self.window_reflected_property(key)?
                } else {
                    self.read_own_property_key(&object, &key)?
                };
                return Ok(Value::Bool(if name.ends_with("propertyIsEnumerable") {
                    property.is_some_and(|p| p.enumerable)
                } else {
                    property.is_some()
                }));
            }
            "Object.defineProperty" | "Reflect.defineProperty" => {
                let object = arg(0);
                if !js_object(&object) {
                    return Err(ScriptError::type_error(
                        "defineProperty target must be an object",
                    ));
                }
                let key = self.property_key(arg(1), doc)?;
                let desc = self.property_descriptor(arg(2), doc)?;
                let defined = self.define_property_key(&object, &key, desc, doc)?;
                if name == "Reflect.defineProperty" {
                    return Ok(Value::Bool(defined));
                }
                if !defined {
                    return Err(ScriptError::type_error("incompatible property definition"));
                }
                return Ok(object);
            }
            "Object.defineProperties" => {
                let object = arg(0);
                if !js_object(&object) {
                    return Err(ScriptError::type_error(
                        "defineProperties target must be an object",
                    ));
                }
                self.define_properties(object.clone(), arg(1), doc)?;
                return Ok(object);
            }
            "Object.getOwnPropertyDescriptor" => {
                let object = self.coerce_object(arg(0))?;
                let key = self.property_key(arg(1), doc)?;
                let property = if matches!(object, Value::Window)
                    && let Some(key) = key.as_string()
                {
                    self.window_reflected_property(key)?
                } else {
                    if self.property_object(&object).is_none()
                        && dom_own_properties::host(&object).is_none()
                        && matches!(key, PropertyKey::String(_))
                    {
                        return Err(ScriptError::unsupported(
                            "host own-property reflection is not implemented",
                        ));
                    }
                    self.work(1 + key.byte_len() / 16)?;
                    self.read_own_property_key(&object, &key)?
                };
                let Some(property) = property else {
                    return Ok(Value::Undefined);
                };
                let mut fields = match property.value {
                    PropertyValue::Data { value, writable } => vec![
                        ("value".into(), value),
                        ("writable".into(), Value::Bool(writable)),
                    ],
                    PropertyValue::Accessor { get, set } => {
                        vec![("get".into(), get), ("set".into(), set)]
                    }
                };
                fields.push(("enumerable".into(), Value::Bool(property.enumerable)));
                fields.push(("configurable".into(), Value::Bool(property.configurable)));
                return self.object_ordered(fields);
            }
            "Object.preventExtensions" | "Object.isExtensible" => {
                let object = arg(0);
                if !js_object(&object) {
                    return Ok(if name.ends_with("isExtensible") {
                        Value::Bool(false)
                    } else {
                        object
                    });
                }
                let id = self.property_object(&object).ok_or_else(|| {
                    ScriptError::unsupported("host extensibility is not implemented")
                })?;
                if name.ends_with("isExtensible") {
                    return Ok(Value::Bool(!self.objects[id].non_extensible));
                }
                self.objects[id].non_extensible = true;
                return Ok(object);
            }
            "Object.create" => {
                let prototype = arg(0);
                if !js_object(&prototype) && prototype != Value::Null {
                    return Err(ScriptError::type_error(
                        "object prototype must be an object or null",
                    ));
                }
                let object = self.object_ordered([])?;
                let Value::Object(id) = object else {
                    unreachable!()
                };
                self.objects[id].prototype = if prototype == Value::Null {
                    None
                } else {
                    Some(prototype)
                };
                if arg(1) != Value::Undefined {
                    self.define_properties(object.clone(), arg(1), doc)?;
                }
                return Ok(object);
            }
            "Object.getPrototypeOf" => {
                let object = self.coerce_object(arg(0))?;
                return Ok(self.prototype_of_in(&object, doc)?.unwrap_or(Value::Null));
            }
            "Object.setPrototypeOf" => {
                let object = arg(0);
                let prototype = arg(1);
                if matches!(object, Value::Null | Value::Undefined) {
                    return Err(ScriptError::type_error(
                        "cannot set null or undefined prototype",
                    ));
                }
                if !js_object(&prototype) && prototype != Value::Null {
                    return Err(ScriptError::type_error(
                        "object prototype must be an object or null",
                    ));
                }
                if !js_object(&object) {
                    return Ok(object);
                }
                self.set_object_prototype_in(&object, prototype, doc)?;
                return Ok(object);
            }
            "Object.getOwnPropertySymbols" | "Reflect.ownKeys" => {
                let keys = if name == "Reflect.ownKeys" {
                    let target = arg(0);
                    if !js_object(&target) {
                        return Err(ScriptError::type_error(
                            "Reflect.ownKeys target must be an object",
                        ));
                    }
                    self.own_property_keys(&target)?
                } else {
                    let target = self.coerce_object(arg(0))?;
                    self.own_symbol_keys(&target)?
                };
                self.charge(keys.len().saturating_mul(std::mem::size_of::<Value>()) + 32)?;
                let mut values = Vec::new();
                values
                    .try_reserve_exact(keys.len())
                    .map_err(|_| ScriptError::resource("property key array allocation failed"))?;
                values.extend(keys.iter().map(PropertyKey::value));
                return self.array(values);
            }
            "Object.keys" | "Object.values" | "Object.getOwnPropertyNames" => {
                let object = self.coerce_object(arg(0))?;
                let keys = self.own_keys(&object)?;
                let mut result = self.own_key_values(keys.len())?;
                for key in keys {
                    self.tick()?;
                    if name != "Object.getOwnPropertyNames"
                        && !self
                            .read_own_property(&object, &key)?
                            .is_some_and(|p| p.enumerable)
                    {
                        continue;
                    }
                    result.push(if name == "Object.values" {
                        self.get_key(object.clone(), &key, doc)?
                    } else {
                        self.string(key)?
                    });
                }
                return self.array(result);
            }
            "Error.toString" => {
                if !js_object(&native.receiver) {
                    return Err(ScriptError::type_error(
                        "Error.toString receiver is not an object",
                    ));
                }
                let name = self.get(native.receiver.clone(), "name", doc)?;
                let message = self.get(native.receiver.clone(), "message", doc)?;
                let name = if name == Value::Undefined {
                    JsString::from("Error")
                } else {
                    self.json_text(name, doc, &mut Vec::new())?
                };
                let message = if message == Value::Undefined {
                    JsString::default()
                } else {
                    self.json_text(message, doc, &mut Vec::new())?
                };
                let mut result = name.units().to_vec();
                if !name.is_empty() && !message.is_empty() {
                    result.extend_from_slice(&[58, 32]);
                }
                result.extend_from_slice(message.units());
                return self.string(result);
            }
            "String.toString" | "String.valueOf" | "Number.toString" | "Number.valueOf"
            | "Boolean.toString" | "Boolean.valueOf" => {
                let value = match &native.receiver {
                    Value::Object(id) => self.objects[*id].boxed.clone().ok_or_else(|| {
                        ScriptError::type_error("receiver lacks boxed primitive data")
                    })?,
                    value if !js_object(value) => value.clone(),
                    _ => {
                        return Err(ScriptError::type_error(
                            "receiver lacks boxed primitive data",
                        ));
                    }
                };
                let matches_brand = matches!(
                    (&value, name.split('.').next()),
                    (Value::String(_), Some("String"))
                        | (Value::Number(_), Some("Number"))
                        | (Value::Bool(_), Some("Boolean"))
                );
                if !matches_brand {
                    return Err(ScriptError::type_error(
                        "incompatible primitive method receiver",
                    ));
                }
                if name == "Number.toString"
                    && let Value::Number(number) = value
                {
                    return self.number_to_string(number, arg(0), doc);
                }
                return if name.ends_with("valueOf") {
                    Ok(value)
                } else {
                    self.string(value.js_string())
                };
            }
            _ => {}
        }
        if native.receiver == Value::Window {
            if name == "Function" {
                let target = self.alloc_native("Function", Value::Window)?;
                return self.dynamic_function(args, target, doc);
            }
            if name.ends_with("Error") && self.native_properties.contains_key(name) {
                let message = if matches!(arg(0), Value::Undefined) {
                    Value::Undefined
                } else {
                    let text = self.json_text(arg(0), doc, &mut Vec::new())?;
                    self.string(text)?
                };
                return self.error_object(name, message);
            }
            if name == "Object" {
                return if matches!(arg(0), Value::Null | Value::Undefined) {
                    self.object_ordered([])
                } else {
                    self.coerce_object(arg(0))
                };
            }
            if name == "Array" {
                if args.len() == 1
                    && let Value::Number(length) = arg(0)
                {
                    if !length.is_finite()
                        || length.fract() != 0.0
                        || !(0.0..=u32::MAX as f64).contains(&length)
                    {
                        return Err(ScriptError::range_error("invalid array length"));
                    }
                    let array = self.array(Vec::new())?;
                    let Value::Array(id) = array else {
                        unreachable!()
                    };
                    self.array_lengths[id].value = length as u32;
                    return Ok(array);
                }
                return self.array(args);
            }
        }
        match &native.receiver {
            Value::Json if name == "parse" => {
                return self.json_parse(arg(0), arg(1), doc);
            }
            Value::Json if name == "stringify" => {
                return self.json_stringify(arg(0), arg(1), arg(2), doc);
            }
            Value::Window if ["String", "Boolean"].contains(&name) => {
                let value = arg(0);
                return match name {
                    "String" => {
                        let text = if args.is_empty() {
                            JsString::default()
                        } else if let Value::Symbol(symbol) = value {
                            self.symbol_text(&symbol)?
                        } else {
                            self.json_text(value, doc, &mut Vec::new())?
                        };
                        self.string(text)
                    }
                    "Boolean" => Ok(Value::Bool(value.truthy())),
                    _ => unreachable!(),
                };
            }
            Value::Console => {
                let mut line = String::new();
                for (index, value) in args.iter().enumerate() {
                    let text = self.display_value(value)?;
                    if line.len().saturating_add(text.len()).saturating_add(1) > MAX_STRING {
                        return Err(ScriptError::resource(
                            "console message length limit exceeded",
                        ));
                    }
                    if index > 0 {
                        line.push(' ');
                    }
                    line.push_str(&text);
                }
                self.charge(line.len() + 24)?;
                if self.console.len() >= 2048 {
                    return Err(ScriptError::resource("console message limit exceeded"));
                }
                self.console.push(line);
                return Ok(Value::Undefined);
            }
            Value::Math => {
                if matches!(name, "min" | "max") {
                    let minimum = name == "min";
                    let mut result = if minimum {
                        f64::INFINITY
                    } else {
                        f64::NEG_INFINITY
                    };
                    for value in args {
                        let number = self.number_value(value.clone(), doc)?;
                        if number.is_nan() || result.is_nan() {
                            result = f64::NAN;
                        } else if (minimum && number < result)
                            || (!minimum && number > result)
                            || (number == 0.0
                                && result == 0.0
                                && (number.is_sign_negative() == minimum))
                        {
                            result = number;
                        }
                    }
                    return Ok(Value::Number(result));
                }
                let a = self.number_value(arg(0), doc)?;
                let value = match name {
                    "abs" => a.abs(),
                    "floor" => a.floor(),
                    "ceil" => a.ceil(),
                    "round" => {
                        if (-0.5..0.0).contains(&a) {
                            -0.0
                        } else if a.fract() == 0.0 || !a.is_finite() {
                            a
                        } else {
                            (a + 0.5).floor()
                        }
                    }
                    "trunc" => a.trunc(),
                    "sqrt" => a.sqrt(),
                    "pow" => a.powf(self.number_value(arg(1), doc)?),
                    "sin" => a.sin(),
                    "cos" => a.cos(),
                    "tan" => a.tan(),
                    "log" => a.ln(),
                    "exp" => a.exp(),
                    "sign" => {
                        if a == 0.0 || a.is_nan() {
                            a
                        } else {
                            a.signum()
                        }
                    }
                    _ => return Err(ScriptError::new("unsupported Math method")),
                };
                return Ok(Value::Number(value));
            }
            Value::String(text) => {
                if name == "replace" {
                    return self.string_replace(text.clone(), arg(0), arg(1), doc);
                }
                if matches!(name, "includes" | "startsWith" | "endsWith")
                    && self.is_regexp(arg(0), doc)?
                {
                    return Err(ScriptError::type_error(
                        "String search argument must not be a RegExp",
                    ));
                }
                let needle = if matches!(
                    name,
                    "includes" | "indexOf" | "lastIndexOf" | "startsWith" | "endsWith"
                ) {
                    self.string_hint(arg(0), doc)?
                } else {
                    JsString::default()
                };
                let units = text.units();
                let len = text.len();
                match name {
                    "toUpperCase" | "toLowerCase" => {
                        return self.string_case(text, name == "toUpperCase");
                    }
                    "trim" => return self.string(text.trimmed_units()),
                    "toString" => return Ok(native.receiver.clone()),
                    "includes" | "indexOf" => {
                        let start = integer_or_infinity(self.number_value(arg(1), doc)?)
                            .clamp(0.0, len as f64) as usize;
                        let found = self.string_find(units, needle.units(), start)?;
                        return Ok(if name == "includes" {
                            Value::Bool(found.is_some())
                        } else {
                            Value::Number(found.map(|i| i as f64).unwrap_or(-1.0))
                        });
                    }
                    "lastIndexOf" => {
                        // ToNumber is observable even for an empty search or
                        // one longer than the receiver. NaN starts at the end.
                        let position = self.number_value(arg(1), doc)?;
                        let start = if position.is_nan() {
                            len
                        } else {
                            integer_or_infinity(position).clamp(0.0, len as f64) as usize
                        };
                        let found = self.string_rfind(units, needle.units(), start)?;
                        return Ok(Value::Number(found.map(|i| i as f64).unwrap_or(-1.0)));
                    }
                    "startsWith" => {
                        let start = integer_or_infinity(self.number_value(arg(1), doc)?)
                            .clamp(0.0, len as f64) as usize;
                        self.work(1 + needle.len() / 8)?;
                        return Ok(Value::Bool(units[start..].starts_with(needle.units())));
                    }
                    "endsWith" => {
                        let end = if matches!(arg(1), Value::Undefined) {
                            len
                        } else {
                            integer_or_infinity(self.number_value(arg(1), doc)?)
                                .clamp(0.0, len as f64) as usize
                        };
                        self.work(1 + needle.len() / 8)?;
                        return Ok(Value::Bool(units[..end].ends_with(needle.units())));
                    }
                    "charAt" | "charCodeAt" | "codePointAt" => {
                        let position = integer_or_infinity(self.number_value(arg(0), doc)?);
                        if position < 0.0 || position >= len as f64 {
                            return if name == "charAt" {
                                self.string("")
                            } else {
                                Ok(if name == "charCodeAt" {
                                    Value::Number(f64::NAN)
                                } else {
                                    Value::Undefined
                                })
                            };
                        }
                        let index = position as usize;
                        let unit = units[index];
                        if name == "charAt" {
                            return self.string(vec![unit]);
                        }
                        let point = if name == "codePointAt"
                            && (0xd800..=0xdbff).contains(&unit)
                            && units
                                .get(index + 1)
                                .is_some_and(|low| (0xdc00..=0xdfff).contains(low))
                        {
                            0x10000
                                + ((unit as u32 - 0xd800) << 10)
                                + (units[index + 1] as u32 - 0xdc00)
                        } else {
                            unit as u32
                        };
                        return Ok(Value::Number(point as f64));
                    }
                    "slice" | "substring" => {
                        let mut start = if name == "slice" {
                            relative_index(self.number_value(arg(0), doc)?, len)
                        } else {
                            integer_or_infinity(self.number_value(arg(0), doc)?)
                                .clamp(0.0, len as f64) as usize
                        };
                        let mut end = if matches!(arg(1), Value::Undefined) {
                            len
                        } else if name == "slice" {
                            relative_index(self.number_value(arg(1), doc)?, len)
                        } else {
                            integer_or_infinity(self.number_value(arg(1), doc)?)
                                .clamp(0.0, len as f64) as usize
                        };
                        if name == "substring" && start > end {
                            std::mem::swap(&mut start, &mut end);
                        }
                        return self.string(&units[start..end.max(start)]);
                    }
                    "split" => {
                        let limit = if matches!(arg(1), Value::Undefined) {
                            u32::MAX as usize
                        } else {
                            to_i32(self.number_value(arg(1), doc)?) as u32 as usize
                        };
                        // ToUint32(limit) precedes separator conversion, even for zero.
                        let needle = self.string_hint(arg(0), doc)?;
                        if limit == 0 {
                            return self.array(Vec::new());
                        }
                        if matches!(arg(0), Value::Undefined) {
                            return self.array(vec![native.receiver.clone()]);
                        }
                        let mut values = Vec::new();
                        if needle.is_empty() {
                            for &unit in units.iter().take(limit) {
                                if values.len() >= 65536 {
                                    return Err(ScriptError::resource(
                                        "array length limit exceeded",
                                    ));
                                }
                                values.push(self.string(vec![unit])?);
                            }
                        } else {
                            let mut start = 0;
                            while values.len() < limit {
                                if values.len() >= 65536 {
                                    return Err(ScriptError::resource(
                                        "array length limit exceeded",
                                    ));
                                }
                                if let Some(index) =
                                    self.string_find(units, needle.units(), start)?
                                {
                                    values.push(self.string(&units[start..index])?);
                                    start = index + needle.len();
                                } else {
                                    values.push(self.string(&units[start..])?);
                                    break;
                                }
                            }
                        }
                        return self.array(values);
                    }
                    _ => {}
                }
            }
            Value::Native(constructor)
                if constructor.name == "String"
                    && matches!(name, "fromCharCode" | "fromCodePoint") =>
            {
                let mut units = Vec::new();
                for value in &args {
                    self.tick()?;
                    let number = self.number_value(value.clone(), doc)?;
                    if name == "fromCharCode" {
                        units.push((to_i32(number) as u32 & 0xffff) as u16);
                    } else {
                        if !number.is_finite()
                            || number.fract() != 0.0
                            || !(0.0..=0x10ffff as f64).contains(&number)
                        {
                            return Err(ScriptError::range_error("invalid Unicode code point"));
                        }
                        let point = number as u32;
                        if point <= 0xffff {
                            units.push(point as u16);
                        } else {
                            let point = point - 0x10000;
                            units.push(0xd800 + (point >> 10) as u16);
                            units.push(0xdc00 + (point & 0x3ff) as u16);
                        }
                    }
                    if units.len() > MAX_STRING {
                        return Err(ScriptError::resource("script string limit exceeded"));
                    }
                }
                return self.string(units);
            }
            Value::Number(_) | Value::Bool(_) if name == "toString" => {
                if let Value::Number(number) = native.receiver {
                    return self.number_to_string(number, arg(0), doc);
                }
                return self.string(native.receiver.js_string());
            }
            _ => {}
        }
        Err(ScriptError::new(format!(
            "unsupported native method '{name}'"
        )))
    }
}

impl JsonReader<'_> {
    fn error(&self, message: &str) -> ScriptError {
        ScriptError {
            message: format!("{message} at UTF-16 offset {}", self.at),
            offset: None,
            kind: ErrorKind::Runtime("SyntaxError"),
            thrown_name: None,
            intrinsic_name: None,
        }
    }
    fn whitespace(&mut self) {
        while self
            .source
            .get(self.at)
            .is_some_and(|u| matches!(*u, 9 | 10 | 13 | 32))
        {
            self.at += 1;
        }
    }
    fn consume(&mut self, byte: u8) -> bool {
        if self.source.get(self.at) == Some(&(byte as u16)) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn starts(&self, text: &str) -> bool {
        self.source
            .get(self.at..self.at + text.len())
            .is_some_and(|units| units.iter().copied().eq(text.bytes().map(u16::from)))
    }
    fn digit(&self) -> bool {
        self.source
            .get(self.at)
            .is_some_and(|u| matches!(*u, 48..=57))
    }
    fn token(&mut self) -> Result<()> {
        self.tokens += 1;
        if self.tokens > MAX_TOKENS {
            Err(ScriptError::resource("JSON token limit exceeded"))
        } else {
            Ok(())
        }
    }
    fn value(&mut self, runtime: &mut Runtime) -> Result<(Value, Option<JsonRecord>)> {
        runtime.json_enter()?;
        let result = self.value_inner(runtime);
        runtime.json_depth -= 1;
        runtime.stack_units -= 1;
        result
    }
    fn value_inner(&mut self, runtime: &mut Runtime) -> Result<(Value, Option<JsonRecord>)> {
        self.whitespace();
        self.token()?;
        let start = self.at;
        let mut children = BTreeMap::new();
        let value = match self.source.get(self.at).copied() {
            Some(34) => {
                let text = self.string()?;
                runtime.string(text)?
            }
            Some(110) if self.starts("null") => {
                self.at += 4;
                Value::Null
            }
            Some(116) if self.starts("true") => {
                self.at += 4;
                Value::Bool(true)
            }
            Some(102) if self.starts("false") => {
                self.at += 5;
                Value::Bool(false)
            }
            Some(45 | 48..=57) => self.number()?,
            Some(91) => {
                self.at += 1;
                self.whitespace();
                let mut values = Vec::new();
                if !self.consume(b']') {
                    loop {
                        let (value, record) = self.value(runtime)?;
                        runtime.charge(64)?;
                        if let Some(record) = record {
                            children.insert(JsString::from(values.len().to_string()), record);
                        }
                        values.push(value);
                        self.whitespace();
                        if self.consume(b']') {
                            break;
                        }
                        if !self.consume(b',') {
                            return Err(self.error("expected ',' or ']' in JSON array"));
                        }
                    }
                }
                runtime.array(values)?
            }
            Some(123) => {
                self.at += 1;
                self.whitespace();
                let mut entries = Vec::new();
                if !self.consume(b'}') {
                    loop {
                        self.whitespace();
                        self.token()?;
                        let key = self.string()?;
                        runtime.charge(160 + key.byte_len().saturating_mul(2))?;
                        self.whitespace();
                        if !self.consume(b':') {
                            return Err(self.error("expected ':' in JSON object"));
                        }
                        let (value, record) = self.value(runtime)?;
                        if let Some(record) = record {
                            children.insert(key.clone(), record);
                        }
                        entries.push((key, value));
                        self.whitespace();
                        if self.consume(b'}') {
                            break;
                        }
                        if !self.consume(b',') {
                            return Err(self.error("expected ',' or '}' in JSON object"));
                        }
                    }
                }
                runtime.object_ordered(entries)?
            }
            _ => return Err(self.error("expected JSON value")),
        };
        let record = if self.record {
            runtime.charge(192)?;
            let source = if json_primitive(&value) {
                runtime.charge((self.at - start) * 2 + 24)?;
                Some(JsString::from(&self.source[start..self.at]))
            } else {
                None
            };
            Some(JsonRecord {
                value: value.clone(),
                source,
                children,
            })
        } else {
            None
        };
        Ok((value, record))
    }
    fn string(&mut self) -> Result<JsString> {
        if !self.consume(b'"') {
            return Err(self.error("expected quoted JSON string"));
        }
        let mut units = Vec::new();
        while let Some(&unit) = self.source.get(self.at) {
            self.at += 1;
            match unit {
                34 => return Ok(units.into()),
                92 => {
                    let escape = self
                        .source
                        .get(self.at)
                        .copied()
                        .ok_or_else(|| self.error("unfinished JSON escape"))?;
                    self.at += 1;
                    units.push(match escape {
                        34 | 92 | 47 => escape,
                        98 => 8,
                        102 => 12,
                        110 => 10,
                        114 => 13,
                        116 => 9,
                        117 => self.hex_unit()?,
                        _ => return Err(self.error("invalid JSON escape")),
                    });
                }
                0..=31 => return Err(self.error("unescaped control character in JSON string")),
                _ => units.push(unit),
            }
        }
        Err(self.error("unterminated JSON string"))
    }
    fn hex_unit(&mut self) -> Result<u16> {
        let mut value = 0;
        for _ in 0..4 {
            let unit = self
                .source
                .get(self.at)
                .copied()
                .ok_or_else(|| self.error("unfinished JSON Unicode escape"))?;
            let digit = match unit {
                48..=57 => unit - 48,
                65..=70 => unit - 55,
                97..=102 => unit - 87,
                _ => return Err(self.error("invalid JSON Unicode escape")),
            };
            value = value * 16 + digit;
            self.at += 1;
        }
        Ok(value)
    }
    fn number(&mut self) -> Result<Value> {
        let start = self.at;
        self.consume(b'-');
        if !self.consume(b'0') {
            if !self
                .source
                .get(self.at)
                .is_some_and(|u| matches!(*u, 49..=57))
            {
                return Err(self.error("invalid JSON number"));
            }
            while self.digit() {
                self.at += 1;
            }
        }
        if self.consume(b'.') {
            let digits = self.at;
            while self.digit() {
                self.at += 1;
            }
            if digits == self.at {
                return Err(self.error("expected fractional digit in JSON number"));
            }
        }
        if self.consume(b'e') || self.consume(b'E') {
            if !self.consume(b'+') {
                self.consume(b'-');
            }
            let digits = self.at;
            while self.digit() {
                self.at += 1;
            }
            if digits == self.at {
                return Err(self.error("expected exponent digit in JSON number"));
            }
        }
        let text: String = self.source[start..self.at]
            .iter()
            .map(|u| char::from(*u as u8))
            .collect();
        Ok(Value::Number(
            text.parse()
                .map_err(|_| self.error("invalid JSON number"))?,
        ))
    }
}

fn json_callable(value: &Value) -> bool {
    matches!(value, Value::Function(_) | Value::Native(_))
}
fn intrinsic_error_type(name: &str) -> Option<&'static str> {
    [
        "Error",
        "TypeError",
        "SyntaxError",
        "ReferenceError",
        "RangeError",
        "EvalError",
        "URIError",
    ]
    .into_iter()
    .find(|candidate| *candidate == name)
}
fn js_object(value: &Value) -> bool {
    !json_primitive(value)
}
fn json_primitive(value: &Value) -> bool {
    matches!(
        value,
        Value::Undefined
            | Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Symbol(_)
    )
}
fn json_same_value(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
        }
        _ => left == right,
    }
}
fn json_array_index(key: &JsString) -> Option<u32> {
    let units = key.units();
    if units.is_empty() || units.len() > 10 || units.len() > 1 && units[0] == u16::from(b'0') {
        return None;
    }
    let mut index = 0u32;
    for &unit in units {
        let digit = unit.checked_sub(u16::from(b'0'))?;
        if digit > 9 {
            return None;
        }
        index = index.checked_mul(10)?.checked_add(u32::from(digit))?;
    }
    (index != u32::MAX).then_some(index)
}
// Rust supplies shortest round-trippable digits; ECMAScript chooses decimal
// notation for exponents -6 through 20 and an explicit '+' for positive ones.
fn json_number(number: f64) -> String {
    if !number.is_finite() {
        return Value::Number(number).to_string();
    }
    if number == 0.0 {
        return "0".into();
    }
    let raw = number.abs().to_string();
    let (mantissa, exponent) = raw
        .split_once(['e', 'E'])
        .map(|(m, e)| (m, e.parse::<i32>().unwrap_or(0)))
        .unwrap_or((&raw, 0));
    let decimal = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let leading = digits.bytes().take_while(|byte| *byte == b'0').count();
    let digits = digits[leading..].trim_end_matches('0');
    let position = decimal + exponent - leading as i32;
    let power = position - 1;
    let mut output = if number.is_sign_negative() {
        "-".to_owned()
    } else {
        String::new()
    };
    if (-6..21).contains(&power) {
        if position <= 0 {
            output.push_str("0.");
            output.push_str(&"0".repeat((-position) as usize));
            output.push_str(digits);
        } else if position as usize >= digits.len() {
            output.push_str(digits);
            output.push_str(&"0".repeat(position as usize - digits.len()));
        } else {
            output.push_str(&digits[..position as usize]);
            output.push('.');
            output.push_str(&digits[position as usize..]);
        }
    } else {
        output.push_str(&digits[..1]);
        if digits.len() > 1 {
            output.push('.');
            output.push_str(&digits[1..]);
        }
        output.push('e');
        if power >= 0 {
            output.push('+');
        }
        output.push_str(&power.to_string());
    }
    output
}

fn to_i32(number: f64) -> i32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    number.trunc().rem_euclid(4294967296.0) as u32 as i32
}
fn relative_index(number: f64, len: usize) -> usize {
    let number = integer_or_infinity(number);
    if number.is_nan() {
        0
    } else if number < 0.0 {
        (len as f64 + number.trunc()).max(0.0) as usize
    } else {
        number.min(len as f64) as usize
    }
}
fn integer_or_infinity(number: f64) -> f64 {
    if number.is_nan() { 0.0 } else { number.trunc() }
}
fn parse_float(text: &str) -> f64 {
    let text =
        text.trim_start_matches(|c: char| u16::try_from(c as u32).is_ok_and(is_js_whitespace));
    if text.starts_with("Infinity") || text.starts_with("+Infinity") {
        return f64::INFINITY;
    }
    if text.starts_with("-Infinity") {
        return f64::NEG_INFINITY;
    }
    let bytes = text.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let digits_start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut digits = end - digits_start;
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        let fractional_start = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        digits += end - fractional_start;
    }
    if digits == 0 {
        return f64::NAN;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let exponent_start = end;
        end += 1;
        if matches!(bytes.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let exponent_digits = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == exponent_digits {
            end = exponent_start;
        }
    }
    text[..end].parse().unwrap_or(f64::NAN)
}
fn parse_int(text: &str, radix: f64) -> f64 {
    let text =
        text.trim_start_matches(|c: char| u16::try_from(c as u32).is_ok_and(is_js_whitespace));
    let sign = if text.starts_with('-') { -1.0 } else { 1.0 };
    let text = text.strip_prefix(['-', '+']).unwrap_or(text);
    let mut radix = to_i32(radix) as u32;
    let mut text = text;
    if radix == 0 {
        radix = if text.starts_with("0x") || text.starts_with("0X") {
            16
        } else {
            10
        };
    }
    if !(2..=36).contains(&radix) {
        return f64::NAN;
    }
    if radix == 16 {
        text = text
            .strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))
            .unwrap_or(text);
    }
    let count = text
        .bytes()
        .take_while(|byte| (*byte as char).is_digit(radix))
        .count();
    if count == 0 {
        return f64::NAN;
    }
    let digits = &text[..count];
    let number = if radix == 10 {
        digits.parse::<f64>().unwrap_or(f64::INFINITY)
    } else if radix.is_power_of_two() {
        radix_number(digits.as_bytes(), radix.trailing_zeros() as usize)
    } else {
        digits.bytes().fold(0.0, |n, byte| {
            n * radix as f64 + (byte as char).to_digit(radix).unwrap() as f64
        })
    };
    sign * number
}
fn event_handler_name(name: &str) -> bool {
    matches!(
        name,
        "onclick"
            | "ontoggle"
            | "ondblclick"
            | "oninput"
            | "onbeforeinput"
            | "onchange"
            | "onsubmit"
            | "onreset"
            | "onkeydown"
            | "onkeyup"
            | "onkeypress"
            | "onfocus"
            | "onblur"
            | "onfocusin"
            | "onfocusout"
            | "onmousedown"
            | "onmouseup"
            | "onmousemove"
            | "onmouseenter"
            | "onmouseleave"
            | "onmouseover"
            | "onmouseout"
            | "onwheel"
            | "oncontextmenu"
            | "onload"
            | "onerror"
            | "onscroll"
            | "onresize"
            | "onunload"
            | "ontouchstart"
            | "ontouchmove"
            | "ontouchend"
    )
}
fn css_name(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            result.push('-');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
fn style_limit(_: crate::cssom::Error) -> ScriptError {
    ScriptError::resource("inline declaration limit exceeded")
}
fn style_method(key: &str) -> Option<&'static str> {
    match key {
        "getPropertyValue" => Some("CSSStyleDeclaration.getPropertyValue"),
        "getPropertyPriority" => Some("CSSStyleDeclaration.getPropertyPriority"),
        "setProperty" => Some("CSSStyleDeclaration.setProperty"),
        "removeProperty" => Some("CSSStyleDeclaration.removeProperty"),
        "item" => Some("CSSStyleDeclaration.item"),
        _ => None,
    }
}
fn style_index(key: &str) -> Option<usize> {
    if key.is_empty() || key.len() > 10 || key.starts_with('0') && key != "0" {
        return None;
    }
    if !key.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    // Web IDL indexed property names exclude 2^32 - 1.
    key.parse::<u32>()
        .ok()
        .filter(|index| *index != u32::MAX)
        .map(|index| index as usize)
}
fn document_element(doc: &Document) -> Option<NodeId> {
    doc.nodes
        .get(doc.root)?
        .children
        .iter()
        .copied()
        .find(|id| {
            matches!(
                doc.nodes.get(*id).map(|node| &node.kind),
                Some(NodeKind::Element(_))
            )
        })
}
fn html_document_child(doc: &Document, tag: &str) -> Option<NodeId> {
    let html = document_element(doc)?;
    if doc.namespace(html) != Some(Namespace::Html) || doc.tag(html) != Some("html") {
        return None;
    }
    doc.nodes.get(html)?.children.iter().copied().find(|id| {
        doc.namespace(*id) == Some(Namespace::Html)
            && (doc.tag(*id) == Some(tag) || tag == "body" && doc.tag(*id) == Some("frameset"))
    })
}
fn import_node(
    runtime: &mut Runtime,
    doc: &mut Document,
    parent: NodeId,
    source: &Document,
    node: NodeId,
    depth: usize,
) -> Result<()> {
    runtime.tick()?;
    if depth >= 96 {
        return Ok(());
    }
    let original = source
        .nodes
        .get(node)
        .ok_or_else(|| ScriptError::type_error("invalid DOM import source"))?;
    let payload = match &original.kind {
        NodeKind::Text(text) | NodeKind::Comment(text) => text.stored_bytes(),
        NodeKind::ProcessingInstruction { target, data } => target.len() + data.stored_bytes(),
        _ => 0,
    };
    runtime.work(payload)?;
    runtime.charge(payload)?;
    match &original.kind {
        NodeKind::Document => return Ok(()),
        NodeKind::DocumentFragment { .. } => {
            for child in &original.children {
                import_node(runtime, doc, parent, source, *child, depth + 1)?;
            }
            return Ok(());
        }
        _ => {}
    }
    // The caller reserves the complete fragment before creating its staging
    // root. Keep the actual typed growth check for any other reached caller.
    runtime.dom_reserve_node_growth(
        doc,
        1 + usize::from(source.template_contents(node).is_some()),
    )?;
    let id = match &original.kind {
        NodeKind::Document | NodeKind::DocumentFragment { .. } => unreachable!(),
        NodeKind::Text(text) => doc
            .create_text_node_owned(text.clone())
            .map_err(processing_instruction::dom_data_error)?,
        NodeKind::Comment(text) => doc
            .create_comment_owned(text.clone())
            .map_err(processing_instruction::dom_data_error)?,
        NodeKind::ProcessingInstruction { target, data } => doc
            .create_processing_instruction_owned(target.clone(), data.clone())
            .map_err(processing_instruction::dom_data_error)?,
        NodeKind::Doctype(doctype) => doc.create_doctype(doctype.clone()),
        NodeKind::Element(element) => {
            let id = doc.create_element_ns(element.namespace, &element.tag);
            for (key, value) in &element.attrs {
                if let Some(namespace) = element.attr_namespaces.get(key) {
                    doc.set_attr_ns(id, *namespace, key, value);
                } else {
                    doc.set_attr(id, key, value);
                }
            }
            id
        }
    };
    if id == doc.root {
        return Err(ScriptError::resource("DOM import storage limit exceeded"));
    }
    doc.append_child(parent, id);
    for child in &original.children {
        import_node(runtime, doc, id, source, *child, depth + 1)?;
    }
    if let Some(source_contents) = source.template_contents(node)
        && let Some(contents) = doc.template_contents(id)
    {
        for child in &source.nodes[source_contents].children {
            import_node(runtime, doc, contents, source, *child, depth + 1)?;
        }
    }
    Ok(())
}

// A strict streaming scalar sink for this existing String-valued serializer.
// Pending high units cross adjacent Text segments; actual markup separates them.
// Planning and emitting use the identical traversal without author callbacks.
struct ScriptHtml<'a> {
    output: Option<&'a mut String>,
    high: Option<u16>,
    bytes: usize,
    units: usize,
}
impl<'a> ScriptHtml<'a> {
    fn new(output: Option<&'a mut String>) -> Self {
        Self {
            output,
            high: None,
            bytes: 0,
            units: 0,
        }
    }
    fn unit(&mut self, unit: u16) -> Result<()> {
        self.units = self
            .units
            .checked_add(1)
            .filter(|length| *length <= MAX_STRING)
            .ok_or_else(|| ScriptError::resource("HTML serialization string limit exceeded"))?;
        let scalar = if let Some(high) = self.high.take() {
            if !(0xdc00..=0xdfff).contains(&unit) {
                return Err(ScriptError::unsupported(
                    "non-scalar JavaScript HTML serialization is not implemented",
                ));
            }
            char::from_u32(0x10000 + ((u32::from(high) - 0xd800) << 10) + u32::from(unit) - 0xdc00)
                .expect("validated surrogate pair")
        } else if (0xd800..=0xdbff).contains(&unit) {
            self.high = Some(unit);
            return Ok(());
        } else {
            char::from_u32(u32::from(unit)).ok_or_else(|| {
                ScriptError::unsupported(
                    "non-scalar JavaScript HTML serialization is not implemented",
                )
            })?
        };
        self.bytes = self
            .bytes
            .checked_add(scalar.len_utf8())
            .filter(|length| *length <= MAX_STRING)
            .ok_or_else(|| ScriptError::resource("HTML serialization byte limit exceeded"))?;
        if let Some(output) = &mut self.output {
            output.push(scalar);
        }
        Ok(())
    }
    fn text(&mut self, runtime: &mut Runtime, text: &str) -> Result<()> {
        runtime.work(1 + 2 * text.len())?;
        for unit in text.encode_utf16() {
            self.unit(unit)?;
        }
        Ok(())
    }
    fn escaped<I: Iterator<Item = u16>>(
        &mut self,
        runtime: &mut Runtime,
        units: I,
        attribute: bool,
    ) -> Result<()> {
        for unit in units {
            runtime.tick()?;
            match unit {
                38 => self.text(runtime, "&amp;")?,
                60 => self.text(runtime, "&lt;")?,
                62 => self.text(runtime, "&gt;")?,
                34 if attribute => self.text(runtime, "&quot;")?,
                _ => self.unit(unit)?,
            }
        }
        Ok(())
    }
    fn finish(&self) -> Result<()> {
        if self.high.is_some() {
            return Err(ScriptError::unsupported(
                "non-scalar JavaScript HTML serialization is not implemented",
            ));
        }
        Ok(())
    }
}

fn serialize_node(
    runtime: &mut Runtime,
    doc: &Document,
    id: NodeId,
    depth: usize,
    out: &mut ScriptHtml<'_>,
) -> Result<()> {
    runtime.tick()?;
    if depth >= 96 {
        return Ok(());
    }
    let node = doc
        .nodes
        .get(id)
        .ok_or_else(|| ScriptError::type_error("invalid HTML serialization node"))?;
    match &node.kind {
        NodeKind::Text(text) => {
            runtime.work(text.stored_bytes())?;
            out.escaped(runtime, text.units(), false)?;
        }
        NodeKind::Comment(text) => {
            out.text(runtime, "<!--")?;
            runtime.work(1 + 2 * text.stored_bytes())?;
            for unit in text.units() {
                out.unit(unit)?;
            }
            out.text(runtime, "-->")?;
        }
        NodeKind::ProcessingInstruction { target, data } => {
            out.text(runtime, "<?")?;
            out.text(runtime, target)?;
            out.text(runtime, " ")?;
            runtime.work(1 + 2 * data.stored_bytes())?;
            for unit in data.units() {
                out.unit(unit)?;
            }
            out.text(runtime, "?>")?;
        }
        NodeKind::Doctype(doctype) => {
            out.text(runtime, "<!DOCTYPE ")?;
            out.text(runtime, &doctype.name)?;
            out.text(runtime, ">")?;
        }
        NodeKind::Document | NodeKind::DocumentFragment { .. } => {
            serialize_children_at(runtime, doc, id, depth, out)?;
        }
        NodeKind::Element(element) => {
            out.text(runtime, "<")?;
            out.text(runtime, &element.tag)?;
            for (key, value) in &element.attrs {
                out.text(runtime, " ")?;
                out.text(runtime, key)?;
                out.text(runtime, "=\"")?;
                runtime.work(value.len())?;
                out.escaped(runtime, value.encode_utf16(), true)?;
                out.text(runtime, "\"")?;
            }
            out.text(runtime, ">")?;
            if element.namespace != Namespace::Html
                || ![
                    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                    "param", "source", "track", "wbr",
                ]
                .contains(&element.tag.as_str())
            {
                serialize_children_at(runtime, doc, id, depth, out)?;
                out.text(runtime, "</")?;
                out.text(runtime, &element.tag)?;
                out.text(runtime, ">")?;
            }
        }
    }
    Ok(())
}
fn serialize_children_at(
    runtime: &mut Runtime,
    doc: &Document,
    id: NodeId,
    depth: usize,
    out: &mut ScriptHtml<'_>,
) -> Result<()> {
    let id = doc.template_contents(id).unwrap_or(id);
    let node = doc
        .nodes
        .get(id)
        .ok_or_else(|| ScriptError::type_error("invalid HTML serialization node"))?;
    for child in &node.children {
        serialize_node(runtime, doc, *child, depth + 1, out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(source: &str) -> Result<Value> {
        Runtime::new().execute(source, &mut Document::parse("<body></body>"))
    }

    fn exact_dom_data(units: &[u16]) -> crate::dom::DomString {
        crate::dom::DomString::from_units_owned(units.to_vec()).unwrap()
    }

    #[test]
    fn exact_dom_getters_preserve_units_across_nodes_and_normalize_only_line_endings() {
        let mut doc = Document::parse("<title></title><div id=x></div><textarea></textarea>");
        let title = doc.query_selector("title").unwrap();
        let parent = doc.query_selector("#x").unwrap();
        let textarea = doc.query_selector("textarea").unwrap();
        for (owner, slices) in [
            (title, vec![vec![0xd83e], vec![0xdd80, 0xd800]]),
            (parent, vec![vec![0x41, 0xd83e], vec![0xdd80, 0xdc00]]),
            (textarea, vec![vec![0xd800, 13], vec![10, 0x42, 13, 0xdc00]]),
        ] {
            for units in slices {
                let text = doc.create_text_node_owned(exact_dom_data(&units)).unwrap();
                doc.append_child(owner, text);
            }
        }
        let mut runtime = Runtime::new();
        for (receiver, key, expected) in [
            (Value::Document, "title", vec![0xd83e, 0xdd80, 0xd800]),
            (
                Value::Node(parent),
                "textContent",
                vec![0x41, 0xd83e, 0xdd80, 0xdc00],
            ),
            (
                Value::Node(parent),
                "innerText",
                vec![0x41, 0xd83e, 0xdd80, 0xdc00],
            ),
            (
                Value::Node(textarea),
                "value",
                vec![0xd800, 10, 0x42, 10, 0xdc00],
            ),
        ] {
            assert_eq!(
                runtime.get(receiver, key, &mut doc).unwrap(),
                Value::String(expected.into())
            );
        }
        let original = doc
            .text_content_units_bounded(textarea, 32, &mut 128)
            .unwrap();
        assert_eq!(original, [0xd800, 13, 10, 0x42, 13, 0xdc00]);
    }

    #[test]
    fn exact_dom_simple_reads_ignore_unrelated_arena_payloads() {
        let mut doc = Document::parse("<p></p><aside></aside>");
        let parent = doc.query_selector("p").unwrap();
        let empty = doc.query_selector("aside").unwrap();
        let text = doc
            .create_text_node_owned(exact_dom_data(&[0xd800, 0]))
            .unwrap();
        doc.append_child(parent, text);
        for _ in 0..1000 {
            doc.create_comment("unrelated");
        }
        let mut runtime = Runtime::new();
        for id in [text, parent] {
            let before = runtime.allocated;
            assert_eq!(runtime.dom_text_units(id, &doc).unwrap(), [0xd800, 0]);
            assert_eq!(runtime.allocated - before, 64 + 4 * 2);
        }
        let before = runtime.allocated;
        assert!(runtime.dom_text_units(empty, &doc).unwrap().is_empty());
        assert_eq!(runtime.allocated - before, 64);
    }

    #[test]
    fn exact_dom_clone_and_import_preserve_nonscalar_payloads_and_pi_target() {
        let mut doc = Document::parse("<div></div>");
        let parent = doc.query_selector("div").unwrap();
        let text = doc
            .create_text_node_owned(exact_dom_data(&[0xd800, 0x41]))
            .unwrap();
        let comment = doc.create_comment_owned(exact_dom_data(&[0xdc00])).unwrap();
        let pi = doc
            .create_processing_instruction_owned(
                "probe".into(),
                exact_dom_data(&[0x3f, 0x3e, 0xdfff]),
            )
            .unwrap();
        for id in [text, comment, pi] {
            doc.append_child(parent, id);
        }
        let mut runtime = Runtime::new();
        let copy = runtime.clone_dom_node(parent, true, &mut doc).unwrap();
        assert_ne!(copy, parent);
        let copied = doc.nodes[copy].children.clone();
        assert_eq!(copied.len(), 3);
        let mut destination = Document::parse("<main></main>");
        let target = destination.query_selector("main").unwrap();
        for source in [text, comment, pi] {
            import_node(&mut runtime, &mut destination, target, &doc, source, 0).unwrap();
        }
        for (tree, ids) in [
            (&doc, copied.as_slice()),
            (&destination, destination.nodes[target].children.as_slice()),
        ] {
            for (id, expected) in ids.iter().zip([
                &[0xd800, 0x41][..],
                &[0xdc00][..],
                &[0x3f, 0x3e, 0xdfff][..],
            ]) {
                let data = match &tree.nodes[*id].kind {
                    NodeKind::Text(data) | NodeKind::Comment(data) => data,
                    NodeKind::ProcessingInstruction { target, data } => {
                        assert_eq!(target, "probe");
                        data
                    }
                    _ => panic!("clone/import changed node kind"),
                };
                assert_eq!(data.units().collect::<Vec<_>>(), expected);
            }
        }
        doc.replace_character_data(text, "changed".into()).unwrap();
        assert!(
            matches!(&doc.nodes[copied[0]].kind, NodeKind::Text(data) if data.units().collect::<Vec<_>>() == [0xd800, 0x41])
        );
    }

    #[test]
    fn exact_dom_clone_prepays_reached_node_vector_growth_before_publication() {
        let mut before = Document::parse("");
        let text = before
            .create_text_node_owned(exact_dom_data(&[0xd800]))
            .unwrap();
        before.nodes.shrink_to_fit();
        let mut admitted = before.clone();
        admitted.nodes.shrink_to_fit();
        let mut runtime = Runtime::new();
        let initial = runtime.allocated;
        runtime.clone_dom_shallow(text, &mut admitted).unwrap();
        let debit = runtime.allocated - initial;
        assert!(debit >= (before.nodes.len() + 1) * std::mem::size_of::<crate::dom::Node>());
        let mut refused = before.clone();
        refused.nodes.shrink_to_fit();
        let mut cut = Runtime::new();
        cut.allocated = MAX_HEAP - debit + 1;
        assert!(
            cut.clone_dom_shallow(text, &mut refused)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(refused.nodes.len(), before.nodes.len());
        assert_eq!(refused.retained_bytes(), before.retained_bytes());
        assert!(
            matches!(&refused.nodes[text].kind, NodeKind::Text(data) if data.raw_units() == Some(&[0xd800][..]))
        );
    }

    #[test]
    fn strict_script_html_preserves_scalar_output_and_pairs_adjacent_text() {
        let mut doc = Document::parse("<div id=x></div><template><b>X</b></template>");
        let parent = doc.query_selector("#x").unwrap();
        let template = doc.query_selector("template").unwrap();
        let first = doc
            .create_text_node_owned(exact_dom_data(&[0xd83e]))
            .unwrap();
        let second = doc
            .create_text_node_owned(exact_dom_data(&[0xdd80, 0x3c, 0x26, 0x3e]))
            .unwrap();
        let comment = doc.create_comment("ok");
        let pi = doc.create_processing_instruction("probe", "?");
        for id in [first, second, comment, pi] {
            doc.append_child(parent, id);
        }
        let mut runtime = Runtime::new();
        assert_eq!(
            runtime.dom_html(parent, false, &doc).unwrap(),
            Value::String("<div id=\"x\">🦀&lt;&amp;&gt;<!--ok--><?probe ??></div>".into())
        );
        assert_eq!(
            runtime.dom_html(template, true, &doc).unwrap(),
            Value::String("<b>X</b>".into())
        );
        assert!(
            matches!(&doc.nodes[first].kind, NodeKind::Text(data) if data.raw_units() == Some(&[0xd83e][..]))
        );
    }

    #[test]
    fn strict_script_html_refuses_unpaired_or_oversized_output_without_partial_success() {
        let mut doc = Document::parse("<div id=x></div>");
        let parent = doc.query_selector("#x").unwrap();
        let text = doc
            .create_text_node_owned(exact_dom_data(&[0xd800]))
            .unwrap();
        doc.append_child(parent, text);
        let mut runtime = Runtime::new();
        let error = runtime.execute("var caught=false;try{document.getElementById('x').innerHTML;}catch(e){caught=true;}", &mut doc).unwrap_err();
        assert!(error.is_unsupported());
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
        assert!(
            matches!(&doc.nodes[text].kind, NodeKind::Text(data) if data.raw_units() == Some(&[0xd800][..]))
        );
        // The previous child-piece serializer returned just "kept" here. A
        // checked serializer reports resource exhaustion, never partial HTML.
        doc.replace_character_data(text, "kept".into()).unwrap();
        let large = doc.create_text_node(&"x".repeat(MAX_STRING + 1));
        doc.append_child(parent, large);
        let mut runtime = Runtime::new();
        assert!(
            runtime
                .dom_html(parent, true, &doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(doc.nodes[parent].children, [text, large]);
    }

    fn identifier_syntax(source: &str, strict: bool) {
        let error = Parser::program_context(source, false, strict).unwrap_err();
        assert_eq!(
            error.intrinsic_error_name(),
            Some("SyntaxError"),
            "{source}: {error}"
        );
        assert!(error.is_parse_error(), "{source}: {error}");
    }

    #[test]
    fn identifiers_decode_across_bindings_parameters_members_and_closures() {
        for strict in [false, true] {
            let (mut runtime, mut document) = property_harness();
            let source = r#"
                var \u0061=1;a+=2;assert.sameValue(\u{61},3);
                let \u0062=4;const \u{63}=5;assert.sameValue(b+c,9);
                function \u0066(\u0078=2,...\u0079){return ()=>x+y[0];}
                assert.sameValue(f(undefined,3)(),5);assert.sameValue(f.name,'f');
                var arrow=(\u0078=4,...\u0079)=>x+y[0];assert.sameValue(arrow(undefined,5),9);
                var result;try{throw 7;}catch(\u0065){result=e;}assert.sameValue(result,7);
                var x=8,π=9,o={\u0078,π,\u0069f:10,\u0074his(){return this.x;}};
                assert.sameValue(o.x,8);assert.sameValue(o.\u03c0,9);assert.sameValue(o.if,10);
                assert.sameValue(o.\u{74}his(),8);assert.sameValue(window.\u0061,3);
                var constructors={\u0066:function(v){this.value=v;}};
                assert.sameValue(new constructors.\u0066(11).value,11);
                var closure=()=>++\u0061;assert.sameValue(closure(),4);assert.sameValue(a,4);
            "#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn identifiers_use_id_properties_and_preserve_code_point_identity() {
        for point in [
            'A',
            '_',
            '$',
            'π',
            '字',
            '\u{2118}',
            '\u{212e}',
            '\u{309b}',
            '\u{037a}',
            '\u{10400}',
        ] {
            let escaped = format!("\\u{{{:x}}}", u32::from(point));
            let source = format!("var {point}=7;{escaped}");
            assert_eq!(run(&source).unwrap(), Value::Number(7.0), "{source}");
            let source = format!("var {escaped}=8;{point}");
            assert_eq!(run(&source).unwrap(), Value::Number(8.0), "{source}");
        }
        for point in [
            '0',
            '\u{0300}',
            '\u{00b7}',
            '\u{203f}',
            '\u{0660}',
            '\u{200c}',
            '\u{200d}',
            '\u{10400}',
        ] {
            let source = format!("var a{point}=9;a\\u{{{:x}}}", u32::from(point));
            assert_eq!(run(&source).unwrap(), Value::Number(9.0), "{source}");
        }
        for point in [
            '\u{0300}', '\u{00b7}', '\u{203f}', '\u{0660}', '\u{200c}', '\u{200d}', '\u{00b2}',
            '\u{0378}', '😀',
        ] {
            identifier_syntax(&format!("var {point}=0;"), false);
            identifier_syntax(&format!("var \\u{{{:x}}}=0;", u32::from(point)), false);
        }
        identifier_syntax("var a²=0;", false);
        assert_eq!(
            run(r"var é=1,e\u0301=2;é+'|'+é").unwrap().to_string(),
            "1|2"
        );
        assert_eq!(run(r"var K=1,K=2;K+'|'+K").unwrap().to_string(), "1|2");
    }

    #[test]
    fn identifiers_reject_malformed_and_invalid_position_escapes() {
        for name in [
            r"\u",
            r"\u12",
            r"\u00G0",
            r"\u{}",
            r"\u{61",
            r"\u{110000}",
            r"\u{ffffffffffffffff}",
            r"\uD800",
            r"\uDC00",
            r"\uD801\uDC00",
            r"\u{d800}",
            r"\x61",
            r"\U0061",
            r"\u0030",
            r"\u0020",
            r"\u003b",
            r"\u0000",
            r"\u2028",
            r"a\u0020",
            r"a\u003b",
            r"a\uD800",
            r"a\q",
        ] {
            identifier_syntax(&format!("var {name}=1;"), false);
        }
        identifier_syntax("var a\\\n=1;", false);
        assert_eq!(
            run(r"var a\u0030=1;var \u0061b=2;a0+ab").unwrap(),
            Value::Number(3.0)
        );
        let zeros = "0".repeat(16_384);
        assert_eq!(
            run(&format!("var \\u{{{zeros}61}}=12;a")).unwrap(),
            Value::Number(12.0)
        );
        let source = r"var π=1;var x\u{2d}=0;";
        let error = Parser::program(source).unwrap_err();
        assert_eq!(error.offset, source.find('\\'));
        assert_eq!(run(r"var 𐐀=3;\u{10400}").unwrap(), Value::Number(3.0));
    }

    #[test]
    fn identifiers_escaped_reserved_words_are_names_only_in_property_positions() {
        for keyword in [
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
        ] {
            let escaped = format!("\\u{:04x}{}", keyword.as_bytes()[0], &keyword[1..]);
            for strict in [false, true] {
                identifier_syntax(&format!("var {escaped}=1;"), strict);
                identifier_syntax(&format!("{escaped};"), strict);
                identifier_syntax(&format!("function {escaped}(){{}}"), strict);
                let source = format!("({{{escaped}:3}}).{escaped}");
                let result = if strict {
                    Runtime::new().execute_strict(&source, &mut Document::parse(""))
                } else {
                    run(&source)
                };
                assert_eq!(result.unwrap(), Value::Number(3.0), "{source}");
            }
        }
        for source in [
            r"function f(){\u0072eturn 1;}",
            r"\u0069f(true){}",
            r"\u0074ypeof 1",
            r"1 \u0069n {}",
            r"({}) \u0069nstanceof Object",
            r"\u0069f:;",
            r"while(true){break \u0069f;}",
            r"while(true){continue \u0069f;}",
        ] {
            identifier_syntax(source, false);
        }
        for source in [
            r"var \u0065val=1;",
            r"var \u0061rguments=1;",
            r"\u0065val=1;",
            r"++\u0061rguments;",
            r"function f(\u0065val){}",
            r"function f(\u0061rguments){'use strict';}",
        ] {
            identifier_syntax(source, true);
        }
    }

    #[test]
    fn identifiers_object_introducers_and_proto_setters_use_distinct_rules() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var value=1,get=7,set=8,async=9;
            var o={get \u0078(){return value;},set \u0078(v){value=v;},g\u0065t(){return 4;},\u0061sync(){return 5;}};
            assert.sameValue(o.x,1);o.x=3;assert.sameValue(value,3);
            assert.sameValue(o.get(),4);assert.sameValue(o.async(),5);
            var shorthand={g\u0065t,s\u0065t,\u0061sync};assert.sameValue(shorthand.get,7);assert.sameValue(shorthand.set,8);assert.sameValue(shorthand.async,9);
            var p={value:6},child={\u005f_proto__:p};assert.sameValue(Object.getPrototypeOf(child),p);
            var own={['__proto__']:p};assert.sameValue(Object.getPrototypeOf(own),Object.prototype);
        "#,&mut document).unwrap();
        for source in [
            r"({g\u0065t x(){}})",
            r"({s\u0065t x(v){}})",
            r"({\u0061sync x(){}})",
            r"({__proto__:null,\u005f_proto__:null})",
            r"({\u0069f})",
        ] {
            identifier_syntax(source, false);
        }
    }

    #[test]
    fn identifiers_script_contextual_names_are_not_blanket_keywords() {
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            let source = r#"
                var async=function(v){return v+1;};assert.sameValue(async(2),3);assert.sameValue(\u0061sync(3),4);
                var a=async=>async+2,b=\u0061sync=>async+3;assert.sameValue(a(1),3);assert.sameValue(b(1),4);
                var await=4;await++;assert.sameValue(\u0061wait,5);
                function f(await){return await+1;}assert.sameValue(f(5),6);
                var g=(\u0061wait=1,...rest)=>await+rest[0];assert.sameValue(g(undefined,2),3);
                var source={await,async};assert.sameValue(source.await,5);assert.sameValue(source.async,async);
            "#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
        assert_eq!(
            run(r"var yield=1;yield++;\u0079ield").unwrap(),
            Value::Number(2.0)
        );
        assert_eq!(
            run(r"var let=1;let+=2;\u006cet").unwrap(),
            Value::Number(3.0)
        );
        assert_eq!(
            run(r"var let={};let instanceof Object").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            run(r"var let='';for(let in {a:1}){}let")
                .unwrap()
                .to_string(),
            "a"
        );
        assert_eq!(
            run("var async=1;async\nfunction f(){return 2;}f()").unwrap(),
            Value::Number(2.0)
        );
        for source in [
            r"var yield=1;",
            r"var \u0079ield=1;",
            r"yield;",
            r"var let=1;",
            r"var \u006cet=1;",
            r"function f(\u0079ield){}",
        ] {
            identifier_syntax(source, true);
        }
        for source in [
            r"let let=1;",
            r"let \u006cet=1;",
            r"\u006cet x=1;",
            r"await 1;",
            r"yield 1;",
            r"\u0061sync function f(){}",
            r"\u0061sync x=>x",
            r"\u0061sync(x)=>x",
            "async\n(x)=>x",
            "async(x)\n=>x",
        ] {
            identifier_syntax(source, false);
        }
    }

    #[test]
    fn identifiers_valid_unavailable_grammars_stay_explicit() {
        for source in [
            "async function f(){}",
            "(async function(){})",
            "async x=>x",
            "async(x)=>x",
            "async()=>1",
            "function* f(){yield 1;}",
            "(function*(){yield 1;})",
            "({*f(){yield 1;}})",
            "label:function f(){}",
            "class C{}",
            "import x from 'x';",
            "export var x=1;",
        ] {
            let error = Parser::program(source).unwrap_err();
            assert!(error.is_unsupported(), "{source}: {error}");
        }
        for source in [
            r"\u0066unction* f(){}",
            r"(\u0066unction*(){})",
            r"\u0069mport x from 'x';",
            r"\u0065xport var x=1;",
        ] {
            identifier_syntax(source, false);
        }
        assert!(run(r"/(?<π>a)/").unwrap_err().is_unsupported());
        assert!(run(r"/a/u").unwrap_err().is_unsupported());
        assert!(run(r"/a/v").unwrap_err().is_unsupported());
    }

    #[test]
    fn identifiers_numeric_adjacency_and_regex_flags_use_source_characters() {
        for source in [
            r"3in {}",
            r"1\u0061",
            r"0x1g",
            r"1.π",
            r"1e+2x",
            r"1\u0030",
            r"1$",
            r"1_",
            "/a/π",
            "/a/g\u{0300}",
            "/a/g\u{200c}",
            r"/a/\u0067",
            r"/a/g\u0069",
            "/a/g$",
            "/a/1",
            "/a/gg",
        ] {
            identifier_syntax(source, false);
        }
        assert_eq!(run("1 in {1:0}").unwrap(), Value::Bool(true));
        assert_eq!(run("1..toString()").unwrap().to_string(), "1");
        assert_eq!(run(r"/1in/.test('1in')").unwrap(), Value::Bool(true));
        assert_eq!(run(r"/\u0061/g.test('a')").unwrap(), Value::Bool(true));
        assert_eq!(run(r"var \u0061=8;a/2").unwrap(), Value::Number(4.0));
    }

    #[test]
    fn identifiers_rescans_preserve_escape_identity_and_asi_boundaries() {
        assert_eq!(
            run(r#"var \u03c0=3;`${\u03c0}:${`x${π}`}:${/[a-z\u0061]/.test('a')}`"#)
                .unwrap()
                .to_string(),
            "3:x3:true"
        );
        assert_eq!(
            run(r#"var \u0061=8;/["']/.test('"')+'|'+a/2"#)
                .unwrap()
                .to_string(),
            "true|4"
        );
        assert_eq!(
            run("\u{feff}var\u{feff}a=2;\u{00a0}a").unwrap(),
            Value::Number(2.0)
        );
        assert_eq!(
            run("function f(){return\u{000b}1;}f()").unwrap(),
            Value::Number(1.0)
        );
        for terminator in ['\n', '\r', '\u{2028}', '\u{2029}'] {
            assert_eq!(
                run(&format!("function f(){{return{terminator}1;}}f()")).unwrap(),
                Value::Undefined
            );
        }
        for point in ['\u{0085}', '\u{180e}'] {
            identifier_syntax(&format!("var{point}a=1;"), false);
        }
        assert_eq!(
            run("var a=1;// \\u000a a=2;\na").unwrap(),
            Value::Number(1.0)
        );
        assert_eq!(
            run(r"function f(){return/*\u000a*/1;}f()").unwrap(),
            Value::Number(1.0)
        );
        let source = r"/a/;var π=1;var x\u002d=2;";
        let error = Parser::program(source).unwrap_err();
        assert_eq!(error.offset, source.find('\\'));
    }

    #[test]
    fn identifiers_decoded_duplicates_and_parse_failure_do_not_create_bindings() {
        for source in [
            r"let a;let \u0061;",
            r"const a=1;let \u0061;",
            r"function f(a,\u0061){'use strict';}",
            r"function f(a=1,\u0061){}",
            r"(a,\u0061)=>a",
            r"function f(a,...\u0061){}",
        ] {
            identifier_syntax(source, false);
        }
        assert_eq!(
            run(r"function f(a,\u0061){return a;}f(1,2)").unwrap(),
            Value::Number(2.0)
        );
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime.execute("var present=1;", &mut document).unwrap();
        let error = runtime
            .execute(
                r"var fresh=2;let lexical;function fn(){}var \u0069f=3;",
                &mut document,
            )
            .unwrap_err();
        assert_eq!(error.intrinsic_error_name(), Some("SyntaxError"));
        assert_eq!(
            runtime
                .execute(
                    "present+'|'+typeof fresh+'|'+typeof lexical+'|'+typeof fn",
                    &mut document
                )
                .unwrap()
                .to_string(),
            "1|undefined|undefined|undefined"
        );
    }

    #[test]
    fn identifiers_compilation_shares_work_storage_and_token_text() {
        fn budget() -> regexp::Budget {
            regexp::Budget {
                steps: MAX_STEPS,
                allocated: 0,
                heap_limit: MAX_HEAP,
                stack_limit: 16,
            }
        }
        let mut ledger = budget();
        let tokens = lex(r"\u0061;", &mut ledger).unwrap();
        let TokenKind::Word(first) = &tokens[0].kind else {
            panic!("word")
        };
        assert!(first.escaped);
        assert_eq!(&*first.value, "a");
        let clone = tokens[0].clone();
        let TokenKind::Word(second) = clone.kind else {
            panic!("word")
        };
        assert!(Rc::ptr_eq(&first.value, &second.value));
        let allocated = ledger.allocated;
        let steps = ledger.steps;
        lex(r"\u0062;", &mut ledger).unwrap();
        assert!(ledger.allocated > allocated);
        assert!(ledger.steps < steps);
        ledger.steps = 0;
        assert!(lex("a", &mut ledger).unwrap_err().is_resource_limit());
        let mut ledger = budget();
        ledger.heap_limit = 0;
        assert!(lex("a", &mut ledger).unwrap_err().is_resource_limit());
        let mut ledger = budget();
        ledger.steps = 3;
        assert!(lex("var π=1", &mut ledger).unwrap_err().is_resource_limit());
        let mut ledger = budget();
        let tokens = lex(r"/\x/", &mut ledger).unwrap();
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Invalid(_)))
        );
        assert!(
            Parser::program(&"/a/;".repeat(400))
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            Parser::program(&"a".repeat(MAX_SOURCE + 1))
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            Parser::program(&format!("var \\u{{{}61}}=1;", "0".repeat(200_000)))
                .unwrap_err()
                .is_resource_limit()
        );
        // Keep the original input as a visible optimization control: replacing
        // repeated binary searches with two reads makes this valid name fit.
        assert!(Parser::program(&format!("var {}=1;", "π".repeat(20_000))).is_ok());
        // The same work limit still terminates sufficiently many actual reads.
        assert!(
            Parser::program(&format!("var {}=1;", "π".repeat(50_001)))
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            Parser::program(&";".repeat(MAX_TOKENS + 1))
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn identifiers_execute_six_unchanged_pinned_function_modes() {
        for source in [
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S13_A7_T1.js"
            ),
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S14_A5_T1.js"
            ),
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S14_A5_T2.js"
            ),
        ] {
            for strict in [false, true] {
                let (mut runtime, mut document) = upstream_harness();
                if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                }
                .unwrap();
            }
        }
    }

    #[test]
    fn identifiers_compact_tokens_precharge_shared_diagnostics() {
        assert_eq!(std::mem::size_of::<Token>(), 48);
        let mut budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        lex("é", &mut budget).unwrap();
        // One raw-start query, rather than an identical query in each helper.
        assert_eq!(MAX_STEPS - budget.steps, 4 + IDENTIFIER_LOOKUP_WORK);
        let required = 4 * std::mem::size_of::<Vec<Token>>()
            + 32
            + 128 * std::mem::size_of::<Token>()
            + 32
            + std::mem::size_of::<ScriptError>()
            + 2 * std::mem::size_of::<usize>();
        budget.steps = MAX_STEPS;
        budget.allocated = 0;
        budget.heap_limit = required - 1;
        let error = lex("@", &mut budget).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(budget.allocated, required);
        budget.steps = MAX_STEPS;
        budget.allocated = 0;
        budget.heap_limit = required;
        let tokens = lex("@", &mut budget).unwrap();
        assert_eq!(budget.allocated, required);
        let TokenKind::Invalid(original) = &tokens[0].kind else {
            panic!("expected provisional syntax diagnostic")
        };
        let TokenKind::Invalid(copy) = tokens[0].clone().kind else {
            panic!("expected shared diagnostic")
        };
        assert!(Rc::ptr_eq(original, &copy));
        assert_eq!(original.offset, Some(0));
        assert!(original.is_parse_error());
    }

    #[test]
    fn identifiers_cold_diagnostics_keep_absolute_rescan_offsets() {
        for source in [
            r"/a/;var \u{};",
            r#"/['"]/;var \u{};"#,
            r"`ok`;var \uD800;",
            r"`before${/a/.test('a')}after`;var \u{};",
            r"`outer${`inner${1}`}`;var \u{};",
        ] {
            let expected = source.rfind(r"\u").unwrap();
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(error.is_parse_error(), "{source}: {error}");
            assert_eq!(error.offset, Some(expected), "{source}: {error}");
        }
        // The provisional malformed string belongs to RegExp source and is
        // discarded under the parser-selected lexical goal.
        assert!(Runtime::parse_only(r#"/['"]/;`ok${1}`;"#).is_ok());
    }

    #[test]
    fn identifiers_parser_source_borrow_does_not_escape_into_retained_code() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<p></p>");
        {
            let source = String::from(
                r#"var \u03c0=7;
                function retained(\u0061=π){return `value:${a}:${/ab/.test('ab')}`;}
                document.querySelector('p').onclick=function(){this.textContent=retained();};"#,
            );
            runtime.execute(&source, &mut document).unwrap();
        }
        assert_eq!(
            runtime
                .execute("retained()", &mut document)
                .unwrap()
                .to_string(),
            "value:7:true"
        );
        let node = document.query_selector("p").unwrap();
        runtime
            .dispatch_event(node, "click", &mut document)
            .unwrap();
        assert_eq!(document.text_content(node), "value:7:true");
    }

    #[test]
    fn identifiers_largest_previously_passing_raw_table_keeps_exact_limits() {
        let source = include_str!(
            "../tests/upstream/test262-identifiers/test/language/identifiers/start-unicode-10.0.0.js"
        );
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn global_values_have_live_descriptor_flags_and_private_realm_identity() {
        for strict in [false, true] {
            let (mut runtime, mut document) = property_harness();
            let source = r#"
                var w=window,d=Object.getOwnPropertyDescriptor(w,'globalThis');
                assert.sameValue(globalThis,w);assert.sameValue(this,w);
                assert.sameValue(d.value,w);assert.sameValue(d.writable,true);
                assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
                assert.sameValue('get' in d,false);assert.sameValue('set' in d,false);
                var names=['undefined','NaN','Infinity'];var values=[void 0,0/0,1/0];
                for(var i=0;i<names.length;i++){
                    var descriptor=Object.getOwnPropertyDescriptor(w,names[i]);
                    assert.sameValue(descriptor.value,values[i]);
                    assert.sameValue(descriptor.writable,false);assert.sameValue(descriptor.enumerable,false);
                    assert.sameValue(descriptor.configurable,false);assert.sameValue('get' in descriptor,false);
                }
                var selfGetter=Object.getOwnPropertyDescriptor(w,'self').get;
                var replacements=[0,null,void 0,false,'other',{},function(){}];
                for(var j=0;j<replacements.length;j++){
                    globalThis=replacements[j];assert.sameValue(w.globalThis,replacements[j]);
                    assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis').enumerable,false);
                    assert.sameValue(this,w);assert.sameValue(selfGetter.call(null),w);
                    assert.sameValue(document.defaultView,w);assert.sameValue(self,w);
                }
                var calls=0,poison={toString(){calls++;throw 1;},valueOf(){calls++;throw 2;}};
                globalThis=poison;assert.sameValue(globalThis,poison);assert.sameValue(calls,0);
                function loose(){return this;}function tight(){'use strict';return this;}
                assert.sameValue(tight(),void 0);
                var eventCount=0;
                w.addEventListener('global-values',function(event){
                    assert.sameValue(this,w);assert.sameValue(event.target,w);eventCount++;
                });
                w.dispatchEvent(new Event('global-values'));assert.sameValue(eventCount,1);
                Object.defineProperty(w,'globalThis',d);assert.sameValue(globalThis,w);
            "#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn global_values_immutable_writes_deletes_and_updates_preserve_order() {
        for name in ["undefined", "NaN", "Infinity"] {
            let (mut runtime, mut document) = property_harness();
            runtime.execute(&format!(r#"
                var w=window,original=w['{name}'],effects=0;
                function right(){{effects++;return {{toString(){{throw 'coerced';}},valueOf(){{throw 'coerced';}}}};}}
                var supplied=right();assert.sameValue({name}=supplied,supplied);
                assert.sameValue(w['{name}'],original);assert.sameValue(effects,1);
                assert.sameValue(delete {name},false);assert.sameValue(delete w['{name}'],false);
                assert.throws(TypeError,function(){{'use strict';{name}=right();}});
                assert.sameValue(effects,2);assert.sameValue(w['{name}'],original);
                assert.throws(TypeError,function(){{'use strict';delete w['{name}'];}});
                var converted=0;var extra={{valueOf(){{converted++;return 2;}}}};
                {name}+=extra;assert.sameValue(converted,1);assert.sameValue(w['{name}'],original);
                assert.throws(TypeError,function(){{'use strict';{name}+=extra;}});
                assert.sameValue(converted,2);assert.sameValue(w['{name}'],original);
                {name}++;assert.sameValue(w['{name}'],original);
                assert.throws(TypeError,function(){{'use strict';++{name};}});
                var {name};assert.sameValue(w['{name}'],original);
                var descriptor=Object.getOwnPropertyDescriptor(w,'{name}');
                assert.sameValue(descriptor.enumerable,false);assert.sameValue(descriptor.writable,false);
            "#),&mut document).unwrap();
            assert!(
                Runtime::parse_only_strict(&format!("delete {name}"))
                    .unwrap_err()
                    .is_parse_error()
            );
        }
    }

    #[test]
    fn global_values_immutable_descriptors_use_same_value_without_author_coercion() {
        for strict in [false, true] {
            let (mut runtime, mut document) = property_harness();
            let source = r#"
                var w=window,names=['undefined','NaN','Infinity'],values=[void 0,0/0,1/0];
                var calls=0,poison={toString(){calls++;throw 'coerced';},valueOf(){calls++;throw 'coerced';}};
                for(var i=0;i<names.length;i++){
                    var name=names[i],value=values[i];
                    assert.sameValue(Object.defineProperty(w,name,{}),w);
                    assert.sameValue(Object.defineProperty(w,name,{value:value,writable:false,enumerable:false,configurable:false}),w);
                    assert.throws(TypeError,function(){Object.defineProperty(w,name,{value:poison});});
                    assert.throws(TypeError,function(){Object.defineProperty(w,name,{writable:true});});
                    assert.throws(TypeError,function(){Object.defineProperty(w,name,{enumerable:true});});
                    assert.throws(TypeError,function(){Object.defineProperty(w,name,{configurable:true});});
                    assert.throws(TypeError,function(){Object.defineProperty(w,name,{get:undefined});});
                    assert.sameValue(w[name],value);
                }
                assert.sameValue(calls,0);
                Object.defineProperty(w,'NaN',{value:Infinity-Infinity});assert.sameValue(w.NaN,0/0);
                assert.throws(TypeError,function(){Object.defineProperty(w,'Infinity',{value:-Infinity});});
                var log='';var descriptor={
                    get enumerable(){log+='e';return false;},get configurable(){log+='c';return false;},
                    get value(){log+='v';return 7;},get writable(){log+='w';return false;}
                };
                assert.throws(TypeError,function(){Object.defineProperty(w,'undefined',descriptor);});
                assert.sameValue(log,'ecvw');assert.sameValue(w.undefined,void 0);
                var reason={},seen;try{Object.defineProperty(w,'Infinity',{get value(){throw reason;}});}catch(error){seen=error;}
                assert.sameValue(seen,reason);assert.sameValue(w.Infinity,1/0);
            "#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn global_values_accessors_reenter_and_borrowed_self_descriptor_keeps_its_target() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,original=Object.getOwnPropertyDescriptor(w,'globalThis'),log='';
            var descriptor={get:function(){assert.sameValue(this,w);log+='g';return 4;},set:function(value){
                assert.sameValue(this,w);log+='s'+value;delete w.globalThis;w.globalThis='replaced';
            },configurable:true};
            Object.defineProperty(w,'globalThis',descriptor);
            var saved=Object.getOwnPropertyDescriptor(w,'globalThis');assert.sameValue(log,'');
            globalThis+=2;assert.sameValue(log,'gs6');assert.sameValue(globalThis,'replaced');
            Object.defineProperty(w,'globalThis',{get:function(){delete w.globalThis;return 'deleted';}});
            assert.sameValue(globalThis,'deleted');assert.sameValue(typeof globalThis,'undefined');
            var reason={},seen;
            Object.defineProperty(w,'globalThis',{get:function(){throw reason;},set:function(){throw reason;},configurable:true});
            try{globalThis;}catch(error){seen=error;}assert.sameValue(seen,reason);
            seen=undefined;try{globalThis=2;}catch(error){seen=error;}assert.sameValue(seen,reason);
            Object.defineProperty(w,'globalThis',{get:undefined,set:undefined});
            assert.sameValue(globalThis,undefined);globalThis=2;assert.sameValue(globalThis,undefined);
            assert.throws(TypeError,function(){'use strict';globalThis=2;});
            Object.defineProperty(w,'globalThis',original);
            var selfDescriptor=Object.getOwnPropertyDescriptor(w,'self');
            Object.defineProperty(w,'globalThis',selfDescriptor);
            globalThis=9;assert.sameValue(self,9);assert.sameValue(globalThis,w);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis').get,selfDescriptor.get);
            Object.defineProperty(w,'self',selfDescriptor);Object.defineProperty(w,'globalThis',original);
            Object.defineProperty(w,'globalThis',{value:7,writable:false,configurable:false});
            globalThis=8;assert.sameValue(globalThis,7);
            assert.throws(TypeError,function(){'use strict';globalThis=8;});
            assert.sameValue(delete w.globalThis,false);
            assert.throws(TypeError,function(){Object.defineProperty(w,'globalThis',original);});
        "#,&mut document).unwrap();
    }

    #[test]
    fn global_values_captured_references_observe_rhs_and_numeric_coercion_mutations() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,original=Object.getOwnPropertyDescriptor(w,'globalThis');
            function remove(){delete w.globalThis;return 4;}
            globalThis=remove();assert.sameValue(globalThis,4);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis').enumerable,true);
            assert.throws(ReferenceError,function(){'use strict';globalThis=remove();});
            assert.sameValue(typeof globalThis,'undefined');assert.sameValue(delete w.globalThis,true);
            function install(){w.globalThis=6;return 7;}
            assert.throws(ReferenceError,function(){'use strict';globalThis=install();});
            assert.sameValue(globalThis,6);
            function member(){'use strict';w.globalThis=remove();}member();assert.sameValue(globalThis,4);
            globalThis={valueOf(){delete w.globalThis;return 10;}};
            assert.sameValue(globalThis++,10);assert.sameValue(globalThis,11);
            globalThis={valueOf(){delete w.globalThis;return 10;}};
            assert.throws(ReferenceError,function(){'use strict';globalThis++;});
            assert.sameValue(typeof globalThis,'undefined');
            Object.defineProperty(w,'globalThis',original);
            function lock(){Object.defineProperty(w,'globalThis',{value:3,writable:false});return 9;}
            globalThis=lock();assert.sameValue(globalThis,3);
            assert.throws(TypeError,function(){'use strict';globalThis=lock();});
            Object.defineProperty(w,'globalThis',original);
            var received=0;
            function installSetter(){Object.defineProperty(w,'globalThis',{get:undefined,set:function(value){received=value;}});return 12;}
            globalThis=installSetter();assert.sameValue(received,12);assert.sameValue(globalThis,undefined);
        "#,&mut document).unwrap();
    }

    #[test]
    fn global_values_inherited_bindings_and_member_receivers_are_bounded_and_coherent() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,original=Object.getOwnPropertyDescriptor(w,'globalThis');
            delete w.globalThis;Object.prototype.globalThis=5;
            assert.sameValue(globalThis,5);assert.sameValue(w.globalThis,5);assert.sameValue('globalThis' in w,true);
            assert.sameValue(delete globalThis,true);assert.sameValue(Object.prototype.globalThis,5);
            globalThis=6;assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis').value,6);
            assert.sameValue(Object.prototype.globalThis,5);delete w.globalThis;
            var saved=7,log='';
            Object.defineProperty(Object.prototype,'globalThis',{get:function(){assert.sameValue(this,w);log+='g';return saved;},set:function(v){assert.sameValue(this,w);log+='s';saved=v;},configurable:true});
            globalThis+=2;assert.sameValue(log,'gs');assert.sameValue(saved,9);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis'),undefined);
            assert.sameValue(delete globalThis,true);assert.sameValue(typeof Object.getOwnPropertyDescriptor(Object.prototype,'globalThis').get,'function');
            Object.defineProperty(Object.prototype,'globalThis',{value:4,writable:false});
            globalThis=2;assert.sameValue(globalThis,4);
            assert.throws(TypeError,function(){'use strict';globalThis=2;});
            delete Object.prototype.globalThis;Object.defineProperty(w,'globalThis',original);
            var child=Object.create(w);child.globalThis=8;
            assert.sameValue(child.globalThis,8);assert.sameValue(w.globalThis,w);
            Object.defineProperty(w,'globalThis',{get:function(){return this;},set:function(value){this.marker=value;}});
            delete child.globalThis;assert.sameValue(child.globalThis,child);
            child.globalThis=10;assert.sameValue(child.marker,10);assert.sameValue(w.marker,undefined);
        "#,&mut document).unwrap();
    }

    #[test]
    fn global_values_global_lexical_var_and_local_bindings_remain_distinct() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute("var w=window,original=Object.getOwnPropertyDescriptor(w,'globalThis');var globalThis;assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis').enumerable,false);",&mut document).unwrap();
        runtime.execute_strict("let globalThis='lexical';assert.sameValue(globalThis,'lexical');assert.sameValue(w.globalThis,w);w.globalThis=4;globalThis='changed';assert.sameValue(w.globalThis,4);assert.sameValue(this,w);",&mut document).unwrap();
        runtime.execute("delete w.globalThis;assert.sameValue(globalThis,'changed');assert.sameValue(w.globalThis,undefined);Object.defineProperty(w,'globalThis',original);assert.sameValue(globalThis,'changed');",&mut document).unwrap();
        for source in ["var globalThis;", "function globalThis(){}"] {
            assert_eq!(
                runtime.execute(source, &mut document).unwrap_err().name(),
                "SyntaxError"
            );
        }
        let (mut runtime, mut document) = property_harness();
        runtime.execute("var w=window;delete w.globalThis;Object.defineProperty(Object.prototype,'globalThis',{set:function(){throw 'inherited setter';},configurable:true});",&mut document).unwrap();
        runtime.execute_strict("var globalThis;assert.sameValue(globalThis,undefined);var d=Object.getOwnPropertyDescriptor(w,'globalThis');assert.sameValue(d.configurable,false);assert.sameValue(d.enumerable,true);assert.sameValue(d.writable,true);delete Object.prototype.globalThis;",&mut document).unwrap();
        assert_eq!(
            runtime
                .execute("let globalThis;", &mut document)
                .unwrap_err()
                .name(),
            "SyntaxError"
        );
        for name in ["undefined", "NaN", "Infinity"] {
            let (mut runtime, mut document) = property_harness();
            assert_eq!(
                runtime
                    .execute(&format!("let before;let {name};"), &mut document)
                    .unwrap_err()
                    .name(),
                "SyntaxError"
            );
            assert!(!runtime.environments[1].bindings.contains_key("before"));
            runtime.execute_strict(&format!("function local({name}){{return {name};}}assert.sameValue(local(2),2);{{let {name}=3;assert.sameValue({name},3);}}"),&mut document).unwrap();
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        assert_eq!(
            runtime
                .execute("let globalThis=globalThis;", &mut document)
                .unwrap_err()
                .name(),
            "ReferenceError"
        );
        assert!(matches!(
            runtime
                .own_property(
                    &Value::Window,
                    &runtime.global_key(TrackedGlobal::GlobalThis)
                )
                .unwrap()
                .value,
            PropertyValue::Data {
                value: Value::Window,
                writable: true
            }
        ));
    }

    #[test]
    fn global_values_function_preflight_checks_all_tracked_names_without_partial_declarations() {
        for strict in [false, true] {
            for kind in TrackedGlobal::ALL {
                let (mut runtime, mut document) = property_harness();
                let name = kind.name();
                if matches!(kind, TrackedGlobal::WindowSelf | TrackedGlobal::GlobalThis) {
                    runtime
                        .execute(
                            &format!(
                                "Object.defineProperty(window,'{name}',{{configurable:false}});"
                            ),
                            &mut document,
                        )
                        .unwrap();
                }
                let global_count = runtime.environments[0].bindings.len();
                let lexical_count = runtime.environments[1].bindings.len();
                let function_count = runtime.functions.len();
                let source = format!(
                    "let freshLexical;var freshVar;function freshFunction(){{}}function {name}(){{}}"
                );
                let error = if strict {
                    runtime.execute_strict(&source, &mut document)
                } else {
                    runtime.execute(&source, &mut document)
                }
                .unwrap_err();
                assert_eq!(error.name(), "TypeError");
                assert_eq!(runtime.environments[0].bindings.len(), global_count);
                assert_eq!(runtime.environments[1].bindings.len(), lexical_count);
                assert_eq!(runtime.functions.len(), function_count);
                runtime.execute("assert.sameValue(typeof freshLexical,'undefined');assert.sameValue(typeof freshVar,'undefined');assert.sameValue(typeof freshFunction,'undefined');",&mut document).unwrap();
                runtime.execute("let freshLexical=1;var freshVar=2;function freshFunction(){return 3;}assert.sameValue(freshLexical+freshVar+freshFunction(),6);",&mut document).unwrap();
            }
        }
        for kind in TrackedGlobal::ALL {
            let (mut runtime, mut document) = property_harness();
            if matches!(kind, TrackedGlobal::WindowSelf | TrackedGlobal::GlobalThis) {
                runtime
                    .execute(
                        &format!(
                            "Object.defineProperty(window,'{}',{{configurable:false}});",
                            kind.name()
                        ),
                        &mut document,
                    )
                    .unwrap();
            }
            runtime.execute("let occupied=1;", &mut document).unwrap();
            let error = runtime
                .execute(
                    &format!(
                        "let fresh;var early;for(var occupied in {{}}){{}}function {}(){{}}",
                        kind.name()
                    ),
                    &mut document,
                )
                .unwrap_err();
            assert_eq!(error.name(), "SyntaxError");
            assert!(!runtime.environments[0].bindings.contains_key("early"));
            assert!(!runtime.environments[1].bindings.contains_key("fresh"));
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let error = runtime
            .execute("function NaN(){}function Infinity(){}", &mut document)
            .unwrap_err();
        assert!(error.message.contains("Infinity"));
        assert!(
            runtime
                .execute("function Infinity(){}function NaN(){}", &mut document)
                .unwrap_err()
                .message
                .contains("NaN")
        );
    }

    #[test]
    fn global_values_function_commit_uses_last_declaration_and_preserves_compatible_flags() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let before = runtime.functions.len();
        assert_eq!(runtime.execute("function globalThis(){return 1;}function self(){return 2;}function globalThis(){return 3;}function self(){return 4;}globalThis()+self();",&mut document).unwrap(),Value::Number(7.0));
        assert_eq!(runtime.functions.len(), before + 2);
        for kind in [TrackedGlobal::WindowSelf, TrackedGlobal::GlobalThis] {
            let property = runtime
                .own_property(&Value::Window, &runtime.global_key(kind))
                .unwrap();
            assert!(!property.configurable);
            assert!(property.enumerable);
            assert!(matches!(
                property.value,
                PropertyValue::Data { writable: true, .. }
            ));
        }
        let (mut runtime, mut document) = property_harness();
        runtime.execute("Object.defineProperty(window,'globalThis',{get:function(){throw 'getter';},set:function(){throw 'setter';},configurable:true});",&mut document).unwrap();
        runtime
            .execute(
                "function globalThis(){return 5;}assert.sameValue(globalThis(),5);",
                &mut document,
            )
            .unwrap();
        runtime
            .execute(
                "function globalThis(){return 6;}assert.sameValue(globalThis(),6);",
                &mut document,
            )
            .unwrap();
        runtime
            .execute(
                "Object.defineProperty(window,'globalThis',{writable:false});",
                &mut document,
            )
            .unwrap();
        assert_eq!(
            runtime
                .execute("function globalThis(){}", &mut document)
                .unwrap_err()
                .name(),
            "TypeError"
        );
    }

    #[test]
    fn global_values_definition_keys_and_descriptor_reentrancy_keep_exact_names_and_order() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,original=Object.getOwnPropertyDescriptor(w,'globalThis'),log='';
            var key={toString(){log+='k';delete w.globalThis;return 'globalThis';}};
            var descriptor={get value(){log+='v';return 5;},configurable:true,writable:true};
            Object.defineProperty(w,key,descriptor);assert.sameValue(log,'kv');assert.sameValue(globalThis,5);
            Object.defineProperty(w,'globalThis',original);
            var poison={get value(){Object.defineProperty(w,'globalThis',{value:7,writable:false,configurable:false});return 8;}};
            assert.throws(TypeError,function(){Object.defineProperty(w,'globalThis',poison);});assert.sameValue(globalThis,7);
            var calls=0;assert.throws(TypeError,function(){Object.defineProperty(null,{toString(){calls++;return 'NaN';}},{value:0});});
            assert.sameValue(calls,0);
            Object.defineProperty(w,{toString(){calls++;return 'NaN';}},{value:0/0});assert.sameValue(calls,1);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'globalThis\u0000'),undefined);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'NaN\uD800'),undefined);
        "#,&mut document).unwrap();
        for key in [
            "'globalThis\\u0000'",
            "'globalThis\\uD800'",
            "'GlobalThis'",
            "'undefined '",
            "'NaN\\uFFFD'",
        ] {
            assert_eq!(
                run(&format!(
                    "Object.defineProperty(window,{key},{{value:1}});window[{key}]"
                ))
                .unwrap(),
                Value::Number(1.0)
            );
        }
        assert_eq!(
            run("Object.defineProperty(window,'other',{value:1});other").unwrap(),
            Value::Number(1.0)
        );
        assert!(matches!(
            run("Object.keys(window)").unwrap(),
            Value::Array(_)
        ));
        assert_eq!(
            run("window.hasOwnProperty('globalThis')").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            run("window.propertyIsEnumerable('NaN')").unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn global_values_share_resource_limits_and_preflight_every_final_property_store() {
        for kind in TrackedGlobal::ALL {
            let mut runtime = Runtime::new();
            let key = runtime.global_key(kind);
            let old = runtime.own_property(&Value::Window, &key).unwrap();
            let count = runtime.environments[0].bindings.len();
            runtime.steps = 0;
            assert!(
                runtime
                    .define_own(&Value::Window, &key, PropertyDescriptor::default())
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.environments[0].bindings.len(), count);
            let current = runtime.own_property(&Value::Window, &key).unwrap();
            assert_eq!(old.enumerable, current.enumerable);
            assert_eq!(old.configurable, current.configurable);
            match (&old.value, &current.value) {
                (PropertyValue::Data { value: a, .. }, PropertyValue::Data { value: b, .. }) => {
                    assert!(json_same_value(a, b))
                }
                (
                    PropertyValue::Accessor { get: a, .. },
                    PropertyValue::Accessor { get: b, .. },
                ) => assert_eq!(a, b),
                _ => panic!("property kind changed"),
            }
        }
        for kind in [
            TrackedGlobal::GlobalThis,
            TrackedGlobal::Undefined,
            TrackedGlobal::Nan,
            TrackedGlobal::Infinity,
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let key = runtime.global_key(kind);
            let before = runtime.allocated;
            let value = runtime
                .binding_value(0, kind.name(), &mut document)
                .unwrap();
            assert_eq!(runtime.allocated, before);
            runtime.allocated = MAX_HEAP;
            assert!(
                runtime
                    .define_own(
                        &Value::Window,
                        &key,
                        PropertyDescriptor {
                            value: Some(value),
                            ..PropertyDescriptor::default()
                        }
                    )
                    .unwrap()
            );
            assert_eq!(runtime.allocated, MAX_HEAP);
        }
        for kind in [TrackedGlobal::WindowSelf, TrackedGlobal::GlobalThis] {
            let mut runtime = Runtime::new();
            let key = runtime.global_key(kind);
            runtime
                .define_own(
                    &Value::Window,
                    &key,
                    PropertyDescriptor::data_property(Value::Number(7.0), true, true, true),
                )
                .unwrap();
            runtime.allocated = MAX_HEAP;
            assert!(
                runtime
                    .define_own(
                        &Value::Window,
                        &key,
                        PropertyDescriptor {
                            get: Some(Value::Undefined),
                            ..PropertyDescriptor::default()
                        }
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.environments[0].bindings[kind.name()].value,
                Value::Number(7.0)
            );
            runtime.delete_property(Value::Window, &key).unwrap();
            let count = runtime.environments[0].bindings.len();
            assert!(
                runtime
                    .define_own(
                        &Value::Window,
                        &key,
                        PropertyDescriptor::data_property(Value::Number(8.0), true, true, true)
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.environments[0].bindings.len(), count);
            assert!(runtime.own_property(&Value::Window, &key).is_none());
            let prototype = runtime.prototypes["EventTarget"];
            runtime.objects[prototype].prototype = Some(Value::Object(prototype));
            assert!(
                runtime
                    .resolve_binding(1, kind.name())
                    .unwrap_err()
                    .is_resource_limit()
            );
        }
        for source in [
            "Object.defineProperty(window,'globalThis',{get:function(){return globalThis;}});try{globalThis;}catch(e){throw 'caught';}",
            "Object.defineProperty(window,'globalThis',{set:function(v){globalThis=v;}});try{globalThis=1;}catch(e){throw 'caught';}",
            "var w=window,d=Object.getOwnPropertyDescriptor(w,'globalThis');try{for(var i=0;i<10000;i++){delete w.globalThis;Object.defineProperty(w,'globalThis',d);}}catch(e){throw 'caught';}",
        ] {
            assert!(run(source).unwrap_err().is_resource_limit(), "{source}");
        }
    }

    #[test]
    fn window_self_starts_as_an_accessor_and_replaces_without_coercion_in_both_modes() {
        for strict in [false, true] {
            let (mut runtime, mut document) = property_harness();
            let source = r#"
                var w=window,d=Object.getOwnPropertyDescriptor(w,'self');
                assert.sameValue(self,w);assert.sameValue(w.self,w);
                assert.sameValue(typeof d.get,'function');assert.sameValue(typeof d.set,'function');
                assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,true);
                assert.sameValue('value' in d,false);assert.sameValue('writable' in d,false);
                verifyProperty(d.get,'name',{value:'get self',writable:false,enumerable:false,configurable:true});
                verifyProperty(d.set,'name',{value:'set self',writable:false,enumerable:false,configurable:true});
                verifyProperty(d.get,'length',{value:0,writable:false,enumerable:false,configurable:true});
                verifyProperty(d.set,'length',{value:1,writable:false,enumerable:false,configurable:true});
                assert.sameValue(Object.getPrototypeOf(d.get),Function.prototype);
                assert.sameValue(d.get.hasOwnProperty('prototype'),false);
                assert.throws(TypeError,function(){new d.get();});
                assert.throws(TypeError,function(){new d.set();});
                var poison={toString(){throw 'coerced';},valueOf(){throw 'coerced';}};
                self=poison;assert.sameValue(self,poison);assert.sameValue(w.self,poison);
                assert.sameValue(globalThis.self,poison);
                var replaced=Object.getOwnPropertyDescriptor(w,'self');
                assert.sameValue(replaced.value,poison);assert.sameValue(replaced.writable,true);
                assert.sameValue(replaced.enumerable,true);assert.sameValue(replaced.configurable,true);
                assert.sameValue('get' in replaced,false);assert.sameValue('set' in replaced,false);
                w.self=0;assert.sameValue(self,0);globalThis.self=null;assert.sameValue(self,null);
                w.self=undefined;assert.sameValue('self' in w,true);
                assert.sameValue(Object.getOwnPropertyDescriptor(w,'self').value,undefined);
                assert.sameValue(d.get.call(w),w);
            "#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn window_self_borrowed_accessors_brand_check_and_restore_deleted_properties() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,d=Object.getOwnPropertyDescriptor(w,'self');
            assert.sameValue(d.get.call(null),w);assert.sameValue(d.get.call(undefined),w);
            var get=d.get,set=d.set;assert.sameValue(get(),w);
            var bad=[{},1,'x',false,document,Object.create(w)];
            for(var i=0;i<bad.length;i++){
                assert.throws(TypeError,function(){d.get.call(bad[i]);});
                assert.throws(TypeError,function(){d.set.call(bad[i],9);});
            }
            assert.sameValue(d.set.call(null,7),undefined);assert.sameValue(self,7);
            assert.sameValue(d.set.call(undefined,8),undefined);assert.sameValue(self,8);
            assert.sameValue(set(),undefined);assert.sameValue('self' in w,true);assert.sameValue(self,undefined);
            assert.sameValue(delete w.self,true);assert.sameValue(typeof self,'undefined');
            assert.sameValue('self' in w,false);assert.sameValue(w.self,undefined);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'self'),undefined);
            assert.sameValue(delete w.self,true);assert.throws(ReferenceError,function(){return self;});
            Object.defineProperty(w,'self',d);assert.sameValue(self,w);
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'self').get,d.get);
            assert.sameValue(delete self,true);assert.sameValue(typeof self,'undefined');
            d.set.call(w,9);assert.sameValue(self,9);
            Object.defineProperty(w,'self',d);assert.sameValue(self,w);
        "#,&mut document).unwrap();
    }

    #[test]
    fn window_self_declarations_and_lexical_shadow_are_distinct_across_scripts() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute("var w=window,original=Object.getOwnPropertyDescriptor(w,'self');var self;assert.sameValue(Object.getOwnPropertyDescriptor(w,'self').get,original.get);",&mut document).unwrap();
        runtime.execute_strict("let self='lexical';assert.sameValue(self,'lexical');assert.sameValue(w.self,w);w.self='property';assert.sameValue(self,'lexical');self='changed';assert.sameValue(w.self,'property');",&mut document).unwrap();
        runtime.execute("delete w.self;assert.sameValue(self,'changed');assert.sameValue('self' in w,false);Object.defineProperty(w,'self',original);assert.sameValue(w.self,w);assert.sameValue(self,'changed');",&mut document).unwrap();
        assert_eq!(
            runtime
                .execute("var self;", &mut document)
                .unwrap_err()
                .name(),
            "SyntaxError"
        );
        assert_eq!(
            runtime
                .execute("function self(){}", &mut document)
                .unwrap_err()
                .name(),
            "SyntaxError"
        );
        assert!(
            Runtime::parse_only("var self;let self;")
                .unwrap_err()
                .is_parse_error()
        );

        let (mut runtime, mut document) = property_harness();
        runtime.execute_strict("var w=window;var self={n:1};assert.sameValue(self,w.self);assert.sameValue(Object.getOwnPropertyDescriptor(w,'self').configurable,true);",&mut document).unwrap();
        runtime.execute("delete w.self;", &mut document).unwrap();
        runtime.execute_strict("var self;assert.sameValue(self,undefined);assert.sameValue(Object.getOwnPropertyDescriptor(w,'self').configurable,false);",&mut document).unwrap();
        assert_eq!(
            runtime
                .execute("let self;", &mut document)
                .unwrap_err()
                .name(),
            "SyntaxError"
        );
        runtime.execute("function local(self){return self;}function own(){var self=3;return self;}assert.sameValue(local(2),2);assert.sameValue(own(),3);assert.sameValue(window.self,undefined);",&mut document).unwrap();

        let (mut runtime, mut document) = property_harness();
        runtime.execute("function self(){return 4;}assert.sameValue(self(),4);assert.sameValue(window.self,self);assert.sameValue(Object.getOwnPropertyDescriptor(window,'self').configurable,false);",&mut document).unwrap();
        runtime
            .execute(
                "Object.defineProperty(window,'self',{writable:false});",
                &mut document,
            )
            .unwrap();
        assert_eq!(
            runtime
                .execute("function self(){}", &mut document)
                .unwrap_err()
                .name(),
            "TypeError"
        );
    }

    #[test]
    fn window_self_descriptor_redefinitions_respect_flags_and_reentrant_accessors() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,original=Object.getOwnPropertyDescriptor(w,'self'),log='';
            function getter(){assert.sameValue(this,w);log+='g';return 4;}
            function setter(value){assert.sameValue(this,w);log+='s'+value;delete w.self;Object.defineProperty(w,'self',{value:'new',writable:true,configurable:true});}
            Object.defineProperty(w,'self',{get:getter,set:setter});
            var descriptor=Object.getOwnPropertyDescriptor(w,'self');assert.sameValue(log,'');
            assert.sameValue(descriptor.get,getter);assert.sameValue(descriptor.set,setter);
            self+=2;assert.sameValue(log,'gs6');assert.sameValue(self,'new');
            Object.defineProperty(w,'self',{get:function(){delete w.self;return 'deleted';},set:undefined,configurable:true});
            assert.sameValue(self,'deleted');assert.sameValue(typeof self,'undefined');
            var reason={};Object.defineProperty(w,'self',{get:function(){throw reason;},set:function(){throw reason;},configurable:true});
            var caught;try{self;}catch(e){caught=e;}assert.sameValue(caught,reason);
            try{self=1;}catch(e){caught=e;}assert.sameValue(caught,reason);
            Object.defineProperty(w,'self',{value:7,writable:false,configurable:true});
            self=8;assert.sameValue(self,7);
            assert.throws(TypeError,function(){'use strict';self=8;});
            Object.defineProperty(w,'self',{get:function(){return 9;},set:undefined,configurable:true});
            self=10;assert.sameValue(self,9);
            assert.throws(TypeError,function(){'use strict';w.self=10;});
            Object.defineProperty(w,'self',original);assert.sameValue(self,w);
            Object.defineProperty(w,'self',{configurable:false});
            assert.sameValue(delete w.self,false);
            assert.throws(TypeError,function(){'use strict';delete w.self;});
            assert.throws(TypeError,function(){self=2;});
            assert.throws(TypeError,function(){original.set.call(w,2);});
            assert.throws(TypeError,function(){Object.defineProperty(w,'self',{value:2});});
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'self').get,original.get);
        "#,&mut document).unwrap();
    }

    #[test]
    fn window_self_rejected_global_functions_leave_no_declarations_behind() {
        for strict in [false, true] {
            for descriptor in [
                "{configurable:false}",
                "{value:7,writable:false,configurable:false}",
                "{value:7,writable:true,enumerable:false,configurable:false}",
            ] {
                let (mut runtime, mut document) = property_harness();
                runtime
                    .execute(
                        &format!("Object.defineProperty(window,'self',{descriptor});"),
                        &mut document,
                    )
                    .unwrap();
                let before = runtime
                    .own_property(
                        &Value::Window,
                        &runtime.global_key(TrackedGlobal::WindowSelf),
                    )
                    .unwrap();
                let global_count = runtime.environments[0].bindings.len();
                let lexical_count = runtime.environments[1].bindings.len();
                let function_count = runtime.functions.len();
                let source =
                    "let freshLexical;var freshVar;function freshFunction(){}function self(){}";
                let error = if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                }
                .unwrap_err();
                assert_eq!(error.name(), "TypeError");
                assert_eq!(runtime.environments[0].bindings.len(), global_count);
                assert_eq!(runtime.environments[1].bindings.len(), lexical_count);
                assert_eq!(runtime.functions.len(), function_count);
                let after = runtime
                    .own_property(
                        &Value::Window,
                        &runtime.global_key(TrackedGlobal::WindowSelf),
                    )
                    .unwrap();
                match (before.value, after.value) {
                    (
                        PropertyValue::Data { value: before, .. },
                        PropertyValue::Data { value: after, .. },
                    ) => assert_eq!(before, after),
                    (
                        PropertyValue::Accessor { get: before, .. },
                        PropertyValue::Accessor { get: after, .. },
                    ) => assert_eq!(before, after),
                    _ => panic!("rejected declaration changed property kind"),
                }
                runtime.execute("assert.sameValue(typeof freshLexical,'undefined');assert.sameValue(typeof freshVar,'undefined');assert.sameValue(typeof freshFunction,'undefined');assert.sameValue('freshVar' in window,false);", &mut document).unwrap();
                runtime.execute("let freshLexical=1;var freshVar=2;function freshFunction(){return 3;}assert.sameValue(freshLexical+freshVar+freshFunction(),6);", &mut document).unwrap();
            }
        }
        let (mut runtime, mut document) = property_harness();
        runtime.execute("Object.defineProperty(window,'self',{value:1,writable:true,enumerable:true,configurable:false});", &mut document).unwrap();
        runtime.execute("function self(){return 7;}assert.sameValue(self(),7);assert.sameValue(Object.getOwnPropertyDescriptor(window,'self').configurable,false);", &mut document).unwrap();
        for declaration in [
            "var occupied;",
            "if(false){var occupied;}",
            "for(var occupied in {}){}",
        ] {
            let (mut runtime, mut document) = property_harness();
            runtime
                .execute(
                    "let occupied=1;Object.defineProperty(window,'self',{configurable:false});",
                    &mut document,
                )
                .unwrap();
            let source = format!("let fresh;var earlier;{declaration}function self(){{}}");
            assert_eq!(
                runtime.execute(&source, &mut document).unwrap_err().name(),
                "SyntaxError"
            );
            assert!(!runtime.environments[0].bindings.contains_key("earlier"));
            assert!(!runtime.environments[1].bindings.contains_key("fresh"));
        }
    }

    #[test]
    fn window_self_captured_references_and_inherited_properties_keep_environment_rules() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,original=Object.getOwnPropertyDescriptor(w,'self');
            function remove(){delete w.self;return 3;}
            self=remove();assert.sameValue(self,3);
            assert.throws(ReferenceError,function(){'use strict';self=remove();});
            assert.sameValue('self' in w,false);
            assert.throws(ReferenceError,function(){'use strict';self=(w.self=4);});
            assert.sameValue(w.self,4);
            (function(){'use strict';w.self=remove();})();assert.sameValue(self,3);
            delete w.self;Object.defineProperty(Object.prototype,'self',{value:'inherited',writable:true,configurable:true});
            assert.sameValue(self,'inherited');assert.sameValue(w.self,'inherited');
            assert.sameValue(Object.getOwnPropertyDescriptor(w,'self'),undefined);
            assert.sameValue(delete self,true);assert.sameValue(self,'inherited');
            self='own';assert.sameValue(Object.prototype.self,'inherited');assert.sameValue(w.self,'own');
            delete w.self;
            Object.defineProperty(Object.prototype,'self',{get:function(){assert.sameValue(this,w);return 5;},set:function(value){assert.sameValue(this,w);Object.defineProperty(w,'self',{value:value,writable:true,configurable:true});},configurable:true});
            self+=2;assert.sameValue(self,7);
            delete w.self;delete Object.prototype.self;
            Object.defineProperty(w,'self',original);
            var child=Object.create(w);
            assert.throws(TypeError,function(){return child.self;});
            assert.throws(TypeError,function(){child.self=3;});
            w.self=1;child.self=2;assert.sameValue(w.self,1);assert.sameValue(child.self,2);
            Object.defineProperty(w,'self',{value:function(){'use strict';return this;},writable:true,configurable:true});
            assert.sameValue(self(),undefined);assert.sameValue(w.self(),w);
        "#,&mut document).unwrap();
    }

    #[test]
    fn window_self_define_property_keys_use_string_hint_before_descriptor_reads() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var w=window,log='',key=[],descriptor={get value(){log+='v';return 8;},configurable:true,writable:true};
            key.toString=function(){log+='k';return 'self';};key.valueOf=function(){throw 'wrong hint';};
            Object.defineProperty(w,key,descriptor);assert.sameValue(log,'kv');assert.sameValue(self,8);
            function functionKey(){}functionKey.toString=function(){log+='f';return 'self';};
            log='';Object.defineProperty(w,functionKey,descriptor);assert.sameValue(log,'fv');
            var object={};log='';Object.defineProperty(object,{toString(){log+='o';return 'x';}},descriptor);
            assert.sameValue(log,'ov');assert.sameValue(object.x,8);
            var reason={},seen;log='';try{Object.defineProperty(w,{toString(){throw reason;}},descriptor);}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(log,'');assert.sameValue(self,8);
            Object.defineProperty(object,{toString(){return '\uD800';}},{value:1});
            Object.defineProperty(object,{toString(){return '\uFFFD';}},{value:2});
            assert.sameValue(object['\uD800'],1);assert.sameValue(object['\uFFFD'],2);
            log='';assert.throws(TypeError,function(){Object.defineProperty(null,key,descriptor);});assert.sameValue(log,'');
            var primitives=[undefined,true,1,'x'];
            for(var i=0;i<primitives.length;i++){
                assert.throws(TypeError,function(){Object.defineProperty(primitives[i],key,descriptor);});
            }
            assert.sameValue(log,'');
            Object.defineProperty(object,key,descriptor);assert.sameValue(object.self,8);assert.sameValue(log,'kv');
            log='';Object.defineProperty(object,functionKey,descriptor);assert.sameValue(log,'fv');
        "#,&mut document).unwrap();
        for key in ["'self\\u0000'", "'self\\uD800'"] {
            assert_eq!(
                run(&format!("Object.defineProperty(window,{key},{{value:1}});window[{key}]===1&&self===window")).unwrap(),
                Value::Bool(true)
            );
        }
    }

    #[test]
    fn window_self_descriptor_and_callback_limits_are_shared_and_preflight_mutation() {
        let mut runtime = Runtime::new();
        let key = runtime.global_key(TrackedGlobal::WindowSelf);
        let original = runtime.own_property(&Value::Window, &key).unwrap();
        runtime
            .define_own(
                &Value::Window,
                &key,
                PropertyDescriptor::data_property(Value::Number(7.0), true, true, true),
            )
            .unwrap();
        let PropertyValue::Accessor { get, set } = original.value else {
            panic!("initial accessor")
        };
        let count = runtime.environments[0].bindings.len();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .define_own(
                    &Value::Window,
                    &key,
                    PropertyDescriptor {
                        get: Some(get),
                        set: Some(set),
                        ..PropertyDescriptor::default()
                    }
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.environments[0].bindings.len(), count);
        let PropertyValue::Data { value, writable } =
            runtime.own_property(&Value::Window, &key).unwrap().value
        else {
            panic!("data retained")
        };
        assert_eq!(value, Value::Number(7.0));
        assert!(writable);
        runtime.steps = 0;
        assert!(
            runtime
                .define_own(
                    &Value::Window,
                    &key,
                    PropertyDescriptor::data_property(Value::Number(8.0), true, true, true)
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            runtime.environments[0].bindings["self"].value,
            Value::Number(7.0)
        );
        let mut runtime = Runtime::new();
        let key = runtime.global_key(TrackedGlobal::WindowSelf);
        runtime.delete_property(Value::Window, &key).unwrap();
        let prototype = runtime.prototypes["EventTarget"];
        runtime.objects[prototype].prototype = Some(Value::Object(prototype));
        let allocated = runtime.allocated;
        assert!(
            runtime
                .resolve_binding(1, "self")
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, allocated);
        assert!(!runtime.environments[0].bindings.contains_key("self"));
        for source in [
            "Object.defineProperty(window,'self',{get:function(){return self;}});try{self;}catch(e){throw 'caught';}",
            "Object.defineProperty(window,'self',{set:function(v){self=v;}});try{self=1;}catch(e){throw 'caught';}",
            "var d=Object.getOwnPropertyDescriptor(window,'self');try{for(var i=0;i<10000;i++){Object.defineProperty(window,'self',d);self;}}catch(e){throw 'caught';}",
            "try{Object.defineProperty(window,{toString(){while(true){}}},{value:1});}catch(e){throw 'caught';}",
        ] {
            assert!(run(source).unwrap_err().is_resource_limit(), "{source}");
        }
    }

    #[test]
    fn window_self_executes_unchanged_bound_function_case_and_keeps_other_globals_protected() {
        let source = include_str!(
            "../tests/upstream/test262-functions/test/language/statements/function/13.2-30-s.js"
        );
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
        assert_eq!(run("var w=window,d=document;window=1;document=2;undefined=3;Infinity=4;NaN=5;window===w&&document===d&&undefined===void 0&&Infinity===1/0&&NaN!==NaN").unwrap(),Value::Bool(true));
        for name in ["window", "document", "undefined", "Infinity", "NaN"] {
            assert_eq!(
                run(&format!("'use strict';{name}=1")).unwrap_err().name(),
                "TypeError"
            );
        }
    }

    #[test]
    fn is_prototype_of_follows_identity_and_internal_chain_changes() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var p={},middle=Object.create(p),child=Object.create(middle),other={};
            assert.sameValue(p.isPrototypeOf(child),true);
            assert.sameValue(middle.isPrototypeOf(child),true);
            assert.sameValue(child.isPrototypeOf(child),false);
            assert.sameValue(p.isPrototypeOf(p),false);
            assert.sameValue(other.isPrototypeOf(child),false);
            assert.sameValue(Object.prototype.isPrototypeOf(Object.create(null)),false);
            assert.sameValue(Object.prototype.isPrototypeOf(child),true);
            Object.setPrototypeOf(child,other);
            assert.sameValue(p.isPrototypeOf(child),false);
            assert.sameValue(other.isPrototypeOf(child),true);
            assert.throws(TypeError,function(){Object.setPrototypeOf(other,child);});
            assert.sameValue(Object.getPrototypeOf(other),Object.prototype);
            assert.sameValue(other.isPrototypeOf(child),true);
            Object.setPrototypeOf(child,null);
            assert.sameValue(Object.prototype.isPrototypeOf(child),false);
            function Factory(){}function FunctionParent(){}
            Factory.prototype=FunctionParent;
            assert.sameValue(FunctionParent.isPrototypeOf(new Factory()),true);
            Factory.prototype=1;
            assert.sameValue(Object.prototype.isPrototypeOf(new Factory()),true);
            assert.sameValue(Function.prototype.isPrototypeOf(Factory),true);
            assert.sameValue(Function.prototype.isPrototypeOf(()=>0),true);
            assert.sameValue(Function.prototype.isPrototypeOf(Factory.bind(null)),true);
            assert.sameValue(Function.prototype.isPrototypeOf(Object),true);
            assert.sameValue(Array.prototype.isPrototypeOf([]),true);
            assert.sameValue(Object.prototype.isPrototypeOf([]),true);
            var native=Object.prototype.isPrototypeOf;
            assert.sameValue(Function.prototype.isPrototypeOf(native),true);
            assert.sameValue(native.call(native,Object.create(native)),true);
            assert.sameValue(native.call(Object,Object.create(Object)),true);
        "#,
                &mut document,
            )
            .unwrap();
    }

    #[test]
    fn is_prototype_of_has_intrinsic_metadata_and_dynamic_alias_receivers() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var method=Object.prototype.isPrototypeOf,p={},child=Object.create(p);
            verifyProperty(Object.prototype,'isPrototypeOf',{value:method,writable:true,enumerable:false,configurable:true});
            verifyProperty(method,'name',{value:'isPrototypeOf',writable:false,enumerable:false,configurable:true});
            verifyProperty(method,'length',{value:1,writable:false,enumerable:false,configurable:true});
            assert.sameValue(method.hasOwnProperty('prototype'),false);
            assert.sameValue(Object.getPrototypeOf(method),Function.prototype);
            assert.sameValue(method.call(p,child),true);
            assert.sameValue(method.apply(p,[child]),true);
            assert.sameValue(method.bind(p)(child),true);
            assert.sameValue(method.bind(p,child)(),true);
            assert.sameValue(method.call({},child),false);
            assert.throws(TypeError,function(){new method(child);});
            assert.throws(TypeError,function(){method(child);});
            assert.sameValue(method(1),false);
            Object.prototype.isPrototypeOf=function(){return 'replacement';};
            assert.sameValue(p.isPrototypeOf(child),'replacement');
            assert.sameValue(method.call(p,child),true);
            delete Object.prototype.isPrototypeOf;
            assert.sameValue(p.isPrototypeOf,undefined);
            assert.sameValue(method.call(p,child),true);
        "#, &mut document).unwrap();
    }

    #[test]
    fn is_prototype_of_checks_argument_type_before_receiver_boxing() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var method=Object.prototype.isPrototypeOf;
            var primitives=[undefined,null,false,true,0,-0,NaN,Infinity,'','x'];
            for(var i=0;i<primitives.length;i++){
                assert.sameValue(method.call(null,primitives[i]),false);
                assert.sameValue(method.call(undefined,primitives[i]),false);
                assert.sameValue(method.call({},primitives[i]),false);
            }
            assert.sameValue(method.call(null),false);
            assert.sameValue(method.call(undefined),false);
            assert.throws(TypeError,function(){method.call(null,{});});
            assert.throws(TypeError,function(){method.call(undefined,{});});
            assert.throws(TypeError,function(){method.call(null,new Number(0));});
            assert.throws(TypeError,function(){method.call(undefined,new String(''));});
            assert.sameValue(method.call(1,Object.create(Number.prototype)),false);
            assert.sameValue(method.call(false,Object.create(Boolean.prototype)),false);
            assert.sameValue(method.call('x',Object.create(String.prototype)),false);
            var boxed=new Number(1),child=Object.create(boxed);
            assert.sameValue(method.call(boxed,child),true);
            assert.sameValue(method.call(1,child),false);
        "#,
                &mut document,
            )
            .unwrap();
    }

    #[test]
    fn is_prototype_of_does_not_read_author_properties_or_coerce_unused_arguments() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var method=Object.prototype.isPrototypeOf,reads=0,p={},child=Object.create(p);
            function poison(){reads++;throw 'property must not be read';}
            var keys=['constructor','prototype','__proto__','valueOf','toString'];
            for(var i=0;i<keys.length;i++){
                Object.defineProperty(p,keys[i],{get:poison,configurable:true});
                Object.defineProperty(child,keys[i],{get:poison,configurable:true});
            }
            Object.getPrototypeOf=poison;
            assert.sameValue(method.call(p,child),true);
            assert.sameValue(method.call(child,p),false);
            assert.sameValue(method.call(null,1),false);
            var extra={get toString(){return poison;},get valueOf(){return poison;}},log='';
            function evaluated(){log+='extra;';return extra;}
            assert.sameValue(method.call(p,child,evaluated()),true);
            assert.sameValue(log,'extra;');assert.sameValue(reads,0);
        "#,
                &mut document,
            )
            .unwrap();
    }

    #[test]
    fn is_prototype_of_shares_work_and_limits_cycles_without_retained_allocations() {
        let mut runtime = Runtime::new();
        let object = runtime.object_ordered([]).unwrap();
        let other = runtime.object_ordered([]).unwrap();
        let id = runtime.property_object(&object).unwrap();
        runtime.objects[id].prototype = Some(object.clone());
        let allocated = runtime.allocated;
        let steps = runtime.steps;
        assert!(
            runtime
                .object_is_prototype_of(other.clone(), object.clone())
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, steps - MAX_DEPTH - 1);
        assert_eq!(runtime.allocated, allocated);
        assert_eq!(runtime.objects[id].prototype, Some(object.clone()));

        runtime.objects[id].prototype = None;
        let mut tail = object;
        for _ in 1..MAX_DEPTH {
            let next = runtime.object_ordered([]).unwrap();
            let id = runtime.property_object(&next).unwrap();
            runtime.objects[id].prototype = Some(tail);
            tail = next;
        }
        let allocated = runtime.allocated;
        assert_eq!(
            runtime
                .object_is_prototype_of(other.clone(), tail.clone())
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(runtime.allocated, allocated);
        let beyond = runtime.object_ordered([]).unwrap();
        let id = runtime.property_object(&beyond).unwrap();
        runtime.objects[id].prototype = Some(tail.clone());
        let allocated = runtime.allocated;
        assert!(
            runtime
                .object_is_prototype_of(other.clone(), beyond)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, allocated);
        runtime.steps = 2;
        assert!(
            runtime
                .object_is_prototype_of(other, tail)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, allocated);

        let mut runtime = Runtime::new();
        let object = runtime.object_ordered([]).unwrap();
        let count = runtime.objects.len();
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime
                .object_is_prototype_of(Value::Number(1.0), Value::Null)
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert!(
            runtime
                .object_is_prototype_of(Value::Number(1.0), object)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.objects.len(), count);

        let error=run("var p={},c=p;for(var i=0;i<70;i++)c=Object.create(c);try{for(var j=0;j<2000;j++)p.isPrototypeOf(c);}catch(e){throw 'caught';}").unwrap_err();
        assert!(error.is_resource_limit());
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let long = Value::String(JsString::from("x".repeat(MAX_STRING)));
        runtime.steps = 1;
        assert_eq!(
            runtime
                .native_call(
                    &Native {
                        properties: None,
                        name: "Object.isPrototypeOf".into(),
                        receiver: Value::Null
                    },
                    vec![long.clone(), long],
                    &mut document
                )
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(runtime.steps, 0);
    }

    #[test]
    fn is_prototype_of_executes_unchanged_existing_function_cases() {
        for source in [
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S13.2.2_A1_T1.js"
            ),
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S13.2.2_A1_T2.js"
            ),
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S13.2.2_A3_T1.js"
            ),
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S13.2.2_A3_T2.js"
            ),
            include_str!(
                "../tests/upstream/test262-functions/test/language/statements/function/S13.2_A5.js"
            ),
        ] {
            for strict in [false, true] {
                let (mut runtime, mut document) = upstream_harness();
                if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                }
                .unwrap();
            }
        }
    }

    #[test]
    fn rest_parameters_capture_fresh_dense_actual_argument_arrays() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function all(...tail){return tail;}function after(a,b,...tail){return tail;}
            assert.sameValue(Array.isArray(all()),true);assert.sameValue(all().length,0);
            assert.sameValue(all()===all(),false);assert.sameValue(after(1).length,0);
            assert.sameValue(after(1,2).length,0);assert.sameValue(after(1,2,3,4).join(','),'3,4');
            var identity={},tail=all(undefined,null,false,0,'',identity);
            assert.sameValue(tail.length,6);assert.sameValue(0 in tail,true);assert.sameValue(tail[0],undefined);
            assert.sameValue(tail[1],null);assert.sameValue(tail[2],false);assert.sameValue(tail[3],0);
            assert.sameValue(tail[4],'');assert.sameValue(tail[5],identity);
            var first=Object.getOwnPropertyDescriptor(tail,'0');assert.sameValue(first.value,undefined);assert.sameValue(first.writable,true);assert.sameValue(first.enumerable,true);assert.sameValue(first.configurable,true);
            var length=Object.getOwnPropertyDescriptor(tail,'length');assert.sameValue(length.value,6);assert.sameValue(length.writable,true);assert.sameValue(length.enumerable,false);assert.sameValue(length.configurable,false);tail.length=5;assert.sameValue(tail.length,5);
            var savedArray=Array,calls=0;
            var priorPrototype=Object.getPrototypeOf(savedArray.prototype);Object.setPrototypeOf(savedArray.prototype,{set 0(v){calls++;}});
            Array=function(){calls++;throw 'global Array called';};
            var copied=all(7);assert.sameValue(copied[0],7);assert.sameValue(calls,0);
            assert.sameValue(Object.getPrototypeOf(copied),savedArray.prototype);
            Object.setPrototypeOf(savedArray.prototype,priorPrototype);Array=savedArray;
        "#, &mut document).unwrap();
    }

    #[test]
    fn rest_parameters_make_arguments_unmapped_without_conflating_body_scope() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function unmap(a,...tail){
                var args=arguments;assert.sameValue(args.length,3);
                args[0]=9;assert.sameValue(a,1);a=8;assert.sameValue(args[0],9);
                args[1]=7;assert.sameValue(tail[0],2);tail[1]=6;assert.sameValue(args[2],3);
                assert.throws(TypeError,function(){return args.callee;});
                assert.throws(TypeError,function(){args.callee=1;});return tail;
            }
            assert.sameValue(unmap(1,2,3).join(','),'2,6');
            function lexical(...tail){let arguments='local';return arguments+tail.length;}
            function declaration(...tail){function arguments(){return 'fn';}return arguments()+tail.length;}
            function variable(...tail){var arguments;return arguments.length+tail.length;}
            assert.sameValue(lexical(1),'local1');assert.sameValue(declaration(1),'fn1');assert.sameValue(variable(1,2),4);
            function sameBinding(...tail){var tail;var get=()=>tail;tail=[9];return get()[0];}
            assert.sameValue(sameBinding(3),9);
            function named(a,...arguments){return arguments;}
            assert.sameValue(named(1,2,3).join(','),'2,3');
            function objectAlias(...tail){arguments[0].changed=3;return tail[0].changed;}
            assert.sameValue(objectAlias({}),3);
        "#, &mut document).unwrap();
    }

    #[test]
    fn rest_parameters_follow_defaults_tdz_original_arguments_and_separate_body_vars() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function mutate(a=(arguments[1]=99),...tail){return tail;}
            assert.sameValue(mutate(undefined,7)[0],7);
            function deferred(a=()=>tail,...tail){return a();}
            assert.sameValue(deferred(undefined,4,5).join(','),'4,5');
            function separated(a=()=>tail,...tail){var tail;tail=[9];return [a()[0],tail[0]];}
            assert.sameValue(separated(undefined,4).join(','),'4,9');
            function read(a=tail,...tail){}function type(a=typeof tail,...tail){}
            function write(a=(tail=1),...tail){}function named(a=arguments,...arguments){}
            assert.throws(ReferenceError,function(){read();});assert.throws(ReferenceError,function(){type();});
            assert.throws(ReferenceError,function(){write();});assert.throws(ReferenceError,function(){named();});
            function bodyShadow(a=()=>arguments,...tail){let arguments='body';return [a().length,arguments,tail.length];}
            assert.sameValue(bodyShadow(undefined,1).join(','),'2,body,1');
            var reason={},body=false;function fail(){throw reason;}function abrupt(a=fail(),...tail){body=true;}
            var seen;try{abrupt();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(body,false);
        "#, &mut document).unwrap();
    }

    #[test]
    fn rest_parameters_preserve_arrow_method_receiver_metadata_and_call_entry_points() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function fixed(a,b,...tail){return [this,a,b,tail];}
            verifyProperty(fixed,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});
            function defaults(a,b=1,...tail){}assert.sameValue(defaults.length,1);
            var zero=(...tail)=>tail;assert.sameValue(zero.length,0);
            var receiver={},called=fixed.call(receiver,1,2,3);assert.sameValue(called[0],receiver);assert.sameValue(called[3][0],3);
            var order='',like={get length(){order+='L';return 3;},get 0(){order+='0';return 1;},get 1(){order+='1';return 2;},get 2(){order+='2';return 3;}};
            assert.sameValue(fixed.apply(receiver,like)[3][0],3);assert.sameValue(order,'L012');
            var bound=fixed.bind(receiver,1);assert.sameValue(bound.length,1);assert.sameValue(bound(2,4)[3][0],4);
            function Make(a,...tail){this.a=a;this.tail=tail;}var boundMaker=Make.bind(null,1),made=new boundMaker(2,3);
            assert.sameValue(made.a,1);assert.sameValue(made.tail.join(','),'2,3');assert.sameValue(made instanceof Make,true);
            var object={m(a,...tail){return [this,tail];}};assert.sameValue(object.m(0,3)[0],object);assert.sameValue(object.m(0,3)[1][0],3);
            assert.sameValue(object.m.name,'m');assert.sameValue(object.m.length,1);
            assert.throws(TypeError,function(){new object.m();});
            function outer(){return ((...tail)=>[this,arguments[0],tail])('tail');}
            var lexical=outer.call(receiver,'outer');assert.sameValue(lexical[0],receiver);assert.sameValue(lexical[1],'outer');assert.sameValue(lexical[2][0],'tail');
            function strictOuter(){'use strict';return function(...tail){return this;};}
            assert.sameValue(strictOuter()(),undefined);assert.sameValue(strictOuter().call(7),7);
            var prefix=(a=/[),]/.test(')'),b=`v${a}`,...tail)=>b+tail.join(',');assert.sameValue(prefix(undefined,undefined,3,4),'vtrue3,4');
            var nested=(a=(x,...rest)=>x+rest.length,...tail)=>a(3,4)+tail.length;assert.sameValue(nested(undefined,7),5);
        "#, &mut document).unwrap();
    }

    #[test]
    fn rest_parameters_reject_invalid_syntax_and_keep_unimplemented_forms_explicit() {
        for source in [
            "function f(...a=[]){}",
            "function f(...a,){}",
            "function f(...a,b){}",
            "function f(a,...a){}",
            "function f(a,a,...tail){}",
            "function f(...a.x){}",
            "function f(...(a)){}",
            "function f(...1){}",
            "function f(...){}",
            "function f(. . .a){}",
            "function f(....a){}",
            "function f(......a){}",
            "function f(... ...a){}",
            "(...a=[])=>a",
            "(...a,)=>a",
            "(...a,b)=>a",
            "(a,...a)=>a",
            "(...a)",
            "(...a)\n=>a",
            "(...a,...b)=>a",
            "(...a.x)=>a",
            "function f(...a){'use strict';}",
            "(...a)=>{'use strict';}",
            "({m(...a){'use strict';}})",
            "'use strict';function f(...a){'use strict';}",
            "'use strict';function f(...eval){}",
            "'use strict';(...arguments)=>arguments",
            "function f(...a){let a;}",
            "(...a)=>{const a=1;}",
            "({set x(...a){}})",
            "({get x(...a){}})",
        ] {
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(
                error.is_parse_error() && !error.is_unsupported(),
                "{source}: {error}"
            );
        }
        for source in [
            "function f(... a){}",
            "function f(.../* comment */a){}",
            "function f(a,...tail){}",
            "(a=1,...tail)=>tail",
            "'use strict';function f(...tail){}",
            "function f(...a){'use\\x20strict';}",
            "function f(a,){}",
            "(a,)=>a",
        ] {
            Runtime::parse_only(source).unwrap();
        }
        for source in [
            "function f(...[a]){}",
            "function f(...{a}){}",
            "(...[a])=>a",
            "(...{a})=>a",
            "function f({a},...tail){}",
        ] {
            assert!(
                Runtime::parse_only(source).unwrap_err().is_unsupported(),
                "{source}"
            );
        }
    }

    #[test]
    fn rest_parameters_preflight_array_storage_and_share_uncatchable_limits() {
        let mut runtime = Runtime::new();
        let arrays = runtime.arrays.len();
        let objects = runtime.objects.len();
        let properties = runtime.array_properties.len();
        runtime.allocated = MAX_HEAP - 64;
        assert!(runtime.rest_arguments(&[]).unwrap_err().is_resource_limit());
        assert_eq!(
            (
                runtime.arrays.len(),
                runtime.objects.len(),
                runtime.array_properties.len()
            ),
            (arrays, objects, properties)
        );
        let mut runtime = Runtime::new();
        let values = vec![Value::Undefined; 1024];
        runtime.steps = 1000;
        let before = runtime.arrays.len();
        assert!(
            runtime
                .rest_arguments(&values)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.arrays.len(), before);
        let mut runtime = Runtime::new();
        let values = vec![Value::Null; 65_537];
        assert!(
            runtime
                .rest_arguments(&values)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.arrays.len() < 10);
        for source in [
            "var caught=false;function f(...a){f();}try{f();}catch(e){caught=true;}",
            "var caught=false;try{for(var i=0;i<100000;i++){((...a)=>a)();}}catch(e){caught=true;}",
            "var caught=false;var f=(...a)=>a;try{for(var i=0;i<100000;i++){f(1,2,3,4,5);}}catch(e){caught=true;}",
            "var caught=false,body=false;var f=(...a)=>{body=true;};try{f.apply(null,{length:65536});}catch(e){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            assert!(
                runtime
                    .execute(source, &mut document)
                    .unwrap_err()
                    .is_resource_limit(),
                "{source}"
            );
            assert_eq!(runtime.lookup(0, "caught").unwrap().1, Value::Bool(false));
            if let Some((_, body)) = runtime.lookup(0, "body") {
                assert_eq!(body, Value::Bool(false));
            }
        }
    }

    #[test]
    fn default_parameters_initialize_in_order_only_for_undefined() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var log='';
            function f(a=(log+='a',1),b=(log+='b',a+1),c=(log+='c',b+1)){
                log+='body';return [a,b,c];
            }
            assert.sameValue(f().join(','),'1,2,3');assert.sameValue(log,'abcbody');
            log='';assert.sameValue(f(null,false,0).join(','),',false,0');assert.sameValue(log,'body');
            log='';assert.sameValue(f(4,undefined,9).join(','),'4,5,9');assert.sameValue(log,'bbody');
            function fresh(a=[],b={}){return [a,b];}
            var one=fresh(),two=fresh();assert.sameValue(one[0]===two[0],false);assert.sameValue(one[1]===two[1],false);
            var reason={};function fail(){log+='fail';throw reason;}
            function abrupt(a=fail(),b=(log+='later')){log+='body';}
            log='';var seen;try{abrupt();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(log,'fail');
        "#, &mut document).unwrap();
    }

    #[test]
    fn default_parameters_tdz_covers_all_bindings_and_initializer_closures() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var a='outer',b='outer',body=false;
            function selfDefault(a=a){body=true;}
            function later(a=b,b=2){body=true;}
            function laterType(a=typeof b,b=2){body=true;}
            function laterWrite(a=(b=3),b=2){body=true;}
            function calledEarly(a=()=>b,b=a()){body=true;}
            assert.throws(ReferenceError,function(){selfDefault();});
            assert.throws(ReferenceError,function(){later(undefined,8);});
            assert.throws(ReferenceError,function(){laterType();});
            assert.throws(ReferenceError,function(){laterWrite();});
            assert.throws(ReferenceError,function(){calledEarly();});
            assert.sameValue(body,false);assert.sameValue(b,'outer');
            function closure(a=()=>b,b=5){return a();}
            assert.sameValue(closure(),5);assert.sameValue(closure(undefined,9),9);
            function earlier(a=1,b=()=>a,c=(a=4)){return b();}
            assert.sameValue(earlier(),4);
        "#,
                &mut document,
            )
            .unwrap();
    }

    #[test]
    fn default_parameters_body_declarations_have_a_separate_environment() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var x='outer',fn='outer fn';
            function vars(a=()=>x){var x='body';return [a(),x];}
            function funcs(a=()=>fn){function fn(){return 'body fn';}return [a(),fn()];}
            function lexicals(a=()=>x){let x='lexical';return [a(),x];}
            assert.sameValue(vars().join(','),'outer,body');
            assert.sameValue(funcs().join(','),'outer fn,body fn');
            assert.sameValue(lexicals().join(','),'outer,lexical');
            function copy(a=1,b=()=>a){var a;assert.sameValue(a,1);a=3;return [b(),a];}
            assert.sameValue(copy().join(','),'1,3');
            function shared(a=1,b=()=>a){a=3;return b();}
            assert.sameValue(shared(),3);
            function nestedVar(a=6,b=()=>a){if(false){var a=9;}return [b(),a];}
            assert.sameValue(nestedVar().join(','),'6,6');
            function functionWins(a=1,b=()=>a){function a(){return 9;}return [b(),a()];}
            assert.sameValue(functionWins().join(','),'1,9');
            function missing(a=bodyName){var bodyName=4;return a;}
            assert.throws(ReferenceError,function(){missing();});
        "#,
                &mut document,
            )
            .unwrap();
    }

    #[test]
    fn default_parameters_arguments_are_unmapped_and_available_before_initializers() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function count(a=arguments.length,b=arguments[1]){return [a,b,arguments.length];}
            assert.sameValue(count().join(','),'0,,0');
            assert.sameValue(count(undefined,7).join(','),'2,7,2');
            function unmapped(a=1,b=(arguments[0]=8,arguments[1]=9,2)){
                assert.sameValue(a,3);assert.sameValue(b,2);a=5;
                assert.sameValue(arguments[0],8);assert.sameValue(arguments[1],9);
                return arguments;
            }
            var args=unmapped(3);assert.sameValue(args.length,1);
            verifyProperty(args,'callee',{enumerable:false,configurable:false});
            assert.throws(TypeError,function(){return args.callee;});
            assert.throws(TypeError,function(){args.callee=1;});
            function supplied(a=(arguments[1]=9),b=2){return b;}
            assert.sameValue(supplied(undefined,4),4);
            function bodyFunction(a=()=>arguments){function arguments(){return 8;}return [a().length,arguments()];}
            function bodyLexical(a=()=>arguments){let arguments='body';return [a().length,arguments];}
            function bodyVar(a=()=>arguments){var arguments;return a()===arguments;}
            assert.sameValue(bodyFunction().join(','),'0,8');assert.sameValue(bodyLexical().join(','),'0,body');
            assert.sameValue(bodyVar(),true);
            function shadow(arguments=arguments){return arguments;}
            function laterShadow(a=arguments,arguments=1){return a;}
            assert.throws(ReferenceError,function(){shadow();});assert.throws(ReferenceError,function(){laterShadow();});
            function lexicalArrow(){return ((a=arguments[0])=>a)();}
            assert.sameValue(lexicalArrow(12),12);
            function simple(a){arguments[0]=9;return a;}assert.sameValue(simple(1),9);
        "#, &mut document).unwrap();
    }

    #[test]
    fn default_parameters_receivers_lengths_names_and_accessor_forms() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function receiver(a=this){return a;}assert.sameValue(receiver(),globalThis);
            var obj={m(a=this){return a;}};assert.sameValue(obj.m(),obj);
            function maker(a=this){this.saved=a;}var made=new maker();assert.sameValue(made.saved,made);
            function outer(){'use strict';return function(a=this){return a;};}
            assert.sameValue(outer()(),undefined);assert.sameValue(outer().call(7),7);
            function arrowOuter(){return ((a=this)=>a)();}assert.sameValue(arrowOuter.call(obj),obj);
            function length(a,b=1,c){}function zero(a=1,b){}function plain(a,b,){}
            verifyProperty(length,'length',{value:1,writable:false,enumerable:false,configurable:true});
            assert.sameValue(zero.length,0);assert.sameValue(plain.length,2);
            var arrow=(a,b=1,c)=>c;assert.sameValue(arrow.length,1);
            function names(a=function(){},b=()=>0,c=function explicit(){},d=a){return [a.name,b.name,c.name,d.name];}
            assert.sameValue(names().join(','),'a,b,explicit,a');
            var unchanged=function original(){};function preserve(a=unchanged){return a.name;}
            assert.sameValue(preserve(),'original');
            var value,accessor={set item(a=17){value=[a,this];},get item(){return value[0];}};
            accessor.item=undefined;assert.sameValue(accessor.item,17);assert.sameValue(value[1],accessor);
            assert.sameValue(Object.getOwnPropertyDescriptor(accessor,'item').set.length,0);
            assert.sameValue(obj.m.length,0);assert.sameValue(obj.m.name,'m');
        "#, &mut document).unwrap();
    }

    #[test]
    fn default_parameters_cover_parser_rescans_nested_templates_and_regex_once() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var call=(a=/[),}]/.test(')'),b=`x${`${a}`}`,c=(n=2)=>n+1)=>[a,b,c()];
            assert.sameValue(call().join(','),'true,xtrue,3');
            function commas(a=(1,2),b=(()=>3)(),c={x:4}){return a+b+c.x;}
            assert.sameValue(commas(),9);
            var nested=(a=(b=`${/x/.test('x')}`)=>b)=>a();assert.sameValue(nested(),'true');
            var grouping=(1,2,3);assert.sameValue(grouping,3);
            assert.sameValue(((a,)=>a)(5),5);assert.sameValue((()=>6)(),6);
            for(var i=(x='x' in {x:1})=>x;i;){assert.sameValue(i(),true);break;}
        "#,
                &mut document,
            )
            .unwrap();
    }

    #[test]
    fn default_parameters_early_errors_strictness_and_unsupported_forms() {
        for source in [
            "function f(a=1,a){}",
            "function f(a,a=1){}",
            "(a=1,a)=>a",
            "({m(a,a){}})",
            "function f(a=1){'use strict';}",
            "(a=1)=>{'use strict';}",
            "'use strict';function f(a=1){'use strict';}",
            "({m(a=1){'use strict';}})",
            "function f(a=1){let a;}",
            "(a=1)=>{const a=2;}",
            "'use strict';function f(eval=1){}",
            "'use strict';(arguments=1)=>0",
            "function f(a=){}",
            "function f(a=1,,b){}",
            "((a))=>a",
            "(a.x=1)=>a",
            "(a+=1)=>a",
            "(a,)",
            "()",
            "(a)\n=>a",
            "a\n=>a",
            "({get item(a=1){}})",
            "(...a=[])=>{}",
            "(...a,)=>{}",
            "(a,...b=1)=>{}",
            "(...a,b)=>{}",
            "(...a.x)=>{}",
            "(. . .a)=>{}",
            "function f(...a=[]){}",
            "function f(...a,){}",
            "function f(....a){}",
            "(....a)=>a",
            "function f(......a){}",
            "function f(... ...a){}",
            "({m(...a=1){}})",
        ] {
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(
                error.is_parse_error() && !error.is_unsupported(),
                "{source}: {error}"
            );
        }
        for source in [
            "function f(a,a){}",
            "function f(a=1){'use\\x20strict';return a;}",
            "'use strict';function f(a=1){return a;}",
            "function f(a=1){; 'use strict';}",
            "(a=1)=>a",
            "function f(a=1,){}",
        ] {
            Runtime::parse_only(source).unwrap();
        }
        for source in [
            "function f({a}){}",
            "function f([a]){}",
            "({a})=>a",
            "([a])=>a",
        ] {
            assert!(
                Runtime::parse_only(source).unwrap_err().is_unsupported(),
                "{source}"
            );
        }
    }

    #[test]
    fn default_parameters_share_limits_and_never_run_body_after_abrupt_initialization() {
        for source in [
            "function f(a=f()){}f();",
            "var caught=false,body=false;function forever(){while(true){}}function f(a=forever()){body=true;}try{f();}catch(e){caught=true;}",
            "var caught=false,body=false;function f(a=(function(){while(true){}})()){body=true;}try{f();}catch(e){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            assert!(
                runtime
                    .execute(source, &mut document)
                    .unwrap_err()
                    .is_resource_limit(),
                "{source}"
            );
            if let Some((_, caught)) = runtime.lookup(0, "caught") {
                assert_eq!(caught, Value::Bool(false));
            }
            if let Some((_, body)) = runtime.lookup(0, "body") {
                assert_eq!(body, Value::Bool(false));
            }
        }
        let nested = format!("function f(a={}1{}){{}}", "(".repeat(100), ")".repeat(100));
        Runtime::parse_only(&nested).unwrap();
        let parameters = (0..1000)
            .map(|i| format!("p{i}{}=1", "x".repeat(36)))
            .collect::<Vec<_>>()
            .join(",");
        let locals = (0..1000)
            .map(|i| format!("q{i}{}", "x".repeat(36)))
            .collect::<Vec<_>>()
            .join(",");
        let wide = format!("function f({parameters}){{let {locals};}}");
        assert!(Runtime::parse_only(&wide).unwrap_err().is_resource_limit());
        let program = Parser::program("function f(a={value:'retained'}){} ").unwrap();
        let Stmt::Function(_, code) = &program.body[0] else {
            panic!("function AST");
        };
        let copied = code.clone();
        assert!(Rc::ptr_eq(
            code.params[0].initializer.as_ref().unwrap(),
            copied.params[0].initializer.as_ref().unwrap()
        ));
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP - 32;
        let before = runtime.functions.len();
        assert!(
            runtime
                .function_value(&code::test_function(code), 1)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.functions.len(), before);
    }

    #[test]
    fn inline_style_aliases_existence_and_indexed_reads_agree() {
        assert_eq!(
            run(r#"
            var style = document.body.style;
            style.backgroundColor = 'red';
            style.cssFloat = 'left';
            style.setProperty('--Tone', 'blue');
            ('backgroundColor' in style) && ('background-color' in style) &&
            ('cssFloat' in style) && ('float' in style) && ('color' in style) &&
            !('COLOR' in style) && !('--Tone' in style) &&
            !('inventedStyle' in style) && style.inventedStyle === undefined &&
            style['--Tone'] === undefined && style.color === '' &&
            style['background-color'] === 'red' && style.float === 'left' &&
            style.getPropertyValue('--Tone') === 'blue' &&
            style.length === 3 && style[0] === 'background-color' &&
            ('0' in style) && !('3' in style) && !('01' in style) &&
            style[3] === undefined && style.item(3) === '' &&
            style.item(4294967296) === 'background-color' &&
            style.parentRule === null;
        "#)
            .unwrap(),
            Value::Bool(true)
        );
        for source in [
            "document.body.style.inventedStyle = 'red'",
            "Object.getOwnPropertyDescriptor(document.body.style, 'color')",
        ] {
            assert!(run(source).unwrap_err().is_unsupported(), "{source}");
        }
        assert_eq!(run("'use strict'; try { document.body.style.length = 3; } catch(e) { e instanceof TypeError; }").unwrap(), Value::Bool(true));
    }

    #[test]
    fn inline_style_methods_have_metadata_brand_and_argument_conversion() {
        assert_eq!(
            run(r#"
            var style = document.body.style;
            var other = document.createElement('i').style;
            var method = style.setProperty;
            var name = Object.getOwnPropertyDescriptor(method, 'name');
            var length = Object.getOwnPropertyDescriptor(method, 'length');
            var order = '';
            var text = {toString(){order += 'x'; return 'color';}};
            var errors = 0;
            try { method(text, text); } catch(e) { if(e instanceof TypeError) errors++; }
            try { style.setProperty(text); } catch(e) { if(e instanceof TypeError) errors++; }
            method.call(other, 'color', 'red', undefined, text);
            var required = [style.getPropertyValue, style.getPropertyPriority,
                            style.removeProperty, style.item];
            for(var i = 0; i < required.length; i++) {
                try { required[i].call(style); } catch(e) { if(e instanceof TypeError) errors++; }
            }
            errors === 6 && order === '' && other.color === 'red' && style.color === '' &&
            method === other.setProperty && method.name === 'setProperty' && method.length === 2 &&
            !('prototype' in method) && !name.writable && !name.enumerable && name.configurable &&
            !length.writable && !length.enumerable && length.configurable &&
            style.getPropertyValue.length === 1 && style.item.length === 1;
        "#)
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn inline_style_author_conversions_precede_snapshot_and_short_circuit_throws() {
        assert_eq!(run(r#"
            var style = document.body.style;
            var order = '';
            style.setProperty(
                {toString(){order += 'p'; return 'COLOR';}},
                {toString(){order += 'v'; style.setProperty('--saved', 'blue'); return 'red';}},
                {toString(){order += 'i'; return 'IMPORTANT';}}
            );
            var first = order === 'pvi' && style.color === 'red' &&
                style.getPropertyValue('--saved') === 'blue' && style.getPropertyPriority('color') === 'important';
            order = '';
            style.setProperty({toString(){order += 'p'; return 'bogus';}},
                {toString(){order += 'v'; return 'red';}},
                {toString(){order += 'i'; return '';}});
            var invalid = order === 'pvi';
            order = '';
            try { style.setProperty('color', {toString(){order += 'v'; throw 42;}},
                {toString(){order += 'i'; return '';}}); } catch(e) { order += e; }
            first && invalid && order === 'v42' && style.color === 'red';
        "#).unwrap(), Value::Bool(true));
    }

    #[test]
    fn inline_style_priorities_null_undefined_and_case_are_distinct() {
        assert_eq!(run(r#"
            var style = document.body.style;
            style.setProperty('COLOR', 'red', 'IMPORTANT');
            style.setProperty('color', 'blue', ' important ');
            var initial = style.color === 'red' && style.getPropertyPriority('cOlOr') === 'important';
            style.setProperty('color', 'blue', undefined);
            var reset = style.color === 'blue' && style.getPropertyPriority('color') === '';
            style.setProperty('--Tone', undefined);
            style.setProperty('--tone', 'red');
            var cases = style.getPropertyValue('--Tone') === 'undefined' && style.getPropertyValue('--tone') === 'red';
            style.color = null;
            style.setProperty('--tone', null, 'invalid');
            var removed = style.color === '' && style.getPropertyValue('--tone') === '';
            style.setProperty('color', 'red', null);
            style.setProperty('color', '', 'invalid');
            initial && reset && cases && removed && style.color === '';
        "#).unwrap(), Value::Bool(true));
    }

    #[test]
    fn inline_style_noop_removal_does_not_create_or_normalize_attributes() {
        let mut doc = Document::parse(
            "<body><div id='raw' style='color:red; COLOR:blue !important; bogus:yes; color:green'></div></body>",
        );
        let raw = doc.query_selector("#raw").unwrap();
        let original = doc.attr(raw, "style").unwrap().to_owned();
        let mut runtime = Runtime::new();
        assert_eq!(
            runtime
                .execute(
                    r#"
            var body = document.body.style;
            var raw = document.getElementById('raw').style;
            body.removeProperty('color') === '' && raw.removeProperty('width') === '';
        "#,
                    &mut doc
                )
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(doc.attr(doc.query_selector("body").unwrap(), "style"), None);
        assert_eq!(doc.attr(raw, "style"), Some(original.as_str()));
        assert_eq!(
            runtime
                .execute("raw.removeProperty('color')", &mut doc)
                .unwrap(),
            Value::String("blue".into())
        );
        assert_eq!(doc.attr(raw, "style"), Some(""));
        doc.set_attr(raw, "style", "--empty: ; color: red");
        assert_eq!(
            runtime
                .execute("raw.removeProperty('--empty')", &mut doc)
                .unwrap(),
            Value::String("".into())
        );
        assert!(!doc.attr(raw, "style").unwrap().contains("--empty"));
    }

    #[test]
    fn inline_style_token_boundaries_and_utf16_css_boundary_are_preserved() {
        assert_eq!(
            run(r#"
            var style = document.body.style;
            style.cssText = '--note: "a;b:c"; color:red; color:blue !important; color:green';
            var first = style.color === 'blue' && style.getPropertyValue('--note') === '"a;b:c"';
            style.setProperty('color', 'red; width: 300px');
            style.setProperty('color', 'red !important');
            var invalid = style.color === 'blue' && style.width === '';
            style.setProperty('--unicode', '\uD800');
            var unicode = style.getPropertyValue('--unicode') === '\uFFFD';
            var absent = style['\uD800'] === undefined && !('\uD800' in style);
            style.cssText = {toString(){return 'color: red';}};
            var canonical = style.cssText === 'color: red;';
            style.cssText = null;
            first && invalid && unicode && absent && canonical && style.length === 0;
        "#)
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn inline_style_quota_failures_are_uncatchable_and_leave_dom_unchanged() {
        let mut doc = Document::parse("<body style='color: red'></body>");
        let body = doc.query_selector("body").unwrap();
        let mut runtime = Runtime::new();
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP - 100;
        let error = runtime
            .style_set(body, "color", Value::String("blue".into()), false, &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(doc.attr(body, "style"), Some("color: red"));
        let mut runtime = Runtime::new();
        let error = runtime.execute("try { for(var n=0;n<10000;n++) document.body.style.color; } catch(e) { document.body.textContent='caught'; }", &mut doc).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(doc.text_content(body), "");
        assert_eq!(doc.attr(body, "style"), Some("color: red"));
        doc.set_attr(
            body,
            "style",
            &format!("--large: {}", "x".repeat(crate::cssom::MAX_INLINE_BYTES)),
        );
        let original = doc.attr(body, "style").unwrap().to_owned();
        let error = Runtime::new()
            .execute("document.body.style.color='blue'", &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(doc.attr(body, "style"), Some(original.as_str()));
    }

    #[test]
    fn member_keys_use_string_hint_at_reference_consumption_and_keep_utf16() {
        assert_eq!(run(r#"
            var order = '';
            var style = document.body.style;
            var key = [];
            key.toString = function(){order += 'k'; return 'backgroundColor';};
            key.valueOf = function(){throw 'wrong hint';};
            function receiver(){order += 'r'; return style;}
            function value(){order += 'v'; return 'red';}
            receiver()[key] = value();
            var assigned = order === 'rvk';
            order = '';
            var read = receiver()[key];
            var readOrder = order === 'rk';
            function methodKey(){}
            methodKey.toString = function(){order += 'm'; return 'setProperty';};
            order = '';
            receiver()[methodKey]('color', value());
            var callOrder = order === 'rmv';
            var plain = {};
            var scalar = {toString(){return 'x';}, valueOf(){throw 'wrong hint';}};
            plain[scalar] = 4;
            var ordinary = plain.x === 4 && (scalar in plain) && delete plain[scalar] && !('x' in plain);
            var nonscalar = [];
            nonscalar.toString = function(){return '\uD800';};
            plain[nonscalar] = 7;
            plain['\uFFFD'] = 8;
            var units = plain[nonscalar] === 7 && plain['\uFFFD'] === 8 && delete plain[nonscalar] && plain['\uD800'] === undefined;
            Object.defineProperty(Object.prototype, '\uD800', {value: 19, configurable: true});
            units = units && ('\uD800' in style) && style['\uD800'] === 19 && style['\uFFFD'] === undefined;
            order = '';
            try { receiver()[{toString(){order += 'e'; throw 5;}}] = value(); } catch(e) { order += e; }
            assigned && readOrder && read === 'red' && callOrder && style.color === 'red' &&
            ordinary && units && order === 'rve5';
        "#).unwrap(), Value::Bool(true));
    }

    #[test]
    fn member_references_defer_plain_assignment_and_cache_compound_and_update_keys() {
        assert_eq!(run(r#"
            var order = '';
            var object = {};
            var name = 'before';
            var key = {toString(){order += 'k'; return name;}};
            function rhs(){order += 'r'; name = 'after'; return 3;}
            object[key] = rhs();
            var plain = order === 'rk' && object.after === 3 && object.before === undefined;
            object.after = 4;
            order = '';
            function compound(){order += 'r'; name = 'other'; return 2;}
            object[key] += compound();
            var compoundResult = order === 'kr' && object.after === 6 && object.other === undefined;
            name = 'after';
            object.after = {valueOf(){order += 'v'; name = 'other'; return 10;}};
            order = '';
            var previous = object[key]++;
            var update = previous === 10 && order === 'kv' && object.after === 11 && object.other === undefined;
            order = '';
            try { object[key] = (function(){order += 'r'; throw 7;})(); } catch(e) { order += e; }
            plain && compoundResult && update && order === 'r7';
        "#).unwrap(), Value::Bool(true));
    }

    #[test]
    fn member_null_bases_throw_before_key_coercion_but_after_assignment_rhs() {
        for (expression, expected) in [
            ("null[key]", "T"),
            ("undefined[key]", "T"),
            ("null[key](rhs())", "T"),
            ("delete null[key]", "T"),
            ("null[key]++", "T"),
            ("null[key] += rhs()", "T"),
            ("null[key] = rhs()", "rT"),
            ("null[(order += 'e', key)]", "eT"),
            ("null[(order += 'e', key)] = rhs()", "erT"),
            ("null[key] = (function(){order += 'r'; throw 2;})()", "r2"),
        ] {
            let source = format!(
                r#"
                var order = '';
                var key = {{toString(){{order += 'k'; return 'x';}}}};
                function rhs(){{order += 'r'; return 1;}}
                try {{ {expression}; }} catch(e) {{ order += e instanceof TypeError ? 'T' : e; }}
                order;
            "#
            );
            assert_eq!(
                run(&source).unwrap(),
                Value::String(expected.into()),
                "{expression}"
            );
        }
    }

    #[test]
    fn window_descriptors_expose_current_global_binding_records_only() {
        assert_eq!(
            run(r#"
            var namespace = CSS;
            var descriptor = Object.getOwnPropertyDescriptor(window, 'CSS');
            var independent = Object.getOwnPropertyDescriptor(self, 'CSS');
            descriptor.value = 12;
            descriptor.writable = false;
            var declared = 7;
            let lexical = 8;
            const constant = 9;
            window.assigned = 10;
            var declaration = Object.getOwnPropertyDescriptor(globalThis, 'declared');
            var assignment = Object.getOwnPropertyDescriptor(window, 'assigned');
            var inheritedReads = 0;
            Object.defineProperty(EventTarget.prototype, 'inheritedDescriptorProbe', {
                get(){ inheritedReads++; return 17; }, configurable: true
            });
            CSS = 21;
            var replacement = Object.getOwnPropertyDescriptor(window, 'CSS');
            independent !== descriptor && independent.value === namespace &&
            independent.writable && !independent.enumerable && independent.configurable &&
            Object.getPrototypeOf(independent) === Object.prototype &&
            Object.keys(independent).join(',') === 'value,writable,enumerable,configurable' &&
            replacement.value === 21 && replacement.writable &&
            !replacement.enumerable && replacement.configurable &&
            declaration.value === 7 && declaration.writable && declaration.enumerable &&
            !declaration.configurable && assignment.value === 10 && assignment.writable &&
            assignment.enumerable && assignment.configurable &&
            Object.getOwnPropertyDescriptor(window, 'lexical') === undefined &&
            Object.getOwnPropertyDescriptor(window, 'constant') === undefined &&
            Object.getOwnPropertyDescriptor(window, 'this') === undefined &&
            Object.getOwnPropertyDescriptor(window, 'absentDescriptorProbe') === undefined &&
            Object.getOwnPropertyDescriptor(window, 'inheritedDescriptorProbe') === undefined &&
            inheritedReads === 0;
        "#)
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn window_descriptors_follow_deletion_recreation_and_key_conversion_order() {
        assert_eq!(run(r#"
            var saved = CSS;
            var removed = delete window.CSS;
            var absent = Object.getOwnPropertyDescriptor(window, 'CSS') === undefined;
            window.CSS = saved;
            var recreated = Object.getOwnPropertyDescriptor(window, 'CSS');
            var trace = '';
            var key = {
                get toString(){
                    trace += 'get-string;';
                    return function(){trace += 'call-string;'; return {};};
                },
                get valueOf(){
                    trace += 'get-value;';
                    return function(){trace += 'call-value;'; window.CSS = 23; return 'CSS';};
                }
            };
            var observed = Object.getOwnPropertyDescriptor(window, key);
            var array = ['wrong']; array.toString = function(){return 'CSS';};
            var joined = ['wrong']; joined.join = function(){return 'CSS';};
            function callableKey(){} callableKey.toString = function(){return 'CSS';};
            var sentinel = {}, threw = false, rejectedNull = false, convertedNull = false;
            try { Object.getOwnPropertyDescriptor(window, {toString(){throw sentinel;}}); }
            catch(error) { threw = error === sentinel; }
            try { Object.getOwnPropertyDescriptor(null, {toString(){convertedNull=true;return 'CSS';}}); }
            catch(error) { rejectedNull = error instanceof TypeError; }
            var deletedDuringKey = Object.getOwnPropertyDescriptor(window, {
                toString(){delete window.CSS; return 'CSS';}
            });
            window.CSS = 23;
            removed && absent && recreated.value === saved && recreated.writable &&
            recreated.enumerable && recreated.configurable && observed.value === 23 &&
            trace === 'get-string;call-string;get-value;call-value;' &&
            Object.getOwnPropertyDescriptor(window, array).value === 23 &&
            Object.getOwnPropertyDescriptor(window, joined).value === 23 &&
            Object.getOwnPropertyDescriptor(window, callableKey).value === 23 &&
            threw && rejectedNull && !convertedNull && deletedDuringKey === undefined;
        "#).unwrap(), Value::Bool(true));
        assert_eq!(
            run("Object.getOwnPropertyDescriptor(window, {toString(){return {};},valueOf(){return {};}})")
                .unwrap_err().name(),
            "TypeError"
        );
    }

    #[test]
    fn window_descriptor_keys_preserve_utf16_and_primitive_conversion() {
        assert_eq!(
            run(r#"
            window['\uFFFD'] = 1;
            window['\uD83D\uDE00'] = 2;
            window[''] = 3;
            window['17'] = 4;
            window['null'] = 5;
            Object.getOwnPropertyDescriptor(window, '\uD800') === undefined &&
            Object.getOwnPropertyDescriptor(window, '\uDC00') === undefined &&
            Object.getOwnPropertyDescriptor(window, {toString(){return '\uD800';}}) === undefined &&
            Object.getOwnPropertyDescriptor(window, '\uFFFD').value === 1 &&
            Object.getOwnPropertyDescriptor(window, '\uD83D\uDE00').value === 2 &&
            Object.getOwnPropertyDescriptor(window, '').value === 3 &&
            Object.getOwnPropertyDescriptor(window, 17).value === 4 &&
            Object.getOwnPropertyDescriptor(window, null).value === 5 &&
            Object.getOwnPropertyDescriptor(window).value === undefined &&
            Object.getOwnPropertyDescriptor({CSS: 6}, {toString(){return 'CSS';}}).value === 6;
        "#)
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn window_descriptor_increment_preserves_explicit_host_limitations() {
        assert_eq!(
            run("Object.defineProperty(window, 'CSS', {value: 1});CSS").unwrap(),
            Value::Number(1.0)
        );
        for source in [
            "Object.getOwnPropertyDescriptor(window, 'onclick')",
            "window.onclick = function(){}; Object.getOwnPropertyDescriptor(window, 'onclick')",
            "Object.defineProperty(window, 'onclick', {value: 1})",
            "delete window.onclick",
            "window.onclick = function(){}; delete window.onclick",
        ] {
            assert!(run(source).unwrap_err().is_unsupported(), "{source}");
        }
        for source in [
            "Object.getOwnPropertyDescriptor(document, 'title')",
            "Object.getOwnPropertyDescriptor(document.createElement('div'), 'textContent')",
        ] {
            assert_eq!(run(source).unwrap(), Value::Undefined, "{source}");
        }
        assert_eq!(
            run("Object.getOwnPropertyNames(window).indexOf('CSS') >= 0").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn window_descriptor_lookup_and_results_share_resource_limits() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<body></body>");
        let key = "x".repeat(4096);
        let error = runtime.execute(&format!(
            "var caught=false; try {{ for(var i=0;i<32;i++) Object.getOwnPropertyDescriptor(window,'{key}'); }} catch(error) {{caught=true;}}"
        ), &mut doc).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);

        let mut runtime = Runtime::new();
        let descriptor = Native {
            properties: None,
            name: "Object.getOwnPropertyDescriptor".into(),
            receiver: Value::Undefined,
        };
        runtime.allocated = MAX_HEAP - 16;
        assert!(
            runtime
                .native_call(
                    &descriptor,
                    vec![Value::Window, Value::String("CSS".into())],
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            runtime
                .own_property(&Value::Window, &"CSS".into())
                .is_some()
        );

        let mut runtime = Runtime::new();
        runtime.steps = 2;
        assert!(
            runtime
                .native_call(
                    &descriptor,
                    vec![Value::Window, Value::String("CSS".into())],
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        let mut runtime = Runtime::new();
        let error = runtime.execute(
            "var key={toString(){return Object.getOwnPropertyDescriptor(window,key);}}; Object.getOwnPropertyDescriptor(window,key);",
            &mut doc).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
    }

    #[test]
    fn css_supports_namespace_function_and_overload_metadata() {
        let runtime = Runtime::new();
        let global = runtime.own_property(&Value::Window, &"CSS".into()).unwrap();
        assert!(!global.enumerable);
        assert!(global.configurable);
        assert!(matches!(
            global.value,
            PropertyValue::Data { writable: true, .. }
        ));
        assert_eq!(
            run(r#"
                var method = CSS.supports;
                var property = Object.getOwnPropertyDescriptor(CSS, 'supports');
                var length = Object.getOwnPropertyDescriptor(method, 'length');
                var name = Object.getOwnPropertyDescriptor(method, 'name');
                var missing = false, constructed = false;
                try { method(); } catch (error) { missing = error instanceof TypeError; }
                try { new method('(display:grid)'); }
                catch (error) { constructed = error instanceof TypeError; }
                typeof CSS === 'object' && CSS === window.CSS && CSS === globalThis.CSS &&
                Object.getPrototypeOf(CSS) === Object.prototype &&
                Object.prototype.toString.call(CSS) === '[object CSS]' &&
                Object.getPrototypeOf(method) === Function.prototype &&
                name.value === 'supports' && !name.writable && !name.enumerable && name.configurable &&
                length.value === 1 && !length.writable && !length.enumerable && length.configurable &&
                property.value === method && property.writable && property.enumerable && property.configurable &&
                !method.hasOwnProperty('prototype') && missing && constructed &&
                method.call(null, 'display', 'grid') && method.call(17, 'display:grid') &&
                !method(undefined) && !method(null) && !method('display:grid', undefined);
            "#).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            run(r#"
                var saved = CSS, method = CSS.supports;
                CSS.supports = 3;
                var replaced = CSS.supports === 3;
                delete CSS.supports;
                CSS.extra = 7;
                CSS = null;
                var assigned = window.CSS === null;
                window.CSS = saved;
                var removed = delete window.CSS;
                replaced && assigned && removed && typeof CSS === 'undefined' &&
                !saved.hasOwnProperty('supports') && saved.extra === 7 && method('display:grid');
            "#)
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn css_supports_converts_left_to_right_without_skipping_or_extra_coercions() {
        assert_eq!(
            run(r#"
                var trace = '', sentinel = {}, same = false;
                function argument(label, result) {
                    trace += 'evaluate-' + label + ';';
                    return {toString() { trace += 'convert-' + label + ';'; return result; }};
                }
                var supported = CSS.supports(argument('property', 'display'),
                    argument('value', 'grid'), argument('extra', 'never'));
                var ordered = trace === 'evaluate-property;evaluate-value;evaluate-extra;convert-property;convert-value;';
                trace = '';
                try { CSS.supports({toString(){trace += 'first'; throw sentinel;}},
                    {toString(){trace += 'second'; return 'grid';}}); }
                catch (error) { same = error === sentinel; }
                var firstThrow = same && trace === 'first';
                trace = '';
                try { CSS.supports('not-a-property', {toString(){trace += 'value'; throw sentinel;}}); }
                catch (error) { same = error === sentinel; }
                var invalidConverted = same && trace === 'value';
                var array = ['incorrect'];
                array.toString = function(){ return '(display:grid)'; };
                var fallback = {toString(){return {};}, valueOf(){return 'display:grid';}};
                supported && ordered && firstThrow && invalidConverted &&
                CSS.supports(array) && CSS.supports(fallback) &&
                !CSS.supports(true) && !CSS.supports(17) &&
                CSS.supports('display', {toString(){return 'grid';}},
                    {toString(){throw 'extra argument was converted';}});
            "#).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            run("CSS.supports({toString(){return {};}, valueOf(){return {};}})")
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(run(r#"
            var trace = '';
            var condition = {
                get toString(){trace += 'get-string;'; return function(){trace += 'call-string;'; return {};};},
                get valueOf(){trace += 'get-value;'; return function(){trace += 'call-value;'; return 'display:grid';};}
            };
            CSS.supports(condition) && trace === 'get-string;call-string;get-value;call-value;';
        "#).unwrap(), Value::Bool(true));
    }

    #[test]
    fn css_supports_separates_properties_values_and_condition_grammars() {
        assert_eq!(
            run(r#"
                CSS.supports('display:grid') && CSS.supports('(display:grid)') &&
                CSS.supports('DiSpLaY', ' /* before */ grid /* after */ ') &&
                CSS.supports('display:grid !important') &&
                CSS.supports('selector(div/**/.item)') &&
                CSS.supports('not selector(div/**/span)') &&
                !CSS.supports('not selector(svg|rect)') &&
                !CSS.supports('selector(div) or selector(svg|rect)') &&
                !CSS.supports(' display', 'grid') && !CSS.supports('display ', 'grid') &&
                !CSS.supports('display/**/', 'grid') && !CSS.supports('dis\\play', 'grid') &&
                !CSS.supports('display', 'grid !important') &&
                !CSS.supports('display', 'grid; color:red') &&
                !CSS.supports('display', 'grid) or (color:red') &&
                !CSS.supports('display:grid) or (color', 'red') &&
                !CSS.supports('display', 'g/**/rid') &&
                !CSS.supports('width', '1 px') &&
                !CSS.supports('--theme', 'blue') && !CSS.supports('position', 'sticky') &&
                CSS.supports('selector([data-name="\ud800"])') &&
                CSS.supports('selector([data-name="\ud800"])') ===
                    CSS.supports('selector([data-name="\ufffd"])') &&
                CSS.supports('selector([data-name="\ud83d\ude00"])');
            "#)
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn css_supports_uses_shared_work_heap_and_string_limits() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<body></body>");
        assert_eq!(runtime.execute(
            "var count = 0; for(var i=0;i<100;i++){if(CSS.supports('display','grid')) count++;} count;",
            &mut doc).unwrap(), Value::Number(100.0));
        let oversized = "x".repeat(crate::css::MAX_SUPPORTS_BYTES + 1);
        assert_eq!(runtime.execute(&format!(
            "var converted = false; var accepted = CSS.supports('{oversized}', {{toString(){{converted=true;return 'grid';}}}}); !accepted && converted;"
        ), &mut doc).unwrap(), Value::Bool(true));

        let query = format!("(display:grid){}", " ".repeat(4096));
        let mut runtime = Runtime::new();
        let error = runtime.execute(&format!(
            "var caught = false; try {{ for(var i=0;i<100;i++) CSS.supports('{query}'); }} catch(error) {{caught=true;}}"
        ), &mut doc).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);

        let mut runtime = Runtime::new();
        runtime.steps = 20;
        let error = runtime
            .css_supports(&[Value::String("display:grid".into())], &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.steps, 0);
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP - 64;
        assert!(
            runtime
                .css_supports(&[Value::String("display:grid".into())], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        let mut runtime = Runtime::new();
        let error = runtime.execute(
            "var recursive={toString(){return CSS.supports(recursive);}}; CSS.supports(recursive);",
            &mut doc,
        ).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
    }

    #[test]
    fn css_supports_drives_dom_changes_and_event_callbacks() {
        let mut doc = Document::parse("<button id=probe>Probe</button><output id=result></output>");
        let mut runtime = Runtime::new();
        runtime.execute(r#"
            var result = document.getElementById('result');
            result.textContent = CSS.supports('display', 'grid') ? 'grid available' : 'fallback';
            document.getElementById('probe').addEventListener('click', function(){
                result.className = CSS.supports('selector(output/**/.ready)') ? 'ready' : 'fallback';
                result.textContent = CSS.supports('position', 'sticky') ? 'sticky' : 'static';
            });
        "#, &mut doc).unwrap();
        let result = doc.query_selector("#result").unwrap();
        assert_eq!(doc.text_content(result), "grid available");
        runtime
            .dispatch_click(doc.query_selector("#probe").unwrap(), &mut doc)
            .unwrap();
        assert_eq!(doc.attr(result, "class"), Some("ready"));
        assert_eq!(doc.text_content(result), "static");
    }

    #[test]
    fn arithmetic_variables_functions_and_loops() {
        assert_eq!(run("let total = 0; for (let i = 0; i < 10; i++) { total += i; } function twice(x) { return x * 2; } twice(total);").unwrap(), Value::Number(90.0));
        assert_eq!(
            run("const square = x => x * x; [1,2,3].map(square).join('-');")
                .unwrap()
                .to_string(),
            "1-4-9"
        );
        assert_eq!(
            run(
                "function counter() { let x = 0; return () => ++x; } const c = counter(); c(); c();"
            )
            .unwrap(),
            Value::Number(2.0)
        );
    }

    #[test]
    fn dom_button_counter_and_event_bubbling() {
        let mut doc = Document::parse(
            "<body><button id='button'><span>Count</span></button><p id='count'>0</p></body>",
        );
        let mut runtime = Runtime::new();
        runtime.execute("let count = 0; const out = document.getElementById('count'); document.querySelector('#button').addEventListener('click', () => { count++; out.textContent = String(count); out.style.color = 'red'; });", &mut doc).unwrap();
        let target = doc.query_selector("span").unwrap();
        runtime.dispatch_click(target, &mut doc).unwrap();
        runtime.dispatch_click(target, &mut doc).unwrap();
        let counter = doc.query_selector("#count").unwrap();
        assert_eq!(doc.text_content(counter), "2");
        assert_eq!(
            runtime.execute("out.style.color", &mut doc).unwrap(),
            Value::String("red".into())
        );
    }

    #[test]
    fn dom_mutation_ready_and_inline_handler() {
        let mut doc = Document::parse(
            "<body><button onclick=\"this.textContent = 'done'; return false\">Go</button><div id='out'></div></body>",
        );
        let mut runtime = Runtime::new();
        runtime.execute("document.addEventListener('DOMContentLoaded', () => { const p = document.createElement('p'); p.textContent = 'ready'; p.classList.add('active'); document.getElementById('out').appendChild(p); });", &mut doc).unwrap();
        runtime.dispatch_dom_content_loaded(&mut doc).unwrap();
        assert_eq!(
            doc.text_content(doc.query_selector("p.active").unwrap()),
            "ready"
        );
        let button = doc.query_selector("button").unwrap();
        runtime.dispatch_click(button, &mut doc).unwrap();
        assert_eq!(doc.text_content(button), "done");
        assert!(runtime.last_default_prevented);
    }

    #[test]
    fn limits_stop_loops_recursion_nesting_and_growth() {
        assert!(
            run("while (true) {};")
                .unwrap_err()
                .message
                .contains("limit")
        );
        assert!(
            run("function recur() { return recur(); } recur();")
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            run(&format!("{}1{}", "(".repeat(1000), ")".repeat(1000))).unwrap(),
            Value::Number(1.0)
        );
        assert!(
            Runtime::parse_only(&format!("{}1{}", "(".repeat(16_000), ")".repeat(16_000)))
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            run("let s = 'x'; while (true) { s = s + s; }")
                .unwrap_err()
                .message
                .contains("limit")
        );
        assert_eq!(
            run("const a = []; a[4294967294] = 1;").unwrap(),
            Value::Number(1.0)
        );
        assert!(
            run("const a = []; while (true) { a.push(1); }")
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn no_host_capabilities_and_const_is_enforced() {
        assert!(
            run("fetch('https://example.com')")
                .unwrap_err()
                .message
                .contains("not defined")
        );
        assert!(run("eval('1 + 1')").unwrap_err().is_unsupported());
        assert!(
            run("const x = 1; x = 2")
                .unwrap_err()
                .message
                .contains("constant")
        );
        assert_eq!(run("typeof missing").unwrap().to_string(), "undefined");
    }

    #[test]
    fn short_circuit_and_equality() {
        assert_eq!(run("false && missing").unwrap(), Value::Bool(false));
        assert_eq!(run("true || missing").unwrap(), Value::Bool(true));
        assert_eq!(run("null ?? 12").unwrap(), Value::Number(12.0));
        assert_eq!(run("1 === '1'").unwrap(), Value::Bool(false));
        assert_eq!(run("1 == '1'").unwrap(), Value::Bool(true));
    }

    #[test]
    fn event_properties_coexist_with_listeners_and_cancel_default() {
        let mut doc = Document::parse("<body><input id='field'><p id='out'></p></body>");
        let mut runtime = Runtime::new();
        runtime.execute("const field = document.getElementById('field'); let clicks = 0; field.addEventListener('click', () => { clicks++; }); field.onclick = () => { clicks += 10; return false; }; field.addEventListener('input', event => { document.getElementById('out').textContent = event.target.value; event.preventDefault(); });", &mut doc).unwrap();
        let field = doc.query_selector("#field").unwrap();
        runtime.dispatch_click(field, &mut doc).unwrap();
        assert!(runtime.last_default_prevented);
        assert_eq!(
            runtime.execute("clicks", &mut doc).unwrap(),
            Value::Number(11.0)
        );
        doc.set_attr(field, "value", "typed value");
        runtime.dispatch_event(field, "input", &mut doc).unwrap();
        assert!(!runtime.last_default_prevented);
        assert_eq!(
            doc.text_content(doc.query_selector("#out").unwrap()),
            "typed value"
        );
    }

    #[test]
    fn template_interpolation_parses_nested_grammars_and_comma_expressions() {
        let (mut runtime, mut doc) = property_harness();
        runtime
            .execute(
                r#"
            assert.sameValue(`plain`, 'plain');assert.sameValue(`${1+2}`, '3');
            assert.sameValue(`a${1}b${2}c`, 'a1b2c');
            assert.sameValue(`outer${`inner${{x:3}.x}end`}tail`, 'outerinner3endtail');
            assert.sameValue(`${'{' + "}"}`, '{}');
            assert.sameValue(`${ /* } ` ${ */ 7 // } ` ${
            }!`, '7!');
            assert.sameValue(`${/[}]/.test('}')}`, 'true');
            assert.sameValue(`${/`/.test('`')}`, 'true');
            assert.sameValue(`${/['}]/.test("'")}`, 'true');
            assert.sameValue(`${12 / 3 / 2}`, '2');
            assert.sameValue(`${/}/.test('}') ? `${2}` : 'no'}`, '2');
            assert.sameValue(`${function(){return '}';}()}`, '}');
            var n=0;assert.sameValue(`${n++, n++, n}`, '2');assert.sameValue(n,2);
            var seen='';for(var i=`${'x' in {x:1}}`;seen==='';){seen=i;}
            assert.sameValue(seen,'true');
            assert.sameValue((`a${1}b`).length,3);
            assert.sameValue(function(){return `a
            b`;}().slice(0,2),'a\n');
            // Template text is not a Directive Prologue string literal.
            function loose(){`use strict`;return this;}assert.sameValue(loose(),window);
        "#,
                &mut doc,
            )
            .unwrap();
    }
    #[test]
    fn template_interpolation_cooks_utf16_escapes_and_line_terminators() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        let source = "`a\r\nb\rc\nd\u{2028}e\u{2029}f`";
        assert_eq!(
            runtime.execute(source, &mut doc).unwrap(),
            Value::String("a\nb\nc\nd\u{2028}e\u{2029}f".into())
        );
        let source = "`a\\\r\nb\\\rc\\\nd\\\u{2028}e\\\u{2029}f`";
        assert_eq!(
            runtime.execute(source, &mut doc).unwrap(),
            Value::String("abcdef".into())
        );
        let source =
            r#"`\uD800${'\uDC00'}\u{DFFF}\u{000000000000000041}\u{1F600}\x41\0\q\`\${raw}`"#;
        let Value::String(text) = runtime.execute(source, &mut doc).unwrap() else {
            panic!("string");
        };
        let mut expected = vec![0xd800, 0xdc00, 0xdfff, 65, 0xd83d, 0xde00, 65, 0, 113, 96];
        expected.extend("${raw}".encode_utf16());
        assert_eq!(text.units(), expected);
        assert_eq!(
            runtime.execute(r#"`\r${'\n'}\t\v\f\b`"#, &mut doc).unwrap(),
            Value::String("\r\n\t\u{b}\u{c}\u{8}".into())
        );
        let Value::String(text) = runtime
            .execute(r#"JSON.stringify(`\uD800${'\uDFFF'}`)"#, &mut doc)
            .unwrap()
        else {
            panic!("string");
        };
        assert_eq!(text.units(), "\"𐏿\"".encode_utf16().collect::<Vec<_>>());
    }
    #[test]
    fn template_substitution_coercion_is_ordered_and_uses_the_string_hint() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var trace='';var x={get toString(){trace+='get;';return function(){trace+='call;';return 'x';};},valueOf(){throw 'wrong hint';}};
            function next(){trace+='next;';return 2;}
            assert.sameValue(`${x}${next()}`, 'x2');assert.sameValue(trace,'get;call;next;');
            var fallback={toString(){return {};},valueOf(){return 7;}};
            assert.sameValue(`${fallback}`, '7');
            assert.sameValue(`${undefined}/${null}/${true}/${false}/${-0}/${NaN}/${Infinity}`, 'undefined/null/true/false/0/NaN/Infinity');
            var a=[1,2];a.toString=function(){return 'custom array';};assert.sameValue(`${a}`, 'custom array');
            var b=[1,2];b.join=function(){return 'custom join';};assert.sameValue(`${b}`, 'custom join');
            var f=function(){};f.toString=function(){return 'custom function';};assert.sameValue(`${f}`, 'custom function');
            var reason={token:1},caught,side=0;
            var bad={get toString(){throw reason;}};
            try{`${bad}${side++}`;}catch(error){caught=error;}
            assert.sameValue(caught,reason);assert.sameValue(side,0);
            assert.throws(TypeError,()=>`${{toString:1,valueOf(){return {};}}}`);
            assert.throws(ReferenceError,()=>`${missingTemplateVariable}`);
            trace='';var first={toString(){trace+='first;';return '1';}};
            var second={toString(){trace+='second;';return '2';}};
            assert.sameValue(`${first}${`${second}`}`, '12');assert.sameValue(trace,'first;second;');
        "#,&mut doc).unwrap();
    }
    #[test]
    fn template_syntax_errors_and_tagged_unsupported_are_distinct() {
        for source in [
            "`",
            "`a${",
            "`a${1",
            "`a${1}",
            "`${}`",
            "`${1,}`",
            "`${1;2}`",
            "`\\01`",
            "`\\8`",
            "`\\9`",
            "`\\08`",
            "`\\xg0`",
            "`\\x+1`",
            "`\\u+001`",
            "`\\u{}`",
            "`\\u{110000}`",
            "`\\u12`",
            "`${1}\\1`",
            "`${ /}/ / }`",
        ] {
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(error.is_parse_error(), "{source}: {error:?}");
        }
        for source in [
            "tag`a`",
            "tag`a${1}b`",
            "tag`\\xg`",
            "tag\n`x`",
            "obj.tag`${1}`",
            "new tag`x`",
            "(function(){})`x`",
            "`a``b`",
        ] {
            assert!(
                Runtime::parse_only(source).unwrap_err().is_unsupported(),
                "{source}"
            );
        }
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        runtime.execute("var touched=false", &mut doc).unwrap();
        assert!(
            runtime
                .execute("touched=true;`${1}\\u{110000}`", &mut doc)
                .unwrap_err()
                .is_parse_error()
        );
        assert_eq!(
            runtime.execute("touched", &mut doc).unwrap(),
            Value::Bool(false)
        );
    }
    #[test]
    fn template_growth_nesting_and_native_coercion_share_resource_limits() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        runtime
            .execute(
                "var caught=false;var big='x';for(var i=0;i<17;i++){big=big+big;}",
                &mut doc,
            )
            .unwrap();
        runtime.execute("big=big+big;", &mut doc).unwrap();
        assert!(
            runtime
                .execute("try{`${big}x`;}catch(error){caught=true;}", &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            runtime.execute("caught", &mut doc).unwrap(),
            Value::Bool(false)
        );
        for source in [
            "var o={toString(){while(true){}}};try{`${o}`;}catch(error){caught=true;}",
            "var o={toString(){return `${o}`;}};try{`${o}`;}catch(error){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var caught=false", &mut doc).unwrap();
            assert!(
                runtime
                    .execute(source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(
                runtime.execute("caught", &mut doc).unwrap(),
                Value::Bool(false)
            );
        }
        let nested = format!("{}0{}", "`${".repeat(200), "}`".repeat(200));
        Runtime::parse_only(&nested).unwrap();
        assert_eq!(run(&nested).unwrap(), Value::String("0".into()));
        let large = format!("`{}`", "x".repeat(MAX_SOURCE));
        assert!(Runtime::parse_only(&large).unwrap_err().is_resource_limit());
        let many = format!("`{}`", "${0}".repeat(5000));
        assert!(Runtime::parse_only(&many).unwrap_err().is_resource_limit());
        // An invalid parse must unwind bounded nesting without recursive drops
        // proportional to a flat source's number of substitutions.
        for suffix in ["", "}", "`", "${", "\\", "/}", "/*}*/"] {
            let sample = format!("`prefix${{`inner${{1}}`}}{suffix}");
            assert!(std::panic::catch_unwind(|| Runtime::parse_only(&sample)).is_ok());
        }
    }
    #[test]
    fn abort_controller_signal_slots_and_reason_identity_are_private() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var c=new AbortController(),s=c.signal;
            assert.sameValue(c.signal,s);assert.sameValue(s instanceof AbortSignal,true);
            assert.sameValue(s instanceof EventTarget,true);assert.sameValue(s.aborted,false);
            assert.sameValue(s.reason,undefined);assert.sameValue(s.throwIfAborted(),undefined);
            assert.sameValue(Object.prototype.toString.call(c),'[object AbortController]');
            assert.sameValue(Object.prototype.toString.call(s),'[object AbortSignal]');
            assert.throws(TypeError,()=>AbortController());assert.throws(TypeError,()=>new AbortSignal());
            assert.throws(TypeError,()=>AbortSignal());assert.throws(TypeError,()=>AbortController.prototype.abort.call(s));
            assert.throws(TypeError,()=>AbortSignal.prototype.throwIfAborted.call({}));
            var getter=Object.getOwnPropertyDescriptor(AbortController.prototype,'signal').get;
            assert.throws(TypeError,()=>getter.call(Object.create(AbortController.prototype)));
            var stateGetter=Object.getOwnPropertyDescriptor(AbortSignal.prototype,'aborted').get;
            assert.throws(TypeError,()=>stateGetter.call(Object.create(AbortSignal.prototype)));
            c.signal=null;s.aborted=true;s.reason='forged';
            assert.sameValue(c.signal,s);assert.sameValue(s.aborted,false);assert.sameValue(s.reason,undefined);
            assert.throws(TypeError,function(){'use strict';s.aborted=true;});
            var reason={get constructor(){throw new Error('must not inspect reason');}},caught;
            c.abort(reason);assert.sameValue(s.reason,reason);assert.sameValue(s.aborted,true);
            try{s.throwIfAborted();}catch(error){caught=error;}assert.sameValue(caught,reason);
            c.abort('second');assert.sameValue(s.reason,reason);
            var d=new AbortController();d.abort(undefined);
            assert.sameValue(d.signal.reason instanceof DOMException,true);
            assert.sameValue(d.signal.reason.name,'AbortError');assert.sameValue(d.signal.reason.code,20);
            assert.sameValue(DOMException.ABORT_ERR,20);
            var same=d.signal.reason;d.abort();assert.sameValue(d.signal.reason,same);
            var a=AbortSignal.abort(null);assert.sameValue(a.reason,null);assert.sameValue(a.aborted,true);
            caught='missing';try{a.throwIfAborted();}catch(error){caught=error;}assert.sameValue(caught,null);
            var b=AbortSignal.abort();assert.sameValue(b.reason.name,'AbortError');
            assert.sameValue(AbortSignal.abort()===b,false);
            assert.sameValue(AbortSignal.any,undefined);assert.sameValue(AbortSignal.timeout,undefined);
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn abort_events_are_synchronous_trusted_idempotent_and_isolated() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var c=new AbortController(),s=c.signal,trace='',saved;
            window.addEventListener('abort',()=>{trace+='window;';});
            s.onabort=function(e){trace+='handler;';assert.sameValue(this,s);assert.sameValue(e.currentTarget,s);return false;};
            s.addEventListener('abort',function(e){
                saved=e;trace+='listener;';assert.sameValue(s.aborted,true);assert.sameValue(s.reason,'first');
                assert.sameValue(e.target,s);assert.sameValue(e.isTrusted,true);assert.sameValue(e.bubbles,false);
                assert.sameValue(e.cancelable,false);assert.sameValue(e.composed,false);assert.sameValue(e.eventPhase,2);
                assert.sameValue(e.composedPath().length,1);c.abort('second');
            });
            assert.sameValue(c.abort('first'),undefined);trace+='after;';
            assert.sameValue(trace,'handler;listener;after;');assert.sameValue(s.reason,'first');
            assert.sameValue(saved.currentTarget,null);assert.sameValue(saved.eventPhase,0);
            assert.sameValue(saved.composedPath().length,0);assert.sameValue(saved.defaultPrevented,false);
            c.abort();assert.sameValue(trace,'handler;listener;after;');
            var d=new AbortController(),seen=0;
            d.signal.onabort=function(e){seen++;assert.sameValue(e.isTrusted,false);};
            d.signal.dispatchEvent(new Event('abort'));
            assert.sameValue(d.signal.aborted,false);assert.sameValue(d.signal.reason,undefined);assert.sameValue(seen,1);
            d.signal.onabort=17;assert.sameValue(d.signal.onabort,null);d.abort();assert.sameValue(seen,1);
            assert.throws(TypeError,()=>Object.getOwnPropertyDescriptor(AbortSignal.prototype,'onabort').get.call({}));
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn abort_listener_options_validate_before_null_duplicates_or_aborted_suppression() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var t=new EventTarget(),c=new AbortController(),d=new AbortController(),trace='',count=0;
            function listener(){count++;}
            var options={get capture(){trace+='capture;';return false;},get once(){trace+='once;';return false;},
                get passive(){trace+='passive;';return false;},get signal(){trace+='signal;';return c.signal;}};
            t.addEventListener('x',null,options);assert.sameValue(trace,'capture;once;passive;signal;');
            trace='';t.addEventListener('x',listener,options);t.addEventListener('x',listener,options);
            assert.sameValue(trace,'capture;once;passive;signal;capture;once;passive;signal;');
            assert.throws(TypeError,()=>t.addEventListener('x',listener,{signal:null}));
            assert.throws(TypeError,()=>t.addEventListener('x',null,{signal:null}));
            assert.throws(TypeError,()=>t.addEventListener('x',listener,{signal:Object.create(AbortSignal.prototype)}));
            assert.throws(TypeError,()=>t.addEventListener('x',null,{signal:{aborted:true}}));
            t.addEventListener('x',listener,{signal:d.signal});d.abort();
            t.dispatchEvent(new Event('x'));assert.sameValue(count,1);
            c.abort();t.dispatchEvent(new Event('x'));assert.sameValue(count,1);
            t.addEventListener('x',listener,{signal:c.signal});t.dispatchEvent(new Event('x'));assert.sameValue(count,1);
            var reentrant=new AbortController();
            t.addEventListener('x',listener,{get signal(){reentrant.abort();return reentrant.signal;}});
            t.dispatchEvent(new Event('x'));assert.sameValue(count,1);
            t.addEventListener('x',listener,{signal:undefined});
            t.removeEventListener('x',listener,{get signal(){throw new Error('remove must not read signal');}});
            t.dispatchEvent(new Event('x'));assert.sameValue(count,1);
            // Public property shadowing cannot substitute private signal state.
            Object.defineProperty(c.signal,'aborted',{value:false});
            t.addEventListener('x',listener,{signal:c.signal});t.dispatchEvent(new Event('x'));assert.sameValue(count,1);
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn abort_removals_precede_callbacks_and_affect_active_dispatch_snapshots() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var t=new EventTarget(),c=new AbortController(),trace='';
            function managed(){trace+='managed;';}
            t.addEventListener('x',function(){trace+='first;';c.abort('stop');},{once:true});
            t.addEventListener('x',managed,{signal:c.signal});
            t.addEventListener('x',function(){trace+='last;';});
            c.signal.addEventListener('abort',function(){trace+='abort;';t.dispatchEvent(new Event('x'));});
            c.signal.addEventListener('abort',()=>{trace+='self-managed;';},{signal:c.signal});
            t.dispatchEvent(new Event('x'));assert.sameValue(trace,'first;abort;last;last;');
            var d=new AbortController(),n=0;
            function again(){n++;}
            t.addEventListener('y',again,{signal:d.signal});t.removeEventListener('y',again);
            t.addEventListener('y',again);d.abort();t.dispatchEvent(new Event('y'));assert.sameValue(n,1);
            var e=new AbortController();
            t.addEventListener('z',function(){n++;e.abort();t.dispatchEvent(new Event('z'));},{once:true,signal:e.signal});
            t.dispatchEvent(new Event('z'));assert.sameValue(n,2);
            var f=new AbortController();f.signal.onabort=()=>{throw 'reported abort callback';};
            f.signal.addEventListener('abort',()=>{n++;});f.abort();assert.sameValue(n,3);
        "#,&mut doc).unwrap();
        assert_eq!(runtime.console.len(), 1, "{:?}", runtime.console);
        assert!(runtime.console[0].contains("reported abort callback"));
    }
    #[test]
    fn abort_long_reasons_charge_formatting_work_and_diagnostic_allocation() {
        for action in ["signal.throwIfAborted()", "throw reason"] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime
                .execute(
                    &format!(
                        "var reason='{}';var signal=AbortSignal.abort(reason);var n=0;",
                        "x".repeat(65_536)
                    ),
                    &mut doc,
                )
                .unwrap();
            let error = runtime
                .execute(
                    &format!("for(n=0;n<100;n++){{try{{{action};}}catch(error){{}}}}"),
                    &mut doc,
                )
                .unwrap_err();
            assert!(error.is_resource_limit(), "{action}");
            let Value::Number(iterations) = runtime.execute("n", &mut doc).unwrap() else {
                panic!("numeric loop counter");
            };
            assert!(iterations < 16.0, "{action}: {iterations}");
        }
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        // Lone surrogates expand to three-byte replacement characters only in
        // the diagnostic; the stored thrown reason must remain exact UTF-16.
        let reason = runtime.string(JsString::from(&[0xd800; 64][..])).unwrap();
        let signal = runtime.abort_signal_object(reason.clone()).unwrap();
        let index = runtime.abort_signal_index(&signal).unwrap();
        runtime.allocated = MAX_HEAP - 64;
        let error = runtime
            .abort_native("AbortSignal.throwIfAborted", signal, &[], &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.abort_signals[index].reason, reason);
    }
    #[test]
    fn abort_observer_work_is_preflighted_and_callback_limits_are_uncatchable() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        let controller = runtime
            .event_construct("AbortController", &[], &mut doc)
            .unwrap();
        let signal = runtime
            .abort_native(
                "AbortController.get.signal",
                controller.clone(),
                &[],
                &mut doc,
            )
            .unwrap();
        let index = runtime.abort_signal_index(&signal).unwrap();
        runtime.execute("function callback(){}", &mut doc).unwrap();
        let callback = runtime.execute("callback", &mut doc).unwrap();
        let options = runtime
            .object_ordered([("signal".into(), signal.clone())])
            .unwrap();
        for i in 0..40 {
            runtime
                .event_target_native(
                    "addEventListener",
                    Value::Document,
                    &[
                        Value::String(format!("event{i}").into()),
                        callback.clone(),
                        options.clone(),
                    ],
                    &mut doc,
                )
                .unwrap();
        }
        assert_eq!(runtime.abort_signals[index].listeners.len(), 40);
        runtime.steps = 39;
        assert!(
            runtime
                .abort_native("AbortController.abort", controller.clone(), &[], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.abort_signals[index].reason, Value::Undefined);
        assert!(
            runtime.abort_signals[index]
                .listeners
                .iter()
                .all(|id| !runtime.listeners[*id].removed)
        );
        runtime.steps = MAX_STEPS;
        runtime
            .abort_native("AbortController.abort", controller, &[], &mut doc)
            .unwrap();
        assert!(runtime.abort_signals[index].listeners.is_empty());
        assert!(runtime.listeners.iter().all(|listener| listener.removed));
        for source in [
            "var c=new AbortController();c.signal.onabort=function(){while(true){}};try{c.abort();}catch(error){caught=true;}",
            "var c=new AbortController();function chain(){var n=new AbortController();n.signal.onabort=chain;n.abort();}c.signal.onabort=chain;try{c.abort();}catch(error){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var caught=false", &mut doc).unwrap();
            assert!(
                runtime
                    .execute(source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime
                    .execute("caught===false && c.signal.aborted===true", &mut doc)
                    .unwrap(),
                Value::Bool(true)
            );
            assert_eq!(runtime.stack_units, 0);
            assert!(runtime.events.iter().all(|event| !event.dispatching
                && event.path.is_empty()
                && event.current_target == Value::Null));
        }
    }
    #[test]
    fn events_have_private_state_readonly_fields_and_standard_construction() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var detail={answer:42}; var e=new CustomEvent('change',{detail:detail,bubbles:true,cancelable:true,composed:true});
            assert.sameValue(e instanceof Event,true); assert.sameValue(e instanceof CustomEvent,true);
            assert.sameValue(e.type,'change');assert.sameValue(e.detail,detail);assert.sameValue(e.isTrusted,false);
            assert.sameValue(e.target,null);assert.sameValue(e.currentTarget,null);assert.sameValue(e.eventPhase,Event.NONE);
            assert.sameValue(e.composed,true);assert.sameValue(e.bubbles,true);assert.sameValue(e.cancelable,true);
            assert.sameValue(CustomEvent.AT_TARGET,2);assert.sameValue(typeof e.timeStamp,'number');
            e.type='wrong';e.target={};e.defaultPrevented=true;e.isTrusted=true;
            assert.sameValue(e.type,'change');assert.sameValue(e.target,null);assert.sameValue(e.defaultPrevented,false);
            assert.throws(TypeError,function(){'use strict';e.type='wrong';});
            assert.throws(TypeError,()=>Object.defineProperty(e,'isTrusted',{value:true}));
            assert.throws(TypeError,()=>Event('x'));assert.throws(TypeError,()=>new Event());
            assert.throws(TypeError,()=>Event.prototype.preventDefault.call({}));
            assert.sameValue(Object.prototype.toString.call(e),'[object CustomEvent]');
            var t=new EventTarget();assert.sameValue(t instanceof EventTarget,true);
            e.preventDefault();assert.sameValue(t.dispatchEvent(e),false);assert.sameValue(e.target,t);
            e.initCustomEvent('other',false,false,17);assert.sameValue(e.detail,17);
            assert.sameValue(e.defaultPrevented,false);assert.sameValue(e.target,null);assert.sameValue(e.type,'other');
            var old=document.createEvent('Event');var caught=false;
            try{t.dispatchEvent(old);}catch(error){caught=error instanceof DOMException && error.name==='InvalidStateError' && error.code===11;}
            assert.sameValue(caught,true);old.initEvent('',false,true);assert.sameValue(t.dispatchEvent(old),true);
            var trace='';new CustomEvent({toString(){trace+='type;';return 'x';}}, {
                get bubbles(){trace+='bubbles;';return false;},get cancelable(){trace+='cancelable;';return false;},
                get composed(){trace+='composed;';return false;},get detail(){trace+='detail;';return 1;}
            });assert.sameValue(trace,'type;bubbles;cancelable;composed;detail;');
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn events_capture_target_bubble_use_fixed_paths_and_clean_up_after_dispatch() {
        let (mut runtime, _) = property_harness();
        let mut doc = Document::parse(
            "<main id=p><button id=c>go</button></main><section id=other></section>",
        );
        runtime.execute(r#"
            var p=document.getElementById('p'),c=document.getElementById('c'),other=document.getElementById('other');
            var trace='',pathGood=false,targetGood=true,seen;
            function listen(target,label,capture){target.addEventListener('x',function(e){
                trace+=label+e.eventPhase+';';targetGood=targetGood && this===target && e.currentTarget===target && e.target===c;
            },capture);}
            listen(window,'w',true);listen(document,'d',true);listen(p,'p',true);listen(c,'c',true);
            listen(c,'c',false);listen(p,'p',false);listen(document,'d',false);listen(window,'w',false);
            c.addEventListener('x',function(e){seen=e;var path=e.composedPath();
                pathGood=path[0]===c && path[1]===p && path[path.length-2]===document && path[path.length-1]===window;
                path.pop();other.appendChild(c);
            });
            var e=new Event('x',{bubbles:true});assert.sameValue(c.dispatchEvent(e),true);
            assert.sameValue(trace,'w1;d1;p1;c2;c2;p3;d3;w3;');assert.sameValue(targetGood,true);assert.sameValue(pathGood,true);
            assert.sameValue(e.currentTarget,null);assert.sameValue(e.eventPhase,0);assert.sameValue(e.composedPath().length,0);
            assert.sameValue(e.target,c);assert.sameValue(seen,e);assert.sameValue(document.documentElement.parentNode,document);
            p.appendChild(c);trace='';c.dispatchEvent(new Event('x'));
            assert.sameValue(trace,'w1;d1;p1;c2;c2;');
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn events_snapshot_listeners_and_honor_removal_addition_once_and_identity() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var t=new EventTarget(),trace='';function late(){trace+='late;';}function removed(){trace+='removed;';}
            t.addEventListener('x',function(){trace+='first;';t.removeEventListener('x',removed);t.addEventListener('x',late);});
            t.addEventListener('x',removed);t.addEventListener('x',function(){trace+='last;';});
            t.dispatchEvent(new Event('x'));assert.sameValue(trace,'first;last;');
            trace='';t.dispatchEvent(new Event('x'));assert.sameValue(trace,'first;last;late;');
            var n=0;function one(){n++;t.dispatchEvent(new Event('once'));}
            t.addEventListener('once',one,{once:true});t.addEventListener('once',one,{once:false});
            t.dispatchEvent(new Event('once'));assert.sameValue(n,1);
            var phases='';function twice(e){phases+=e.eventPhase;}
            t.addEventListener('both',twice,true);t.addEventListener('both',twice,false);
            t.removeEventListener('both',twice,{capture:true, get once(){throw new Error('must not read');}});
            t.dispatchEvent(new Event('both'));assert.sameValue(phases,'2');
            var observer={count:0,handleEvent(e){this.count++;this.handleEvent=function(){this.count+=10;};}};
            t.addEventListener('object',observer);t.dispatchEvent(new Event('object'));t.dispatchEvent(new Event('object'));
            assert.sameValue(observer.count,11);t.removeEventListener('object',observer);t.dispatchEvent(new Event('object'));assert.sameValue(observer.count,11);
            var captureAdded=0;t.addEventListener('target',function(){t.addEventListener('target',()=>{captureAdded++;});},true);
            t.dispatchEvent(new Event('target'));assert.sameValue(captureAdded,1);
            t.addEventListener('null',null);t.addEventListener('null',undefined);t.removeEventListener('null',null);
            assert.throws(TypeError,()=>t.addEventListener('x',1));
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn events_stop_flags_passive_cancellation_and_reentrancy_are_independent() {
        let (mut runtime, _) = property_harness();
        let mut doc = Document::parse("<main><button></button></main>");
        runtime.execute(r#"
            var p=document.querySelector('main'),c=document.querySelector('button'),trace='';
            c.addEventListener('x',function(e){trace+='one;';e.stopPropagation();e.cancelBubble=false;});
            c.addEventListener('x',function(){trace+='two;';});p.addEventListener('x',function(){trace+='parent;';});
            var e=new Event('x',{bubbles:true});c.dispatchEvent(e);assert.sameValue(trace,'one;two;');assert.sameValue(e.cancelBubble,false);
            trace='';c.addEventListener('immediate',function(e){trace+='one;';e.stopImmediatePropagation();});
            c.addEventListener('immediate',function(){trace+='two;';});c.dispatchEvent(new Event('immediate'));assert.sameValue(trace,'one;');
            trace='';c.addEventListener('capture-stop',function(e){trace+='capture;';e.stopPropagation();},true);
            c.addEventListener('capture-stop',function(){trace+='same-invocation;';},true);
            c.addEventListener('capture-stop',function(){trace+='bubble;';});
            c.dispatchEvent(new Event('capture-stop',{bubbles:true}));assert.sameValue(trace,'capture;same-invocation;');
            c.addEventListener('passive',function(e){e.preventDefault();e.returnValue=false;},{passive:true});
            var passive=new Event('passive',{cancelable:true});assert.sameValue(c.dispatchEvent(passive),true);assert.sameValue(passive.defaultPrevented,false);
            c.addEventListener('cancel',function(e){e.preventDefault();e.returnValue=true;return false;});
            var cancel=new Event('cancel',{cancelable:true});assert.sameValue(c.dispatchEvent(cancel),false);assert.sameValue(cancel.returnValue,false);
            assert.sameValue(c.dispatchEvent(new Event('cancel')),true);
            var wheel;document.body.addEventListener('wheel',function(e){e.preventDefault();});
            wheel=new Event('wheel',{cancelable:true});assert.sameValue(document.body.dispatchEvent(wheel),true);
            var explicit=document.createElement('aside');document.body.appendChild(explicit);
            explicit.addEventListener('wheel',function(e){e.preventDefault();},{passive:false});
            assert.sameValue(explicit.dispatchEvent(new Event('wheel',{cancelable:true})),false);
            var t=new EventTarget(),inner=new Event('inner'),outer=new Event('outer'),invalid=false,stable=false;
            t.addEventListener('inner',function(e){e.stopImmediatePropagation();});
            t.addEventListener('outer',function(e){try{t.dispatchEvent(e);}catch(err){invalid=err.name==='InvalidStateError';}
                t.dispatchEvent(inner);e.initEvent('changed',false,false);stable=e.type==='outer' && e.currentTarget===t && e.eventPhase===2;
            });
            t.dispatchEvent(outer);assert.sameValue(invalid,true);assert.sameValue(stable,true);assert.sameValue(inner.currentTarget,null);
            assert.sameValue(outer.currentTarget,null);assert.sameValue(outer.type,'outer');
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn events_report_listener_errors_and_keep_dispatching() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var t=new EventTarget(),trace='';
            t.addEventListener('x',function(){trace+='first;';throw new Error('listener failed');});
            t.addEventListener('x',{get handleEvent(){throw new TypeError('getter failed');}});
            t.addEventListener('x',function(){trace+='last;';});
            var e=new Event('x');assert.sameValue(t.dispatchEvent(e),true);assert.sameValue(trace,'first;last;');
            assert.sameValue(e.currentTarget,null);assert.sameValue(e.eventPhase,0);assert.sameValue(e.composedPath().length,0);
        "#,&mut doc).unwrap();
        assert_eq!(runtime.console.len(), 2);
    }
    #[test]
    fn events_inline_handler_registration_and_attribute_mutation_preserve_order() {
        let (mut runtime, _) = property_harness();
        let mut doc = Document::parse(
            "<button onclick=\"trace+='inline;';return false\"></button><input onclick='bad syntax )'>",
        );
        runtime.execute(r#"
            var trace='',button=document.querySelector('button');
            button.addEventListener('click',function(){trace+='listener;';});
            var e=new Event('click',{cancelable:true});assert.sameValue(button.dispatchEvent(e),false);assert.sameValue(trace,'inline;listener;');
            button.onclick=function(){trace+='property;';return false;};trace='';button.dispatchEvent(new Event('click'));
            assert.sameValue(trace,'property;listener;');button.onclick=null;
            button.onclick=function(){trace+='last;';};trace='';button.dispatchEvent(new Event('click'));assert.sameValue(trace,'listener;last;');
            button.setAttribute('onclick',"trace+='attribute;';");trace='';button.dispatchEvent(new Event('click'));assert.sameValue(trace,'listener;attribute;');
            button.removeAttribute('onclick');trace='';button.dispatchEvent(new Event('click'));assert.sameValue(trace,'listener;');
            var bad=document.querySelector('input');bad.addEventListener('click',()=>{trace+='survived;';});
            trace='';bad.dispatchEvent(new Event('click'));assert.sameValue(trace,'survived;');assert.sameValue(bad.onclick,null);
            var nothing={};button.onclick=nothing;assert.sameValue(button.onclick,nothing);
            trace='';button.dispatchEvent(new Event('click'));assert.sameValue(trace,'listener;');
            button.onclick=12;assert.sameValue(button.onclick,null);
        "#,&mut doc).unwrap();
        assert_eq!(runtime.console.len(), 1);
    }
    #[test]
    fn events_detached_and_template_paths_do_not_reach_document_or_window() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var hits=0;document.addEventListener('x',()=>{hits++;},true);window.addEventListener('x',()=>{hits++;});
            var parent=document.createElement('main'),child=document.createElement('button');parent.appendChild(child);
            var path;child.addEventListener('x',e=>{path=e.composedPath();});child.dispatchEvent(new Event('x',{bubbles:true,composed:true}));
            assert.sameValue(path.length,2);assert.sameValue(path[1],parent);assert.sameValue(hits,0);
            var card=document.createElement('template');card.innerHTML='<button></button>';document.body.appendChild(card);
            child=card.content.querySelector('button');child.addEventListener('x',e=>{path=e.composedPath();});
            child.dispatchEvent(new Event('x',{bubbles:true,composed:true}));assert.sameValue(path.length,2);assert.sameValue(path[1],card.content);assert.sameValue(hits,0);
        "#,&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn events_readiness_trust_and_host_cancellation_match_event_types() {
        let (mut runtime, mut doc) = property_harness();
        runtime.execute(r#"
            var trace='',trusted=true,loadTarget=false;
            document.addEventListener('DOMContentLoaded',e=>{trace+='document;';trusted=trusted&&e.isTrusted;});
            window.addEventListener('DOMContentLoaded',e=>{trace+='window;';trusted=trusted&&e.isTrusted;});
            document.addEventListener('load',()=>{trace+='wrong;';});
            window.onload=function(e){trace+='load;';loadTarget=e.target===document&&e.currentTarget===window;trusted=trusted&&e.isTrusted;};
        "#,&mut doc).unwrap();
        runtime.dispatch_dom_content_loaded(&mut doc).unwrap();
        runtime.dispatch_dom_content_loaded(&mut doc).unwrap();
        runtime.execute("assert.sameValue(trace,'document;window;load;');assert.sameValue(trusted,true);assert.sameValue(loadTarget,true);",&mut doc).unwrap();
        assert!(runtime.console.is_empty(), "{:?}", runtime.console);
    }
    #[test]
    fn events_charge_default_passive_tree_scans_and_long_type_lookups_before_work() {
        let mut doc = Document::parse(&format!("{}<body>", "<!-- retained -->".repeat(1000)));
        let mut runtime = Runtime::new();
        runtime.execute("function callback(){}", &mut doc).unwrap();
        let callback = runtime.lookup(0, "callback").unwrap().1;
        let node = doc.create_element("button");
        let before = runtime.listeners.len();
        runtime.steps = 128;
        let error = runtime
            .event_target_native(
                "addEventListener",
                Value::Node(node),
                &[Value::String("wheel".into()), callback],
                &mut doc,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.listeners.len(), before);
        runtime.steps = MAX_STEPS;
        let event = runtime
            .event_object(
                "x".repeat(1024).into(),
                false,
                false,
                false,
                Value::Null,
                false,
            )
            .unwrap();
        let id = runtime.event_index(&event).unwrap();
        runtime.steps = 128;
        let error = runtime
            .dispatch_event_object(EventTarget::Node(node), event, None, &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert!(!runtime.events[id].dispatching);
        assert!(runtime.events[id].path.is_empty());
        assert_eq!(runtime.events[id].current_target, Value::Null);
    }
    #[test]
    fn repeated_inline_handler_reads_charge_cached_source_comparisons() {
        let mut doc = Document::parse(&format!(
            "<button onclick='{}return 1'></button>",
            " ".repeat(30_000)
        ));
        let mut runtime = Runtime::new();
        runtime
            .execute(
                "var button=document.querySelector('button');button.onclick;",
                &mut doc,
            )
            .unwrap();
        // Previously these 3,000 reads completed within one entry while
        // comparing 90,024,000 cached-source bytes without charging that work.
        let error = runtime
            .execute("var i;for(i=0;i<3000;i++){button.onclick;}", &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        let Value::Number(reads) = runtime.execute("i", &mut doc).unwrap() else {
            panic!("loop counter must remain numeric");
        };
        assert!(
            reads < 4.0,
            "comparison work must consume the shared budget"
        );
        // Exhaustion does not remove or corrupt the compiled handler.
        assert_eq!(
            runtime.execute("button.onclick()", &mut doc).unwrap(),
            Value::Number(1.0)
        );
        assert_eq!(runtime.stack_units, 0);
    }
    #[test]
    fn events_quota_and_reentrant_termination_are_uncatchable_and_clean_state() {
        for source in [
            "var t=new EventTarget();var e=new Event('x');t.addEventListener('x',function(){while(true){}});try{t.dispatchEvent(e);}catch(err){caught=true;}",
            "var t=new EventTarget();var e=new Event('x');t.addEventListener('x',function(){t.dispatchEvent(new Event('x'));});try{t.dispatchEvent(e);}catch(err){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var caught=false;", &mut doc).unwrap();
            assert!(
                runtime
                    .execute(source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.execute("caught", &mut doc).unwrap(),
                Value::Bool(false)
            );
            assert_eq!(
                runtime
                    .execute(
                        "e.currentTarget===null && e.eventPhase===0 && e.composedPath().length===0",
                        &mut doc
                    )
                    .unwrap(),
                Value::Bool(true)
            );
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(runtime.calls, 0);
        }
    }

    #[test]
    fn inner_html_inserts_a_fragment_and_tracks_dom_storage() {
        let mut doc = Document::parse("<body><div id='out'>before</div></body>");
        let mut runtime = Runtime::new();
        let before = doc.retained_bytes();
        runtime.execute("document.getElementById('out').innerHTML = '<p class=active>after</p>'; const text = document.createTextNode(' additional'); document.getElementById('out').appendChild(text);", &mut doc).unwrap();
        assert_eq!(doc.query_selector_all("html").len(), 1);
        assert_eq!(
            doc.text_content(doc.query_selector("#out").unwrap()),
            "after additional"
        );
        assert!(doc.retained_bytes() > before);
    }

    #[test]
    fn textarea_value_reads_initial_text_and_script_assignments_use_editor_storage() {
        let mut doc = Document::parse(
            r#"<textarea id=json value=ignored>{"label":"é🦀","count":2}</textarea>"#,
        );
        let mut runtime = Runtime::new();
        assert_eq!(
            runtime
                .execute(
                    "const field = document.getElementById('json'); JSON.parse(field.value).count;",
                    &mut doc
                )
                .unwrap(),
            Value::Number(2.0)
        );
        let node = doc.query_selector("#json").unwrap();
        runtime.execute("let inputs = 0; field.addEventListener('input',function(){ inputs++; }); field.value = JSON.stringify({ done:true });",&mut doc).unwrap();
        assert_eq!(doc.text_content(node), r#"{"done":true}"#);
        assert_eq!(doc.attr(node, "value"), Some("ignored"));
        assert_eq!(
            runtime
                .execute("JSON.parse(field.value).done", &mut doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime.execute("inputs", &mut doc).unwrap(),
            Value::Number(0.0)
        );
        runtime
            .execute("field.value = 'first\\r\\nsecond\\rthird\\n';", &mut doc)
            .unwrap();
        assert_eq!(doc.text_content(node), "first\nsecond\nthird\n");
        assert_eq!(
            runtime
                .execute("field.value", &mut doc)
                .unwrap()
                .to_string(),
            "first\nsecond\nthird\n"
        );
        runtime.execute("field.value = null;", &mut doc).unwrap();
        assert!(doc.text_content(node).is_empty());
        assert_eq!(
            runtime
                .execute("field.value", &mut doc)
                .unwrap()
                .to_string(),
            ""
        );
        runtime.execute("field.value = 42;", &mut doc).unwrap();
        assert_eq!(doc.text_content(node), "42");
    }

    #[test]
    fn textarea_native_edit_input_script_and_form_values_stay_in_sync() {
        let html = "<form action='/send'><textarea id=field name=body>initial</textarea><button id=send>Send</button></form><script>const field = document.getElementById('field'); let events = 0; field.addEventListener('input',function(event) { events++; event.target.value = event.target.value.toUpperCase(); });</script>";
        let mut page = crate::page::Page::from_html(
            url::Url::parse("https://example.test/").unwrap(),
            html,
            true,
        );
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        let node = page.document.query_selector("#field").unwrap();
        let submit = page.document.query_selector("#send").unwrap();
        assert!(page.can_edit_control(node));
        for (sequence, typed, result) in [
            (1, "é\nfirst", "É\nFIRST"),
            (2, "É\nFIRST🦀b", "É\nFIRST🦀B"),
        ] {
            // This is the worker's textarea Edit storage and dispatch path;
            // snapshots then read the same text for edit acknowledgements.
            page.document.set_text_content(node, typed);
            page.runtime
                .dispatch_event(node, "input", &mut page.document)
                .unwrap();
            assert_eq!(page.document.text_content(node), result);
            assert_eq!(
                page.runtime
                    .execute("field.value", &mut page.document)
                    .unwrap()
                    .to_string(),
                result
            );
            assert_eq!(
                page.runtime.execute("events", &mut page.document).unwrap(),
                Value::Number(sequence as f64)
            );
            let snapshot = page.document.clone();
            assert_eq!(snapshot.text_content(node), result);
            let navigation = page.click(submit).unwrap();
            let url = url::Url::parse(&navigation.address).unwrap();
            assert_eq!(
                url.query_pairs()
                    .find(|(name, _)| name == "body")
                    .unwrap()
                    .1,
                result.replace('\n', "\r\n")
            );
        }
    }

    #[test]
    fn large_ast_chains_and_native_output_are_bounded() {
        assert!(
            run(&format!("1{}", "+1".repeat(5000)))
                .unwrap_err()
                .message
                .contains("limit")
        );
        assert_eq!(
            run(&format!("x{}", ".x".repeat(5000)))
                .unwrap_err()
                .intrinsic_error_name(),
            Some("ReferenceError")
        );
        assert!(
            Runtime::parse_only(&format!("x{}", ".x".repeat(20_000)))
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            run("let s = 'x'; for (let i = 0; i < 17; i++) { s += s; } console.log(s,s,s);")
                .unwrap_err()
                .message
                .contains("limit")
        );
        assert!(
            Parser::program(&";".repeat(MAX_SOURCE + 1))
                .unwrap_err()
                .message
                .contains("source")
        );
        assert_eq!(parse_float("12.5e+"), 12.5);
        assert_eq!(parse_float("-12.5e2xyz"), -1250.0);
        assert!(parse_float(&"+".repeat(MAX_STRING)).is_nan());
    }

    #[test]
    fn malformed_sources_never_panic() {
        let alphabet = [
            "a", "0", "'", "\"", "`", "\\", "(", ")", "[", "]", "{", "}", ";", ".", "!", "+", "=",
            "/", "\n", "é", "🦀", "$", " ",
        ];
        let mut state = 0xace12497_u32;
        for length in 0..256 {
            let mut source = String::new();
            for _ in 0..length {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                source.push_str(alphabet[state as usize % alphabet.len()]);
            }
            let _ = run(&source);
        }
    }

    #[test]
    fn throw_preserves_primitive_values_and_object_identity() {
        assert_eq!(
            run("let result; try { throw 'failure'; } catch (error) { result = error; } result;")
                .unwrap()
                .to_string(),
            "failure"
        );
        assert_eq!(run("const failure = { code: 7 }; let same = false; try { throw failure; } catch (error) { same = error === failure; error.code = 8; } same && failure.code === 8;").unwrap(), Value::Bool(true));
        assert_eq!(
            run(
                "let result = 4; try { throw undefined; } catch (error) { result = error; } result;"
            )
            .unwrap(),
            Value::Undefined
        );
        assert_eq!(
            run("let result; try { throw [1,2,3]; } catch (error) { result = error[1]; } result;")
                .unwrap(),
            Value::Number(2.0)
        );
        assert!(
            run("throw 'uncaught';")
                .unwrap_err()
                .message
                .contains("uncaught")
        );
    }

    #[test]
    fn catch_exposes_names_and_messages_for_runtime_errors() {
        assert_eq!(run("let names = []; try { missingVariable; } catch (error) { names.push(error.name); } try { null.property; } catch (error) { names.push(error.name); } const fixed = 1; try { fixed = 2; } catch (error) { names.push(error.name); } const values = []; try { values.length = -1; } catch (error) { names.push(error.name); } names.join(',');").unwrap().to_string(), "ReferenceError,TypeError,TypeError,RangeError");
        assert_eq!(run("let message; try { noSuchFunction(); } catch (error) { message = error.message; } message.includes('not defined');").unwrap(), Value::Bool(true));
    }

    #[test]
    fn catch_binding_is_scoped_and_optional_binding_works() {
        assert_eq!(run("let error = 'outer'; let caught = ''; try { throw 'inner'; } catch (error) { let local = 1; caught = error; error = 'changed'; } error + ':' + caught + ':' + typeof local;").unwrap().to_string(), "outer:inner:undefined");
        assert_eq!(
            run("let caught = false; try { throw null; } catch { caught = true; } caught;")
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(run("let result; try { throw 4; } catch (onlyInCatch) { result = onlyInCatch; } finally { result += typeof onlyInCatch; } result;").unwrap().to_string(), "4undefined");
    }

    #[test]
    fn finally_preserves_or_overrides_returns_and_throws() {
        assert_eq!(
            run(
                "function value() { try { return 7; } finally { const cleanup = true; } } value();"
            )
            .unwrap(),
            Value::Number(7.0)
        );
        assert_eq!(run("function value() { try { try { return 1; } finally { return 2; } } finally { return 3; } } value();").unwrap(), Value::Number(3.0));
        assert_eq!(
            run("function value() { try { throw 'initial'; } finally { return 4; } } value();")
                .unwrap(),
            Value::Number(4.0)
        );
        assert_eq!(run("function value() { try { return 1; } finally { throw { code: 9 }; } } let result; try { value(); } catch (error) { result = error.code; } result;").unwrap(), Value::Number(9.0));
    }

    #[test]
    fn finally_runs_for_both_caught_and_propagated_exceptions() {
        assert_eq!(run("let log = ''; function raise() { try { throw 'x'; } finally { log += 'inner;'; } } try { raise(); } catch (error) { log += error; } finally { log += 'outer;'; } log;").unwrap().to_string(), "inner;xouter;");
        assert_eq!(run("let log = ''; try { try { throw 'first'; } catch (error) { throw 'second'; } finally { log += 'cleanup;'; } } catch (error) { log += error; } log;").unwrap().to_string(), "cleanup;second");
        assert_eq!(
            run("let value = 0; try { value = 2; } finally { value += 3; } value;").unwrap(),
            Value::Number(5.0)
        );
    }

    #[test]
    fn finally_preserves_and_overrides_break_and_continue() {
        assert_eq!(run("let log = ''; for (let i = 0; i < 4; i++) { try { if (i === 1) continue; if (i === 3) break; log += String(i); } finally { log += 'f'; } } log;").unwrap().to_string(), "0ff2ff");
        assert_eq!(
            run("let n = 0; while (n < 5) { try { n++; continue; } finally { break; } } n;")
                .unwrap(),
            Value::Number(1.0)
        );
        assert_eq!(
            run("let n = 0; while (n < 3) { try { n++; break; } finally { continue; } } n;")
                .unwrap(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn resource_failures_cannot_be_caught_or_overridden_by_finally() {
        for body in [
            "while (true) {}",
            "const values = []; values.length = 1000000; while (true) { values.push(1); }",
            "function recur() { recur(); } recur();",
            "let text = 'x'; while (true) { text += text; }",
        ] {
            let mut doc = Document::parse("<body></body>");
            let mut runtime = Runtime::new();
            let source = format!(
                "let caught = false; let finalized = false; try {{ {body} }} catch (error) {{ caught = true; }} finally {{ finalized = true; }}"
            );
            let error = runtime.execute(&source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit(), "{error}");
            assert_eq!(
                runtime.execute("caught || finalized", &mut doc).unwrap(),
                Value::Bool(false)
            );
        }
        assert!(run("function evade() { try { while (true) {} } finally { return 'escaped'; } } evade();").unwrap_err().is_resource_limit());
        assert!(
            run("while (true) { try { missing(); } catch (error) {} }")
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(run("let result; try { throw 'script allocation limit exceeded'; } catch (error) { result = error; } result;").unwrap().to_string(), "script allocation limit exceeded");
    }

    #[test]
    fn throw_restricted_line_breaks_and_try_grammar_are_checked() {
        for line_break in ["\n", "\r", "\u{2028}", "/*\n*/"] {
            let source = format!("try {{ throw{line_break}'error'; }} catch {{ }}");
            assert!(run(&source).unwrap_err().message.contains("line break"));
        }
        assert!(
            run("try { 1; }")
                .unwrap_err()
                .message
                .contains("requires catch or finally")
        );
        assert!(run("try 1; catch (error) { }").is_err());
        assert!(run("try { throw; } catch (error) { }").is_err());
        assert_eq!(
            run("function value() { return\n 7; } value();").unwrap(),
            Value::Undefined
        );
    }

    #[test]
    fn json_parses_strict_grammar_and_orders_object_properties() {
        assert_eq!(run(r#"const value = JSON.parse('{"z":1,"2":"two","1":"one","a":true,"z":3,"__proto__":{"safe":7}}'); JSON.stringify(value);"#).unwrap().to_string(),r#"{"1":"one","2":"two","z":3,"a":true,"__proto__":{"safe":7}}"#);
        assert_eq!(
            run("JSON.stringify({ z:1, a:2, '10':10, '2':2 })")
                .unwrap()
                .to_string(),
            r#"{"2":2,"10":10,"z":1,"a":2}"#
        );
        assert_eq!(
            run("let value = { z:1 }; value.a = 2; value.z = 3; JSON.stringify(value);")
                .unwrap()
                .to_string(),
            r#"{"z":3,"a":2}"#
        );
        assert_eq!(
            run("JSON.parse(' \\t\\r\\n[null,true,false,-2.5e+2] ')[3]").unwrap(),
            Value::Number(-250.0)
        );
        assert_eq!(run("JSON.parse(null)").unwrap(), Value::Null);
        assert_eq!(run("JSON.parse(12)").unwrap(), Value::Number(12.0));
        assert_eq!(run("JSON.parse([12])").unwrap(), Value::Number(12.0));
        assert_eq!(run("JSON.parse([[true]])").unwrap(), Value::Bool(true));
        assert_eq!(
            run("JSON.parse({ toString:function() { return '17'; } })").unwrap(),
            Value::Number(17.0)
        );
        assert_eq!(run("JSON.parse({ toString:function() { return {}; }, valueOf:function() { return '23'; } })").unwrap(),Value::Number(23.0));
        assert!(matches!(
            run("JSON.parse({ toString:1 })").unwrap_err().kind,
            ErrorKind::Runtime("TypeError")
        ));
    }

    #[test]
    fn json_rejects_extensions_and_malformed_syntax() {
        let mut document = Document::parse("");
        for text in [
            "",
            "undefined",
            "NaN",
            "Infinity",
            "+1",
            "01",
            "-01",
            ".1",
            "1.",
            "1e",
            "1e+",
            "--1",
            "0x10",
            "true false",
            "[1,]",
            "[,1]",
            "[1 2]",
            "{a:1}",
            "{'a':1}",
            "{\"a\":1,}",
            "{\"a\" 1}",
            "/*comment*/1",
            "//comment\n1",
            "\u{feff}1",
            "\u{a0}1",
            "\"raw\nline\"",
            "\"\\x41\"",
            "\"\\v\"",
            "\"\\u12gg\"",
            "\"unfinished",
            "\"\\🦀\"",
            "[\"\\\"]",
        ] {
            let error = Runtime::new()
                .json_parse(
                    Value::String(JsString::from(text)),
                    Value::Undefined,
                    &mut document,
                )
                .unwrap_err();
            assert!(
                matches!(error.kind, ErrorKind::Runtime("SyntaxError")),
                "{text:?}: {error}"
            );
        }
        assert_eq!(
            run("let name; try { JSON.parse('[1,]'); } catch (error) { name = error.name; } name;")
                .unwrap()
                .to_string(),
            "SyntaxError"
        );
    }

    #[test]
    fn json_unicode_escapes_and_numbers_preserve_supported_values() {
        let mut document = Document::parse("");
        let mut runtime = Runtime::new();
        let value = runtime
            .json_parse(
                Value::String(JsString::from(
                    r#""\uD83E\uDD80\u0000\u2028é\/\b\f\n\r\t\"\\""#,
                )),
                Value::Undefined,
                &mut document,
            )
            .unwrap();
        assert_eq!(value.to_string(), "🦀\0\u{2028}é/\u{8}\u{c}\n\r\t\"\\");
        let serialized = runtime
            .json_stringify(value, Value::Undefined, Value::Undefined, &mut document)
            .unwrap();
        assert_eq!(
            serialized.to_string(),
            "\"🦀\\u0000\u{2028}é/\\b\\f\\n\\r\\t\\\"\\\\\""
        );
        for (text, expected) in [
            (r#""\ud800""#, vec![0xd800]),
            (r#""\udfff""#, vec![0xdfff]),
            (r#""\ud800\u0041""#, vec![0xd800, 65]),
        ] {
            let value = runtime
                .json_parse(
                    Value::String(JsString::from(text)),
                    Value::Undefined,
                    &mut document,
                )
                .unwrap();
            assert_eq!(value, Value::String(JsString::from(expected)));
        }
        assert_eq!(
            run("1 / JSON.parse('-0')").unwrap(),
            Value::Number(f64::NEG_INFINITY)
        );
        assert_eq!(
            run("JSON.parse('1e400')").unwrap(),
            Value::Number(f64::INFINITY)
        );
        assert_eq!(
            run("JSON.parse('9007199254740993')").unwrap(),
            Value::Number(9007199254740992.0)
        );
        assert_eq!(
            run("JSON.stringify([-0,NaN,Infinity,-Infinity,0.000001,0.0000001,1e20,1e21])")
                .unwrap()
                .to_string(),
            "[0,null,null,null,0.000001,1e-7,100000000000000000000,1e+21]"
        );
        for (number, expected) in [
            (f64::from_bits(1), "5e-324"),
            (f64::MAX, "1.7976931348623157e+308"),
            (-0.25, "-0.25"),
            (120.0, "120"),
            (1000000000000000100.0, "1000000000000000100"),
        ] {
            assert_eq!(json_number(number), expected);
        }
    }

    #[test]
    fn json_omits_unrepresentable_properties_and_rejects_cycles() {
        assert_eq!(run("JSON.stringify(JSON)").unwrap().to_string(), "{}");
        assert_eq!(
            run("JSON.stringify(Math,['PI'])").unwrap().to_string(),
            "{\"PI\":3.141592653589793}"
        );
        assert_eq!(run("JSON.stringify(undefined)").unwrap(), Value::Undefined);
        assert_eq!(
            run("JSON.stringify(function() {})").unwrap(),
            Value::Undefined
        );
        assert_eq!(run("JSON.stringify({ missing:undefined, callable:function(){}, keep:null, values:[undefined,function(){},NaN] })").unwrap().to_string(),r#"{"keep":null,"values":[null,null,null]}"#);
        assert_eq!(
            run("const values = []; values.length = 3; JSON.stringify(values)")
                .unwrap()
                .to_string(),
            "[null,null,null]"
        );
        assert_eq!(
            run("const child = { x:1 }; JSON.stringify([child,child]);")
                .unwrap()
                .to_string(),
            r#"[{"x":1},{"x":1}]"#
        );
        for source in [
            "const value = {}; value.self = value; JSON.stringify(value);",
            "const value = []; value.push(value); JSON.stringify(value);",
        ] {
            let error = run(source).unwrap_err();
            assert!(matches!(error.kind, ErrorKind::Runtime("TypeError")));
            assert!(error.message.contains("cyclic"));
        }
        assert_eq!(run("const value = {}; value.self=value; JSON.stringify(value,function(key,value) { if (key==='self') return undefined; return value; });").unwrap().to_string(),"{}");
    }

    #[test]
    fn json_replacer_to_json_and_indentation_follow_callback_order() {
        assert_eq!(run("let log = ''; const value = { toJSON:function(key) { log += 'toJSON:' + key + ';'; return { z:3,a:1 }; } }; const text = JSON.stringify(value,function(key,item) { log += 'replace:' + key + ';'; if (key === 'z') return item * 2; return item; },2); log + text;").unwrap().to_string(),"toJSON:;replace:;replace:z;replace:a;{\n  \"z\": 6,\n  \"a\": 1\n}");
        assert_eq!(run("JSON.stringify({ a:1,b:2,nested:{a:3,b:4},arr:[1,2] },['b','nested','arr','b',true],1)").unwrap().to_string(),"{\n \"b\": 2,\n \"nested\": {\n  \"b\": 4\n },\n \"arr\": [\n  1,\n  2\n ]\n}");
        assert_eq!(
            run("JSON.stringify({ a:1, '2':2 },[2,'a',2],99)")
                .unwrap()
                .to_string(),
            "{\n          \"2\": 2,\n          \"a\": 1\n}"
        );
        assert_eq!(
            run("JSON.stringify([1],null,'abcdefghijk')")
                .unwrap()
                .to_string(),
            "[\nabcdefghij1\n]"
        );
        assert_eq!(
            run("JSON.stringify([1],null,'123456789🦀').charCodeAt(11)").unwrap(),
            Value::Number(0xd83e as f64)
        );
        assert_eq!(
            run("JSON.stringify({a:1},false,true)").unwrap().to_string(),
            r#"{"a":1}"#
        );
        assert_eq!(run("const value = { a:2 }; JSON.stringify(value,function(key,item) { if(key==='a') return this.a + 3; return item; });").unwrap().to_string(),r#"{"a":5}"#);
        assert_eq!(
            run("JSON.stringify(1,function(){return undefined;})").unwrap(),
            Value::Undefined
        );
    }

    #[test]
    fn json_reviver_walks_children_deletes_and_exposes_original_source() {
        assert_eq!(run(r#"let log = ''; const value = JSON.parse('{"b":2,"a":[1,2]}',function(key,item,context) { log += key + ';'; if (key==='b') return undefined; if (key==='0') return undefined; if (typeof item==='number') return item*3; return item; }); log + JSON.stringify(value);"#).unwrap().to_string(),r#"b;0;1;a;;{"a":[null,6]}"#);
        assert_eq!(run(r#"JSON.parse('{"precise":9007199254740993}',function(key,item,context) { if (key==='precise') return context.source; return item; }).precise;"#).unwrap().to_string(),"9007199254740993");
        assert_eq!(
            run(r#"JSON.parse('  -0  ',function(key,item,context) { return context.source; });"#)
                .unwrap()
                .to_string(),
            "-0"
        );
        assert_eq!(run(r#"JSON.parse('{"a":1,"b":2}',function(key,item,context) { if (key==='a') this.b=3; if (key==='b') return typeof context.source; return item; }).b;"#).unwrap().to_string(),"undefined");
        assert_eq!(run(r#"JSON.parse('{"a":1,"b":0}',function(key,item,context) { if (key==='a') this.b=-0; if (key==='b') return typeof context.source; return item; }).b;"#).unwrap().to_string(),"undefined");
        assert_eq!(run(r#"JSON.parse('{"a":1,"a":2}',function(key,item,context) { if(key==='a')return context.source; return item; }).a;"#).unwrap().to_string(),"2");
        assert_eq!(
            run("JSON.parse('3',function(){ return undefined; })").unwrap(),
            Value::Undefined
        );
        assert_eq!(run("JSON.parse('true',false)").unwrap(), Value::Bool(true));
        assert_eq!(run("let caught; try { JSON.parse('1',function(){ throw 'reviver failure'; }); } catch(error) { caught=error; } caught;").unwrap().to_string(),"reviver failure");
    }

    #[test]
    fn json_limits_remain_uncatchable_in_callbacks_and_native_recursion() {
        let mut document = Document::parse("");
        for text in [
            format!(
                "{}0{}",
                "[".repeat(MAX_DEPTH + 1),
                "]".repeat(MAX_DEPTH + 1)
            ),
            format!("[{}0]", "0,".repeat(MAX_TOKENS)),
            " ".repeat(MAX_STRING + 1),
        ] {
            let error = Runtime::new()
                .json_parse(
                    Value::String(JsString::from(text)),
                    Value::Undefined,
                    &mut document,
                )
                .unwrap_err();
            assert!(error.is_resource_limit(), "{error}");
        }
        for body in [
            "JSON.parse('1',function() { while(true) {} });",
            "JSON.stringify(1,function() { while(true) {} });",
            "function recur(key,value) { return JSON.stringify(value,recur); } JSON.stringify(1,recur);",
            "let value={toString:function(){return JSON.parse(value);}};JSON.parse(value);",
            "let nested=0;for(let i=0;i<97;i++)nested=[nested];let depth=0;function recur(){depth++;if(depth<12)return JSON.stringify(1,recur);return JSON.stringify(nested);}JSON.stringify(1,recur);",
            "function recur(){return recur.apply(null,[]);}recur();",
            "function Recur(){new Recur();}new Recur();",
            "let text='xxxxxxxx'; for(let i=0;i<13;i++)text+=text; const values=[text,text,text,text,text]; JSON.stringify(values);",
        ] {
            let mut runtime = Runtime::new();
            let source = format!(
                "let caught=false; let cleaned=false; try {{ {body} }} catch(error) {{ caught=true; }} finally {{ cleaned=true; }}"
            );
            let error = runtime.execute(&source, &mut document).unwrap_err();
            assert!(error.is_resource_limit(), "{body}: {error}");
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.lookup(1, "cleaned").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.json_depth, 0);
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.eval_depth, 0);
            assert_eq!(runtime.stack_units, 0);
        }
        assert!(
            run("JSON.stringify(document)")
                .unwrap_err()
                .message
                .contains("host objects")
        );
        assert!(
            run("JSON.parse('{\"__proto__\":{\"fetch\":1}}'); fetch('https://example.com');")
                .unwrap_err()
                .message
                .contains("not defined")
        );
    }

    #[test]
    fn deeply_nested_function_bodies_keep_shared_call_and_work_limits() {
        let source = format!(
            "function recur(){{{}return recur();{}}}recur();",
            "{".repeat(40),
            "}".repeat(40)
        );
        Runtime::parse_only(&source).unwrap();
        let mut runtime = Runtime::new();
        let error = runtime
            .execute(&source, &mut Document::parse(""))
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.stack_units, 0);
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.eval_depth, 0);
    }

    #[test]
    fn json_mutation_smoke_and_native_allocation_limits() {
        let seeds = [
            r#"{"x":[1,true,null,"é🦀"],"y":-1e-9}"#,
            r#"[{},[],"\u0000\ud83e\udd80"]"#,
            r#"{"__proto__":1,"__proto__":2}"#,
        ];
        let inserts = [
            '{', '}', '[', ']', '"', '\\', ':', ',', '0', 'e', 'é', '🦀', '\0', '\n',
        ];
        let mut random = 0x1234_5678u32;
        let mut document = Document::parse("");
        for iteration in 0..500 {
            let mut characters: Vec<char> = seeds[iteration % seeds.len()].chars().collect();
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            let position = random as usize % characters.len();
            match iteration % 3 {
                0 => characters.insert(position, inserts[random as usize % inserts.len()]),
                1 => {
                    characters.remove(position);
                }
                _ => characters.truncate(position),
            }
            let source: String = characters.into_iter().collect();
            let mut runtime = Runtime::new();
            if let Ok(value) = runtime.json_parse(
                Value::String(JsString::from(source)),
                Value::Undefined,
                &mut document,
            ) {
                let encoded = runtime
                    .json_stringify(value, Value::Undefined, Value::Undefined, &mut document)
                    .unwrap();
                let value = runtime
                    .json_parse(encoded.clone(), Value::Undefined, &mut document)
                    .unwrap();
                assert_eq!(
                    runtime
                        .json_stringify(value, Value::Undefined, Value::Undefined, &mut document)
                        .unwrap(),
                    encoded
                );
            }
            assert_eq!(runtime.json_depth, 0);
        }
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP - 32;
        let error = runtime
            .json_parse(
                Value::String(JsString::from("{\"a\":[1,2,3]}")),
                Value::Undefined,
                &mut document,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert!(error.message.contains("allocation"));
        assert_eq!(runtime.json_depth, 0);
    }

    fn string_units(source: &str) -> Vec<u16> {
        let Value::String(value) = run(source).unwrap() else {
            panic!("expected string");
        };
        value.units().to_vec()
    }

    #[test]
    fn utf16_literals_constructors_and_comparisons_preserve_code_units() {
        assert_eq!(
            string_units(r#"'\ud800\uD83E\uDD80\udfff'"#),
            [0xd800, 0xd83e, 0xdd80, 0xdfff]
        );
        assert_eq!(
            string_units(r#"'\u{1f980}\u{d800}'"#),
            [0xd83e, 0xdd80, 0xd800]
        );
        assert_eq!(
            run(r#"'\ud83e\udd80' === '🦀' && '\ud800' !== '�' && '🦀' < '\ue000'"#).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            string_units("String.fromCharCode(55296,129408,-1,NaN)"),
            [0xd800, 0xf980, 0xffff, 0]
        );
        assert_eq!(
            string_units("String.fromCodePoint(129408,55296)"),
            [0xd83e, 0xdd80, 0xd800]
        );
        assert!(matches!(
            run("String.fromCodePoint(1114112)").unwrap_err().kind,
            ErrorKind::Runtime("RangeError")
        ));
        assert!(run(r#"'\u{110000}'"#).is_err());
        assert_eq!(string_units(r#"String('\ud800')"#), [0xd800]);
    }

    #[test]
    fn utf16_indexing_slicing_and_position_arguments_use_code_units() {
        assert_eq!(run("'A🦀Z'.length").unwrap(), Value::Number(4.0));
        assert_eq!(string_units("'A🦀Z'[1]"), [0xd83e]);
        assert_eq!(string_units("'A🦀Z'.charAt(2)"), [0xdd80]);
        assert_eq!(
            run("'A🦀Z'.charCodeAt(1)").unwrap(),
            Value::Number(0xd83e as f64)
        );
        assert_eq!(
            run("'A🦀Z'.codePointAt(1)").unwrap(),
            Value::Number(129408.0)
        );
        assert_eq!(
            run("'A🦀Z'.codePointAt(2)").unwrap(),
            Value::Number(0xdd80 as f64)
        );
        assert_eq!(string_units("'A🦀Z'.slice(1,2)"), [0xd83e]);
        assert_eq!(string_units("'A🦀Z'.substring(2,1)"), [0xd83e]);
        assert_eq!(string_units("'A🦀Z'.slice(-2,-1)"), [0xdd80]);
        assert_eq!(run("'abc'.slice(-0.5)").unwrap().to_string(), "abc");
        assert_eq!(
            run("'abc'.substring(NaN,undefined)").unwrap().to_string(),
            "abc"
        );
        assert_eq!(run("'abc'.charAt(-0.5)").unwrap().to_string(), "a");
        assert_eq!(run("'abc'.charAt(Infinity)").unwrap().to_string(), "");
        assert!(
            run("'abc'.charCodeAt(-1)")
                .unwrap()
                .as_number()
                .unwrap()
                .is_nan()
        );
        assert_eq!(run("'abc'.codePointAt(3)").unwrap(), Value::Undefined);
        assert_eq!(run("'abc'['01']").unwrap(), Value::Undefined);
    }

    #[test]
    fn utf16_search_split_join_case_and_concatenation_are_lossless() {
        assert_eq!(run(r#"'🦀'.includes('\udd80') && '🦀'.startsWith('\udd80',1) && '🦀'.endsWith('\ud83e',1)"#).unwrap(),Value::Bool(true));
        assert_eq!(
            run(r#"'A🦀Z'.indexOf('\udd80')"#).unwrap(),
            Value::Number(2.0)
        );
        assert_eq!(run("'abcabc'.indexOf('a',1)").unwrap(), Value::Number(3.0));
        assert_eq!(
            run("JSON.stringify('🦀'.split(''))").unwrap().to_string(),
            r#"["\ud83e","\udd80"]"#
        );
        assert_eq!(
            run("JSON.stringify('a,b,'.split(',',2))")
                .unwrap()
                .to_string(),
            r#"["a","b"]"#
        );
        assert_eq!(
            run("JSON.stringify('abc'.split(undefined))")
                .unwrap()
                .to_string(),
            r#"["abc"]"#
        );
        assert_eq!(
            run("JSON.stringify('abc'.split('',0))")
                .unwrap()
                .to_string(),
            "[]"
        );
        assert_eq!(string_units(r#"'\ud83e'+'\udd80'"#), [0xd83e, 0xdd80]);
        assert_eq!(
            string_units(r#"['\ud800','\udfff'].join('\ud801')"#),
            [0xd800, 0xd801, 0xdfff]
        );
        assert_eq!(string_units(r#"String([['\ud800']])"#), [0xd800]);
        assert_eq!(
            string_units(r#"'\ud800ßΣ\udfff'.toUpperCase()"#),
            [0xd800, 83, 83, 0x3a3, 0xdfff]
        );
        assert_eq!(
            string_units(r#"'ΟΣ\ud800'.toLowerCase()"#),
            [0x3bf, 0x3c2, 0xd800]
        );
        assert_eq!(string_units(r#"'\ufeff \ud800 \u00a0'.trim()"#), [0xd800]);
        assert_eq!(run("'\\u0085'.trim().length").unwrap(), Value::Number(1.0));
    }

    #[test]
    fn utf16_property_keys_and_json_callbacks_do_not_alias_surrogates() {
        assert_eq!(run(r#"const object={'\ud800':1,'\ud801':2}; object['\ud800']+=3; JSON.stringify(object);"#).unwrap().to_string(),r#"{"\ud800":4,"\ud801":2}"#);
        assert_eq!(run(r#"const object=JSON.parse('{"\\ud800":1,"\\ud801":2}'); object['\ud800']+object['\ud801'];"#).unwrap(),Value::Number(3.0));
        assert_eq!(
            run(r#"JSON.stringify({'\ud800':1,'\ud801':2},['\ud801']);"#)
                .unwrap()
                .to_string(),
            r#"{"\ud801":2}"#
        );
        assert_eq!(run(r#"JSON.stringify(JSON.parse('{"\\ud800":1}',function(key,value,context) { if(key.charCodeAt(0)===55296)return context.source;return value;}));"#).unwrap().to_string(),r#"{"\ud800":"1"}"#);
        assert_eq!(
            run(
                r#"JSON.parse('"\\ud800"',function(key,value,context) { return context.source; });"#
            )
            .unwrap()
            .to_string(),
            r#""\ud800""#
        );
        assert_eq!(
            run(r#"JSON.stringify('\ud800',function(key,value) { return value; })"#)
                .unwrap()
                .to_string(),
            r#""\ud800""#
        );
    }

    #[test]
    fn every_utf16_code_unit_round_trips_through_json() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let original = Value::String(JsString::from((0..=u16::MAX).collect::<Vec<_>>()));
        let encoded = runtime
            .json_stringify(
                original.clone(),
                Value::Undefined,
                Value::Undefined,
                &mut document,
            )
            .unwrap();
        let decoded = runtime
            .json_parse(encoded, Value::Undefined, &mut document)
            .unwrap();
        assert_eq!(decoded, original);
        let raw = Value::String(JsString::from(vec![34, 0xd800, 34]));
        assert_eq!(
            runtime
                .json_parse(raw, Value::Undefined, &mut document)
                .unwrap(),
            Value::String(JsString::from(vec![0xd800]))
        );
    }

    #[test]
    fn utf16_numeric_conversion_uses_ecmascript_whitespace_and_grammar() {
        assert_eq!(
            run("Number('\\ufeff +1.5e2\\u00a0')").unwrap(),
            Value::Number(150.0)
        );
        assert_eq!(
            run("Number('0x20000000000003')").unwrap(),
            Value::Number(9007199254740996.0)
        );
        assert_eq!(
            run("Number('0b101') + Number('0o77')").unwrap(),
            Value::Number(68.0)
        );
        assert_eq!(
            run("1 / Number('-0')").unwrap(),
            Value::Number(f64::NEG_INFINITY)
        );
        for source in [
            "Number('\\ud800')",
            "Number('inf')",
            "Number('\\u00851')",
            "Number('+0x1')",
            "parseFloat('\\u00851')",
        ] {
            assert!(
                run(source).unwrap().as_number().unwrap().is_nan(),
                "{source}"
            );
        }
        assert_eq!(
            run("parseFloat('\\ufeff12.5\\ud800')").unwrap(),
            Value::Number(12.5)
        );
        assert_eq!(
            run("parseInt('\\ufeff12\\ud800')").unwrap(),
            Value::Number(12.0)
        );
        assert_eq!(
            run("parseInt('900719925474099267')").unwrap(),
            Value::Number(900719925474099300.0)
        );
        assert_eq!(
            run("parseInt('20000000000003',16)").unwrap(),
            Value::Number(9007199254740996.0)
        );
        assert_eq!(run("parseInt('12',Infinity)").unwrap(), Value::Number(12.0));
        assert_eq!(run("String(1e21)").unwrap().to_string(), "1e+21");
    }

    #[test]
    fn utf16_text_content_preserves_script_units_with_explicit_presentation_replacement() {
        let mut document = Document::parse("<p id=out></p>");
        let mut runtime = Runtime::new();
        let result=runtime.execute(r#"const original='\ud800🦀\udfff'; const out=document.getElementById('out'); out.textContent=original; original;"#,&mut document).unwrap();
        assert_eq!(
            result,
            Value::String(JsString::from(vec![0xd800, 0xd83e, 0xdd80, 0xdfff]))
        );
        assert_eq!(
            document.text_content(document.query_selector("#out").unwrap()),
            "�🦀�"
        );
        assert_eq!(
            runtime.execute("out.textContent", &mut document).unwrap(),
            Value::String(JsString::from(vec![0xd800, 0xd83e, 0xdd80, 0xdfff]))
        );
    }

    #[test]
    fn utf16_growth_and_expensive_native_search_remain_uncatchable() {
        for body in [
            "let text=String.fromCharCode(55296); while(true) {text+=text;}",
            "let text='a';for(let i=0;i<15;i++)text+=text; let needle=text+'b';while(true){text.includes(needle.slice(1));}",
            "let text='1';for(let i=0;i<15;i++)text+=text; while(true){ +text; }",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let source = format!("let caught=false;try {{{body}}} catch(error) {{caught=true;}}");
            assert!(
                runtime
                    .execute(&source, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
        }
    }

    #[test]
    fn utf16_repeated_host_writes_charge_linear_string_work() {
        for assignment in [
            "array.length=text;",
            "out.textContent=text;",
            "f.apply(null,{length:text});",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("<p id=out></p>");
            let source = format!(
                "function f(){{}}let caught=false;let attempts=0;let text='0';for(let i=0;i<15;i++)text+=text;const array=[];const out=document.getElementById('out');try{{while(true){{attempts++;{assignment}}}}}catch(error){{caught=true;}}"
            );
            assert!(
                runtime
                    .execute(&source, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            let attempts = runtime
                .lookup(1, "attempts")
                .unwrap()
                .1
                .as_number()
                .unwrap();
            assert!((1.0..100.0).contains(&attempts), "{attempts} host writes");
        }
    }

    #[test]
    fn function_scope_hoisting_lexical_this_and_parse_contexts_are_preserved() {
        let (mut runtime, mut document) = upstream_harness();
        runtime.execute("function outer(){assert.sameValue(x,undefined);if(false){var x=7;}if(true){var x=9;}var x;return x;}assert.sameValue(outer(),9);function replace(){replace=7;}replace();assert.sameValue(replace,7);const named=function inner(x){return x?inner(0):3;};assert.sameValue(named(1),3);assert.sameValue(typeof inner,'undefined');function maker(){return ()=>this.x;}const arrow=maker.call({x:4});assert.sameValue(arrow.call({x:9}),4);function count(){return arguments.length;}assert.sameValue(count.apply(null,{0:1,1:2,length:2}),2);assert.sameValue(count.apply(null,{length:NaN}),0);assert.sameValue(1 instanceof (()=>1),false);", &mut document).unwrap();
        for source in [
            "return 1;",
            "break;",
            "continue;",
            "while(true){function bad(){break;}}",
            "switch(1){case 1:continue;}",
        ] {
            assert!(
                Runtime::parse_only(source).unwrap_err().is_parse_error(),
                "{source}"
            );
        }
    }

    #[test]
    fn intrinsic_properties_are_hidden_and_function_prototype_is_callable() {
        let (mut runtime, mut document) = upstream_harness();
        runtime.execute("function F(){}F.extra=1;assert.compareArray(Object.keys(F),['extra']);assert.compareArray(Object.keys(F.prototype),[]);assert.compareArray(Object.keys(Object.prototype),[]);assert.compareArray(Object.keys(new Error('bad')),[]);assert.sameValue(JSON.stringify(new Error('bad')),'{}');assert.sameValue(new Error().hasOwnProperty('message'),false);assert.sameValue(new Error('').hasOwnProperty('message'),true);assert.sameValue(typeof Function.prototype,'function');assert.sameValue(Function.prototype(),undefined);assert.sameValue(Object.getPrototypeOf(F),Function.prototype);assert.sameValue(F instanceof Function,true);assert.sameValue(Function.prototype instanceof Function,false);assert.throws(TypeError,function(){String.prototype.toString.call(1);});", &mut document).unwrap();
        assert_eq!(
            runtime
                .execute("(new Function('return 1'))()", &mut document)
                .unwrap(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn foreign_namespace_names_and_fragment_import_metadata_survive_script_access() {
        let mut document = Document::parse("<div id=out></div>");
        let mut runtime = Runtime::new();
        runtime.execute("document.getElementById('out').innerHTML='<svg><linearGradient id=paint xlink:href=\"#x\"></linearGradient></svg><math><mi id=letter>x</mi></math>';", &mut document).unwrap();
        assert_eq!(
            runtime
                .execute("document.getElementById('out').nodeName", &mut document)
                .unwrap()
                .to_string(),
            "DIV"
        );
        assert_eq!(
            runtime
                .execute("document.getElementById('paint').nodeName", &mut document)
                .unwrap()
                .to_string(),
            "linearGradient"
        );
        assert_eq!(
            runtime
                .execute(
                    "document.getElementById('paint').namespaceURI",
                    &mut document
                )
                .unwrap()
                .to_string(),
            Namespace::Svg.uri()
        );
        assert_eq!(
            runtime
                .execute(
                    "document.getElementById('letter').namespaceURI",
                    &mut document
                )
                .unwrap()
                .to_string(),
            Namespace::MathMl.uri()
        );
        let paint = document.query_selector("#paint").unwrap();
        let NodeKind::Element(element) = &document.nodes[paint].kind else {
            panic!("missing imported element")
        };
        assert_eq!(
            element.attr_namespaces.get("xlink:href"),
            Some(&crate::dom::AttributeNamespace::XLink)
        );
    }

    #[test]
    fn thrown_constructor_diagnostics_charge_reused_large_names() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let error = runtime.execute("function F(){}let text='a';for(let i=0;i<15;i++)text+=text;Object.defineProperty(F,'name',{value:text});const item=new F();let attempts=0;let caught=0;while(true){attempts++;try{throw item;}catch(e){caught++;}}", &mut document).unwrap_err();
        assert!(error.is_resource_limit());
        let attempts = runtime
            .lookup(1, "attempts")
            .unwrap()
            .1
            .as_number()
            .unwrap();
        let caught = runtime.lookup(1, "caught").unwrap().1.as_number().unwrap();
        assert!((1.0..100.0).contains(&attempts));
        assert_eq!(caught, attempts - 1.0);
    }

    #[test]
    fn document_metadata_bindings_ignore_foreign_and_inactive_elements() {
        let mut document = Document::parse(
            "<svg><title id=svg-title>foreign</title></svg><template><title id=template-title>inactive</title></template>",
        );
        let mut runtime = Runtime::new();
        assert_eq!(
            runtime
                .execute("document.title", &mut document)
                .unwrap()
                .to_string(),
            ""
        );
        runtime
            .execute("document.title='active';", &mut document)
            .unwrap();
        assert_eq!(
            runtime
                .execute("document.title", &mut document)
                .unwrap()
                .to_string(),
            "active"
        );
        assert_eq!(
            document.text_content(document.query_selector("#svg-title").unwrap()),
            "foreign"
        );
        assert_eq!(
            document.text_content(
                document
                    .query_selector_from(
                        document
                            .template_contents(document.query_selector("template").unwrap())
                            .unwrap(),
                        "#template-title"
                    )
                    .unwrap()
            ),
            "inactive"
        );
        let html = document_element(&document).unwrap();
        let head = html_document_child(&document, "head").unwrap();
        let body = html_document_child(&document, "body").unwrap();
        document.remove_child(html, head);
        document.remove_child(html, body);
        for tag in ["head", "body"] {
            let foreign = document.create_element_ns(Namespace::Svg, tag);
            document.append_child(html, foreign);
        }
        assert_eq!(
            runtime.execute("document.head", &mut document).unwrap(),
            Value::Null
        );
        assert_eq!(
            runtime.execute("document.body", &mut document).unwrap(),
            Value::Null
        );
        let count = document.nodes.len();
        runtime
            .execute(
                "document.title='must not append to foreign head';",
                &mut document,
            )
            .unwrap();
        assert_eq!(document.nodes.len(), count);
        document.remove_child(document.root, html);
        assert_eq!(
            runtime
                .execute("document.documentElement", &mut document)
                .unwrap(),
            Value::Null
        );
    }

    #[test]
    fn canonical_error_identity_cannot_be_spoofed_by_constructor_names_or_properties() {
        for source in [
            "function Fake(){}Object.defineProperty(Fake,'name',{value:'TypeError'});throw new Fake();",
            "throw {constructor:TypeError};",
            "throw Object.create(TypeError.prototype);",
            "const error=new TypeError();error.constructor=RangeError;throw error;",
        ] {
            assert_eq!(
                run(source).unwrap_err().intrinsic_error_name(),
                None,
                "{source}"
            );
        }
        for source in [
            "throw new TypeError('real');",
            "try{null.x;}catch(error){throw error;}",
            "Object.defineProperty(TypeError,'name',{value:'ForgedDiagnostic'});throw new TypeError();",
        ] {
            assert_eq!(
                run(source).unwrap_err().intrinsic_error_name(),
                Some("TypeError"),
                "{source}"
            );
        }
        assert_eq!(
            run("JSON.parse('{')").unwrap_err().intrinsic_error_name(),
            Some("SyntaxError")
        );
        let error = run("function Fake(){}Object.defineProperty(Fake,'name',{value:'TypeError'});throw new Fake();").unwrap_err();
        assert_eq!(error.name(), "TypeError");
        assert_eq!(error.intrinsic_error_name(), None);
    }

    #[test]
    fn inner_html_uses_context_tree_modes_and_foreign_namespaces() {
        let mut document = Document::parse(
            "<table id=t></table><select id=s></select><textarea id=text></textarea><svg id=v></svg><template id=unsupported>kept</template>",
        );
        let mut runtime = Runtime::new();
        runtime.execute(r##"
            document.getElementById('t').innerHTML='<tr><td>cell</td></tr>';
            document.getElementById('s').innerHTML='<div>word</div><option>choice</option>';
            document.getElementById('text').innerHTML='a&amp;<b>';
            document.getElementById('v').innerHTML='<linearGradient id="paint" viewBox="0 0 1 1" xlink:href="#x"/>';
        "##,&mut document).unwrap();
        assert!(document.query_selector("#t > tbody > tr > td").is_some());
        // Current WHATWG customizable-select parsing retains generic children.
        assert!(document.query_selector("#s > div").is_some());
        assert!(document.query_selector("#s > option").is_some());
        assert_eq!(
            document.text_content(document.query_selector("#text").unwrap()),
            "a&<b>"
        );
        let paint = document.query_selector("#paint").unwrap();
        assert_eq!(document.namespace(paint), Some(Namespace::Svg));
        assert_eq!(document.tag(paint), Some("linearGradient"));
        assert_eq!(document.attr(paint, "viewBox"), Some("0 0 1 1"));
        runtime
            .execute(
                "document.getElementById('unsupported').innerHTML='<b>changed</b>';",
                &mut document,
            )
            .unwrap();
        let template = document.query_selector("#unsupported").unwrap();
        let content = document.template_contents(template).unwrap();
        assert_eq!(document.text_content(content), "changed");
        assert!(document.nodes[template].children.is_empty());
        assert!(document.query_selector("#unsupported b").is_none());
    }

    #[test]
    fn literal_accessors_methods_and_holes_keep_property_semantics() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var object={saved:2,get value(){return this.saved;},set value(next){this.saved=next;},method(a){return this.saved+a;}};
            object.value=4;assert.sameValue(object.value,4);assert.sameValue(object.method(3),7);
            var desc=Object.getOwnPropertyDescriptor(object,'value');
            assert.sameValue(desc.get.name,'get value');assert.sameValue(desc.get.length,0);
            assert.sameValue(desc.set.name,'set value');assert.sameValue(desc.set.length,1);
            assert.sameValue(desc.enumerable,true);assert.sameValue(desc.configurable,true);
            assert.sameValue(object.method.hasOwnProperty('prototype'),false);
            assert.throws(TypeError,function(){new object.method();});
            var a=[,undefined,,3,];assert.sameValue(a.length,4);assert.compareArray(Object.keys(a),['1','3']);
            assert.sameValue(JSON.stringify(a),'[null,null,null,3]');
            assert.sameValue(Array.isArray(Array.prototype),true);assert.sameValue(Array.prototype.length,0);
            assert.sameValue(Object.getPrototypeOf([]),Array.prototype);
            var reads=0;var json=JSON.stringify({get a(){reads++;return 1;},b:2});
            assert.sameValue(json,'{"a":1,"b":2}');assert.sameValue(reads,1);
            var keep=JSON.parse('{"a":1,"b":2}',function(k,v){if(k==='a'){Object.defineProperty(this,'b',{value:9,configurable:false});}if(k==='b')return undefined;return v;});
            assert.sameValue(keep.b,9);
        "#,&mut document).unwrap();
        for source in [
            "({get x(a){return a;}})",
            "({set x(){}})",
            "({set x(a,b){}})",
        ] {
            assert!(Runtime::parse_only(source).unwrap_err().is_parse_error());
        }
    }

    fn property_harness() -> (Runtime, Document) {
        let (mut runtime, mut document) = upstream_harness();
        runtime
            .execute(
                include_str!("../tests/upstream/test262/harness/propertyHelper.js"),
                &mut document,
            )
            .unwrap();
        (runtime, document)
    }

    #[test]
    fn disclosure_toggle_event_constructor_converts_dictionaries_and_protects_private_state() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var source=document.createElement('summary');
            var e=new ToggleEvent('toggle',{oldState:'closed',newState:'open',source:source,bubbles:true,cancelable:true,composed:true});
            assert.sameValue(e instanceof ToggleEvent,true);assert.sameValue(e instanceof Event,true);
            assert.sameValue(e instanceof CustomEvent,false);
            assert.sameValue(Object.prototype.toString.call(e),'[object ToggleEvent]');
            assert.sameValue(e.oldState,'closed');assert.sameValue(e.newState,'open');assert.sameValue(e.source,source);
            assert.sameValue(e.bubbles,true);assert.sameValue(e.cancelable,true);assert.sameValue(e.composed,true);
            assert.sameValue(e.isTrusted,false);assert.sameValue(ToggleEvent.AT_TARGET,2);
            assert.sameValue(ToggleEvent.length,1);assert.sameValue(ToggleEvent.name,'ToggleEvent');
            e.oldState='other';assert.sameValue(e.oldState,'closed');
            assert.throws(TypeError,function(){'use strict';e.source=null;});
            var getter=Object.getOwnPropertyDescriptor(ToggleEvent.prototype,'oldState').get;
            assert.sameValue(getter.call(e),'closed');assert.throws(TypeError,()=>getter.call(new Event('toggle')));
            assert.throws(TypeError,()=>getter.call({oldState:'closed'}));
            assert.throws(TypeError,()=>ToggleEvent('toggle'));assert.throws(TypeError,()=>new ToggleEvent());
            assert.throws(TypeError,()=>new ToggleEvent('x',1));
            assert.throws(TypeError,()=>new ToggleEvent('x',{source:{}}));
            assert.throws(TypeError,()=>new ToggleEvent('x',{source:document}));
            assert.throws(TypeError,()=>new ToggleEvent('x',{source:document.createTextNode('x')}));
            var defaults=new ToggleEvent('x',null);assert.sameValue(defaults.oldState,'');
            assert.sameValue(defaults.newState,'');assert.sameValue(defaults.source,null);
            var converted=new ToggleEvent('x',{oldState:null,newState:undefined,source:undefined});
            assert.sameValue(converted.oldState,'null');assert.sameValue(converted.newState,'');
            assert.sameValue(new ToggleEvent('x',{oldState:'\uD800'}).oldState,'\uD800');
            e.initEvent('again',false,false);assert.sameValue(e.oldState,'closed');assert.sameValue(e.source,source);
            var log='';function state(label){return {toString(){log+=label+';';return label;}};}
            var ordered=new ToggleEvent(state('type'),{
                get bubbles(){log+='bubbles;';return false;},get cancelable(){log+='cancelable;';return false;},
                get composed(){log+='composed;';return false;},get newState(){log+='get new;';return state('new');},
                get oldState(){log+='get old;';return state('old');},get source(){log+='source;';return null;}
            });
            assert.sameValue(log,'type;bubbles;cancelable;composed;get new;new;get old;old;source;');
            assert.sameValue(ordered.oldState,'old');assert.sameValue(ordered.newState,'new');
            var reason={},after=0;
            assert.throws(TypeError,()=>new ToggleEvent('x',{newState:{toString(){return {};},valueOf(){return {};}}}));
            try{new ToggleEvent('x',{get newState(){throw reason;},get oldState(){after++;return '';}});}catch(caught){assert.sameValue(caught,reason);}
            assert.sameValue(after,0);
        "#,&mut document).unwrap();
    }

    #[test]
    fn disclosure_reflection_coerces_names_but_not_booleans_and_respects_namespaces() {
        let (mut runtime, _) = property_harness();
        let mut document = Document::parse(
            "<details id=a></details><details id=b></details><svg><details id=foreign open name=svg /></svg>",
        );
        runtime.execute(r#"
            var a=document.getElementById('a'),b=document.getElementById('b'),foreign=document.getElementById('foreign');
            assert.sameValue(a.open,false);assert.sameValue(a.name,'');
            var conversions=0;var truthy={toString(){conversions++;throw 'coerced boolean';},valueOf(){conversions++;throw 'coerced boolean';}};
            a.open=truthy;assert.sameValue(a.open,true);assert.sameValue(a.getAttribute('open'),'');assert.sameValue(conversions,0);
            a.open='false';assert.sameValue(a.open,true);a.open=0;assert.sameValue(a.open,false);
            a.setAttribute('OPEN','false');assert.sameValue(a.open,true);
            a.removeAttribute('open');assert.sameValue(a.open,false);
            a.name={toString(){conversions++;return 'group';}};b.name='group';
            assert.sameValue(a.getAttribute('name'),'group');assert.sameValue(conversions,1);
            a.open=true;b.open=true;assert.sameValue(a.open,false);assert.sameValue(b.open,true);
            a.name=null;assert.sameValue(a.name,'null');
            a.name='\uD800';assert.sameValue(a.name,'\uFFFD');
            assert.sameValue(foreign.open,undefined);assert.sameValue(foreign.name,undefined);
            assert.sameValue(foreign.getAttribute('name'),'svg');
            var previous=a.name,reason={};try{a.name={toString(){throw reason;}};}catch(e){assert.sameValue(e,reason);}
            assert.sameValue(a.name,previous);
        "#,&mut document).unwrap();
    }

    #[test]
    fn disclosure_checkpoint_coalesces_delivers_trusted_events_and_handles_reentrant_changes() {
        let (mut runtime, _) = property_harness();
        let mut document = Document::parse(
            "<details id=d ontoggle='inlineCount++;'><summary>Label</summary><p>Content</p></details>",
        );
        runtime.execute(r#"
            var d=document.getElementById('d'),count=0,inlineCount=0,capture=0,bubble=0,last;
            document.addEventListener('toggle',function(){capture++;},true);
            document.addEventListener('toggle',function(){bubble++;});
            d.addEventListener('toggle',function(e){
                count++;last=e;assert.sameValue(e.target,d);assert.sameValue(e.currentTarget,d);
                assert.sameValue(e instanceof ToggleEvent,true);assert.sameValue(e.isTrusted,true);
                assert.sameValue(e.bubbles,false);assert.sameValue(e.cancelable,false);assert.sameValue(e.source,null);
                e.preventDefault();assert.sameValue(e.defaultPrevented,false);
            });
            d.open=true;d.open=false;assert.sameValue(count,0);assert.sameValue(inlineCount,0);
        "#,&mut document).unwrap();
        runtime.dispatch_details_toggles(&mut document).unwrap();
        runtime.execute(r#"
            assert.sameValue(count,1);assert.sameValue(inlineCount,1);assert.sameValue(capture,1);assert.sameValue(bubble,0);
            assert.sameValue(last.oldState,'closed');assert.sameValue(last.newState,'closed');
            assert.sameValue(last.currentTarget,null);assert.sameValue(last.eventPhase,0);assert.sameValue(last.composedPath().length,0);
            var trace='';d.ontoggle=function(e){trace+=e.oldState+'>'+e.newState+';';if(d.open){d.open=false;}return false;};
            d.open=true;assert.sameValue(count,1);
        "#,&mut document).unwrap();
        runtime.dispatch_details_toggles(&mut document).unwrap();
        runtime.execute(r#"
            assert.sameValue(count,3);assert.sameValue(trace,'closed>open;closed>closed;');assert.sameValue(d.open,false);
            assert.sameValue(inlineCount,1);assert.sameValue(last.defaultPrevented,false);
            var detached=document.createElement('details'),detachedCount=0;
            detached.ontoggle=function(e){detachedCount++;assert.sameValue(e.composedPath().length,1);};detached.open=true;
        "#,&mut document).unwrap();
        runtime.dispatch_details_toggles(&mut document).unwrap();
        runtime
            .execute(
                "assert.sameValue(detachedCount,1);assert.sameValue(capture,3);",
                &mut document,
            )
            .unwrap();
        assert!(!document.has_pending_details_toggles());
    }

    #[test]
    fn disclosure_checkpoint_limits_tasks_and_preserves_unstarted_notifications_on_failure() {
        let (mut runtime, _) = property_harness();
        let mut document = Document::parse(&"<details></details>".repeat(70));
        runtime.execute("var count=0;document.querySelectorAll('details').forEach(function(d){d.addEventListener('toggle',function(){count++;});});",&mut document).unwrap();
        for node in document.query_selector_all("details") {
            document.set_attr(node, "open", "");
        }
        runtime.dispatch_details_toggles(&mut document).unwrap();
        assert_eq!(
            runtime.execute("count", &mut document).unwrap(),
            Value::Number(64.0)
        );
        assert!(document.has_pending_details_toggles());
        runtime.dispatch_details_toggles(&mut document).unwrap();
        assert_eq!(
            runtime.execute("count", &mut document).unwrap(),
            Value::Number(70.0)
        );
        assert!(!document.has_pending_details_toggles());

        let mut runtime = Runtime::new();
        let mut document = Document::parse("<details></details>");
        let node = document.query_selector("details").unwrap();
        document.set_attr(node, "open", "");
        let queued = document.peek_details_toggle().unwrap();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .dispatch_details_toggles(&mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(document.peek_details_toggle(), Some(queued));
        assert!(!document.details_toggle_running());
        document.remove_attr(node, "open");
        let coalesced = document.peek_details_toggle().unwrap();
        assert_eq!(
            (coalesced.event.old_open, coalesced.event.new_open),
            (false, false)
        );
        document.begin_details_toggle(coalesced.id).unwrap();
        assert!(!document.has_pending_details_toggles());
        assert!(document.finish_details_toggle(coalesced.id));
        assert_eq!(runtime.stack_units, 0);
        assert!(!document.details_toggle_running());

        let mut runtime = Runtime::new();
        let mut document = Document::parse("<details id=a></details><details id=b></details>");
        runtime.execute("var caught=false;document.getElementById('a').ontoggle=function(){try{while(true){}}catch(e){caught=true;}};",&mut document).unwrap();
        let a = document.query_selector("#a").unwrap();
        let b = document.query_selector("#b").unwrap();
        document.set_attr(a, "open", "");
        document.set_attr(b, "open", "");
        assert!(
            runtime
                .dispatch_details_toggles(&mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(document.peek_details_toggle().unwrap().event.node, b);
        assert_eq!(runtime.stack_units, 0);
        assert!(!runtime.events.last().unwrap().dispatching);
        assert_eq!(
            runtime.execute("caught", &mut document).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn disclosure_reentrant_trackers_cover_multiple_listeners_nested_events_and_groups() {
        let cases = [
            (
                "<details id=d></details>",
                r#"
                var d=document.getElementById('d'),count=0,trace='';
                d.addEventListener('toggle',function(e){count++;trace+=e.oldState+'>'+e.newState+';';});
                d.addEventListener('toggle',function(){if(count===1)d.open=false;});
                d.addEventListener('toggle',function(){if(count===1)d.open=true;});
                d.open=true;
            "#,
                "closed>open;closed>open;",
            ),
            (
                "<details id=d></details>",
                r#"
                var d=document.getElementById('d'),count=0,trace='';
                d.ontoggle=function(e){
                    trace+=(e.isTrusted?'N:':'S:')+e.oldState+'>'+e.newState+';';
                    if(e.isTrusted && count++===0)d.dispatchEvent(new ToggleEvent('toggle',{oldState:'old',newState:'new'}));
                    else if(!e.isTrusted)d.open=false;
                };
                d.open=true;
            "#,
                "N:closed>open;S:old>new;N:closed>closed;",
            ),
            (
                "<details id=a name=g></details><details id=b name=g></details>",
                r#"
                var a=document.getElementById('a'),b=document.getElementById('b'),trace='';
                document.addEventListener('toggle',function(e){trace+=e.target.id+':'+e.oldState+'>'+e.newState+';';},true);
                a.ontoggle=function(e){if(e.newState==='open')b.open=true;};
                a.open=true;
            "#,
                "a:closed>open;b:closed>open;a:closed>closed;",
            ),
        ];
        for (html, source, expected) in cases {
            let mut runtime = Runtime::new();
            let mut document = Document::parse(html);
            runtime.execute(source, &mut document).unwrap();
            runtime.dispatch_details_toggles(&mut document).unwrap();
            assert_eq!(
                runtime.execute("trace", &mut document).unwrap().to_string(),
                expected
            );
            assert!(!document.details_toggle_running());
            assert!(!document.has_pending_details_toggles());
        }
    }

    #[test]
    fn disclosure_untracked_task_uses_newer_tracker_across_checkpoint_boundaries() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse(&format!(
            "{}<details id=d></details>",
            "<details></details>".repeat(63)
        ));
        runtime
            .execute(
                r#"
            var d=document.getElementById('d'),count=0,trace='';
            d.ontoggle=function(e){
                count++;trace+=e.oldState+'>'+e.newState+';';
                if(count===1){d.open=false;d.open=true;}
                else if(count===2){d.open=true;}
            };
        "#,
                &mut document,
            )
            .unwrap();
        for node in document.query_selector_all("details") {
            document.set_attr(node, "open", "");
        }
        runtime.dispatch_details_toggles(&mut document).unwrap();
        assert_eq!(
            runtime.execute("trace", &mut document).unwrap().to_string(),
            "closed>open;"
        );
        let untracked = document.peek_details_toggle().unwrap();
        assert_eq!(
            (untracked.event.old_open, untracked.event.new_open),
            (false, true)
        );
        runtime.execute("d.open=false;", &mut document).unwrap();
        assert_eq!(document.peek_details_toggle(), Some(untracked));
        runtime.dispatch_details_toggles(&mut document).unwrap();
        assert_eq!(
            runtime
                .execute("trace+'|'+d.open", &mut document)
                .unwrap()
                .to_string(),
            "closed>open;closed>open;open>open;|true"
        );
        assert!(!document.details_toggle_running());
        assert!(!document.has_pending_details_toggles());
    }

    #[test]
    fn disclosure_task_cleanup_preserves_reentrant_queue_after_resource_or_listener_errors() {
        for failure in ["throw new Error('reported');", "while(true){}"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("<details id=a></details><details id=b></details>");
            runtime.execute(&format!(r#"
                var a=document.getElementById('a'),b=document.getElementById('b'),count=0,trace='',caught=false;
                a.ontoggle=function(e){{
                    count++;trace+=e.oldState+'>'+e.newState+';';
                    if(count===1){{a.open=false;try{{{failure}}}catch(e){{caught=true;throw e;}}}}
                }};
                a.open=true;b.open=true;
            "#), &mut document).unwrap();
            let result = runtime.dispatch_details_toggles(&mut document);
            assert!(!document.details_toggle_running());
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            if failure.starts_with("while") {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(
                    document.peek_details_toggle().unwrap().event.node,
                    document.query_selector("#b").unwrap()
                );
                assert_eq!(
                    runtime.execute("caught", &mut document).unwrap(),
                    Value::Bool(false)
                );
                // Cleared tracker must not cancel the already queued closed>closed task.
                runtime.execute("a.open=true;", &mut document).unwrap();
                runtime.dispatch_details_toggles(&mut document).unwrap();
                assert_eq!(
                    runtime.execute("trace", &mut document).unwrap().to_string(),
                    "closed>open;closed>closed;closed>open;"
                );
            } else {
                result.unwrap();
                assert_eq!(
                    runtime.execute("trace", &mut document).unwrap().to_string(),
                    "closed>open;closed>closed;"
                );
                assert_eq!(
                    runtime.execute("caught", &mut document).unwrap(),
                    Value::Bool(true)
                );
            }
            assert!(!document.details_toggle_running());
            assert!(!document.has_pending_details_toggles());
        }
    }

    #[test]
    fn disclosure_nested_checkpoint_rejection_does_not_reset_work_or_clear_active_task() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<details id=a></details><details id=b></details>");
        for node in document.query_selector_all("details") {
            document.set_attr(node, "open", "");
        }
        let active = document.peek_details_toggle().unwrap();
        document.begin_details_toggle(active.id).unwrap();
        let pending = document.peek_details_toggle();
        runtime.steps = 7;
        let allocated = runtime.allocated;
        let error = runtime.dispatch_details_toggles(&mut document).unwrap_err();
        assert_eq!(error.intrinsic_error_name(), Some("TypeError"));
        assert_eq!(runtime.steps, 7);
        assert_eq!(runtime.allocated, allocated);
        assert!(document.details_toggle_running());
        assert_eq!(document.peek_details_toggle(), pending);
        assert!(document.finish_details_toggle(active.id));
        runtime.dispatch_details_toggles(&mut document).unwrap();
        assert!(!document.has_pending_details_toggles());
    }

    #[test]
    fn disclosure_mutation_and_constructor_coercion_preflight_uncatchable_limits() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<details></details>");
        let node = document.query_selector("details").unwrap();
        runtime.steps = 0;
        assert!(
            runtime
                .set(Value::Node(node), "open", Value::Bool(true), &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(document.attr(node, "open").is_none());
        assert!(!document.has_pending_details_toggles());
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .set(Value::Node(node), "open", Value::Bool(true), &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(document.attr(node, "open").is_none());
        assert!(!document.has_pending_details_toggles());
        for source in [
            "try{new ToggleEvent('x',{newState:{toString(){while(true){}}}});}catch(e){caught=true;}",
            "var value={toString(){return new ToggleEvent('x',{newState:value}).newState;}};try{new ToggleEvent('x',{newState:value});}catch(e){caught=true;}",
            "try{document.querySelector('details').name={toString(){while(true){}}};}catch(e){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("<details></details>");
            runtime.execute("var caught=false", &mut document).unwrap();
            assert!(
                runtime
                    .execute(source, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(
                runtime.execute("caught", &mut document).unwrap(),
                Value::Bool(false)
            );
        }
    }

    #[test]
    fn disclosure_copy_group_work_is_charged_before_allocating_or_replacing_the_target() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<main><p>preserved</p></main>");
        let target = document.query_selector("main").unwrap();
        let before = document.nodes.len();
        let markup = "<details name=g open></details>".repeat(500);
        assert!(
            runtime
                .set_inner_html(target, &markup, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(document.nodes.len(), before);
        assert_eq!(document.text_content(target), "preserved");
        assert!(!document.has_pending_details_toggles());

        let mut runtime = Runtime::new();
        let mut document = Document::parse(&format!("<main>{markup}</main>"));
        while let Some(task) = document.peek_details_toggle() {
            document.begin_details_toggle(task.id).unwrap();
            assert!(document.finish_details_toggle(task.id));
        }
        let source = document.query_selector("main").unwrap();
        let before = document.nodes.len();
        assert!(
            runtime
                .clone_dom_node(source, true, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(document.nodes.len(), before);
        assert!(!document.has_pending_details_toggles());
    }

    fn number_static_modes(source: &str) {
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            let result = if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            };
            result.unwrap_or_else(|error| panic!("strict={strict}: {error}"));
        }
    }

    #[test]
    fn scope_walk_preserves_declaration_order_and_function_boundaries() {
        let program = Parser::program(
            r#"
            var first;
            if(false){var yes;}else{var no;}
            mark:while(false){var loopName;}
            do{var doName;}while(false);
            for(var init;false;){var forName;}
            for(var key in {}){var inName;}
            switch(0){case 0:var caseName;break;default:var defaultName;}
            try{var tried;}catch(error){var caught;}finally{var finalized;}
            function hidden(){var secret;}
            var expression=function(){var secretToo;};
            "#,
        )
        .unwrap();
        let mut walk = StatementWalk::new(program.body.iter());
        let mut names = Vec::new();
        while let Some(statement) = walk.next(|| Ok(())).unwrap() {
            match statement {
                Stmt::Var(bindings, DeclarationKind::Var) => {
                    names.extend(bindings.iter().map(|(name, _)| name.as_str()))
                }
                Stmt::ForIn(ForBinding::Declaration(name, DeclarationKind::Var), _, _)
                | Stmt::ForOf(ForBinding::Declaration(name, DeclarationKind::Var), _, _) => {
                    names.push(name)
                }
                _ => {}
            }
        }
        assert_eq!(
            names,
            [
                "first",
                "yes",
                "no",
                "loopName",
                "doName",
                "init",
                "forName",
                "key",
                "inName",
                "caseName",
                "defaultName",
                "tried",
                "caught",
                "finalized",
                "expression",
            ]
        );
        let mut runtime = Runtime::new();
        let unit = code::test_unit(&program.body);
        runtime.hoist_vars(&unit, &unit.body, 1).unwrap();
        for name in names {
            assert_eq!(runtime.lookup(1, name).unwrap().1, Value::Undefined);
        }
        for name in ["secret", "secretToo", "error", "hidden"] {
            assert!(runtime.lookup(1, name).is_none(), "{name}");
        }
    }

    #[test]
    fn scope_walk_rejects_depth_and_work_before_advancing_and_stays_terminal() {
        let mut nested = Stmt::Empty;
        for _ in 0..MAX_DEPTH {
            nested = Stmt::Block(vec![nested]);
        }
        let mut walk = StatementWalk::new(std::iter::once(&nested));
        let mut count = 0;
        while walk.next(|| Ok(())).unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, MAX_DEPTH + 1);
        let excessive = Stmt::Block(vec![nested]);
        let mut walk = StatementWalk::new(std::iter::once(&excessive));
        let error = loop {
            match walk.next(|| Ok(())) {
                Ok(Some(_)) => {}
                Ok(None) => panic!("missing traversal depth stop"),
                Err(error) => break error,
            }
        };
        assert!(error.is_resource_limit());
        assert!(
            walk.next(|| panic!("terminal traversal did work"))
                .unwrap()
                .is_none()
        );

        let cases = Stmt::Switch(
            Expr::Literal(Value::Number(0.0)),
            (0..1024).map(|_| (None, Vec::new())).collect(),
        );
        let mut walk = StatementWalk::new(std::iter::once(&cases));
        assert!(walk.next(|| Ok(())).unwrap().is_some());
        let mut work = 0;
        assert!(
            walk.next(|| {
                work += 1;
                Ok(())
            })
            .unwrap()
            .is_none()
        );
        assert!(work >= 1024, "empty case lists must consume work");
        let mut walk = StatementWalk::new(std::iter::once(&cases));
        assert!(
            walk.next(|| Err(ScriptError::resource("test budget")))
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(walk.depth, 0, "failed precharge must not push children");
        assert!(
            walk.next(|| panic!("terminal traversal did work"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn scope_walk_width_uses_one_cursor_and_hoisting_shares_runtime_work() {
        let wide = Stmt::Block(vec![Stmt::Empty; 16_000]);
        let mut walk = StatementWalk::new(std::iter::once(&wide));
        let mut count = 0;
        while walk.next(|| Ok(())).unwrap().is_some() {
            assert!(walk.depth <= 1);
            count += 1;
        }
        assert_eq!(count, 16_001);
        let body = [Stmt::Var(
            vec![("unreached".into(), None)],
            DeclarationKind::Var,
        )];
        let unit = code::test_unit(&body);
        let mut runtime = Runtime::new();
        runtime.steps = 0;
        assert!(
            runtime
                .hoist_vars(&unit, &unit.body, 1)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.lookup(1, "unreached").is_none());
        runtime.steps = MAX_STEPS;
        runtime.stack_units = MAX_STACK_UNITS;
        runtime.hoist_vars(&unit, &unit.body, 1).unwrap();
        assert_eq!(runtime.stack_units, MAX_STACK_UNITS);
        assert_eq!(runtime.lookup(1, "unreached").unwrap().1, Value::Undefined);
    }

    #[test]
    fn scope_walk_keeps_nested_lexical_conflicts_and_scope_boundaries() {
        for strict in [false, true] {
            for source in [
                "let x;{var x;}",
                "let x;label:{var x;}",
                "let x;if(false){var x;}",
                "let x;while(false){var x;}",
                "let x;do{var x;}while(false);",
                "let x;for(var x;false;){}",
                "let x;for(var x in {}){}",
                "let x;try{var x;}finally{}",
                "let x;try{}catch(e){var x;}",
                "let x;try{}finally{var x;}",
                "for(let x;;){label:{var x;}}",
                "for(let x in {}){if(false){var x;}}",
                "switch(0){case 0:let x;break;case 1:var x;}",
                "switch(0){case 0:let x;break;default:let x;}",
                "switch(0){case 0:function x(){}break;default:let x;}",
                "function f(){let x;function x(){}}",
                "var f=()=>{let x;{var x;}};",
            ] {
                let error = Parser::program_context(source, false, strict).unwrap_err();
                assert!(error.is_parse_error(), "strict={strict}: {source}: {error}");
            }
            for source in [
                "let x;function f(){var x;}",
                "let x;var f=function(){var x;};",
                "let x;var f=()=>{var x;};",
                "let x;{let x;}",
                "switch(0){case 0:{let x;}break;default:{let x;}}",
                "for(let x;;){function f(){var x;}break;}",
                "for(let x in {}){var f=function(){var x;};}",
                "function f(){var x;function x(){}}",
            ] {
                Parser::program_context(source, false, strict)
                    .unwrap_or_else(|error| panic!("strict={strict}: {source}: {error}"));
            }
        }
        let error = Parser::program("let z;let a;let z;let a;").unwrap_err();
        assert_eq!(error.message, "duplicate lexical binding 'z'");
        let error = Parser::program("let z;let a;{var z;var a;}").unwrap_err();
        assert_eq!(
            error.message,
            "lexical and var declarations conflict for 'a'"
        );
    }

    fn labels_modes(source: &str) {
        for strict in [false, true] {
            let (mut runtime, mut doc) = upstream_harness();
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            result.unwrap_or_else(|error| panic!("strict={strict}: {error}"));
        }
    }

    #[test]
    fn labels_blocks_chains_and_nested_jumps_stop_at_the_right_target() {
        labels_modes(
            r#"
            var trace='';outer:{trace+='A';inner:{trace+='B';break outer;}trace+='X';}trace+='C';
            assert.sameValue(trace,'ABC');
            trace='';outer:alias:{trace+='A';inner:{trace+='B';break inner;}trace+='C';break alias;trace+='X';}
            assert.sameValue(trace,'ABC');
            var n=0;outer:while(n<3){n++;inner:while(true){trace+=n;continue outer;}trace+='X';}
            assert.sameValue(trace,'ABC123');
            trace='';outer:alias:for(var i=0;i<3;i++){for(var j=0;j<2;j++){trace+=i;continue alias;}trace+='X';}
            assert.sameValue(trace,'012');assert.sameValue(i,3);
            outer:{break outer;}outer:{break outer;}
            var same=2;same:{same++;break same;}assert.sameValue(same,3);
        "#,
        );
    }

    #[test]
    fn labels_loop_kinds_switches_and_iteration_closures_preserve_order() {
        labels_modes(
            r#"
            var n=0,checks=0;outer:do{n++;if(n<3)continue outer;break outer;}while(++checks<5);
            assert.sameValue(n,3);assert.sameValue(checks,2);
            var trace='';outer:for(var key in {a:1,b:2,c:3}){for(var j=0;j<2;j++){trace+=key;continue outer;}}
            assert.sameValue(trace,'abc');
            trace='';outer:for(var i=0;i<3;i++){switch(i){case 0:trace+='A';continue outer;case 1:trace+='B';break;default:break outer;}trace+='C';}
            assert.sameValue(trace,'ABC');
            trace='';label:switch(1){case 1:trace+='A';while(true){break label;}default:trace+='X';}assert.sameValue(trace,'A');
            var fs=[];outer:for(let i=0;i<3;i++){fs.push(function(){return i;});inner:while(true){continue outer;}}
            assert.sameValue(fs[0](),0);assert.sameValue(fs[1](),1);assert.sameValue(fs[2](),2);
            trace='';outer:for(var x in {a:1,b:2}){for(var y in {z:1}){trace+=x;break outer;}}assert.sameValue(trace,'a');
        "#,
        );
    }

    #[test]
    fn labels_finalizers_preserve_or_replace_break_continue_return_and_throw() {
        labels_modes(
            r#"
            var trace='';outer:for(var i=0;i<3;i++){try{trace+=i;continue outer;}finally{trace+='F';}}
            assert.sameValue(trace,'0F1F2F');
            trace='';outer:for(var i=0;i<3;i++){try{trace+='A';continue outer;}finally{trace+='F';break outer;}}
            assert.sameValue(trace,'AF');
            trace='';outer:for(var i=0;i<2;i++){try{trace+='A';break outer;}finally{trace+='F';continue outer;}}
            assert.sameValue(trace,'AFAF');
            function f(){outer:{try{break outer;}finally{return 7;}}return 9;}assert.sameValue(f(),7);
            function g(){outer:{try{return 7;}finally{break outer;}}return 9;}assert.sameValue(g(),9);
            var reason={},seen;try{outer:{try{break outer;}finally{throw reason;}}}catch(e){seen=e;}assert.sameValue(seen,reason);
            trace='';outer:{try{inner:{try{break outer;}finally{trace+='I';}}}finally{trace+='O';}}assert.sameValue(trace,'IO');
            trace='';outer:{try{throw reason;}finally{trace+='F';break outer;}}assert.sameValue(trace,'F');
        "#,
        );
    }

    #[test]
    fn labels_var_hoists_and_nested_function_scopes_remain_separate() {
        labels_modes(
            r#"
            assert.sameValue(x,undefined);outer:{break outer;var x=7;}assert.sameValue(x,undefined);
            var n=0;outer:var y=++n;assert.sameValue(y,1);
            function f(){outer:{break outer;}return 2;}
            outer:{n=f();break outer;}assert.sameValue(n,2);
            outer:{var fn=function(){outer:{break outer;}return 3;};n=fn();break outer;}assert.sameValue(n,3);
            outer:{var arrow=()=>{outer:{break outer;}return 4;};n=arrow();break outer;}assert.sameValue(n,4);
            outer:{var object={f(){outer:{break outer;}return 5;}};n=object.f();break outer;}assert.sameValue(n,5);
            let shadow=1;outer:{let shadow=2;assert.sameValue(shadow,2);break outer;}assert.sameValue(shadow,1);
        "#,
        );
        for source in [
            "let x;outer:var x;",
            "{let x;outer:{var x;}}",
            "outer:let x=1;",
            "outer:const x=1;",
        ] {
            identifier_syntax(source, false);
            identifier_syntax(source, true);
        }
    }

    #[test]
    fn labels_early_errors_reject_unknown_duplicate_nonloop_and_cross_function_targets() {
        for source in [
            "break missing;",
            "while(true){break missing;}",
            "while(true){continue missing;}",
            "a:a:;",
            "a:{a:;}",
            "a:{while(true){continue a;}}",
            "a:{continue a;}",
            "while(true){a:switch(0){default:continue a;}}",
            "a:if(true)while(true){continue a;}",
            "a:{break;}",
            "a:{continue;}",
            "a:{function f(){break a;}}",
            "a:while(false){function f(){continue a;}}",
            "a:{var f=()=>{break a;};}",
            "a:while(false){var f=()=>{continue a;};}",
            "a:{var x={f(){break a;}};}",
            "a:{function f(x=function(){break a;}){}}",
            "a:{var f=(x=function(){break a;})=>1;}",
            "L:let\n[a]=0;",
            "L:class C{}",
            "if(false)class C{}",
            "a:{break a;}break a;",
            "a:while(false){}continue a;",
            r"a:\u0061:;",
            "if:;",
            r"\u0069f:;",
        ] {
            identifier_syntax(source, false);
            identifier_syntax(source, true);
        }
        for source in [
            "label:function f(){}",
            "a:b:function f(){}",
            "let:;",
            "yield:;",
        ] {
            identifier_syntax(source, true);
        }
        assert!(
            Parser::program("label:function f(){}")
                .unwrap_err()
                .is_unsupported()
        );
    }

    #[test]
    fn labels_unicode_reserved_contexts_and_asi_preserve_identifier_rules() {
        labels_modes(
            r#"
            var n=0;\u0061:{n++;break a;n++;}a:{n++;break \u0061;n++;}π:{n++;break π;n++;}
            assert.sameValue(n,3);
            eval:{n++;break eval;}arguments:{n++;break arguments;}assert.sameValue(n,5);
            await:{break await;}async:{break async;}
        "#,
        );
        let (mut runtime, mut doc) = upstream_harness();
        runtime
            .execute(
                "if(false){L:let\nx=1;L:let\n{}}if(false)let\nx=1;",
                &mut doc,
            )
            .unwrap();
        runtime
            .execute("let:{break let;}yield:{break yield;}", &mut doc)
            .unwrap();
        for line in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}", "/*\n*/"] {
            labels_modes(&format!(
                "var n=0;outer:while(true){{n++;break{line}missing;}}assert.sameValue(n,1);"
            ));
            labels_modes(&format!(
                "var n=0;outer:while(n<2){{n++;continue{line}missing;}}assert.sameValue(n,2);"
            ));
            identifier_syntax(&format!("outer:{{break{line}outer;}}"), false);
        }
    }

    #[test]
    fn labels_flat_chains_and_parser_work_storage_remain_bounded() {
        let chain = (0..MAX_DEPTH)
            .map(|i| format!("label{i}:"))
            .collect::<String>();
        labels_modes(&format!(
            "var n=0;{chain}{{n++;break label0;n++;}}assert.sameValue(n,1);"
        ));
        let source = format!("extra:{chain};");
        assert!(Parser::program(&source).unwrap_err().is_resource_limit());
        let deep = format!(
            "{};{}",
            (0..MAX_DEPTH)
                .map(|i| format!("l{i}:{{"))
                .collect::<String>(),
            "}".repeat(MAX_DEPTH)
        );
        assert!(Parser::program(&deep).unwrap_err().is_resource_limit());
        let mut budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let tokens = lex("label:;", &mut budget).unwrap();
        let mut parser = Parser {
            tokens,
            source: "label:;",
            lex_work: 7,
            compile_budget: budget,
            pos: 0,
            depth: 0,
            function_depth: 0,
            new_target_allowed: false,
            loop_depth: 0,
            switch_depth: 0,
            labels: Vec::new(),
            next_label: 0,
            allow_in: true,
            strict: false,
        };
        parser.compile_budget.allocated = 0;
        parser.compile_budget.heap_limit = 29;
        assert!(parser.labeled_statement().unwrap_err().is_resource_limit());
        assert!(parser.labels.is_empty());
        assert_eq!(parser.labels.capacity(), 0);
        parser.labels = (0..MAX_DEPTH)
            .map(|i| ActiveLabel {
                name: format!("{i:032}"),
                target: i,
                iteration: true,
            })
            .collect();
        parser.compile_budget.steps = MAX_DEPTH * 5;
        assert!(
            parser
                .label_target("00000000000000000000000000000000")
                .unwrap_err()
                .is_resource_limit()
        );
        parser.compile_budget.steps = MAX_DEPTH * 5 + 1;
        assert_eq!(
            parser
                .label_target("00000000000000000000000000000000")
                .unwrap(),
            Some((0, true))
        );
        assert_eq!(parser.compile_budget.steps, 0);
    }

    #[test]
    fn labels_statement_dispatch_rejects_deep_syntax_before_native_stack_exhaustion() {
        for (open, close) in [
            ("{", "}"),
            ("if(true)", ""),
            ("while(false)", ""),
            ("for(;;)", ""),
            ("do ", "while(false);"),
            ("try{", "}finally{}"),
            ("switch(0){default:", "}"),
            ("function f(){", "}"),
        ] {
            let source = format!("{};{}", open.repeat(MAX_DEPTH), close.repeat(MAX_DEPTH));
            let error = Parser::program(&source).unwrap_err();
            assert!(error.is_resource_limit(), "{open}: {error}");
        }
    }

    #[test]
    fn labels_infinite_control_flow_remains_an_uncatchable_host_stop() {
        for loop_source in [
            "outer:while(true){continue outer;}",
            "outer:for(;;){inner:while(true){continue outer;}}",
            "outer:do{continue outer;}while(true);",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime
                .execute("var caught=false,finalized=false;", &mut doc)
                .unwrap();
            let source =
                format!("try{{{loop_source}}}catch(e){{caught=true;}}finally{{finalized=true;}}");
            assert!(
                runtime
                    .execute(&source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.environments[0].bindings["caught"].value,
                Value::Bool(false)
            );
            assert_eq!(
                runtime.environments[0].bindings["finalized"].value,
                Value::Bool(false)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
    }

    fn equality_modes(source: &str) {
        for (op, equal, unequal) in [("==", "true", "false"), ("!=", "false", "true")] {
            for strict in [false, true] {
                let (mut runtime, mut doc) = upstream_harness();
                let source = source
                    .replace("@OP@", op)
                    .replace("@EQ@", equal)
                    .replace("@NE@", unequal);
                let result = if strict {
                    runtime.execute_strict(&source, &mut doc)
                } else {
                    runtime.execute(&source, &mut doc)
                };
                result.unwrap_or_else(|e| panic!("{op} strict={strict}: {e}"));
            }
        }
    }

    #[test]
    fn equality_ordinary_type_lattice_preserves_nullish_and_numeric_rules() {
        equality_modes(
            r#"
            assert.sameValue(null@OP@undefined,@EQ@);assert.sameValue(undefined@OP@null,@EQ@);
            assert.sameValue(false@OP@null,@NE@);assert.sameValue(null@OP@false,@NE@);
            assert.sameValue(0@OP@null,@NE@);assert.sameValue(''@OP@null,@NE@);
            assert.sameValue(undefined@OP@false,@NE@);assert.sameValue(false@OP@undefined,@NE@);
            assert.sameValue(false@OP@'',@EQ@);assert.sameValue('0'@OP@false,@EQ@);
            assert.sameValue(true@OP@'1',@EQ@);assert.sameValue(' 1 '@OP@1,@EQ@);
            assert.sameValue(16@OP@'0x10',@EQ@);assert.sameValue('0b10'@OP@2,@EQ@);
            assert.sameValue('0o10'@OP@8,@EQ@);assert.sameValue('no number'@OP@0,@NE@);
            assert.sameValue(NaN@OP@NaN,@NE@);assert.sameValue(NaN@OP@'NaN',@NE@);
            assert.sameValue(-0@OP@0,@EQ@);assert.sameValue(Infinity@OP@'Infinity',@EQ@);
            assert.sameValue('01'@OP@'1',@NE@);assert.sameValue('01'@OP@1,@EQ@);
            assert.sameValue(new String('1')@OP@1,@EQ@);assert.sameValue(1@OP@new String('1'),@EQ@);
            assert.sameValue(new Number(0)@OP@false,@EQ@);assert.sameValue(new Boolean(false)@OP@'0',@EQ@);
            assert.sameValue([]@OP@false,@EQ@);assert.sameValue([1]@OP@true,@EQ@);
            assert.sameValue({valueOf:function(){return false;}}@OP@0,@EQ@);
            assert.sameValue({valueOf:function(){return null;}}@OP@0,@NE@);
            assert.sameValue({valueOf:function(){return undefined;}}@OP@0,@NE@);
        "#,
        );
    }

    #[test]
    fn equality_live_conversion_uses_original_receiver_after_both_expressions() {
        equality_modes(
            r#"
            var trace='',object={get valueOf(){trace+='L';return function(){assert.sameValue(this,object);trace+='l';
                Object.defineProperty(object,'toString',{get:function(){trace+='T';return function(){assert.sameValue(this,object);trace+='t';return '1';};},configurable:true});return {};};}};
            function a(){trace+='A';return object;}function b(){trace+='B';return 1;}
            assert.sameValue(a()@OP@b(),@EQ@);assert.sameValue(trace,'ABLlTt');
            trace='';assert.sameValue(b()@OP@a(),@EQ@);assert.sameValue(trace,'BALlTt');
            object={valueOf:function(){throw 'stale';}};trace='';
            function update(){trace+='B';object.valueOf=function(){trace+='L';return 1;};return 1;}
            assert.sameValue(a()@OP@update(),@EQ@);assert.sameValue(trace,'ABL');
            var saved=object;function replace(){trace+='B';object={valueOf:function(){throw 'unused';}};return 1;}
            trace='';assert.sameValue(a()@OP@replace(),@EQ@);assert.sameValue(trace,'ABL');
            var n=0,value={valueOf:function(){n++;return 1;}};
            assert.sameValue(value@OP@1,@EQ@);assert.sameValue(n,1);
            trace='';object={valueOf:function(){trace+='L';return 1;}};
            function c(){trace+='C';return {valueOf:function(){trace+='V';return 1;}};}
            assert.sameValue(a()@OP@b()@OP@c(),true);assert.sameValue(trace,'ABLCV');
        "#,
        );
    }

    #[test]
    fn equality_abrupt_completion_and_failed_primitive_conversion_preserve_identity() {
        equality_modes(
            r#"
            var trace='',reason={},seen,object={get valueOf(){trace+='L';throw reason;}};
            function rhs(){trace+='B';return 1;}
            try{object@OP@rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'BL');
            trace='';seen=undefined;try{rhs()@OP@object;}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'BL');
            trace='';seen=undefined;try{object@OP@(function(){trace+='B';throw reason;})();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'B');
            object={valueOf:function(){return {};},get toString(){throw reason;}};
            seen=undefined;try{object@OP@false;}catch(e){seen=e;}assert.sameValue(seen,reason);
            assert.throws(TypeError,function(){return Object.create(null)@OP@1;});
            assert.throws(TypeError,function(){return 1@OP@{valueOf:function(){return {};},toString:function(){return {};}};});
            assert.sameValue({valueOf:null,toString:function(){return '1';}}@OP@true,@EQ@);
        "#,
        );
    }

    #[test]
    fn equality_identity_and_nullish_pairs_never_read_conversion_hooks() {
        equality_modes(
            r#"
            var object={get valueOf(){throw 'unused';},get toString(){throw 'unused';}},array=[],fn=function(){};
            assert.sameValue(object@OP@object,@EQ@);assert.sameValue(object@OP@{},@NE@);
            assert.sameValue(object@OP@null,@NE@);assert.sameValue(null@OP@object,@NE@);
            assert.sameValue(object@OP@undefined,@NE@);assert.sameValue(undefined@OP@object,@NE@);
            assert.sameValue(array@OP@array,@EQ@);assert.sameValue(array@OP@[],@NE@);
            assert.sameValue(fn@OP@fn,@EQ@);assert.sameValue(fn@OP@function(){},@NE@);
            assert.sameValue(new Number(1)@OP@new Number(1),@NE@);
            var nan=new Number(NaN);assert.sameValue(nan@OP@nan,@EQ@);assert.sameValue(nan@OP@NaN,@NE@);
        "#,
        );
    }

    #[test]
    fn equality_utf16_and_strict_operators_do_not_coerce_or_normalize() {
        equality_modes(
            r#"
            assert.sameValue(new String('\uD800')@OP@'\uD800',@EQ@);
            assert.sameValue('\uDC00'@OP@new String('\uDC00'),@EQ@);
            assert.sameValue(new String('\uD800')@OP@'\uDC00',@NE@);
            assert.sameValue(new String('\uD800\uDC00')@OP@'\uD800\uDC00',@EQ@);
            assert.sameValue(new String('\u00E9')@OP@'e\u0301',@NE@);
            assert.sameValue(new String('a\0b')@OP@'a\0b',@EQ@);
            assert.sameValue('\uD800'@OP@0,@NE@);
        "#,
        );
        for (op, equal, unequal) in [("===", "true", "false"), ("!==", "false", "true")] {
            for strict in [false, true] {
                let (mut runtime, mut doc) = upstream_harness();
                let source=r#"
                    var object={get valueOf(){throw 'unused';},get toString(){throw 'unused';}};
                    assert.sameValue(object@OP@1,@NE@);assert.sameValue(false@OP@object,@NE@);
                    assert.sameValue(null@OP@object,@NE@);assert.sameValue(object@OP@object,@EQ@);
                    assert.sameValue(1@OP@'1',@NE@);assert.sameValue(null@OP@undefined,@NE@);
                    assert.sameValue(new Number(1)@OP@1,@NE@);assert.sameValue(false@OP@null,@NE@);
                    assert.sameValue(NaN@OP@NaN,@NE@);assert.sameValue(-0@OP@0,@EQ@);
                    assert.sameValue('\uD800'@OP@'\uD800',@EQ@);assert.sameValue('\uD800'@OP@'\uDC00',@NE@);
                    var trace='';function a(){trace+='A';return object;}function b(){trace+='B';return 1;}
                    assert.sameValue(a()@OP@b(),@NE@);assert.sameValue(trace,'AB');
                "#.replace("@OP@",op).replace("@EQ@",equal).replace("@NE@",unequal);
                let result = if strict {
                    runtime.execute_strict(&source, &mut doc)
                } else {
                    runtime.execute(&source, &mut doc)
                };
                result.unwrap_or_else(|e| panic!("{op} strict={strict}: {e}"));
            }
        }
    }

    #[test]
    fn equality_charges_only_required_string_work_and_numeric_storage() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        let a = Value::String(JsString::from(vec![97; 32]));
        let b = a.clone();
        runtime.allocated = MAX_HEAP;
        runtime.steps = 9;
        assert!(
            runtime
                .loosely_equal(a.clone(), b.clone(), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        runtime.steps = 10;
        assert!(runtime.loosely_equal(a, b, &mut doc).unwrap());
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        let large = Value::String(JsString::from(vec![97; MAX_STRING]));
        assert!(
            !runtime
                .loosely_equal(large.clone(), Value::Null, &mut doc)
                .unwrap()
        );
        assert!(
            !runtime
                .loosely_equal(Value::Undefined, large, &mut doc)
                .unwrap()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        runtime.steps = 2;
        runtime.allocated = MAX_HEAP - 36;
        assert!(
            runtime
                .loosely_equal(Value::String("12".into()), Value::Number(12.0), &mut doc)
                .unwrap()
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        runtime.steps = 2;
        assert!(
            runtime
                .loosely_equal(Value::Number(12.0), Value::String("12".into()), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, MAX_HEAP + 36);
        runtime.steps = 1;
        assert!(
            runtime
                .loosely_equal(Value::Bool(false), Value::Number(0.0), &mut doc)
                .unwrap()
        );
        assert_eq!(runtime.steps, 0);
    }

    #[test]
    fn equality_conversion_precedes_large_string_limit_and_shares_callback_budget() {
        for reverse in [false, true] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime
                .execute(
                    "var seen=false;var object={valueOf:function(){seen=true;return 'x';}};",
                    &mut doc,
                )
                .unwrap();
            let object = runtime.environments[0].bindings["object"].value.clone();
            let text = Value::String(JsString::from(vec![97; MAX_STRING]));
            runtime.steps = 1024;
            let (left, right) = if reverse {
                (object, text)
            } else {
                (text, object)
            };
            assert!(
                runtime
                    .loosely_equal(left, right, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.environments[0].bindings["seen"].value,
                Value::Bool(true)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
        for op in ["==", "!="] {
            for body in ["while(true){}".to_owned(), format!("return object{op}1;")] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                runtime
                    .execute("var caught=false,finished=false;", &mut doc)
                    .unwrap();
                let source = format!(
                    "var object={{valueOf:function(){{{body}}}}};try{{object{op}1;finished=true;}}catch(e){{caught=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!(
                    runtime.environments[0].bindings["finished"].value,
                    Value::Bool(false)
                );
                assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            }
        }
    }

    fn relational_modes(source: &str) {
        for (op, lexical, numeric, inclusive) in [
            ("<", "false", "true", "false"),
            (">", "true", "false", "false"),
            ("<=", "false", "true", "true"),
            (">=", "true", "false", "true"),
        ] {
            for strict in [false, true] {
                let (mut runtime, mut doc) = upstream_harness();
                let source = source
                    .replace("@OP@", op)
                    .replace("@LEX@", lexical)
                    .replace("@NUM@", numeric)
                    .replace("@EQ@", inclusive);
                let result = if strict {
                    runtime.execute_strict(&source, &mut doc)
                } else {
                    runtime.execute(&source, &mut doc)
                };
                result.unwrap_or_else(|e| panic!("{op} strict={strict}: {e}"));
            }
        }
    }

    #[test]
    fn relational_selects_string_or_numeric_order_after_both_conversions() {
        relational_modes(
            r#"
            assert.sameValue(new String('2')@OP@new String('10'),@LEX@);
            assert.sameValue({valueOf:function(){return '2';}}@OP@'10',@LEX@);
            assert.sameValue('2'@OP@{valueOf:function(){return '10';}},@LEX@);
            assert.sameValue(new Number(2)@OP@new String('10'),@NUM@);
            assert.sameValue(new String('2')@OP@new Number(10),@NUM@);
            assert.sameValue([2]@OP@[10],@LEX@);assert.sameValue([2]@OP@10,@NUM@);
            assert.sameValue({valueOf:null,toString:function(){return '2';}}@OP@'10',@LEX@);
            var calls=0,object={valueOf:function(){calls++;return calls===1?'2':'10';}};
            assert.sameValue(object@OP@object,@LEX@);assert.sameValue(calls,2);
        "#,
        );
    }

    #[test]
    fn relational_preserves_source_order_live_fallbacks_and_captured_values() {
        relational_modes(
            r#"
            var trace='',right={valueOf:function(){throw 'stale';}},left={get valueOf(){trace+='L';return function(){
                assert.sameValue(this,left);trace+='l';right.valueOf=function(){trace+='R';return '10';};
                Object.defineProperty(left,'toString',{get:function(){trace+='T';return function(){assert.sameValue(this,left);trace+='t';return '2';};},configurable:true});return {};};},
                toString:function(){throw 'stale';}};
            function a(){trace+='A';return left;}function b(){trace+='B';return right;}
            assert.sameValue(a()@OP@b(),@LEX@);assert.sameValue(trace,'ABLlTtR');
            trace='';left={valueOf:function(){trace+='L';return '2';}};right={valueOf:function(){trace+='R';return '10';}};
            function c(){trace+='C';return {valueOf:function(){trace+='V';return 3;}};}
            assert.sameValue(a()@OP@b()@OP@c(),@NUM@);assert.sameValue(trace,'ABLRCV');
            var original={valueOf:function(){trace+='L';return '2';}},selected=original;
            function first(){trace+='A';return selected;}function second(){trace+='B';selected={valueOf:function(){throw 'unused';}};return '10';}
            trace='';assert.sameValue(first()@OP@second(),@LEX@);assert.sameValue(trace,'ABL');
        "#,
        );
    }

    #[test]
    fn relational_abrupt_completion_stops_later_hooks_and_preserves_identity() {
        relational_modes(
            r#"
            var trace='',reason={},seen,left={get valueOf(){trace+='L';throw reason;}},right={get valueOf(){trace+='R';throw 'unused';}};
            function rhs(){trace+='B';return right;}
            try{left@OP@rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'BL');
            left={valueOf:function(){trace+='L';return '2';}};right={get valueOf(){trace+='R';throw reason;}};
            trace='';seen=undefined;try{left@OP@right;}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'LR');
            trace='';seen=undefined;try{left@OP@(function(){trace+='B';throw reason;})();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'B');
            assert.throws(TypeError,function(){return Object.create(null)@OP@1;});
            assert.throws(TypeError,function(){return {valueOf:function(){return {};},toString:function(){return {};}}@OP@1;});
        "#,
        );
    }

    #[test]
    fn relational_utf16_prefixes_nan_and_numeric_boundaries() {
        relational_modes(
            r#"
            assert.sameValue(new String('\uD800\uDC00')@OP@'\uE000',@NUM@);
            assert.sameValue('\uD800'@OP@new String('\uDC00'),@NUM@);
            assert.sameValue(new String('a')@OP@'aa',@NUM@);assert.sameValue(''@OP@new String('a'),@NUM@);
            assert.sameValue(new String('same')@OP@'same',@EQ@);
            assert.sameValue(new String('10')@OP@'2',@NUM@);assert.sameValue(new String(' 2 ')@OP@10,@NUM@);
            assert.sameValue(new String('0x10')@OP@17,@NUM@);
            assert.sameValue(null@OP@1,@NUM@);assert.sameValue(false@OP@true,@NUM@);
            assert.sameValue(-Infinity@OP@Infinity,@NUM@);assert.sameValue(-0@OP@0,@EQ@);
            assert.sameValue(Infinity@OP@Infinity,@EQ@);assert.sameValue(-Infinity@OP@-Infinity,@EQ@);
            assert.sameValue(NaN@OP@1,false);assert.sameValue(1@OP@NaN,false);
            assert.sameValue(undefined@OP@0,false);assert.sameValue(0@OP@undefined,false);
            assert.sameValue(new String('no number')@OP@1,false);assert.sameValue(1@OP@new String('no number'),false);
            assert.sameValue('\uD800'@OP@1,false);assert.sameValue(1@OP@'\uD800',false);
        "#,
        );
    }

    #[test]
    fn relational_string_work_and_numeric_storage_are_charged_before_use() {
        for (op, expected) in [("<", true), (">", false), ("<=", true), (">=", false)] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            let a = Value::String(JsString::from(vec![97; 32]));
            let b = Value::String(JsString::from(vec![98; 33]));
            runtime.allocated = MAX_HEAP;
            runtime.steps = 9;
            assert!(
                runtime
                    .relational_value(op, a.clone(), b.clone(), &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.allocated, MAX_HEAP);
            runtime.steps = 10;
            assert_eq!(
                runtime.relational_value(op, a, b, &mut doc).unwrap(),
                Value::Bool(expected)
            );
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            runtime.steps = MAX_STEPS;
            runtime.allocated = MAX_HEAP - 36;
            assert_eq!(
                runtime
                    .relational_value(
                        op,
                        Value::String("12".into()),
                        Value::Number(13.0),
                        &mut doc
                    )
                    .unwrap(),
                Value::Bool(expected)
            );
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert!(
                runtime
                    .relational_value(
                        op,
                        Value::String("12".into()),
                        Value::Number(13.0),
                        &mut doc
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.allocated, MAX_HEAP + 36);
        }
    }

    #[test]
    fn relational_both_hooks_precede_large_string_resource_checks() {
        for op in ["<", ">", "<=", ">="] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var text='',seen=false;var right={valueOf:function(){seen=true;return text;}};",&mut doc).unwrap();
            let text = Value::String(JsString::from(vec![97; MAX_STRING]));
            runtime.environments[0]
                .bindings
                .get_mut("text")
                .unwrap()
                .value = text.clone();
            let right = runtime.environments[0].bindings["right"].value.clone();
            runtime.steps = 1024;
            assert!(
                runtime
                    .relational_value(op, text, right, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.environments[0].bindings["seen"].value,
                Value::Bool(true)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
    }

    #[test]
    fn relational_recursive_and_looping_conversion_remains_uncatchable() {
        for op in ["<", ">", "<=", ">="] {
            for body in ["while(true){}".to_owned(), format!("return object{op}1;")] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                runtime
                    .execute("var caught=false,finished=false;", &mut doc)
                    .unwrap();
                let source = format!(
                    "var object={{valueOf:function(){{{body}}}}};try{{object{op}1;finished=true;}}catch(e){{caught=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!(
                    runtime.environments[0].bindings["finished"].value,
                    Value::Bool(false)
                );
                assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            }
        }
    }

    fn uri_modes(source: &str) {
        for name in [
            "encodeURI",
            "encodeURIComponent",
            "decodeURI",
            "decodeURIComponent",
        ] {
            let encoding = name.starts_with("encode");
            for strict in [false, true] {
                let (mut runtime, mut doc) = upstream_harness();
                let source = source
                    .replace("@FN@", name)
                    .replace("@INPUT@", if encoding { "'a b'" } else { "'%61%20%62'" })
                    .replace("@OUTPUT@", if encoding { "'a%20b'" } else { "'a b'" })
                    .replace("@BAD@", if encoding { "'\\uD800'" } else { "'%ED%A0%80'" });
                let result = if strict {
                    runtime.execute_strict(&source, &mut doc)
                } else {
                    runtime.execute(&source, &mut doc)
                };
                result.unwrap_or_else(|e| panic!("{name} strict={strict}: {e}"));
            }
        }
    }

    #[test]
    fn uri_globals_preserve_live_string_hint_hooks_and_argument_order() {
        uri_modes(
            r#"
            var trace='',object={get toString(){trace+='T';return function(){assert.sameValue(this,object);trace+='t';
                object.valueOf=function(){assert.sameValue(this,object);trace+='v';return @INPUT@;};return {};};},
                valueOf:function(){throw 'stale';}};
            function first(){trace+='A';return object;}function extra(){trace+='B';return {toString:function(){throw 'unused';}};}
            assert.sameValue(@FN@(first(),extra()),@OUTPUT@);assert.sameValue(trace,'ABTtv');
            object={toString:null,valueOf:function(){return @INPUT@;}};assert.sameValue(@FN@(object),@OUTPUT@);
            object={toString:function(){return @INPUT@;},get valueOf(){throw 'unused';}};assert.sameValue(@FN@(object),@OUTPUT@);
            assert.sameValue(@FN@.call({toString:function(){throw 'unused';}},@INPUT@),@OUTPUT@);
            assert.sameValue(@FN@(),'undefined');assert.sameValue(@FN@(undefined),'undefined');assert.sameValue(@FN@(null),'null');
            assert.sameValue(@FN@(true),'true');assert.sameValue(@FN@(-0),'0');assert.sameValue(@FN@(new Number(12)),'12');
        "#,
        );
    }

    #[test]
    fn uri_globals_preserve_abrupt_identity_and_canonical_uri_errors() {
        uri_modes(
            r#"
            var reason={},seen,called=false,object={get toString(){throw reason;},get valueOf(){called=true;throw 'unused';}};
            try{@FN@(object);}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(called,false);
            seen=undefined;object={toString:function(){return {};},get valueOf(){throw reason;}};
            try{@FN@(object);}catch(e){seen=e;}assert.sameValue(seen,reason);
            assert.throws(TypeError,function(){@FN@({toString:function(){return {};},valueOf:function(){return {};}});});
            var actual=URIError;URIError=function replacement(){};
            assert.throws(actual,function(){@FN@(@BAD@);});
            try{@FN@(@BAD@);}catch(e){assert.sameValue(e.constructor,actual);assert.sameValue(e instanceof actual,true);}
        "#,
        );
    }

    #[test]
    fn uri_globals_metadata_aliases_and_nonconstructability() {
        uri_modes(
            r#"
            var saved=@FN@;assert.sameValue(saved.name,'@FN@');assert.sameValue(saved.length,1);
            assert.sameValue(Object.getPrototypeOf(saved),Function.prototype);assert.sameValue(saved.hasOwnProperty('prototype'),false);
            var desc=Object.getOwnPropertyDescriptor(saved,'name');assert.sameValue(desc.value,'@FN@');
            assert.sameValue(desc.writable,false);assert.sameValue(desc.enumerable,false);assert.sameValue(desc.configurable,true);
            desc=Object.getOwnPropertyDescriptor(saved,'length');assert.sameValue(desc.value,1);
            assert.sameValue(desc.writable,false);assert.sameValue(desc.enumerable,false);assert.sameValue(desc.configurable,true);
            desc=Object.getOwnPropertyDescriptor(globalThis,'@FN@');assert.sameValue(desc.value,saved);
            assert.sameValue(desc.writable,true);assert.sameValue(desc.enumerable,false);assert.sameValue(desc.configurable,true);
            assert.throws(TypeError,function(){new saved();});var bound=saved.bind(null,@INPUT@);
            assert.throws(TypeError,function(){new bound();});assert.sameValue(bound(),@OUTPUT@);
            Object.defineProperty(saved,'name',{value:'changed'});@FN@=function(){throw 'replacement';};
            assert.sameValue(saved(@INPUT@),@OUTPUT@);assert.sameValue(delete globalThis['@FN@'],true);
            globalThis['@FN@']=saved;assert.sameValue(@FN@,saved);
        "#,
        );
    }

    #[test]
    fn uri_globals_reserved_characters_utf16_and_strict_percent_sequences() {
        number_static_modes(
            r#"
            var reserved=';/?:@&=+$,#',escaped='%3B%2F%3F%3A%40%26%3D%2B%24%2C%23';
            assert.sameValue(encodeURI(reserved),reserved);assert.sameValue(encodeURIComponent(reserved),escaped);
            assert.sameValue(decodeURI(escaped),escaped);assert.sameValue(decodeURIComponent(escaped),reserved);
            assert.sameValue(decodeURI('%3b%2f%3f%3a%40%26%3d%2b%24%2c%23'),'%3b%2f%3f%3a%40%26%3d%2b%24%2c%23');
            var plain="AZaz09-_.!~*'()";assert.sameValue(encodeURI(plain),plain);assert.sameValue(encodeURIComponent(plain),plain);
            assert.sameValue(encodeURI('[] %'),'%5B%5D%20%25');
            assert.sameValue(decodeURIComponent('+%2B%2520'), '++%20');assert.sameValue(decodeURI('%41%00%7f'),'A\u0000\u007f');
            var unicode='\u0000\u0080\u07FF\u0800\uD7FF\uE000\uFFFF\uD800\uDC00\uDBFF\uDFFF';
            var encoded='%00%C2%80%DF%BF%E0%A0%80%ED%9F%BF%EE%80%80%EF%BF%BF%F0%90%80%80%F4%8F%BF%BF';
            assert.sameValue(encodeURI(unicode),encoded);assert.sameValue(encodeURIComponent(unicode),encoded);
            assert.sameValue(decodeURI(encoded),unicode);assert.sameValue(decodeURIComponent(encoded),unicode);
            assert.sameValue(decodeURI('\uD800%41\uDC00'),'\uD800A\uDC00');
            assert.sameValue(decodeURIComponent('\uDC00\uD800'),'\uDC00\uD800');
            var bad=['%','%0','%GG','%u0041','%80','%C0%80','%C1%BF','%C2','%C2%20','%E0%80%80',
                '%ED%A0%80','%ED%BF%BF','%F0%80%80%80','%F4%90%80%80','%F5%80%80%80','%FF','%E2%82%AC%80'];
            for(var i=0;i<bad.length;i++){
                assert.throws(URIError,function(){decodeURI(bad[i]);});assert.throws(URIError,function(){decodeURIComponent(bad[i]);});
            }
            bad=['\uD800','\uDC00','\uD800a','\uD800\uD800','\uDC00\uD800'];
            for(var i=0;i<bad.length;i++){
                assert.throws(URIError,function(){encodeURI(bad[i]);});assert.throws(URIError,function(){encodeURIComponent(bad[i]);});
            }
        "#,
        );
    }

    #[test]
    fn uri_globals_precharge_exact_output_transients_and_ignore_extra_content() {
        for name in [
            "encodeURI",
            "encodeURIComponent",
            "decodeURI",
            "decodeURIComponent",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            let function = runtime.environments[0].bindings[name].value.clone();
            let huge = Value::String(JsString::from(vec![97; MAX_STRING]));
            runtime.steps = 5;
            runtime.allocated = MAX_HEAP - 68;
            assert_eq!(
                runtime
                    .call(
                        function.clone(),
                        vec![Value::String("a".into()), huge.clone()],
                        huge,
                        &mut doc
                    )
                    .unwrap(),
                Value::String("a".into())
            );
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            runtime.steps = MAX_STEPS;
            assert!(
                runtime
                    .call(
                        function,
                        vec![Value::String("a".into())],
                        Value::Null,
                        &mut doc
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
        let mut runtime = Runtime::new();
        let mode = js_uri::Mode::Encode { component: true };
        let input = JsString::from(" ");
        runtime.steps = 3;
        runtime.allocated = MAX_HEAP - 76;
        assert!(
            runtime
                .uri_value(&input, mode)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, MAX_HEAP - 76);
        runtime.steps = 4;
        assert_eq!(
            runtime.uri_value(&input, mode).unwrap(),
            Value::String("%20".into())
        );
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    }

    #[test]
    fn uri_globals_complete_conversion_before_output_and_input_limit_stops() {
        for output_limit in [false, true] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var text='',seen=false;var object={toString:function(){seen=true;return text;}};",&mut doc).unwrap();
            runtime.environments[0]
                .bindings
                .get_mut("text")
                .unwrap()
                .value = Value::String(JsString::from(if output_limit {
                vec![0x800; 30_000]
            } else {
                vec![97; MAX_STRING]
            }));
            let error = runtime
                .execute("encodeURIComponent(object);", &mut doc)
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert!(
                error.message.contains(if output_limit {
                    "string limit"
                } else {
                    "instruction limit"
                }),
                "{error}"
            );
            assert_eq!(
                runtime.environments[0].bindings["seen"].value,
                Value::Bool(true)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
    }

    #[test]
    fn uri_globals_conversion_callbacks_share_uncatchable_limits() {
        for name in [
            "encodeURI",
            "encodeURIComponent",
            "decodeURI",
            "decodeURIComponent",
        ] {
            for body in [
                "while(true){}".to_owned(),
                format!("return {name}(object);"),
            ] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                runtime
                    .execute("var caught=false,finished=false;", &mut doc)
                    .unwrap();
                let source = format!(
                    "var object={{toString:function(){{{body}}}}};try{{{name}(object);finished=true;}}catch(e){{caught=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!(
                    runtime.environments[0].bindings["finished"].value,
                    Value::Bool(false)
                );
                assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            }
        }
    }

    fn logical_assignment_modes(source: &str) {
        for (operator, take, skip) in [("&&=", "1", "0"), ("||=", "0", "1"), ("??=", "null", "0")] {
            for strict in [false, true] {
                let (mut runtime, mut document) = upstream_harness();
                let source = source
                    .replace("@OP@", operator)
                    .replace("@TAKE@", take)
                    .replace("@SKIP@", skip)
                    .replace("@STRICT@", if strict { "true" } else { "false" });
                let result = if strict {
                    runtime.execute_strict(&source, &mut document)
                } else {
                    runtime.execute(&source, &mut document)
                };
                result.unwrap_or_else(|error| panic!("{operator} strict={strict}: {error}"));
            }
        }
    }

    #[test]
    fn logical_assignment_checks_truthiness_without_conversion_and_retains_identity() {
        number_static_modes(
            r#"
            var falsy=[undefined,null,false,0,-0,NaN,''],truthy=[true,1,-1,Infinity,'0',[],{},new Boolean(false)],rhs={};
            for(var i=0;i<falsy.length;i++){
                var value=falsy[i];assert.sameValue(value&&=(function(){throw 'unused';})(),falsy[i]);
                assert.sameValue(value,falsy[i]);assert.sameValue(value||=rhs,rhs);assert.sameValue(value,rhs);
            }
            for(var i=0;i<truthy.length;i++){
                var value=truthy[i];assert.sameValue(value||=(function(){throw 'unused';})(),truthy[i]);
                assert.sameValue(value??=(function(){throw 'unused';})(),truthy[i]);
                assert.sameValue(value&&=rhs,rhs);assert.sameValue(value,rhs);
            }
            for(var i=2;i<falsy.length;i++){
                var value=falsy[i];assert.sameValue(value??=(function(){throw 'unused';})(),falsy[i]);
            }
            var value=null;assert.sameValue(value??=rhs,rhs);value=undefined;assert.sameValue(value??=rhs,rhs);
            var object={get valueOf(){throw 'unused';},get toString(){throw 'unused';}};
            value=object;assert.sameValue(value||=1,object);assert.sameValue(value??=1,object);assert.sameValue(value&&=rhs,rhs);
        "#,
        );
    }

    #[test]
    fn logical_assignment_captures_reference_key_and_inherited_receiver_once() {
        logical_assignment_modes(
            r#"
            var trace='',stored,other={x:99},prototype={get x(){assert.sameValue(this,object);trace+='G';return @TAKE@;},
                set x(v){assert.sameValue(this,object);trace+='S';stored=v;}};
            var object=Object.create(prototype),selected=object,result={};
            function target(){trace+='O';return selected;}
            function key(){trace+='K';return {toString:function(){trace+='C';return 'x';}};}
            function rhs(){trace+='R';selected=other;return result;}
            assert.sameValue(target()[key()]@OP@rhs(),result);assert.sameValue(stored,result);
            assert.sameValue(trace,'OKCGRS');assert.sameValue(other.x,99);
            Object.defineProperty(prototype,'x',{get:function(){trace+='G';return @SKIP@;},set:function(){throw 'unused';},configurable:true});
            selected=object;trace='';assert.sameValue(target()[key()]@OP@rhs(),@SKIP@);assert.sameValue(trace,'OKCG');
        "#,
        );
    }

    #[test]
    fn logical_assignment_short_circuit_avoids_readonly_writes_but_taken_path_does_not() {
        logical_assignment_modes(
            r#"
            var calls=0;function rhs(){calls++;return 9;}
            const held=@SKIP@;assert.sameValue(held@OP@rhs(),@SKIP@);assert.sameValue(calls,0);
            const locked=@TAKE@;assert.throws(TypeError,function(){locked@OP@rhs();});assert.sameValue(calls,1);
            var object={};Object.defineProperty(object,'x',{value:@SKIP@,writable:false,configurable:true});
            assert.sameValue(object.x@OP@rhs(),@SKIP@);assert.sameValue(calls,1);
            Object.defineProperty(object,'x',{value:@TAKE@});
            if(@STRICT@)assert.throws(TypeError,function(){object.x@OP@rhs();});
            else assert.sameValue(object.x@OP@rhs(),9);
            assert.sameValue(object.x,@TAKE@);assert.sameValue(calls,2);
            Object.defineProperty(object,'x',{get:function(){return @SKIP@;},configurable:true});
            assert.sameValue(object.x@OP@rhs(),@SKIP@);assert.sameValue(calls,2);
            assert.throws(ReferenceError,function(){missingLogical@OP@rhs();});assert.sameValue(calls,2);
            assert.throws(ReferenceError,function(){early@OP@rhs();let early;});assert.sameValue(calls,2);
        "#,
        );
    }

    #[test]
    fn logical_assignment_abrupt_stages_preserve_identity_and_prior_effects() {
        logical_assignment_modes(
            r#"
            var reason={},seen,trace='',object={get x(){trace+='G';return @TAKE@;},set x(v){trace+='S';throw reason;}};
            function rhs(){trace+='R';return 7;}
            try{object.x@OP@rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'GRS');
            trace='';seen=undefined;function bad(){trace+='R';throw reason;}
            try{object.x@OP@bad();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'GR');
            Object.defineProperty(object,'x',{get:function(){trace+='G';throw reason;},configurable:true});
            trace='';seen=undefined;try{object.x@OP@rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'G');
            trace='';seen=undefined;var key={toString:function(){trace+='K';throw reason;}};
            try{object[key]@OP@rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'K');
            trace='';seen=undefined;function target(){trace+='O';throw reason;}
            try{target()[key]@OP@rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'O');
        "#,
        );
    }

    #[test]
    fn logical_assignment_names_only_anonymous_identifier_rhs_functions() {
        logical_assignment_modes(
            r#"
            var named=@TAKE@;named@OP@function(){};assert.sameValue(named.name,'named');
            var descriptor=Object.getOwnPropertyDescriptor(named,'name');assert.sameValue(descriptor.writable,false);
            assert.sameValue(descriptor.enumerable,false);assert.sameValue(descriptor.configurable,true);
            var arrow=@TAKE@;arrow@OP@()=>7;assert.sameValue(arrow.name,'arrow');assert.sameValue(arrow(),7);
            var wrapped=@TAKE@;(wrapped)@OP@(function(){});assert.sameValue(wrapped.name,'wrapped');
            var explicit=@TAKE@;explicit@OP@function own(){};assert.sameValue(explicit.name,'own');
            var indirect=@TAKE@;indirect@OP@(0,function(){});assert.sameValue(indirect.name,'');
            var object={x:@TAKE@};object.x@OP@function(){};assert.sameValue(object.x.name,'');
            var \u{10400}=@TAKE@;\u{10400}@OP@function(){};assert.sameValue(\u{10400}.name,'\uD801\uDC00');
            var held=@SKIP@;assert.sameValue(held@OP@function(){},@SKIP@);
        "#,
        );
    }

    #[test]
    fn logical_assignment_is_right_associative_and_preserves_expression_precedence() {
        logical_assignment_modes(
            r#"
            var trace='',a={get x(){trace+='A';return @TAKE@;},set x(v){trace+='a';}},
                b={get x(){trace+='B';return @TAKE@;},set x(v){trace+='b';}};
            function rhs(){trace+='R';return 3;}
            assert.sameValue(a.x@OP@b.x@OP@rhs(),3);assert.sameValue(trace,'ABRba');
            var value=@TAKE@;assert.sameValue(value@OP@1+2*3,7);
            value=@TAKE@;assert.sameValue(value@OP@false?2:4,4);
            value=@TAKE@;value
            @OP@
            5;assert.sameValue(value,5);
        "#,
        );
    }

    #[test]
    fn logical_assignment_rejects_invalid_targets_and_split_tokens_before_effects() {
        for op in ["&&=", "||=", "??="] {
            for target in ["1", "a+b", "fn()", "[x]", "({x})", "this"] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                runtime.execute("var seen=false;", &mut doc).unwrap();
                let error = runtime
                    .execute(&format!("seen=true;{target}{op}1;"), &mut doc)
                    .unwrap_err();
                assert_eq!(error.name(), "SyntaxError");
                assert_eq!(
                    runtime.environments[0].bindings["seen"].value,
                    Value::Bool(false)
                );
            }
            for target in ["eval", "arguments"] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                assert_eq!(
                    runtime
                        .execute_strict(&format!("{target}{op}1;"), &mut doc)
                        .unwrap_err()
                        .name(),
                    "SyntaxError"
                );
            }
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            assert_eq!(
                runtime
                    .execute(&format!("var a=1;a{} =2;", &op[..2]), &mut doc)
                    .unwrap_err()
                    .name(),
                "SyntaxError"
            );
        }
    }

    #[test]
    fn logical_assignment_skips_rhs_allocations_and_shares_callback_limits() {
        for (op, skip, take) in [
            ("&&=", Value::Number(0.0), Value::Number(1.0)),
            ("||=", Value::Number(1.0), Value::Number(0.0)),
            ("??=", Value::Number(0.0), Value::Null),
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("");
            runtime.execute("var held=0;", &mut doc).unwrap();
            runtime.environments[0]
                .bindings
                .get_mut("held")
                .unwrap()
                .value = skip.clone();
            let expr = Expr::Assign(
                op.into(),
                Box::new(Expr::Ident("held".into())),
                Box::new(Expr::Object(Vec::new())),
            );
            let unit = code::test_unit(&[Stmt::Expr(expr)]);
            let code::Stmt::Expr(expr) = unit.stmt(unit.body[0]) else {
                panic!("expression code");
            };
            runtime.steps = MAX_STEPS;
            runtime.allocated = MAX_HEAP;
            assert_eq!(runtime.eval(&unit, expr, 0, &mut doc).unwrap(), skip);
            assert_eq!(runtime.allocated, MAX_HEAP);
            let short_steps = MAX_STEPS - runtime.steps;
            runtime.steps = short_steps;
            assert_eq!(runtime.eval(&unit, expr, 0, &mut doc).unwrap(), skip);
            assert_eq!(runtime.steps, 0);
            runtime.steps = MAX_STEPS;
            runtime.environments[0]
                .bindings
                .get_mut("held")
                .unwrap()
                .value = take;
            assert!(
                runtime
                    .eval(&unit, expr, 0, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                (runtime.calls, runtime.eval_depth, runtime.stack_units),
                (0, 0, 0)
            );
        }
        for op in ["&&=", "||=", "??="] {
            for body in ["while(true){}", "return recurse();"] {
                let mut runtime = Runtime::new();
                let mut doc = Document::parse("");
                let take = if op == "&&=" { "1" } else { "null" };
                runtime
                    .execute("var caught=false,wrote=false;", &mut doc)
                    .unwrap();
                let source = format!(
                    "function recurse(){{{body}}}var object={{get x(){{return {take};}},set x(v){{wrote=true;}}}};try{{object.x{op}recurse();}}catch(e){{caught=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(
                    runtime.environments[0].bindings["wrote"].value,
                    Value::Bool(false)
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!(
                    (runtime.calls, runtime.eval_depth, runtime.stack_units),
                    (0, 0, 0)
                );
            }
        }
    }

    #[test]
    fn addition_selects_numeric_or_string_behavior_after_both_conversions() {
        number_static_modes(
            r#"
            assert.sameValue({valueOf:function(){return '1';}}+2,'12');
            assert.sameValue(2+{valueOf:function(){return '1';}},'21');
            assert.sameValue(new String('a')+new String('b'),'ab');
            assert.sameValue(new Number(2)+new Boolean(true),3);
            assert.sameValue(new String('a')+null,'anull');assert.sameValue(undefined+new String('a'),'undefineda');
            assert.sameValue([]+1,'1');assert.sameValue([1,2]+3,'1,23');assert.sameValue({}+1,'[object Object]1');
            assert.sameValue(null+true,1);assert.sameValue(false+undefined,NaN);
            assert.sameValue(-0+-0,-0);assert.sameValue(0+-0,0);assert.sameValue(Infinity+-Infinity,NaN);
            assert.sameValue(''+(-0),'0');assert.sameValue(''+1e21,'1e+21');assert.sameValue(''+1e-7,'1e-7');
            var calls=0,o={valueOf:function(){calls++;return 'x';},get toString(){throw 'unused';}};
            assert.sameValue(o+'y','xy');assert.sameValue(calls,1);
        "#,
        );
    }

    #[test]
    fn addition_preserves_expression_hook_order_and_left_association() {
        number_static_modes(
            r#"
            var trace='',left={valueOf:function(){trace+='L';return 'a';}},right={valueOf:function(){trace+='R';return 1;}};
            function a(){trace+='A';return left;}function b(){trace+='B';return right;}function c(){trace+='C';return 2;}
            assert.sameValue(a()+b()+c(),'a12');assert.sameValue(trace,'ABLRC');
            trace='';assert.sameValue(a()+(b()+c()),'a3');assert.sameValue(trace,'ABCRL');
            trace='';left={get valueOf(){trace+='V';return function(){assert.sameValue(this,left);trace+='v';
                Object.defineProperty(left,'toString',{get:function(){trace+='T';return function(){assert.sameValue(this,left);trace+='t';return 'x';};},configurable:true});
                right.valueOf=function(){trace+='R';return 'y';};return {};};},toString:function(){throw 'stale';}};
            right={valueOf:function(){throw 'stale';}};
            assert.sameValue(left+right,'xy');assert.sameValue(trace,'VvTtR');
            assert.sameValue({valueOf:null,toString:function(){return 'z';}}+false,'zfalse');
            assert.throws(TypeError,function(){return Object.create(null)+1;});
            assert.throws(TypeError,function(){return {valueOf:function(){return {};},toString:function(){return [];}}+1;});
        "#,
        );
    }

    #[test]
    fn addition_abrupt_completion_preserves_exact_identity_and_stops_later_hooks() {
        number_static_modes(
            r#"
            var reason={},seen,trace='',left={get valueOf(){trace+='L';throw reason;}},right={get valueOf(){trace+='R';throw 'unused';}};
            function rhs(){trace+='B';return right;}
            try{left+rhs();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'BL');
            trace='';seen=undefined;left={valueOf:function(){trace+='L';return 'x';}};
            right={get valueOf(){trace+='R';throw reason;}};
            try{left+right;}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'LR');
            trace='';seen=undefined;try{left+(function(){trace+='B';throw reason;})();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'B');
            var object={get x(){trace+='G';return left;},set x(value){trace+='S';}};trace='';seen=undefined;
            try{object.x+=right;}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(trace,'GLR');
        "#,
        );
    }

    #[test]
    fn addition_compound_reference_and_utf16_strings_remain_lossless() {
        number_static_modes(
            r#"
            var trace='',stored,object={get x(){trace+='G';return {valueOf:function(){trace+='L';return '\uD800';}};},
                set x(value){trace+='S';stored=value;}};
            function target(){trace+='O';return object;}function key(){trace+='K';return 'x';}
            function rhs(){trace+='R';return {valueOf:function(){trace+='V';return '\uDC00';}};}
            var result=(target()[key()]+=rhs());assert.sameValue(result,'\uD800\uDC00');assert.sameValue(stored,result);
            assert.sameValue(trace,'OKGRLVS');assert.sameValue(result.length,2);
            assert.sameValue(result.charCodeAt(0),0xD800);assert.sameValue(result.charCodeAt(1),0xDC00);
            assert.sameValue('\uDC00'+{valueOf:function(){return '\uD800';}},'\uDC00\uD800');
            const fixed='x';trace='';assert.throws(TypeError,function(){fixed+=rhs();});assert.sameValue(trace,'RV');
        "#,
        );
    }

    #[test]
    fn addition_precharges_copy_work_and_vec_rc_storage_before_allocation() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let a = Value::String(JsString::from(vec![0xd800; 32]));
        let b = Value::String(JsString::from(vec![0xdc00; 33]));
        let before = runtime.allocated;
        runtime.steps = 16;
        assert!(
            runtime
                .addition_value(a.clone(), b.clone(), &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((runtime.allocated, runtime.steps), (before, 0));
        runtime.allocated = MAX_HEAP - 324;
        runtime.steps = 17;
        let Value::String(result) = runtime
            .addition_value(a.clone(), b.clone(), &mut document)
            .unwrap()
        else {
            panic!("string")
        };
        assert_eq!(result.len(), 65);
        assert_eq!(&result.units()[..32], &[0xd800; 32]);
        assert_eq!(&result.units()[32..], &[0xdc00; 33]);
        assert_eq!((runtime.allocated, runtime.steps), (MAX_HEAP, 0));
        runtime.steps = 17;
        assert!(
            runtime
                .addition_value(a, b, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, MAX_HEAP + 324);
    }

    #[test]
    fn addition_preserves_both_hook_effects_before_heap_or_string_limit_failure() {
        for length_limit in [false, true] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            runtime.execute("var text='',readL=false,readR=false;var left={valueOf:function(){readL=true;return text;}},right={valueOf:function(){readR=true;return 'x';}};",&mut document).unwrap();
            runtime.environments[0]
                .bindings
                .get_mut("text")
                .unwrap()
                .value = Value::String(JsString::from(vec![
                97;
                if length_limit {
                    MAX_STRING
                } else {
                    8192
                }
            ]));
            let left = runtime.environments[0].bindings["left"].value.clone();
            let right = runtime.environments[0].bindings["right"].value.clone();
            if !length_limit {
                runtime.allocated = MAX_HEAP - 4096;
            }
            runtime.steps = MAX_STEPS;
            let error = runtime
                .addition_value(left, right, &mut document)
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert!(
                error.to_string().contains(if length_limit {
                    "string limit"
                } else {
                    "allocation"
                }),
                "{error}"
            );
            assert_eq!(
                runtime.environments[0].bindings["readL"].value,
                Value::Bool(true)
            );
            assert_eq!(
                runtime.environments[0].bindings["readR"].value,
                Value::Bool(true)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
    }

    #[test]
    fn addition_recursive_and_looping_hooks_share_uncatchable_limits() {
        for body in ["return object+1;", "while(true){}"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            runtime
                .execute("var wrote=false,caught=false;", &mut document)
                .unwrap();
            let source = format!(
                "var object={{valueOf:function(){{{body}}}}},target={{get x(){{return object;}},set x(value){{wrote=true;}}}};try{{target.x+='x';}}catch(e){{caught=true;}}"
            );
            assert!(
                runtime
                    .execute(&source, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.environments[0].bindings["wrote"].value,
                Value::Bool(false)
            );
            assert_eq!(
                runtime.environments[0].bindings["caught"].value,
                Value::Bool(false)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
    }

    fn compound_bitwise_modes(source: &str) {
        for (operator, result) in [
            ("<<=", 18),
            (">>=", 4),
            (">>>=", 4),
            ("&=", 1),
            ("^=", 8),
            ("|=", 9),
        ] {
            for strict in [false, true] {
                let (mut runtime, mut document) = upstream_harness();
                let source = source
                    .replace("@OP@", operator)
                    .replace("@RESULT@", &result.to_string())
                    .replace("@STRICT@", if strict { "true" } else { "false" });
                let result = if strict {
                    runtime.execute_strict(&source, &mut document)
                } else {
                    runtime.execute(&source, &mut document)
                };
                result.unwrap_or_else(|e| panic!("{operator} strict={strict}: {e}"));
            }
        }
    }

    #[test]
    fn compound_bitwise_references_are_read_once_and_keep_the_original_receiver() {
        compound_bitwise_modes(
            r#"
            var trace='',stored,old={valueOf:function(){trace+='L';return 9;}},
                right={valueOf:function(){trace+='V';return 1;}};
            var prototype={get x(){assert.sameValue(this,object);trace+='G';return old;},
                set x(value){assert.sameValue(this,object);trace+='S';stored=value;}};
            var object=Object.create(prototype),other={x:99},selected=object;
            function target(){trace+='O';return selected;}
            function key(){trace+='K';return {toString:function(){trace+='C';return 'x';}};}
            function rhs(){trace+='R';selected=other;return right;}
            var result=(target()[key()]@OP@rhs());
            assert.sameValue(result,@RESULT@);assert.sameValue(stored,result);
            assert.sameValue(trace,'OKCGRLVS');assert.sameValue(other.x,99);
            trace='';old.valueOf=function(){trace+='L';right.valueOf=function(){trace+='V';return 1;};return 9;};
            right.valueOf=function(){throw 'stale';};
            result=(object.x@OP@right);assert.sameValue(result,@RESULT@);assert.sameValue(trace,'GLVS');
        "#,
        );
    }

    #[test]
    fn compound_bitwise_abrupt_stages_preserve_identity_and_prior_effects() {
        compound_bitwise_modes(
            r#"
            var reason={},seen,trace='',stored;
            var object={get x(){trace+='G';return {valueOf:function(){trace+='L';throw reason;}};},
                set x(value){trace+='S';stored=value;}};
            function rhs(){trace+='R';return {valueOf:function(){trace+='V';return 1;}};}
            try{object.x@OP@rhs();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'GRL');assert.sameValue(stored,undefined);
            trace='';seen=undefined;
            try{object.x@OP@(function(){trace+='R';throw reason;})();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'GR');
            trace='';seen=undefined;var key={toString:function(){trace+='K';throw reason;}};
            try{object[key]@OP@rhs();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'K');
            trace='';seen=undefined;object={get x(){trace+='G';return 9;},set x(value){trace+='S';stored=value;throw reason;}};
            try{object.x@OP@rhs();}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(stored,@RESULT@);assert.sameValue(trace,'GRVS');
        "#,
        );
    }

    #[test]
    fn compound_bitwise_readonly_const_tdz_and_missing_bindings_keep_order() {
        compound_bitwise_modes(
            r#"
            var trace='',returned,seen,object={};Object.defineProperty(object,'x',{value:9,writable:false});
            function rhs(){trace+='R';return {valueOf:function(){trace+='V';return 1;}};}
            try{returned=(object.x@OP@rhs());}catch(e){seen=e;}
            assert.sameValue(object.x,9);assert.sameValue(trace,'RV');
            if(@STRICT@){assert.sameValue(seen.constructor,TypeError);assert.sameValue(returned,undefined);}
            else{assert.sameValue(seen,undefined);assert.sameValue(returned,@RESULT@);}
            trace='';const fixed=9;assert.throws(TypeError,function(){fixed@OP@rhs();});
            assert.sameValue(fixed,9);assert.sameValue(trace,'RV');
            trace='';assert.throws(ReferenceError,function(){missingCompoundTarget@OP@rhs();});
            assert.sameValue(trace,'');
            assert.throws(ReferenceError,function(){later@OP@rhs();let later=9;});assert.sameValue(trace,'');
        "#,
        );
    }

    #[test]
    fn compound_bitwise_preserves_to_int32_and_masked_shift_boundaries() {
        number_static_modes(
            r#"
            var a=-1;assert.sameValue(a>>>=0,4294967295);assert.sameValue(a,4294967295);
            a=-1;assert.sameValue(a>>=1,-1);a=-1;assert.sameValue(a<<=1,-2);
            a=1;assert.sameValue(a<<=-1,-2147483648);a=-2147483648;assert.sameValue(a>>>=-1,1);
            a=4294967297;assert.sameValue(a<<=32,1);a=5.9;assert.sameValue(a>>=33.9,2);
            a=-0;assert.sameValue(a|=0,0);a=NaN;assert.sameValue(a^=1,1);
            a=Infinity;assert.sameValue(a&=-1,0);a=-Infinity;assert.sameValue(a>>>=1,0);
            a='-1';assert.sameValue(a>>>=0,4294967295);a='9';assert.sameValue(a&='1',1);
            a=null;assert.sameValue(a|=true,1);a=undefined;assert.sameValue(a^=true,1);
            var b=2;a=9;assert.sameValue(a<<=b>>>=1,18);assert.sameValue(a,18);assert.sameValue(b,1);
            var x=9,y=1;assert.sameValue(x>>=y+1,2);assert.sameValue(x,2);
            var x=9;x
            >>>=1;assert.sameValue(x,4);
        "#,
        );
    }

    #[test]
    fn compound_bitwise_invalid_targets_and_split_tokens_reject_before_effects() {
        for operator in ["<<=", ">>=", ">>>=", "&=", "^=", "|="] {
            for target in ["1", "(1+2)", "true", "{}"] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                runtime
                    .execute("var touched=false;", &mut document)
                    .unwrap();
                let source = format!("touched=true;{target}{operator}1;");
                let error = runtime.execute(&source, &mut document).unwrap_err();
                assert_eq!(
                    error.intrinsic_error_name(),
                    Some("SyntaxError"),
                    "{source}: {error}"
                );
                assert_eq!(
                    runtime.environments[0].bindings["touched"].value,
                    Value::Bool(false)
                );
            }
        }
        for source in [
            "var a=9;a > >>=1;",
            "var a=9;a >> =1;",
            "var a=9;a & =1;",
            "var a=9;a >>>==1;",
        ] {
            assert_eq!(
                run(source).unwrap_err().intrinsic_error_name(),
                Some("SyntaxError"),
                "{source}"
            );
        }
    }

    #[test]
    fn compound_bitwise_executes_the_unchanged_hexadecimal_helper() {
        let (mut runtime, mut document) = upstream_harness();
        runtime
            .execute(
                include_str!(
                    "../tests/upstream/test262-numeric-parsing/harness/decimalToHexString.js"
                ),
                &mut document,
            )
            .unwrap();
        for strict in [false, true] {
            let source = r#"assert.sameValue(decimalToHexString(0),'0000');assert.sameValue(decimalToHexString(-1),'FFFFFFFF');
                assert.sameValue(decimalToHexString(4294967297),'0001');assert.sameValue(decimalToHexString(0xd800),'D800');
                assert.sameValue(decimalToHexString(0xffff),'FFFF');assert.sameValue(decimalToPercentHexString(255),'%FF');"#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn compound_bitwise_resource_failure_cannot_write_or_be_caught() {
        for operator in ["<<=", ">>=", ">>>=", "&=", "^=", "|="] {
            for body in ["while(true){}", "return Number(object.x);"] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                runtime
                    .execute("var wrote=false,caught=false;", &mut document)
                    .unwrap();
                let source = format!(
                    "var object={{get x(){{return {{valueOf:function(){{{body}}}}};}},set x(value){{wrote=true;}}}};try{{object.x{operator}1;}}catch(e){{caught=true;}}"
                );
                let error = runtime.execute(&source, &mut document).unwrap_err();
                assert!(error.is_resource_limit(), "{operator}: {error}");
                assert_eq!(
                    runtime.environments[0].bindings["wrote"].value,
                    Value::Bool(false)
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            }
        }
    }

    #[test]
    fn numeric_parsing_converts_string_before_radix_and_reads_live_fallback() {
        number_static_modes(
            r#"
            var methods=[parseInt,parseFloat];
            for(var i=0;i<2;i++){
                var m=methods[i],trace='',o={get toString(){trace+='T';return function(){
                    assert.sameValue(this,o);trace+='t';Object.defineProperty(o,'valueOf',{
                        get:function(){trace+='V';return function(){assert.sameValue(this,o);trace+='v';return '12';};},configurable:true});return {};};},
                    valueOf:function(){throw 'stale';}};
                var r={get valueOf(){trace+='R';return function(){assert.sameValue(this,r);trace+='r';return 10;};}};
                assert.sameValue(m(o,r),12);assert.sameValue(trace,i===0?'TtVvRr':'TtVv');
                assert.sameValue(m({toString:null,valueOf:function(){return '13';}},10),13);
                assert.sameValue(m({toString:function(){return 14;},get valueOf(){throw 'unused';}},10),14);
                assert.throws(TypeError,function(){m({toString:function(){return {};},valueOf:function(){return [];}});});
            }
            var trace='',input={toString:function(){trace+='s';radix.valueOf=function(){trace+='r';return 16;};return '10';}},radix={};
            assert.sameValue(parseInt(input,radix),16);assert.sameValue(trace,'sr');
            trace='';assert.sameValue(parseInt({toString:function(){trace+='s';return '12';}},1),NaN);
            assert.sameValue(trace,'s');
        "#,
        );
    }

    #[test]
    fn numeric_parsing_preserves_abrupt_identity_and_argument_evaluation() {
        number_static_modes(
            r#"
            var methods=[parseInt,parseFloat];
            for(var i=0;i<2;i++){
                var m=methods[i],trace='',reason={},seen;
                function first(){trace+='a';return {toString:function(){trace+='s';throw reason;}};}
                function extra(){trace+='b';return {get toString(){throw 'unused';}};}
                try{m(first(),10,extra());}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'abs');trace='';seen=undefined;
                try{m(first(),10,(function(){trace+='t';throw reason;})());}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'at');
                var later=false;seen=undefined;
                try{m({get toString(){throw reason;}},{get valueOf(){later=true;return function(){return 10;};}});}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(later,false);
            }
            var trace='',reason={},seen;
            try{parseInt({toString:function(){trace+='s';return '12';}},{valueOf:function(){trace+='r';throw reason;}});}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(trace,'sr');
        "#,
        );
    }

    #[test]
    fn numeric_parsing_aliases_metadata_and_nonconstruction_keep_identity() {
        number_static_modes(
            r#"
            var N=Number,methods=[parseInt,parseFloat],names=['parseInt','parseFloat'];
            for(var i=0;i<2;i++){
                var m=methods[i],name=names[i],length=i===0?2:1;
                assert.sameValue(N[name],m);assert.sameValue(m.name,name);assert.sameValue(m.length,length);
                var d=Object.getOwnPropertyDescriptor(N,name);
                assert.sameValue(d.value,m);assert.sameValue(d.writable,true);
                assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
                var n=Object.getOwnPropertyDescriptor(m,'name'),l=Object.getOwnPropertyDescriptor(m,'length');
                assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
                assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
                assert.sameValue(Object.getPrototypeOf(m),Function.prototype);
                assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);
                assert.throws(TypeError,function(){new m('1');});
                var bound=m.bind(null,'12',10);assert.throws(TypeError,function(){new bound();});
                Object.defineProperty(m,'name',{value:'changed'});assert.sameValue(bound(),12);
                N[name]=function(){throw 'replacement';};assert.sameValue(m('13',10),13);
                assert.sameValue(delete N[name],true);Object.defineProperty(N,name,d);
            }
            var I=parseInt,F=parseFloat;parseInt=function(){throw 1;};parseFloat=parseInt;Number={};
            assert.sameValue(N.parseInt,I);assert.sameValue(N.parseFloat,F);
            assert.sameValue(I({toString:function(){return '14';}},10),14);
            assert.sameValue(F({toString:function(){return '1.5';}}),1.5);
        "#,
        );
    }

    #[test]
    fn numeric_parsing_prefix_grammar_radix_wrapping_and_rounding() {
        number_static_modes(
            r#"
            var cases=[['  -0x10tail',undefined,-16],['0b11',undefined,0],['0o11',undefined,0],
                ['0x10',10,0],['0x10',16,16],['08',undefined,8],['11',4294967298,3],
                ['11',-4294967294,3],['12',4294967296,12],['12',NaN,12],['12',Infinity,12],
                ['12',2.9,1],['12',-2,NaN],['12',37,NaN],['-0tail',10,-0],['+zZ',36,1295],
                ['\uFEFF12\uD800',10,12],['\uD80012',10,NaN],['\u180E12',10,NaN],
                ['0x',16,NaN],['+',10,NaN],['1_2',10,1],['１２',10,NaN],
                ['900719925474099267',10,900719925474099300],['20000000000003',16,9007199254740996]];
            for(var i=0;i<cases.length;i++)assert.sameValue(parseInt(cases[i][0],cases[i][1]),cases[i][2]);
            var floats=[['1.25e2x',125],['1e+',1],['1.e-',1],['.5x',0.5],['.e1',NaN],['+Infinityx',Infinity],
                ['-Infinityx',-Infinity],['inf',NaN],['-0tail',-0],['-1e-999',-0],['1e309',Infinity],
                ['0x10',0],['0b1',0],['\u00A0\u202912.5\uDC00',12.5],['\u008512',NaN],
                ['1_2',1],['\uD80012',NaN],['1.00000000000000011102230246251565404236316680908203125',1]];
            for(var i=0;i<floats.length;i++)assert.sameValue(parseFloat(floats[i][0]),floats[i][1]);
            assert.sameValue(parseFloat(),NaN);assert.sameValue(parseInt(),NaN);
            assert.sameValue(parseFloat(true),NaN);assert.sameValue(parseInt(null),NaN);
            assert.sameValue(parseFloat([12]),12);assert.sameValue(parseInt(new Number(12)),12);
        "#,
        );
    }

    #[test]
    fn numeric_parsing_precharges_only_required_converted_input() {
        for name in ["parseInt", "parseFloat"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let function = runtime.environments[0].bindings[name].value.clone();
            let huge = Value::String(JsString::from(vec![120; MAX_STRING]));
            let radix = if name == "parseInt" {
                Value::Number(10.0)
            } else {
                huge.clone()
            };
            runtime.allocated = MAX_HEAP - 44;
            runtime.steps = 5;
            assert_eq!(
                runtime
                    .call(
                        function.clone(),
                        vec![Value::String("12".into()), radix, huge.clone()],
                        Value::Document,
                        &mut document
                    )
                    .unwrap(),
                Value::Number(12.0)
            );
            assert_eq!(
                (
                    runtime.allocated,
                    runtime.steps,
                    runtime.calls,
                    runtime.stack_units
                ),
                (MAX_HEAP, 0, 0, 0)
            );
            runtime.steps = MAX_STEPS;
            assert!(
                runtime
                    .call(
                        function,
                        vec![Value::String("12".into())],
                        Value::Null,
                        &mut document
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            if name == "parseInt" {
                runtime.steps = 2;
                let function = runtime.environments[0].bindings[name].value.clone();
                let Value::Number(result) = runtime
                    .call(
                        function,
                        vec![huge, Value::Number(1.0)],
                        Value::Document,
                        &mut document,
                    )
                    .unwrap()
                else {
                    panic!("number result")
                };
                assert!(result.is_nan());
                assert_eq!(
                    (
                        runtime.allocated,
                        runtime.steps,
                        runtime.calls,
                        runtime.stack_units
                    ),
                    // Failed charges remain in the shared ledger; the invalid
                    // radix return neither allocates nor refunds that charge.
                    (MAX_HEAP + 44, 0, 0, 0)
                );
            }
        }
    }

    #[test]
    fn numeric_parsing_preserves_hook_effects_before_scratch_failure() {
        for name in ["parseInt", "parseFloat"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let source = format!(
                "var stringRead=false,radixRead=false,text='{}';var input={{toString:function(){{stringRead=true;return text;}}}},radix={{valueOf:function(){{radixRead=true;return 10;}}}};",
                "1".repeat(8192)
            );
            runtime.execute(&source, &mut document).unwrap();
            let function = runtime.environments[0].bindings[name].value.clone();
            let arguments = vec![
                runtime.environments[0].bindings["input"].value.clone(),
                runtime.environments[0].bindings["radix"].value.clone(),
            ];
            runtime.allocated = MAX_HEAP - 4096;
            runtime.steps = MAX_STEPS;
            let error = runtime
                .call(function, arguments, Value::Null, &mut document)
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert!(error.to_string().contains("allocation"), "{error}");
            assert_eq!(
                runtime.environments[0].bindings["stringRead"].value,
                Value::Bool(true)
            );
            assert_eq!(
                runtime.environments[0].bindings["radixRead"].value,
                Value::Bool(name == "parseInt")
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
    }

    #[test]
    fn numeric_parsing_recursive_and_looping_hooks_share_uncatchable_limits() {
        for name in ["parseInt", "parseFloat"] {
            for body in [format!("return {name}(o);"), "while(true){}".into()] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                runtime.execute("var caught=false;", &mut document).unwrap();
                let source = format!(
                    "var o={{toString:function(){{{body}}}}};try{{{name}(o);}}catch(e){{caught=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut document)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            }
        }
    }

    #[test]
    fn numeric_conversion_observes_live_hooks_receivers_and_primitive_fallback() {
        number_static_modes(
            r#"
            var methods=[Number,isFinite,isNaN];
            for(var i=0;i<methods.length;i++){
                var fn=methods[i],trace='',o={get valueOf(){trace+='V';return function(){
                    assert.sameValue(this,o);trace+='v';
                    Object.defineProperty(o,'toString',{get:function(){trace+='T';return function(){
                        assert.sameValue(this,o);trace+='t';return '7';};},configurable:true});return {};};},
                    toString:function(){throw 'stale';}};
                assert.sameValue(fn(o),i===0?7:i===1);assert.sameValue(trace,'VvTt');
                trace='';o={valueOf:null,toString:function(){trace+='t';return '-0';}};
                assert.sameValue(fn(o),i===0?-0:i===1);assert.sameValue(trace,'t');
                o={valueOf:function(){return undefined;},get toString(){throw 'unused';}};
                assert.sameValue(fn(o),i===0?NaN:i===2);
                assert.throws(TypeError,function(){fn(Object.create(null));});
                assert.throws(TypeError,function(){fn({valueOf:function(){return {};},toString:function(){return [];}});});
            }
        "#,
        );
    }

    #[test]
    fn numeric_conversion_preserves_abrupt_identity_and_argument_order() {
        number_static_modes(
            r#"
            var methods=[Number,isFinite,isNaN];
            for(var i=0;i<methods.length;i++){
                var fn=methods[i],trace='',reason={},seen,bomb={get valueOf(){throw 'receiver';}};
                function first(){trace+='a';return {valueOf:function(){trace+='v';throw reason;}};}
                function extra(){trace+='b';return bomb;}
                try{fn.call(bomb,first(),extra());}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'abv');
                trace='';seen=undefined;
                try{fn(first(),(function(){trace+='t';throw reason;})());}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'at');
                seen=undefined;trace='';
                try{fn({get valueOf(){trace+='V';throw reason;},get toString(){throw 'unused';}});}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'V');
                seen=undefined;trace='';
                try{fn({valueOf:function(){trace+='v';return {};},get toString(){trace+='T';throw reason;}});}catch(e){seen=e;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'vT');
            }
        "#,
        );
    }

    #[test]
    fn numeric_conversion_boxes_bound_constructors_and_keeps_saved_aliases() {
        number_static_modes(
            r#"
            var N=Number,F=isFinite,I=isNaN,prototype=N.prototype,trace='';
            var o={valueOf:function(){trace+='v';return -0;}};
            var B=N.bind({valueOf:function(){throw 'receiver';}},o);
            Number=function(){throw 'replacement';};isFinite=function(){throw 'replacement';};isNaN=isFinite;
            var a=new N(),b=new N(undefined),c=new B({valueOf:function(){throw 'extra';}});
            assert.sameValue(a.valueOf(),0);assert.sameValue(b.valueOf(),NaN);
            assert.sameValue(c.valueOf(),-0);assert.sameValue(Object.getPrototypeOf(c),prototype);
            assert.sameValue(N(c),-0);assert.sameValue(F(c),true);assert.sameValue(I(b),true);
            assert.sameValue(trace,'v');assert.sameValue(N(),0);assert.sameValue(N(undefined),NaN);
            assert.sameValue(F(),false);assert.sameValue(I(),true);
            assert.sameValue(F.name,'isFinite');assert.sameValue(I.name,'isNaN');
            assert.sameValue(F.length,1);assert.sameValue(I.length,1);
            assert.throws(TypeError,function(){new F(1);});assert.throws(TypeError,function(){new I(1);});
        "#,
        );
    }

    #[test]
    fn numeric_conversion_strings_preserve_utf16_grammar_and_signed_zero() {
        number_static_modes(
            r#"
            var cases=[['',0],[' \t\r\n\uFEFF',0],['\u00A0-0\u2029',-0],
                ['0x20000000000001',9007199254740992],['0o10',8],['0b11',3],
                ['1e309',Infinity],['-1e-999',-0],['+Infinity',Infinity],['-Infinity',-Infinity],
                ['\u180E1',NaN],['1\u0085',NaN],['\uD800',NaN],['\uDC00',NaN],
                ['+0x10',NaN],['-0b1',NaN],['1_0',NaN],['1e',NaN],['inf',NaN],['1x',NaN]];
            for(var i=0;i<cases.length;i++){
                var text=cases[i][0],expected=cases[i][1],o={valueOf:function(){return text;}};
                assert.sameValue(Number(text),expected);assert.sameValue(Number(o),expected);
                assert.sameValue(isFinite(o),Number.isFinite(expected));
                assert.sameValue(isNaN(o),Number.isNaN(expected));
            }
            var reads=0,o={valueOf:function(){reads++;return 1;}};
            assert.sameValue(Number.isFinite(o),false);assert.sameValue(Number.isNaN(o),false);
            assert.sameValue(reads,0);assert.sameValue(isFinite(o),true);assert.sameValue(reads,1);
        "#,
        );
    }

    #[test]
    fn numeric_conversion_ignores_prebuilt_extra_values_at_heap_ceiling() {
        for name in ["Number", "isFinite", "isNaN"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let function = runtime.environments[0].bindings[name].value.clone();
            let huge = Value::String(JsString::from(vec![120; MAX_STRING]));
            let array = runtime.array(vec![Value::Undefined; 65_536]).unwrap();
            runtime.allocated = MAX_HEAP;
            runtime.steps = 2;
            assert_eq!(
                runtime
                    .call(
                        function.clone(),
                        vec![Value::Number(1.0), huge, array],
                        Value::Document,
                        &mut document
                    )
                    .unwrap(),
                if name == "Number" {
                    Value::Number(1.0)
                } else {
                    Value::Bool(name == "isFinite")
                }
            );
            assert_eq!(
                (
                    runtime.allocated,
                    runtime.steps,
                    runtime.calls,
                    runtime.stack_units
                ),
                (MAX_HEAP, 0, 0, 0)
            );
            runtime.steps = 1;
            assert!(
                runtime
                    .call(
                        function,
                        vec![Value::Number(1.0)],
                        Value::Null,
                        &mut document
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                (
                    runtime.allocated,
                    runtime.steps,
                    runtime.calls,
                    runtime.stack_units
                ),
                (MAX_HEAP, 0, 0, 0)
            );
        }
    }

    #[test]
    fn numeric_conversion_precharges_strings_after_preserving_getter_effects() {
        for hook in ["valueOf", "toString"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let source = format!(
                "var touched=false,caught=false,text='{}';var o={{valueOf:null,get {hook}(){{touched=true;return function(){{return text;}};}}}};o;",
                "1".repeat(8192)
            );
            let object = runtime.execute(&source, &mut document).unwrap();
            runtime.allocated = MAX_HEAP - 4096;
            runtime.steps = MAX_STEPS;
            let function = runtime.environments[0].bindings["Number"].value.clone();
            let error = runtime
                .call(function, vec![object], Value::Null, &mut document)
                .unwrap_err();
            assert!(error.is_resource_limit(), "{hook}: {error}");
            assert!(error.to_string().contains("allocation"), "{hook}: {error}");
            assert_eq!(
                runtime.environments[0].bindings["touched"].value,
                Value::Bool(true)
            );
            assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
        }
        let mut runtime = Runtime::new();
        let text = Value::String(JsString::from("123"));
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .primitive_number_value(text)
                .unwrap_err()
                .is_resource_limit()
        );
        let before = runtime.allocated;
        runtime.steps = 0;
        assert!(
            runtime
                .primitive_number_value(Value::String("1".into()))
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, before);
    }

    #[test]
    fn numeric_conversion_recursive_hooks_keep_shared_exhaustion_uncatchable() {
        for name in ["Number", "isFinite", "isNaN"] {
            for body in [format!("return {name}(o);"), "while(true){}".into()] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                runtime.execute("var caught=false;", &mut document).unwrap();
                let source = format!(
                    "var o={{valueOf:function(){{{body}}}}};try{{{name}(o);}}catch(e){{caught=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut document)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
                assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
            }
        }
    }

    #[test]
    fn numeric_conversion_array_index_decoder_preserves_canonical_boundaries() {
        for (key, expected) in [
            ("0", Some(0)),
            ("1", Some(1)),
            ("4294967294", Some(u32::MAX - 1)),
            ("", None),
            ("00", None),
            ("01", None),
            ("-0", None),
            ("+1", None),
            ("1.0", None),
            ("1e0", None),
            (" 1", None),
            ("4294967295", None),
            ("4294967296", None),
            ("9999999999", None),
            ("10000000000", None),
            ("１", None),
        ] {
            assert_eq!(json_array_index(&key.into()), expected, "{key}");
        }
        for units in [
            vec![0xd800],
            vec![0x31, 0xdc00],
            vec![0],
            vec![0x30, 0],
            vec![0x39; 100_000],
        ] {
            assert_eq!(json_array_index(&JsString::from(units)), None);
        }
        // Differential control against the previous canonical UTF-8/decimal
        // definition, including both valid indices and deliberately bad keys.
        let mut state = 0x6713_cdef_u32;
        for _ in 0..10_000 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            for key in [state.to_string(), format!("0{state}"), format!("{state}x")] {
                let old = key
                    .parse::<u32>()
                    .ok()
                    .filter(|n| *n != u32::MAX && n.to_string() == key);
                assert_eq!(json_array_index(&key.as_str().into()), old, "{key}");
            }
        }
    }

    #[test]
    fn number_statics_constants_have_exact_values_and_descriptors() {
        number_static_modes(
            r#"
            var names=['EPSILON','MAX_VALUE','MIN_VALUE','MAX_SAFE_INTEGER','MIN_SAFE_INTEGER',
                       'NaN','POSITIVE_INFINITY','NEGATIVE_INFINITY'];
            var values=[2.220446049250313e-16,1.7976931348623157e308,5e-324,
                        9007199254740991,-9007199254740991,NaN,Infinity,-Infinity];
            for(var i=0;i<names.length;i++){
                assert.sameValue(Number[names[i]],values[i]);
                var d=Object.getOwnPropertyDescriptor(Number,names[i]);
                assert.sameValue(d.value,values[i]);assert.sameValue(d.writable,false);
                assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
                Object.defineProperty(Number,names[i],{value:values[i]});
                assert.throws(TypeError,function(){Object.defineProperty(Number,names[i],{value:7});});
                assert.throws(TypeError,function(){Object.defineProperty(Number,names[i],{writable:true});});
                assert.throws(TypeError,function(){Object.defineProperty(Number,names[i],{enumerable:true});});
                assert.throws(TypeError,function(){Object.defineProperty(Number,names[i],{configurable:true});});
                assert.sameValue(Number[names[i]],values[i]);
            }
            assert.sameValue(Number.MIN_VALUE/2,0);assert.sameValue(-Number.MIN_VALUE/2,-0);
            assert.sameValue(Number.MIN_VALUE*2,1e-323);
            assert.sameValue(Number.MAX_VALUE*2,Infinity);
            assert.sameValue(1+Number.EPSILON,1.0000000000000002);
            assert.sameValue(1+Number.EPSILON/2,1);
            assert.sameValue(Number.MAX_SAFE_INTEGER+1,Number.MAX_SAFE_INTEGER+2);
        "#,
        );
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        for (name, bits) in [
            ("EPSILON", 0x3cb0_0000_0000_0000),
            ("MAX_VALUE", 0x7fef_ffff_ffff_ffff),
            ("MIN_VALUE", 1),
            ("MAX_SAFE_INTEGER", 0x433f_ffff_ffff_ffff),
            ("MIN_SAFE_INTEGER", 0xc33f_ffff_ffff_ffff),
        ] {
            let Value::Number(value) = runtime
                .get_key(
                    Runtime::native("Number", Value::Window),
                    &name.into(),
                    &mut document,
                )
                .unwrap()
            else {
                panic!("missing {name}")
            };
            assert_eq!(value.to_bits(), bits, "{name}");
        }
    }

    #[test]
    fn number_statics_constants_resist_strict_and_sloppy_writes() {
        let source = r#"
            var names=['EPSILON','MAX_VALUE','MIN_VALUE','MAX_SAFE_INTEGER','MIN_SAFE_INTEGER',
                       'NaN','POSITIVE_INFINITY','NEGATIVE_INFINITY'];
            for(var i=0;i<names.length;i++){
                var value=Number[names[i]];assert.sameValue(typeof value,'number');
                if(strict){
                    assert.throws(TypeError,function(){Number[names[i]]=0;});
                    assert.throws(TypeError,function(){delete Number[names[i]];});
                }else{Number[names[i]]=0;assert.sameValue(delete Number[names[i]],false);}
                assert.sameValue(Number[names[i]],value);
            }
        "#;
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            runtime
                .execute(&format!("var strict={strict};"), &mut document)
                .unwrap();
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn number_statics_classify_binary64_edges_without_integer_casts() {
        number_static_modes(
            r#"
            var methods=[Number.isFinite,Number.isNaN,Number.isInteger,Number.isSafeInteger];
            var cases=[
                [0,true,false,true,true],[-0,true,false,true,true],
                [NaN,false,true,false,false],[Infinity,false,false,false,false],[-Infinity,false,false,false,false],
                [5e-324,true,false,false,false],[-5e-324,true,false,false,false],
                [2.2250738585072014e-308,true,false,false,false],
                [1.5,true,false,false,false],[-1.5,true,false,false,false],
                [9007199254740991,true,false,true,true],[-9007199254740991,true,false,true,true],
                [9007199254740992,true,false,true,false],[-9007199254740992,true,false,true,false],
                [9007199254740993,true,false,true,false],
                [1.7976931348623157e308,true,false,true,false],[-1.7976931348623157e308,true,false,true,false],
                [4500000000000000.1,true,false,true,true],[4500000000000000.5,true,false,false,false],
                [1.0000000000000002,true,false,false,false],[0.9999999999999999,true,false,false,false]
            ];
            for(var m=0;m<methods.length;m++){
                assert.sameValue(typeof methods[m],'function');
                for(var i=0;i<cases.length;i++)assert.sameValue(methods[m](cases[i][0]),cases[i][m+1]);
            }
            assert.sameValue(1/cases[1][0],-Infinity);
        "#,
        );
    }

    #[test]
    fn number_statics_do_not_coerce_nonnumbers_or_inspect_receivers() {
        number_static_modes(
            r#"
            var reads=0,bomb={get valueOf(){reads++;throw 1;},get toString(){reads++;throw 2;}};
            var inputs=[undefined,null,true,false,'1','NaN','',[],[1],{},Object.create(null),
                        new Number(1),new Number(NaN),Number.prototype,function(){},bomb,window,document,Math,JSON];
            var methods=[Number.isFinite,Number.isNaN,Number.isInteger,Number.isSafeInteger];
            for(var m=0;m<methods.length;m++){
                var fn=methods[m],positive=m===1?NaN:1;
                assert.sameValue(typeof fn,'function');assert.sameValue(fn(positive),true);
                for(var i=0;i<inputs.length;i++)assert.sameValue(fn(inputs[i]),false);
                assert.sameValue(fn(),false);assert.sameValue(fn(positive,bomb),true);
                assert.sameValue(fn.call(bomb,positive),true);assert.sameValue(fn.call(null,positive),true);
                assert.sameValue(fn.apply(undefined,[positive,bomb]),true);
                assert.sameValue(fn.bind(bomb,positive)(),true);
            }
            assert.sameValue(reads,0);
            assert.sameValue(isFinite('1'),true);assert.sameValue(Number.isFinite('1'),false);
            assert.sameValue(isNaN(undefined),true);assert.sameValue(Number.isNaN(undefined),false);
        "#,
        );
    }

    #[test]
    fn number_statics_metadata_and_saved_aliases_survive_mutation() {
        number_static_modes(
            r#"
            var owner=Number,names=['isFinite','isNaN','isInteger','isSafeInteger'];
            for(var i=0;i<names.length;i++){
                var name=names[i],fn=owner[name],positive=i===1?NaN:1;
                assert.sameValue(typeof fn,'function');assert.sameValue(fn(positive),true);
                var d=Object.getOwnPropertyDescriptor(owner,name);
                assert.sameValue(d.value,fn);assert.sameValue(d.writable,true);
                assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
                assert.sameValue(Object.getPrototypeOf(fn),Function.prototype);
                assert.sameValue(Object.getOwnPropertyDescriptor(fn,'prototype'),undefined);
                assert.throws(TypeError,function(){new fn(positive);});
                var n=Object.getOwnPropertyDescriptor(fn,'name'),l=Object.getOwnPropertyDescriptor(fn,'length');
                assert.sameValue(n.value,name);assert.sameValue(n.writable,false);
                assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
                assert.sameValue(l.value,1);assert.sameValue(l.writable,false);
                assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
                Object.defineProperty(fn,'name',{value:'changed'});assert.sameValue(fn(positive),true);
                owner[name]=function(){throw 7;};assert.sameValue(fn(positive),true);
                assert.sameValue(delete owner[name],true);assert.sameValue(owner[name],undefined);
                Number={};assert.sameValue(fn.call({},positive),true);Number=owner;
                Object.defineProperty(owner,name,d);Object.defineProperty(fn,'name',n);
                assert.sameValue(owner[name],fn);
            }
        "#,
        );
    }

    #[test]
    fn number_statics_preserve_argument_evaluation_and_abrupt_identity() {
        number_static_modes(
            r#"
            var methods=[Number.isFinite,Number.isNaN,Number.isInteger,Number.isSafeInteger];
            for(var i=0;i<methods.length;i++){
                var fn=methods[i],positive=i===1?NaN:1,trace='',reason={},seen;
                assert.sameValue(typeof fn,'function');assert.sameValue(fn(positive),true);
                function first(){trace+='a';return positive;}function second(){trace+='b';return {};}
                assert.sameValue(fn(first(),second()),true);assert.sameValue(trace,'ab');
                trace='';try{fn(first(),(function(){trace+='t';throw reason;})());}catch(error){seen=error;}
                assert.sameValue(seen,reason);assert.sameValue(trace,'at');
            }
        "#,
        );
    }

    #[test]
    fn number_statics_private_calls_have_constant_work_and_no_heap() {
        for name in ["isFinite", "isNaN", "isInteger", "isSafeInteger"] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let function = runtime
                .get_key(
                    Runtime::native("Number", Value::Window),
                    &name.into(),
                    &mut document,
                )
                .unwrap();
            let huge = Value::String(JsString::from(vec![120; MAX_STRING]));
            let array = runtime.array(vec![Value::Undefined; 65_536]).unwrap();
            let arguments = vec![
                Value::Number(if name == "isNaN" { f64::NAN } else { 1.0 }),
                huge.clone(),
                array.clone(),
            ];
            runtime.allocated = MAX_HEAP;
            runtime.steps = 5;
            assert_eq!(
                runtime
                    .call(function.clone(), arguments, Value::Window, &mut document)
                    .unwrap(),
                Value::Bool(true)
            );
            assert_eq!(
                (
                    runtime.allocated,
                    runtime.steps,
                    runtime.calls,
                    runtime.stack_units
                ),
                (MAX_HEAP, 0, 0, 0)
            );
            for value in [huge, array, Value::Document] {
                runtime.steps = 5;
                assert_eq!(
                    runtime
                        .call(function.clone(), vec![value], Value::Null, &mut document)
                        .unwrap(),
                    Value::Bool(false)
                );
                assert_eq!(
                    (
                        runtime.allocated,
                        runtime.steps,
                        runtime.calls,
                        runtime.stack_units
                    ),
                    (MAX_HEAP, 0, 0, 0)
                );
            }
            runtime.steps = 4;
            assert!(
                runtime
                    .call(
                        function,
                        vec![Value::Number(1.0)],
                        Value::Null,
                        &mut document
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                (
                    runtime.allocated,
                    runtime.steps,
                    runtime.calls,
                    runtime.stack_units
                ),
                (MAX_HEAP, 0, 0, 0)
            );
        }
    }

    #[test]
    fn number_statics_bootstrap_precharges_before_new_metadata() {
        let mut runtime = Runtime::new();
        let properties = runtime.native_properties["Number"];
        let key = JsString::from("EPSILON");
        runtime.objects[properties].remove(&(&key).into());
        let objects = runtime.objects.len();
        let registry = runtime.native_properties.len();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .initialize_number_statics()
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            runtime
                .own_property(&Runtime::native("Number", Value::Window), &key)
                .is_none()
        );
        assert_eq!(
            (runtime.objects.len(), runtime.native_properties.len()),
            (objects, registry)
        );
    }

    #[test]
    fn number_statics_keep_shared_exhaustion_uncatchable() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime.execute("var caught=false;", &mut document).unwrap();
        assert!(
            runtime
                .execute(
                    "try{while(true){Number.isFinite(1);}}catch(e){caught=true;}",
                    &mut document
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            runtime.environments[0].bindings["caught"].value,
            Value::Bool(false)
        );
        assert_eq!((runtime.calls, runtime.stack_units), (0, 0));
    }

    #[test]
    fn number_statics_enable_the_unchanged_reduce_right_safe_limit_source() {
        number_static_modes(include_str!(
            "../tests/upstream/test262-array-reduce/test/built-ins/Array/prototype/reduceRight/length-near-integer-limit.js"
        ));
    }

    fn reduction_modes(source: &str) {
        for (name, right) in [("reduce", false), ("reduceRight", true)] {
            for strict in [false, true] {
                let (mut runtime, mut document) = upstream_harness();
                runtime
                    .execute(
                        &format!(
                            "var methodName='{name}',method=Array.prototype.{name},right={right};"
                        ),
                        &mut document,
                    )
                    .unwrap();
                let result = if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                };
                result.unwrap_or_else(|error| panic!("{name} strict={strict}: {error}"));
            }
        }
    }

    #[test]
    fn array_reduce_metadata_aliases_and_call_apply_bind() {
        reduction_modes(
            r#"
            assert.sameValue(method.name,methodName);assert.sameValue(method.length,1);
            var d=Object.getOwnPropertyDescriptor(Array.prototype,methodName);
            assert.sameValue(d.value,method);assert.sameValue(d.writable,true);
            assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);
            var n=Object.getOwnPropertyDescriptor(method,'name'),l=Object.getOwnPropertyDescriptor(method,'length');
            assert.sameValue(n.writable,false);assert.sameValue(n.enumerable,false);assert.sameValue(n.configurable,true);
            assert.sameValue(l.writable,false);assert.sameValue(l.enumerable,false);assert.sameValue(l.configurable,true);
            assert.sameValue(Object.getOwnPropertyDescriptor(method,'prototype'),undefined);
            assert.throws(TypeError,function(){new method(function(){});});
            function sum(a,v){return a+v;}var poison={toString(){throw 'coercion';},valueOf(){throw 'coercion';}};
            assert.sameValue(method.call([1,2],sum,0,poison),3);
            assert.sameValue(method.apply([1,2],[sum,0,poison]),3);
            assert.sameValue(method.bind([1,2],sum,0)(),3);
            assert.sameValue(delete Array.prototype[methodName],true);
            assert.sameValue([1,2][methodName],undefined);assert.sameValue(method.call([1,2],sum,0),3);
            Object.defineProperty(Array.prototype,methodName,d);
            assert.sameValue([1,2][methodName],method);
        "#,
        );
    }

    #[test]
    fn array_reduce_boxes_and_converts_length_before_validating_callback() {
        reduction_modes(
            r#"
            var log='',reason={},indexReads=0,callback={get call(){throw 'call probe';},toString(){throw 'coercion';}};
            assert.throws(TypeError,function(){method.call(null,callback);});
            assert.throws(TypeError,function(){method.call(undefined,callback);});
            var object={get length(){log+='l';return {valueOf(){log+='v';return 2;},toString(){throw 'wrong hint';}};},
                get 0(){indexReads++;return 1;},get 1(){indexReads++;return 2;}};
            assert.throws(TypeError,function(){method.call(object,callback,0);});
            assert.sameValue(log,'lv');assert.sameValue(indexReads,0);
            log='';assert.throws(TypeError,function(){method.call(object);});assert.sameValue(log,'lv');
            object={get length(){throw reason;}};var seen;
            try{method.call(object,null);}catch(error){seen=error;}assert.sameValue(seen,reason);
            object={length:{valueOf(){throw reason;}}};seen=undefined;
            try{method.call(object,null);}catch(error){seen=error;}assert.sameValue(seen,reason);
            assert.throws(TypeError,function(){method.call({length:0},null,7);});
        "#,
        );
    }

    #[test]
    fn array_reduce_initial_presence_holes_and_undefined_are_distinct() {
        reduction_modes(
            r#"
            var count=0;function callback(a,v){count++;return a;}
            assert.throws(TypeError,function(){method.call([],callback);});
            assert.throws(TypeError,function(){method.call(new Array(3),callback);});
            assert.sameValue(method.call([],callback,undefined),undefined);
            var initial={};assert.sameValue(method.call(new Array(3),callback,initial),initial);
            assert.sameValue(count,0);
            var sparse=new Array(3);sparse[1]=undefined;
            assert.sameValue(method.call(sparse,callback),undefined);assert.sameValue(count,0);
            assert.sameValue(method.call(sparse,callback,initial),initial);assert.sameValue(count,1);
            var observed=[];method.call([undefined,undefined],function(a,v,k){
                assert.sameValue(a,undefined);assert.sameValue(v,undefined);observed.push(k);return undefined;
            });assert.sameValue(observed.join(','),right?'0':'1');
            count=0;assert.sameValue(method.call({length:0},callback,null),null);assert.sameValue(count,0);
            var fn=function(){};assert.sameValue(method.call([],callback,fn),fn);
        "#,
        );
    }

    #[test]
    fn array_reduce_captures_tolength_and_preserves_primitive_boxing() {
        reduction_modes(
            r#"
            var initial={},calls=0;function cb(a){calls++;return a;}
            var emptyLengths=[undefined,null,false,NaN,-1,-Infinity,'not a number',-0.5];
            for(var i=0;i<emptyLengths.length;i++)assert.sameValue(method.call({length:emptyLengths[i]},cb,initial),initial);
            assert.sameValue(calls,0);
            for(var i=0;i<2;i++){
                var primitive=i===0?false:42;assert.sameValue(method.call(primitive,cb,initial),initial);
                assert.throws(TypeError,function(){method.call(primitive,cb);});
            }
            var object={0:1,1:2,2:100,length:{valueOf(){return '2.9';}}};
            assert.sameValue(method.call(object,function(a,v){return a+v;},0),3);
            var boxed,units=[],indices=[];
            method.call('\uD800A\uDC00',function(a,v,k,o){
                assert.sameValue(typeof o,'object');if(boxed===undefined)boxed=o;else assert.sameValue(o,boxed);
                assert.sameValue(v.length,1);units.push(v.charCodeAt(0));indices.push(k);return a;
            },0);
            assert.sameValue(units.join(','),right?'56320,65,55296':'55296,65,56320');
            assert.sameValue(indices.join(','),right?'2,1,0':'0,1,2');
        "#,
        );
    }

    #[test]
    fn array_reduce_inherited_non_enumerable_properties_and_generic_receivers() {
        reduction_modes(
            r#"
            var prototype={};Object.defineProperty(prototype,'1',{value:7,enumerable:false});
            var object=Object.create(prototype);object.length=3;
            assert.sameValue(method.call(object,function(){throw 'unexpected callback';}),7);
            object[0]=2;object[2]=5;var seen=[];
            assert.sameValue(method.call(object,function(a,v,k,o){assert.sameValue(o,object);seen.push(k);return a+v;},0),14);
            assert.sameValue(seen.join(','),right?'2,1,0':'0,1,2');
            var bare=Object.create(null);bare.length=1;bare[0]=9;
            assert.sameValue(method.call(bare,function(){throw 'callback';}),9);
            function indexed(a,b){}indexed[0]=3;indexed[1]=4;
            assert.sameValue(method.call(indexed,function(a,v){return a+v;},0),7);
            function argumentsCase(a,b){var object=arguments;return method.call(object,function(total,value,index,receiver){
                assert.sameValue(receiver,object);return total+value;
            },0);}assert.sameValue(argumentsCase(2,3),5);
            var array=new Array(3);var p=Object.create(Array.prototype);p[1]=8;Object.setPrototypeOf(array,p);
            assert.sameValue(method.call(array,function(){throw 'callback';}),8);
        "#,
        );
    }

    #[test]
    fn array_reduce_getters_and_callbacks_observe_live_mutation_with_saved_length() {
        reduction_modes(
            r#"
            var first=right?2:0,last=right?0:2,reads=0,log='',prototype={};prototype[last]=9;
            var object={length:3,0:2,1:2,2:2};
            Object.defineProperty(object,''+first,{get:function(){
                assert.sameValue(this,object);reads++;object.length=1;object[1]=7;delete object[last];
                Object.setPrototypeOf(object,prototype);object[3]=100;return 1;
            },configurable:true});
            var result=method.call(object,function(a,v,k){log+=k;return a+v;},0);
            assert.sameValue(result,17);assert.sameValue(reads,1);assert.sameValue(log,right?'210':'012');
            object={length:4};object[right?3:0]=1;object[right?0:3]=100;
            var indices=[];result=method.call(object,function(a,v,k,o){
                indices.push(k);if(indices.length===1){o[right?2:1]=2;delete o[right?0:3];o.length=50;o[49]=1000;}
                return a+v;
            },0);assert.sameValue(result,3);assert.sameValue(indices.join(','),right?'3,2':'0,1');
            var array=[1,2,3];result=method.call(array,function(a,v,k){array.length=0;return a+v;},0);
            assert.sameValue(result,right?3:1);assert.sameValue(array.length,0);
        "#,
        );
    }

    #[test]
    fn array_reduce_callback_arguments_receivers_and_results_use_actual_callee_rules() {
        for name in ["reduce", "reduceRight"] {
            let (mut runtime, mut document) = upstream_harness();
            runtime.execute(r#"
                var count=0,receiver={},captured;
                function loose(a,v,k,o){assert.sameValue(this,window);assert.sameValue(arguments.length,4);count++;return a+v;}
                function tight(a,v,k,o){'use strict';assert.sameValue(this,undefined);assert.sameValue(arguments.length,4);count++;return a+v;}
                function make(){'use strict';return (a,v)=>{assert.sameValue(this,receiver);return a+v;};}
                var arrow=make.call(receiver);
            "#,&mut document).unwrap();
            runtime.execute_strict(&format!(r#"
                var method=Array.prototype.{name};assert.sameValue(method.call([1,2],loose,0),3);
                assert.sameValue(method.call([1,2],tight,0),3);assert.sameValue(count,4);
                assert.sameValue(method.call([1,2],arrow,0),3);
                var bound=function(prefix,a,v,k,o){{assert.sameValue(this,receiver);assert.sameValue(prefix,7);assert.sameValue(arguments.length,5);return a+v;}}.bind(receiver,7);
                assert.sameValue(method.call([1,2],bound,0),3);
                var poison={{toString(){{throw 'coercion';}},valueOf(){{throw 'coercion';}}}};
                assert.sameValue(method.call([1,2],function(a,v){{return poison;}},poison),poison);
            "#),&mut document).unwrap();
        }
    }

    #[test]
    fn array_reduce_abrupt_completion_preserves_prior_effects_and_reentry() {
        reduction_modes(
            r#"
            var reason={},seen,calls=0,first=right?1:0,object={length:2};
            Object.defineProperty(object,''+first,{get:function(){this.changed=1;throw reason;}});
            try{method.call(object,function(){calls++;},0);}catch(error){seen=error;}
            assert.sameValue(seen,reason);assert.sameValue(object.changed,1);assert.sameValue(calls,0);
            object={length:2,0:1,1:2};seen=undefined;
            try{method.call(object,function(a,v,k,o){calls++;o.changed=2;throw reason;},0);}catch(error){seen=error;}
            assert.sameValue(seen,reason);assert.sameValue(object.changed,2);assert.sameValue(calls,1);
            var nested=false;assert.sameValue(method.call([1,2],function(a,v){
                if(!nested){nested=true;assert.sameValue(method.call([3,4],function(x,y){return x+y;},0),7);}
                return a+v;
            },0),3);
            var text;try{method.call([1],function(){throw '\uD800';},0);}catch(error){text=error;}
            assert.sameValue(text.charCodeAt(0),55296);
        "#,
        );
    }

    #[test]
    fn array_reduce_full_safe_integer_range_visits_bounded_live_indices() {
        reduction_modes(
            r#"
            var object={length:9007199254740991},reason={},log=[];
            var first=right?9007199254740990:0,last=right?9007199254740988:2;
            object[first]=1;object[last]=3;var seen;
            try{method.call(object,function(a,v,k,o){log.push(k);a.push(v);if(v===3)throw a;return a;},[]);}catch(error){seen=error;}
            assert.sameValue(seen.join(','),'1,3');
            assert.sameValue(log.join(','),right?'9007199254740990,9007199254740988':'0,2');
            object.length=Infinity;seen=undefined;
            try{method.call(object,function(){throw reason;},0);}catch(error){seen=error;}
            assert.sameValue(seen,reason);
        "#,
        );
    }

    #[test]
    fn array_reduce_host_boundaries_are_checked_only_when_reached() {
        for name in ["reduce", "reduceRight"] {
            for source in [
                format!("Array.prototype.{name}.call(window,function(){{}},0);"),
                format!(
                    "var o=Object.create(window);o.length=1;Array.prototype.{name}.call(o,function(){{}},0);"
                ),
            ] {
                assert!(run(&source).unwrap_err().is_unsupported(), "{source}");
            }
            assert_eq!(run(&format!("var o=Object.create(window);o.length=1;o[0]=7;Array.prototype.{name}.call(o,function(){{throw 'unexpected';}});" )).unwrap(),Value::Number(7.0));
            assert!(run(&format!("var o={{length:1,0:1}};Array.prototype.{name}.call(o,function(){{}});Object.defineProperty([], '0', {{get:function(){{return 1;}}}});")).is_ok());
        }
    }

    #[test]
    fn array_reduce_resources_are_shared_uncatchable_and_guard_recursive_callbacks() {
        for name in ["reduce", "reduceRight"] {
            for source in [
                format!(
                    "try{{Array.prototype.{name}.call({{length:9007199254740991}},function(){{}},0);}}catch(e){{caught=true;}}"
                ),
                format!(
                    "function f(){{return [1].{name}(f,0);}}try{{f();}}catch(e){{caught=true;}}"
                ),
                format!(
                    "try{{[1].{name}(function(){{while(true){{}}}},0);}}catch(e){{caught=true;}}"
                ),
                format!(
                    "var o={{length:1,get 0(){{return Array.prototype.{name}.call(o,function(){{}},0);}}}};try{{Array.prototype.{name}.call(o,function(){{}},0);}}catch(e){{caught=true;}}"
                ),
            ] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                runtime.execute("var caught=false;", &mut document).unwrap();
                assert!(
                    runtime
                        .execute(&source, &mut document)
                        .unwrap_err()
                        .is_resource_limit(),
                    "{source}"
                );
                assert_eq!(runtime.calls, 0);
                assert_eq!(runtime.stack_units, 0);
                assert_eq!(
                    runtime.environments[0].bindings["caught"].value,
                    Value::Bool(false)
                );
            }
        }
    }

    #[test]
    fn array_reduce_private_preflight_retains_getter_effect_before_callback_failure() {
        fn setup() -> (Runtime, Document, Value, Value) {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            runtime.execute("var marker=0,called=0,object={length:1,get 0(){marker=1;return 2;}};function callback(){called=1;return 3;}",&mut document).unwrap();
            let object = runtime.environments[0].bindings["object"].value.clone();
            let callback = runtime.environments[0].bindings["callback"].value.clone();
            (runtime, document, object, callback)
        }
        let (mut measured, mut document, object, _) = setup();
        let before = measured.allocated;
        measured.charge(64).unwrap();
        measured
            .reduce_get(&object, &"length".into(), &mut document)
            .unwrap();
        let key = measured.reduce_index_key(0).unwrap();
        measured.reduce_property(&object, &key).unwrap();
        measured.reduce_get(&object, &key, &mut document).unwrap();
        let before_callback = measured.allocated - before;
        for direction in [ReduceDirection::Left, ReduceDirection::Right] {
            let (mut runtime, mut document, object, callback) = setup();
            runtime.allocated =
                MAX_HEAP - before_callback - (32 + 4 * std::mem::size_of::<Value>() - 1);
            let error = runtime
                .array_reduce(
                    object,
                    callback,
                    Some(Value::Number(0.0)),
                    direction,
                    &mut document,
                )
                .unwrap_err();
            assert!(error.is_resource_limit());
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
            assert_eq!(
                runtime.environments[0].bindings["marker"].value,
                Value::Number(1.0)
            );
            assert_eq!(
                runtime.environments[0].bindings["called"].value,
                Value::Number(0.0)
            );
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
        }
        let mut runtime = Runtime::new();
        runtime.steps = 1;
        let before = runtime.allocated;
        assert!(runtime.reduce_index_key(0).unwrap_err().is_resource_limit());
        assert_eq!(runtime.allocated, before);
        runtime.steps = 17;
        runtime.allocated = MAX_HEAP - 64;
        assert_eq!(
            runtime
                .reduce_index_key(9_007_199_254_740_990)
                .unwrap()
                .to_string(),
            "9007199254740990"
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(runtime.steps, 0);
        runtime.steps = 17;
        runtime.allocated = MAX_HEAP - 63;
        assert!(runtime.reduce_index_key(0).unwrap_err().is_resource_limit());
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
    }

    #[test]
    fn array_reduce_private_prototype_cycles_and_per_edge_scratch_are_bounded() {
        let mut runtime = Runtime::new();
        let a = runtime.object_ordered([]).unwrap();
        let b = runtime.object_ordered([]).unwrap();
        let (Value::Object(aid), Value::Object(bid)) = (a.clone(), b.clone()) else {
            panic!("objects")
        };
        runtime.objects[aid].prototype = Some(b);
        runtime.objects[bid].prototype = Some(a.clone());
        let before = runtime.allocated;
        let key = JsString::from("0");
        runtime.steps = MAX_STEPS;
        assert!(
            runtime
                .reduce_property(&a, &key)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated - before, MAX_DEPTH * 128);
        assert_eq!(MAX_STEPS - runtime.steps, MAX_DEPTH);
        runtime.allocated = MAX_HEAP - 127;
        runtime.steps = MAX_STEPS;
        assert!(
            runtime
                .reduce_property(&a, &key)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, MAX_HEAP + 1);
    }

    #[test]
    fn array_reduce_executes_unchanged_small_sort_stability_cases() {
        for source in [
            include_str!(
                "../tests/upstream/test262-array-sort/test/built-ins/Array/prototype/sort/stability-5-elements.js"
            ),
            include_str!(
                "../tests/upstream/test262-array-sort/test/built-ins/Array/prototype/sort/stability-11-elements.js"
            ),
        ] {
            for strict in [false, true] {
                let (mut runtime, mut document) = upstream_harness();
                if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                }
                .unwrap();
            }
        }
    }

    #[test]
    fn array_sort_metadata_aliases_and_comparator_validation_order() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var sort=Array.prototype.sort;
            verifyProperty(Array.prototype,'sort',{value:sort,writable:true,enumerable:false,configurable:true});
            verifyProperty(sort,'name',{value:'sort',writable:false,enumerable:false,configurable:true});
            verifyProperty(sort,'length',{value:1,writable:false,enumerable:false,configurable:true});
            assert.sameValue(Object.getPrototypeOf(sort),Function.prototype);assert.sameValue(sort.hasOwnProperty('prototype'),false);
            assert.throws(TypeError,function(){new sort();});
            var reads=0,coerced=0,o={get length(){reads++;throw 'length';}};
            var invalid=[null,0,'undefined',{},[],{toString(){coerced++;return 'function';},valueOf(){coerced++;return 1;}}];
            for(var i=0;i<invalid.length;i++){assert.throws(TypeError,function(){sort.call(o,invalid[i]);});}
            assert.sameValue(reads,0);assert.sameValue(coerced,0);
            assert.throws(TypeError,function(){sort.call(null);});assert.throws(TypeError,function(){sort.call(undefined);});
            var seen;try{sort.call(o,undefined);}catch(e){seen=e;}assert.sameValue(seen,'length');assert.sameValue(reads,1);
            var a=[2,1],extras=0;
            assert.sameValue(sort.call(a,undefined,{toString(){throw 'extra';}},extras++),a);
            assert.sameValue(extras,1);assert.sameValue(a.join(','),'1,2');
            Array.prototype.sort=function(){return 'changed';};assert.sameValue(a.sort(),'changed');
            a=[3,1,2];assert.sameValue(sort.apply(a,[]),a);assert.sameValue(a.join(','),'1,2,3');
            delete Array.prototype.sort;a=[2,1];assert.sameValue(sort.bind(a)(),a);assert.sameValue(a.join(','),'1,2');
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_length_conversion_boxing_and_readonly_single_item_copy_back() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var sort=Array.prototype.sort,log='';
            var o={get length(){log+='l';return {valueOf(){log+='n';return 3.9;}};},set length(v){throw 'length write';},0:3,1:1,2:2,3:'outside'};
            assert.sameValue(sort.call(o),o);assert.sameValue(log,'ln');
            assert.sameValue(o[0],1);assert.sameValue(o[1],2);assert.sameValue(o[2],3);assert.sameValue(o[3],'outside');
            var lengths=[undefined,NaN,-1,-Infinity,'bad',0];
            for(var i=0;i<lengths.length;i++){var empty={length:lengths[i],0:'unchanged'};sort.call(empty);assert.sameValue(empty[0],'unchanged');}
            var text={length:'2',0:'b',1:'a'};sort.call(text);assert.sameValue(text[0],'a');
            assert.sameValue(sort.call('').valueOf(),'');assert.sameValue(sort.call(2).valueOf(),2);assert.sameValue(sort.call(false).valueOf(),false);
            assert.throws(TypeError,function(){sort.call('a');});assert.throws(TypeError,function(){sort.call('ab');});
            log='';var one={length:1,get 0(){log+='g';return 7;},set 0(v){log+='s'+v;}};
            sort.call(one,function(){throw 'one comparison';});assert.sameValue(log,'gs7');
            var reason={},seen;try{sort.call({get length(){throw reason;}});}catch(e){seen=e;}assert.sameValue(seen,reason);
            seen=undefined;try{sort.call({length:{valueOf(){throw reason;}}});}catch(e){seen=e;}assert.sameValue(seen,reason);
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_default_utf16_order_and_stable_comparators() {
        for strict in [false, true] {
            let (mut runtime, mut document) = property_harness();
            let source = r#"
                var a=[10,2,1,-1,NaN,Infinity,null,false,true];assert.sameValue(a.sort(),a);
                assert.sameValue(a.map(function(value){return String(value);}).join(','),'-1,1,10,2,Infinity,NaN,false,null,true');
                var unicode=['\uE000','\uD800\uDC00','\uD800','\uDC00','a','','\u0000'];unicode.sort();
                assert.compareArray(unicode,['','\u0000','a','\uD800','\uD800\uDC00','\uDC00','\uE000']);
                var records=[];for(var i=0;i<37;i++)records.push({rank:i%4,id:i});
                records.sort(function(a,b){return a.rank-b.rank;});
                for(var j=1;j<records.length;j++){
                    assert.sameValue(records[j-1].rank<=records[j].rank,true);
                    if(records[j-1].rank===records[j].rank)assert.sameValue(records[j-1].id<records[j].id,true);
                }
                var zero=[{id:1},{id:2},{id:3},{id:4},{id:5}];
                zero.sort(function(){return NaN;});assert.sameValue(zero.map(function(v){return v.id;}).join(','),'1,2,3,4,5');
                zero.sort(function(){return -0;});assert.sameValue(zero.map(function(v){return v.id;}).join(','),'1,2,3,4,5');
                zero.sort(function(){return null;});assert.sameValue(zero.map(function(v){return v.id;}).join(','),'1,2,3,4,5');
                var equal=[{id:1,toString(){return 'x';}},{id:2,toString(){return 'x';}},{id:3,toString(){return 'x';}}];
                equal.sort();assert.sameValue(equal.map(function(v){return v.id;}).join(','),'1,2,3');
            "#;
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn array_sort_distinguishes_undefined_holes_and_inherited_indices() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var sort=Array.prototype.sort,a=[,undefined,3,,1,undefined],calls=0;
            a.sort(function(x,y){calls++;assert.notSameValue(x,undefined);assert.notSameValue(y,undefined);return x-y;});
            assert.sameValue(calls>0,true);assert.sameValue(a.length,6);
            assert.sameValue(a[0],1);assert.sameValue(a[1],3);assert.sameValue(a[2],undefined);assert.sameValue(a[3],undefined);
            assert.sameValue(a.hasOwnProperty('2'),true);assert.sameValue(a.hasOwnProperty('3'),true);
            assert.sameValue(a.hasOwnProperty('4'),false);assert.sameValue(a.hasOwnProperty('5'),false);
            var p={1:2},o=Object.create(p);o.length=3;o[0]=3;o[2]=1;
            sort.call(o,function(a,b){return a-b;});assert.sameValue(o[0],1);assert.sameValue(o[1],2);assert.sameValue(o[2],3);
            assert.sameValue(o.hasOwnProperty('1'),true);assert.sameValue(p[1],2);
            var inherited={2:undefined},tail=Object.create(inherited);tail.length=3;tail[0]=1;
            sort.call(tail);assert.sameValue(tail.hasOwnProperty('1'),true);assert.sameValue(tail.hasOwnProperty('2'),false);
            assert.sameValue(2 in tail,true);assert.sameValue(tail[2],undefined);
            var allHoles=[];allHoles.length=4;sort.call(allHoles,function(){throw 'holes comparison';});
            assert.sameValue(allHoles.length,4);assert.sameValue(Object.keys(allHoles).length,0);
            var hidden={length:2,1:1};Object.defineProperty(hidden,'0',{value:2,writable:true,enumerable:false,configurable:true});
            sort.call(hidden);assert.sameValue(hidden[0],1);assert.sameValue(hidden[1],2);
            assert.sameValue(Object.getOwnPropertyDescriptor(hidden,'0').enumerable,false);
            var received,counter=0,setterOnly={length:2,set 0(v){received=v;},1:1};
            sort.call(setterOnly,function(){counter++;throw 'undefined callback';});
            assert.sameValue(counter,0);assert.sameValue(received,1);assert.sameValue(setterOnly[1],undefined);
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_collects_live_properties_before_comparing_or_writing() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var sort=Array.prototype.sort,reads='',writes='',comparisons=0;
            var o={length:4,get 0(){reads+='0';return 3;},set 0(v){writes+='0='+v+';';},get 1(){reads+='1';delete this[2];Object.defineProperty(this,'3',{get:function(){reads+='3';return 4;},configurable:true});return 1;},set 1(v){writes+='1='+v+';';},2:2};
            sort.call(o,function(a,b){comparisons++;assert.sameValue(reads,'013');assert.sameValue(writes,'');return a-b;});
            assert.sameValue(comparisons>0,true);assert.sameValue(writes,'0=1;1=3;');assert.sameValue(o[2],4);
            assert.sameValue(o.hasOwnProperty('3'),false);assert.sameValue(reads,'013');
            var proto={1:9},changing=Object.create(proto);changing.length=2;
            Object.defineProperty(changing,'0',{get:function(){delete proto[1];return 1;},set:function(v){writes=String(v);},configurable:true});
            sort.call(changing);assert.sameValue(writes,'1');assert.sameValue(changing.hasOwnProperty('1'),false);
            var reason={},seen,log='',broken={length:3,get 0(){log+='0';return 3;},set 0(v){log+='write';},get 1(){log+='1';throw reason;},get 2(){log+='2';return 1;}};
            try{sort.call(broken,function(){log+='compare';});}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(log,'01');
            var shrinking={length:3,get 0(){this.length=1;this[2]=1;return 2;},set 0(v){writes=String(v);},1:3};
            sort.call(shrinking);assert.sameValue(shrinking.length,1);assert.sameValue(writes,'1');assert.sameValue(shrinking[2],3);
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_default_conversion_is_live_ordered_and_fallible() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var log='',left={toString(){log+='L';return 'b';},valueOf(){throw 'wrong hint';}},right={toString(){log+='R';return 'a';}};
            var a=[left,right];a.sort();assert.sameValue(log,'LR');assert.sameValue(a[0],right);assert.sameValue(a[1],left);
            log='';var reason={},seen,stop={toString(){log+='L';throw reason;}},never={toString(){log+='R';return 'x';}};
            a=[stop,never];try{a.sort();}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(log,'L');assert.sameValue(a[0],stop);
            log='';var fallback={toString(){log+='s';return {};},valueOf(){log+='v';return 'a';}},other={toString(){log+='o';return 'b';}};
            a=[fallback,other];a.sort();assert.sameValue(log,'svo');
            var invalid={toString(){return {};},valueOf(){return {};}};assert.throws(TypeError,function(){[invalid,'x'].sort();});
            var array=[],fn=function(){};array.toString=function(){return 'z';};fn.toString=function(){return 'a';};
            a=[array,fn];a.sort();assert.sameValue(a[0],fn);assert.sameValue(a[1],array);
            var conversions=0,shared={toString(){conversions++;return 'same';}};
            a=[shared,shared,shared,shared];a.sort();assert.sameValue(conversions>4,true);
            var target=[{toString(){target[2]='author';return 'c';}},{toString(){return 'a';}},{toString(){return 'b';}}];
            var original=target[2];target.sort();assert.sameValue(target[1],original);
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_comparator_receivers_results_and_abrupt_effects_are_preserved() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var calls=0,a=[3,1,2];a.sort(function(x,y){assert.sameValue(this,window);assert.sameValue(arguments.length,2);calls++;return x-y;});
            assert.sameValue(calls>0,true);assert.sameValue(a.join(','),'1,2,3');
            a=[3,1,2];a.sort(function(x,y){'use strict';assert.sameValue(this,undefined);return x-y;});assert.sameValue(a.join(','),'1,2,3');
            var context={sign:-1};a.sort(function(x,y){return this.sign*(x-y);}.bind(context));assert.sameValue(a.join(','),'3,2,1');
            function arrow(){var receiver=this;return (x,y)=>{assert.sameValue(this,receiver);return x-y;};}
            a.sort(arrow.call(context));assert.sameValue(a.join(','),'1,2,3');
            var conversions=0;a=[3,2,1];a.sort(function(x,y){return {valueOf(){conversions++;return String(x-y);},toString(){throw 'wrong number hint';}};});
            assert.sameValue(conversions>0,true);assert.sameValue(a.join(','),'1,2,3');
            var reason={},seen,attempts=0;a=[3,2,1];
            try{a.sort(function(){attempts++;a[0]=9;throw reason;});}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(attempts,1);assert.sameValue(a.join(','),'9,2,1');
            attempts=0;seen=undefined;a=[3,2,1];
            try{a.sort(function(){attempts++;return {valueOf(){a[0]=8;throw reason;}};});}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(attempts,1);assert.sameValue(a.join(','),'8,2,1');
            var log='',o={length:2,get 0(){return 2;},set 0(v){log+='set0';},1:1};
            try{Array.prototype.sort.call(o,function(){throw reason;});}catch(e){seen=e;}
            assert.sameValue(log,'');assert.sameValue(o[1],1);
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_mutating_length_and_reentrant_sort_use_the_collected_snapshot() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var a=[3,1,2],first=true;
            a.sort(function(x,y){if(first){first=false;a.length=0;}return x-y;});
            assert.sameValue(a.join(','),'1,2,3');assert.sameValue(a.length,3);
            var sparse=[3,,1,,];sparse.length=5;first=true;
            sparse.sort(function(x,y){if(first){first=false;sparse.length=0;}return x-y;});
            assert.sameValue(sparse.join(','),'1,3');assert.sameValue(sparse.length,2);
            a=[3,1,2];first=true;a.sort(function(x,y){if(first){first=false;a[8]='outside';}return x-y;});
            assert.sameValue(a.length,9);assert.sameValue(a[8],'outside');assert.sameValue(a[0],1);
            var nested=[3,2,1],entered=false;
            nested.sort(function(x,y){if(!entered){entered=true;nested.sort(function(a,b){return b-a;});}return x-y;});
            assert.sameValue(nested.join(','),'1,2,3');
            var values=[3,2,1];first=true;values.sort(function(x,y){if(first){first=false;values[0]='changed';delete values[1];values.push(0);}return x-y;});
            assert.sameValue(values.join(','),'1,2,3,0');
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_strict_copy_back_and_deletion_keep_partial_effects() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var sort=Array.prototype.sort,log='',o={length:3,get 0(){return 3;},set 0(v){log+='0='+v+';';},get 1(){return 1;},set 1(v){log+='1='+v+';';},get 2(){return 2;},set 2(v){log+='2='+v+';';}};
            sort.call(o);assert.sameValue(log,'0=1;1=2;2=3;');
            var locked={length:3,0:3,1:2,2:1};Object.defineProperty(locked,'1',{writable:false});
            assert.throws(TypeError,function(){sort.call(locked);});
            assert.sameValue(locked[0],1);assert.sameValue(locked[1],2);assert.sameValue(locked[2],1);
            var reason={},seen,setter={length:3,0:3,get 1(){return 2;},set 1(v){throw reason;},2:1};
            try{sort.call(setter);}catch(e){seen=e;}assert.sameValue(seen,reason);assert.sameValue(setter[0],1);assert.sameValue(setter[2],1);
            var fixedTail={length:4,1:2};Object.defineProperty(fixedTail,'3',{value:1,writable:true,configurable:false});
            assert.throws(TypeError,function(){sort.call(fixedTail);});
            assert.sameValue(fixedTail[0],1);assert.sameValue(fixedTail[1],2);assert.sameValue(fixedTail[3],1);
            var nonextensible=Object.preventExtensions({length:3,1:2,2:1});
            assert.throws(TypeError,function(){sort.call(nonextensible);});assert.sameValue(nonextensible[1],2);assert.sameValue(nonextensible[2],1);
            var proto={set 0(v){log=String(v);}},child=Object.create(proto);child.length=2;child[1]=1;
            sort.call(child);assert.sameValue(log,'1');assert.sameValue(child.hasOwnProperty('0'),false);assert.sameValue(child[1],undefined);
            var deletion={length:5,2:'b',4:'a'};
            Object.defineProperty(deletion,'0',{set:function(v){this[2]='temporary';},configurable:true});
            Object.defineProperty(deletion,'4',{configurable:false});
            assert.throws(TypeError,function(){sort.call(deletion);});
            assert.sameValue(deletion[1],'b');assert.sameValue(deletion[2],undefined);
            assert.sameValue(deletion.hasOwnProperty('3'),false);assert.sameValue(deletion[4],'a');
        "#,&mut document).unwrap();
    }

    #[test]
    fn array_sort_generic_arguments_functions_and_explicit_exotic_limits() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var sort=Array.prototype.sort;
            function mapped(a,b){sort.call(arguments);return a+'|'+b;}
            function strict(a,b){'use strict';sort.call(arguments);return a+'|'+b+'|'+arguments[0];}
            function defaults(a=2,b=1){sort.call(arguments);return a+'|'+b+'|'+arguments[0];}
            assert.sameValue(mapped(2,1),'1|2');assert.sameValue(strict(2,1),'2|1|1');assert.sameValue(defaults(2,1),'2|1|1');
            function object(a,b,c){}object[0]=3;object[1]=1;object[2]=2;
            assert.sameValue(sort.call(object),object);assert.sameValue(object[0],1);assert.sameValue(object[1],2);assert.sameValue(object[2],3);
            assert.sameValue(object.length,3);
            var empty={};assert.sameValue(sort.call(empty),empty);assert.sameValue(Object.keys(empty).length,0);
        "#,&mut document).unwrap();
        for source in [
            "Array.prototype.sort.call(window)",
            "Array.prototype.sort.call(document)",
        ] {
            assert!(run(source).unwrap_err().is_unsupported(), "{source}");
        }
    }

    #[test]
    fn array_sort_work_heap_callbacks_and_collection_limits_remain_uncatchable() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let error=runtime.execute("var log='',caught=false,o={get length(){log+='l';return {valueOf(){log+='n';return Infinity;}};},get 0(){log+='index';return 1;}};try{Array.prototype.sort.call(o);}catch(e){caught=true;}",&mut document).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(
            runtime
                .execute("log+'|'+caught", &mut document)
                .unwrap()
                .to_string(),
            "ln|false"
        );
        for source in [
            "var a=[2,1];function comparator(){return a.sort(comparator);}try{a.sort(comparator);}catch(e){throw 'caught';}",
            "var o={length:2,get 0(){Array.prototype.sort.call(o);return 1;}};try{Array.prototype.sort.call(o);}catch(e){throw 'caught';}",
            "var a=[{toString(){a.sort();return 'a';}},'b'];try{a.sort();}catch(e){throw 'caught';}",
            "var a=[2,1];try{a.sort(function(){while(true){};});}catch(e){throw 'caught';}",
            "var o={length:65536};try{Array.prototype.sort.call(o);}catch(e){throw 'caught';}",
            "var a=[4,3,2,1];try{for(var i=0;i<10000;i++)a.sort();}catch(e){throw 'caught';}",
        ] {
            assert!(run(source).unwrap_err().is_resource_limit(), "{source}");
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let text = Value::String(JsString::from("x".repeat(MAX_STRING)));
        let items = vec![text.clone(), text.clone(), text.clone(), text];
        assert!(
            runtime
                .sort_order(&items, &Value::Undefined, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let object = runtime
            .object_ordered([("length".into(), Value::Number(0.0))])
            .unwrap();
        runtime.steps = 10;
        assert_eq!(
            runtime
                .native_call(
                    &Native {
                        properties: None,
                        name: "Array.sort".into(),
                        receiver: object.clone()
                    },
                    vec![
                        Value::Undefined,
                        Value::String(JsString::from("x".repeat(MAX_STRING)))
                    ],
                    &mut document
                )
                .unwrap(),
            object
        );
        runtime.allocated = MAX_HEAP;
        runtime.steps = 10;
        assert_eq!(
            runtime
                .array_sort(Value::Number(1.0), Value::Number(0.0), &mut document)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert!(
            runtime
                .array_sort(Value::Number(1.0), Value::Undefined, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn array_sort_private_scratch_failures_preserve_receiver_and_prior_author_effects() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime.execute("var touched=0,o={length:2,get 0(){touched++;return 2;},set 0(v){touched+=10;},1:1};",&mut document).unwrap();
        let object = runtime.lookup(1, "o").unwrap().1;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .array_sort(object.clone(), Value::Undefined, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "touched").unwrap().1, Value::Number(0.0));
        assert!(matches!(
            runtime.own_property(&object, &"1".into()).unwrap().value,
            PropertyValue::Data {
                value: Value::Number(1.0),
                ..
            }
        ));
        let items = vec![Value::Number(2.0), Value::Number(1.0)];
        assert!(
            runtime
                .sort_order(&items, &Value::Undefined, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(items, vec![Value::Number(2.0), Value::Number(1.0)]);
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let error = runtime.execute(
            "var reads=0,writes=0,o={length:2,get 0(){reads++;this[1]=9;return {toString(){while(true){}}};},set 0(v){writes++;},1:1};Array.prototype.sort.call(o);",
            &mut document,
        ).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(1.0));
        assert_eq!(runtime.lookup(1, "writes").unwrap().1, Value::Number(0.0));
        let object = runtime.lookup(1, "o").unwrap().1;
        assert!(matches!(
            runtime.own_property(&object, &"1".into()).unwrap().value,
            PropertyValue::Data {
                value: Value::Number(9.0),
                ..
            }
        ));
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime.execute("var a=[3,2,1];", &mut document).unwrap();
        let Value::Array(id) = runtime.lookup(1, "a").unwrap().1 else {
            panic!("array")
        };
        let original = runtime.arrays[id].clone();
        runtime.steps = 15;
        assert!(
            runtime
                .array_sort(Value::Array(id), Value::Undefined, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.arrays[id], original);
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime
            .execute(
                "var calls=0;function comparator(){calls++;return 0;}",
                &mut document,
            )
            .unwrap();
        let comparator = runtime.lookup(1, "comparator").unwrap().1;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .sort_compare(
                    &Value::Number(2.0),
                    &Value::Number(1.0),
                    &comparator,
                    &mut document
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(0.0));
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let object = runtime
            .object_ordered([("length".into(), Value::Number(2.0))])
            .unwrap();
        let id = runtime.property_object(&object).unwrap();
        runtime.objects[id].prototype = Some(object.clone());
        assert!(
            runtime
                .array_sort(object, Value::Undefined, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn array_sort_executes_unchanged_pinned_closure_case_in_both_modes() {
        let source = include_str!(
            "../tests/upstream/test262-functions/test/language/statements/function/S13.2.1_A5_T1.js"
        );
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap();
        }
    }

    #[test]
    fn array_reverse_preserves_holes_inheritance_and_generic_receiver_identity() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var a=[1,2,3,4];delete a[1];
            assert.sameValue(a.reverse(),a);assert.sameValue(a.length,4);
            assert.sameValue(a[0],4);assert.sameValue(a[1],3);
            assert.sameValue(a.hasOwnProperty('2'),false);assert.sameValue(a[3],1);
            var p={0:'inherited'},o=Object.create(p);o.length=3;o[2]='own';
            assert.sameValue(Array.prototype.reverse.call(o),o);
            assert.sameValue(o[0],'own');assert.sameValue(o[2],'inherited');
            assert.sameValue(o.hasOwnProperty('1'),false);assert.sameValue(p[0],'inherited');
            var sparse={length:5,1:undefined};Array.prototype.reverse.call(sparse);
            assert.sameValue(sparse.hasOwnProperty('1'),false);
            assert.sameValue(sparse.hasOwnProperty('3'),true);
            var log='',like={get length(){log+='length;';return {valueOf(){log+='number;';return 3.9;}};},0:'a',2:'b'};
            Array.prototype.reverse.call(like);assert.sameValue(log,'length;number;');
            assert.sameValue(like[0],'b');assert.sameValue(like[2],'a');
            var noLength={0:'kept'};assert.sameValue(Array.prototype.reverse.call(noLength),noLength);
            assert.sameValue(noLength[0],'kept');
            assert.sameValue(Array.prototype.reverse.call('x').valueOf(),'x');
            assert.sameValue(Array.prototype.reverse.call(2).valueOf(),2);
            function mapped(a,b){Array.prototype.reverse.call(arguments);return a+'|'+b;}
            function unmapped(a,b){'use strict';Array.prototype.reverse.call(arguments);return a+'|'+b+'|'+arguments[0];}
            assert.sameValue(mapped('a','b'),'b|a');assert.sameValue(unmapped('a','b'),'a|b|b');
            assert.throws(TypeError,function(){Array.prototype.reverse.call(null);});
            assert.throws(TypeError,function(){Array.prototype.reverse.call(undefined);});
            assert.throws(TypeError,function(){Array.prototype.reverse.call('ab');});
            verifyProperty(Array.prototype.reverse,'length',{value:0,writable:false,enumerable:false,configurable:true});
            verifyProperty(Array.prototype,'reverse',{writable:true,enumerable:false,configurable:true});
        "#, &mut document).unwrap();
    }

    #[test]
    fn array_reverse_observes_accessor_order_and_mutations_between_steps() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var log='',o={length:2,get 0(){log+='get0;';return 'a';},set 0(v){log+='set0='+v+';';},get 1(){log+='get1;';return 'b';},set 1(v){log+='set1='+v+';';}};
            Array.prototype.reverse.call(o);assert.sameValue(log,'get0;get1;set0=b;set1=a;');
            var changing={length:2,get 0(){delete this[1];return 'a';},1:'b'};
            Array.prototype.reverse.call(changing);
            assert.sameValue(changing.hasOwnProperty('0'),false);assert.sameValue(changing[1],'a');
            var inserted={length:2,get 0(){this[1]='new';return 'old';},set 0(v){log=v;}};
            Array.prototype.reverse.call(inserted);assert.sameValue(log,'new');assert.sameValue(inserted[1],'old');
            var proto={set 0(v){assert.sameValue(this,child);log=v;}},child=Object.create(proto);child.length=2;child[1]='upper';
            Array.prototype.reverse.call(child);assert.sameValue(log,'upper');
            assert.sameValue(child[1],undefined);assert.sameValue(child.hasOwnProperty('0'),false);
            var detached={length:4,0:1,3:4};
            Object.defineProperty(detached,'0',{get:function(){this.length=0;this[1]=2;return 1;},set:function(v){log=v;},configurable:true});
            Array.prototype.reverse.call(detached);assert.sameValue(detached[2],2);assert.sameValue(detached.hasOwnProperty('1'),false);
        "#, &mut document).unwrap();
    }

    #[test]
    fn array_reverse_strict_writes_and_deletes_preserve_abrupt_partial_effects() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var readOnly={length:2,0:'lower'};
            Object.defineProperty(readOnly,'1',{value:'upper',writable:false});
            assert.throws(TypeError,function(){Array.prototype.reverse.call(readOnly);});
            assert.sameValue(readOnly[0],'upper');assert.sameValue(readOnly[1],'upper');
            var upperOnly={length:2};Object.defineProperty(upperOnly,'1',{value:'kept',configurable:false});
            assert.throws(TypeError,function(){Array.prototype.reverse.call(upperOnly);});
            assert.sameValue(upperOnly[0],'kept');assert.sameValue(upperOnly[1],'kept');
            var lowerOnly={length:2};Object.defineProperty(lowerOnly,'0',{value:'kept',configurable:false});
            assert.throws(TypeError,function(){Array.prototype.reverse.call(lowerOnly);});
            assert.sameValue(lowerOnly.hasOwnProperty('1'),false);
            var sealed=Object.preventExtensions({length:2,0:'lost'});
            assert.throws(TypeError,function(){Array.prototype.reverse.call(sealed);});
            assert.sameValue(sealed.hasOwnProperty('0'),false);
            var reason={},seen,log='',getter={length:2,get 0(){throw reason;},get 1(){log+='wrong';}};
            try{Array.prototype.reverse.call(getter);}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(log,'');
            var setter={length:2,set 0(v){throw reason;},get 1(){log+='get;';return 1;},set 1(v){log+='wrong';}};
            try{Array.prototype.reverse.call(setter);}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(log,'get;');
        "#, &mut document).unwrap();
    }

    #[test]
    fn array_reverse_charges_lengths_pair_work_and_recursive_accessors() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let error=runtime.execute("var order='',caught=false,o={get length(){order+='get;';return {valueOf(){order+='number;';return Infinity;}};},0:'untouched'};try{Array.prototype.reverse.call(o);}catch(e){caught=true;}",&mut document).unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(
            runtime
                .execute("order+'|'+caught+'|'+o[0]", &mut document)
                .unwrap()
                .to_string(),
            "get;number;|false|untouched"
        );
        for source in [
            "var a={length:65536};try{for(var i=0;i<10;i++)Array.prototype.reverse.call(a);}catch(e){throw 'caught';}",
            "var a={length:2,get 0(){return Array.prototype.reverse.call(a);}};Array.prototype.reverse.call(a);",
        ] {
            assert!(run(source).unwrap_err().is_resource_limit(), "{source}");
        }
        assert_eq!(
            run("var o={length:-Infinity,0:1};Array.prototype.reverse.call(o);o[0]").unwrap(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn number_radix_formats_safe_integers_and_special_values_exactly() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            assert.sameValue((255).toString(16),'ff');assert.sameValue((-255).toString(16),'-ff');
            assert.sameValue((9007199254740991).toString(16),'1fffffffffffff');
            assert.sameValue((9007199254740991).toString(2),'11111111111111111111111111111111111111111111111111111');
            assert.sameValue((9007199254740991).toString(36),'2gosa7pa2gv');
            for(var radix=2;radix<=36;radix++){
                assert.sameValue(radix.toString(radix),'10');
                assert.sameValue((-radix).toString(radix),'-10');
                assert.sameValue((radix*radix+1).toString(radix),'101');
                assert.sameValue((-0).toString(radix),'0');
            }
            assert.sameValue(NaN.toString(16),'NaN');assert.sameValue(Infinity.toString(2),'Infinity');
            assert.sameValue((-Infinity).toString(36),'-Infinity');
            assert.sameValue((1.25).toString(10),'1.25');assert.sameValue((1e21).toString(),'1e+21');
            assert.sameValue((35).toString(36),'z');assert.sameValue((10).toString(11),'a');
            assert.sameValue((5).toString(2.9),'101');assert.sameValue((255).toString('16'),'ff');
            var boxed=new Number(15);boxed.valueOf=function(){throw 'wrong receiver coercion';};
            assert.sameValue(boxed.toString(16),'f');
            verifyProperty(Number.prototype.toString,'length',{value:1,writable:false,enumerable:false,configurable:true});
        "#, &mut document).unwrap();
    }

    #[test]
    fn number_radix_coercion_branding_errors_and_unsupported_ranges_are_explicit() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var log='',radix={get valueOf(){log+='get;';return function(){log+='value;';return {};};},toString(){log+='string;';return '16';}};
            assert.sameValue((255).toString(radix),'ff');assert.sameValue(log,'get;value;string;');
            log='';assert.throws(TypeError,function(){Number.prototype.toString.call({},radix);});assert.sameValue(log,'');
            var reason={},seen;try{(1).toString({valueOf(){throw reason;}});}catch(e){seen=e;}assert.sameValue(seen,reason);
            var invalid=[null,false,NaN,0,-0,1,-2,37,Infinity,-Infinity];
            for(var i=0;i<invalid.length;i++){
                assert.throws(RangeError,function(){return (1).toString(invalid[i]);});
                assert.throws(RangeError,function(){return NaN.toString(invalid[i]);});
                assert.throws(RangeError,function(){return Infinity.toString(invalid[i]);});
            }
            assert.throws(TypeError,function(){(1).toString({valueOf(){return {};},toString(){return {};}});});
        "#, &mut document).unwrap();
        for source in [
            "(0.5).toString(2)",
            "(9007199254740992).toString(16)",
            "(1e100).toString(36)",
        ] {
            assert!(run(source).unwrap_err().is_unsupported(), "{source}");
        }
        assert!(
            run("(1).toString({valueOf(){while(true){}}})")
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn function_parameter_copies_consume_work_and_heap_before_retention() {
        let params = (0..1000)
            .map(|i| format!("p{i}{}", "x".repeat(36)))
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            "var caught=false;try{{for(var i=0;i<1000;i++){{var o={{m({params}){{return 1;}}}};}}}}catch(e){{caught=true;}}"
        );
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        assert!(
            runtime
                .execute(&source, &mut document)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(0, "caught").unwrap().1, Value::Bool(false));
        assert!(runtime.functions.len() < 100);
        // Directly exhaust the allocation allowance while leaving work available.
        let program = Parser::program(&format!("({{m({params}){{}}}})")).unwrap();
        let Stmt::Expr(Expr::Object(entries)) = &program.body[0] else {
            panic!("method AST");
        };
        let ObjectEntry::Method(code) = &entries[0].1 else {
            panic!("method code");
        };
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP - 1024;
        let before = runtime.functions.len();
        assert!(
            runtime
                .function_value(&code::test_function(code), 1)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.functions.len(), before);
        let mut named = code.clone();
        named.params.clear();
        named.name = Some("n".repeat(8192));
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP - 1024;
        let before = runtime.functions.len();
        assert!(
            runtime
                .function_value(&code::test_function(&named), 1)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.functions.len(), before);
    }

    #[test]
    fn array_number_methods_execute_unchanged_json_ascii_case() {
        let source = include_str!(
            "../tests/upstream/test262/test/built-ins/JSON/stringify/value-string-escape-ascii.js"
        );
        for strict in [false, true] {
            let (mut runtime, mut document) = upstream_harness();
            if strict {
                runtime.execute_strict(source, &mut document).unwrap();
            } else {
                runtime.execute(source, &mut document).unwrap();
            }
        }
    }

    #[test]
    fn object_literals_evaluate_keys_coercions_and_values_in_source_order() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var log='';
            var key={get toString(){log+='get;';return function(){log+='string;';return 'x';};},valueOf(){throw 'wrong hint';}};
            function keyExpression(){log+='key;';return key;}
            function valueExpression(){log+='value;';return 4;}
            var o={ [keyExpression()]:valueExpression(), [(log+='next;', 'y')]:5 };
            assert.sameValue(log,'key;get;string;value;next;');
            assert.sameValue(o.x,4);assert.sameValue(o.y,5);
            var fallback={toString(){return {};},valueOf(){return 8;}};
            assert.sameValue({[fallback]:6}[8],6);
            var array=[1,2];array.toString=function(){return 'array key';};
            assert.sameValue({[array]:7}['array key'],7);
            var func=function(){};func.toString=function(){return 'function key';};
            assert.sameValue({[func]:8}['function key'],8);
            var primitive={[-0]:1,[NaN]:2,[Infinity]:3,[null]:4,[undefined]:5,[true]:6};
            assert.sameValue(primitive['0'],1);assert.sameValue(primitive.NaN,2);
            assert.sameValue(primitive.Infinity,3);assert.sameValue(primitive.null,4);
            assert.sameValue(primitive.undefined,5);assert.sameValue(primitive.true,6);
            var reason={},seen,after=0;
            try{({[{get toString(){throw reason;}}]:after++,[(after++, 'later')]:0});}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(after,0);
            assert.throws(TypeError,function(){return {[{toString(){return {};},valueOf(){return {};}}]:after++};});
            assert.sameValue(after,0);
            try{({[keyExpression()]:(function(){throw reason;})(),[(after++,'later')]:0});}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(after,0);
            var initial;for(initial={['x' in {x:1}]:9};false;){}
            assert.sameValue(initial.true,9);
            assert.sameValue({[(1,2)]:3}[2],3);
            var ordered={b:1,[2]:2,a:3,[1]:4,['b']:5};
            assert.compareArray(Object.keys(ordered),['1','2','b','a']);assert.sameValue(ordered.b,5);
        "#, &mut document).unwrap();
    }

    #[test]
    fn object_literals_methods_and_accessors_preserve_receivers_names_and_descriptors() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var name='method',slot='slot',source=3,seen;
            var o={ [name](a,b){return this;}, get [slot](){seen=this;return source;}, set [slot](value){seen=this;source=value;} };
            assert.sameValue(o.method(1,2),o);assert.sameValue(o.method.call(null),window);
            assert.sameValue(o.method.call(7).valueOf(),7);
            assert.sameValue(o.slot,3);assert.sameValue(seen,o);o.slot=9;assert.sameValue(source,9);
            var descriptor=Object.getOwnPropertyDescriptor(o,'slot');
            verifyProperty(o,'method',{value:o.method,writable:true,enumerable:true,configurable:true},{restore:true});
            verifyProperty(o.method,'name',{value:'method',writable:false,enumerable:false,configurable:true},{restore:true});
            verifyProperty(o.method,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});
            assert.sameValue(descriptor.get.name,'get slot');assert.sameValue(descriptor.get.length,0);
            assert.sameValue(descriptor.set.name,'set slot');assert.sameValue(descriptor.set.length,1);
            assert.sameValue(descriptor.enumerable,true);assert.sameValue(descriptor.configurable,true);
            assert.sameValue(Object.prototype.hasOwnProperty.call(o.method,'prototype'),false);
            assert.sameValue(Object.prototype.hasOwnProperty.call(descriptor.get,'prototype'),false);
            assert.throws(TypeError,()=>new o.method());assert.throws(TypeError,()=>new descriptor.get());
            var replacement={get [slot](){return 1;},set [slot](x){source=x;},get [slot](){return 2;}};
            replacement.slot=11;assert.sameValue(source,11);assert.sameValue(replacement.slot,2);
            var dataLast={get ['x'](){return 1;},['x']:2};
            verifyProperty(dataLast,'x',{value:2,writable:true,enumerable:true,configurable:true});
            var getterLast={['x']:2,get ['x'](){return 3;}};
            assert.sameValue(getterLast.x,3);assert.sameValue(Object.getOwnPropertyDescriptor(getterLast,'x').set,undefined);
            var strict={['method'](){'use strict';return this;},get ['x'](){'use strict';return this;}};
            assert.sameValue(strict.method.call(null),null);
            assert.sameValue(strict.method.call(8),8);
            assert.sameValue(Object.getOwnPropertyDescriptor(strict,'x').get.call(undefined),undefined);
            function inheritedStrict(){'use strict';return {['method'](){return this;}};}
            assert.sameValue(inheritedStrict().method.call(undefined),undefined);
            var inherited=Object.create(o);assert.sameValue(inherited.slot,11);assert.sameValue(seen,inherited);
            inherited.slot=12;assert.sameValue(seen,inherited);assert.sameValue(source,12);
            var shorthand=7;var ambiguous={get(){return 1;},set:2,async(){return 3;},shorthand};
            assert.sameValue(ambiguous.get(),1);assert.sameValue(ambiguous.set,2);
            assert.sameValue(ambiguous.async(),3);assert.sameValue(ambiguous.shorthand,7);
        "#, &mut document).unwrap();
    }

    #[test]
    fn object_literals_function_naming_preserves_utf16_without_renaming_references() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var key='\uD800',other='\uDC00';
            var o={ [key](){}, get [other](){return 1;}, set [other](v){}, ['']:function(){}, [42]:()=>0 };
            assert.sameValue(o[key].name,key);
            assert.sameValue(Object.getOwnPropertyDescriptor(o,other).get.name,'get '+other);
            assert.sameValue(Object.getOwnPropertyDescriptor(o,other).set.name,'set '+other);
            assert.sameValue(o[''].name,'');assert.sameValue(o[42].name,'42');
            assert.sameValue({'\uD800'(){}}[key].name,key);
            assert.sameValue({get '\uD800'(){}}[key],undefined);
            var accessor=Object.getOwnPropertyDescriptor({get '\uD800'(){}},key).get;
            assert.sameValue(accessor.name,'get '+key);
            var named=function declared(){};var o2={a:named,b:function explicit(){},c:function(){},d:()=>0,e:(function(){})};
            assert.sameValue(o2.a.name,'declared');assert.sameValue(o2.b.name,'explicit');
            assert.sameValue(o2.c.name,'c');assert.sameValue(o2.d.name,'d');assert.sameValue(o2.e.name,'e');
            assert.sameValue(named.name,'declared');
            var anonymous=[function(){}][0];var o3={x:anonymous,y:(0,function(){})};
            assert.sameValue(o3.x.name,'');assert.sameValue(o3.y.name,'');
            var existing=function old(){};var shorthand={existing};assert.sameValue(shorthand.existing.name,'old');
            var bareProto={__proto__:function(){}};assert.sameValue(Object.getPrototypeOf(bareProto).name,'');
            var namedProto={['__proto__']:function(){}};assert.sameValue(namedProto.__proto__.name,'__proto__');
        "#, &mut document).unwrap();
    }

    #[test]
    fn object_literals_prototype_setters_are_distinct_from_ordinary_proto_properties() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var base={inherited:4};var a={__proto__:base,own:5};
            assert.sameValue(Object.getPrototypeOf(a),base);assert.sameValue(a.inherited,4);
            assert.sameValue(Object.prototype.hasOwnProperty.call(a,'__proto__'),false);
            var n={'__pr\u006fto__':null,x:1};assert.sameValue(Object.getPrototypeOf(n),null);
            assert.sameValue(Object.prototype.hasOwnProperty.call(n,'__proto__'),false);
            var effects=0;var ignored={__proto__:(effects++,17)};
            assert.sameValue(effects,1);assert.sameValue(Object.getPrototypeOf(ignored),Object.prototype);
            assert.sameValue(Object.prototype.hasOwnProperty.call(ignored,'__proto__'),false);
            var coercions=0;var p={toString(){coercions++;return 'wrong';},valueOf(){coercions++;return null;}};
            assert.sameValue(Object.getPrototypeOf({__proto__:p}),p);assert.sameValue(coercions,0);
            var seen='',setter={set x(value){throw 'inherited setter invoked';}};
            var ordered={[(seen+='key;','x')]:(seen+='value;',1),__proto__:(seen+='proto;',setter),y:(seen+='last;',2)};
            assert.sameValue(seen,'key;value;proto;last;');assert.sameValue(ordered.x,1);
            assert.sameValue({__proto__:setter,x:3}.x,3);
            var __proto__=8;var ordinary={__proto__,['__proto__']:9};
            assert.sameValue(ordinary.__proto__,9);assert.sameValue(Object.getPrototypeOf(ordinary),Object.prototype);
            var combined={['__proto__']:7,__proto__:null};
            assert.sameValue(combined.__proto__,7);assert.sameValue(Object.getPrototypeOf(combined),null);
            var method={__proto__(){return 10;}};assert.sameValue(method.__proto__(),10);
            var access={get __proto__(){return 11;},set __proto__(v){effects=v;}};
            assert.sameValue(access.__proto__,11);access.__proto__=12;assert.sameValue(effects,12);
            var parsed=JSON.parse('{"__proto__":{"x":1}}');
            assert.sameValue(Object.getPrototypeOf(parsed),Object.prototype);
            assert.sameValue(Object.prototype.hasOwnProperty.call(parsed,'__proto__'),true);
            assert.sameValue(parsed.__proto__.x,1);
        "#, &mut document).unwrap();
    }

    #[test]
    fn object_literals_early_errors_and_unsupported_method_forms_are_explicit() {
        for source in [
            "({[1,2]:3})",
            "({[]:1})",
            "({[1]})",
            "({'x'})",
            "({1})",
            "({'get' x(){}})",
            "({'set' x(v){}})",
            "({get x(v){}})",
            "({set x(){}})",
            "({set x(a,b){}})",
            "({m(a,a){}})",
            "({['m'](a,a){}})",
            "({m(eval){'use strict';}})",
            "({__proto__:null,'__proto__':null})",
            "({__proto__:1,'__pr\\u006fto__':2})",
        ] {
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(error.is_parse_error(), "{source}: {error:?}");
        }
        Runtime::parse_only("({m(a=1){return a;}})").unwrap();
        for source in [
            "({*m(){}})",
            "({async m(){}})",
            "({async ['m'](){}})",
            "({async *m(){}})",
            "({...x})",
            "({m(){return super.x;}})",
            "({m({a}){}})",
        ] {
            assert!(
                Runtime::parse_only(source).unwrap_err().is_unsupported(),
                "{source}"
            );
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime.execute("var touched=false", &mut document).unwrap();
        let error = runtime
            .execute("touched=true;({__proto__:1,__proto__:2})", &mut document)
            .unwrap_err();
        assert!(error.is_parse_error());
        assert_eq!(
            runtime.execute("touched", &mut document).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn object_literals_charge_keys_names_prototype_walks_and_nested_coercion() {
        for source in [
            "var key={toString(){while(true){}}};try{({[key]:1});}catch(e){caught=true;}",
            "var key={toString(){return Object.keys({[key]:1})[0];}};try{({[key]:1});}catch(e){caught=true;}",
            "var proto=null;try{for(var i=0;i<200;i++){proto={__proto__:proto};}}catch(e){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            runtime.execute("var caught=false", &mut document).unwrap();
            let error = runtime.execute(source, &mut document).unwrap_err();
            assert!(error.is_resource_limit(), "{source}: {error:?}");
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(
                runtime.execute("caught", &mut document).unwrap(),
                Value::Bool(false)
            );
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime
            .execute(
                "var caught=false;var key='x';for(var i=0;i<15;i++){key=key+key;}",
                &mut document,
            )
            .unwrap();
        let error = runtime
            .execute(
                "try{for(var n=0;n<500;n++){({get [key](){}});}}catch(e){caught=true;}",
                &mut document,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(
            runtime.execute("caught", &mut document).unwrap(),
            Value::Bool(false)
        );
        let nested = format!("{}0{}", "({[".repeat(200), "]:1})".repeat(200));
        Runtime::parse_only(&nested).unwrap();
        assert_eq!(
            run(&format!("{nested}['[object Object]']")).unwrap(),
            Value::Number(1.0)
        );
        let large = format!("({{{}}})", "['x']:1,".repeat(MAX_TOKENS));
        assert!(Runtime::parse_only(&large).unwrap_err().is_resource_limit());
    }

    #[test]
    fn ordinary_descriptors_enforce_defaults_transitions_and_same_value() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var o={};Object.defineProperty(o,'x',{value:1});
            verifyProperty(o,'x',{value:1,writable:false,enumerable:false,configurable:false});
            o.x=2;assert.sameValue(o.x,1);assert.sameValue(delete o.x,false);
            assert.throws(TypeError,function(){Object.defineProperty(o,'x',{value:2});});
            assert.throws(TypeError,function(){Object.defineProperty(o,'x',{enumerable:true});});
            assert.throws(TypeError,function(){Object.defineProperty(o,'x',{get:function(){return 1;}});});
            Object.defineProperty(o,'nan',{value:NaN});Object.defineProperty(o,'nan',{value:NaN});
            Object.defineProperty(o,'zero',{value:-0});
            assert.throws(TypeError,function(){Object.defineProperty(o,'zero',{value:0});});
            Object.defineProperty(o,'change',{value:7,writable:true,enumerable:true,configurable:true});
            var getter=function(){return 8;};Object.defineProperty(o,'change',{get:getter});
            verifyProperty(o,'change',{get:getter,set:undefined,enumerable:true,configurable:true},{restore:true});
            Object.defineProperty(o,'change',{value:9});
            verifyProperty(o,'change',{value:9,writable:false,enumerable:true,configurable:true},{restore:true});
            Object.preventExtensions(o);o.extra=2;assert.sameValue(o.extra,undefined);
            assert.sameValue(Object.isExtensible(o),false);
            assert.throws(TypeError,function(){Object.defineProperty(o,'extra',{value:1});});
            Object.setPrototypeOf(o,Object.getPrototypeOf(o));
            assert.throws(TypeError,function(){Object.setPrototypeOf(o,null);});
            assert.sameValue(Object.getOwnPropertyDescriptor('x','0').writable,false);
            assert.sameValue(Object.getOwnPropertyDescriptor('x','0').configurable,false);
            assert.compareArray(Object.keys('x'),['0']);
        "#,&mut document).unwrap();
    }

    #[test]
    fn inherited_accessors_descriptor_conversion_and_definition_order_are_observable() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var p={};var hits=0;var get=function(){hits++;return this.saved;};
            var set=function(v){this.saved=v;};Object.defineProperty(p,'x',{get:get,set:set,enumerable:true});
            var c=Object.create(p);c.x=9;assert.sameValue(c.x,9);assert.sameValue(hits,1);
            assert.sameValue(c.hasOwnProperty('x'),false);assert.sameValue(c.saved,9);
            Object.getOwnPropertyDescriptor(p,'x');assert.sameValue(hits,1);
            Object.defineProperty(p,'locked',{value:3});c.locked=4;assert.sameValue(c.hasOwnProperty('locked'),false);
            var log='';var proto={};var desc=Object.create(proto);
            Object.defineProperty(proto,'enumerable',{get:function(){log+='e';return true;}});
            Object.defineProperty(proto,'configurable',{get:function(){log+='c';return true;}});
            Object.defineProperty(proto,'value',{get:function(){log+='v';return 12;}});
            Object.defineProperty(proto,'writable',{get:function(){log+='w';return true;}});
            Object.defineProperty(c,'data',desc);assert.sameValue(log,'ecvw');assert.sameValue(c.data,12);
            var target={};assert.throws(TypeError,function(){Object.defineProperties(target,{a:{value:1},b:{get:1}});});
            assert.sameValue(target.hasOwnProperty('a'),false);
            Object.defineProperties(target,{a:{value:1,enumerable:true},b:{value:2}});
            assert.compareArray(Object.keys(target),['a']);
            assert.throws(TypeError,function(){Object.defineProperty(target,'bad',{value:1,set:undefined});});
            var key={toString:function(){return 'coerced';}};target[key]=5;assert.sameValue(target.coerced,5);
        "#,&mut document).unwrap();
    }

    #[test]
    fn for_in_handles_prototypes_shadowing_deletions_and_per_iteration_bindings() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var p={shadow:1,inherited:2};var o=Object.create(p);o.b=1;o['2']=2;o.a=3;o['1']=1;
            Object.defineProperty(o,'shadow',{value:9});var keys=[];
            for(var key in o){keys.push(key);if(key==='1')delete o.a;}
            assert.compareArray(keys,['1','2','b','inherited']);
            var fns=[];for(let name in {x:1,y:2})fns.push(function(){return name;});
            assert.sameValue(fns[0](),'x');assert.sameValue(fns[1](),'y');assert.sameValue(typeof name,'undefined');
            var target={};var calls=0;function ref(){calls++;return target;}
            for(ref().key in {a:1,b:2}){}assert.sameValue(calls,2);assert.sameValue(target.key,'b');
            var count=0;for(const key in null)count++;for(var key in undefined)count++;
            assert.sameValue(count,0);for(const unit in 'ab')count++;assert.sameValue(count,2);
            var array=[undefined,2];delete array[0];assert.sameValue('0' in array,false);
            assert.sameValue('length' in array,true);assert.compareArray(Object.keys(array),['1']);
            array.length=4;assert.sameValue(array.hasOwnProperty('3'),false);array[3]=undefined;
            assert.sameValue(array.hasOwnProperty('3'),true);assert.sameValue(JSON.stringify(array),'[null,2,null,null]');
            array[4294967295]=1;assert.sameValue(array.length,4);
        "#,&mut document).unwrap();
        for source in [
            "for(var a,b in {}){}",
            "for(const a=1 in {}){}",
            "for(1 in {}){}",
        ] {
            assert!(Runtime::parse_only(source).unwrap_err().is_parse_error());
        }
    }

    #[test]
    fn unchanged_property_helper_verifies_intrinsics_and_rejects_false_descriptors() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            verifyCallableProperty(String.prototype,'charAt','charAt',1,undefined,{restore:true});
            verifyCallableProperty(String.prototype,'substring','substring',2,undefined,{restore:true});
            verifyCallableProperty(JSON,'parse','parse',2,undefined,{restore:true});
            verifyCallableProperty(JSON,'stringify','stringify',3,undefined,{restore:true});
            assert.sameValue('abc'.charAt(1),'b');assert.sameValue(JSON.parse('3'),3);
            function F(a,b){this.sum=a+b;}var Bound=F.bind(null,3);var item=new Bound(4);
            assert.sameValue(item.sum,7);assert.sameValue(item instanceof Bound,true);
            assert.sameValue(item instanceof F,true);assert.sameValue(Bound.length,1);
            assert.sameValue(Bound.name,'bound F');assert.sameValue(Bound.hasOwnProperty('prototype'),false);
            var receiver={x:4};function plus(a,b){return this.x+a+b;}
            var bound=plus.bind(receiver,2);assert.sameValue(bound.call({x:99},3),9);
        "#,&mut document).unwrap();
        for source in [
            "verifyProperty({x:1},'x',{writable:false})",
            "verifyProperty({x:1},'x',{enumerable:false})",
            "verifyProperty({x:1},'x',{configurable:false})",
            "verifyProperty({x:1},'x',{value:2})",
        ] {
            let error = runtime.execute(source, &mut document).unwrap_err();
            assert_eq!(error.name(), "Test262Error", "{source}: {error}");
        }
    }

    #[test]
    fn accessors_and_bound_chains_cannot_escape_resource_termination() {
        for source in [
            "let o={};Object.defineProperty(o,'x',{get:function(){return o.x;}});o.x;",
            "let o={};Object.defineProperty(o,'x',{set:function(v){o.x=v;}});o.x=1;",
            "let desc={};Object.defineProperty(desc,'value',{get:function(){while(true){}}});Object.defineProperty({},'x',desc);",
            "function f(){}let target=f;for(let i=0;i<100;i++)target=target.bind(null);target();",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let source = format!("let caught=false;try{{{source}}}catch(e){{caught=true;}}");
            assert!(
                runtime
                    .execute(&source, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(runtime.calls, 0);
        }
        assert!(run("Object.defineProperty([],'0',{get:function(){return 1;}})").is_ok());
        assert_eq!(
            run("Object.defineProperty([],'0',{value:1})[0]").unwrap(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn string_methods_coerce_receivers_and_positions_and_json_unboxes_primitives() {
        let (mut runtime, mut document) = upstream_harness();
        runtime.execute(r#"
            var receiver={toString:function(){return 'abc';}};
            var position={valueOf:function(){return 1;}};
            assert.sameValue(String.prototype.charAt.call(receiver,position),'b');
            assert.sameValue(String.prototype.slice.call(receiver,position),'bc');
            assert.sameValue(String.fromCharCode({valueOf:function(){return 65;}}),'A');
            assert.throws(TypeError,function(){String.prototype.charAt.call(null);});
            assert.throws(TypeError,function(){'a'.charAt({valueOf:function(){return {};},toString:function(){return {};}});});
            assert.sameValue(JSON.stringify([new Number(2),new String('x'),new Boolean(false)]),'[2,"x",false]');
        "#,&mut document).unwrap();
    }

    fn upstream_harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime
            .execute(
                include_str!("../tests/upstream/test262/harness/assert.js"),
                &mut document,
            )
            .unwrap();
        runtime
            .execute(
                include_str!("../tests/upstream/test262/harness/sta.js"),
                &mut document,
            )
            .unwrap();
        (runtime, document)
    }

    #[test]
    fn unchanged_test262_harness_accepts_and_rejects_assertions() {
        let (mut runtime, mut document) = upstream_harness();
        runtime.execute("assert(true);assert.sameValue(NaN,NaN);assert.sameValue(-0,-0);assert.notSameValue(0,-0);assert.compareArray([1,NaN,-0],[1,NaN,-0]);assert.throws(TypeError,function(){null.missing;});assert.throws(SyntaxError,function(){JSON.parse('{');});", &mut document).unwrap();
        for source in [
            "assert(false)",
            "assert.sameValue(0,-0)",
            "assert.notSameValue(NaN,NaN)",
            "assert.compareArray([1],[2])",
            "assert.throws(TypeError,function(){return 1;})",
            "assert.throws(TypeError,function(){throw new RangeError('wrong');})",
            "assert.throws(TypeError,function(){throw {name:'TypeError'};})",
        ] {
            let error = runtime.execute(source, &mut document).unwrap_err();
            assert_eq!(error.name(), "Test262Error", "{source}: {error}");
            assert!(!error.is_parse_error());
        }
    }

    #[test]
    fn switch_fallthrough_default_search_and_abrupt_control_are_ordered() {
        assert_eq!(run("let out='';switch(2){case 1:out+='a';break;default:out+='d';case 2:out+='b';case 3:out+='c';}out").unwrap().to_string(), "bc");
        assert_eq!(run("let out='';switch(9){case 1:out+='a';break;default:out+='d';case 2:out+='b';case 3:out+='c';}out").unwrap().to_string(), "dbc");
        assert_eq!(run("let hits=0;function probe(x){hits++;return x;}switch(2){case probe(2):break;case probe(3):break;}hits").unwrap(), Value::Number(1.0));
        assert_eq!(
            run("function f(){switch(1){case 1:try{return 3;}finally{return 4;}}}f()").unwrap(),
            Value::Number(4.0)
        );
        assert!(
            Runtime::parse_only("switch(1){default:;default:;}")
                .unwrap_err()
                .is_parse_error()
        );
    }

    #[test]
    fn constructors_own_properties_prototypes_and_error_identity_work() {
        let (mut runtime, mut document) = upstream_harness();
        runtime.execute("function Parent(x){this.x=x;}Parent.prototype.read=function(){return this.x;};Parent.extra=9;const item=new Parent(7);assert.sameValue(item.read(),7);assert.sameValue(item.constructor,Parent);assert.sameValue(item instanceof Parent,true);assert.sameValue(item instanceof Object,true);assert.sameValue(Parent.extra,9);assert.sameValue(Parent.name,'Parent');assert.sameValue(Parent.length,1);assert.sameValue(Object.getPrototypeOf(item),Parent.prototype);assert.sameValue(String(new TypeError('bad')),'TypeError: bad');assert.sameValue(new TypeError() instanceof Error,true);assert.sameValue(new TypeError().constructor,TypeError);function Alternate(){return {x:3};}assert.sameValue(new Alternate().x,3);function Primitive(){this.x=8;return 99;}assert.sameValue(new Primitive().x,8);assert.sameValue(Object.prototype.toString.call([1]),'[object Array]');", &mut document).unwrap();
    }

    #[test]
    fn prototype_cycles_strict_directives_and_constructor_quotas_are_explicit() {
        Runtime::parse_only("'use strict'; 1").unwrap();
        Runtime::parse_only("function f(){'use strict';return 1;}").unwrap();
        assert_eq!(run("const a={};const b=Object.create(a);let name;try{Object.setPrototypeOf(a,b);}catch(e){name=e.name;}name").unwrap().to_string(), "TypeError");
        assert!(
            run("function Recur(){return new Recur();}new Recur();")
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            run("let caught=false;try{new (()=>1);}catch(e){caught=e instanceof TypeError;}caught")
                .unwrap(),
            Value::Bool(true)
        );
    }
    #[test]
    fn strict_directives_are_raw_lexical_and_do_not_leak_between_programs() {
        for source in [
            "'use strict'; missing=1;",
            "'other'; 'use strict'; missing=1;",
            "function f(){'use strict';return function(){missing=1;};}f()();",
        ] {
            assert_eq!(
                run(source).unwrap_err().name(),
                "ReferenceError",
                "{source}"
            );
        }
        for source in [
            "('use strict'); missing=1;",
            "'use\\x20strict';missing=1;",
            "`use strict`;missing=1;",
            "0;'use strict';missing=1;",
            "{ 'use strict';missing=1; }",
        ] {
            assert_eq!(run(source).unwrap(), Value::Number(1.0), "{source}");
        }
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime
            .execute(
                "'use strict';function strict(){return this;}",
                &mut document,
            )
            .unwrap();
        assert_eq!(
            runtime.execute("loose=1;strict()", &mut document).unwrap(),
            Value::Undefined
        );
        assert_eq!(
            runtime.execute("this.loose", &mut document).unwrap(),
            Value::Number(1.0)
        );
        assert_eq!(
            runtime
                .execute("'use strict';this===globalThis", &mut document)
                .unwrap(),
            Value::Bool(true)
        );
    }
    #[test]
    fn strict_early_errors_validate_bindings_targets_and_legacy_literals() {
        for source in [
            "'use strict';var eval;",
            "function f(arguments){'use strict';}",
            "function eval(){'use strict';}",
            "function f(a,a){'use strict';}",
            "'use strict';function f(a,a){}",
            "'use strict';eval=1;",
            "'use strict';arguments++;",
            "'use strict';delete x;",
            "'use strict';with({}){}",
            "'use strict';var interface;",
            "'use strict';010;",
            "'use strict';08;",
            "'use strict';'\\1';",
            "'\\1';'use strict';",
            "'use strict';'\\8';",
            "(a,a)=>a;",
            "({method(a,a){}});",
            "let x;let x;",
            "let x;{var x;}",
            "function f(x){let x;}",
            "try{}catch(x){let x;}",
            "for(let x=0;x<1;x++){var x;}",
            "for(let x in {}){var x;}",
            "'use strict';if(true)function f(){}",
        ] {
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(error.is_parse_error(), "{source}: {error}");
        }
        for source in [
            "function f(a,a){}",
            "var eval;",
            "var interface;",
            "010;",
            "'\\1';",
            "'use strict';'\\0';",
            "'use strict';delete this;",
        ] {
            Runtime::parse_only(source).unwrap();
        }
        assert_eq!(run("010").unwrap(), Value::Number(8.0));
    }
    #[test]
    fn strict_receivers_and_property_failures_follow_the_callee_and_reference() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function strict(){'use strict';return this;}
            function loose(){return this;}
            assert.sameValue(strict(),undefined);
            assert.sameValue(strict.call(null),null);
            assert.sameValue(strict.call(7),7);
            assert.sameValue(loose.call(null),globalThis);
            assert.sameValue(typeof loose.call(7),'object');
            function inherited(){'use strict';return ()=>this;}
            assert.sameValue(inherited.call(9).call(2),9);
            var frozen={};Object.defineProperty(frozen,'x',{value:1});
            function writes(){'use strict';frozen.x=2;}
            function deletes(){'use strict';delete frozen.x;}
            assert.throws(TypeError,writes);assert.throws(TypeError,deletes);
            frozen.x=2;assert.sameValue(frozen.x,1);assert.sameValue(delete frozen.x,false);
            assert.throws(TypeError,function(){'use strict';(1).x=2;});
            assert.throws(TypeError,function(){'use strict';NaN=1;});
            NaN=1;assert.sameValue(NaN,NaN);
            var empty={};Object.preventExtensions(empty);
            assert.throws(TypeError,function(){'use strict';empty.x=1;});
            var accessor={set x(v){'use strict';assert.sameValue(this,accessor);this.saved=v;}};
            function assign(){'use strict';accessor.x=4;}assign();assert.sameValue(accessor.saved,4);
            stray=1;assert.sameValue(globalThis.stray,1);assert.sameValue(delete stray,true);
            assert.sameValue(typeof stray,'undefined');
        "#,&mut document).unwrap();
    }
    #[test]
    fn captured_globals_can_be_deleted_during_rhs_or_numeric_coercion() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            x=0;function remove(){delete globalThis.x;return 1;}
            assert.sameValue(x=remove(),1);assert.sameValue(x,1);
            x=2;assert.sameValue(x+=remove(),3);assert.sameValue(x,3);
            x={valueOf(){delete globalThis.x;return 4;}};
            assert.sameValue(x++,4);assert.sameValue(x,5);
            x={valueOf(){delete globalThis.x;return 7;}};
            assert.sameValue(--x,6);assert.sameValue(x,6);
            function strictAssign(){'use strict';x=remove();}
            function strictCompound(){'use strict';x+=remove();}
            function strictUpdate(){'use strict';x++;}
            x=0;assert.throws(ReferenceError,strictAssign);assert.sameValue(typeof x,'undefined');
            x=0;assert.throws(ReferenceError,strictCompound);assert.sameValue(typeof x,'undefined');
            x={valueOf(){delete globalThis.x;return 8;}};
            assert.throws(ReferenceError,strictUpdate);assert.sameValue(typeof x,'undefined');
            // A property reference is distinct: strict PutValue can create a
            // missing property on an extensible receiver after RHS deletion.
            x=0;function propertyAssign(){'use strict';globalThis.x=remove();}
            propertyAssign();assert.sameValue(x,1);assert.sameValue(delete x,true);
            var permanent=2;
            function keep(){assert.sameValue(delete globalThis.permanent,false);return 3;}
            permanent=keep();assert.sameValue(permanent,3);
            function local(a){a={valueOf(){assert.sameValue(delete a,false);return 4;}};a++;return a;}
            assert.sameValue(local(1),5);
        "#,&mut document).unwrap();
        runtime.execute("transient=1;", &mut document).unwrap();
        assert_eq!(
            runtime
                .execute(
                    "var transient;delete globalThis.transient;var transient;typeof transient",
                    &mut document
                )
                .unwrap(),
            Value::String("undefined".into())
        );
    }
    #[test]
    fn regexp_literals_use_parser_lexical_goals_and_constructor_identity() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            assert.sameValue(12 / 3 / 2,2);
            assert.sameValue(/=/.test('='),true);
            assert.sameValue(/["']/.test("'"),true);
            assert.sameValue(/[/*]/.test('/'),true);
            assert.sameValue(/[/]/.test('/'),true);
            function pattern(){return /a\/b/i;}
            assert.sameValue(pattern().test('A/b'),true);
            if(true) /a/.test('a');
            var re=/a/g;
            assert.sameValue(re instanceof RegExp,true);
            assert.sameValue(RegExp(re),re);
            assert.notSameValue(new RegExp(re),re);
            assert.sameValue(new RegExp(re).flags,'g');
            assert.sameValue(new RegExp(re,'i').flags,'i');
            assert.sameValue(RegExp().source,'(?:)');
            assert.sameValue(String(/a/gi),'/a/gi');
            assert.sameValue(Object.prototype.toString.call(re),'[object RegExp]');
            assert.sameValue(JSON.stringify({re:re}),'{"re":{}}');
            assert.sameValue(new RegExp('/\n').source,'\\/\\n');
            verifyProperty(re,'lastIndex',{value:0,writable:true,enumerable:false,configurable:false});
            verifyProperty(RegExp,'length',{value:2,writable:false,enumerable:false,configurable:true});
            verifyProperty(RegExp.prototype.exec,'length',{value:1,writable:false,enumerable:false,configurable:true});
            assert.throws(SyntaxError,()=>new RegExp('('));
            assert.throws(SyntaxError,()=>new RegExp('a','gg'));
            assert.throws(SyntaxError,()=>new RegExp('(?<x>a)\\k'));
            assert.throws(SyntaxError,()=>new RegExp('[\\k](?<x>a)'));
            assert.throws(TypeError,()=>RegExp.prototype.exec.call({},'a'));
            assert.throws(TypeError,()=>RegExp.prototype.test.call(null,'a'));
        "#,&mut document).unwrap();
        assert!(Runtime::parse_only("/[/").unwrap_err().is_parse_error());
        assert!(Runtime::parse_only("/a/gg").unwrap_err().is_parse_error());
        for source in [r"/(?<x>a)\k/", r"/[\k](?<x>a)/"] {
            let error = Runtime::parse_only(source).unwrap_err();
            assert!(error.is_parse_error());
            assert_eq!(error.intrinsic_error_name(), Some("SyntaxError"));
        }
        assert!(Runtime::parse_only("/a/u").unwrap_err().is_unsupported());
    }
    #[test]
    fn flat_binary_chains_and_do_while_preserve_evaluation_and_abrupt_flow() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            var trace='';function value(n){trace+=n;return n;}
            assert.sameValue(value(2)*value(3)+value(4)*value(5)-value(1),25);
            assert.sameValue(trace,'23451');
            var n=0;do {n++;if(n<3)continue;break;}while(n<8);
            assert.sameValue(n,3);
            var once=0;do {once++;}while(false);assert.sameValue(once,1);
            function f(){do {try{return 1;}finally{return 2;}}while(true);}
            assert.sameValue(f(),2);
            assert.sameValue(false && missing || 3,3);
        "#,
                &mut document,
            )
            .unwrap();
        assert_eq!(
            run(&format!("0{}", "+1".repeat(1000))).unwrap(),
            Value::Number(1000.0)
        );
        assert_eq!(
            run(&format!("''{}", "+'x'".repeat(1000))).unwrap(),
            Value::String("x".repeat(1000).into())
        );
        assert!(run("do {} while(true)").unwrap_err().is_resource_limit());
    }
    #[test]
    fn document_url_and_base_mutations_use_frozen_cache_and_preflight_quotas() {
        let mut document =
            Document::parse("<head><base id=base href='../one/'></head><body><p>old</p></body>");
        document.initialize_url(url::Url::parse("https://site.test/a/page.html").unwrap());
        let mut runtime = Runtime::new();
        runtime
            .execute(
                "function eq(a,b){if(a!==b)throw new Error('unexpected base URL');}",
                &mut document,
            )
            .unwrap();
        runtime
            .execute(
                r#"
            eq(document.URL,'https://site.test/a/page.html');eq(document.documentURI,document.URL);
            eq(document.baseURI,'https://site.test/one/');
            var base=document.getElementById('base');base.href='../two/';
            eq(document.baseURI,'https://site.test/two/');
            base.removeAttribute('href');eq(document.baseURI,document.URL);
            document.body.innerHTML='<base href="/dynamic/"><p>new</p>';
            eq(document.baseURI,'https://site.test/dynamic/');
            document.body.textContent='done';eq(document.baseURI,document.URL);
            var card=document.createElement('template');card.innerHTML='<base href="/template/">';
            eq(document.baseURI,document.URL);document.body.appendChild(card.content);
            eq(document.baseURI,'https://site.test/template/');
            document.querySelector('body base').remove();eq(document.baseURI,document.URL);
        "#,
                &mut document,
            )
            .unwrap();
        let mut document = Document::parse(&format!(
            "<base id=base href='/old/'><body><main>{}</main></body>",
            "<i></i>".repeat(1000)
        ));
        document.initialize_url(url::Url::parse("https://site.test/page").unwrap());
        let base = document.query_selector("#base").unwrap();
        let body = document.query_selector("body").unwrap();
        let old_children = document.nodes[body].children.clone();
        let mut runtime = Runtime::new();
        runtime.steps = 128;
        let error = runtime
            .set_inner_html(body, "<base href='/new/'>changed", &mut document)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(document.nodes[body].children, old_children);
        assert_eq!(document.base_url().as_str(), "https://site.test/old/");
        runtime.steps = 128;
        let error = runtime
            .native_call(
                &Native {
                    properties: None,
                    name: "DOM.setAttribute".into(),
                    receiver: Value::Node(base),
                },
                vec![Value::String("href".into()), Value::String("/new/".into())],
                &mut document,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(document.attr(base, "href"), Some("/old/"));
        assert_eq!(document.base_url().as_str(), "https://site.test/old/");
    }
    #[test]
    fn regexp_exec_indices_named_captures_and_last_index_are_observable() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var re=/(?<letter>a)(b)?/dg;var found=re.exec('xa ab');
            assert.sameValue(found[0],'a');assert.sameValue(found[1],'a');assert.sameValue(found[2],undefined);
            assert.sameValue(found.index,1);assert.sameValue(found.input,'xa ab');
            assert.sameValue(found.groups.letter,'a');assert.sameValue(Object.getPrototypeOf(found.groups),null);
            assert.sameValue(found.indices[0][0],1);assert.sameValue(found.indices[0][1],2);
            assert.sameValue(found.indices.groups.letter,found.indices[1]);
            var ordered=/(?<b>x)(?<a>y)/d.exec('xy');
            assert.sameValue(Object.keys(ordered.groups).join(','),'b,a');
            assert.sameValue(Object.keys(ordered.indices.groups).join(','),'b,a');
            assert.sameValue(ordered.indices.groups.b,ordered.indices[1]);
            assert.sameValue(ordered.indices.groups.a,ordered.indices[2]);
            assert.sameValue(found.indices[2],undefined);assert.sameValue(re.lastIndex,2);
            assert.sameValue(re.exec('xa ab')[0],'ab');assert.sameValue(re.lastIndex,5);
            assert.sameValue(re.exec('xa ab'),null);assert.sameValue(re.lastIndex,0);
            var sticky=/a/y;sticky.lastIndex=1;
            assert.sameValue(sticky.exec('ba')[0],'a');assert.sameValue(sticky.lastIndex,2);
            sticky.lastIndex=0;assert.sameValue(sticky.exec('ba'),null);assert.sameValue(sticky.lastIndex,0);
            var plain=/a/;plain.lastIndex=99;assert.sameValue(plain.exec('ba').index,1);assert.sameValue(plain.lastIndex,99);
            var empty=/(?:)/g;assert.sameValue(empty.exec('x')[0],'');assert.sameValue(empty.lastIndex,0);
            var reads=0;plain.lastIndex={valueOf(){reads++;return 99;}};
            plain.exec('a');assert.sameValue(reads,1);
            var frozen=/a/g;Object.defineProperty(frozen,'lastIndex',{writable:false});
            assert.throws(TypeError,()=>frozen.exec('a'));
            var other={exec(s){assert.sameValue(s,'abc');return {};}};
            assert.sameValue(RegExp.prototype.test.call(other,'abc'),true);
            other.exec=function(){return 1;};assert.throws(TypeError,()=>RegExp.prototype.test.call(other,'abc'));
        "#,&mut document).unwrap();
    }
    #[test]
    fn regexp_backtracking_assertions_and_utf16_preserve_capture_semantics() {
        let (mut runtime, mut document) = property_harness();
        runtime
            .execute(
                r#"
            assert.sameValue(/(a|ab)+?b/.exec('aab')[0],'aab');
            assert.sameValue(/a+?/.exec('aaaa')[0],'a');
            assert.sameValue(/(a?)*$/.exec('')[1],undefined);
            assert.sameValue(/(a?){2}/.exec('')[1],'');
            assert.sameValue(/(a|(b))+/.exec('ba')[2],undefined);
            assert.sameValue(/(?=(a+))a*b\1/.exec('baaabac')[0],'aba');
            assert.sameValue(/^(?<word>\w+)\s+\k<word>$/i.test('Hello HELLO'),true);
            assert.sameValue(/\b(?!bad)\w+\b/.exec('bad good')[0],'good');
            assert.sameValue(/a$/.test('a\n'),false);assert.sameValue(/^b$/m.test('a\nb\nc'),true);
            assert.sameValue(/./.test('\n'),false);assert.sameValue(/./s.test('\n'),true);
            assert.sameValue(/[^]/.test('\n'),true);assert.sameValue(/[]/.test('a'),false);
            assert.sameValue(/[\s\S]/.test('\uD800'),true);
            assert.sameValue(/../.exec('😀')[0].length,2);
            assert.sameValue(/\uD800/.exec('\uD800')[0].charCodeAt(0),55296);
            assert.sameValue(/[a-z]/i.test('ſ'),false);assert.sameValue(/[ς]/i.test('Σ'),true);
        "#,
                &mut document,
            )
            .unwrap();
    }
    #[test]
    fn regexp_string_methods_handle_empty_matches_captures_and_replacement_callbacks() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            assert.sameValue('aba'.match(/a/g).join(','),'a,a');
            assert.sameValue('aba'.match(/z/g),null);
            assert.sameValue('a'.match(/(?:)/g).length,2);
            assert.sameValue('😀'.match(/./g).length,2);
            assert.sameValue('abc'.match('b').index,1);
            var re=/b/g;re.lastIndex=-0;assert.sameValue('abc'.search(re),1);assert.sameValue(re.lastIndex,-0);
            assert.sameValue('ab12cd34'.replace(/(\d+)/g,'[$1]'),'ab[12]cd[34]');
            assert.sameValue('abc'.replace(/b/,'$$:$&:$'+String.fromCharCode(96)+':$\''),'a$:b:a:cc');
            assert.sameValue('ab'.replace(/(?<a>a)(b)?/,'$<a>:$2:$3:$01'),'a:b:$3:a');
            assert.sameValue('ab'.replace('b','$&$&'),'abb');
            var calls=0;var expression=/\d/g;
            assert.sameValue('a1b2'.replace(expression,function(m,offset,input){
                assert.sameValue(expression.lastIndex,0);assert.sameValue(input,'a1b2');calls++;return offset;
            }),'a1b3');assert.sameValue(calls,2);
            assert.sameValue('ab'.replace(/(?:)/g,'-'),'-a-b-');
            assert.sameValue('a,b;c'.split(/([,;])/).join('|'),'a|,|b|;|c');
            assert.sameValue('ab'.split(/(?:)/).join(','),'a,b');
            assert.sameValue(''.split(/(?:)/).length,0);
            assert.sameValue('a,'.split(/,/).join('|'),'a|');
            assert.sameValue('ab'.split(/(x)?b/)[1],undefined);
            assert.sameValue('a,b'.split(/,/ ,1).length,1);
            assert.throws(TypeError,()=>'a'.includes(/a/));
            assert.throws(TypeError,()=>'a'.startsWith(/a/));
        "#,&mut document).unwrap();
    }
    #[test]
    fn regexp_string_iteration_observes_flags_once_and_preserves_coercion_order() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            var re=/a/g;
            Object.defineProperty(re,'flags',{value:'',configurable:true});
            Object.defineProperty(re,'global',{get(){throw new Error('unexpected global read');}});
            var result='aa'.match(re);
            assert.sameValue(result.length,1);assert.sameValue(result.index,0);
            re.lastIndex=0;assert.sameValue('aa'.replace(re,'b'),'ba');
            var trace='';
            Object.defineProperty(re,'flags',{get(){
                trace+='flags;';
                return {toString(){trace+='convert;';return '';}};
            }});
            re.lastIndex=0;
            assert.sameValue('aa'.replace(re,{toString(){trace+='replacement;';return 'b';}}),'ba');
            assert.sameValue(trace,'replacement;flags;convert;');
            trace='';re.lastIndex=0;assert.sameValue('aa'.match(re).length,1);
            assert.sameValue(trace,'flags;convert;');
            var empty=/(?:)/g;Object.defineProperty(empty,'flags',{value:'gu'});
            assert.sameValue('😀'.match(empty).length,2);
            assert.sameValue('😀'.replace(empty,'-'),'-😀-');
            var broken=/a/g;broken.lastIndex=1;
            Object.defineProperty(broken,'flags',{get(){throw new TypeError('flags');}});
            assert.throws(TypeError,()=>'aa'.match(broken));assert.sameValue(broken.lastIndex,1);
            assert.throws(TypeError,()=>'aa'.replace(broken,'b'));assert.sameValue(broken.lastIndex,1);
        "#,&mut document).unwrap();
    }
    #[test]
    fn regexp_hostile_backtracking_and_native_callbacks_share_uncatchable_budgets() {
        for source in [
            "var caught=false;try { /(a+)+$/.test('aaaaaaaaaaaaaaaaaaaaaaaa!'); } catch(e) {caught=true;}",
            "var caught=false;try { var re=/a/g;re.exec=function(){return {0:'',length:1,index:0};};'a'.match(re); } catch(e){caught=true;}",
            "var caught=false;try { function f(){return 'a'.replace(/a/,f);}f(); } catch(e){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            assert!(
                runtime
                    .execute(source, &mut document)
                    .unwrap_err()
                    .is_resource_limit(),
                "{source}"
            );
            assert_eq!(
                runtime.environments[0].bindings["caught"].value,
                Value::Bool(false)
            );
        }
    }
    #[test]
    fn repeated_dom_moves_charge_template_subtrees_and_leave_failed_moves_unapplied() {
        for method in ["appendChild", "append"] {
            let mut document = Document::parse(&format!(
                "<main id=left><section id=group><template>{}</template></section></main><aside id=right></aside>",
                "<i></i>".repeat(1500)
            ));
            let group = document.query_selector("#group").unwrap();
            let left = document.query_selector("#left").unwrap();
            let right = document.query_selector("#right").unwrap();
            let template = document.query_selector("template").unwrap();
            let contents = document.template_contents(template).unwrap();
            let mut runtime = Runtime::new();
            let error = runtime
                .execute(
                    &format!(
                        r#"
                var caught=false;var moved=0;
                var group=document.getElementById('group');
                var target=document.getElementById('right');
                var other=document.getElementById('left');
                try {{ for(var i=0;i<1000;i++) {{
                    target.{method}(group);moved++;
                    var swap=target;target=other;other=swap;
                }} }} catch(e) {{caught=true;}}
            "#
                    ),
                    &mut document,
                )
                .unwrap_err();
            assert!(error.is_resource_limit(), "{method}: {error}");
            assert_eq!(
                runtime.execute("caught", &mut document).unwrap(),
                Value::Bool(false)
            );
            let Value::Number(moved) = runtime.execute("moved", &mut document).unwrap() else {
                panic!("move counter is numeric");
            };
            assert!(moved > 0.0 && moved < 100.0, "{method}: {moved}");
            let (last, other) = if moved as usize % 2 == 1 {
                (right, left)
            } else {
                (left, right)
            };
            assert_eq!(document.nodes[group].parent, Some(last));
            assert_eq!(document.nodes[last].children, vec![group]);
            assert!(document.nodes[other].children.is_empty());
            assert_eq!(document.nodes[contents].children.len(), 1500);
        }
    }
    #[test]
    fn fragment_transfer_and_sibling_removal_preflight_before_mutation() {
        let mut document = Document::parse("<main><p></p></main><aside></aside>");
        let source = document.query_selector("main").unwrap();
        let target = document.query_selector("aside").unwrap();
        let child = document.query_selector("p").unwrap();
        let fragment = document.create_document_fragment();
        for _ in 0..64 {
            let node = document.create_element("i");
            document.append_child(fragment, node);
        }
        let children = document.nodes[fragment].children.clone();
        let mut runtime = Runtime::new();
        // The subtree can be inspected, but the final child transfer cannot.
        runtime.steps = 150;
        let error = runtime
            .native_call(
                &Native {
                    properties: None,
                    name: "DOM.appendChild".into(),
                    receiver: Value::Node(target),
                },
                vec![Value::Node(fragment)],
                &mut document,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(document.nodes[fragment].children, children);
        assert!(document.nodes[target].children.is_empty());
        assert!(
            children
                .iter()
                .all(|id| document.nodes[*id].parent == Some(fragment))
        );
        // A tiny moved subtree can still require scanning a wide old parent.
        for _ in 0..64 {
            let sibling = document.create_element("i");
            document.append_child(source, sibling);
        }
        for method in ["appendChild", "removeChild", "remove"] {
            runtime.steps = 16;
            let receiver = if method == "appendChild" {
                target
            } else if method == "removeChild" {
                source
            } else {
                child
            };
            let error = runtime
                .native_call(
                    &Native {
                        properties: None,
                        name: format!("DOM.{method}"),
                        receiver: Value::Node(receiver),
                    },
                    vec![Value::Node(child)],
                    &mut document,
                )
                .unwrap_err();
            assert!(error.is_resource_limit(), "{method}: {error}");
            assert_eq!(document.nodes[child].parent, Some(source));
            assert_eq!(document.nodes[source].children.len(), 65);
            assert!(document.nodes[target].children.is_empty());
        }
    }
    #[test]
    fn arguments_are_unmapped_in_strict_code_and_mapped_with_descriptor_detachment_otherwise() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            function strict(a){'use strict';
                assert.sameValue(Array.isArray(arguments),false);
                assert.sameValue(Object.prototype.toString.call(arguments),'[object Arguments]');
                arguments[0]=2;assert.sameValue(a,1);a=3;assert.sameValue(arguments[0],2);
                assert.throws(TypeError,()=>arguments.callee);
                assert.throws(TypeError,()=>{arguments.callee=1;});
                verifyProperty(arguments,'callee',{enumerable:false,configurable:false});
            }strict(1);
            function loose(a){
                a=2;assert.sameValue(arguments[0],2);arguments[0]=3;assert.sameValue(a,3);
                Object.defineProperty(arguments,'0',{value:4});assert.sameValue(a,4);
                Object.defineProperty(arguments,'0',{writable:false});a=5;assert.sameValue(arguments[0],4);
                assert.sameValue(arguments.callee,loose);
            }loose(1);
            function removed(a){delete arguments[0];a=2;assert.sameValue(arguments[0],undefined);}removed(1);
            function duplicates(a,a){arguments[0]=8;assert.sameValue(a,2);arguments[1]=9;assert.sameValue(a,9);}duplicates(1,2);
            assert.throws(TypeError,()=>strict.caller);assert.throws(TypeError,()=>strict.arguments);
        "#,&mut document).unwrap();
    }
    #[test]
    fn lexical_tdz_global_separation_and_iteration_closures_are_preserved() {
        let (mut runtime, mut document) = property_harness();
        runtime.execute(r#"
            assert.throws(ReferenceError,function(){'use strict';typeof x;let x;});
            assert.throws(ReferenceError,function(){'use strict';x=1;let x;});
            assert.throws(ReferenceError,function(){'use strict';let x=x;});
            assert.throws(ReferenceError,function(){'use strict';for(let x in x){}});
            assert.throws(ReferenceError,function(){'use strict';switch(1){case x:let x=1;}});
            let privateName=3;globalThis.privateName=9;
            assert.sameValue(privateName,3);assert.sameValue(globalThis.privateName,9);
            var closures=[];for(let i=0;i<3;i++){closures.push(()=>i);}
            assert.sameValue(closures[0](),0);assert.sameValue(closures[2](),2);
            var keys=[];for(let key in {a:1,b:2}){keys.push(()=>key);}
            assert.sameValue(keys[0](),'a');assert.sameValue(keys[1](),'b');
            function f(){'use strict';{function block(){return 1;}assert.sameValue(block(),1);}assert.sameValue(typeof block,'undefined');}f();
        "#,&mut document).unwrap();
    }
    #[test]
    fn template_fragments_clone_query_and_move_without_activating_nested_content() {
        let mut document = Document::parse(
            "<!doctype html><template id=card><article><b class=label>first</b><svg viewBox='0 0 3 3'><circle r=1/></svg><template id=nested><i>hidden</i></template></article></template><main id=out></main>",
        );
        let (mut runtime, _) = property_harness();
        runtime.execute(r#"
            'use strict';
            var template=document.getElementById('card');
            assert.sameValue(template.content.nodeType,11);
            assert.sameValue(template.content.nodeName,'#document-fragment');
            assert.sameValue(template.content.parentNode,null);
            assert.sameValue(template.childNodes.length,0);
            assert.sameValue(template.textContent,'');
            assert.sameValue(document.querySelector('.label'),null);
            var copy=template.content.cloneNode(true);
            copy.querySelector('.label').textContent='second';
            assert.sameValue(template.content.querySelector('.label').textContent,'first');
            assert.sameValue(copy.querySelector('svg').getAttribute('viewBox'),'0 0 3 3');
            copy.querySelector('svg').setAttribute('viewBox','0 0 4 4');
            assert.sameValue(copy.querySelector('svg').getAttribute('viewBox'),'0 0 4 4');
            assert.sameValue(copy.querySelector('i'),null);
            assert.sameValue(copy.querySelector('#nested').content.querySelector('i').textContent,'hidden');
            document.getElementById('out').appendChild(copy);
            assert.sameValue(copy.childNodes.length,0);
            assert.sameValue(document.querySelector('.label').textContent,'second');
            assert.sameValue(document.querySelector('i'),null);
            var clonedTemplate=template.cloneNode(true);
            assert.sameValue(clonedTemplate.content.querySelector('.label').textContent,'first');
            assert.sameValue(template.cloneNode(false).content.childNodes.length,0);
            template.innerHTML='<template><em>deep</em></template><p>replacement</p>';
            assert.sameValue(template.content.querySelector('p').textContent,'replacement');
            assert.sameValue(template.content.querySelector('template').content.querySelector('em').textContent,'deep');
            assert.sameValue(document.characterSet,'UTF-8');
            assert.sameValue(document.charset,document.inputEncoding);
            assert.sameValue(document.compatMode,'CSS1Compat');
            assert.throws(TypeError,function(){document.characterSet='windows-1252';});
        "#,&mut document).unwrap();
    }
}
