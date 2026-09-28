//! Bounded stylesheet import loading. Fetch authority always remains the document.
use crate::{
    css::{CascadeLayer, StyleSource, supports_matches_with_budget},
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
pub(crate) type Sources = Vec<StyleSource>;

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
    supports_work: usize,
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
            supports_work: MAX_BYTES,
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
        let sources = self.expand(
            fetcher,
            &sheet,
            &mut path,
            &StyleSource::new(""),
            diagnostics,
        )?;
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
        let sources = self.expand(
            fetcher,
            &sheet,
            &mut Vec::new(),
            &StyleSource::new(""),
            diagnostics,
        )?;
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
        scope: &StyleSource,
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
            if import.supports.as_deref().is_some_and(|condition| {
                !supports_matches_with_budget(condition, true, &mut self.supports_work)
            }) {
                // Failed capability conditions neither fetch nor establish a
                // layer, even if this URL was already loaded by another link.
                continue;
            }
            // Order statements before an import establish its position. Preserve
            // each original parser boundary while retaining shared anonymous
            // layer identity and media conditions as metadata, never CSS text.
            self.flush_source(&mut sources, &mut output, scope)?;
            let mut import_scope = scope.clone();
            if !import.media.is_empty() {
                self.charge_expansion(import.media.len())?;
                import_scope.media.push(import.media.into());
            }
            if let Some(layer) = import.layer {
                let layer = match layer {
                    ImportLayer::Anonymous => CascadeLayer::anonymous(scope.layer.clone()),
                    ImportLayer::Named(name) => {
                        self.charge_expansion(name.len())?;
                        CascadeLayer::named(scope.layer.clone(), &name)
                    }
                };
                let Some(layer) = layer else {
                    diagnostic(
                        diagnostics,
                        "stylesheet import layer name/depth limit reached",
                    );
                    continue;
                };
                import_scope.layer = Some(layer);
                // A valid layered import declares its layer even when fetching
                // fails. Its media conditions still govern that declaration.
                if sources.len() >= MAX_IMPORTS {
                    return Err("stylesheet segment count exceeded".into());
                }
                sources.push(import_scope.clone());
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
            let expanded = self.expand(fetcher, &imported, path, &import_scope, diagnostics);
            path.pop();
            match expanded {
                Ok(imported) => {
                    for part in imported {
                        if sources.len() >= MAX_IMPORTS {
                            return Err("stylesheet segment count exceeded".into());
                        }
                        sources.push(part);
                    }
                }
                Err(error) => diagnostic(diagnostics, &error),
            }
        }
        // Parse boundaries prevent malformed imported EOF constructs from
        // consuming rules belonging to the importing sheet or its siblings.
        self.flush_source(&mut sources, &mut output, scope)?;
        Ok(sources)
    }

    fn charge_expansion(&mut self, bytes: usize) -> Result<(), String> {
        if bytes > MAX_BYTES.saturating_sub(self.expanded) {
            return Err("stylesheet expanded source budget exceeded".into());
        }
        self.expanded += bytes;
        Ok(())
    }

    fn flush_source(
        &self,
        sources: &mut Sources,
        output: &mut String,
        scope: &StyleSource,
    ) -> Result<(), String> {
        if !output.is_empty() {
            if sources.len() >= MAX_IMPORTS {
                return Err("stylesheet segment count exceeded".into());
            }
            sources.push(StyleSource {
                source: std::mem::take(output).into(),
                layer: scope.layer.clone(),
                media: scope.media.clone(),
            });
        }
        Ok(())
    }
}

fn diagnostic(diagnostics: &mut Vec<String>, message: &str) {
    if diagnostics.len() < 256 {
        diagnostics.push(message.to_owned());
    }
}

/// Structural guard for media-condition metadata. This is not the full Media
/// Queries grammar; matching supported features remains the CSS evaluator's
/// job. Delimiters inside strings/escapes stay data, while bare rule delimiters,
/// incomplete tokens and excessive nesting fail closed.
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

