//! Exact CPU lookup geometry for the square-corner raster background mode.
//! Sampling is separate from Image's established integer-edge mapping.
use crate::{Frame, Profile, Rect, Result, scope::CoordinateState, word};

/// Allocation-free reservation shared with the Native adapter. `loop_work`
/// retains Canvas's candidate loop before its final integer clip rejection;
/// `bounds` and `lut_work` describe the smaller GPU-visible lookup window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundImageGeometry {
    pub bounds: Option<(u32, u32, u32, u32)>,
    pub loop_work: u64,
    pub lut_work: u64,
    pub(super) translated_tile: Rect,
}

impl BackgroundImageGeometry {
    pub fn new(
        frame: Frame,
        rect: Rect,
        tile: Rect,
        repeat: [bool; 2],
        coordinates: &CoordinateState,
    ) -> Result<Self> {
        // Public callers can supply a Frame unrelated to their coordinate
        // state. Check the closed largest viewport before any integer casts.
        Profile::Native.validate_viewport(frame.width, frame.height)?;
        rect.validate()?;
        tile.validate()?;
        if tile.width <= 0.0 || tile.height <= 0.0 {
            return Err("nonpositive background tile".into());
        }
        let offset = coordinates.offset();
        let clip = coordinates.clip();
        // Match Canvas command dispatch's f32 additions, then preserve this
        // sampling rectangle independently of later destination clipping.
        let rect = rect.translated(offset);
        let tile = tile.translated(offset);
        let mut info = Self {
            bounds: None,
            loop_work: 0,
            lut_work: 0,
            translated_tile: tile,
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return Ok(info);
        }
        let visible = rect.intersect(clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return Ok(info);
        }
        let mut x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as i32;
        let mut y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as i32;
        let mut x1 = (visible.x + visible.width).ceil().min(frame.width as f32) as i32;
        let mut y1 = (visible.y + visible.height).ceil().min(frame.height as f32) as i32;
        if !repeat[0] {
            (x0, x1) = nonrepeated_axis(x0, x1, tile.x, tile.width);
        }
        if !repeat[1] {
            (y0, y1) = nonrepeated_axis(y0, y1, tile.y, tile.height);
        }
        if x1 <= x0 || y1 <= y0 {
            return Ok(info);
        }
        // The viewport check and clipping bound these differences by 1280 and
        // 1024. Retain the original painter tariff before final blend clipping.
        let pixels = (x1 - x0) as u64 * (y1 - y0) as u64;
        info.loop_work = 8 + pixels * (3 + u64::from(repeat[0]) + u64::from(repeat[1]));
        x1 = x1.min((clip.x + clip.width).ceil() as i32);
        y1 = y1.min((clip.y + clip.height).ceil() as i32);
        if x1 <= x0 || y1 <= y0 {
            return Ok(info);
        }
        let width = (x1 - x0) as u32;
        let height = (y1 - y0) as u32;
        info.bounds = Some((x0 as u32, y0 as u32, width, height));
        // Named preparation tariff, not a measured instruction count. It is
        // prepaid by Native; standalone planning bounds it by the existing
        // command/LUT caps before allocating or writing the packed input.
        info.lut_work = 8
            + (3 + u64::from(repeat[0])) * u64::from(width)
            + (3 + u64::from(repeat[1])) * u64::from(height);
        Ok(info)
    }
}

// Keep the same center-relative end refinement as Canvas's gradient helper.
// A geometric tile intersection loses a subnormal extent at an exact center.
fn nonrepeated_axis(low: i32, high: i32, origin: f32, extent: f32) -> (i32, i32) {
    let origin = f64::from(origin);
    let extent = f64::from(extent);
    let low = low.max((origin - 0.5).ceil() as i32);
    let mut end = high.min((origin + extent - 0.5).ceil() as i32);
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

pub(super) fn append_table(
    bytes: &mut Vec<u8>,
    start: u32,
    count: u32,
    origin: f32,
    extent: f32,
    source_extent: u32,
    repeat: bool,
) -> Result<u32> {
    let base = u32::try_from(bytes.len() / 4).map_err(|_| "LUT offset overflow")?;
    let end = start.checked_add(count).ok_or("LUT extent overflow")?;
    if source_extent == 0 || !origin.is_finite() || !extent.is_finite() || extent <= 0.0 {
        return Err("invalid background sampling axis".into());
    }
    let origin = f64::from(origin);
    let extent = f64::from(extent);
    for destination in start..end {
        let local = (f64::from(destination) + 0.5) - origin;
        let phase = if repeat {
            local.rem_euclid(extent)
        } else {
            local
        };
        // Division then multiplication matches Canvas. A Euclidean adjustment
        // rounding phase to extent clamps to the last texel, never wraps to0.
        let index = ((phase / extent * f64::from(source_extent)) as u32).min(source_extent - 1);
        if index >= source_extent {
            return Err("source index outside validated image".into());
        }
        word(bytes, index);
    }
    Ok(base)
}
