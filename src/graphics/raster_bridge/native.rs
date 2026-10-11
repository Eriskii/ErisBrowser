//! Whole-scene Native preparation. Phases share every resource ledger and one
//! font session, but each has an independent caller clip and fixed-coordinate
//! scope. Returned plans retain no borrowed scene or font/image leases.
use super::*;
use crate::graphics::{
    Fonts, Rect as BrowserRect,
    box_decoration::{
        BoxMaskError, BoxMaskReservation, BoxMaskTile, BoxMaskTiles, BoxMaskWindow, BoxPaint,
    },
    gradient::{OpaqueGradient, OpaqueGradientError, OpaqueGradientInfo},
    text_masks::{self, MaskInfo, MaskKey, Session, TextRun},
};
use eris_raster_core::{
    BackgroundImageGeometry, MAX_MASK_COVERAGE_BYTES, MAX_NATIVE_PHASES, MAX_ROW_ENTRIES,
    PARAM_STRIDE, Phase, Profile, SourceMask,
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
    /// CPU-prepared sources, separate from the loaded ImageStore statistics.
    pub generated_gradient_sources: usize,
    pub generated_gradient_bytes: usize,
    pub gradient_evaluation_work: u64,
    pub gradient_preparation_work: u64,
    pub text_input_bytes: usize,
    pub text_input_scalars: usize,
    pub text: text_masks::Stats,
    pub rounded_masks: usize,
    pub rounded_loop_work: u64,
    pub box_masks: usize,
    /// Eligible Color box commands lowered without coverage or row payloads.
    pub box_rectangles: usize,
    pub box_loop_work: u64,
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

fn gradient_error(error: OpaqueGradientError, phase: usize, command: usize) -> NativeFallback {
    let kind = match error {
        OpaqueGradientError::Unsupported => FallbackKind::UnsupportedGradient,
        OpaqueGradientError::Geometry => FallbackKind::InvalidGeometry,
        OpaqueGradientError::Allocation => FallbackKind::AllocationFailure,
    };
    at(kind, phase, command)
}
fn browser_rect(rect: Rect) -> BrowserRect {
    BrowserRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn box_error(error: BoxMaskError, phase: usize, command: usize) -> NativeFallback {
    let kind = match error {
        BoxMaskError::InvalidGeometry => FallbackKind::InvalidGeometry,
        BoxMaskError::MaskLimit => FallbackKind::MaskPreparation,
        BoxMaskError::WorkLimit => FallbackKind::CpuPaintBudget,
        BoxMaskError::AllocationFailure => FallbackKind::AllocationFailure,
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
    full_area_commands: usize,
    text_bytes: usize,
    scalars: usize,
    command_key_bytes: usize,
    image_commands: usize,
}

#[derive(Clone, Copy)]
enum NativeScopeTag {
    Clip,
    Fixed,
    Opacity,
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
        let mut stack = [NativeScopeTag::Clip; MAX_SCOPES];
        let mut depth = 0;
        for (index, command) in input.commands.iter().enumerate() {
            let location = Some(index);
            let leaf = |error| NativeFallback::leaf(error, Some(phase));
            match command {
                DrawCommand::BoxDecoration(decoration) => {
                    validate_box_shape(&decoration.shape, location).map_err(leaf)?;
                    if let Some(cutout) = &decoration.cutout {
                        validate_box_shape(cutout, location).map_err(leaf)?;
                    }
                    // Unsupported paints remain a complete-scene refusal before
                    // source or mask preparation, even when hidden/suppressed.
                    if !matches!(decoration.paint, BoxPaint::Color(_)) {
                        return Err(at(FallbackKind::UnsupportedBoxDecoration, phase, index));
                    }
                }
                DrawCommand::BackgroundImage {
                    rect,
                    tile,
                    radius,
                    key,
                    ..
                } => {
                    if !bounded_scalar(*radius) {
                        return Err(at(FallbackKind::InvalidGeometry, phase, index));
                    }
                    // Rounded coverage is not separable into X/Y lookup tables.
                    // Refuse even missing/offscreen/suppressed commands before
                    // any image table, font mask or source payload preparation.
                    if *radius != 0.0 {
                        return Err(at(FallbackKind::UnsupportedBackgroundImage, phase, index));
                    }
                    validate_rect(convert_rect(*rect), location).map_err(leaf)?;
                    validate_rect(convert_rect(*tile), location).map_err(leaf)?;
                    if tile.width <= 0.0 || tile.height <= 0.0 {
                        return Err(at(FallbackKind::InvalidGeometry, phase, index));
                    }
                    add_key(
                        key.len(),
                        &mut counts.command_key_bytes,
                        MAX_COMMAND_KEY_BYTES,
                        location,
                    )
                    .map_err(leaf)?;
                    counts.image_commands += 1;
                }
                DrawCommand::LinearGradient {
                    rect,
                    tile,
                    repeat,
                    radius,
                    gradient,
                    current_color,
                    stop_scale,
                } => {
                    if !bounded_scalar(*radius) {
                        return Err(at(FallbackKind::InvalidGeometry, phase, index));
                    }
                    // Even hidden gradients refuse rounded shapes and every
                    // enclosing opacity scope: RGB8 preparation must not precede
                    // the software painter's fractional-channel group blending.
                    if *radius != 0.0
                        || stack[..depth]
                            .iter()
                            .any(|tag| matches!(tag, NativeScopeTag::Opacity))
                    {
                        return Err(at(FallbackKind::UnsupportedGradient, phase, index));
                    }
                    validate_rect(convert_rect(*rect), location).map_err(leaf)?;
                    validate_rect(convert_rect(*tile), location).map_err(leaf)?;
                    // Structural validation is bounded by 32 stops. The helper
                    // checks alpha before its empty-window return and allocates
                    // neither a resolved stop array nor source pixels here.
                    OpaqueGradient::new_with_repeat(
                        target.width,
                        target.height,
                        *rect,
                        *tile,
                        browser_rect(target.caller_clip),
                        gradient,
                        *current_color,
                        *stop_scale,
                        *repeat,
                    )
                    .map_err(|error| gradient_error(error, phase, index))?;
                }
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
                    stack[depth] = NativeScopeTag::Clip;
                    depth += 1;
                }
                DrawCommand::PushFixed => {
                    if depth == MAX_SCOPES {
                        return Err(at(FallbackKind::ScopeLimit, phase, index));
                    }
                    stack[depth] = NativeScopeTag::Fixed;
                    depth += 1;
                }
                DrawCommand::PushOpacity { opacity } => {
                    if !opacity.is_finite() || !(0.0..=1.0).contains(opacity) {
                        return Err(at(FallbackKind::InvalidGeometry, phase, index));
                    }
                    if depth == MAX_SCOPES {
                        return Err(at(FallbackKind::ScopeLimit, phase, index));
                    }
                    stack[depth] = NativeScopeTag::Opacity;
                    depth += 1;
                }
                DrawCommand::PopClip | DrawCommand::PopFixed | DrawCommand::PopOpacity => {
                    if depth == 0 {
                        return Err(at(FallbackKind::InvalidScope, phase, index));
                    }
                    depth -= 1;
                    if !matches!(
                        (command, stack[depth]),
                        (DrawCommand::PopClip, NativeScopeTag::Clip)
                            | (DrawCommand::PopFixed, NativeScopeTag::Fixed)
                            | (DrawCommand::PopOpacity, NativeScopeTag::Opacity)
                    ) {
                        return Err(at(FallbackKind::InvalidScope, phase, index));
                    }
                }
            }
            counts.nontext += 1;
            // Preserve every operation/parameter slot. Rects and Images get a
            // separate pre-blend geometry reservation after this complete
            // preflight; Line and opacity retain their full-frame allowance.
            if !matches!(
                command,
                DrawCommand::Rect { .. }
                    | DrawCommand::Image { .. }
                    | DrawCommand::BackgroundImage { .. }
                    | DrawCommand::LinearGradient { .. }
                    | DrawCommand::BoxDecoration(_)
                    | DrawCommand::PushClip { .. }
                    | DrawCommand::PopClip
                    | DrawCommand::PushFixed
                    | DrawCommand::PopFixed
            ) {
                counts.full_area_commands += 1;
            }
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

/// Canvas::image's destination loop, before blend rejects fractional clip
/// edges. Inputs come from validated commands and the current coordinate state;
/// neither source-image lookup nor opacity/alpha inspection changes this bound.
fn image_loop_work(target: Frame, rect: BrowserRect, clip: Rect) -> u64 {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return 0;
    }
    let visible = rect.intersect(BrowserRect {
        x: clip.x,
        y: clip.y,
        width: clip.width,
        height: clip.height,
    });
    if visible.width <= 0.0 || visible.height <= 0.0 {
        return 0;
    }
    // Unlike Rect, Image does not raise its lower loop bound to ceil(clip).
    // Those extra sampled pixels still consume work even if blend clips them.
    let x0 = visible.x.floor().max(0.0) as i32;
    let y0 = visible.y.floor().max(0.0) as i32;
    let x1 = (visible.x + visible.width).ceil().min(target.width as f32) as i32;
    let y1 = (visible.y + visible.height)
        .ceil()
        .min(target.height as f32) as i32;
    (x1 - x0).max(0) as u64 * (y1 - y0).max(0) as u64
}

// Fixed metadata indexed by original command, never allocated per gradient.
struct GradientReservation {
    windows: [Option<OpaqueGradientInfo>; MAX_COMMANDS],
    sources: usize,
    bytes: usize,
    evaluation: u64,
    packing: u64,
}
impl GradientReservation {
    fn new() -> Self {
        Self {
            windows: [None; MAX_COMMANDS],
            sources: 0,
            bytes: 0,
            evaluation: 0,
            packing: 0,
        }
    }
}

// The route retains only funded metadata until lowering. A rectangle carries
// the conservative software-reference tariff, never an unfunded mask scan.
// Keep the existing bounded mask metadata inline; boxing would add a separate
// fallible allocation for every retained command.
#[allow(clippy::large_enum_variant)]
enum BoxReservation {
    Rectangle { work: u64 },
    Mask(BoxMaskReservation),
}

/// Called only after complete original-input validation. No coverage, glyphs
/// or source payloads are allocated. The fixed window table is caller-owned;
/// one bounded CoordinateState scope Vec is live and dropped per phase.
fn geometry_pixel_work(
    target: Frame,
    phases: &[NativePhase<'_>],
    gradients: &mut GradientReservation,
    boxes: &mut [Option<BoxReservation>],
    remaining: &mut u64,
) -> NativeResult<u64> {
    let initial = *remaining;
    let mut ordinal = 0;
    for (phase, input) in phases.iter().enumerate() {
        if !input.commands.iter().any(|command| {
            matches!(
                command,
                DrawCommand::Rect { .. }
                    | DrawCommand::Image { .. }
                    | DrawCommand::BackgroundImage { .. }
                    | DrawCommand::LinearGradient { .. }
                    | DrawCommand::BoxDecoration(_)
            )
        }) {
            ordinal += input.commands.len();
            continue;
        }
        // Frame/range validation is already complete; the only remaining
        // failure in this constructor is its fallible MAX_SCOPES reservation.
        let mut state = CoordinateState::new_for_profile(Profile::Native, input.frame)
            .map_err(|_| NativeFallback::new(FallbackKind::AllocationFailure, Some(phase), None))?;
        for (index, command) in input.commands.iter().enumerate() {
            let slot = ordinal;
            ordinal += 1;
            let scope = match command {
                DrawCommand::BoxDecoration(decoration) => {
                    let offset = state.offset();
                    let outer = decoration.shape.translated(offset.0, offset.1);
                    let cutout = decoration
                        .cutout
                        .map(|shape| shape.translated(offset.0, offset.1));
                    let clip = browser_rect(state.clip());
                    let direct =
                        BoxMaskWindow::rect_work(target.width, target.height, outer, cutout, clip)
                            .map_err(|error| box_error(error, phase, index))?;
                    boxes[slot] = Some(if let Some(work) = direct {
                        // Exact all-covered candidates need no mask scan or
                        // payload. Retain Canvas's conservative 8 + 3P debit,
                        // including transparent and suppressed commands.
                        *remaining = remaining
                            .checked_sub(work)
                            .ok_or_else(|| at(FallbackKind::CpuPaintBudget, phase, index))?;
                        BoxReservation::Rectangle { work }
                    } else {
                        let window =
                            BoxMaskWindow::new(target.width, target.height, outer, cutout, clip)
                                .map_err(|error| box_error(error, phase, index))?;
                        BoxReservation::Mask(
                            window
                                .reserve(remaining)
                                .map_err(|error| box_error(error, phase, index))?,
                        )
                    });
                    continue;
                }
                DrawCommand::BackgroundImage {
                    rect, tile, repeat, ..
                } => {
                    let geometry = BackgroundImageGeometry::new(
                        target,
                        convert_rect(*rect),
                        convert_rect(*tile),
                        *repeat,
                        &state,
                    )
                    .map_err(|_| at(FallbackKind::InvalidGeometry, phase, index))?;
                    // Preserve Canvas's full candidate tariff and prepay the
                    // extra per-axis LUT work before source packing. Missing,
                    // transparent and suppressed sources keep this conservative
                    // upper bound even if no actual LUT is later emitted.
                    let work = geometry
                        .loop_work
                        .checked_add(geometry.lut_work)
                        .ok_or_else(|| at(FallbackKind::CpuPaintBudget, phase, index))?;
                    charge_geometry(remaining, work)?;
                    continue;
                }
                DrawCommand::Rect { rect, .. } => {
                    let offset = state.offset();
                    let translated = Rect::new(
                        rect.x + offset.0,
                        rect.y + offset.1,
                        rect.width,
                        rect.height,
                    );
                    // Radius does not affect Canvas's earlier loop-work debit.
                    // Literal zero obtains that bound without rounded-mask caps
                    // or allocation. The real radius still controls lowering.
                    let disposition =
                        RoundedTiles::prepare_native(target, translated, state.clip(), 0.0)
                            .map_err(|_| at(FallbackKind::InvalidGeometry, phase, index))?;
                    let work = match disposition {
                        RoundedTilesDisposition::Empty { loop_work }
                        | RoundedTilesDisposition::ZeroRadius { loop_work } => loop_work,
                        RoundedTilesDisposition::Tiles(shape) => shape.info().loop_work,
                    };
                    // Keep this conservative charge for transparent colors and
                    // zero-opacity descendants; geometry alone determines it.
                    charge_geometry(remaining, work)?;
                    continue;
                }
                DrawCommand::Image { rect, .. } => {
                    let offset = state.offset();
                    let work =
                        image_loop_work(target, rect.translated(offset.0, offset.1), state.clip());
                    // Retain a geometric upper bound for missing sources,
                    // transparent samples and zero-opacity descendants too.
                    charge_geometry(remaining, work)?;
                    continue;
                }
                DrawCommand::PushClip { rect } => Command::PushClip(convert_rect(*rect)),
                DrawCommand::PopClip => Command::PopClip,
                DrawCommand::PushFixed => Command::PushFixed,
                DrawCommand::PopFixed => Command::PopFixed,
                DrawCommand::PushOpacity { opacity } => Command::PushOpacity(*opacity),
                DrawCommand::PopOpacity => Command::PopOpacity,
                DrawCommand::Text { .. } | DrawCommand::Line { .. } => continue,
                DrawCommand::LinearGradient {
                    rect,
                    tile,
                    repeat,
                    gradient,
                    current_color,
                    stop_scale,
                    ..
                } => {
                    let offset = state.offset();
                    let Some(prepared) = OpaqueGradient::new_with_repeat(
                        target.width,
                        target.height,
                        rect.translated(offset.0, offset.1),
                        tile.translated(offset.0, offset.1),
                        browser_rect(state.clip()),
                        gradient,
                        *current_color,
                        *stop_scale,
                        *repeat,
                    )
                    .map_err(|error| gradient_error(error, phase, index))?
                    else {
                        continue;
                    };
                    let info = prepared.info();
                    let packing = u64::from(info.width) * u64::from(info.height);
                    let failure = at(FallbackKind::CpuPaintBudget, phase, index);
                    let work = info.loop_work.checked_add(packing).ok_or(failure)?;
                    charge_geometry(remaining, work)?;
                    gradients.evaluation = gradients
                        .evaluation
                        .checked_add(info.loop_work)
                        .ok_or(failure)?;
                    gradients.packing = gradients.packing.checked_add(packing).ok_or(failure)?;
                    gradients.bytes = gradients
                        .bytes
                        .checked_add(info.rgba_bytes())
                        .ok_or(failure)?;
                    gradients.sources += 1;
                    gradients.windows[slot] = Some(info);
                    continue;
                }
            };
            state
                .apply(&scope)
                .map_err(|_| at(FallbackKind::InvalidScope, phase, index))?;
        }
        state
            .finish()
            .map_err(|_| NativeFallback::new(FallbackKind::InvalidScope, Some(phase), None))?;
    }
    Ok(initial - *remaining)
}

fn charge_geometry(remaining: &mut u64, work: u64) -> NativeResult<()> {
    // Preserve the old aggregate-budget diagnostic for existing primitives,
    // while refusing before any later box classification scan.
    *remaining = remaining
        .checked_sub(work)
        .ok_or_else(|| NativeFallback::new(FallbackKind::CpuPaintBudget, None, None))?;
    Ok(())
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
    box_loop_work: u64,
}
impl Ledger {
    // One generated window is one source and exactly width+height LUT words.
    // Commit only after all shared limits and the GPU byte bound admit it.
    fn gradient(&mut self, width: u32, height: u32, failure: NativeFallback) -> NativeResult<()> {
        let mut next = *self;
        next.sources = next
            .sources
            .checked_add(1)
            .filter(|&n| n <= MAX_SOURCE_ENTRIES)
            .ok_or(failure)?;
        let bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or(failure)?;
        next.image_bytes = next
            .image_bytes
            .checked_add(bytes)
            .filter(|&n| n <= eris_raster_core::MAX_SOURCE_RGBA_BYTES)
            .ok_or(failure)?;
        next.image_lut_words = next
            .image_lut_words
            .checked_add(width as usize)
            .and_then(|n| n.checked_add(height as usize))
            .filter(|&n| n <= Profile::Native.max_lut_entries())
            .ok_or(failure)?;
        next.check_gpu(failure)?;
        *self = next;
        Ok(())
    }
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
        // Its complete loop work was included in the earlier Rect reservation.
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
    fn box_mask(
        &mut self,
        shape: &BoxMaskReservation,
        phase: usize,
        command: usize,
    ) -> NativeResult<()> {
        let info = shape.info();
        let mut next = *self;
        next.operations = next
            .operations
            .checked_add(info.tile_count.saturating_sub(1))
            .filter(|&n| n <= MAX_COMMANDS)
            .ok_or_else(|| at(FallbackKind::CommandLimit, phase, command))?;
        next.box_loop_work = next
            .box_loop_work
            .checked_add(info.classification_work)
            .and_then(|n| n.checked_add(info.materialization_work))
            .ok_or_else(|| at(FallbackKind::CpuPaintBudget, phase, command))?;
        // Both CPU work stages were prepaid by the original-order geometry
        // scan. Admit every partition's shared resources before allocation.
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
    Box(usize),
}
#[derive(Clone, Copy)]
enum RowSource {
    Font(usize),
    Rounded(usize),
    Box(usize),
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
    // Geometry reservation and later lowering use that state at disjoint times.
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
        // Owned generated RGBA coexists with the final packed core input.
        (1, eris_raster_core::MAX_SOURCE_RGBA_BYTES),
        (MAX_SOURCE_ENTRIES, size_of::<RasterImage>()),
        (2, size_of::<GradientReservation>()),
        (1, size_of::<OpaqueGradient<'_>>()),
        // Background geometry is stack metadata, never a generated raster.
        // Bound both return-value/caller copies during the charged prepass.
        (2, size_of::<BackgroundImageGeometry>()),
        // ResolvedGradient: <=32 five-f64 stops plus bounded geometry/header.
        (32, 5 * size_of::<f64>()),
        (1, 256),
        (MAX_COMMANDS + 1, PARAM_STRIDE),
        (1, MAX_MASK_COVERAGE_BYTES),
        (MAX_ROW_ENTRIES, size_of::<i32>()),
        (text_masks::MAX_MASK_AREA + 4, size_of::<f32>()),
        (text_masks::MAX_COLD_REQUESTS, font_record),
        (MAX_SOURCE_ENTRIES, size_of::<Entry<'_>>()),
        (MAX_SOURCE_ENTRIES, size_of::<&Arc<RasterImage>>()),
        // reserve_exact may hold the old source-view table while allocating
        // its loaded+generated replacement. Bound both full tables.
        (MAX_SOURCE_ENTRIES * 2, size_of::<SourceImage<'_>>()),
        (MAX_COMMANDS, size_of::<Command>()),
        (MAX_COMMANDS, size_of::<Vec<i32>>()),
        (MAX_COMMANDS, size_of::<RowSource>()),
        (MAX_SOURCE_ENTRIES, size_of::<MaskSource>()),
        (MAX_SOURCE_ENTRIES, size_of::<(MaskKey, usize)>()),
        (MAX_COMMANDS, size_of::<RoundedTile>()),
        (MAX_COMMANDS, size_of::<Option<BoxReservation>>()),
        (MAX_COMMANDS, size_of::<BoxMaskTile>()),
        (2, size_of::<BoxMaskWindow>() + size_of::<BoxReservation>()),
        (1, size_of::<BoxMaskTiles>()),
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
        (19, size_of::<Vec<u8>>()),
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
    let mut gradients = GradientReservation::new();
    let full_area_work = area
        .checked_mul(counts.full_area_commands as u64)
        .ok_or_else(|| NativeFallback::new(FallbackKind::CpuPaintBudget, None, None))?;
    let mut remaining = cpu_limit
        .checked_sub(full_area_work)
        .ok_or_else(|| NativeFallback::new(FallbackKind::CpuPaintBudget, None, None))?;
    let mut boxes = reserve_native::<Option<BoxReservation>>(counts.original)?;
    boxes.resize_with(counts.original, || None);
    let geometry_work =
        geometry_pixel_work(target, phases, &mut gradients, &mut boxes, &mut remaining)?;
    let cpu_nontext = full_area_work + geometry_work;
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
            if let DrawCommand::Image { key, .. } | DrawCommand::BackgroundImage { key, .. } =
                command
            {
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
        box_loop_work: 0,
    };
    ledger.check_gpu(NativeFallback::new(FallbackKind::PlannerLimit, None, None))?;
    // Reserve every generated source before the first materialization, sharing
    // loaded-image and later font/rounded source allowances. No payload exists
    // yet; a late admission failure drops metadata only.
    let mut gradient_ordinal = 0;
    for (phase, input) in phases.iter().enumerate() {
        for index in 0..input.commands.len() {
            if let Some(info) = gradients.windows[gradient_ordinal] {
                ledger.gradient(
                    info.width,
                    info.height,
                    at(FallbackKind::PlannerLimit, phase, index),
                )?;
            }
            gradient_ordinal += 1;
        }
    }
    sources
        .try_reserve_exact(gradients.sources)
        .map_err(|_| NativeFallback::new(FallbackKind::AllocationFailure, None, None))?;
    let mut generated = reserve_native::<RasterImage>(gradients.sources)?;
    let mut session = Session::new(fonts, cpu_limit - cpu_nontext)
        .map_err(|_| NativeFallback::new(FallbackKind::CpuPaintBudget, None, None))?;
    let mut lowered = reserve_native(MAX_COMMANDS)?;
    let mut font_rows = reserve_native::<Vec<i32>>(MAX_COMMANDS)?;
    let mut row_sources = reserve_native(MAX_COMMANDS)?;
    let mut mask_sources = reserve_native(MAX_SOURCE_ENTRIES)?;
    let mut keys = reserve_native::<(MaskKey, usize)>(MAX_SOURCE_ENTRIES)?;
    let mut rounded = reserve_native::<RoundedTile>(MAX_COMMANDS)?;
    let mut box_tiles = reserve_native::<BoxMaskTile>(MAX_COMMANDS)?;
    let mut box_rectangles = 0;
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
            let gradient_window = gradients.windows[ordinal];
            let box_window = boxes[ordinal].take();
            ordinal += 1;
            let lowered_command = match command {
                DrawCommand::BoxDecoration(decoration) => {
                    let BoxPaint::Color(color) = decoration.paint else {
                        unreachable!("complete preflight refused unsupported box paint");
                    };
                    let reservation = box_window
                        .ok_or_else(|| at(FallbackKind::MaskPreparation, phase_index, index))?;
                    match reservation {
                        BoxReservation::Rectangle { work } => {
                            ledger.box_loop_work =
                                ledger.box_loop_work.checked_add(work).ok_or_else(|| {
                                    at(FallbackKind::CpuPaintBudget, phase_index, index)
                                })?;
                            box_rectangles += 1;
                            // The proof used framebuffer geometry, but core
                            // must apply its current phase/fixed offset once.
                            Command::Rect {
                                rect: convert_rect(decoration.shape.rect),
                                rgba: [color.r, color.g, color.b, color.a],
                                radius: 0.0,
                            }
                        }
                        BoxReservation::Mask(reservation) => {
                            let coverage = reservation.materialize(
                                |shape| ledger.box_mask(shape, phase_index, index),
                                |error| box_error(error, phase_index, index),
                            )?;
                            for tile in coverage.into_tiles() {
                                lowered.push(Command::Glyph {
                                    source: mask_sources.len() as u32,
                                    rows: row_sources.len() as u32,
                                    y: tile.info().origin_y,
                                    rgba: [color.r, color.g, color.b, color.a],
                                });
                                mask_sources.push(MaskSource::Box(box_tiles.len()));
                                row_sources.push(RowSource::Box(box_tiles.len()));
                                box_tiles.push(tile);
                            }
                            continue;
                        }
                    }
                }
                DrawCommand::BackgroundImage {
                    rect, tile, repeat, ..
                } => {
                    let Some(source) = source_image else {
                        continue;
                    };
                    // Core applies the current phase/fixed offset once. Keep
                    // the raw tile phase, separate from the paint outline.
                    Command::BackgroundImage {
                        rect: convert_rect(*rect),
                        tile: convert_rect(*tile),
                        repeat: *repeat,
                        source,
                    }
                }
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
                DrawCommand::PushOpacity { opacity } => Command::PushOpacity(*opacity),
                DrawCommand::PopOpacity => Command::PopOpacity,
                DrawCommand::LinearGradient {
                    rect,
                    tile,
                    repeat,
                    gradient,
                    current_color,
                    stop_scale,
                    ..
                } => {
                    let Some(info) = gradient_window else {
                        continue;
                    };
                    let offset = state.offset();
                    let prepared = OpaqueGradient::new_with_repeat(
                        target.width,
                        target.height,
                        rect.translated(offset.0, offset.1),
                        tile.translated(offset.0, offset.1),
                        browser_rect(state.clip()),
                        gradient,
                        *current_color,
                        *stop_scale,
                        *repeat,
                    )
                    .map_err(|error| gradient_error(error, phase_index, index))?
                    .ok_or_else(|| at(FallbackKind::InvalidGeometry, phase_index, index))?;
                    let source = (unique.len() + generated.len()) as u32;
                    let rgba = prepared
                        .materialize()
                        .map_err(|error| gradient_error(error, phase_index, index))?;
                    generated.push(RasterImage {
                        width: info.width,
                        height: info.height,
                        rgba,
                    });
                    // Already framebuffer-relative integer bounds. ScreenImage
                    // neither reapplies the phase offset nor normalizes samples.
                    Command::ScreenImage {
                        x: info.x as i32,
                        y: info.y as i32,
                        source,
                    }
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
            MaskSource::Box(index) => {
                let tile = &box_tiles[index];
                SourceMask {
                    width: tile.info().width,
                    height: tile.info().height,
                    coverage: tile.coverage(),
                }
            }
        });
    }
    let mut rows = reserve_native(row_sources.len())?;
    for source in &row_sources {
        rows.push(match *source {
            RowSource::Font(index) => font_rows[index].as_slice(),
            RowSource::Rounded(index) => rounded[index].row_origins(),
            RowSource::Box(index) => box_tiles[index].row_origins(),
        });
    }
    // Do not borrow generated entries until their owner Vec is complete.
    // Both the owners and these views remain live through core packing.
    for image in &generated {
        sources.push(SourceImage {
            width: image.width,
            height: image.height,
            rgba: &image.rgba,
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
        .map_err(|error| {
            let kind = if error.starts_with("unsupported opacity") {
                FallbackKind::UnsupportedOpacity
            } else {
                FallbackKind::PlannerLimit
            };
            NativeFallback::new(kind, None, None)
        })?;
    // The planner admits cropped group regions without allocating their pixel
    // surfaces on the CPU. Retain every earlier source/parameter/conversion
    // allowance and add the complete GPU scratch arena before publishing stats.
    ledger.gpu_upper = ledger
        .gpu_upper
        .checked_add(plan.group_scratch_bytes())
        .filter(|&bytes| bytes <= Profile::Native.max_gpu_buffer_bytes())
        .ok_or_else(|| NativeFallback::new(FallbackKind::PlannerLimit, None, None))?;
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
        generated_gradient_sources: gradients.sources,
        generated_gradient_bytes: gradients.bytes,
        gradient_evaluation_work: gradients.evaluation,
        gradient_preparation_work: gradients.packing,
        text_input_bytes: counts.text_bytes,
        text_input_scalars: counts.scalars,
        text: session.stats(),
        rounded_masks: rounded.len(),
        rounded_loop_work: ledger.rounded_loop_work,
        box_masks: box_tiles.len(),
        box_rectangles,
        box_loop_work: ledger.box_loop_work,
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

#[cfg(test)]
mod gradient_tests;

#[cfg(test)]
mod background_image_tests;

#[cfg(test)]
mod box_decoration_tests;

#[cfg(test)]
mod box_prepass_tests;

#[cfg(test)]
mod box_fastpath_tests;
