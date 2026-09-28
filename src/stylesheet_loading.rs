//! Bounded stylesheet import loading. Fetch authority always remains the document.
use crate::{
    net::{Fetcher, ResourceKind},
    text_encoding,
};
use encoding_rs::Encoding;
use std::{collections::HashMap, sync::Arc};
use url::Url;

const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMPORTS: usize = 256;
const MAX_DEPTH: usize = 16;
const MAX_URL_BYTES: usize = 32 * 1024 * 1024;
pub(crate) type Sources = Vec<Arc<str>>;

#[derive(Clone, Debug)]
struct Sheet {
    source: Arc<str>,
    url: Url,
    encoding: &'static Encoding,
}

pub(crate) struct Loader {
    document_url: Url,
    environment_encoding: &'static Encoding,
    cache: HashMap<(String, &'static str), Result<Sheet, String>>,
    decoded: usize,
    expanded: usize,
    imports: usize,
    scanned: usize,
    url_work: usize,
    url_storage: usize,
    segments: usize,
}

impl Loader {
    pub(crate) fn new(document_url: &Url, environment_encoding: &'static Encoding) -> Self {
        Self {
            document_url: document_url.clone(),
            environment_encoding,
            cache: HashMap::new(),
            decoded: 0,
            expanded: 0,
            imports: 0,
            scanned: 0,
            url_work: 0,
            url_storage: 0,
            segments: 0,
        }
    }
    pub(crate) fn external(
        &mut self,
        fetcher: &mut Fetcher,
        url: &Url,
        diagnostics: &mut Vec<String>,
    ) -> Result<Sources, String> {
        self.check_segments()?;
        let sheet = self.fetch(fetcher, url, self.environment_encoding)?;
        self.charge_url_work(sheet.url.as_str().len())?;
        let mut path = vec![sheet.url.clone()];
        let sources = self.expand(fetcher, &sheet, &mut path, diagnostics)?;
        self.retain_sources(sources)
    }

    pub(crate) fn inline(
        &mut self,
        fetcher: &mut Fetcher,
        source: &str,
        base: &Url,
        diagnostics: &mut Vec<String>,
    ) -> Result<Sources, String> {
        self.check_segments()?;
        self.charge_url_work(base.as_str().len())?;
        if source.len() > MAX_BYTES.saturating_sub(self.decoded) {
            return Err("stylesheet decoded source budget exceeded".into());
        }
        self.decoded += source.len();
        let sheet = Sheet {
            source: source.into(),
            url: base.clone(),
            encoding: self.environment_encoding,
        };
        let sources = self.expand(fetcher, &sheet, &mut Vec::new(), diagnostics)?;
        self.retain_sources(sources)
    }

