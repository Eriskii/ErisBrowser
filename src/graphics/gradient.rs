//! Bounded linear gradients using premultiplied, gamma-encoded sRGB.
use super::{Color, Rect};
use std::sync::Arc;

pub const MAX_GRADIENT_STOPS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientDirection {
    /// CSS degrees: zero points up and positive angles turn clockwise.
    Angle(f64),
    /// Corner directions depend on the positioning box's aspect ratio.
    Corner { right: bool, bottom: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientColor {
    Color(Color),
    CurrentColor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradientPosition {
    pub px: f64,
    /// Percentage units, so 100 means the gradient line's full length.
    pub percent: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub color: GradientColor,
    pub position: Option<GradientPosition>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    pub direction: GradientDirection,
    /// Double-position authored stops are expanded before this representation.
    pub stops: Arc<[GradientStop]>,
}

impl LinearGradient {
    pub fn valid(&self) -> bool {
        !self.stops.is_empty()
            && self.stops.len() <= MAX_GRADIENT_STOPS
            && match self.direction {
                GradientDirection::Angle(angle) => angle.is_finite(),
                GradientDirection::Corner { .. } => true,
            }
            && self.stops.iter().all(|stop| {
                stop.position
                    .is_none_or(|position| position.px.is_finite() && position.percent.is_finite())
            })
    }
}

#[derive(Clone, Copy, Default)]
struct ResolvedStop {
    position: f64,
    /// Premultiplied channels followed by alpha, all in [0, 1].
    color: [f64; 4],
}

struct ResolvedGradient {
    dx: f64,
    dy: f64,
    length: f64,
    tile: Rect,
    repeat: [bool; 2],
    stops: [ResolvedStop; MAX_GRADIENT_STOPS],
    count: usize,
}

impl ResolvedGradient {
    fn new(source: &LinearGradient, tile: Rect, color: Color, scale: f32) -> Option<Self> {
        if !source.valid()
            || ![tile.x, tile.y, tile.width, tile.height, scale]
                .iter()
                .all(|value| value.is_finite())
            || tile.width <= 0.0
            || tile.height <= 0.0
            || scale <= 0.0
        {
            return None;
        }
        let width = f64::from(tile.width);
        let height = f64::from(tile.height);
        let (dx, dy) = match source.direction {
            GradientDirection::Angle(angle) => {
                // Cardinal angles must not acquire a tiny cross-axis phase
                // from trigonometric rounding at an exact hard transition.
                match angle.rem_euclid(360.0) {
                    0.0 => (0.0, -1.0),
                    90.0 => (1.0, 0.0),
                    180.0 => (0.0, 1.0),
                    270.0 => (-1.0, 0.0),
                    angle => {
                        let radians = angle.to_radians();
                        (radians.sin(), -radians.cos())
                    }
                }
            }
            GradientDirection::Corner { right, bottom } => {
                let diagonal = width.hypot(height);
                (
                    height / diagonal * if right { 1.0 } else { -1.0 },
                    width / diagonal * if bottom { 1.0 } else { -1.0 },
                )
            }
        };
        let length = dx.abs() * width + dy.abs() * height;
        if !length.is_finite() || length <= 0.0 {
            return None;
        }
        let mut result = Self {
            dx,
            dy,
            length,
            tile,
            repeat: [true, true],
            stops: [ResolvedStop::default(); MAX_GRADIENT_STOPS],
            count: source.stops.len(),
        };
        let mut positioned = [false; MAX_GRADIENT_STOPS];
        for (index, stop) in source.stops.iter().enumerate() {
            let color = match stop.color {
                GradientColor::Color(color) => color,
                GradientColor::CurrentColor => color,
            };
            let alpha = f64::from(color.a) / 255.0;
            result.stops[index].color = [
                f64::from(color.r) / 255.0 * alpha,
                f64::from(color.g) / 255.0 * alpha,
                f64::from(color.b) / 255.0 * alpha,
                alpha,
            ];
            if let Some(position) = stop.position {
                let resolved = position.px * f64::from(scale) + position.percent / 100.0 * length;
                if !resolved.is_finite() {
                    return None;
                }
                result.stops[index].position = resolved;
                positioned[index] = true;
            }
        }
        // A single authored color is solid; its position cannot create a gap.
        if result.count == 1 {
            return Some(result);
        }
        let last = result.count - 1;
        if !positioned[0] {
            result.stops[0].position = 0.0;
            positioned[0] = true;
        }
        if !positioned[last] {
            result.stops[last].position = length;
            positioned[last] = true;
        }
        // Explicit positions never move backwards, including the implicit
        // endpoint positions. Then distribute each run of omitted positions.
        let mut previous = 0;
        for (index, present) in positioned.iter().enumerate().take(result.count).skip(1) {
            if !present {
                continue;
            }
            let from = result.stops[previous].position;
            let to = result.stops[index].position.max(from);
            if !(to - from).is_finite() {
                return None;
            }
            result.stops[index].position = to;
            for missing in previous + 1..index {
                let t = (missing - previous) as f64 / (index - previous) as f64;
                result.stops[missing].position = from + (to - from) * t;
            }
            previous = index;
        }
        Some(result)
    }

    fn sample(&self, x: f64, y: f64) -> [f64; 4] {
        if self.count == 1 {
            return self.stops[0].color;
        }
        let width = f64::from(self.tile.width);
        let height = f64::from(self.tile.height);
        let local_x = x - f64::from(self.tile.x);
        let local_y = y - f64::from(self.tile.y);
        let x = if self.repeat[0] {
            local_x.rem_euclid(width)
        } else {
            local_x
        } - width * 0.5;
        let y = if self.repeat[1] {
            local_y.rem_euclid(height)
        } else {
            local_y
        } - height * 0.5;
        let position = x * self.dx + y * self.dy + self.length * 0.5;
        let stops = &self.stops[..self.count];
        // The last stop at an equal position wins. This also avoids division
        // by zero at coincident stops and makes sharp edges deterministic.
        let upper = stops.partition_point(|stop| stop.position <= position);
        if upper == 0 {
            return stops[0].color;
        }
        if upper == self.count {
            return stops[self.count - 1].color;
        }
        let left = stops[upper - 1];
        let right = stops[upper];
        let t = ((position - left.position) / (right.position - left.position)).clamp(0.0, 1.0);
        std::array::from_fn(|channel| {
            left.color[channel] + (right.color[channel] - left.color[channel]) * t
        })
    }
}

#[derive(Clone, Copy)]
struct GradientLoop {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    work: u64,
}

fn gradient_loop(
    width: u32,
    height: u32,
    rect: Rect,
    clip: Rect,
    stops: usize,
) -> Option<GradientLoop> {
    let visible = rect.intersect(clip);
    if visible.width <= 0.0 || visible.height <= 0.0 {
        return None;
    }
    let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as i32;
    let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as i32;
    let x1 = (visible.x + visible.width).ceil().min(width as f32) as i32;
    let y1 = (visible.y + visible.height).ceil().min(height as f32) as i32;
    let pixels = (x1 - x0).max(0) as u64 * (y1 - y0).max(0) as u64;
    if pixels == 0 {
        return None;
    }
    let search = u64::from(usize::BITS - (stops - 1).leading_zeros()) + 1;
    Some(GradientLoop {
        x0,
        y0,
        x1,
        y1,
        work: pixels * (search + 1) + stops as u64 * 4,
    })
}

// Repetition changes the sampling window, never the rounded paint outline.
// A nonrepeated image contributes exactly where a pixel center lies in its
// half-open tile interval. Use f64 here like ResolvedGradient's local samples;
// a fractional tile edge is not the integer-containment Canvas clip rule.
fn gradient_loop_with_repeat(
    width: u32,
    height: u32,
    rect: Rect,
    tile: Rect,
    clip: Rect,
    stops: usize,
    repeat: [bool; 2],
) -> Option<GradientLoop> {
    let mut bounds = gradient_loop(width, height, rect, clip, stops)?;
    if repeat == [true, true] {
        return Some(bounds);
    }
    if ![tile.x, tile.y, tile.width, tile.height]
        .into_iter()
        .all(f32::is_finite)
        || tile.width <= 0.0
        || tile.height <= 0.0
    {
        return None;
    }
    if !repeat[0] {
        (bounds.x0, bounds.x1) = nonrepeated_axis(bounds.x0, bounds.x1, tile.x, tile.width);
    }
    if !repeat[1] {
        (bounds.y0, bounds.y1) = nonrepeated_axis(bounds.y0, bounds.y1, tile.y, tile.height);
    }
    if bounds.x1 <= bounds.x0 || bounds.y1 <= bounds.y0 {
        return None;
    }
    let pixels = (bounds.x1 - bounds.x0) as u64 * (bounds.y1 - bounds.y0) as u64;
    let search = u64::from(usize::BITS - (stops - 1).leading_zeros()) + 1;
    let disabled_axes = u64::from(!repeat[0]) + u64::from(!repeat[1]);
    bounds.work = 16 + stops as u64 * 4 + pixels * (search + 1 + disabled_axes);
    Some(bounds)
}

pub(super) fn nonrepeated_axis(low: i32, high: i32, origin: f32, extent: f32) -> (i32, i32) {
    let origin = f64::from(origin);
    let extent = f64::from(extent);
    let low = low.max((origin - 0.5).ceil() as i32);
    let mut end = high.min((origin + extent - 0.5).ceil() as i32);
    // The sum can round away a subnormal extent at an exact pixel center.
    // Refine the adjacent candidate against the same local coordinates used
    // by the sampler, instead of manufacturing an epsilon or expanding tiles.
    let inside = |pixel: i32| {
        let local = (f64::from(pixel) + 0.5) - origin;
        local >= 0.0 && local < extent
    };
    if end >= low && end < high && inside(end) {
        end += 1;
    }
    if end > low && !inside(end - 1) {
        end -= 1;
    }
    (low, end)
}

#[cfg(feature = "raster-bridge")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OpaqueGradientError {
    Unsupported,
    Geometry,
    Allocation,
}

#[cfg(feature = "raster-bridge")]
#[derive(Clone, Copy, Debug)]
pub(crate) struct OpaqueGradientInfo {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// The software painter's complete candidate-loop tariff, including
    /// samples later rejected by half-open clip containment.
    pub loop_work: u64,
}

#[cfg(feature = "raster-bridge")]
impl OpaqueGradientInfo {
    pub fn rgba_bytes(self) -> usize {
        self.width as usize * self.height as usize * 4
    }
}

/// Bounded metadata only. The caller must reserve work, source slots, source
/// bytes and peak storage before calling materialize. The retained borrow
/// prevents sampling a different gradient from the one admitted.
#[cfg(feature = "raster-bridge")]
pub(crate) struct OpaqueGradient<'a> {
    info: OpaqueGradientInfo,
    gradient: &'a LinearGradient,
    tile: Rect,
    repeat: [bool; 2],
    current_color: Color,
    stop_scale: f32,
}

#[cfg(feature = "raster-bridge")]
impl<'a> OpaqueGradient<'a> {
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: u32,
        height: u32,
        rect: Rect,
        tile: Rect,
        clip: Rect,
        gradient: &'a LinearGradient,
        current_color: Color,
        stop_scale: f32,
    ) -> Result<Option<Self>, OpaqueGradientError> {
        Self::new_with_repeat(
            width,
            height,
            rect,
            tile,
            clip,
            gradient,
            current_color,
            stop_scale,
            [true, true],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_repeat(
        width: u32,
        height: u32,
        rect: Rect,
        tile: Rect,
        clip: Rect,
        gradient: &'a LinearGradient,
        current_color: Color,
        stop_scale: f32,
        repeat: [bool; 2],
    ) -> Result<Option<Self>, OpaqueGradientError> {
        // This helper ceiling makes integer-to-f32 placement exact and keeps
        // the RGBA byte product representable on 32-bit hosts. The Native
        // caller applies its stricter viewport and source storage limits.
        if width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || !gradient.valid()
            || ![
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                tile.x,
                tile.y,
                tile.width,
                tile.height,
                clip.x,
                clip.y,
                clip.width,
                clip.height,
                stop_scale,
            ]
            .into_iter()
            .all(f32::is_finite)
            || ![
                rect.x + rect.width,
                rect.y + rect.height,
                tile.x + tile.width,
                tile.y + tile.height,
                clip.x + clip.width,
                clip.y + clip.height,
            ]
            .into_iter()
            .all(f32::is_finite)
            || tile.width <= 0.0
            || tile.height <= 0.0
            || stop_scale <= 0.0
        {
            return Err(OpaqueGradientError::Geometry);
        }
        if gradient.stops.iter().any(|stop| match stop.color {
            GradientColor::Color(color) => color.a != 255,
            GradientColor::CurrentColor => current_color.a != 255,
        }) {
            return Err(OpaqueGradientError::Unsupported);
        }
        if rect.width <= 0.0 || rect.height <= 0.0 || clip.width <= 0.0 || clip.height <= 0.0 {
            return Ok(None);
        }
        let Some(bounds) = gradient_loop_with_repeat(
            width,
            height,
            rect,
            tile,
            clip,
            gradient.stops.len(),
            repeat,
        ) else {
            return Ok(None);
        };
        // Retain the full loop tariff above, while storing only pixels that
        // blend_gradient would actually accept. No transparent holes arise.
        let x1 = bounds.x1.min((clip.x + clip.width).ceil() as i32);
        let y1 = bounds.y1.min((clip.y + clip.height).ceil() as i32);
        if x1 <= bounds.x0 || y1 <= bounds.y0 {
            return Ok(None);
        }
        Ok(Some(Self {
            info: OpaqueGradientInfo {
                x: bounds.x0 as u32,
                y: bounds.y0 as u32,
                width: (x1 - bounds.x0) as u32,
                height: (y1 - bounds.y0) as u32,
                loop_work: bounds.work,
            },
            gradient,
            tile,
            repeat,
            current_color,
            stop_scale,
        }))
    }

    pub fn info(&self) -> OpaqueGradientInfo {
        self.info
    }

    pub fn materialize(self) -> Result<Vec<u8>, OpaqueGradientError> {
        let mut resolved = ResolvedGradient::new(
            self.gradient,
            self.tile,
            self.current_color,
            self.stop_scale,
        )
        .ok_or(OpaqueGradientError::Geometry)?;
        resolved.repeat = self.repeat;
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(self.info.rgba_bytes())
            .map_err(|_| OpaqueGradientError::Allocation)?;
        for y in self.info.y..self.info.y + self.info.height {
            for x in self.info.x..self.info.x + self.info.width {
                let color = resolved.sample(f64::from(x) + 0.5, f64::from(y) + 0.5);
                rgba.extend_from_slice(&[
                    (color[0] * 255.0).round() as u8,
                    (color[1] * 255.0).round() as u8,
                    (color[2] * 255.0).round() as u8,
                    255,
                ]);
            }
        }
        Ok(rgba)
    }
}

impl super::Canvas {
    #[allow(clippy::too_many_arguments)]
    pub fn linear_gradient(
        &mut self,
        rect: Rect,
        tile: Rect,
        radius: f32,
        gradient: &LinearGradient,
        current_color: Color,
        stop_scale: f32,
    ) {
        self.linear_gradient_with_repeat(
            rect,
            tile,
            radius,
            gradient,
            current_color,
            stop_scale,
            [true, true],
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn linear_gradient_with_repeat(
        &mut self,
        rect: Rect,
        tile: Rect,
        radius: f32,
        gradient: &LinearGradient,
        current_color: Color,
        stop_scale: f32,
        repeat: [bool; 2],
    ) {
        if self.suppressed()
            || ![rect.x, rect.y, rect.width, rect.height, radius]
                .iter()
                .all(|value| value.is_finite())
            || rect.width <= 0.0
            || rect.height <= 0.0
            || radius < 0.0
            || !gradient.valid()
        {
            return;
        }
        let Some(bounds) = gradient_loop_with_repeat(
            self.width,
            self.height,
            rect,
            tile,
            self.clip,
            gradient.stops.len(),
            repeat,
        ) else {
            return;
        };
        // Charge resolution plus a conservative binary-search/evaluation
        // tariff before any per-pixel work. No temporary image is allocated.
        if !self.consume_pixels(bounds.work) {
            return;
        }
        let Some(mut resolved) = ResolvedGradient::new(gradient, tile, current_color, stop_scale)
        else {
            return;
        };
        resolved.repeat = repeat;
        let radius = radius.min(rect.width / 2.0).min(rect.height / 2.0);
        for y in bounds.y0..bounds.y1 {
            for x in bounds.x0..bounds.x1 {
                if self.paint_exhausted {
                    return;
                }
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let coverage = if radius == 0.0 {
                    255
                } else {
                    let dx = (rect.x + radius - px)
                        .max(px - (rect.x + rect.width - radius))
                        .max(0.0);
                    let dy = (rect.y + radius - py)
                        .max(py - (rect.y + rect.height - radius))
                        .max(0.0);
                    ((radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0) * 255.0) as u8
                };
                self.blend_gradient(
                    x,
                    y,
                    resolved.sample(f64::from(x) + 0.5, f64::from(y) + 0.5),
                    coverage,
                );
            }
        }
    }

    fn blend_gradient(&mut self, x: i32, y: i32, color: [f64; 4], coverage: u8) {
        let coverage = self.box_coverage(x, y, coverage);
        if coverage == 0 || color[3] == 0.0 || !self.clip.contains(x as f32, y as f32) {
            return;
        }
        let coverage = f64::from(coverage) / 255.0;
        let alpha = color[3] * coverage;
        let inverse = 1.0 - alpha;
        if !self.layers.is_empty() {
            if !self.materialize_layers() {
                return;
            }
            let layer = self.layers.last_mut().expect("gradient opacity layer");
            let super::LayerPixels::Allocated(pixels) = &mut layer.pixels else {
                self.paint_exhausted = true;
                return;
            };
            let x = x as usize;
            let y = y as usize;
            if x < layer.x
                || y < layer.y
                || x >= layer.x + layer.width
                || y >= layer.y + layer.height
            {
                return;
            }
            let destination = &mut pixels[(y - layer.y) * layer.width + x - layer.x];
            let channel = |index: usize, shift| {
                (color[index] * coverage * 65_535.0
                    + ((*destination >> shift) & 65_535u64) as f64 * inverse)
                    .round() as u64
            };
            *destination = (channel(3, 48) << 48)
                | (channel(0, 32) << 32)
                | (channel(1, 16) << 16)
                | channel(2, 0);
        } else {
            let destination = &mut self.pixels[y as usize * self.width as usize + x as usize];
            let channel = |index: usize, shift| {
                (color[index] * coverage * 255.0
                    + ((*destination >> shift) & 255u32) as f64 * inverse)
                    .round() as u32
            };
            *destination = (channel(0, 16) << 16) | (channel(1, 8) << 8) | channel(2, 0);
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "raster-bridge"))]
mod opaque_tests;

#[cfg(test)]
mod tiling_tests;
