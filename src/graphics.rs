//! Software display-list painter. No browser engine or GPU is involved.
use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use std::{cell::RefCell, collections::HashMap, path::Path, sync::Arc};

pub mod text_masks;

const MAX_PAINT_COMMANDS: usize = 200_000;
const MAX_PAINT_GLYPHS: usize = 100_000;
const MAX_TEXT_GLYPHS: usize = 32_768;
const MAX_PAINT_PIXELS: u64 = 32_000_000;
const MAX_CLIP_DEPTH: usize = 128;
const MAX_LAYER_PIXELS: usize = 8_388_608;
const MAX_LAYER_ALLOCATED_PIXELS: usize = 16_777_216;

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
    /// Composite enclosed commands as one isolated, transparent group.
    PushOpacity {
        opacity: f32,
    },
    PopOpacity,
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

/// Pixels are premultiplied RGBA16; the public window framebuffer remains RGB.
/// A layer covers the caller viewport, rather than the current clip: a fixed
/// descendant may legitimately escape that clip while retaining group opacity.
struct OpacityLayer {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    pixels: LayerPixels,
    opacity: f32,
}

enum LayerPixels {
    Pending,
    Suppressed,
    Allocated(Vec<u64>),
}

fn premultiplied_over(source: u64, destination: u64) -> u64 {
    let inverse = 65_535 - (source >> 48);
    let channel = |shift: u32| {
        ((source >> shift) & 65_535u64)
            + (((destination >> shift) & 65_535u64) * inverse + 32_767) / 65_535
    };
    (channel(48) << 48) | (channel(32) << 32) | (channel(16) << 16) | channel(0)
}

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
    clip: Rect,
    budget: Option<PaintBudget>,
    paint_exhausted: bool,
    layers: Vec<OpacityLayer>,
    // The first pending ancestor is cached: ordinary pixels do not scan the
    // layer stack. Only a first visible, nontransparent contribution allocates.
    first_pending: Option<usize>,
    layer_allocated: usize,
    layer_live: usize,
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
            layers: Vec::new(),
            first_pending: None,
            layer_allocated: 0,
            layer_live: 0,
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

    fn suppressed(&self) -> bool {
        self.layers
            .last()
            .is_some_and(|layer| matches!(layer.pixels, LayerPixels::Suppressed))
    }

    fn begin_opacity(&mut self, opacity: f32, viewport: Rect) {
        let x = viewport.x.ceil().max(0.0) as usize;
        let y = viewport.y.ceil().max(0.0) as usize;
        let width = ((viewport.x + viewport.width).ceil().max(0.0) as usize).saturating_sub(x);
        let height = ((viewport.y + viewport.height).ceil().max(0.0) as usize).saturating_sub(y);
        let pixels = if opacity == 0.0 || self.suppressed() || width == 0 || height == 0 {
            LayerPixels::Suppressed
        } else {
            self.first_pending.get_or_insert(self.layers.len());
            LayerPixels::Pending
        };
        self.layers.push(OpacityLayer {
            x,
            y,
            width,
            height,
            pixels,
            opacity,
        });
    }

    fn materialize_layers(&mut self) -> bool {
        if self.paint_exhausted {
            return false;
        }
        let Some(first) = self.first_pending else {
            return true;
        };
        // Allocate outer ancestors before children. A child can then composite
        // without allocating its parent while its own surface remains live.
        for index in first..self.layers.len() {
            let layer = &self.layers[index];
            if !matches!(layer.pixels, LayerPixels::Pending) {
                self.paint_exhausted = true;
                return false;
            }
            let count = layer.width * layer.height;
            if count > MAX_LAYER_PIXELS.saturating_sub(self.layer_live)
                || count > MAX_LAYER_ALLOCATED_PIXELS.saturating_sub(self.layer_allocated)
                || !self.consume_pixels(count as u64)
            {
                self.paint_exhausted = true;
                return false;
            }
            let mut pixels = Vec::new();
            if pixels.try_reserve_exact(count).is_err() {
                self.paint_exhausted = true;
                return false;
            }
            pixels.resize(count, 0);
            self.layer_live += count;
            self.layer_allocated += count;
            self.layers[index].pixels = LayerPixels::Allocated(pixels);
        }
        self.first_pending = None;
        true
    }

    fn end_opacity(&mut self) -> bool {
        let Some(layer) = self.layers.pop() else {
            self.paint_exhausted = true;
            return false;
        };
        if self
            .first_pending
            .is_some_and(|first| first >= self.layers.len())
        {
            self.first_pending = None;
        }
        let LayerPixels::Allocated(pixels) = layer.pixels else {
            return true;
        };
        self.layer_live = self.layer_live.saturating_sub(pixels.len());
        if !self.consume_pixels(pixels.len() as u64) {
            return false;
        }
        if self.layers.last().is_some_and(|parent| {
            !matches!(&parent.pixels, LayerPixels::Allocated(parent_pixels) if parent_pixels.len() == pixels.len())
        }) {
            self.paint_exhausted = true;
            return false;
        }
        for (index, pixel) in pixels.into_iter().enumerate() {
            // Keep opacity unquantized until compositing. RGBA16 retains enough
            // intermediate precision to avoid dark fringes and repeated 8-bit
            // rounding through nested translucent groups.
            let inverse = 1.0 - (pixel >> 48) as f32 / 65_535.0 * layer.opacity;
            let channel = |shift: u32, destination: f32| {
                ((pixel >> shift) & 65_535u64) as f32 * layer.opacity + destination * inverse
            };
            let x = layer.x + index % layer.width;
            let y = layer.y + index / layer.width;
            if let Some(parent) = self.layers.last_mut() {
                // All nonempty layers use the same caller-viewport rectangle.
                if let LayerPixels::Allocated(parent_pixels) = &mut parent.pixels {
                    let destination = &mut parent_pixels[index];
                    let component = |shift: u32| {
                        channel(shift, ((*destination >> shift) & 65_535u64) as f32).round() as u64
                    };
                    *destination = (component(48) << 48)
                        | (component(32) << 32)
                        | (component(16) << 16)
                        | component(0);
                }
            } else {
                let destination = &mut self.pixels[y * self.width as usize + x];
                let component = |source_shift: u32, destination_shift: u32| {
                    (channel(
                        source_shift,
                        (((*destination >> destination_shift) & 255u32) * 257) as f32,
                    ) / 257.0)
                        .round() as u32
                };
                *destination =
                    (component(32, 16) << 16) | (component(16, 8) << 8) | component(0, 0);
            }
        }
        true
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
        if let Some(layer) = self.layers.last() {
            if matches!(layer.pixels, LayerPixels::Suppressed) {
                return;
            }
            let x = x as usize;
            let y = y as usize;
            if x < layer.x
                || y < layer.y
                || x >= layer.x + layer.width
                || y >= layer.y + layer.height
            {
                return;
            }
            if !self.materialize_layers() {
                return;
            }
            let layer = self
                .layers
                .last_mut()
                .expect("opacity layer is still present");
            let LayerPixels::Allocated(pixels) = &mut layer.pixels else {
                self.paint_exhausted = true;
                return;
            };
            let alpha = u64::from(a) * 257;
            let source = (alpha << 48)
                | (((u64::from(color.r) * alpha + 127) / 255) << 32)
                | (((u64::from(color.g) * alpha + 127) / 255) << 16)
                | ((u64::from(color.b) * alpha + 127) / 255);
            let pixel = &mut pixels[(y - layer.y) * layer.width + x - layer.x];
            *pixel = premultiplied_over(source, *pixel);
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
        if self.suppressed()
            || ![rect.x, rect.y, rect.width, rect.height, radius]
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
        if radius == 0.0 && color.a == 255 && self.layers.is_empty() {
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
        if self.suppressed()
            || !x.is_finite()
            || !y.is_finite()
            || !size.is_finite()
            || color.a == 0
        {
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
        if self.suppressed()
            || rect.width <= 0.0
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
            Opacity { layer: bool },
        }
        self.paint_exhausted = false;
        self.layers.clear();
        self.first_pending = None;
        self.layer_allocated = 0;
        self.layer_live = 0;
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
                DrawCommand::PushOpacity { opacity } => {
                    if scopes.len() >= MAX_CLIP_DEPTH
                        || !opacity.is_finite()
                        || !(0.0..=1.0).contains(opacity)
                    {
                        self.paint_exhausted = true;
                        break;
                    }
                    let layer = *opacity < 1.0;
                    if layer {
                        self.begin_opacity(*opacity, caller_clip);
                    }
                    scopes.push(Scope::Opacity { layer });
                }
                DrawCommand::PopOpacity => {
                    let Some(Scope::Opacity { layer }) = scopes.pop() else {
                        self.paint_exhausted = true;
                        break;
                    };
                    if layer && !self.end_opacity() {
                        break;
                    }
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
        self.paint_exhausted |= !scopes.is_empty();
        // Unfinished groups never leak partially composited layers into the UI
        // or retain temporary memory after invalid input or quota exhaustion.
        self.layers.clear();
        self.first_pending = None;
        self.layer_live = 0;
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
    fn colored_rect(x: f32, y: f32, width: f32, height: f32, color: Color) -> DrawCommand {
        DrawCommand::Rect {
            rect: Rect {
                x,
                y,
                width,
                height,
            },
            color,
            radius: 0.0,
        }
    }

    #[test]
    fn group_opacity_composites_overlap_once_and_preserves_transparent_holes() {
        let mut canvas = Canvas::new(8, 4).unwrap();
        canvas.paint(
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(0.0, 0.0, 4.0, 4.0, Color::rgb(255, 0, 0)),
                colored_rect(2.0, 0.0, 4.0, 4.0, Color::rgb(0, 0, 255)),
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[0], 0xff8080);
        assert_eq!(canvas.pixels[2], 0x8080ff);
        assert_eq!(canvas.pixels[5], 0x8080ff);
        assert_eq!(canvas.pixels[6], 0xffffff);
        assert!(!canvas.exhausted());
        assert!(canvas.layers.is_empty());
    }

    #[test]
    fn nested_opacity_and_translucent_images_use_premultiplied_alpha() {
        let mut images = ImageStore::new();
        images.insert(
            "alpha".into(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 128],
            }),
        );
        let mut canvas = Canvas::new(8, 4).unwrap();
        canvas.paint(
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(0.0, 0.0, 2.0, 4.0, Color::rgb(255, 0, 0)),
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(0.0, 0.0, 4.0, 4.0, Color::rgb(0, 0, 255)),
                DrawCommand::PopOpacity,
                DrawCommand::Image {
                    rect: Rect {
                        x: 4.0,
                        y: 0.0,
                        width: 2.0,
                        height: 4.0,
                    },
                    key: "alpha".into(),
                },
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &images,
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[0], 0xbf80bf);
        assert_eq!(canvas.pixels[2], 0xbfbfff);
        assert_eq!(canvas.pixels[4], 0xffbfbf);
        assert_eq!(canvas.pixels[6], 0xffffff);
        assert!(!canvas.exhausted());
    }

    #[test]
    fn opacity_groups_include_text_coverage_and_opaque_background() {
        let fonts = Fonts::new();
        let content = vec![
            colored_rect(0.0, 0.0, 64.0, 32.0, Color::rgb(200, 80, 20)),
            DrawCommand::Text {
                x: 2.0,
                y: 0.0,
                text: "Hi".into(),
                size: 24.0,
                color: Color::BLACK,
                bold: false,
                italic: false,
                monospace: false,
            },
        ];
        let mut direct = Canvas::new(64, 32).unwrap();
        direct.paint(&content, &fonts, &ImageStore::new(), 0.0, 0.0);
        let mut grouped = Canvas::new(64, 32).unwrap();
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.5 }];
        commands.extend(content);
        commands.push(DrawCommand::PopOpacity);
        grouped.paint(&commands, &fonts, &ImageStore::new(), 0.0, 0.0);
        assert!(direct.pixels.contains(&0));
        for (before, after) in direct.pixels.iter().zip(&grouped.pixels) {
            for shift in [0, 8, 16] {
                let expected = ((((before >> shift) & 255) as f32 + 255.0) * 0.5).round() as i32;
                let actual = ((after >> shift) & 255) as i32;
                assert!((expected - actual).abs() <= 1);
            }
        }
        assert!(!grouped.exhausted());
    }

    #[test]
    fn fixed_descendants_escape_clips_but_retain_group_opacity() {
        let mut canvas = Canvas::new(12, 12).unwrap();
        let viewport = Rect {
            x: 0.0,
            y: 2.0,
            width: 12.0,
            height: 8.0,
        };
        canvas.set_clip(viewport);
        canvas.paint_with_viewport(
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PushClip {
                    rect: Rect {
                        x: 0.0,
                        y: 8.0,
                        width: 3.0,
                        height: 3.0,
                    },
                },
                DrawCommand::PushFixed,
                colored_rect(0.0, 0.0, 12.0, 12.0, Color::rgb(255, 0, 0)),
                DrawCommand::PushClip {
                    rect: Rect {
                        x: 4.0,
                        y: 1.0,
                        width: 2.0,
                        height: 2.0,
                    },
                },
                colored_rect(0.0, 0.0, 12.0, 12.0, Color::rgb(0, 0, 255)),
                DrawCommand::PopClip,
                DrawCommand::PopFixed,
                DrawCommand::PopClip,
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &ImageStore::new(),
            (0.0, -6.0),
            (0.0, 2.0),
        );
        assert_eq!(canvas.pixels[12], 0xffffff);
        assert_eq!(canvas.pixels[3 * 12 + 4], 0x8080ff);
        assert_eq!(canvas.pixels[6 * 12 + 8], 0xff8080);
        assert_eq!(canvas.pixels[10 * 12], 0xffffff);
        assert_eq!(canvas.clip, viewport);
        assert!(!canvas.exhausted());
    }

    #[test]
    fn invalid_and_exhausted_opacity_streams_release_surfaces_and_restore_caller() {
        let fonts = Fonts::new();
        for tail in [
            vec![],
            vec![DrawCommand::PopClip],
            vec![DrawCommand::PushOpacity { opacity: f32::NAN }],
            vec![DrawCommand::PushOpacity { opacity: -0.1 }],
            vec![DrawCommand::PushOpacity { opacity: 1.1 }],
            vec![DrawCommand::PushOpacity { opacity: 0.5 }; MAX_CLIP_DEPTH],
        ] {
            let mut canvas = Canvas::new(128, 128).unwrap();
            let viewport = Rect {
                x: 2.0,
                y: 2.0,
                width: 124.0,
                height: 124.0,
            };
            canvas.set_clip(viewport);
            let mut commands = vec![
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(0.0, 0.0, 128.0, 128.0, Color::BLACK),
            ];
            commands.extend(tail);
            canvas.paint(&commands, &fonts, &ImageStore::new(), 0.0, 0.0);
            assert!(canvas.exhausted());
            assert!(canvas.layers.is_empty());
            assert_eq!(canvas.clip, viewport);
            assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
            canvas.rect(viewport, Color::BLACK, 0.0);
            assert_eq!(canvas.pixels[2 * 128 + 2], 0);
        }
    }

    #[test]
    fn opacity_storage_caps_are_checked_before_allocation_and_zero_groups_allocate_nothing() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        let viewport = canvas.clip;
        for peak_limit in [true, false] {
            let mut limited = Canvas::new(8, 8).unwrap();
            if peak_limit {
                limited.layer_live = MAX_LAYER_PIXELS - 64;
            } else {
                limited.layer_allocated = MAX_LAYER_ALLOCATED_PIXELS - 64;
            }
            limited.begin_opacity(0.5, viewport);
            assert!(matches!(limited.layers[0].pixels, LayerPixels::Pending));
            limited.blend(0, 0, Color::BLACK, 255);
            assert!(
                matches!(&limited.layers[0].pixels, LayerPixels::Allocated(pixels) if pixels.len() == 64)
            );
            let allocated_before_failure = limited.layer_allocated;
            let live_before_failure = limited.layer_live;
            if !peak_limit {
                // Completing a layer releases live storage, but must not
                // refund the cumulative allocation budget.
                assert!(limited.end_opacity());
                assert_eq!(limited.layer_live, 0);
            }
            limited.begin_opacity(0.5, viewport);
            limited.blend(0, 0, Color::BLACK, 255);
            assert!(limited.exhausted());
            assert!(matches!(
                limited.layers.last().unwrap().pixels,
                LayerPixels::Pending
            ));
            assert_eq!(limited.layer_allocated, allocated_before_failure);
            if peak_limit {
                assert_eq!(limited.layer_live, live_before_failure);
            }
        }
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.0 }; 128];
        commands.extend((0..1000).map(|_| colored_rect(0.0, 0.0, 1e6, 1e6, Color::BLACK)));
        commands.extend(vec![DrawCommand::PopOpacity; 128]);
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert!(canvas.layers.is_empty());
        assert_eq!(canvas.layer_allocated, 0);
        assert_eq!(canvas.layer_live, 0);
        assert!(canvas.first_pending.is_none());
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
    }

    #[test]
    fn nested_viewport_surfaces_hit_peak_storage_before_the_pixel_work_limit() {
        // Seven RGBA16 viewports need 68.36 MiB simultaneously. Their combined
        // initialization/compositing work is 17.92M pixels, below this canvas's
        // 20.48M work allowance, so the independent peak-storage cap must win.
        let mut canvas = Canvas::new(1600, 800).unwrap();
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.5 }; 7];
        commands.push(colored_rect(0.0, 0.0, 1.0, 1.0, Color::BLACK));
        commands.extend(vec![DrawCommand::PopOpacity; 7]);
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert!(canvas.exhausted());
        assert!(canvas.layers.is_empty());
        assert_eq!(canvas.layer_allocated, 6 * 1600 * 800);
        assert_eq!(canvas.layer_live, 0);
        assert!(canvas.first_pending.is_none());
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
        // A failed materialization does not disable direct toolbar painting.
        canvas.rect(canvas.clip, Color::WHITE, 0.0);
        // Opacity-one scopes need no surface and preserve ordinary paint.
        canvas.paint(
            &[
                DrawCommand::PushOpacity { opacity: 1.0 },
                colored_rect(0.0, 0.0, 1.0, 1.0, Color::BLACK),
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert!(!canvas.exhausted());
        assert_eq!(canvas.pixels[0], 0);
        assert_eq!(canvas.layer_allocated, 0);
    }

    #[test]
    fn empty_nested_opacity_groups_do_not_allocate_or_charge_surface_work() {
        // Eager allocation would exceed peak storage on the seventh group.
        let mut canvas = Canvas::new(1600, 800).unwrap();
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.5 }; MAX_CLIP_DEPTH];
        commands.extend(vec![DrawCommand::PopOpacity; MAX_CLIP_DEPTH]);
        canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert_eq!(canvas.layer_allocated, 0);
        assert_eq!(canvas.layer_live, 0);
        assert!(canvas.first_pending.is_none());
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
    }

    #[test]
    fn offscreen_and_fully_transparent_opacity_content_keeps_surfaces_pending() {
        let mut images = ImageStore::new();
        images.insert(
            "clear".into(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 0],
            }),
        );
        let mut canvas = Canvas::new(64, 32).unwrap();
        let text = |y, text: &str| DrawCommand::Text {
            x: 0.0,
            y,
            text: text.into(),
            size: 16.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        };
        canvas.paint(
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(0.0, 64.0, 64.0, 32.0, Color::BLACK),
                text(64.0, "offscreen"),
                DrawCommand::Image {
                    rect: Rect {
                        x: 0.0,
                        y: 64.0,
                        width: 64.0,
                        height: 32.0,
                    },
                    key: "clear".into(),
                },
                // These commands intersect the viewport but contain no ink.
                colored_rect(0.0, 0.0, 64.0, 32.0, Color::rgba(255, 0, 0, 0)),
                text(0.0, "    "),
                DrawCommand::Image {
                    rect: canvas.clip,
                    key: "clear".into(),
                },
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &images,
            0.0,
            0.0,
        );
        assert!(!canvas.exhausted());
        assert_eq!(canvas.layer_allocated, 0);
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
    }

    #[test]
    fn first_visible_pixel_materializes_pending_ancestors_once() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        canvas.paint(
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PopOpacity,
                DrawCommand::PushOpacity { opacity: 0.0 },
                colored_rect(0.0, 0.0, 8.0, 8.0, Color::BLACK),
                DrawCommand::PopOpacity,
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(0.0, 0.0, 1.0, 1.0, Color::rgb(255, 0, 0)),
                colored_rect(1.0, 0.0, 1.0, 1.0, Color::rgb(255, 0, 0)),
                DrawCommand::PopOpacity,
                colored_rect(2.0, 0.0, 1.0, 1.0, Color::BLACK),
                DrawCommand::PushOpacity { opacity: 0.5 },
                colored_rect(3.0, 0.0, 1.0, 1.0, Color::rgb(0, 0, 255)),
                DrawCommand::PopOpacity,
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert!(!canvas.exhausted());
        assert_eq!(canvas.layer_allocated, 3 * 64);
        assert_eq!(canvas.layer_live, 0);
        assert!(canvas.first_pending.is_none());
        assert_eq!(
            &canvas.pixels[..5],
            &[0xffbfbf, 0xffbfbf, 0x808080, 0xbfbfff, 0xffffff]
        );
    }

    #[test]
    fn pending_groups_materialize_for_fixed_descendants_of_empty_document_clips() {
        let mut canvas = Canvas::new(8, 8).unwrap();
        let viewport = Rect {
            x: 1.0,
            y: 2.0,
            width: 6.0,
            height: 4.0,
        };
        canvas.set_clip(viewport);
        canvas.paint_with_viewport(
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PushClip {
                    rect: Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 8.0,
                        height: 8.0,
                    },
                },
                colored_rect(0.0, 0.0, 8.0, 8.0, Color::BLACK),
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PushFixed,
                colored_rect(0.0, 0.0, 8.0, 8.0, Color::rgb(255, 0, 0)),
                DrawCommand::PopFixed,
                DrawCommand::PopOpacity,
                DrawCommand::PopClip,
                DrawCommand::PopOpacity,
            ],
            &Fonts::new(),
            &ImageStore::new(),
            (0.0, -100.0),
            (0.0, 0.0),
        );
        assert!(!canvas.exhausted());
        assert_eq!(canvas.layer_allocated, 2 * 6 * 4);
        assert_eq!(canvas.clip, viewport);
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    canvas.pixels[y * 8 + x],
                    if viewport.contains(x as f32, y as f32) {
                        0xffbfbf
                    } else {
                        0xffffff
                    }
                );
            }
        }
    }

    #[test]
    fn transparent_drawing_still_charges_work_and_failed_materialization_is_discarded() {
        let mut canvas = Canvas::new(64, 64).unwrap();
        let mut images = ImageStore::new();
        images.insert(
            "clear".into(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 0],
            }),
        );
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.5 }; 2];
        // Transparent source pixels require sampling work, but no surface.
        // 243 * 4096 samples leave enough of the 1M budget for one surface,
        // but not two. No first source pixel may escape the unfinished group.
        commands.extend((0..243).map(|_| DrawCommand::Image {
            rect: canvas.clip,
            key: "clear".into(),
        }));
        commands.push(colored_rect(0.0, 0.0, 1.0, 1.0, Color::BLACK));
        commands.extend(vec![DrawCommand::PopOpacity; 2]);
        canvas.paint(&commands, &Fonts::new(), &images, 0.0, 0.0);
        assert!(canvas.exhausted());
        assert_eq!(canvas.layer_allocated, 4096);
        assert_eq!(canvas.layer_live, 0);
        assert!(canvas.first_pending.is_none());
        assert!(canvas.layers.is_empty());
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0xffffff));
        canvas.rect(canvas.clip, Color::BLACK, 0.0);
        assert!(canvas.pixels.iter().all(|pixel| *pixel == 0));
    }

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
