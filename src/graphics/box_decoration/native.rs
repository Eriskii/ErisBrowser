//! Funded Native coverage preparation using the existing sixteen-sample mask.
//!
//! Raw Native coordinates are checked by the adapter. This helper receives
//! shapes translated once and preserves the software painter's f32 window.
use super::{CoverageClass, MAX_COORDINATE, RoundedBox, SETUP_WORK, ShapeMask};
use crate::graphics::Rect;
use eris_raster_core::{
    MAX_MASK_AXIS, MAX_MASK_COVERAGE_BYTES, MAX_MASK_PIXELS, MAX_ROW_ENTRIES, Profile,
};
use std::mem::size_of;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BoxMaskError {
    InvalidGeometry,
    MaskLimit,
    WorkLimit,
    AllocationFailure,
}

#[derive(Clone, Copy)]
struct PixelWindow {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BoxMaskTileInfo {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    pub coverage_bytes: usize,
    pub row_entries: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BoxMaskInfo {
    pub candidate_pixels: u64,
    pub ambiguous_pixels: u64,
    pub shape_count: u64,
    pub classification_work: u64,
    pub materialization_work: u64,
    pub tile_count: usize,
    pub coverage_bytes: usize,
    pub row_entries: usize,
    pub cpu_payload_bytes: usize,
    pub packed_input_bytes: usize,
}

/// Validated all-covered candidate metadata, before paint-specific funding.
/// Construction stays private; no caller can invent a proof by filling fields.
pub(crate) struct BoxRectProof {
    candidate_pixels: u64,
    classification_work: u64,
}

impl BoxRectProof {
    fn new(candidate_pixels: u64) -> Result<Self, BoxMaskError> {
        let classification_work = if candidate_pixels == 0 {
            0
        } else {
            candidate_pixels
                .checked_mul(2)
                .and_then(|work| work.checked_add(SETUP_WORK))
                .ok_or(BoxMaskError::WorkLimit)?
        };
        Ok(Self {
            candidate_pixels,
            classification_work,
        })
    }

    pub(crate) fn candidate_pixels(&self) -> u64 {
        self.candidate_pixels
    }

    pub(crate) fn classification_work(&self) -> u64 {
        self.classification_work
    }
}

pub(crate) struct BoxMaskWindow {
    mask: ShapeMask,
    candidate: PixelWindow,
    partition: [Option<BoxMaskTileInfo>; 2],
    info: BoxMaskInfo,
}

// Neither object is Clone/Copy. The adapter moves its counted reservation
// from the original-command table into one materialization.
pub(crate) struct BoxMaskReservation {
    window: BoxMaskWindow,
}

pub(crate) struct BoxMaskTile {
    info: BoxMaskTileInfo,
    coverage: Vec<u8>,
    rows: Vec<i32>,
}

pub(crate) struct BoxMaskTiles {
    tiles: [Option<BoxMaskTile>; 2],
}

fn reserve_vec<T>(count: usize) -> Result<Vec<T>, BoxMaskError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| BoxMaskError::AllocationFailure)?;
    Ok(values)
}

fn debit(remaining: &mut u64, work: u64) -> Result<(), BoxMaskError> {
    *remaining = remaining.checked_sub(work).ok_or(BoxMaskError::WorkLimit)?;
    Ok(())
}

impl BoxMaskWindow {
    /// Preserve the Color route's conservative software tariff, 8 + 3P.
    /// The caller prepays this result before retaining the selected route.
    pub(crate) fn rect_work(
        width: u32,
        height: u32,
        outer: RoundedBox,
        cutout: Option<RoundedBox>,
        clip: Rect,
    ) -> Result<Option<u64>, BoxMaskError> {
        Self::rect_proof(width, height, outer, cutout, clip)?
            .map(|proof| {
                proof
                    .classification_work()
                    .checked_add(proof.candidate_pixels())
                    .ok_or(BoxMaskError::WorkLimit)
            })
            .transpose()
    }