struct Import {
    url: String,
    media: String,
    layer: Option<ImportLayer>,
    supports: Option<String>,
}
enum ImportLayer {
    Anonymous,
    Named(String),
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
    let before_conditions = cursor.at;
    let first = cursor.ident();
    let layer = if first.eq_ignore_ascii_case("layer") {
        if cursor.eat('(') {
            let start = cursor.at;
            // Escaped closing parentheses belong to the layer-name token.
            while let Some(ch) = cursor.peek() {
                if cursor.rest().starts_with("/*") {
                    cursor.at += cursor.rest().find("*/")? + 2;
                    continue;
                }
                if ch == ')' {
                    break;
                }
                cursor.next();
                if ch == '\\' {
                    cursor.escape()?;
                }
            }
            let name = cursor.source[start..cursor.at].trim().to_owned();
            if !cursor.eat(')') || CascadeLayer::named(None, &name).is_none() {
                return None;
            }
            Some(ImportLayer::Named(name))
        } else {
            Some(ImportLayer::Anonymous)
        }
    } else {
        cursor.at = before_conditions;
        None
    };
    cursor.space();
    let before_supports = cursor.at;
    let supports = if cursor.ident().eq_ignore_ascii_case("supports") {
        // Whitespace before '(' would be an identifier, not a function token.
        Some(cursor.supports_condition()?.to_owned())
    } else {
        cursor.at = before_supports;
        None
    };
    cursor.space();
    let media = cursor.rest().trim().to_owned();
    if !valid_media_condition(&media) {
        return None;
    }
    Some(Import {
        url,
        media,
        layer,
        supports,
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
                // A backslash followed by a newline is not a valid escape.
                // Leave both code points for the surrounding grammar; dropping
                // them can turn malformed input into import/url/layer keywords.
                if self
                    .rest()
                    .chars()
                    .nth(1)
                    .is_some_and(|next| matches!(next, '\n' | '\r' | '\x0c'))
                {
                    break;
                }
                self.next();
                // CSS Syntax consumes an EOF escape as U+FFFD, never as an
                // empty string that would preserve the preceding keyword.
                value.push(self.escape().unwrap_or('\u{fffd}'));
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

    // Bound extraction before allocating strings or scanning nested blocks.
    // The capability evaluator validates tokens and grammar independently.
    fn supports_condition(&mut self) -> Option<&'a str> {
        if !self.eat('(') {
            return None;
        }
        let start = self.at;
        let mut end = self.source.len().min(start.saturating_add(16 * 1024 + 1));
        while !self.source.is_char_boundary(end) {
            end -= 1;
        }
        let source = &self.source[start..end];
        // Reuse the shared tokenizer: comments produce no tokens, while URL
        // contents and bad-URL remnants retain their own boundaries.
        let mut work = 256 * 1024;
        let tokens = crate::selectors::tokens(source, &mut work).ok()?;
        let mut brackets = vec!['('];
        for token in tokens {
            if let Some(open) = token.open() {
                if brackets.len() >= 16 {
                    return None;
                }
                brackets.push(open);
            } else if let crate::selectors::Kind::Close(close) = token.kind {
                if brackets.pop()?
                    != match close {
                        ')' => '(',
                        ']' => '[',
                        _ => '{',
                    }
                {
                    return None;
                }
                if brackets.is_empty() {
                    self.at = start + token.end;
                    return Some(&self.source[start..start + token.start]);
                }
            } else if token.kind == crate::selectors::Kind::Bad {
                return None;
            }
        }
        None
    }

    // Returns the prelude's end and whether a block follows. Balanced tokens
    // keep URL/string delimiters distinct from the enclosing at-rule boundary.
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
                let before = self.at;
                let name = self.ident();
                if self.at == before {
                    // Invalid escape delimiters are still part of the raw
                    // prelude. Consume one code point to make progress until
                    // the rule's semicolon, where parse_import can reject it.
                    self.next();
                }
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
    #[test]
    fn supports_import_review_regressions_keep_comments_and_long_namespace_failures_closed() {
        let (mut loader, base) = cached_loader(&[("a.css", Ok("#x{color:red}"))]);
        let prefix = "x".repeat(1100);
        let source = format!(
            "@layer first; @import 'a.css' layer(rejected) supports(not selector({prefix}|rect)); @import 'a.css' layer(rejected) supports(selector(div/**/span)); @import 'a.css' layer(first) supports(/* ) }} '\" */ display:flex); @layer last{{#x{{color:green}}}}"
        );
        let sources = loader
            .inline(
                &mut Fetcher::for_document(&base),
                &source,
                &base,
                &mut Vec::new(),
            )
            .unwrap();
        assert_eq!(loader.imports, 1);
        let doc = crate::dom::Document::parse("<p id=x>text</p>");
        let style = &crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0)
            [doc.query_selector("#x").unwrap()];
        assert_eq!(style.color, crate::graphics::Color::rgb(0, 128, 0));
    }

