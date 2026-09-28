//! Independent CSS parsing, selector cascade, inheritance and computed values.
use crate::dom::{
    Document, Namespace, NodeId, NodeKind, matches_selector_with_budget, split_top_level,
};
use crate::graphics::Color;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

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
    pub grid_template_columns: Vec<Length>,
    pub position: String,
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub overflow: String,
    pub border_radius: f32,
    pub opacity: f32,
    pub box_sizing: String,
    pub list_style_type: String,
    pub vertical_align: String,
}
impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            flow_root: false,
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
            grid_template_columns: vec![],
            position: "static".into(),
            top: Length::Auto,
            right: Length::Auto,
            bottom: Length::Auto,
            left: Length::Auto,
            overflow: "visible".into(),
            border_radius: 0.0,
            opacity: 1.0,
            box_sizing: "content-box".into(),
            list_style_type: "disc".into(),
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
}
#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<String>,
    pub declarations: Vec<Declaration>,
}

pub fn parse_stylesheet(source: &str, width: f32, height: f32) -> Vec<Rule> {
    let clean = strip_comments(source);
    let mut rules = vec![];
    parse_rules(&clean, width, height, 0, &mut rules);
    rules
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
fn parse_rules(source: &str, width: f32, height: f32, depth: usize, rules: &mut Vec<Rule>) {
    if depth > 16 {
        return;
    }
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
            start = i + 1;
        } else if c == b'{' && parens == 0 {
            let header = source[start..i].trim();
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
            if let Some(media) = header.strip_prefix("@media") {
                if media_matches(media, width, height) {
                    parse_rules(body, width, height, depth + 1, rules);
                }
            } else if let Some(supports) = header.strip_prefix("@supports") {
                if supports_matches(supports) {
                    parse_rules(body, width, height, depth + 1, rules);
                }
            } else if header.starts_with("@layer") || header.starts_with("@container") {
                if header.starts_with("@layer") {
                    parse_rules(body, width, height, depth + 1, rules);
                }
            } else if !header.starts_with('@') && !header.is_empty() {
                let selectors = split_top_level(header, ',')
                    .into_iter()
                    .filter(|s| !s.is_empty() && s.len() <= 4096)
                    .take(128)
                    .map(str::to_owned)
                    .collect();
                rules.push(Rule {
                    selectors,
                    declarations: parse_declarations(body),
                });
            }
            start = i.saturating_add(1);
        }
        i += 1;
    }
}
pub fn media_matches(query: &str, width: f32, height: f32) -> bool {
    split_top_level(query, ',').into_iter().any(|part| {
        let q = part.trim().to_ascii_lowercase();
        let (negated, q) = if let Some(rest) = q.strip_prefix("not ") {
            (true, rest)
        } else {
            (false, q.as_str())
        };
        let q = q.strip_prefix("only ").unwrap_or(q);
        let mut matches = !q.starts_with("print") && !q.starts_with("speech");
        let mut cursor = q;
        while let Some(open) = cursor.find('(') {
            let Some(close) = cursor[open + 1..].find(')') else {
                return false;
            };
            let feature = cursor[open + 1..open + 1 + close].trim();
            let test = if let Some((name, value)) = feature.split_once(':') {
                let value = value.trim();
                let n = parse_length(value, 16.0, 16.0, width, height)
                    .and_then(|l| l.resolve(width))
                    .unwrap_or(-1.0);
                match name.trim() {
                    "min-width" => width >= n,
                    "max-width" => width <= n,
                    "width" => (width - n).abs() < 0.01,
                    "min-height" => height >= n,
                    "max-height" => height <= n,
                    "height" => (height - n).abs() < 0.01,
                    "orientation" => {
                        if value == "landscape" {
                            width >= height
                        } else {
                            height > width
                        }
                    }
                    "prefers-color-scheme" => value == "light",
                    "prefers-reduced-motion" => value == "reduce",
                    "hover" | "any-hover" => value == "hover",
                    "pointer" | "any-pointer" => value == "fine",
                    "display-mode" => value == "browser",
                    _ => false,
                }
            } else {
                matches!(feature, "color" | "hover" | "any-hover")
            };
            matches &= test;
            cursor = &cursor[open + close + 2..];
        }
        if negated { !matches } else { matches }
    })
}
fn supports_matches(query: &str) -> bool {
    if query.len() > 4096 || query.matches("not ").take(17).count() > 16 {
        return false;
    }
    let q = query.trim();
    if let Some(rest) = q.strip_prefix("not ") {
        return !supports_matches(rest);
    }
    let q = q.trim_matches(['(', ')']);
    if let Some((name, value)) = q.split_once(':') {
        match name.trim() {
            "display" => matches!(
                value.trim(),
                "block" | "flow-root" | "inline" | "inline-block" | "flex" | "grid" | "none"
            ),
            "float" => matches!(value.trim(), "none" | "left" | "right"),
            "clear" => matches!(value.trim(), "none" | "left" | "right" | "both"),
            "color" | "background-color" => parse_color(value.trim()).is_some(),
            "width" | "height" | "margin" | "padding" => {
                parse_length(value.trim(), 16.0, 16.0, 800.0, 600.0).is_some()
            }
            _ => false,
        }
    } else {
        false
    }
}
pub fn parse_declarations(source: &str) -> Vec<Declaration> {
    let mut declarations = vec![];
    for part in split_top_level(source, ';').into_iter().take(4096) {
        let Some((name, value)) = part.split_once(':') else {
            continue;
        };
        let name = if name.trim().starts_with("--") {
            name.trim().to_owned()
        } else {
            name.trim().to_ascii_lowercase()
        };
        let mut value = value.trim();
        if name.is_empty() || value.is_empty() || name.len() > 256 || value.len() > 65_536 {
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
            if !(matches!(
                keyword.as_str(),
                "none" | "left" | "right" | "inherit" | "initial" | "unset" | "revert"
            ) || name == "clear" && keyword == "both")
            {
                continue;
            }
        }
        expand_declaration(&name, value, important, &mut declarations);
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
fn expand_declaration(name: &str, value: &str, important: bool, out: &mut Vec<Declaration>) {
    let add = |out: &mut Vec<Declaration>, name: &str, value: &str| {
        out.push(Declaration {
            name: name.into(),
            value: value.into(),
            important,
        })
    };
    if matches!(
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
    let mut rules = vec![];
    for source in sources.iter().take(256) {
        rules.extend(parse_stylesheet(source, width, height));
        if rules.len() >= 10_000 {
            rules.truncate(10_000);
            break;
        }
    }
    compute_styles_with_rules(doc, &rules, width, height)
}
struct IndexedRule<'a> {
    selector: &'a str,
    specificity: u32,
    declarations: &'a [Declaration],
    order: u32,
}
type CascadePriority = (bool, u32, u32, u32);
type CascadedProperties = BTreeMap<String, (CascadePriority, String)>;
pub fn compute_styles_with_rules(
    doc: &Document,
    rules: &[Rule],
    width: f32,
    height: f32,
) -> Vec<ComputedStyle> {
    let mut indexed = vec![];
    let mut by_key: HashMap<String, Vec<usize>> = HashMap::new();
    for (order, rule) in rules.iter().enumerate().take(10_000) {
        for selector in &rule.selectors {
            let index = indexed.len();
            by_key
                .entry(selector_key(selector))
                .or_default()
                .push(index);
            indexed.push(IndexedRule {
                selector,
                specificity: specificity(selector),
                declarations: &rule.declarations,
                order: order as u32,
            });
        }
    }
    let mut styles = vec![ComputedStyle::default(); doc.nodes.len()];
    let empty_variables = Arc::new(BTreeMap::new());
    let mut variables: Vec<Arc<BTreeMap<String, String>>> = vec![empty_variables; doc.nodes.len()];
    let mut pending = vec![doc.root];
    let mut visited = vec![false; doc.nodes.len()];
    let mut work = 20_000_000usize;
    let mut retained_variable_bytes = 0usize;
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
        apply_user_agent(&mut style, doc, id, tag);
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
        let mut cascade: CascadedProperties = BTreeMap::new();
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
                if matches_selector_with_budget(doc, id, rule.selector, &mut work) {
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
                        let priority = (
                            decl.important,
                            rule.specificity,
                            rule.order,
                            decl_order as u32,
                        );
                        if cascade
                            .get(&decl.name)
                            .is_none_or(|(old, _)| priority >= *old)
                        {
                            cascade.insert(decl.name.clone(), (priority, decl.value.clone()));
                        }
                    }
                }
            }
            if let Some(inline) = doc.attr(id, "style") {
                for (decl_order, decl) in parse_declarations(inline).into_iter().enumerate() {
                    if !supported_property(&decl.name) || decl.value.len() > 4096 {
                        continue;
                    }
                    let priority = (decl.important, 1 << 30, u32::MAX, decl_order as u32);
                    if cascade
                        .get(&decl.name)
                        .is_none_or(|(old, _)| priority >= *old)
                    {
                        cascade.insert(decl.name, (priority, decl.value));
                    }
                }
            }
            let inherited_variable_size: usize = vars.iter().map(|(k, v)| k.len() + v.len()).sum();
            let new_variable_size: usize = cascade
                .iter()
                .filter(|(k, _)| k.starts_with("--"))
                .map(|(k, (_, v))| k.len() + v.len())
                .sum();
            if new_variable_size > 0
                && retained_variable_bytes + inherited_variable_size + new_variable_size
                    <= 8 * 1024 * 1024
            {
                retained_variable_bytes += inherited_variable_size + new_variable_size;
                let map = Arc::make_mut(&mut vars);
                for (name, (_, value)) in &cascade {
                    if name.starts_with("--")
                        && value.len() <= 4096
                        && (map.len() < 128 || map.contains_key(name))
                    {
                        map.insert(name.clone(), value.clone());
                    }
                }
            }
            let mut resolved: BTreeMap<String, String> = BTreeMap::new();
            for (name, (_, value)) in &cascade {
                if !name.starts_with("--")
                    && let Some(value) = resolve_vars(value, &vars, 0)
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
                    .map(String::as_str)
                    .unwrap_or(if native_border { "solid" } else { "none" });
                if matches!(border_style, "none" | "hidden") {
                    *edge_mut(&mut style.border_width, side) = 0.0;
                }
            }
        }
        if matches!(style.position.as_str(), "absolute" | "fixed") {
            style.float = "none".into();
        } else if style.float != "none"
            && matches!(style.display, Display::Inline | Display::InlineBlock)
        {
            // CSS2 section 9.7 blockifies floating inline boxes.
            style.display = Display::Block;
        }
        variables[id] = vars;
        styles[id] = style;
        pending.extend(node.children.iter().rev().copied());
    }
    styles
}
fn supported_property(name: &str) -> bool {
    name.starts_with("--")
        || matches!(
            name,
            "display"
                | "float"
                | "clear"
                | "width"
                | "height"
                | "min-width"
                | "min-height"
                | "max-width"
                | "max-height"
                | "margin-top"
                | "margin-right"
                | "margin-bottom"
                | "margin-left"
                | "padding-top"
                | "padding-right"
                | "padding-bottom"
                | "padding-left"
                | "border-top-width"
                | "border-right-width"
                | "border-bottom-width"
                | "border-left-width"
                | "border-top-style"
                | "border-right-style"
                | "border-bottom-style"
                | "border-left-style"
                | "border-top-color"
                | "border-right-color"
                | "border-bottom-color"
                | "border-left-color"
                | "color"
                | "background-color"
                | "font-size"
                | "font-family"
                | "font-style"
                | "font-weight"
                | "line-height"
                | "text-align"
                | "white-space"
                | "text-decoration"
                | "flex-direction"
                | "flex-wrap"
                | "justify-content"
                | "align-items"
                | "align-self"
                | "order"
                | "gap"
                | "row-gap"
                | "column-gap"
                | "flex-grow"
                | "flex-shrink"
                | "flex-basis"
                | "grid-template-columns"
                | "position"
                | "top"
                | "right"
                | "bottom"
                | "left"
                | "overflow"
                | "overflow-x"
                | "overflow-y"
                | "border-radius"
                | "opacity"
                | "box-sizing"
                | "list-style-type"
                | "vertical-align"
        )
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
    specificity_inner(selector, 0)
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
                i = selector[i..]
                    .find(']')
                    .map(|n| i + n + 1)
                    .unwrap_or(selector.len());
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
                    let mut level = 1;
                    i += 1;
                    while i < selector.len() {
                        if selector.as_bytes()[i] == b'(' {
                            level += 1;
                        } else if selector.as_bytes()[i] == b')' {
                            level -= 1;
                            if level == 0 {
                                break;
                            }
                        }
                        i += 1;
                    }
                    argument = Some(&selector[start..i]);
                    i = (i + 1).min(selector.len());
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
fn resolve_vars(value: &str, variables: &BTreeMap<String, String>, depth: usize) -> Option<String> {
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
    let replacement = resolve_vars(replacement, variables, depth + 1)?;
    let new = format!("{}{}{}", &value[..start], replacement, &value[end + 1..]);
    resolve_vars(&new, variables, depth + 1)
}
fn apply_user_agent(s: &mut ComputedStyle, doc: &Document, id: NodeId, tag: &str) {
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
    let keyword = matches!(name, "float" | "clear").then(|| value.trim().to_ascii_lowercase());
    let value = keyword.as_deref().unwrap_or_else(|| value.trim());
    if value.len() > 256
        && matches!(
            name,
            "font-family"
                | "text-decoration"
                | "justify-content"
                | "align-items"
                | "align-self"
                | "list-style-type"
                | "vertical-align"
        )
    {
        return;
    }
    let initial = ComputedStyle::default();
    if matches!(value, "inherit" | "initial" | "unset" | "revert") {
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
        );
        let from = if value == "inherit" || value == "unset" && inherited {
            parent.unwrap_or(&initial)
        } else {
            &initial
        };
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
        "gap" | "row-gap" | "column-gap" => {
            if let Some(v) = words(value)
                .first()
                .and_then(|v| parse_length(v, s.font_size, root_font, width, height))
                .and_then(|v| v.resolve(width))
            {
                s.gap = v.max(0.0);
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
        "grid-template-columns" => {
            s.grid_template_columns =
                parse_grid_tracks(value, s.font_size, root_font, width, height)
        }
        "position" => {
            if matches!(
                value,
                "static" | "relative" | "absolute" | "fixed" | "sticky"
            ) {
                s.position = value.into();
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
        "position" => s.position = p.position.clone(),
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
fn parse_grid_tracks(value: &str, font: f32, root: f32, width: f32, height: f32) -> Vec<Length> {
    let expanded = if let Some(inner) = value
        .strip_prefix("repeat(")
        .and_then(|x| x.strip_suffix(')'))
    {
        let args = split_top_level(inner, ',');
        let n = args
            .first()
            .and_then(|n| n.trim().parse::<usize>().ok())
            .unwrap_or(1)
            .min(64);
        args.get(1)
            .map(|v| vec![*v; n].join(" "))
            .unwrap_or_default()
    } else {
        value.into()
    };
    let tokens = words(&expanded);
    tokens
        .into_iter()
        .take(64)
        .filter_map(|s| {
            if let Some(n) = s.strip_suffix("fr").and_then(finite_number) {
                (n >= 0.0).then_some(Length::Fr(n))
            } else {
                parse_length(s, font, root, width, height)
            }
        })
        .collect()
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
    use super::*;
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
    fn fractional_grid_tracks_remain_distinct_from_percentages() {
        assert_eq!(
            parse_grid_tracks("100px 25% 1fr 2fr", 16.0, 16.0, 800.0, 600.0),
            vec![
                Length::Px(100.0),
                Length::Percent(25.0),
                Length::Fr(1.0),
                Length::Fr(2.0)
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
        let tracks = parse_grid_tracks("repeat(999999,1fr)", 16.0, 16.0, 800.0, 600.0);
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
                0
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
