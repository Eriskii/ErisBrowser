//! Software display-list painter. No browser engine or GPU is involved.
use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use std::{cell::RefCell, collections::HashMap, path::Path, sync::Arc};

const MAX_PAINT_COMMANDS: usize = 200_000;
const MAX_PAINT_GLYPHS: usize = 100_000;
const MAX_TEXT_GLYPHS: usize = 32_768;
const MAX_PAINT_PIXELS: u64 = 32_000_000;
const MAX_CLIP_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn packed(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
    pub fn translated(self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            ..self
        }
    }
    pub fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: (self.x + self.width).min(other.x + other.width).max(x) - x,
            height: (self.y + self.height).min(other.y + other.height).max(y) - y,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    PushClip {
        rect: Rect,
    },
    PopClip,
    /// Enter viewport coordinates, retaining only the caller's viewport clip.
    PushFixed,
    /// Restore the enclosing coordinate and clipping scope.
    PopFixed,
    Rect {
        rect: Rect,
        color: Color,
        radius: f32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        color: Color,
        bold: bool,
        italic: bool,
        monospace: bool,
    },
    Image {
        rect: Rect,
        key: String,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
        width: f32,
    },
}

struct GlyphBitmap {
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    alpha: Vec<u8>,
}
type GlyphCache = HashMap<(usize, char, u16), Arc<GlyphBitmap>>;
pub struct Fonts {
    faces: [FontArc; 3],
    cache: RefCell<GlyphCache>,
}
impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}
impl Fonts {
    pub fn new() -> Self {
        Self {
            faces: [
                FontArc::try_from_slice(include_bytes!("../assets/DejaVuSans.ttf"))
                    .expect("bundled font"),
                FontArc::try_from_slice(include_bytes!("../assets/DejaVuSans-Bold.ttf"))
                    .expect("bundled font"),
                FontArc::try_from_slice(include_bytes!("../assets/DejaVuSansMono.ttf"))
                    .expect("bundled font"),
            ],
            cache: RefCell::new(HashMap::new()),
        }
    }
    fn face_index(bold: bool, mono: bool) -> usize {
        if mono { 2 } else { usize::from(bold) }
    }
    // ab_glyph's scale describes ascent minus descent; CSS font-size describes the em square.
    fn scale(&self, index: usize, size: f32) -> PxScale {
        let face = &self.faces[index];
        let size = if size.is_finite() { size } else { 16.0 };
        PxScale::from(
            size.clamp(1.0, 512.0) * face.height_unscaled() / face.units_per_em().unwrap_or(2048.0),
        )
    }
    pub fn measure(
        &self,
        text: &str,
        size: f32,
        bold: bool,
        _italic: bool,
        monospace: bool,
    ) -> f32 {
        let i = Self::face_index(bold, monospace);
        let f = self.faces[i].as_scaled(self.scale(i, size));
        let mut width = 0.0;
        let mut prev = None;
        for c in text.chars() {
            let id = f.glyph_id(c);
            if let Some(p) = prev {
                width += f.kern(p, id);
            }
            width += f.h_advance(id);
            prev = Some(id);
        }
        width
    }
    fn bitmap(&self, index: usize, ch: char, size: f32) -> Arc<GlyphBitmap> {
        let q = (size.clamp(1.0, 512.0) * 8.0).round() as u16;
        let key = (index, ch, q);
        if let Some(g) = self.cache.borrow().get(&key) {
            return g.clone();
        }
        let f = self.faces[index].as_scaled(self.scale(index, q as f32 / 8.0));
        let glyph = f
            .glyph_id(ch)
            .with_scale_and_position(f.scale(), point(0.0, f.ascent()));
        let bitmap = if let Some(outline) = f.outline_glyph(glyph) {
            let b = outline.px_bounds();
            let w = b.width() as usize;
            let h = b.height() as usize;
            let mut alpha = vec![0; w * h];
            outline.draw(|x, y, a| {
                alpha[y as usize * w + x as usize] = (a * 255.0) as u8;
            });
            GlyphBitmap {
                x: b.min.x as i32,
                y: b.min.y as i32,
                width: w,
                height: h,
                alpha,
            }
        } else {
            GlyphBitmap {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                alpha: Vec::new(),
            }
        };
        let g = Arc::new(bitmap);
        let mut cache = self.cache.borrow_mut();
        // Bound retained glyph memory. Large glyphs are used without being cached.
        if g.alpha.len() < 16_384 {
            if cache.len() >= 2048 {
                cache.clear();
            }
            cache.insert(key, g.clone());
        }
        g
    }
}

