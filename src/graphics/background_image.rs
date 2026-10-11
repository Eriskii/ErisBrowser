//! Destination-bounded, nearest-neighbor raster background sampling.
use super::{Canvas, Color, RasterImage, Rect, gradient::nonrepeated_axis};

fn valid_rect(rect: Rect) -> bool {
    [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        rect.x + rect.width,
        rect.y + rect.height,
    ]
    .into_iter()
    .all(f32::is_finite)
        && rect.width > 0.0
        && rect.height > 0.0
}

fn source_coordinate(pixel: i32, origin: f32, extent: f32, dimension: u32, repeat: bool) -> usize {
    let extent = f64::from(extent);
    let local = (f64::from(pixel) + 0.5) - f64::from(origin);
    let phase = if repeat {
        local.rem_euclid(extent)
    } else {
        local
    };
    // A negative remainder very near zero can round up to the full extent
    // during Euclidean adjustment. Its nearest texel is the last one, not a
    // manufactured wrap to the first. Exact period boundaries still yield 0.
    ((phase / extent * f64::from(dimension)) as usize).min(dimension as usize - 1)
}

impl Canvas {
    /// Paint an already decoded raster; this method never resolves or fetches.
    /// Nearest sampling uses destination centers independently of Image's
    /// established edge-coordinate scaling. Work depends on visible pixels,
    /// including for subnormal tile sizes; no tile-sized bitmap is allocated.
    pub fn background_image(
        &mut self,
        rect: Rect,
        tile: Rect,
        radius: f32,
        image: &RasterImage,
        repeat: [bool; 2],
    ) {
        if self.suppressed()
            || !valid_rect(rect)
            || !valid_rect(tile)
            || !radius.is_finite()
            || radius < 0.0
            || image.width == 0
            || image.height == 0
        {
            return;
        }
        let required = (image.width as usize)
            .checked_mul(image.height as usize)
            .and_then(|pixels| pixels.checked_mul(4));
        if required.is_none_or(|bytes| bytes > image.rgba.len()) {
            return;
        }
        let visible = rect.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return;
        }
        // Preserve the painter's integer-origin clipping, while tile sampling
        // and nonrepeat inclusion use centers. Tighten nonrepeat axes once.
        let mut x0 = visible.x.floor().max(self.clip.x.ceil()).max(0.0) as i32;
        let mut y0 = visible.y.floor().max(self.clip.y.ceil()).max(0.0) as i32;
        let mut x1 = (visible.x + visible.width).ceil().min(self.width as f32) as i32;
        let mut y1 = (visible.y + visible.height).ceil().min(self.height as f32) as i32;
        if !repeat[0] {
            (x0, x1) = nonrepeated_axis(x0, x1, tile.x, tile.width);
        }
        if !repeat[1] {
            (y0, y1) = nonrepeated_axis(y0, y1, tile.y, tile.height);
        }
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let pixels = (x1 - x0) as u64 * (y1 - y0) as u64;
        // Fixed setup plus two coordinate/sample units, a fetch/blend unit,
        // and one extra unit per repeated axis. This is a conservative work
        // tariff, not an instruction count. Prepay before touching pixels;
        // transparent and rounded-away candidates consume the same allowance.
        let work = 8 + pixels * (3 + u64::from(repeat[0]) + u64::from(repeat[1]));
        if !self.consume_pixels(work) {
            return;
        }
        let radius = radius.min(rect.width / 2.0).min(rect.height / 2.0);
        for y in y0..y1 {
            let sy = source_coordinate(y, tile.y, tile.height, image.height, repeat[1]);
            for x in x0..x1 {
                if self.paint_exhausted {
                    return;
                }
                let sx = source_coordinate(x, tile.x, tile.width, image.width, repeat[0]);
                let offset = (sy * image.width as usize + sx) * 4;
                // The checked complete raster above bounds every sample.
                let rgba = &image.rgba[offset..offset + 4];
                let coverage = if radius == 0.0 {
                    255
                } else {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let dx = (rect.x + radius - px)
                        .max(px - (rect.x + rect.width - radius))
                        .max(0.0);
                    let dy = (rect.y + radius - py)
                        .max(py - (rect.y + rect.height - radius))
                        .max(0.0);
                    ((radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0) * 255.0) as u8
                };
                self.blend(
                    x,
                    y,
                    Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]),
                    coverage,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests;
