//! A deliberately small, capability-limited JavaScript interpreter.
//!
//! This is a custom language implementation, not an ECMAScript conformance claim.
//! Every entry point enforces execution, nesting, source, and allocation limits.
//! Scripts have DOM access but no filesystem, network, process, or host-eval access.
//! Strings and ordinary property keys preserve UTF-16 code units. Conversion to
//! UTF-8 is lossy only at the display/DOM boundary; JSON retains lone surrogates.

use crate::dom::{Document, Namespace, NodeId, NodeKind};
use crate::js_string::{JsString, is_js_whitespace, radix_number};
use crate::regexp::{self, RegExp};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Native {
    name: String,
    receiver: Value,
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
fn regexp_error(error: regexp::Error) -> ScriptError {
    match error {
        regexp::Error::Syntax(message) => ScriptError::syntax(message),
        regexp::Error::Unsupported(message) => ScriptError::unsupported(message),
        regexp::Error::Resource(message) => ScriptError::resource(message),
    }
}

#[derive(Clone, Debug)]
enum TokenKind {
    Word(String),
    Number(f64),
    String(JsString),
    Symbol(String),
    RegExp(Rc<RegExp>),
    TemplateStart,
    Invalid(ScriptError),
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
    let mut pos = start + 1;
    let mut legacy_literal = false;
    let mut interpolation = false;
    let mut value = Vec::<u16>::new();
    let mut closed = false;
    while pos < source.len() {
        if let Some(budget) = budget.as_deref_mut() {
            budget.work(1).map_err(regexp_error)?;
            budget.allocated = budget.allocated.saturating_add(8);
            if budget.allocated > MAX_HEAP {
                return Err(ScriptError::resource("template storage limit exceeded"));
            }
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
    }
    Ok((JsString::from(value), pos, legacy_literal, interpolation))
}

fn lex(source: &str) -> Result<Vec<Token>> {
    if source.len() > MAX_SOURCE {
        return Err(ScriptError::resource("script source limit exceeded"));
    }
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut line_break_before = false;
    // The parser supplies the RegExp lexical goal at a primary expression.
    // Keep provisional division-goal errors as tokens: quotes and comments
    // inside a yet-unrecognized pattern must not reject the whole script.
    let result = (|| -> Result<()> {
        while pos < source.len() {
            let ch = source[pos..].chars().next().unwrap();
            if ch.is_whitespace() {
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
                let (value, end, legacy, _) = quoted_text(source, start, ch, None)?;
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
            } else if ch.is_alphabetic() || ch == '_' || ch == '$' {
                pos += ch.len_utf8();
                while let Some(c) = source[pos..].chars().next() {
                    if !(c.is_alphanumeric() || c == '_' || c == '$') {
                        break;
                    }
                    pos += c.len_utf8();
                }
                TokenKind::Word(source[start..pos].to_owned())
            } else {
                let operator = [
                    "===", "!==", ">>>", "**=", "=>", "==", "!=", "<=", ">=", "&&", "||", "??",
                    "++", "--", "+=", "-=", "*=", "/=", "%=", "**", "<<", ">>",
                ]
                .into_iter()
                .find(|op| source[pos..].starts_with(op));
                if let Some(op) = operator {
                    pos += op.len();
                    TokenKind::Symbol(op.to_owned())
                } else if "{}[]().,;:?+-*/%<>=!~&|^".contains(ch) {
                    pos += 1;
                    TokenKind::Symbol(ch.to_string())
                } else {
                    return Err(ScriptError::at(
                        format!("unsupported character {ch:?}"),
                        pos,
                    ));
                }
            };
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
        tokens.push(Token {
            offset: error.offset.unwrap_or(pos),
            kind: TokenKind::Invalid(error),
            line_break_before,
            string_literal: false,
            use_strict: false,
            legacy_literal: false,
        });
    }
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

#[derive(Clone, Debug)]
struct FunctionCode {
    params: Vec<String>,
    body: Rc<Vec<Stmt>>,
    name: Option<String>,
    arrow: bool,
    self_name: bool,
    constructable: bool,
    strict: bool,
}
#[derive(Debug)]
struct Program {
    body: Vec<Stmt>,
    strict: bool,
    compiled_storage: usize,
}
#[derive(Clone, Debug)]
enum Expr {
    Literal(Value),
    RegExp(Rc<RegExp>),
    Ident(String),
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
#[derive(Clone, Debug)]
enum PropertyName {
    Literal(JsString, bool), // The boolean preserves IdentifierName syntax for shorthand/accessors.
    Computed(Box<Expr>),
}
#[derive(Clone, Debug)]
enum ObjectEntry {
    Data(Expr),
    Method(FunctionCode),
    Accessor(FunctionCode, bool),
    Prototype(Expr),
}
#[derive(Clone, Debug)]
enum Stmt {
    Empty,
    Expr(Expr),
    Var(Vec<(String, Option<Expr>)>, DeclarationKind),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    DoWhile(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    ForIn(ForBinding, Expr, Box<Stmt>),
    Switch(Expr, Vec<(Option<Expr>, Vec<Stmt>)>),
    Function(String, FunctionCode),
    Return(Option<Expr>),
    Throw(Expr),
    Try(Box<Stmt>, Option<CatchClause>, Option<Box<Stmt>>),
    Break,
    Continue,
}
#[derive(Clone, Debug)]
enum ForBinding {
    Declaration(String, DeclarationKind),
    Target(Expr),
}

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

struct Parser {
    tokens: Vec<Token>,
    source: String,
    lex_work: usize,
    compile_budget: regexp::Budget,
    pos: usize,
    depth: usize,
    function_depth: usize,
    loop_depth: usize,
    switch_depth: usize,
    allow_in: bool,
    strict: bool,
}
impl Parser {
    fn program(source: &str) -> Result<Program> {
        Self::program_context(source, false, false)
    }
    fn program_context(source: &str, function: bool, strict: bool) -> Result<Program> {
        let mut parser = Self {
            tokens: lex(source)?,
            source: source.into(),
            lex_work: source.len(),
            compile_budget: regexp::Budget {
                steps: MAX_STEPS,
                allocated: 0,
                heap_limit: MAX_HEAP,
                stack_limit: 16,
            },
            pos: 0,
            depth: 0,
            function_depth: usize::from(function),
            loop_depth: 0,
            switch_depth: 0,
            allow_in: true,
            strict,
        };
        let body = parser.directive_body(false)?;
        Self::check_scope(&body, false)?;
        Ok(Program {
            body,
            strict: parser.strict,
            compiled_storage: parser.compile_budget.allocated,
        })
    }
    fn directive_body(&mut self, block: bool) -> Result<Vec<Stmt>> {
        let mut body = Vec::new();
        let mut prologue = true;
        let start = self.pos;
        while !(if block { self.eat("}") } else { self.done() }) {
            if self.done() {
                return Err(self.error("unterminated function body"));
            }
            let token = self.tokens[self.pos].clone();
            let before = self.pos;
            let statement = self.statement()?;
            let bare_string = token.string_literal
                && matches!(&statement, Stmt::Expr(Expr::Literal(Value::String(_))))
                && (self.pos == before + 1
                    || self.pos == before + 2
                        && matches!(&self.tokens[before + 1].kind, TokenKind::Symbol(s) if s == ";"));
            if prologue && bare_string {
                if token.use_strict {
                    self.strict = true;
                    if self.tokens[start..self.pos]
                        .iter()
                        .any(|token| token.legacy_literal)
                    {
                        return Err(self.error("legacy escapes are forbidden in strict directives"));
                    }
                }
            } else {
                prologue = false;
            }
            body.push(statement);
        }
        Ok(body)
    }
    fn done(&self) -> bool {
        matches!(self.tokens[self.pos].kind, TokenKind::End)
    }
    fn is(&self, text: &str) -> bool {
        matches!(&self.tokens[self.pos].kind, TokenKind::Word(s) | TokenKind::Symbol(s) if s == text)
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.is(text) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Result<()> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(self.error(format!("expected '{text}'")))
        }
    }
    fn error(&self, message: impl Into<String>) -> ScriptError {
        if let TokenKind::Invalid(error) = &self.tokens[self.pos].kind {
            return error.clone();
        }
        ScriptError::at(message, self.tokens[self.pos].offset)
    }
    fn resource_error(&self, message: impl Into<String>) -> ScriptError {
        let mut error = ScriptError::resource(message);
        error.offset = Some(self.tokens[self.pos].offset);
        error
    }
    fn identifier(&mut self) -> Result<String> {
        if let TokenKind::Word(s) = &self.tokens[self.pos].kind {
            let name = s.clone();
            self.pos += 1;
            Ok(name)
        } else {
            Err(self.error("expected identifier"))
        }
    }
    fn binding_identifier(&mut self) -> Result<String> {
        if self.is("{") || self.is("[") || self.is(".") {
            return Err(ScriptError::unsupported(
                "destructuring and rest bindings are not implemented",
            ));
        }
        let name = self.identifier()?;
        self.validate_identifier(&name, true)?;
        Ok(name)
    }
    fn validate_identifier(&self, name: &str, binding: bool) -> Result<()> {
        if matches!(
            name,
            "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
        ) || self.strict
            && (matches!(
                name,
                "implements"
                    | "interface"
                    | "let"
                    | "package"
                    | "private"
                    | "protected"
                    | "public"
                    | "static"
                    | "yield"
            ) || binding && matches!(name, "eval" | "arguments"))
        {
            return Err(self.error(format!(
                "invalid {} '{name}'",
                if binding { "binding" } else { "identifier" }
            )));
        }
        Ok(())
    }
    fn assignment_target(&self, target: &Expr) -> Result<()> {
        if !matches!(target, Expr::Ident(name) if name != "this")
            && !matches!(target, Expr::Member(..))
        {
            return Err(self.error("invalid assignment target"));
        }
        if self.strict
            && matches!(target, Expr::Ident(name) if matches!(name.as_str(), "eval"|"arguments"))
        {
            return Err(self.error("strict assignment to eval or arguments"));
        }
        Ok(())
    }
    fn semicolon(&mut self) -> Result<()> {
        if self.eat(";") || self.done() || self.is("}") || self.tokens[self.pos].line_break_before {
            Ok(())
        } else {
            Err(self.error("expected ';'"))
        }
    }
    fn enter(&mut self) -> Result<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.resource_error("parser nesting limit exceeded"));
        }
        Ok(())
    }
    fn statement(&mut self) -> Result<Stmt> {
        self.enter()?;
        let result = self.statement_inner();
        self.depth -= 1;
        result
    }
    fn controlled_statement(&mut self) -> Result<Stmt> {
        let statement = self.statement()?;
        if matches!(
            statement,
            Stmt::Var(_, DeclarationKind::Let | DeclarationKind::Const)
        ) {
            return Err(self.error("lexical declarations require a statement list"));
        }
        if matches!(statement, Stmt::Function(..)) {
            if self.strict {
                return Err(self.error("strict function declarations require a statement list"));
            }
            return Err(ScriptError::unsupported(
                "legacy conditional function declarations are not implemented",
            ));
        }
        Ok(statement)
    }
    fn statement_inner(&mut self) -> Result<Stmt> {
        if self.eat(";") {
            return Ok(Stmt::Empty);
        }
        if self.eat("{") {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.is("let") || self.is("const") || self.is("var") {
            let declaration = self.declaration()?;
            self.semicolon()?;
            return Ok(declaration);
        }
        if self.eat("function") {
            let name = self.binding_identifier()?;
            let mut code = self.function()?;
            let saved = self.strict;
            self.strict = code.strict;
            self.validate_identifier(&name, true)?;
            self.strict = saved;
            code.name = Some(name.clone());
            return Ok(Stmt::Function(name, code));
        }
        if self.eat("switch") {
            self.expect("(")?;
            let value = self.expression()?;
            self.expect(")")?;
            self.expect("{")?;
            let mut cases = Vec::new();
            let mut has_default = false;
            self.switch_depth += 1;
            while !self.eat("}") {
                let condition = if self.eat("case") {
                    Some(self.expression()?)
                } else if self.eat("default") {
                    if has_default {
                        return Err(self.error("duplicate switch default"));
                    }
                    has_default = true;
                    None
                } else {
                    return Err(self.error("expected case or default"));
                };
                self.expect(":")?;
                let mut body = Vec::new();
                while !self.is("case") && !self.is("default") && !self.is("}") {
                    if self.done() {
                        return Err(self.error("unterminated switch"));
                    }
                    body.push(self.statement()?);
                }
                cases.push((condition, body));
            }
            self.switch_depth -= 1;
            let combined = cases
                .iter()
                .flat_map(|(_, body)| body.iter().cloned())
                .collect::<Vec<_>>();
            Self::check_scope(&combined, true)?;
            return Ok(Stmt::Switch(value, cases));
        }
        if self.eat("if") {
            self.expect("(")?;
            let condition = self.sequence()?;
            self.expect(")")?;
            let yes = Box::new(self.controlled_statement()?);
            let no = if self.eat("else") {
                Some(Box::new(self.controlled_statement()?))
            } else {
                None
            };
            if self.strict
                && (matches!(&*yes, Stmt::Function(..))
                    || no
                        .as_deref()
                        .is_some_and(|s| matches!(s, Stmt::Function(..))))
            {
                return Err(self.error("strict function declarations require a statement list"));
            }
            return Ok(Stmt::If(condition, yes, no));
        }
        if self.eat("while") {
            self.expect("(")?;
            let condition = self.sequence()?;
            self.expect(")")?;
            self.loop_depth += 1;
            let body = self.controlled_statement()?;
            self.loop_depth -= 1;
            return Ok(Stmt::While(condition, Box::new(body)));
        }
        if self.eat("do") {
            self.loop_depth += 1;
            let body = self.controlled_statement()?;
            self.loop_depth -= 1;
            self.expect("while")?;
            self.expect("(")?;
            let condition = self.sequence()?;
            self.expect(")")?;
            self.eat(";");
            return Ok(Stmt::DoWhile(condition, Box::new(body)));
        }
        if self.eat("for") {
            self.expect("(")?;
            let saved_in = self.allow_in;
            self.allow_in = false;
            let init = if self.is(";") {
                None
            } else if self.is("let") || self.is("const") || self.is("var") {
                Some(Box::new(self.declaration()?))
            } else {
                Some(Box::new(Stmt::Expr(self.sequence()?)))
            };
            self.allow_in = saved_in;
            if self.eat("in") {
                let binding = match init.map(|init| *init) {
                    Some(Stmt::Var(mut bindings, kind))
                        if bindings.len() == 1 && bindings[0].1.is_none() =>
                    {
                        ForBinding::Declaration(bindings.remove(0).0, kind)
                    }
                    Some(Stmt::Expr(target @ (Expr::Ident(_) | Expr::Member(..)))) => {
                        self.assignment_target(&target)?;
                        ForBinding::Target(target)
                    }
                    _ => return Err(self.error("invalid for-in binding")),
                };
                let object = self.sequence()?;
                self.expect(")")?;
                self.loop_depth += 1;
                let body = self.controlled_statement()?;
                self.loop_depth -= 1;
                if let ForBinding::Declaration(name, kind) = &binding
                    && *kind != DeclarationKind::Var
                {
                    let mut vars = BTreeSet::new();
                    Self::var_names(&body, &mut vars);
                    if vars.contains(name.as_str()) {
                        return Err(self.error("for-in lexical binding conflicts with var"));
                    }
                }
                return Ok(Stmt::ForIn(binding, object, Box::new(body)));
            }
            self.expect(";")?;
            let test = if self.is(";") {
                None
            } else {
                Some(self.sequence()?)
            };
            self.expect(";")?;
            let update = if self.is(")") {
                None
            } else {
                Some(self.sequence()?)
            };
            self.expect(")")?;
            self.loop_depth += 1;
            let body = self.controlled_statement()?;
            self.loop_depth -= 1;
            if let Some(init) = &init
                && let Stmt::Var(bindings, kind) = &**init
                && *kind != DeclarationKind::Var
            {
                let mut vars = BTreeSet::new();
                Self::var_names(&body, &mut vars);
                if bindings
                    .iter()
                    .any(|(name, _)| vars.contains(name.as_str()))
                {
                    return Err(self.error("for lexical binding conflicts with var"));
                }
                Self::check_scope(std::slice::from_ref(&**init), false)?;
            }
            return Ok(Stmt::For(init, test, update, Box::new(body)));
        }
        if self.eat("return") {
            if self.function_depth == 0 {
                return Err(self.error("return outside function"));
            }
            let result = if self.is(";")
                || self.is("}")
                || self.done()
                || self.tokens[self.pos].line_break_before
            {
                None
            } else {
                Some(self.sequence()?)
            };
            self.semicolon()?;
            return Ok(Stmt::Return(result));
        }
        if self.eat("throw") {
            if self.tokens[self.pos].line_break_before {
                return Err(self.error("line break is not allowed after throw"));
            }
            let value = self.sequence()?;
            self.semicolon()?;
            return Ok(Stmt::Throw(value));
        }
        if self.eat("try") {
            self.expect("{")?;
            let body = Box::new(Stmt::Block(self.block()?));
            let handler = if self.eat("catch") {
                let binding = if self.eat("(") {
                    let name = self.binding_identifier()?;
                    self.expect(")")?;
                    Some(name)
                } else {
                    None
                };
                self.expect("{")?;
                let body = self.block()?;
                if let Some(name) = &binding {
                    Self::check_parameter_lexicals(std::slice::from_ref(name), &body)?;
                }
                Some(CatchClause { binding, body })
            } else {
                None
            };
            let finalizer = if self.eat("finally") {
                self.expect("{")?;
                Some(Box::new(Stmt::Block(self.block()?)))
            } else {
                None
            };
            if handler.is_none() && finalizer.is_none() {
                return Err(self.error("try requires catch or finally"));
            }
            return Ok(Stmt::Try(body, handler, finalizer));
        }
        if self.eat("break") {
            if self.loop_depth == 0 && self.switch_depth == 0 {
                return Err(self.error("break outside loop or switch"));
            }
            if !self.tokens[self.pos].line_break_before
                && matches!(self.tokens[self.pos].kind, TokenKind::Word(_))
            {
                return Err(ScriptError::unsupported(
                    "labeled control flow is not implemented",
                ));
            }
            self.semicolon()?;
            return Ok(Stmt::Break);
        }
        if self.eat("continue") {
            if self.loop_depth == 0 {
                return Err(self.error("continue outside loop"));
            }
            if !self.tokens[self.pos].line_break_before
                && matches!(self.tokens[self.pos].kind, TokenKind::Word(_))
            {
                return Err(ScriptError::unsupported(
                    "labeled control flow is not implemented",
                ));
            }
            self.semicolon()?;
            return Ok(Stmt::Continue);
        }
        for unsupported in [
            "class", "import", "export", "catch", "finally", "do", "with", "async", "await",
            "yield",
        ] {
            if self.is(unsupported) {
                if self.strict && matches!(unsupported, "with" | "yield") {
                    return Err(self.error(format!("'{unsupported}' is forbidden in strict code")));
                }
                return Err(ScriptError::unsupported(format!(
                    "'{unsupported}' is not supported"
                )));
            }
        }
        if matches!(self.tokens[self.pos].kind, TokenKind::Word(_))
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|t| matches!(&t.kind, TokenKind::Symbol(s) if s == ":"))
        {
            return Err(ScriptError::unsupported(
                "labeled statements are not implemented",
            ));
        }
        let expression = self.sequence()?;
        self.semicolon()?;
        Ok(Stmt::Expr(expression))
    }
    fn declaration(&mut self) -> Result<Stmt> {
        let kind = if self.eat("const") {
            DeclarationKind::Const
        } else if self.eat("var") {
            DeclarationKind::Var
        } else {
            self.expect("let")?;
            DeclarationKind::Let
        };
        let mut bindings = Vec::new();
        loop {
            let name = self.binding_identifier()?;
            if kind != DeclarationKind::Var && name == "let" {
                return Err(self.error("lexical declaration cannot bind let"));
            }
            let value = if self.eat("=") {
                Some(self.expression()?)
            } else {
                None
            };
            if kind == DeclarationKind::Const && value.is_none() && !self.is("in") {
                return Err(self.error("const declaration needs a value"));
            }
            bindings.push((name, value));
            if !self.eat(",") {
                break;
            }
        }
        Ok(Stmt::Var(bindings, kind))
    }
    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut body = Vec::new();
        while !self.eat("}") {
            if self.done() {
                return Err(self.error("unterminated block"));
            }
            body.push(self.statement()?);
        }
        Self::check_scope(&body, true)?;
        Ok(body)
    }
    fn check_scope(body: &[Stmt], block_functions: bool) -> Result<()> {
        let mut lexical = BTreeSet::new();
        for statement in body {
            match statement {
                Stmt::Var(bindings, kind) if *kind != DeclarationKind::Var => {
                    for (name, _) in bindings {
                        if !lexical.insert(name.as_str()) {
                            return Err(ScriptError::at(
                                format!("duplicate lexical binding '{name}'"),
                                0,
                            ));
                        }
                    }
                }
                Stmt::Function(name, _) if block_functions && !lexical.insert(name.as_str()) => {
                    return Err(ScriptError::at(
                        format!("duplicate block binding '{name}'"),
                        0,
                    ));
                }
                _ => {}
            }
        }
        let mut vars = BTreeSet::new();
        for statement in body {
            Self::var_names(statement, &mut vars);
        }
        if !block_functions {
            for statement in body {
                if let Stmt::Function(name, _) = statement {
                    vars.insert(name.as_str());
                }
            }
        }
        if let Some(name) = lexical.intersection(&vars).next() {
            return Err(ScriptError::at(
                format!("lexical and var declarations conflict for '{name}'"),
                0,
            ));
        }
        Ok(())
    }
    fn var_names<'a>(statement: &'a Stmt, names: &mut BTreeSet<&'a str>) {
        match statement {
            Stmt::Var(bindings, DeclarationKind::Var) => {
                names.extend(bindings.iter().map(|(name, _)| name.as_str()))
            }
            Stmt::Block(body) => {
                for statement in body {
                    Self::var_names(statement, names);
                }
            }
            Stmt::If(_, yes, no) => {
                Self::var_names(yes, names);
                if let Some(no) = no {
                    Self::var_names(no, names);
                }
            }
            Stmt::While(_, body) | Stmt::DoWhile(_, body) => Self::var_names(body, names),
            Stmt::For(init, _, _, body) => {
                if let Some(init) = init {
                    Self::var_names(init, names);
                }
                Self::var_names(body, names);
            }
            Stmt::ForIn(binding, _, body) => {
                if let ForBinding::Declaration(name, DeclarationKind::Var) = binding {
                    names.insert(name);
                }
                Self::var_names(body, names);
            }
            Stmt::Switch(_, cases) => {
                for (_, body) in cases {
                    for statement in body {
                        Self::var_names(statement, names);
                    }
                }
            }
            Stmt::Try(body, handler, finalizer) => {
                Self::var_names(body, names);
                if let Some(handler) = handler {
                    for statement in &handler.body {
                        Self::var_names(statement, names);
                    }
                }
                if let Some(finalizer) = finalizer {
                    Self::var_names(finalizer, names);
                }
            }
            _ => {}
        }
    }
    fn object_key(&mut self) -> Result<PropertyName> {
        if self.eat("[") {
            // ComputedPropertyName uses AssignmentExpression[+In], rather
            // than Expression: commas need an explicit pair of parentheses.
            let saved = self.allow_in;
            self.allow_in = true;
            let expression = self.expression()?;
            self.expect("]")?;
            self.allow_in = saved;
            return Ok(PropertyName::Computed(Box::new(expression)));
        }
        if self.strict && self.tokens[self.pos].legacy_literal {
            return Err(self.error("legacy literals are forbidden in strict code"));
        }
        let (key, identifier) = match self.tokens[self.pos].kind.clone() {
            TokenKind::Word(value) => (value.into(), true),
            TokenKind::String(value) => (value, false),
            TokenKind::Number(value) => (json_number(value).into(), false),
            _ => return Err(self.error("expected object property")),
        };
        self.pos += 1;
        Ok(PropertyName::Literal(key, identifier))
    }
    fn function(&mut self) -> Result<FunctionCode> {
        self.expect("(")?;
        let mut params = Vec::new();
        if !self.eat(")") {
            loop {
                params.push(self.binding_identifier()?);
                if self.is("=") {
                    return Err(ScriptError::unsupported(
                        "default parameters are not implemented",
                    ));
                }
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        self.expect("{")?;
        let saved = (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
        );
        self.loop_depth = 0;
        self.switch_depth = 0;
        self.allow_in = true;
        self.function_depth += 1;
        let body = self.directive_body(true)?;
        Self::check_scope(&body, false)?;
        self.function_depth -= 1;
        let strict = self.strict;
        for param in &params {
            self.validate_identifier(param, true)?;
        }
        if strict && params.iter().collect::<BTreeSet<_>>().len() != params.len() {
            return Err(self.error("duplicate strict function parameter"));
        }
        Self::check_parameter_lexicals(&params, &body)?;
        (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
        ) = saved;
        Ok(FunctionCode {
            params,
            body: Rc::new(body),
            name: None,
            arrow: false,
            self_name: false,
            constructable: true,
            strict,
        })
    }
    fn arrow(&mut self, params: Vec<String>) -> Result<Expr> {
        let saved = (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
        );
        self.loop_depth = 0;
        self.switch_depth = 0;
        self.allow_in = true;
        self.function_depth += 1;
        let body = if self.eat("{") {
            self.directive_body(true)?
        } else {
            vec![Stmt::Return(Some(self.expression()?))]
        };
        let strict = self.strict;
        Self::check_scope(&body, false)?;
        Self::check_parameter_lexicals(&params, &body)?;
        for param in &params {
            self.validate_identifier(param, true)?;
        }
        if params.iter().collect::<BTreeSet<_>>().len() != params.len() {
            return Err(self.error("duplicate arrow parameter"));
        }
        self.function_depth -= 1;
        (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
        ) = saved;
        Ok(Expr::Function(FunctionCode {
            params,
            body: Rc::new(body),
            name: None,
            arrow: true,
            self_name: false,
            constructable: false,
            strict,
        }))
    }
    fn expression(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.assignment();
        self.depth -= 1;
        result
    }
    fn check_parameter_lexicals(params: &[String], body: &[Stmt]) -> Result<()> {
        for statement in body {
            if let Stmt::Var(bindings, kind) = statement
                && *kind != DeclarationKind::Var
                && bindings.iter().any(|(name, _)| params.contains(name))
            {
                return Err(ScriptError::at(
                    "parameter conflicts with lexical declaration",
                    0,
                ));
            }
        }
        Ok(())
    }
    fn sequence(&mut self) -> Result<Expr> {
        let first = self.expression()?;
        if !self.eat(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        loop {
            items.push(self.expression()?);
            if !self.eat(",") {
                break;
            }
        }
        Ok(Expr::Sequence(items))
    }
    fn assignment(&mut self) -> Result<Expr> {
        let mut left = self.binary(1)?;
        if self.eat("?") {
            let yes = self.expression()?;
            self.expect(":")?;
            let no = self.expression()?;
            left = Expr::Conditional(Box::new(left), Box::new(yes), Box::new(no));
        }
        for operator in ["=", "+=", "-=", "*=", "/=", "%=", "**="] {
            if self.eat(operator) {
                self.assignment_target(&left)?;
                return Ok(Expr::Assign(
                    operator.to_owned(),
                    Box::new(left),
                    Box::new(self.expression()?),
                ));
            }
        }
        Ok(left)
    }
    fn binary(&mut self, min_precedence: u8) -> Result<Expr> {
        self.enter()?;
        let result = self.binary_inner(min_precedence);
        self.depth -= 1;
        result
    }
    fn binary_inner(&mut self, min_precedence: u8) -> Result<Expr> {
        let left = self.unary()?;
        let mut operations = Vec::new();
        let mut chain = 0;
        while let TokenKind::Symbol(s) | TokenKind::Word(s) = &self.tokens[self.pos].kind {
            let op = s.clone();
            if op == "in" && !self.allow_in {
                break;
            }
            let precedence = match op.as_str() {
                "||" | "??" => 1,
                "&&" => 2,
                "|" => 3,
                "^" => 4,
                "&" => 5,
                "==" | "!=" | "===" | "!==" => 6,
                "<" | ">" | "<=" | ">=" | "instanceof" | "in" => 7,
                "<<" | ">>" | ">>>" => 8,
                "+" | "-" => 9,
                "*" | "/" | "%" => 10,
                "**" => 11,
                _ => break,
            };
            if precedence < min_precedence {
                break;
            }
            chain += 1;
            if chain > 4096 {
                return Err(self.resource_error("expression chain limit exceeded"));
            }
            self.pos += 1;
            let right = self.binary(if op == "**" {
                precedence
            } else {
                precedence + 1
            })?;
            operations.push((op, right));
        }
        // Folding an all-literal string addition chain preserves evaluation
        // order and avoids constructing every quadratic intermediate string.
        if let Expr::Literal(Value::String(first)) = &left
            && !operations.is_empty()
            && operations
                .iter()
                .all(|(op, value)| op == "+" && matches!(value, Expr::Literal(Value::String(_))))
        {
            let length = operations.iter().fold(first.len(), |length, (_, value)| {
                if let Expr::Literal(Value::String(text)) = value {
                    length.saturating_add(text.len())
                } else {
                    unreachable!()
                }
            });
            if length > MAX_STRING {
                return Err(self.resource_error("script string limit exceeded"));
            }
            self.compile_budget
                .work(length / 8 + 1)
                .map_err(regexp_error)?;
            self.compile_budget.allocated =
                self.compile_budget.allocated.saturating_add(length * 4);
            if self.compile_budget.allocated > MAX_HEAP {
                return Err(self.resource_error("script allocation limit exceeded"));
            }
            let mut units = Vec::with_capacity(length);
            units.extend_from_slice(first.units());
            for (_, value) in operations {
                let Expr::Literal(Value::String(text)) = value else {
                    unreachable!()
                };
                units.extend_from_slice(text.units());
            }
            return Ok(Expr::Literal(Value::String(units.into())));
        }
        Ok(if operations.is_empty() {
            left
        } else {
            Expr::BinaryChain(Box::new(left), operations)
        })
    }
    fn unary(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.unary_inner();
        self.depth -= 1;
        result
    }
    fn unary_inner(&mut self) -> Result<Expr> {
        for op in ["!", "-", "+", "~", "typeof", "void", "delete"] {
            if self.eat(op) {
                let value = self.unary()?;
                if self.strict
                    && op == "delete"
                    && matches!(&value, Expr::Ident(name) if name != "this")
                {
                    return Err(self.error("strict code cannot delete an identifier"));
                }
                return Ok(Expr::Unary(op.to_owned(), Box::new(value)));
            }
        }
        if self.is("++") || self.is("--") {
            let delta = if self.eat("++") {
                1.0
            } else {
                self.pos += 1;
                -1.0
            };
            let value = self.unary()?;
            self.assignment_target(&value)?;
            return Ok(Expr::Update(Box::new(value), delta, true));
        }
        let mut value = self.new_expression()?;
        let mut chain = 0;
        loop {
            chain += 1;
            if chain > MAX_DEPTH / 2 {
                return Err(self.resource_error("expression chain limit exceeded"));
            }
            if self.eat(".") {
                let property = self.identifier()?;
                value = Expr::Member(
                    Box::new(value),
                    Box::new(Expr::Literal(Value::String(JsString::from(property)))),
                );
            } else if self.eat("[") {
                let property = self.expression()?;
                self.expect("]")?;
                value = Expr::Member(Box::new(value), Box::new(property));
            } else if self.eat("(") {
                let mut arguments = Vec::new();
                if !self.eat(")") {
                    loop {
                        arguments.push(self.expression()?);
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                value = Expr::Call(Box::new(value), arguments);
            } else if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
                return Err(ScriptError::unsupported(
                    "tagged template literals are not implemented",
                ));
            } else {
                break;
            }
        }
        if !self.tokens[self.pos].line_break_before && self.eat("++") {
            self.assignment_target(&value)?;
            value = Expr::Update(Box::new(value), 1.0, false);
        } else if !self.tokens[self.pos].line_break_before && self.eat("--") {
            self.assignment_target(&value)?;
            value = Expr::Update(Box::new(value), -1.0, false);
        }
        Ok(value)
    }
    fn primary(&mut self) -> Result<Expr> {
        if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
            return self.template_literal();
        }
        if self.is("/") || self.is("/=") {
            return self.regexp_literal();
        }
        if self.eat("(") {
            let saved = self.pos;
            let mut params = Vec::new();
            let mut arrow = false;
            if self.eat(")") {
                arrow = self.eat("=>");
            } else {
                while let TokenKind::Word(name) = &self.tokens[self.pos].kind {
                    params.push(name.clone());
                    self.pos += 1;
                    if self.eat(")") {
                        arrow = self.eat("=>");
                        break;
                    }
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            if arrow {
                return self.arrow(params);
            }
            self.pos = saved;
            let value = self.sequence()?;
            self.expect(")")?;
            return Ok(value);
        }
        if self.eat("[") {
            let saved = self.allow_in;
            self.allow_in = true;
            let mut items = Vec::new();
            while !self.eat("]") {
                if self.eat(",") {
                    items.push(None);
                    continue;
                }
                items.push(Some(self.expression()?));
                if self.eat("]") {
                    break;
                }
                self.expect(",")?;
            }
            self.allow_in = saved;
            return Ok(Expr::Array(items));
        }
        if self.eat("{") {
            let saved = self.allow_in;
            self.allow_in = true;
            let mut entries = Vec::new();
            let mut prototype_setter = false;
            while !self.eat("}") {
                if self.is("*") || self.is(".") {
                    return Err(ScriptError::unsupported(
                        "generator methods and object spread are not implemented",
                    ));
                }
                self.compile_budget.work(1).map_err(regexp_error)?;
                self.compile_budget.allocated = self
                    .compile_budget
                    .allocated
                    .saturating_add(2 * std::mem::size_of::<(PropertyName, ObjectEntry)>());
                if self.compile_budget.allocated > MAX_HEAP {
                    return Err(self.resource_error("object literal allocation limit exceeded"));
                }
                let mut key = self.object_key()?;
                let accessor = matches!(&key, PropertyName::Literal(name, true)
                    if name == &JsString::from("get") || name == &JsString::from("set"));
                if matches!(&key, PropertyName::Literal(name, true) if name == &JsString::from("async"))
                    && !self.is(":")
                    && !self.is("(")
                    && !self.is(",")
                    && !self.is("}")
                    && !self.tokens[self.pos].line_break_before
                {
                    return Err(ScriptError::unsupported(
                        "async methods are not implemented",
                    ));
                }
                let value = if accessor
                    && !self.is(":")
                    && !self.is("(")
                    && !self.is(",")
                    && !self.is("}")
                {
                    let setter = matches!(&key, PropertyName::Literal(name, _) if name == &JsString::from("set"));
                    key = self.object_key()?;
                    let mut code = self.function()?;
                    if code.params.len() != usize::from(setter) {
                        return Err(self.error("invalid accessor parameter count"));
                    }
                    code.constructable = false;
                    ObjectEntry::Accessor(code, setter)
                } else if self.is("(") {
                    let mut code = self.function()?;
                    if code.params.iter().collect::<BTreeSet<_>>().len() != code.params.len() {
                        return Err(self.error("duplicate method parameter"));
                    }
                    code.constructable = false;
                    ObjectEntry::Method(code)
                } else {
                    if self.eat(":") {
                        let is_prototype = matches!(&key, PropertyName::Literal(name, _) if name == &JsString::from("__proto__"));
                        if is_prototype && prototype_setter {
                            return Err(self.error("duplicate __proto__ prototype setter"));
                        }
                        let expression = self.expression()?;
                        if is_prototype {
                            prototype_setter = true;
                            ObjectEntry::Prototype(expression)
                        } else {
                            ObjectEntry::Data(expression)
                        }
                    } else {
                        let PropertyName::Literal(name, true) = &key else {
                            return Err(self.error("object shorthand requires an identifier"));
                        };
                        let name = name
                            .to_utf8()
                            .map_err(|_| self.error("object shorthand requires an identifier"))?;
                        self.validate_identifier(&name, false)?;
                        ObjectEntry::Data(Expr::Ident(name))
                    }
                };
                entries.push((key, value));
                if self.eat("}") {
                    break;
                }
                self.expect(",")?;
            }
            self.allow_in = saved;
            return Ok(Expr::Object(entries));
        }
        if self.eat("function") {
            let name = if matches!(self.tokens[self.pos].kind, TokenKind::Word(_)) {
                Some(self.binding_identifier()?)
            } else {
                None
            };
            let mut code = self.function()?;
            if let Some(name) = &name {
                let saved = self.strict;
                self.strict = code.strict;
                self.validate_identifier(name, true)?;
                self.strict = saved;
            }
            code.name = name;
            code.self_name = code.name.is_some();
            return Ok(Expr::Function(code));
        }
        let token = self.tokens[self.pos].clone();
        if self.strict && token.legacy_literal {
            return Err(ScriptError::at(
                "legacy literals are forbidden in strict code",
                token.offset,
            ));
        }
        if !self.done() {
            self.pos += 1;
        }
        match token.kind {
            TokenKind::RegExp(pattern) => Ok(Expr::RegExp(pattern)),
            TokenKind::Invalid(error) => Err(error),
            TokenKind::Number(n) => Ok(Expr::Literal(Value::Number(n))),
            TokenKind::String(s) => Ok(Expr::Literal(Value::String(s))),
            TokenKind::Word(s) if s == "true" || s == "false" => {
                Ok(Expr::Literal(Value::Bool(s == "true")))
            }
            TokenKind::Word(s) if s == "null" => Ok(Expr::Literal(Value::Null)),
            TokenKind::Word(s) if s == "super" => Err(ScriptError::unsupported(
                "super property and constructor references are not implemented",
            )),
            TokenKind::Word(s) => {
                if self.eat("=>") {
                    self.arrow(vec![s])
                } else {
                    if s != "this" {
                        self.validate_identifier(&s, false)?;
                    }
                    Ok(Expr::Ident(s))
                }
            }
            _ => Err(ScriptError::at("expected expression", token.offset)),
        }
    }
    fn template_literal(&mut self) -> Result<Expr> {
        let start = self.tokens[self.pos].offset;
        let (head, mut at, _, mut interpolation) =
            quoted_text(&self.source, start, '`', Some(&mut self.compile_budget))?;
        self.pos += 1;
        self.rescan_suffix(at)?;
        let mut tail = Vec::new();
        while interpolation {
            if tail.len() >= 4096 {
                return Err(self.resource_error("template substitution limit exceeded"));
            }
            self.compile_budget.allocated = self
                .compile_budget
                .allocated
                .saturating_add(std::mem::size_of::<(Expr, JsString)>() * 2);
            if self.compile_budget.allocated > MAX_HEAP {
                return Err(self.resource_error("template storage limit exceeded"));
            }
            let saved = self.allow_in;
            self.allow_in = true;
            let expression = self.sequence();
            self.allow_in = saved;
            let expression = expression?;
            let start = self.tokens[self.pos].offset;
            self.expect("}")?;
            let (text, end, _, next) =
                quoted_text(&self.source, start, '`', Some(&mut self.compile_budget))?;
            tail.push((expression, text));
            at = end;
            interpolation = next;
            self.rescan_suffix(at)?;
        }
        Ok(if tail.is_empty() {
            Expr::Literal(Value::String(head))
        } else {
            Expr::Template(head, tail)
        })
    }
    fn rescan_suffix(&mut self, at: usize) -> Result<()> {
        self.tokens.truncate(self.pos);
        self.lex_work = self.lex_work.saturating_add(self.source.len() - at);
        if self.lex_work > MAX_SOURCE * 32 {
            return Err(ScriptError::resource(
                "script lexical rescan limit exceeded",
            ));
        }
        let suffix = lex(&self.source[at..])?;
        if self.tokens.len().saturating_add(suffix.len()) > MAX_TOKENS {
            return Err(ScriptError::resource("script token limit exceeded"));
        }
        self.tokens.extend(suffix.into_iter().map(|mut token| {
            token.offset += at;
            if let TokenKind::Invalid(error) = &mut token.kind
                && let Some(offset) = &mut error.offset
            {
                *offset += at;
            }
            token
        }));
        Ok(())
    }
    fn regexp_literal(&mut self) -> Result<Expr> {
        let start = self.tokens[self.pos].offset;
        let mut at = start + 1;
        let mut class = false;
        let mut escaped = false;
        let mut end = None;
        while at < self.source.len() {
            let ch = self.source[at..].chars().next().unwrap();
            if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                return Err(ScriptError::at("line terminator in RegExp literal", at));
            }
            if !escaped {
                if ch == '/' && !class {
                    end = Some(at);
                    break;
                }
                if ch == '[' {
                    class = true;
                }
                if ch == ']' {
                    class = false;
                }
            }
            escaped = !escaped && ch == '\\';
            at += ch.len_utf8();
        }
        let end = end.ok_or_else(|| ScriptError::at("unterminated RegExp literal", start))?;
        at = end + 1;
        while at < self.source.len() {
            let ch = self.source[at..].chars().next().unwrap();
            if !(ch.is_alphanumeric() || matches!(ch, '_' | '$' | '\\')) {
                break;
            }
            at += ch.len_utf8();
        }
        let pattern = RegExp::compile(
            self.source[start + 1..end].into(),
            &self.source[end + 1..at].into(),
            &mut self.compile_budget,
        )
        .map_err(|error| {
            let mut error = regexp_error(error);
            if !error.is_resource_limit() && !error.is_unsupported() {
                error.offset = Some(start);
            }
            error
        })?;
        let pattern = Rc::new(pattern);
        self.tokens[self.pos].kind = TokenKind::RegExp(pattern.clone());
        self.pos += 1;
        self.rescan_suffix(at)?;
        Ok(Expr::RegExp(pattern))
    }
    fn new_expression(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.new_expression_inner();
        self.depth -= 1;
        result
    }
    fn new_expression_inner(&mut self) -> Result<Expr> {
        if !self.eat("new") {
            return self.primary();
        }
        let mut constructor = self.new_expression()?;
        let mut count = 0;
        loop {
            count += 1;
            if count > MAX_DEPTH / 2 {
                return Err(self.resource_error("constructor chain limit exceeded"));
            }
            if self.eat(".") {
                let key = self.identifier()?;
                constructor = Expr::Member(
                    Box::new(constructor),
                    Box::new(Expr::Literal(Value::String(key.into()))),
                );
            } else if self.eat("[") {
                let key = self.expression()?;
                self.expect("]")?;
                constructor = Expr::Member(Box::new(constructor), Box::new(key));
            } else if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
                return Err(ScriptError::unsupported(
                    "tagged template literals are not implemented",
                ));
            } else {
                break;
            }
        }
        let mut args = Vec::new();
        if self.eat("(") && !self.eat(")") {
            loop {
                args.push(self.expression()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(Expr::New(Box::new(constructor), args))
    }
}

#[derive(Clone)]
struct Binding {
    value: Value,
    mutable: bool,
    initialized: bool,
    strict_immutable: bool,
    global_property: bool,
    enumerable: bool,
    deletable: bool,
}
struct Environment {
    bindings: BTreeMap<String, Binding>,
    parent: Option<usize>,
    function_scope: bool,
    strict: bool,
}
#[derive(Clone)]
struct Function {
    code: FunctionCode,
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
    Normal(Value),
    Return(Value),
    Break,
    Continue,
}
enum Reference {
    Binding(usize, String, bool),
    Unresolvable(String, bool),
    Property(Value, JsString, bool),
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
    fn accessor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }
    fn data(&self) -> bool {
        self.value.is_some() || self.writable.is_some()
    }
}
#[derive(Default)]
struct ScriptObject {
    values: BTreeMap<JsString, Property>,
    order: Vec<JsString>,
    prototype: Option<Value>,
    boxed: Option<Value>,
    non_extensible: bool,
    intrinsic_error: Option<&'static str>,
    parameter_map: BTreeMap<JsString, (usize, String)>,
    arguments: bool,
    regexp: Option<Rc<RegExp>>,
    event: Option<usize>,
    event_target: bool,
    abort: Option<AbortSlot>,
    namespace: Option<&'static str>,
}
impl ScriptObject {
    fn get(&self, key: impl Into<JsString>) -> Option<&Value> {
        self.values
            .get(&key.into())
            .and_then(|property| match &property.value {
                PropertyValue::Data { value, .. } => Some(value),
                PropertyValue::Accessor { .. } => None,
            })
    }
    fn contains_key(&self, key: impl Into<JsString>) -> bool {
        self.values.contains_key(&key.into())
    }
    fn insert(&mut self, key: JsString, value: Value) {
        self.insert_property(key, Property::data(value, true, true, true));
    }
    fn insert_property(&mut self, key: JsString, property: Property) {
        if !self.values.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.values.insert(key, property);
    }
    fn remove(&mut self, key: &JsString) {
        self.values.remove(key);
        self.order.retain(|item| item != key);
    }
    fn insert_hidden(&mut self, key: JsString, value: Value) {
        self.insert_property(key, Property::data(value, true, false, true));
    }
    fn attributes(&mut self, key: &str, writable: bool, enumerable: bool, configurable: bool) {
        if let Some(property) = self.values.get_mut(&JsString::from(key)) {
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
    environments: Vec<Environment>,
    functions: Vec<Function>,
    arrays: Vec<Vec<Value>>,
    array_properties: Vec<usize>,
    array_holes: Vec<BTreeSet<usize>>,
    array_prototype: Option<usize>,
    objects: Vec<ScriptObject>,
    prototypes: BTreeMap<&'static str, usize>,
    native_properties: BTreeMap<String, usize>,
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
    eval_depth: usize,
    json_depth: usize,
    stack_units: usize,
    pub console: Vec<String>,
    pub last_default_prevented: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
impl Runtime {
    pub fn new() -> Self {
        let mut bindings = BTreeMap::new();
        for (name, value) in [
            ("undefined", Value::Undefined),
            ("NaN", Value::Number(f64::NAN)),
            ("Infinity", Value::Number(f64::INFINITY)),
            ("document", Value::Document),
            ("window", Value::Window),
            ("self", Value::Window),
            ("globalThis", Value::Window),
            ("this", Value::Window),
            ("console", Value::Console),
            ("Math", Value::Math),
            ("JSON", Value::Json),
        ] {
            bindings.insert(
                name.to_owned(),
                Binding {
                    value,
                    mutable: false,
                    initialized: true,
                    strict_immutable: false,
                    global_property: name != "this",
                    enumerable: true,
                    deletable: false,
                },
            );
        }
        for name in [
            "String",
            "Number",
            "Boolean",
            "parseInt",
            "parseFloat",
            "isNaN",
            "isFinite",
            "Object",
            "Array",
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
        ] {
            bindings.insert(
                name.to_owned(),
                Binding {
                    value: Self::native(name, Value::Window),
                    mutable: true,
                    initialized: true,
                    strict_immutable: false,
                    global_property: true,
                    enumerable: true,
                    deletable: true,
                },
            );
        }
        let mut runtime = Self {
            environments: vec![
                Environment {
                    bindings,
                    parent: None,
                    function_scope: true,
                    strict: false,
                },
                Environment {
                    bindings: BTreeMap::new(),
                    parent: Some(0),
                    function_scope: false,
                    strict: false,
                },
            ],
            functions: Vec::new(),
            arrays: Vec::new(),
            array_properties: Vec::new(),
            array_holes: Vec::new(),
            array_prototype: None,
            objects: Vec::new(),
            prototypes: BTreeMap::new(),
            native_properties: BTreeMap::new(),
            function_prototype: 0,
            events: Vec::new(),
            abort_signals: Vec::new(),
            listeners: Vec::new(),
            event_listeners: BTreeMap::new(),
            event_handlers: BTreeMap::new(),
            readiness_fired: false,
            started: std::time::Instant::now(),
            steps: MAX_STEPS,
            allocated: 2048,
            calls: 0,
            eval_depth: 0,
            json_depth: 0,
            stack_units: 0,
            console: Vec::new(),
            last_default_prevented: false,
        };
        runtime
            .initialize_intrinsics()
            .expect("fixed intrinsic bootstrap fits runtime limits");
        runtime
    }

    pub fn parse_only(source: &str) -> Result<()> {
        Parser::program(source).map(|_| ())
    }
    /// Parse a script using a caller-supplied strict parse goal without altering
    /// its source bytes (used by the unchanged Test262 mode adapter).
    pub fn parse_only_strict(source: &str) -> Result<()> {
        Parser::program_context(source, false, true).map(|_| ())
    }

    fn initialize_intrinsics(&mut self) -> Result<()> {
        for name in [
            "Object",
            "Function",
            "Array",
            "String",
            "Number",
            "Boolean",
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
        self.functions.push(Function {
            code: FunctionCode {
                params: Vec::new(),
                body: Rc::new(Vec::new()),
                name: None,
                arrow: true,
                self_name: false,
                constructable: false,
                strict: true,
            },
            environment: 0,
            properties: self.prototypes["Function"],
            bound: None,
        });
        self.objects[self.prototypes["Function"]]
            .insert_hidden("name".into(), Value::String(JsString::default()));
        self.objects[self.prototypes["Function"]]
            .insert_hidden("length".into(), Value::Number(0.0));
        let thrower = Self::native("ThrowTypeError", Value::Undefined);
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
        let Value::Array(array_prototype) = self.array(Vec::new())? else {
            unreachable!()
        };
        self.array_properties[array_prototype] = self.prototypes["Array"];
        self.array_prototype = Some(array_prototype);
        for name in [
            "Object",
            "Function",
            "Array",
            "String",
            "Number",
            "Boolean",
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
            let constructor = Self::native(name, Value::Window);
            let prototype = self.prototypes[name];
            let Value::Object(properties) = self.object_ordered([
                ("name".into(), Value::String(name.into())),
                (
                    "length".into(),
                    Value::Number(match name {
                        "RegExp" => 2.0,
                        "EventTarget" | "DOMException" | "AbortController" | "AbortSignal" => 0.0,
                        _ => 1.0,
                    }),
                ),
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
            self.objects[id].insert_hidden(key.into(), Self::native(method, Value::Undefined));
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
            self.objects[properties].insert_hidden(
                method.into(),
                Self::native(&format!("Object.{method}"), Value::Undefined),
            );
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
            ("String", "charAt", 1),
            ("String", "charCodeAt", 1),
            ("String", "codePointAt", 1),
            ("String", "slice", 2),
            ("String", "substring", 2),
            ("String", "includes", 1),
            ("String", "startsWith", 1),
            ("String", "endsWith", 1),
            ("String", "indexOf", 1),
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
            ("Array", "filter", 1),
            ("Array", "includes", 1),
            ("Array", "indexOf", 1),
            ("Array", "slice", 2),
            ("Array", "reverse", 0),
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
        ] {
            let full = format!("{owner}.{key}");
            let value = self.intrinsic_function(&full, key, length)?;
            self.objects[self.native_properties[owner]].insert_hidden(key.into(), value);
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
        let supports = self.intrinsic_function("CSS.supports", "supports", 1)?;
        let namespace = self.object_ordered([("supports".into(), supports)])?;
        if let Value::Object(id) = namespace {
            self.objects[id].namespace = Some("CSS");
        }
        self.charge(128)?;
        self.environments[0].bindings.insert(
            "CSS".into(),
            Binding {
                value: namespace,
                mutable: true,
                initialized: true,
                strict_immutable: false,
                global_property: true,
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
        Ok(())
    }

    fn intrinsic_function(&mut self, full: &str, name: &str, length: usize) -> Result<Value> {
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
        let program = Parser::program(source)?;
        self.execute_program(source, program, document)
    }
    pub fn execute_strict(&mut self, source: &str, document: &mut Document) -> Result<Value> {
        let program = Parser::program_context(source, false, true)?;
        self.execute_program(source, program, document)
    }
    fn execute_program(
        &mut self,
        source: &str,
        program: Program,
        document: &mut Document,
    ) -> Result<Value> {
        self.charge(source.len().saturating_mul(3))?;
        self.charge(program.compiled_storage)?;
        self.steps = MAX_STEPS;
        let saved = self.environments[1].strict;
        self.environments[1].strict = program.strict;
        let completion = self.statements(&program.body, 1, document);
        self.environments[1].strict = saved;
        match completion? {
            Flow::Normal(value) => Ok(value),
            Flow::Return(_) => Err(ScriptError::new("return outside function")),
            _ => Err(ScriptError::new("loop control outside loop")),
        }
    }

    fn initialize_events(&mut self) -> Result<()> {
        self.objects[self.native_properties["CustomEvent"]].prototype =
            Some(Self::native("Event", Value::Window));
        self.objects[self.native_properties["ToggleEvent"]].prototype =
            Some(Self::native("Event", Value::Window));
        self.objects[self.native_properties["AbortSignal"]].prototype =
            Some(Self::native("EventTarget", Value::Window));
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
        Ok(())
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
        self.objects[id].insert_property(
            "isTrusted".into(),
            Property {
                value: PropertyValue::Accessor {
                    get: Self::native("Event.get.isTrusted", Value::Undefined),
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
            Ok("InvalidStateError") => 11.0,
            Ok("NotSupportedError") => 9.0,
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
            let program = Parser::program_context(&source, true, false)?;
            self.charge(program.compiled_storage)?;
            self.function_value(
                &FunctionCode {
                    params: vec!["event".into()],
                    body: Rc::new(program.body),
                    name: Some(format!("on{kind}")),
                    arrow: false,
                    self_name: false,
                    constructable: false,
                    strict: program.strict,
                },
                1,
            )
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
        let id = self.arrays.len();
        self.arrays.push(values);
        let Value::Object(properties) = self.object_ordered([])? else {
            unreachable!()
        };
        self.objects[properties].prototype = self
            .array_prototype
            .map(Value::Array)
            .or_else(|| self.prototypes.get("Array").copied().map(Value::Object));
        self.array_properties.push(properties);
        self.array_holes.push(BTreeSet::new());
        Ok(Value::Array(id))
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
            let thrower = Self::native("ThrowTypeError", Value::Undefined);
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
        Ok(object)
    }
    fn object(&mut self, values: BTreeMap<String, Value>) -> Result<Value> {
        self.object_ordered(values.into_iter().map(|(key, value)| (key.into(), value)))
    }
    fn object_ordered(
        &mut self,
        values: impl IntoIterator<Item = (JsString, Value)>,
    ) -> Result<Value> {
        self.charge(72 + std::mem::size_of::<Option<AbortSlot>>())?;
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
        self.charge(128)?;
        let id = self.environments.len();
        self.environments.push(Environment {
            bindings: BTreeMap::new(),
            parent: Some(parent),
            function_scope: false,
            strict: self.environments[parent].strict,
        });
        Ok(id)
    }
    fn define(&mut self, env: usize, name: &str, value: Value, mutable: bool) -> Result<()> {
        if !self.environments[env].bindings.contains_key(name) {
            self.charge(name.len() + 128)?;
        } else if !self.environments[env].bindings[name].mutable {
            return Err(ScriptError::new(format!(
                "cannot redeclare constant '{name}'"
            )));
        }
        self.environments[env].bindings.insert(
            name.into(),
            Binding {
                value,
                mutable,
                initialized: true,
                strict_immutable: !mutable,
                global_property: env == 0,
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
    fn binding_value(&self, env: usize, name: &str) -> Result<Value> {
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
    fn instantiate_lexical(&mut self, body: &[Stmt], env: usize) -> Result<()> {
        for statement in body {
            if let Stmt::Var(bindings, kind) = statement
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
        for statement in body {
            if let Stmt::Var(bindings, kind) = statement
                && *kind != DeclarationKind::Var
            {
                for (name, _) in bindings {
                    self.charge(name.len() + 128)?;
                    self.environments[env].bindings.insert(
                        name.clone(),
                        Binding {
                            value: Value::Undefined,
                            mutable: *kind != DeclarationKind::Const,
                            initialized: false,
                            strict_immutable: *kind == DeclarationKind::Const,
                            global_property: false,
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
    fn hoist_vars(&mut self, body: &[Stmt], env: usize) -> Result<()> {
        let owner = self.var_scope(env);
        for statement in body {
            self.hoist_statement(statement, owner)?;
        }
        Ok(())
    }
    fn hoist_statement(&mut self, statement: &Stmt, owner: usize) -> Result<()> {
        self.enter_stack(1)?;
        let result = self.hoist_statement_inner(statement, owner);
        self.stack_units -= 1;
        result
    }
    fn hoist_statement_inner(&mut self, statement: &Stmt, owner: usize) -> Result<()> {
        self.tick()?;
        match statement {
            Stmt::Var(bindings, DeclarationKind::Var) => {
                for (name, _) in bindings {
                    if owner == 0 && self.environments[1].bindings.contains_key(name) {
                        return Err(ScriptError::syntax(format!(
                            "global lexical binding conflicts with var '{name}'"
                        )));
                    }
                    if !self.environments[owner].bindings.contains_key(name) {
                        self.define(owner, name, Value::Undefined, true)?;
                    }
                }
            }
            Stmt::Block(body) => self.hoist_vars(body, owner)?,
            Stmt::If(_, yes, no) => {
                self.hoist_statement(yes, owner)?;
                if let Some(no) = no {
                    self.hoist_statement(no, owner)?;
                }
            }
            Stmt::While(_, body) | Stmt::DoWhile(_, body) => self.hoist_statement(body, owner)?,
            Stmt::For(init, _, _, body) => {
                if let Some(init) = init {
                    self.hoist_statement(init, owner)?;
                }
                self.hoist_statement(body, owner)?;
            }
            Stmt::ForIn(binding, _, body) => {
                if let ForBinding::Declaration(name, DeclarationKind::Var) = binding
                    && !self.environments[owner].bindings.contains_key(name)
                {
                    self.define(owner, name, Value::Undefined, true)?;
                }
                self.hoist_statement(body, owner)?;
            }
            Stmt::Switch(_, cases) => {
                for (_, body) in cases {
                    self.hoist_vars(body, owner)?;
                }
            }
            Stmt::Try(body, handler, finalizer) => {
                self.hoist_statement(body, owner)?;
                if let Some(handler) = handler {
                    self.hoist_vars(&handler.body, owner)?;
                }
                if let Some(finalizer) = finalizer {
                    self.hoist_statement(finalizer, owner)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn function_value(&mut self, code: &FunctionCode, environment: usize) -> Result<Value> {
        self.charge_function_code_copy(code)?;
        let id = self.functions.len();
        let Value::Object(properties) = self.object_ordered([
            (
                "name".into(),
                Value::String(code.name.clone().unwrap_or_default().into()),
            ),
            ("length".into(), Value::Number(code.params.len() as f64)),
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
    fn charge_function_code_copy(&mut self, code: &FunctionCode) -> Result<()> {
        // Function bodies are shared, but Clone owns every parameter and name.
        // Charge the scan first, then the retained copies before any allocation.
        self.work(1 + code.params.len())?;
        let text_bytes = code
            .params
            .iter()
            .fold(0usize, |size, name| size.saturating_add(name.len()));
        let name_bytes = code.name.as_ref().map_or(0, String::len);
        self.work(1 + text_bytes.saturating_add(name_bytes) / 8)?;
        self.charge(
            128usize
                .saturating_add(
                    code.params
                        .len()
                        .saturating_mul(std::mem::size_of::<String>()),
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
    fn native(name: &str, receiver: Value) -> Value {
        Value::Native(Rc::new(Native {
            name: name.into(),
            receiver,
        }))
    }

    fn statements(&mut self, body: &[Stmt], env: usize, doc: &mut Document) -> Result<Flow> {
        self.instantiate_lexical(body, env)?;
        self.hoist_vars(body, env)?;
        for statement in body {
            if let Stmt::Function(name, code) = statement {
                if env == 1 && self.environments[1].bindings.contains_key(name) {
                    return Err(ScriptError::syntax(format!(
                        "global lexical binding conflicts with function '{name}'"
                    )));
                }
                let function = self.function_value(code, env)?;
                self.define(if env == 1 { 0 } else { env }, name, function, true)?;
            }
        }
        let mut last = Value::Undefined;
        for statement in body {
            match self.statement(statement, env, doc)? {
                Flow::Normal(value) => last = value,
                flow => return Ok(flow),
            }
        }
        Ok(Flow::Normal(last))
    }
    fn statement(&mut self, statement: &Stmt, env: usize, doc: &mut Document) -> Result<Flow> {
        self.enter_stack(1)?;
        let result = self.statement_inner(statement, env, doc);
        self.stack_units -= 1;
        result
    }
    fn statement_inner(
        &mut self,
        statement: &Stmt,
        env: usize,
        doc: &mut Document,
    ) -> Result<Flow> {
        self.tick()?;
        match statement {
            Stmt::Empty | Stmt::Function(_, _) => {}
            Stmt::Expr(expression) => return Ok(Flow::Normal(self.eval(expression, env, doc)?)),
            Stmt::Var(bindings, kind) => {
                let owner = if *kind == DeclarationKind::Var {
                    self.var_scope(env)
                } else {
                    env
                };
                for (name, expression) in bindings {
                    if *kind == DeclarationKind::Var && expression.is_none() {
                        continue;
                    }
                    let value = if let Some(expression) = expression {
                        self.eval(expression, env, doc)?
                    } else {
                        Value::Undefined
                    };
                    if *kind == DeclarationKind::Var {
                        let target = self.lookup(env, name).map_or(owner, |(owner, _)| owner);
                        self.write_reference(
                            Reference::Binding(target, name.clone(), self.environments[env].strict),
                            value,
                            doc,
                        )?;
                    } else if let Some(binding) = self.environments[owner].bindings.get_mut(name)
                        && !binding.initialized
                    {
                        binding.value = value;
                        binding.initialized = true;
                    } else {
                        self.define(owner, name, value, *kind != DeclarationKind::Const)?;
                        self.environments[owner]
                            .bindings
                            .get_mut(name)
                            .unwrap()
                            .global_property = false;
                    }
                }
            }
            Stmt::Block(body) => {
                let child = self.environment(env)?;
                return self.statements(body, child, doc);
            }
            Stmt::If(condition, yes, no) => {
                if self.eval(condition, env, doc)?.truthy() {
                    return self.statement(yes, env, doc);
                }
                if let Some(no) = no {
                    return self.statement(no, env, doc);
                }
            }
            Stmt::While(condition, body) => {
                while self.eval(condition, env, doc)?.truthy() {
                    self.tick()?;
                    match self.statement(body, env, doc)? {
                        Flow::Break => break,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                        _ => {}
                    }
                }
            }
            Stmt::DoWhile(condition, body) => loop {
                self.tick()?;
                match self.statement(body, env, doc)? {
                    Flow::Break => break,
                    Flow::Return(value) => return Ok(Flow::Return(value)),
                    _ => {}
                }
                if !self.eval(condition, env, doc)?.truthy() {
                    break;
                }
            },
            Stmt::For(init, condition, update, body) => {
                return self.for_loop(
                    init.as_deref(),
                    condition.as_ref(),
                    update.as_ref(),
                    body,
                    env,
                    doc,
                );
            }
            Stmt::ForIn(binding, expression, body) => {
                return self.for_in(binding, expression, body, env, doc);
            }
            Stmt::Switch(expression, cases) => {
                return self.switch_statement(expression, cases, env, doc);
            }
            Stmt::Return(expression) => {
                return Ok(Flow::Return(if let Some(expression) = expression {
                    self.eval(expression, env, doc)?
                } else {
                    Value::Undefined
                }));
            }
            Stmt::Throw(expression) => {
                let value = self.eval(expression, env, doc)?;
                let name = self.thrown_name(&value, doc)?;
                let intrinsic = self.thrown_intrinsic_name(&value, doc)?;
                let mut error = self.thrown_error(value)?;
                error.thrown_name = name;
                error.intrinsic_name = intrinsic;
                return Err(error);
            }
            Stmt::Try(body, handler, finalizer) => {
                let mut completion = self.statement(body, env, doc);
                if let Err(error) = completion {
                    // Quota exhaustion is a host termination, not a JavaScript
                    // exception. Running either handler could hide that failure.
                    if error.is_resource_limit() || error.is_unsupported() {
                        return Err(error);
                    }
                    completion = if let Some(handler) = handler {
                        let catch_env = self.environment(env)?;
                        if let Some(binding) = &handler.binding {
                            let value = self.exception_value(error)?;
                            self.define(catch_env, binding, value, true)?;
                        }
                        self.statements(&handler.body, catch_env, doc)
                    } else {
                        Err(error)
                    };
                }
                if completion
                    .as_ref()
                    .is_err_and(|error| error.is_resource_limit() || error.is_unsupported())
                {
                    return completion;
                }
                if let Some(finalizer) = finalizer {
                    match self.statement(finalizer, env, doc)? {
                        Flow::Normal(_) => {}
                        abrupt => return Ok(abrupt),
                    }
                }
                return completion;
            }
            Stmt::Break => return Ok(Flow::Break),
            Stmt::Continue => return Ok(Flow::Continue),
        }
        Ok(Flow::Normal(Value::Undefined))
    }
    fn for_loop(
        &mut self,
        init: Option<&Stmt>,
        condition: Option<&Expr>,
        update: Option<&Expr>,
        body: &Stmt,
        env: usize,
        doc: &mut Document,
    ) -> Result<Flow> {
        let mut child = self.environment(env)?;
        let mut names = Vec::new();
        if let Some(init) = init {
            self.instantiate_lexical(std::slice::from_ref(init), child)?;
            self.statement(init, child, doc)?;
            if let Stmt::Var(bindings, DeclarationKind::Let) = init {
                names.extend(bindings.iter().map(|(name, _)| name.clone()));
            }
        }
        if !names.is_empty() {
            child = self.iteration_environment(child, env, &names)?;
        }
        loop {
            self.tick()?;
            if let Some(condition) = condition
                && !self.eval(condition, child, doc)?.truthy()
            {
                break;
            }
            match self.statement(body, child, doc)? {
                Flow::Break => break,
                flow @ Flow::Return(_) => return Ok(flow),
                _ => {}
            }
            if !names.is_empty() {
                child = self.iteration_environment(child, env, &names)?;
            }
            if let Some(update) = update {
                self.eval(update, child, doc)?;
            }
        }
        Ok(Flow::Normal(Value::Undefined))
    }
    fn iteration_environment(
        &mut self,
        previous: usize,
        parent: usize,
        names: &[String],
    ) -> Result<usize> {
        let next = self.environment(parent)?;
        for name in names {
            self.charge(128 + name.len())?;
            let binding = self.environments[previous].bindings[name].clone();
            self.environments[next]
                .bindings
                .insert(name.clone(), binding);
        }
        Ok(next)
    }
    fn switch_statement(
        &mut self,
        expression: &Expr,
        cases: &[(Option<Expr>, Vec<Stmt>)],
        env: usize,
        doc: &mut Document,
    ) -> Result<Flow> {
        let value = self.eval(expression, env, doc)?;
        let child = self.environment(env)?;
        for (_, body) in cases {
            self.instantiate_lexical(body, child)?;
            for statement in body {
                if let Stmt::Function(name, code) = statement {
                    let function = self.function_value(code, child)?;
                    self.define(child, name, function, true)?;
                }
            }
        }
        let mut default = None;
        let mut start = None;
        for (index, (condition, _)) in cases.iter().enumerate() {
            self.tick()?;
            if let Some(condition) = condition {
                let case = self.eval(condition, child, doc)?;
                if self.binary_value("===", value.clone(), case, doc)? == Value::Bool(true) {
                    start = Some(index);
                    break;
                }
            } else {
                default = Some(index);
            }
        }
        let mut last = Value::Undefined;
        if let Some(start) = start.or(default) {
            for (_, body) in &cases[start..] {
                for statement in body {
                    match self.statement(statement, child, doc)? {
                        Flow::Normal(value) => last = value,
                        Flow::Break => return Ok(Flow::Normal(last)),
                        abrupt => return Ok(abrupt),
                    }
                }
            }
        }
        Ok(Flow::Normal(last))
    }
    fn for_in(
        &mut self,
        binding: &ForBinding,
        expression: &Expr,
        body: &Stmt,
        env: usize,
        doc: &mut Document,
    ) -> Result<Flow> {
        let expression_env = if let ForBinding::Declaration(name, kind) = binding
            && *kind != DeclarationKind::Var
        {
            let child = self.environment(env)?;
            self.define(
                child,
                name,
                Value::Undefined,
                *kind != DeclarationKind::Const,
            )?;
            self.environments[child]
                .bindings
                .get_mut(name)
                .unwrap()
                .initialized = false;
            child
        } else {
            env
        };
        let value = self.eval(expression, expression_env, doc)?;
        if matches!(value, Value::Null | Value::Undefined) {
            return Ok(Flow::Normal(Value::Undefined));
        }
        let mut cursor = Some(self.coerce_object(value)?);
        let mut visited = BTreeSet::new();
        let mut depth = 0;
        while let Some(object) = cursor {
            if depth >= MAX_DEPTH {
                return Err(ScriptError::resource("for-in prototype depth exceeded"));
            }
            depth += 1;
            for key in self.own_keys(&object)? {
                self.work(1 + key.len() / 8)?;
                if visited.contains(&key) {
                    continue;
                }
                let Some(property) = self.own_property(&object, &key) else {
                    continue;
                };
                self.charge(32 + key.byte_len())?;
                visited.insert(key.clone());
                if !property.enumerable {
                    continue;
                }
                let value = self.string(key)?;
                let scope = match binding {
                    ForBinding::Declaration(name, DeclarationKind::Var) => {
                        let owner = self.var_scope(env);
                        self.define(owner, name, value, true)?;
                        env
                    }
                    ForBinding::Declaration(name, kind) => {
                        let child = self.environment(env)?;
                        self.define(child, name, value, *kind != DeclarationKind::Const)?;
                        child
                    }
                    ForBinding::Target(target) => {
                        let reference = self.reference(target, env, doc)?;
                        self.write_reference(reference, value, doc)?;
                        env
                    }
                };
                match self.statement(body, scope, doc)? {
                    Flow::Break => return Ok(Flow::Normal(Value::Undefined)),
                    flow @ Flow::Return(_) => return Ok(flow),
                    _ => {}
                }
            }
            cursor = self.prototype_of(&object);
        }
        Ok(Flow::Normal(Value::Undefined))
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
        let units = if let Value::String(text) = &value {
            text.len()
        } else {
            0
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
        Ok((constructor == Self::native(name, Value::Window)).then_some(name))
    }
    fn eval(&mut self, expression: &Expr, env: usize, doc: &mut Document) -> Result<Value> {
        if self.eval_depth >= MAX_DEPTH {
            return Err(ScriptError::resource(
                "expression evaluation nesting limit exceeded",
            ));
        }
        self.enter_stack(1)?;
        self.eval_depth += 1;
        let result = self.eval_inner(expression, env, doc);
        self.eval_depth -= 1;
        self.stack_units -= 1;
        result
    }
    fn eval_inner(&mut self, expression: &Expr, env: usize, doc: &mut Document) -> Result<Value> {
        self.tick()?;
        match expression {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Template(head, tail) => {
                let mut output = Vec::new();
                self.append_template_text(&mut output, head)?;
                for (expression, text) in tail {
                    let value = self.eval(expression, env, doc)?;
                    let cooked = self.string_hint(value, doc)?;
                    self.append_template_text(&mut output, &cooked)?;
                    self.append_template_text(&mut output, text)?;
                }
                self.string(output)
            }
            Expr::RegExp(pattern) => self.regexp_object(pattern.clone()),
            Expr::Ident(name) => {
                let (owner, _) = self
                    .lookup(env, name)
                    .ok_or_else(|| ScriptError::reference(format!("'{name}' is not defined")))?;
                self.binding_value(owner, name)
            }
            Expr::Sequence(items) => {
                let mut last = Value::Undefined;
                for item in items {
                    last = self.eval(item, env, doc)?;
                }
                Ok(last)
            }
            Expr::Array(items) => {
                let mut values = Vec::new();
                let mut holes = BTreeSet::new();
                for (index, item) in items.iter().enumerate() {
                    values.push(if let Some(item) = item {
                        self.eval(item, env, doc)?
                    } else {
                        holes.insert(index);
                        Value::Undefined
                    });
                }
                self.charge(holes.len().saturating_mul(32))?;
                let array = self.array(values)?;
                let Value::Array(id) = array else {
                    unreachable!()
                };
                self.array_holes[id] = holes;
                Ok(array)
            }
            Expr::Object(items) => {
                let object = self.object_ordered([])?;
                for (key, entry) in items {
                    if let ObjectEntry::Prototype(expression) = entry {
                        let prototype = self.eval(expression, env, doc)?;
                        if js_object(&prototype) || prototype == Value::Null {
                            self.set_object_prototype(&object, prototype)?;
                        }
                        continue;
                    }
                    // ToPropertyKey precedes RHS evaluation and function
                    // creation. This runtime's supported keys are strings.
                    let key = match key {
                        PropertyName::Literal(key, _) => key.clone(),
                        PropertyName::Computed(expression) => {
                            let value = self.eval(expression, env, doc)?;
                            self.string_hint(value, doc)?
                        }
                    };
                    let mut desc = PropertyDescriptor {
                        enumerable: Some(true),
                        configurable: Some(true),
                        ..PropertyDescriptor::default()
                    };
                    match entry {
                        ObjectEntry::Data(expression) => {
                            let value = self.eval(expression, env, doc)?;
                            if matches!(expression, Expr::Function(code) if code.name.is_none()) {
                                self.set_function_name(&value, &key, None)?;
                            }
                            desc.value = Some(value);
                            desc.writable = Some(true);
                        }
                        ObjectEntry::Method(code) => {
                            let function = self.function_value(code, env)?;
                            self.set_function_name(&function, &key, None)?;
                            desc.value = Some(function);
                            desc.writable = Some(true);
                        }
                        ObjectEntry::Accessor(code, setter) => {
                            let function = self.function_value(code, env)?;
                            self.set_function_name(
                                &function,
                                &key,
                                Some(if *setter { "set" } else { "get" }),
                            )?;
                            if *setter {
                                desc.set = Some(function);
                            } else {
                                desc.get = Some(function);
                            }
                        }
                        ObjectEntry::Prototype(_) => unreachable!(),
                    }
                    self.define_own(&object, &key, desc)?;
                }
                Ok(object)
            }
            Expr::Unary(op, expression) => {
                if op == "delete" {
                    return match &**expression {
                        Expr::Member(object, key) => {
                            let object = self.eval(object, env, doc)?;
                            let value = self.eval(key, env, doc)?;
                            let key = self.json_text(value, doc, &mut Vec::new())?;
                            let deleted = self.delete_property(object, &key)?;
                            if !deleted && self.environments[env].strict {
                                return Err(ScriptError::type_error(
                                    "cannot delete a non-configurable property",
                                ));
                            }
                            Ok(Value::Bool(deleted))
                        }
                        Expr::Ident(name) if name == "this" => Ok(Value::Bool(true)),
                        Expr::Ident(name) => {
                            if let Some((owner, _)) = self.lookup(env, name) {
                                let removable = self.environments[owner].bindings[name].deletable;
                                if removable {
                                    self.environments[owner].bindings.remove(name);
                                }
                                Ok(Value::Bool(removable))
                            } else {
                                Ok(Value::Bool(true))
                            }
                        }
                        _ => {
                            self.eval(expression, env, doc)?;
                            Ok(Value::Bool(true))
                        }
                    };
                }
                if op == "typeof"
                    && let Expr::Ident(name) = &**expression
                    && self.lookup(env, name).is_none()
                {
                    return self.string("undefined");
                }
                let value = self.eval(expression, env, doc)?;
                if let Value::String(text) = &value {
                    self.work(1 + text.len() / 8)?;
                }
                match op.as_str() {
                    "!" => Ok(Value::Bool(!value.truthy())),
                    "-" => Ok(Value::Number(-self.number_value(value, doc)?)),
                    "+" => Ok(Value::Number(self.number_value(value, doc)?)),
                    "~" => Ok(Value::Number(
                        (!to_i32(self.number_value(value, doc)?)) as f64,
                    )),
                    "void" => Ok(Value::Undefined),
                    "typeof" => self.string(match value {
                        Value::Undefined => "undefined",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) => "number",
                        Value::String(_) => "string",
                        Value::Function(_) | Value::Native(_) => "function",
                        _ => "object",
                    }),
                    _ => Err(ScriptError::new("unknown unary operator")),
                }
            }
            Expr::BinaryChain(left, operations) => {
                let mut left = self.eval(left, env, doc)?;
                for (op, right) in operations {
                    self.tick()?;
                    if op == "&&" && !left.truthy()
                        || op == "||" && left.truthy()
                        || op == "??" && !matches!(left, Value::Undefined | Value::Null)
                    {
                        continue;
                    }
                    let right = self.eval(right, env, doc)?;
                    left = if matches!(op.as_str(), "&&" | "||" | "??") {
                        right
                    } else {
                        self.binary_value(op, left, right, doc)?
                    };
                }
                Ok(left)
            }
            Expr::Conditional(condition, yes, no) => {
                if self.eval(condition, env, doc)?.truthy() {
                    self.eval(yes, env, doc)
                } else {
                    self.eval(no, env, doc)
                }
            }
            Expr::Assign(op, left, right) => {
                let reference = self.reference(left, env, doc)?;
                let old = if op != "=" {
                    Some(self.read_reference(&reference, doc)?)
                } else {
                    None
                };
                let mut value = self.eval(right, env, doc)?;
                if let Some(old) = old {
                    value = self.binary_value(&op[..op.len() - 1], old, value, doc)?;
                }
                self.write_reference(reference, value.clone(), doc)?;
                Ok(value)
            }
            Expr::Update(target, delta, prefix) => {
                let reference = self.reference(target, env, doc)?;
                let previous = self.read_reference(&reference, doc)?;
                if let Value::String(text) = &previous {
                    self.work(1 + text.len() / 8)?;
                }
                let old = self.number_value(previous, doc)?;
                let value = Value::Number(old + delta);
                self.write_reference(reference, value.clone(), doc)?;
                Ok(if *prefix { value } else { Value::Number(old) })
            }
            Expr::Member(object, property) => {
                let object = self.eval(object, env, doc)?;
                let value = self.eval(property, env, doc)?;
                let property = self.json_text(value, doc, &mut Vec::new())?;
                self.get_key(object, &property, doc)
            }
            Expr::Call(callee, arguments) => {
                let (function, receiver) = if let Expr::Member(object, property) = &**callee {
                    let receiver = self.eval(object, env, doc)?;
                    let value = self.eval(property, env, doc)?;
                    let property = self.json_text(value, doc, &mut Vec::new())?;
                    (self.get_key(receiver.clone(), &property, doc)?, receiver)
                } else {
                    (self.eval(callee, env, doc)?, Value::Undefined)
                };
                let arguments = arguments
                    .iter()
                    .map(|argument| self.eval(argument, env, doc))
                    .collect::<Result<Vec<_>>>()?;
                self.call(function, arguments, receiver, doc)
            }
            Expr::New(callee, arguments) => {
                let constructor = self.eval(callee, env, doc)?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.eval(argument, env, doc))
                    .collect::<Result<Vec<_>>>()?;
                self.construct(constructor, arguments, doc)
            }
            Expr::Function(code) => self.function_value(code, env),
        }
    }
    fn binary_value(
        &mut self,
        op: &str,
        left: Value,
        right: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        for value in [&left, &right] {
            if let Value::String(text) = value {
                self.work(1 + text.len() / 8)?;
            }
        }
        if op == "instanceof" {
            let mut right = right;
            let mut bound_depth = 0;
            while let Value::Function(id) = right {
                let Some(bound) = &self.functions[id].bound else {
                    break;
                };
                if bound_depth >= MAX_DEPTH {
                    return Err(ScriptError::resource("bound function chain limit exceeded"));
                }
                right = bound.target.clone();
                bound_depth += 1;
                self.tick()?;
            }
            if !json_callable(&right) {
                return Err(ScriptError::type_error(
                    "instanceof right-hand side is not callable",
                ));
            }
            if !js_object(&left) {
                return Ok(Value::Bool(false));
            }
            let prototype = self
                .lookup_property(&right, &"prototype".into(), doc)?
                .ok_or_else(|| ScriptError::type_error("constructor prototype is not an object"))?;
            if !js_object(&prototype) {
                return Err(ScriptError::type_error(
                    "constructor prototype is not an object",
                ));
            }
            let mut cursor = self.prototype_of(&left);
            for _ in 0..MAX_DEPTH {
                self.tick()?;
                let Some(value) = cursor else {
                    return Ok(Value::Bool(false));
                };
                if value == prototype {
                    return Ok(Value::Bool(true));
                }
                cursor = self.prototype_of(&value);
            }
            return Err(ScriptError::resource("prototype chain limit exceeded"));
        }
        if op == "in" {
            if !js_object(&right) {
                return Err(ScriptError::type_error(
                    "in right-hand side is not an object",
                ));
            }
            return Ok(Value::Bool(
                self.find_property(&right, &left.js_string())?.is_some(),
            ));
        }
        if op == "+" && (matches!(left, Value::String(_)) || matches!(right, Value::String(_))) {
            let a = left.js_string();
            let b = right.js_string();
            if a.len().saturating_add(b.len()) > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
            self.work(1 + (a.len() + b.len()) / 8)?;
            let mut units = Vec::with_capacity(a.len() + b.len());
            units.extend_from_slice(a.units());
            units.extend_from_slice(b.units());
            return self.string(units);
        }
        if ["==", "!=", "===", "!=="].contains(&op) {
            let strict = op.len() == 3;
            let equal = left == right
                || !strict
                    && (matches!(
                        (&left, &right),
                        (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null)
                    ) || matches!(
                        (&left, &right),
                        (Value::Number(_), Value::String(_))
                            | (Value::String(_), Value::Number(_))
                            | (Value::Bool(_), _)
                            | (_, Value::Bool(_))
                    ) && left.number() == right.number());
            return Ok(Value::Bool(if op.starts_with('!') {
                !equal
            } else {
                equal
            }));
        }
        if let (Value::String(a), Value::String(b)) = (&left, &right) {
            match op {
                "<" => return Ok(Value::Bool(a < b)),
                ">" => return Ok(Value::Bool(a > b)),
                "<=" => return Ok(Value::Bool(a <= b)),
                ">=" => return Ok(Value::Bool(a >= b)),
                _ => {}
            }
        }
        let a = self.number_value(left, doc)?;
        let b = self.number_value(right, doc)?;
        Ok(match op {
            "+" => Value::Number(a + b),
            "-" => Value::Number(a - b),
            "*" => Value::Number(a * b),
            "/" => Value::Number(a / b),
            "%" => Value::Number(a % b),
            "**" => Value::Number(a.powf(b)),
            "<" => Value::Bool(a < b),
            ">" => Value::Bool(a > b),
            "<=" => Value::Bool(a <= b),
            ">=" => Value::Bool(a >= b),
            "&" => Value::Number((to_i32(a) & to_i32(b)) as f64),
            "|" => Value::Number((to_i32(a) | to_i32(b)) as f64),
            "^" => Value::Number((to_i32(a) ^ to_i32(b)) as f64),
            "<<" => Value::Number(to_i32(a).wrapping_shl(to_i32(b) as u32 & 31) as f64),
            ">>" => Value::Number(to_i32(a).wrapping_shr(to_i32(b) as u32 & 31) as f64),
            ">>>" => Value::Number((to_i32(a) as u32).wrapping_shr(to_i32(b) as u32 & 31) as f64),
            _ => return Err(ScriptError::new(format!("unsupported operator '{op}'"))),
        })
    }
    fn reference(
        &mut self,
        expression: &Expr,
        env: usize,
        doc: &mut Document,
    ) -> Result<Reference> {
        match expression {
            Expr::Ident(name) => {
                let strict = self.environments[env].strict;
                Ok(if let Some((owner, _)) = self.lookup(env, name) {
                    Reference::Binding(owner, name.clone(), strict)
                } else {
                    Reference::Unresolvable(name.clone(), strict)
                })
            }
            Expr::Member(object, property) => {
                let object = self.eval(object, env, doc)?;
                let value = self.eval(property, env, doc)?;
                let property = self.json_text(value, doc, &mut Vec::new())?;
                Ok(Reference::Property(
                    object,
                    property,
                    self.environments[env].strict,
                ))
            }
            _ => Err(ScriptError::type_error("invalid assignment target")),
        }
    }
    fn read_reference(&mut self, reference: &Reference, doc: &mut Document) -> Result<Value> {
        match reference {
            Reference::Binding(env, name, _) => self.binding_value(*env, name),
            Reference::Unresolvable(name, _) => {
                Err(ScriptError::reference(format!("'{name}' is not defined")))
            }
            Reference::Property(object, key, _) => self.get_key(object.clone(), key, doc),
        }
    }
    fn write_reference(
        &mut self,
        reference: Reference,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        match reference {
            Reference::Binding(env, name, strict) => {
                // A global object binding can disappear while evaluating the
                // RHS or ToNumber/valueOf for an update. The reference retains
                // its environment, not a guaranteed-live map entry. Follow
                // Object Environment Record SetMutableBinding: strict writes
                // to a removed binding throw; sloppy writes recreate it.
                let Some(binding) = self.environments[env].bindings.get_mut(&name) else {
                    if env != 0 || strict {
                        return Err(ScriptError::reference(format!("'{name}' is not defined")));
                    }
                    return self.write_reference(Reference::Unresolvable(name, false), value, doc);
                };
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
                Ok(())
            }
            Reference::Unresolvable(name, strict) => {
                if strict {
                    return Err(ScriptError::reference(format!("'{name}' is not defined")));
                }
                self.define(0, &name, value, true)?;
                self.environments[0]
                    .bindings
                    .get_mut(&name)
                    .unwrap()
                    .deletable = true;
                Ok(())
            }
            Reference::Property(object, key, strict) => {
                self.set_key_strict(object, &key, value, strict, doc)
            }
        }
    }
    fn call(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if self.calls >= MAX_CALLS {
            return Err(ScriptError::resource("script call stack limit exceeded"));
        }
        self.enter_stack(4)?;
        self.calls += 1;
        let result = self.call_inner(function, arguments, receiver, doc);
        self.calls -= 1;
        self.stack_units -= 4;
        result
    }
    fn call_inner(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        match function {
            Value::Function(id) => {
                // Release the immutable arena borrow before charging the copy.
                // Only the Rc body is shared by FunctionCode::clone.
                let parameters = self.functions[id].code.params.len();
                self.work(1 + parameters)?;
                let code = &self.functions[id].code;
                let text_bytes = code
                    .params
                    .iter()
                    .fold(0usize, |size, name| size.saturating_add(name.len()));
                let name_bytes = code.name.as_ref().map_or(0, String::len);
                let bound_bytes = self.functions[id].bound.as_ref().map_or(0, |bound| {
                    bound
                        .arguments
                        .len()
                        .saturating_mul(std::mem::size_of::<Value>())
                });
                self.work(1 + text_bytes.saturating_add(name_bytes) / 8)?;
                self.charge(
                    128usize
                        .saturating_add(parameters.saturating_mul(std::mem::size_of::<String>()))
                        .saturating_add(text_bytes)
                        .saturating_add(name_bytes)
                        .saturating_add(bound_bytes),
                )?;
                let function = self.functions[id].clone();
                if let Some(bound) = function.bound {
                    self.charge(
                        (bound.arguments.len() + arguments.len())
                            .saturating_mul(std::mem::size_of::<Value>()),
                    )?;
                    let mut combined = bound.arguments;
                    combined.extend(arguments);
                    return self.call(bound.target, combined, bound.receiver, doc);
                }
                let env = self.environment(function.environment)?;
                self.environments[env].function_scope = true;
                self.environments[env].strict = function.code.strict;
                if !function.code.arrow {
                    let receiver = if function.code.strict {
                        receiver
                    } else if matches!(receiver, Value::Undefined | Value::Null) {
                        Value::Window
                    } else {
                        self.coerce_object(receiver)?
                    };
                    self.define(env, "this", receiver, false)?;
                }
                for (i, parameter) in function.code.params.iter().enumerate() {
                    self.define(
                        env,
                        parameter,
                        arguments.get(i).cloned().unwrap_or(Value::Undefined),
                        true,
                    )?;
                }
                let shadows_arguments = function.code.params.iter().any(|p| p == "arguments")
                    || function.code.body.iter().any(|s| matches!(s, Stmt::Function(name, _) if name == "arguments")
                        || matches!(s, Stmt::Var(bindings, kind) if *kind != DeclarationKind::Var && bindings.iter().any(|(name, _)| name == "arguments")));
                if !function.code.arrow && !shadows_arguments {
                    let args = self.arguments_object(
                        &arguments,
                        function.code.strict,
                        Value::Function(id),
                    )?;
                    if !function.code.strict {
                        let Value::Object(id) = args else {
                            unreachable!()
                        };
                        let mut seen = BTreeSet::new();
                        for (index, name) in function.code.params.iter().enumerate().rev() {
                            if seen.insert(name) && index < arguments.len() {
                                self.charge(96 + name.len())?;
                                self.objects[id]
                                    .parameter_map
                                    .insert(index.to_string().into(), (env, name.clone()));
                            }
                        }
                    }
                    self.define(env, "arguments", args, true)?;
                }
                match self.statements(&function.code.body, env, doc)? {
                    Flow::Return(value) => Ok(value),
                    Flow::Normal(_) => Ok(Value::Undefined),
                    _ => Err(ScriptError::new("loop control outside loop")),
                }
            }
            Value::Native(native) => {
                if native.name.contains('.') {
                    self.native_call(
                        &Native {
                            name: native.name.clone(),
                            receiver,
                        },
                        arguments,
                        doc,
                    )
                } else {
                    self.native_call(&native, arguments, doc)
                }
            }
            _ => Err(ScriptError::type_error("value is not callable")),
        }
    }

    fn property_object(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Object(id) => Some(*id),
            Value::Function(id) => Some(self.functions[*id].properties),
            Value::Array(id) => Some(self.array_properties[*id]),
            Value::Native(native) => self.native_properties.get(&native.name).copied(),
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
            Value::Null | Value::Undefined => return None,
            _ => "Object",
        };
        self.prototypes.get(name).copied().map(Value::Object)
    }
    fn set_object_prototype(&mut self, object: &Value, prototype: Value) -> Result<()> {
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
            cursor = self.prototype_of(&value);
        }
        self.objects[id].prototype = next;
        Ok(())
    }
    fn own_property(&self, receiver: &Value, key: &JsString) -> Option<Property> {
        if matches!(receiver, Value::Window) {
            let key = key.to_utf8().ok()?;
            return self.environments[0]
                .bindings
                .get(&key)
                .filter(|b| b.global_property)
                .map(|b| Property::data(b.value.clone(), b.mutable, b.enumerable, b.deletable));
        }
        if let Some(id) = self.property_object(receiver)
            && let Some(property) = self.objects[id].values.get(key)
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
            if key == &JsString::from("length") {
                return Some(Property::data(
                    Value::Number(self.arrays[*id].len() as f64),
                    true,
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
            if key == &JsString::from("length") {
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
    fn find_property(&mut self, receiver: &Value, key: &JsString) -> Result<Option<Property>> {
        let mut cursor = Some(receiver.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else {
                return Ok(None);
            };
            self.work(1 + key.len() / 16)?;
            if let Some(property) = self.own_property(&value, key) {
                return Ok(Some(property));
            }
            cursor = self.prototype_of(&value);
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }
    fn lookup_property(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        let Some(property) = self.find_property(receiver, key)? else {
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
    fn own_keys(&mut self, receiver: &Value) -> Result<Vec<JsString>> {
        let mut keys = Vec::new();
        if let Value::Array(id) = receiver {
            self.charge(self.arrays[*id].len().saturating_mul(64))?;
            keys.extend(
                (0..self.arrays[*id].len())
                    .filter(|i| !self.array_holes[*id].contains(i))
                    .map(|i| JsString::from(i.to_string())),
            );
            keys.push("length".into());
        }
        let text_len = match receiver {
            Value::String(text) => Some(text.len()),
            Value::Object(id) => match &self.objects[*id].boxed {
                Some(Value::String(text)) => Some(text.len()),
                _ => None,
            },
            _ => None,
        };
        if let Some(length) = text_len {
            self.charge(length.saturating_mul(64))?;
            keys.extend((0..length).map(|i| JsString::from(i.to_string())));
            keys.push("length".into());
        }
        if let Some(id) = self.property_object(receiver) {
            self.charge(
                self.objects[id]
                    .order
                    .iter()
                    .map(|key| key.byte_len() + 32)
                    .sum(),
            )?;
            keys.extend(self.objects[id].order.iter().cloned());
        } else if js_object(receiver) {
            return Err(ScriptError::unsupported(
                "host own-property enumeration is not implemented",
            ));
        }
        let count = keys.len();
        self.work(1 + count.saturating_mul(1 + count.checked_ilog2().unwrap_or(0) as usize) / 8)?;
        keys.sort_by_key(|key| {
            json_array_index(key)
                .map(|index| (false, index))
                .unwrap_or((true, 0))
        });
        let mut seen = BTreeSet::new();
        keys.retain(|key| seen.insert(key.clone()));
        Ok(keys)
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
            if self.find_property(&object, &key.into())?.is_none() {
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
        self.work(1 + key.len() / 8)?;
        let id = self.property_object(receiver).ok_or_else(|| {
            ScriptError::unsupported("host property definition is not implemented")
        })?;
        if matches!(receiver, Value::Array(_))
            && (key == &JsString::from("length") || json_array_index(key).is_some())
        {
            return Err(ScriptError::unsupported(
                "array indexed/length descriptor mutation is not implemented",
            ));
        }
        let current = self.own_property(receiver, key);
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
        } else if self.objects[id].non_extensible {
            return Ok(false);
        }
        let mapping = self.objects[id].parameter_map.get(key).cloned();
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
        if !self.objects[id].contains_key(key) {
            self.charge(256 + key.byte_len().saturating_mul(2))?;
        }
        self.objects[id].insert_property(key.clone(), property);
        if let Some((env, name)) = mapping {
            if let Some(value) = mapped_value {
                self.environments[env]
                    .bindings
                    .get_mut(&name)
                    .unwrap()
                    .value = value;
            }
            if sever_mapping {
                self.objects[id].parameter_map.remove(key);
            }
        }
        Ok(true)
    }
    fn define_properties(
        &mut self,
        object: Value,
        properties: Value,
        doc: &mut Document,
    ) -> Result<()> {
        let properties = self.coerce_object(properties)?;
        let mut descriptors = Vec::new();
        for key in self.own_keys(&properties)? {
            if !self
                .own_property(&properties, &key)
                .is_some_and(|p| p.enumerable)
            {
                continue;
            }
            let value = self.get_key(properties.clone(), &key, doc)?;
            let desc = self.property_descriptor(value, doc)?;
            self.charge(256 + key.byte_len())?;
            descriptors.push((key, desc));
        }
        for (key, desc) in descriptors {
            if !self.define_own(&object, &key, desc)? {
                return Err(ScriptError::type_error("incompatible property definition"));
            }
        }
        Ok(())
    }
    fn delete_property(&mut self, receiver: Value, key: &JsString) -> Result<bool> {
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot delete property of null or undefined",
            ));
        }
        self.work(1 + key.len() / 8)?;
        let Some(property) = self.own_property(&receiver, key) else {
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
            if let Ok(key) = key.to_utf8() {
                self.environments[0].bindings.remove(&key);
            }
            return Ok(true);
        }
        if let Value::Array(id) = receiver
            && let Some(index) = json_array_index(key)
        {
            self.charge(32)?;
            self.array_holes[id].insert(index as usize);
            self.arrays[id][index as usize] = Value::Undefined;
            return Ok(true);
        }
        if let Some(id) = self.property_object(&receiver) {
            self.work(1 + self.objects[id].order.len() / 8)?;
            self.objects[id].remove(key);
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
            let lower_value = if self.find_property(&object, &lower_key)?.is_some() {
                Some(self.get_key(object.clone(), &lower_key, doc)?)
            } else {
                None
            };
            let upper_value = if self.find_property(&object, &upper_key)?.is_some() {
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
    fn number_value(&mut self, value: Value, doc: &mut Document) -> Result<f64> {
        if let Value::String(text) = &value {
            self.work(1 + text.len() / 8)?;
        }
        if !js_object(&value) {
            return Ok(value.number());
        }
        for key in ["valueOf", "toString"] {
            let method = self.get(value.clone(), key, doc)?;
            if json_callable(&method) {
                let primitive = self.call(method, Vec::new(), value.clone(), doc)?;
                if !js_object(&primitive) {
                    if let Value::String(text) = &primitive {
                        self.work(1 + text.len() / 8)?;
                    }
                    return Ok(primitive.number());
                }
            }
        }
        Err(ScriptError::type_error(
            "object cannot be converted to a number",
        ))
    }
    fn construct(
        &mut self,
        constructor: Value,
        arguments: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        self.enter_stack(4)?;
        let result = self.construct_inner(constructor, arguments, doc);
        self.stack_units -= 4;
        result
    }
    fn construct_inner(
        &mut self,
        constructor: Value,
        arguments: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        match &constructor {
            Value::Native(native)
                if native.receiver == Value::Window
                    && matches!(
                        native.name.as_str(),
                        "Event"
                            | "CustomEvent"
                            | "ToggleEvent"
                            | "EventTarget"
                            | "DOMException"
                            | "AbortController"
                            | "AbortSignal"
                    ) =>
            {
                self.event_construct(&native.name, &arguments, doc)
            }
            Value::Native(native)
                if native.name == "RegExp" && native.receiver == Value::Window =>
            {
                self.regexp_create(
                    arguments.first().cloned().unwrap_or(Value::Undefined),
                    arguments.get(1).cloned().unwrap_or(Value::Undefined),
                    false,
                    doc,
                )
            }
            Value::Function(id) if self.functions[*id].bound.is_some() => {
                let bound = self.functions[*id].bound.clone().unwrap();
                self.charge(
                    (bound.arguments.len() + arguments.len())
                        .saturating_mul(std::mem::size_of::<Value>()),
                )?;
                let mut combined = bound.arguments;
                combined.extend(arguments);
                self.construct(bound.target, combined, doc)
            }
            Value::Function(id) if self.functions[*id].code.constructable => {
                let prototype = self.get(constructor.clone(), "prototype", doc)?;
                let instance = self.object_ordered([])?;
                let Value::Object(id) = instance else {
                    unreachable!()
                };
                if js_object(&prototype) {
                    self.objects[id].prototype = Some(prototype);
                }
                let result = self.call(constructor, arguments, instance.clone(), doc)?;
                Ok(if js_object(&result) { result } else { instance })
            }
            Value::Native(native)
                if native.receiver == Value::Window
                    && matches!(
                        native.name.as_str(),
                        "Object"
                            | "Function"
                            | "Array"
                            | "String"
                            | "Number"
                            | "Boolean"
                            | "Error"
                            | "TypeError"
                            | "SyntaxError"
                            | "ReferenceError"
                            | "RangeError"
                            | "EvalError"
                            | "URIError"
                    ) =>
            {
                let name = native.name.clone();
                let result = self.call(constructor, arguments, Value::Window, doc)?;
                if matches!(name.as_str(), "String" | "Number" | "Boolean") {
                    self.coerce_object(result)
                } else {
                    Ok(result)
                }
            }
            _ => Err(ScriptError::type_error("value is not a constructor")),
        }
    }

    fn get_key(&mut self, receiver: Value, key: &JsString, doc: &mut Document) -> Result<Value> {
        self.work(1 + key.len() / 8)?;
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
    fn set_key(
        &mut self,
        receiver: Value,
        key: &JsString,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        self.set_key_strict(receiver, key, value, false, doc)
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
        if let Some(property) = self.find_property(&receiver, key)? {
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
                if self.own_property(&receiver, key).is_none() && self.objects[id].non_extensible {
                    return Self::failed_write(strict);
                }
            } else {
                if let Some(property) = self.objects[id].values.get_mut(key) {
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
        let key = key
            .to_utf8()
            .map_err(|_| ScriptError::type_error("non-scalar host property name is unsupported"))?;
        if matches!(receiver, Value::Window)
            && let Some(binding) = self.environments[0].bindings.get(&key)
            && binding.global_property
        {
            return self.write_reference(Reference::Binding(0, key, strict), value, doc);
        }
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
        if matches!(receiver, Value::Node(_) | Value::Document | Value::Window)
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
        if self.property_object(&receiver).is_some()
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
            || matches!(key, "textContent" | "innerText" | "innerHTML" | "outerHTML")
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
            Value::Array(id) => {
                if key == "length" {
                    return Ok(Value::Number(self.arrays[*id].len() as f64));
                }
                if let Ok(index) = key.parse::<usize>() {
                    return Ok(self.arrays[*id]
                        .get(index)
                        .cloned()
                        .unwrap_or(Value::Undefined));
                }
                if [
                    "push", "pop", "shift", "unshift", "join", "forEach", "map", "filter",
                    "includes", "indexOf", "slice",
                ]
                .contains(&key)
                {
                    return Ok(Self::native(key, receiver));
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
                    return Ok(Self::native(key, receiver));
                }
            }
            Value::Native(native)
                if native.name == "String" && matches!(key, "fromCharCode" | "fromCodePoint") =>
            {
                return Ok(Self::native(key, receiver));
            }
            Value::Number(_) | Value::Bool(_) if key == "toString" => {
                return Ok(Self::native(key, receiver));
            }
            Value::Window => {
                if let Some((_, value)) = self.lookup(0, key) {
                    return Ok(value);
                }
            }
            Value::Console if ["log", "warn", "error", "info", "debug"].contains(&key) => {
                return Ok(Self::native(key, receiver));
            }
            Value::Json if ["parse", "stringify"].contains(&key) => {
                return Ok(Self::native(key, receiver));
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
                    return Ok(Self::native(key, receiver));
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
                "title" => {
                    return self.string(doc.title());
                }
                "readyState" => return self.string("complete"),
                "querySelector"
                | "querySelectorAll"
                | "getElementById"
                | "getElementsByTagName"
                | "getElementsByClassName"
                | "createElement"
                | "createTextNode"
                | "createDocumentFragment"
                | "createEvent" => return Ok(Self::native(key, receiver)),
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
                    "textContent" | "innerText" => return self.string(doc.text_content(id)),
                    "innerHTML" => return self.string(serialize_children(doc, id)),
                    "outerHTML" => return self.string(serialize_node(doc, id, 0)),
                    "value" if doc.tag(id) == Some("textarea") => {
                        self.work(1 + doc.nodes.len() / 8)?;
                        let text = doc.text_content(id);
                        self.charge(text.len().saturating_mul(2))?;
                        return self.string(text.replace("\r\n", "\n").replace('\r', "\n"));
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
                    "appendChild" | "append" | "removeChild" | "remove" | "setAttribute"
                    | "getAttribute" | "hasAttribute" | "removeAttribute" | "querySelector"
                    | "querySelectorAll" => return Ok(Self::native(key, receiver)),
                    "cloneNode" => return Ok(Self::native(key, receiver)),
                    _ => {}
                }
            }
            Value::Style(id) => {
                self.work(1 + doc.attr(*id, "style").unwrap_or("").len() / 16)?;
                if ["setProperty", "getPropertyValue", "removeProperty"].contains(&key) {
                    return Ok(Self::native(key, receiver));
                }
                if key == "cssText" {
                    return self.string(doc.attr(*id, "style").unwrap_or(""));
                }
                return self.string(style_get(doc, *id, &css_name(key)));
            }
            Value::ClassList(id) => {
                if ["add", "remove", "toggle", "contains"].contains(&key) {
                    return Ok(Self::native(key, receiver));
                }
                if key == "length" {
                    return Ok(Value::Number(
                        doc.attr(*id, "class")
                            .unwrap_or("")
                            .split_whitespace()
                            .count() as f64,
                    ));
                }
            }
            _ => {}
        }
        Ok(Value::Undefined)
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
            Value::Array(id) => {
                if key == "length" {
                    let n = value.number();
                    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
                        return Err(ScriptError::range_error("invalid array length"));
                    }
                    if n > 65536.0 {
                        return Err(ScriptError::resource("array length limit exceeded"));
                    }
                    let new = n as usize;
                    if new > self.arrays[id].len() {
                        self.charge((new - self.arrays[id].len()) * std::mem::size_of::<Value>())?;
                    }
                    let old = self.arrays[id].len();
                    if new > old {
                        self.charge((new - old).saturating_mul(32))?;
                        self.array_holes[id].extend(old..new);
                    }
                    self.array_holes[id].retain(|index| *index < new);
                    self.arrays[id].resize(new, Value::Undefined);
                } else {
                    let index = key.parse::<usize>().map_err(|_| {
                        ScriptError::new("only indexed array properties are supported")
                    })?;
                    if index >= 65536 {
                        return Err(ScriptError::resource("array index limit exceeded"));
                    }
                    if index >= self.arrays[id].len() {
                        self.charge(
                            (index + 1 - self.arrays[id].len()) * std::mem::size_of::<Value>(),
                        )?;
                        let old = self.arrays[id].len();
                        self.charge((index - old).saturating_mul(32))?;
                        self.array_holes[id].extend(old..index);
                        self.arrays[id].resize(index + 1, Value::Undefined);
                    }
                    self.array_holes[id].remove(&index);
                    self.arrays[id][index] = value;
                }
            }
            Value::Window => {
                if let Some((env, _)) = self.lookup(0, key) {
                    self.write_reference(Reference::Binding(env, key.into(), false), value, doc)?;
                } else {
                    self.define(0, key, value, true)?;
                    self.environments[0]
                        .bindings
                        .get_mut(key)
                        .unwrap()
                        .deletable = true;
                }
            }
            Value::Document if key == "title" => {
                let id = if let Some(id) = doc.first_html_element("title") {
                    id
                } else {
                    let Some(parent) = html_document_child(doc, "head") else {
                        return Ok(());
                    };
                    self.ensure_dom_capacity(doc, 1)?;
                    let id = doc.create_element("title");
                    doc.append_child(parent, id);
                    id
                };
                self.ensure_dom_capacity(doc, 1)?;
                let text = value.to_string();
                self.charge(text.len())?;
                self.charge_dom_clear(id, doc)?;
                doc.set_text_content(id, &text);
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
                let text =
                    if key == "value" && doc.tag(id) == Some("textarea") && value == Value::Null {
                        String::new()
                    } else {
                        value.to_string()
                    };
                self.charge(text.len())?;
                match key {
                    "textContent" | "innerText" => {
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
                    "checked" | "disabled" | "hidden" => {
                        if value.truthy() {
                            doc.set_attr(id, key, "");
                        } else {
                            doc.remove_attr(id, key);
                        }
                    }
                    _ => {
                        return Err(ScriptError::new(format!(
                            "unsupported DOM property '{key}'"
                        )));
                    }
                }
            }
            Value::Style(id) => {
                self.work(1 + doc.attr(id, "style").unwrap_or("").len() / 16)?;
                let text = value.to_string();
                self.charge(text.len())?;
                if key == "cssText" {
                    doc.set_attr(id, "style", &text);
                } else {
                    style_set(doc, id, &css_name(key), &text);
                }
            }
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

    // append_child validates the entire host-inclusive subtree, walks the
    // destination's host-inclusive ancestors, and removes the old sibling
    // entry. Account for all that work before it changes either child list.
    fn charge_dom_append(&mut self, parent: NodeId, child: NodeId, doc: &Document) -> Result<()> {
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
            return Ok(());
        }
        let mut cursor = Some(parent);
        let mut ancestors = 0;
        while let Some(id) = cursor {
            self.tick()?;
            if id == child || ancestors >= crate::dom::MAX_DEPTH {
                return Ok(());
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
                return Ok(());
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
        self.charge(inserted.saturating_mul(2 * std::mem::size_of::<NodeId>()))
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
        let id = doc.template_contents(id).unwrap_or(id);
        let staging = doc.create_document_fragment();
        for child in fragment.nodes[fragment.root].children.clone() {
            import_node(doc, staging, &fragment, child, 0);
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
            NodeKind::Text(text) | NodeKind::Comment(text) => text.len(),
            NodeKind::ProcessingInstruction { target, data } => target.len() + data.len(),
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
        let kind = doc.nodes[source].kind.clone();
        let id = match kind {
            NodeKind::Document => unreachable!(),
            NodeKind::DocumentFragment { .. } => doc.create_document_fragment(),
            NodeKind::Text(text) => doc.create_text_node(&text),
            NodeKind::Comment(text) => doc.create_comment(&text),
            NodeKind::ProcessingInstruction { target, data } => {
                doc.create_processing_instruction(&target, &data)
            }
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

    // Expression evaluation, statement nesting, native callbacks and JSON can
    // recurse into one another. Independent limits do not bound their combined
    // Rust stack; calls reserve extra units for the native dispatcher frames.
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
        let bytes = object.order.iter().map(JsString::byte_len).sum::<usize>();
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
    fn string_hint(&mut self, value: Value, doc: &mut Document) -> Result<JsString> {
        let mut primitive = value.clone();
        if js_object(&value) {
            for key in ["toString", "valueOf"] {
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
        let result = (|| match value {
            Value::String(text) => {
                self.charge(text.byte_len())?;
                Ok(text)
            }
            Value::Number(number) => Ok(json_number(number).into()),
            Value::Object(id) => {
                for key in ["toString", "valueOf"] {
                    let convert = self.get(Value::Object(id), key, doc)?;
                    if json_callable(&convert) {
                        let converted = self.call(convert, Vec::new(), Value::Object(id), doc)?;
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
                let length = self.arrays[id].len();
                for index in 0..length {
                    self.tick()?;
                    let value = self.arrays[id]
                        .get(index)
                        .cloned()
                        .unwrap_or(Value::Undefined);
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
                    let len = self.arrays[id].len();
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
                } else if matches!(value, Value::Object(_)) {
                    self.define_own(
                        &value,
                        &key,
                        PropertyDescriptor {
                            value: Some(child),
                            writable: Some(true),
                            enumerable: Some(true),
                            configurable: Some(true),
                            ..PropertyDescriptor::default()
                        },
                    )?;
                } else {
                    self.set_key(value.clone(), &key, child, doc)?;
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
            let len = self.arrays[id].len();
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
            Value::Undefined | Value::Function(_) | Value::Native(_) => return Ok(false),
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
                    let len = self.arrays[id].len();
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
                        Value::Undefined | Value::Function(_) | Value::Native(_)
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
    fn regexp_create(
        &mut self,
        pattern: Value,
        flags: Value,
        identity: bool,
        doc: &mut Document,
    ) -> Result<Value> {
        let existing = self.regexp_slot(&pattern);
        if identity
            && existing.is_some()
            && flags == Value::Undefined
            && self.get(pattern.clone(), "constructor", doc)?
                == Self::native("RegExp", Value::Window)
        {
            return Ok(pattern);
        }
        let source = if let Some(existing) = &existing {
            existing.source.clone()
        } else if pattern == Value::Undefined {
            JsString::default()
        } else {
            self.json_text(pattern, doc, &mut Vec::new())?
        };
        let flags = if flags == Value::Undefined {
            existing.map_or_else(JsString::default, |p| p.flags.text())
        } else {
            self.json_text(flags, doc, &mut Vec::new())?
        };
        let mut budget = self.regexp_budget();
        let result = RegExp::compile(source, &flags, &mut budget);
        self.steps = budget.steps;
        self.allocated = self.allocated.saturating_add(budget.allocated);
        self.regexp_object(Rc::new(result.map_err(regexp_error)?))
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
            Reference::Property(value, "lastIndex".into(), true),
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
    fn string_regexp(
        &mut self,
        name: &str,
        text: JsString,
        pattern: Value,
        argument: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let regex = self.regexp_slot(&pattern);
        if name == "split" {
            return self.regexp_split(regex.as_ref().unwrap(), text, argument, doc);
        }
        let receiver = if matches!(name, "match" | "search") && regex.is_none() {
            self.regexp_create(pattern, Value::Undefined, false, doc)?
        } else {
            pattern
        };
        if name == "search" {
            let previous = self.get(receiver.clone(), "lastIndex", doc)?;
            if !json_same_value(&previous, &Value::Number(0.0)) {
                self.regexp_last_index(receiver.clone(), 0, doc)?;
            }
            let result = self.regexp_exec(receiver.clone(), &text, doc)?;
            let current = self.get(receiver.clone(), "lastIndex", doc)?;
            if !json_same_value(&current, &previous) {
                self.write_reference(
                    Reference::Property(receiver, "lastIndex".into(), true),
                    previous,
                    doc,
                )?;
            }
            return if result == Value::Null {
                Ok(Value::Number(-1.0))
            } else {
                self.get(result, "index", doc)
            };
        }
        if name == "match" {
            let (global, unicode) = self.regexp_iteration_flags(receiver.clone(), doc)?;
            if !global {
                return self.regexp_exec(receiver, &text, doc);
            }
            self.regexp_last_index(receiver.clone(), 0, doc)?;
            let mut results = Vec::new();
            loop {
                self.tick()?;
                let result = self.regexp_exec(receiver.clone(), &text, doc)?;
                if result == Value::Null {
                    break;
                }
                let matched = self.get(result, "0", doc)?;
                let matched = self.json_text(matched, doc, &mut Vec::new())?;
                if results.len() >= 65536 {
                    return Err(ScriptError::resource("array length limit exceeded"));
                }
                self.charge(std::mem::size_of::<Value>())?;
                results.push(Value::String(matched.clone()));
                if matched.is_empty() {
                    self.advance_regexp(receiver.clone(), &text, unicode, doc)?;
                }
            }
            return if results.is_empty() {
                Ok(Value::Null)
            } else {
                self.array(results)
            };
        }
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
        let flags = self.json_text(flags, doc, &mut Vec::new())?;
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
    fn regexp_split(
        &mut self,
        pattern: &RegExp,
        text: JsString,
        limit: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let limit = if limit == Value::Undefined {
            u32::MAX as usize
        } else {
            to_i32(self.number_value(limit, doc)?) as u32 as usize
        };
        if limit == 0 {
            return self.array(Vec::new());
        }
        if text.is_empty() {
            return if self.regexp_find(pattern, &text, 0, true)?.is_some() {
                self.array(Vec::new())
            } else {
                self.array(vec![Value::String(text)])
            };
        }
        let mut values = Vec::new();
        let mut previous = 0;
        let mut position = 0;
        while position < text.len() {
            self.tick()?;
            let Some(found) = self.regexp_find(pattern, &text, position, true)? else {
                position += 1;
                continue;
            };
            let end = found.span().1;
            if end == previous {
                position += 1;
                continue;
            }
            if values.len() >= 65536 {
                return Err(ScriptError::resource("array length limit exceeded"));
            }
            self.charge(std::mem::size_of::<Value>())?;
            values.push(self.string(&text.units()[previous..position])?);
            if values.len() == limit {
                return self.array(values);
            }
            for span in found.captures.iter().skip(1) {
                if values.len() >= 65536 {
                    return Err(ScriptError::resource("array length limit exceeded"));
                }
                self.charge(std::mem::size_of::<Value>())?;
                values.push(match span {
                    Some((a, b)) => self.string(&text.units()[*a..*b])?,
                    None => Value::Undefined,
                });
                if values.len() == limit {
                    return self.array(values);
                }
            }
            previous = end;
            position = end;
        }
        if values.len() >= 65536 {
            return Err(ScriptError::resource("array length limit exceeded"));
        }
        self.charge(std::mem::size_of::<Value>())?;
        values.push(self.string(&text.units()[previous..])?);
        self.array(values)
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

    fn native_call(
        &mut self,
        native: &Native,
        args: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
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
        let normalized;
        let native = if let Some(method) = native.name.strip_prefix("String.")
            && !matches!(method, "toString" | "valueOf")
        {
            let receiver = if matches!(method, "fromCharCode" | "fromCodePoint") {
                Self::native("String", Value::Window)
            } else {
                if matches!(native.receiver, Value::Null | Value::Undefined) {
                    return Err(ScriptError::type_error(
                        "String method receiver is null or undefined",
                    ));
                }
                Value::String(self.json_text(native.receiver.clone(), doc, &mut Vec::new())?)
            };
            normalized = Native {
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
                name: method.into(),
                receiver: native.receiver.clone(),
            };
            &normalized
        } else if let Some(method) = native.name.strip_prefix("JSON.") {
            normalized = Native {
                name: method.into(),
                receiver: Value::Json,
            };
            &normalized
        } else if let Some(method) = native.name.strip_prefix("Math.") {
            normalized = Native {
                name: method.into(),
                receiver: Value::Math,
            };
            &normalized
        } else {
            native
        };
        let name = native.name.as_str();
        if let Some(method) = name.strip_prefix("RegExp.") {
            return self.regexp_native(method, native.receiver.clone(), &args, doc);
        }
        if name == "RegExp" && native.receiver == Value::Window {
            return self.regexp_create(
                args.first().cloned().unwrap_or(Value::Undefined),
                args.get(1).cloned().unwrap_or(Value::Undefined),
                true,
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
                    code: FunctionCode {
                        params: Vec::new(),
                        body: Rc::new(Vec::new()),
                        name: None,
                        arrow: true,
                        self_name: false,
                        constructable: false,
                        strict: true,
                    },
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
                    let object = arg(1);
                    if !js_object(&object) {
                        return Err(ScriptError::type_error("apply arguments must be an object"));
                    }
                    let length = self.get(object.clone(), "length", doc)?;
                    if let Value::String(text) = &length {
                        self.work(1 + text.len() / 16)?;
                    }
                    let length = integer_or_infinity(length.number()).max(0.0);
                    if length > 65536.0 {
                        return Err(ScriptError::resource("apply argument limit exceeded"));
                    }
                    let length = length.max(0.0).floor() as usize;
                    self.charge(length.saturating_mul(std::mem::size_of::<Value>()))?;
                    let mut parameters = Vec::with_capacity(length);
                    for index in 0..length {
                        self.tick()?;
                        parameters.push(self.get(object.clone(), &index.to_string(), doc)?);
                    }
                    parameters
                };
                return self.call(native.receiver.clone(), parameters, receiver, doc);
            }
            "Object.toString" => {
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
                let key = self.json_text(arg(0), doc, &mut Vec::new())?;
                let object = self.coerce_object(native.receiver.clone())?;
                if self.property_object(&object).is_none() {
                    return Err(ScriptError::unsupported(
                        "host own-property reflection is not implemented",
                    ));
                }
                let property = self.own_property(&object, &key);
                return Ok(Value::Bool(if name.ends_with("propertyIsEnumerable") {
                    property.is_some_and(|p| p.enumerable)
                } else {
                    property.is_some()
                }));
            }
            "Object.defineProperty" => {
                let object = arg(0);
                if !js_object(&object) {
                    return Err(ScriptError::type_error(
                        "defineProperty target must be an object",
                    ));
                }
                let key = self.json_text(arg(1), doc, &mut Vec::new())?;
                let desc = self.property_descriptor(arg(2), doc)?;
                if !self.define_own(&object, &key, desc)? {
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
                let key = self.json_text(arg(1), doc, &mut Vec::new())?;
                if self.property_object(&object).is_none() {
                    return Err(ScriptError::unsupported(
                        "host own-property reflection is not implemented",
                    ));
                }
                let Some(property) = self.own_property(&object, &key) else {
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
                if matches!(object, Value::Array(_)) {
                    return Err(ScriptError::unsupported(
                        "array extensibility restrictions are not implemented",
                    ));
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
                return Ok(self.prototype_of(&object).unwrap_or(Value::Null));
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
                self.set_object_prototype(&object, prototype)?;
                return Ok(object);
            }
            "Object.keys" | "Object.values" | "Object.getOwnPropertyNames" => {
                let object = self.coerce_object(arg(0))?;
                let keys = self.own_keys(&object)?;
                let mut result = Vec::new();
                for key in keys {
                    self.tick()?;
                    if name != "Object.getOwnPropertyNames"
                        && !self
                            .own_property(&object, &key)
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
                return Err(ScriptError::unsupported(
                    "dynamic Function construction is not implemented",
                ));
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
                    if length > 65536.0 {
                        return Err(ScriptError::resource("array length limit exceeded"));
                    }
                    let array = self.array(vec![Value::Undefined; length as usize])?;
                    let Value::Array(id) = array else {
                        unreachable!()
                    };
                    self.charge((length as usize).saturating_mul(32))?;
                    self.array_holes[id].extend(0..length as usize);
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
            Value::Window
                if [
                    "String",
                    "Number",
                    "Boolean",
                    "parseInt",
                    "parseFloat",
                    "isNaN",
                    "isFinite",
                ]
                .contains(&name) =>
            {
                let value = arg(0);
                return match name {
                    "String" => {
                        let text = if args.is_empty() {
                            JsString::default()
                        } else {
                            self.json_text(value, doc, &mut Vec::new())?
                        };
                        self.string(text)
                    }
                    "Number" => Ok(Value::Number(if args.is_empty() {
                        0.0
                    } else {
                        value.number()
                    })),
                    "Boolean" => Ok(Value::Bool(value.truthy())),
                    "isNaN" => Ok(Value::Bool(value.number().is_nan())),
                    "isFinite" => Ok(Value::Bool(value.number().is_finite())),
                    "parseFloat" => Ok(Value::Number(parse_float(
                        &value.js_string().to_utf8_lossy(),
                    ))),
                    "parseInt" => Ok(Value::Number(parse_int(
                        &value.js_string().to_utf8_lossy(),
                        args.get(1).map(Value::number).unwrap_or(0.0),
                    ))),
                    _ => unreachable!(),
                };
            }
            Value::Console => {
                let mut line = String::new();
                for (index, value) in args.iter().enumerate() {
                    let text = value.to_string();
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
                let a = arg(0).number();
                let b = arg(1).number();
                let value = match name {
                    "abs" => a.abs(),
                    "floor" => a.floor(),
                    "ceil" => a.ceil(),
                    "round" => (a + 0.5).floor(),
                    "trunc" => a.trunc(),
                    "sqrt" => a.sqrt(),
                    "pow" => a.powf(b),
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
                    "min" => args.iter().map(Value::number).fold(f64::INFINITY, |a, b| {
                        if a.is_nan() || b.is_nan() {
                            f64::NAN
                        } else {
                            a.min(b)
                        }
                    }),
                    "max" => args
                        .iter()
                        .map(Value::number)
                        .fold(f64::NEG_INFINITY, |a, b| {
                            if a.is_nan() || b.is_nan() {
                                f64::NAN
                            } else {
                                a.max(b)
                            }
                        }),
                    _ => return Err(ScriptError::new("unsupported Math method")),
                };
                return Ok(Value::Number(value));
            }
            Value::Array(id) => {
                let id = *id;
                match name {
                    "push" | "unshift" => {
                        if self.arrays[id].len().saturating_add(args.len()) > 65536 {
                            return Err(ScriptError::resource("array length limit exceeded"));
                        }
                        self.charge(args.len() * std::mem::size_of::<Value>())?;
                        if name == "push" {
                            self.arrays[id].extend(args);
                        } else {
                            let count = args.len();
                            self.array_holes[id] = self.array_holes[id]
                                .iter()
                                .map(|index| index + count)
                                .collect();
                            self.arrays[id].splice(0..0, args);
                        }
                        return Ok(Value::Number(self.arrays[id].len() as f64));
                    }
                    "pop" => {
                        let value = self.arrays[id].pop().unwrap_or(Value::Undefined);
                        self.array_holes[id].remove(&self.arrays[id].len());
                        return Ok(value);
                    }
                    "shift" => {
                        return Ok(if self.arrays[id].is_empty() {
                            Value::Undefined
                        } else {
                            self.array_holes[id] = self.array_holes[id]
                                .iter()
                                .filter_map(|index| index.checked_sub(1))
                                .collect();
                            self.arrays[id].remove(0)
                        });
                    }
                    "join" => {
                        let separator = if matches!(arg(0), Value::Undefined) {
                            JsString::from(",")
                        } else {
                            arg(0).js_string()
                        };
                        let mut result = Vec::new();
                        for index in 0..self.arrays[id].len() {
                            let value = self.arrays[id]
                                .get(index)
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            let text = if matches!(value, Value::Null | Value::Undefined) {
                                JsString::default()
                            } else {
                                self.json_text(value, doc, &mut vec![id])?
                            };
                            let separator_len = if index > 0 { separator.len() } else { 0 };
                            if result
                                .len()
                                .saturating_add(text.len())
                                .saturating_add(separator_len)
                                > MAX_STRING
                            {
                                return Err(ScriptError::resource("script string limit exceeded"));
                            }
                            self.charge((text.len() + separator_len) * 2)?;
                            if index > 0 {
                                result.extend_from_slice(separator.units());
                            }
                            result.extend_from_slice(text.units());
                        }
                        return self.string(result);
                    }
                    "includes" => {
                        return Ok(Value::Bool(self.arrays[id].iter().any(|value| {
                            *value == arg(0)
                                || matches!(value, Value::Number(n) if n.is_nan())
                                    && matches!(arg(0), Value::Number(n) if n.is_nan())
                        })));
                    }
                    "indexOf" => {
                        return Ok(Value::Number(
                            self.arrays[id]
                                .iter()
                                .position(|value| *value == arg(0))
                                .map(|i| i as f64)
                                .unwrap_or(-1.0),
                        ));
                    }
                    "slice" => {
                        let len = self.arrays[id].len();
                        let start = relative_index(arg(0).number(), len);
                        let end = args
                            .get(1)
                            .map(|v| relative_index(v.number(), len))
                            .unwrap_or(len)
                            .max(start);
                        return self.array(self.arrays[id][start..end].to_vec());
                    }
                    "forEach" | "map" | "filter" => {
                        let callback = arg(0);
                        let len = self.arrays[id].len();
                        let mut result = Vec::new();
                        for index in 0..len {
                            self.tick()?;
                            let value = self.arrays[id]
                                .get(index)
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            let returned = self.call(
                                callback.clone(),
                                vec![value.clone(), Value::Number(index as f64), Value::Array(id)],
                                Value::Window,
                                doc,
                            )?;
                            if name == "map" {
                                result.push(returned);
                            } else if name == "filter" && returned.truthy() {
                                result.push(value);
                            }
                        }
                        return if name == "forEach" {
                            Ok(Value::Undefined)
                        } else {
                            self.array(result)
                        };
                    }
                    _ => {}
                }
            }
            Value::String(text) => {
                if matches!(name, "match" | "search" | "replace")
                    || name == "split" && self.regexp_slot(&arg(0)).is_some()
                {
                    return self.string_regexp(name, text.clone(), arg(0), arg(1), doc);
                }
                if matches!(name, "includes" | "startsWith" | "endsWith")
                    && self.regexp_slot(&arg(0)).is_some()
                {
                    return Err(ScriptError::type_error(
                        "String search argument must not be a RegExp",
                    ));
                }
                let needle = if matches!(
                    name,
                    "includes" | "indexOf" | "startsWith" | "endsWith" | "split"
                ) {
                    self.json_text(arg(0), doc, &mut Vec::new())?
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
            Value::Style(id) => {
                self.work(1 + doc.attr(*id, "style").unwrap_or("").len() / 16)?;
                let property = arg(0).to_string();
                let previous = style_get(doc, *id, &property);
                if name == "getPropertyValue" {
                    return self.string(previous);
                }
                let value = if name == "removeProperty" {
                    String::new()
                } else {
                    arg(1).to_string()
                };
                self.charge(property.len() + value.len() + 64)?;
                style_set(doc, *id, &property, &value);
                return if name == "removeProperty" {
                    self.string(previous)
                } else {
                    Ok(Value::Undefined)
                };
            }
            Value::ClassList(id) => {
                self.work(1 + doc.attr(*id, "class").unwrap_or("").len() / 16)?;
                let mut classes: Vec<String> = doc
                    .attr(*id, "class")
                    .unwrap_or("")
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect();
                let token = arg(0).to_string();
                if name == "contains" {
                    return Ok(Value::Bool(classes.contains(&token)));
                }
                let mut present = classes.contains(&token);
                if name == "toggle" {
                    if token.is_empty() || token.chars().any(char::is_whitespace) {
                        return Err(ScriptError::new("invalid class token"));
                    }
                    present = args.get(1).map(Value::truthy).unwrap_or(!present);
                    classes.retain(|item| item != &token);
                    if present {
                        classes.push(token);
                    }
                } else {
                    for value in args {
                        let token = value.to_string();
                        if token.is_empty() || token.chars().any(char::is_whitespace) {
                            return Err(ScriptError::new("invalid class token"));
                        }
                        if name == "add" && !classes.contains(&token) {
                            classes.push(token);
                        } else if name == "remove" {
                            classes.retain(|item| item != &token);
                        }
                    }
                }
                let classes = classes.join(" ");
                self.charge(classes.len())?;
                doc.set_attr(*id, "class", &classes);
                return if name == "toggle" {
                    Ok(Value::Bool(present))
                } else {
                    Ok(Value::Undefined)
                };
            }
            _ => {}
        }

        if [
            "querySelector",
            "querySelectorAll",
            "getElementById",
            "getElementsByTagName",
            "getElementsByClassName",
        ]
        .contains(&name)
        {
            self.work(1 + doc.nodes.len() / 8)?;
            let input = arg(0).to_string();
            let selector = match name {
                "getElementsByClassName" => format!(
                    ".{}",
                    input.split_whitespace().collect::<Vec<_>>().join(".")
                ),
                _ => input.clone(),
            };
            let root = if let Value::Node(id) = native.receiver {
                id
            } else {
                doc.root
            };
            let candidates = if name == "getElementById" {
                doc.query_selector_all_from(root, "*")
                    .into_iter()
                    .filter(|id| doc.attr(*id, "id") == Some(input.as_str()))
                    .collect::<Vec<_>>()
            } else {
                doc.query_selector_all_from(root, &selector)
            };
            self.charge(candidates.len() * 8)?;
            return if ["querySelector", "getElementById"].contains(&name) {
                Ok(candidates
                    .first()
                    .copied()
                    .map(Value::Node)
                    .unwrap_or(Value::Null))
            } else {
                self.array(candidates.into_iter().map(Value::Node).collect())
            };
        }
        if name == "createElement" {
            let tag = arg(0).to_string();
            if tag.is_empty()
                || !tag
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Err(ScriptError::new("invalid element tag name"));
            }
            self.ensure_dom_capacity(
                doc,
                if tag.eq_ignore_ascii_case("template") {
                    2
                } else {
                    1
                },
            )?;
            return Ok(Value::Node(doc.create_element(&tag.to_ascii_lowercase())));
        }
        if name == "createTextNode" {
            let text = arg(0).to_string();
            self.ensure_dom_capacity(doc, 1)?;
            self.charge(text.len())?;
            return Ok(Value::Node(doc.create_text_node(&text)));
        }
        if name == "createDocumentFragment" {
            self.ensure_dom_capacity(doc, 1)?;
            return Ok(Value::Node(doc.create_document_fragment()));
        }
        if let Value::Node(id) = native.receiver {
            match name {
                "cloneNode" => {
                    return self
                        .clone_dom_node(id, arg(0).truthy(), doc)
                        .map(Value::Node);
                }
                "getAttribute" => {
                    return match doc.attr(id, &arg(0).to_string()) {
                        Some(value) => self.string(value),
                        None => Ok(Value::Null),
                    };
                }
                "hasAttribute" => {
                    return Ok(Value::Bool(doc.attr(id, &arg(0).to_string()).is_some()));
                }
                "setAttribute" => {
                    let key = arg(0).to_string();
                    let text = arg(1).to_string();
                    self.charge(key.len() + text.len() + 64)?;
                    let work = doc.base_attribute_work(id, &key);
                    self.work(work.saturating_add(if work > 0 { text.len() } else { 0 }))?;
                    self.charge_details_attribute(id, &key, text.len(), doc)?;
                    doc.set_attr(id, &key, &text);
                    self.event_attribute_changed(id, &key, doc)?;
                    return Ok(Value::Undefined);
                }
                "removeAttribute" => {
                    let key = arg(0).to_string();
                    self.work(doc.base_attribute_work(id, &key))?;
                    self.charge_details_attribute(id, &key, 0, doc)?;
                    doc.remove_attr(id, &key);
                    self.event_attribute_changed(id, &key, doc)?;
                    return Ok(Value::Undefined);
                }
                "appendChild" | "removeChild" => {
                    let Value::Node(child) = arg(0) else {
                        return Err(ScriptError::new("expected DOM node"));
                    };
                    if name == "appendChild" {
                        self.charge_dom_append(id, child, doc)?;
                        doc.append_child(id, child);
                    } else {
                        if doc.nodes.get(child).and_then(|node| node.parent) != Some(id) {
                            return Err(ScriptError::new("node is not a child"));
                        }
                        self.charge_dom_remove(id, doc)?;
                        self.work(doc.base_remove_work(child))?;
                        doc.remove_child(id, child);
                    }
                    return Ok(Value::Node(child));
                }
                "append" => {
                    for value in args {
                        let child = if let Value::Node(child) = value {
                            child
                        } else {
                            let text = value.to_string();
                            self.ensure_dom_capacity(doc, 1)?;
                            self.charge(text.len())?;
                            doc.create_text_node(&text)
                        };
                        self.charge_dom_append(id, child, doc)?;
                        doc.append_child(id, child);
                    }
                    return Ok(Value::Undefined);
                }
                "remove" => {
                    if let Some(parent) = doc.nodes[id].parent {
                        self.charge_dom_remove(parent, doc)?;
                        self.work(doc.base_remove_work(id))?;
                        doc.remove_child(parent, id);
                    }
                    return Ok(Value::Undefined);
                }
                _ => {}
            }
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
        Value::Undefined | Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
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
    let key = key.to_utf8().ok()?;
    let index = key.parse::<u32>().ok()?;
    (index != u32::MAX && index.to_string() == key).then_some(index)
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
fn style_get(doc: &Document, id: NodeId, key: &str) -> String {
    doc.attr(id, "style")
        .unwrap_or("")
        .split(';')
        .filter_map(|entry| entry.split_once(':'))
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case(key))
        .map(|(_, value)| value.trim())
        .next_back()
        .unwrap_or("")
        .to_owned()
}
fn style_set(doc: &mut Document, id: NodeId, key: &str, value: &str) {
    let mut styles: Vec<String> = doc
        .attr(id, "style")
        .unwrap_or("")
        .split(';')
        .filter_map(|entry| entry.split_once(':'))
        .filter(|(name, _)| !name.trim().eq_ignore_ascii_case(key))
        .map(|(name, value)| format!("{}: {}", name.trim(), value.trim()))
        .collect();
    if !value.is_empty() {
        styles.push(format!("{key}: {value}"));
    }
    doc.set_attr(id, "style", &styles.join("; "));
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
fn import_node(doc: &mut Document, parent: NodeId, source: &Document, node: NodeId, depth: usize) {
    if depth >= 96 || doc.nodes.len() >= MAX_NODES {
        return;
    }
    let id = match &source.nodes[node].kind {
        NodeKind::Document => return,
        NodeKind::DocumentFragment { .. } => {
            for child in &source.nodes[node].children {
                import_node(doc, parent, source, *child, depth + 1);
            }
            return;
        }
        NodeKind::Text(text) => doc.create_text_node(text),
        NodeKind::Comment(text) => doc.create_comment(text),
        NodeKind::ProcessingInstruction { target, data } => {
            doc.create_processing_instruction(target, data)
        }
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
        return;
    }
    doc.append_child(parent, id);
    for child in &source.nodes[node].children {
        import_node(doc, id, source, *child, depth + 1);
    }
    if let Some(source_contents) = source.template_contents(node)
        && let Some(contents) = doc.template_contents(id)
    {
        for child in &source.nodes[source_contents].children {
            import_node(doc, contents, source, *child, depth + 1);
        }
    }
}
fn escape_html(text: &str, attribute: bool) -> String {
    let mut result = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    if attribute {
        result = result.replace('"', "&quot;");
    }
    result
}
fn serialize_node(doc: &Document, id: NodeId, depth: usize) -> String {
    if depth >= 96 {
        return String::new();
    }
    match &doc.nodes[id].kind {
        NodeKind::Text(text) => escape_html(text, false),
        NodeKind::Comment(text) => format!("<!--{text}-->"),
        NodeKind::ProcessingInstruction { .. } => doc.outer_html(id),
        NodeKind::Doctype(_) => doc.outer_html(id),
        NodeKind::Document | NodeKind::DocumentFragment { .. } => {
            serialize_children_at(doc, id, depth)
        }
        NodeKind::Element(element) => {
            let mut result = format!("<{}", element.tag);
            for (key, value) in &element.attrs {
                result.push_str(&format!(" {key}=\"{}\"", escape_html(value, true)));
            }
            result.push('>');
            if element.namespace != Namespace::Html
                || ![
                    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                    "param", "source", "track", "wbr",
                ]
                .contains(&element.tag.as_str())
            {
                result.push_str(&serialize_children_at(doc, id, depth));
                result.push_str(&format!("</{}>", element.tag));
            }
            result
        }
    }
}
fn serialize_children_at(doc: &Document, id: NodeId, depth: usize) -> String {
    let id = doc.template_contents(id).unwrap_or(id);
    let mut result = String::new();
    for child in &doc.nodes[id].children {
        let piece = serialize_node(doc, *child, depth + 1);
        if result.len().saturating_add(piece.len()) > MAX_STRING {
            break;
        }
        result.push_str(&piece);
    }
    result
}
fn serialize_children(doc: &Document, id: NodeId) -> String {
    serialize_children_at(doc, id, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(source: &str) -> Result<Value> {
        Runtime::new().execute(source, &mut Document::parse("<body></body>"))
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
        assert_eq!(style_get(&doc, counter, "color"), "red");
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
        assert!(
            run(&format!("{}1{}", "(".repeat(1000), ")".repeat(1000)))
                .unwrap_err()
                .message
                .contains("nesting")
        );
        assert!(
            run("let s = 'x'; while (true) { s = s + s; }")
                .unwrap_err()
                .message
                .contains("limit")
        );
        assert!(
            run("const a = []; a[4294967294] = 1;")
                .unwrap_err()
                .message
                .contains("limit")
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
        assert!(
            Runtime::parse_only(&nested)
                .unwrap_err()
                .is_resource_limit()
        );
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
        assert!(
            run(&format!("x{}", ".x".repeat(5000)))
                .unwrap_err()
                .message
                .contains("limit")
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
            "const values = []; values.length = 1000000;",
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
    fn statement_nesting_and_function_calls_share_the_native_stack_budget() {
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
    fn utf16_dom_boundary_is_explicitly_lossy_without_changing_script_values() {
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
            Value::String(JsString::from("�🦀�"))
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
        assert!(
            runtime
                .execute("new Function('return 1')", &mut document)
                .unwrap_err()
                .is_unsupported()
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
                .function_value(code, 1)
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
                .function_value(&named, 1)
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
        for source in [
            "({*m(){}})",
            "({async m(){}})",
            "({async ['m'](){}})",
            "({async *m(){}})",
            "({...x})",
            "({m(){return super.x;}})",
            "({m(a=1){}})",
            "({m(...args){}})",
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
        assert!(
            Runtime::parse_only(&nested)
                .unwrap_err()
                .is_resource_limit()
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
        assert!(
            run("Object.defineProperty([],'0',{get:function(){return 1;}})")
                .unwrap_err()
                .is_unsupported()
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
                    name: "setAttribute".into(),
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
                    name: "appendChild".into(),
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
                        name: method.into(),
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
