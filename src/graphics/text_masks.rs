//! Bounded mask preparation for the three immutable bundled fonts.
//!
//! This API never paints a target and never uses `Fonts`' existing warm cache.
//! The pinned font library remains a trusted boundary: outline construction
//! allocates an internal curve Vec before bounds are available, and `draw`
//! allocates infallible f32 raster scratch. Cold calls are capped before outline
//! construction; bounds, scratch, coverage and callback work are admitted before
//! `draw`. These checks do not make the library's allocations fallible or bound
//! every internal curve operation. Author and system fonts are not accepted.

use super::{Color, Fonts, Rect};
use ab_glyph::{Font, ScaleFont, point};

pub const MAX_COLD_REQUESTS: usize = 256;
pub const MAX_MASK_AXIS: usize = 1_024;
pub const MAX_MASK_AREA: usize = 262_144;
pub const MAX_UNIQUE_COVERAGE_BYTES: usize = 262_144;
pub const MAX_RUN_TEXT_BYTES: usize = 32_768;
pub const MAX_FRAME_TEXT_BYTES: usize = 65_536;
pub const MAX_FRAME_SCALARS: usize = 4_096;
pub const MAX_CPU_PIXEL_ALLOWANCE: u64 = 32_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    PixelAllowance,
    RunTextBytes,
    FrameTextBytes,
    FrameScalars,
    ColdRequests,
    MaskAxis,
    MaskArea,
    UniqueCoverage,
    CpuPixels,
    Geometry,
    RowIndex,
    Allocation,
    RasterBounds,
    CallerRefused,
    SessionFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(match self {
            Self::PixelAllowance => "text CPU pixel allowance exceeds limit",
            Self::RunTextBytes => "text run byte limit",
            Self::FrameTextBytes => "text frame byte limit",
            Self::FrameScalars => "text frame scalar limit",
            Self::ColdRequests => "cold glyph request limit",
            Self::MaskAxis => "glyph mask axis limit",
            Self::MaskArea => "glyph mask area limit",
            Self::UniqueCoverage => "unique glyph coverage limit",
            Self::CpuPixels => "text CPU pixel work limit",
            Self::Geometry => "glyph placement geometry",
            Self::RowIndex => "glyph row outside mask",
            Self::Allocation => "glyph preparation allocation",
            Self::RasterBounds => "font raster callback outside admitted mask",
            Self::CallerRefused => "glyph caller refused preparation",
            Self::SessionFailed => "glyph frame session already failed",
        })
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug)]
pub struct TextRun<'a> {
    pub x: f32,
    pub y: f32,
    pub text: &'a str,
    pub size: f32,
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
    pub monospace: bool,
}

