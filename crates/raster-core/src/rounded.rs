//! Exact Canvas rounded-rectangle geometry coverage, prepared on the CPU.
//!
//! This is coverage only, not a framebuffer or a text rasterizer. A caller may
//! pass the resulting mask and absolute row origins to the existing mask
//! kernel, which applies color and ordered integer blending on the GPU.
use crate::{
    Frame, MAX_COORDINATE, MAX_MASK_AXIS, MAX_MASK_COVERAGE_BYTES, MAX_MASK_PIXELS,
    MAX_ROW_ENTRIES, Profile, Rect, Result, SourceMask, reserved,
};

/// Storage for one prepared shape, excluding Vec headers and allocator
/// overhead. The caller must aggregate these amounts with all other sources,
/// temporary buffers, parameters and targets before materializing any shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundedInfo {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    /// Full Canvas loop area, including pixels later rejected by blend's clip.
    pub loop_work: u64,
    pub coverage_bytes: usize,
    pub row_entries: usize,
    /// Coverage bytes plus the CPU i32 row table.
    pub cpu_payload_bytes: usize,
    /// One u32 for every coverage cell and every row origin in the GPU arena.
    pub packed_input_bytes: usize,
}

/// Preparation never allocates coverage or row storage. A zero radius must
/// retain the caller's ordinary rectangle path: opaque Canvas rectangles do
/// not perform the rounded path's additional upper-clip test.
#[derive(Clone, Copy, Debug)]
pub enum RoundedDisposition {
    Empty { loop_work: u64 },
    ZeroRadius { loop_work: u64 },
    Shape(RoundedShape),
}
impl RoundedDisposition {
    pub fn loop_work(&self) -> u64 {
        match self {
            Self::Empty { loop_work } | Self::ZeroRadius { loop_work } => *loop_work,
            Self::Shape(shape) => shape.info.loop_work,
        }
    }
}

/// Immutable, validated geometry. No backing storage or source color is kept.
#[derive(Clone, Copy, Debug)]
pub struct RoundedShape {
    rect: Rect,
    radius: f32,
    info: RoundedInfo,
}

/// Owned coverage with absolute framebuffer placement. It contains no RGB.
#[derive(Debug)]
pub struct RoundedCoverage {
    info: RoundedInfo,
    coverage: Vec<u8>,
    rows: Vec<i32>,
}
impl RoundedCoverage {
    pub fn info(&self) -> RoundedInfo {
        self.info
    }
    pub fn mask(&self) -> SourceMask<'_> {
        SourceMask {
            width: self.info.width,
            height: self.info.height,
            coverage: &self.coverage,
        }
    }
    pub fn row_origins(&self) -> &[i32] {
        &self.rows
    }
    pub fn origin(&self) -> (i32, i32) {
        (self.info.origin_x, self.info.origin_y)
    }
    pub fn origin_y(&self) -> i32 {
        self.info.origin_y
    }
}

fn valid_geometry(rect: Rect) -> bool {
    [rect.x, rect.y]
        .into_iter()
        .all(|v| v.is_finite() && v.abs() <= 2.0 * MAX_COORDINATE)
        && [rect.width, rect.height]
            .into_iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
}

impl RoundedShape {
    /// `translated_rect` and `clip` are already in framebuffer coordinates;
    /// translation is never repeated here. Positions allow the sum of two
    /// admitted coordinates, while extents/radius retain the scalar cap.
    /// Only the frame dimensions are used; the caller owns scope validation.
    /// All supplied geometry is validated before empty geometry is omitted.
    pub fn prepare(
        profile: Profile,
        frame: Frame,
        translated_rect: Rect,
        clip: Rect,
        radius: f32,
    ) -> Result<RoundedDisposition> {
        Self::prepare_with_width(profile, frame, translated_rect, clip, radius, MAX_MASK_AXIS)
    }

