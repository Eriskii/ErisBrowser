//! A deliberately small, capability-limited JavaScript interpreter.
//!
//! This is a custom language implementation, not an ECMAScript conformance claim.
//! Every entry point enforces execution, nesting, source, and allocation limits.
//! Scripts have DOM access but no filesystem, network, process, or host-eval access.

use crate::dom::{Document, NodeId, NodeKind};
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

const MAX_SOURCE: usize = 256 * 1024;
const MAX_TOKENS: usize = 32_768;
const MAX_DEPTH: usize = 96;
const MAX_CALLS: usize = 48;
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
    String(Rc<str>),
    Array(usize),
    Object(usize),
    Function(usize),
    Node(NodeId),
    Document,
    Window,
    Console,
    Math,
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
            Self::String(s) if s.trim().is_empty() => 0.0,
            Self::String(s) => s.trim().parse().unwrap_or(f64::NAN),
            _ => f64::NAN,
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
}

#[derive(Clone, Debug, PartialEq)]
enum ErrorKind {
    Runtime(&'static str),
    Resource,
    Thrown(Value),
}

impl ScriptError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("Error"),
        }
    }
    fn at(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset: Some(offset),
            kind: ErrorKind::Runtime("SyntaxError"),
        }
    }
    fn resource(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Resource,
        }
    }
    fn reference(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("ReferenceError"),
        }
    }
    fn type_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("TypeError"),
        }
    }
    fn range_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
            kind: ErrorKind::Runtime("RangeError"),
        }
    }
    fn thrown(value: Value) -> Self {
        Self {
            message: format!("uncaught exception: {value}"),
            offset: None,
            kind: ErrorKind::Thrown(value),
        }
    }
    /// Resource failures terminate script execution and cannot enter catch/finally.
    pub fn is_resource_limit(&self) -> bool {
        matches!(self.kind, ErrorKind::Resource)
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

#[derive(Clone, Debug)]
enum TokenKind {
    Word(String),
    Number(f64),
    String(Rc<str>),
    Symbol(String),
    End,
}
#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    offset: usize,
    line_break_before: bool,
}

fn lex(source: &str) -> Result<Vec<Token>> {
    if source.len() > MAX_SOURCE {
        return Err(ScriptError::resource("script source limit exceeded"));
    }
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut line_break_before = false;
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
        let kind = if ch == '\'' || ch == '"' || ch == '`' {
            let quote = ch;
            pos += 1;
            let mut value = String::new();
            let mut closed = false;
            while pos < source.len() {
                let c = source[pos..].chars().next().unwrap();
                pos += c.len_utf8();
                if c == quote {
                    closed = true;
                    break;
                }
                if quote == '`' && c == '$' && source[pos..].starts_with('{') {
                    return Err(ScriptError::at(
                        "template interpolation is not supported",
                        pos - 1,
                    ));
                }
                if c == '\\' {
                    let e = source[pos..]
                        .chars()
                        .next()
                        .ok_or_else(|| ScriptError::at("unterminated escape", pos))?;
                    pos += e.len_utf8();
                    match e {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        'b' => value.push('\u{0008}'),
                        'f' => value.push('\u{000c}'),
                        'v' => value.push('\u{000b}'),
                        '0' => value.push('\0'),
                        '\n' => {}
                        'u' | 'x' => {
                            let len = if e == 'u' { 4 } else { 2 };
                            let end = pos
                                .checked_add(len)
                                .filter(|end| *end <= source.len())
                                .ok_or_else(|| {
                                    ScriptError::at("incomplete character escape", pos)
                                })?;
                            let hex = source
                                .get(pos..end)
                                .ok_or_else(|| ScriptError::at("invalid character escape", pos))?;
                            let n = u32::from_str_radix(hex, 16)
                                .map_err(|_| ScriptError::at("invalid character escape", pos))?;
                            value.push(char::from_u32(n).unwrap_or('\u{fffd}'));
                            pos = end;
                        }
                        _ => value.push(e),
                    }
                } else {
                    if c == '\n' && quote != '`' {
                        return Err(ScriptError::at("newline in string", pos));
                    }
                    value.push(c);
                }
            }
            if !closed {
                return Err(ScriptError::at("unterminated string", start));
            }
            TokenKind::String(Rc::from(value))
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
                TokenKind::Number(
                    source[start..pos]
                        .parse()
                        .map_err(|_| ScriptError::at("invalid number", start))?,
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
                "===", "!==", ">>>", "**=", "=>", "==", "!=", "<=", ">=", "&&", "||", "??", "++",
                "--", "+=", "-=", "*=", "/=", "%=", "**", "<<", ">>",
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
        });
        line_break_before = false;
        if tokens.len() > MAX_TOKENS {
            return Err(ScriptError::resource("script token limit exceeded"));
        }
    }
    tokens.push(Token {
        kind: TokenKind::End,
        offset: source.len(),
        line_break_before,
    });
    Ok(tokens)
}

