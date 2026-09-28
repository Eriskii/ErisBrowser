//! Bounded, independent CSS box construction, line layout, and display lists.
//!
//! This implements the useful core of block, inline, flex and grid layout. It is
//! deliberately explicit about its limits: it is not a complete CSS formatter.
use crate::css::{ComputedStyle, Display, Length};
use crate::dom::{Document, NodeId, NodeKind};
use crate::graphics::{Color, DrawCommand, Fonts, Rect};
use std::cell::Cell as Counter;

const MAX_DEPTH: usize = 128;
const MAX_VISITS: usize = 100_000;
const MAX_COMMANDS: usize = 200_000;
const MAX_GLYPHS: usize = 500_000;
const MAX_EXTENT: f32 = 1_000_000.0;

#[derive(Debug, Clone)]
pub struct HitRegion {
    pub node: NodeId,
    pub rect: Rect,
}

pub struct LayoutResult {
    pub commands: Vec<DrawCommand>,
    pub hit_regions: Vec<HitRegion>,
    pub content_height: f32,
}

impl LayoutResult {
    /// Paint order is also hit-test order; descendants win over ancestors.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        self.hit_regions.iter().rev().find_map(|hit| {
            (x >= hit.rect.x
                && y >= hit.rect.y
                && x < hit.rect.x + hit.rect.width
                && y < hit.rect.y + hit.rect.height)
                .then_some(hit.node)
        })
    }
}

#[derive(Clone, Copy, Default)]
struct Sides {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl Sides {
    fn horizontal(self) -> f32 {
        self.left + self.right
    }
    fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Default)]
struct Size {
    width: f32,
    height: f32,
}

struct Fragment {
    size: Size,
    commands: Vec<DrawCommand>,
    hits: Vec<HitRegion>,
    rounded_border: bool,
}

enum InlineKind {
    Text(String),
    Space(String),
    Box(Fragment),
    Break,
}

struct InlineItem {
    node: NodeId,
    owner: NodeId,
    kind: InlineKind,
    width: f32,
    height: f32,
    preserve: bool,
}

struct Engine<'a> {
    doc: &'a Document,
    styles: &'a [ComputedStyle],
    fonts: &'a Fonts,
    viewport: Size,
    commands: Vec<DrawCommand>,
    hits: Vec<HitRegion>,
    visits: usize,
    glyphs_left: usize,
    intrinsic_work_left: Counter<usize>,
    canvas_background_node: Option<NodeId>,
    fallback: ComputedStyle,
}

/// Construct a display list in document coordinates. Scrolling is a paint-time
/// transform; callers should not rerun layout for every scroll event.
pub fn layout(
    doc: &Document,
    styles: &[ComputedStyle],
    width: f32,
    height: f32,
    fonts: &Fonts,
) -> LayoutResult {
    let viewport = Size {
        width: finite(width, 800.0).clamp(1.0, 100_000.0),
        height: finite(height, 600.0).clamp(1.0, 100_000.0),
    };
    let html = doc.query_selector("html");
    let canvas_background_node = html
        .filter(|id| {
            styles
                .get(*id)
                .is_some_and(|style| style.background_color.a > 0)
        })
        .or_else(|| {
            doc.query_selector("body").filter(|id| {
                styles.get(*id).is_some_and(|style| {
                    style.display != Display::None && style.background_color.a > 0
                })
            })
        });
    let mut engine = Engine {
        doc,
        styles,
        fonts,
        viewport,
        commands: Vec::new(),
        hits: Vec::new(),
        visits: 0,
        glyphs_left: MAX_GLYPHS,
        intrinsic_work_left: Counter::new(MAX_GLYPHS),
        canvas_background_node,
        fallback: ComputedStyle::default(),
    };
    if let Some(id) = canvas_background_node {
        let style = engine.style(id);
        engine.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, viewport.width, viewport.height),
            color: faded(style.background_color, style.opacity),
            radius: 0.0,
        });
    }
    let size = if doc.nodes.get(doc.root).is_some() {
        engine.layout_box(doc.root, 0.0, 0.0, viewport.width, Some(viewport.width), 0)
    } else {
        Size::default()
    };
    let painted_bottom = engine
        .hits
        .iter()
        .filter(|hit| hit.rect.width > 0.0 && hit.rect.height > 0.0)
        .map(|hit| hit.rect.y + hit.rect.height)
        .fold(size.height, f32::max);
    let content_height = painted_bottom.max(viewport.height).min(MAX_EXTENT);
    if canvas_background_node.is_some()
        && let Some(DrawCommand::Rect { rect, .. }) = engine.commands.first_mut()
    {
        rect.height = content_height;
    }
    engine.commands.retain(|command| match command {
        DrawCommand::Rect { rect, color, .. } => {
            color.a > 0 && rect.width > 0.0 && rect.height > 0.0
        }
        DrawCommand::Text { text, color, .. } => !text.is_empty() && color.a > 0,
        _ => true,
    });
    LayoutResult {
        commands: engine.commands,
        hit_regions: engine.hits,
        content_height,
    }
}