    fn check_segments(&self) -> Result<(), String> {
        if self.segments >= MAX_IMPORTS {
            Err("stylesheet segment count exceeded".into())
        } else {
            Ok(())
        }
    }
    fn retain_sources(&mut self, sources: Sources) -> Result<Sources, String> {
        if sources.len() > MAX_IMPORTS.saturating_sub(self.segments) {
            return Err("stylesheet segment count exceeded".into());
        }
        self.segments += sources.len();
        Ok(sources)
    }
    fn charge_url_work(&mut self, bytes: usize) -> Result<(), String> {
        if bytes > MAX_URL_BYTES.saturating_sub(self.url_work) {
            self.url_work = MAX_URL_BYTES;
            return Err("stylesheet URL work budget exceeded".into());
        }
        self.url_work += bytes;
        Ok(())
    }
    fn fetch(
        &mut self,
        fetcher: &mut Fetcher,
        url: &Url,
        fallback: &'static Encoding,
    ) -> Result<Sheet, String> {
        // Both successful and failed lookups pay for the URL clone, key and
        // hashing. Long bases must not turn many tiny import names into an
        // unbounded cache of large strings, even when no fetch succeeds.
        self.charge_url_work(url.as_str().len().saturating_mul(3))?;
        let mut url = url.clone();
        url.set_fragment(None);
        let key = (url.to_string(), fallback.name());
        if let Some(cached) = self.cache.get(&key) {
            let clone_bytes = match cached {
                Ok(sheet) => sheet.url.as_str().len(),
                Err(error) => error.len(),
            };
            self.charge_url_work(clone_bytes)?;
            return self.cache[&key].clone();
        }
        if self.cache.len() >= MAX_IMPORTS {
            return Err("stylesheet resource count exceeded".into());
        }
        if key.0.len() > MAX_URL_BYTES.saturating_sub(self.url_storage) {
            return Err("stylesheet URL storage budget exceeded".into());
        }
        let result: Result<Sheet, String> = (|| {
            let response = fetcher.fetch(&url, Some(&self.document_url), ResourceKind::Style)?;
            let encoding =
                text_encoding::css_encoding(&response.bytes, &response.content_type, fallback);
            let source = text_encoding::decode(&response.bytes, encoding);
            if source.len() > MAX_BYTES.saturating_sub(self.decoded) {
                return Err("stylesheet decoded source budget exceeded".into());
            }
            self.decoded += source.len();
            let mut url = response.url;
            url.set_fragment(None);
            Ok(Sheet {
                source: source.into(),
                url,
                encoding,
            })
        })();
        let clone_bytes = match &result {
            Ok(sheet) => sheet.url.as_str().len(),
            Err(error) => error.len(),
        };
        self.charge_url_work(clone_bytes)?;
        let storage = key.0.len().saturating_add(clone_bytes);
        if storage > MAX_URL_BYTES.saturating_sub(self.url_storage) {
            return Err("stylesheet URL storage budget exceeded".into());
        }
        self.url_storage += storage;
        self.cache.insert(key, result.clone());
        result
    }

    fn append(&mut self, output: &mut String, value: &str) -> Result<(), String> {
        if value.len() > MAX_BYTES.saturating_sub(self.expanded) {
            return Err("stylesheet expanded source budget exceeded".into());
        }
        self.expanded += value.len();
        output.push_str(value);
        Ok(())
    }

