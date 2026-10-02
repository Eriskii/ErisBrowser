//! Whole-scene Native preparation. Phases share every resource ledger and one
//! font session, but each has an independent caller clip and fixed-coordinate
//! scope. Returned plans retain no borrowed scene or font/image leases.
use super::*;
use crate::graphics::{
    Fonts,
    text_masks::{self, MaskInfo, MaskKey, Session, TextRun},
};
use eris_raster_core::{
    MAX_MASK_COVERAGE_BYTES, MAX_NATIVE_PHASES, MAX_ROW_ENTRIES, PARAM_STRIDE, Phase, Profile,
    SourceMask,
    rounded::{RoundedTile, RoundedTiles, RoundedTilesCoverage, RoundedTilesDisposition},
    scope::CoordinateState,
};
use std::cell::Cell;

#[derive(Clone, Copy, Debug)]
pub struct NativePhase<'a> {
    pub frame: Frame,
    pub commands: &'a [DrawCommand],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeFallback {
    pub kind: FallbackKind,
    pub phase_index: Option<usize>,
    /// Original command within the indicated phase, never a lowered index.
    pub command_index: Option<usize>,
}
impl NativeFallback {
    fn new(kind: FallbackKind, phase: Option<usize>, command: Option<usize>) -> Self {
        Self {
            kind,
            phase_index: phase,
            command_index: command,
        }
    }
    fn leaf(error: FallbackReason, phase: Option<usize>) -> Self {
        Self::new(error.kind, phase, error.command_index)
    }
    pub fn category(&self) -> &'static str {
        FallbackReason::new(self.kind, self.command_index).category()
    }
}
impl std::fmt::Display for NativeFallback {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(self.category())?;
        if let Some(phase) = self.phase_index {
            write!(out, " at phase {phase}")?;
        }
        if let Some(command) = self.command_index {
            write!(out, " original command {command}")?;
        }
        Ok(())
    }
}
impl std::error::Error for NativeFallback {}
type NativeResult<T> = std::result::Result<T, NativeFallback>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSceneStats {
    pub phases: usize,
    pub bridge: BridgeStats,
    pub text_input_bytes: usize,
    pub text_input_scalars: usize,
    pub text: text_masks::Stats,
    pub rounded_masks: usize,
    pub rounded_loop_work: u64,
    pub mask_sources: usize,
    pub coverage_bytes: usize,
    pub row_entries: usize,
    pub cpu_pixel_upper_bound: u64,
    pub gpu_buffer_upper_bound: u64,
    /// Conservative simultaneous owned CPU storage, without a reference frame.
    /// Excludes allocator/driver overhead and trusted font outline allocations.
    pub preparation_peak_bytes: usize,
}

#[derive(Debug)]
pub struct NativeScenePlan {
    plan: Plan,
    stats: NativeSceneStats,
}
impl NativeScenePlan {
    pub fn plan(&self) -> &Plan {
        &self.plan
    }
    pub fn into_plan(self) -> Plan {
        self.plan
    }
    pub fn stats(&self) -> &NativeSceneStats {
        &self.stats
    }
    pub fn retained_cpu_bytes(&self) -> NativeResult<usize> {
        self.plan
            .retained_cpu_bytes()
            .ok()
            .and_then(|bytes| bytes.checked_sub(std::mem::size_of::<Plan>()))
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or_else(|| NativeFallback::new(FallbackKind::AllocationFailure, None, None))
    }
}

fn reserve_native<T>(count: usize) -> NativeResult<Vec<T>> {
    reserve(count).map_err(|error| NativeFallback::leaf(error, None))
}
fn at(kind: FallbackKind, phase: usize, command: usize) -> NativeFallback {
    NativeFallback::new(kind, Some(phase), Some(command))
}
fn font_error(error: text_masks::Error, phase: usize, command: usize) -> NativeFallback {
    use text_masks::Error;
    let kind = match error {
        Error::CpuPixels | Error::PixelAllowance => FallbackKind::CpuPaintBudget,
        Error::RunTextBytes | Error::FrameTextBytes | Error::FrameScalars => {
            FallbackKind::TextLimit
        }
        Error::Geometry | Error::RowIndex => FallbackKind::InvalidGeometry,
        Error::Allocation => FallbackKind::AllocationFailure,
        _ => FallbackKind::MaskPreparation,
    };
    at(kind, phase, command)
}