/// Opaque cache identity; all fields describe the same immutable bundled fonts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MaskKey {
    face: u8,
    character: char,
    eighths: u16,
}
impl MaskKey {
    pub fn face_index(self) -> usize {
        usize::from(self.face)
    }
    pub fn character(self) -> char {
        self.character
    }
    pub fn quantized_eighths(self) -> u16 {
        self.eighths
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskInfo {
    key: MaskKey,
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    area: usize,
}
impl MaskInfo {
    pub fn key(self) -> MaskKey {
        self.key
    }
    pub fn bearing_x(self) -> i32 {
        self.x
    }
    pub fn bearing_y(self) -> i32 {
        self.y
    }
    pub fn width(self) -> usize {
        self.width
    }
    pub fn height(self) -> usize {
        self.height
    }
    pub fn area(self) -> usize {
        self.area
    }
}

/// Absolute integer placement. Rows retain Canvas's original f32 shear order.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    info: MaskInfo,
    pen: f32,
    size: f32,
    y: i32,
    color: Color,
    italic: bool,
}
impl Placement {
    pub fn origin_y(self) -> i32 {
        self.y
    }
    pub fn color(self) -> Color {
        self.color
    }
    pub fn pen(self) -> f32 {
        self.pen
    }
    pub fn size(self) -> f32 {
        self.size
    }
    pub fn italic(self) -> bool {
        self.italic
    }
    pub fn row_x(self, row: usize) -> Result<i32, Error> {
        if row >= self.info.height {
            return Err(Error::RowIndex);
        }
        let shear = if self.italic {
            (self.size - (self.info.y as f32 + row as f32)) * 0.18
        } else {
            0.0
        };
        rounded_i32(self.pen + shear)?
            .checked_add(self.info.x)
            .ok_or(Error::Geometry)
    }
    fn new(info: MaskInfo, pen: f32, run: TextRun<'_>, size: f32) -> Result<Self, Error> {
        let y = rounded_i32(run.y)?
            .checked_add(info.y)
            .ok_or(Error::Geometry)?;
        y.checked_add(info.height.saturating_sub(1) as i32)
            .ok_or(Error::Geometry)?;
        let placement = Self {
            info,
            pen,
            size,
            y,
            color: run.color,
            italic: run.italic,
        };
        // Row origins are monotone for this fixed shear. Checking both endpoints
        // bounds every intermediate origin and the complete horizontal span.
        if info.height != 0 {
            for row in [0, info.height - 1] {
                placement
                    .row_x(row)?
                    .checked_add(info.width.saturating_sub(1) as i32)
                    .ok_or(Error::Geometry)?;
            }
        }
        Ok(placement)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaskView<'a> {
    info: MaskInfo,
    coverage: &'a [u8],
}
impl<'a> MaskView<'a> {
    pub fn info(self) -> MaskInfo {
        self.info
    }
    pub fn coverage(self) -> &'a [u8] {
        self.coverage
    }
}

/// Cumulative admission charges. On success coverage bytes equal retained mask
/// bytes. A failed session may retain debits for an allocation that was refused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub input_bytes: usize,
    pub input_scalars: usize,
    pub visited_scalars: usize,
    pub occurrences: usize,
    pub cold_requests: usize,
    pub unique_coverage_bytes: usize,
    pub cold_mask_work: u64,
    pub paint_pixels_used: u64,
    pub paint_pixels_remaining: u64,
    pub max_raster_scratch_bytes: usize,
    pub mask_lookup_comparisons: usize,
}

struct Record {
    info: MaskInfo,
    coverage: Vec<u8>,
}
impl Record {
    fn view(&self) -> MaskView<'_> {
        MaskView {
            info: self.info,
            coverage: &self.coverage,
        }
    }
}

/// One frame's private cache, including empty masks in first-acquisition order.
/// All errors poison the session; callers must refuse the complete GPU frame.
pub struct Session<'a> {
    fonts: &'a Fonts,
    masks: Vec<Record>,
    stats: Stats,
    failed: bool,
}

impl<'a> Session<'a> {
    pub fn new(fonts: &'a Fonts, cpu_pixel_allowance: u64) -> Result<Self, Error> {
        if cpu_pixel_allowance > MAX_CPU_PIXEL_ALLOWANCE {
            return Err(Error::PixelAllowance);
        }
        Ok(Self {
            fonts,
            masks: Vec::new(),
            stats: Stats {
                paint_pixels_remaining: cpu_pixel_allowance,
                ..Stats::default()
            },
            failed: false,
        })
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
    pub fn masks(&self) -> impl ExactSizeIterator<Item = MaskView<'_>> {
        self.masks.iter().map(Record::view)
    }