    // Only the Native partition API may admit a wider temporary geometry.
    // It never exposes that shape as an individually materializable mask.
    fn prepare_with_width(
        profile: Profile,
        frame: Frame,
        translated_rect: Rect,
        clip: Rect,
        radius: f32,
        max_width: u32,
    ) -> Result<RoundedDisposition> {
        profile.validate_viewport(frame.width, frame.height)?;
        if !valid_geometry(translated_rect)
            || !valid_geometry(clip)
            || !radius.is_finite()
            || radius.abs() > MAX_COORDINATE
        {
            return Err("invalid or over-limit rounded geometry".into());
        }
        if translated_rect.width <= 0.0
            || translated_rect.height <= 0.0
            || clip.width <= 0.0
            || clip.height <= 0.0
        {
            return Ok(RoundedDisposition::Empty { loop_work: 0 });
        }
        // Preserve Canvas's f32 intersection and reconstructed endpoint order.
        // The scalar bounds above keep endpoints, differences and squared
        // distances finite, well below both the f32 and i32 ranges.
        let visible = translated_rect.intersect(clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return Ok(RoundedDisposition::Empty { loop_work: 0 });
        }
        let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as i32;
        let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as i32;
        let x1 = (visible.x + visible.width).ceil().min(frame.width as f32) as i32;
        let y1 = (visible.y + visible.height).ceil().min(frame.height as f32) as i32;
        let loop_work = u64::try_from((x1 - x0).max(0))
            .ok()
            .and_then(|w| w.checked_mul((y1 - y0).max(0) as u64))
            .ok_or("rounded loop work overflow")?;
        if loop_work == 0 {
            return Ok(RoundedDisposition::Empty { loop_work });
        }
        let radius = radius
            .max(0.0)
            .min(translated_rect.width / 2.0)
            .min(translated_rect.height / 2.0);
        if radius == 0.0 {
            return Ok(RoundedDisposition::ZeroRadius { loop_work });
        }
        // blend() tests integer origins against the half-open active clip.
        // Crop that rejection without changing the earlier loop-work debit.
        let right = x1.min((clip.x + clip.width).ceil() as i32);
        let bottom = y1.min((clip.y + clip.height).ceil() as i32);
        if right <= x0 || bottom <= y0 {
            return Ok(RoundedDisposition::Empty { loop_work });
        }
        let width = (right - x0) as u32;
        let height = (bottom - y0) as u32;
        if width > max_width || height > MAX_MASK_AXIS {
            return Err("rounded mask dimensions".into());
        }
        let coverage_bytes = (width as usize)
            .checked_mul(height as usize)
            .ok_or("rounded mask size overflow")?;
        if coverage_bytes > MAX_MASK_PIXELS || coverage_bytes > MAX_MASK_COVERAGE_BYTES {
            return Err("rounded mask coverage budget".into());
        }
        let row_entries = height as usize;
        if row_entries > MAX_ROW_ENTRIES {
            return Err("rounded row budget".into());
        }
        let cpu_payload_bytes = row_entries
            .checked_mul(std::mem::size_of::<i32>())
            .and_then(|rows| rows.checked_add(coverage_bytes))
            .ok_or("rounded CPU storage overflow")?;
        let packed_input_bytes = coverage_bytes
            .checked_add(row_entries)
            .and_then(|words| words.checked_mul(std::mem::size_of::<u32>()))
            .ok_or("rounded input storage overflow")?;
        Ok(RoundedDisposition::Shape(Self {
            rect: translated_rect,
            radius,
            info: RoundedInfo {
                origin_x: x0,
                origin_y: y0,
                width,
                height,
                loop_work,
                coverage_bytes,
                row_entries,
                cpu_payload_bytes,
                packed_input_bytes,
            },
        }))
    }

    pub fn info(&self) -> RoundedInfo {
        self.info
    }

    /// The caller's cumulative work/storage check runs before either fallible
    /// reserve and before any coverage evaluation. An allocation failure yields
    /// no partial mask. The callback must debit `loop_work`, not cropped area.
    pub fn materialize(
        &self,
        preflight: impl FnOnce(&RoundedShape) -> Result<()>,
    ) -> Result<RoundedCoverage> {
        self.materialize_with(preflight, reserved::<u8>, reserved::<i32>)
    }

