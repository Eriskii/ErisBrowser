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
pub const MAX_LUT_ENTRIES: usize = MAX_COMMANDS * (MAX_WIDTH + MAX_HEIGHT) as usize;
pub type Result<T> = std::result::Result<T, String>;

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
    PushClip(Rect),
    PopClip,
    PushFixed,
    PopFixed,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawKind {
    Rectangle,
    Image,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImageParameters {
    source_base: u32,
    source_width: u32,
    source_height: u32,
    source_pixels: u32,
    x_base: u32,
    y_base: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32, // source 0xAARRGGBB, including the opaque clear
    image: Option<ImageParameters>,
}
impl Draw {
    pub fn kind(self) -> DrawKind {
        if self.image.is_some() {
            DrawKind::Image
        } else {
            DrawKind::Rectangle
        }
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
    draws: Vec<Draw>,
    invocations: u64,
    gpu_buffer_bytes: u64,
    parameters: Vec<u8>,
    input: Vec<u8>,
}
impl Plan {
    pub fn frame(&self) -> Frame {
        self.frame
    }
    pub fn draws(&self) -> &[Draw] {
        &self.draws
    }
    pub fn invocations(&self) -> u64 {
        self.invocations
    }
    pub fn gpu_buffer_bytes(&self) -> u64 {
        self.gpu_buffer_bytes
    }
    pub fn parameters(&self) -> &[u8] {
        &self.parameters
    }
    pub fn input_bytes(&self) -> &[u8] {
        &self.input
    }
    pub fn has_images(&self) -> bool {
        !self.input.is_empty()
    }
}
#[derive(Clone, Copy)]
enum Scope {
    Clip(Rect),
    Fixed { clip: Rect, offset: (f32, f32) },
}
#[derive(Clone, Copy)]
struct ImageDraft {
    rect: Rect,
    source: usize,
}
struct PendingDraw {
    draw: Draw,
    image: Option<ImageDraft>,
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
            fields[8..].copy_from_slice(&[
                image.source_base,
                image.source_width,
                image.source_height,
                image.source_pixels,
                image.x_base,
                image.y_base,
                draw.width,
                draw.height,
            ]);
        }
        for (field, slot) in fields.into_iter().zip(record[..64].as_chunks_mut::<4>().0) {
            slot.copy_from_slice(&field.to_le_bytes());
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
    if frame.width == 0 || frame.height == 0 || frame.width > MAX_WIDTH || frame.height > MAX_HEIGHT
    {
        return Err("viewport budget".into());
    }
    if frame.clear > 0x00ff_ffff {
        return Err("clear color must be packed RGB".into());
    }
    if commands.len() > MAX_COMMANDS {
        return Err("command budget".into());
    }
    frame.caller_clip.validate()?;
    for v in [
        frame.document_offset.0,
        frame.document_offset.1,
        frame.viewport_offset.0,
        frame.viewport_offset.1,
    ] {
        if !v.is_finite() || v.abs() > MAX_COORDINATE {
            return Err("invalid offset".into());
        }
    }
    let source_words = source_pixels(sources)?;
    let viewport = Rect::new(0.0, 0.0, frame.width as f32, frame.height as f32);
    let caller = frame.caller_clip.normalized_clip(viewport);
    let mut clip = caller;
    let mut offset = frame.document_offset;
    let mut scopes = reserved(MAX_SCOPES)?;
    let clear = Draw {
        x: 0,
        y: 0,
        width: frame.width,
        height: frame.height,
        // Clear uses the rectangle shader but must always replace the target.
        color: 0xff00_0000 | frame.clear,
        image: None,
    };
    let mut pending = reserved(commands.len() + 1)?;
    pending.push(PendingDraw {
        draw: clear,
        image: None,
    });
    let mut invocations = clear.invocations();
    let mut lut_words = 0usize;
    let mut has_images = false;
    for command in commands {
        let (rect, color, image) = match *command {
            Command::Unsupported(kind) => return Err(format!("unsupported {kind}")),
            Command::PushClip(rect) => {
                rect.validate()?;
                if scopes.len() == MAX_SCOPES {
                    return Err("scope budget".into());
                }
                scopes.push(Scope::Clip(clip));
                clip = clip
                    .intersect(rect.translated(offset))
                    .normalized_clip(viewport);
                continue;
            }
            Command::PopClip => {
                match scopes.pop() {
                    Some(Scope::Clip(old)) => clip = old,
                    _ => return Err("mismatched clip scope".into()),
                }
                continue;
            }
            Command::PushFixed => {
                if scopes.len() == MAX_SCOPES {
                    return Err("scope budget".into());
                }
                scopes.push(Scope::Fixed { clip, offset });
                clip = caller;
                offset = frame.viewport_offset;
                continue;
            }
            Command::PopFixed => {
                match scopes.pop() {
                    Some(Scope::Fixed {
                        clip: old,
                        offset: old_offset,
                    }) => {
                        clip = old;
                        offset = old_offset;
                    }
                    _ => return Err("mismatched fixed scope".into()),
                }
                continue;
            }
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
        };
        let blend = image.is_some() || color >> 24 != 255;
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
        };
        invocations = invocations
            .checked_add(draw.invocations())
            .ok_or("invocation overflow")?;
        if invocations > MAX_INVOCATIONS {
            return Err("GPU invocation budget".into());
        }
        if image.is_some() {
            has_images = true;
            lut_words = lut_words
                .checked_add(width as usize + height as usize)
                .ok_or("LUT count overflow")?;
            if lut_words > MAX_LUT_ENTRIES {
                return Err("LUT entry budget".into());
            }
        }
        pending.push(PendingDraw { draw, image });
    }
    if !scopes.is_empty() {
        return Err("unclosed scope".into());
    }
    let arena_bytes = if has_images {
        source_words
            .checked_add(lut_words)
            .and_then(|n| n.checked_mul(4))
            .ok_or("input arena overflow")?
    } else {
        0
    };
    let parameter_size = pending
        .len()
        .checked_mul(PARAM_STRIDE)
        .ok_or("parameter overflow")?;
    let gpu_buffer_bytes = (u64::from(frame.width) * u64::from(frame.height) * 8)
        .checked_add(parameter_size as u64)
        .and_then(|n| n.checked_add(arena_bytes as u64))
        .ok_or("GPU buffer overflow")?;
    if gpu_buffer_bytes > MAX_GPU_BUFFER_BYTES {
        return Err("GPU buffer budget".into());
    }
    // Allocation/scanning below has already been bounded by the complete plan.
    // Alpha never triggers a CPU scan or draw suppression. If any image is
    // visible, all supplied source words get exactly one packing pass.
    let mut input = reserved(arena_bytes)?;
    let mut bases = [0u32; MAX_SOURCE_ENTRIES];
    if has_images {
        for (index, source) in sources.iter().enumerate() {
            bases[index] = (input.len() / 4) as u32;
            for rgba in source.rgba.as_chunks::<4>().0 {
                word(&mut input, packed_rgba(*rgba));
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
        draws,
        invocations,
        gpu_buffer_bytes,
        parameters,
        input,
    })
}

pub mod alpha_fixtures;
pub mod fixtures;
pub mod image_fixtures;
#[cfg(test)]
mod tests;
