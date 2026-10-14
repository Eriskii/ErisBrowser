//! An all-covered integer interior for individually oversized Color masks.
//! Existing small-mask preparation is deliberately outside this helper.
use super::{
    BoxMaskError, BoxMaskInfo, BoxMaskTile, BoxMaskTileInfo, CoverageClass, MAX_COORDINATE,
    MAX_MASK_AXIS, MAX_MASK_COVERAGE_BYTES, MAX_MASK_PIXELS, MAX_ROW_ENTRIES, PixelWindow, Profile,
    Rect, RoundedBox, SETUP_WORK, ShapeMask, debit, reserve_vec,
};
use std::mem::size_of;

pub(crate) struct BoxCoreWindow {
    mask: ShapeMask,
    candidate: PixelWindow,
    stored: PixelWindow,
    core: PixelWindow,
    partition: [Option<BoxMaskTileInfo>; 7],
    info: BoxMaskInfo,
}

// Reservations and payload owners move exactly once into the native batch.
pub(crate) struct BoxCoreReservation {
    window: BoxCoreWindow,
}
pub(crate) struct BoxCoreTiles {
    tiles: [Option<BoxMaskTile>; 7],
}

// The inner window is nonempty and wholly contained in the outer window.
// Top/bottom span the outer width; the two sides span only the inner height.
fn bands(outer: PixelWindow, inner: PixelWindow) -> [PixelWindow; 4] {
    let right = outer.x + outer.width as i32;
    let bottom = outer.y + outer.height as i32;
    let inner_right = inner.x + inner.width as i32;
    let inner_bottom = inner.y + inner.height as i32;
    [
        PixelWindow {
            x: outer.x,
            y: outer.y,
            width: outer.width,
            height: (inner.y - outer.y) as u32,
        },
        PixelWindow {
            x: outer.x,
            y: inner_bottom,
            width: outer.width,
            height: (bottom - inner_bottom) as u32,
        },
        PixelWindow {
            x: outer.x,
            y: inner.y,
            width: (inner.x - outer.x) as u32,
            height: inner.height,
        },
        PixelWindow {
            x: inner_right,
            y: inner.y,
            width: (right - inner_right) as u32,
            height: inner.height,
        },
    ]
}

