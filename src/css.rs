//! Independent CSS parsing, selector cascade, inheritance and computed values.
use crate::dom::{
    Document, Namespace, NodeId, NodeKind, matches_compiled_selector, split_top_level,
};
use crate::graphics::Color;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Length {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
    /// Flexible grid track weight; resolved after fixed tracks and gaps.
    Fr(f32),
}
impl Length {
    pub fn resolve(self, reference: f32) -> Option<f32> {
        match self {
            Self::Auto | Self::Fr(_) => None,
            Self::Px(v) => Some(v),
            Self::Percent(v) => Some(reference * v / 100.0),
        }
    }
}
/// Track minima and maxima stay distinct until the grid has a containing size.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum GridBreadth {
    #[default]
    Auto,
    MinContent,
    MaxContent,
    Length(Length),
}
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GridTrack {
    pub min: GridBreadth,
    pub max: GridBreadth,
}
impl GridTrack {
    pub fn single(value: GridBreadth) -> Self {
        Self {
            min: if matches!(value, GridBreadth::Length(Length::Fr(_))) {
                GridBreadth::Auto
            } else {
                value
            },
            max: value,
        }
    }
}
// Defaults and explicitly inherited lists share immutable storage. Computed
// styles can number in the tens of thousands even for a small stylesheet.
static EMPTY_GRID_TRACKS: LazyLock<Arc<[GridTrack]>> = LazyLock::new(|| Arc::from([]));
static AUTO_GRID_TRACKS: LazyLock<Arc<[GridTrack]>> =
    LazyLock::new(|| Arc::from([GridTrack::default()]));

const MAX_RETAINED_GRID_BYTES: usize = 4 * 1024 * 1024;

#[derive(Default)]
struct GridTrackPool {
    lists: HashMap<Vec<u32>, Arc<[GridTrack]>>,
    bytes: usize,
}
impl GridTrackPool {
    fn intern(&mut self, tracks: &mut Arc<[GridTrack]>, implicit: bool) {
        // Shared candidates are initial values or inherit an array already
        // interned for an ancestor during this same style computation.
        if Arc::strong_count(tracks) > 1 {
            return;
        }
        let mut key = Vec::with_capacity(tracks.len() * 4);
        for track in tracks.iter() {
            for breadth in [track.min, track.max] {
                let (tag, bits) = match breadth {
                    GridBreadth::Auto => (0, 0),
                    GridBreadth::MinContent => (1, 0),
                    GridBreadth::MaxContent => (2, 0),
                    GridBreadth::Length(Length::Auto) => (3, 0),
                    GridBreadth::Length(Length::Px(n)) => (4, n.to_bits()),
                    GridBreadth::Length(Length::Percent(n)) => (5, n.to_bits()),
                    GridBreadth::Length(Length::Fr(n)) => (6, n.to_bits()),
                };
                key.extend([tag, bits]);
            }
        }
        if let Some(shared) = self.lists.get(&key) {
            *tracks = Arc::clone(shared);
            return;
        }
        let bytes = std::mem::size_of_val(tracks.as_ref());
        // Empty arrays otherwise consume no charged bytes but still grow the
        // hash table; canonicalize them to the shared initial empty list.
        if tracks.is_empty() {
            *tracks = Arc::clone(&EMPTY_GRID_TRACKS);
            return;
        }
        if bytes > MAX_RETAINED_GRID_BYTES.saturating_sub(self.bytes) {
            *tracks = Arc::clone(if implicit {
                &AUTO_GRID_TRACKS
            } else {
                &EMPTY_GRID_TRACKS
            });
            return;
        }
        self.bytes += bytes;
        self.lists.insert(key, Arc::clone(tracks));
    }
    fn style(&mut self, style: &mut ComputedStyle) {
        self.intern(&mut style.grid_template_columns, false);
        self.intern(&mut style.grid_template_rows, false);
        self.intern(&mut style.grid_auto_columns, true);
        self.intern(&mut style.grid_auto_rows, true);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GridLine {
    #[default]
    Auto,
    Line(i32),
    Span(usize),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}
impl<T: Copy> Edges<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}
impl<T: Default> Default for Edges<T> {
    fn default() -> Self {
        Self {
            top: T::default(),
            right: T::default(),
            bottom: T::default(),
            left: T::default(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    None,
    Block,
    #[default]
    Inline,
    InlineBlock,
    Flex,
    Grid,
}
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub display: Display,
    pub flow_root: bool,
    pub list_item: bool,
    pub float: String,
    pub clear: String,
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub margin: Edges<Length>,
    pub padding: Edges<Length>,
    pub border_width: Edges<f32>,
    pub color: Color,
    pub background_color: Color,
    pub border_color: Color,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub font_family: String,
    pub font_style: String,
    pub text_align: String,
    pub white_space: String,
    pub text_decoration: String,
    pub flex_direction: String,
    pub flex_wrap: String,
    pub justify_content: String,
    pub align_items: String,
    pub align_self: String,
    pub order: i32,
    pub gap: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Length,
    pub row_gap: Length,
    pub column_gap: Length,
    pub grid_template_columns: Arc<[GridTrack]>,
    pub grid_template_rows: Arc<[GridTrack]>,
    pub grid_auto_columns: Arc<[GridTrack]>,
    pub grid_auto_rows: Arc<[GridTrack]>,
    pub grid_column_start: GridLine,
    pub grid_column_end: GridLine,
    pub grid_row_start: GridLine,
    pub grid_row_end: GridLine,
    pub grid_auto_flow: String,
    pub justify_items: String,
    pub justify_self: String,
    pub align_content: String,
    pub position: String,
    pub z_index: Option<i32>,
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub overflow: String,
    pub border_radius: f32,
    pub opacity: f32,
    pub box_sizing: String,
    pub list_style_type: String,
    pub list_style_position: String,
    pub vertical_align: String,
}
impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            flow_root: false,
            list_item: false,
            float: "none".into(),
            clear: "none".into(),
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            margin: Edges::all(Length::Px(0.0)),
            padding: Edges::all(Length::Px(0.0)),
            border_width: Edges::all(0.0),
            color: Color::BLACK,
            background_color: Color::TRANSPARENT,
            border_color: Color::BLACK,
            font_size: 16.0,
            line_height: 19.2,
            font_weight: 400,
            font_family: "sans-serif".into(),
            font_style: "normal".into(),
            text_align: "start".into(),
            white_space: "normal".into(),
            text_decoration: "none".into(),
            flex_direction: "row".into(),
            flex_wrap: "nowrap".into(),
            justify_content: "normal".into(),
            align_items: "stretch".into(),
            align_self: "auto".into(),
            order: 0,
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Length::Auto,
            row_gap: Length::Px(0.0),
            column_gap: Length::Px(0.0),
            grid_template_columns: Arc::clone(&EMPTY_GRID_TRACKS),
            grid_template_rows: Arc::clone(&EMPTY_GRID_TRACKS),
            grid_auto_columns: Arc::clone(&AUTO_GRID_TRACKS),
            grid_auto_rows: Arc::clone(&AUTO_GRID_TRACKS),
            grid_column_start: GridLine::Auto,
            grid_column_end: GridLine::Auto,
            grid_row_start: GridLine::Auto,
            grid_row_end: GridLine::Auto,
            grid_auto_flow: "row".into(),
            justify_items: "stretch".into(),
            justify_self: "auto".into(),
            align_content: "normal".into(),
            position: "static".into(),
            z_index: None,
            top: Length::Auto,
            right: Length::Auto,
            bottom: Length::Auto,
            left: Length::Auto,
            overflow: "visible".into(),
            border_radius: 0.0,
            opacity: 1.0,
            box_sizing: "content-box".into(),
            list_style_type: "disc".into(),
            list_style_position: "outside".into(),
            vertical_align: "baseline".into(),
        }
    }
}
impl ComputedStyle {
    fn inherited(parent: Option<&Self>) -> Self {
        let mut s = Self::default();
        if let Some(p) = parent {
            s.color = p.color;
            s.font_size = p.font_size;
            s.line_height = p.line_height;
            s.font_weight = p.font_weight;
            s.font_family = p.font_family.clone();
            s.font_style = p.font_style.clone();
            s.text_align = p.text_align.clone();
            s.white_space = p.white_space.clone();
            s.list_style_type = p.list_style_type.clone();
            s.list_style_position = p.list_style_position.clone();
            s.text_decoration = p.text_decoration.clone();
        }
        s
    }
}
#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub value: String,
    pub important: bool,
    // Pending substitution preserves shorthand semantics until var() resolves.
    pending_shorthand: Option<Arc<str>>,
}
#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<String>,
    pub declarations: Vec<Declaration>,
    pub layer: Option<Arc<CascadeLayer>>,
}

const MAX_LAYERS: usize = 1024;
const MAX_LAYER_DEPTH: usize = 16;
const MAX_LAYER_NAME_BYTES: usize = 64 * 1024;
const MAX_STYLE_BYTES: usize = 8 * 1024 * 1024;

/// A source boundary preserves stylesheet EOF semantics and imported layers.
#[derive(Debug, Clone)]
pub struct StyleSource {
    pub source: Arc<str>,
    pub layer: Option<Arc<CascadeLayer>>,
    /// Nested import/link conditions are a conjunction, not a textual wrapper.
    pub media: Vec<Arc<str>>,
}
impl StyleSource {
    pub fn new(source: impl Into<Arc<str>>) -> Self {
        Self {
            source: source.into(),
            layer: None,
            media: Vec::new(),
        }
    }
}

/// Named identities merge within their parent. Anonymous identities are the
/// identity of this Arc allocation; they cannot collide with authored names.
#[derive(Debug)]
pub struct CascadeLayer {
    name: Option<String>,
    parent: Option<Arc<CascadeLayer>>,
    depth: usize,
}
impl CascadeLayer {
    pub fn named(parent: Option<Arc<Self>>, name: &str) -> Option<Arc<Self>> {
        let names = layer_name(name)?;
        let depth = parent.as_ref().map_or(0, |p| p.depth);
        if names.len() + depth > MAX_LAYER_DEPTH {
            return None;
        }
        let mut parent = parent;
        for name in names {
            let depth = parent.as_ref().map_or(1, |p| p.depth + 1);
            parent = Some(Arc::new(Self {
                name: Some(name),
                parent,
                depth,
            }));
        }
        parent
    }
    pub fn anonymous(parent: Option<Arc<Self>>) -> Option<Arc<Self>> {
        let depth = parent.as_ref().map_or(1, |p| p.depth + 1);
        (depth <= MAX_LAYER_DEPTH).then(|| {
            Arc::new(Self {
                name: None,
                parent,
                depth,
            })
        })
    }
}

// CSS <ident> tokens, including escaped code points. Layer names are case
// sensitive; CSS-wide keywords are forbidden as individual components.
fn layer_name(source: &str) -> Option<Vec<String>> {
    if source.len() > 4096 {
        return None;
    }
    let clean = strip_comments(source);
    let mut rest = media_trim(&clean);
    let mut names = Vec::new();
    loop {
        let (name, consumed) = css_identifier(rest)?;
        if name.is_empty()
            || name.len() > 1024
            || matches!(
                name.to_ascii_lowercase().as_str(),
                "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "revert-rule"
            )
        {
            return None;
        }
        names.push(name);
        if names.len() > MAX_LAYER_DEPTH {
            return None;
        }
        rest = rest[consumed..].trim_start_matches(media_space);
        if rest.is_empty() {
            return Some(names);
        }
        rest = rest.strip_prefix('.')?.trim_start_matches(media_space);
    }
}

fn css_identifier(rest: &str) -> Option<(String, usize)> {
    let mut name = String::new();
    let mut consumed = 0;
    let mut chars = rest.char_indices().peekable();
    let first = rest.chars().next()?;
    let second = rest.chars().nth(1);
    let start = |c: char| matches!(c, '_' | '\0') || c.is_ascii_alphabetic() || !c.is_ascii();
    if !(start(first)
        || first == '\\'
        || first == '-' && second.is_some_and(|c| start(c) || matches!(c, '-' | '\\')))
    {
        return None;
    }
    while let Some((at, ch)) = chars.next() {
        if name.len() > 1024 {
            return None;
        }
        if ch == '\\' {
            let (_, next) = chars.next()?;
            if matches!(next, '\n' | '\r' | '\x0c') {
                return None;
            }
            if next.is_ascii_hexdigit() {
                let mut value = next.to_digit(16)?;
                consumed = at + 1 + next.len_utf8();
                for _ in 1..6 {
                    if let Some((offset, digit)) = chars.peek().copied()
                        && digit.is_ascii_hexdigit()
                    {
                        chars.next();
                        value = value * 16 + digit.to_digit(16)?;
                        consumed = offset + digit.len_utf8();
                    } else {
                        break;
                    }
                }
                if let Some((offset, space)) = chars.peek().copied()
                    && media_space(space)
                {
                    chars.next();
                    consumed = offset + space.len_utf8();
                    if space == '\r' && chars.peek().is_some_and(|(_, c)| *c == '\n') {
                        let (offset, _) = chars.next()?;
                        consumed = offset + 1;
                    }
                }
                name.push(
                    char::from_u32(value)
                        .filter(|c| *c != '\0')
                        .unwrap_or('\u{fffd}'),
                );
            } else {
                name.push(if next == '\0' { '\u{fffd}' } else { next });
                consumed = at + 1 + next.len_utf8();
            }
        } else if start(ch) || ch.is_ascii_digit() || ch == '-' {
            name.push(if ch == '\0' { '\u{fffd}' } else { ch });
            consumed = at + ch.len_utf8();
        } else {
            break;
        }
    }
    (!name.is_empty() && name.len() <= 1024).then_some((name, consumed))
}

pub fn parse_stylesheet(source: &str, width: f32, height: f32) -> Vec<Rule> {
    parse_stylesheet_in_layer(source, None, width, height)
}
pub fn parse_stylesheet_in_layer(
    source: &str,
    layer: Option<Arc<CascadeLayer>>,
    width: f32,
    height: f32,
) -> Vec<Rule> {
    let mut rules = Vec::new();
    parse_source(
        source,
        &layer,
        width,
        height,
        &mut rules,
        &mut ParseBudget::new(),
    );
    rules
}
fn parse_source(
    source: &str,
    layer: &Option<Arc<CascadeLayer>>,
    width: f32,
    height: f32,
    rules: &mut Vec<Rule>,
    budget: &mut ParseBudget,
) {
    if rules.len() >= 10_000 {
        return;
    }
    let mut end = source.len().min(MAX_STYLE_BYTES);
    while !source.is_char_boundary(end) {
        end -= 1;
    }
    let source = &source[..end];
    if let Some(layer) = layer {
        rules.push(Rule {
            selectors: vec![],
            declarations: vec![],
            layer: Some(layer.clone()),
        });
    }
    parse_rules(source, width, height, 0, layer, true, rules, budget);
}

struct ParseBudget {
    work: usize,
    names: usize,
    layers: usize,
    declarations: usize,
}
impl ParseBudget {
    fn new() -> Self {
        Self {
            work: MAX_STYLE_BYTES * 4,
            names: MAX_LAYER_NAME_BYTES,
            layers: 10_000,
            declarations: 16 * 1024 * 1024,
        }
    }
    fn layer(&mut self, name: &str) -> bool {
        if self.layers == 0 || name.len() > self.names {
            return false;
        }
        self.layers -= 1;
        self.names -= name.len();
        true
    }
}
fn strip_comments(source: &str) -> String {
    let source = if source.len() > 8 * 1024 * 1024 {
        let mut i = 8 * 1024 * 1024;
        while !source.is_char_boundary(i) {
            i -= 1;
        }
        &source[..i]
    } else {
        source
    };
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            out.push(c);
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            out.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            out.push(c);
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for n in chars.by_ref() {
                if prev == '*' && n == '/' {
                    break;
                }
                prev = n;
            }
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}
#[allow(clippy::too_many_arguments)]
fn parse_rules(
    source: &str,
    width: f32,
    height: f32,
    depth: usize,
    layer: &Option<Arc<CascadeLayer>>,
    apply: bool,
    rules: &mut Vec<Rule>,
    budget: &mut ParseBudget,
) {
    if depth > 16 || source.len() > budget.work {
        return;
    }
    budget.work -= source.len();
    let bytes = source.as_bytes();
    let mut start = 0;
    let mut i = 0;
    let mut quote = None;
    let mut escape = false;
    let mut parens = 0;
    while i < bytes.len() && rules.len() < 10_000 {
        let c = bytes[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if c == b'\\' {
            escape = true;
            i += 1;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if c == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i = source[i + 2..]
                .find("*/")
                .map_or(bytes.len(), |end| i + end + 4);
            continue;
        }
        if matches!(c, b'\'' | b'"') {
            quote = Some(c);
            i += 1;
            continue;
        }
        if c == b'(' || c == b'[' {
            parens += 1;
        } else if c == b')' || c == b']' {
            parens -= 1;
        }
        if c == b';' && parens == 0 {
            layer_statement(&source[start..i], layer, rules, budget);
            start = i + 1;
        } else if c == b'{' && parens == 0 {
            let raw_header = rule_start(&source[start..i]);
            let clean_header = strip_comments(raw_header);
            let header = clean_header.trim();
            let body_start = i + 1;
            let mut nesting = 1;
            let mut q = None;
            let mut escaped = false;
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if escaped {
                    escaped = false;
                    i += 1;
                    continue;
                }
                if c == b'\\' {
                    escaped = true;
                    i += 1;
                    continue;
                }
                if let Some(quote) = q {
                    if c == quote {
                        q = None;
                    }
                    i += 1;
                    continue;
                }
                if c == b'/' && bytes.get(i + 1) == Some(&b'*') {
                    i = source[i + 2..]
                        .find("*/")
                        .map_or(bytes.len(), |end| i + end + 4);
                    continue;
                }
                if matches!(c, b'\'' | b'"') {
                    q = Some(c);
                } else if c == b'{' {
                    nesting += 1;
                } else if c == b'}' {
                    nesting -= 1;
                    if nesting == 0 {
                        break;
                    }
                }
                i += 1;
            }
            let body = &source[body_start..i];
            if let Some(media) = at_rule(header, "media") {
                if media_matches_with_budget(media, width, height, &mut budget.work) {
                    parse_rules(body, width, height, depth + 1, layer, apply, rules, budget);
                }
            } else if let Some(supports) = at_rule(raw_header, "supports") {
                if supports_matches_with_budget(supports, false, &mut budget.work) {
                    parse_rules(body, width, height, depth + 1, layer, apply, rules, budget);
                }
            } else if let Some(name) = at_rule(header, "layer") {
                if budget.layer(name) {
                    let child = if name.is_empty() {
                        CascadeLayer::anonymous(layer.clone())
                    } else {
                        CascadeLayer::named(layer.clone(), name)
                    };
                    if let Some(child) = child {
                        rules.push(Rule {
                            selectors: vec![],
                            declarations: vec![],
                            layer: Some(child.clone()),
                        });
                        parse_rules(
                            body,
                            width,
                            height,
                            depth + 1,
                            &Some(child),
                            apply,
                            rules,
                            budget,
                        );
                    }
                }
            } else if at_rule(header, "container").is_some() {
                // Element-dependent conditions always establish global layer
                // order. Container matching itself is not implemented.
                parse_rules(body, width, height, depth + 1, layer, false, rules, budget);
            } else if apply && !header.starts_with('@') && !header.is_empty() {
                let selectors = crate::selectors::parse_list(raw_header, &mut budget.work)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|selector| selector.source)
                    .collect();
                rules.push(Rule {
                    selectors,
                    declarations: parse_declarations_with_limit(
                        &strip_comments(body),
                        &mut budget.declarations,
                    ),
                    layer: layer.clone(),
                });
            }
            start = i.saturating_add(1);
        }
        i += 1;
    }
    if start < source.len() && rules.len() < 10_000 {
        layer_statement(&source[start..], layer, rules, budget);
    }
}
fn layer_statement(
    header: &str,
    layer: &Option<Arc<CascadeLayer>>,
    rules: &mut Vec<Rule>,
    budget: &mut ParseBudget,
) {
    let header = strip_comments(header);
    if let Some(names) = at_rule(header.trim(), "layer") {
        let names = split_top_level(names, ',');
        // An invalid name invalidates the entire order statement.
        if !names.is_empty() && names.iter().all(|name| layer_name(name).is_some()) {
            for name in names {
                if !budget.layer(name) {
                    break;
                }
                if let Some(child) = CascadeLayer::named(layer.clone(), name) {
                    rules.push(Rule {
                        selectors: vec![],
                        declarations: vec![],
                        layer: Some(child),
                    });
                }
                if rules.len() >= 10_000 {
                    break;
                }
            }
        }
    }
}

// Preserve conditional preludes until their own evaluator sees token boundaries.
// Other consumers retain the existing comment normalization behavior.
fn rule_start(mut header: &str) -> &str {
    loop {
        header = media_trim(header);
        if !header.starts_with("/*") {
            return header;
        }
        let Some(end) = header[2..].find("*/") else {
            return "";
        };
        header = &header[end + 4..];
    }
}

fn at_rule<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    let rest = header.strip_prefix('@')?;
    let (keyword, consumed) = css_identifier(rest)?;
    keyword
        .eq_ignore_ascii_case(name)
        .then(|| media_trim(&rest[consumed..]))
}
/// Bounded Media Queries 4 subset. Conditions retain three-valued logic until
/// the query-list boundary; unknown features never become true merely via not.
/// https://drafts.csswg.org/mediaqueries-4/#mq-syntax
pub fn media_matches(query: &str, width: f32, height: f32) -> bool {
    let mut work = MAX_MEDIA_WORK;
    media_matches_with_budget(query, width, height, &mut work)
}
const MAX_MEDIA_BYTES: usize = 65_536;
const MAX_MEDIA_WORK: usize = 2 * 1024 * 1024;
const MAX_MEDIA_TERMS: usize = 64;
const MAX_MEDIA_DEPTH: usize = 16;