    /// `preflight` runs for each reached mask, cached or new, before owned
    /// coverage/cache growth and font rasterization. `is_new` identifies first
    /// acquisition. Callback references cannot outlive their visit. A caller may
    /// instead record keys and borrow all masks after the complete text walk.
    pub fn visit(
        &mut self,
        run: TextRun<'_>,
        clip: Rect,
        mut preflight: impl FnMut(MaskInfo, Placement, bool) -> Result<(), ()>,
        mut visitor: impl FnMut(MaskView<'_>, Placement) -> Result<(), ()>,
    ) -> Result<(), Error> {
        if self.failed {
            return Err(Error::SessionFailed);
        }
        let outcome = self.visit_inner(run, clip, &mut preflight, &mut visitor);
        if outcome.is_err() {
            self.failed = true;
        }
        outcome
    }

    fn visit_inner(
        &mut self,
        run: TextRun<'_>,
        clip: Rect,
        preflight: &mut impl FnMut(MaskInfo, Placement, bool) -> Result<(), ()>,
        visitor: &mut impl FnMut(MaskView<'_>, Placement) -> Result<(), ()>,
    ) -> Result<(), Error> {
        if run.text.len() > MAX_RUN_TEXT_BYTES {
            return Err(Error::RunTextBytes);
        }
        let bytes = self
            .stats
            .input_bytes
            .checked_add(run.text.len())
            .filter(|&n| n <= MAX_FRAME_TEXT_BYTES)
            .ok_or(Error::FrameTextBytes)?;
        // The byte preflight above bounds this complete scan, including text
        // that Canvas will later skip through its normal coarse culling.
        let scalars = self
            .stats
            .input_scalars
            .checked_add(run.text.chars().count())
            .filter(|&n| n <= MAX_FRAME_SCALARS)
            .ok_or(Error::FrameScalars)?;
        self.stats.input_bytes = bytes;
        self.stats.input_scalars = scalars;
        if ![clip.x, clip.y, clip.width, clip.height]
            .into_iter()
            .all(f32::is_finite)
            || clip.width < 0.0
            || clip.height < 0.0
        {
            return Err(Error::Geometry);
        }
        // Preserve Canvas::text's no-op and coarse clipping order. Source-byte
        // admission above is a separate conservative bridge limit.
        if !run.x.is_finite() || !run.y.is_finite() || !run.size.is_finite() || run.color.a == 0 {
            return Ok(());
        }
        let size = run.size.clamp(1.0, 512.0);
        if clip.width <= 0.0
            || clip.height <= 0.0
            || run.y > clip.y + clip.height
            || run.y + size * 1.5 < clip.y
        {
            return Ok(());
        }
        let face = Fonts::face_index(run.bold, run.monospace);
        let fonts = self.fonts;
        let scaled = fonts.faces[face].as_scaled(fonts.scale(face, size));
        let mut pen = run.x;
        let mut previous = None;
        for ch in run.text.chars() {
            // Full input is already limited to 4096 scalars per frame, below
            // Canvas's 32768/run and 100000/frame visited-glyph limits.
            self.stats.visited_scalars += 1;
            let id = scaled.glyph_id(ch);
            if let Some(previous) = previous {
                pen += scaled.kern(previous, id);
            }
            previous = Some(id);
            if pen > clip.x + clip.width + size {
                break;
            }
            if pen + size >= clip.x {
                let key = MaskKey {
                    face: face as u8,
                    character: ch,
                    eighths: (size.clamp(1.0, 512.0) * 8.0).round() as u16,
                };
                let mut cached = None;
                for (index, record) in self.masks.iter().enumerate() {
                    self.stats.mask_lookup_comparisons += 1;
                    if record.info.key == key {
                        cached = Some(index);
                        break;
                    }
                }
                if let Some(index) = cached {
                    let info = self.masks[index].info;
                    let placement = Placement::new(info, pen, run, size)?;
                    self.charge_occurrence(info)?;
                    preflight(info, placement, false).map_err(|()| Error::CallerRefused)?;
                    visitor(self.masks[index].view(), placement)
                        .map_err(|()| Error::CallerRefused)?;
                } else {
                    if self.stats.cold_requests >= MAX_COLD_REQUESTS {
                        return Err(Error::ColdRequests);
                    }
                    self.stats.cold_requests += 1;
                    // Trusted pinned-font call: its internal outline Vec is
                    // allocated before bounds can be inspected. No warm cache.
                    let quantized =
                        fonts.faces[face].as_scaled(fonts.scale(face, key.eighths as f32 / 8.0));
                    let glyph = quantized
                        .glyph_id(ch)
                        .with_scale_and_position(quantized.scale(), point(0.0, quantized.ascent()));
                    let outline = quantized.outline_glyph(glyph);
                    let info = if let Some(outline) = &outline {
                        let bounds = outline.px_bounds();
                        checked_info(
                            key,
                            bounds.min.x,
                            bounds.min.y,
                            bounds.width(),
                            bounds.height(),
                        )?
                    } else {
                        checked_info(key, 0.0, 0.0, 0.0, 0.0)?
                    };
                    let unique_bytes = self
                        .stats
                        .unique_coverage_bytes
                        .checked_add(info.area)
                        .filter(|&n| n <= MAX_UNIQUE_COVERAGE_BYTES)
                        .ok_or(Error::UniqueCoverage)?;
                    let placement = Placement::new(info, pen, run, size)?;
                    self.charge_occurrence(info)?;
                    preflight(info, placement, true).map_err(|()| Error::CallerRefused)?;
                    self.stats.unique_coverage_bytes = unique_bytes;
                    self.stats.cold_mask_work += info.area.max(1) as u64;
                    if outline.is_some() {
                        self.stats.max_raster_scratch_bytes = self
                            .stats
                            .max_raster_scratch_bytes
                            .max((info.area + 4) * size_of::<f32>());
                    }
                    let mut coverage = Vec::new();
                    coverage
                        .try_reserve_exact(info.area)
                        .map_err(|_| Error::Allocation)?;
                    coverage.resize(info.area, 0);
                    self.masks
                        .try_reserve_exact(1)
                        .map_err(|_| Error::Allocation)?;
                    if let Some(outline) = outline {
                        let mut outside = false;
                        // Dependency draw is noncancellable and allocates the
                        // admitted scratch above with its own infallible Vec.
                        outline.draw(|x, y, coverage_value| {
                            let (x, y) = (x as usize, y as usize);
                            if x < info.width && y < info.height {
                                coverage[y * info.width + x] = (coverage_value * 255.0) as u8;
                            } else {
                                outside = true;
                            }
                        });
                        if outside {
                            return Err(Error::RasterBounds);
                        }
                    }
                    self.masks.push(Record { info, coverage });
                    visitor(self.masks.last().expect("inserted mask").view(), placement)
                        .map_err(|()| Error::CallerRefused)?;
                }
            }
            pen += scaled.h_advance(id);
        }
        Ok(())
    }

    fn charge_occurrence(&mut self, info: MaskInfo) -> Result<(), Error> {
        let pixels = info.area.max(1) as u64;
        if pixels > self.stats.paint_pixels_remaining {
            self.stats.paint_pixels_used += self.stats.paint_pixels_remaining;
            self.stats.paint_pixels_remaining = 0;
            return Err(Error::CpuPixels);
        }
        self.stats.paint_pixels_remaining -= pixels;
        self.stats.paint_pixels_used += pixels;
        self.stats.occurrences += 1;
        Ok(())
    }
}

fn rounded_i32(value: f32) -> Result<i32, Error> {
    let value = value.round();
    if !value.is_finite() || !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        return Err(Error::Geometry);
    }
    Ok(value as i32)
}

fn checked_info(key: MaskKey, x: f32, y: f32, width: f32, height: f32) -> Result<MaskInfo, Error> {
    if ![x, y, width, height].into_iter().all(f32::is_finite) || width < 0.0 || height < 0.0 {
        return Err(Error::Geometry);
    }
    if width > MAX_MASK_AXIS as f32 || height > MAX_MASK_AXIS as f32 {
        return Err(Error::MaskAxis);
    }
    // Outline pixel bounds have integer bearings; reject unexpected fractional
    // values rather than change the CPU's truncating bearing conversion.
    if x.fract() != 0.0 || y.fract() != 0.0 {
        return Err(Error::Geometry);
    }
    let (width, height) = (width as usize, height as usize);
    let area = width
        .checked_mul(height)
        .filter(|&n| n <= MAX_MASK_AREA)
        .ok_or(Error::MaskArea)?;
    Ok(MaskInfo {
        key,
        x: rounded_i32(x)?,
        y: rounded_i32(y)?,
        width,
        height,
        area,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn clip() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0,
        }
    }
    fn run(text: &str) -> TextRun<'_> {
        TextRun {
            x: 0.0,
            y: 0.0,
            text,
            size: 16.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        }
    }
    fn key() -> MaskKey {
        MaskKey {
            face: 0,
            character: 'A',
            eighths: 80,
        }
    }
    fn accept(_: MaskInfo, _: Placement, _: bool) -> Result<(), ()> {
        Ok(())
    }
    fn discard(_: MaskView<'_>, _: Placement) -> Result<(), ()> {
        Ok(())
    }

