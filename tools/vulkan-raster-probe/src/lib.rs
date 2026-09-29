#![forbid(unsafe_code)]

pub const MAX_WIDTH: u32 = 320;
pub const MAX_HEIGHT: u32 = 240;
pub const MAX_COMMANDS: usize = 256;
pub const MAX_SCOPES: usize = 32;
pub const MAX_INVOCATIONS: u64 = 4_000_000;
pub const MAX_GPU_BUFFER_BYTES: u64 = 1_048_576;
pub const PARAM_STRIDE: usize = 256;
pub const MAX_COORDINATE: f32 = 1_000_000.0;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub color: u32,
}
impl Draw {
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
    pub frame: Frame,
    pub draws: Vec<Draw>,
    pub invocations: u64,
    pub gpu_buffer_bytes: u64,
}
impl Plan {
    pub fn parameters(&self) -> Vec<u8> {
        // Only metadata, never target pixels. Padding makes dynamic offsets meet
        // the requested 256-byte uniform alignment; every slice remains immutable.
        let mut bytes = vec![0; self.draws.len() * PARAM_STRIDE];
        for (draw, record) in self
            .draws
            .iter()
            .zip(bytes.as_chunks_mut::<PARAM_STRIDE>().0.iter_mut())
        {
            let fields = [
                draw.x,
                draw.y,
                draw.width,
                draw.height,
                self.frame.width,
                self.frame.height,
                draw.color,
                0,
            ];
            for (field, slot) in fields.into_iter().zip(record[..32].as_chunks_mut::<4>().0) {
                slot.copy_from_slice(&field.to_le_bytes());
            }
        }
        bytes
    }
}
#[derive(Clone, Copy)]
enum Scope {
    Clip(Rect),
    Fixed { clip: Rect, offset: (f32, f32) },
}

// Complete validation/planning is required before any instance/device operation.
// This is intentionally a strict subset of Canvas: unsupported content rejects
// the whole stream, including content hidden by a clip or zero-sized rectangle.
pub fn plan(frame: Frame, commands: &[Command]) -> Result<Plan> {
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
    let viewport = Rect::new(0.0, 0.0, frame.width as f32, frame.height as f32);
    let caller = frame.caller_clip.normalized_clip(viewport);
    let mut clip = caller;
    let mut offset = frame.document_offset;
    let mut scopes = Vec::with_capacity(MAX_SCOPES);
    let clear = Draw {
        x: 0,
        y: 0,
        width: frame.width,
        height: frame.height,
        color: frame.clear,
    };
    let mut draws = Vec::with_capacity(commands.len() + 1);
    draws.push(clear);
    let mut invocations = clear.invocations();
    for command in commands {
        match *command {
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
            }
            Command::PopClip => match scopes.pop() {
                Some(Scope::Clip(old)) => clip = old,
                _ => return Err("mismatched clip scope".into()),
            },
            Command::PushFixed => {
                if scopes.len() == MAX_SCOPES {
                    return Err("scope budget".into());
                }
                scopes.push(Scope::Fixed { clip, offset });
                clip = caller;
                offset = frame.viewport_offset;
            }
            Command::PopFixed => match scopes.pop() {
                Some(Scope::Fixed {
                    clip: old,
                    offset: old_offset,
                }) => {
                    clip = old;
                    offset = old_offset
                }
                _ => return Err("mismatched fixed scope".into()),
            },
            Command::Rect { rect, rgba, radius } => {
                rect.validate()?;
                if rgba[3] != 255 || radius != 0.0 {
                    return Err("only opaque unrounded rectangles are supported".into());
                }
                let rect = rect.translated(offset);
                if rect.width <= 0.0 || rect.height <= 0.0 {
                    continue;
                }
                let visible = rect.intersect(clip);
                if visible.width <= 0.0 || visible.height <= 0.0 {
                    continue;
                }
                // Exact current Canvas::rect opaque fast-path operations. The
                // rectangle uses floor(left)/ceil(right), while a fractional
                // clip starts at ceil(left) and also ends at ceil(right).
                let x0 = visible.x.floor().max(clip.x.ceil()).max(0.0) as u32;
                let y0 = visible.y.floor().max(clip.y.ceil()).max(0.0) as u32;
                let x1 = (visible.x + visible.width).ceil().min(frame.width as f32) as u32;
                let y1 = (visible.y + visible.height).ceil().min(frame.height as f32) as u32;
                if x1 <= x0 || y1 <= y0 {
                    continue;
                }
                let draw = Draw {
                    x: x0,
                    y: y0,
                    width: x1 - x0,
                    height: y1 - y0,
                    color: (u32::from(rgba[0]) << 16)
                        | (u32::from(rgba[1]) << 8)
                        | u32::from(rgba[2]),
                };
                invocations = invocations
                    .checked_add(draw.invocations())
                    .ok_or("invocation overflow")?;
                if invocations > MAX_INVOCATIONS {
                    return Err("GPU invocation budget".into());
                }
                draws.push(draw);
            }
        }
    }
    if !scopes.is_empty() {
        return Err("unclosed scope".into());
    }
    let gpu_buffer_bytes =
        u64::from(frame.width) * u64::from(frame.height) * 8 + (draws.len() * PARAM_STRIDE) as u64;
    if gpu_buffer_bytes > MAX_GPU_BUFFER_BYTES {
        return Err("GPU buffer budget".into());
    }
    Ok(Plan {
        frame,
        draws,
        invocations,
        gpu_buffer_bytes,
    })
}

pub mod fixtures;
#[cfg(test)]
mod tests;