fn media_matches_with_budget(query: &str, width: f32, height: f32, work: &mut usize) -> bool {
    if query.len() > MAX_MEDIA_BYTES
        || !width.is_finite()
        || !height.is_finite()
        || width < 0.0
        || height < 0.0
    {
        return false;
    }
    // The public API has its own cap; stylesheet conditions additionally share
    // the parser's remaining work, including metadata from imported sources.
    let available = (*work).min(MAX_MEDIA_WORK);
    let mut evaluator = MediaEvaluator {
        width: f64::from(width),
        height: f64::from(height),
        work: available,
        terms_left: MAX_MEDIA_TERMS,
        exhausted: false,
    };
    let result = evaluator.list(query);
    *work -= available - evaluator.work;
    result && !evaluator.exhausted
}
struct MediaEvaluator {
    width: f64,
    height: f64,
    work: usize,
    terms_left: usize,
    exhausted: bool,
}
impl MediaEvaluator {
    fn spend(&mut self, amount: usize) -> Result<(), ()> {
        if amount > self.work {
            self.work = 0;
            self.exhausted = true;
            return Err(());
        }
        self.work -= amount;
        Ok(())
    }
    fn list(&mut self, query: &str) -> bool {
        if self.spend(query.len().saturating_mul(3) + 1).is_err() {
            return false;
        }
        let clean = strip_comments(query);
        if media_trim(&clean).is_empty() {
            return true;
        }
        let mut brackets = Vec::new();
        let mut start = 0;
        let mut count = 0;
        let mut matched = false;
        let mut at = 0;
        while at < clean.len() {
            let (token, consumed) = media_token(&clean[at..]);
            match token {
                MediaToken::Open(ch) => {
                    if brackets.len() >= MAX_MEDIA_DEPTH {
                        self.exhausted = true;
                        return false;
                    }
                    brackets.push(ch);
                }
                MediaToken::Close(ch) => {
                    let expected = match ch {
                        ')' => '(',
                        ']' => '[',
                        _ => '{',
                    };
                    if brackets.last() == Some(&expected) {
                        brackets.pop();
                    }
                }
                MediaToken::Comma if brackets.is_empty() => {
                    count += 1;
                    if count >= 64 {
                        self.exhausted = true;
                        return false;
                    }
                    matched |= self.query(&clean[start..at]) == Some(true);
                    if self.exhausted {
                        return false;
                    }
                    start = at + consumed;
                }
                _ => {}
            }
            at += consumed;
        }
        // Evaluate every member even after a match, so limits do not depend on
        // truth-value short circuiting or allow trailing oversized conditions.
        let last = self.query(&clean[start..]) == Some(true);
        matched || last
    }
    fn query(&mut self, query: &str) -> Option<bool> {
        self.spend(query.len() + 1).ok()?;
        let mut query = media_trim(query);
        let mut negated = false;
        let mut only = false;
        if let Some(rest) = media_operator(query, "not") {
            negated = true;
            query = rest;
        } else if let Some(rest) = media_operator(query, "only") {
            only = true;
            query = rest;
        }
        let function = media_word(query).is_some_and(|(_, rest)| rest.starts_with('('));
        if query.starts_with('(') || function {
            if only {
                return None;
            }
            if negated {
                let (value, rest) = self.in_parens(query, 0).ok()?;
                return media_trim(rest)
                    .is_empty()
                    .then_some(value)
                    .flatten()
                    .map(|v| !v);
            }
            return self.condition(query, 0, true).ok().flatten();
        }
        let (kind, rest) = media_word(query)?;
        if matches!(kind.as_str(), "not" | "only" | "and" | "or" | "layer") {
            return None;
        }
        let mut result = Some(matches!(kind.as_str(), "screen" | "all"));
        let rest = media_trim(rest);
        if !rest.is_empty() {
            let condition = media_operator(rest, "and")?;
            // A media type permits a condition without top-level `or`.
            result = media_and(result, self.condition(condition, 0, false).ok()?);
        }
        if negated { result.map(|v| !v) } else { result }
    }
    fn condition(&mut self, query: &str, depth: usize, allow_or: bool) -> Result<Option<bool>, ()> {
        self.spend(query.len() + 1)?;
        let query = media_trim(query);
        if let Some(rest) = media_operator(query, "not") {
            let (value, tail) = self.in_parens(rest, depth)?;
            if !media_trim(tail).is_empty() {
                return Err(());
            }
            return Ok(value.map(|v| !v));
        }
        let (mut result, mut rest) = self.in_parens(query, depth)?;
        let mut conjunction = None;
        loop {
            rest = media_trim(rest);
            if rest.is_empty() {
                return Ok(result);
            }
            let (and, next) = if let Some(next) = media_operator(rest, "and") {
                (true, next)
            } else if allow_or && let Some(next) = media_operator(rest, "or") {
                (false, next)
            } else {
                return Err(());
            };
            if conjunction.is_some_and(|previous| previous != and) {
                return Err(());
            }
            conjunction = Some(and);
            let (value, tail) = self.in_parens(next, depth)?;
            result = if and {
                media_and(result, value)
            } else {
                media_or(result, value)
            };
            rest = tail;
        }
    }
    fn in_parens<'a>(
        &mut self,
        query: &'a str,
        depth: usize,
    ) -> Result<(Option<bool>, &'a str), ()> {
        self.spend(query.len() + 1)?;
        let (inner, tail, function) = if query.starts_with('(') {
            let (inner, tail) = media_parentheses(query).ok_or(())?;
            (inner, tail, false)
        } else {
            let (name, rest) = media_word(query).ok_or(())?;
            if name == "url" && !media_url_quoted(rest) {
                // Unquoted url() produces a URL token, not a function token.
                return Err(());
            }
            // Whitespace would make this an identifier followed by a block,
            // which does not match the general-enclosed function production.
            let (inner, tail) = media_parentheses(rest).ok_or(())?;
            (inner, tail, true)
        };
        if depth >= MAX_MEDIA_DEPTH {
            self.exhausted = true;
            return Err(());
        }
        if !function {
            if let Ok(value) = self.condition(inner, depth + 1, true) {
                return Ok((value, tail));
            }
            if self.exhausted {
                return Err(());
            }
        }
        if self.terms_left == 0 {
            self.exhausted = true;
            return Err(());
        }
        self.terms_left -= 1;
        self.spend(inner.len() + 1)?;
        // Unknown functions and parenthesized future syntax stay unknown.
        let result = if function {
            None
        } else {
            media_feature(inner, self.width, self.height)
        };
        Ok((result, tail))
    }
}
fn media_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n' | '\x0c')
}
fn media_trim(value: &str) -> &str {
    value.trim_matches(media_space)
}
fn media_word(value: &str) -> Option<(String, &str)> {
    let (word, consumed) = css_identifier(value)?;
    Some((word.to_ascii_lowercase(), &value[consumed..]))
}
fn media_name(value: &str) -> Option<String> {
    let (word, rest) = media_word(media_trim(value))?;
    rest.is_empty().then_some(word)
}
fn media_operator<'a>(value: &'a str, operator: &str) -> Option<&'a str> {
    let (word, rest) = media_word(value)?;
    (word == operator && rest.chars().next().is_some_and(media_space)).then(|| media_trim(rest))
}
fn media_and(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}
fn media_or(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}
fn media_parentheses(query: &str) -> Option<(&str, &str)> {
    let query = query.strip_prefix('(')?;
    let mut brackets = vec!['('];
    let mut at = 0;
    while at < query.len() {
        let (token, consumed) = media_token(&query[at..]);
        match token {
            MediaToken::Open(ch) => {
                if brackets.len() >= MAX_MEDIA_DEPTH {
                    return None;
                }
                brackets.push(ch);
            }
            MediaToken::Close(ch) => {
                if brackets.pop()?
                    != match ch {
                        ')' => '(',
                        ']' => '[',
                        _ => '{',
                    }
                {
                    return None;
                }
                if brackets.is_empty() {
                    return Some((&query[..at], &query[at + consumed..]));
                }
            }
            MediaToken::Bad => return None,
            _ => {}
        }
        at += consumed;
    }
    None
}
#[derive(PartialEq)]
enum MediaToken {
    Other,
    Open(char),
    Close(char),
    Comma,
    Bad,
}
// This cursor recognizes the token boundaries needed by media conditions. It
// does not retain token values or fetch URLs. In particular, punctuation inside
// an unquoted URL is data, and bad URL remnants consume through their own `)`.
fn media_token(source: &str) -> (MediaToken, usize) {
    use crate::selectors::Kind;
    let (kind, count) = crate::selectors::token(source);
    (
        match kind {
            Kind::Function => MediaToken::Open('('),
            Kind::Open(ch) => MediaToken::Open(ch),
            Kind::Close(ch) => MediaToken::Close(ch),
            Kind::Comma => MediaToken::Comma,
            Kind::Bad => MediaToken::Bad,
            _ => MediaToken::Other,
        },
        count,
    )
}
use crate::selectors::{number_end as media_number_end, url_quoted as media_url_quoted};
#[derive(Clone, Copy, PartialEq)]
enum MediaComparison {
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
}
impl MediaComparison {
    fn compare(self, left: f64, right: f64) -> bool {
        match self {
            Self::Less => left < right,
            Self::LessEqual => left <= right,
            Self::Equal => left == right,
            Self::GreaterEqual => left >= right,
            Self::Greater => left > right,
        }
    }
    fn direction(self) -> i8 {
        match self {
            Self::Less | Self::LessEqual => -1,
            Self::Greater | Self::GreaterEqual => 1,
            Self::Equal => 0,
        }
    }
}
fn media_dimension(name: &str, width: f64, height: f64) -> Option<f64> {
    match media_name(name)?.as_str() {
        "width" => Some(width),
        "height" => Some(height),
        _ => None,
    }
}
fn media_range(feature: &str, width: f64, height: f64) -> Option<bool> {
    let mut operators = [(0, 0, MediaComparison::Equal); 2];
    let mut count = 0;
    let bytes = feature.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        let ch = bytes[at];
        if matches!(ch, b'<' | b'>' | b'=') {
            if count == operators.len() {
                return None;
            }
            let equal = ch != b'=' && bytes.get(at + 1) == Some(&b'=');
            let comparison = match (ch, equal) {
                (b'<', false) => MediaComparison::Less,
                (b'<', true) => MediaComparison::LessEqual,
                (b'>', false) => MediaComparison::Greater,
                (b'>', true) => MediaComparison::GreaterEqual,
                _ => MediaComparison::Equal,
            };
            let end = at + 1 + usize::from(equal);
            operators[count] = (at, end, comparison);
            count += 1;
            at = end;
        } else {
            at += 1;
        }
    }
    let (start, end, comparison) = operators[0];
    let left = media_trim(&feature[..start]);
    if count == 1 {
        let right = media_trim(&feature[end..]);
        if let Some(actual) = media_dimension(left, width, height) {
            return Some(comparison.compare(actual, media_length(right, width, height)?));
        }
        return Some(comparison.compare(
            media_length(left, width, height)?,
            media_dimension(right, width, height)?,
        ));
    }
    if count != 2 {
        return None;
    }
    let (second_start, second_end, second) = operators[1];
    if comparison.direction() == 0 || comparison.direction() != second.direction() {
        return None;
    }
    let actual = media_dimension(&feature[end..second_start], width, height)?;
    let lower = media_length(left, width, height)?;
    let upper = media_length(media_trim(&feature[second_end..]), width, height)?;
    Some(comparison.compare(lower, actual) && second.compare(actual, upper))
}
fn media_length(value: &str, width: f64, height: f64) -> Option<f64> {
    // Consume exactly one CSS number/dimension. Rust float parsing alone also
    // accepts forms such as `1.` that CSS tokenizes as a number plus delimiter.
    let at = media_number_end(value)?;
    let number = value[..at].parse::<f64>().ok()?;
    if !number.is_finite() || number.abs() > 1_000_000.0 {
        return None;
    }
    if at == value.len() {
        return (number == 0.0).then_some(number);
    }
    let (unit, rest) = media_word(&value[at..])?;
    if !rest.is_empty() {
        return None;
    }
    let scale = match unit.as_str() {
        "px" => 1.0,
        "em" | "rem" | "pc" => 16.0,
        "ex" | "ch" => 8.0,
        "vw" | "dvw" | "svw" | "lvw" => width / 100.0,
        "vh" | "dvh" | "svh" | "lvh" => height / 100.0,
        "vmin" => width.min(height) / 100.0,
        "vmax" => width.max(height) / 100.0,
        "pt" => 96.0 / 72.0,
        "in" => 96.0,
        "cm" => 96.0 / 2.54,
        "mm" => 96.0 / 25.4,
        "q" => 96.0 / 101.6,
        _ => return None,
    };
    let result = number * scale;
    (result.is_finite() && result.abs() <= 1_000_000.0).then_some(result)
}
fn media_feature(feature: &str, width: f64, height: f64) -> Option<bool> {
    let feature = media_trim(feature);
    let Some((name, value)) = feature.split_once(':') else {
        if let Some(name) = media_name(feature) {
            return match name.as_str() {
                "color" | "hover" | "any-hover" | "pointer" | "any-pointer" => Some(true),
                "width" => Some(width > 0.0),
                "height" => Some(height > 0.0),
                _ => None,
            };
        }
        return media_range(feature, width, height);
    };
    let name = media_name(name)?;
    let value = media_trim(value);
    match name.as_str() {
        "min-width" | "max-width" | "width" | "min-height" | "max-height" | "height" => {
            let n = media_length(value, width, height)?;
            let actual = if name.ends_with("height") {
                height
            } else {
                width
            };
            Some(if name.starts_with("min-") {
                actual >= n
            } else if name.starts_with("max-") {
                actual <= n
            } else {
                actual == n
            })
        }
        "orientation" => match media_name(value)?.as_str() {
            "landscape" => Some(width > height),
            "portrait" => Some(height >= width),
            _ => None,
        },
        "prefers-color-scheme" => match media_name(value)?.as_str() {
            "light" => Some(true),
            "dark" => Some(false),
            _ => None,
        },
        "prefers-reduced-motion" => match media_name(value)?.as_str() {
            "reduce" => Some(true),
            "no-preference" => Some(false),
            _ => None,
        },
        "hover" | "any-hover" => match media_name(value)?.as_str() {
            "hover" => Some(true),
            "none" => Some(false),
            _ => None,
        },
        "pointer" | "any-pointer" => match media_name(value)?.as_str() {
            "fine" => Some(true),
            "coarse" | "none" => Some(false),
            _ => None,
        },
        "display-mode" => match media_name(value)?.as_str() {
            "browser" => Some(true),
            "fullscreen" | "standalone" | "minimal-ui" | "picture-in-picture" => Some(false),
            _ => None,
        },
        _ => None,
    }
}
/// A conservative, bounded CSS Conditional Rules 3/4 capability query.
/// This is an @supports condition, not the CSS.supports() implied-declaration API.
pub fn supports_matches(query: &str) -> bool {
    let mut work = MAX_SUPPORTS_WORK;
    supports_matches_with_budget(query, false, &mut work)
}
const MAX_SUPPORTS_BYTES: usize = 16 * 1024;
const MAX_SUPPORTS_WORK: usize = 256 * 1024;
const MAX_SUPPORTS_TERMS: usize = 64;