    // The reserve seam lets private tests prove ordering without installing a
    // process-global allocator. Production supplies the shared fallible reserve.
    pub(super) fn materialize_with(
        &self,
        preflight: impl FnOnce(&RoundedShape) -> Result<()>,
        coverage_reserve: impl FnOnce(usize) -> Result<Vec<u8>>,
        row_reserve: impl FnOnce(usize) -> Result<Vec<i32>>,
    ) -> Result<RoundedCoverage> {
        preflight(self)?;
        let mut coverage = coverage_reserve(self.info.coverage_bytes)?;
        let mut rows = row_reserve(self.info.row_entries)?;
        // The shared reserve guarantees these invariants. Keep the internal
        // test seam from accidentally introducing an infallible growth path.
        if !coverage.is_empty()
            || coverage.capacity() < self.info.coverage_bytes
            || !rows.is_empty()
            || rows.capacity() < self.info.row_entries
        {
            return Err("invalid rounded reserve result".into());
        }
        rows.resize(self.info.row_entries, self.info.origin_x);
        for y in self.info.origin_y..self.info.origin_y + self.info.height as i32 {
            for x in self.info.origin_x..self.info.origin_x + self.info.width as i32 {
                coverage.push(self.coverage_at(x, y));
            }
        }
        Ok(RoundedCoverage {
            info: self.info,
            coverage,
            rows,
        })
    }

    fn coverage_at(&self, x: i32, y: i32) -> u8 {
        // Keep these f32 operations in the same order as Canvas::rect. Both
        // callers supply original framebuffer coordinates, never tile-local x/y.
        let px = x as f32 + 0.5;
        let py = y as f32 + 0.5;
        let dx = (self.rect.x + self.radius - px)
            .max(px - (self.rect.x + self.rect.width - self.radius))
            .max(0.0);
        let dy = (self.rect.y + self.radius - py)
            .max(py - (self.rect.y + self.rect.height - self.radius))
            .max(0.0);
        ((self.radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0) * 255.0) as u8
    }
}

/// Native-only partition metadata. Work belongs to the original primitive,
/// while storage and tile count cover all strips together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundedTilesInfo {
    pub loop_work: u64,
    pub tile_count: usize,
    pub coverage_bytes: usize,
    pub row_entries: usize,
    pub cpu_payload_bytes: usize,
    pub packed_input_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundedTileInfo {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    pub coverage_bytes: usize,
    pub row_entries: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum RoundedTilesDisposition {
    Empty { loop_work: u64 },
    ZeroRadius { loop_work: u64 },
    Tiles(RoundedTiles),
}

/// At most two disjoint horizontal strips. No source geometry is rebased or
/// re-intersected at a strip boundary. The legacy single-mask API is unchanged.
#[derive(Clone, Copy, Debug)]
pub struct RoundedTiles {
    shape: RoundedShape,
    info: RoundedTilesInfo,
}

#[derive(Debug)]
pub struct RoundedTile {
    info: RoundedTileInfo,
    coverage: Vec<u8>,
    rows: Vec<i32>,
}
impl RoundedTile {
    pub fn info(&self) -> RoundedTileInfo {
        self.info
    }
    pub fn mask(&self) -> SourceMask<'_> {
        SourceMask {
            width: self.info.width,
            height: self.info.height,
            coverage: &self.coverage,
        }
    }
    pub fn row_origins(&self) -> &[i32] {
        &self.rows
    }
    pub fn origin_y(&self) -> i32 {
        self.info.origin_y
    }
}

#[derive(Debug)]
pub struct RoundedTilesCoverage {
    tiles: [Option<RoundedTile>; 2],
}
impl RoundedTilesCoverage {
    pub fn into_tiles(self) -> impl Iterator<Item = RoundedTile> {
        self.tiles.into_iter().flatten()
    }
}