fn validate_native_frame(frame: Frame, phase: Option<usize>) -> NativeResult<()> {
    Profile::Native
        .validate_viewport(frame.width, frame.height)
        .map_err(|_| NativeFallback::new(FallbackKind::ViewportLimit, phase, None))?;
    if frame.clear > 0x00ff_ffff
        || ![
            frame.document_offset.0,
            frame.document_offset.1,
            frame.viewport_offset.0,
            frame.viewport_offset.1,
        ]
        .into_iter()
        .all(bounded_scalar)
    {
        return Err(NativeFallback::new(
            FallbackKind::InvalidGeometry,
            phase,
            None,
        ));
    }
    validate_rect(frame.caller_clip, None).map_err(|error| NativeFallback::leaf(error, phase))
}

#[derive(Default)]
struct InputCounts {
    original: usize,
    nontext: usize,
    text_bytes: usize,
    scalars: usize,
    command_key_bytes: usize,
    image_commands: usize,
}
fn preflight(target: Frame, phases: &[NativePhase<'_>]) -> NativeResult<InputCounts> {
    validate_native_frame(target, None)?;
    if target.caller_clip != Rect::new(0.0, 0.0, target.width as f32, target.height as f32)
        || target.document_offset != (0.0, 0.0)
        || target.viewport_offset != (0.0, 0.0)
    {
        return Err(NativeFallback::new(
            FallbackKind::InvalidGeometry,
            None,
            None,
        ));
    }
    if phases.len() > MAX_NATIVE_PHASES {
        return Err(NativeFallback::new(FallbackKind::CommandLimit, None, None));
    }
    let mut counts = InputCounts::default();
    for (phase, input) in phases.iter().enumerate() {
        validate_native_frame(input.frame, Some(phase))?;
        if (input.frame.width, input.frame.height, input.frame.clear)
            != (target.width, target.height, target.clear)
        {
            return Err(NativeFallback::new(
                FallbackKind::InvalidGeometry,
                Some(phase),
                None,
            ));
        }
        counts.original = counts
            .original
            .checked_add(input.commands.len())
            .filter(|&count| count <= MAX_COMMANDS)
            .ok_or_else(|| NativeFallback::new(FallbackKind::CommandLimit, Some(phase), None))?;
    }
    // Complete structural/input preflight precedes image table or mask allocation.
    for (phase, input) in phases.iter().enumerate() {
        let mut stack = [ScopeTag::Clip; MAX_SCOPES];
        let mut depth = 0;
        for (index, command) in input.commands.iter().enumerate() {
            let location = Some(index);
            let leaf = |error| NativeFallback::leaf(error, Some(phase));
            match command {
                DrawCommand::Rect { rect, radius, .. } => {
                    validate_rect(convert_rect(*rect), location).map_err(leaf)?;
                    if !bounded_scalar(*radius) {
                        return Err(at(FallbackKind::InvalidGeometry, phase, index));
                    }
                }
                DrawCommand::Image { rect, key } => {
                    validate_rect(convert_rect(*rect), location).map_err(leaf)?;
                    add_key(
                        key.len(),
                        &mut counts.command_key_bytes,
                        MAX_COMMAND_KEY_BYTES,
                        location,
                    )
                    .map_err(leaf)?;
                    counts.image_commands += 1;
                }
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    ..
                } => {
                    line_rect(*x1, *y1, *x2, *y2, *width, index).map_err(leaf)?;
                }
                DrawCommand::Text {
                    x, y, size, text, ..
                } => {
                    if ![*x, *y, *size].into_iter().all(bounded_scalar) {
                        return Err(at(FallbackKind::InvalidGeometry, phase, index));
                    }
                    counts.text_bytes = counts
                        .text_bytes
                        .checked_add(text.len())
                        .filter(|&bytes| {
                            bytes <= text_masks::MAX_FRAME_TEXT_BYTES
                                && text.len() <= text_masks::MAX_RUN_TEXT_BYTES
                        })
                        .ok_or_else(|| at(FallbackKind::TextLimit, phase, index))?;
                    counts.scalars = counts
                        .scalars
                        .checked_add(text.chars().count())
                        .filter(|&scalars| scalars <= text_masks::MAX_FRAME_SCALARS)
                        .ok_or_else(|| at(FallbackKind::TextLimit, phase, index))?;
                    continue;
                }
                DrawCommand::PushClip { rect } => {
                    validate_rect(convert_rect(*rect), location).map_err(leaf)?;
                    if depth == MAX_SCOPES {
                        return Err(at(FallbackKind::ScopeLimit, phase, index));
                    }
                    stack[depth] = ScopeTag::Clip;
                    depth += 1;
                }
                DrawCommand::PushFixed => {
                    if depth == MAX_SCOPES {
                        return Err(at(FallbackKind::ScopeLimit, phase, index));
                    }
                    stack[depth] = ScopeTag::Fixed;
                    depth += 1;
                }
                DrawCommand::PopClip | DrawCommand::PopFixed => {
                    if depth == 0 {
                        return Err(at(FallbackKind::InvalidScope, phase, index));
                    }
                    depth -= 1;
                    if !matches!(
                        (command, stack[depth]),
                        (DrawCommand::PopClip, ScopeTag::Clip)
                            | (DrawCommand::PopFixed, ScopeTag::Fixed)
                    ) {
                        return Err(at(FallbackKind::InvalidScope, phase, index));
                    }
                }
                DrawCommand::PushOpacity { .. } | DrawCommand::PopOpacity => {
                    return Err(at(FallbackKind::UnsupportedOpacity, phase, index));
                }
            }
            counts.nontext += 1;
        }
        if depth != 0 {
            return Err(NativeFallback::new(
                FallbackKind::InvalidScope,
                Some(phase),
                None,
            ));
        }
    }
    Ok(counts)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ledger {
    operations: usize,
    sources: usize,
    masks: usize,
    rows: usize,
    row_tables: usize,
    coverage: usize,
    image_bytes: usize,
    image_lut_words: usize,
    targets: u64,
    cpu_pixels: u64,
    cpu_limit: u64,
    gpu_upper: u64,
    rounded_loop_work: u64,
}
impl Ledger {
    fn check_gpu(&mut self, failure: NativeFallback) -> NativeResult<()> {
        self.gpu_upper = self
            .coverage
            .checked_add(self.rows)
            .and_then(|words| words.checked_add(self.image_lut_words))
            .and_then(|words| words.checked_mul(4))
            .and_then(|bytes| bytes.checked_add(self.image_bytes))
            .and_then(|bytes| {
                self.operations
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(PARAM_STRIDE))
                    .and_then(|parameters| bytes.checked_add(parameters))
            })
            .and_then(|bytes| self.targets.checked_add(bytes as u64))
            .filter(|&bytes| bytes <= Profile::Native.max_gpu_buffer_bytes())
            .ok_or(failure)?;
        Ok(())
    }
    fn mask(
        &mut self,
        area: usize,
        height: usize,
        fresh: bool,
        text: bool,
        failure: NativeFallback,
    ) -> NativeResult<()> {
        if text {
            self.operations = self
                .operations
                .checked_add(1)
                .filter(|&n| n <= MAX_COMMANDS)
                .ok_or(NativeFallback {
                    kind: FallbackKind::CommandLimit,
                    ..failure
                })?;
            self.cpu_pixels = self
                .cpu_pixels
                .checked_add(area.max(1) as u64)
                .filter(|&n| n <= self.cpu_limit)
                .ok_or(NativeFallback {
                    kind: FallbackKind::CpuPaintBudget,
                    ..failure
                })?;
        }
        self.rows = self
            .rows
            .checked_add(height)
            .filter(|&n| n <= MAX_ROW_ENTRIES)
            .ok_or(NativeFallback {
                kind: FallbackKind::GlyphRowLimit,
                ..failure
            })?;
        self.row_tables = self
            .row_tables
            .checked_add(1)
            .filter(|&n| n <= MAX_COMMANDS)
            .ok_or(NativeFallback {
                kind: FallbackKind::CommandLimit,
                ..failure
            })?;
        if fresh {
            self.sources = self
                .sources
                .checked_add(1)
                .filter(|&n| n <= MAX_SOURCE_ENTRIES)
                .ok_or(failure)?;
            self.masks += 1;
            self.coverage = self
                .coverage
                .checked_add(area)
                .filter(|&n| n <= MAX_MASK_COVERAGE_BYTES)
                .ok_or(failure)?;
        }
        self.check_gpu(NativeFallback {
            kind: FallbackKind::PlannerLimit,
            ..failure
        })
    }
    fn font(
        &mut self,
        info: MaskInfo,
        fresh: bool,
        phase: usize,
        command: usize,
    ) -> NativeResult<()> {
        self.mask(
            info.area(),
            info.height(),
            fresh,
            true,
            at(FallbackKind::MaskPreparation, phase, command),
        )
    }
    fn rounded(&mut self, shape: &RoundedTiles, phase: usize, command: usize) -> NativeResult<()> {
        let info = shape.info();
        // One original nontext command was already counted. Charge all extra
        // lowered operations and every tile's source/row/storage before the
        // group's first allocation. Failure leaves the caller's ledger intact.
        let mut next = *self;
        next.operations = next
            .operations
            .checked_add(info.tile_count - 1)
            .filter(|&n| n <= MAX_COMMANDS)
            .ok_or_else(|| at(FallbackKind::CommandLimit, phase, command))?;
        next.rounded_loop_work = next
            .rounded_loop_work
            .checked_add(info.loop_work)
            .ok_or_else(|| at(FallbackKind::CpuPaintBudget, phase, command))?;
        // Its complete loop work is covered by the original nontext area debit.
        for tile in shape.tiles() {
            next.mask(
                tile.coverage_bytes,
                tile.row_entries,
                true,
                false,
                at(FallbackKind::MaskPreparation, phase, command),
            )?;
        }
        *self = next;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum MaskSource {
    Font(usize),
    Rounded(usize),
}
#[derive(Clone, Copy)]
enum RowSource {
    Font(usize),
    Rounded(usize),
}

/// Closed structural bound available before scene/UI allocation. Includes the
/// final packed Plan overlapping preparation, one maximum font raster scratch,
/// 64 owned UI command slots and 16 KiB of UI strings. An optional packed CPU
/// reference is counted once. Borrowed snapshot/image/font assets, the existing
/// CPU font cache, trusted outline-library allocations and allocator overhead
/// are not charged as newly owned scene payload; this is not an RSS bound.
pub fn preparation_peak_bytes(target: Frame, include_reference: bool) -> NativeResult<usize> {
    use std::mem::{align_of, size_of};
    validate_native_frame(target, None)?;
    let failure = || NativeFallback::new(FallbackKind::AllocationFailure, None, None);
    // A second copy of the complete core structural allowance safely covers
    // the adapter's additional live CoordinateState and its bounded scope Vec.
    let core = eris_raster_core::native_planner_metadata_peak_bytes().map_err(|_| failure())?;
    let input_words = MAX_MASK_COVERAGE_BYTES
        .checked_add(MAX_ROW_ENTRIES)
        .and_then(|n| n.checked_add(Profile::Native.max_lut_entries()))
        .ok_or_else(failure)?;
    let input_bytes = input_words
        .checked_mul(4)
        .and_then(|n| n.checked_add(eris_raster_core::MAX_SOURCE_RGBA_BYTES))
        .ok_or_else(failure)?;
    // Session::Record has exactly MaskInfo + Vec<u8>; the sum plus both
    // alignments bounds layout padding without exposing that private record.
    let font_record = size_of::<MaskInfo>()
        + size_of::<Vec<u8>>()
        + align_of::<MaskInfo>()
        + align_of::<Vec<u8>>();
    let terms = [
        (2, core),
        (1, input_bytes),
        (MAX_COMMANDS + 1, PARAM_STRIDE),
        (1, MAX_MASK_COVERAGE_BYTES),
        (MAX_ROW_ENTRIES, size_of::<i32>()),
        (text_masks::MAX_MASK_AREA + 4, size_of::<f32>()),
        (text_masks::MAX_COLD_REQUESTS, font_record),
        (MAX_SOURCE_ENTRIES, size_of::<Entry<'_>>()),
        (MAX_SOURCE_ENTRIES, size_of::<&Arc<RasterImage>>()),
        (MAX_SOURCE_ENTRIES, size_of::<SourceImage<'_>>()),
        (MAX_COMMANDS, size_of::<Command>()),
        (MAX_COMMANDS, size_of::<Vec<i32>>()),
        (MAX_COMMANDS, size_of::<RowSource>()),
        (MAX_SOURCE_ENTRIES, size_of::<MaskSource>()),
        (MAX_SOURCE_ENTRIES, size_of::<(MaskKey, usize)>()),
        (MAX_COMMANDS, size_of::<RoundedTile>()),
        // Fixed Native partition metadata and the all-or-nothing pair of
        // owners coexist with the reserved destination Vec during handoff.
        (
            1,
            size_of::<RoundedTiles>() + size_of::<RoundedTilesCoverage>(),
        ),
        (MAX_SOURCE_ENTRIES * 2, size_of::<SourceMask<'_>>()),
        (MAX_COMMANDS, size_of::<&[i32]>()),
        (MAX_NATIVE_PHASES, size_of::<Phase<'_>>()),
        (MAX_COMMANDS, size_of::<Option<u32>>()),
        (MAX_NATIVE_PHASES, size_of::<(usize, usize)>()),
        (
            1,
            size_of::<Session<'_>>()
                + size_of::<Ledger>()
                + size_of::<InputCounts>()
                + size_of::<NativeScenePlan>(),
        ),
        // Headers for temporary vectors and the two caller-owned UI vectors.
        (16, size_of::<Vec<u8>>()),
        (64, size_of::<DrawCommand>()),
        (1, 16 * 1024),
    ];
    let mut bytes = terms.into_iter().try_fold(0usize, |total, (count, size)| {
        count
            .checked_mul(size)
            .and_then(|n| total.checked_add(n))
            .ok_or_else(failure)
    })?;
    if include_reference {
        bytes = (target.width as usize)
            .checked_mul(target.height as usize)
            .and_then(|area| area.checked_mul(size_of::<u32>()))
            .and_then(|reference| bytes.checked_add(reference))
            .ok_or_else(failure)?;
    }
    Ok(bytes)
}

pub fn plan_native_scene(
    target: Frame,
    phases: &[NativePhase<'_>],
    images: &ImageStore,
    fonts: &Fonts,
) -> NativeResult<NativeScenePlan> {
    let counts = preflight(target, phases)?;
    let preparation_peak_bytes = preparation_peak_bytes(target, false)?;
    if images.len() > MAX_SOURCE_ENTRIES || images.capacity() > MAX_STORE_CAPACITY {
        return Err(NativeFallback::new(
            FallbackKind::ImageStoreLimit,
            None,
            None,
        ));
    }
    let mut store_key_bytes = 0;
    for key in images.keys() {
        add_key(key.len(), &mut store_key_bytes, MAX_STORE_KEY_BYTES, None)
            .map_err(|error| NativeFallback::leaf(error, None))?;
    }
    key_work(
        images.len(),
        store_key_bytes,
        counts.command_key_bytes,
        counts.image_commands,
    )
    .map_err(|error| NativeFallback::leaf(error, None))?;
    let area = u64::from(target.width) * u64::from(target.height);
    let cpu_limit = (area * 16).clamp(1_000_000, 32_000_000);
    let cpu_nontext = area * counts.nontext as u64;
    if cpu_nontext > cpu_limit {
        return Err(NativeFallback::new(
            FallbackKind::CpuPaintBudget,
            None,
            None,
        ));
    }
    let mut entries = reserve_native(images.len())?;
    for (key, image) in images {
        entries.push((key.as_str(), image));
    }
    sort_entries(&mut entries);
    let mut unique = reserve_native(images.len())?;
    let mut image_ids = [None; MAX_COMMANDS];
    let mut ordinal = 0;
    let mut missing_images = 0;
    for phase in phases {
        for command in phase.commands {
            if let DrawCommand::Image { key, .. } = command {
                if let Some(image) = find_image(&entries, key) {
                    image_ids[ordinal] = Some(source_id(&mut unique, image));
                } else {
                    missing_images += 1;
                }
            }
            ordinal += 1;
        }
    }
    let referenced_sources = unique.len();
    for (_, image) in &entries {
        source_id(&mut unique, image);
    }
    let mut sources = reserve_native(unique.len())?;
    for image in &unique {
        sources.push(SourceImage {
            width: image.width,
            height: image.height,
            rgba: &image.rgba,
        });
    }
    let total_rgba_bytes = eris_raster_core::validate_source_images(&sources)
        .map_err(|_| NativeFallback::new(FallbackKind::InvalidImage, None, None))?
        * 4;
    let referenced_rgba_bytes = sources[..referenced_sources]
        .iter()
        .map(|s| s.rgba.len())
        .sum();
    let conversion = Profile::Native
        .conversion_buffer_bytes(target.width, target.height)
        .map_err(|_| NativeFallback::new(FallbackKind::PlannerLimit, None, None))?;
    let mut ledger = Ledger {
        operations: counts.nontext,
        sources: unique.len(),
        masks: 0,
        rows: 0,
        row_tables: 0,
        coverage: 0,
        image_bytes: total_rgba_bytes,
        image_lut_words: counts.image_commands * (target.width as usize + target.height as usize),
        targets: area * 4 + conversion,
        cpu_pixels: cpu_nontext,
        cpu_limit,
        gpu_upper: 0,
        rounded_loop_work: 0,
    };
    ledger.check_gpu(NativeFallback::new(FallbackKind::PlannerLimit, None, None))?;
    let mut session = Session::new(fonts, cpu_limit - cpu_nontext)
        .map_err(|_| NativeFallback::new(FallbackKind::CpuPaintBudget, None, None))?;
    let mut lowered = reserve_native(MAX_COMMANDS)?;
    let mut font_rows = reserve_native::<Vec<i32>>(MAX_COMMANDS)?;
    let mut row_sources = reserve_native(MAX_COMMANDS)?;
    let mut mask_sources = reserve_native(MAX_SOURCE_ENTRIES)?;
    let mut keys = reserve_native::<(MaskKey, usize)>(MAX_SOURCE_ENTRIES)?;
    let mut rounded = reserve_native::<RoundedTile>(MAX_COMMANDS)?;
    let mut ranges = [(0, 0); MAX_NATIVE_PHASES];
    ordinal = 0;
    for (phase_index, phase) in phases.iter().enumerate() {
        let mut state =
            CoordinateState::new_for_profile(Profile::Native, phase.frame).map_err(|_| {
                NativeFallback::new(FallbackKind::InvalidScope, Some(phase_index), None)
            })?;
        let start = lowered.len();
        for (index, command) in phase.commands.iter().enumerate() {
            let source_image = image_ids[ordinal];
            ordinal += 1;
            let lowered_command = match command {
                DrawCommand::Text {
                    x,
                    y,
                    size,
                    text,
                    color,
                    bold,
                    italic,
                    monospace,
                } => {
                    let offset = state.offset();
                    let clip = state.clip();
                    let run = TextRun {
                        x: *x + offset.0,
                        y: *y + offset.1,
                        text,
                        size: *size,
                        color: *color,
                        bold: *bold,
                        italic: *italic,
                        monospace: *monospace,
                    };
                    let clip = crate::graphics::Rect {
                        x: clip.x,
                        y: clip.y,
                        width: clip.width,
                        height: clip.height,
                    };
                    let refusal = Cell::new(None);
                    session
                        .visit(
                            run,
                            clip,
                            |info, _, fresh| {
                                ledger
                                    .font(info, fresh, phase_index, index)
                                    .map_err(|error| {
                                        refusal.set(Some(error));
                                    })
                            },
                            |mask, placement| {
                                let info = mask.info();
                                let id = match keys.iter().find(|(key, _)| *key == info.key()) {
                                    Some((_, id)) => *id,
                                    None => {
                                        let id = mask_sources.len();
                                        mask_sources.push(MaskSource::Font(keys.len()));
                                        keys.push((info.key(), id));
                                        id
                                    }
                                };
                                let mut rows = reserve_native(info.height()).map_err(|_| {
                                    refusal.set(Some(at(
                                        FallbackKind::AllocationFailure,
                                        phase_index,
                                        index,
                                    )));
                                })?;
                                for row in 0..info.height() {
                                    rows.push(placement.row_x(row).map_err(|_| {
                                        refusal.set(Some(at(
                                            FallbackKind::InvalidGeometry,
                                            phase_index,
                                            index,
                                        )));
                                    })?);
                                }
                                let color = placement.color();
                                lowered.push(Command::Glyph {
                                    source: id as u32,
                                    rows: row_sources.len() as u32,
                                    y: placement.origin_y(),
                                    rgba: [color.r, color.g, color.b, color.a],
                                });
                                row_sources.push(RowSource::Font(font_rows.len()));
                                font_rows.push(rows);
                                Ok(())
                            },
                        )
                        .map_err(|error| {
                            refusal
                                .get()
                                .unwrap_or_else(|| font_error(error, phase_index, index))
                        })?;
                    continue;
                }
                DrawCommand::Rect {
                    rect,
                    color,
                    radius,
                } => {
                    let rect = convert_rect(*rect);
                    let offset = state.offset();
                    let translated = Rect::new(
                        rect.x + offset.0,
                        rect.y + offset.1,
                        rect.width,
                        rect.height,
                    );
                    let disposition =
                        RoundedTiles::prepare_native(target, translated, state.clip(), *radius)
                            .map_err(|_| at(FallbackKind::MaskPreparation, phase_index, index))?;
                    match disposition {
                        RoundedTilesDisposition::Empty { .. } => continue,
                        RoundedTilesDisposition::ZeroRadius { .. } => Command::Rect {
                            rect,
                            rgba: [color.r, color.g, color.b, color.a],
                            radius: 0.0,
                        },
                        RoundedTilesDisposition::Tiles(shape) => {
                            if color.a == 0 {
                                continue;
                            }
                            let refusal = Cell::new(None);
                            let coverage = shape
                                .materialize(|shape| {
                                    ledger.rounded(shape, phase_index, index).map_err(|error| {
                                        refusal.set(Some(error));
                                        "native rounded preflight".to_owned()
                                    })
                                })
                                .map_err(|_| {
                                    refusal.get().unwrap_or_else(|| {
                                        at(FallbackKind::AllocationFailure, phase_index, index)
                                    })
                                })?;
                            // Consecutive disjoint strips preserve the original
                            // primitive's order and blend each pixel once.
                            for tile in coverage.into_tiles() {
                                lowered.push(Command::Glyph {
                                    source: mask_sources.len() as u32,
                                    rows: row_sources.len() as u32,
                                    y: tile.origin_y(),
                                    rgba: [color.r, color.g, color.b, color.a],
                                });
                                mask_sources.push(MaskSource::Rounded(rounded.len()));
                                row_sources.push(RowSource::Rounded(rounded.len()));
                                rounded.push(tile);
                            }
                            continue;
                        }
                    }
                }
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                    width,
                } => Command::Rect {
                    rect: line_rect(*x1, *y1, *x2, *y2, *width, index)
                        .map_err(|error| NativeFallback::leaf(error, Some(phase_index)))?,
                    rgba: [color.r, color.g, color.b, color.a],
                    radius: 0.0,
                },
                DrawCommand::Image { rect, .. } => {
                    let Some(source) = source_image else {
                        continue;
                    };
                    Command::Image {
                        rect: convert_rect(*rect),
                        source,
                    }
                }
                DrawCommand::PushClip { rect } => Command::PushClip(convert_rect(*rect)),
                DrawCommand::PopClip => Command::PopClip,
                DrawCommand::PushFixed => Command::PushFixed,
                DrawCommand::PopFixed => Command::PopFixed,
                DrawCommand::PushOpacity { .. } | DrawCommand::PopOpacity => {
                    unreachable!("preflight refused opacity")
                }
            };
            state
                .apply(&lowered_command)
                .map_err(|_| at(FallbackKind::InvalidScope, phase_index, index))?;
            lowered.push(lowered_command);
        }
        state.finish().map_err(|_| {
            NativeFallback::new(FallbackKind::InvalidScope, Some(phase_index), None)
        })?;
        ranges[phase_index] = (start, lowered.len());
    }
    let mut font_masks = reserve_native(keys.len())?;
    if session.masks().len() != keys.len() {
        return Err(NativeFallback::new(
            FallbackKind::MaskPreparation,
            None,
            None,
        ));
    }
    for (mask, (key, _)) in session.masks().zip(&keys) {
        let info = mask.info();
        if info.key() != *key {
            return Err(NativeFallback::new(
                FallbackKind::MaskPreparation,
                None,
                None,
            ));
        }
        font_masks.push(SourceMask {
            width: info.width() as u32,
            height: info.height() as u32,
            coverage: mask.coverage(),
        });
    }
    let mut masks = reserve_native(mask_sources.len())?;
    for source in &mask_sources {
        masks.push(match *source {
            MaskSource::Font(index) => font_masks[index],
            MaskSource::Rounded(index) => rounded[index].mask(),
        });
    }
    let mut rows = reserve_native(row_sources.len())?;
    for source in &row_sources {
        rows.push(match *source {
            RowSource::Font(index) => font_rows[index].as_slice(),
            RowSource::Rounded(index) => rounded[index].row_origins(),
        });
    }
    let mut core_phases = reserve_native(phases.len())?;
    for (phase, (start, end)) in phases.iter().zip(ranges) {
        core_phases.push(Phase {
            frame: phase.frame,
            commands: &lowered[start..end],
        });
    }
    let plan = eris_raster_core::plan_native_phases(target, &core_phases, &sources, &masks, &rows)
        .map_err(|_| NativeFallback::new(FallbackKind::PlannerLimit, None, None))?;
    if plan.gpu_buffer_bytes() > ledger.gpu_upper {
        return Err(NativeFallback::new(FallbackKind::PlannerLimit, None, None));
    }
    let stats = NativeSceneStats {
        phases: phases.len(),
        bridge: BridgeStats {
            original_commands: counts.original,
            lowered_commands: lowered.len(),
            image_store_entries: images.len(),
            unique_sources: unique.len(),
            referenced_sources,
            total_rgba_bytes,
            referenced_rgba_bytes,
            missing_images,
        },
        text_input_bytes: counts.text_bytes,
        text_input_scalars: counts.scalars,
        text: session.stats(),
        rounded_masks: rounded.len(),
        rounded_loop_work: ledger.rounded_loop_work,
        mask_sources: masks.len(),
        coverage_bytes: ledger.coverage,
        row_entries: ledger.rows,
        cpu_pixel_upper_bound: ledger.cpu_pixels,
        gpu_buffer_upper_bound: ledger.gpu_upper,
        preparation_peak_bytes,
    };
    Ok(NativeScenePlan { plan, stats })
}

#[cfg(test)]
mod tests;