pub(crate) fn supports_matches_with_budget(
    query: &str,
    implied_declaration: bool,
    work: &mut usize,
) -> bool {
    if query.len() > MAX_SUPPORTS_BYTES {
        return false;
    }
    let available = (*work).min(MAX_SUPPORTS_WORK);
    let mut evaluator = SupportsEvaluator {
        work: available,
        terms: MAX_SUPPORTS_TERMS,
        exhausted: false,
    };
    let result = (|| {
        evaluator.spend(query.len().saturating_mul(3) + 1)?;
        let tokens = crate::selectors::tokens(query, &mut evaluator.work).map_err(|_| ())?;
        evaluator.tokens(&tokens)?;
        let clean = strip_comments(query);
        if implied_declaration && supports_declaration_parts(media_trim(&clean)).is_some() {
            evaluator.declaration(media_trim(&clean))
        } else {
            evaluator.condition(query, &tokens, 0)
        }
    })();
    *work -= available - evaluator.work;
    result == Ok(true) && !evaluator.exhausted
}
struct SupportsEvaluator {
    work: usize,
    terms: usize,
    exhausted: bool,
}
impl SupportsEvaluator {
    fn spend(&mut self, amount: usize) -> Result<(), ()> {
        if amount > self.work {
            self.work = 0;
            self.exhausted = true;
            return Err(());
        }
        self.work -= amount;
        Ok(())
    }
    fn term(&mut self) -> Result<(), ()> {
        if self.terms == 0 {
            self.exhausted = true;
            return Err(());
        }
        self.terms -= 1;
        Ok(())
    }
    fn tokens(&mut self, tokens: &[crate::selectors::Token<'_>]) -> Result<(), ()> {
        use crate::selectors::Kind;
        self.spend(tokens.len() + 1)?;
        let mut stack = Vec::new();
        for token in tokens {
            if let Some(open) = token.open() {
                if stack.len() >= MAX_MEDIA_DEPTH {
                    self.exhausted = true;
                    return Err(());
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
                    return Err(());
                }
            } else if token.kind == Kind::Bad {
                return Err(());
            }
        }
        if stack.is_empty() { Ok(()) } else { Err(()) }
    }
    fn condition(
        &mut self,
        source: &str,
        tokens: &[crate::selectors::Token<'_>],
        depth: usize,
    ) -> Result<bool, ()> {
        use crate::selectors::{Kind, trim};
        self.spend(tokens.len() + 1)?;
        if depth >= MAX_MEDIA_DEPTH {
            self.exhausted = true;
            return Err(());
        }
        let tokens = trim(tokens);
        if tokens
            .first()
            .is_some_and(|t| t.kind == Kind::Ident && t.is_name("not"))
        {
            let rest = trim(&tokens[1..]);
            let (value, consumed) = self.leaf(source, rest, depth)?;
            return if trim(&rest[consumed..]).is_empty() {
                Ok(!value)
            } else {
                Err(())
            };
        }
        let (mut value, consumed) = self.leaf(source, tokens, depth)?;
        let mut rest = trim(&tokens[consumed..]);
        let mut operator = None;
        while !rest.is_empty() {
            let token = &rest[0];
            let and = if token.kind == Kind::Ident && token.is_name("and") {
                true
            } else if token.kind == Kind::Ident && token.is_name("or") {
                false
            } else {
                return Err(());
            };
            if operator.is_some_and(|old| old != and) {
                return Err(());
            }
            operator = Some(and);
            rest = trim(&rest[1..]);
            let (right, consumed) = self.leaf(source, rest, depth)?;
            value = if and { value & right } else { value | right };
            rest = trim(&rest[consumed..]);
        }
        Ok(value)
    }
    fn leaf(
        &mut self,
        source: &str,
        tokens: &[crate::selectors::Token<'_>],
        depth: usize,
    ) -> Result<(bool, usize), ()> {
        use crate::selectors::{Kind, closing, trim};
        self.spend(tokens.len() + 1)?;
        let first = tokens.first().ok_or(())?;
        if !matches!(first.kind, Kind::Open('(') | Kind::Function) {
            return Err(());
        }
        let end = closing(tokens, 0, MAX_MEDIA_DEPTH, &mut self.work).map_err(|_| ())?;
        let inner = trim(&tokens[1..end]);
        let raw = &source[first.end..tokens[end].start];
        if first.kind == Kind::Open('(') {
            let clean = strip_comments(raw);
            let clean = media_trim(&clean);
            if supports_declaration_parts(clean).is_some() {
                return Ok((self.declaration(clean)?, end + 1));
            }
            if inner.first().is_some_and(|t| {
                matches!(t.kind, Kind::Open('(') | Kind::Function)
                    || t.kind == Kind::Ident && t.is_name("not")
            }) {
                return Ok((self.condition(source, inner, depth + 1)?, end + 1));
            }
            self.term()?;
            return Ok((false, end + 1));
        }
        self.term()?;
        let value = if first.is_name("selector") {
            crate::selectors::supports(raw, &mut self.work).map_err(|_| ())?
        } else {
            false
        };
        Ok((value, end + 1))
    }
    fn declaration(&mut self, source: &str) -> Result<bool, ()> {
        self.term()?;
        self.spend(source.len().saturating_mul(8) + 1)?;
        let Some((name, mut value)) = supports_declaration_parts(source) else {
            return Ok(false);
        };
        if name.len() > 256 || value.len() > 4096 {
            return Ok(false);
        }
        if let Some((before, important)) = value.rsplit_once('!') {
            if !media_trim(important).eq_ignore_ascii_case("important") {
                return Ok(false);
            }
            value = media_trim(before);
        }
        if value.is_empty() || value.contains([';', '!', '{', '}']) {
            return Ok(false);
        }
        // Escaped property/value identifiers are not decoded by the actual
        // declaration application path. Do not claim that spelling works.
        if name.contains('\\') || value.contains('\\') {
            return Ok(false);
        }
        Ok(supports_property(&name.to_ascii_lowercase(), value))
    }
}
fn supports_declaration_parts(source: &str) -> Option<(&str, &str)> {
    let (_, end) = css_identifier(source)?;
    let rest = media_trim(&source[end..]);
    Some((&source[..end], media_trim(rest.strip_prefix(':')?)))
}
fn supports_number(value: &str) -> Option<f32> {
    (media_number_end(value)? == value.len())
        .then(|| finite_number(value))
        .flatten()
}
fn supports_length(value: &str, percent: bool, negative: bool) -> bool {
    let Some(end) = media_number_end(value) else {
        return false;
    };
    let Some(number) = supports_number(&value[..end]) else {
        return false;
    };
    if !negative && number < 0.0 {
        return false;
    }
    let unit = &value[end..];
    (unit.is_empty() && number == 0.0
        || percent && unit == "%"
        || matches!(
            unit.to_ascii_lowercase().as_str(),
            "px" | "em"
                | "rem"
                | "ex"
                | "ch"
                | "vw"
                | "vh"
                | "vmin"
                | "vmax"
                | "dvw"
                | "dvh"
                | "svw"
                | "svh"
                | "lvw"
                | "lvh"
                | "pt"
                | "pc"
                | "in"
                | "cm"
                | "mm"
                | "q"
        ))
        && parse_length(value, 16.0, 16.0, 800.0, 600.0).is_some()
}
fn supports_color(value: &str) -> bool {
    if value.eq_ignore_ascii_case("currentcolor") {
        return true;
    }
    let lower = value.to_ascii_lowercase();
    if !lower.contains('(') {
        return parse_color(value).is_some();
    }
    let Some((name, inner)) = lower.split_once('(') else {
        return false;
    };
    let Some(inner) = inner.strip_suffix(')') else {
        return false;
    };
    if !matches!(name, "rgb" | "rgba" | "hsl" | "hsla") {
        return false;
    }
    let (channels, alpha) = if inner.contains(',') {
        if inner.contains('/') {
            return false;
        }
        let parts: Vec<_> = inner.split(',').map(media_trim).collect();
        if !(3..=4).contains(&parts.len()) {
            return false;
        }
        (parts[..3].to_vec(), parts.get(3).copied())
    } else {
        let (channels, alpha) = inner
            .split_once('/')
            .map_or((inner, None), |(c, a)| (c, Some(media_trim(a))));
        let parts: Vec<_> = channels.split_ascii_whitespace().collect();
        if parts.len() != 3 {
            return false;
        }
        (parts, alpha)
    };
    let number = |v: &str| supports_number(v.strip_suffix('%').unwrap_or(v)).is_some();
    if alpha.is_some_and(|a| !number(a)) {
        return false;
    }
    let valid = if name.starts_with("rgb") {
        channels.iter().all(|v| number(v))
            && (!inner.contains(',')
                || channels.iter().all(|v| v.ends_with('%'))
                || channels.iter().all(|v| !v.ends_with('%')))
    } else {
        let hue = channels[0]
            .strip_suffix("deg")
            .or_else(|| channels[0].strip_suffix("rad"))
            .or_else(|| channels[0].strip_suffix("turn"))
            .unwrap_or(channels[0]);
        supports_number(hue).is_some()
            && channels[1..].iter().all(|v| v.ends_with('%') && number(v))
    };
    valid && parse_color(value).is_some()
}
fn supports_property(name: &str, value: &str) -> bool {
    let known = matches!(
        name,
        "display"
            | "float"
            | "clear"
            | "position"
            | "box-sizing"
            | "flex-direction"
            | "flex-wrap"
            | "width"
            | "height"
            | "min-width"
            | "min-height"
            | "max-width"
            | "max-height"
            | "flex-basis"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "inset"
            | "margin"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
            | "padding"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "color"
            | "background-color"
            | "background"
            | "opacity"
            | "order"
            | "z-index"
            | "flex-grow"
            | "flex-shrink"
            | "grid-template-columns"
            | "grid-template-rows"
            | "grid-auto-columns"
            | "grid-auto-rows"
            | "grid-column-start"
            | "grid-column-end"
            | "grid-row-start"
            | "grid-row-end"
            | "grid-auto-flow"
    );
    if !known {
        return false;
    }
    if css_wide(value) {
        return true;
    }
    match name {
        // These spellings are exactly those currently consumed by apply_property.
        "display" => matches!(
            value,
            "none" | "block" | "flow-root" | "inline" | "inline-block" | "flex" | "grid"
        ),
        "float" => matches!(
            value.to_ascii_lowercase().as_str(),
            "none" | "left" | "right"
        ),
        "clear" => matches!(
            value.to_ascii_lowercase().as_str(),
            "none" | "left" | "right" | "both"
        ),
        "position" => matches!(
            value.to_ascii_lowercase().as_str(),
            "static" | "relative" | "absolute" | "fixed"
        ),
        "box-sizing" => matches!(value, "content-box" | "border-box"),
        "flex-direction" => matches!(value, "row" | "row-reverse" | "column" | "column-reverse"),
        "flex-wrap" => matches!(value, "nowrap" | "wrap" | "wrap-reverse"),
        "color" | "background-color" => supports_color(value),
        "background" => {
            supports_color(value)
                && (!value.eq_ignore_ascii_case("currentcolor") || value == "currentcolor")
        }
        "opacity" => supports_number(value).is_some(),
        "flex-grow" | "flex-shrink" => supports_number(value).is_some_and(|v| v >= 0.0),
        "order" | "z-index" => {
            name == "z-index" && value.eq_ignore_ascii_case("auto") || supports_integer(value)
        }
        "grid-auto-flow" => valid_grid_auto_flow(value),
        "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
            let lower = value.to_ascii_lowercase();
            let parts: Vec<_> = lower.split_ascii_whitespace().collect();
            let valid = lower == "auto"
                || supports_integer(&lower)
                || parts.len() == 2
                    && parts.contains(&"span")
                    && parts.iter().any(|v| supports_integer(v));
            valid && parse_grid_line(value).is_some()
        }
        "grid-template-columns" | "grid-template-rows" | "grid-auto-columns" | "grid-auto-rows" => {
            let lower = value.to_ascii_lowercase();
            (!name.starts_with("grid-auto-") || lower != "none" && !lower.contains("repeat("))
                && supports_tracks(&lower, 0)
                && parse_grid_tracks(value, 16.0, 16.0, 800.0, 600.0).is_some()
        }
        _ => {
            let spacing = name.starts_with("margin")
                || name.starts_with("padding")
                || matches!(name, "inset" | "top" | "right" | "bottom" | "left");
            let signed = spacing && !name.starts_with("padding");
            let auto = signed
                || matches!(
                    name,
                    "width" | "height" | "min-width" | "min-height" | "flex-basis"
                );
            let none = matches!(name, "max-width" | "max-height");
            let gap = matches!(name, "gap" | "row-gap" | "column-gap");
            let count = if matches!(name, "margin" | "padding" | "inset") {
                4
            } else if name == "gap" {
                2
            } else {
                1
            };
            let parts: Vec<_> = value.split_ascii_whitespace().collect();
            !parts.is_empty()
                && parts.len() <= count
                && parts.iter().all(|v| {
                    auto && v.eq_ignore_ascii_case("auto")
                        || none && v.eq_ignore_ascii_case("none")
                        || gap && v.eq_ignore_ascii_case("normal")
                        || supports_length(v, true, signed)
                })
        }
    }
}
fn supports_integer(value: &str) -> bool {
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) && value.parse::<i32>().is_ok()
}
fn supports_tracks(value: &str, depth: usize) -> bool {
    fn count(value: &str, depth: usize) -> Option<usize> {
        if depth > 1 {
            return None;
        }
        if depth == 0 && value == "none" {
            return Some(0);
        }
        let tokens = words(value);
        if tokens.is_empty() || tokens.len() > 64 {
            return None;
        }
        let breadth = |value: &str, fraction: bool| {
            matches!(value, "auto" | "min-content" | "max-content")
                || supports_length(value, true, false)
                || fraction
                    && value
                        .strip_suffix("fr")
                        .and_then(supports_number)
                        .is_some_and(|v| v >= 0.0)
        };
        let mut total = 0usize;
        for v in tokens {
            let tracks = if let Some(inner) =
                v.strip_prefix("repeat(").and_then(|v| v.strip_suffix(')'))
            {
                let args = split_top_level(inner, ',');
                if depth != 0 || args.len() != 2 || !supports_integer(args[0]) {
                    return None;
                }
                let repeat = args[0].parse::<usize>().ok()?;
                if !(1..=64).contains(&repeat) {
                    return None;
                }
                repeat.checked_mul(count(args[1], depth + 1)?)?
            } else if let Some(inner) = v.strip_prefix("minmax(").and_then(|v| v.strip_suffix(')'))
            {
                let args = split_top_level(inner, ',');
                if args.len() != 2 || !breadth(args[0], false) || !breadth(args[1], true) {
                    return None;
                }
                1
            } else {
                if !breadth(v, true) {
                    return None;
                }
                1
            };
            total = total.checked_add(tracks)?;
            if total > 64 {
                return None;
            }
        }
        Some(total)
    }
    count(value, depth).is_some()
}
pub fn parse_declarations(source: &str) -> Vec<Declaration> {
    parse_declarations_with_limit(&strip_comments(source), &mut (1024 * 1024))
}
fn parse_declarations_with_limit(source: &str, bytes_left: &mut usize) -> Vec<Declaration> {
    if source.len() > MAX_STYLE_BYTES {
        return Vec::new();
    }
    let mut declarations = vec![];
    for part in split_top_level(source, ';').into_iter().take(4096) {
        if declarations.len() >= 8192 {
            break;
        }
        let Some((name, value)) = part.split_once(':') else {
            continue;
        };
        let name = if name.trim().starts_with("--") {
            name.trim().to_owned()
        } else {
            name.trim().to_ascii_lowercase()
        };
        let mut value = value.trim();
        if name.is_empty() || value.is_empty() || name.len() > 256 || value.len() > 4096 {
            continue;
        }
        let lower = value.to_ascii_lowercase();
        let important = lower
            .rfind('!')
            .is_some_and(|i| lower[i + 1..].trim() == "important");
        if important {
            value = value[..value.rfind('!').unwrap_or(value.len())].trim_end();
        }
        if matches!(name.as_str(), "float" | "clear") && !value.contains("var(") {
            let keyword = value.to_ascii_lowercase();
            if !(css_wide(value)
                || matches!(
                    keyword.as_str(),
                    "none"
                        | "left"
                        | "right"
                        | "inherit"
                        | "initial"
                        | "unset"
                        | "revert"
                        | "revert-layer"
                )
                || name == "clear" && keyword == "both")
            {
                continue;
            }
        }
        if !value.contains("var(") && !css_wide(value) {
            let valid = match name.as_str() {
                "grid-template-columns"
                | "grid-template-rows"
                | "grid-auto-columns"
                | "grid-auto-rows" => parse_grid_tracks(value, 16.0, 16.0, 800.0, 600.0)
                    .is_some_and(|tracks| !name.starts_with("grid-auto-") || !tracks.is_empty()),
                "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
                    parse_grid_line(value).is_some()
                }
                "grid-auto-flow" => valid_grid_auto_flow(value),
                "z-index" => value.eq_ignore_ascii_case("auto") || value.parse::<i32>().is_ok(),
                "position" => matches!(
                    value.to_ascii_lowercase().as_str(),
                    "static" | "relative" | "absolute" | "fixed" | "sticky"
                ),
                _ => true,
            };
            if !valid {
                continue;
            }
        }
        let start = declarations.len();
        expand_declaration(&name, value, important, &mut declarations);
        let cost: usize = declarations[start..]
            .iter()
            .map(|decl| decl.name.len() + decl.value.len())
            .sum();
        if cost > *bytes_left || declarations.len() > 8192 {
            declarations.truncate(start);
            break;
        }
        *bytes_left -= cost;
    }
    declarations
}
fn words(source: &str) -> Vec<&str> {
    let mut result = vec![];
    let mut start = None;
    let mut depth = 0;
    let mut quote = None;
    let mut escape = false;
    for (i, c) in source.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            start.get_or_insert(i);
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            start.get_or_insert(i);
            continue;
        }
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
        }
        if c.is_ascii_whitespace() && depth == 0 {
            if let Some(s) = start.take() {
                result.push(&source[s..i]);
            }
        } else {
            start.get_or_insert(i);
        }
    }
    if let Some(s) = start {
        result.push(&source[s..]);
    }
    result
}
fn wide_keyword(value: &str) -> Option<String> {
    // Recognize unescaped keywords directly, avoiding temporary identifier and
    // lowercase strings for ordinary values. Escapes still need token decoding.
    if !value.contains('\\') {
        return ["inherit", "initial", "unset", "revert", "revert-layer"]
            .into_iter()
            .find(|word| value.eq_ignore_ascii_case(word))
            .map(str::to_owned);
    }
    let (word, consumed) = css_identifier(value)?;
    if consumed != value.len() {
        return None;
    }
    let word = word.to_ascii_lowercase();
    matches!(
        word.as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
    .then_some(word)
}
fn css_wide(value: &str) -> bool {
    wide_keyword(value).is_some()
}

fn shorthand_properties(name: &str) -> Option<Vec<String>> {
    let list: &[&str] = match name {
        "all" => SUPPORTED_PROPERTIES,
        "gap" | "grid-gap" => &["row-gap", "column-gap"],
        "grid-row" => &["grid-row-start", "grid-row-end"],
        "grid-column" => &["grid-column-start", "grid-column-end"],
        "grid-area" => &[
            "grid-row-start",
            "grid-column-start",
            "grid-row-end",
            "grid-column-end",
        ],
        "place-items" => &["align-items", "justify-items"],
        "place-self" => &["align-self", "justify-self"],
        "place-content" => &["align-content", "justify-content"],
        "inset" => &["top", "right", "bottom", "left"],
        "background" => &["background-color"],
        "list-style" => &["list-style-type", "list-style-position"],
        "flex" => &["flex-grow", "flex-shrink", "flex-basis"],
        "flex-flow" => &["flex-direction", "flex-wrap"],
        "font" => &[
            "font-size",
            "font-family",
            "font-weight",
            "font-style",
            "line-height",
        ],
        "margin" | "padding" | "border-width" | "border-color" | "border-style" => {
            return Some(
                ["top", "right", "bottom", "left"]
                    .iter()
                    .map(|side| {
                        if let Some(suffix) = name.strip_prefix("border-") {
                            format!("border-{side}-{suffix}")
                        } else {
                            format!("{name}-{side}")
                        }
                    })
                    .collect(),
            );
        }
        "margin-inline" | "padding-inline" | "margin-block" | "padding-block" => {
            let prefix = name.split('-').next()?;
            let sides = if name.ends_with("inline") {
                ["left", "right"]
            } else {
                ["top", "bottom"]
            };
            return Some(
                sides
                    .into_iter()
                    .map(|side| format!("{prefix}-{side}"))
                    .collect(),
            );
        }
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let sides: &[&str] = if name == "border" {
                &["top", "right", "bottom", "left"]
            } else {
                &[name.strip_prefix("border-")?]
            };
            return Some(
                sides
                    .iter()
                    .flat_map(|side| {
                        ["width", "style", "color"]
                            .into_iter()
                            .map(move |part| format!("border-{side}-{part}"))
                    })
                    .collect(),
            );
        }
        _ => return None,
    };
    Some(list.iter().map(|name| (*name).into()).collect())
}
fn expand_declaration(name: &str, value: &str, important: bool, out: &mut Vec<Declaration>) {
    if name == "list-style" && !css_wide(value) && !value.contains("var(") {
        let lower = value.to_ascii_lowercase();
        let parts = words(&lower);
        let mut kind = None;
        let mut position = None;
        if parts.is_empty() || parts.len() > 2 {
            return;
        }
        for part in parts {
            if matches!(part, "inside" | "outside") {
                if position.replace(part).is_some() {
                    return;
                }
            } else if matches!(
                part,
                "none"
                    | "disc"
                    | "circle"
                    | "square"
                    | "decimal"
                    | "decimal-leading-zero"
                    | "disclosure-open"
                    | "disclosure-closed"
            ) {
                if kind.replace(part).is_some() {
                    return;
                }
            } else {
                return;
            }
        }
        for (name, value) in [
            ("list-style-type", kind.unwrap_or("disc")),
            ("list-style-position", position.unwrap_or("outside")),
        ] {
            out.push(Declaration {
                name: name.into(),
                value: value.into(),
                important,
                pending_shorthand: None,
            });
        }
        return;
    }
    let global = css_wide(value);
    let has_vars = value.contains("var(");
    if let Some(properties) = shorthand_properties(name) {
        if global || has_vars {
            let pending = has_vars.then(|| Arc::<str>::from(name));
            for name in properties {
                out.push(Declaration {
                    name,
                    value: if global {
                        wide_keyword(value).unwrap_or_default()
                    } else {
                        value.into()
                    },
                    important,
                    pending_shorthand: pending.clone(),
                });
            }
            return;
        }
        if name == "all" || words(value).iter().any(|word| css_wide(word)) {
            return;
        }
    }
    let global_value = wide_keyword(value);
    let value = global_value.as_deref().unwrap_or(value);
    let normalized = (!value.to_ascii_lowercase().contains("var(")
        && (name.starts_with("grid-")
            || name.starts_with("place-")
            || matches!(name, "gap" | "row-gap" | "column-gap")))
    .then(|| value.to_ascii_lowercase());
    let value = normalized.as_deref().unwrap_or(value);
    let add = |out: &mut Vec<Declaration>, name: &str, value: &str| {
        out.push(Declaration {
            name: name.into(),
            value: value.into(),
            important,
            pending_shorthand: None,
        })
    };
    if matches!(name, "gap" | "grid-gap") {
        let values = words(value);
        if (1..=2).contains(&values.len()) {
            if !value.contains("var(") && values.iter().any(|part| !matches!(*part,"normal"|"inherit"|"initial"|"unset"|"revert") && !matches!(parse_length(part,16.0,16.0,800.0,600.0),Some(Length::Px(n)|Length::Percent(n)) if n>=0.0)) { return; }
            if values.len() == 2
                && values
                    .iter()
                    .any(|part| matches!(*part, "inherit" | "initial" | "unset" | "revert"))
            {
                return;
            }
            add(out, "row-gap", values[0]);
            add(out, "column-gap", values.get(1).unwrap_or(&values[0]));
        }
    } else if matches!(name, "grid-row" | "grid-column" | "grid-area") {
        let values = split_top_level(value, '/');
        let count = if name == "grid-area" { 4 } else { 2 };
        if values.is_empty() || values.len() > count {
            return;
        }
        let global = matches!(value, "inherit" | "initial" | "unset" | "revert");
        if !global
            && !value.contains("var(")
            && values.iter().any(|v| parse_grid_line(v.trim()).is_none())
        {
            return;
        }
        let names: &[&str] = match name {
            "grid-row" => &["grid-row-start", "grid-row-end"],
            "grid-column" => &["grid-column-start", "grid-column-end"],
            _ => &[
                "grid-row-start",
                "grid-column-start",
                "grid-row-end",
                "grid-column-end",
            ],
        };
        for (i, name) in names.iter().enumerate() {
            add(
                out,
                name,
                if global {
                    value
                } else {
                    values.get(i).map(|v| v.trim()).unwrap_or("auto")
                },
            );
        }
    } else if matches!(name, "place-items" | "place-self" | "place-content") {
        let values = words(value);
        if (1..=2).contains(&values.len()) {
            let suffix = name.strip_prefix("place-").unwrap_or("items");
            add(out, &format!("align-{suffix}"), values[0]);
            add(
                out,
                &format!("justify-{suffix}"),
                values.get(1).unwrap_or(&values[0]),
            );
        }
    } else if matches!(
        name,
        "margin" | "padding" | "border-width" | "border-color" | "border-style" | "inset"
    ) {
        let values = words(value);
        if values.is_empty() || values.len() > 4 {
            return;
        }
        let sides = [
            values[0],
            *values.get(1).unwrap_or(&values[0]),
            *values.get(2).unwrap_or(&values[0]),
            *values
                .get(3)
                .or_else(|| values.get(1))
                .unwrap_or(&values[0]),
        ];
        for (side, value) in ["top", "right", "bottom", "left"].into_iter().zip(sides) {
            let prop = if name == "inset" {
                side.into()
            } else if let Some(suffix) = name.strip_prefix("border-") {
                format!("border-{side}-{suffix}")
            } else {
                format!("{name}-{side}")
            };
            add(out, &prop, value);
        }
    } else if matches!(
        name,
        "margin-inline" | "padding-inline" | "margin-block" | "padding-block"
    ) {
        let values = words(value);
        if values.is_empty() || values.len() > 2 {
            return;
        }
        let prefix = name.split('-').next().unwrap_or("");
        let sides = if name.ends_with("inline") {
            ["left", "right"]
        } else {
            ["top", "bottom"]
        };
        for (i, side) in sides.into_iter().enumerate() {
            add(
                out,
                &format!("{prefix}-{side}"),
                values.get(i).unwrap_or(&values[0]),
            );
        }
    } else if matches!(
        name,
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left"
    ) {
        let mut width = "medium";
        let mut style = "none";
        let mut color = "currentcolor";
        for word in words(value) {
            if matches!(
                word,
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
            ) {
                style = word;
            } else if matches!(word, "thin" | "medium" | "thick")
                || parse_length(word, 16.0, 16.0, 800.0, 600.0).is_some()
            {
                width = word;
            } else {
                color = word;
            }
        }
        let sides: Vec<&str> = if name == "border" {
            vec!["top", "right", "bottom", "left"]
        } else {
            vec![name.strip_prefix("border-").unwrap_or("top")]
        };
        for side in sides {
            add(out, &format!("border-{side}-width"), width);
            add(out, &format!("border-{side}-style"), style);
            add(out, &format!("border-{side}-color"), color);
        }
    } else if name == "background" {
        let candidate = words(value)
            .into_iter()
            .find(|v| *v == "currentcolor" || v.starts_with("var(") || parse_color(v).is_some())
            .unwrap_or("transparent");
        add(out, "background-color", candidate);
    } else if name == "flex" {
        match value {
            "none" => {
                add(out, "flex-grow", "0");
                add(out, "flex-shrink", "0");
                add(out, "flex-basis", "auto");
            }
            "auto" => {
                add(out, "flex-grow", "1");
                add(out, "flex-shrink", "1");
                add(out, "flex-basis", "auto");
            }
            _ => {
                let values = words(value);
                add(out, "flex-grow", values.first().copied().unwrap_or("0"));
                if values.get(1).is_some_and(|x| x.parse::<f32>().is_ok()) {
                    add(out, "flex-shrink", values[1]);
                    add(out, "flex-basis", values.get(2).copied().unwrap_or("0%"));
                } else {
                    add(out, "flex-shrink", "1");
                    add(out, "flex-basis", values.get(1).copied().unwrap_or("0%"));
                }
            }
        }
    } else if name == "flex-flow" {
        for word in words(value) {
            if matches!(word, "row" | "row-reverse" | "column" | "column-reverse") {
                add(out, "flex-direction", word);
            } else if matches!(word, "wrap" | "nowrap" | "wrap-reverse") {
                add(out, "flex-wrap", word);
            }
        }
    } else if name == "font" {
        let values = words(value);
        let mut size_index = None;
        for (i, word) in values.iter().enumerate() {
            if word.parse::<u16>().is_ok() {
                add(out, "font-weight", word);
                continue;
            }
            let size = word.split('/').next().unwrap_or(word);
            if parse_length(size, 16.0, 16.0, 800.0, 600.0).is_some()
                || matches!(
                    size,
                    "small" | "medium" | "large" | "x-large" | "xx-large" | "smaller" | "larger"
                )
            {
                size_index = Some(i);
                break;
            }
            if matches!(*word, "italic" | "oblique" | "normal") {
                add(out, "font-style", word);
            } else if matches!(*word, "bold" | "bolder" | "lighter") {
                add(out, "font-weight", word);
            }
        }
        if let Some(i) = size_index {
            let (size, line) = values[i].split_once('/').unwrap_or((values[i], "normal"));
            add(out, "font-size", size);
            add(out, "line-height", line);
            if i + 1 < values.len() {
                add(out, "font-family", &values[i + 1..].join(" "));
            }
        }
    } else {
        let name = match name {
            "margin-inline-start" => "margin-left",
            "margin-inline-end" => "margin-right",
            "margin-block-start" => "margin-top",
            "margin-block-end" => "margin-bottom",
            "padding-inline-start" => "padding-left",
            "padding-inline-end" => "padding-right",
            "padding-block-start" => "padding-top",
            "padding-block-end" => "padding-bottom",
            "grid-row-gap" => "row-gap",
            "grid-column-gap" => "column-gap",
            "inline-size" => "width",
            "block-size" => "height",
            "text-decoration-line" => "text-decoration",
            _ => name,
        };
        add(out, name, value);
    }
}

pub fn compute_styles(
    doc: &Document,
    sources: &[String],
    width: f32,
    height: f32,
) -> Vec<ComputedStyle> {
    let mut bytes_left = MAX_STYLE_BYTES;
    let sources: Vec<_> = sources
        .iter()
        .take(256)
        .take_while(|source| {
            if source.len() > bytes_left {
                return false;
            }
            bytes_left -= source.len();
            true
        })
        .map(|source| StyleSource::new(source.as_str()))
        .collect();
    compute_styles_from_sources(doc, &sources, width, height)
}
pub fn compute_styles_from_sources(
    doc: &Document,
    sources: &[StyleSource],
    width: f32,
    height: f32,
) -> Vec<ComputedStyle> {
    let mut rules = Vec::new();
    let mut budget = ParseBudget::new();
    let mut bytes_left = MAX_STYLE_BYTES;
    for source in sources.iter().take(256) {
        if source.media.len() > 32 {
            continue;
        }
        let cost = source.media.iter().fold(source.source.len(), |sum, media| {
            sum.saturating_add(media.len())
        });
        if cost > bytes_left {
            break;
        }
        bytes_left -= cost;
        if !source
            .media
            .iter()
            .all(|media| media_matches_with_budget(media, width, height, &mut budget.work))
        {
            continue;
        }
        parse_source(
            &source.source,
            &source.layer,
            width,
            height,
            &mut rules,
            &mut budget,
        );
        if rules.len() >= 10_000 {
            rules.truncate(10_000);
            break;
        }
    }
    compute_styles_with_rules(doc, &rules, width, height)
}

