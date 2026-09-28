//! Bounded inline declaration storage. DOM bindings perform conversions first,
//! then stage this object, serialize successfully, and commit one attribute write.
use crate::css::{self, Length};
use crate::selectors::{self, Kind, Token};

pub(crate) const MAX_INLINE_BYTES: usize = 64 * 1024;
pub(crate) const MAX_DECLARATIONS: usize = 256;
pub(crate) const MAX_VALUE_BYTES: usize = 4096;
const MAX_NAME_BYTES: usize = 256;
const MAX_DEPTH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Error {
    Limit,
}
fn spend(work: &mut usize, cost: usize) -> Result<(), Error> {
    selectors::spend(work, cost).map_err(|_| Error::Limit)
}
/// Conservative cumulative allocation allowance for one parse + one operation
/// + serialization, including borrowed token vectors and shorthand expansion.
pub(crate) fn scratch_bytes(input_bytes: usize) -> usize {
    input_bytes.saturating_mul(512).saturating_add(32 * 1024)
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    name: String,
    value: String,
    important: bool,
    // Only pending substitutions keep shorthand identity. Other shorthands are
    // expanded, so removing a winner cannot resurrect an older declaration.
    pending: Option<Vec<String>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct InlineStyle {
    records: Vec<Record>,
}
fn alias(name: &str) -> &str {
    match name {
        "inline-size" => "width",
        "block-size" => "height",
        "grid-row-gap" => "row-gap",
        "grid-column-gap" => "column-gap",
        "margin-inline-start" => "margin-left",
        "margin-inline-end" => "margin-right",
        "margin-block-start" => "margin-top",
        "margin-block-end" => "margin-bottom",
        "padding-inline-start" => "padding-left",
        "padding-inline-end" => "padding-right",
        "padding-block-start" => "padding-top",
        "padding-block-end" => "padding-bottom",
        "text-decoration-line" => "text-decoration",
        _ => name,
    }
}
fn name(name: &str) -> Option<String> {
    if name.len() > MAX_NAME_BYTES || name.is_empty() {
        return None;
    }
    if name.starts_with("--") {
        // API names are literal, not CSS source: backslashes are not decoded.
        return (name != "--" && !name.contains('\0')).then(|| name.to_owned());
    }
    let lower = name.to_ascii_lowercase();
    let lower = alias(&lower);
    (css::supported_property(lower) || css::shorthand_properties(lower).is_some())
        .then(|| lower.to_owned())
}
pub(crate) fn recognized_property(property: &str) -> bool {
    name(property).is_some()
}
fn targets(name: &str) -> Vec<String> {
    css::shorthand_properties(name).unwrap_or_else(|| vec![name.into()])
}
fn matches_name(record: &Record, property: &str) -> bool {
    record.name == property
        || record
            .pending
            .as_ref()
            .is_some_and(|names| names.iter().any(|name| name == property))
}
impl InlineStyle {
    pub(crate) fn parse(source: &str, work: &mut usize) -> Result<Self, Error> {
        if source.len() > MAX_INLINE_BYTES {
            return Err(Error::Limit);
        }
        let tokens = selectors::tokens(source, work).map_err(|_| Error::Limit)?;
        spend(work, tokens.len().saturating_mul(2) + 1)?;
        let mut style = Self::default();
        let mut depth = 0usize;
        let mut start = 0;
        for (i, token) in tokens.iter().enumerate() {
            if token.open().is_some() {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err(Error::Limit);
                }
            } else if matches!(token.kind, Kind::Close(_)) {
                depth = depth.saturating_sub(1);
            } else if token.kind == Kind::Delim(';') && depth == 0 {
                style.parse_part(source, &tokens[start..i], work)?;
                start = i + 1;
            }
        }
        style.parse_part(source, &tokens[start..], work)?;
        style.check_size()?;
        Ok(style)
    }
    fn parse_part(
        &mut self,
        source: &str,
        tokens: &[Token<'_>],
        work: &mut usize,
    ) -> Result<(), Error> {
        let significant: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind != Kind::Whitespace)
            .collect();
        let [property, colon, rest @ ..] = significant.as_slice() else {
            return Ok(());
        };
        if property.kind != Kind::Ident || colon.kind != Kind::Delim(':') {
            return Ok(());
        }
        let Some((decoded, consumed)) = css::css_identifier(property.raw) else {
            return Ok(());
        };
        if consumed != property.raw.len() {
            return Ok(());
        }
        let Some(property) = name(&decoded) else {
            return Ok(());
        };
        let important = rest.len() >= 2
            && rest[rest.len() - 2].kind == Kind::Delim('!')
            && rest[rest.len() - 1].kind == Kind::Ident
            && rest[rest.len() - 1].is_name("important");
        let end = if important {
            rest[rest.len() - 2].start
        } else {
            tokens.last().map_or(colon.end, |t| t.end)
        };
        let value = &source[colon.end..end];
        if value.len() > MAX_VALUE_BYTES {
            return Err(Error::Limit);
        }
        if let Some(records) = records(&property, value, important, work)? {
            for record in records {
                spend(
                    work,
                    self.records.iter().map(|r| r.name.len() + 1).sum::<usize>()
                        + record.name.len()
                        + 1,
                )?;
                if let Some(index) = self.records.iter().position(|old| old.name == record.name) {
                    if self.records[index].important && !record.important {
                        continue;
                    }
                    // A pending shorthand between duplicate longhands can affect
                    // their precedence. Keep the last winning source position.
                    self.records.remove(index);
                }
                self.records.push(record);
                self.check_size()?;
            }
        }
        Ok(())
    }
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    pub(crate) fn item(&self, index: usize) -> &str {
        self.records.get(index).map_or("", |r| r.name.as_str())
    }
    fn query_work(&self, property: &str, work: &mut usize) -> Result<(), Error> {
        spend(work, property.len().saturating_add(1))?;
        let names = name(property).map(|p| targets(&p)).unwrap_or_default();
        let records = self
            .records
            .iter()
            .map(|r| {
                r.name.len()
                    + 1
                    + r.pending
                        .as_ref()
                        .map_or(0, |p| p.iter().map(|s| s.len() + 1).sum())
            })
            .sum::<usize>();
        spend(
            work,
            (records + property.len() + 1).saturating_mul(names.len().max(1)),
        )
    }

    fn winner(&self, property: &str) -> Option<(usize, &Record)> {
        let mut result: Option<(usize, &Record)> = None;
        for (index, record) in self.records.iter().enumerate() {
            if matches_name(record, property)
                && result.is_none_or(|(_, old)| record.important || !old.important)
            {
                result = Some((index, record));
            }
        }
        result
    }
    pub(crate) fn get_property_value(
        &self,
        property: &str,
        work: &mut usize,
    ) -> Result<String, Error> {
        self.query_work(property, work)?;
        let Some(property) = name(property) else {
            return Ok(String::new());
        };
        let Some(longhands) = css::shorthand_properties(&property) else {
            let value = self
                .winner(&property)
                .filter(|(_, r)| r.pending.is_none())
                .map_or("", |(_, r)| r.value.as_str());
            spend(work, value.len() + 1)?;
            return Ok(value.into());
        };
        let values: Option<Vec<_>> = longhands.iter().map(|p| self.winner(p)).collect();
        let Some(values) = values else {
            return Ok(String::new());
        };
        if values
            .iter()
            .any(|(_, r)| r.important != values[0].1.important)
        {
            return Ok(String::new());
        }
        if values.iter().any(|(_, r)| r.pending.is_some()) {
            if values
                .iter()
                .all(|(i, r)| *i == values[0].0 && r.name == property)
            {
                spend(work, values[0].1.value.len() + 1)?;
                return Ok(values[0].1.value.clone());
            }
            return Ok(String::new());
        }
        let raw: Vec<_> = values.iter().map(|(_, r)| r.value.as_str()).collect();
        spend(
            work,
            raw.iter()
                .map(|v| v.len() + 1)
                .sum::<usize>()
                .saturating_mul(4)
                + 1,
        )?;
        Ok(shorthand_value(&property, &raw))
    }
    pub(crate) fn get_property_priority(
        &self,
        property: &str,
        work: &mut usize,
    ) -> Result<&'static str, Error> {
        self.query_work(property, work)?;
        let Some(property) = name(property) else {
            return Ok("");
        };
        Ok(
            if targets(&property)
                .iter()
                .all(|p| self.winner(p).is_some_and(|(_, r)| r.important))
            {
                "important"
            } else {
                ""
            },
        )
    }
    pub(crate) fn set_property(
        &mut self,
        property: &str,
        value: &str,
        priority: &str,
        work: &mut usize,
    ) -> Result<bool, Error> {
        spend(
            work,
            property
                .len()
                .saturating_add(value.len())
                .saturating_add(priority.len())
                .saturating_add(1),
        )?;
        let Some(property) = name(property) else {
            return Ok(false);
        };
        if value.is_empty() {
            let before = self.records.len();
            self.remove_property(&property, work)?;
            return Ok(before != self.records.len());
        }
        if !(priority.is_empty() || priority.eq_ignore_ascii_case("important")) {
            return Ok(false);
        }
        if value.len() > MAX_VALUE_BYTES {
            return Err(Error::Limit);
        }
        let important = !priority.is_empty();
        let Some(replacements) = records(&property, value, important, work)? else {
            return Ok(false);
        };
        self.query_work(&property, work)?;
        let names = targets(&property);
        // Partial removal of a pending shorthand cannot be represented by plain
        // declaration text. A later value is safe only if it can win the priority.
        if self.records.iter().any(|r| {
            r.pending.as_ref().is_some_and(|parts| {
                parts.iter().any(|p| names.contains(p))
                    && !parts.iter().all(|p| names.contains(p))
                    && r.important
                    && !important
            })
        }) {
            return Ok(false);
        }
        spend(work, self.storage_bytes().saturating_mul(2) + 1)?;
        let mut staged = self.clone();
        if replacements.iter().any(|r| r.pending.is_some()) {
            staged.records.retain(|r| {
                if let Some(parts) = &r.pending {
                    !parts.iter().all(|p| names.contains(p))
                } else {
                    !names.contains(&r.name)
                }
            });
            staged.records.extend(replacements);
        } else {
            staged.records.retain(|r| {
                r.pending
                    .as_ref()
                    .is_none_or(|parts| !parts.iter().all(|p| names.contains(p)))
            });
            for replacement in replacements {
                let existing = staged
                    .records
                    .iter()
                    .position(|r| r.name == replacement.name);
                let last_pending = staged
                    .records
                    .iter()
                    .rposition(|r| r.pending.is_some() && matches_name(r, &replacement.name));
                if let Some(index) = existing {
                    if last_pending.is_none_or(|pending| pending < index) {
                        staged.records[index] = replacement;
                        continue;
                    }
                    staged.records.remove(index);
                }
                staged.records.push(replacement);
            }
        }
        staged.check_size()?;
        let changed = *self != staged;
        *self = staged;
        Ok(changed)
    }
    pub(crate) fn remove_property(
        &mut self,
        property: &str,
        work: &mut usize,
    ) -> Result<String, Error> {
        let previous = self.get_property_value(property, work)?;
        let Some(property) = name(property) else {
            return Ok(previous);
        };
        let names = targets(&property);
        self.query_work(&property, work)?;
        if self.records.iter().any(|r| {
            r.pending.as_ref().is_some_and(|parts| {
                parts.iter().any(|p| names.contains(p)) && !parts.iter().all(|p| names.contains(p))
            })
        }) {
            return Ok(String::new());
        }
        self.records.retain(|r| {
            if let Some(parts) = &r.pending {
                !parts.iter().all(|p| names.contains(p))
            } else {
                !names.contains(&r.name)
            }
        });
        Ok(previous)
    }
    fn storage_bytes(&self) -> usize {
        self.records
            .iter()
            .map(|r| {
                serialized_name_len(&r.name) + r.value.len() + if r.important { 15 } else { 4 }
            })
            .sum()
    }
    fn check_size(&self) -> Result<(), Error> {
        if self.records.len() > MAX_DECLARATIONS || self.storage_bytes() > MAX_INLINE_BYTES {
            Err(Error::Limit)
        } else {
            Ok(())
        }
    }
    pub(crate) fn serialize(&self, work: &mut usize) -> Result<String, Error> {
        self.check_size()?;
        spend(work, self.storage_bytes().saturating_mul(2) + 1)?;
        let mut out = String::with_capacity(self.storage_bytes());
        for record in &self.records {
            if !out.is_empty() {
                out.push(' ');
            }
            write_name(&mut out, &record.name);
            out.push_str(": ");
            out.push_str(&record.value);
            if record.important {
                out.push_str(" !important");
            }
            out.push(';');
        }
        Ok(out)
    }
}
fn serialized_name_len(name: &str) -> usize {
    name.chars()
        .map(|c| {
            if c >= '\u{80}' || c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
                c.len_utf8()
            } else {
                8
            }
        })
        .sum()
}
fn write_name(out: &mut String, name: &str) {
    use std::fmt::Write;
    for c in name.chars() {
        if c >= '\u{80}' || c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
            out.push(c);
        } else {
            let _ = write!(out, "\\{:x} ", c as u32);
        }
    }
}
fn shorthand_value(name: &str, values: &[&str]) -> String {
    if values.iter().all(|v| *v == values[0])
        && (values[0].eq_ignore_ascii_case("inherit")
            || values[0].eq_ignore_ascii_case("initial")
            || values[0].eq_ignore_ascii_case("unset")
            || values[0].eq_ignore_ascii_case("revert")
            || values[0].eq_ignore_ascii_case("revert-layer"))
    {
        return values[0].into();
    }
    match name {
        "margin" | "padding" | "inset" | "border-width" | "border-color" | "border-style" => {
            let count = if values[3] != values[1] {
                4
            } else if values[2] != values[0] {
                3
            } else if values[1] != values[0] {
                2
            } else {
                1
            };
            values[..count].join(" ")
        }
        "gap" | "grid-gap" | "margin-inline" | "margin-block" | "padding-inline"
        | "padding-block" | "place-items" | "place-self" | "place-content" => {
            if values[0] == values[1] {
                values[0].into()
            } else {
                values.join(" ")
            }
        }
        "background" => values[0].into(),
        "border"
            if values
                .as_chunks::<3>()
                .0
                .iter()
                .all(|chunk| chunk == &values[..3]) =>
        {
            values[..3].join(" ")
        }
        "border-top" | "border-right" | "border-bottom" | "border-left" | "flex" | "flex-flow"
        | "list-style" => values.join(" "),
        "grid-column" | "grid-row" | "grid-area" => values.join(" / "),
        // Full font/all shorthand reconstruction is intentionally absent.
        _ => String::new(),
    }
}