    #[test]
    fn text_masks_bound_information_and_signed_row_origins_without_allocation() {
        let info = checked_info(key(), -1.0, 2.0, 3.0, 3.0).unwrap();
        let mut text = run("");
        text.y = -0.5;
        text.italic = true;
        let placement = Placement::new(info, 0.25, text, 10.0).unwrap();
        assert_eq!(placement.origin_y(), 1);
        assert_eq!(placement.row_x(0), Ok(1));
        assert_eq!(placement.row_x(1), Ok(1));
        assert_eq!(placement.row_x(2), Ok(0));
        assert_eq!(placement.row_x(3), Err(Error::RowIndex));
        text.italic = false;
        assert_eq!(
            Placement::new(info, -0.5, text, 10.0).unwrap().row_x(0),
            Ok(-2)
        );
        assert_eq!(
            checked_info(key(), 0.0, 0.0, 1_025.0, 1.0),
            Err(Error::MaskAxis)
        );
        assert_eq!(
            checked_info(key(), 0.0, 0.0, 513.0, 512.0),
            Err(Error::MaskArea)
        );
        assert_eq!(
            checked_info(key(), 0.5, 0.0, 1.0, 1.0),
            Err(Error::Geometry)
        );
        assert_eq!(rounded_i32(2_147_483_648.0), Err(Error::Geometry));
        assert_eq!(rounded_i32(f32::NAN), Err(Error::Geometry));
        assert_eq!(
            checked_info(key(), 0.0, 0.0, 512.0, 512.0).unwrap().area(),
            MAX_MASK_AREA
        );
    }