// Layer precedence is a postorder walk: each parent's implicit final
// sublayer follows its named children; the root is unlayered author CSS.
#[derive(Default)]
struct LayerNode {
    children: Vec<usize>,
}
struct LayerRegistry {
    nodes: Vec<LayerNode>,
    named: HashMap<(usize, String), usize>,
    identities: HashMap<usize, Option<usize>>,
    names_left: usize,
}
impl LayerRegistry {
    fn new() -> Self {
        Self {
            nodes: vec![LayerNode::default()],
            named: HashMap::new(),
            identities: HashMap::new(),
            names_left: MAX_LAYER_NAME_BYTES,
        }
    }
    fn register(&mut self, layer: &Arc<CascadeLayer>) -> Option<usize> {
        let pointer = Arc::as_ptr(layer) as usize;
        if let Some(id) = self.identities.get(&pointer) {
            return *id;
        }
        let parent = match &layer.parent {
            Some(p) => self.register(p)?,
            None => 0,
        };
        if let Some(name) = &layer.name
            && let Some(&id) = self.named.get(&(parent, name.clone()))
        {
            self.identities.insert(pointer, Some(id));
            return Some(id);
        }
        let bytes = layer.name.as_ref().map_or(0, String::len);
        let id = if self.nodes.len() > MAX_LAYERS || bytes > self.names_left {
            None
        } else {
            self.names_left -= bytes;
            let id = self.nodes.len();
            self.nodes.push(LayerNode::default());
            self.nodes[parent].children.push(id);
            if let Some(name) = &layer.name {
                self.named.insert((parent, name.clone()), id);
            }
            Some(id)
        };
        self.identities.insert(pointer, id);
        id
    }
    fn ranks(&self) -> Vec<usize> {
        fn visit(nodes: &[LayerNode], id: usize, ranks: &mut [usize], next: &mut usize) {
            for &child in &nodes[id].children {
                visit(nodes, child, ranks, next);
            }
            ranks[id] = *next;
            *next += 1;
        }
        let mut ranks = vec![0; self.nodes.len()];
        visit(&self.nodes, 0, &mut ranks, &mut 0);
        ranks
    }
}
struct IndexedRule<'a> {
    selector: String,
    specificity: u32,
    declarations: &'a [Declaration],
    order: u32,
    layer: usize,
}
type CascadePriority = (bool, bool, usize, u32, u32, u32);
type CascadeBucket = (bool, bool, usize);
struct Candidate {
    priority: CascadePriority,
    value: String,
    pending_shorthand: Option<Arc<str>>,
}
#[derive(Default)]
struct CascadedProperties {
    properties: BTreeMap<String, BTreeMap<CascadeBucket, Candidate>>,
    count: usize,
    bytes: usize,
}
impl CascadedProperties {
    fn insert(&mut self, decl: &Declaration, inline: bool, layer: usize, tail: (u32, u32, u32)) {
        let (name, value, important) = (decl.name.as_str(), decl.value.as_str(), decl.important);
        let precedence = if important {
            MAX_LAYERS + 1 - layer
        } else {
            layer
        };
        let priority = (important, inline, precedence, tail.0, tail.1, tail.2);
        let bucket = (important, inline, layer);
        let old = self.properties.get(name).and_then(|p| p.get(&bucket));
        if old.is_some_and(|c| c.priority > priority) {
            return;
        }
        let old_bytes = old.map_or(0, |c| c.value.len() + name.len());
        let bytes = self.bytes - old_bytes + value.len() + name.len();
        if bytes > 1024 * 1024 || old.is_none() && self.count >= 4096 {
            return;
        }
        if old.is_none() {
            self.count += 1;
        }
        self.bytes = bytes;
        self.properties.entry(name.into()).or_default().insert(
            bucket,
            Candidate {
                priority,
                value: value.into(),
                pending_shorthand: decl.pending_shorthand.clone(),
            },
        );
    }
}
// Rollback excludes every declaration between a layer's normal and important
// positions. Inline important rollback is explicitly exempt: it removes just
// the element-attached declarations and can expose stylesheet !important.
fn cascaded_value(
    name: &str,
    candidates: &BTreeMap<CascadeBucket, Candidate>,
    variables: Option<&BTreeMap<String, String>>,
    work: &mut usize,
) -> Option<String> {
    let sort_cost = candidates
        .len()
        .saturating_mul(candidates.len().max(1).ilog2() as usize + 1);
    if sort_cost > *work {
        *work = 0;
        return None;
    }
    *work -= sort_cost;
    let mut ordered: Vec<_> = candidates.iter().collect();
    ordered.sort_unstable_by_key(|(_, candidate)| std::cmp::Reverse(candidate.priority));
    let mut remove_inline = false;
    let mut earlier_than = None;
    for (&(important, inline, layer), candidate) in ordered {
        if remove_inline && inline
            || earlier_than.is_some_and(|rank| important || inline || layer >= rank)
        {
            continue;
        }
        if candidate.value.len() > *work {
            *work = 0;
            return None;
        }
        *work -= candidate.value.len();
        let value = if let Some(variables) = variables {
            let value = resolve_vars(&candidate.value, variables, 0, work)
                .unwrap_or_else(|| "unset".into());
            if let Some(shorthand) = &candidate.pending_shorthand {
                let count =
                    shorthand_properties(shorthand).map_or(1, |properties| properties.len());
                let cost = value.len().saturating_mul(count);
                if cost > *work {
                    *work = 0;
                    return None;
                }
                *work -= cost;
                let mut expanded = Vec::new();
                expand_declaration(shorthand, &value, false, &mut expanded);
                expanded
                    .into_iter()
                    .find(|decl| decl.name == name)
                    .map_or_else(|| "unset".into(), |decl| decl.value)
            } else {
                value
            }
        } else {
            candidate.value.clone()
        };
        let value = wide_keyword(&value).unwrap_or(value);
        if value == "revert-layer" {
            if inline {
                remove_inline = true;
            } else {
                earlier_than = Some(layer);
            }
        } else if value == "revert" {
            return None;
        } else {
            return Some(value);
        }
    }
    None
}
pub fn compute_styles_with_rules(
    doc: &Document,
    rules: &[Rule],
    width: f32,
    height: f32,
) -> Vec<ComputedStyle> {
    let mut indexed = vec![];
    let mut selector_bytes = 0usize;
    let mut work = 20_000_000usize;
    let mut by_key: HashMap<String, Vec<usize>> = HashMap::new();
    let mut layers = LayerRegistry::new();
    let rule_layers: Vec<_> = rules
        .iter()
        .take(10_000)
        .map(|rule| {
            rule.layer
                .as_ref()
                .map_or(Some(0), |layer| layers.register(layer))
        })
        .collect();
    let ranks = layers.ranks();
    for (order, rule) in rules.iter().enumerate().take(10_000) {
        let Some(layer) = rule_layers[order] else {
            continue;
        };
        for selector in rule
            .selectors
            .iter()
            .take(128)
            .filter(|selector| selector.len() <= 4096)
        {
            let Ok(compiled) = crate::selectors::parse_list(selector, &mut work) else {
                continue;
            };
            for selector in compiled {
                let key = selector_key(&selector.source);
                let cost =
                    selector.source.len() + key.len() + std::mem::size_of::<IndexedRule<'_>>() + 16;
                if cost > MAX_STYLE_BYTES.saturating_sub(selector_bytes) {
                    break;
                }
                selector_bytes += cost;
                let index = indexed.len();
                by_key.entry(key).or_default().push(index);
                indexed.push(IndexedRule {
                    specificity: specificity_inner(&selector.source, 0),
                    selector: selector.source,
                    declarations: &rule.declarations,
                    order: order as u32,
                    layer: ranks[layer],
                });
            }
        }
    }
    let mut styles = vec![ComputedStyle::default(); doc.nodes.len()];
    let mut first_summaries = vec![false; doc.nodes.len()];
    let mut border_styles = vec![Edges::all(false); doc.nodes.len()];
    let empty_variables = Arc::new(BTreeMap::new());
    let mut variables: Vec<Arc<BTreeMap<String, String>>> = vec![empty_variables; doc.nodes.len()];
    let mut pending = vec![doc.root];
    let mut visited = vec![false; doc.nodes.len()];
    let mut retained_variable_bytes = 0usize;
    let mut grid_tracks = GridTrackPool::default();
    let mut root_font = 16.0;
    while let Some(id) = pending.pop() {
        let Some(node) = doc.nodes.get(id) else {
            continue;
        };
        if visited[id] {
            continue;
        }
        visited[id] = true;
        let parent = node.parent.and_then(|p| styles.get(p));
        let mut style = ComputedStyle::inherited(parent);
        if matches!(node.kind, NodeKind::Document) {
            style.display = Display::Block;
        } else if matches!(
            node.kind,
            NodeKind::Comment(_) | NodeKind::Doctype(_) | NodeKind::ProcessingInstruction { .. }
        ) {
            style.display = Display::None;
        }
        let tag = doc.tag(id).unwrap_or("");
        if tag == "details"
            && doc.namespace(id) == Some(Namespace::Html)
            && let Some(summary) = doc.first_summary(id)
            && let Some(first) = first_summaries.get_mut(summary)
        {
            *first = true;
        }
        apply_user_agent(&mut style, doc, id, tag, first_summaries[id]);
        style.line_height = style.font_size
            * parent
                .map(|p| p.line_height / p.font_size.max(1.0))
                .unwrap_or(1.2);
        let inherited_vars = node
            .parent
            .and_then(|p| variables.get(p))
            .cloned()
            .unwrap_or_default();
        let mut vars = inherited_vars;
        // Origin, importance, selector specificity and source order are applied per property.
        let mut cascade = CascadedProperties::default();
        if !tag.is_empty() {
            let mut candidates = BTreeSet::new();
            for key in ["*".to_owned(), format!("t:{}", tag.to_ascii_lowercase())] {
                if let Some(r) = by_key.get(&key) {
                    let take = r
                        .len()
                        .min(work)
                        .min(8192usize.saturating_sub(candidates.len()));
                    work -= take;
                    candidates.extend(r.iter().take(take).copied());
                }
            }
            if let Some(value) = doc.attr(id, "id")
                && let Some(r) = by_key.get(&format!("i:{value}"))
            {
                let take = r
                    .len()
                    .min(work)
                    .min(8192usize.saturating_sub(candidates.len()));
                work -= take;
                candidates.extend(r.iter().take(take).copied());
            }
            if let Some(classes) = doc.attr(id, "class") {
                for class in classes.split_ascii_whitespace().take(1024) {
                    if let Some(r) = by_key.get(&format!("c:{class}")) {
                        let take = r
                            .len()
                            .min(work)
                            .min(8192usize.saturating_sub(candidates.len()));
                        work -= take;
                        candidates.extend(r.iter().take(take).copied());
                    }
                }
            }
            for candidate in candidates {
                if work == 0 {
                    break;
                }
                let rule = &indexed[candidate];
                if matches_compiled_selector(doc, id, &rule.selector, &mut work) {
                    for (decl_order, decl) in rule.declarations.iter().enumerate() {
                        if !supported_property(&decl.name) || decl.value.len() > 4096 {
                            continue;
                        }
                        let cost = decl.name.len() + decl.value.len();
                        if work < cost {
                            work = 0;
                            break;
                        }
                        work -= cost;
                        cascade.insert(
                            decl,
                            false,
                            rule.layer,
                            (rule.specificity, rule.order, decl_order as u32),
                        );
                    }
                }
            }
            if let Some(inline) = doc.attr(id, "style")
                && inline.len() <= work
            {
                work -= inline.len();
                for (decl_order, decl) in parse_declarations(inline).into_iter().enumerate() {
                    if !supported_property(&decl.name) || decl.value.len() > 4096 {
                        continue;
                    }
                    let cost = decl.name.len() + decl.value.len();
                    if cost > work {
                        work = 0;
                        break;
                    }
                    work -= cost;
                    cascade.insert(
                        &decl,
                        true,
                        ranks[0],
                        (1 << 30, u32::MAX, decl_order as u32),
                    );
                }
            }
            let mut custom = BTreeMap::new();
            for (name, candidates) in &cascade.properties {
                if name.starts_with("--")
                    && let Some(value) = cascaded_value(name, candidates, None, &mut work)
                {
                    custom.insert(name.clone(), value);
                }
            }
            let inherited_variable_size: usize = vars.iter().map(|(k, v)| k.len() + v.len()).sum();
            let new_variable_size: usize = custom.iter().map(|(k, v)| k.len() + v.len()).sum();
            if new_variable_size > 0
                && retained_variable_bytes + inherited_variable_size + new_variable_size
                    <= 8 * 1024 * 1024
            {
                retained_variable_bytes += inherited_variable_size + new_variable_size;
                let map = Arc::make_mut(&mut vars);
                for (name, value) in custom {
                    if value.eq_ignore_ascii_case("initial") {
                        map.remove(&name);
                    } else if !matches!(value.to_ascii_lowercase().as_str(), "inherit" | "unset")
                        && value.len() <= 4096
                        && (map.len() < 128 || map.contains_key(&name))
                    {
                        map.insert(name, value);
                    }
                }
            }
            let mut resolved: BTreeMap<String, String> = BTreeMap::new();
            for (name, candidates) in &cascade.properties {
                if !name.starts_with("--")
                    && let Some(value) = cascaded_value(name, candidates, Some(&vars), &mut work)
                {
                    resolved.insert(name.clone(), value);
                }
            }
            // Font-relative units use the element's computed font size.
            if let Some(value) = resolved.get("font-size") {
                apply_property(
                    &mut style,
                    "font-size",
                    value,
                    parent,
                    root_font,
                    width,
                    height,
                );
            }
            if tag == "html" && doc.namespace(id) == Some(Namespace::Html) {
                root_font = style.font_size;
            }
            if let Some(value) = resolved.get("color") {
                apply_property(&mut style, "color", value, parent, root_font, width, height);
            }
            let native_border = doc.namespace(id) == Some(Namespace::Html)
                && matches!(tag, "button" | "input" | "select" | "textarea" | "hr");
            if !native_border {
                style.border_color = style.color;
            }
            for (name, value) in &resolved {
                if name != "font-size" && name != "color" {
                    apply_property(&mut style, name, value, parent, root_font, width, height);
                }
            }
            if !resolved.contains_key("line-height") {
                let ratio = parent
                    .map(|p| p.line_height / p.font_size.max(1.0))
                    .unwrap_or(1.2);
                style.line_height = style.font_size * ratio;
            }
            for side in ["top", "right", "bottom", "left"] {
                let border_style = resolved
                    .get(&format!("border-{side}-style"))
                    .map(|value| value.to_ascii_lowercase());
                let visible = match border_style.as_deref() {
                    Some("inherit") => {
                        node.parent
                            .and_then(|p| border_styles.get(p))
                            .is_some_and(|edges| {
                                let mut edges = *edges;
                                *edge_mut(&mut edges, side)
                            })
                    }
                    Some("initial" | "unset" | "none" | "hidden") => false,
                    Some(
                        "solid" | "dotted" | "dashed" | "double" | "groove" | "ridge" | "inset"
                        | "outset",
                    ) => true,
                    _ => native_border,
                };
                *edge_mut(&mut border_styles[id], side) = visible;
                if !visible {
                    *edge_mut(&mut style.border_width, side) = 0.0;
                }
            }
        }
        if matches!(style.position.as_str(), "absolute" | "fixed") {
            style.float = "none".into();
            if matches!(style.display, Display::Inline | Display::InlineBlock) {
                style.display = Display::Block;
            }
        } else if style.float != "none"
            && matches!(style.display, Display::Inline | Display::InlineBlock)
        {
            // CSS2 section 9.7 blockifies floating inline boxes.
            style.display = Display::Block;
        }
        variables[id] = vars;
        grid_tracks.style(&mut style);
        styles[id] = style;
        pending.extend(node.children.iter().rev().copied());
    }
    styles
}
const SUPPORTED_PROPERTIES: &[&str] = &[
    "display",
    "float",
    "clear",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "color",
    "background-color",
    "font-size",
    "font-family",
    "font-style",
    "font-weight",
    "line-height",
    "text-align",
    "white-space",
    "text-decoration",
    "flex-direction",
    "flex-wrap",
    "justify-content",
    "align-items",
    "align-self",
    "order",
    "row-gap",
    "column-gap",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "grid-template-columns",
    "grid-template-rows",
    "grid-auto-columns",
    "grid-auto-rows",
    "grid-auto-flow",
    "grid-column-start",
    "grid-column-end",
    "grid-row-start",
    "grid-row-end",
    "justify-items",
    "justify-self",
    "align-content",
    "position",
    "z-index",
    "top",
    "right",
    "bottom",
    "left",
    "overflow",
    "overflow-x",
    "overflow-y",
    "border-radius",
    "opacity",
    "box-sizing",
    "list-style-type",
    "list-style-position",
    "vertical-align",
];
fn supported_property(name: &str) -> bool {
    name.starts_with("--") || SUPPORTED_PROPERTIES.contains(&name)
}