    /// Prove all-covered original Box candidates and exact raw Rect clipping.
    /// Inputs are already translated. The immutable proof exposes the full
    /// outer candidate count, before an image's later no-repeat narrowing.
    /// No pixel scan, payload allocation or work debit occurs here.
    pub(crate) fn rect_proof(
        width: u32,
        height: u32,
        outer: RoundedBox,
        cutout: Option<RoundedBox>,
        clip: Rect,
    ) -> Result<Option<BoxRectProof>, BoxMaskError> {
        // Keep invalid input observable before empty/ineligible early returns.
        if width == 0
            || height == 0
            || width > Profile::Native.max_width()
            || height > Profile::Native.max_height()
            || !outer.valid()
            || cutout.is_some_and(|shape| !shape.valid())
            || ![clip.x, clip.y, clip.width, clip.height]
                .into_iter()
                .all(|v| v.is_finite() && f64::from(v).abs() <= MAX_COORDINATE)
        {
            return Err(BoxMaskError::InvalidGeometry);
        }
        if cutout.is_some()
            || outer.radii.iter().any(|[rx, ry]| *rx != 0.0 && *ry != 0.0)
            || ![
                outer.rect.x,
                outer.rect.y,
                outer.rect.width,
                outer.rect.height,
            ]
            .into_iter()
            .all(|v| v.fract() == 0.0)
        {
            return Ok(None);
        }
        if outer.rect.width <= 0.0
            || outer.rect.height <= 0.0
            || clip.width <= 0.0
            || clip.height <= 0.0
        {
            return BoxRectProof::new(0).map(Some);
        }
        // Preserve the original Canvas/core f32 candidate-window expression.
        let visible = outer.rect.intersect(clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return BoxRectProof::new(0).map(Some);
        }
        let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as i32;
        let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as i32;
        let x1 = (visible.x + visible.width).ceil().min(width as f32) as i32;
        let y1 = (visible.y + visible.height).ceil().min(height as f32) as i32;
        if x1 <= x0 || y1 <= y0 {
            return BoxRectProof::new(0).map(Some);
        }
        // Box always blends through its mask. Opaque root Rect omits blend's
        // upper clip check, so prove that all candidate cells pass it anyway.
        // The integer shape bounds also prove all sixteen samples are inside.
        if f64::from(x0) < f64::from(outer.rect.x)
            || f64::from(y0) < f64::from(outer.rect.y)
            || f64::from(x1) > f64::from(outer.rect.x) + f64::from(outer.rect.width)
            || f64::from(y1) > f64::from(outer.rect.y) + f64::from(outer.rect.height)
            || x1 as f32 > (clip.x + clip.width).ceil()
            || y1 as f32 > (clip.y + clip.height).ceil()
        {
            return Ok(None);
        }
        let pixels = ((x1 - x0) as u64)
            .checked_mul((y1 - y0) as u64)
            .ok_or(BoxMaskError::WorkLimit)?;
        BoxRectProof::new(pixels).map(Some)
    }