    fn expand(
        &mut self,
        fetcher: &mut Fetcher,
        sheet: &Sheet,
        path: &mut Vec<Url>,
        diagnostics: &mut Vec<String>,
    ) -> Result<Sources, String> {
        let source = sheet.source.as_ref();
        // Reusing a cached sheet must still pay for parsing it. Otherwise a DAG
        // of repeated imports could multiply work without consuming fetch bytes.
        if source.len() > (MAX_BYTES * 4).saturating_sub(self.scanned) {
            return Err("stylesheet import scan budget exceeded".into());
        }
        self.scanned += source.len();
        let mut sources = Vec::new();
        let mut output = String::new();
        let mut cursor = Cursor::new(source);
        loop {
            let start = cursor.at;
            cursor.space();
            // CSS stylesheet-level legacy comment delimiters are ignored tokens.
            if cursor.rest().starts_with("<!--") || cursor.rest().starts_with("-->") {
                cursor.at += if cursor.rest().starts_with("<!--") {
                    4
                } else {
                    3
                };
                continue;
            }
            if !cursor.eat('@') {
                self.append(&mut output, &source[start..])?;
                break;
            }
            let name = cursor.ident();
            let prelude = cursor.at;
            let Some((end, block)) = cursor.statement_end() else {
                self.append(&mut output, &source[start..])?;
                break;
            };
            if name.eq_ignore_ascii_case("charset") && !block {
                continue;
            }
            if name.eq_ignore_ascii_case("layer") && !block {
                self.append(&mut output, &source[start..cursor.at])?;
                continue;
            }
            if !name.eq_ignore_ascii_case("import") || block {
                self.append(&mut output, &source[start..])?;
                break;
            }
            let Some(import) = parse_import(&source[prelude..end]) else {
                continue;
            };
            if import.unsupported {
                diagnostic(
                    diagnostics,
                    "stylesheet import layers/supports conditions are unsupported",
                );
                continue;
            }
            self.imports += 1;
            if self.imports > MAX_IMPORTS || path.len() >= MAX_DEPTH {
                diagnostic(diagnostics, "stylesheet import count/depth limit reached");
                continue;
            }
            if let Err(error) =
                self.charge_url_work(sheet.url.as_str().len().saturating_add(import.url.len()))
            {
                diagnostic(diagnostics, &error);
                continue;
            }
            let mut url = match sheet.url.join(&import.url) {
                Ok(url) => url,
                Err(_) => continue,
            };
            url.set_fragment(None);
            if let Err(error) = self.charge_url_work(
                url.as_str()
                    .len()
                    .saturating_mul(path.len().saturating_add(1)),
            ) {
                diagnostic(diagnostics, &error);
                continue;
            }
            if path.contains(&url) {
                continue;
            }
            let imported = match self.fetch(fetcher, &url, sheet.encoding) {
                Ok(imported) => imported,
                Err(error) => {
                    diagnostic(diagnostics, &format!("stylesheet import: {error}"));
                    continue;
                }
            };
            if let Err(error) = self.charge_url_work(
                imported
                    .url
                    .as_str()
                    .len()
                    .saturating_mul(path.len().saturating_add(1)),
            ) {
                diagnostic(diagnostics, &error);
                continue;
            }
            if path.contains(&imported.url) {
                continue;
            }
            path.push(imported.url.clone());
            let expanded = self.expand(fetcher, &imported, path, diagnostics);
            path.pop();
            match expanded {
                Ok(imported) => {
                    for part in imported {
                        if sources.len() >= MAX_IMPORTS {
                            return Err("stylesheet segment count exceeded".into());
                        }
                        if import.media.is_empty() {
                            sources.push(part);
                        } else {
                            if !valid_media_source(&part) {
                                diagnostic(
                                    diagnostics,
                                    "malformed stylesheet cannot be wrapped in a media condition",
                                );
                                continue;
                            }
                            let mut conditional = String::new();
                            self.append(&mut conditional, "@media ")?;
                            self.append(&mut conditional, &import.media)?;
                            self.append(&mut conditional, " {\n")?;
                            self.append(&mut conditional, &part)?;
                            self.append(&mut conditional, "\n}\n")?;
                            sources.push(conditional.into());
                        }
                    }
                }
                Err(error) => diagnostic(diagnostics, &error),
            }
        }
        // Parse boundaries prevent malformed imported EOF constructs from
        // consuming rules belonging to the importing sheet or its siblings.
        if !output.is_empty() {
            if sources.len() >= MAX_IMPORTS {
                return Err("stylesheet segment count exceeded".into());
            }
            sources.push(output.into());
        }
        Ok(sources)
    }
}

fn diagnostic(diagnostics: &mut Vec<String>, message: &str) {
    if diagnostics.len() < 256 {
        diagnostics.push(message.to_owned());
    }
}

/// Structural guard for text embedded in a generated @media prelude. This is
/// not the full Media Queries grammar; matching supported features remains the
/// CSS evaluator's job. Delimiters inside strings/escapes stay data, while bare
/// rule delimiters, incomplete tokens and excessive nesting fail closed.
pub(crate) fn valid_media_condition(source: &str) -> bool {
    if source.len() > 65_536 {
        return false;
    }
    let mut cursor = Cursor::new(source);
    let mut brackets = Vec::new();
    while let Some(ch) = cursor.peek() {
        if cursor.rest().starts_with("/*") {
            let Some(end) = cursor.rest()[2..].find("*/") else {
                return false;
            };
            cursor.at += end + 4;
            continue;
        }
        if matches!(ch, '\'' | '"') {
            if cursor.string().is_none() {
                return false;
            }
            continue;
        }
        cursor.next();
        match ch {
            '\\' if cursor.escape().is_none() => return false,
            '{' | '}' | ';' => return false,
            '(' | '[' => {
                if brackets.len() >= 128 {
                    return false;
                }
                brackets.push(ch);
            }
            ')' | ']' if brackets.pop() != Some(if ch == ')' { '(' } else { '[' }) => return false,
            _ => {}
        }
    }
    brackets.is_empty()
}

/// A source segment must not be able to close its enclosing generated media
/// block. Follow the CSS rule parser's lexical string/comment/escape boundaries;
/// incomplete EOF constructs remain isolated inside their own segment. Dropping
/// a sheet with a stray top-level closer is conservative CSS error recovery.
pub(crate) fn valid_media_source(source: &str) -> bool {
    if source.len() > MAX_BYTES {
        return false;
    }
    let mut chars = source.chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0usize;
    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '\'' | '"') {
            quote = Some(ch);
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = ' ';
            for ch in chars.by_ref() {
                if previous == '*' && ch == '/' {
                    break;
                }
                previous = ch;
            }
            continue;
        }
        match ch {
            '{' => depth += 1,
            '}' if depth == 0 => return false,
            '}' => depth -= 1,
            _ => {}
        }
    }
    true
}

