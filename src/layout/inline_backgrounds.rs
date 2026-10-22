//! Sliced inline decoration from retained owner geometry, independent of paint admission.
use super::*;
use crate::graphics::box_decoration::RoundedBox;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum InlineLayer {
    #[default]
    None,
    Clip,
    Color,
    Image,
    Border,
}

pub(super) const FIRST: u8 = 1;
pub(super) const LAST: u8 = 2;
const MAX_STRIP: f64 = 16_000_000.0;

// Pending layer geometry must obey the same origin rule as its completed
// decoration. Generic Rect translation clamps and would change image phase.
pub(super) fn translate_pending(
    command: &mut DrawCommand,
    owner: &mut PaintOwner,
    dx: f32,
    dy: f32,
) -> bool {
    if owner.inline_layer == InlineLayer::None {
        return false;
    }
    let area = match command {
        DrawCommand::Rect { rect, .. } | DrawCommand::PushClip { rect } => rect,
        _ => return false,
    };
    let x = f64::from(area.x) + f64::from(dx);
    let y = f64::from(area.y) + f64::from(dy);
    let moved = Rect {
        x: x as f32,
        y: y as f32,
        ..*area
    };
    if x.is_finite()
        && y.is_finite()
        && x.abs() <= f64::from(MAX_EXTENT)
        && y.abs() <= f64::from(MAX_EXTENT)
        && (RoundedBox {
            rect: moved,
            radii: [[0.0; 2]; 4],
        })
        .valid()
    {
        *area = moved;
    } else {
        *area = Rect::default();
        owner.inline_layer = InlineLayer::None;
        if let DrawCommand::Rect { color, .. } = command {
            *color = Color::TRANSPARENT;
        }
    }
    true
}

#[derive(Clone, Copy, Default)]
struct Metrics {
    widths: [f64; 3],
    heights: [f32; 3],
    border: Sides,
    padding: Sides,
    sides: u8,
    seen: bool,
    invalid: bool,
}

impl Metrics {
    fn add(&mut self, geometry: BoxGeometry, border: Sides, padding: Sides) {
        let boxes = [geometry.border, geometry.padding, geometry.content];
        for (index, area) in boxes.into_iter().enumerate() {
            if ![area.x, area.y, area.width, area.height]
                .into_iter()
                .all(f32::is_finite)
                || area.width < 0.0
                || area.height < 0.0
            {
                self.invalid = true;
                return;
            }
            self.widths[index] += f64::from(area.width);
            self.invalid |= self.widths[index] > MAX_STRIP;
        }
        // Resolve forward against the original containing width. Subtracting
        // translated endpoints can erase an inset or manufacture a negative one.
        if !self.seen {
            self.heights = boxes.map(|area| area.height);
            self.border.top = border.top;
            self.border.bottom = border.bottom;
            self.padding.top = padding.top;
            self.padding.bottom = padding.bottom;
        }
        if geometry.inline_sides & FIRST != 0 {
            self.border.left = border.left;
            self.padding.left = padding.left;
        }
        if geometry.inline_sides & LAST != 0 {
            self.border.right = border.right;
            self.padding.right = padding.right;
        }
        self.sides |= geometry.inline_sides;
        self.seen = true;
    }

    fn boxes(
        self,
        fragment: Rect,
        prefix: f32,
        radius: f32,
    ) -> Option<background_boxes::BackgroundBoxes> {
        if !self.seen || self.invalid || !prefix.is_finite() || prefix < 0.0 {
            return None;
        }
        let left = f64::from(fragment.x) - f64::from(prefix);
        if left.abs() > MAX_STRIP {
            return None;
        }
        let mut radii = [[radius; 2]; 4];
        if self.sides & FIRST == 0 {
            radii[0] = [0.0; 2];
            radii[3] = [0.0; 2];
        }
        if self.sides & LAST == 0 {
            radii[1] = [0.0; 2];
            radii[2] = [0.0; 2];
        }
        let outer = RoundedBox {
            rect: Rect {
                x: left as f32,
                y: fragment.y,
                width: self.widths[0] as f32,
                height: fragment.height,
            },
            radii,
        };
        let mut boxes = background_boxes::BackgroundBoxes::new(outer, self.border, self.padding)?;
        // Sum each selected box independently. Margins and descendant visual
        // movement never participate; omitted image commands cannot rephase it.
        boxes.padding.rect.width = self.widths[1] as f32;
        boxes.content.rect.width = self.widths[2] as f32;
        boxes.padding.rect.height = self.heights[1];
        boxes.content.rect.height = self.heights[2];
        (boxes.border.valid() && boxes.padding.valid() && boxes.content.valid()).then_some(boxes)
    }
}

fn fragment_clip(area: Rect, sides: u8) -> Option<Rect> {
    let left = if sides & FIRST != 0 {
        area.x.floor()
    } else {
        area.x
    };
    let right = area.x + area.width;
    let right = if sides & LAST != 0 {
        right.ceil()
    } else {
        right
    };
    let top = area.y.floor();
    let bottom = (area.y + area.height).ceil();
    let result = Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    };
    RoundedBox {
        rect: result,
        radii: [[0.0; 2]; 4],
    }
    .valid()
    .then_some(result)
}