    /// Constant-size metadata only. Inputs are already in framebuffer space;
    /// neither translation nor clip normalization is repeated here.
    pub(crate) fn new(
        width: u32,
        height: u32,
        outer: RoundedBox,
        cutout: Option<RoundedBox>,
        clip: Rect,
    ) -> Result<Self, BoxMaskError> {
        if width == 0
            || height == 0
            || width > Profile::Native.max_width()
            || height > Profile::Native.max_height()
            || !outer.valid()
            || cutout.is_some_and(|shape| !shape.valid())
            || ![clip.x, clip.y, clip.width, clip.height]
                .into_iter()
                .all(|v| v.is_finite() && f64::from(v).abs() <= MAX_COORDINATE)
        {
            return Err(BoxMaskError::InvalidGeometry);
        }
        let mut value = Self {
            mask: ShapeMask { outer, cutout },
            candidate: PixelWindow {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            partition: [None, None],
            info: BoxMaskInfo {
                candidate_pixels: 0,
                ambiguous_pixels: 0,
                shape_count: 1 + u64::from(cutout.is_some()),
                classification_work: 0,
                materialization_work: 0,
                tile_count: 0,
                coverage_bytes: 0,
                row_entries: 0,
                cpu_payload_bytes: 0,
                packed_input_bytes: 0,
            },
        };
        if outer.rect.width <= 0.0
            || outer.rect.height <= 0.0
            || clip.width <= 0.0
            || clip.height <= 0.0
        {
            return Ok(value);
        }
        let visible = outer.rect.intersect(clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return Ok(value);
        }
        // Preserve Canvas::box_decoration_offset and Canvas::rect arithmetic,
        // including reconstructed f32 endpoints before integer conversion.
        let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as i32;
        let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as i32;
        let x1 = (visible.x + visible.width).ceil().min(width as f32) as i32;
        let y1 = (visible.y + visible.height).ceil().min(height as f32) as i32;
        if x1 <= x0 || y1 <= y0 {
            return Ok(value);
        }
        value.candidate = PixelWindow {
            x: x0,
            y: y0,
            width: (x1 - x0) as u32,
            height: (y1 - y0) as u32,
        };
        value.info.candidate_pixels = u64::from(value.candidate.width)
            .checked_mul(u64::from(value.candidate.height))
            .ok_or(BoxMaskError::WorkLimit)?;
        // blend() rejects integer origins outside the half-open clip. Storage
        // may omit those cells, while both work stages retain the original P.
        let right = x1.min((clip.x + clip.width).ceil() as i32);
        let bottom = y1.min((clip.y + clip.height).ceil() as i32);
        if right <= x0 || bottom <= y0 {
            return Ok(value);
        }
        let stored_width = (right - x0) as u32;
        let stored_height = (bottom - y0) as u32;
        let area = (stored_width as usize)
            .checked_mul(stored_height as usize)
            .filter(|&n| n <= MAX_MASK_PIXELS && n <= MAX_MASK_COVERAGE_BYTES)
            .ok_or(BoxMaskError::MaskLimit)?;
        let count = stored_width.div_ceil(MAX_MASK_AXIS) as usize;
        if count > 2 || stored_height > MAX_MASK_AXIS {
            return Err(BoxMaskError::MaskLimit);
        }
        let rows = (stored_height as usize)
            .checked_mul(count)
            .filter(|&n| n <= MAX_ROW_ENTRIES)
            .ok_or(BoxMaskError::MaskLimit)?;
        value.info.tile_count = count;
        value.info.coverage_bytes = area;
        value.info.row_entries = rows;
        value.info.cpu_payload_bytes = rows
            .checked_mul(size_of::<i32>())
            .and_then(|n| n.checked_add(area))
            .ok_or(BoxMaskError::MaskLimit)?;
        value.info.packed_input_bytes = area
            .checked_add(rows)
            .and_then(|n| n.checked_mul(size_of::<u32>()))
            .ok_or(BoxMaskError::MaskLimit)?;
        for (index, slot) in value.partition.iter_mut().take(count).enumerate() {
            let column = index as u32 * MAX_MASK_AXIS;
            let tile_width = (stored_width - column).min(MAX_MASK_AXIS);
            *slot = Some(BoxMaskTileInfo {
                origin_x: x0 + column as i32,
                origin_y: y0,
                width: tile_width,
                height: stored_height,
                coverage_bytes: tile_width as usize * stored_height as usize,
                row_entries: stored_height as usize,
            });
        }
        Ok(value)
    }

    pub(crate) fn reserve(
        self,
        remaining_work: &mut u64,
    ) -> Result<BoxMaskReservation, BoxMaskError> {
        self.reserve_with(remaining_work, || {})
    }

    // Private hook proves that no classifier runs before its debit. Production
    // supplies a no-op; no caller can request an unfunded coverage evaluation.
    fn reserve_with(
        mut self,
        remaining_work: &mut u64,
        mut classified: impl FnMut(),
    ) -> Result<BoxMaskReservation, BoxMaskError> {
        let p = self.info.candidate_pixels;
        let s = self.info.shape_count;
        if p == 0 {
            return Ok(BoxMaskReservation { window: self });
        }
        let classification = p
            .checked_mul(s)
            .and_then(|n| n.checked_add(SETUP_WORK))
            .ok_or(BoxMaskError::WorkLimit)?;
        debit(remaining_work, classification)?;
        let mut ambiguous = 0u64;
        let candidate = self.candidate;
        for y in candidate.y..candidate.y + candidate.height as i32 {
            for x in candidate.x..candidate.x + candidate.width as i32 {
                classified();
                ambiguous += u64::from(self.mask.classify(x, y) == CoverageClass::Partial);
            }
        }
        let sample_work = ambiguous
            .checked_mul(16)
            .and_then(|n| n.checked_mul(s))
            .ok_or(BoxMaskError::WorkLimit)?;
        let materialization = p
            .checked_mul(s)
            .and_then(|n| n.checked_add(sample_work))
            .and_then(|n| n.checked_add(p))
            .and_then(|n| n.checked_add(self.info.coverage_bytes as u64))
            .and_then(|n| n.checked_add(self.info.row_entries as u64))
            .ok_or(BoxMaskError::WorkLimit)?;
        debit(remaining_work, materialization)?;
        self.info.ambiguous_pixels = ambiguous;
        self.info.classification_work = classification;
        self.info.materialization_work = materialization;
        Ok(BoxMaskReservation { window: self })
    }
}

impl BoxMaskReservation {
    pub(crate) fn info(&self) -> BoxMaskInfo {
        self.window.info
    }