#[derive(Clone, Debug)]
struct FunctionCode {
    params: Vec<String>,
    body: Rc<Vec<Stmt>>,
}
#[derive(Clone, Debug)]
enum Expr {
    Literal(Value),
    Ident(String),
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Unary(String, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>, Box<Expr>),
    Update(Box<Expr>, f64, bool),
    Member(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Function(FunctionCode),
}
#[derive(Clone, Debug)]
enum Stmt {
    Empty,
    Expr(Expr),
    Var(Vec<(String, Option<Expr>)>, bool),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Function(String, FunctionCode),
    Return(Option<Expr>),
    Throw(Expr),
    Try(Box<Stmt>, Option<CatchClause>, Option<Box<Stmt>>),
    Break,
    Continue,
}

#[derive(Clone, Debug)]
struct CatchClause {
    binding: Option<String>,
    body: Vec<Stmt>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
}
impl Parser {
    fn program(source: &str) -> Result<Vec<Stmt>> {
        let mut parser = Self {
            tokens: lex(source)?,
            pos: 0,
            depth: 0,
        };
        let mut statements = Vec::new();
        while !parser.done() {
            statements.push(parser.statement()?);
        }
        Ok(statements)
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
    fn statement_inner(&mut self) -> Result<Stmt> {
        if self.eat(";") {
            return Ok(Stmt::Empty);
        }
        if self.eat("{") {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.is("let") || self.is("const") || self.is("var") {
            let declaration = self.declaration()?;
            self.eat(";");
            return Ok(declaration);
        }
        if self.eat("function") {
            let name = self.identifier()?;
            return Ok(Stmt::Function(name, self.function()?));
        }
        if self.eat("if") {
            self.expect("(")?;
            let condition = self.expression()?;
            self.expect(")")?;
            let yes = Box::new(self.statement()?);
            let no = if self.eat("else") {
                Some(Box::new(self.statement()?))
            } else {
                None
            };
            return Ok(Stmt::If(condition, yes, no));
        }
        if self.eat("while") {
            self.expect("(")?;
            let condition = self.expression()?;
            self.expect(")")?;
            return Ok(Stmt::While(condition, Box::new(self.statement()?)));
        }
        if self.eat("for") {
            self.expect("(")?;
            let init = if self.is(";") {
                None
            } else if self.is("let") || self.is("const") || self.is("var") {
                Some(Box::new(self.declaration()?))
            } else {
                Some(Box::new(Stmt::Expr(self.expression()?)))
            };
            self.expect(";")?;
            let test = if self.is(";") {
                None
            } else {
                Some(self.expression()?)
            };
            self.expect(";")?;
            let update = if self.is(")") {
                None
            } else {
                Some(self.expression()?)
            };
            self.expect(")")?;
            return Ok(Stmt::For(init, test, update, Box::new(self.statement()?)));
        }
        if self.eat("return") {
            let result = if self.is(";")
                || self.is("}")
                || self.done()
                || self.tokens[self.pos].line_break_before
            {
                None
            } else {
                Some(self.expression()?)
            };
            self.eat(";");
            return Ok(Stmt::Return(result));
        }
        if self.eat("throw") {
            if self.tokens[self.pos].line_break_before {
                return Err(self.error("line break is not allowed after throw"));
            }
            let value = self.expression()?;
            self.eat(";");
            return Ok(Stmt::Throw(value));
        }
        if self.eat("try") {
            self.expect("{")?;
            let body = Box::new(Stmt::Block(self.block()?));
            let handler = if self.eat("catch") {
                let binding = if self.eat("(") {
                    let name = self.identifier()?;
                    self.expect(")")?;
                    Some(name)
                } else {
                    None
                };
                self.expect("{")?;
                Some(CatchClause {
                    binding,
                    body: self.block()?,
                })
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
            self.eat(";");
            return Ok(Stmt::Break);
        }
        if self.eat("continue") {
            self.eat(";");
            return Ok(Stmt::Continue);
        }
        for unsupported in [
            "class", "import", "export", "catch", "finally", "switch", "do", "with", "async",
            "await", "yield",
        ] {
            if self.is(unsupported) {
                return Err(self.error(format!("'{unsupported}' is not supported")));
            }
        }
        let expression = self.expression()?;
        self.eat(";");
        Ok(Stmt::Expr(expression))
    }
    fn declaration(&mut self) -> Result<Stmt> {
        let mutable = !self.eat("const");
        if mutable {
            self.pos += 1;
        }
        let mut bindings = Vec::new();
        loop {
            let name = self.identifier()?;
            let value = if self.eat("=") {
                Some(self.expression()?)
            } else {
                None
            };
            if !mutable && value.is_none() {
                return Err(self.error("const declaration needs a value"));
            }
            bindings.push((name, value));
            if !self.eat(",") {
                break;
            }
        }
        Ok(Stmt::Var(bindings, mutable))
    }
    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut body = Vec::new();
        while !self.eat("}") {
            if self.done() {
                return Err(self.error("unterminated block"));
            }
            body.push(self.statement()?);
        }
        Ok(body)
    }
    fn function(&mut self) -> Result<FunctionCode> {
        self.expect("(")?;
        let mut params = Vec::new();
        if !self.eat(")") {
            loop {
                params.push(self.identifier()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        self.expect("{")?;
        Ok(FunctionCode {
            params,
            body: Rc::new(self.block()?),
        })
    }
    fn arrow(&mut self, params: Vec<String>) -> Result<Expr> {
        let body = if self.eat("{") {
            self.block()?
        } else {
            vec![Stmt::Return(Some(self.expression()?))]
        };
        Ok(Expr::Function(FunctionCode {
            params,
            body: Rc::new(body),
        }))
    }
    fn expression(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.assignment();
        self.depth -= 1;
        result
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
                if !matches!(left, Expr::Ident(_) | Expr::Member(_, _)) {
                    return Err(self.error("invalid assignment target"));
                }
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
        let mut left = self.unary()?;
        let mut chain = 0;
        while let TokenKind::Symbol(s) | TokenKind::Word(s) = &self.tokens[self.pos].kind {
            let op = s.clone();
            let precedence = match op.as_str() {
                "||" | "??" => 1,
                "&&" => 2,
                "|" => 3,
                "^" => 4,
                "&" => 5,
                "==" | "!=" | "===" | "!==" => 6,
                "<" | ">" | "<=" | ">=" => 7,
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
            if chain > MAX_DEPTH / 2 {
                return Err(self.resource_error("expression chain limit exceeded"));
            }
            self.pos += 1;
            let right = self.binary(if op == "**" {
                precedence
            } else {
                precedence + 1
            })?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.unary_inner();
        self.depth -= 1;
        result
    }
    fn unary_inner(&mut self) -> Result<Expr> {
        for op in ["!", "-", "+", "~", "typeof", "void"] {
            if self.eat(op) {
                return Ok(Expr::Unary(op.to_owned(), Box::new(self.unary()?)));
            }
        }
        if self.is("++") || self.is("--") {
            let delta = if self.eat("++") {
                1.0
            } else {
                self.pos += 1;
                -1.0
            };
            return Ok(Expr::Update(Box::new(self.unary()?), delta, true));
        }
        let mut value = self.primary()?;
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
                    Box::new(Expr::Literal(Value::String(Rc::from(property)))),
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
            } else {
                break;
            }
        }
        if self.eat("++") {
            value = Expr::Update(Box::new(value), 1.0, false);
        } else if self.eat("--") {
            value = Expr::Update(Box::new(value), -1.0, false);
        }
        Ok(value)
    }
    fn primary(&mut self) -> Result<Expr> {
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
            let value = self.expression()?;
            self.expect(")")?;
            return Ok(value);
        }
        if self.eat("[") {
            let mut items = Vec::new();
            if !self.eat("]") {
                loop {
                    items.push(self.expression()?);
                    if self.eat("]") {
                        break;
                    }
                    self.expect(",")?;
                    if self.eat("]") {
                        break;
                    }
                }
            }
            return Ok(Expr::Array(items));
        }
        if self.eat("{") {
            let mut entries = Vec::new();
            if !self.eat("}") {
                loop {
                    let key = match self.tokens[self.pos].kind.clone() {
                        TokenKind::Word(s) => s,
                        TokenKind::String(s) => s.to_string(),
                        TokenKind::Number(n) => n.to_string(),
                        _ => return Err(self.error("expected object property")),
                    };
                    self.pos += 1;
                    let value = if self.eat(":") {
                        self.expression()?
                    } else {
                        Expr::Ident(key.clone())
                    };
                    entries.push((key, value));
                    if self.eat("}") {
                        break;
                    }
                    self.expect(",")?;
                    if self.eat("}") {
                        break;
                    }
                }
            }
            return Ok(Expr::Object(entries));
        }
        if self.eat("function") {
            if matches!(self.tokens[self.pos].kind, TokenKind::Word(_)) {
                self.pos += 1;
            }
            return Ok(Expr::Function(self.function()?));
        }
        let token = self.tokens[self.pos].clone();
        if !self.done() {
            self.pos += 1;
        }
        match token.kind {
            TokenKind::Number(n) => Ok(Expr::Literal(Value::Number(n))),
            TokenKind::String(s) => Ok(Expr::Literal(Value::String(s))),
            TokenKind::Word(s) if s == "true" || s == "false" => {
                Ok(Expr::Literal(Value::Bool(s == "true")))
            }
            TokenKind::Word(s) if s == "null" => Ok(Expr::Literal(Value::Null)),
            TokenKind::Word(s) if s == "new" => Err(ScriptError::at(
                "constructors are not supported",
                token.offset,
            )),
            TokenKind::Word(s) => {
                if self.eat("=>") {
                    self.arrow(vec![s])
                } else {
                    Ok(Expr::Ident(s))
                }
            }
            _ => Err(ScriptError::at("expected expression", token.offset)),
        }
    }
}

#[derive(Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}
struct Environment {
    bindings: BTreeMap<String, Binding>,
    parent: Option<usize>,
}
#[derive(Clone)]
struct Function {
    code: FunctionCode,
    environment: usize,
}
enum Flow {
    Normal(Value),
    Return(Value),
    Break,
    Continue,
}
enum Reference {
    Binding(usize, String),
    Property(Value, String),
}

pub struct Runtime {
    environments: Vec<Environment>,
    functions: Vec<Function>,
    arrays: Vec<Vec<Value>>,
    objects: Vec<BTreeMap<String, Value>>,
    handlers: BTreeMap<(NodeId, String), Vec<Value>>,
    property_handlers: BTreeMap<(NodeId, String), Value>,
    ready: Vec<(String, Value)>,
    steps: usize,
    allocated: usize,
    calls: usize,
    eval_depth: usize,
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
        ] {
            bindings.insert(
                name.to_owned(),
                Binding {
                    value,
                    mutable: false,
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
        ] {
            bindings.insert(
                name.to_owned(),
                Binding {
                    value: Self::native(name, Value::Window),
                    mutable: false,
                },
            );
        }
        Self {
            environments: vec![Environment {
                bindings,
                parent: None,
            }],
            functions: Vec::new(),
            arrays: Vec::new(),
            objects: Vec::new(),
            handlers: BTreeMap::new(),
            property_handlers: BTreeMap::new(),
            ready: Vec::new(),
            steps: MAX_STEPS,
            allocated: 2048,
            calls: 0,
            eval_depth: 0,
            console: Vec::new(),
            last_default_prevented: false,
        }
    }

    pub fn execute(&mut self, source: &str, document: &mut Document) -> Result<Value> {
        let program = Parser::program(source)?;
        self.charge(source.len().saturating_mul(3))?;
        self.steps = MAX_STEPS;
        match self.statements(&program, 0, document)? {
            Flow::Normal(value) => Ok(value),
            Flow::Return(_) => Err(ScriptError::new("return outside function")),
            _ => Err(ScriptError::new("loop control outside loop")),
        }
    }

    pub fn dispatch_dom_content_loaded(&mut self, document: &mut Document) -> Result<()> {
        self.steps = MAX_STEPS;
        let callbacks = std::mem::take(&mut self.ready);
        for (event_type, callback) in callbacks {
            let event = self.object(BTreeMap::from([
                ("type".into(), Value::String(Rc::from(event_type))),
                ("target".into(), Value::Document),
                ("currentTarget".into(), Value::Document),
                ("defaultPrevented".into(), Value::Bool(false)),
            ]))?;
            self.call(callback, vec![event], Value::Document, document)?;
        }
        Ok(())
    }

    pub fn dispatch_click(&mut self, target: NodeId, document: &mut Document) -> Result<()> {
        self.dispatch_event(target, "click", document)
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
        if target >= document.nodes.len() {
            return Err(ScriptError::new("invalid event target"));
        }
        self.steps = MAX_STEPS;
        self.last_default_prevented = false;
        let mut path = Vec::new();
        let mut cursor = Some(target);
        while let Some(id) = cursor {
            if path.len() >= 256 {
                return Err(ScriptError::resource("event path limit exceeded"));
            }
            path.push(id);
            cursor = document.nodes[id].parent;
        }
        let event = self.object(BTreeMap::from([
            ("type".into(), Value::String(Rc::from(event_type))),
            ("target".into(), Value::Node(target)),
            ("defaultPrevented".into(), Value::Bool(false)),
            ("cancelBubble".into(), Value::Bool(false)),
        ]))?;
        let Value::Object(event_id) = event else {
            unreachable!()
        };
        for id in path {
            self.tick()?;
            self.objects[event_id].insert("currentTarget".into(), Value::Node(id));
            if let Some(callback) = self
                .property_handlers
                .get(&(id, event_type.into()))
                .cloned()
            {
                if matches!(callback, Value::Function(_) | Value::Native(_))
                    && self.call(callback, vec![event.clone()], Value::Node(id), document)?
                        == Value::Bool(false)
                {
                    self.last_default_prevented = true;
                }
            } else if let Some(source) = document
                .attr(id, &format!("on{event_type}"))
                .map(str::to_owned)
            {
                self.charge(source.len().saturating_mul(3))?;
                let program = Parser::program(&source)?;
                let env = self.environment(0)?;
                self.define(env, "event", event.clone(), false)?;
                self.define(env, "this", Value::Node(id), false)?;
                if let Flow::Return(Value::Bool(false)) =
                    self.statements(&program, env, document)?
                {
                    self.last_default_prevented = true;
                }
            }
            let handlers = self
                .handlers
                .get(&(id, event_type.into()))
                .cloned()
                .unwrap_or_default();
            for callback in handlers {
                self.call(callback, vec![event.clone()], Value::Node(id), document)?;
            }
            if self.objects[event_id]
                .get("cancelBubble")
                .is_some_and(Value::truthy)
            {
                break;
            }
        }
        self.last_default_prevented |= self.objects[event_id]
            .get("defaultPrevented")
            .is_some_and(Value::truthy);
        Ok(())
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
    fn string(&mut self, text: impl Into<String>) -> Result<Value> {
        let text = text.into();
        if text.len() > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.charge(text.len() + 24)?;
        Ok(Value::String(Rc::from(text)))
    }
    fn array(&mut self, values: Vec<Value>) -> Result<Value> {
        self.charge(32 + values.len() * std::mem::size_of::<Value>())?;
        let id = self.arrays.len();
        self.arrays.push(values);
        Ok(Value::Array(id))
    }
    fn object(&mut self, values: BTreeMap<String, Value>) -> Result<Value> {
        self.charge(48 + values.len() * 128)?;
        let id = self.objects.len();
        self.objects.push(values);
        Ok(Value::Object(id))
    }
    fn environment(&mut self, parent: usize) -> Result<usize> {
        self.charge(128)?;
        let id = self.environments.len();
        self.environments.push(Environment {
            bindings: BTreeMap::new(),
            parent: Some(parent),
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
        self.environments[env]
            .bindings
            .insert(name.into(), Binding { value, mutable });
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
    fn function_value(&mut self, code: &FunctionCode, environment: usize) -> Result<Value> {
        self.charge(128)?;
        let id = self.functions.len();
        self.functions.push(Function {
            code: code.clone(),
            environment,
        });
        Ok(Value::Function(id))
    }
    fn native(name: &str, receiver: Value) -> Value {
        Value::Native(Rc::new(Native {
            name: name.into(),
            receiver,
        }))
    }

    fn statements(&mut self, body: &[Stmt], env: usize, doc: &mut Document) -> Result<Flow> {
        for statement in body {
            if let Stmt::Function(name, code) = statement {
                let function = self.function_value(code, env)?;
                self.define(env, name, function, true)?;
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
        self.tick()?;
        match statement {
            Stmt::Empty | Stmt::Function(_, _) => {}
            Stmt::Expr(expression) => return Ok(Flow::Normal(self.eval(expression, env, doc)?)),
            Stmt::Var(bindings, mutable) => {
                for (name, expression) in bindings {
                    let value = if let Some(expression) = expression {
                        self.eval(expression, env, doc)?
                    } else {
                        Value::Undefined
                    };
                    self.define(env, name, value, *mutable)?;
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
            Stmt::For(init, condition, update, body) => {
                let child = self.environment(env)?;
                if let Some(init) = init {
                    self.statement(init, child, doc)?;
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
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                        _ => {}
                    }
                    if let Some(update) = update {
                        self.eval(update, child, doc)?;
                    }
                }
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
                return Err(ScriptError::thrown(value));
            }
            Stmt::Try(body, handler, finalizer) => {
                let mut completion = self.statement(body, env, doc);
                if let Err(error) = completion {
                    // Quota exhaustion is a host termination, not a JavaScript
                    // exception. Running either handler could hide that failure.
                    if error.is_resource_limit() {
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
                    .is_err_and(|error| error.is_resource_limit())
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
                let name = self.string(name)?;
                let message = self.string(message)?;
                self.object(BTreeMap::from([
                    ("name".into(), name),
                    ("message".into(), message),
                ]))
            }
            error => Err(error),
        }
    }
    fn eval(&mut self, expression: &Expr, env: usize, doc: &mut Document) -> Result<Value> {
        if self.eval_depth >= MAX_DEPTH {
            return Err(ScriptError::resource(
                "expression evaluation nesting limit exceeded",
            ));
        }
        self.eval_depth += 1;
        let result = self.eval_inner(expression, env, doc);
        self.eval_depth -= 1;
        result
    }
    fn eval_inner(&mut self, expression: &Expr, env: usize, doc: &mut Document) -> Result<Value> {
        self.tick()?;
        match expression {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Ident(name) => self
                .lookup(env, name)
                .map(|(_, value)| value)
                .ok_or_else(|| ScriptError::reference(format!("'{name}' is not defined"))),
            Expr::Array(items) => {
                let values = items
                    .iter()
                    .map(|item| self.eval(item, env, doc))
                    .collect::<Result<Vec<_>>>()?;
                self.array(values)
            }
            Expr::Object(items) => {
                let mut values = BTreeMap::new();
                for (key, expression) in items {
                    values.insert(key.clone(), self.eval(expression, env, doc)?);
                }
                self.object(values)
            }
            Expr::Unary(op, expression) => {
                if op == "typeof"
                    && let Expr::Ident(name) = &**expression
                    && self.lookup(env, name).is_none()
                {
                    return self.string("undefined");
                }
                let value = self.eval(expression, env, doc)?;
                match op.as_str() {
                    "!" => Ok(Value::Bool(!value.truthy())),
                    "-" => Ok(Value::Number(-value.number())),
                    "+" => Ok(Value::Number(value.number())),
                    "~" => Ok(Value::Number((!to_i32(value.number())) as f64)),
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
            Expr::Binary(op, left, right) => {
                let left = self.eval(left, env, doc)?;
                if op == "&&" && !left.truthy()
                    || op == "||" && left.truthy()
                    || op == "??" && !matches!(left, Value::Undefined | Value::Null)
                {
                    return Ok(left);
                }
                let right = self.eval(right, env, doc)?;
                if op == "&&" || op == "||" || op == "??" {
                    return Ok(right);
                }
                self.binary_value(op, left, right)
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
                    value = self.binary_value(&op[..op.len() - 1], old, value)?;
                }
                self.write_reference(reference, value.clone(), doc)?;
                Ok(value)
            }
            Expr::Update(target, delta, prefix) => {
                let reference = self.reference(target, env, doc)?;
                let old = self.read_reference(&reference, doc)?.number();
                let value = Value::Number(old + delta);
                self.write_reference(reference, value.clone(), doc)?;
                Ok(if *prefix { value } else { Value::Number(old) })
            }
            Expr::Member(object, property) => {
                let object = self.eval(object, env, doc)?;
                let property = self.eval(property, env, doc)?.to_string();
                self.get(object, &property, doc)
            }
            Expr::Call(callee, arguments) => {
                let (function, receiver) = if let Expr::Member(object, property) = &**callee {
                    let receiver = self.eval(object, env, doc)?;
                    let property = self.eval(property, env, doc)?.to_string();
                    (self.get(receiver.clone(), &property, doc)?, receiver)
                } else {
                    (self.eval(callee, env, doc)?, Value::Window)
                };
                let arguments = arguments
                    .iter()
                    .map(|argument| self.eval(argument, env, doc))
                    .collect::<Result<Vec<_>>>()?;
                self.call(function, arguments, receiver, doc)
            }
            Expr::Function(code) => self.function_value(code, env),
        }
    }
    fn binary_value(&mut self, op: &str, left: Value, right: Value) -> Result<Value> {
        if op == "+" && (matches!(left, Value::String(_)) || matches!(right, Value::String(_))) {
            let a = left.to_string();
            let b = right.to_string();
            if a.len().saturating_add(b.len()) > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
            return self.string(a + &b);
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
        let a = left.number();
        let b = right.number();
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
                let owner = self
                    .lookup(env, name)
                    .map(|(owner, _)| owner)
                    .ok_or_else(|| ScriptError::reference(format!("'{name}' is not defined")))?;
                Ok(Reference::Binding(owner, name.clone()))
            }
            Expr::Member(object, property) => {
                let object = self.eval(object, env, doc)?;
                let property = self.eval(property, env, doc)?.to_string();
                Ok(Reference::Property(object, property))
            }
            _ => Err(ScriptError::type_error("invalid assignment target")),
        }
    }
    fn read_reference(&mut self, reference: &Reference, doc: &mut Document) -> Result<Value> {
        match reference {
            Reference::Binding(env, name) => {
                Ok(self.environments[*env].bindings[name].value.clone())
            }
            Reference::Property(object, key) => self.get(object.clone(), key, doc),
        }
    }
    fn write_reference(
        &mut self,
        reference: Reference,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        match reference {
            Reference::Binding(env, name) => {
                let binding = self.environments[env].bindings.get_mut(&name).unwrap();
                if !binding.mutable {
                    return Err(ScriptError::type_error(format!(
                        "cannot assign to constant '{name}'"
                    )));
                }
                binding.value = value;
                Ok(())
            }
            Reference::Property(object, key) => self.set(object, &key, value, doc),
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
        self.calls += 1;
        let result = self.call_inner(function, arguments, receiver, doc);
        self.calls -= 1;
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
                let function = self.functions[id].clone();
                let env = self.environment(function.environment)?;
                self.define(env, "this", receiver, false)?;
                for (i, parameter) in function.code.params.iter().enumerate() {
                    self.define(
                        env,
                        parameter,
                        arguments.get(i).cloned().unwrap_or(Value::Undefined),
                        true,
                    )?;
                }
                match self.statements(&function.code.body, env, doc)? {
                    Flow::Return(value) => Ok(value),
                    Flow::Normal(_) => Ok(Value::Undefined),
                    _ => Err(ScriptError::new("loop control outside loop")),
                }
            }
            Value::Native(native) => self.native_call(&native, arguments, doc),
            _ => Err(ScriptError::type_error("value is not callable")),
        }
    }

    fn get(&mut self, receiver: Value, key: &str, doc: &mut Document) -> Result<Value> {
        if matches!(receiver, Value::Document)
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
                if ["preventDefault", "stopPropagation"].contains(&key) {
                    return Ok(Self::native(key, receiver));
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
                    return Ok(Value::Number(text.encode_utf16().count() as f64));
                }
                if let Ok(index) = key.parse::<usize>() {
                    return match text.chars().nth(index) {
                        Some(c) => self.string(c.to_string()),
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
                    "toString",
                ]
                .contains(&key)
                {
                    return Ok(Self::native(key, receiver));
                }
            }
            Value::Number(_) | Value::Bool(_) if key == "toString" => {
                return Ok(Self::native(key, receiver));
            }
            Value::Window => {
                if let Some((_, value)) = self.lookup(0, key) {
                    return Ok(value);
                }
                if ["addEventListener", "removeEventListener"].contains(&key) {
                    return Ok(Self::native(key, receiver));
                }
            }
            Value::Console if ["log", "warn", "error", "info", "debug"].contains(&key) => {
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
                "body" | "head" => {
                    return Ok(doc
                        .query_selector(key)
                        .map(Value::Node)
                        .unwrap_or(Value::Null));
                }
                "documentElement" => {
                    return Ok(doc
                        .query_selector("html")
                        .map(Value::Node)
                        .unwrap_or(Value::Node(doc.root)));
                }
                "title" => {
                    let text = doc
                        .query_selector("title")
                        .map(|id| doc.text_content(id))
                        .unwrap_or_default();
                    return self.string(text);
                }
                "readyState" => return self.string("complete"),
                "querySelector"
                | "querySelectorAll"
                | "getElementById"
                | "getElementsByTagName"
                | "getElementsByClassName"
                | "createElement"
                | "createTextNode"
                | "addEventListener"
                | "removeEventListener" => return Ok(Self::native(key, receiver)),
                _ => {}
            },
            Value::Node(id) => {
                let id = *id;
                if id >= doc.nodes.len() {
                    return Err(ScriptError::new("invalid DOM node"));
                }
                match key {
                    "textContent" | "innerText" => return self.string(doc.text_content(id)),
                    "innerHTML" => return self.string(serialize_children(doc, id)),
                    "outerHTML" => return self.string(serialize_node(doc, id, 0)),
                    "id" | "title" | "value" | "href" | "src" | "type" => {
                        return self.string(doc.attr(id, key).unwrap_or(""));
                    }
                    "className" => return self.string(doc.attr(id, "class").unwrap_or("")),
                    "tagName" | "nodeName" => {
                        return self.string(doc.tag(id).unwrap_or("#text").to_ascii_uppercase());
                    }
                    "nodeType" => {
                        return Ok(Value::Number(match doc.nodes[id].kind {
                            NodeKind::Document => 9.0,
                            NodeKind::Element(_) => 1.0,
                            NodeKind::Text(_) => 3.0,
                        }));
                    }
                    "parentNode" | "parentElement" => {
                        return Ok(doc.nodes[id].parent.map(Value::Node).unwrap_or(Value::Null));
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
                    "onclick" | "oninput" | "onchange" | "onsubmit" | "onkeydown" | "onkeyup" => {
                        return Ok(self
                            .property_handlers
                            .get(&(id, key[2..].into()))
                            .cloned()
                            .unwrap_or(Value::Null));
                    }
                    "style" => return Ok(Value::Style(id)),
                    "classList" => return Ok(Value::ClassList(id)),
                    "checked" | "disabled" | "hidden" => {
                        return Ok(Value::Bool(doc.attr(id, key).is_some()));
                    }
                    "addEventListener"
                    | "removeEventListener"
                    | "appendChild"
                    | "append"
                    | "removeChild"
                    | "remove"
                    | "setAttribute"
                    | "getAttribute"
                    | "hasAttribute"
                    | "removeAttribute"
                    | "querySelector"
                    | "querySelectorAll" => return Ok(Self::native(key, receiver)),
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
        match receiver {
            Value::Object(id) => {
                if !self.objects[id].contains_key(key) {
                    self.charge(key.len() + 96)?;
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
                        self.arrays[id].resize(index + 1, Value::Undefined);
                    }
                    self.arrays[id][index] = value;
                }
            }
            Value::Window => {
                if let Some((env, _)) = self.lookup(0, key) {
                    self.write_reference(Reference::Binding(env, key.into()), value, doc)?;
                } else {
                    self.define(0, key, value, true)?;
                }
            }
            Value::Document if key == "title" => {
                let id = if let Some(id) = doc.query_selector("title") {
                    id
                } else {
                    self.ensure_dom_capacity(doc, 1)?;
                    let id = doc.create_element("title");
                    let parent = doc.query_selector("head").unwrap_or(doc.root);
                    doc.append_child(parent, id);
                    id
                };
                self.ensure_dom_capacity(doc, 1)?;
                let text = value.to_string();
                self.charge(text.len())?;
                doc.set_text_content(id, &text);
            }
            Value::Node(id) => {
                if id >= doc.nodes.len() {
                    return Err(ScriptError::new("invalid DOM node"));
                }
                if [
                    "onclick",
                    "oninput",
                    "onchange",
                    "onsubmit",
                    "onkeydown",
                    "onkeyup",
                ]
                .contains(&key)
                {
                    if !matches!(
                        value,
                        Value::Function(_) | Value::Native(_) | Value::Null | Value::Undefined
                    ) {
                        return Err(ScriptError::new("event handler must be a function"));
                    }
                    self.charge(64)?;
                    self.property_handlers.insert((id, key[2..].into()), value);
                    return Ok(());
                }
                let text = value.to_string();
                self.charge(text.len())?;
                match key {
                    "textContent" | "innerText" => {
                        self.ensure_dom_capacity(doc, 1)?;
                        doc.set_text_content(id, &text);
                    }
                    "innerHTML" => self.set_inner_html(id, &text, doc)?,
                    "className" => doc.set_attr(id, "class", &text),
                    "id" | "title" | "value" | "href" | "src" | "type" => {
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

    fn set_inner_html(&mut self, id: NodeId, source: &str, doc: &mut Document) -> Result<()> {
        if source.len() > MAX_SOURCE {
            return Err(ScriptError::resource("HTML fragment source limit exceeded"));
        }
        let fragment = Document::parse(source);
        self.ensure_dom_capacity(doc, fragment.nodes.len())?;
        for child in doc.nodes[id].children.clone() {
            doc.nodes[child].parent = None;
        }
        doc.nodes[id].children.clear();
        let fragment_root = fragment.query_selector("body").unwrap_or(fragment.root);
        for child in fragment.nodes[fragment_root].children.clone() {
            import_node(doc, id, &fragment, child, 0);
        }
        Ok(())
    }

    fn native_call(
        &mut self,
        native: &Native,
        args: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        let name = native.name.as_str();
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
        match &native.receiver {
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
                    "String" => self.string(if args.is_empty() {
                        String::new()
                    } else {
                        value.to_string()
                    }),
                    "Number" => Ok(Value::Number(if args.is_empty() {
                        0.0
                    } else {
                        value.number()
                    })),
                    "Boolean" => Ok(Value::Bool(value.truthy())),
                    "isNaN" => Ok(Value::Bool(value.number().is_nan())),
                    "isFinite" => Ok(Value::Bool(value.number().is_finite())),
                    "parseFloat" => Ok(Value::Number(parse_float(&value.to_string()))),
                    "parseInt" => Ok(Value::Number(parse_int(
                        &value.to_string(),
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
                            self.arrays[id].splice(0..0, args);
                        }
                        return Ok(Value::Number(self.arrays[id].len() as f64));
                    }
                    "pop" => return Ok(self.arrays[id].pop().unwrap_or(Value::Undefined)),
                    "shift" => {
                        return Ok(if self.arrays[id].is_empty() {
                            Value::Undefined
                        } else {
                            self.arrays[id].remove(0)
                        });
                    }
                    "join" => {
                        let separator = args
                            .first()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| ",".into());
                        let mut result = String::new();
                        for (i, value) in self.arrays[id].iter().enumerate() {
                            let text = if matches!(value, Value::Null | Value::Undefined) {
                                String::new()
                            } else {
                                value.to_string()
                            };
                            if result
                                .len()
                                .saturating_add(text.len())
                                .saturating_add(separator.len())
                                > MAX_STRING
                            {
                                return Err(ScriptError::resource("script string limit exceeded"));
                            }
                            if i > 0 {
                                result.push_str(&separator);
                            }
                            result.push_str(&text);
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
                let needle = arg(0).to_string();
                match name {
                    "toUpperCase" => return self.string(text.to_uppercase()),
                    "toLowerCase" => return self.string(text.to_lowercase()),
                    "trim" => return self.string(text.trim()),
                    "toString" => return Ok(native.receiver.clone()),
                    "includes" => return Ok(Value::Bool(text.contains(&needle))),
                    "startsWith" => return Ok(Value::Bool(text.starts_with(&needle))),
                    "endsWith" => return Ok(Value::Bool(text.ends_with(&needle))),
                    "indexOf" => {
                        return Ok(Value::Number(
                            text.find(&needle)
                                .map(|i| text[..i].encode_utf16().count() as f64)
                                .unwrap_or(-1.0),
                        ));
                    }
                    "charAt" => {
                        return self.string(
                            text.chars()
                                .nth(arg(0).number().max(0.0) as usize)
                                .map(|c| c.to_string())
                                .unwrap_or_default(),
                        );
                    }
                    "slice" | "substring" => {
                        let characters: Vec<char> = text.chars().collect();
                        let len = characters.len();
                        let mut start = if name == "slice" {
                            relative_index(arg(0).number(), len)
                        } else {
                            arg(0).number().max(0.0).min(len as f64) as usize
                        };
                        let mut end = args
                            .get(1)
                            .map(|value| {
                                if name == "slice" {
                                    relative_index(value.number(), len)
                                } else {
                                    value.number().max(0.0).min(len as f64) as usize
                                }
                            })
                            .unwrap_or(len);
                        if name == "substring" && start > end {
                            std::mem::swap(&mut start, &mut end);
                        }
                        return self
                            .string(characters[start..end.max(start)].iter().collect::<String>());
                    }
                    "split" => {
                        let limit = args
                            .get(1)
                            .map(|value| value.number().max(0.0) as usize)
                            .unwrap_or(65536)
                            .min(65536);
                        let pieces: Vec<String> = if args.is_empty() {
                            vec![text.to_string()]
                        } else if needle.is_empty() {
                            text.chars().take(limit).map(|c| c.to_string()).collect()
                        } else {
                            text.split(&needle).take(limit).map(str::to_owned).collect()
                        };
                        let mut values = Vec::new();
                        for piece in pieces {
                            values.push(self.string(piece)?);
                        }
                        return self.array(values);
                    }
                    _ => {}
                }
            }
            Value::Number(_) | Value::Bool(_) if name == "toString" => {
                return self.string(native.receiver.to_string());
            }
            Value::Object(id) => {
                if name == "preventDefault" {
                    self.objects[*id].insert("defaultPrevented".into(), Value::Bool(true));
                    return Ok(Value::Undefined);
                }
                if name == "stopPropagation" {
                    self.objects[*id].insert("cancelBubble".into(), Value::Bool(true));
                    return Ok(Value::Undefined);
                }
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

        if ["addEventListener", "removeEventListener"].contains(&name) {
            let event = arg(0).to_string();
            let callback = arg(1);
            if !matches!(callback, Value::Function(_) | Value::Native(_)) {
                return Err(ScriptError::new("event listener must be a function"));
            }
            if event == "DOMContentLoaded" || event == "load" {
                if name == "addEventListener" {
                    self.charge(64)?;
                    self.ready.push((event, callback));
                } else {
                    self.ready
                        .retain(|(kind, item)| *kind != event || *item != callback);
                }
            } else {
                let id = if let Value::Node(id) = native.receiver {
                    id
                } else {
                    doc.root
                };
                if name == "addEventListener" {
                    self.charge(event.len() + 96)?;
                    let handlers = self.handlers.entry((id, event)).or_default();
                    if !handlers.contains(&callback) {
                        handlers.push(callback);
                    }
                } else if let Some(handlers) = self.handlers.get_mut(&(id, event)) {
                    handlers.retain(|item| *item != callback);
                }
            }
            return Ok(Value::Undefined);
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
            let candidates = if name == "getElementById" {
                doc.query_selector_all("*")
                    .into_iter()
                    .filter(|id| doc.attr(*id, "id") == Some(input.as_str()))
                    .collect::<Vec<_>>()
            } else {
                doc.query_selector_all(&selector)
            };
            self.charge(candidates.len() * 8)?;
            let candidates: Vec<NodeId> = candidates
                .into_iter()
                .filter(|id| match native.receiver {
                    Value::Node(parent) => is_descendant(doc, *id, parent),
                    _ => true,
                })
                .collect();
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
            self.ensure_dom_capacity(doc, 1)?;
            return Ok(Value::Node(doc.create_element(&tag.to_ascii_lowercase())));
        }
        if name == "createTextNode" {
            let text = arg(0).to_string();
            self.ensure_dom_capacity(doc, 1)?;
            self.charge(text.len())?;
            return Ok(Value::Node(doc.create_text_node(&text)));
        }
        if let Value::Node(id) = native.receiver {
            match name {
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
                    doc.set_attr(id, &key.to_ascii_lowercase(), &text);
                    return Ok(Value::Undefined);
                }
                "removeAttribute" => {
                    doc.remove_attr(id, &arg(0).to_string());
                    return Ok(Value::Undefined);
                }
                "appendChild" | "removeChild" => {
                    let Value::Node(child) = arg(0) else {
                        return Err(ScriptError::new("expected DOM node"));
                    };
                    if name == "appendChild" {
                        doc.append_child(id, child);
                    } else {
                        if doc.nodes.get(child).and_then(|node| node.parent) != Some(id) {
                            return Err(ScriptError::new("node is not a child"));
                        }
                        doc.nodes[id].children.retain(|item| *item != child);
                        doc.nodes[child].parent = None;
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
                        doc.append_child(id, child);
                    }
                    return Ok(Value::Undefined);
                }
                "remove" => {
                    if let Some(parent) = doc.nodes[id].parent {
                        doc.nodes[parent].children.retain(|child| *child != id);
                        doc.nodes[id].parent = None;
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

fn to_i32(number: f64) -> i32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    number.trunc().rem_euclid(4294967296.0) as u32 as i32
}
fn relative_index(number: f64, len: usize) -> usize {
    if number.is_nan() {
        0
    } else if number < 0.0 {
        (len as f64 + number.trunc()).max(0.0) as usize
    } else {
        number.min(len as f64) as usize
    }
}
fn parse_float(text: &str) -> f64 {
    let text = text.trim_start();
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
    let text = text.trim_start();
    let sign = if text.starts_with('-') { -1.0 } else { 1.0 };
    let text = text.strip_prefix(['-', '+']).unwrap_or(text);
    let mut radix = radix as u32;
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
    let mut n = 0.0;
    let mut count = 0;
    for c in text.chars() {
        let Some(digit) = c.to_digit(radix) else {
            break;
        };
        n = n * radix as f64 + digit as f64;
        count += 1;
    }
    if count == 0 { f64::NAN } else { sign * n }
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
fn is_descendant(doc: &Document, child: NodeId, parent: NodeId) -> bool {
    let mut cursor = doc.nodes.get(child).and_then(|node| node.parent);
    for _ in 0..256 {
        let Some(id) = cursor else {
            return false;
        };
        if id == parent {
            return true;
        }
        cursor = doc.nodes.get(id).and_then(|node| node.parent);
    }
    false
}
fn import_node(doc: &mut Document, parent: NodeId, source: &Document, node: NodeId, depth: usize) {
    if depth >= 96 || doc.nodes.len() >= MAX_NODES {
        return;
    }
    let id = match &source.nodes[node].kind {
        NodeKind::Document => return,
        NodeKind::Text(text) => doc.create_text_node(text),
        NodeKind::Element(element) => {
            let id = doc.create_element(&element.tag);
            for (key, value) in &element.attrs {
                doc.set_attr(id, key, value);
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
        NodeKind::Document => serialize_children_at(doc, id, depth),
        NodeKind::Element(element) => {
            let mut result = format!("<{}", element.tag);
            for (key, value) in &element.attrs {
                result.push_str(&format!(" {key}=\"{}\"", escape_html(value, true)));
            }
            result.push('>');
            if ![
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
                .message
                .contains("call stack")
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
            run("const a = []; a[4294967295] = 1;")
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
        assert!(
            run("eval('1 + 1')")
                .unwrap_err()
                .message
                .contains("not defined")
        );
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
        assert!(runtime.last_default_prevented);
        assert_eq!(
            doc.text_content(doc.query_selector("#out").unwrap()),
            "typed value"
        );
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
}