impl Engine<'_> {
    pub(super) fn paint_inline_background(
        &mut self,
        geometry: BoxGeometry,
        prefix: f32,
        physical_sides: u8,
    ) {
        let node = geometry.node;
        if self.canvas_background_node == Some(node)
            || geometry.border.width <= 0.0
            || geometry.border.height <= 0.0
            || !prefix.is_finite()
            || prefix < 0.0
            || f64::from(prefix) > MAX_STRIP
        {
            return;
        }
        let style = self.style(node);
        if style.background_color.a == 0
            && style.backgrounds.image_slots().iter().all(Option::is_none)
            && (style.border_color.a == 0
                || self.borders(node).horizontal() + self.borders(node).vertical() == 0.0)
        {
            return;
        }
        // Color, authored image slots, border and the balanced clip pair are
        // one admission. None slots preserve authored indexing and geometry.
        let image_count = style.backgrounds.image_slots().len();
        if !self.admit_decoration_slots(image_count + 4) {
            return;
        }
        let saved = self.paint_owner;
        self.paint_owner = PaintOwner {
            node,
            background: true,
            paint_visibility: self.style(node).visibility,
            inline_layer: InlineLayer::Clip,
            inline_prefix: prefix,
            inline_sides: physical_sides,
            background_image_index: 0,
        };
        self.push(DrawCommand::PushClip {
            rect: geometry.border,
        });
        self.paint_owner.inline_layer = InlineLayer::Color;
        self.push(DrawCommand::Rect {
            rect: geometry.border,
            color: Color::TRANSPARENT,
            radius: 0.0,
        });
        for image_index in (0..image_count).rev() {
            self.paint_owner.inline_layer = InlineLayer::Image;
            self.paint_owner.background_image_index = image_index as u8;
            self.push(DrawCommand::Rect {
                rect: geometry.border,
                color: Color::TRANSPARENT,
                radius: 0.0,
            });
        }
        self.paint_owner.background_image_index = 0;
        self.paint_owner.inline_layer = InlineLayer::Border;
        self.push(DrawCommand::Rect {
            rect: geometry.border,
            color: Color::TRANSPARENT,
            radius: 0.0,
        });
        self.paint_owner.inline_layer = InlineLayer::None;
        self.push(DrawCommand::PopClip);
        self.paint_owner = saved;
        self.inline_gradients = true;
    }

    pub(super) fn finish_inline_backgrounds(&mut self) {
        if !self.inline_gradients {
            return;
        }
        self.inline_gradients = false;
        if self.doc.nodes.len() > MAX_VISITS
            || !background_boxes::spend(&mut self.position_work_left, self.commands.len())
        {
            return;
        }
        let (pending, extra_images) =
            self.owners
                .iter()
                .fold((0usize, 0usize), |(pending, extra), owner| {
                    (
                        pending + usize::from(owner.inline_layer != InlineLayer::None),
                        extra
                            + usize::from(
                                owner.inline_layer == InlineLayer::Image
                                    && owner.background_image_index != 0,
                            ),
                    )
                });
        let cost = self
            .doc
            .nodes
            .len()
            .saturating_add(self.boxes.len().saturating_mul(8))
            .saturating_add(self.commands.len())
            .saturating_add(pending.saturating_mul(background_boxes::SHAPE_WORK))
            .saturating_add(extra_images.saturating_mul(background_boxes::LAYER_WORK));
        if !background_boxes::spend(&mut self.position_work_left, cost) {
            return;
        }
        let mut metrics = Vec::new();
        if metrics.try_reserve_exact(self.doc.nodes.len()).is_err() {
            return;
        }
        metrics.resize(self.doc.nodes.len(), Metrics::default());
        for &geometry in &self.boxes {
            if geometry.inline
                && let Some(metric) = metrics.get_mut(geometry.node)
            {
                metric.add(
                    geometry,
                    self.borders(geometry.node),
                    self.padding(geometry.node, geometry.inline_basis),
                );
            }
        }
        let mut group_boxes = None;
        for index in 0..self.commands.len() {
            let owner = self.owners[index];
            if owner.inline_layer == InlineLayer::None {
                continue;
            }
            self.owners[index].inline_layer = InlineLayer::None;
            let area = match self.commands[index] {
                DrawCommand::Rect { rect, .. } | DrawCommand::PushClip { rect } => rect,
                _ => continue,
            };
            if owner.inline_layer == InlineLayer::Clip {
                group_boxes = None;
                self.commands[index] = DrawCommand::PushClip {
                    rect: fragment_clip(area, owner.inline_sides).unwrap_or_default(),
                };
                continue;
            }
            let style = self.style(owner.node);
            let boxes = if let Some(boxes) = group_boxes {
                boxes
            } else {
                let Some(boxes) = metrics.get(owner.node).copied().and_then(|metric| {
                    metric.boxes(area, owner.inline_prefix, style.border_radius)
                }) else {
                    continue;
                };
                group_boxes = Some(boxes);
                boxes
            };
            let command = match owner.inline_layer {
                InlineLayer::Color => background_boxes::color(
                    boxes.select(style.backgrounds.bottom_clip()),
                    style.background_color,
                    None,
                ),
                InlineLayer::Image => {
                    let image_index = usize::from(owner.background_image_index);
                    let Some(layer) = style.backgrounds.layer(image_index) else {
                        continue;
                    };
                    let origin = boxes.select(layer.origin).rect;
                    let shape = boxes.select(layer.clip);
                    let current_color = style.color;
                    self.background_command(
                        owner.node,
                        image_index,
                        shape.rect,
                        origin,
                        0.0,
                        current_color,
                        false,
                    )
                    .map(|command| background_boxes::image(command, shape))
                    .unwrap_or_else(background_boxes::empty)
                }
                InlineLayer::Border => {
                    if boxes.border == boxes.padding {
                        background_boxes::empty()
                    } else {
                        background_boxes::color(
                            boxes.border,
                            style.border_color,
                            Some(boxes.padding),
                        )
                    }
                }
                InlineLayer::None | InlineLayer::Clip => unreachable!(),
            };
            self.commands[index] = command;
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod translation_tests;