fn selector_key(selector: &str) -> String {
    // Index only the final compound, outside functional pseudo selectors and attributes.
    let mut start = 0;
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in selector.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            continue;
        }
        if matches!(c, '(' | '[') {
            depth += 1;
        } else if matches!(c, ')' | ']') {
            depth -= 1;
        } else if depth == 0 && (c.is_ascii_whitespace() || matches!(c, '>' | '+' | '~')) {
            start = i + c.len_utf8();
        }
    }
    let compound = selector[start..].trim();
    let mut first_class = None;
    let mut i = 0;
    while i < compound.len() {
        let b = compound.as_bytes()[i];
        if b == b':' || b == b'[' {
            break;
        }
        if b == b'#' || b == b'.' {
            let end = identifier_end(compound, i + 1);
            if end == i + 1 {
                break;
            }
            let value = &compound[i + 1..end];
            if b == b'#' {
                return format!("i:{value}");
            }
            first_class.get_or_insert_with(|| format!("c:{value}"));
            i = end;
        } else {
            let c = compound[i..].chars().next().unwrap_or(' ');
            i += c.len_utf8();
        }
    }
    if let Some(class) = first_class {
        return class;
    }
    let end = identifier_end(compound, 0);
    if end > 0 {
        format!("t:{}", compound[..end].to_ascii_lowercase())
    } else {
        "*".into()
    }
}
fn identifier_end(s: &str, start: usize) -> usize {
    let mut end = start;
    for (i, c) in s[start..].char_indices() {
        if c.is_alphanumeric() || matches!(c, '-' | '_') || !c.is_ascii() {
            end = start + i + c.len_utf8();
        } else {
            break;
        }
    }
    end
}
pub fn specificity(selector: &str) -> u32 {
    crate::selectors::parse_list(selector, &mut 100_000)
        .map(|list| {
            list.iter()
                .map(|s| specificity_inner(&s.source, 0))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0)
}
fn specificity_inner(selector: &str, depth: usize) -> u32 {
    if depth > 32 {
        return 0;
    }
    let mut score = 0u32;
    let mut i = 0;
    let mut expect_type = true;
    while i < selector.len() {
        let b = selector.as_bytes()[i];
        match b {
            b'#' | b'.' => {
                score = score.saturating_add(if b == b'#' { 1 << 20 } else { 1 << 10 });
                i = identifier_end(selector, i + 1);
                expect_type = false;
            }
            b'[' => {
                score = score.saturating_add(1 << 10);
                i = selector_block_end(selector, i, b'[', b']')
                    .map_or(selector.len(), |end| end + 1);
                expect_type = false;
            }
            b':' => {
                let pseudo_element = selector.as_bytes().get(i + 1) == Some(&b':');
                i += 1 + usize::from(pseudo_element);
                let end = identifier_end(selector, i);
                let name = &selector[i..end];
                i = end;
                let mut argument = None;
                if selector.as_bytes().get(i) == Some(&b'(') {
                    let start = i + 1;
                    let end = selector_block_end(selector, i, b'(', b')').unwrap_or(selector.len());
                    argument = Some(&selector[start..end]);
                    i = (end + 1).min(selector.len());
                }
                if name != "where" {
                    score = score.saturating_add(if pseudo_element {
                        1
                    } else if matches!(name, "is" | "not" | "has") {
                        argument
                            .map(|a| {
                                split_top_level(a, ',')
                                    .into_iter()
                                    .map(|x| specificity_inner(x, depth + 1))
                                    .max()
                                    .unwrap_or(0)
                            })
                            .unwrap_or(0)
                    } else {
                        1 << 10
                    });
                }
                expect_type = false;
            }
            b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'+' | b'~' => {
                expect_type = true;
                i += 1;
            }
            b'*' => {
                expect_type = false;
                i += 1;
            }
            _ => {
                let end = identifier_end(selector, i);
                if end > i {
                    if expect_type {
                        score = score.saturating_add(1);
                    }
                    i = end;
                    expect_type = false;
                } else {
                    i += selector[i..].chars().next().unwrap_or(' ').len_utf8();
                }
            }
        }
    }
    score
}
fn selector_block_end(source: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (at, byte) in source.bytes().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if byte == q {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(at);
            }
        }
    }
    None
}
fn resolve_vars(
    value: &str,
    variables: &BTreeMap<String, String>,
    depth: usize,
    work: &mut usize,
) -> Option<String> {
    if value.len() > *work {
        *work = 0;
        return None;
    }
    *work -= value.len();
    if depth > 16 || value.len() > 65_536 {
        return None;
    }
    let Some(start) = value.find("var(") else {
        return Some(value.into());
    };
    let mut level = 1;
    let mut end = start + 4;
    for b in value.as_bytes().iter().skip(end) {
        if *b == b'(' {
            level += 1;
        } else if *b == b')' {
            level -= 1;
            if level == 0 {
                break;
            }
        }
        end += 1;
    }
    if end >= value.len() {
        return None;
    }
    let args = split_top_level(&value[start + 4..end], ',');
    let key = args.first()?.trim();
    let replacement = variables
        .get(key)
        .map(String::as_str)
        .or_else(|| args.get(1).copied())?;
    let replacement = resolve_vars(replacement, variables, depth + 1, work)?;
    let size = start + replacement.len() + value.len() - end - 1;
    if size > 65_536 || size > *work {
        return None;
    }
    let new = format!("{}{}{}", &value[..start], replacement, &value[end + 1..]);
    resolve_vars(&new, variables, depth + 1, work)
}
fn apply_user_agent(
    s: &mut ComputedStyle,
    doc: &Document,
    id: NodeId,
    tag: &str,
    first_summary: bool,
) {
    if doc.namespace(id) != Some(Namespace::Html) {
        if doc.namespace(id) == Some(Namespace::Svg) {
            if tag == "svg" {
                s.display = Display::InlineBlock;
                if let Some(width) = doc
                    .attr(id, "width")
                    .and_then(|value| parse_length(value, s.font_size, 16.0, 800.0, 600.0))
                {
                    s.width = width;
                }
                if let Some(height) = doc
                    .attr(id, "height")
                    .and_then(|value| parse_length(value, s.font_size, 16.0, 800.0, 600.0))
                {
                    s.height = height;
                }
            } else if matches!(
                tag,
                "title" | "desc" | "style" | "script" | "metadata" | "defs"
            ) {
                // These SVG elements are non-rendering; foreign lookalikes do
                // not inherit the behavior of HTML metadata or controls.
                s.display = Display::None;
            }
        }
        return;
    }
    s.list_item = tag == "li" || first_summary;
    if first_summary {
        s.list_style_position = "inside".into();
        s.list_style_type = if doc.nodes[id]
            .parent
            .is_some_and(|parent| doc.attr(parent, "open").is_some())
        {
            "disclosure-open"
        } else {
            "disclosure-closed"
        }
        .into();
    }
    s.display = match tag {
        "noscript" if doc.scripting_enabled() => Display::None,
        "html" | "body" | "div" | "p" | "section" | "article" | "main" | "header" | "footer"
        | "nav" | "aside" | "address" | "blockquote" | "figure" | "figcaption" | "h1" | "h2"
        | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "li" | "dl" | "dt" | "dd" | "pre" | "form"
        | "fieldset" | "legend" | "hr" | "table" | "thead" | "tbody" | "tfoot" | "tr"
        | "details" | "summary" | "center" => Display::Block,
        "head" | "title" | "style" | "script" | "meta" | "link" | "base" | "template"
        | "source" | "track" => Display::None,
        "button" | "input" | "select" | "textarea" | "img" | "canvas" | "video" | "audio"
        | "iframe" => Display::InlineBlock,
        _ => s.display,
    };
    match tag {
        "body" => s.margin = Edges::all(Length::Px(8.0)),
        "p" | "blockquote" | "figure" | "pre" | "dl" | "ul" | "ol" => {
            s.margin.top = Length::Px(s.font_size);
            s.margin.bottom = Length::Px(s.font_size);
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let (factor, margin) = match tag {
                "h1" => (2.0, 0.67),
                "h2" => (1.5, 0.83),
                "h3" => (1.17, 1.0),
                "h4" => (1.0, 1.33),
                "h5" => (0.83, 1.67),
                _ => (0.67, 2.33),
            };
            s.font_size *= factor;
            s.font_weight = 700;
            s.margin.top = Length::Px(s.font_size * margin);
            s.margin.bottom = s.margin.top;
        }
        "button" | "input" | "select" | "textarea" => {
            s.font_size = 13.0;
            s.padding = Edges {
                top: Length::Px(3.0),
                right: Length::Px(7.0),
                bottom: Length::Px(3.0),
                left: Length::Px(7.0),
            };
            s.border_width = Edges::all(1.0);
            s.border_color = Color::rgb(118, 118, 118);
            s.background_color = if tag == "button" {
                Color::rgb(239, 239, 239)
            } else {
                Color::WHITE
            };
            s.border_radius = 2.0;
            if tag == "input" {
                if doc.attr(id, "type").is_some_and(|kind| {
                    kind.eq_ignore_ascii_case("checkbox") || kind.eq_ignore_ascii_case("radio")
                }) {
                    // The control painter supplies the native 16px box. Keep
                    // these defaults in the UA layer so authored sizing wins.
                    s.padding = Edges::all(Length::Px(0.0));
                    s.border_width = Edges::all(0.0);
                    s.background_color = Color::TRANSPARENT;
                    s.border_radius = 0.0;
                } else {
                    s.width = Length::Px(180.0);
                }
            }
        }
        "hr" => {
            s.margin.top = Length::Px(8.0);
            s.margin.bottom = Length::Px(8.0);
            s.border_width.top = 1.0;
            s.border_color = Color::rgb(128, 128, 128);
        }
        "a" if doc.attr(id, "href").is_some() => {
            s.color = Color::rgb(0, 0, 238);
            s.text_decoration = "underline".into();
        }
        "strong" | "b" => s.font_weight = 700,
        "th" => {
            s.font_weight = 700;
            s.text_align = "center".into();
        }
        "em" | "i" | "cite" | "dfn" | "var" => s.font_style = "italic".into(),
        "u" | "ins" => s.text_decoration = "underline".into(),
        "s" | "strike" | "del" => s.text_decoration = "line-through".into(),
        "small" => s.font_size *= 0.8333,
        "big" => s.font_size *= 1.2,
        "sub" | "sup" => {
            s.font_size *= 0.8333;
            s.vertical_align = tag.into();
        }
        "mark" => s.background_color = Color::rgb(255, 255, 0),
        _ => {}
    }
    if matches!(tag, "ul" | "ol") {
        s.padding.left = Length::Px(40.0);
        if tag == "ol" {
            s.list_style_type = "decimal".into();
        }
    }
    if matches!(tag, "blockquote" | "figure") {
        s.margin.left = Length::Px(40.0);
        s.margin.right = Length::Px(40.0);
    }
    if tag == "dd" {
        s.margin.left = Length::Px(40.0);
    }
    if matches!(tag, "pre" | "code" | "kbd" | "samp" | "tt" | "textarea") {
        s.font_family = "monospace".into();
    }
    if tag == "pre" {
        s.white_space = "pre".into();
    }
    if tag == "center" {
        s.text_align = "center".into();
    }
    if let Some(width) = doc
        .attr(id, "width")
        .and_then(|x| parse_length(x, s.font_size, 16.0, 800.0, 600.0))
    {
        s.width = width;
    }
    if let Some(height) = doc
        .attr(id, "height")
        .and_then(|x| parse_length(x, s.font_size, 16.0, 800.0, 600.0))
    {
        s.height = height;
    }
    if let Some(color) = doc.attr(id, "bgcolor").and_then(parse_color) {
        s.background_color = color;
    }
    if let Some(color) = doc.attr(id, "color").and_then(parse_color) {
        s.color = color;
    }
    if let Some(align) = doc.attr(id, "align") {
        s.text_align = align.into();
    }
    if doc.attr(id, "hidden").is_some()
        || tag == "input"
            && doc
                .attr(id, "type")
                .is_some_and(|kind| kind.eq_ignore_ascii_case("hidden"))
    {
        s.display = Display::None;
    }
}
fn edge_mut<'a, T>(edges: &'a mut Edges<T>, side: &str) -> &'a mut T {
    match side {
        "top" => &mut edges.top,
        "right" => &mut edges.right,
        "bottom" => &mut edges.bottom,
        _ => &mut edges.left,
    }
}
fn apply_property(
    s: &mut ComputedStyle,
    name: &str,
    value: &str,
    parent: Option<&ComputedStyle>,
    root_font: f32,
    width: f32,
    height: f32,
) {
    let keyword = (name.starts_with("grid-")
        || matches!(
            name,
            "float"
                | "clear"
                | "row-gap"
                | "column-gap"
                | "justify-items"
                | "justify-self"
                | "align-content"
        ))
    .then(|| value.trim().to_ascii_lowercase());
    let value = keyword.as_deref().unwrap_or_else(|| value.trim());
    if value.len() > 256
        && matches!(
            name,
            "font-family"
                | "text-decoration"
                | "justify-content"
                | "align-items"
                | "align-self"
                | "justify-items"
                | "justify-self"
                | "align-content"
                | "grid-auto-flow"
                | "list-style-type"
                | "list-style-position"
                | "vertical-align"
        )
    {
        return;
    }
    let initial = ComputedStyle::default();
    if css_wide(value) {
        let value = value.to_ascii_lowercase();
        let inherited = matches!(
            name,
            "color"
                | "font-size"
                | "font-weight"
                | "font-family"
                | "font-style"
                | "line-height"
                | "text-align"
                | "white-space"
                | "list-style-type"
                | "list-style-position"
        );
        let from = if value == "inherit" || value == "unset" && inherited {
            parent.unwrap_or(&initial)
        } else {
            &initial
        };
        if value != "inherit"
            && let Some(rest) = name.strip_prefix("border-")
        {
            if let Some(side) = rest.strip_suffix("-width") {
                *edge_mut(&mut s.border_width, side) = 3.0;
                return;
            }
            if rest.ends_with("-color") {
                s.border_color = s.color;
                return;
            }
        }
        copy_property(s, from, name);
        return;
    }
    let length = || parse_length(value, s.font_size, root_font, width, height);
    let color = || {
        if value.eq_ignore_ascii_case("currentcolor") {
            Some(s.color)
        } else {
            parse_color(value)
        }
    };
    match name {
        "display" => {
            if matches!(
                value,
                "none"
                    | "block"
                    | "flow-root"
                    | "list-item"
                    | "table"
                    | "table-row"
                    | "table-row-group"
                    | "table-header-group"
                    | "table-footer-group"
                    | "inline"
                    | "contents"
                    | "inline-block"
                    | "table-cell"
                    | "flex"
                    | "inline-flex"
                    | "grid"
                    | "inline-grid"
            ) {
                s.flow_root = value == "flow-root";
                s.list_item = value == "list-item";
            }
            s.display = match value {
                "none" => Display::None,
                "block" | "flow-root" | "list-item" | "table" | "table-row" | "table-row-group"
                | "table-header-group" | "table-footer-group" => Display::Block,
                "inline" | "contents" => Display::Inline,
                "inline-block" | "table-cell" => Display::InlineBlock,
                "flex" | "inline-flex" => Display::Flex,
                "grid" | "inline-grid" => Display::Grid,
                _ => s.display,
            };
        }
        "float" => {
            let value = value.to_ascii_lowercase();
            if matches!(value.as_str(), "none" | "left" | "right") {
                s.float = value;
            }
        }
        "clear" => {
            let value = value.to_ascii_lowercase();
            if matches!(value.as_str(), "none" | "left" | "right" | "both") {
                s.clear = value;
            }
        }
        "width" => {
            if let Some(v) = length() {
                s.width = v;
            }
        }
        "height" => {
            if let Some(v) = length() {
                s.height = v;
            }
        }
        "min-width" => {
            if let Some(v) = length() {
                s.min_width = v;
            }
        }
        "max-width" => {
            if let Some(v) = length() {
                s.max_width = v;
            }
        }
        "min-height" => {
            if let Some(v) = length() {
                s.min_height = v;
            }
        }
        "max-height" => {
            if let Some(v) = length() {
                s.max_height = v;
            }
        }
        "top" => {
            if let Some(v) = length() {
                s.top = v;
            }
        }
        "right" => {
            if let Some(v) = length() {
                s.right = v;
            }
        }
        "bottom" => {
            if let Some(v) = length() {
                s.bottom = v;
            }
        }
        "left" => {
            if let Some(v) = length() {
                s.left = v;
            }
        }
        "color" => {
            if let Some(v) = color() {
                s.color = v;
            }
        }
        "background-color" => {
            if let Some(v) = color() {
                s.background_color = v;
            }
        }
        "border-top-color" | "border-right-color" | "border-bottom-color" | "border-left-color" => {
            if let Some(v) = color() {
                s.border_color = v;
            }
        }
        "font-size" => {
            let parent_size = parent.map(|p| p.font_size).unwrap_or(16.0);
            let parsed = match value {
                "xx-small" => Some(9.0),
                "x-small" => Some(10.0),
                "small" => Some(13.0),
                "medium" => Some(16.0),
                "large" => Some(18.0),
                "x-large" => Some(24.0),
                "xx-large" => Some(32.0),
                "xxx-large" => Some(48.0),
                "smaller" => Some(parent_size / 1.2),
                "larger" => Some(parent_size * 1.2),
                _ => parse_length(value, parent_size, root_font, width, height)
                    .and_then(|v| v.resolve(parent_size)),
            };
            if let Some(v) = parsed
                && v >= 0.0
            {
                s.font_size = v.clamp(1.0, 512.0);
            }
        }
        "font-weight" => {
            let weight = match value {
                "normal" => 400,
                "bold" => 700,
                "bolder" => {
                    if parent.map(|p| p.font_weight).unwrap_or(400) < 550 {
                        700
                    } else {
                        900
                    }
                }
                "lighter" => 300,
                _ => value.parse().unwrap_or(s.font_weight),
            };
            s.font_weight = weight.clamp(1, 1000);
        }
        "font-family" => s.font_family = value.to_owned(),
        "font-style" => {
            if matches!(value, "normal" | "italic" | "oblique") {
                s.font_style = value.into();
            }
        }
        "line-height" => {
            let parsed = if value == "normal" {
                Some(s.font_size * 1.2)
            } else if let Some(n) = finite_number(value) {
                Some(n * s.font_size)
            } else {
                length().and_then(|l| l.resolve(s.font_size))
            };
            if let Some(v) = parsed
                && v >= 0.0
            {
                s.line_height = v.min(4096.0);
            }
        }
        "text-align" => {
            if matches!(
                value,
                "left" | "right" | "center" | "justify" | "start" | "end"
            ) {
                s.text_align = value.into();
            }
        }
        "white-space" => {
            if matches!(
                value,
                "normal" | "nowrap" | "pre" | "pre-wrap" | "pre-line" | "break-spaces"
            ) {
                s.white_space = value.into();
            }
        }
        "text-decoration" => s.text_decoration = value.into(),
        "flex-direction" => {
            if matches!(value, "row" | "row-reverse" | "column" | "column-reverse") {
                s.flex_direction = value.into();
            }
        }
        "flex-wrap" => {
            if matches!(value, "nowrap" | "wrap" | "wrap-reverse") {
                s.flex_wrap = value.into();
            }
        }
        "justify-content" => s.justify_content = value.into(),
        "align-items" => s.align_items = value.into(),
        "align-self" => {
            if matches!(
                value,
                "auto"
                    | "normal"
                    | "stretch"
                    | "start"
                    | "end"
                    | "self-start"
                    | "self-end"
                    | "flex-start"
                    | "flex-end"
                    | "center"
                    | "baseline"
            ) {
                s.align_self = value.into();
            }
        }
        "order" => {
            if let Ok(order) = value.parse::<i32>() {
                s.order = order;
            }
        }
        "row-gap" | "column-gap" => {
            let parsed = if value == "normal" {
                Some(Length::Px(0.0))
            } else {
                length()
            };
            if let Some(v @ (Length::Px(_) | Length::Percent(_))) = parsed
                && v.resolve(width).is_some_and(|v| v >= 0.0)
            {
                if name == "row-gap" {
                    s.row_gap = v;
                    s.gap = v.resolve(width).unwrap_or(0.0);
                } else {
                    s.column_gap = v;
                }
            }
        }
        "flex-grow" => {
            if let Some(v) = finite_number(value) {
                s.flex_grow = v.max(0.0);
            }
        }
        "flex-shrink" => {
            if let Some(v) = finite_number(value) {
                s.flex_shrink = v.max(0.0);
            }
        }
        "flex-basis" => {
            if let Some(v) = length() {
                s.flex_basis = v;
            }
        }
        "grid-template-columns" | "grid-template-rows" | "grid-auto-columns" | "grid-auto-rows" => {
            if name.starts_with("grid-auto-") && value.contains("repeat(") {
                return;
            }
            if let Some(tracks) = parse_grid_tracks(value, s.font_size, root_font, width, height) {
                match name {
                    "grid-template-columns" => s.grid_template_columns = tracks.into(),
                    "grid-template-rows" => s.grid_template_rows = tracks.into(),
                    "grid-auto-columns" if !tracks.is_empty() => {
                        s.grid_auto_columns = tracks.into()
                    }
                    "grid-auto-rows" if !tracks.is_empty() => s.grid_auto_rows = tracks.into(),
                    _ => {}
                }
            }
        }
        "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end" => {
            if let Some(line) = parse_grid_line(value) {
                match name {
                    "grid-column-start" => s.grid_column_start = line,
                    "grid-column-end" => s.grid_column_end = line,
                    "grid-row-start" => s.grid_row_start = line,
                    _ => s.grid_row_end = line,
                }
            }
        }
        "grid-auto-flow" => {
            if valid_grid_auto_flow(value) {
                s.grid_auto_flow = value.into();
            }
        }
        "justify-items" => s.justify_items = value.into(),
        "justify-self" => s.justify_self = value.into(),
        "align-content" => s.align_content = value.into(),
        "z-index" => {
            if value.eq_ignore_ascii_case("auto") {
                s.z_index = None;
            } else if let Ok(value) = value.parse::<i32>() {
                s.z_index = Some(value);
            }
        }
        "position" => {
            let value = value.to_ascii_lowercase();
            if matches!(
                value.as_str(),
                "static" | "relative" | "absolute" | "fixed" | "sticky"
            ) {
                s.position = value;
            }
        }
        "overflow" | "overflow-x" | "overflow-y" => {
            if matches!(value, "visible" | "hidden" | "clip" | "scroll" | "auto") {
                s.overflow = value.into();
            }
        }
        "border-radius" => {
            if let Some(v) = words(value)
                .first()
                .and_then(|v| parse_length(v, s.font_size, root_font, width, height))
                .and_then(|v| v.resolve(width))
            {
                s.border_radius = v.max(0.0);
            }
        }
        "opacity" => {
            if let Some(v) = finite_number(value) {
                s.opacity = v.clamp(0.0, 1.0);
            }
        }
        "box-sizing" => {
            if matches!(value, "content-box" | "border-box") {
                s.box_sizing = value.into();
            }
        }
        "list-style-type" => s.list_style_type = value.into(),
        "list-style-position" if matches!(value, "inside" | "outside") => {
            s.list_style_position = value.into()
        }
        "vertical-align" => s.vertical_align = value.into(),
        _ => {
            if let Some(side) = name.strip_prefix("margin-") {
                if ["top", "right", "bottom", "left"].contains(&side)
                    && let Some(v) = length()
                {
                    *edge_mut(&mut s.margin, side) = v;
                }
            } else if let Some(side) = name.strip_prefix("padding-") {
                if ["top", "right", "bottom", "left"].contains(&side)
                    && let Some(v) = length()
                    && v.resolve(100.0).is_some_and(|x| x >= 0.0)
                {
                    *edge_mut(&mut s.padding, side) = v;
                }
            } else if let Some(side) = name
                .strip_prefix("border-")
                .and_then(|s| s.strip_suffix("-width"))
            {
                let n = match value {
                    "thin" => Some(1.0),
                    "medium" => Some(3.0),
                    "thick" => Some(5.0),
                    _ => length().and_then(|v| v.resolve(width)),
                };
                if let Some(v) = n
                    && v >= 0.0
                {
                    *edge_mut(&mut s.border_width, side) = v;
                }
            }
        }
    }
}
fn copy_property(s: &mut ComputedStyle, p: &ComputedStyle, name: &str) {
    match name {
        "display" => {
            s.display = p.display;
            s.flow_root = p.flow_root;
            s.list_item = p.list_item;
        }
        "float" => s.float = p.float.clone(),
        "clear" => s.clear = p.clear.clone(),
        "width" => s.width = p.width,
        "height" => s.height = p.height,
        "min-width" => s.min_width = p.min_width,
        "min-height" => s.min_height = p.min_height,
        "max-width" => s.max_width = p.max_width,
        "max-height" => s.max_height = p.max_height,
        "color" => s.color = p.color,
        "background-color" => s.background_color = p.background_color,
        "font-size" => s.font_size = p.font_size,
        "font-weight" => s.font_weight = p.font_weight,
        "font-family" => s.font_family = p.font_family.clone(),
        "font-style" => s.font_style = p.font_style.clone(),
        "line-height" => s.line_height = p.line_height,
        "text-align" => s.text_align = p.text_align.clone(),
        "white-space" => s.white_space = p.white_space.clone(),
        "text-decoration" => s.text_decoration = p.text_decoration.clone(),
        "list-style-type" => s.list_style_type = p.list_style_type.clone(),
        "list-style-position" => s.list_style_position = p.list_style_position.clone(),
        "vertical-align" => s.vertical_align = p.vertical_align.clone(),
        "border-radius" => s.border_radius = p.border_radius,
        "position" => s.position = p.position.clone(),
        "z-index" => s.z_index = p.z_index,
        "overflow" | "overflow-x" | "overflow-y" => s.overflow = p.overflow.clone(),
        "opacity" => s.opacity = p.opacity,
        "box-sizing" => s.box_sizing = p.box_sizing.clone(),
        "flex-direction" => s.flex_direction = p.flex_direction.clone(),
        "flex-wrap" => s.flex_wrap = p.flex_wrap.clone(),
        "justify-content" => s.justify_content = p.justify_content.clone(),
        "align-items" => s.align_items = p.align_items.clone(),
        "align-self" => s.align_self = p.align_self.clone(),
        "order" => s.order = p.order,
        "flex-grow" => s.flex_grow = p.flex_grow,
        "flex-shrink" => s.flex_shrink = p.flex_shrink,
        "flex-basis" => s.flex_basis = p.flex_basis,
        "row-gap" => {
            s.row_gap = p.row_gap;
            s.gap = p.gap;
        }
        "column-gap" => s.column_gap = p.column_gap,
        "grid-template-columns" => s.grid_template_columns = p.grid_template_columns.clone(),
        "grid-template-rows" => s.grid_template_rows = p.grid_template_rows.clone(),
        "grid-auto-columns" => s.grid_auto_columns = p.grid_auto_columns.clone(),
        "grid-auto-rows" => s.grid_auto_rows = p.grid_auto_rows.clone(),
        "grid-column-start" => s.grid_column_start = p.grid_column_start,
        "grid-column-end" => s.grid_column_end = p.grid_column_end,
        "grid-row-start" => s.grid_row_start = p.grid_row_start,
        "grid-row-end" => s.grid_row_end = p.grid_row_end,
        "grid-auto-flow" => s.grid_auto_flow = p.grid_auto_flow.clone(),
        "justify-items" => s.justify_items = p.justify_items.clone(),
        "justify-self" => s.justify_self = p.justify_self.clone(),
        "align-content" => s.align_content = p.align_content.clone(),
        "top" => s.top = p.top,
        "right" => s.right = p.right,
        "bottom" => s.bottom = p.bottom,
        "left" => s.left = p.left,
        _ => {
            if let Some(side) = name.strip_prefix("margin-") {
                let mut e = p.margin;
                *edge_mut(&mut s.margin, side) = *edge_mut(&mut e, side);
            } else if let Some(side) = name.strip_prefix("padding-") {
                let mut e = p.padding;
                *edge_mut(&mut s.padding, side) = *edge_mut(&mut e, side);
            } else if let Some(rest) = name.strip_prefix("border-") {
                if let Some(side) = rest.strip_suffix("-width") {
                    let mut e = p.border_width;
                    *edge_mut(&mut s.border_width, side) = *edge_mut(&mut e, side);
                } else if rest.ends_with("-color") {
                    s.border_color = p.border_color;
                }
            }
        }
    }
}
fn finite_number(s: &str) -> Option<f32> {
    s.parse::<f32>()
        .ok()
        .filter(|v| v.is_finite() && v.abs() <= 1_000_000.0)
}
pub fn parse_length(
    value: &str,
    font: f32,
    root_font: f32,
    width: f32,
    height: f32,
) -> Option<Length> {
    // Nested arithmetic cannot consume unbounded native stack space.
    if value.len() > 4096 {
        return None;
    }
    let value = value.trim().to_ascii_lowercase();
    if value.matches("calc(").take(17).count() > 16 {
        return None;
    }
    if matches!(
        value.as_str(),
        "auto" | "none" | "normal" | "max-content" | "min-content" | "fit-content"
    ) {
        return Some(Length::Auto);
    }
    if let Some(expression) = value
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_calc(expression, font, root_font, width, height);
    }
    for (unit, scale) in [
        ("rem", root_font),
        ("em", font),
        ("ex", font * 0.5),
        ("ch", font * 0.5),
        ("vw", width / 100.0),
        ("vh", height / 100.0),
        ("vmin", width.min(height) / 100.0),
        ("vmax", width.max(height) / 100.0),
        ("dvw", width / 100.0),
        ("dvh", height / 100.0),
        ("svw", width / 100.0),
        ("svh", height / 100.0),
        ("lvw", width / 100.0),
        ("lvh", height / 100.0),
        ("px", 1.0),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
    ] {
        if let Some(n) = value.strip_suffix(unit) {
            return finite_number(n)
                .map(|n| Length::Px((n * scale).clamp(-1_000_000.0, 1_000_000.0)));
        }
    }
    if let Some(n) = value.strip_suffix('%') {
        return finite_number(n).map(Length::Percent);
    }
    finite_number(&value).map(Length::Px)
}
fn parse_calc(
    expression: &str,
    font: f32,
    root_font: f32,
    width: f32,
    height: f32,
) -> Option<Length> {
    let terms = words(expression);
    if terms.len() == 1 {
        return parse_length(terms[0], font, root_font, width, height);
    }
    let mut total = 0.0;
    let mut percent = None;
    let mut sign = 1.0;
    for term in terms {
        if term == "+" {
            sign = 1.0;
            continue;
        }
        if term == "-" {
            sign = -1.0;
            continue;
        }
        let (v, is_percent) = match parse_length(term, font, root_font, width, height)? {
            Length::Px(v) => (v, false),
            Length::Percent(v) => (v, true),
            Length::Auto | Length::Fr(_) => return None,
        };
        if percent.is_some_and(|p| p != is_percent) {
            return None;
        }
        percent = Some(is_percent);
        total += sign * v;
        sign = 1.0;
    }
    if !total.is_finite() {
        return None;
    }
    Some(if percent == Some(true) {
        Length::Percent(total)
    } else {
        Length::Px(total)
    })
}
fn parse_grid_line(value: &str) -> Option<GridLine> {
    let lower = value.trim().to_ascii_lowercase();
    if lower == "auto" {
        return Some(GridLine::Auto);
    }
    let parts = words(&lower);
    if parts.len() == 2 && parts.contains(&"span") {
        let n = parts
            .iter()
            .find(|v| **v != "span")?
            .parse::<usize>()
            .ok()?;
        return (n > 0).then_some(GridLine::Span(n.min(256)));
    }
    let n = lower.parse::<i32>().ok()?;
    (n != 0).then_some(GridLine::Line(n))
}
fn valid_grid_auto_flow(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let parts = words(&lower);
    (1..=2).contains(&parts.len())
        && parts
            .iter()
            .all(|v| matches!(*v, "row" | "column" | "dense"))
        && parts
            .iter()
            .filter(|v| matches!(**v, "row" | "column"))
            .count()
            <= 1
        && parts.iter().filter(|v| **v == "dense").count() <= 1
}
fn parse_grid_tracks(
    value: &str,
    font: f32,
    root: f32,
    width: f32,
    height: f32,
) -> Option<Vec<GridTrack>> {
    fn breadth(value: &str, units: [f32; 4]) -> Option<GridBreadth> {
        match value.trim() {
            "auto" => Some(GridBreadth::Auto),
            "min-content" => Some(GridBreadth::MinContent),
            "max-content" => Some(GridBreadth::MaxContent),
            v => {
                let length = if let Some(n) = v.strip_suffix("fr") {
                    Length::Fr(finite_number(n)?)
                } else {
                    parse_length(v, units[0], units[1], units[2], units[3])?
                };
                match length {
                    Length::Px(n) | Length::Percent(n) | Length::Fr(n) if n >= 0.0 => {
                        Some(GridBreadth::Length(length))
                    }
                    _ => None,
                }
            }
        }
    }
    fn list(value: &str, units: [f32; 4], depth: usize) -> Option<Vec<GridTrack>> {
        if depth > 8 {
            return None;
        }
        let mut tracks = Vec::new();
        for token in words(value) {
            if let Some(inner) = token
                .strip_prefix("repeat(")
                .and_then(|s| s.strip_suffix(')'))
            {
                if depth > 0 {
                    return None;
                }
                let args = split_top_level(inner, ',');
                if args.len() != 2 {
                    return None;
                }
                let count = args[0].trim().parse::<usize>().ok()?.min(64);
                if count == 0 {
                    return None;
                }
                let repeated = list(args[1].trim(), units, depth + 1)?;
                for _ in 0..count {
                    tracks.extend(
                        repeated
                            .iter()
                            .copied()
                            .take(64usize.saturating_sub(tracks.len())),
                    );
                }
            } else if let Some(inner) = token
                .strip_prefix("minmax(")
                .and_then(|s| s.strip_suffix(')'))
            {
                let args = split_top_level(inner, ',');
                if args.len() != 2 {
                    return None;
                }
                let min = breadth(args[0], units)?;
                let max = breadth(args[1], units)?;
                if matches!(min, GridBreadth::Length(Length::Fr(_))) {
                    return None;
                }
                if tracks.len() < 64 {
                    tracks.push(GridTrack { min, max });
                }
            } else {
                let track = GridTrack::single(breadth(token, units)?);
                if tracks.len() < 64 {
                    tracks.push(track);
                }
            }
        }
        (!tracks.is_empty()).then_some(tracks)
    }
    let value = value.trim().to_ascii_lowercase();
    if value == "none" {
        return Some(Vec::new());
    }
    list(&value, [font, root, width, height], 0)
}