    #[test]
    fn text_masks_empty_records_repeat_in_first_acquisition_order() {
        let fonts = Fonts::new();
        let mut session = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        let mut keys = Vec::new();
        let mut fresh = Vec::new();
        session
            .visit(
                run(" A A"),
                clip(),
                |info, _, is_new| {
                    keys.push(info.key());
                    fresh.push(is_new);
                    Ok(())
                },
                discard,
            )
            .unwrap();
        assert_eq!(fresh, [true, true, false, false]);
        assert_eq!(keys[0], keys[2]);
        assert_eq!(keys[1], keys[3]);
        let masks: Vec<_> = session.masks().collect();
        assert_eq!(masks.len(), 2);
        assert_eq!(masks[0].info().key().character(), ' ');
        assert!(masks[0].coverage().is_empty());
        assert_eq!(masks[1].info().key().character(), 'A');
        assert!(!masks[1].coverage().is_empty());
        assert_eq!(
            session.stats().paint_pixels_used,
            2 + 2 * masks[1].info().area() as u64
        );
        assert_eq!(session.stats().cold_requests, 2);
        assert_eq!(session.stats().occurrences, 4);
        assert_eq!(
            session.stats().unique_coverage_bytes,
            masks[1].coverage().len()
        );
    }

    #[test]
    fn text_masks_are_independent_of_warm_cache_and_preserve_published_mask_bytes() {
        let fonts = Fonts::new();
        let warm = fonts.bitmap(0, 'A', 16.0);
        let mut session = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        session.visit(run("AA"), clip(), accept, discard).unwrap();
        let actual = session.masks().next().unwrap();
        assert_eq!(actual.coverage(), warm.alpha);
        assert_eq!(
            (actual.info().bearing_x(), actual.info().bearing_y()),
            (warm.x, warm.y)
        );
        assert_eq!(
            (actual.info().width(), actual.info().height()),
            (warm.width, warm.height)
        );
        assert_eq!(session.stats().cold_requests, 1);
        assert_eq!(session.stats().cold_mask_work, warm.alpha.len() as u64);
        assert_eq!(
            session.stats().max_raster_scratch_bytes,
            4 * (warm.alpha.len() + 4)
        );
        let mut fresh = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        fresh.visit(run("AA"), clip(), accept, discard).unwrap();
        assert_eq!(fresh.stats(), session.stats());
    }