    #[test]
    fn supports_import_comment_tokens_share_query_and_matching_semantics() {
        for (condition, expected) in [
            ("selector(div/**/.x)", true),
            ("selector(./**/x)", true),
            ("selector(:/**/IS(.x))", true),
            ("selector([x~/**/=a])", true),
            ("selector([x~ /**/=a])", false),
            ("selector(#/**/x)", false),
            ("selector(:is/**/(.x))", false),
            ("not/**/selector(div/**/span)", true),
            (r#"selector(div/* ) } '" */span) or (display:flex)"#, true),
            ("not selector(svg/**/|rect)", false),
            ("future(url(foo/**/bar)) or selector(div)", true),
            (r#"selector([data-v="/* ) ] */"] )"#, true),
        ] {
            let (mut loader, base) = cached_loader(&[("good.css", Ok("div/**/.x{color:green}"))]);
            let source = format!(
                "@import 'good.css' layer(theme) supports({condition}); @layer theme{{.x{{color:red}}}}"
            );
            let sources = loader
                .inline(
                    &mut Fetcher::for_document(&base),
                    &source,
                    &base,
                    &mut Vec::new(),
                )
                .unwrap();
            assert_eq!(loader.imports, usize::from(expected), "{condition}");
            assert_eq!(
                sources.iter().any(|s| s.layer.is_some()),
                expected,
                "{condition}"
            );
            let doc = crate::dom::Document::parse("<div class=x id=x></div>");
            let styles = crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0);
            assert_eq!(
                styles[doc.query_selector("#x").unwrap()].color,
                if expected {
                    crate::graphics::Color::rgb(0, 128, 0)
                } else {
                    crate::graphics::Color::rgb(255, 0, 0)
                },
                "{condition}"
            );
        }
    }

    #[test]
    fn supports_imports_skip_fetch_and_layer_registration_when_false_or_invalid() {
        for condition in [
            "display:subgrid",
            "padding:auto",
            "not not (display:flex)",
            "(display:flex) or (color:red) and (width:1px)",
            "selector(:has(p))",
            "future(url(foo bar))",
            "not future(url(foo bar))",
            "position:sticky",
            "selector(div/**/span)",
        ] {
            let (mut loader, base) = cached_loader(&[("missing.css", Err("must not fetch"))]);
            let mut diagnostics = Vec::new();
            let source = format!(
                "@import 'missing.css' layer(theme) supports({condition}); @layer base{{#x{{color:green}}}} @layer theme{{#x{{color:red}}}}"
            );
            let sources = loader
                .inline(
                    &mut Fetcher::for_document(&base),
                    &source,
                    &base,
                    &mut diagnostics,
                )
                .unwrap();
            assert_eq!(loader.imports, 0, "{condition}");
            assert!(diagnostics.is_empty(), "{condition}");
            assert!(sources.iter().all(|s| s.layer.is_none()), "{condition}");
            let doc = crate::dom::Document::parse("<p id=x>text</p>");
            let styles = crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0);
            assert_eq!(
                styles[doc.query_selector("#x").unwrap()].color,
                crate::graphics::Color::rgb(255, 0, 0),
                "{condition}"
            );
        }
    }
    #[test]
    fn supports_imports_preserve_layer_order_media_and_shared_anonymous_identity() {
        let (mut loader, base) = cached_loader(&[
            (
                "outer.css",
                Ok(
                    "@import 'inner.css' supports((display:grid) and selector(p)); @layer child{#x{color:blue!important}}",
                ),
            ),
            ("inner.css", Ok("@layer child{#x{color:red!important}}")),
        ]);
        let sources = loader
            .inline(
                &mut Fetcher::for_document(&base),
                "@import 'outer.css' layer supports(display:flex) (width >= 400px);",
                &base,
                &mut Vec::new(),
            )
            .unwrap();
        assert_eq!(loader.imports, 2);
        let layers: Vec<_> = sources.iter().filter_map(|s| s.layer.as_ref()).collect();
        assert!(layers.len() >= 3);
        assert!(layers.iter().all(|layer| Arc::ptr_eq(layer, layers[0])));
        let doc = crate::dom::Document::parse("<p id=x>text</p>");
        let x = doc.query_selector("#x").unwrap();
        assert_eq!(
            crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0)[x].color,
            crate::graphics::Color::rgb(0, 0, 255)
        );
        assert_eq!(
            crate::css::compute_styles_from_sources(&doc, &sources, 399.0, 300.0)[x].color,
            crate::graphics::Color::BLACK
        );
        let (mut loader, base) = cached_loader(&[("missing.css", Err("missing"))]);
        let mut diagnostics = Vec::new();
        let sources=loader.inline(&mut Fetcher::for_document(&base),
            "@import 'missing.css' layer(theme) supports(not (display:subgrid)); @layer base{#x{color:green}} @layer theme{#x{color:red}}",&base,&mut diagnostics).unwrap();
        assert_eq!(loader.imports, 1);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0)[x].color,
            crate::graphics::Color::rgb(0, 128, 0)
        );
    }
    #[test]
    fn supports_import_syntax_handles_comments_escapes_and_limits() {
        let import = parse_import(
            r#"'a' layer(theme) s\75 pports( (display:flex)/**/and selector([x="("]) ) screen"#,
        )
        .unwrap();
        assert_eq!(import.media, "screen");
        assert!(import.supports.unwrap().contains("selector"));
        for source in [
            "'a' supports (display:flex)",
            "'a' supports(display:flex",
            "'a' supports(display:flex))",
            "'a' supports\\\n(display:flex)",
            "'a' supports\\",
            "'a' supports((display:flex])",
        ] {
            assert!(parse_import(source).is_none(), "{source:?}");
        }
        assert!(parse_import(&format!("'a' supports({})", "x".repeat(16 * 1024 + 1))).is_none());
        assert!(
            parse_import(&format!(
                "'a' supports({}(display:flex){})",
                "(".repeat(17),
                ")".repeat(17)
            ))
            .is_none()
        );
        let (mut loader, base) = cached_loader(&[("a.css", Ok("p{color:red}"))]);
        loader.supports_work = 1;
        let sources = loader
            .inline(
                &mut Fetcher::for_document(&base),
                "@import 'a.css' layer(foo) supports(not (unknown:value));",
                &base,
                &mut Vec::new(),
            )
            .unwrap();
        assert_eq!(loader.supports_work, 0);
        assert_eq!(loader.imports, 0);
        assert!(sources.is_empty());
    }

    use super::*;
    fn cached_loader(sheets: &[(&str, Result<&str, &str>)]) -> (Loader, Url) {
        let base = Url::parse("https://example.test/main.css").unwrap();
        let mut loader = Loader::new(&base, encoding_rs::UTF_8);
        for (name, source) in sheets {
            let url = base.join(name).unwrap();
            loader.cache.insert(
                (url.to_string(), "UTF-8"),
                match source {
                    Ok(source) => Ok(Sheet {
                        source: (*source).into(),
                        url,
                        encoding: encoding_rs::UTF_8,
                    }),
                    Err(error) => Err((*error).into()),
                },
            );
        }
        (loader, base)
    }

    #[test]
    fn invalid_keyword_escapes_never_fetch_or_register_an_import_layer() {
        for source in [
            "@import 'a.css' layer\\\n;",
            "@import 'a.css' layer\\\r;",
            "@import 'a.css' layer\\\x0c;",
            "@import 'a.css' layer\\",
            "@import\\\n 'a.css' layer;",
            "@import\\",
            "@import url\\\n('a.css') layer;",
            "@import url\\",
            "@import 'a.css' supports\\\n(display:block);",
            "@import 'a.css' supports\\",
        ] {
            let (mut loader, base) = cached_loader(&[("a.css", Ok("#x{color:red}"))]);
            let output = loader
                .inline(
                    &mut Fetcher::for_document(&base),
                    source,
                    &base,
                    &mut Vec::new(),
                )
                .unwrap();
            assert_eq!(loader.imports, 0, "{source:?}");
            assert!(output.iter().all(|part| part.layer.is_none()), "{source:?}");
            let doc = crate::dom::Document::parse("<p id=x>text</p>");
            let styles = crate::css::compute_styles_from_sources(&doc, &output, 400.0, 300.0);
            assert_ne!(
                styles[doc.query_selector("#x").unwrap()].color,
                crate::graphics::Color::rgb(255, 0, 0),
                "{source:?}"
            );
        }
    }
    #[test]
    fn keyword_escape_recovery_preserves_later_imports_and_valid_escapes() {
        for invalid in [
            "@import 'a.css' layer\\\n;",
            "@import\\\n 'a.css';",
            "@import url\\\n('a.css');",
        ] {
            let (mut loader, base) = cached_loader(&[("a.css", Ok("#x{color:red}"))]);
            let source = format!("{invalid} @\\69 mport u\\72 l('a.css') l\\61 yer(default);");
            let output = loader
                .inline(
                    &mut Fetcher::for_document(&base),
                    &source,
                    &base,
                    &mut Vec::new(),
                )
                .unwrap();
            assert_eq!(loader.imports, 1, "{source:?}");
            let doc = crate::dom::Document::parse("<p id=x>text</p>");
            let styles = crate::css::compute_styles_from_sources(&doc, &output, 400.0, 300.0);
            assert_eq!(
                styles[doc.query_selector("#x").unwrap()].color,
                crate::graphics::Color::rgb(255, 0, 0),
                "{source:?}"
            );
        }
        let mut cursor = Cursor::new("layer\\");
        assert_eq!(cursor.ident(), "layer\u{fffd}");
        assert!(cursor.rest().is_empty());
        let mut cursor = Cursor::new("layer\\\n");
        assert_eq!(cursor.ident(), "layer");
        assert_eq!(cursor.rest(), "\\\n");
        assert_eq!(Cursor::new("\\\n;").statement_end(), Some((2, false)));
    }
    #[test]
    fn imported_layers_obey_prior_statements_and_reverse_important_order() {
        let (mut loader, base) = cached_loader(&[
            ("theme.css", Ok("#x{color:blue;background:blue!important}")),
            ("base.css", Ok("#x{color:red;background:red!important}")),
        ]);
        let sources = loader.inline(&mut Fetcher::for_document(&base),
            "@layer foundation, theme; @import 'theme.css' layer(theme); @import 'base.css' layer(foundation);",
            &base, &mut Vec::new()).unwrap();
        let doc = crate::dom::Document::parse("<p id=x>sample</p>");
        let styles = crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0);
        let style = &styles[doc.query_selector("#x").unwrap()];
        assert_eq!(style.color, crate::graphics::Color::rgb(0, 0, 255));
        assert_eq!(
            style.background_color,
            crate::graphics::Color::rgb(255, 0, 0)
        );
    }

    #[test]
    fn anonymous_import_is_one_shared_layer_across_separate_parser_inputs() {
        let (mut loader, base) = cached_loader(&[
            (
                "outer.css",
                Ok("@import 'inner.css'; @layer theme {#x{color:blue!important}}"),
            ),
            ("inner.css", Ok("@layer theme {#x{color:red!important}}")),
        ]);
        let sources = loader
            .inline(
                &mut Fetcher::for_document(&base),
                "@import 'outer.css' layer;",
                &base,
                &mut Vec::new(),
            )
            .unwrap();
        let layers = sources
            .iter()
            .filter_map(|s| s.layer.as_ref())
            .collect::<Vec<_>>();
        assert!(layers.len() >= 3);
        assert!(layers.iter().all(|layer| Arc::ptr_eq(layer, layers[0])));
        let doc = crate::dom::Document::parse("<p id=x>sample</p>");
        let styles = crate::css::compute_styles_from_sources(&doc, &sources, 400.0, 300.0);
        assert_eq!(
            styles[doc.query_selector("#x").unwrap()].color,
            crate::graphics::Color::rgb(0, 0, 255)
        );
    }

    #[test]
    fn failed_layer_import_registers_its_position_only_under_matching_media() {
        let (mut loader, base) = cached_loader(&[("missing.css", Err("missing"))]);
        let mut diagnostics = Vec::new();
        let sources = loader.inline(&mut Fetcher::for_document(&base),
            "@import 'missing.css' layer(theme) (min-width:400px); @layer base{#x{color:green}} @layer theme{#x{color:red}}",
            &base, &mut diagnostics).unwrap();
        assert_eq!(diagnostics.len(), 1);
        let doc = crate::dom::Document::parse("<p id=x>sample</p>");
        let x = doc.query_selector("#x").unwrap();
        let narrow = crate::css::compute_styles_from_sources(&doc, &sources, 300.0, 200.0);
        let wide = crate::css::compute_styles_from_sources(&doc, &sources, 500.0, 200.0);
        assert_eq!(narrow[x].color, crate::graphics::Color::rgb(255, 0, 0));
        assert_eq!(wide[x].color, crate::graphics::Color::rgb(0, 128, 0));
    }

    #[test]
    fn nested_imports_retain_named_parent_and_independent_media_conditions() {
        let (mut loader, base) = cached_loader(&[
            (
                "outer.css",
                Ok(
                    "@import 'inner.css' layer(child) (max-width:500px); @layer child{#x{color:blue!important}}",
                ),
            ),
            ("inner.css", Ok("#x{color:red!important;background:green}")),
        ]);
        let sources = loader.inline(&mut Fetcher::for_document(&base),
            "@import 'outer.css' layer(parent) (min-width:300px); @layer parent.child{#x{color:green!important}}",
            &base, &mut Vec::new()).unwrap();
        let doc = crate::dom::Document::parse("<p id=x>sample</p>");
        let x = doc.query_selector("#x").unwrap();
        for width in [200.0, 400.0, 600.0] {
            let styles = crate::css::compute_styles_from_sources(&doc, &sources, width, 200.0);
            assert_eq!(styles[x].color, crate::graphics::Color::rgb(0, 128, 0));
            assert_eq!(
                styles[x].background_color,
                if width == 400.0 {
                    crate::graphics::Color::rgb(0, 128, 0)
                } else {
                    crate::graphics::Color::TRANSPARENT
                }
            );
        }
    }
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
                sources
                    .iter()
                    .map(|s| s.source.as_ref())
                    .collect::<Vec<_>>(),
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
        let styles = crate::css::compute_styles_from_sources(&document, &sources, 400.0, 300.0);
        assert_ne!(
            styles[document.query_selector("#x").unwrap()].color,
            crate::graphics::Color::rgb(255, 0, 0)
        );
        assert!(diagnostics.is_empty());
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
            assert_eq!(sources.len(), 2);
            let styles = crate::css::compute_styles_from_sources(&document, &sources, 400.0, 300.0);
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
            .map(|s| s.source.as_ref())
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
        assert!(import.supports.is_none());
        assert_eq!(parse_import("url(a\\)b.css)").unwrap().url, "a)b.css");
        assert!(parse_import("url(a b.css)").is_none());
        assert!(parse_import("url(\"unterminated)").is_none());
        assert_eq!(
            parse_import("'a' supports(display: grid)")
                .unwrap()
                .supports
                .as_deref(),
            Some("display: grid")
        );
        assert!(parse_import("'a' layer(theme)").unwrap().supports.is_none());
        assert!(matches!(
            parse_import("'a' layer").unwrap().layer,
            Some(ImportLayer::Anonymous)
        ));
        assert!(parse_import("'a' layer()").is_none());
        assert!(parse_import("'a' layer(initial)").is_none());
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
