//! Bounded inline declaration storage. DOM bindings perform conversions first,
//! then stage this object and commit its serialization with persistent typed state.
use crate::css::{self, Length};
use crate::dom::{DomDataError, DomMutationBudget};
use crate::selectors::{self, Kind, Token};
use std::{mem::size_of, sync::Arc};

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
struct PendingValue {
    shorthand: Arc<str>,
    tokens: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum RecordValue {
    Tokens(String),
    Pending(Arc<PendingValue>),
}
impl RecordValue {
    fn source(&self) -> &str {
        match self {
            Self::Tokens(tokens) => tokens,
            Self::Pending(pending) => &pending.tokens,
        }
    }
    fn pending(&self) -> Option<&PendingValue> {
        match self {
            Self::Tokens(_) => None,
            Self::Pending(pending) => Some(pending),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    name: String,
    value: RecordValue,
    important: bool,
}
impl Record {
    fn logical_bytes(&self) -> Result<usize, Error> {
        add(
            add(serialized_name_len(&self.name), self.value.source().len())?,
            if self.important { 15 } else { 4 },
        )
    }
}
#[derive(Clone, Copy, Debug)]
struct StyleSummary {
    logical_bytes: usize,
    retained_bytes: usize,
    clone_work: usize,
    release_work: usize,
    names_work: usize,
    pending_records: usize,
}
#[derive(Debug)]
pub(crate) struct InlineStyle {
    records: Vec<Record>,
    summary: StyleSummary,
}
impl PartialEq for InlineStyle {
    fn eq(&self, other: &Self) -> bool {
        self.records == other.records
    }
}
impl Eq for InlineStyle {}
impl Clone for InlineStyle {
    fn clone(&self) -> Self {
        let records = self.records.clone();
        // Runtime callers pay the cached copy, summary and teardown tariff first.
        let summary =
            Self::summarize(&records, records.capacity()).expect("bounded inline clone summary");
        Self { records, summary }
    }
}
impl Default for InlineStyle {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            summary: StyleSummary {
                logical_bytes: 0,
                retained_bytes: arc_bytes(),
                clone_work: 2,
                release_work: 1,
                names_work: 0,
                pending_records: 0,
            },
        }
    }
}
pub(crate) struct InlineDeclarationRef<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub important: bool,
    pub pending_shorthand: Option<&'a Arc<str>>,
}
#[derive(Debug)]
pub(crate) struct PreparedInlineStyle {
    style: Arc<InlineStyle>,
    serialized: String,
}
impl PreparedInlineStyle {
    pub(crate) fn serialized(&self) -> &str {
        &self.serialized
    }
    pub(crate) fn style(&self) -> &InlineStyle {
        &self.style
    }
    pub(crate) fn into_parts(self) -> (Arc<InlineStyle>, String) {
        (self.style, self.serialized)
    }
}
fn add(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_add(b).ok_or(Error::Limit)
}
fn mul(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_mul(b).ok_or(Error::Limit)
}
fn arc_bytes() -> usize {
    2 * size_of::<usize>() + size_of::<InlineStyle>()
}
fn summary_work(count: usize) -> Result<usize, Error> {
    add(1, mul(264, count)?)
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
    (css::supported_property(lower) || css::shorthand_property_count(lower).is_some())
        .then(|| lower.to_owned())
}
pub(crate) fn recognized_property(property: &str) -> bool {
    name(property).is_some()
}
fn targets(property: &str, work: &mut usize) -> Result<Vec<String>, Error> {
    let count = css::shorthand_property_count(property).unwrap_or(1);
    spend(work, add(1, mul(count, add(property.len(), 32)?)?)?)?;
    Ok(css::shorthand_properties(property).unwrap_or_else(|| vec![property.into()]))
}
impl InlineStyle {
    fn summarize(records: &[Record], capacity: usize) -> Result<StyleSummary, Error> {
        let mut logical_bytes = 0;
        let mut retained_bytes = add(arc_bytes(), mul(capacity, size_of::<Record>())?)?;
        let mut names_work = 0;
        let mut pending_records = 0;
        for record in records {
            logical_bytes = add(logical_bytes, record.logical_bytes()?)?;
            names_work = add(names_work, add(record.name.len(), 1)?)?;
            let payload = match &record.value {
                RecordValue::Tokens(tokens) => tokens.capacity(),
                RecordValue::Pending(pending) => {
                    pending_records += 1;
                    add(
                        add(4 * size_of::<usize>(), size_of::<PendingValue>())?,
                        add(pending.tokens.capacity(), pending.shorthand.len())?,
                    )?
                }
            };
            retained_bytes = add(retained_bytes, add(record.name.capacity(), payload)?)?;
        }
        if records.len() > MAX_DECLARATIONS || logical_bytes > MAX_INLINE_BYTES {
            return Err(Error::Limit);
        }
        let release_work = add(add(1, records.len())?, logical_bytes)?;
        let clone_work = add(
            add(summary_work(records.len())?, logical_bytes)?,
            release_work,
        )?;
        Ok(StyleSummary {
            logical_bytes,
            retained_bytes,
            clone_work,
            release_work,
            names_work,
            pending_records,
        })
    }
    fn refresh_summary(&mut self, work: &mut usize) -> Result<(), Error> {
        spend(work, summary_work(self.records.len())?)?;
        self.summary = Self::summarize(&self.records, self.records.capacity())?;
        Ok(())
    }
    #[cfg(test)]
    fn from_records_for_test(records: Vec<Record>) -> Self {
        let summary = Self::summarize(&records, records.capacity()).expect("bounded test records");
        Self { records, summary }
    }
    pub(crate) fn logical_bytes(&self) -> usize {
        self.summary.logical_bytes
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.summary.retained_bytes
    }
    pub(crate) fn clone_work(&self) -> usize {
        self.summary.clone_work
    }
    pub(crate) fn release_work(&self) -> usize {
        self.summary.release_work
    }
    pub(crate) fn declarations(&self) -> impl ExactSizeIterator<Item = InlineDeclarationRef<'_>> {
        self.records.iter().map(|record| InlineDeclarationRef {
            name: &record.name,
            value: record.value.source(),
            important: record.important,
            pending_shorthand: record.value.pending().map(|p| &p.shorthand),
        })
    }
    pub(crate) fn try_clone_for_edit(
        &self,
        budget: &mut DomMutationBudget,
    ) -> Result<Self, DomDataError> {
        budget.work(self.clone_work())?;
        budget.charge(self.retained_bytes())?;
        Ok(self.clone())
    }
    pub(crate) fn prepare_commit(
        self,
        budget: &mut DomMutationBudget,
    ) -> Result<PreparedInlineStyle, DomDataError> {
        // Pays incoming-block teardown even when serialization or later DOM admission refuses.
        budget.work(
            self.release_work()
                .checked_add(2)
                .ok_or(DomDataError::LimitExceeded)?,
        )?;
        budget.charge(
            self.logical_bytes()
                .checked_add(arc_bytes())
                .ok_or(DomDataError::LimitExceeded)?,
        )?;
        let serialized = self
            .serialize(&mut budget.steps)
            .map_err(|_| DomDataError::LimitExceeded)?;
        Ok(PreparedInlineStyle {
            style: Arc::new(self),
            serialized,
        })
    }
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
        style.refresh_summary(work)?;
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
                    add(self.summary.names_work, add(record.name.len(), 1)?)?,
                )?;
                let index = self.records.iter().position(|old| old.name == record.name);
                if index.is_some_and(|i| self.records[i].important && !record.important) {
                    continue;
                }
                self.admit_replacement(index, &record)?;
                // Parse precedence retains the last winning source position.
                if let Some(index) = index {
                    self.records.remove(index);
                }
                self.records.push(record);
            }
        }
        Ok(())
    }
    // Called only inside a funded merge/mutation pass; keep admission metadata
    // incremental so no record-vector growth precedes count/logical admission.
    fn admit_replacement(
        &mut self,
        index: Option<usize>,
        replacement: &Record,
    ) -> Result<(), Error> {
        let old = index.map(|i| &self.records[i]);
        let old_bytes = old.map_or(Ok(0), Record::logical_bytes)?;
        let new_bytes = replacement.logical_bytes()?;
        let logical = add(
            self.summary
                .logical_bytes
                .checked_sub(old_bytes)
                .ok_or(Error::Limit)?,
            new_bytes,
        )?;
        let count = add(self.records.len(), usize::from(index.is_none()))?;
        if logical > MAX_INLINE_BYTES || count > MAX_DECLARATIONS {
            return Err(Error::Limit);
        }
        let old_names = old.map_or(0, |r| r.name.len() + 1);
        self.summary.names_work = add(
            self.summary
                .names_work
                .checked_sub(old_names)
                .ok_or(Error::Limit)?,
            replacement.name.len() + 1,
        )?;
        self.summary.logical_bytes = logical;
        Ok(())
    }
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    pub(crate) fn item(&self, index: usize) -> &str {
        self.records
            .get(index)
            .map_or("", |record| record.name.as_str())
    }
    fn query_targets(
        &self,
        property: &str,
        work: &mut usize,
    ) -> Result<Option<(String, Vec<String>)>, Error> {
        spend(work, property.len().saturating_add(1))?;
        let Some(normalized) = name(property) else {
            spend(work, add(self.summary.names_work, add(property.len(), 1)?)?)?;
            return Ok(None);
        };
        let names = targets(&normalized, work)?;
        spend(
            work,
            mul(
                add(self.summary.names_work, add(property.len(), 1)?)?,
                names.len().max(1),
            )?,
        )?;
        Ok(Some((normalized, names)))
    }
    fn winner(&self, property: &str) -> Option<&Record> {
        self.records.iter().find(|record| record.name == property)
    }
    pub(crate) fn get_property_value(
        &self,
        property: &str,
        work: &mut usize,
    ) -> Result<String, Error> {
        let Some((property, names)) = self.query_targets(property, work)? else {
            return Ok(String::new());
        };
        if css::shorthand_property_count(&property).is_none() {
            let value = self
                .winner(&property)
                .and_then(|record| match &record.value {
                    RecordValue::Tokens(tokens) => Some(tokens.as_str()),
                    RecordValue::Pending(_) => None,
                })
                .unwrap_or("");
            spend(work, value.len() + 1)?;
            return Ok(value.into());
        }
        // This prepayment precedes any pending-source content/importance checks.
        if self.summary.pending_records != 0 {
            spend(work, add(self.logical_bytes(), 1)?)?;
        }
        let values: Option<Vec<_>> = names.iter().map(|p| self.winner(p)).collect();
        let Some(values) = values else {
            return Ok(String::new());
        };
        if values.iter().any(|r| r.important != values[0].important) {
            return Ok(String::new());
        }
        if let Some(pending) = values[0].value.pending() {
            if pending.shorthand.as_ref() == property
                && values.iter().all(|r| r.value.pending() == Some(pending))
            {
                spend(work, pending.tokens.len() + 1)?;
                return Ok(pending.tokens.clone());
            }
            return Ok(String::new());
        }
        if values.iter().any(|r| r.value.pending().is_some()) {
            return Ok(String::new());
        }
        let raw: Vec<_> = values.iter().map(|r| r.value.source()).collect();
        spend(
            work,
            raw.iter()
                .map(|v| v.len() + 1)
                .sum::<usize>()
                .saturating_mul(8)
                + 1,
        )?;
        // Explicit longhand substitutions cannot be combined into an equivalent
        // shorthand. Original Pending reconstruction returned above; this paid
        // token-aware scan ignores comments, strings and URL payloads.
        if raw.iter().any(|value| css::contains_var_function(value)) {
            return Ok(String::new());
        }
        if property == "background" && !raw.iter().all(|v| *v == raw[0] && wide(v)) {
            return css::serialize_background_shorthand(&raw, work).map_err(|_| Error::Limit);
        }
        Ok(shorthand_value(&property, &raw))
    }
    pub(crate) fn get_property_priority(
        &self,
        property: &str,
        work: &mut usize,
    ) -> Result<&'static str, Error> {
        let Some((_, names)) = self.query_targets(property, work)? else {
            return Ok("");
        };
        Ok(
            if names
                .iter()
                .all(|p| self.winner(p).is_some_and(|r| r.important))
            {
                "important"
            } else {
                ""
            },
        )
    }
    fn mutation_work(&self, property: &str, count: usize, work: &mut usize) -> Result<(), Error> {
        spend(
            work,
            add(
                1,
                mul(
                    mul(2, count)?,
                    add(self.summary.names_work, add(property.len(), 1)?)?,
                )?,
            )?,
        )
    }
    fn commit_staged(&mut self, mut staged: Self, work: &mut usize) -> Result<bool, Error> {
        staged.refresh_summary(work)?;
        spend(
            work,
            add(
                1,
                mul(2, add(self.logical_bytes(), staged.logical_bytes())?)?,
            )?,
        )?;
        let changed = *self != staged;
        *self = staged;
        Ok(changed)
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
        let Some(replacements) = records(&property, value, !priority.is_empty(), work)? else {
            return Ok(false);
        };
        let Some((_, names)) = self.query_targets(&property, work)? else {
            return Ok(false);
        };
        spend(work, self.clone_work())?;
        self.mutation_work(&property, names.len(), work)?;
        let mut staged = self.clone();
        for replacement in replacements {
            let index = staged
                .records
                .iter()
                .position(|r| r.name == replacement.name);
            staged.admit_replacement(index, &replacement)?;
            if let Some(index) = index {
                staged.records[index] = replacement;
            } else {
                staged.records.push(replacement);
            }
        }
        self.commit_staged(staged, work)
    }
    pub(crate) fn remove_property(
        &mut self,
        property: &str,
        work: &mut usize,
    ) -> Result<String, Error> {
        let previous = self.get_property_value(property, work)?;
        let Some((property, names)) = self.query_targets(property, work)? else {
            return Ok(previous);
        };
        spend(work, self.clone_work())?;
        self.mutation_work(&property, names.len(), work)?;
        let mut staged = self.clone();
        staged.records.retain(|r| !names.contains(&r.name));
        self.commit_staged(staged, work)?;
        Ok(previous)
    }
    fn check_size(&self) -> Result<(), Error> {
        if self.records.len() > MAX_DECLARATIONS || self.logical_bytes() > MAX_INLINE_BYTES {
            Err(Error::Limit)
        } else {
            Ok(())
        }
    }
    pub(crate) fn serialize(&self, work: &mut usize) -> Result<String, Error> {
        self.check_size()?;
        spend(work, add(mul(2, self.logical_bytes())?, 1)?)?;
        let mut considered = [false; MAX_DECLARATIONS];
        let mut skip = [false; MAX_DECLARATIONS];
        let mut shorthand = [false; MAX_DECLARATIONS];
        if self.summary.pending_records != 0 {
            spend(work, self.records.len() + 1)?;
            for (i, record) in self.records.iter().enumerate() {
                if considered[i] {
                    continue;
                }
                let Some(pending) = record.value.pending() else {
                    continue;
                };
                spend(
                    work,
                    add(
                        add(
                            1,
                            mul(self.records.len(), add(pending.shorthand.len(), 4)?)?,
                        )?,
                        self.logical_bytes(),
                    )?,
                )?;
                let mut count = 0;
                let mut complete = true;
                let mut group = [false; MAX_DECLARATIONS];
                for (j, other) in self.records.iter().enumerate() {
                    if let Some(source) = other.value.pending()
                        && source.shorthand == pending.shorthand
                    {
                        considered[j] = true;
                        group[j] = true;
                        count += 1;
                        complete &= source == pending && other.important == record.important;
                    }
                }
                complete &= css::shorthand_property_count(&pending.shorthand) == Some(count);
                if complete {
                    shorthand[i] = true;
                    // The paid origin scan also funds this fixed flag publication.
                    for (j, matches) in group.iter().enumerate().take(self.records.len()) {
                        if j != i && *matches {
                            skip[j] = true;
                        }
                    }
                }
            }
        }
        let mut out = String::with_capacity(self.logical_bytes());
        for (i, record) in self.records.iter().enumerate() {
            if skip[i] {
                continue;
            }
            if !out.is_empty() {
                out.push(' ');
            }
            if shorthand[i] {
                let pending = record.value.pending().expect("planned pending shorthand");
                write_name(&mut out, &pending.shorthand);
                out.push_str(": ");
                out.push_str(&pending.tokens);
            } else {
                write_name(&mut out, &record.name);
                out.push_str(": ");
                if let RecordValue::Tokens(value) = &record.value {
                    out.push_str(value);
                }
            }
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
    // Normalized CSS-wide values can reconstruct only one identical keyword.
    // The mutually exclusive paths inspect each remaining component once,
    // within the caller's existing synthesis allowance.
    let first = values[0];
    if wide(first) {
        return if values[1..].iter().all(|value| *value == first) {
            first.into()
        } else {
            String::new()
        };
    }
    if values[1..].iter().any(|value| wide(value)) {
        return String::new();
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
        "background" => {
            let mut layer =
                if values[2] == "auto auto" && values[3] == "0% 0%" && values[4] == "repeat" {
                    if values[1] == "none" {
                        values[0].into()
                    } else if values[0] == "transparent" {
                        values[1].into()
                    } else {
                        format!("{} {}", values[1], values[0])
                    }
                } else {
                    // A size always follows a position and slash. Keep authored
                    // longhand syntax while emitting an unambiguous complete layer.
                    format!(
                        "{} {} / {} {} {}",
                        values[1], values[3], values[2], values[4], values[0]
                    )
                };
            // Defaults preserve the existing layer serialization. One box
            // sets both properties; different values need origin then clip.
            if values[5] != "padding-box" || values[6] != "border-box" {
                layer.push(' ');
                layer.push_str(values[5]);
                if values[5] != values[6] {
                    layer.push(' ');
                    layer.push_str(values[6]);
                }
            }
            layer
        }
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
    let count = css::shorthand_property_count(property).unwrap_or(1);
    spend(work, add(1, mul(count, add(property.len(), 32)?)?)?)?;
    // Flat temporary records are paid for even if a later validation/stage fails.
    spend(work, add(1, mul(8, count)?)?)?;
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
            value: RecordValue::Tokens(value.into()),
            important,
        }]));
    }
    if value.is_empty() {
        return Ok(None);
    }
    if css::contains_var_function(value) {
        spend(work, add(1, mul(count, add(value.len(), 64)?)?)?)?;
        if let Some(names) = css::shorthand_properties(property) {
            let source = Arc::new(PendingValue {
                shorthand: Arc::from(property),
                tokens: value.into(),
            });
            return Ok(Some(
                names
                    .into_iter()
                    .map(|name| Record {
                        name,
                        value: RecordValue::Pending(Arc::clone(&source)),
                        important,
                    })
                    .collect(),
            ));
        }
        return Ok(Some(vec![Record {
            name: property.into(),
            value: RecordValue::Tokens(value.into()),
            important,
        }]));
    }
    if !css::valid_calculations(value, work).map_err(|_| Error::Limit)? {
        return Ok(None);
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
    spend(work, count.saturating_mul(value.len() + 64) + 1)?;
    if matches!(
        property,
        "border-spacing"
            | "background-image"
            | "background-size"
            | "background-position"
            | "background-repeat"
            | "background-origin"
            | "background-clip"
    ) {
        // These normalized longhands expand unchanged. Their complete value
        // was just validated; retain it without a Declaration roundtrip and
        // another identical parser pass. Pending var() values returned above.
        return Ok(Some(vec![Record {
            name: property.into(),
            value: RecordValue::Tokens(normalized),
            important,
        }]));
    }
    let mut expanded = Vec::new();
    css::expand_declaration_with_budget(property, value, important, &mut expanded, work)
        .map_err(|_| Error::Limit)?;
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
                value: RecordValue::Tokens(d.value),
                important: d.important,
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
// The renderer and CSSOM share one typed additive calculation grammar. CSSOM
// retains authored units; actual font/viewport coefficients are computed later.
fn length(value: &str, percent: bool, negative: bool, _depth: usize, work: &mut usize) -> bool {
    if spend(work, value.len().saturating_mul(4) + 1).is_err() || value.is_empty() {
        return false;
    }
    if value.contains('(') {
        return match css::parse_calculation(value, [16.0, 16.0, 800.0, 600.0], work) {
            Ok(Some(length)) => percent || !length.depends_on_percentage(),
            Err(_) => {
                *work = 0;
                false
            }
            _ => false,
        };
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
    if css::shorthand_property_count(property).is_none() {
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
        "background" => {
            css::valid_background_property(property, value, work).map_err(|_| Error::Limit)?
        }
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
    if property == "visibility" {
        return css::valid_visibility(value, work).map_err(|_| Error::Limit);
    }
    if property == "border-spacing" {
        return css::valid_border_spacing(value, work).map_err(|_| Error::Limit);
    }
    if matches!(
        property,
        "background-image"
            | "background-size"
            | "background-position"
            | "background-repeat"
            | "background-origin"
            | "background-clip"
    ) {
        return css::valid_background_property(property, value, work).map_err(|_| Error::Limit);
    }
    if matches!(property, "object-fit" | "object-position") {
        return css::valid_object_property(property, value, work).map_err(|_| Error::Limit);
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
            (1..=4).contains(&parts.len())
                && parts
                    .iter()
                    .all(|p| length(p, !p.contains('('), false, 0, work))
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
    fn object_properties_cssom_preserves_valid_winners_and_priority() {
        let mut style = parse(
            "object-fit:contain;object-position:25% 75%;object-fit:fill cover!important;object-position:top 20%!important",
        );
        assert_eq!(get(&style, "object-fit"), "contain");
        assert_eq!(get(&style, "object-position"), "25% 75%");
        for (property, values) in [
            (
                "object-fit",
                &["auto", "contain cover", "fill scale-down", "cover cover"][..],
            ),
            (
                "object-position",
                &[
                    "auto",
                    "1",
                    "top 20%",
                    "left 5px top",
                    "center 1px bottom 2px",
                    "calc(1px/**/+/**/2px)",
                ][..],
            ),
        ] {
            for value in values {
                let before = style.clone();
                assert!(
                    !set(&mut style, property, value, "important"),
                    "{property}:{value}"
                );
                assert_eq!(style, before);
            }
        }
        assert!(set(
            &mut style,
            "OBJECT-FIT",
            "COVER SCALE-DOWN",
            "important"
        ));
        assert!(set(
            &mut style,
            "object-position",
            "right calc(25% + 2px) bottom -5px",
            ""
        ));
        assert_eq!(get(&style, "object-fit"), "cover scale-down");
        assert_eq!(
            style
                .get_property_priority("object-fit", &mut WORK.clone())
                .unwrap(),
            "important"
        );
        assert_eq!(
            get(&style, "object-position"),
            "right calc(25% + 2px) bottom -5px"
        );
        assert_eq!(parse(&text(&style)), style);
        assert_eq!(
            style
                .remove_property("object-fit", &mut WORK.clone())
                .unwrap(),
            "cover scale-down"
        );
        assert_eq!(get(&style, "object-fit"), "");
    }

    #[test]
    fn object_properties_cssom_pending_variables_and_serialized_cascade() {
        let style = parse(
            "object-fit:cover;object-position:left;--fit:bogus;--pos:top 20%;object-fit:var(--fit);object-position:var(--pos)",
        );
        assert_eq!(get(&style, "object-fit"), "var(--fit)");
        assert_eq!(get(&style, "object-position"), "var(--pos)");
        let document = crate::dom::Document::parse(&format!("<img id=x style='{}'>", text(&style)));
        let computed = css::compute_styles(&document, &[], 800.0, 600.0);
        let computed = &computed[document.query_selector("#x").unwrap()];
        assert_eq!(computed.object_fit, css::ObjectFit::Fill);
        assert_eq!(computed.object_position, css::ObjectPosition::default());
        let style = parse("object-fit:cover/**/scale-down;object-position:top/**/left");
        assert_eq!(get(&style, "object-fit"), "cover scale-down");
        assert_eq!(get(&style, "object-position"), "top left");
        assert_eq!(parse(&text(&style)), style);
        for keyword in ["inherit", "initial", "unset", "revert", "revert-layer"] {
            let mut style = InlineStyle::default();
            assert!(set(&mut style, "object-fit", keyword, ""));
            assert!(set(&mut style, "object-position", keyword, ""));
            assert_eq!(get(&style, "object-position"), keyword);
        }
    }

    #[test]
    fn object_position_cssom_exact_and_short_work_preserve_atomic_storage() {
        let original = parse("object-fit:contain;object-position:20% 30%;color:red");
        let value = "right calc(25% + 2px) bottom calc(1em - 3px)";
        let mut measured = original.clone();
        let mut work = WORK;
        assert!(
            measured
                .set_property("object-position", value, "important", &mut work)
                .unwrap()
        );
        let cost = WORK - work;
        assert!(cost > 0);
        let mut exact = original.clone();
        let mut exact_work = cost;
        assert!(
            exact
                .set_property("object-position", value, "important", &mut exact_work)
                .unwrap()
        );
        assert_eq!(exact_work, 0);
        assert_eq!(exact, measured);
        for limit in [0, 1, cost - 1] {
            let mut short = original.clone();
            let mut short_work = limit;
            assert_eq!(
                short.set_property("object-position", value, "important", &mut short_work),
                Err(Error::Limit)
            );
            assert_eq!(short, original);
        }
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
        assert_eq!(
            s.remove_property("margin-top", &mut WORK.clone()).unwrap(),
            "10px"
        );
        assert_eq!(get(&s, "margin-top"), "");
        assert!(set(&mut s, "margin", "4px", ""));
        assert_eq!(get(&s, "margin"), "4px");
        let mut s = parse("margin:var(--x) !important");
        assert!(set(&mut s, "margin-left", "3px", ""));
        assert_eq!(
            s.get_property_priority("margin-left", &mut WORK.clone()),
            Ok("")
        );
        assert_eq!(
            s.get_property_priority("margin-top", &mut WORK.clone()),
            Ok("important")
        );
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
    fn inline_calculations_share_typed_grammar_and_preserve_atomic_mutations() {
        let mut work = 1_000_000;
        let mut style =
            InlineStyle::parse("width:77px;border-radius:7px;margin:8px", &mut work).unwrap();
        for value in [
            "calc(1px +)",
            "calc(0 + 1px)",
            "calc(1px/**/+/**/2px)",
            "calc(1px * 2)",
            "calc(1px) trailing",
        ] {
            assert!(
                !style.set_property("width", value, "", &mut work).unwrap(),
                "{value}"
            );
            assert_eq!(
                style.get_property_value("width", &mut work).unwrap(),
                "77px"
            );
        }
        assert!(
            style
                .set_property("width", r"c\61 lc((50% - 2em) + 1rem)", "", &mut work)
                .unwrap()
        );
        assert_eq!(
            style.get_property_value("width", &mut work).unwrap(),
            "calc((50% - 2em) + 1rem)"
        );
        assert!(
            style
                .set_property("margin", "calc(1px)/**/calc(2px)", "", &mut work)
                .unwrap()
        );
        assert!(
            !style
                .set_property("border-radius", "calc(10% + 1px)", "", &mut work)
                .unwrap()
        );
        assert!(
            style
                .set_property("width", "calc(-1px)", "", &mut work)
                .unwrap()
        );
        let text = style.serialize(&mut work).unwrap();
        let doc = crate::dom::Document::parse(&format!("<div style='{text}'></div>"));
        let styles = css::compute_styles(&doc, &[], 800.0, 600.0);
        let computed = &styles[doc.query_selector("div").unwrap()];
        assert_eq!(computed.width.resolve(0.0), Some(0.0));
        assert_eq!(computed.margin.top.resolve(0.0), Some(1.0));
        assert_eq!(computed.margin.left.resolve(0.0), Some(2.0));
        let old = style.serialize(&mut work).unwrap();
        let mut short = 9;
        assert_eq!(
            style.set_property("width", "calc(50% + 1px)", "", &mut short),
            Err(Error::Limit)
        );
        assert_eq!(old, style.serialize(&mut work).unwrap());
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

#[cfg(test)]
mod gradient_tests;

#[cfg(test)]
mod background_tests;

#[cfg(test)]
mod url_background_tests;

#[cfg(test)]
mod background_box_tests;

#[cfg(test)]
mod background_layers_tests;

#[cfg(test)]
#[path = "cssom/border_spacing_tariff_tests.rs"]
mod border_spacing_tariff_tests;

#[cfg(test)]
mod border_spacing_independent_tests;

#[cfg(test)]
mod pending_record_tests;

#[cfg(test)]
mod pending_record_accounting_tests;

#[cfg(test)]
mod var_getter_tests;

#[cfg(test)]
mod var_getter_budget_tests;

#[cfg(test)]
mod wide_getter_tests;

#[cfg(test)]
mod wide_getter_budget_tests;