impl RoundedTiles {
    pub fn prepare_native(
        frame: Frame,
        translated_rect: Rect,
        clip: Rect,
        radius: f32,
    ) -> Result<RoundedTilesDisposition> {
        let shape = match RoundedShape::prepare_with_width(
            Profile::Native,
            frame,
            translated_rect,
            clip,
            radius,
            Profile::Native.max_width(),
        )? {
            RoundedDisposition::Empty { loop_work } => {
                return Ok(RoundedTilesDisposition::Empty { loop_work });
            }
            RoundedDisposition::ZeroRadius { loop_work } => {
                return Ok(RoundedTilesDisposition::ZeroRadius { loop_work });
            }
            RoundedDisposition::Shape(shape) => shape,
        };
        let original = shape.info();
        // Native viewport validation already bounds the original height. The
        // complete area check above still precedes any split: tiling does not
        // authorize more coverage than a single primitive/whole scene allows.
        let count = original.width.div_ceil(MAX_MASK_AXIS) as usize;
        if count > 2 {
            return Err("rounded Native partition budget".into());
        }
        let rows = original
            .row_entries
            .checked_mul(count)
            .filter(|&n| n <= MAX_ROW_ENTRIES)
            .ok_or("rounded partition row budget")?;
        let cpu_payload_bytes = rows
            .checked_mul(std::mem::size_of::<i32>())
            .and_then(|n| n.checked_add(original.coverage_bytes))
            .ok_or("rounded partition CPU storage overflow")?;
        let packed_input_bytes = original
            .coverage_bytes
            .checked_add(rows)
            .and_then(|n| n.checked_mul(std::mem::size_of::<u32>()))
            .ok_or("rounded partition input storage overflow")?;
        Ok(RoundedTilesDisposition::Tiles(Self {
            shape,
            info: RoundedTilesInfo {
                loop_work: original.loop_work,
                tile_count: count,
                coverage_bytes: original.coverage_bytes,
                row_entries: rows,
                cpu_payload_bytes,
                packed_input_bytes,
            },
        }))
    }

    pub fn info(&self) -> RoundedTilesInfo {
        self.info
    }
    pub fn tiles(&self) -> impl Iterator<Item = RoundedTileInfo> + '_ {
        let original = self.shape.info();
        (0..self.info.tile_count).map(move |index| {
            let column = index as u32 * MAX_MASK_AXIS;
            let width = (original.width - column).min(MAX_MASK_AXIS);
            RoundedTileInfo {
                origin_x: original.origin_x + column as i32,
                origin_y: original.origin_y,
                width,
                height: original.height,
                coverage_bytes: width as usize * original.height as usize,
                row_entries: original.row_entries,
            }
        })
    }

    /// One whole-group cumulative check runs before every reserve or coverage
    /// evaluation. All storage is reserved before evaluating the first cell.
    pub fn materialize(
        &self,
        preflight: impl FnOnce(&Self) -> Result<()>,
    ) -> Result<RoundedTilesCoverage> {
        self.materialize_with(preflight, reserved::<u8>, reserved::<i32>)
    }

    pub(super) fn materialize_with(
        &self,
        preflight: impl FnOnce(&Self) -> Result<()>,
        mut coverage_reserve: impl FnMut(usize) -> Result<Vec<u8>>,
        mut row_reserve: impl FnMut(usize) -> Result<Vec<i32>>,
    ) -> Result<RoundedTilesCoverage> {
        preflight(self)?;
        let mut tiles = [None, None];
        for (slot, info) in tiles.iter_mut().zip(self.tiles()) {
            let coverage = coverage_reserve(info.coverage_bytes)?;
            let rows = row_reserve(info.row_entries)?;
            if !coverage.is_empty()
                || coverage.capacity() < info.coverage_bytes
                || !rows.is_empty()
                || rows.capacity() < info.row_entries
            {
                return Err("invalid rounded partition reserve result".into());
            }
            *slot = Some(RoundedTile {
                info,
                coverage,
                rows,
            });
        }
        for tile in tiles.iter_mut().flatten() {
            tile.rows.resize(tile.info.row_entries, tile.info.origin_x);
            for y in tile.info.origin_y..tile.info.origin_y + tile.info.height as i32 {
                for x in tile.info.origin_x..tile.info.origin_x + tile.info.width as i32 {
                    tile.coverage.push(self.shape.coverage_at(x, y));
                }
            }
        }
        Ok(RoundedTilesCoverage { tiles })
    }
}
