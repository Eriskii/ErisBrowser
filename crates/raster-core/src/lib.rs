#![forbid(unsafe_code)]

pub const MAX_WIDTH: u32 = 320;
pub const MAX_HEIGHT: u32 = 240;
pub const MAX_COMMANDS: usize = 256;
pub const MAX_SCOPES: usize = 32;
pub const MAX_INVOCATIONS: u64 = 4_000_000;
pub const MAX_GPU_BUFFER_BYTES: u64 = 1_048_576;
pub const PARAM_STRIDE: usize = 256;
pub const MAX_COORDINATE: f32 = 1_000_000.0;
pub const MAX_SOURCE_ENTRIES: usize = 256;
pub const MAX_SOURCE_RGBA_BYTES: usize = 1_048_576;
pub const MAX_MASK_AXIS: u32 = 1024;
pub const MAX_MASK_PIXELS: usize = 262_144;
pub const MAX_MASK_COVERAGE_BYTES: usize = 262_144;
pub const MAX_ROW_ENTRIES: usize = 65_536;
pub const MAX_LUT_ENTRIES: usize = MAX_COMMANDS * (MAX_WIDTH + MAX_HEIGHT) as usize;
pub type Result<T> = std::result::Result<T, String>;

pub mod profile;
pub use profile::Profile;
mod opacity;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    fn validate(self) -> Result<()> {
        if [self.x, self.y, self.width, self.height]
            .into_iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
        {
            Ok(())
        } else {
            Err("invalid or over-limit rectangle".into())
        }
    }
    fn translated(self, offset: (f32, f32)) -> Self {
        Self {
            x: self.x + offset.0,
            y: self.y + offset.1,
            ..self
        }
    }
    fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: (self.x + self.width).min(other.x + other.width).max(x) - x,
            height: (self.y + self.height).min(other.y + other.height).max(y) - y,
        }
    }
    fn normalized_clip(self, viewport: Self) -> Self {
        if self.width <= 0.0 || self.height <= 0.0 {
            Self::new(0.0, 0.0, 0.0, 0.0)
        } else {
            self.intersect(viewport)
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Command {
    Rect {
        rect: Rect,
        rgba: [u8; 4],
        radius: f32,
    },
    Image {
        rect: Rect,
        source: u32,
    },
    /// Mask source and row-table indices are separate from image source IDs.
    /// Row origins and y are absolute framebuffer coordinates.
    Glyph {
        source: u32,
        rows: u32,
        y: i32,
        rgba: [u8; 4],
    },
    PushClip(Rect),
    PopClip,
    PushFixed,
    PopFixed,
    PushOpacity(f32),
    PopOpacity,
    Unsupported(&'static str),
}
pub fn rect(x: f32, y: f32, width: f32, height: f32, color: u32) -> Command {
    Command::Rect {
        rect: Rect::new(x, y, width, height),
        rgba: [(color >> 16) as u8, (color >> 8) as u8, color as u8, 255],
        radius: 0.0,
    }
}
fn packed_rgba(rgba: [u8; 4]) -> u32 {
    (u32::from(rgba[3]) << 24)
        | (u32::from(rgba[0]) << 16)
        | (u32::from(rgba[1]) << 8)
        | u32::from(rgba[2])
}
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub clear: u32,
    pub caller_clip: Rect,
    pub document_offset: (f32, f32),
    pub viewport_offset: (f32, f32),
}
impl Frame {
    pub fn new(width: u32, height: u32, clear: u32) -> Self {
        Self {
            width,
            height,
            clear,
            caller_clip: Rect::new(0.0, 0.0, width as f32, height as f32),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SourceImage<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
}
#[derive(Clone, Copy, Debug)]
pub struct SourceMask<'a> {
    pub width: u32,
    pub height: u32,
    pub coverage: &'a [u8],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawKind {
    Rectangle,
    Image,
    Glyph,
    GroupClear,
    GroupComposite,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImageParameters {
    source_base: u32,
    source_width: u32,
    source_height: u32,
    source_pixels: u32,
    x_base: u32,
    y_base: u32,
    // A glyph reuses the six input words; y is an absolute signed origin.
    glyph_y: Option<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32, // source 0xAARRGGBB, including the opaque clear
    image: Option<ImageParameters>,
    group: Option<opacity::Parameters>,
}
impl Draw {
    pub fn kind(self) -> DrawKind {
        if let Some(group) = self.group {
            match group.operation {
                opacity::Operation::Clear => return DrawKind::GroupClear,
                opacity::Operation::Composite => return DrawKind::GroupComposite,
                opacity::Operation::Paint => {}
            }
        }
        match self.image {
            Some(input) if input.glyph_y.is_some() => DrawKind::Glyph,
            Some(_) => DrawKind::Image,
            None => DrawKind::Rectangle,
        }
    }
    pub fn target_is_group(self) -> bool {
        self.group.is_some_and(|group| group.destination.is_some())
    }
    /// Exact k in k/256 for a materialized group's final composite operation.
    pub fn opacity_numerator(self) -> Option<u32> {
        self.group
            .filter(|group| group.operation == opacity::Operation::Composite)
            .map(|group| group.opacity)
    }
    pub fn bounds(self) -> (u32, u32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }
    pub fn groups(self) -> (u32, u32) {
        (self.width.div_ceil(8), self.height.div_ceil(8))
    }
    fn invocations(self) -> u64 {
        let (x, y) = self.groups();
        u64::from(x) * u64::from(y) * 64
    }
}
#[derive(Debug)]
pub struct Plan {
    frame: Frame,
    profile: Profile,
    conversion_invocations: u64,
    conversion_buffer_bytes: u64,
    draws: Vec<Draw>,
    invocations: u64,
    gpu_buffer_bytes: u64,
    group_scratch_bytes: u64,
    parameters: Vec<u8>,
    input: Vec<u8>,
}
impl Plan {
    pub fn profile(&self) -> Profile {
        self.profile
    }
    /// Work reserved for the mandatory Native packed-to-surface pass.
    pub fn conversion_invocations(&self) -> u64 {
        self.conversion_invocations
    }
    /// Native padded conversion output plus its uniform; excludes readback.
    pub fn conversion_buffer_bytes(&self) -> u64 {
        self.conversion_buffer_bytes
    }
    pub fn raster_invocations(&self) -> u64 {
        self.invocations - self.conversion_invocations
    }
    pub fn frame(&self) -> Frame {
        self.frame
    }
    pub fn draws(&self) -> &[Draw] {
        &self.draws
    }
    /// Total admitted work, including Native conversion when selected.
    pub fn invocations(&self) -> u64 {
        self.invocations
    }
    pub fn gpu_buffer_bytes(&self) -> u64 {
        self.gpu_buffer_bytes
    }
    /// Disjoint cropped RGBA16 intermediates retained for this entire frame.
    pub fn group_scratch_bytes(&self) -> u64 {
        self.group_scratch_bytes
    }
    pub fn parameters(&self) -> &[u8] {
        &self.parameters
    }
    pub fn input_bytes(&self) -> &[u8] {
        &self.input
    }
    /// Owned CPU residency: this Plan's fixed metadata plus actual capacities
    /// of its three vectors. Excludes allocator overhead and all GPU storage.
    pub fn retained_cpu_bytes(&self) -> Result<usize> {
        Self::retained_capacity_bytes(
            self.draws.capacity(),
            self.parameters.capacity(),
            self.input.capacity(),
        )
    }
    fn retained_capacity_bytes(draws: usize, parameters: usize, input: usize) -> Result<usize> {
        draws
            .checked_mul(std::mem::size_of::<Draw>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .and_then(|bytes| bytes.checked_add(parameters))
            .and_then(|bytes| bytes.checked_add(input))
            .ok_or_else(|| "retained plan CPU byte overflow".into())
    }
    pub fn has_images(&self) -> bool {
        self.draws.iter().any(|draw| draw.kind() == DrawKind::Image)
    }
    pub fn has_glyphs(&self) -> bool {
        self.draws.iter().any(|draw| draw.kind() == DrawKind::Glyph)
    }
    pub fn has_input(&self) -> bool {
        !self.input.is_empty()
    }
}
#[derive(Clone, Copy)]
struct ImageDraft {
    rect: Rect,
    source: usize,
}
#[derive(Clone, Copy)]
struct GlyphDraft {
    source: usize,
    rows: usize,
    y: i32,
}
struct PendingDraw {
    draw: Draw,
    image: Option<ImageDraft>,
    glyph: Option<GlyphDraft>,
    group: opacity::Pending,
}
fn reserved<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| "bounded planner allocation")?;
    Ok(values)
}
fn source_pixels(sources: &[SourceImage<'_>]) -> Result<usize> {
    if sources.len() > MAX_SOURCE_ENTRIES {
        return Err("source entry budget".into());
    }
    let mut total = 0u64;
    // Refuse bad lengths and aggregate sizes before inspecting source-sized data.
    for source in sources {
        if source.width == 0 || source.height == 0 {
            return Err("source dimensions".into());
        }
        let bytes = u64::from(source.width)
            .checked_mul(u64::from(source.height))
            .and_then(|n| n.checked_mul(4))
            .ok_or("source byte overflow")?;
        if bytes != source.rgba.len() as u64 {
            return Err("exact source RGBA length".into());
        }
        total = total.checked_add(bytes).ok_or("source byte overflow")?;
        if total > MAX_SOURCE_RGBA_BYTES as u64 {
            return Err("source byte budget".into());
        }
    }
    Ok(total as usize / 4)
}
fn mask_storage(masks: &[SourceMask<'_>], row_tables: &[&[i32]]) -> Result<(usize, usize)> {
    if masks.len() > MAX_SOURCE_ENTRIES || row_tables.len() > MAX_COMMANDS {
        return Err("mask or row table entry budget".into());
    }
    let mut pixels = 0usize;
    for mask in masks {
        if mask.width > MAX_MASK_AXIS
            || mask.height > MAX_MASK_AXIS
            || (mask.width == 0) != (mask.height == 0)
        {
            return Err("mask dimensions".into());
        }
        let area = (mask.width as usize)
            .checked_mul(mask.height as usize)
            .ok_or("mask area overflow")?;
        if area > MAX_MASK_PIXELS || area != mask.coverage.len() {
            return Err("exact mask coverage length or area budget".into());
        }
        pixels = pixels.checked_add(area).ok_or("mask total overflow")?;
        if pixels > MAX_MASK_COVERAGE_BYTES {
            return Err("mask coverage budget".into());
        }
    }
    let mut rows = 0usize;
    for table in row_tables {
        if table.len() > MAX_MASK_AXIS as usize {
            return Err("row table height budget".into());
        }
        rows = rows.checked_add(table.len()).ok_or("row total overflow")?;
        if rows > MAX_ROW_ENTRIES {
            return Err("row entry budget".into());
        }
    }
    Ok((pixels, rows))
}

fn glyph_coverage(
    mask: &SourceMask<'_>,
    rows: &[i32],
    y: i32,
    clip: Rect,
    frame: Frame,
) -> Option<(u32, u32, u32, u32)> {
    if mask.width == 0 || clip.width <= 0.0 || clip.height <= 0.0 {
        return None;
    }
    // Integer origins obey Canvas::blend's half-open clip. Use wide integers
    // before adding extents: every signed i32 source origin is legal.
    let left = clip.x.ceil().max(0.0) as i64;
    let right = (clip.x + clip.width).ceil().min(frame.width as f32) as i64;
    let top = clip.y.ceil().max(0.0) as i64;
    let bottom = (clip.y + clip.height).ceil().min(frame.height as f32) as i64;
    let start = (top - i64::from(y)).clamp(0, i64::from(mask.height)) as usize;
    let end = (bottom - i64::from(y)).clamp(0, i64::from(mask.height)) as usize;
    let (mut x0, mut x1, mut y0, mut y1) = (right, left, bottom, top);
    for (row, origin) in rows.iter().enumerate().take(end).skip(start) {
        let lo = i64::from(*origin).max(left);
        let hi = (i64::from(*origin) + i64::from(mask.width)).min(right);
        if hi > lo {
            x0 = x0.min(lo);
            x1 = x1.max(hi);
            let dy = i64::from(y) + row as i64;
            y0 = y0.min(dy);
            y1 = y1.max(dy + 1);
        }
    }
    (x1 > x0 && y1 > y0).then_some((x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32))
}

fn add_invocations(total: u64, draw: Draw) -> Result<u64> {
    profile::add_work(total, draw.invocations())
}
fn coverage(rect: Rect, clip: Rect, frame: Frame, blend: bool) -> Option<(u32, u32, u32, u32)> {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }
    let visible = rect.intersect(clip);
    if visible.width <= 0.0 || visible.height <= 0.0 {
        return None;
    }
    // Keep the existing rectangle fast-path f32 reconstruction unchanged.
    let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as u32;
    let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as u32;
    let mut x1 = (visible.x + visible.width).ceil().min(frame.width as f32);
    let mut y1 = (visible.y + visible.height).ceil().min(frame.height as f32);
    if blend {
        // Images and translucent rectangles use Canvas::blend's half-open
        // integer-origin containment after their own floor/ceil loop bounds.
        x1 = x1.min((clip.x + clip.width).ceil());
        y1 = y1.min((clip.y + clip.height).ceil());
    }
    let (x1, y1) = (x1 as u32, y1 as u32);
    (x1 > x0 && y1 > y0).then_some((x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0)))
}
fn sample_index(destination: i32, origin: f32, extent: f32, source_extent: u32) -> u32 {
    // Deliberately preserve Canvas's scalar f32 subtraction/division/multiplication
    // order and truncation. The destination is an integer origin, not its center.
    (((destination as f32 - origin) / extent) * source_extent as f32)
        .clamp(0.0, source_extent.saturating_sub(1) as f32) as u32
}
fn word(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn append_table(
    bytes: &mut Vec<u8>,
    start: u32,
    count: u32,
    origin: f32,
    extent: f32,
    source_extent: u32,
) -> Result<u32> {
    let base = u32::try_from(bytes.len() / 4).map_err(|_| "LUT offset overflow")?;
    for destination in start..start.checked_add(count).ok_or("LUT extent overflow")? {
        let index = sample_index(destination as i32, origin, extent, source_extent);
        if index >= source_extent {
            return Err("source index outside validated image".into());
        }
        word(bytes, index);
    }
    Ok(base)
}
fn parameter_bytes(frame: Frame, draws: &[Draw]) -> Result<Vec<u8>> {
    let size = draws
        .len()
        .checked_mul(PARAM_STRIDE)
        .ok_or("parameter overflow")?;
    let mut bytes = reserved(size)?;
    bytes.resize(size, 0);
    for (draw, record) in draws.iter().zip(bytes.as_chunks_mut::<PARAM_STRIDE>().0) {
        let mut fields = [0u32; 16];
        fields[..8].copy_from_slice(&[
            draw.x,
            draw.y,
            draw.width,
            draw.height,
            frame.width,
            frame.height,
            draw.color,
            0,
        ]);
        if let Some(image) = draw.image {
            fields[8..12].copy_from_slice(&[
                image.source_base,
                image.source_width,
                image.source_height,
                image.source_pixels,
            ]);
            fields[12..].copy_from_slice(&if let Some(y) = image.glyph_y {
                [image.x_base, y as u32, image.source_height, 0]
            } else {
                [image.x_base, image.y_base, draw.width, draw.height]
            });
        }
        for (field, slot) in fields.into_iter().zip(record[..64].as_chunks_mut::<4>().0) {
            slot.copy_from_slice(&field.to_le_bytes());
        }
        if let Some(group) = draw.group {
            group.write(frame, record);
        }
    }
    Ok(bytes)
}

// Complete validation/planning is required before any instance/device operation.
// Unsupported content rejects the whole stream even if hidden or zero-sized.
pub fn plan(frame: Frame, commands: &[Command]) -> Result<Plan> {
    plan_with_images(frame, commands, &[])
}
pub fn plan_with_images(
    frame: Frame,
    commands: &[Command],
    sources: &[SourceImage<'_>],
) -> Result<Plan> {
    plan_with_masks(frame, commands, sources, &[], &[])
}
/// Build a complete immutable plan before any GPU initialization. All supplied
/// sources/tables are validated, including hidden and unreferenced entries.
pub fn plan_with_masks(
    frame: Frame,
    commands: &[Command],
    sources: &[SourceImage<'_>],
    masks: &[SourceMask<'_>],
    row_tables: &[&[i32]],
) -> Result<Plan> {
    plan_with_masks_for_profile(Profile::Probe, frame, commands, sources, masks, row_tables)
}
/// Additive policy selection; old entry points retain the exact Probe policy.
/// Native admission reserves conversion work/storage but performs no presentation.
pub fn plan_with_masks_for_profile(
    profile: Profile,
    frame: Frame,
    commands: &[Command],
    sources: &[SourceImage<'_>],
    masks: &[SourceMask<'_>],
    row_tables: &[&[i32]],
) -> Result<Plan> {
    profile.validate_viewport(frame.width, frame.height)?;
    if frame.clear > 0x00ff_ffff {
        return Err("clear color must be packed RGB".into());
    }
    if commands.len() > MAX_COMMANDS {
        return Err("command budget".into());
    }
    let coordinates = scope::CoordinateState::new_for_profile(profile, frame)?;
    let mut draft = PlannerDraft::new(profile, frame, commands.len(), sources, masks, row_tables)?;
    draft.append(commands, coordinates)?;
    draft.finish()
}

/// One trusted composition phase; commands cannot reset another phase's scope.
#[derive(Clone, Copy, Debug)]
pub struct Phase<'a> {
    pub frame: Frame,
    pub commands: &'a [Command],
}
pub const MAX_NATIVE_PHASES: usize = 4;

/// Plan one Native target with one clear, one conversion reservation and global
/// budgets. Each phase starts fresh coordinates and must close its own scopes.
/// Target clip/offsets must be canonical; phase dimensions/clear must match it.
/// Zero phases still validate all supplied sources and produce the clear plan.
pub fn plan_native_phases(
    target: Frame,
    phases: &[Phase<'_>],
    sources: &[SourceImage<'_>],
    masks: &[SourceMask<'_>],
    row_tables: &[&[i32]],
) -> Result<Plan> {
    let profile = Profile::Native;
    profile.validate_viewport(target.width, target.height)?;
    if target.clear > 0x00ff_ffff {
        return Err("clear color must be packed RGB".into());
    }
    if phases.len() > MAX_NATIVE_PHASES {
        return Err("native phase budget".into());
    }
    let canonical = Frame::new(target.width, target.height, target.clear);
    if target.caller_clip != canonical.caller_clip
        || target.document_offset != (0.0, 0.0)
        || target.viewport_offset != (0.0, 0.0)
    {
        return Err("native target must have full clip and zero offsets".into());
    }
    let mut command_count = 0usize;
    for phase in phases {
        if phase.frame.width != target.width
            || phase.frame.height != target.height
            || phase.frame.clear != target.clear
        {
            return Err("native phase target mismatch".into());
        }
        command_count = command_count
            .checked_add(phase.commands.len())
            .filter(|&count| count <= MAX_COMMANDS)
            .ok_or("command budget")?;
    }
    // At most four bounded states; validate all descriptor geometry before any
    // source preparation. They are consumed in order, never shared or reset.
    let mut states = reserved(phases.len())?;
    for phase in phases {
        states.push(scope::CoordinateState::new_for_profile(
            profile,
            phase.frame,
        )?);
    }
    let mut draft = PlannerDraft::new(profile, target, command_count, sources, masks, row_tables)?;
    for (phase, coordinates) in phases.iter().zip(states) {
        draft.append(phase.commands, coordinates)?;
    }
    draft.finish()
}

struct PlannerDraft<'a> {
    frame: Frame,
    profile: Profile,
    sources: &'a [SourceImage<'a>],
    masks: &'a [SourceMask<'a>],
    row_tables: &'a [&'a [i32]],
    source_words: usize,
    mask_words: usize,
    row_words: usize,
    conversion_invocations: u64,
    conversion_buffer_bytes: u64,
    invocations: u64,
    pending: Vec<PendingDraw>,
    lut_words: usize,
    has_input: bool,
    opacity: opacity::State,
}

/// Conservative structural CPU peak for Native planning at the closed limits.
/// Counts the draft, final Plan, pending/final draw storage, four independent
/// coordinate states and scope arrays, and fixed packing-offset tables even
/// where their lifetimes do not overlap. Input/parameter arenas and caller-
/// borrowed source/mask/row descriptors are excluded and must be added by the
/// caller. Reservation sizes exclude allocator overhead; this is not a total
/// process or font-preparation memory bound.
pub fn native_planner_metadata_peak_bytes() -> Result<usize> {
    let slots = MAX_COMMANDS
        .checked_add(1)
        .ok_or("planner metadata overflow")?;
    let counts = [
        (1, std::mem::size_of::<Plan>()),
        (1, std::mem::size_of::<PlannerDraft<'_>>()),
        (slots, std::mem::size_of::<PendingDraw>()),
        (slots, std::mem::size_of::<Draw>()),
        (1, std::mem::size_of::<Vec<scope::CoordinateState>>()),
        (
            MAX_NATIVE_PHASES,
            std::mem::size_of::<scope::CoordinateState>(),
        ),
        (
            MAX_NATIVE_PHASES
                .checked_mul(MAX_SCOPES)
                .ok_or("planner metadata overflow")?,
            std::mem::size_of::<scope::Scope>(),
        ),
        (
            MAX_SOURCE_ENTRIES
                .checked_mul(2)
                .and_then(|n| n.checked_add(MAX_COMMANDS))
                .ok_or("planner metadata overflow")?,
            std::mem::size_of::<u32>(),
        ),
    ];
    counts
        .into_iter()
        .try_fold(opacity::metadata_peak_bytes()?, |total, (count, size)| {
            count
                .checked_mul(size)
                .and_then(|bytes| total.checked_add(bytes))
                .ok_or_else(|| "planner metadata overflow".into())
        })
}
impl<'a> PlannerDraft<'a> {
    fn new(
        profile: Profile,
        frame: Frame,
        command_count: usize,
        sources: &'a [SourceImage<'a>],
        masks: &'a [SourceMask<'a>],
        row_tables: &'a [&'a [i32]],
    ) -> Result<Self> {
        // Preserve the image-only API's original source validation and diagnostics.
        let source_words = source_pixels(sources)?;
        if sources
            .len()
            .checked_add(masks.len())
            .is_none_or(|n| n > MAX_SOURCE_ENTRIES)
        {
            return Err("combined source entry budget".into());
        }
        let (mask_words, row_words) = mask_storage(masks, row_tables)?;
        let clear = Draw {
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height,
            // Clear uses the rectangle shader but must always replace the target.
            color: 0xff00_0000 | frame.clear,
            image: None,
            group: None,
        };
        let conversion_invocations = profile.conversion_invocations(frame.width, frame.height)?;
        let conversion_buffer_bytes = profile.conversion_buffer_bytes(frame.width, frame.height)?;
        let invocations = profile::add_work(clear.invocations(), conversion_invocations)?;
        let mut pending = reserved(command_count + 1)?;
        pending.push(PendingDraw {
            draw: clear,
            image: None,
            glyph: None,
            group: opacity::Pending::Root,
        });
        let lut_words = 0usize;
        let has_input = false;
        Ok(Self {
            frame,
            profile,
            sources,
            masks,
            row_tables,
            source_words,
            mask_words,
            row_words,
            conversion_invocations,
            conversion_buffer_bytes,
            invocations,
            pending,
            lut_words,
            has_input,
            opacity: opacity::State::new(command_count)?,
        })
    }

    fn append(
        &mut self,
        commands: &[Command],
        mut coordinates: scope::CoordinateState,
    ) -> Result<()> {
        let frame = self.frame;
        let profile = self.profile;
        let sources = self.sources;
        let masks = self.masks;
        let row_tables = self.row_tables;
        let mut invocations = self.invocations;
        let mut lut_words = self.lut_words;
        let mut has_input = self.has_input;
        let pending = &mut self.pending;
        for command in commands {
            if coordinates.apply(command)? {
                let operation = match *command {
                    Command::PushOpacity(value) => self.opacity.push(value)?,
                    Command::PopOpacity => self.opacity.pop()?,
                    _ => None,
                };
                if let Some(operation) = operation {
                    pending.push(operation);
                }
                continue;
            }
            let clip = coordinates.clip();
            let offset = coordinates.offset();
            if let Command::Glyph {
                source,
                rows,
                y,
                rgba,
            } = *command
            {
                let source = source as usize;
                let rows = rows as usize;
                let mask = masks.get(source).ok_or("mask ID outside validated table")?;
                let origins = row_tables
                    .get(rows)
                    .ok_or("row ID outside validated table")?;
                if origins.len() != mask.height as usize {
                    return Err("glyph row count differs from mask height".into());
                }
                // Validate IDs and row geometry even when transparent or clipped.
                if rgba[3] == 0 || self.opacity.suppressed() {
                    continue;
                }
                let Some((x, dy, width, height)) = glyph_coverage(mask, origins, y, clip, frame)
                else {
                    continue;
                };
                let draw = Draw {
                    x,
                    y: dy,
                    width,
                    height,
                    color: packed_rgba(rgba),
                    image: None,
                    group: None,
                };
                invocations = add_invocations(invocations, draw)?;
                has_input = true;
                pending.push(PendingDraw {
                    draw,
                    image: None,
                    glyph: Some(GlyphDraft { source, rows, y }),
                    group: self.opacity.paint(draw),
                });
                continue;
            }
            let (rect, color, image) = match *command {
                Command::Unsupported(kind) => return Err(format!("unsupported {kind}")),
                Command::Rect { rect, rgba, radius } => {
                    rect.validate()?;
                    if radius != 0.0 {
                        return Err("only unrounded rectangles are supported".into());
                    }
                    if rgba[3] == 0 {
                        continue;
                    }
                    (rect.translated(offset), packed_rgba(rgba), None)
                }
                Command::Image { rect, source } => {
                    rect.validate()?;
                    let source = source as usize;
                    if sources.get(source).is_none() {
                        return Err("source ID outside validated table".into());
                    }
                    let rect = rect.translated(offset);
                    (rect, 0, Some(ImageDraft { rect, source }))
                }
                Command::PushClip(_)
                | Command::PopClip
                | Command::PushFixed
                | Command::PopFixed
                | Command::PushOpacity(_)
                | Command::PopOpacity
                | Command::Glyph { .. } => unreachable!("handled state or glyph command"),
            };
            if self.opacity.suppressed() {
                continue;
            }
            // CPU opaque rectangles inside an opacity layer use blend's
            // half-open integer clip, not the opaque-root fill fast path.
            let blend = image.is_some() || color >> 24 != 255 || self.opacity.in_group();
            let Some((x, y, width, height)) = coverage(rect, clip, frame, blend) else {
                continue;
            };
            let draw = Draw {
                x,
                y,
                width,
                height,
                color,
                image: None,
                group: None,
            };
            invocations = add_invocations(invocations, draw)?;
            if image.is_some() {
                has_input = true;
                lut_words = lut_words
                    .checked_add(width as usize + height as usize)
                    .ok_or("LUT count overflow")?;
                if lut_words > profile.max_lut_entries() {
                    return Err("LUT entry budget".into());
                }
            }
            pending.push(PendingDraw {
                draw,
                image,
                glyph: None,
                group: self.opacity.paint(draw),
            });
        }
        coordinates.finish()?;
        self.opacity.finish_phase()?;
        self.invocations = invocations;
        self.lut_words = lut_words;
        self.has_input = has_input;
        Ok(())
    }

    fn finish(self) -> Result<Plan> {
        let Self {
            frame,
            profile,
            sources,
            masks,
            row_tables,
            source_words,
            mask_words,
            row_words,
            conversion_invocations,
            conversion_buffer_bytes,
            mut invocations,
            mut pending,
            lut_words,
            has_input,
            opacity,
        } = self;
        let group_scratch_bytes = opacity.materialize(&mut pending, &mut invocations)?;
        let arena_bytes = if has_input {
            source_words
                .checked_add(mask_words)
                .and_then(|n| n.checked_add(row_words))
                .and_then(|n| n.checked_add(lut_words))
                .and_then(|n| n.checked_mul(4))
                .ok_or("input arena overflow")?
        } else {
            0
        };
        let parameter_size = pending
            .len()
            .checked_mul(PARAM_STRIDE)
            .ok_or("parameter overflow")?;
        let packed_target_bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
        let second_buffer_bytes = match profile {
            Profile::Probe => packed_target_bytes,
            Profile::Native => conversion_buffer_bytes,
        };
        let gpu_buffer_bytes = packed_target_bytes
            .checked_add(second_buffer_bytes)
            .and_then(|n| n.checked_add(parameter_size as u64))
            .and_then(|n| n.checked_add(arena_bytes as u64))
            .and_then(|n| n.checked_add(group_scratch_bytes))
            .ok_or("GPU buffer overflow")?;
        profile.validate_buffer_bytes(gpu_buffer_bytes)?;
        // Allocation/scanning below has already been bounded by the complete plan.
        // Input alpha/coverage never triggers a CPU scan or draw suppression. If
        // an input-backed draw is visible, all supplied sources and row tables
        // get exactly one packing pass.
        let mut input = reserved(arena_bytes)?;
        let mut bases = [0u32; MAX_SOURCE_ENTRIES];
        let mut mask_bases = [0u32; MAX_SOURCE_ENTRIES];
        let mut row_bases = [0u32; MAX_COMMANDS];
        if has_input {
            for (index, source) in sources.iter().enumerate() {
                bases[index] = (input.len() / 4) as u32;
                for rgba in source.rgba.as_chunks::<4>().0 {
                    word(&mut input, packed_rgba(*rgba));
                }
            }
            for (index, mask) in masks.iter().enumerate() {
                mask_bases[index] = (input.len() / 4) as u32;
                for alpha in mask.coverage {
                    word(&mut input, u32::from(*alpha));
                }
            }
            for (index, rows) in row_tables.iter().enumerate() {
                row_bases[index] = (input.len() / 4) as u32;
                for origin in *rows {
                    word(&mut input, *origin as u32);
                }
            }
        }
        let mut draws = reserved(pending.len())?;
        for item in pending {
            let mut draw = item.draw;
            if let Some(image) = item.image {
                let source = &sources[image.source];
                let x_base = append_table(
                    &mut input,
                    draw.x,
                    draw.width,
                    image.rect.x,
                    image.rect.width,
                    source.width,
                )?;
                let y_base = append_table(
                    &mut input,
                    draw.y,
                    draw.height,
                    image.rect.y,
                    image.rect.height,
                    source.height,
                )?;
                draw.image = Some(ImageParameters {
                    source_base: bases[image.source],
                    source_width: source.width,
                    source_height: source.height,
                    source_pixels: (source.rgba.len() / 4) as u32,
                    x_base,
                    y_base,
                    glyph_y: None,
                });
            }
            if let Some(glyph) = item.glyph {
                let mask = &masks[glyph.source];
                draw.image = Some(ImageParameters {
                    source_base: mask_bases[glyph.source],
                    source_width: mask.width,
                    source_height: mask.height,
                    source_pixels: mask.coverage.len() as u32,
                    x_base: row_bases[glyph.rows],
                    y_base: 0,
                    glyph_y: Some(glyph.y),
                });
            }
            draws.push(draw);
        }
        if input.len() != arena_bytes {
            return Err("planned arena size mismatch".into());
        }
        let parameters = parameter_bytes(frame, &draws)?;
        Ok(Plan {
            frame,
            profile,
            conversion_invocations,
            conversion_buffer_bytes,
            draws,
            invocations,
            gpu_buffer_bytes,
            group_scratch_bytes,
            parameters,
            input,
        })
    }
}

/// Validate every supplied image and return its aggregate pixel count.
///
/// This is the planner's original validation path, exposed so adapters can
/// reject malformed or excessive sources before preparing dependent content.
/// The validator reads dimensions and slice lengths, never source pixels.
pub fn validate_source_images(sources: &[SourceImage<'_>]) -> Result<usize> {
    source_pixels(sources)
}

#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(test)]
mod opacity_arithmetic_tests;
#[cfg(test)]
mod opacity_tests;
#[cfg(test)]
mod phase_tests;
#[cfg(test)]
mod profile_tests;
pub mod rounded;
#[cfg(test)]
mod rounded_tests;
pub mod scope;
#[cfg(feature = "gpu")]
pub mod surface;
#[cfg(all(test, feature = "gpu"))]
mod surface_tests;

// Keep the original unit-test bodies and independent fixture bytes in one
// repository-owned location. Only core test builds compile these modules;
// the production core has no fixtures, expected pixels or probe dependency.
#[cfg(test)]
#[path = "../../../tools/vulkan-raster-probe/src/alpha_fixtures.rs"]
mod alpha_fixtures;
#[cfg(test)]
#[path = "../../../tools/vulkan-raster-probe/src/fixtures.rs"]
mod fixtures;
#[cfg(test)]
#[path = "../../../tools/vulkan-raster-probe/src/glyph_tests.rs"]
mod glyph_tests;
#[cfg(test)]
#[path = "../../../tools/vulkan-raster-probe/src/image_fixtures.rs"]
mod image_fixtures;
#[cfg(test)]
#[path = "../../../tools/vulkan-raster-probe/src/tests.rs"]
mod tests;