#[derive(Clone, Debug)]
pub struct RasterImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
pub type ImageStore = HashMap<String, Arc<RasterImage>>;

struct PaintBudget {
    pixels: u64,
    glyphs: usize,
}

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
    clip: Rect,
    budget: Option<PaintBudget>,
    paint_exhausted: bool,
}
impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        if width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || u64::from(width) * u64::from(height) > 16_777_216
        {
            return Err("viewport exceeds 16 megapixels or 8192 pixels per axis".into());
        }
        Ok(Self {
            width,
            height,
            pixels: vec![0xffffff; width as usize * height as usize],
            clip: Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            budget: None,
            paint_exhausted: false,
        })
    }

    /// True when the most recent display list exceeded its work budget.
    /// Callers should report that the page's painting was limited.
    pub fn exhausted(&self) -> bool {
        self.paint_exhausted
    }

    fn consume_pixels(&mut self, pixels: u64) -> bool {
        let Some(budget) = &mut self.budget else {
            return true;
        };
        if pixels > budget.pixels {
            budget.pixels = 0;
            self.paint_exhausted = true;
            false
        } else {
            budget.pixels -= pixels;
            true
        }
    }

    fn consume_glyph(&mut self) -> bool {
        let Some(budget) = &mut self.budget else {
            return true;
        };
        if budget.glyphs == 0 {
            self.paint_exhausted = true;
            false
        } else {
            budget.glyphs -= 1;
            true
        }
    }
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.packed());
    }
    pub fn set_clip(&mut self, clip: Rect) {
        self.clip = if ![clip.x, clip.y, clip.width, clip.height]
            .iter()
            .all(|value| value.is_finite())
            || clip.width <= 0.0
            || clip.height <= 0.0
        {
            Rect::default()
        } else {
            clip.intersect(Rect {
                x: 0.0,
                y: 0.0,
                width: self.width as f32,
                height: self.height as f32,
            })
        };
    }
    fn blend(&mut self, x: i32, y: i32, color: Color, coverage: u8) {
        if x < 0
            || y < 0
            || x >= self.width as i32
            || y >= self.height as i32
            || !self.clip.contains(x as f32, y as f32)
        {
            return;
        }
        let a = u32::from(color.a) * u32::from(coverage) / 255;
        if a == 0 {
            return;
        }
        let p = &mut self.pixels[y as usize * self.width as usize + x as usize];
        if a == 255 {
            *p = color.packed();
            return;
        }
        let inv = 255 - a;
        let r = (u32::from(color.r) * a + ((*p >> 16) & 255) * inv + 127) / 255;
        let g = (u32::from(color.g) * a + ((*p >> 8) & 255) * inv + 127) / 255;
        let b = (u32::from(color.b) * a + (*p & 255) * inv + 127) / 255;
        *p = (r << 16) | (g << 8) | b;
    }
    pub fn rect(&mut self, rect: Rect, color: Color, radius: f32) {
        if ![rect.x, rect.y, rect.width, rect.height, radius]
            .iter()
            .all(|v| v.is_finite())
            || rect.width <= 0.0
            || rect.height <= 0.0
            || color.a == 0
        {
            return;
        }
        let visible = rect.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return;
        }
        // The opaque scanline fast path must obey the same fractional clip
        // boundary as blend(), which tests each pixel's integer origin.
        let x0 = visible.x.floor().max(self.clip.x.ceil()).max(0.0) as i32;
        let y0 = visible.y.floor().max(self.clip.y.ceil()).max(0.0) as i32;
        let x1 = (visible.x + visible.width).ceil().min(self.width as f32) as i32;
        let y1 = (visible.y + visible.height).ceil().min(self.height as f32) as i32;
        if !self.consume_pixels((x1 - x0).max(0) as u64 * (y1 - y0).max(0) as u64) {
            return;
        }
        let radius = radius.max(0.0).min(rect.width / 2.0).min(rect.height / 2.0);
        if radius == 0.0 && color.a == 255 {
            for y in y0..y1 {
                if x1 > x0 {
                    self.pixels[y as usize * self.width as usize + x0 as usize
                        ..y as usize * self.width as usize + x1 as usize]
                        .fill(color.packed());
                }
            }
            return;
        }
        for y in y0..y1 {
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let dx = (rect.x + radius - px)
                    .max(px - (rect.x + rect.width - radius))
                    .max(0.0);
                let dy = (rect.y + radius - py)
                    .max(py - (rect.y + rect.height - radius))
                    .max(0.0);
                let cov = if radius == 0.0 {
                    255
                } else {
                    ((radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0) * 255.0) as u8
                };
                self.blend(x, y, color, cov);
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        fonts: &Fonts,
        x: f32,
        y: f32,
        text: &str,
        size: f32,
        color: Color,
        bold: bool,
        italic: bool,
        monospace: bool,
    ) {
        if !x.is_finite() || !y.is_finite() || !size.is_finite() || color.a == 0 {
            return;
        }
        let size = size.clamp(1.0, 512.0);
        if self.clip.width <= 0.0
            || self.clip.height <= 0.0
            || y > self.clip.y + self.clip.height
            || y + size * 1.5 < self.clip.y
        {
            return;
        }
        let index = Fonts::face_index(bold, monospace);
        let f = fonts.faces[index].as_scaled(fonts.scale(index, size));
        let mut pen = x;
        let mut prev = None;
        for (index_in_text, ch) in text.chars().enumerate() {
            if index_in_text >= MAX_TEXT_GLYPHS || !self.consume_glyph() {
                self.paint_exhausted = true;
                break;
            }
            let id = f.glyph_id(ch);
            if let Some(p) = prev {
                pen += f.kern(p, id);
            }
            prev = Some(id);
            if pen > self.clip.x + self.clip.width + size {
                break;
            }
            if pen + size >= self.clip.x {
                let b = fonts.bitmap(index, ch, size);
                // Charge the actual loop work, including off-clip glyph pixels.
                if !self.consume_pixels((b.width as u64 * b.height as u64).max(1)) {
                    return;
                }
                for gy in 0..b.height {
                    for gx in 0..b.width {
                        let shear = if italic {
                            (size - (b.y as f32 + gy as f32)) * 0.18
                        } else {
                            0.0
                        };
                        self.blend(
                            (pen + shear).round() as i32 + b.x + gx as i32,
                            y.round() as i32 + b.y + gy as i32,
                            color,
                            b.alpha[gy * b.width + gx],
                        );
                    }
                }
            }
            pen += f.h_advance(id);
        }
    }
    pub fn image(&mut self, rect: Rect, img: &RasterImage) {
        if rect.width <= 0.0
            || rect.height <= 0.0
            || img.width == 0
            || img.height == 0
            || ![rect.x, rect.y, rect.width, rect.height]
                .iter()
                .all(|v| v.is_finite())
        {
            return;
        }
        let required = (img.width as usize)
            .checked_mul(img.height as usize)
            .and_then(|pixels| pixels.checked_mul(4));
        if required.is_none_or(|bytes| bytes > img.rgba.len()) {
            return;
        }
        let r = rect.intersect(self.clip);
        if r.width <= 0.0 || r.height <= 0.0 {
            return;
        }
        let x0 = r.x.floor().max(0.0) as i32;
        let y0 = r.y.floor().max(0.0) as i32;
        let x1 = (r.x + r.width).ceil().min(self.width as f32) as i32;
        let y1 = (r.y + r.height).ceil().min(self.height as f32) as i32;
        if !self.consume_pixels((x1 - x0).max(0) as u64 * (y1 - y0).max(0) as u64) {
            return;
        }
        for y in y0..y1 {
            let sy = (((y as f32 - rect.y) / rect.height) * img.height as f32)
                .clamp(0.0, img.height.saturating_sub(1) as f32) as usize;
            for x in x0..x1 {
                let sx = (((x as f32 - rect.x) / rect.width) * img.width as f32)
                    .clamp(0.0, img.width.saturating_sub(1) as f32)
                    as usize;
                let p = (sy * img.width as usize + sx) * 4;
                if let Some(c) = img.rgba.get(p..p + 4) {
                    self.blend(x, y, Color::rgba(c[0], c[1], c[2], c[3]), 255);
                }
            }
        }
    }
    pub fn paint(
        &mut self,
        commands: &[DrawCommand],
        fonts: &Fonts,
        images: &ImageStore,
        dx: f32,
        dy: f32,
    ) {
        self.paint_with_viewport(commands, fonts, images, (dx, dy), (dx, dy));
    }
    pub fn paint_with_viewport(
        &mut self,
        commands: &[DrawCommand],
        fonts: &Fonts,
        images: &ImageStore,
        document_offset: (f32, f32),
        viewport_offset: (f32, f32),
    ) {
        enum Scope {
            Clip(Rect),
            Fixed { clip: Rect, offset: (f32, f32) },
        }
        self.paint_exhausted = false;
        let caller_clip = self.clip;
        let mut scopes = Vec::new();
        let (mut dx, mut dy) = document_offset;
        self.budget = Some(PaintBudget {
            pixels: (u64::from(self.width) * u64::from(self.height) * 16)
                .clamp(1_000_000, MAX_PAINT_PIXELS),
            glyphs: MAX_PAINT_GLYPHS,
        });
        for (index, command) in commands.iter().enumerate() {
            if self.paint_exhausted || index >= MAX_PAINT_COMMANDS {
                self.paint_exhausted = true;
                break;
            }
            match command {
                DrawCommand::PushClip { rect } => {
                    if scopes.len() >= MAX_CLIP_DEPTH {
                        self.paint_exhausted = true;
                        break;
                    }
                    scopes.push(Scope::Clip(self.clip));
                    let nested = rect.translated(dx, dy);
                    if [nested.x, nested.y, nested.width, nested.height]
                        .iter()
                        .all(|value| value.is_finite())
                    {
                        self.set_clip(self.clip.intersect(nested));
                    } else {
                        self.set_clip(Rect::default());
                    }
                }
                DrawCommand::PopClip => {
                    let Some(Scope::Clip(previous)) = scopes.pop() else {
                        self.paint_exhausted = true;
                        break;
                    };
                    self.clip = previous;
                }
                DrawCommand::PushFixed => {
                    if scopes.len() >= MAX_CLIP_DEPTH {
                        self.paint_exhausted = true;
                        break;
                    }
                    scopes.push(Scope::Fixed {
                        clip: self.clip,
                        offset: (dx, dy),
                    });
                    self.clip = caller_clip;
                    (dx, dy) = viewport_offset;
                }
                DrawCommand::PopFixed => {
                    let Some(Scope::Fixed { clip, offset }) = scopes.pop() else {
                        self.paint_exhausted = true;
                        break;
                    };
                    self.clip = clip;
                    (dx, dy) = offset;
                }
                DrawCommand::Rect {
                    rect,
                    color,
                    radius,
                } => self.rect(rect.translated(dx, dy), *color, *radius),
                DrawCommand::Text {
                    x,
                    y,
                    text,
                    size,
                    color,
                    bold,
                    italic,
                    monospace,
                } => self.text(
                    fonts,
                    x + dx,
                    y + dy,
                    text,
                    *size,
                    *color,
                    *bold,
                    *italic,
                    *monospace,
                ),
                DrawCommand::Image { rect, key } => {
                    if let Some(img) = images.get(key) {
                        self.image(rect.translated(dx, dy), img);
                    }
                }
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                    width,
                } => {
                    let rect = Rect {
                        x: x1.min(*x2) + dx,
                        y: y1.min(*y2) + dy,
                        width: (x2 - x1).abs().max(*width),
                        height: (y2 - y1).abs().max(*width),
                    };
                    self.rect(rect, *color, 0.0);
                }
            }
        }
        // Browser chrome uses direct primitives after the page display list;
        // a hostile page cannot consume the toolbar's ability to paint.
        self.budget = None;
        self.clip = caller_clip;
    }
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let rgb: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
            .collect();
        image::save_buffer(path, &rgb, self.width, self.height, image::ColorType::Rgb8)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_scope_uses_viewport_origin_and_restores_document_clips() {
        let mut canvas = Canvas::new(12, 12).unwrap();
        canvas.clear(Color::WHITE);
        let viewport = Rect {
            x: 0.0,
            y: 2.0,
            width: 12.0,
            height: 8.0,
        };
        canvas.set_clip(viewport);
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 12.0,
            height: 12.0,
        };
        canvas.paint_with_viewport(
            &[
                DrawCommand::PushClip {
                    rect: Rect {
                        x: 0.0,
                        y: 8.0,
                        width: 3.0,
                        height: 3.0,
                    },
                },
                DrawCommand::PushFixed,
                DrawCommand::Rect {
                    rect,
                    color: Color::rgb(255, 0, 0),
                    radius: 0.0,
                },
                DrawCommand::PushClip {
                    rect: Rect {
                        x: 4.0,
                        y: 1.0,
                        width: 2.0,
                        height: 2.0,
                    },
                },
                DrawCommand::Rect {
                    rect,
                    color: Color::rgb(0, 0, 255),
                    radius: 0.0,
                },
                DrawCommand::PopClip,
                DrawCommand::PopFixed,
                DrawCommand::Rect {
                    rect: Rect { y: 8.0, ..rect },
                    color: Color::rgb(0, 255, 0),
                    radius: 0.0,
                },
                DrawCommand::PopClip,
            ],
            &Fonts::new(),
            &ImageStore::new(),
            (0.0, -6.0),
            (0.0, 2.0),
        );
        assert_eq!(canvas.pixels[12], 0xffffff); // Toolbar remains outside caller clip.
        assert_eq!(canvas.pixels[2 * 12 + 1], 0x00ff00); // Document offset restored.
        assert_eq!(canvas.pixels[3 * 12 + 4], 0x0000ff); // Nested fixed clip.
        assert_eq!(canvas.pixels[6 * 12 + 8], 0xff0000); // Ancestor document clip escaped.
        assert_eq!(canvas.pixels[10 * 12], 0xffffff); // Status area remains protected.
        assert!(!canvas.exhausted());
        assert_eq!(canvas.clip, viewport);
    }
    #[test]
    fn opaque_fast_path_respects_fractional_clip_edges() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        canvas.set_clip(Rect {
            x: 2.5,
            y: 2.5,
            width: 3.0,
            height: 3.0,
        });
        canvas.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            Color::BLACK,
            0.0,
        );
        assert_eq!(canvas.pixels[3 * 8 + 2], 0xffffff);
        assert_eq!(canvas.pixels[2 * 8 + 3], 0xffffff);
        assert_eq!(canvas.pixels[3 * 8 + 3], 0);
        assert_eq!(canvas.pixels[5 * 8 + 5], 0);
        assert_eq!(canvas.pixels[6 * 8 + 5], 0xffffff);
    }
    #[test]
    fn nested_display_list_clips_intersect_and_restore_caller_viewport() {
        let mut canvas = Canvas::new(10, 10).unwrap();
        canvas.set_clip(Rect {
            x: 2.0,
            y: 2.0,
            width: 6.0,
            height: 6.0,
        });
        let fill = |color| DrawCommand::Rect {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            color,
            radius: 0.0,
        };
        let commands = [
            DrawCommand::PushClip {
                rect: Rect {
                    x: 4.0,
                    y: 4.0,
                    width: 8.0,
                    height: 8.0,
                },
            },
            fill(Color::rgb(255, 0, 0)),
            DrawCommand::PushClip {
                rect: Rect {
                    x: 5.0,
                    y: 5.0,
                    width: 1.0,
                    height: 1.0,
                },
            },
            fill(Color::rgb(0, 0, 255)),
            DrawCommand::PopClip,
            DrawCommand::Rect {
                rect: Rect {
                    x: 7.0,
                    y: 7.0,
                    width: 2.0,
                    height: 2.0,
                },
                color: Color::rgb(0, 255, 0),
                radius: 0.0,
            },
            DrawCommand::PopClip,
            DrawCommand::Rect {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 3.0,
                    height: 3.0,
                },
                color: Color::rgb(255, 255, 0),
                radius: 0.0,
            },
        ];
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert_eq!(canvas.pixels[4 * 10 + 4], 0xff0000);
        assert_eq!(canvas.pixels[5 * 10 + 5], 0x0000ff);
        assert_eq!(canvas.pixels[7 * 10 + 7], 0x00ff00);
        assert_eq!(canvas.pixels[2 * 10 + 2], 0xffff00);
        assert_eq!(canvas.pixels[8 * 10 + 8], 0xffffff);
        canvas.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            Color::BLACK,
            0.0,
        );
        assert_eq!(canvas.pixels[2 * 10 + 2], 0);
        assert_eq!(canvas.pixels[11], 0xffffff);
        assert_eq!(canvas.pixels[8 * 10 + 8], 0xffffff);
    }

    #[test]
    fn malformed_or_excessive_clip_scopes_cannot_escape_caller_clip() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        let caller = Rect {
            x: 2.0,
            y: 2.0,
            width: 4.0,
            height: 4.0,
        };
        canvas.set_clip(caller);
        canvas.paint(
            &[DrawCommand::PushClip {
                rect: Rect {
                    x: 3.0,
                    y: 3.0,
                    width: 1.0,
                    height: 1.0,
                },
            }],
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.clip, caller);
        let commands = vec![DrawCommand::PushClip { rect: caller }; MAX_CLIP_DEPTH + 1];
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert!(canvas.exhausted());
        assert_eq!(canvas.clip, caller);
        canvas.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            Color::BLACK,
            0.0,
        );
        assert_eq!(canvas.pixels[2 * 8 + 2], 0);
        assert_eq!(canvas.pixels[0], 0xffffff);
    }

    #[test]
    fn clip_coordinates_follow_display_list_translation() {
        let mut canvas = Canvas::new(10, 10).unwrap();
        let commands = [
            DrawCommand::PushClip {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 2.0,
                    height: 2.0,
                },
            },
            DrawCommand::Rect {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                color: Color::BLACK,
                radius: 0.0,
            },
            DrawCommand::PopClip,
        ];
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 3.0, 4.0);
        assert_eq!(canvas.pixels[4 * 10 + 3], 0);
        assert_eq!(canvas.pixels[5 * 10 + 4], 0);
        assert_eq!(canvas.pixels[3 * 10 + 3], 0xffffff);
        assert_eq!(canvas.pixels[4 * 10 + 5], 0xffffff);
    }
    #[test]
    fn clipping_and_alpha() {
        let mut c = Canvas::new(8, 8).unwrap();
        c.clear(Color::WHITE);
        c.set_clip(Rect {
            x: 2.0,
            y: 2.0,
            width: 3.0,
            height: 3.0,
        });
        c.rect(
            Rect {
                x: -10.0,
                y: -10.0,
                width: 30.0,
                height: 30.0,
            },
            Color::rgba(0, 0, 0, 128),
            0.0,
        );
        assert_eq!(c.pixels[0], 0xffffff);
        assert_eq!(c.pixels[3 * 8 + 3], 0x7f7f7f);
        assert_eq!(c.pixels[7 * 8 + 7], 0xffffff);
    }
    #[test]
    fn huge_viewports_rejected() {
        assert!(Canvas::new(u32::MAX, 4).is_err());
        assert!(Canvas::new(8192, 8192).is_err());
        assert!(Canvas::new(0, 100).is_err());
    }
    #[test]
    fn measured_text_is_painted() {
        let f = Fonts::new();
        assert!(f.measure("Hello", 16.0, false, false, false) > 30.0);
        let mut c = Canvas::new(100, 30).unwrap();
        c.text(
            &f,
            2.0,
            2.0,
            "Hello",
            16.0,
            Color::BLACK,
            false,
            false,
            false,
        );
        assert!(c.pixels.iter().any(|p| *p != 0xffffff));
    }

    #[test]
    fn overlapping_page_paint_is_bounded_and_chrome_still_paints() {
        let fonts = Fonts::new();
        let mut canvas = Canvas::new(64, 64).unwrap();
        let command = DrawCommand::Rect {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 64.0,
                height: 64.0,
            },
            color: Color::rgba(20, 40, 60, 180),
            radius: 8.0,
        };
        canvas.paint(&vec![command; 1_000], &fonts, &ImageStore::new(), 0.0, 0.0);
        assert!(canvas.exhausted());
        canvas.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 64.0,
                height: 8.0,
            },
            Color::WHITE,
            0.0,
        );
        assert_eq!(canvas.pixels[3 * 64 + 30], 0xffffff);
        canvas.paint(
            &[DrawCommand::Rect {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 64.0,
                    height: 64.0,
                },
                color: Color::rgb(0, 0, 255),
                radius: 0.0,
            }],
            &fonts,
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert!(!canvas.exhausted());
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0x0000ff));
    }

    #[test]
    fn offscreen_shapes_do_not_consume_visible_pixel_budget() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        let commands = vec![
            DrawCommand::Rect {
                rect: Rect {
                    x: 1000.0,
                    y: 1000.0,
                    width: 1_000_000.0,
                    height: 1_000_000.0
                },
                color: Color::BLACK,
                radius: 10.0
            };
            1000
        ];
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
    }

    #[test]
    fn malformed_raster_dimensions_are_rejected_before_indexing() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        let image = RasterImage {
            width: u32::MAX,
            height: u32::MAX,
            rgba: vec![],
        };
        canvas.image(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            &image,
        );
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
    }

    #[test]
    fn enormous_italic_size_is_clamped_before_coordinate_math() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        canvas.text(
            &Fonts::new(),
            0.0,
            0.0,
            "I",
            f32::MAX,
            Color::BLACK,
            false,
            true,
            false,
        );
        assert_eq!(canvas.pixels.len(), 64);
    }

    #[test]
    fn zero_advance_text_has_a_glyph_work_limit() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        let command = DrawCommand::Text {
            x: 0.0,
            y: 0.0,
            text: "\u{200d}".repeat(MAX_TEXT_GLYPHS + 1),
            size: 12.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        };
        canvas.paint(&[command], &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert!(canvas.exhausted());
    }
}
