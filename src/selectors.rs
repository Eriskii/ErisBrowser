//! Bounded token-aware selector compilation. Matching strings are emitted only
//! after parsing token grammar; comments never invent whitespace or join tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Ident,
    Function,
    Hash(bool),
    String,
    Number,
    Dimension,
    Percentage,
    Whitespace,
    Open(char),
    Close(char),
    Comma,
    Delim(char),
    Url,
    Bad,
    Other,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct Token<'a> {
    pub kind: Kind,
    pub raw: &'a str,
    pub start: usize,
    pub end: usize,
}
impl Token<'_> {
    pub fn is_name(&self, name: &str) -> bool {
        let raw = if self.kind == Kind::Function {
            &self.raw[..self.raw.len() - 1]
        } else {
            self.raw
        };
        let mut at = 0;
        let mut expected = name.chars();
        while at < raw.len() {
            let (ch, next) = if raw[at..].starts_with('\\') {
                escape(raw, at + 1)
            } else {
                let ch = raw[at..].chars().next().unwrap();
                (ch, at + ch.len_utf8())
            };
            if expected.next() != Some(ch.to_ascii_lowercase()) {
                return false;
            }
            at = next;
        }
        expected.next().is_none()
    }
    pub fn open(&self) -> Option<char> {
        match self.kind {
            Kind::Open(ch) => Some(ch),
            Kind::Function => Some('('),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Error {
    Invalid,
    Namespace,
    Limit,
}
pub(crate) fn spend(work: &mut usize, cost: usize) -> Result<(), Error> {
    if cost > *work {
        *work = 0;
        return Err(Error::Limit);
    }
    *work -= cost;
    Ok(())
}
pub(crate) fn space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | '\x0c')
}
// Byte spans are borrowed; maximum storage is bounded independently of source
// size, and every scan is prepaid. EOF comments produce no token.
pub(crate) fn tokens<'a>(source: &'a str, work: &mut usize) -> Result<Vec<Token<'a>>, Error> {
    if source.len() > 512 * 1024 {
        return Err(Error::Limit);
    }
    spend(work, source.len().saturating_mul(2) + 1)?;
    let mut out = Vec::new();
    let mut at = 0;
    while at < source.len() {
        if source[at..].starts_with("/*") {
            at = source[at + 2..]
                .find("*/")
                .map_or(source.len(), |end| at + end + 4);
            continue;
        }
        if out.len() >= 16384 {
            return Err(Error::Limit);
        }
        let (kind, count) = token(&source[at..]);
        out.push(Token {
            kind,
            raw: &source[at..at + count],
            start: at,
            end: at + count,
        });
        at += count;
    }
    Ok(out)
}
pub(crate) fn token(source: &str) -> (Kind, usize) {
    let first = source.chars().next().unwrap();
    if space(first) {
        return (
            Kind::Whitespace,
            source.find(|c| !space(c)).unwrap_or(source.len()),
        );
    }
    if matches!(first, '\'' | '"') {
        let mut at = 1;
        while at < source.len() {
            let ch = source[at..].chars().next().unwrap();
            if ch == first {
                return (Kind::String, at + 1);
            }
            if matches!(ch, '\n' | '\r' | '\x0c') {
                return (Kind::Bad, at);
            }
            if ch == '\\' {
                at += 1;
                if source[at..].starts_with("\r\n") {
                    at += 2;
                } else if source[at..].starts_with(['\n', '\r', '\x0c']) {
                    at += 1;
                } else {
                    at = escape(source, at).1;
                }
            } else {
                at += ch.len_utf8();
            }
        }
        return (Kind::String, at);
    }
    if let Some(mut end) = number_end(source) {
        if ident_start(&source[end..]) {
            end += ident_sequence(&source[end..]).0;
            return (Kind::Dimension, end);
        }
        if source[end..].starts_with('%') {
            return (Kind::Percentage, end + 1);
        }
        return (Kind::Number, end);
    }
    if matches!(first, '@' | '#') {
        let rest = &source[1..];
        if ident_start(rest) || first == '#' && rest.chars().next().is_some_and(name_char) {
            return (
                if first == '#' {
                    Kind::Hash(ident_start(rest))
                } else {
                    Kind::Other
                },
                1 + ident_sequence(rest).0,
            );
        }
    }
    if ident_start(source) {
        let (end, url) = ident_sequence(source);
        if source[end..].starts_with('(') {
            if url && !url_quoted(&source[end..]) {
                return url_token(source, end + 1);
            }
            return (Kind::Function, end + 1);
        }
        return (Kind::Ident, end);
    }
    (
        match first {
            '(' | '[' | '{' => Kind::Open(first),
            ')' | ']' | '}' => Kind::Close(first),
            ',' => Kind::Comma,
            _ => Kind::Delim(first),
        },
        first.len_utf8(),
    )
}
fn name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '\0') || !ch.is_ascii()
}
pub(crate) fn ident_start(source: &str) -> bool {
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let start = |ch: char| ch.is_ascii_alphabetic() || matches!(ch, '_' | '\0') || !ch.is_ascii();
    let escape =
        |rest: &str| rest.starts_with('\\') && !rest[1..].starts_with(['\n', '\r', '\x0c']);
    start(first)
        || escape(source)
        || first == '-'
            && (chars.next().is_some_and(|ch| start(ch) || ch == '-') || escape(&source[1..]))
}
pub(crate) fn ident_sequence(source: &str) -> (usize, bool) {
    let mut at = 0;
    let mut count = 0;
    let mut url = true;
    while at < source.len() {
        let mut ch = source[at..].chars().next().unwrap();
        if ch == '\\' && !source[at + 1..].starts_with(['\n', '\r', '\x0c']) {
            (ch, at) = escape(source, at + 1);
        } else if name_char(ch) {
            at += ch.len_utf8();
        } else {
            break;
        }
        url &= Some(ch.to_ascii_lowercase()) == ['u', 'r', 'l'].get(count).copied();
        count += 1;
    }
    (at, url && count == 3)
}
// `at` follows the backslash. Input preprocessing's CRLF/FF rules are applied
// locally while consuming optional escape whitespace, without allocating.
fn escape(source: &str, mut at: usize) -> (char, usize) {
    let Some(ch) = source[at..].chars().next() else {
        return ('\u{fffd}', at);
    };
    if !ch.is_ascii_hexdigit() {
        return (ch, at + ch.len_utf8());
    }
    let mut value = 0;
    for _ in 0..6 {
        let Some(ch) = source
            .as_bytes()
            .get(at)
            .filter(|ch| ch.is_ascii_hexdigit())
        else {
            break;
        };
        value = value * 16 + char::from(*ch).to_digit(16).unwrap();
        at += 1;
    }
    if source[at..].starts_with("\r\n") {
        at += 2;
    } else if source[at..].chars().next().is_some_and(space) {
        at += 1;
    }
    (
        char::from_u32(value)
            .filter(|ch| *ch != '\0')
            .unwrap_or('\u{fffd}'),
        at,
    )
}
pub(crate) fn url_quoted(rest: &str) -> bool {
    rest.strip_prefix('(')
        .is_some_and(|inner| inner.trim_start_matches(space).starts_with(['\'', '"']))
}
fn url_token(source: &str, mut at: usize) -> (Kind, usize) {
    while source[at..].chars().next().is_some_and(space) {
        at += 1;
    }
    let mut bad = false;
    while at < source.len() {
        let ch = source[at..].chars().next().unwrap();
        at += ch.len_utf8();
        if ch == ')' {
            return (if bad { Kind::Bad } else { Kind::Url }, at);
        }
        if ch == '\\' && !source[at..].starts_with(['\n', '\r', '\x0c']) {
            at = escape(source, at).1;
        } else if !bad {
            if space(ch) {
                while source[at..].chars().next().is_some_and(space) {
                    at += 1;
                }
                bad = at < source.len() && !source[at..].starts_with(')');
            } else if matches!(ch, '\'' | '"' | '(' | '\\' | '\x01'..='\x08' | '\x0b' | '\x0e'..='\x1f' | '\x7f')
            {
                bad = true;
            }
        }
    }
    (if bad { Kind::Bad } else { Kind::Url }, at)
}
pub(crate) fn number_end(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut at = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let start = at;
    while bytes.get(at).is_some_and(u8::is_ascii_digit) {
        at += 1;
    }
    let mut digits = at - start;
    if bytes.get(at) == Some(&b'.') && bytes.get(at + 1).is_some_and(u8::is_ascii_digit) {
        at += 1;
        let start = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        digits += at - start;
    }
    if digits == 0 {
        return None;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        let mut exponent = at + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        if bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            at = exponent + 1;
            while bytes.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
        }
    }
    Some(at)
}

