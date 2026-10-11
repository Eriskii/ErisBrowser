//! Fixed-size background box selection, independent of image positioning.
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use std::sync::Arc;

pub(super) const SHAPE_WORK: usize = 16;
#[derive(Clone, Copy)]
pub(super) struct BackgroundBoxes {
    pub(super) border: RoundedBox,
    pub(super) padding: RoundedBox,
    pub(super) content: RoundedBox,
}
impl BackgroundBoxes {
    pub(super) fn new(outer: RoundedBox, border: Sides, padding: Sides) -> Option<Self> {
        let b = [border.top, border.right, border.bottom, border.left];
        let p = [padding.top, padding.right, padding.bottom, padding.left];
        if !p.into_iter().all(|v| v.is_finite() && v >= 0.0) {
            return None;
        }
        let mut total = [0.0; 4];
        for i in 0..4 {
            let value = f64::from(b[i]) + f64::from(p[i]);
            if !value.is_finite() || value > 16_000_000.0 {
                return None;
            }
            total[i] = value as f32;
        }
        // Every result starts from the original outer radii. In particular,
        // never normalize the truncated padding arcs before deriving content.
        Some(Self {
            border: outer.normalized()?,
            padding: outer.inset(b)?,
            content: outer.inset(total)?,
        })
    }
    pub(super) fn select(self, which: BackgroundBox) -> RoundedBox {
        match which {
            BackgroundBox::Border => self.border,
            BackgroundBox::Padding => self.padding,
            BackgroundBox::Content => self.content,
        }
    }
    pub(super) fn from_style(
        outer: RoundedBox,
        style: &ComputedStyle,
        reference: f32,
    ) -> Option<Self> {
        // Match the forward box-layout resolutions. Recovering zero insets by
        // subtracting translated endpoints can manufacture a negative side.
        let border = &style.border_width;
        let padding = &style.padding;
        Self::new(
            outer,
            Sides {
                top: extent(border.top),
                right: extent(border.right),
                bottom: extent(border.bottom),
                left: extent(border.left),
            },
            Sides {
                top: resolve(padding.top, reference).unwrap_or(0.0).max(0.0),
                right: resolve(padding.right, reference).unwrap_or(0.0).max(0.0),
                bottom: resolve(padding.bottom, reference).unwrap_or(0.0).max(0.0),
                left: resolve(padding.left, reference).unwrap_or(0.0).max(0.0),
            },
        )
    }
}
#[derive(Clone, Copy)]
pub(super) enum PaintRole {
    Color,
    Image(u8),
    Top,
    Right,
    Bottom,
    Left,
    RoundOuter,
    RoundInner,
    Ring,
    Unused,
}
// Two bytes replace Fragment's former image/rounded booleans. The overflow
// bit records the cell's own opener even when its decoration was refused.
#[derive(Clone, Copy, Default)]
pub(super) struct PaintSlots {
    image_count: u8,
    flags: u8,
}
impl PaintSlots {
    const ADMITTED: u8 = 4;
    const OVERFLOW: u8 = 8;
    pub(super) fn new(image: bool, rounded: bool) -> Self {
        Self {
            image_count: u8::from(image),
            flags: Self::ADMITTED | u8::from(rounded),
        }
    }
    pub(super) fn layers(count: usize, rounded: bool) -> Self {
        debug_assert!(count <= 8);
        Self {
            image_count: count as u8,
            flags: Self::ADMITTED | 2 | u8::from(rounded),
        }
    }
    pub(super) fn images(self) -> usize {
        usize::from(self.image_count)
    }
    pub(super) fn layered(self) -> bool {
        self.flags & 2 != 0
    }
    pub(super) fn rounded(self) -> bool {
        self.flags & 1 != 0
    }
    pub(super) fn omitted(mut self) -> Self {
        self.flags &= !Self::ADMITTED;
        self
    }
    pub(super) fn with_overflow(mut self, value: bool) -> Self {
        if value {
            self.flags |= Self::OVERFLOW;
        } else {
            self.flags &= !Self::OVERFLOW;
        }
        self
    }
    pub(super) fn overflow(self) -> bool {
        self.flags & Self::OVERFLOW != 0
    }
    pub(super) fn len(self) -> usize {
        if self.flags & Self::ADMITTED == 0 {
            0
        } else {
            5 + self.images()
        }
    }
    pub(super) fn role(self, index: usize) -> PaintRole {
        if index >= self.len() {
            return PaintRole::Unused;
        }
        if self.rounded() && !self.layered() {
            return match index {
                0 => PaintRole::RoundOuter,
                1 => PaintRole::RoundInner,
                2 if self.image_count != 0 => PaintRole::Image(0),
                2 => PaintRole::Right,
                3 if self.image_count == 0 => PaintRole::Bottom,
                4 if self.image_count == 0 => PaintRole::Left,
                _ => PaintRole::Unused,
            };
        }
        if index == 0 {
            return PaintRole::Color;
        }
        if index <= self.images() {
            return PaintRole::Image((self.images() - index) as u8);
        }
        if self.layered() && self.rounded() {
            return if index == self.images() + 1 {
                PaintRole::Ring
            } else {
                PaintRole::Unused
            };
        }
        match index - self.images() {
            1 => PaintRole::Top,
            2 => PaintRole::Right,
            3 => PaintRole::Bottom,
            4 => PaintRole::Left,
            _ => PaintRole::Unused,
        }
    }
}
pub(super) const LAYER_WORK: usize = 16;
pub(super) fn layers_work(count: usize) -> Option<usize> {
    count.checked_mul(LAYER_WORK)?.checked_add(SHAPE_WORK)
}
pub(super) fn empty() -> DrawCommand {
    DrawCommand::Rect {
        rect: Rect::default(),
        color: Color::TRANSPARENT,
        radius: 0.0,
    }
}
pub(super) fn color(shape: RoundedBox, color: Color, cutout: Option<RoundedBox>) -> DrawCommand {
    if color.a == 0 {
        return empty();
    }
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape,
        cutout,
        paint: BoxPaint::Color(color),
    }))
}
pub(super) fn image(command: DrawCommand, shape: RoundedBox) -> DrawCommand {
    let paint = match command {
        DrawCommand::LinearGradient {
            tile,
            repeat,
            gradient,
            current_color,
            stop_scale,
            ..
        } => BoxPaint::LinearGradient {
            tile,
            repeat,
            gradient,
            current_color,
            stop_scale,
        },
        DrawCommand::BackgroundImage {
            tile, repeat, key, ..
        } => BoxPaint::Image { tile, repeat, key },
        _ => return empty(),
    };
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape,
        cutout: None,
        paint,
    }))
}
pub(super) fn shape_path(style: &ComputedStyle, border: Sides, legacy_rounded: bool) -> bool {
    style.backgrounds.bottom_clip() != BackgroundBox::Border
        || style.border_radius > 0.0
            && border.horizontal() + border.vertical() > 0.0
            && !legacy_rounded
}
pub(super) fn nondefault(style: &ComputedStyle) -> bool {
    style.backgrounds.image_slots().len() > 1
        || style.backgrounds.layer(0).unwrap().origin != BackgroundBox::Padding
        || style.backgrounds.bottom_clip() != BackgroundBox::Border
}
pub(super) fn spend(work: &mut usize, cost: usize) -> bool {
    if cost > *work {
        false
    } else {
        *work -= cost;
        true
    }
}