impl Engine<'_> {
    fn style(&self, id: NodeId) -> &ComputedStyle {
        self.styles.get(id).unwrap_or(&self.fallback)
    }

    fn is_hidden(&self, id: NodeId) -> bool {
        self.style(id).display == Display::None
            || self.doc.tag(id) == Some("input")
                && self
                    .doc
                    .attr(id, "type")
                    .is_some_and(|kind| kind.eq_ignore_ascii_case("hidden"))
    }

    fn enter(&mut self, id: NodeId, depth: usize) -> bool {
        if depth > MAX_DEPTH
            || self.visits >= MAX_VISITS
            || self.commands.len() >= MAX_COMMANDS
            || self.doc.nodes.get(id).is_none()
        {
            return false;
        }
        self.visits += 1;
        true
    }

    fn push(&mut self, command: DrawCommand) {
        if self.commands.len() < MAX_COMMANDS {
            self.commands.push(command);
        }
    }

    fn margins(&self, id: NodeId, reference: f32) -> Sides {
        let margin = &self.style(id).margin;
        Sides {
            top: resolve(margin.top, reference).unwrap_or(0.0),
            right: resolve(margin.right, reference).unwrap_or(0.0),
            bottom: resolve(margin.bottom, reference).unwrap_or(0.0),
            left: resolve(margin.left, reference).unwrap_or(0.0),
        }
    }

    fn padding(&self, id: NodeId, reference: f32) -> Sides {
        let padding = &self.style(id).padding;
        Sides {
            top: resolve(padding.top, reference).unwrap_or(0.0).max(0.0),
            right: resolve(padding.right, reference).unwrap_or(0.0).max(0.0),
            bottom: resolve(padding.bottom, reference).unwrap_or(0.0).max(0.0),
            left: resolve(padding.left, reference).unwrap_or(0.0).max(0.0),
        }
    }

    fn borders(&self, id: NodeId) -> Sides {
        let border = &self.style(id).border_width;
        Sides {
            top: extent(border.top),
            right: extent(border.right),
            bottom: extent(border.bottom),
            left: extent(border.left),
        }
    }

    fn width_for(&self, id: NodeId, available: f32, containing_width: f32) -> f32 {
        let style = self.style(id);
        let padding = self.padding(id, containing_width);
        let border = self.borders(id);
        let extra = padding.horizontal() + border.horizontal();
        let css_to_border = if style.box_sizing == "border-box" {
            0.0
        } else {
            extra
        };
        let tag = self.doc.tag(id).unwrap_or("");
        let mut width = resolve(style.width, containing_width)
            .map(|value| value + css_to_border)
            .unwrap_or_else(|| match tag {
                "img" | "svg" | "canvas" | "video" => {
                    let natural = self.natural_size(id);
                    let value = self.attr_number(id, "width").unwrap_or_else(|| {
                        resolve(style.height, self.viewport.height)
                            .or_else(|| self.attr_number(id, "height"))
                            .map(|height| height * natural.width / natural.height.max(1.0))
                            .unwrap_or(natural.width)
                    });
                    value + extra
                }
                "input"
                    if self.doc.attr(id, "type").is_some_and(|kind| {
                        kind.eq_ignore_ascii_case("checkbox") || kind.eq_ignore_ascii_case("radio")
                    }) =>
                {
                    16.0 + extra
                }
                "input" => {
                    self.attr_number(id, "size").unwrap_or(20.0) * style.font_size * 0.55
                        + 12.0
                        + extra
                }
                "textarea" => {
                    self.attr_number(id, "cols").unwrap_or(20.0) * style.font_size * 0.6
                        + 12.0
                        + extra
                }
                "select" => self.intrinsic_width(id, containing_width).max(80.0) + 22.0 + extra,
                _ => available,
            });
        if let Some(max) = resolve(style.max_width, containing_width) {
            width = width.min(max + css_to_border);
        }
        if let Some(min) = resolve(style.min_width, containing_width) {
            width = width.max(min + css_to_border);
        }
        extent(width).max(extra)
    }

    fn layout_box(
        &mut self,
        id: NodeId,
        mut x: f32,
        mut y: f32,
        available: f32,
        forced_width: Option<f32>,
        depth: usize,
    ) -> Size {
        if !self.enter(id, depth) || self.is_hidden(id) {
            return Size::default();
        }
        let style = self.style(id).clone();
        if style.position == "relative" {
            x += resolve(style.left, available)
                .unwrap_or_else(|| -resolve(style.right, available).unwrap_or(0.0));
            y += resolve(style.top, self.viewport.height)
                .unwrap_or_else(|| -resolve(style.bottom, self.viewport.height).unwrap_or(0.0));
        }
        let padding = self.padding(id, available);
        let border = self.borders(id);
        let width = forced_width
            .map(extent)
            .unwrap_or_else(|| self.width_for(id, available, available));
        let inner_width = (width - padding.horizontal() - border.horizontal()).max(0.0);
        let inner_x = x + border.left + padding.left;
        let inner_y = y + border.top + padding.top;
        let paint_start = self.commands.len();
        // Reserve paint slots before descendants, then fill in the height.
        for _ in 0..5 {
            self.push(DrawCommand::Rect {
                rect: rect(x, y, 0.0, 0.0),
                color: rgba(0, 0, 0, 0),
                radius: 0.0,
            });
        }
        let hit_start = self.hits.len();
        self.hits.push(HitRegion {
            node: id,
            rect: rect(x, y, width, 0.0),
        });
        let clip_index = if matches!(style.overflow.as_str(), "hidden" | "clip") {
            let index = self.commands.len();
            self.push(DrawCommand::PushClip {
                rect: Rect::default(),
            });
            Some(index)
        } else {
            None
        };
        let tag = self.doc.tag(id).unwrap_or("").to_owned();
        let children = self.doc.nodes[id].children.clone();
        let natural_height = if matches!(tag.as_str(), "img" | "svg" | "canvas" | "video") {
            self.paint_replaced(id, &tag, inner_x, inner_y, inner_width)
        } else if matches!(tag.as_str(), "input" | "textarea" | "select") {
            self.paint_control(id, &tag, inner_x, inner_y, inner_width)
        } else if matches!(self.doc.nodes[id].kind, NodeKind::Text(_)) {
            self.layout_inline(
                &[id],
                inner_x,
                inner_y,
                inner_width,
                &style.text_align,
                depth + 1,
            )
        } else if style.display == Display::Flex {
            self.layout_flex(&children, inner_x, inner_y, inner_width, &style, depth + 1)
        } else if style.display == Display::Grid {
            self.layout_grid(&children, inner_x, inner_y, inner_width, &style, depth + 1)
        } else if tag == "table" {
            self.layout_table(&children, inner_x, inner_y, inner_width, depth + 1)
        } else {
            self.layout_flow(
                &children,
                inner_x,
                inner_y,
                inner_width,
                &style.text_align,
                depth + 1,
            )
        };
        let extras = padding.vertical() + border.vertical();
        let css_to_border = if style.box_sizing == "border-box" {
            0.0
        } else {
            extras
        };
        let mut height = resolve(style.height, self.viewport.height)
            .map(|value| value + css_to_border)
            .unwrap_or(natural_height + extras);
        if let Some(max) = resolve(style.max_height, self.viewport.height) {
            height = height.min(max + css_to_border);
        }
        if let Some(min) = resolve(style.min_height, self.viewport.height) {
            height = height.max(min + css_to_border);
        }
        height = extent(height).max(extras);
        // Canvas background propagation is emitted once before the root box.
        let paint_height = height;
        let background = if self.canvas_background_node == Some(id) {
            Color::TRANSPARENT
        } else {
            faded(style.background_color, style.opacity)
        };
        let border_color = faded(style.border_color, style.opacity);
        let mut replacements = [
            DrawCommand::Rect {
                rect: rect(x, y, width, paint_height),
                color: background,
                radius: style.border_radius,
            },
            DrawCommand::Rect {
                rect: rect(x, y, width, border.top),
                color: border_color,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(x + width - border.right, y, border.right, height),
                color: border_color,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(x, y + height - border.bottom, width, border.bottom),
                color: border_color,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(x, y, border.left, height),
                color: border_color,
                radius: 0.0,
            },
        ];
        if self.rounded_border(id) {
            replacements[0] = DrawCommand::Rect {
                rect: rect(x, y, width, paint_height),
                color: border_color,
                radius: style.border_radius,
            };
            replacements[1] = DrawCommand::Rect {
                rect: rect(
                    x + border.left,
                    y + border.top,
                    width - border.horizontal(),
                    paint_height - border.vertical(),
                ),
                color: background,
                radius: (style.border_radius - border.top).max(0.0),
            };
            for command in &mut replacements[2..] {
                *command = DrawCommand::Rect {
                    rect: rect(x, y, 0.0, 0.0),
                    color: Color::TRANSPARENT,
                    radius: 0.0,
                };
            }
        }
        for (offset, command) in replacements.into_iter().enumerate() {
            if let Some(slot) = self.commands.get_mut(paint_start + offset) {
                *slot = command;
            }
        }
        self.hits[hit_start].rect.height = height;
        if tag == "li" && style.list_style_type != "none" {
            self.paint_list_marker(id, inner_x, inner_y, &style);
        }
        if let Some(index) = clip_index {
            let clip = rect(
                x + border.left,
                y + border.top,
                width - border.horizontal(),
                height - border.vertical(),
            );
            if let Some(command) = self.commands.get_mut(index) {
                *command = DrawCommand::PushClip { rect: clip };
            }
            self.push(DrawCommand::PopClip);
            for hit in &mut self.hits[hit_start + 1..] {
                hit.rect = hit.rect.intersect(clip);
            }
        }
        Size { width, height }
    }

    fn layout_flow(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        align: &str,
        depth: usize,
    ) -> f32 {
        let mut cursor = y;
        let mut previous_bottom = 0.0;
        let mut inline = Vec::new();
        let mut positioned = Vec::new();
        for &child in children {
            let style = self.style(child);
            if self.is_hidden(child) {
                continue;
            }
            if style.position == "absolute" || style.position == "fixed" {
                positioned.push(child);
                continue;
            }
            if matches!(
                style.display,
                Display::Block | Display::Flex | Display::Grid
            ) {
                if !inline.is_empty() {
                    let h = self.layout_inline(
                        &inline,
                        x,
                        cursor + previous_bottom,
                        width,
                        align,
                        depth,
                    );
                    if h > 0.0 {
                        cursor += previous_bottom + h;
                        previous_bottom = 0.0;
                    }
                    inline.clear();
                }
                let margin = self.margins(child, width);
                cursor += collapsed_margin(previous_bottom, margin.top);
                let available = (width - margin.horizontal()).max(0.0);
                let child_width = self.width_for(child, available, width);
                let child_x = x + self.block_left(child, width, child_width, margin);
                let child_size =
                    self.layout_box(child, child_x, cursor, width, Some(child_width), depth);
                cursor += child_size.height;
                previous_bottom = margin.bottom;
            } else {
                inline.push(child);
            }
            if cursor >= MAX_EXTENT || self.visits >= MAX_VISITS {
                break;
            }
        }
        if !inline.is_empty() {
            let h = self.layout_inline(&inline, x, cursor + previous_bottom, width, align, depth);
            if h > 0.0 {
                cursor += previous_bottom + h;
                previous_bottom = 0.0;
            }
        }
        let flow_height = extent(cursor - y + previous_bottom);
        for child in positioned {
            let style = self.style(child).clone();
            let fixed = style.position == "fixed";
            let reference_width = if fixed { self.viewport.width } else { width };
            let reference_height = if fixed {
                self.viewport.height
            } else {
                flow_height
            };
            let bx = if fixed { 0.0 } else { x };
            let by = if fixed { 0.0 } else { y };
            let left = resolve(style.left, reference_width);
            let right = resolve(style.right, reference_width);
            let mut child_width = self.width_for(child, reference_width, reference_width);
            if matches!(style.width, Length::Auto) {
                child_width = match (left, right) {
                    (Some(l), Some(r)) => (reference_width - l - r).max(0.0),
                    _ => self
                        .intrinsic_width(child, reference_width)
                        .min(reference_width),
                };
            }
            let px = bx
                + left.unwrap_or_else(|| {
                    reference_width - right.unwrap_or(reference_width - child_width) - child_width
                });
            let top = resolve(style.top, reference_height);
            let bottom = resolve(style.bottom, reference_height);
            let fragment = self.fragment(child, reference_width, child_width, depth);
            let py = by
                + top.unwrap_or_else(|| {
                    bottom
                        .map(|b| reference_height - b - fragment.size.height)
                        .unwrap_or(0.0)
                });
            self.append_fragment(fragment, px, py);
        }
        flow_height
    }

    fn block_left(&self, id: NodeId, width: f32, box_width: f32, margin: Sides) -> f32 {
        let style = self.style(id);
        let free = (width - box_width - margin.horizontal()).max(0.0);
        match (
            matches!(style.margin.left, Length::Auto),
            matches!(style.margin.right, Length::Auto),
        ) {
            (true, true) => margin.left + free / 2.0,
            (true, false) => margin.left + free,
            _ => margin.left,
        }
    }

    fn fragment(&mut self, id: NodeId, available: f32, width: f32, depth: usize) -> Fragment {
        let command_start = self.commands.len();
        let hit_start = self.hits.len();
        let size = self.layout_box(id, 0.0, 0.0, available, Some(width), depth);
        Fragment {
            size,
            commands: self.commands.split_off(command_start),
            hits: self.hits.split_off(hit_start),
            rounded_border: self.rounded_border(id),
        }
    }

    fn rounded_border(&self, id: NodeId) -> bool {
        let style = self.style(id);
        let border = self.borders(id);
        style.border_radius > 0.0
            && self.canvas_background_node != Some(id)
            && style.background_color.a == 255
            && border.top > 0.0
            && border.top == border.right
            && border.top == border.bottom
            && border.top == border.left
    }

    fn append_fragment(&mut self, mut fragment: Fragment, x: f32, y: f32) {
        for command in &mut fragment.commands {
            translate(command, x, y);
        }
        for hit in &mut fragment.hits {
            hit.rect.x += x;
            hit.rect.y += y;
        }
        self.commands.extend(
            fragment
                .commands
                .into_iter()
                .take(MAX_COMMANDS.saturating_sub(self.commands.len())),
        );
        self.hits.extend(
            fragment
                .hits
                .into_iter()
                .take(MAX_COMMANDS.saturating_sub(self.hits.len())),
        );
    }

    fn collect_inline(
        &mut self,
        id: NodeId,
        owner: NodeId,
        width: f32,
        depth: usize,
        output: &mut Vec<InlineItem>,
    ) {
        if !self.enter(id, depth) || self.is_hidden(id) {
            return;
        }
        match &self.doc.nodes[id].kind {
            NodeKind::Text(text) => {
                let text = text.chars().take(self.glyphs_left).collect::<String>();
                self.tokenize(id, owner, &text, output);
            }
            NodeKind::Element(_) => {
                let tag = self.doc.tag(id).unwrap_or("");
                if tag == "br" {
                    output.push(InlineItem {
                        node: id,
                        owner: id,
                        kind: InlineKind::Break,
                        width: 0.0,
                        height: self.style(id).line_height,
                        preserve: true,
                    });
                    return;
                }
                if matches!(
                    tag,
                    "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
                ) || self.style(id).display == Display::InlineBlock
                {
                    let margin = self.margins(id, width);
                    let inner_width = if matches!(self.style(id).width, Length::Auto)
                        && !matches!(
                            tag,
                            "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
                        ) {
                        self.intrinsic_width(id, width).min(width)
                    } else {
                        self.width_for(id, width, width)
                    };
                    let fragment = self.fragment(id, width, inner_width, depth + 1);
                    output.push(InlineItem {
                        node: id,
                        owner: id,
                        width: fragment.size.width + margin.horizontal(),
                        height: fragment.size.height + margin.vertical(),
                        kind: InlineKind::Box(fragment),
                        preserve: false,
                    });
                } else {
                    let children = self.doc.nodes[id].children.clone();
                    for child in children {
                        self.collect_inline(child, id, width, depth + 1, output);
                    }
                }
            }
            _ => {
                let children = self.doc.nodes[id].children.clone();
                for child in children {
                    self.collect_inline(child, owner, width, depth + 1, output);
                }
            }
        }
    }

    fn tokenize(&mut self, id: NodeId, owner: NodeId, text: &str, output: &mut Vec<InlineItem>) {
        let style = self.style(id).clone();
        let preserve = matches!(
            style.white_space.as_str(),
            "pre" | "pre-wrap" | "break-spaces"
        );
        let newlines = preserve || style.white_space == "pre-line";
        let mut word = String::new();
        let mut whitespace = String::new();
        let mut last_cr = false;
        for character in text.chars().take(self.glyphs_left) {
            self.glyphs_left = self.glyphs_left.saturating_sub(1);
            let is_cr = character == '\r';
            if character == '\n' && last_cr {
                last_cr = false;
                continue;
            }
            last_cr = is_cr;
            if matches!(character, '\n' | '\r') && newlines {
                self.emit_text(id, owner, &mut word, false, preserve, output);
                self.emit_text(id, owner, &mut whitespace, true, preserve, output);
                output.push(InlineItem {
                    node: id,
                    owner,
                    kind: InlineKind::Break,
                    width: 0.0,
                    height: line_height(&style),
                    preserve,
                });
            } else if character.is_ascii_whitespace() {
                self.emit_text(id, owner, &mut word, false, preserve, output);
                if preserve {
                    if character == '\t' {
                        whitespace.push_str("    ");
                    } else {
                        whitespace.push(' ');
                    }
                } else if whitespace.is_empty() {
                    whitespace.push(' ');
                }
            } else {
                self.emit_text(id, owner, &mut whitespace, true, preserve, output);
                if character != '\0' {
                    word.push(character);
                }
            }
        }
        self.emit_text(id, owner, &mut word, false, preserve, output);
        self.emit_text(id, owner, &mut whitespace, true, preserve, output);
    }

    fn emit_text(
        &self,
        id: NodeId,
        owner: NodeId,
        text: &mut String,
        space: bool,
        preserve: bool,
        output: &mut Vec<InlineItem>,
    ) {
        if text.is_empty() {
            return;
        }
        let style = self.style(id);
        let value = std::mem::take(text);
        let width = self.measure(&value, style);
        output.push(InlineItem {
            node: id,
            owner,
            kind: if space {
                InlineKind::Space(value)
            } else {
                InlineKind::Text(value)
            },
            width,
            height: line_height(style),
            preserve,
        });
    }

    fn layout_inline(
        &mut self,
        nodes: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        align: &str,
        depth: usize,
    ) -> f32 {
        let mut tokens = Vec::new();
        for &id in nodes {
            self.collect_inline(id, id, width, depth, &mut tokens);
        }
        let mut line = Vec::new();
        let mut line_width = 0.0;
        let mut cursor = y;
        let mut pending_break = false;
        for token in tokens {
            if matches!(token.kind, InlineKind::Break) {
                cursor += self
                    .paint_line(std::mem::take(&mut line), x, cursor, width, align, false)
                    .max(token.height);
                line_width = 0.0;
                pending_break = true;
                continue;
            }
            let space = matches!(token.kind, InlineKind::Space(_));
            if space
                && !token.preserve
                && (line.is_empty()
                    || line
                        .last()
                        .is_some_and(|item: &InlineItem| matches!(item.kind, InlineKind::Space(_))))
            {
                continue;
            }
            let can_wrap = !matches!(
                self.style(token.node).white_space.as_str(),
                "pre" | "nowrap"
            );
            if can_wrap && !line.is_empty() && line_width + token.width > width && !space {
                cursor += self.paint_line(std::mem::take(&mut line), x, cursor, width, align, true);
                line_width = 0.0;
            }
            line_width += token.width;
            line.push(token);
            pending_break = false;
        }
        if !line.is_empty() {
            cursor += self.paint_line(line, x, cursor, width, align, false);
        } else if pending_break {
            // A trailing <br> establishes an empty final line.
            cursor += nodes
                .first()
                .map(|id| line_height(self.style(*id)))
                .unwrap_or(0.0);
        }
        extent(cursor - y)
    }

    fn paint_line(
        &mut self,
        mut line: Vec<InlineItem>,
        x: f32,
        y: f32,
        available: f32,
        align: &str,
        wrapped: bool,
    ) -> f32 {
        while line
            .last()
            .is_some_and(|item| matches!(item.kind, InlineKind::Space(_)) && !item.preserve)
        {
            line.pop();
        }
        if line.is_empty() {
            return 0.0;
        }
        let width: f32 = line.iter().map(|item| item.width).sum();
        let baseline = line
            .iter()
            .map(|item| match item.kind {
                InlineKind::Text(_) | InlineKind::Space(_) => {
                    let size = font_size(self.style(item.node));
                    size * 0.95 + (item.height - size * 1.3).max(0.0) / 2.0
                }
                _ => item.height,
            })
            .fold(0.0, f32::max);
        let descent = line
            .iter()
            .filter_map(|item| match item.kind {
                InlineKind::Text(_) | InlineKind::Space(_) => {
                    let size = font_size(self.style(item.node));
                    Some(size * 0.35 + (item.height - size * 1.3).max(0.0) / 2.0)
                }
                _ => None,
            })
            .fold(0.0, f32::max);
        let height = line
            .iter()
            .map(|item| item.height)
            .fold(baseline + descent, f32::max);
        let space_count = line
            .iter()
            .filter(|item| matches!(item.kind, InlineKind::Space(_)))
            .count();
        let extra_space = if align == "justify" && wrapped && space_count > 0 {
            (available - width).max(0.0) / space_count as f32
        } else {
            0.0
        };
        let mut cursor = x + match align {
            "center" => (available - width).max(0.0) * 0.5,
            "right" | "end" => (available - width).max(0.0),
            _ => 0.0,
        };
        for item in line {
            let style = self.style(item.node).clone();
            let is_space = matches!(item.kind, InlineKind::Space(_));
            match item.kind {
                InlineKind::Text(text) | InlineKind::Space(text) => {
                    let text_y = y + baseline - style.font_size * 0.95;
                    let item_rect = rect(
                        cursor,
                        y,
                        item.width + if is_space { extra_space } else { 0.0 },
                        height,
                    );
                    let owner_style = self.style(item.owner);
                    if owner_style.background_color.a > 0 && owner_style.display == Display::Inline
                    {
                        self.push(DrawCommand::Rect {
                            rect: item_rect,
                            color: faded(owner_style.background_color, owner_style.opacity),
                            radius: owner_style.border_radius,
                        });
                    }
                    self.push(DrawCommand::Text {
                        x: cursor,
                        y: text_y,
                        text,
                        size: font_size(&style),
                        color: faded(style.color, style.opacity),
                        bold: style.font_weight >= 600,
                        italic: style.font_style == "italic" || style.font_style == "oblique",
                        monospace: monospace(&style),
                    });
                    if style.text_decoration.contains("underline") {
                        self.push(DrawCommand::Line {
                            x1: cursor,
                            y1: text_y + style.font_size * 1.12,
                            x2: cursor + item.width,
                            y2: text_y + style.font_size * 1.12,
                            color: style.color,
                            width: (style.font_size / 16.0).max(1.0),
                        });
                    }
                    if style.text_decoration.contains("line-through") {
                        self.push(DrawCommand::Line {
                            x1: cursor,
                            y1: text_y + style.font_size * 0.65,
                            x2: cursor + item.width,
                            y2: text_y + style.font_size * 0.65,
                            color: style.color,
                            width: (style.font_size / 16.0).max(1.0),
                        });
                    }
                    self.hits.push(HitRegion {
                        node: item.node,
                        rect: item_rect,
                    });
                }
                InlineKind::Box(fragment) => {
                    let margin = self.margins(item.node, available);
                    let offset = match style.vertical_align.as_str() {
                        "top" | "text-top" => 0.0,
                        "middle" => (height - item.height) / 2.0,
                        "bottom" | "text-bottom" => height - item.height,
                        _ => baseline - item.height,
                    };
                    self.append_fragment(fragment, cursor + margin.left, y + offset + margin.top);
                }
                InlineKind::Break => {}
            }
            cursor += item.width + if is_space { extra_space } else { 0.0 };
        }
        height
    }

    fn layout_flex(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        style: &ComputedStyle,
        depth: usize,
    ) -> f32 {
        let mut items = self.flow_items(children);
        if style.flex_direction.ends_with("reverse") {
            items.reverse();
        }
        let gap = extent(style.gap);
        if style.flex_direction.starts_with("column") {
            let mut cursor = y;
            for (index, id) in items.into_iter().enumerate() {
                if index > 0 {
                    cursor += gap;
                }
                let margin = self.margins(id, width);
                let available = (width - margin.horizontal()).max(0.0);
                let child_width = if style.align_items == "stretch" {
                    self.width_for(id, available, width)
                } else {
                    self.intrinsic_width(id, available).min(available)
                };
                let offset = match style.align_items.as_str() {
                    "center" => (available - child_width) / 2.0,
                    "flex-end" | "end" => available - child_width,
                    _ => 0.0,
                };
                cursor += margin.top;
                let size = self.layout_box(
                    id,
                    x + margin.left + offset,
                    cursor,
                    width,
                    Some(child_width),
                    depth,
                );
                cursor += size.height + margin.bottom;
            }
            return extent(cursor - y);
        }
        let bases: Vec<f32> = items
            .iter()
            .map(|&id| {
                let child = self.style(id);
                resolve(child.flex_basis, width)
                    .unwrap_or_else(|| {
                        if matches!(child.width, Length::Auto) {
                            self.intrinsic_width(id, width)
                        } else {
                            self.width_for(id, width, width)
                        }
                    })
                    .max(0.0)
            })
            .collect();
        let mut rows = Vec::<Vec<(NodeId, f32)>>::new();
        let mut row = Vec::new();
        let mut used = 0.0;
        for (id, base) in items.into_iter().zip(bases) {
            let outer = base + self.margins(id, width).horizontal();
            if style.flex_wrap != "nowrap" && !row.is_empty() && used + gap + outer > width {
                rows.push(std::mem::take(&mut row));
                used = 0.0;
            }
            if !row.is_empty() {
                used += gap;
            }
            row.push((id, base));
            used += outer;
        }
        if !row.is_empty() {
            rows.push(row);
        }
        if style.flex_wrap == "wrap-reverse" {
            rows.reverse();
        }
        let mut cursor_y = y;
        for (row_index, row) in rows.into_iter().enumerate() {
            if row_index > 0 {
                cursor_y += gap;
            }
            let gaps = gap * row.len().saturating_sub(1) as f32;
            let total = row
                .iter()
                .map(|(id, base)| base + self.margins(*id, width).horizontal())
                .sum::<f32>()
                + gaps;
            let free = width - total;
            let grow: f32 = row
                .iter()
                .map(|(id, _)| extent(self.style(*id).flex_grow))
                .sum();
            let shrink: f32 = row
                .iter()
                .map(|(id, base)| extent(self.style(*id).flex_shrink) * base)
                .sum();
            let mut fragments = Vec::new();
            let mut row_height = 0.0f32;
            let mut row_width = gaps;
            for (id, base) in row {
                let child = self.style(id);
                let adjustment = if free > 0.0 && grow > 0.0 {
                    free * extent(child.flex_grow) / grow
                } else if free < 0.0 && shrink > 0.0 {
                    free * extent(child.flex_shrink) * base / shrink
                } else {
                    0.0
                };
                let margin = self.margins(id, width);
                let fragment = self.fragment(id, width, (base + adjustment).max(0.0), depth);
                row_height = row_height.max(fragment.size.height + margin.vertical());
                row_width += fragment.size.width + margin.horizontal();
                fragments.push((id, fragment, margin));
            }
            let remaining = (width - row_width).max(0.0);
            let (offset, between) =
                distribution(&style.justify_content, remaining, fragments.len());
            let mut cursor_x = x + offset;
            for (id, mut fragment, margin) in fragments {
                let align = style.align_items.as_str();
                let free_cross = (row_height - fragment.size.height - margin.vertical()).max(0.0);
                let y_offset = match align {
                    "center" => free_cross * 0.5,
                    "end" | "flex-end" => free_cross,
                    _ => 0.0,
                };
                if align == "stretch" && matches!(self.style(id).height, Length::Auto) {
                    stretch_fragment(&mut fragment, row_height - margin.vertical());
                }
                let advance = fragment.size.width + margin.horizontal();
                self.append_fragment(
                    fragment,
                    cursor_x + margin.left,
                    cursor_y + margin.top + y_offset,
                );
                cursor_x += advance + gap + between;
            }
            cursor_y += row_height;
        }
        extent(cursor_y - y)
    }

    fn layout_grid(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        style: &ComputedStyle,
        depth: usize,
    ) -> f32 {
        let items = self.flow_items(children);
        let tracks = if style.grid_template_columns.is_empty() {
            vec![Length::Auto]
        } else {
            style.grid_template_columns.clone()
        };
        let count = tracks.len().clamp(1, 1024);
        let gap = extent(style.gap);
        let available = (width - gap * count.saturating_sub(1) as f32).max(0.0);
        let fixed: f32 = tracks
            .iter()
            .take(count)
            .filter_map(|track| resolve(*track, width))
            .map(|value| value.max(0.0))
            .sum();
        let autos = tracks
            .iter()
            .take(count)
            .filter(|track| matches!(track, Length::Auto))
            .count();
        let fraction_total: f64 = tracks
            .iter()
            .take(count)
            .filter_map(|track| match track {
                Length::Fr(weight) => Some(f64::from(finite(*weight, 0.0).max(0.0))),
                _ => None,
            })
            .sum();
        let remaining = (available - fixed).max(0.0);
        let auto_width = if fraction_total > 0.0 {
            0.0
        } else {
            remaining / autos.max(1) as f32
        };
        let widths: Vec<f32> = tracks
            .iter()
            .take(count)
            .map(|track| match track {
                Length::Fr(weight) => {
                    remaining
                        * (f64::from(finite(*weight, 0.0).max(0.0)) / fraction_total.max(1.0))
                            as f32
                }
                _ => resolve(*track, width).unwrap_or(auto_width).max(0.0),
            })
            .collect();
        let mut cursor_y = y;
        for (row_index, row) in items.chunks(count).enumerate() {
            if row_index > 0 {
                cursor_y += gap;
            }
            let mut fragments = Vec::new();
            let mut row_height = 0.0f32;
            for (column, &id) in row.iter().enumerate() {
                let margin = self.margins(id, widths[column]);
                let child_width = self.width_for(
                    id,
                    (widths[column] - margin.horizontal()).max(0.0),
                    widths[column],
                );
                let fragment = self.fragment(id, widths[column], child_width, depth);
                row_height = row_height.max(fragment.size.height + margin.vertical());
                fragments.push((id, fragment, margin));
            }
            let mut cursor_x = x;
            for (column, (id, mut fragment, margin)) in fragments.into_iter().enumerate() {
                let offset = match style.align_items.as_str() {
                    "center" => {
                        (row_height - fragment.size.height - margin.vertical()).max(0.0) / 2.0
                    }
                    "end" | "flex-end" => {
                        (row_height - fragment.size.height - margin.vertical()).max(0.0)
                    }
                    _ => 0.0,
                };
                if style.align_items == "stretch" && matches!(self.style(id).height, Length::Auto) {
                    stretch_fragment(&mut fragment, row_height - margin.vertical());
                }
                self.append_fragment(
                    fragment,
                    cursor_x + margin.left,
                    cursor_y + margin.top + offset,
                );
                cursor_x += widths[column] + gap;
            }
            cursor_y += row_height;
        }
        extent(cursor_y - y)
    }

    fn layout_table(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        depth: usize,
    ) -> f32 {
        struct Cell {
            node: NodeId,
            row: usize,
            column: usize,
            columns: usize,
            rows: usize,
            fragment: Option<Fragment>,
        }
        let mut rows = Vec::new();
        let mut captions = Vec::new();
        let mut pending: Vec<NodeId> = children.iter().rev().copied().collect();
        let mut scanned = 0usize;
        while let Some(id) = pending.pop() {
            scanned += 1;
            if scanned > 4096 {
                break;
            }
            if self.is_hidden(id) {
                continue;
            }
            match self.doc.tag(id) {
                Some("tr") => rows.push(id),
                Some("caption") => captions.push(id),
                Some("thead" | "tbody" | "tfoot") => {
                    pending.extend(self.doc.nodes[id].children.iter().rev().copied());
                }
                _ => {}
            }
        }
        let mut caption_height = 0.0;
        for caption in captions {
            let fragment = self.fragment(caption, width, width, depth);
            let height = fragment.size.height;
            self.append_fragment(fragment, x, y + caption_height);
            caption_height += height;
        }
        if rows.is_empty() {
            return caption_height;
        }
        let mut cells = Vec::new();
        let mut occupied = [0usize; 128];
        let mut column_count = 1usize;
        for (row, &id) in rows.iter().enumerate() {
            let mut column = 0usize;
            for &node in &self.doc.nodes[id].children {
                if self.is_hidden(node) || !matches!(self.doc.tag(node), Some("td" | "th")) {
                    continue;
                }
                while column < occupied.len() && occupied[column] > 0 {
                    column += 1;
                }
                if column >= occupied.len() {
                    break;
                }
                let columns = self.attr_number(node, "colspan").unwrap_or(1.0) as usize;
                let columns = columns.clamp(1, occupied.len() - column);
                let rowspan = self.attr_number(node, "rowspan").unwrap_or(1.0) as usize;
                let rowspan = if rowspan == 0 {
                    rows.len() - row
                } else {
                    rowspan.min(rows.len() - row)
                };
                for slot in &mut occupied[column..column + columns] {
                    *slot = rowspan;
                }
                cells.push(Cell {
                    node,
                    row,
                    column,
                    columns,
                    rows: rowspan,
                    fragment: None,
                });
                column += columns;
                column_count = column_count.max(column);
            }
            for slot in &mut occupied {
                *slot = slot.saturating_sub(1);
            }
        }
        // Track sizing accepts explicit cell widths, distributing remaining space
        // over auto tracks. Full intrinsic table sizing is a separate algorithm.
        let spacing = 2.0;
        let available = (width - spacing * (column_count + 1) as f32).max(0.0);
        let mut tracks = vec![0.0f32; column_count];
        for cell in &cells {
            if let Some(value) = resolve(self.style(cell.node).width, width)
                .or_else(|| self.attr_number(cell.node, "width"))
            {
                let share = (value - spacing * cell.columns.saturating_sub(1) as f32).max(0.0)
                    / cell.columns as f32;
                for track in &mut tracks[cell.column..cell.column + cell.columns] {
                    *track = track.max(share);
                }
            }
        }
        let fixed = tracks.iter().sum::<f32>();
        let autos = tracks.iter().filter(|track| **track == 0.0).count();
        let remaining = (available - fixed).max(0.0);
        if autos > 0 {
            for track in &mut tracks {
                if *track == 0.0 {
                    *track = remaining / autos as f32;
                }
            }
        } else {
            for track in &mut tracks {
                *track += remaining / column_count as f32;
            }
        }
        let mut heights: Vec<f32> = rows
            .iter()
            .map(|&id| {
                resolve(self.style(id).height, self.viewport.height)
                    .unwrap_or(0.0)
                    .max(0.0)
            })
            .collect();
        for cell in &mut cells {
            let cell_width = tracks[cell.column..cell.column + cell.columns]
                .iter()
                .sum::<f32>()
                + spacing * cell.columns.saturating_sub(1) as f32;
            let fragment = self.fragment(cell.node, cell_width, cell_width, depth + 1);
            if cell.rows == 1 {
                heights[cell.row] = heights[cell.row].max(fragment.size.height);
            }
            cell.fragment = Some(fragment);
        }
        for cell in &cells {
            if cell.rows > 1 {
                let tracks = &mut heights[cell.row..cell.row + cell.rows];
                let allocated =
                    tracks.iter().sum::<f32>() + spacing * cell.rows.saturating_sub(1) as f32;
                let extra = (cell
                    .fragment
                    .as_ref()
                    .map(|fragment| fragment.size.height)
                    .unwrap_or(0.0)
                    - allocated)
                    .max(0.0)
                    / cell.rows as f32;
                for height in tracks {
                    *height += extra;
                }
            }
        }
        let mut row_y = Vec::with_capacity(rows.len());
        let mut cursor = y + caption_height + spacing;
        for (row, &id) in rows.iter().enumerate() {
            row_y.push(cursor);
            let style = self.style(id);
            let row_rect = rect(
                x + spacing,
                cursor,
                available + spacing * column_count.saturating_sub(1) as f32,
                heights[row],
            );
            self.push(DrawCommand::Rect {
                rect: row_rect,
                color: faded(style.background_color, style.opacity),
                radius: 0.0,
            });
            self.hits.push(HitRegion {
                node: id,
                rect: row_rect,
            });
            cursor += heights[row] + spacing;
        }
        for cell in cells {
            let Some(mut fragment) = cell.fragment else {
                continue;
            };
            let cell_height = heights[cell.row..cell.row + cell.rows].iter().sum::<f32>()
                + spacing * cell.rows.saturating_sub(1) as f32;
            let extra = (cell_height - fragment.size.height).max(0.0);
            let offset = match self.style(cell.node).vertical_align.as_str() {
                "middle" => extra / 2.0,
                "bottom" | "text-bottom" => extra,
                _ => 0.0,
            };
            let content_start =
                if matches!(fragment.commands.get(5), Some(DrawCommand::PushClip { .. })) {
                    6
                } else {
                    5
                };
            for command in fragment.commands.iter_mut().skip(content_start) {
                translate(command, 0.0, offset);
            }
            for hit in fragment.hits.iter_mut().skip(1) {
                hit.rect.y += offset;
            }
            stretch_fragment(&mut fragment, cell_height);
            let cell_x = x
                + spacing
                + tracks[..cell.column].iter().sum::<f32>()
                + spacing * cell.column as f32;
            self.append_fragment(fragment, cell_x, row_y[cell.row]);
        }
        extent(cursor - y)
    }

    fn flow_items(&self, children: &[NodeId]) -> Vec<NodeId> {
        children
            .iter()
            .copied()
            .filter(|&id| {
                if self.is_hidden(id) {
                    return false;
                }
                match self.doc.nodes.get(id).map(|node| &node.kind) {
                    Some(NodeKind::Text(text)) => !text.trim().is_empty(),
                    Some(_) => true,
                    None => false,
                }
            })
            .collect()
    }

    fn intrinsic_width(&self, id: NodeId, available: f32) -> f32 {
        let style = self.style(id);
        if let Some(width) = resolve(style.width, available) {
            let extra = if style.box_sizing == "border-box" {
                0.0
            } else {
                self.padding(id, available).horizontal() + self.borders(id).horizontal()
            };
            return extent(width + extra);
        }
        if matches!(
            self.doc.tag(id),
            Some("img" | "svg" | "canvas" | "video" | "input" | "textarea")
        ) {
            return self.width_for(id, available, available);
        }
        // Do not recurse through an adversarial tree for intrinsic sizing.
        let mut stack = vec![(id, 0usize)];
        let mut measured = 0.0f32;
        let mut count = 0usize;
        while let Some((node, depth)) = stack.pop() {
            let work_left = self.intrinsic_work_left.get();
            if count >= 4096 || depth > MAX_DEPTH || work_left == 0 {
                break;
            }
            self.intrinsic_work_left.set(work_left - 1);
            count += 1;
            let Some(node_value) = self.doc.nodes.get(node) else {
                continue;
            };
            if self.is_hidden(node) {
                continue;
            }
            if let NodeKind::Text(text) = &node_value.kind {
                let bounded: String = text
                    .chars()
                    .take(4096.min(self.intrinsic_work_left.get()))
                    .collect();
                self.intrinsic_work_left.set(
                    self.intrinsic_work_left
                        .get()
                        .saturating_sub(bounded.chars().count()),
                );
                measured += self.measure(&bounded, self.style(node));
            } else {
                stack.extend(
                    node_value
                        .children
                        .iter()
                        .rev()
                        .map(|&child| (child, depth + 1)),
                );
            }
            if measured >= available {
                break;
            }
        }
        (measured + self.padding(id, available).horizontal() + self.borders(id).horizontal())
            .clamp(0.0, MAX_EXTENT)
    }

    fn measure(&self, text: &str, style: &ComputedStyle) -> f32 {
        finite(
            self.fonts.measure(
                text,
                font_size(style),
                style.font_weight >= 600,
                style.font_style == "italic" || style.font_style == "oblique",
                monospace(style),
            ),
            0.0,
        )
        .max(0.0)
    }

    fn attr_number(&self, id: NodeId, attribute: &str) -> Option<f32> {
        self.doc
            .attr(id, attribute)?
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(extent)
    }

    fn natural_size(&self, id: NodeId) -> Size {
        Size {
            width: self
                .attr_number(id, "data-eris-natural-width")
                .or_else(|| self.attr_number(id, "width"))
                .unwrap_or(300.0),
            height: self
                .attr_number(id, "data-eris-natural-height")
                .or_else(|| self.attr_number(id, "height"))
                .unwrap_or(150.0),
        }
    }

    fn paint_replaced(&mut self, id: NodeId, tag: &str, x: f32, y: f32, width: f32) -> f32 {
        let style = self.style(id).clone();
        let specified = resolve(style.height, self.viewport.height)
            .map(|height| {
                if style.box_sizing == "border-box" {
                    (height - self.padding(id, width).vertical() - self.borders(id).vertical())
                        .max(0.0)
                } else {
                    height
                }
            })
            .or_else(|| self.attr_number(id, "height"));
        let natural = self.natural_size(id);
        let height = specified
            .unwrap_or(width * natural.height / natural.width.max(1.0))
            .max(0.0);
        if tag == "svg" {
            self.push(DrawCommand::Image {
                rect: rect(x, y, width, height),
                key: format!("eris-inline-svg:{id}"),
            });
        } else if tag == "img" {
            self.push(DrawCommand::Rect {
                rect: rect(x, y, width, height),
                color: rgba(236, 238, 242, 255),
                radius: 0.0,
            });
            if let Some(alt) = self.doc.attr(id, "alt").filter(|alt| !alt.is_empty()) {
                self.push(DrawCommand::Text {
                    x: x + 4.0,
                    y: y + 4.0,
                    text: alt.chars().take(256).collect(),
                    size: font_size(&style).min(16.0),
                    color: style.color,
                    bold: false,
                    italic: false,
                    monospace: false,
                });
            }
            if let Some(src) = self.doc.attr(id, "src").filter(|src| !src.is_empty()) {
                self.push(DrawCommand::Image {
                    rect: rect(x, y, width, height),
                    key: src.to_owned(),
                });
            }
        } else if tag == "video" {
            self.push(DrawCommand::Rect {
                rect: rect(x, y, width, height),
                color: rgba(20, 23, 30, 255),
                radius: 0.0,
            });
            if let Some(poster) = self.doc.attr(id, "poster") {
                self.push(DrawCommand::Image {
                    rect: rect(x, y, width, height),
                    key: poster.to_owned(),
                });
            }
        }
        extent(height)
    }

    fn paint_control(&mut self, id: NodeId, tag: &str, x: f32, y: f32, width: f32) -> f32 {
        let style = self.style(id).clone();
        let kind = self
            .doc
            .attr(id, "type")
            .unwrap_or("text")
            .to_ascii_lowercase();
        if tag == "input" && kind == "hidden" {
            return 0.0;
        }
        let check = tag == "input" && matches!(kind.as_str(), "checkbox" | "radio");
        let intrinsic_height = if check {
            16.0
        } else if tag == "textarea" {
            self.attr_number(id, "rows").unwrap_or(2.0) * line_height(&style) + 10.0
        } else {
            line_height(&style) + 10.0
        };
        let height = resolve(style.height, self.viewport.height)
            .map(|height| {
                if style.box_sizing == "border-box" {
                    (height - self.padding(id, width).vertical() - self.borders(id).vertical())
                        .max(0.0)
                } else {
                    height.max(0.0)
                }
            })
            .unwrap_or(intrinsic_height);
        self.push(DrawCommand::Rect {
            rect: rect(x, y, width, height),
            color: rgba(160, 167, 180, 255),
            radius: if kind == "radio" { 8.0 } else { 3.0 },
        });
        self.push(DrawCommand::Rect {
            rect: rect(x + 1.0, y + 1.0, (width - 2.0).max(0.0), height - 2.0),
            color: rgba(255, 255, 255, 255),
            radius: if kind == "radio" { 7.0 } else { 2.0 },
        });
        if check {
            if self.doc.attr(id, "checked").is_some() {
                self.push(DrawCommand::Rect {
                    rect: rect(x + 4.0, y + 4.0, (width - 8.0).max(0.0), height - 8.0),
                    color: rgba(50, 105, 225, 255),
                    radius: if kind == "radio" { 4.0 } else { 1.0 },
                });
            }
            return height;
        }
        let text = if tag == "textarea" {
            self.doc.text_content(id)
        } else if tag == "select" {
            let mut selected = None;
            let mut first = None;
            let mut stack = self.doc.nodes[id].children.clone();
            stack.reverse();
            let mut count = 0usize;
            while let Some(node) = stack.pop() {
                count += 1;
                if count > 4096 {
                    break;
                }
                if self.doc.tag(node) == Some("option") {
                    if first.is_none() {
                        first = Some(node);
                    }
                    if self.doc.attr(node, "selected").is_some() {
                        selected = Some(node);
                        break;
                    }
                }
                if let Some(node) = self.doc.nodes.get(node) {
                    stack.extend(node.children.iter().rev().copied());
                }
            }
            selected
                .or(first)
                .map(|node| self.doc.text_content(node))
                .unwrap_or_default()
        } else {
            self.doc
                .attr(id, "value")
                .or_else(|| self.doc.attr(id, "placeholder"))
                .unwrap_or("")
                .to_owned()
        };
        let text = if kind == "password" {
            "•".repeat(text.chars().count().min(256))
        } else {
            text.chars().take(4096).collect()
        };
        let text = fit_text(
            &text,
            (width - if tag == "select" { 25.0 } else { 12.0 }).max(0.0),
            |value| self.measure(value, &style),
        );
        self.push(DrawCommand::Text {
            x: x + 6.0,
            y: y + 5.0,
            text,
            size: font_size(&style),
            color: style.color,
            bold: style.font_weight >= 600,
            italic: false,
            monospace: monospace(&style),
        });
        if tag == "select" {
            self.push(DrawCommand::Text {
                x: x + (width - 18.0).max(0.0),
                y: y + 5.0,
                text: "▾".into(),
                size: font_size(&style),
                color: style.color,
                bold: false,
                italic: false,
                monospace: false,
            });
        }
        height
    }

    fn paint_list_marker(&mut self, id: NodeId, x: f32, y: f32, style: &ComputedStyle) {
        let marker = match style.list_style_type.as_str() {
            "decimal" | "decimal-leading-zero" => {
                let parent = self.doc.nodes[id].parent;
                let mut number = parent
                    .and_then(|parent| self.attr_number(parent, "start"))
                    .unwrap_or(1.0) as usize;
                if let Some(parent) = parent {
                    for &child in &self.doc.nodes[parent].children {
                        if child == id {
                            break;
                        }
                        if self.doc.tag(child) == Some("li") {
                            number += 1;
                        }
                    }
                }
                if let Some(value) = self.attr_number(id, "value") {
                    number = value as usize;
                }
                format!("{number}.")
            }
            "circle" => "◦".into(),
            "square" => "▪".into(),
            _ => "•".into(),
        };
        let marker_width = self.measure(&marker, style);
        self.push(DrawCommand::Text {
            x: x - marker_width - style.font_size * 0.5,
            y,
            text: marker,
            size: font_size(style),
            color: style.color,
            bold: false,
            italic: false,
            monospace: monospace(style),
        });
    }
}

fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn extent(value: f32) -> f32 {
    finite(value, 0.0).clamp(0.0, MAX_EXTENT)
}

fn resolve(length: Length, reference: f32) -> Option<f32> {
    match length {
        Length::Auto | Length::Fr(_) => None,
        Length::Px(value) => Some(finite(value, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT)),
        Length::Percent(value) => {
            Some(finite(reference * value / 100.0, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT))
        }
    }
}

fn font_size(style: &ComputedStyle) -> f32 {
    finite(style.font_size, 16.0).clamp(1.0, 1024.0)
}

fn line_height(style: &ComputedStyle) -> f32 {
    finite(style.line_height, font_size(style) * 1.3).clamp(1.0, 4096.0)
}

fn monospace(style: &ComputedStyle) -> bool {
    let family = style.font_family.to_ascii_lowercase();
    family.contains("monospace") || family.contains("courier") || family.contains("consolas")
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x: finite(x, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT),
        y: finite(y, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT),
        width: extent(width),
        height: extent(height),
    }
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

fn faded(mut color: Color, opacity: f32) -> Color {
    color.a = (f32::from(color.a) * finite(opacity, 1.0).clamp(0.0, 1.0)).round() as u8;
    color
}

fn collapsed_margin(previous: f32, next: f32) -> f32 {
    previous.max(next).max(0.0) + previous.min(next).min(0.0)
}