    #[test]
    fn text_masks_quantization_and_monospace_override_are_explicit() {
        let fonts = Fonts::new();
        let mut session = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        let mut text = run("A");
        text.size = 10.0624;
        session.visit(text, clip(), accept, discard).unwrap();
        text.size = 10.0625;
        session.visit(text, clip(), accept, discard).unwrap();
        text.bold = true;
        text.monospace = true;
        text.italic = true;
        session.visit(text, clip(), accept, discard).unwrap();
        let keys: Vec<_> = session.masks().map(|mask| mask.info().key()).collect();
        assert_eq!(keys.len(), 3);
        assert_eq!((keys[0].face_index(), keys[0].quantized_eighths()), (0, 80));
        assert_eq!((keys[1].face_index(), keys[1].quantized_eighths()), (0, 81));
        assert_eq!((keys[2].face_index(), keys[2].quantized_eighths()), (2, 81));
    }

    #[test]
    fn text_masks_advance_uses_original_size_while_mask_identity_is_shared() {
        let fonts = Fonts::new();
        let mut session = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        let mut small = Vec::new();
        let mut large = Vec::new();
        let mut text = run("AA");
        text.size = 10.01;
        session
            .visit(
                text,
                clip(),
                |_, placement, _| {
                    small.push(placement.pen());
                    Ok(())
                },
                discard,
            )
            .unwrap();
        text.size = 10.04;
        session
            .visit(
                text,
                clip(),
                |_, placement, _| {
                    large.push(placement.pen());
                    Ok(())
                },
                discard,
            )
            .unwrap();
        assert_eq!(session.masks().len(), 1);
        assert_eq!(small.len(), 2);
        assert_eq!(large.len(), 2);
        assert_eq!(small[0], 0.0);
        assert_eq!(large[0], 0.0);
        assert!(large[1] > small[1]);
        assert_eq!(session.stats().cold_requests, 1);
    }

    #[test]
    fn text_masks_caller_refusal_precedes_mask_and_cache_allocation() {
        let fonts = Fonts::new();
        let mut session = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        let calls = Cell::new(0);
        let result = session.visit(
            run("A"),
            clip(),
            |_, _, is_new| {
                assert!(is_new);
                calls.set(calls.get() + 1);
                Err(())
            },
            |_, _| panic!("refused preflight must not visit"),
        );
        assert_eq!(result, Err(Error::CallerRefused));
        assert_eq!(calls.get(), 1);
        assert_eq!(session.masks.len(), 0);
        assert_eq!(session.masks.capacity(), 0);
        assert_eq!(session.stats().cold_requests, 1);
        assert_eq!(session.stats().unique_coverage_bytes, 0);
        assert_eq!(session.stats().cold_mask_work, 0);
        assert_eq!(session.stats().max_raster_scratch_bytes, 0);
        assert_eq!(
            session.visit(run(""), clip(), accept, discard),
            Err(Error::SessionFailed)
        );
    }

    #[test]
    fn text_masks_cached_and_visitor_refusals_remain_terminal() {
        let fonts = Fonts::new();
        let mut session = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        session.visit(run("A"), clip(), accept, discard).unwrap();
        let bytes = session.stats().unique_coverage_bytes;
        assert_eq!(
            session.visit(
                run("A"),
                clip(),
                |_, _, is_new| {
                    assert!(!is_new);
                    Err(())
                },
                discard
            ),
            Err(Error::CallerRefused)
        );
        assert_eq!(session.stats().cold_requests, 1);
        assert_eq!(session.stats().unique_coverage_bytes, bytes);
        let mut visitor_failure = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        assert_eq!(
            visitor_failure.visit(run("A"), clip(), accept, |_, _| Err(())),
            Err(Error::CallerRefused)
        );
        assert_eq!(visitor_failure.masks().len(), 1);
        assert_eq!(
            visitor_failure.visit(run("A"), clip(), accept, discard),
            Err(Error::SessionFailed)
        );
    }