pub fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim().to_ascii_lowercase();
    if value == "transparent" {
        return Some(Color::TRANSPARENT);
    }
    if let Some(hex) = value.strip_prefix('#') {
        let digit = |s: &str| u8::from_str_radix(s, 16).ok();
        return match hex.len() {
            3 | 4 => {
                if !hex.is_ascii() {
                    return None;
                }
                Some(Color::rgba(
                    digit(&hex[0..1])? * 17,
                    digit(&hex[1..2])? * 17,
                    digit(&hex[2..3])? * 17,
                    if hex.len() == 4 {
                        digit(&hex[3..4])? * 17
                    } else {
                        255
                    },
                ))
            }
            6 | 8 => {
                if !hex.is_ascii() {
                    return None;
                }
                Some(Color::rgba(
                    digit(&hex[0..2])?,
                    digit(&hex[2..4])?,
                    digit(&hex[4..6])?,
                    if hex.len() == 8 {
                        digit(&hex[6..8])?
                    } else {
                        255
                    },
                ))
            }
            _ => None,
        };
    }
    if let Some(inner) = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("rgba("))
        .and_then(|v| v.strip_suffix(')'))
    {
        let normalized = inner.replace([',', '/'], " ");
        let parts: Vec<_> = normalized.split_ascii_whitespace().collect();
        if parts.len() < 3 || parts.len() > 4 {
            return None;
        }
        let channel = |v: &str| -> Option<u8> {
            let n = if let Some(p) = v.strip_suffix('%') {
                finite_number(p)? * 2.55
            } else {
                finite_number(v)?
            };
            Some(n.round().clamp(0.0, 255.0) as u8)
        };
        return Some(Color::rgba(
            channel(parts[0])?,
            channel(parts[1])?,
            channel(parts[2])?,
            parts
                .get(3)
                .map(|v| alpha(v))
                .transpose_option()?
                .unwrap_or(255),
        ));
    }
    if let Some(inner) = value
        .strip_prefix("hsl(")
        .or_else(|| value.strip_prefix("hsla("))
        .and_then(|v| v.strip_suffix(')'))
    {
        let normalized = inner.replace([',', '/'], " ");
        let p: Vec<_> = normalized.split_ascii_whitespace().collect();
        if p.len() < 3 || p.len() > 4 {
            return None;
        }
        let h = if let Some(v) = p[0].strip_suffix("turn") {
            finite_number(v)? * 360.0
        } else if let Some(v) = p[0].strip_suffix("rad") {
            finite_number(v)?.to_degrees()
        } else {
            finite_number(p[0].trim_end_matches("deg"))?
        };
        let sat = (finite_number(p[1].strip_suffix('%')?)? / 100.0).clamp(0.0, 1.0);
        let light = (finite_number(p[2].strip_suffix('%')?)? / 100.0).clamp(0.0, 1.0);
        let c = (1.0 - (2.0 * light - 1.0).abs()) * sat;
        let hp = h.rem_euclid(360.0) / 60.0;
        let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
        let (r, g, b) = match hp as u8 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = light - c / 2.0;
        return Some(Color::rgba(
            ((r + m) * 255.0).round() as u8,
            ((g + m) * 255.0).round() as u8,
            ((b + m) * 255.0).round() as u8,
            p.get(3)
                .map(|v| alpha(v))
                .transpose_option()?
                .unwrap_or(255),
        ));
    }
    let hex = match value.as_str() {
        "aliceblue" => 0xf0f8ff,
        "antiquewhite" => 0xfaebd7,
        "aqua" | "cyan" => 0x00ffff,
        "aquamarine" => 0x7fffd4,
        "azure" => 0xf0ffff,
        "beige" => 0xf5f5dc,
        "bisque" => 0xffe4c4,
        "black" => 0x000000,
        "blanchedalmond" => 0xffebcd,
        "blue" => 0x0000ff,
        "blueviolet" => 0x8a2be2,
        "brown" => 0xa52a2a,
        "burlywood" => 0xdeb887,
        "cadetblue" => 0x5f9ea0,
        "chartreuse" => 0x7fff00,
        "chocolate" => 0xd2691e,
        "coral" => 0xff7f50,
        "cornflowerblue" => 0x6495ed,
        "cornsilk" => 0xfff8dc,
        "crimson" => 0xdc143c,
        "darkblue" => 0x00008b,
        "darkcyan" => 0x008b8b,
        "darkgoldenrod" => 0xb8860b,
        "darkgray" | "darkgrey" => 0xa9a9a9,
        "darkgreen" => 0x006400,
        "darkkhaki" => 0xbdb76b,
        "darkmagenta" => 0x8b008b,
        "darkolivegreen" => 0x556b2f,
        "darkorange" => 0xff8c00,
        "darkorchid" => 0x9932cc,
        "darkred" => 0x8b0000,
        "darksalmon" => 0xe9967a,
        "darkseagreen" => 0x8fbc8f,
        "darkslateblue" => 0x483d8b,
        "darkslategray" | "darkslategrey" => 0x2f4f4f,
        "darkturquoise" => 0x00ced1,
        "darkviolet" => 0x9400d3,
        "deeppink" => 0xff1493,
        "deepskyblue" => 0x00bfff,
        "dimgray" | "dimgrey" => 0x696969,
        "dodgerblue" => 0x1e90ff,
        "firebrick" => 0xb22222,
        "floralwhite" => 0xfffaf0,
        "forestgreen" => 0x228b22,
        "fuchsia" | "magenta" => 0xff00ff,
        "gainsboro" => 0xdcdcdc,
        "ghostwhite" => 0xf8f8ff,
        "gold" => 0xffd700,
        "goldenrod" => 0xdaa520,
        "gray" | "grey" => 0x808080,
        "green" => 0x008000,
        "greenyellow" => 0xadff2f,
        "honeydew" => 0xf0fff0,
        "hotpink" => 0xff69b4,
        "indianred" => 0xcd5c5c,
        "indigo" => 0x4b0082,
        "ivory" => 0xfffff0,
        "khaki" => 0xf0e68c,
        "lavender" => 0xe6e6fa,
        "lavenderblush" => 0xfff0f5,
        "lawngreen" => 0x7cfc00,
        "lemonchiffon" => 0xfffacd,
        "lightblue" => 0xadd8e6,
        "lightcoral" => 0xf08080,
        "lightcyan" => 0xe0ffff,
        "lightgoldenrodyellow" => 0xfafad2,
        "lightgray" | "lightgrey" => 0xd3d3d3,
        "lightgreen" => 0x90ee90,
        "lightpink" => 0xffb6c1,
        "lightsalmon" => 0xffa07a,
        "lightseagreen" => 0x20b2aa,
        "lightskyblue" => 0x87cefa,
        "lightslategray" | "lightslategrey" => 0x778899,
        "lightsteelblue" => 0xb0c4de,
        "lightyellow" => 0xffffe0,
        "lime" => 0x00ff00,
        "limegreen" => 0x32cd32,
        "linen" => 0xfaf0e6,
        "maroon" => 0x800000,
        "mediumaquamarine" => 0x66cdaa,
        "mediumblue" => 0x0000cd,
        "mediumorchid" => 0xba55d3,
        "mediumpurple" => 0x9370db,
        "mediumseagreen" => 0x3cb371,
        "mediumslateblue" => 0x7b68ee,
        "mediumspringgreen" => 0x00fa9a,
        "mediumturquoise" => 0x48d1cc,
        "mediumvioletred" => 0xc71585,
        "midnightblue" => 0x191970,
        "mintcream" => 0xf5fffa,
        "mistyrose" => 0xffe4e1,
        "moccasin" => 0xffe4b5,
        "navajowhite" => 0xffdead,
        "navy" => 0x000080,
        "oldlace" => 0xfdf5e6,
        "olive" => 0x808000,
        "olivedrab" => 0x6b8e23,
        "orange" => 0xffa500,
        "orangered" => 0xff4500,
        "orchid" => 0xda70d6,
        "palegoldenrod" => 0xeee8aa,
        "palegreen" => 0x98fb98,
        "paleturquoise" => 0xafeeee,
        "palevioletred" => 0xdb7093,
        "papayawhip" => 0xffefd5,
        "peachpuff" => 0xffdab9,
        "peru" => 0xcd853f,
        "pink" => 0xffc0cb,
        "plum" => 0xdda0dd,
        "powderblue" => 0xb0e0e6,
        "purple" => 0x800080,
        "rebeccapurple" => 0x663399,
        "red" => 0xff0000,
        "rosybrown" => 0xbc8f8f,
        "royalblue" => 0x4169e1,
        "saddlebrown" => 0x8b4513,
        "salmon" => 0xfa8072,
        "sandybrown" => 0xf4a460,
        "seagreen" => 0x2e8b57,
        "seashell" => 0xfff5ee,
        "sienna" => 0xa0522d,
        "silver" => 0xc0c0c0,
        "skyblue" => 0x87ceeb,
        "slateblue" => 0x6a5acd,
        "slategray" | "slategrey" => 0x708090,
        "snow" => 0xfffafa,
        "springgreen" => 0x00ff7f,
        "steelblue" => 0x4682b4,
        "tan" => 0xd2b48c,
        "teal" => 0x008080,
        "thistle" => 0xd8bfd8,
        "tomato" => 0xff6347,
        "turquoise" => 0x40e0d0,
        "violet" => 0xee82ee,
        "wheat" => 0xf5deb3,
        "white" => 0xffffff,
        "whitesmoke" => 0xf5f5f5,
        "yellow" => 0xffff00,
        "yellowgreen" => 0x9acd32,
        _ => return None,
    };
    Some(Color::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8))
}
fn alpha(s: &str) -> Option<u8> {
    let v = if let Some(p) = s.strip_suffix('%') {
        finite_number(p)? / 100.0
    } else {
        finite_number(s)?
    };
    Some((v.clamp(0.0, 1.0) * 255.0).round() as u8)
}
trait TransposeOption<T> {
    fn transpose_option(self) -> Option<Option<T>>;
}
impl<T> TransposeOption<T> for Option<Option<T>> {
    fn transpose_option(self) -> Option<Option<T>> {
        match self {
            None => Some(None),
            Some(Some(v)) => Some(Some(v)),
            Some(None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn supports_comment_tokens_agree_across_queries_blocks_and_actual_selectors() {
        for (selector, valid) in [
            ("div/**/.x", true),
            ("div /**/.x", true),
            ("./**/x", true),
            ("div/**/span", false),
            ("#/**/id", false),
            (":/**/IS(.x)", true),
            (":is/**/(.x)", false),
            ("[x~/**/=a]", true),
            ("[x~ /**/=a]", false),
            ("[x=a/**/S]", true),
            ("p:nth-child(2n/**/+/**/1)", true),
            ("p:nth-child(2/**/n+1)", false),
            (":empty(p)", false),
            (r#"[x="/* ) ] #fake */"]"#, true),
        ] {
            let query = format!("selector({selector})");
            assert_eq!(supports_matches(&query), valid, "{query}");
            assert_eq!(
                supports_matches(&format!("not/**/{query}")),
                !valid,
                "{query}"
            );
            let doc = Document::parse("<p id=x></p>");
            let sheet = format!("#x{{color:red}} @supports {query} {{#x{{color:green}}}}");
            let styles = compute_styles(&doc, &[sheet], 400.0, 300.0);
            assert_eq!(
                styles[doc.query_selector("#x").unwrap()].color,
                if valid {
                    Color::rgb(0, 128, 0)
                } else {
                    Color::rgb(255, 0, 0)
                },
                "{query}"
            );
        }
        assert!(supports_matches("selector(div) /* unterminated"));
        assert!(supports_matches(
            r#"selector(div/* ) } '" */span) or (display:flex)"#
        ));
        assert!(supports_matches("(display:flex)and/**/selector(./**/x)"));
        assert!(supports_matches(
            "unknown(url(foo/**/bar)) or selector(div)"
        ));
        assert!(!supports_matches("not selector(svg/**/|rect)"));
        let doc = Document::parse(
            r#"<div id=own class=x data-v="] #fake"><span id=child class=x></span></div>"#,
        );
        let sheet = r#".x{color:red} div/**/.x{color:green} div /**/.x{background:blue}
            [data-v="] #fake"]{width:10px} #own{width:20px}"#;
        let styles = compute_styles(&doc, &[sheet.into()], 400.0, 300.0);
        let own = &styles[doc.query_selector("#own").unwrap()];
        let child = &styles[doc.query_selector("#child").unwrap()];
        assert_eq!(own.color, Color::rgb(0, 128, 0));
        assert_eq!(child.color, Color::rgb(255, 0, 0));
        assert_eq!(child.background_color, Color::rgb(0, 0, 255));
        assert_eq!(own.width, Length::Px(20.0));
        assert_eq!(specificity("div/**/.x"), specificity("div.x"));
        assert_eq!(specificity(r#":is([data-v=") #fake"],.x)"#), 1024);
        assert_eq!(specificity(r#"[data-v="] #fake"]"#), 1024);
        assert_eq!(specificity("div/**/span"), 0);
    }

    #[test]
    fn supports_review_regressions_preserve_raw_comments_namespaces_and_capabilities() {
        let long_prefix = "x".repeat(1100);
        for query in [
            "(position:sticky)".to_owned(),
            "selector(div/**/span)".into(),
            r"s\65 lector(div/**/span)".into(),
            format!("not selector({long_prefix}|rect)"),
            format!("(display:flex) or selector(a[{long_prefix}|href])"),
        ] {
            assert!(!supports_matches(&query), "{query}");
            let doc = Document::parse("<p id=x>text</p>");
            let source = format!("#x{{color:green}} @supports {query} {{#x{{color:red}}}}");
            let style =
                &compute_styles(&doc, &[source], 400.0, 300.0)[doc.query_selector("#x").unwrap()];
            assert_eq!(style.color, Color::rgb(0, 128, 0), "{query}");
        }
        assert!(supports_matches(r#"selector([data-value="/*literal*/"] )"#));
        assert!(supports_matches("not (position:sticky)"));
        let doc = Document::parse("<p id=x>text</p>");
        let source = r#"/* } ) " ' */ @layer before;
            @supports /* } ) " ' */ (display:flex) {
                /* } ) " ' */ @layer after {#x{color:green /* } */}}
            }
            @layer before {#x {color:red}}
            @supports selector(div/**/span) {@layer false;}
            @layer tail, false;
            @layer tail {#x{background:red}}
            @layer false {#x{background:green}}
            @media (width >= 100px) {#x{width:20px/*) } "*/}}
            #x{height:30px; /* unterminated } "
        "#;
        let style = &compute_styles(&doc, &[source.into()], 400.0, 300.0)
            [doc.query_selector("#x").unwrap()];
        assert_eq!(style.color, Color::rgb(0, 128, 0));
        assert_eq!(style.background_color, Color::rgb(0, 128, 0));
        assert_eq!(style.width, Length::Px(20.0));
        assert_eq!(style.height, Length::Px(30.0));
    }

    #[test]
    fn summary_markers_use_first_html_summary_and_author_display_and_list_overrides() {
        let doc = Document::parse(
            "<details id=d><div>before</div><summary id=first><b id=child>Summary</b></summary><summary id=second>Second</summary></details><details open><summary id=open>Open</summary></details><summary id=outside>Outside</summary><ul><li id=li>Item</li></ul>",
        );
        let rules = "#first{list-style:none inside} #open{display:block} #li{list-style:square inside} #second{display:list-item;list-style:disclosure-open outside} #child{list-style-position:inherit}";
        let styles = compute_styles(&doc, &[rules.into()], 400.0, 300.0);
        let style = |id| &styles[doc.query_selector(id).unwrap()];
        assert!(style("#first").list_item);
        assert_eq!(style("#first").list_style_type, "none");
        assert_eq!(style("#first").list_style_position, "inside");
        assert!(!style("#child").list_item);
        assert_eq!(style("#child").list_style_position, "inside");
        assert!(!style("#open").list_item);
        assert_eq!(style("#open").list_style_type, "disclosure-open");
        assert!(!style("#outside").list_item);
        assert!(style("#li").list_item);
        assert_eq!(style("#li").list_style_type, "square");
        assert_eq!(style("#second").list_style_type, "disclosure-open");
        assert_eq!(style("#second").list_style_position, "outside");
        let styles = compute_styles(&doc, &[], 400.0, 300.0);
        assert_eq!(
            styles[doc.query_selector("#first").unwrap()].list_style_type,
            "disclosure-closed"
        );
        assert!(!styles[doc.query_selector("#second").unwrap()].list_item);
    }
    #[test]
    fn list_style_shorthand_global_values_and_invalid_pairs_are_atomic() {
        let doc = Document::parse(
            "<ul style='list-style:circle inside'><li id=a style='list-style:inherit'>a</li><li id=b style='display:block;display:inherit;list-style:initial'>b</li><li id=c style='list-style:square inside;list-style:none bogus;list-style:inside outside'>c</li></ul>",
        );
        let styles = compute_styles(&doc, &[], 400.0, 300.0);
        let a = &styles[doc.query_selector("#a").unwrap()];
        assert_eq!(a.list_style_type, "circle");
        assert_eq!(a.list_style_position, "inside");
        let b = &styles[doc.query_selector("#b").unwrap()];
        assert!(!b.list_item);
        assert_eq!(b.list_style_type, "disc");
        assert_eq!(b.list_style_position, "outside");
        let c = &styles[doc.query_selector("#c").unwrap()];
        assert_eq!(c.list_style_type, "square");
        assert_eq!(c.list_style_position, "inside");
    }

    #[test]
    fn supports_conditions_preserve_boolean_grammar_and_unknown_negation() {
        for query in [
            "(display:flex)",
            "(display:flex) and (width:1px)",
            "(display:flex) or (unknown:value)",
            "not (unknown:value)",
            "((display:flex) or (display:subgrid)) and (opacity:.5)",
            "not ((display:subgrid) and (color:red))",
            "not future(foo)",
            "(display:flex)/**/and/**/(color:red)",
            "(DISPLAY:flex) AND (COLOR:RED)",
            "(display:flex !important)",
            "(display:flex ! IMPORTANT)",
            "(display:revert-layer)",
            "future(url(abc)) or (display:flex)",
        ] {
            assert!(supports_matches(query), "{query}");
        }
        for query in [
            "",
            "display:flex",
            "(display:flex",
            "display:flex)",
            "not not (display:flex)",
            "(display:flex) and (color:red) or (width:1px)",
            "(display:flex) or (color:red) and (width:1px)",
            "not (display:flex) or (color:red)",
            "(display:flex), (color:red)",
            "(display:flex) and",
            "(display:flex) garbage",
            "url(foo) or (display:flex)",
            "future(url(foo bar)) or (display:flex)",
            "not future(url(foo bar))",
            "not (display:flex",
            "not ((display:flex) and)",
        ] {
            assert!(!supports_matches(query), "{query}");
        }
    }
    #[test]
    fn supports_declarations_use_strict_positive_value_grammar() {
        for query in [
            "(width:0)",
            "(width:1e2px)",
            "(width:10%)",
            "(height:auto)",
            "(min-width:0)",
            "(max-height:none)",
            "(margin:-1px auto 2em 3%)",
            "(padding:0 1px)",
            "(inset:0 auto)",
            "(gap:normal 1rem)",
            "(position:FIXED)",
            "(flex-wrap:wrap-reverse)",
            "(box-sizing:border-box)",
            "(flex-grow:0.5)",
            "(order:-3)",
            "(z-index:auto)",
            "(opacity:2)",
            "(color:rgba(20, 30, 40, .5))",
            "(color:rgb(10% 20% 30% / 20%))",
            "(color:hsl(1turn, 50%, 30%))",
            "(color:HSLA(120deg 50% 40% / .5))",
            "(color:currentcolor)",
            "(background:#ff000080)",
            "(grid-template-columns:repeat(2,minmax(10px,1fr) 20%))",
            "(grid-auto-rows:min-content 1fr)",
            "(grid-auto-flow:column dense)",
            "(grid-column-start:span 3)",
            "(grid-row-end:-2)",
        ] {
            assert!(supports_matches(query), "{query}");
        }
        for query in [
            "(padding:auto)",
            "(padding:-1px)",
            "(width:-1px)",
            "(width:20)",
            "(width:1.px)",
            "(width:1 px)",
            "(width:NaNpx)",
            "(width:none)",
            "(width:min-content)",
            "(padding:1px 2px 3px 4px 5px)",
            "(margin:1px junk)",
            "(gap:-1px)",
            "(flex-grow:-1)",
            "(order:1.0)",
            "(z-index:1e2)",
            "(opacity:20%)",
            "(display:contents)",
            "(display:subgrid)",
            "(display:flex garbage)",
            "(display:flex !garbage)",
            "(display:flex; color:red)",
            "(color:rgb(1,,2,3))",
            "(color:rgb(1 2 3 0.5))",
            "(color:rgb(1%,2,3))",
            "(color:rgb(1 / 2 / 3))",
            "(color:hsl(1deg 2 3))",
            "(color:lab(30% 0 0))",
            "(background:linear-gradient(red,blue))",
            "(background:red garbage)",
            "(grid-template-columns:subgrid)",
            "(grid-template-columns:10)",
            "(grid-template-columns:minmax(1fr,2fr))",
            "(grid-auto-rows:repeat(2,1fr))",
            "(grid-column-start:0)",
            "(grid-column-start:span -1)",
            "(--custom:anything)",
            "(width:var(--size))",
            "(width:calc(1px + 2px))",
            "(transform:rotate(1deg))",
            "(display:FLEX)",
            r"(d\69 splay:flex)",
        ] {
            assert!(!supports_matches(query), "{query}");
        }
    }
    #[test]
    fn supports_selector_is_structural_and_recursively_unforgiving() {
        for selector in [
            "missing",
            "*",
            "div.missing#absent",
            "main > div + p ~ a",
            "article .child",
            "[data-x]",
            "[data-x='value']",
            "[data-x^=prefix i]",
            "[lang|=en]",
            "div:first-child",
            "DIV:FIRST-CHILD",
            "div:nth-child(2n + 1)",
            "div:nth-last-of-type(-n+3)",
            "div:not(.a,.b)",
            ":is(div,p):where(.a,.b)",
            ":not(:is(.a,.b))",
            ":root",
            ":empty",
        ] {
            assert!(
                supports_matches(&format!("selector({selector})")),
                "{selector}"
            );
        }
        for selector in [
            "",
            "div,p",
            "> div",
            "div >",
            "div > > p",
            "#",
            ".123",
            "div..a",
            "[x==foo]",
            "[x='foo' bogus]",
            "[x=123]",
            "div::before",
            "div::marker",
            "div:has(p)",
            "div:hover",
            "div:scope",
            "div:first-child()",
            "div:nth-child(2 n)",
            "div:nth-child(n 1)",
            "div:nth-child(2n of .x)",
            "svg|rect",
            "*|rect",
            ":is(div,:unknown)",
            ":not(:unknown)",
            ":where(div,)",
            r".a\62",
        ] {
            assert!(
                !supports_matches(&format!("selector({selector})")),
                "{selector}"
            );
        }
        assert!(supports_matches("not selector(:has(p))"));
        assert!(!supports_matches("not selector(svg|rect)"));
        assert!(!supports_matches(
            "(display:flex) or selector(a[xlink|href])"
        ));
        assert!(supports_matches("selector(.absent) and (display:grid)"));
    }
    #[test]
    fn supports_limits_never_become_matches_through_not_or_short_circuit() {
        let huge = "x".repeat(MAX_SUPPORTS_BYTES + 1);
        assert!(!supports_matches(&huge));
        assert!(!supports_matches(&format!("not ({huge})")));
        let deep = format!("not {}(unknown:value){}", "(".repeat(30), ")".repeat(30));
        assert!(!supports_matches(&deep));
        let many = vec!["(display:flex)"; MAX_SUPPORTS_TERMS + 1].join(" or ");
        assert!(!supports_matches(&many));
        let selectors = vec!["div"; 66].join(" ");
        assert!(!supports_matches(&format!("not selector({selectors})")));
        let mut work = 1;
        assert!(!supports_matches_with_budget(
            "not (unknown:value)",
            false,
            &mut work
        ));
        assert_eq!(work, 0);
        let mut work = 500;
        let mut matches = 0;
        for _ in 0..20 {
            matches += usize::from(supports_matches_with_budget(
                "(display:flex)",
                false,
                &mut work,
            ));
        }
        assert!(matches > 0 && matches < 20);
        assert_eq!(work, 0);
        let mut work = MAX_SUPPORTS_WORK;
        assert!(supports_matches_with_budget(
            "display:flex",
            true,
            &mut work
        ));
        assert!(!supports_matches_with_budget(
            "display:flex",
            false,
            &mut work
        ));
    }
    #[test]
    fn supports_controls_layers_with_actual_computed_values() {
        let doc = Document::parse("<div id=x></div>");
        let source = "@supports (padding:auto) {@layer rejected;} @supports selector(#absent) and (display:grid) {@layer accepted;} @layer later,accepted,rejected; @layer rejected {#x{color:red}} @layer later {#x{color:green}} @supports ((width:2em) and (color:rgb(1 2 3))) {#x{width:2em;background:rgb(1 2 3);opacity:.5}}";
        let style = &compute_styles(&doc, &[source.into()], 800.0, 600.0)
            [doc.query_selector("#x").unwrap()];
        assert_eq!(style.color, Color::rgb(255, 0, 0));
        assert_eq!(style.width, Length::Px(32.0));
        assert_eq!(style.background_color, Color::rgb(1, 2, 3));
        assert_eq!(style.opacity, 0.5);
    }

    use super::*;
    fn layered_style(css: &str, inline: &str) -> ComputedStyle {
        let doc = Document::parse(&format!("<div id=target style='{inline}'>text</div>"));
        compute_styles(&doc, &[css.into()], 800.0, 600.0)[doc.query_selector("#target").unwrap()]
            .clone()
    }

    #[test]
    fn layer_syntax_handles_escaped_at_rules_eof_and_invalid_statements_atomically() {
        let doc = Document::parse("<div></div>");
        let sources = [
            r"@\6c ayer first,second".into(),
            "@layer second{div{color:blue}} @layer first{div{color:red}}".into(),
        ];
        assert_eq!(
            compute_styles(&doc, &sources, 800.0, 600.0)[doc.query_selector("div").unwrap()].color,
            Color::rgb(0, 0, 255)
        );
        let css =
            "@layer first,initial; @layer second{div{color:blue}} @layer first{div{color:red}}";
        assert_eq!(layered_style(css, "").color, Color::rgb(255, 0, 0));
        let css = "@layer first; @layer first,second {div{display:none}} div{color:green}";
        assert_eq!(layered_style(css, "").display, Display::Block);
        assert!(layer_name("a\u{000b}.b").is_none());
        assert_eq!(
            layered_style(
                r"@layer a {div{color:red}} @layer b{div{color:\72 evert-layer}}",
                ""
            )
            .color,
            Color::rgb(255, 0, 0)
        );
        assert_eq!(
            layered_style("div{color:red}", "color:revert-layer/* comment */").color,
            Color::rgb(255, 0, 0)
        );
    }
    #[test]
    fn global_border_keywords_and_expansion_storage_have_real_computed_semantics() {
        let css = "@layer a{div{border:5px solid red}} @layer b{div{border-top-style:initial;border-left-color:initial;color:blue}}";
        let style = layered_style(css, "");
        assert_eq!(style.border_width.top, 0.0);
        let doc = Document::parse(
            "<section style='border:0 solid red'><div style='border:5px solid blue;border-style:inherit'></div></section>",
        );
        let styles = compute_styles(&doc, &[], 800.0, 600.0);
        assert_eq!(
            styles[doc.query_selector("div").unwrap()].border_width.top,
            5.0
        );
        let style = layered_style("div{border:5px solid red;border-width:initial}", "");
        assert_eq!(style.border_width.top, 3.0);
        let source = format!("all:var(--x,{});", "x".repeat(4000)).repeat(100);
        let declarations = parse_declarations(&source);
        assert!(
            declarations
                .iter()
                .map(|d| d.name.len() + d.value.len())
                .sum::<usize>()
                <= 1024 * 1024
        );
        assert!(declarations.len() <= 8192);
    }
    #[test]
    fn layers_override_specificity_and_reopening_preserves_first_order() {
        let css = "@layer first, second; @layer second {div{color:blue}} @layer first {#target{color:red}}";
        assert_eq!(layered_style(css, "").color, Color::rgb(0, 0, 255));
        assert_eq!(
            layered_style(&format!("{css} div{{color:green}}"), "").color,
            Color::rgb(0, 128, 0)
        );
        assert_eq!(
            layered_style(css, "color:purple").color,
            Color::rgb(128, 0, 128)
        );
        let doc = Document::parse("<div></div>");
        let sources = [
            "@layer first,second;".into(),
            "@layer second {div{color:blue}}".into(),
            "@layer first {div{color:red}}".into(),
        ];
        let styles = compute_styles(&doc, &sources, 800.0, 600.0);
        assert_eq!(
            styles[doc.query_selector("div").unwrap()].color,
            Color::rgb(0, 0, 255)
        );
    }
    #[test]
    fn nested_layers_have_implicit_final_sublayers_and_important_reverses() {
        let css = "@layer first,second; @layer first.inner {div{color:red}} @layer first {div{color:green}} @layer second {div{color:blue}}";
        assert_eq!(layered_style(css, "").color, Color::rgb(0, 0, 255));
        let first_only = "@layer first.inner {div{color:red}} @layer first {div{color:green}}";
        assert_eq!(layered_style(first_only, "").color, Color::rgb(0, 128, 0));
        let important = css
            .replace("color:red", "color:red!important")
            .replace("color:green", "color:green!important")
            .replace("color:blue", "color:blue!important");
        assert_eq!(
            layered_style(
                &format!("{important} #target{{color:black!important}}"),
                "color:yellow"
            )
            .color,
            Color::rgb(255, 0, 0)
        );
        assert_eq!(
            layered_style(&important, "color:purple!important").color,
            Color::rgb(128, 0, 128)
        );
    }
    #[test]
    fn layer_names_decode_escapes_and_keep_case_and_anonymous_identity() {
        let css = r"@layer \66 irst, Second; @layer Second {div{color:blue}} @layer first {div{color:red}} @layer second {div{color:green}}";
        assert_eq!(layered_style(css, "").color, Color::rgb(0, 128, 0));
        let css = r"@layer a\.b, a.b; @layer a.b {div{color:blue}} @layer a\.b {div{color:red}}";
        assert_eq!(layered_style(css, "").color, Color::rgb(0, 0, 255));
        assert!(CascadeLayer::named(None, "outer/**/.inner").is_some());
        assert!(CascadeLayer::named(None, "foo/**/bar").is_none());
        assert!(CascadeLayer::named(None, "default").is_some());
        assert!(CascadeLayer::named(None, "revert-rule").is_none());
        assert_eq!(
            layered_style("@layer default {div{color:red}}", "").color,
            Color::rgb(255, 0, 0)
        );
        for invalid in [
            "",
            "initial",
            "x.REVERT-LAYER",
            "a..b",
            "1x",
            "-1x",
            "a, b",
            "a b",
            "a.",
            "a\\\n",
        ] {
            assert!(CascadeLayer::named(None, invalid).is_none(), "{invalid:?}");
        }
        let shared = CascadeLayer::anonymous(None).unwrap();
        let make = |source, layer| StyleSource {
            source: Arc::from(source),
            layer: Some(layer),
            media: vec![],
        };
        let sources = [
            make("@layer theme {div{color:red}}", shared.clone()),
            make("div{color:blue}", CascadeLayer::anonymous(None).unwrap()),
            make("@layer theme {div{color:green}}", shared),
        ];
        let doc = Document::parse("<div></div>");
        assert_eq!(
            compute_styles_from_sources(&doc, &sources, 800.0, 600.0)
                [doc.query_selector("div").unwrap()]
            .color,
            Color::rgb(0, 0, 255)
        );
    }
    #[test]
    fn conditional_layers_register_only_under_matching_global_conditions() {
        for conditional in ["@media print", "@supports (unknown: invalid)"] {
            let css = format!(
                "{conditional} {{ @layer first; }} @layer second {{div{{color:blue}}}} @layer first {{div{{color:red}}}}"
            );
            assert_eq!(layered_style(&css, "").color, Color::rgb(255, 0, 0));
        }
        let css = "@container (width > 10px) {@layer first {div{color:purple}}} @layer second{div{color:blue}} @layer first{div{color:red}}";
        assert_eq!(layered_style(css, "").color, Color::rgb(0, 0, 255));
        let doc = Document::parse("<div></div>");
        let id = doc.query_selector("div").unwrap();
        let empty = StyleSource {
            source: "".into(),
            layer: CascadeLayer::named(None, "first"),
            media: vec!["print".into()],
        };
        let rest = StyleSource::new("@layer second{div{color:blue}} @layer first{div{color:red}}");
        assert_eq!(
            compute_styles_from_sources(&doc, &[empty.clone(), rest.clone()], 800.0, 600.0)[id]
                .color,
            Color::rgb(255, 0, 0)
        );
        let empty = StyleSource {
            media: vec!["screen".into()],
            ..empty
        };
        assert_eq!(
            compute_styles_from_sources(&doc, &[empty, rest], 800.0, 600.0)[id].color,
            Color::rgb(0, 0, 255)
        );
    }
    #[test]
    fn revert_layer_rolls_back_the_entire_layer_and_intervening_tiers() {
        let base = "@layer first {div{color:red}} @layer second {div{color:blue} #target{color:revert-layer}}";
        assert_eq!(layered_style(base, "").color, Color::rgb(255, 0, 0));
        assert_eq!(
            layered_style(&format!("{base} div{{color:green;color:revert-layer}}"), "").color,
            Color::rgb(255, 0, 0)
        );
        let important = "@layer first {div{color:red}} @layer second {div{color:blue!important;color:revert-layer!important}} @layer third {div{color:green!important}}";
        assert_eq!(
            layered_style(important, "color:purple").color,
            Color::rgb(255, 0, 0)
        );
        let earliest = "@layer first{div{color:revert-layer!important}} @layer second{div{color:blue!important}}";
        assert_eq!(
            layered_style(earliest, "").color,
            ComputedStyle::default().color
        );
        let sheet = "@layer first{div{color:red!important}} div{color:blue!important}";
        assert_eq!(
            layered_style(sheet, "color:green;color:revert-layer!important").color,
            Color::rgb(255, 0, 0)
        );
        assert_eq!(
            layered_style("div{color:blue}", "color:green;color:revert-layer").color,
            Color::rgb(0, 0, 255)
        );
        // Origin rollback ignores every author layer; UA div display remains block.
        assert_eq!(
            layered_style("@layer first{div{display:none}} div{display:revert}", "").display,
            Display::Block
        );
    }
    #[test]
    fn revert_layer_expands_shorthands_aliases_and_all_without_resetting_custom_properties() {
        let css = "@layer base {div{margin:1px 2px 3px 4px;border:5px solid red;font:italic bold 20px/2 monospace;flex:2 3 10px;background:blue;height:17px;--ink:green}} @layer theme{div{margin:99px;border:9px solid green;font:10px serif;flex:9;background:red;height:99px; margin:revert-layer;border:revert-layer;font:revert-layer;flex:revert-layer;background:revert-layer;block-size:revert-layer}}";
        let style = layered_style(css, "");
        assert_eq!(style.margin.left, Length::Px(4.0));
        assert_eq!(style.border_width.top, 5.0);
        assert_eq!(style.font_size, 20.0);
        assert_eq!(style.font_weight, 700);
        assert_eq!(style.font_style, "italic");
        assert_eq!(style.flex_grow, 2.0);
        assert_eq!(style.flex_shrink, 3.0);
        assert_eq!(style.flex_basis, Length::Px(10.0));
        assert_eq!(style.background_color, Color::rgb(0, 0, 255));
        assert_eq!(style.height, Length::Px(17.0));
        let all = "@layer base{div{width:31px;color:red}} @layer theme{div{width:99px;color:blue;--ink:green;all:revert-layer}} div{background:var(--ink)}";
        let style = layered_style(all, "");
        assert_eq!(style.width, Length::Px(31.0));
        assert_eq!(style.color, Color::rgb(255, 0, 0));
        assert_eq!(style.background_color, Color::rgb(0, 128, 0));
    }
    #[test]
    fn layered_custom_properties_and_pending_shorthands_follow_rollback() {
        let css = "@layer base{div{--ink:red;--space:1px 2px;color:var(--ink);margin:var(--space)}} @layer theme{div{--ink:blue;--ink:revert-layer;margin:var(--missing,revert-layer)}}";
        let style = layered_style(css, "");
        assert_eq!(style.color, Color::rgb(255, 0, 0));
        assert_eq!(style.margin.right, Length::Px(2.0));
        let style = layered_style(
            "@layer base{div{--ink:red}} @layer theme{div{--ink:initial}} div{color:var(--ink,blue)}",
            "",
        );
        assert_eq!(style.color, Color::rgb(0, 0, 255));
        let style = layered_style(
            "@layer base{div{color:red}} @layer theme{div{color:var(--missing,revert-layer)}}",
            "",
        );
        assert_eq!(style.color, Color::rgb(255, 0, 0));
        let doc = Document::parse("<section style='--ink:green'><div id=target></div></section>");
        let styles = compute_styles(
            &doc,
            &["@layer a{div{--ink:blue}} @layer b{div{--ink:inherit;color:var(--ink)}}".into()],
            800.0,
            600.0,
        );
        assert_eq!(
            styles[doc.query_selector("div").unwrap()].color,
            Color::rgb(0, 128, 0)
        );
    }
    #[test]
    fn layer_and_candidate_limits_are_bounded_and_do_not_promote_rejected_layers() {
        let mut registry = LayerRegistry::new();
        let mut retained = Vec::new();
        for _ in 0..MAX_LAYERS {
            let layer = CascadeLayer::anonymous(None).unwrap();
            assert!(registry.register(&layer).is_some());
            retained.push(layer);
        }
        assert!(
            registry
                .register(&CascadeLayer::anonymous(None).unwrap())
                .is_none()
        );
        assert_eq!(registry.ranks().len(), MAX_LAYERS + 1);
        let mut css = (0..MAX_LAYERS)
            .map(|i| format!("@layer n{i};"))
            .collect::<String>();
        css.push_str("@layer beyond {div{display:none}}");
        assert_eq!(layered_style(&css, "").display, Display::Block);
        let mut parent = None;
        for _ in 0..MAX_LAYER_DEPTH {
            parent = CascadeLayer::anonymous(parent);
            assert!(parent.is_some());
        }
        assert!(CascadeLayer::anonymous(parent).is_none());
        let mut cascade = CascadedProperties::default();
        for n in 0..5000 {
            let decl = Declaration {
                name: format!("--v{n}"),
                value: "x".repeat(512),
                important: false,
                pending_shorthand: None,
            };
            cascade.insert(&decl, false, 0, (0, 0, n));
        }
        assert!(cascade.count <= 4096);
        assert!(cascade.bytes <= 1024 * 1024);
        let doc = Document::parse("<div></div>");
        let source = StyleSource {
            source: "div{display:none}".into(),
            layer: None,
            media: vec!["screen".into(); 33],
        };
        assert_eq!(
            compute_styles_from_sources(&doc, &[source], 800.0, 600.0)
                [doc.query_selector("div").unwrap()]
            .display,
            Display::Block
        );
        let variables = BTreeMap::from([("--x".into(), "var(--x)var(--x)".into())]);
        let mut work = 20;
        assert!(resolve_vars("var(--x)", &variables, 0, &mut work).is_none());
        assert_eq!(work, 0);
    }
    #[test]
    fn media_types_modifiers_conjunctions_and_lists_have_explicit_grammar() {
        for query in [
            "screen",
            "all",
            "ONLY SCREEN",
            "not print",
            "not bogus",
            "screen and (min-width:20em) and (max-height:400px)",
            "(color) and (hover)",
            "(color) or (hover)",
            "print, screen and (width:400px)",
            "bogus, (height:300px)",
            "not (min-width:800px)",
            "screen /*a*/ and /*b*/ (color)",
            "screen and not (hover:none)",
            "screen), all",
        ] {
            assert!(media_matches(query, 400.0, 300.0), "must match: {query}");
        }
        for query in [
            "bogus",
            "speech",
            "tv",
            "print",
            "screenish",
            "not screen",
            "only (color)",
            "not only screen",
            "or and (color)",
            "layer",
            "screen garbage",
            "screen and",
            "screen and(color)",
            "screen (color)",
            "screen and (color) garbage",
            "(color) (hover)",
            "not (color) and (hover)",
            "not screen garbage",
            "print, bogus",
            ",print,",
            "screen and (color",
            "screen and ()",
        ] {
            assert!(
                !media_matches(query, 400.0, 300.0),
                "must not match: {query}"
            );
        }
        assert!(media_matches("", 400.0, 300.0));
    }

    #[test]
    fn media_unknown_features_and_invalid_values_remain_unknown_through_negation() {
        for feature in [
            "(unknown-feature)",
            "(max-weight:3kg)",
            "(min-width:garbage)",
            "(min-width:50%)",
            "(min-width:400)",
            "(max-height:auto)",
            "(min-height:1e999px)",
            "(orientation:diagonal)",
            "(pointer:laser)",
            "(resolution > 2dppx)",
            "(min-width)",
        ] {
            assert!(!media_matches(feature, 400.0, 300.0), "{feature}");
            assert!(
                !media_matches(&format!("not {feature}"), 400.0, 300.0),
                "not {feature}"
            );
            assert!(
                !media_matches(&format!("not screen and {feature}"), 400.0, 300.0),
                "not screen and {feature}"
            );
        }
        assert!(media_matches(
            "(min-width:0) and (width:25em) and (height:300px)",
            400.0,
            300.0
        ));
        assert!(media_matches("(orientation:portrait)", 300.0, 300.0));
        assert!(!media_matches("(orientation:landscape)", 300.0, 300.0));
        assert!(media_matches(
            "(prefers-color-scheme:light) and (pointer:fine)",
            400.0,
            300.0
        ));
    }

    #[test]
    fn media_evaluation_bounds_lists_features_and_nesting_and_keeps_cascade_conditions() {
        assert!(!media_matches(
            &format!("{}screen", "bogus,".repeat(64)),
            400.0,
            300.0
        ));
        assert!(!media_matches(
            &format!("{}(color)", "(color) and ".repeat(64)),
            400.0,
            300.0
        ));
        assert!(!media_matches(
            &format!("{}color{}", "(".repeat(17), ")".repeat(17)),
            400.0,
            300.0
        ));
        assert!(!media_matches(&" ".repeat(65_537), 400.0, 300.0));
        assert!(!media_matches("screen", f32::NAN, 300.0));
        let doc = Document::parse("<p id=x>text</p>");
        let sources=vec!["#x{color:green} @media bogus {#x{color:red}} @media not (unsupported-feature){#x{color:red}} @media (min-width:garbage){#x{color:red}} @media screen and (min-width:400px){#x{background:blue}}".to_owned()];
        let x = doc.query_selector("#x").unwrap();
        let wide = compute_styles(&doc, &sources, 400.0, 300.0);
        let narrow = compute_styles(&doc, &sources, 300.0, 300.0);
        assert_eq!(wide[x].color, Color::rgb(0, 128, 0));
        assert_eq!(narrow[x].color, Color::rgb(0, 128, 0));
        assert_eq!(wide[x].background_color, Color::rgb(0, 0, 255));
        assert_eq!(narrow[x].background_color, Color::TRANSPARENT);
    }

    #[test]
    fn media_ranges_support_exact_reversed_and_double_comparisons() {
        for query in [
            "(width = 400px)",
            "(400px = width)",
            "(width>=400px)",
            "(400px<=width)",
            "(width<=400px)",
            "(400px>=width)",
            "(width > 399.999px)",
            "(399.999px < width)",
            "(width < 400.001px)",
            "(400.001px > width)",
            "(height=300px)",
            "(300px=height)",
            "(200px < width <= 400px)",
            "(400px >= width > 200px)",
            "(400px <= width <= 400px)",
            "(400px >= width >= 400px)",
            "(1px < height < 301px)",
            "(301px > height > 1px)",
            "(width > -1px)",
            "(-1px < width)",
            "not (height <= -1px)",
            "(min-width:400px) and (max-width:400px)",
        ] {
            assert!(media_matches(query, 400.0, 300.0), "{query}");
        }
        for query in [
            "(width > 400px)",
            "(400px < width)",
            "(width < 400px)",
            "(400px > width)",
            "(height > 300px)",
            "(height < 300px)",
            "(width = 400.001px)",
            "(width:400.001px)",
            "(width:399.999px)",
            "(400px < width < 401px)",
            "(399px < width < 400px)",
            "(400px > width > 399px)",
            "(401px > width > 400px)",
            "(500px < width < 300px)",
            "(300px > width > 500px)",
            "(width = -1px)",
            "(max-width:-1px)",
        ] {
            assert!(!media_matches(query, 400.0, 300.0), "{query}");
        }
        assert!(media_matches("(width:0) and (height = -0)", 0.0, 0.0));
        assert!(!media_matches("(width) or (height)", 0.0, 0.0));
        assert!(media_matches("(400px < width < 401px)", 400.5, 300.0));
    }

    #[test]
    fn media_ranges_resolve_css_number_tokens_and_mixed_length_units() {
        for query in [
            "(20em < width <= 25rem)",
            "(25EM >= WIDTH > +2e2PX)",
            "(width = 100vw)",
            "(height = 100vh)",
            "(width = 25pc)",
            "(height = 225pt)",
            "(50ex <= width <= 50ch)",
            "(width = 100vmax)",
            "(height = 100vmin)",
            "(width = 100svw)",
            "(width = 100lvw)",
            "(width = 100dvw)",
            "(height = 100svh)",
            "(height = 100lvh)",
            "(height = 100dvh)",
            "(.1px < width < +5e2px)",
            "(0e3 <= width)",
            r"(\77 idth >= 4e2p\78)",
            r"sCrEeN \61 nd (WIDTH:400PX)",
        ] {
            assert!(media_matches(query, 400.0, 300.0), "{query}");
        }
        for query in [
            "(width=1in)",
            "(width=2.54cm)",
            "(width=25.4mm)",
            "(width=101.6q)",
            "(width=72pt)",
        ] {
            assert!(media_matches(query, 96.0, 100.0), "{query}");
        }
        // Relative media units use initial metrics, never authored font-size.
        let doc = Document::parse("<p id=x>text</p>");
        let styles = compute_styles(
            &doc,
            &["html{font-size:100px}@media (width=25em){#x{color:red}}".into()],
            400.0,
            300.0,
        );
        assert_eq!(
            styles[doc.query_selector("#x").unwrap()].color,
            Color::rgb(255, 0, 0)
        );
    }

    #[test]
    fn media_grouped_conditions_obey_precedence_and_type_restrictions() {
        for query in [
            "(width > 1000px) or (height >= 300px)",
            "((width > 1000px) or (height >= 300px)) and (color)",
            "(not (width > 1000px)) and ((hover: none) or (pointer: fine))",
            "not ((width > 1000px) or (height < 300px))",
            "screen and ((width > 1000px) or (height >= 300px))",
            "screen and not ((width > 1000px) or (height < 300px))",
            "not print and ((width > 1000px) or (color))",
            "(color)or (hover)",
            "((color))",
            r"\6e ot ((width > 1000px) \6f r (height < 300px))",
            "((width=400px) and (height=300px)), print",
        ] {
            assert!(media_matches(query, 400.0, 300.0), "{query}");
        }
        for query in [
            "(color) and (hover) or (pointer)",
            "(color) or (hover) and (pointer)",
            "not (hover:none) or (color)",
            "not (hover:none) and (color)",
            "(color) or not (hover:none)",
            "(color) and not (hover:none)",
            "screen and (color) or (hover)",
            "not screen and (hover:none) or (color)",
            "screen or (color)",
            "only ((color))",
            "not not (hover:none)",
            "(color) or",
            "or (color)",
            "(color) or(hover)",
            "(color) or () trailing",
            "(color) and ((height > 500px) or (width > 500px))",
            "not ((color) or (unknown-feature))",
        ] {
            assert!(!media_matches(query, 400.0, 300.0), "{query}");
        }
    }

    #[test]
    fn media_unknown_ranges_and_general_enclosed_use_three_valued_logic() {
        for unknown in [
            "(width > auto)",
            "(width > 40%)",
            "(width > 400)",
            "(width > 400 px)",
            "(width > 1.px)",
            "(width > .px)",
            "(width > +px)",
            "(width > 1e999px)",
            "(width > 1000001px)",
            "(width > 1000000in)",
            "(width > calc(1px + 1px))",
            "(width > NaNpx)",
            "(width > infinity)",
            "(width == 400px)",
            "(width => 400px)",
            "(width !< 400px)",
            "(width < = 500px)",
            "(500px > = width)",
            "(width >=)",
            "(min-width > 1px)",
            "(width > height)",
            "(height:1px:2px)",
            "(0 < width = 400px)",
            "(0 < width > 400px)",
            "(0 < width < 500px < 900px)",
            "(resolution >= 2dppx)",
            "(orientation > portrait)",
            "(unknown > 1px)",
            "unknown(function)",
            "(future syntax [with, commas])",
            "()",
        ] {
            assert!(!media_matches(unknown, 400.0, 300.0), "{unknown}");
            assert!(
                !media_matches(&format!("not {unknown}"), 400.0, 300.0),
                "not {unknown}"
            );
            assert!(
                !media_matches(&format!("{unknown} or (width > 1000px)"), 400.0, 300.0),
                "{unknown} OR false"
            );
            assert!(
                media_matches(&format!("{unknown} or (color)"), 400.0, 300.0),
                "{unknown} OR true"
            );
            assert!(
                !media_matches(&format!("{unknown} and (color)"), 400.0, 300.0),
                "{unknown} AND true"
            );
            assert!(
                media_matches(
                    &format!("not ({unknown} and (width > 1000px))"),
                    400.0,
                    300.0
                ),
                "NOT ({unknown} AND false)"
            );
        }
        assert!(media_matches("not print and (unknown)", 400.0, 300.0));
        assert!(media_matches("not(color) or (hover)", 400.0, 300.0));
        assert!(media_matches(
            "(not (width > 500px) and (color)) or (hover)",
            400.0,
            300.0
        ));
        assert!(!media_matches(
            "not (not (width > 500px) and (color))",
            400.0,
            300.0
        ));
    }

    #[test]
    fn media_general_enclosed_distinguishes_url_tokens_and_rejects_bad_tokens() {
        for invalid in [
            "url(foo)",
            "URL(foo)",
            "url()",
            r"\75rl(foo)",
            r"u\72l(foo)",
            "url( foo )",
            "unknown(url(foo bar))",
            "(url(foo bar))",
            "unknown(url(foo\"bar))",
            "unknown(url(foo'bar))",
            "unknown(url(foo(bar))",
            "unknown(url(foo\\\nbar))",
            "unknown(url(foo\\\r\nbar))",
            "unknown(url(foo\u{7f}bar))",
            "unknown(url(foo\u{b}bar))",
            r"unknown(\75rl(foo bar))",
            r"(URL(foo bar))",
            "unknown(\"bad\n)",
            "(\"bad\r)",
        ] {
            assert!(
                !media_matches(&format!("{invalid} or (color)"), 400.0, 300.0),
                "{invalid}"
            );
            assert!(
                !media_matches(&format!("(color) or {invalid}"), 400.0, 300.0),
                "{invalid}"
            );
            assert!(
                !media_matches(&format!("not {invalid}"), 400.0, 300.0),
                "not {invalid}"
            );
        }
        for unknown in [
            r#"url("foo")"#,
            "url( \t'foo' )",
            r#"\75rl("foo")"#,
            "unknown(url(foo))",
            "(url(foo))",
            "unknown(url())",
            "(url(  ))",
            r"unknown(url(foo\ bar))",
            r"unknown(url(foo\20 bar))",
            r"unknown(url(foo\28 bar\29))",
            r"unknown(url(foo\)bar))",
            "unknown(url(foo[bar{baz]))",
            "unknown(url(foo,bar))",
            "unknown(1url(foo bar))",
            "unknown(1.0url(foo bar))",
            "unknown(1e2url(foo bar))",
            "unknown(#url(foo bar))",
            "unknown(@url(foo bar))",
            "unknown(üurl(foo bar))",
            r#"unknown("url(foo bar)")"#,
            r#"unknown(url("foo bar"))"#,
            r"unknown(u\72l(foo))",
            r"unknown(\75\72\6c(foo))",
            r"unknown(url(foo\000020bar))",
        ] {
            assert!(!media_matches(unknown, 400.0, 300.0), "{unknown}");
            assert!(
                media_matches(&format!("{unknown} or (color)"), 400.0, 300.0),
                "{unknown}"
            );
            assert!(
                media_matches(&format!("(color) or {unknown}"), 400.0, 300.0),
                "{unknown}"
            );
        }
        let punctuation = format!("unknown(url({})) or (color)", "[{},]".repeat(1000));
        assert!(media_matches(&punctuation, 400.0, 300.0));
        // The surrounding CSS parser still treats comments as separators;
        // comments inside unquoted URL payloads remain an explicit limitation.
        assert!(!media_matches(
            "unknown(url(foo/**/bar)) or (color)",
            400.0,
            300.0
        ));
    }

    #[test]
    fn media_bad_url_recovery_keeps_lists_and_stylesheet_conditions_separate() {
        for query in [
            "url(foo), (color)",
            "url(foo bar), (color)",
            "unknown(url(foo bar)), (color)",
            "(color), unknown(url(foo bar))",
            "unknown(\"bad\n), (color)",
            "(color), unknown(\"bad\n)",
            "url(foo bar, bogus), (color)",
        ] {
            assert!(media_matches(query, 400.0, 300.0), "{query}");
        }
        for query in [
            "url(foo, screen)",
            "url(foo bar, screen)",
            "unknown(url(foo bar, screen))",
            "url(foo\\), screen)",
            "unknown(url(foo\\), screen))",
        ] {
            assert!(!media_matches(query, 400.0, 300.0), "{query}");
        }
        let doc = Document::parse("<p id=x>text</p>");
        let mut sources = vec![StyleSource::new("#x{color:green}")];
        for invalid in ["url(foo) or (color)", "unknown(url(foo bar)) or (color)"] {
            sources.push(StyleSource {
                source: "#x{color:red}".into(),
                layer: None,
                media: vec![invalid.into()],
            });
            sources.push(StyleSource::new(format!(
                "@media {invalid} {{#x{{color:red}}}}"
            )));
        }
        sources.push(StyleSource::new(
            r#"@media url("foo") or (color) {#x{background:blue}}"#,
        ));
        sources.push(StyleSource {
            source: "#x{height:12px}".into(),
            layer: None,
            media: vec!["unknown(url(foo)) or (color)".into()],
        });
        let style = &compute_styles_from_sources(&doc, &sources, 400.0, 300.0)
            [doc.query_selector("#x").unwrap()];
        assert_eq!(style.color, Color::rgb(0, 128, 0));
        assert_eq!(style.background_color, Color::rgb(0, 0, 255));
        assert_eq!(style.height, Length::Px(12.0));
    }

    #[test]
    fn media_conditions_have_shared_work_term_depth_and_list_limits() {
        let conjunction = std::iter::repeat_n("(color)", 64)
            .collect::<Vec<_>>()
            .join(" and ");
        assert!(media_matches(&conjunction, 400.0, 300.0));
        assert!(!media_matches(
            &format!("{conjunction} and (color)"),
            400.0,
            300.0
        ));
        let alternatives = std::iter::repeat_n("(color)", 64)
            .collect::<Vec<_>>()
            .join(" or ");
        assert!(media_matches(&alternatives, 400.0, 300.0));
        assert!(!media_matches(
            &format!("{alternatives} or (unknown)"),
            400.0,
            300.0
        ));
        assert!(!media_matches(
            &format!("{conjunction}, (color)"),
            400.0,
            300.0
        ));
        let nested = format!("{}color{}", "(".repeat(16), ")".repeat(16));
        assert!(media_matches(&nested, 400.0, 300.0));
        assert!(!media_matches(&format!("({nested})"), 400.0, 300.0));
        assert!(!media_matches(&format!("all, ({nested})"), 400.0, 300.0));
        assert!(!media_matches(
            &format!("(color) or ({}x)", " ".repeat(MAX_MEDIA_BYTES)),
            400.0,
            300.0
        ));
        assert!(!media_matches("all", -1.0, 300.0));
        assert!(!media_matches("all", 400.0, f32::INFINITY));
        let mut work = 1;
        assert!(!media_matches_with_budget("all", 400.0, 300.0, &mut work));
        assert_eq!(work, 0);
        let mut work = 400;
        let mut matches = 0;
        for _ in 0..100 {
            matches += usize::from(media_matches_with_budget(
                "((width >= 200px) or (color))",
                400.0,
                300.0,
                &mut work,
            ));
        }
        assert!(matches > 0 && matches < 100);
        assert_eq!(work, 0);
    }

    #[test]
    fn media_ranges_control_layers_and_import_metadata_at_viewport_boundaries() {
        let doc = Document::parse("<p id=x>text</p>");
        let conditional = StyleSource {
            source: "#x{color:red}".into(),
            layer: CascadeLayer::named(None, "conditional"),
            media: vec![
                "(200px < width <= 400px)".into(),
                "(height = 300px) or (orientation:portrait)".into(),
            ],
        };
        let sources = [
            conditional,
            StyleSource::new(
                "@layer fallback {#x{color:green}} @media ((width > 400px) or (height > 300px)) {#x{background:blue}}",
            ),
            StyleSource::new("@layer conditional {#x{color:red}}"),
        ];
        let x = doc.query_selector("#x").unwrap();
        let inside = compute_styles_from_sources(&doc, &sources, 400.0, 300.0);
        let outside = compute_styles_from_sources(&doc, &sources, 400.5, 300.0);
        assert_eq!(inside[x].color, Color::rgb(0, 128, 0));
        assert_eq!(inside[x].background_color, Color::TRANSPARENT);
        assert_eq!(outside[x].color, Color::rgb(255, 0, 0));
        assert_eq!(outside[x].background_color, Color::rgb(0, 0, 255));
        let empty = StyleSource {
            source: "".into(),
            layer: CascadeLayer::named(None, "outer"),
            media: vec!["(width > 400px) or (width <= 400px) and (color)".into()],
        };
        let sheets = [
            empty,
            StyleSource::new(
                "@layer other, outer;@layer outer{#x{color:red}}@layer other{#x{color:green}}",
            ),
        ];
        assert_eq!(
            compute_styles_from_sources(&doc, &sheets, 400.0, 300.0)[x].color,
            Color::rgb(255, 0, 0)
        );
    }

    #[test]
    fn positioned_values_preserve_valid_cascade_and_blockify_out_of_flow() {
        let doc = Document::parse(
            "<style>main{z-index:-7}#a{position:ABSOLUTE;float:left;z-index:inherit}#b{position:relative;position:bogus;z-index:4;z-index:1.5}#c{z-index:initial}#d{--Layer:9;--layer:2;z-index:var(--Layer)}</style><main><i id=a></i><i id=b></i><i id=c></i><i id=d></i></main>",
        );
        let styles = compute_styles(&doc, &doc.stylesheets(), 300.0, 200.0);
        let style = |selector| &styles[doc.query_selector(selector).unwrap()];
        assert_eq!(style("#a").position, "absolute");
        assert_eq!(style("#a").display, Display::Block);
        assert_eq!(style("#a").float, "none");
        assert_eq!(style("#a").z_index, Some(-7));
        assert_eq!(style("#b").z_index, Some(4));
        assert_eq!(style("#b").position, "relative");
        assert_eq!(style("#c").z_index, None);
        assert_eq!(style("#d").z_index, Some(9));
    }

    #[test]
    fn float_clear_cascade_blockification_and_flow_root_are_distinct() {
        let doc = Document::parse(
            "<style>main{float:LEFT;clear:both;display:flow-root}#a{float:inherit;clear:INHERIT}#b{float:right;float:invalid;clear:left;clear:invalid}#c{float:left;position:absolute}#d{display:flow-root;display:block}#e{float:unset;clear:unset}</style><main><i id=a></i><i id=b></i><i id=c></i><i id=d></i><i id=e></i><i id=f></i></main>",
        );
        let styles = compute_styles(&doc, &doc.stylesheets(), 300.0, 200.0);
        let style = |selector| &styles[doc.query_selector(selector).unwrap()];
        assert!(style("main").flow_root);
        assert_eq!(style("#a").float, "left");
        assert_eq!(style("#a").clear, "both");
        assert_eq!(style("#a").display, Display::Block);
        assert_eq!(style("#b").float, "right");
        assert_eq!(style("#b").clear, "left");
        assert_eq!(style("#c").float, "none");
        assert!(!style("#d").flow_root);
        for selector in ["#e", "#f"] {
            assert_eq!(style(selector).float, "none");
            assert_eq!(style(selector).clear, "none");
        }
    }
    #[test]
    fn foreign_type_candidates_preserve_case_sensitive_matching() {
        let doc =
            Document::parse("<svg><linearGradient id=g gradientUnits=userSpaceOnUse /></svg>");
        let sheets = vec!["linearGradient { color:#123456 } lineargradient { color:red } linearGradient[gradientUnits] { background:#abcdef } linearGradient[gradientunits] { background:red }".into()];
        let styles = compute_styles(&doc, &sheets, 400.0, 300.0);
        let gradient = doc.query_selector("#g").unwrap();
        assert_eq!(styles[gradient].color, Color::rgb(0x12, 0x34, 0x56));
        assert_eq!(
            styles[gradient].background_color,
            Color::rgb(0xab, 0xcd, 0xef)
        );
    }
    #[test]
    fn user_agent_defaults_distinguish_html_controls_and_svg_metadata() {
        let mut doc = Document::parse(
            "<svg id=s width=24 height=16><input id=si type=hidden hidden bgcolor=red width=99 style='border-width:4px' /><title id=st>svg</title></svg><math><input id=mi type=hidden hidden bgcolor=red width=99>foreign</input><title id=mt>math</title><style id=ms>text</style></math><input id=hi type=hidden>",
        );
        let html_svg = doc.create_element("svg");
        doc.append_child(doc.query_selector("body").unwrap(), html_svg);
        let styles = compute_styles(&doc, &[], 400.0, 300.0);
        for selector in ["#si", "#mi", "#mt", "#ms"] {
            let style = &styles[doc.query_selector(selector).unwrap()];
            assert_eq!(style.display, Display::Inline, "{selector}");
            assert_eq!(style.width, Length::Auto, "{selector}");
            assert_eq!(style.border_width.top, 0.0, "{selector}");
            assert_eq!(style.background_color, Color::TRANSPARENT, "{selector}");
        }
        assert_eq!(
            styles[doc.query_selector("#st").unwrap()].display,
            Display::None
        );
        assert_eq!(
            styles[doc.query_selector("#hi").unwrap()].display,
            Display::None
        );
        let svg = &styles[doc.query_selector("#s").unwrap()];
        assert_eq!(svg.display, Display::InlineBlock);
        assert_eq!(svg.width, Length::Px(24.0));
        assert_eq!(svg.height, Length::Px(16.0));
        assert_eq!(styles[html_svg].display, Display::Inline);
    }

    #[test]
    fn flex_item_order_alignment_and_global_keywords() {
        let doc = Document::parse(
            "<style>main{order:3;align-self:center;flex-grow:2}#a{order:-2;align-self:flex-end}#b{order:inherit;align-self:inherit;flex-grow:inherit}#c{order:1.5;align-self:invalid}</style><main><i id=a></i><i id=b></i><i id=c></i><i id=d></i></main>",
        );
        let styles = compute_styles(&doc, &doc.stylesheets(), 400.0, 300.0);
        let style = |selector| &styles[doc.query_selector(selector).unwrap()];
        assert_eq!(style("#a").order, -2);
        assert_eq!(style("#a").align_self, "flex-end");
        assert_eq!(style("#b").order, 3);
        assert_eq!(style("#b").align_self, "center");
        assert_eq!(style("#b").flex_grow, 2.0);
        assert_eq!(style("#c").order, 0);
        assert_eq!(style("#c").align_self, "auto");
        assert_eq!(style("#d").order, 0);
        assert_eq!(style("#d").align_self, "auto");
    }

    #[test]
    fn noscript_visibility_follows_document_scripting_flag() {
        for scripting in [false, true] {
            let doc = Document::parse_with_scripting(
                "<body><noscript>Fallback</noscript></body>",
                scripting,
            );
            let styles = compute_styles(&doc, &[], 400.0, 300.0);
            let node = doc.query_selector("noscript").unwrap();
            assert_eq!(styles[node].display == Display::None, scripting);
        }
    }
    #[test]
    fn repeated_explicit_grid_lists_are_interned_after_unit_resolution() {
        let implicit = "minmax(2px,1fr) ".repeat(64);
        let source = format!(
            "<style>i{{grid-template-columns:repeat(64,1em);grid-template-rows:repeat(64,1fr);grid-auto-columns:{implicit};grid-auto-rows:{implicit}}}#different{{font-size:20px}}</style>{}<i id=different></i>",
            "<i></i>".repeat(1000)
        );
        let doc = Document::parse(&source);
        let styles = compute_styles(&doc, &doc.stylesheets(), 800.0, 600.0);
        let ids = doc.query_selector_all("i");
        let first = &styles[ids[0]];
        for &id in &ids[..1000] {
            let child = &styles[id];
            for (own, shared) in [
                (&child.grid_template_columns, &first.grid_template_columns),
                (&child.grid_template_rows, &first.grid_template_rows),
                (&child.grid_auto_columns, &first.grid_auto_columns),
                (&child.grid_auto_rows, &first.grid_auto_rows),
            ] {
                assert_eq!(own.len(), 64);
                assert!(Arc::ptr_eq(own, shared));
            }
        }
        let different = &styles[doc.query_selector("#different").unwrap()];
        assert!(!Arc::ptr_eq(
            &different.grid_template_columns,
            &first.grid_template_columns
        ));
        assert_eq!(
            different.grid_template_columns[0],
            GridTrack::single(GridBreadth::Length(Length::Px(20.0)))
        );
        assert!(Arc::ptr_eq(
            &different.grid_template_rows,
            &first.grid_template_rows
        ));
    }

    #[test]
    fn unique_computed_grid_storage_has_a_retained_byte_cap() {
        let mut pool = GridTrackPool::default();
        let mut retained = Vec::new();
        let count = MAX_RETAINED_GRID_BYTES / (64 * std::mem::size_of::<GridTrack>());
        for index in 0..count + 2 {
            let mut tracks: Arc<[GridTrack]> =
                vec![GridTrack::single(GridBreadth::Length(Length::Px(index as f32))); 64].into();
            pool.intern(&mut tracks, false);
            retained.push(tracks);
        }
        assert_eq!(pool.bytes, MAX_RETAINED_GRID_BYTES);
        assert_eq!(pool.lists.len(), count);
        assert!(retained[count].is_empty() && retained[count + 1].is_empty());
        let mut implicit: Arc<[GridTrack]> =
            vec![GridTrack::single(GridBreadth::Length(Length::Px(999_999.0))); 64].into();
        pool.intern(&mut implicit, true);
        assert!(Arc::ptr_eq(&implicit, &AUTO_GRID_TRACKS));
        let mut duplicate: Arc<[GridTrack]> =
            vec![GridTrack::single(GridBreadth::Length(Length::Px(0.0))); 64].into();
        pool.intern(&mut duplicate, false);
        assert!(Arc::ptr_eq(&duplicate, &retained[0]));
    }

    #[test]
    fn grid_shorthands_preserve_case_sensitive_custom_property_references() {
        let doc = Document::parse(
            "<div style='--Track:70px;--track:10px;grid-template-columns:var(--Track);gap:var(--Track)'></div>",
        );
        let styles = compute_styles(&doc, &[], 300.0, 200.0);
        let style = &styles[doc.query_selector("div").unwrap()];
        assert_eq!(
            style.grid_template_columns[0],
            GridTrack::single(GridBreadth::Length(Length::Px(70.0)))
        );
        assert_eq!(style.row_gap, Length::Px(70.0));
        assert_eq!(style.column_gap, Length::Px(70.0));
    }

    #[test]
    fn inherited_grid_track_lists_share_storage_and_overrides_remain_independent() {
        let implicit = "1fr ".repeat(64);
        let source = format!(
            "<style>body{{grid-template-columns:repeat(64,1fr);grid-template-rows:repeat(64,2fr);grid-auto-columns:{implicit};grid-auto-rows:{implicit}}}i{{grid-template-columns:inherit;grid-template-rows:inherit;grid-auto-columns:inherit;grid-auto-rows:inherit}}#override{{grid-template-columns:25px}}</style>{}<i id=override></i>",
            "<i></i>".repeat(5000)
        );
        let doc = Document::parse(&source);
        let styles = compute_styles(&doc, &doc.stylesheets(), 800.0, 600.0);
        let parent = &styles[doc.query_selector("body").unwrap()];
        let mut siblings = 0;
        for id in doc.query_selector_all("i") {
            if doc.attr(id, "id") == Some("override") {
                continue;
            }
            let child = &styles[id];
            siblings += 1;
            for (own, inherited) in [
                (&child.grid_template_columns, &parent.grid_template_columns),
                (&child.grid_template_rows, &parent.grid_template_rows),
                (&child.grid_auto_columns, &parent.grid_auto_columns),
                (&child.grid_auto_rows, &parent.grid_auto_rows),
            ] {
                assert_eq!(own.len(), 64);
                assert!(Arc::ptr_eq(own, inherited));
            }
        }
        assert_eq!(siblings, 5000);
        let own = &styles[doc.query_selector("#override").unwrap()];
        assert_eq!(own.grid_template_columns.len(), 1);
        assert!(!Arc::ptr_eq(
            &own.grid_template_columns,
            &parent.grid_template_columns
        ));
        assert_eq!(parent.grid_template_columns.len(), 64);
        let first = ComputedStyle::default();
        let second = ComputedStyle::default();
        assert!(Arc::ptr_eq(
            &first.grid_auto_rows,
            &second.grid_auto_columns
        ));
        assert!(Arc::ptr_eq(
            &first.grid_template_columns,
            &second.grid_template_rows
        ));
    }

    #[test]
    fn grid_tracks_preserve_minmax_repeat_and_reject_invalid_overrides() {
        let doc = Document::parse(
            "<div style='grid-template-columns:20px repeat(2,minmax(10px,1fr) max-content);grid-template-columns:minmax(1fr,10px);grid-template-rows:30% auto;grid-auto-rows:12px 24px;grid-auto-columns:minmax(min-content,60px)'></div>",
        );
        let styles = compute_styles(&doc, &[], 400.0, 300.0);
        let style = &styles[doc.query_selector("div").unwrap()];
        assert_eq!(style.grid_template_columns.len(), 5);
        assert_eq!(
            style.grid_template_columns[1],
            GridTrack {
                min: GridBreadth::Length(Length::Px(10.0)),
                max: GridBreadth::Length(Length::Fr(1.0))
            }
        );
        assert_eq!(
            style.grid_template_columns[4],
            GridTrack::single(GridBreadth::MaxContent)
        );
        assert_eq!(
            style.grid_template_rows[0],
            GridTrack::single(GridBreadth::Length(Length::Percent(30.0)))
        );
        assert_eq!(style.grid_auto_rows.len(), 2);
        assert_eq!(style.grid_auto_columns[0].min, GridBreadth::MinContent);
        for invalid in [
            "repeat(0,1fr)",
            "minmax(1fr,10px)",
            "10px bogus",
            "minmax(-10px,auto)",
            "repeat(2,none)",
        ] {
            assert!(
                parse_grid_tracks(invalid, 16.0, 16.0, 400.0, 300.0).is_none(),
                "{invalid}"
            );
        }
    }
    #[test]
    fn grid_keywords_and_gaps_are_case_insensitive_and_invalid_shorthands_are_atomic() {
        let doc = Document::parse(
            "<div style='gap:5px 9px;gap:-3px 20px;grid-auto-flow:COLUMN DENSE;grid-row:SPAN 2;grid-template-columns:20px;grid-template-columns:repeat(2,repeat(2,1fr))'></div>",
        );
        let styles = compute_styles(&doc, &[], 300.0, 200.0);
        let style = &styles[doc.query_selector("div").unwrap()];
        assert_eq!(style.row_gap, Length::Px(5.0));
        assert_eq!(style.column_gap, Length::Px(9.0));
        assert_eq!(style.grid_auto_flow, "column dense");
        assert_eq!(style.grid_row_start, GridLine::Span(2));
        assert_eq!(style.grid_template_columns.len(), 1);
    }

    #[test]
    fn grid_shorthands_expand_before_cascade_and_global_values_copy_all_fields() {
        let doc = Document::parse(
            "<main style='gap:4px 8px;grid-area:2 / -3 / span 2 / -1;grid-column-start:3;grid-auto-flow:column dense;place-items:center end;grid-auto-rows:30px'><i style='grid-area:inherit;gap:inherit;grid-auto-rows:inherit;justify-items:inherit'></i></main>",
        );
        let styles = compute_styles(&doc, &[], 400.0, 300.0);
        let parent = &styles[doc.query_selector("main").unwrap()];
        let child = &styles[doc.query_selector("i").unwrap()];
        assert_eq!(parent.row_gap, Length::Px(4.0));
        assert_eq!(parent.column_gap, Length::Px(8.0));
        assert_eq!(parent.grid_column_start, GridLine::Line(3));
        assert_eq!(parent.grid_column_end, GridLine::Line(-1));
        assert_eq!(parent.grid_row_start, GridLine::Line(2));
        assert_eq!(parent.grid_row_end, GridLine::Span(2));
        assert_eq!(parent.grid_auto_flow, "column dense");
        assert_eq!(parent.align_items, "center");
        assert_eq!(child.justify_items, "end");
        assert_eq!(child.row_gap, parent.row_gap);
        assert_eq!(child.column_gap, parent.column_gap);
        assert_eq!(child.grid_column_start, parent.grid_column_start);
        assert_eq!(child.grid_auto_rows, parent.grid_auto_rows);
        assert_eq!(parse_grid_line("0"), None);
        assert_eq!(parse_grid_line("span -2"), None);
        assert_eq!(parse_grid_line("2 span"), Some(GridLine::Span(2)));
    }

    #[test]
    fn fractional_grid_tracks_remain_distinct_from_percentages() {
        assert_eq!(
            parse_grid_tracks("100px 25% 1fr 2fr", 16.0, 16.0, 800.0, 600.0).unwrap(),
            vec![
                GridTrack::single(GridBreadth::Length(Length::Px(100.0))),
                GridTrack::single(GridBreadth::Length(Length::Percent(25.0))),
                GridTrack::single(GridBreadth::Length(Length::Fr(1.0))),
                GridTrack::single(GridBreadth::Length(Length::Fr(2.0)))
            ]
        );
        assert_eq!(Length::Fr(2.0).resolve(800.0), None);
        assert!(parse_length("1fr", 16.0, 16.0, 800.0, 600.0).is_none());
    }

    #[test]
    fn table_headers_are_bold_centered_with_author_overrides() {
        let doc = Document::parse(
            "<table><tr><th id=default>Title</th><th id=custom style='font-weight:normal;text-align:left'>Label</th></tr></table>",
        );
        let styles = compute_styles(&doc, &[], 800.0, 600.0);
        let default = doc.query_selector("#default").unwrap();
        let custom = doc.query_selector("#custom").unwrap();
        assert_eq!(styles[default].font_weight, 700);
        assert_eq!(styles[default].text_align, "center");
        assert_eq!(styles[custom].font_weight, 400);
        assert_eq!(styles[custom].text_align, "left");
    }
    #[test]
    fn native_controls_and_text_keep_ua_border_and_inherited_line_height() {
        let d = Document::parse("<div style='line-height:2'><span>Text</span><input><hr></div>");
        let styles = compute_styles(&d, &[], 800.0, 600.0);
        let span = d.query_selector("span").unwrap();
        assert_eq!(styles[span].line_height, 32.0);
        assert_eq!(styles[d.nodes[span].children[0]].line_height, 32.0);
        assert_eq!(
            styles[d.query_selector("input").unwrap()].border_width.top,
            1.0
        );
        assert_eq!(
            styles[d.query_selector("hr").unwrap()].border_width.top,
            1.0
        );
    }
    #[test]
    fn nested_css_and_large_value_expansion_are_bounded() {
        let nested = format!("{}1px{}", "CALC(".repeat(1000), ")".repeat(1000));
        assert!(parse_length(&nested, 16.0, 16.0, 800.0, 600.0).is_none());
        assert!(!supports_matches(&format!(
            "{}(display:block)",
            "not ".repeat(1000)
        )));
        let tracks = parse_grid_tracks("repeat(999999,1fr)", 16.0, 16.0, 800.0, 600.0).unwrap();
        assert_eq!(tracks.len(), 64);
    }
    #[test]
    fn cascade_specificity_important_and_inheritance() {
        let d = Document::parse("<div id=a class=x style='color:green'><span>text</span></div>");
        let s=compute_styles(&d,&["#a { color: red; font-size:20px } .x { color: blue !important } div { color: black }".into()],800.0,600.0);
        let a = d.query_selector("#a").unwrap();
        let span = d.query_selector("span").unwrap();
        assert_eq!(s[a].color, Color::rgb(0, 0, 255));
        assert_eq!(s[span].color, s[a].color);
        assert_eq!(s[span].font_size, 20.0);
    }
    #[test]
    fn shorthand_order_and_box_model() {
        let d = Document::parse(
            "<div style='margin-left:99px;margin:1px 2px 3px 4px;padding:2em;border:3px solid #abc;font-size:10px'></div>",
        );
        let s = compute_styles(&d, &[], 800.0, 600.0);
        let a = d.query_selector("div").unwrap();
        assert_eq!(s[a].margin.left, Length::Px(4.0));
        assert_eq!(s[a].padding.top, Length::Px(20.0));
        assert_eq!(s[a].border_width.top, 3.0);
        assert_eq!(s[a].border_color, Color::rgb(170, 187, 204));
    }
    #[test]
    fn media_and_variables() {
        let d = Document::parse("<div><p>x</p></div>");
        let s=compute_styles(&d,&[":root { --ink:#135; } p {color:var(--ink);width:50vw} @media (max-width:600px) {p{display:none}}".into()],500.0,400.0);
        let p = d.query_selector("p").unwrap();
        assert_eq!(s[p].display, Display::None);
        assert_eq!(s[p].color, Color::rgb(17, 51, 85));
        assert_eq!(s[p].width, Length::Px(250.0));
        assert!(
            resolve_vars(
                "var(--x)",
                &BTreeMap::from([("--x".into(), "var(--x)".into())]),
                0,
                &mut 100_000,
            )
            .is_none()
        );
    }
    #[test]
    fn colors_and_lengths() {
        assert_eq!(
            parse_color("hsl(120 100% 50% / 50%)"),
            Some(Color::rgba(0, 255, 0, 128))
        );
        assert_eq!(parse_color("#1234"), Some(Color::rgba(17, 34, 51, 68)));
        assert_eq!(
            parse_length("2rem", 10.0, 16.0, 800.0, 600.0),
            Some(Length::Px(32.0))
        );
        assert_eq!(parse_length("NaNpx", 16.0, 16.0, 800.0, 600.0), None);
    }
    #[test]
    fn quoted_delimiters_and_comments() {
        let rules = parse_stylesheet(
            "/*x*/ [title='a;b'] {font-family:'a;b';color:rgb(1,2,3)} @media print {p{color:red}}",
            800.0,
            600.0,
        );
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].declarations.len(), 2);
        assert_eq!(specificity(":where(#x) div.a"), 1025);
    }
}