fn distribution(justify: &str, free: f32, count: usize) -> (f32, f32) {
    match justify {
        "center" => (free * 0.5, 0.0),
        "end" | "flex-end" => (free, 0.0),
        "space-between" if count > 1 => (0.0, free / (count - 1) as f32),
        "space-around" if count > 0 => (free / count as f32 / 2.0, free / count as f32),
        "space-evenly" if count > 0 => (free / (count + 1) as f32, free / (count + 1) as f32),
        _ => (0.0, 0.0),
    }
}

fn translate(command: &mut DrawCommand, dx: f32, dy: f32) {
    match command {
        DrawCommand::Rect { rect, .. }
        | DrawCommand::Image { rect, .. }
        | DrawCommand::PushClip { rect } => {
            rect.x += dx;
            rect.y += dy;
        }
        DrawCommand::Text { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        DrawCommand::Line { x1, y1, x2, y2, .. } => {
            *x1 += dx;
            *x2 += dx;
            *y1 += dy;
            *y2 += dy;
        }
        DrawCommand::PopClip => {}
    }
}

/// Stretch the principal box paint, keeping child content at its start edge.
fn stretch_fragment(fragment: &mut Fragment, height: f32) {
    let height = extent(height).max(fragment.size.height);
    if let Some(hit) = fragment.hits.first_mut() {
        hit.rect.height = height;
    }
    let old_height = fragment.size.height;
    for (index, command) in fragment.commands.iter_mut().take(5).enumerate() {
        if let DrawCommand::Rect { rect, .. } = command {
            match index {
                1 if fragment.rounded_border => rect.height += height - old_height,
                0 | 2 | 4 => rect.height = height,
                3 => rect.y += height - old_height,
                _ => {}
            }
        }
    }
    if let Some(DrawCommand::PushClip { rect }) = fragment.commands.get_mut(5) {
        rect.height += height - old_height;
    }
    fragment.size.height = height;
}

/// Use binary search at character boundaries, avoiding quadratic measurement.
fn fit_text(text: &str, width: f32, measure: impl Fn(&str) -> f32) -> String {
    let single_line = text.lines().next().unwrap_or("");
    if measure(single_line) <= width {
        return single_line.to_owned();
    }
    let boundaries: Vec<usize> = single_line
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(single_line.len()))
        .collect();
    let mut low = 0;
    let mut high = boundaries.len().saturating_sub(1);
    while low < high {
        let middle = (low + high).div_ceil(2);
        if measure(&single_line[..boundaries[middle]]) <= width {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    single_line[..boundaries[low]].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(source: &str, width: f32) -> (Document, LayoutResult) {
        let document = Document::parse(source);
        let styles = crate::css::compute_styles(&document, &document.stylesheets(), width, 400.0);
        let result = layout(&document, &styles, width, 400.0, &Fonts::new());
        (document, result)
    }

    fn bounds(document: &Document, result: &LayoutResult, selector: &str) -> Rect {
        let node = document.query_selector(selector).expect("element exists");
        result
            .hit_regions
            .iter()
            .find(|hit| hit.node == node)
            .expect("element is laid out")
            .rect
    }

    #[test]
    fn block_box_model_and_auto_margins() {
        let (doc, result) = render(
            "<style>body{margin:0}#box{width:100px;height:30px;padding:10px;border:2px solid red;margin-left:auto;margin-right:auto}</style><div id=box></div>",
            300.0,
        );
        let bounds = bounds(&doc, &result, "#box");
        assert_eq!(bounds.width, 124.0);
        assert_eq!(bounds.height, 54.0);
        assert_eq!(bounds.x, 88.0);
    }

    #[test]
    fn text_wraps_without_losing_words() {
        let (_, result) = render(
            "<style>body{margin:0}p{margin:0;font-size:16px}</style><p>alpha beta gamma delta epsilon</p>",
            100.0,
        );
        let words: Vec<(&str, f32)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, y, .. } if !text.trim().is_empty() => {
                    Some((text.as_str(), *y))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            words.iter().map(|(text, _)| *text).collect::<Vec<_>>(),
            ["alpha", "beta", "gamma", "delta", "epsilon"]
        );
        assert!(words.last().unwrap().1 > words.first().unwrap().1);
        assert!(words.windows(2).all(|pair| pair[1].1 >= pair[0].1));
    }

    #[test]
    fn flex_grow_distributes_remaining_space() {
        let (doc, result) = render(
            "<style>body{margin:0}#row{display:flex;width:300px;gap:10px}#row div{width:50px;height:20px;flex-grow:1}</style><div id=row><div id=a></div><div id=b></div></div>",
            400.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        assert_eq!(a.width, 145.0);
        assert_eq!(b.width, 145.0);
        assert_eq!(b.x - a.x, 155.0);
        assert_eq!(a.y, b.y);
    }

    #[test]
    fn grid_places_rows_and_fixed_tracks() {
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;grid-template-columns:80px 120px;gap:10px}#grid div{height:25px}</style><div id=grid><div id=a></div><div id=b></div><div id=c></div></div>",
            300.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        let c = bounds(&doc, &result, "#c");
        assert_eq!(a.width, 80.0);
        assert_eq!(b.width, 120.0);
        assert_eq!(b.x - a.x, 90.0);
        assert_eq!(c.y - a.y, 35.0);
    }

    #[test]
    fn padded_auto_and_fractional_grids_keep_tiles_inside_content_box() {
        for tracks in ["auto auto auto", "1fr 1fr 1fr"] {
            let source = format!(
                "<style>*{{box-sizing:border-box}}body{{margin:0}}section{{width:360px;padding:20px;border:2px solid black}}#grid{{display:grid;padding:10px;border:1px solid black;gap:12px;grid-template-columns:{tracks}}}#grid div{{padding:20px;border:1px solid blue}}</style><section><div id=grid><div id=a>A</div><div id=b>B</div><div id=c>C</div></div></section>"
            );
            let (doc, result) = render(&source, 400.0);
            let grid = bounds(&doc, &result, "#grid");
            let a = bounds(&doc, &result, "#a");
            let b = bounds(&doc, &result, "#b");
            let c = bounds(&doc, &result, "#c");
            assert_eq!(grid.width, 316.0, "{tracks}");
            assert_eq!(a.width, 90.0, "{tracks}");
            assert_eq!(b.x - (a.x + a.width), 12.0, "{tracks}");
            assert_eq!(a.x, grid.x + 11.0, "{tracks}");
            assert_eq!(c.x + c.width, grid.x + grid.width - 11.0, "{tracks}");
        }
    }

    #[test]
    fn fractional_tracks_share_space_after_fixed_tracks_and_gaps() {
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;width:400px;gap:20px;grid-template-columns:100px 1fr 2fr}#grid div{height:10px}</style><div id=grid><div id=a></div><div id=b></div><div id=c></div></div>",
            500.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        let c = bounds(&doc, &result, "#c");
        assert_eq!(a.width, 100.0);
        assert!((b.width - 260.0 / 3.0).abs() < 0.001);
        assert!((c.width - 520.0 / 3.0).abs() < 0.001);
        assert!((c.x + c.width - 400.0).abs() < 0.001);
    }

    #[test]
    fn percentage_tracks_keep_container_percentage_and_fr_uses_remainder() {
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;width:400px;gap:20px;grid-template-columns:100px 25% 1fr}#grid div{height:10px}</style><div id=grid><div id=a></div><div id=b></div><div id=c></div></div>",
            500.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 100.0);
        assert_eq!(bounds(&doc, &result, "#b").width, 100.0);
        assert_eq!(bounds(&doc, &result, "#c").width, 160.0);
        let (_, result) = render(
            "<style>body{margin:0}#grid{display:grid;width:400px;gap:20px;grid-template-columns:50% 50%}#grid div{height:10px}</style><div id=grid><div id=a></div><div id=b></div></div>",
            500.0,
        );
        let widths: Vec<f32> = result
            .hit_regions
            .iter()
            .filter(|hit| hit.rect.height == 10.0 && hit.rect.width == 200.0)
            .map(|hit| hit.rect.width)
            .collect();
        assert_eq!(widths, [200.0, 200.0]);
    }

    #[test]
    fn table_cells_follow_columns_and_spanning_rows() {
        let (doc, result) = render(
            "<style>body{margin:0}table{width:300px}td{padding:0;height:20px}</style><table><tr><td id=a rowspan=2>A</td><td id=b>B</td></tr><tr><td id=c>C</td></tr><tr><td id=d colspan=2>D</td></tr></table>",
            400.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        let c = bounds(&doc, &result, "#c");
        let d = bounds(&doc, &result, "#d");
        assert_eq!(a.y, b.y);
        assert_eq!(b.x, c.x);
        assert!(c.y > b.y);
        assert!(d.y >= a.y + a.height);
        assert_eq!(a.height, b.height + c.height + 2.0);
        assert_eq!(d.width, a.width + b.width + 2.0);
    }

    #[test]
    fn rounded_opaque_borders_leave_corner_pixels_clear() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>body{margin:0}div{width:50px;height:50px;border:3px solid red;border-radius:12px;background:blue}</style><div></div>",
            100.0,
        );
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[0], 0xffffff);
        assert_eq!(canvas.pixels[28], 0xff0000);
        assert_eq!(canvas.pixels[28 * 100 + 28], 0x0000ff);
    }

    #[test]
    fn body_background_propagates_across_canvas_without_double_blending() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>body{width:60px;margin:0 auto;background:rgba(255,0,0,0.5)}</style><p>page</p>",
            100.0,
        );
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[99 * 100], canvas.pixels[99 * 100 + 50]);
        assert_eq!(canvas.pixels[99 * 100], 0xff7f7f);
    }

    #[test]
    fn html_background_takes_precedence_over_body_propagation() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>html{background:blue}body{width:50px;height:20px;margin:0;background:red}</style>",
            100.0,
        );
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[0], 0xff0000);
        assert_eq!(canvas.pixels[99], 0x0000ff);
        assert_eq!(canvas.pixels[99 * 100], 0x0000ff);
    }

    #[test]
    fn image_css_width_preserves_natural_aspect_ratio() {
        let (doc, result) = render(
            "<style>body{margin:0}img{width:120px}</style><img id=photo src=photo.png data-eris-natural-width=400 data-eris-natural-height=200>",
            300.0,
        );
        let photo = bounds(&doc, &result, "#photo");
        assert_eq!(photo.width, 120.0);
        assert_eq!(photo.height, 60.0);
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Image { rect, key } if key == "photo.png" && rect.width == 120.0 && rect.height == 60.0)));
    }

    #[test]
    fn inline_svg_is_one_replaced_image() {
        let (doc, result) = render(
            "<style>body{margin:0}svg{width:200px}</style><svg id=vector data-eris-natural-width=40 data-eris-natural-height=20><text>vector label</text></svg>",
            300.0,
        );
        let node = doc.query_selector("#vector").unwrap();
        let geometry = bounds(&doc, &result, "#vector");
        assert_eq!(geometry.width, 200.0);
        assert_eq!(geometry.height, 100.0);
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Image { key, .. } if key == &format!("eris-inline-svg:{node}"))));
        assert!(
            !result
                .commands
                .iter()
                .any(|command| matches!(command, DrawCommand::Text { .. }))
        );
    }

    #[test]
    fn extreme_css_geometry_never_emits_nonfinite_coordinates() {
        let (_, result) = render(
            "<style>body{margin:0}div{display:flex;width:1e38px;padding:1e38%;gap:1e38px}span{width:1e38%;height:1e38px;position:relative;left:1e38%}</style><div><span>huge</span><span>page</span></div>",
            800.0,
        );
        assert!(result.content_height.is_finite());
        for command in &result.commands {
            let coordinates = match command {
                DrawCommand::Rect { rect, .. }
                | DrawCommand::Image { rect, .. }
                | DrawCommand::PushClip { rect } => {
                    vec![rect.x, rect.y, rect.width, rect.height]
                }
                DrawCommand::Text { x, y, size, .. } => vec![*x, *y, *size],
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    ..
                } => vec![*x1, *y1, *x2, *y2, *width],
                DrawCommand::PopClip => vec![],
            };
            assert!(coordinates.iter().all(|value| value.is_finite()));
        }
    }

    #[test]
    fn hidden_subtrees_never_paint_or_hit() {
        let (doc, result) = render(
            "<style>#hidden{display:none}</style><div id=hidden><a href='/secret'>secret</a></div><p>visible</p>",
            300.0,
        );
        let hidden = doc.query_selector("#hidden").unwrap();
        assert!(result.hit_regions.iter().all(|hit| hit.node != hidden));
        assert!(!result.commands.iter().any(
            |command| matches!(command, DrawCommand::Text { text, .. } if text.contains("secret"))
        ));
    }

    #[test]
    fn mixed_case_password_type_never_emits_plaintext() {
        let (_, result) = render("<input type=PaSsWoRd value='secret123'>", 300.0);
        let text: Vec<&str> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, ["•••••••••"]);
        assert!(!result.commands.iter().any(
            |command| matches!(command, DrawCommand::Text { text, .. } if text.contains("secret"))
        ));
    }

    #[test]
    fn mixed_case_checkbox_and_radio_use_intrinsic_control_geometry() {
        use crate::graphics::{Canvas, ImageStore};
        for kind in ["CHECKBOX", "ChEcKbOx", "RADIO", "RaDiO"] {
            let source = format!(
                "<style>body{{margin:0}}input{{width:auto;padding:0;border:0}}</style><input id=control type={kind} checked value='must not paint'>"
            );
            let (doc, result) = render(&source, 100.0);
            assert_eq!(
                bounds(&doc, &result, "#control"),
                rect(0.0, 0.0, 16.0, 16.0),
                "{kind}"
            );
            assert!(
                !result
                    .commands
                    .iter()
                    .any(|command| matches!(command, DrawCommand::Text { .. })),
                "{kind}"
            );
            let mut canvas = Canvas::new(100, 50).unwrap();
            canvas.paint(
                &result.commands,
                &Fonts::new(),
                &ImageStore::new(),
                0.0,
                0.0,
            );
            assert_eq!(canvas.pixels[8 * 100 + 8], 0x3269e1, "{kind}");
        }
    }

    #[test]
    fn checkbox_and_radio_defaults_do_not_inherit_text_input_width() {
        for kind in ["checkbox", "CHECKBOX", "radio", "RADIO"] {
            let source = format!("<style>body{{margin:0}}</style><input id=control type={kind}>");
            let (doc, result) = render(&source, 300.0);
            assert_eq!(
                bounds(&doc, &result, "#control"),
                rect(0.0, 0.0, 16.0, 16.0),
                "{kind}"
            );
        }
    }

    #[test]
    fn authored_checkbox_sizing_and_box_model_override_native_defaults() {
        let source = "<style>body{margin:0}input{width:40px;height:30px;padding:3px;border:2px solid red}</style><input id=control type=checkbox>";
        let (doc, result) = render(source, 300.0);
        assert_eq!(
            bounds(&doc, &result, "#control"),
            rect(0.0, 0.0, 50.0, 40.0)
        );
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Rect { rect: bounds, color, .. } if *bounds == rect(5.0, 5.0, 40.0, 30.0) && *color == rgba(160, 167, 180, 255))));
        let (doc, result) = render(
            &source.replace("width:40px", "box-sizing:border-box;width:40px"),
            300.0,
        );
        assert_eq!(
            bounds(&doc, &result, "#control"),
            rect(0.0, 0.0, 40.0, 30.0)
        );
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Rect { rect: bounds, color, .. } if *bounds == rect(5.0, 5.0, 30.0, 20.0) && *color == rgba(160, 167, 180, 255))));
    }

    #[test]
    fn mixed_case_hidden_input_neither_paints_nor_reserves_layout_space() {
        let (doc, result) = render(
            "<style>body{margin:0}input{display:block;width:200px;margin:30px}</style><input id=hidden type=HiDdEn value='secret'><span>visible</span>",
            300.0,
        );
        let hidden = doc.query_selector("#hidden").unwrap();
        assert!(result.hit_regions.iter().all(|hit| hit.node != hidden));
        let painted: Vec<_> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { x, y, text, .. } => Some((*x, *y, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(painted, [(0.0, 0.0, "visible")]);
    }

    #[test]
    fn nested_overflow_clips_pixels_and_link_hit_regions_but_keeps_borders() {
        use crate::graphics::{Canvas, ImageStore};
        for overflow in ["hidden", "clip"] {
            let source = format!(
                "<style>body{{margin:0}}#outer{{width:60px;height:60px;border:2px solid black;background:blue;overflow:{overflow}}}#inner{{width:50px;height:50px;margin:30px 0 0 30px;overflow:{overflow};background:red}}#link{{display:block;width:100px;height:100px;background:lime}}</style><div id=outer><div id=inner><a id=link href='/hidden'>Link</a></div></div>"
            );
            let (doc, result) = render(&source, 100.0);
            let link = bounds(&doc, &result, "#link");
            assert_eq!(link, rect(32.0, 32.0, 30.0, 30.0));
            assert_eq!(result.hit_test(70.0, 50.0), doc.query_selector("body"));
            let mut canvas = Canvas::new(100, 100).unwrap();
            canvas.paint(
                &result.commands,
                &Fonts::new(),
                &ImageStore::new(),
                0.0,
                0.0,
            );
            assert_eq!(canvas.pixels[55 * 100 + 55], 0x00ff00, "{overflow}");
            assert_eq!(canvas.pixels[55 * 100 + 25], 0x0000ff, "{overflow}");
            assert_eq!(canvas.pixels[55 * 100 + 70], 0xffffff, "{overflow}");
            assert_eq!(canvas.pixels[70 * 100 + 55], 0xffffff, "{overflow}");
            assert_eq!(canvas.pixels[63 * 100 + 40], 0x000000, "{overflow}");
        }
    }

    #[test]
    fn clipped_absolute_content_does_not_extend_document_scroll_height() {
        let (_, result) = render(
            "<style>body{margin:0}div{position:relative;width:100px;height:50px;overflow:hidden}a{position:absolute;top:1000px;display:block;width:50px;height:50px}</style><div><a href='/hidden'>Hidden</a></div>",
            200.0,
        );
        assert_eq!(result.content_height, 400.0);
        assert_eq!(result.hit_test(10.0, 1010.0), None);
    }

    #[test]
    fn clipping_in_grid_fragments_uses_positioned_coordinates() {
        use crate::graphics::{Canvas, ImageStore};
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;grid-template-columns:50px 50px;gap:20px}#grid div{height:30px;overflow:hidden}a{display:block;width:100px;height:100px;background:red}</style><div id=grid><div></div><div><a id=link href='/x'></a></div></div>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#link"), rect(70.0, 0.0, 50.0, 30.0));
        let mut canvas = Canvas::new(200, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[10 * 200 + 80], 0xff0000);
        assert_eq!(canvas.pixels[10 * 200 + 130], 0xffffff);
        assert_eq!(canvas.pixels[40 * 200 + 80], 0xffffff);
    }

    #[test]
    fn preformatted_newlines_and_spaces_survive() {
        let (_, result) = render(
            "<style>body{margin:0}pre{margin:0}</style><pre>a  b\nc</pre>",
            300.0,
        );
        let text: Vec<(&str, f32)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, y, .. } => Some((text.as_str(), *y)),
                _ => None,
            })
            .collect();
        assert!(text.iter().any(|(text, _)| *text == "  "));
        let a_y = text.iter().find(|(text, _)| *text == "a").unwrap().1;
        let c_y = text.iter().find(|(text, _)| *text == "c").unwrap().1;
        assert!(c_y > a_y);
    }

    #[test]
    fn deep_input_and_nonfinite_viewport_stay_bounded() {
        let source = format!("{}deep{}", "<div>".repeat(300), "</div>".repeat(300));
        let document = Document::parse(&source);
        let styles = crate::css::compute_styles(&document, &[], 800.0, 600.0);
        let result = layout(&document, &styles, f32::NAN, f32::INFINITY, &Fonts::new());
        assert!(result.content_height.is_finite());
        assert!(result.commands.len() <= MAX_COMMANDS);
        assert!(result.hit_regions.iter().all(|hit| {
            [hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height]
                .iter()
                .all(|v| v.is_finite())
        }));
    }

    #[test]
    fn margins_collapse_positive_and_negative_values() {
        assert_eq!(collapsed_margin(12.0, 20.0), 20.0);
        assert_eq!(collapsed_margin(-12.0, -20.0), -20.0);
        assert_eq!(collapsed_margin(-12.0, 20.0), 8.0);
    }

    #[test]
    fn untrusted_dimensions_are_finite_and_bounded() {
        assert_eq!(extent(f32::NAN), 0.0);
        assert_eq!(extent(f32::INFINITY), 0.0);
        assert_eq!(resolve(Length::Percent(50.0), 200.0), Some(100.0));
        let bounds = rect(f32::NAN, f32::INFINITY, f32::MAX, -1.0);
        assert!(bounds.x.is_finite() && bounds.y.is_finite());
        assert_eq!(bounds.width, MAX_EXTENT);
        assert_eq!(bounds.height, 0.0);
    }

    #[test]
    fn text_clipping_respects_unicode_boundaries() {
        let text = "a日本語🙂";
        assert_eq!(
            fit_text(text, 3.0, |value| value.chars().count() as f32),
            "a日本"
        );
        assert_eq!(
            fit_text(text, 0.0, |value| value.chars().count() as f32),
            ""
        );
    }

    #[test]
    fn hit_testing_prefers_later_painted_descendants() {
        let result = LayoutResult {
            commands: Vec::new(),
            hit_regions: vec![
                HitRegion {
                    node: 1,
                    rect: rect(0.0, 0.0, 100.0, 100.0),
                },
                HitRegion {
                    node: 2,
                    rect: rect(20.0, 20.0, 20.0, 20.0),
                },
            ],
            content_height: 100.0,
        };
        assert_eq!(result.hit_test(21.0, 21.0), Some(2));
        assert_eq!(result.hit_test(0.0, 0.0), Some(1));
        assert_eq!(result.hit_test(100.0, 100.0), None);
    }
}