    #[test]
    fn text_masks_zero_allowance_culling_and_one_short_paint_work() {
        let fonts = Fonts::new();
        assert!(matches!(
            Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE + 1),
            Err(Error::PixelAllowance)
        ));
        let mut no_work = Session::new(&fonts, 0).unwrap();
        let mut text = run("A");
        text.color.a = 0;
        no_work.visit(text, clip(), accept, discard).unwrap();
        text.color.a = 255;
        text.y = 241.0;
        no_work.visit(text, clip(), accept, discard).unwrap();
        assert_eq!(no_work.stats().visited_scalars, 0);
        assert_eq!(no_work.stats().cold_requests, 0);
        text.y = 0.0;
        text.x = 337.0;
        no_work.visit(text, clip(), accept, discard).unwrap();
        assert_eq!(no_work.stats().visited_scalars, 1);
        assert_eq!(no_work.stats().occurrences, 0);
        let mut measure = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        measure.visit(run("A"), clip(), accept, discard).unwrap();
        let exact = measure.stats().paint_pixels_used;
        assert!(exact > 0);
        let mut one_short = Session::new(&fonts, exact - 1).unwrap();
        assert_eq!(
            one_short.visit(
                run("A"),
                clip(),
                |_, _, _| panic!("pixel refusal precedes callback"),
                discard
            ),
            Err(Error::CpuPixels)
        );
        assert_eq!(one_short.masks().len(), 0);
        assert_eq!(one_short.stats().paint_pixels_remaining, 0);
        let mut exact_session = Session::new(&fonts, exact).unwrap();
        exact_session
            .visit(run("A"), clip(), accept, discard)
            .unwrap();
        assert_eq!(exact_session.stats().paint_pixels_remaining, 0);
        let mut empty = Session::new(&fonts, 0).unwrap();
        assert_eq!(
            empty.visit(run(" "), clip(), accept, discard),
            Err(Error::CpuPixels)
        );
    }

    #[test]
    fn text_masks_full_input_and_cold_limits_refuse_before_growth() {
        let fonts = Fonts::new();
        let too_long = "a".repeat(MAX_RUN_TEXT_BYTES + 1);
        let mut bytes = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        assert_eq!(
            bytes.visit(run(&too_long), clip(), accept, discard),
            Err(Error::RunTextBytes)
        );
        assert_eq!(bytes.stats().cold_requests, 0);
        let too_many = "a".repeat(MAX_FRAME_SCALARS + 1);
        let mut scalars = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        let mut text = run(&too_many);
        text.color.a = 0;
        assert_eq!(
            scalars.visit(text, clip(), accept, discard),
            Err(Error::FrameScalars)
        );
        let mut frame_bytes = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        frame_bytes.stats.input_bytes = MAX_FRAME_TEXT_BYTES;
        assert_eq!(
            frame_bytes.visit(run("a"), clip(), accept, discard),
            Err(Error::FrameTextBytes)
        );
        let mut cold = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        cold.stats.cold_requests = MAX_COLD_REQUESTS;
        assert_eq!(
            cold.visit(run("A"), clip(), accept, discard),
            Err(Error::ColdRequests)
        );
        assert_eq!(cold.masks.capacity(), 0);
        let mut coverage = Session::new(&fonts, MAX_CPU_PIXEL_ALLOWANCE).unwrap();
        coverage.stats.unique_coverage_bytes = MAX_UNIQUE_COVERAGE_BYTES;
        assert_eq!(
            coverage.visit(run("A"), clip(), accept, discard),
            Err(Error::UniqueCoverage)
        );
        assert_eq!(coverage.masks.capacity(), 0);
    }
}