    pub(crate) fn tiles(&self) -> impl Iterator<Item = BoxMaskTileInfo> + '_ {
        self.window.partition.iter().flatten().copied()
    }

    /// The adapter's one cumulative admission covers both strips and executes
    /// before any payload reserve. CPU classification/sample work is funded.
    pub(crate) fn materialize<E>(
        self,
        admit: impl FnOnce(&Self) -> Result<(), E>,
        error: impl Fn(BoxMaskError) -> E,
    ) -> Result<BoxMaskTiles, E> {
        self.materialize_with(admit, error, reserve_vec::<u8>, reserve_vec::<i32>, || {})
    }

    fn materialize_with<E>(
        self,
        admit: impl FnOnce(&Self) -> Result<(), E>,
        error: impl Fn(BoxMaskError) -> E,
        mut coverage_reserve: impl FnMut(usize) -> Result<Vec<u8>, BoxMaskError>,
        mut row_reserve: impl FnMut(usize) -> Result<Vec<i32>, BoxMaskError>,
        mut evaluated: impl FnMut(),
    ) -> Result<BoxMaskTiles, E> {
        admit(&self)?;
        let mut result = BoxMaskTiles {
            tiles: [None, None],
        };
        for (slot, info) in result.tiles.iter_mut().zip(self.tiles()) {
            let coverage = coverage_reserve(info.coverage_bytes).map_err(&error)?;
            let rows = row_reserve(info.row_entries).map_err(&error)?;
            if !coverage.is_empty()
                || coverage.capacity() < info.coverage_bytes
                || !rows.is_empty()
                || rows.capacity() < info.row_entries
            {
                return Err(error(BoxMaskError::AllocationFailure));
            }
            *slot = Some(BoxMaskTile {
                info,
                coverage,
                rows,
            });
        }
        // Every reserve succeeded. No geometry is rebased at the strip seam.
        for tile in result.tiles.iter_mut().flatten() {
            tile.rows.resize(tile.info.row_entries, tile.info.origin_x);
            for y in tile.info.origin_y..tile.info.origin_y + tile.info.height as i32 {
                for x in tile.info.origin_x..tile.info.origin_x + tile.info.width as i32 {
                    evaluated();
                    tile.coverage.push(self.window.mask.coverage(x, y));
                }
            }
        }
        Ok(result)
    }
}

impl BoxMaskTiles {
    pub(crate) fn into_tiles(self) -> impl Iterator<Item = BoxMaskTile> {
        self.tiles.into_iter().flatten()
    }
}

impl BoxMaskTile {
    pub(crate) fn info(&self) -> BoxMaskTileInfo {
        self.info
    }
    pub(crate) fn coverage(&self) -> &[u8] {
        &self.coverage
    }
    pub(crate) fn row_origins(&self) -> &[i32] {
        &self.rows
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod rect_tests;

#[cfg(test)]
mod proof_tests;