impl BoxCoreWindow {
    /// Fixed metadata only. Validation remains observable even for ineligible
    /// shapes; used radii are never normalized or replaced with circular arcs.
    pub(crate) fn new(
        width: u32,
        height: u32,
        outer: RoundedBox,
        cutout: Option<RoundedBox>,
        clip: Rect,
    ) -> Result<Option<Self>, BoxMaskError> {
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
            || ![
                outer.rect.x,
                outer.rect.y,
                outer.rect.width,
                outer.rect.height,
            ]
            .into_iter()
            .all(|v| v.fract() == 0.0)
            || outer.rect.width <= 0.0
            || outer.rect.height <= 0.0
            || clip.width <= 0.0
            || clip.height <= 0.0
        {
            return Ok(None);
        }
        // Exactly the old f32 candidate and stored-window expressions. C pays
        // both original pixel tariffs even when blend's upper clip omits cells.
        let visible = outer.rect.intersect(clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return Ok(None);
        }
        let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as i32;
        let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as i32;
        let x1 = (visible.x + visible.width).ceil().min(width as f32) as i32;
        let y1 = (visible.y + visible.height).ceil().min(height as f32) as i32;
        if x1 <= x0 || y1 <= y0 {
            return Ok(None);
        }
        let candidate = PixelWindow {
            x: x0,
            y: y0,
            width: (x1 - x0) as u32,
            height: (y1 - y0) as u32,
        };
        let right = x1.min((clip.x + clip.width).ceil() as i32);
        let bottom = y1.min((clip.y + clip.height).ceil() as i32);
        if right <= x0 || bottom <= y0 {
            return Ok(None);
        }
        let stored = PixelWindow {
            x: x0,
            y: y0,
            width: (right - x0) as u32,
            height: (bottom - y0) as u32,
        };
        let area = (stored.width as usize)
            .checked_mul(stored.height as usize)
            .ok_or(BoxMaskError::MaskLimit)?;
        // Do not reinterpret another old MaskLimit, or route individually
        // admitted masks around the shared mask allowance.
        if area <= MAX_MASK_PIXELS && area <= MAX_MASK_COVERAGE_BYTES {
            return Ok(None);
        }
        let radii = outer.radii.map(|[rx, ry]| {
            if rx == 0.0 || ry == 0.0 {
                [0.0, 0.0]
            } else {
                [f64::from(rx), f64::from(ry)]
            }
        });
        let left = (f64::from(outer.rect.x) + radii[0][0].max(radii[3][0]))
            .ceil()
            .max(f64::from(x0));
        let top = (f64::from(outer.rect.y) + radii[0][1].max(radii[1][1]))
            .ceil()
            .max(f64::from(y0));
        let core_right = (f64::from(outer.rect.x) + f64::from(outer.rect.width)
            - radii[1][0].max(radii[2][0]))
        .floor()
        .min(f64::from(right));
        let core_bottom = (f64::from(outer.rect.y) + f64::from(outer.rect.height)
            - radii[2][1].max(radii[3][1]))
        .floor()
        .min(f64::from(bottom));
        if core_right <= left || core_bottom <= top {
            return Ok(None);
        }
        let core = PixelWindow {
            x: left as i32,
            y: top as i32,
            width: (core_right - left) as u32,
            height: (core_bottom - top) as u32,
        };
        let core_area = (core.width as usize)
            .checked_mul(core.height as usize)
            .ok_or(BoxMaskError::MaskLimit)?;
        let edges = area.checked_sub(core_area).ok_or(BoxMaskError::MaskLimit)?;
        if edges > MAX_MASK_PIXELS || edges > MAX_MASK_COVERAGE_BYTES {
            return Ok(None);
        }
        let mut value = Self {
            mask: ShapeMask { outer, cutout },
            candidate,
            stored,
            core,
            partition: [None; 7],
            info: BoxMaskInfo {
                candidate_pixels: u64::from(candidate.width)
                    .checked_mul(u64::from(candidate.height))
                    .ok_or(BoxMaskError::WorkLimit)?,
                ambiguous_pixels: 0,
                shape_count: 1,
                classification_work: 0,
                materialization_work: 0,
                tile_count: 0,
                coverage_bytes: edges,
                row_entries: 0,
                cpu_payload_bytes: 0,
                packed_input_bytes: 0,
            },
        };
        // At most seven strips: two each across top/bottom, and at most
        // three for the sides whose combined widths are less than1280.
        for band in bands(value.stored, value.core) {
            if band.width == 0 || band.height == 0 {
                continue;
            }
            if band.height > MAX_MASK_AXIS {
                return Ok(None);
            }
            for column in 0..band.width.div_ceil(MAX_MASK_AXIS) {
                let width = (band.width - column * MAX_MASK_AXIS).min(MAX_MASK_AXIS);
                let Some(slot) = value.partition.get_mut(value.info.tile_count) else {
                    return Ok(None);
                };
                *slot = Some(BoxMaskTileInfo {
                    origin_x: band.x + (column * MAX_MASK_AXIS) as i32,
                    origin_y: band.y,
                    width,
                    height: band.height,
                    coverage_bytes: width as usize * band.height as usize,
                    row_entries: band.height as usize,
                });
                value.info.tile_count += 1;
                value.info.row_entries = value
                    .info
                    .row_entries
                    .checked_add(band.height as usize)
                    .filter(|&n| n <= MAX_ROW_ENTRIES)
                    .ok_or(BoxMaskError::MaskLimit)?;
            }
        }
        value.info.cpu_payload_bytes = value
            .info
            .row_entries
            .checked_mul(size_of::<i32>())
            .and_then(|n| n.checked_add(edges))
            .ok_or(BoxMaskError::MaskLimit)?;
        value.info.packed_input_bytes = edges
            .checked_add(value.info.row_entries)
            .and_then(|n| n.checked_mul(size_of::<u32>()))
            .ok_or(BoxMaskError::MaskLimit)?;
        Ok(Some(value))
    }