#[cfg(test)]
mod tests;

// Keep command arrays and shape preparation outside recursive box layout.
// This also bounds debug-build stack frames without raising the depth limit.
impl Engine<'_> {
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_principal_background(
        &mut self,
        id: NodeId,
        style: &ComputedStyle,
        area: Rect,
        border: Sides,
        padding: Sides,
        paint_start: usize,
        slots: PaintSlots,
    ) {
        if slots.len() == 0 {
            return;
        }
        if slots.layered() {
            self.paint_layered_principal(id, style, area, border, padding, paint_start, slots);
            return;
        }
        let has_image = slots.images() != 0;
        let Rect {
            x,
            y,
            width,
            height,
        } = area;
        // Canvas background propagation is emitted once before the root box.
        let paint_height = height;
        let background = if self.canvas_background_node == Some(id) {
            Color::TRANSPARENT
        } else {
            style.background_color
        };
        let border_color = style.border_color;
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
        let legacy_rounded = self.rounded_border(id);
        let shaped = background_boxes::shape_path(style, border, legacy_rounded);
        if shaped || background_boxes::nondefault(style) {
            let mut commands = std::array::from_fn::<_, 6, _>(|_| background_boxes::empty());
            let boxes =
                background_boxes::spend(&mut self.position_work_left, background_boxes::SHAPE_WORK)
                    .then(|| {
                        BackgroundBoxes::new(
                            crate::graphics::box_decoration::RoundedBox {
                                rect: rect(x, y, width, height),
                                radii: [[style.border_radius; 2]; 4],
                            },
                            border,
                            padding,
                        )
                    })
                    .flatten();
            if let Some(boxes) = boxes {
                let clip = boxes.select(style.backgrounds.bottom_clip());
                let origin = boxes
                    .select(style.backgrounds.layer(0).unwrap().origin)
                    .rect;
                if shaped {
                    commands[0] = background_boxes::color(clip, background, None);
                    if has_image
                        && let Some(command) = self.background_command(
                            id,
                            0,
                            clip.rect,
                            origin,
                            0.0,
                            style.color,
                            true,
                        )
                    {
                        commands[1] = background_boxes::image(command, clip);
                    }
                    let border_index = 1 + usize::from(has_image);
                    if style.border_radius > 0.0 {
                        commands[border_index] = background_boxes::color(
                            boxes.border,
                            border_color,
                            Some(boxes.padding),
                        );
                    } else {
                        for (slot, command) in commands
                            .iter_mut()
                            .skip(border_index)
                            .zip(replacements.into_iter().skip(1))
                        {
                            *slot = command;
                        }
                    }
                } else {
                    // Origin changes alone retain the safe legacy paint/AA.
                    if has_image {
                        let area = boxes.padding.rect;
                        let command = self
                            .background_command(
                                id,
                                0,
                                if legacy_rounded {
                                    area
                                } else {
                                    boxes.border.rect
                                },
                                origin,
                                if legacy_rounded {
                                    (style.border_radius - border.top).max(0.0)
                                } else {
                                    style.border_radius
                                },
                                style.color,
                                true,
                            )
                            .unwrap_or_else(background_boxes::empty);
                        let [background, top, right, bottom, left] = replacements;
                        commands = if legacy_rounded {
                            [background, top, command, right, bottom, left]
                        } else {
                            [background, command, top, right, bottom, left]
                        };
                    } else {
                        for (slot, command) in commands.iter_mut().zip(replacements) {
                            *slot = command;
                        }
                    }
                }
            }
            for (offset, command) in commands
                .into_iter()
                .take(5 + usize::from(has_image))
                .enumerate()
            {
                if let Some(slot) = self.commands.get_mut(paint_start + offset) {
                    *slot = command;
                }
            }
        } else if has_image {
            let area = rect(
                x + border.left,
                y + border.top,
                width - border.horizontal(),
                height - border.vertical(),
            );
            let rounded = self.rounded_border(id);
            let command = self
                .background_command(
                    id,
                    0,
                    if rounded {
                        area
                    } else {
                        rect(x, y, width, height)
                    },
                    area,
                    if rounded {
                        (style.border_radius - border.top).max(0.0)
                    } else {
                        style.border_radius
                    },
                    style.color,
                    // Table stretching may give a currently zero-sized tile a
                    // positive used area. Real keys remain bounded and paid once.
                    true,
                )
                .unwrap_or(DrawCommand::Rect {
                    rect: Rect::default(),
                    color: Color::TRANSPARENT,
                    radius: 0.0,
                });
            let [background, top, right, bottom, left] = replacements;
            let commands = if rounded {
                [background, top, command, right, bottom, left]
            } else {
                [background, command, top, right, bottom, left]
            };
            for (offset, command) in commands.into_iter().enumerate() {
                if let Some(slot) = self.commands.get_mut(paint_start + offset) {
                    *slot = command;
                }
            }
        } else {
            for (offset, command) in replacements.into_iter().enumerate() {
                if let Some(slot) = self.commands.get_mut(paint_start + offset) {
                    *slot = command;
                }
            }
        }
    }
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn paint_layered_principal(
        &mut self,
        id: NodeId,
        style: &ComputedStyle,
        area: Rect,
        border: Sides,
        padding: Sides,
        start: usize,
        slots: PaintSlots,
    ) {
        let Some(cost) = layers_work(slots.images()) else {
            return;
        };
        if !spend(&mut self.position_work_left, cost) {
            return;
        }
        let Some(boxes) = BackgroundBoxes::new(
            RoundedBox {
                rect: area,
                radii: [[style.border_radius; 2]; 4],
            },
            border,
            padding,
        ) else {
            return;
        };
        for offset in 0..slots.len() {
            let command = match slots.role(offset) {
                PaintRole::Color => color(
                    boxes.select(style.backgrounds.bottom_clip()),
                    if self.canvas_background_node == Some(id) {
                        Color::TRANSPARENT
                    } else {
                        style.background_color
                    },
                    None,
                ),
                PaintRole::Image(index) => {
                    let layer = style.backgrounds.layer(usize::from(index)).unwrap();
                    let shape = boxes.select(layer.clip);
                    let origin = boxes.select(layer.origin).rect;
                    self.background_command(
                        id,
                        usize::from(index),
                        shape.rect,
                        origin,
                        0.0,
                        style.color,
                        true,
                    )
                    .map(|command| image(command, shape))
                    .unwrap_or_else(empty)
                }
                PaintRole::Ring => color(boxes.border, style.border_color, Some(boxes.padding)),
                PaintRole::Top => DrawCommand::Rect {
                    rect: rect(area.x, area.y, area.width, border.top),
                    color: style.border_color,
                    radius: 0.0,
                },
                PaintRole::Right => DrawCommand::Rect {
                    rect: rect(
                        area.x + area.width - border.right,
                        area.y,
                        border.right,
                        area.height,
                    ),
                    color: style.border_color,
                    radius: 0.0,
                },
                PaintRole::Bottom => DrawCommand::Rect {
                    rect: rect(
                        area.x,
                        area.y + area.height - border.bottom,
                        area.width,
                        border.bottom,
                    ),
                    color: style.border_color,
                    radius: 0.0,
                },
                PaintRole::Left => DrawCommand::Rect {
                    rect: rect(area.x, area.y, border.left, area.height),
                    color: style.border_color,
                    radius: 0.0,
                },
                _ => empty(),
            };
            self.commands[start + offset] = command;
        }
    }
}