struct Import {
    url: String,
    media: String,
    unsupported: bool,
}
fn parse_import(source: &str) -> Option<Import> {
    let mut cursor = Cursor::new(source);
    cursor.space();
    let url = if matches!(cursor.peek(), Some('\'' | '"')) {
        cursor.string()?
    } else {
        if !cursor.ident().eq_ignore_ascii_case("url") || !cursor.eat('(') {
            return None;
        }
        cursor.url()?
    };
    cursor.space();
    let media = cursor.rest().trim().to_owned();
    if !valid_media_condition(&media) {
        return None;
    }
    let first = cursor.ident();
    let unsupported = first.eq_ignore_ascii_case("layer") || first.eq_ignore_ascii_case("supports");
    Some(Import {
        url,
        media,
        unsupported,
    })
}

struct Cursor<'a> {
    source: &'a str,
    at: usize,
}
fn css_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n' | '\x0c')
}
impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, at: 0 }
    }
    fn rest(&self) -> &'a str {
        &self.source[self.at..]
    }
    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.at += ch.len_utf8();
        Some(ch)
    }
    fn eat(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.next();
            true
        } else {
            false
        }
    }
    fn space(&mut self) {
        loop {
            while self.peek().is_some_and(css_space) {
                self.next();
            }
            if self.rest().starts_with("/*") {
                self.at += self.rest().find("*/").map_or(self.rest().len(), |n| n + 2);
            } else {
                break;
            }
        }
    }
    fn escape(&mut self) -> Option<char> {
        let ch = self.next()?;
        if matches!(ch, '\n' | '\r' | '\x0c') {
            return None;
        }
        if ch.is_ascii_hexdigit() {
            let mut code = ch.to_digit(16)?;
            for _ in 1..6 {
                let Some(value) = self.peek().and_then(|ch| ch.to_digit(16)) else {
                    break;
                };
                self.next();
                code = code * 16 + value;
            }
            if self.peek().is_some_and(css_space) {
                let cr = self.next() == Some('\r');
                if cr {
                    self.eat('\n');
                }
            }
            Some(
                char::from_u32(code)
                    .filter(|ch| *ch != '\0')
                    .unwrap_or('\u{fffd}'),
            )
        } else {
            Some(if ch == '\0' { '\u{fffd}' } else { ch })
        }
    }
    fn ident(&mut self) -> String {
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.next();
                let Some(escaped) = self.escape() else {
                    break;
                };
                value.push(escaped);
            } else if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || !ch.is_ascii() {
                self.next();
                value.push(ch);
            } else {
                break;
            }
        }
        value
    }
    fn string(&mut self) -> Option<String> {
        let quote = self.next()?;
        let mut value = String::new();
        while let Some(ch) = self.next() {
            if ch == quote {
                return Some(value);
            }
            if matches!(ch, '\n' | '\r' | '\x0c') {
                return None;
            }
            if ch == '\\' {
                if matches!(self.peek(), Some('\n' | '\r' | '\x0c')) {
                    let cr = self.next() == Some('\r');
                    if cr {
                        self.eat('\n');
                    }
                } else {
                    value.push(self.escape()?);
                }
            } else {
                value.push(if ch == '\0' { '\u{fffd}' } else { ch });
            }
        }
        None
    }
    // The opening parenthesis of a url() token has already been consumed.
    // Unquoted URL tokens may contain braces and semicolons; those characters
    // are URL data rather than enclosing at-rule delimiters.
    fn url(&mut self) -> Option<String> {
        self.space();
        let url = if matches!(self.peek(), Some('\'' | '"')) {
            self.string()?
        } else {
            let mut url = String::new();
            while let Some(ch) = self.peek() {
                if ch == ')' || css_space(ch) {
                    break;
                }
                self.next();
                if ch == '\\' {
                    url.push(self.escape()?);
                } else if matches!(ch, '\'' | '"' | '(') || ch.is_control() {
                    return None;
                } else {
                    url.push(ch);
                }
            }
            url
        };
        self.space();
        self.eat(')').then_some(url)
    }

    // Returns the prelude's end and whether a block follows. A balanced prelude
    // prevents import conditions from breaking out of their generated wrapper.
    fn statement_end(&mut self) -> Option<(usize, bool)> {
        let mut brackets = Vec::new();
        let mut invalid_closer = false;
        while let Some(ch) = self.peek() {
            if self.rest().starts_with("/*") {
                self.space();
                continue;
            }
            if matches!(ch, '\'' | '"') {
                self.string()?;
                continue;
            }
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '\\') || !ch.is_ascii() {
                let name = self.ident();
                if name.eq_ignore_ascii_case("url") && self.eat('(') {
                    self.url()?;
                }
                continue;
            }
            let at = self.at;
            self.next();
            match ch {
                '\\' => {
                    self.escape()?;
                }
                '(' | '[' => {
                    if brackets.len() >= 128 {
                        return None;
                    }
                    brackets.push(ch);
                }
                ')' | ']' => {
                    if brackets.last() == Some(&if ch == ')' { '(' } else { '[' }) {
                        brackets.pop();
                    } else {
                        invalid_closer = true;
                    }
                }
                // An unmatched close is an invalid prelude token, not the end
                // of the stylesheet. Consume to this rule's semicolon so the
                // next import can be considered independently.
                ';' if brackets.is_empty() => return Some((at, false)),
                '{' if brackets.is_empty() => return Some((at, true)),
                '{' | '}' => return None,
                _ => {}
            }
        }
        (brackets.is_empty() && !invalid_closer).then_some((self.at, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_prelude_recovers_at_semicolon_and_url_token_delimiters_remain_data() {
        let base = Url::parse("https://example.test/main.css").unwrap();
        for source in [
            "@import 'bad.css' screen); @import 'a{b};c.css';",
            r"@import url(a{b};c.css);",
            r"@import u\72l(a{b};c.css);",
        ] {
            let mut loader = Loader::new(&base, encoding_rs::UTF_8);
            let child = base.join("a{b};c.css").unwrap();
            loader.cache.insert(
                (child.to_string(), "UTF-8"),
                Ok(Sheet {
                    source: "#x{color:red}".into(),
                    url: child,
                    encoding: encoding_rs::UTF_8,
                }),
            );
            let mut diagnostics = Vec::new();
            let sources = loader
                .inline(
                    &mut Fetcher::for_document(&base),
                    source,
                    &base,
                    &mut diagnostics,
                )
                .unwrap();
            assert!(diagnostics.is_empty(), "{source}: {diagnostics:?}");
            assert_eq!(
                sources.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
                ["#x{color:red}"],
                "{source}"
            );
            assert_eq!(
                loader.cache.len(),
                1,
                "invalid import must not trigger a fetch"
            );
        }
    }

    #[test]
    fn media_preludes_cannot_inject_rules_or_leave_incomplete_tokens() {
        for media in [
            "print {} p {color:red} @media screen",
            "screen; p {color:red}",
            "screen )",
            "screen [)",
            "screen /* eof",
            "screen and (x: \"eof",
            "screen and (x: 2",
        ] {
            assert!(!valid_media_condition(media), "{media}");
            assert!(
                parse_import(&format!("'a.css' {media}")).is_none(),
                "{media}"
            );
        }
        for media in [
            "",
            "screen and (min-width: 10px)",
            r#"screen and (x: "};[")"#,
            r"screen and (x: \7b )",
            "screen /*ok*/ and (color)",
        ] {
            assert!(valid_media_condition(media), "{media}");
        }
        assert!(!valid_media_condition(&"(".repeat(129)));
    }

    #[test]
    fn malformed_import_cannot_escape_false_media_condition() {
        let base = Url::parse("https://example.test/main.css").unwrap();
        let child = base.join("bad.css").unwrap();
        let mut loader = Loader::new(&base, encoding_rs::UTF_8);
        loader.cache.insert(
            (child.to_string(), "UTF-8"),
            Ok(Sheet {
                source: "} #x{color:red}".into(),
                url: child,
                encoding: encoding_rs::UTF_8,
            }),
        );
        let mut diagnostics = Vec::new();
        let sources = loader
            .inline(
                &mut Fetcher::for_document(&base),
                "@import 'bad.css' print; #x{background:blue}",
                &base,
                &mut diagnostics,
            )
            .unwrap();
        let document = crate::dom::Document::parse("<p id=x>text</p>");
        let sources = sources.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let styles = crate::css::compute_styles(&document, &sources, 400.0, 300.0);
        assert_ne!(
            styles[document.query_selector("#x").unwrap()].color,
            crate::graphics::Color::rgb(255, 0, 0)
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.contains("malformed stylesheet"))
        );
        for source in [
            r#"p{content:"}"}"#,
            r"p{content:\}}",
            "/* } */ p{}",
            "p{",
            "p{content:'eof",
            "/* eof",
        ] {
            assert!(valid_media_source(source), "{source}");
        }
        assert!(!valid_media_source("p{} } p{color:red}"));
    }

    #[test]
    fn long_base_failures_and_cached_url_clones_share_work_and_storage_limits() {
        let document = Url::parse("https://example.test/").unwrap();
        let base = Url::parse(&format!(
            "unsupported://example.test/{}/",
            "a".repeat(4 * 1024 * 1024)
        ))
        .unwrap();
        let source = (0..256)
            .map(|n| format!("@import '{n}.css';"))
            .collect::<String>();
        let mut loader = Loader::new(&document, encoding_rs::UTF_8);
        let mut diagnostics = Vec::new();
        loader
            .inline(
                &mut Fetcher::for_document(&document),
                &source,
                &base,
                &mut diagnostics,
            )
            .unwrap();
        assert!(loader.url_work <= MAX_URL_BYTES);
        assert!(loader.url_storage <= MAX_URL_BYTES);
        assert!(
            loader.cache.len() < 8,
            "long cache keys must stop before multiplying into gigabytes"
        );
        assert!(diagnostics.iter().any(|d| d.contains("URL work budget")));
        let short = Url::parse("https://example.test/cached.css").unwrap();
        let mut loader = Loader::new(&document, encoding_rs::UTF_8);
        loader.cache.insert(
            (short.to_string(), "UTF-8"),
            Ok(Sheet {
                source: "".into(),
                url: base.clone(),
                encoding: encoding_rs::UTF_8,
            }),
        );
        loader.url_work = MAX_URL_BYTES - 1024;
        assert!(
            loader
                .fetch(
                    &mut Fetcher::for_document(&document),
                    &short,
                    encoding_rs::UTF_8
                )
                .unwrap_err()
                .contains("URL work budget")
        );
        let mut loader = Loader::new(&document, encoding_rs::UTF_8);
        loader.url_storage = MAX_URL_BYTES;
        assert!(
            loader
                .fetch(
                    &mut Fetcher::for_document(&document),
                    &short,
                    encoding_rs::UTF_8
                )
                .unwrap_err()
                .contains("URL storage budget")
        );
        assert!(loader.cache.is_empty());
    }

    #[test]
    fn retained_segments_are_bounded_across_separate_top_level_stylesheets() {
        let base = Url::parse("https://example.test/").unwrap();
        let mut loader = Loader::new(&base, encoding_rs::UTF_8);
        let mut fetcher = Fetcher::for_document(&base);
        for _ in 0..MAX_IMPORTS {
            assert_eq!(
                loader
                    .inline(&mut fetcher, "p{}", &base, &mut Vec::new())
                    .unwrap()
                    .len(),
                1
            );
        }
        assert!(
            loader
                .inline(&mut fetcher, "p{}", &base, &mut Vec::new())
                .unwrap_err()
                .contains("segment count")
        );
        assert!(
            loader
                .external(&mut fetcher, &base, &mut Vec::new())
                .unwrap_err()
                .contains("segment count")
        );
        assert_eq!(loader.segments, MAX_IMPORTS);
        assert!(loader.cache.is_empty());
    }

    #[test]
    fn malformed_imported_eof_cannot_swallow_parent_rules() {
        let base = Url::parse("https://example.test/main.css").unwrap();
        let child = base.join("bad.css").unwrap();
        let document = crate::dom::Document::parse("<p id=x>visible</p>");
        let x = document.query_selector("#x").unwrap();
        for bad in [
            "/* open comment",
            "p { color: 'open string",
            "@media screen {",
            "p {",
        ] {
            let mut loader = Loader::new(&base, encoding_rs::UTF_8);
            loader.cache.insert(
                (child.to_string(), "UTF-8"),
                Ok(Sheet {
                    source: bad.into(),
                    url: child.clone(),
                    encoding: encoding_rs::UTF_8,
                }),
            );
            let sources = loader
                .inline(
                    &mut Fetcher::for_document(&base),
                    "@import 'bad.css'; #x {color:green}",
                    &base,
                    &mut Vec::new(),
                )
                .unwrap();
            let sources = sources
                .iter()
                .map(|source| source.to_string())
                .collect::<Vec<_>>();
            assert_eq!(sources.len(), 2);
            let styles = crate::css::compute_styles(&document, &sources, 400.0, 300.0);
            assert_eq!(
                styles[x].color,
                crate::graphics::Color::rgb(0, 128, 0),
                "{bad}"
            );
        }
    }
    #[test]
    fn repeated_imports_keep_cascade_positions_while_cycles_and_late_imports_stop() {
        let base = Url::parse("https://example.test/main.css").unwrap();
        let mut loader = Loader::new(&base, encoding_rs::UTF_8);
        for (path, source) in [
            ("a.css", "@import 'b.css'; .a { color:red }"),
            ("b.css", "@import 'a.css'; .b { color:blue }"),
        ] {
            let url = base.join(path).unwrap();
            loader.cache.insert(
                (url.to_string(), "UTF-8"),
                Ok(Sheet {
                    source: source.into(),
                    url,
                    encoding: encoding_rs::UTF_8,
                }),
            );
        }
        let mut fetcher = Fetcher::for_document(&base);
        let mut diagnostics = Vec::new();
        let expanded = loader
            .inline(
                &mut fetcher,
                "@import 'a.css'; @import 'a.css'; .end{color:green} @import 'late.css';",
                &base,
                &mut diagnostics,
            )
            .unwrap();
        assert!(diagnostics.is_empty());
        let expanded = expanded
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(expanded.matches(".a {").count(), 2);
        assert_eq!(expanded.matches(".b {").count(), 2);
        assert!(expanded.find(".b {").unwrap() < expanded.find(".a {").unwrap());
        assert!(expanded.rfind(".a {").unwrap() < expanded.find(".end{").unwrap());
        assert_eq!(
            loader.cache.len(),
            2,
            "late import must never reach the fetch path"
        );
    }
    #[test]
    fn import_tokenization_handles_escapes_comments_and_media() {
        let import =
            parse_import(" /*a*/ uRl(\"a\\20 b.css\") /*b*/ screen and (min-width: 10px)").unwrap();
        assert_eq!(import.url, "a b.css");
        assert_eq!(import.media, "screen and (min-width: 10px)");
        assert!(!import.unsupported);
        assert_eq!(parse_import("url(a\\)b.css)").unwrap().url, "a)b.css");
        assert!(parse_import("url(a b.css)").is_none());
        assert!(parse_import("url(\"unterminated)").is_none());
        assert!(
            parse_import("'a' supports(display: grid)")
                .unwrap()
                .unsupported
        );
        assert!(parse_import("'a' layer(theme)").unwrap().unsupported);
    }
    #[test]
    fn prelude_scanner_rejects_scope_breakouts_and_bounds_nesting() {
        for source in ["'a';x", "url('a;b') screen;"] {
            let mut cursor = Cursor::new(source);
            assert!(cursor.statement_end().is_some());
        }
        for source in [
            "'a' screen )",
            "'a' screen ({x})",
            "'a' screen }",
            "'a' /* eof",
        ] {
            let mut cursor = Cursor::new(source);
            if source.ends_with("eof") {
                assert!(cursor.statement_end().is_some());
            } else {
                assert!(cursor.statement_end().is_none());
            }
        }
        assert!(Cursor::new(&"(".repeat(129)).statement_end().is_none());
    }
}