    pub(crate) fn core_rect(&self) -> Rect {
        Rect {
            x: self.core.x as f32,
            y: self.core.y as f32,
            width: self.core.width as f32,
            height: self.core.height as f32,
        }
    }

    pub(crate) fn reserve(self, remaining: &mut u64) -> Result<BoxCoreReservation, BoxMaskError> {
        self.reserve_with(remaining, || {}, || {})
    }

    fn reserve_with(
        mut self,
        remaining: &mut u64,
        mut proof_query: impl FnMut(),
        mut classified: impl FnMut(),
    ) -> Result<BoxCoreReservation, BoxMaskError> {
        let p = self.info.candidate_pixels;
        let classification = p
            .checked_add(SETUP_WORK + 4)
            .ok_or(BoxMaskError::WorkLimit)?;
        debit(remaining, classification)?;
        let left = f64::from(self.core.x) + 0.125;
        let top = f64::from(self.core.y) + 0.125;
        let right = f64::from(self.core.x) + f64::from(self.core.width) - 0.125;
        let bottom = f64::from(self.core.y) + f64::from(self.core.height) - 0.125;
        let mut covered = true;
        // All four calls execute even on a defensive failure. Convexity of the
        // existing used rounded shape proves every interior sample from these
        // vertices; no radii or absolute sample positions change.
        for (x, y) in [(left, top), (right, top), (right, bottom), (left, bottom)] {
            proof_query();
            covered &= self.mask.outer.contains(x, y);
        }
        if !covered {
            return Err(BoxMaskError::MaskLimit);
        }
        let mut ambiguous = 0u64;
        for band in bands(self.candidate, self.core) {
            for y in band.y..band.y + band.height as i32 {
                for x in band.x..band.x + band.width as i32 {
                    classified();
                    ambiguous += u64::from(self.mask.classify(x, y) == CoverageClass::Partial);
                }
            }
        }
        let materialization = p
            .checked_mul(2)
            .and_then(|n| ambiguous.checked_mul(16).and_then(|a| n.checked_add(a)))
            .and_then(|n| n.checked_add(self.info.coverage_bytes as u64))
            .and_then(|n| n.checked_add(self.info.row_entries as u64))
            .ok_or(BoxMaskError::WorkLimit)?;
        debit(remaining, materialization)?;
        self.info.ambiguous_pixels = ambiguous;
        self.info.classification_work = classification;
        self.info.materialization_work = materialization;
        Ok(BoxCoreReservation { window: self })
    }
}

impl BoxCoreReservation {
    pub(crate) fn info(&self) -> BoxMaskInfo {
        self.window.info
    }
    pub(crate) fn core_rect(&self) -> Rect {
        self.window.core_rect()
    }
    pub(crate) fn tiles(&self) -> impl Iterator<Item = BoxMaskTileInfo> + '_ {
        self.window.partition.iter().flatten().copied()
    }
    pub(crate) fn materialize<E>(
        self,
        admit: impl FnOnce(&Self) -> Result<(), E>,
        error: impl Fn(BoxMaskError) -> E,
    ) -> Result<BoxCoreTiles, E> {
        self.materialize_with(admit, error, reserve_vec::<u8>, reserve_vec::<i32>, || {})
    }
    fn materialize_with<E>(
        self,
        admit: impl FnOnce(&Self) -> Result<(), E>,
        error: impl Fn(BoxMaskError) -> E,
        mut coverage_reserve: impl FnMut(usize) -> Result<Vec<u8>, BoxMaskError>,
        mut row_reserve: impl FnMut(usize) -> Result<Vec<i32>, BoxMaskError>,
        mut evaluated: impl FnMut(),
    ) -> Result<BoxCoreTiles, E> {
        admit(&self)?;
        debug_assert!(self.core_rect().width > 0.0 && self.core_rect().height > 0.0);
        let mut result = BoxCoreTiles {
            tiles: std::array::from_fn(|_| None),
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
        // No coverage evaluation until every coverage/row reserve succeeded.
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
impl BoxCoreTiles {
    pub(crate) fn into_tiles(self) -> impl Iterator<Item = BoxMaskTile> {
        self.tiles.into_iter().flatten()
    }
}

#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