fn unpaired_escape(raw: &str) -> bool {
    raw.as_bytes()
        .iter()
        .rev()
        .take_while(|b| **b == b'\\')
        .count()
        % 2
        != 0
}
fn records(
    property: &str,
    raw: &str,
    important: bool,
    work: &mut usize,
) -> Result<Option<Vec<Record>>, Error> {
    let tokens = selectors::tokens(raw, work).map_err(|_| Error::Limit)?;
    spend(
        work,
        raw.len().saturating_mul(8) + tokens.len().saturating_mul(4) + 1,
    )?;
    // Raw EOF repair is deliberately not serialized: an unmatched escape
    // could consume the semicolon which joins the next declaration.
    if tokens.iter().any(|t| {
        unpaired_escape(t.raw)
            || t.kind == Kind::Url
                && (!t.raw.ends_with(')') || unpaired_escape(&t.raw[..t.raw.len() - 1]))
    }) {
        return Ok(None);
    }
    let mut nesting = 0usize;
    for token in &tokens {
        if token.open().is_some() {
            nesting += 1;
            if nesting > MAX_DEPTH {
                return Err(Error::Limit);
            }
        } else if matches!(token.kind, Kind::Close(_)) {
            nesting = nesting.saturating_sub(1);
        }
    }
    if !css::valid_variable_value(raw, work) {
        if *work == 0 {
            return Err(Error::Limit);
        }
        return Ok(None);
    }
    let significant: Vec<_> = tokens
        .iter()
        .filter(|t| t.kind != Kind::Whitespace)
        .collect();
    let value = significant
        .first()
        .zip(significant.last())
        .map_or("", |(first, last)| &raw[first.start..last.end]);
    let custom = property.starts_with("--");
    if custom {
        return Ok(Some(vec![Record {
            name: property.into(),
            value: value.into(),
            important,
            pending: None,
        }]));
    }
    if value.is_empty() {
        return Ok(None);
    }
    if css::contains_var_function(value) {
        return Ok(Some(vec![Record {
            name: property.into(),
            value: value.into(),
            important,
            pending: css::shorthand_properties(property),
        }]));
    }
    let mut calc = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if token.open().is_some() {
            calc.push(
                token.kind == Kind::Function && token.is_name("calc")
                    || calc.last().copied().unwrap_or(false),
            );
        } else if matches!(token.kind, Kind::Close(_)) {
            calc.pop();
        } else if calc.last() == Some(&true)
            && matches!(token.kind, Kind::Delim('+' | '-'))
            && (i == 0
                || tokens[i - 1].kind != Kind::Whitespace
                || tokens.get(i + 1).is_none_or(|t| t.kind != Kind::Whitespace))
        {
            return Ok(None);
        }
    }
    // Normalize only tokens whose grammar is already known. Strings are copied
    // verbatim, and a removed comment cannot concatenate adjacent CSS tokens.
    let mut normalized = String::new();
    let mut end = 0;
    let first = significant[0].start;
    let last = significant.last().unwrap().end;
    for token in tokens.iter().filter(|t| t.start >= first && t.end <= last) {
        if token.start > end && !normalized.is_empty() {
            normalized.push(' ');
        }
        match token.kind {
            Kind::Ident if property != "font-family" && property != "font" => {
                let Some((ident, consumed)) = css::css_identifier(token.raw) else {
                    return Ok(None);
                };
                if consumed != token.raw.len() {
                    return Ok(None);
                }
                if selectors::token(&ident).0 != Kind::Ident
                    || ident.chars().any(|c| {
                        !(c >= '\u{80}' || c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                    })
                {
                    return Ok(None);
                }
                normalized.push_str(&ident.to_ascii_lowercase());
            }
            Kind::Function => {
                let Some((ident, _)) = css::css_identifier(&token.raw[..token.raw.len() - 1])
                else {
                    return Ok(None);
                };
                if selectors::token(&ident).0 != Kind::Ident
                    || ident.chars().any(|c| {
                        !(c >= '\u{80}' || c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                    })
                {
                    return Ok(None);
                }
                normalized.push_str(&ident.to_ascii_lowercase());
                normalized.push('(');
            }
            Kind::Dimension => normalized.push_str(&token.raw.to_ascii_lowercase()),
            _ => normalized.push_str(token.raw),
        }
        end = token.end;
    }
    let value = normalized.as_str();
    if !valid_value(property, value, work)? {
        return Ok(None);
    }
    let count = css::shorthand_properties(property).map_or(1, |names| names.len());
    spend(work, count.saturating_mul(value.len() + 64) + 1)?;
    let mut expanded = Vec::new();
    css::expand_declaration(property, value, important, &mut expanded);
    if expanded.is_empty() {
        return Ok(None);
    }
    // A shorthand is accepted atomically, never as a valid prefix followed by
    // ignored garbage. Every emitted longhand must also pass its value grammar.
    for decl in &expanded {
        if !valid_longhand(&decl.name, &decl.value, work)? {
            return Ok(None);
        }
    }
    Ok(Some(
        expanded
            .into_iter()
            .map(|d| Record {
                name: d.name,
                value: d.value,
                important: d.important,
                pending: None,
            })
            .collect(),
    ))
}
fn wide(value: &str) -> bool {
    matches!(
        value,
        "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    )
}
fn number(value: &str) -> Option<f32> {
    if value.is_empty() {
        return None;
    }
    let (kind, n) = selectors::token(value);
    if kind != Kind::Number || n != value.len() {
        return None;
    }
    value.parse::<f32>().ok().filter(|n| n.is_finite())
}
// Calc's supported grammar is sums/differences of lengths, percentages and
// nested calc(). Mixed units are retained, although layout cannot yet resolve
// every mixed-category expression. Products/min/max/clamp remain unsupported.
fn length(value: &str, percent: bool, negative: bool, depth: usize, work: &mut usize) -> bool {
    if spend(work, value.len().saturating_mul(4) + 1).is_err() {
        return false;
    }
    if depth > MAX_DEPTH || value.is_empty() {
        return false;
    }
    if let Some(inner) = value
        .strip_prefix("calc(")
        .and_then(|v| v.strip_suffix(')'))
    {
        // Borrow each top-level component directly. Nested calculations do not
        // allocate another full token vector at every recursion level.
        let mut at = 0;
        let mut operand = true;
        let mut seen = false;
        while at < inner.len() {
            let (kind, count) = selectors::token(&inner[at..]);
            if kind == Kind::Whitespace {
                at += count;
                continue;
            }
            let start = at;
            at += count;
            let mut nesting = usize::from(matches!(kind, Kind::Function | Kind::Open(_)));
            while nesting > 0 && at < inner.len() {
                let (kind, count) = selectors::token(&inner[at..]);
                if matches!(kind, Kind::Function | Kind::Open(_)) {
                    nesting += 1;
                } else if matches!(kind, Kind::Close(_)) {
                    nesting -= 1;
                }
                at += count;
            }
            if nesting != 0 {
                return false;
            }
            let part = &inner[start..at];
            if operand {
                if !length(part, percent, true, depth + 1, work) {
                    return false;
                }
            } else if !matches!(part, "+" | "-") {
                return false;
            }
            // Binary +/- requires whitespace on both sides. Token adjacency
            // must not repair calc(1px+ 2px) into valid arithmetic.
            if !operand
                && (start == 0
                    || !inner[..start].ends_with(selectors::space)
                    || at == inner.len()
                    || !inner[at..].starts_with(selectors::space))
            {
                return false;
            }
            operand = !operand;
            seen = true;
        }
        return seen && !operand;
    }

    let (kind, n) = selectors::token(value);
    if n != value.len() || !matches!(kind, Kind::Dimension | Kind::Percentage | Kind::Number) {
        return false;
    }
    if kind == Kind::Percentage && !percent {
        return false;
    }
    if kind == Kind::Number {
        return number(value) == Some(0.0);
    }
    matches!(css::parse_length(value,16.0,16.0,800.0,600.0), Some(Length::Px(n)|Length::Percent(n)) if negative || n>=0.0)
}
fn components<'a>(source: &'a str, tokens: &[Token<'_>]) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    for token in tokens {
        if depth == 0 && token.kind == Kind::Whitespace {
            if let Some(begin) = start.take() {
                parts.push(&source[begin..token.start]);
            }
        } else {
            start.get_or_insert(token.start);
            if token.open().is_some() {
                depth += 1;
            } else if matches!(token.kind, Kind::Close(_)) {
                depth = depth.saturating_sub(1);
            }
        }
    }
    if let Some(begin) = start {
        parts.push(&source[begin..tokens.last().unwrap().end]);
    }
    parts
}
fn color(value: &str) -> bool {
    css::supports_property("color", value)
}
fn border_style(value: &str) -> bool {
    matches!(
        value,
        "none"
            | "hidden"
            | "solid"
            | "dashed"
            | "dotted"
            | "double"
            | "groove"
            | "ridge"
            | "inset"
            | "outset"
    )
}
fn border_width(value: &str, work: &mut usize) -> bool {
    matches!(value, "thin" | "medium" | "thick") || length(value, false, false, 0, work)
}
fn valid_value(property: &str, value: &str, work: &mut usize) -> Result<bool, Error> {
    spend(work, value.len().saturating_mul(12) + 1)?;
    if wide(value) {
        return Ok(true);
    }
    if css::shorthand_properties(property).is_none() {
        return valid_longhand(property, value, work);
    }
    let tokens = selectors::tokens(value, work).map_err(|_| Error::Limit)?;
    let parts = components(value, &tokens);
    let valid = match property {
        "margin" | "padding" | "inset" | "border-width" | "border-color" | "border-style" => {
            (1..=4).contains(&parts.len())
                && parts.iter().all(|v| match property {
                    "margin" | "inset" => *v == "auto" || length(v, true, true, 0, work),
                    "padding" => length(v, true, false, 0, work),
                    "border-width" => border_width(v, work),
                    "border-color" => color(v),
                    _ => border_style(v),
                })
        }
        "margin-inline" | "margin-block" | "padding-inline" | "padding-block" => {
            (1..=2).contains(&parts.len())
                && parts.iter().all(|v| {
                    property.starts_with("margin") && *v == "auto"
                        || length(v, true, property.starts_with("margin"), 0, work)
                })
        }
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let (mut widths, mut styles, mut colors) = (0, 0, 0);
            (1..=3).contains(&parts.len())
                && parts.iter().all(|v| {
                    if border_width(v, work) {
                        widths += 1;
                        widths == 1
                    } else if border_style(v) {
                        styles += 1;
                        styles == 1
                    } else if color(v) {
                        colors += 1;
                        colors == 1
                    } else {
                        false
                    }
                })
        }
        "background" => color(value),
        "flex" => {
            matches!(value, "auto" | "none")
                || (1..=3).contains(&parts.len())
                    && number(parts[0]).is_some_and(|n| n >= 0.0)
                    && (parts.len() == 1
                        || if let Some(n) = number(parts[1]) {
                            n >= 0.0
                                && (parts.len() == 2
                                    || parts[2] == "auto"
                                    || length(parts[2], true, false, 0, work))
                        } else {
                            parts.len() == 2
                                && (parts[1] == "auto" || length(parts[1], true, false, 0, work))
                        })
        }
        "flex-flow" => {
            let directions = parts
                .iter()
                .filter(|v| matches!(**v, "row" | "row-reverse" | "column" | "column-reverse"))
                .count();
            let wraps = parts
                .iter()
                .filter(|v| matches!(**v, "wrap" | "nowrap" | "wrap-reverse"))
                .count();
            !parts.is_empty() && directions <= 1 && wraps <= 1 && directions + wraps == parts.len()
        }
        "font" => {
            let size = parts.iter().position(|p| {
                let size = p.split('/').next().unwrap_or(p);
                length(size, true, false, 0, work)
                    || matches!(
                        size,
                        "small"
                            | "medium"
                            | "large"
                            | "x-large"
                            | "xx-large"
                            | "smaller"
                            | "larger"
                    )
            });
            if let Some(i) = size {
                i + 1 < parts.len()
                    && parts[..i].iter().all(|p| {
                        matches!(
                            *p,
                            "normal" | "italic" | "oblique" | "bold" | "bolder" | "lighter"
                        ) || number(p).is_some_and(|n| (1.0..=1000.0).contains(&n))
                    })
                    && parts[i].split('/').count() <= 2
            } else {
                false
            }
        }
        "all" => false,
        // Expansion plus validation of every longhand handles these forms.
        "gap" | "grid-gap" | "grid-column" | "grid-row" | "grid-area" | "place-items"
        | "place-self" | "place-content" | "list-style" => true,
        _ => false,
    };
    if *work == 0 {
        Err(Error::Limit)
    } else {
        Ok(valid)
    }
}
fn valid_longhand(property: &str, value: &str, work: &mut usize) -> Result<bool, Error> {
    spend(work, value.len().saturating_mul(12) + 1)?;
    if wide(value) {
        return Ok(true);
    }
    // Existing strict positive grammar covers colors, numerical flex/Grid data
    // and common dimensions. The remaining renderer grammar is validated here
    // without broadening CSS.supports capability claims.
    if css::supports_property(property, value) {
        return Ok(true);
    }
    let valid = match property {
        "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height"
        | "flex-basis" => {
            matches!(
                value,
                "auto" | "min-content" | "max-content" | "fit-content"
            ) || matches!(property, "max-width" | "max-height") && value == "none"
                || length(value, true, false, 0, work)
        }
        "top" | "right" | "bottom" | "left" | "margin-top" | "margin-right" | "margin-bottom"
        | "margin-left" => value == "auto" || length(value, true, true, 0, work),
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            length(value, true, false, 0, work)
        }
        "row-gap" | "column-gap" => value == "normal" || length(value, true, false, 0, work),
        "border-top-width" | "border-right-width" | "border-bottom-width" | "border-left-width" => {
            border_width(value, work)
        }
        "border-top-style" | "border-right-style" | "border-bottom-style" | "border-left-style" => {
            border_style(value)
        }
        "border-top-color" | "border-right-color" | "border-bottom-color" | "border-left-color" => {
            color(value)
        }
        "display" => matches!(
            value,
            "list-item"
                | "table"
                | "table-row"
                | "table-row-group"
                | "table-header-group"
                | "table-footer-group"
                | "table-cell"
                | "contents"
                | "inline-flex"
                | "inline-grid"
        ),
        "position" => value == "sticky",
        "font-size" => {
            matches!(
                value,
                "xx-small"
                    | "x-small"
                    | "small"
                    | "medium"
                    | "large"
                    | "x-large"
                    | "xx-large"
                    | "xxx-large"
                    | "smaller"
                    | "larger"
            ) || length(value, true, false, 0, work)
        }
        "font-family" => {
            let tokens = selectors::tokens(value, work).map_err(|_| Error::Limit)?;
            let mut count = 0;
            let mut quoted = false;
            let mut valid = true;
            for t in tokens.iter().filter(|t| t.kind != Kind::Whitespace) {
                match t.kind {
                    Kind::Ident if !quoted => {
                        if css::css_identifier(t.raw)
                            .is_none_or(|(n, _)| wide(&n.to_ascii_lowercase()))
                        {
                            valid = false;
                        }
                        count += 1;
                    }
                    Kind::String if count == 0 => {
                        count = 1;
                        quoted = true;
                    }
                    Kind::Comma if count > 0 => {
                        count = 0;
                        quoted = false;
                    }
                    _ => valid = false,
                }
            }
            valid && count > 0
        }
        "font-style" => matches!(value, "normal" | "italic" | "oblique"),
        "font-weight" => {
            matches!(value, "normal" | "bold" | "bolder" | "lighter")
                || number(value).is_some_and(|n| (1.0..=1000.0).contains(&n))
        }
        "line-height" => {
            value == "normal"
                || number(value).is_some_and(|n| n >= 0.0)
                || length(value, true, false, 0, work)
        }
        "text-align" => matches!(
            value,
            "left" | "right" | "center" | "justify" | "start" | "end"
        ),
        "white-space" => matches!(
            value,
            "normal" | "nowrap" | "pre" | "pre-wrap" | "pre-line" | "break-spaces"
        ),
        "text-decoration" => {
            let parts: Vec<_> = value.split_ascii_whitespace().collect();
            (1..=3).contains(&parts.len())
                && (value == "none"
                    || parts.iter().enumerate().all(|(i, v)| {
                        matches!(*v, "underline" | "overline" | "line-through")
                            && !parts[..i].contains(v)
                    }))
        }
        "align-items" | "align-self" | "justify-items" | "justify-self" => {
            matches!(
                value,
                "normal"
                    | "stretch"
                    | "start"
                    | "end"
                    | "flex-start"
                    | "flex-end"
                    | "center"
                    | "baseline"
            ) || property.ends_with("self") && value == "auto"
        }
        "justify-content" | "align-content" => matches!(
            value,
            "normal"
                | "stretch"
                | "start"
                | "end"
                | "flex-start"
                | "flex-end"
                | "center"
                | "space-between"
                | "space-around"
                | "space-evenly"
        ),
        "overflow" | "overflow-x" | "overflow-y" => {
            matches!(value, "visible" | "hidden" | "clip" | "scroll" | "auto")
        }
        "border-radius" => {
            let tokens = selectors::tokens(value, work).map_err(|_| Error::Limit)?;
            let parts = components(value, &tokens);
            (1..=4).contains(&parts.len()) && parts.iter().all(|p| length(p, true, false, 0, work))
        }
        "list-style-type" => matches!(
            value,
            "none"
                | "disc"
                | "circle"
                | "square"
                | "decimal"
                | "decimal-leading-zero"
                | "disclosure-open"
                | "disclosure-closed"
        ),
        "list-style-position" => matches!(value, "inside" | "outside"),
        "vertical-align" => {
            matches!(
                value,
                "baseline"
                    | "sub"
                    | "super"
                    | "text-top"
                    | "text-bottom"
                    | "middle"
                    | "top"
                    | "bottom"
            ) || length(value, true, true, 0, work)
        }
        _ => false,
    };
    if *work == 0 {
        Err(Error::Limit)
    } else {
        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const WORK: usize = 20_000_000;
    fn parse(source: &str) -> InlineStyle {
        InlineStyle::parse(source, &mut WORK.clone()).unwrap()
    }
    fn get(style: &InlineStyle, name: &str) -> String {
        style.get_property_value(name, &mut WORK.clone()).unwrap()
    }
    fn text(style: &InlineStyle) -> String {
        style.serialize(&mut WORK.clone()).unwrap()
    }
    fn set(style: &mut InlineStyle, name: &str, value: &str, priority: &str) -> bool {
        style
            .set_property(name, value, priority, &mut WORK.clone())
            .unwrap()
    }
    #[test]
    fn inline_tokens_preserve_nested_delimiters_and_recover_declarations() {
        let mut s = parse(
            r#"--note:"a;b:c"; --nested:fn(a;b:[c;d]); font-family:"A;B:C", serif; color:red; broken; width:10px"#,
        );
        assert_eq!(get(&s, "--note"), r#""a;b:c""#);
        assert_eq!(get(&s, "--nested"), "fn(a;b:[c;d])");
        assert_eq!(get(&s, "font-family"), r#""A;B:C", serif"#);
        assert!(set(&mut s, "color", "green", ""));
        assert_eq!(parse(&text(&s)), s);
        assert_eq!(get(&s, "width"), "10px");
        assert_eq!(
            get(&parse("color: red; width: 12px; height: fn(;bad)"), "color"),
            "red"
        );
        for source in ["--x:'unclosed", "--x:url(unclosed", "--x:var(--x", "--x:[)"] {
            assert_eq!(parse(source).len(), 0, "{source}");
        }
        let mut s = parse("color:red");
        for value in [
            "'unclosed",
            "url(unclosed",
            "var(--x",
            "[)",
            "red; width:100px",
            "red !important",
            "var(--x) !important",
        ] {
            let before = s.clone();
            assert!(!set(&mut s, "color", value, ""), "{value}");
            assert_eq!(s, before);
        }
        // EOF comments are discarded; an appended declaration stays outside them.
        assert!(set(&mut s, "--x", "green/*unclosed", ""));
        assert!(set(&mut s, "color", "blue", ""));
        assert_eq!(get(&parse(&text(&s)), "color"), "blue");
    }
    #[test]
    fn inline_raw_escapes_cannot_consume_serialization_delimiters() {
        for source in ["--x:before;color:red", "color:red;--x:before !important"] {
            let mut s = parse(source);
            for value in ["a\\", "url(a\\)", "a\\\\\\"] {
                let before = s.clone();
                assert!(!set(&mut s, "--x", value, ""), "{value}");
                assert_eq!(s, before);
                assert_eq!(get(&parse(&text(&s)), "color"), "red");
            }
            for value in ["a\\ ", "a\\31 ", "a\\\\", "url(a\\\\)"] {
                assert!(set(&mut s, "--x", value, "important"), "{value}");
                assert_eq!(get(&s, "--x"), value);
                assert_eq!(get(&parse(&text(&s)), "color"), "red");
                assert_eq!(get(&parse(&text(&s)), "--x"), value);
            }
        }
        for source in [
            "font-family:before; color:red",
            "color:red;font-family:before",
        ] {
            let mut s = parse(source);
            let before = s.clone();
            assert!(!set(&mut s, "font-family", "a\\", ""));
            assert_eq!(s, before);
            assert!(set(&mut s, "font-family", "a\\ ", ""));
            assert_eq!(get(&parse(&text(&s)), "color"), "red");
            assert_eq!(get(&parse(&text(&s)), "font-family"), "a\\ ");
        }
        let s = parse("--x:a\\ ; color: red;");
        assert_eq!(get(&s, "--x"), "a\\ ");
        assert_eq!(get(&parse(&text(&s)), "color"), "red");
    }
    #[test]
    fn inline_names_are_literal_case_sensitive_custom_and_safely_serialized() {
        let mut s = parse(r"--Tone:green; --tone:red; --a\:b:blue; --\54 one:green; c\6f lor:red");
        assert_eq!(get(&s, "--Tone"), "green");
        assert_eq!(get(&s, "--tone"), "red");
        assert_eq!(get(&s, "--a:b"), "blue");
        assert_eq!(get(&s, r"--a\:b"), "");
        assert_eq!(get(&s, "COLOR"), "red");
        assert_eq!(get(&s, " color"), "");
        assert_eq!(get(&s, r"c\6flor"), "");
        assert!(set(&mut s, "color", "green", ""));
        for name in [
            "--semi;colon",
            "--colon:name",
            "--white space",
            "--quote\"",
            "--slash\\x",
            "--brace{}",
            "--comment/*",
            "--control\u{7f}\u{1}",
        ] {
            assert!(set(&mut s, name, "red", ""));
        }
        assert_eq!(parse(&text(&s)), s);
        assert!(!recognized_property("Color "));
        assert!(recognized_property("background-color"));
        assert!(!recognized_property("backgroundColor"));
    }
    #[test]
    fn inline_priority_replacement_and_empty_removal_are_atomic() {
        let mut s =
            parse("color:red; color:green !/**/ IMPORTANT; color:blue; --empty:/**/ !important");
        assert_eq!(get(&s, "color"), "green");
        assert_eq!(
            s.get_property_priority("COLOR", &mut WORK.clone()).unwrap(),
            "important"
        );
        assert_eq!(get(&s, "--empty"), "");
        assert_eq!(
            s.get_property_priority("--empty", &mut WORK.clone())
                .unwrap(),
            "important"
        );
        for priority in ["!important", " important", "important ", "invalid"] {
            let before = s.clone();
            assert!(!set(&mut s, "color", "red", priority));
            assert_eq!(s, before);
        }
        assert!(set(&mut s, "color", "blue", ""));
        assert_eq!(
            s.get_property_priority("color", &mut WORK.clone()).unwrap(),
            ""
        );
        assert_eq!(
            s.remove_property("color", &mut WORK.clone()).unwrap(),
            "blue"
        );
        assert_eq!(get(&s, "color"), "");
        assert!(set(&mut s, "--empty", "", "INVALID"));
        assert_eq!(s.len(), 0);
    }
    #[test]
    fn inline_shorthand_winners_do_not_resurrect_shadowed_values() {
        let mut s = parse(
            "margin:1px 2px; margin-left:3px !important; margin-left:4px; border:2px solid red; flex:1 1 40px",
        );
        assert_eq!(get(&s, "margin-left"), "3px");
        assert_eq!(get(&s, "margin"), "");
        assert!(set(&mut s, "margin-left", "5px", ""));
        assert_eq!(get(&s, "margin"), "1px 2px 1px 5px");
        assert_eq!(
            s.remove_property("margin", &mut WORK.clone()).unwrap(),
            "1px 2px 1px 5px"
        );
        assert_eq!(get(&s, "margin-left"), "");
        assert_eq!(get(&s, "border"), "2px solid red");
        assert_eq!(get(&s, "flex"), "1 1 40px");
        assert!(set(&mut s, "border-left-color", "blue", ""));
        assert_eq!(get(&s, "border-top-color"), "red");
        assert_eq!(get(&s, "border-left-color"), "blue");
        assert_eq!(parse(&text(&s)), s);
    }
    #[test]
    fn inline_pending_shorthands_preserve_order_and_safe_overrides() {
        let mut s = parse(
            "--space:2px; margin-top:1px; margin:var(--space); background:var(--Tone); color:red",
        );
        assert_eq!(get(&s, "margin"), "var(--space)");
        assert_eq!(get(&s, "margin-top"), "");
        assert!(set(&mut s, "color", "green", ""));
        assert!(text(&s).contains("margin: var(--space)"));
        assert!(text(&s).contains("background: var(--Tone)"));
        assert!(set(&mut s, "margin-top", "10px", ""));
        assert_eq!(get(&s, "margin-top"), "10px");
        assert_eq!(get(&s, "margin"), "");
        let before = s.clone();
        assert_eq!(
            s.remove_property("margin-top", &mut WORK.clone()).unwrap(),
            ""
        );
        assert_eq!(s, before);
        assert!(set(&mut s, "margin", "4px", ""));
        assert_eq!(get(&s, "margin"), "4px");
        let mut s = parse("margin:var(--x) !important");
        let before = s.clone();
        assert!(!set(&mut s, "margin-left", "3px", ""));
        assert_eq!(s, before);
        assert!(set(&mut s, "margin-left", "3px", "important"));
        assert_eq!(get(&s, "margin-left"), "3px");
    }
    #[test]
    fn inline_unrelated_edits_preserve_renderer_values_and_reject_malformed_values() {
        let source = r#"width:calc(100% - 10px); height:calc(100px - 10px); font:italic bold 12px/1.5 "A;B:C",serif; border:2px solid red; flex:1 1 40px; grid-template-columns:repeat(2,minmax(10px,1fr)); margin:var(--space); background:var(--Tone)"#;
        let mut s = parse(source);
        assert_eq!(get(&s, "width"), "calc(100% - 10px)");
        assert_eq!(get(&s, "height"), "calc(100px - 10px)");
        assert_eq!(get(&s, "font-family"), r#""A;B:C",serif"#);
        let before = text(&s);
        assert!(set(&mut s, "color", "green", ""));
        assert!(text(&s).starts_with(&before));
        for (name, value) in [
            ("width", "garbage"),
            ("width", "5"),
            ("width", r"\31 px"),
            ("width", "calc(1px+ 2px)"),
            ("width", "calc(1px +2px)"),
            ("width", "calc(1px/**/+/**/2px)"),
            ("width", "1/**/px"),
            ("width", "calc(10px 20px)"),
            ("color", "r/**/ed"),
            ("color", "rgb(1,2,3) junk"),
            ("border", "1px 2px solid red"),
            ("border", "solid red junk"),
            ("font-family", "serif, "),
            ("font", "12px/"),
            ("flex", "1 1 40px junk"),
            ("margin", "1px inherit"),
            ("background", "notacolor"),
        ] {
            let before = s.clone();
            assert!(!set(&mut s, name, value, ""), "{name}:{value}");
            assert_eq!(s, before);
        }
    }
    #[test]
    fn inline_bounds_exhaustion_and_snapshot_atomicity() {
        assert_eq!(
            InlineStyle::parse(&"x".repeat(MAX_INLINE_BYTES + 1), &mut WORK.clone()),
            Err(Error::Limit)
        );
        assert_eq!(
            InlineStyle::parse(
                &format!("--x:{}", "x".repeat(MAX_VALUE_BYTES + 1)),
                &mut WORK.clone()
            ),
            Err(Error::Limit)
        );
        let source = (0..MAX_DECLARATIONS + 1)
            .map(|i| format!("--x{i}:a;"))
            .collect::<String>();
        assert_eq!(
            InlineStyle::parse(&source, &mut WORK.clone()),
            Err(Error::Limit)
        );
        let mut s = parse("color:red; width:10px; height:20px");
        let before = s.clone();
        let mut work = 5;
        assert_eq!(
            s.set_property("color", "green", "", &mut work),
            Err(Error::Limit)
        );
        assert_eq!(work, 0);
        assert_eq!(s, before);
        assert_eq!(
            s.set_property(
                "--big",
                &"x".repeat(MAX_VALUE_BYTES + 1),
                "",
                &mut WORK.clone()
            ),
            Err(Error::Limit)
        );
        assert_eq!(s, before);
        let mut reasonable = 100_000;
        let mut ordinary =
            InlineStyle::parse("color:red; width:10px; height:20px", &mut reasonable).unwrap();
        assert!(
            ordinary
                .set_property("color", "green", "", &mut reasonable)
                .unwrap()
        );
        assert_eq!(
            ordinary
                .get_property_value("color", &mut reasonable)
                .unwrap(),
            "green"
        );
        assert!(!ordinary.serialize(&mut reasonable).unwrap().is_empty());
        let nested = format!("{}10px{}", "calc(".repeat(12), ")".repeat(12));
        let mut nested_work = 40;
        assert_eq!(
            valid_longhand("width", &nested, &mut nested_work),
            Err(Error::Limit)
        );
        assert_eq!(nested_work, 0);
        let mut recurse_work = nested.len() * 13 + 10;
        assert_eq!(
            valid_longhand("width", &nested, &mut recurse_work),
            Err(Error::Limit)
        );
        assert_eq!(recurse_work, 0);
        assert_eq!(ordinary.item(99), "");
        assert_eq!(ordinary.item(0), "color");
    }
}