pub(crate) fn trim<'t, 's>(mut tokens: &'t [Token<'s>]) -> &'t [Token<'s>] {
    while tokens.first().is_some_and(|t| t.kind == Kind::Whitespace) {
        tokens = &tokens[1..];
    }
    while tokens.last().is_some_and(|t| t.kind == Kind::Whitespace) {
        tokens = &tokens[..tokens.len() - 1];
    }
    tokens
}
pub(crate) fn closing(
    tokens: &[Token<'_>],
    start: usize,
    limit: usize,
    work: &mut usize,
) -> Result<usize, Error> {
    let mut stack = Vec::new();
    for (at, token) in tokens.iter().enumerate().skip(start) {
        spend(work, 1)?;
        if let Some(open) = token.open() {
            if stack.len() >= limit {
                return Err(Error::Limit);
            }
            stack.push(open);
        } else if let Kind::Close(close) = token.kind {
            if stack.pop()
                != Some(match close {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                })
            {
                return Err(Error::Invalid);
            }
            if stack.is_empty() {
                return Ok(at);
            }
        } else if token.kind == Kind::Bad {
            return Err(Error::Invalid);
        }
    }
    Err(Error::Invalid)
}
#[derive(Debug)]
pub(crate) struct ParsedSelector {
    pub source: String,
    pub supported: bool,
    pub compounds: usize,
    pub depth: usize,
}
struct Parser<'w> {
    work: &'w mut usize,
    parts: usize,
    max_depth: usize,
}
impl Parser<'_> {
    fn compound(
        &mut self,
        tokens: &[Token<'_>],
        depth: usize,
    ) -> Result<(String, bool, usize), Error> {
        if depth > 64 {
            return Err(Error::Limit);
        }
        self.max_depth = self.max_depth.max(depth);
        if self.parts >= 1024 {
            return Err(Error::Limit);
        }
        self.parts += 1;
        let mut out = String::new();
        let mut at = 0;
        let mut supported = true;
        if let Some(token) = tokens.first()
            && matches!(token.kind, Kind::Ident | Kind::Delim('*'))
        {
            self.plain(token)?;
            out.push_str(token.raw);
            at += 1;
        }
        while let Some(token) = tokens.get(at) {
            spend(self.work, 1)?;
            match token.kind {
                Kind::Hash(true) => {
                    self.plain(token)?;
                    out.push_str(token.raw);
                    at += 1;
                }
                Kind::Delim('.') => {
                    let next = tokens.get(at + 1).ok_or(Error::Invalid)?;
                    if next.kind != Kind::Ident {
                        return Err(Error::Invalid);
                    }
                    self.plain(next)?;
                    out.push('.');
                    out.push_str(next.raw);
                    at += 2;
                }
                Kind::Open('[') => {
                    let end = closing(tokens, at, 64, self.work)?;
                    out.push_str(&self.attribute(&tokens[at + 1..end])?);
                    at = end + 1;
                }
                Kind::Delim(':') => {
                    let name = tokens.get(at + 1).ok_or(Error::Invalid)?;
                    if !matches!(name.kind, Kind::Ident | Kind::Function) {
                        return Err(Error::Invalid);
                    }
                    self.plain(name)?;
                    let lower = name
                        .raw
                        .strip_suffix('(')
                        .unwrap_or(name.raw)
                        .to_ascii_lowercase();
                    let raw = lower.as_str();
                    out.push(':');
                    out.push_str(raw);
                    if name.kind == Kind::Function {
                        let end = closing(tokens, at + 1, 64, self.work)?;
                        let inner = &tokens[at + 2..end];
                        out.push('(');
                        match raw {
                            "is" | "where" | "not" => {
                                let mut first = true;
                                for part in self.split(inner)? {
                                    match self.complex(part, depth + 1) {
                                        Ok((value, positive)) => {
                                            if !first {
                                                out.push(',');
                                            }
                                            out.push_str(&value);
                                            first = false;
                                            supported &= positive;
                                        }
                                        Err(Error::Invalid) if matches!(raw, "is" | "where") => {
                                            supported = false;
                                        }
                                        Err(error) => return Err(error),
                                    }
                                }
                            }
                            "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                                out.push_str(&self.nth(inner)?);
                            }
                            "lang" => {
                                let inner = trim(inner);
                                if inner.len() != 1
                                    || !matches!(inner[0].kind, Kind::Ident | Kind::String)
                                {
                                    return Err(Error::Invalid);
                                }
                                self.plain(&inner[0])?;
                                out.push_str(inner[0].raw);
                                supported = false;
                            }
                            _ => return Err(Error::Invalid),
                        }
                        out.push(')');
                        at = end + 1;
                    } else {
                        let positive = matches!(
                            raw,
                            "root"
                                | "empty"
                                | "first-child"
                                | "last-child"
                                | "only-child"
                                | "first-of-type"
                                | "last-of-type"
                                | "only-of-type"
                        );
                        if !positive
                            && !matches!(
                                raw,
                                "scope"
                                    | "checked"
                                    | "disabled"
                                    | "enabled"
                                    | "required"
                                    | "optional"
                                    | "link"
                                    | "any-link"
                            )
                        {
                            return Err(Error::Invalid);
                        }
                        supported &= positive;
                        at += 2;
                    }
                }
                _ => break,
            }
        }
        if out.is_empty() {
            return Err(Error::Invalid);
        }
        Ok((out, supported, at))
    }
    fn plain(&self, token: &Token<'_>) -> Result<(), Error> {
        if token.raw.contains(['\\', '\0']) {
            return Err(Error::Invalid);
        }
        if token.kind == Kind::String
            && (token.raw.len() < 2 || token.raw.as_bytes().last() != token.raw.as_bytes().first())
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    fn attribute(&mut self, tokens: &[Token<'_>]) -> Result<String, Error> {
        let tokens = trim(tokens);
        let name = tokens
            .first()
            .filter(|t| t.kind == Kind::Ident)
            .ok_or(Error::Invalid)?;
        self.plain(name)?;
        let mut out = format!("[{}", name.raw);
        let mut at = 1;
        while tokens.get(at).is_some_and(|t| t.kind == Kind::Whitespace) {
            at += 1;
        }
        if at == tokens.len() {
            out.push(']');
            return Ok(out);
        }
        if let Some(Token {
            kind: Kind::Delim(ch @ ('~' | '|' | '^' | '$' | '*')),
            ..
        }) = tokens.get(at)
        {
            out.push(*ch);
            at += 1;
        }
        if tokens.get(at).is_none_or(|t| t.kind != Kind::Delim('=')) {
            return Err(Error::Invalid);
        }
        out.push('=');
        at += 1;
        while tokens.get(at).is_some_and(|t| t.kind == Kind::Whitespace) {
            at += 1;
        }
        let value = tokens
            .get(at)
            .filter(|t| matches!(t.kind, Kind::String | Kind::Ident))
            .ok_or(Error::Invalid)?;
        self.plain(value)?;
        out.push_str(value.raw);
        at += 1;
        // The modifier follows a separate token. A string naturally separates
        // it without whitespace; identifiers require a lexical separator.
        while tokens.get(at).is_some_and(|t| t.kind == Kind::Whitespace) {
            at += 1;
        }
        if let Some(flag) = tokens.get(at) {
            if flag.kind != Kind::Ident || !matches!(flag.raw, "i" | "I" | "s" | "S") {
                return Err(Error::Invalid);
            }
            out.push(' ');
            out.push_str(if flag.is_name("i") { "i" } else { "s" });
            at += 1;
        }
        if at != tokens.len() {
            return Err(Error::Invalid);
        }
        out.push(']');
        Ok(out)
    }
    fn nth(&mut self, tokens: &[Token<'_>]) -> Result<String, Error> {
        let tokens = trim(tokens);
        let Some(first) = tokens.first() else {
            return Err(Error::Invalid);
        };
        let mut nonspace = tokens
            .iter()
            .filter(|t| t.kind != Kind::Whitespace)
            .copied()
            .take(6)
            .collect::<Vec<_>>();
        if nonspace.len() > 5 {
            return Err(Error::Invalid);
        }
        for token in &nonspace {
            self.plain(token)?;
        }
        if first.kind == Kind::Number {
            if nonspace.len() != 1 || first.raw.parse::<i64>().is_err() {
                return Err(Error::Invalid);
            }
            return Ok(first.raw.into());
        }
        if nonspace.len() == 1
            && first.kind == Kind::Ident
            && matches!(first.raw.to_ascii_lowercase().as_str(), "odd" | "even")
        {
            return Ok(first.raw.to_ascii_lowercase());
        }
        let mut prefix = String::new();
        if first.kind == Kind::Delim('+') {
            // +n requires adjacency in the token grammar, excluding real space.
            if tokens.get(1).is_none_or(|t| t.kind != Kind::Ident) {
                return Err(Error::Invalid);
            }
            prefix.push('+');
            nonspace.remove(0);
        }
        let first = nonspace.first().ok_or(Error::Invalid)?;
        if !matches!(first.kind, Kind::Ident | Kind::Dimension) {
            return Err(Error::Invalid);
        }
        let raw = first.raw.to_ascii_lowercase();
        let (a, b) = raw.split_once('n').ok_or(Error::Invalid)?;
        if !(matches!(a, "" | "-") || a.parse::<i64>().is_ok()) {
            return Err(Error::Invalid);
        }
        if first.kind == Kind::Ident && !matches!(a, "" | "-") {
            return Err(Error::Invalid);
        }
        prefix.push_str(&raw);
        let rest = &nonspace[1..];
        if b.is_empty() {
            match rest {
                [] => {}
                [value]
                    if value.kind == Kind::Number
                        && value.raw.starts_with(['+', '-'])
                        && value.raw.parse::<i64>().is_ok() =>
                {
                    prefix.push_str(value.raw)
                }
                [sign, value]
                    if matches!(sign.kind, Kind::Delim('+') | Kind::Delim('-'))
                        && value.kind == Kind::Number
                        && value.raw.bytes().all(|b| b.is_ascii_digit())
                        && value.raw.parse::<i64>().is_ok() =>
                {
                    prefix.push_str(sign.raw);
                    prefix.push_str(value.raw);
                }
                _ => return Err(Error::Invalid),
            }
        } else if b == "-" {
            if let [value] = rest
                && value.kind == Kind::Number
                && value.raw.bytes().all(|b| b.is_ascii_digit())
                && value.raw.parse::<i64>().is_ok()
            {
                prefix.push_str(value.raw);
            } else {
                return Err(Error::Invalid);
            }
        } else if !(b.starts_with('-')
            && b[1..].bytes().all(|b| b.is_ascii_digit())
            && b.parse::<i64>().is_ok()
            && rest.is_empty())
        {
            return Err(Error::Invalid);
        }
        Ok(prefix)
    }
    fn split<'t>(&mut self, tokens: &'t [Token<'t>]) -> Result<Vec<&'t [Token<'t>]>, Error> {
        let mut parts = Vec::new();
        let mut start = 0;
        let mut at = 0;
        while at < tokens.len() {
            spend(self.work, 1)?;
            if tokens[at].open().is_some() {
                at = closing(tokens, at, 64, self.work)?;
            } else if tokens[at].kind == Kind::Comma {
                parts.push(&tokens[start..at]);
                start = at + 1;
                if parts.len() >= 128 {
                    return Err(Error::Limit);
                }
            } else if matches!(tokens[at].kind, Kind::Close(_) | Kind::Bad) {
                return Err(Error::Invalid);
            }
            at += 1;
        }
        parts.push(&tokens[start..]);
        Ok(parts)
    }
    fn complex(&mut self, tokens: &[Token<'_>], depth: usize) -> Result<(String, bool), Error> {
        let tokens = trim(tokens);
        let bytes = tokens
            .first()
            .zip(tokens.last())
            .map_or(0, |(a, b)| b.end - a.start);
        spend(self.work, bytes.saturating_mul(2) + 1)?;
        let mut at = 0;
        let mut out = String::new();
        let mut supported = true;
        loop {
            let (compound, positive, consumed) = self.compound(&tokens[at..], depth)?;
            supported &= positive;
            out.push_str(&compound);
            at += consumed;
            let before = at;
            while tokens.get(at).is_some_and(|t| t.kind == Kind::Whitespace) {
                at += 1;
            }
            if at == tokens.len() {
                break;
            }
            if let Kind::Delim(ch @ ('>' | '+' | '~')) = tokens[at].kind {
                out.push(ch);
                at += 1;
                while tokens.get(at).is_some_and(|t| t.kind == Kind::Whitespace) {
                    at += 1;
                }
            } else if before != at {
                out.push(' ');
            } else {
                return Err(Error::Invalid);
            }
            if out.len() > 8192 {
                return Err(Error::Limit);
            }
        }
        Ok((out, supported))
    }
}
fn namespace_check(tokens: &[Token<'_>]) -> Result<(), Error> {
    for (at, three) in tokens.windows(2).enumerate() {
        if three[0].kind == Kind::Ident && three[1].kind == Kind::Delim('|') {
            // Exclude the attribute matcher and column combinator.
            if tokens
                .get(at + 2)
                .is_none_or(|t| !matches!(t.kind, Kind::Delim('=') | Kind::Delim('|')))
            {
                return Err(Error::Namespace);
            }
        }
    }
    Ok(())
}
pub(crate) fn parse_list(source: &str, work: &mut usize) -> Result<Vec<ParsedSelector>, Error> {
    let tokens = tokens(source, work)?;
    namespace_check(&tokens)?;
    let mut parser = Parser {
        work,
        parts: 0,
        max_depth: 0,
    };
    let mut result = Vec::new();
    for part in parser.split(&tokens)? {
        let part = trim(part);
        if part
            .first()
            .zip(part.last())
            .is_none_or(|(a, b)| b.end - a.start > 4096)
        {
            return Err(Error::Invalid);
        }
        let before = parser.parts;
        parser.max_depth = 0;
        let (source, supported) = parser.complex(part, 0)?;
        result.push(ParsedSelector {
            source,
            supported,
            compounds: parser.parts - before,
            depth: parser.max_depth,
        });
    }
    Ok(result)
}
pub(crate) fn supports(source: &str, work: &mut usize) -> Result<bool, Error> {
    if source.len() > 4096 {
        return Err(Error::Limit);
    }
    match parse_list(source, work) {
        Ok(list) => {
            if list.iter().any(|s| s.compounds > 64 || s.depth >= 16) {
                return Err(Error::Limit);
            }
            Ok(list.len() == 1 && list[0].supported)
        }
        Err(Error::Invalid) => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_boundaries_ignore_comments_without_inventing_whitespace() {
        let tokens = tokens(
            "div/**/.x div /**/.x #/**/id :/**/is(p) :is/**/(p)",
            &mut 10000,
        )
        .unwrap();
        let kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();
        assert_eq!(
            &kinds[..4],
            &[Kind::Ident, Kind::Delim('.'), Kind::Ident, Kind::Whitespace]
        );
        assert_eq!(
            &kinds[4..9],
            &[
                Kind::Ident,
                Kind::Whitespace,
                Kind::Delim('.'),
                Kind::Ident,
                Kind::Whitespace
            ]
        );
        assert_eq!(&kinds[9..11], &[Kind::Delim('#'), Kind::Ident]);
        assert!(kinds.contains(&Kind::Function));
        assert_eq!(
            tokens.iter().filter(|t| t.kind == Kind::Function).count(),
            1
        );
        let urls = super::tokens(
            r#"url(foo/**/bar) url(foo bar) url("/*literal*/")"#,
            &mut 10000,
        )
        .unwrap();
        assert_eq!(urls[0].kind, Kind::Url);
        assert_eq!(urls[2].kind, Kind::Bad);
        assert!(
            urls.iter()
                .any(|t| t.kind == Kind::String && t.raw == r#""/*literal*/""#)
        );
    }
    #[test]
    fn compiled_selectors_serialize_only_after_token_grammar_validation() {
        for (raw, expected) in [
            ("div/**/.x", "div.x"),
            ("div /**/.x", "div .x"),
            ("./**/x", ".x"),
            (":/**/IS(.x)", ":is(.x)"),
            ("[x~/**/=a]", "[x~=a]"),
            ("[x=a/**/I]", "[x=a i]"),
            (r#"[x="/* ) ] */"S]"#, r#"[x="/* ) ] */" s]"#),
            ("p:nth-child(2n/**/+/**/1)", "p:nth-child(2n+1)"),
            ("p/**/>/**/.x", "p>.x"),
            (":not(/**/.x/**/)", ":not(.x)"),
        ] {
            let list = parse_list(raw, &mut 100000).unwrap();
            assert_eq!(list[0].source, expected, "{raw}");
            assert!(list[0].supported, "{raw}");
        }
        for raw in [
            "div/**/span",
            "#/**/id",
            ":is/**/(p)",
            "[x~ /**/=a]",
            "p:nth-child(2/**/n+1)",
            ":empty(foo)",
            ":not(:madeup)",
            "p,",
            ",p",
        ] {
            assert!(parse_list(raw, &mut 100000).is_err(), "{raw}");
        }
        assert_eq!(
            parse_list(":is(p,:madeup)", &mut 10000).unwrap()[0].source,
            ":is(p)"
        );
        assert!(!supports(":is(p,:madeup)", &mut 10000).unwrap());
        assert_eq!(
            parse_list("x/**/|rect", &mut 10000).unwrap_err(),
            Error::Namespace
        );
        assert_eq!(
            parse_list(&format!("{}/*x*/|rect", "x".repeat(1100)), &mut 10000).unwrap_err(),
            Error::Namespace
        );
    }
    #[test]
    fn token_and_selector_growth_is_charged_and_bounded() {
        let mut work = 2;
        assert_eq!(parse_list("p/**/.x", &mut work).unwrap_err(), Error::Limit);
        assert_eq!(work, 0);
        assert!(parse_list(&"* ".repeat(20000), &mut 1000000).is_err());
        let deep = format!("{}p{}", ":is(".repeat(65), ")".repeat(65));
        assert_eq!(parse_list(&deep, &mut 1000000).unwrap_err(), Error::Limit);
        let large_nested = format!(
            "{}[x='{}']{}",
            ":is(".repeat(16),
            "a".repeat(3000),
            ")".repeat(16)
        );
        assert_eq!(
            parse_list(&large_nested, &mut 10000).unwrap_err(),
            Error::Limit
        );
        let mut input = String::new();
        for _ in 0..4000 {
            input.push_str("/*x*/");
        }
        input.push('p');
        assert_eq!(parse_list(&input, &mut 100000).unwrap()[0].source, "p");
    }
}
