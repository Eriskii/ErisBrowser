//! Independent, bounded SVG geometry and software rasterization.
//!
//! Supports common shapes, paths, transforms, solid paints, and text. Filters,
//! gradients, masks, embedded content, scripts, and external references are not
//! evaluated. This module does not implement the complete SVG specification.

use crate::css::parse_color;
use crate::dom::{Document, Namespace, NodeId};
use crate::graphics::{Color, RasterImage};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use std::collections::BTreeMap;

const MAX_SOURCE: usize = 512 * 1024;
const MAX_ELEMENTS: usize = 4096;
const MAX_POINTS: usize = 32_768;
const MAX_WORK: usize = 48_000_000;
const MAX_COORDINATE: f32 = 1_000_000.0;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, Default)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
    fn distance(self, other: Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Clone, Copy, Debug)]
struct Matrix {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}
impl Matrix {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    fn translate(x: f32, y: f32) -> Self {
        Self {
            e: x,
            f: y,
            ..Self::IDENTITY
        }
    }
    fn scale(x: f32, y: f32) -> Self {
        Self {
            a: x,
            d: y,
            ..Self::IDENTITY
        }
    }
    fn multiply(self, rhs: Self) -> Self {
        Self {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            e: self.a * rhs.e + self.c * rhs.f + self.e,
            f: self.b * rhs.e + self.d * rhs.f + self.f,
        }
    }
    fn apply(self, p: Point) -> Point {
        Point::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
        )
    }
    fn valid(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
    }
    fn inverse(self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if !det.is_finite() || det.abs() < 1e-10 {
            return None;
        }
        Some(Self {
            a: self.d / det,
            b: -self.b / det,
            c: -self.c / det,
            d: self.a / det,
            e: (self.c * self.f - self.d * self.e) / det,
            f: (self.b * self.e - self.a * self.f) / det,
        })
    }
    fn magnitude(self) -> f32 {
        ((self.a.hypot(self.b) + self.c.hypot(self.d)) * 0.5).max(0.0)
    }
}

#[derive(Clone, Debug)]
struct Style {
    fill: Option<Color>,
    stroke: Option<Color>,
    color: Color,
    stroke_width: f32,
    opacity: f32,
    fill_opacity: f32,
    stroke_opacity: f32,
    evenodd: bool,
    font_size: f32,
    bold: bool,
    anchor: String,
    hidden: bool,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Some(Color::BLACK),
            stroke: None,
            color: Color::BLACK,
            stroke_width: 1.0,
            opacity: 1.0,
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
            evenodd: false,
            font_size: 16.0,
            bold: false,
            anchor: "start".into(),
            hidden: false,
        }
    }
}
impl Style {
    fn derive(&self, doc: &Document, id: NodeId) -> Self {
        let mut result = self.clone();
        let mut attrs = BTreeMap::new();
        for name in [
            "fill",
            "stroke",
            "color",
            "stroke-width",
            "opacity",
            "fill-opacity",
            "stroke-opacity",
            "fill-rule",
            "font-size",
            "font-weight",
            "text-anchor",
            "display",
            "visibility",
        ] {
            if let Some(value) = doc.attr(id, name) {
                attrs.insert(name.to_owned(), value.to_owned());
            }
        }
        for declaration in doc.attr(id, "style").unwrap_or("").split(';') {
            if let Some((key, value)) = declaration.split_once(':') {
                attrs.insert(key.trim().to_ascii_lowercase(), value.trim().to_owned());
            }
        }
        if let Some(color) = attrs.get("color").and_then(|v| parse_color(v)) {
            result.color = color;
        }
        for (key, value) in attrs {
            let paint = || {
                if value.trim().eq_ignore_ascii_case("currentColor") {
                    Some(result.color)
                } else {
                    parse_color(value.trim())
                }
            };
            match key.as_str() {
                "fill" => result.fill = paint(),
                "stroke" => result.stroke = paint(),
                "stroke-width" => {
                    result.stroke_width = length(&value)
                        .unwrap_or(result.stroke_width)
                        .clamp(0.0, 1024.0)
                }
                "opacity" => result.opacity *= unit(&value),
                "fill-opacity" => result.fill_opacity = unit(&value),
                "stroke-opacity" => result.stroke_opacity = unit(&value),
                "fill-rule" => result.evenodd = value.trim() == "evenodd",
                "font-size" => {
                    result.font_size = length(&value).unwrap_or(result.font_size).clamp(1.0, 512.0)
                }
                "font-weight" => {
                    result.bold = value == "bold" || value.parse::<u32>().is_ok_and(|n| n >= 600)
                }
                "text-anchor" => result.anchor = value,
                "display" if value.trim() == "none" => result.hidden = true,
                "visibility" if value.trim() == "hidden" || value.trim() == "collapse" => {
                    result.hidden = true
                }
                _ => {}
            }
        }
        result
    }
    fn fill_color(&self) -> Option<Color> {
        self.fill.map(|mut color| {
            color.a = (color.a as f32 * self.opacity * self.fill_opacity).round() as u8;
            color
        })
    }
    fn stroke_color(&self) -> Option<Color> {
        self.stroke.map(|mut color| {
            color.a = (color.a as f32 * self.opacity * self.stroke_opacity).round() as u8;
            color
        })
    }
}

#[derive(Default)]
struct Subpath {
    points: Vec<Point>,
    closed: bool,
}

/// Render SVG into a transparent RGBA image without browser-engine dependencies.
pub fn render(source: &str, width: Option<u32>, height: Option<u32>) -> Result<RasterImage> {
    render_with_budget(source, width, height, crate::page::MAX_DECODED_IMAGE_BYTES)
}
pub(crate) fn render_with_budget(
    source: &str,
    width: Option<u32>,
    height: Option<u32>,
    budget: usize,
) -> Result<RasterImage> {
    if budget < 4 {
        return Err("decoded image budget exhausted".into());
    }
    if source.len() > MAX_SOURCE {
        return Err("SVG source exceeds 512 KiB".into());
    }
    let document = Document::parse(source);
    if document.nodes.len() > MAX_ELEMENTS {
        return Err("SVG element limit exceeded".into());
    }
    let root = document
        .query_selector_all("svg")
        .into_iter()
        .find(|&id| document.namespace(id) == Some(Namespace::Svg))
        .ok_or("SVG root element is missing")?;
    let viewbox = document
        .attr(root, "viewBox")
        .map(numbers)
        .transpose()?
        .filter(|v| v.len() == 4 && v[2] > 0.0 && v[3] > 0.0);
    let natural_width = document
        .attr(root, "width")
        .and_then(length)
        .or_else(|| viewbox.as_ref().map(|v| v[2]))
        .unwrap_or(300.0);
    let natural_height = document
        .attr(root, "height")
        .and_then(length)
        .or_else(|| viewbox.as_ref().map(|v| v[3]))
        .unwrap_or(150.0);
    let width = width.unwrap_or_else(|| natural_width.max(1.0).round() as u32);
    let height = height.unwrap_or_else(|| natural_height.max(1.0).round() as u32);
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) > 4_194_304
    {
        return Err("SVG raster exceeds 4 megapixels or 4096 pixels per axis".into());
    }
    if u64::from(width) * u64::from(height) * 4 > budget as u64 {
        return Err("SVG raster exceeds remaining decoded image budget".into());
    }
    let mut matrix = Matrix::IDENTITY;
    if let Some(v) = viewbox {
        let sx = width as f32 / v[2];
        let sy = height as f32 / v[3];
        let aspect = document
            .attr(root, "preserveAspectRatio")
            .unwrap_or("xMidYMid meet");
        if aspect.trim() == "none" {
            matrix = Matrix::scale(sx, sy).multiply(Matrix::translate(-v[0], -v[1]));
        } else {
            let scale = if aspect.contains("slice") {
                sx.max(sy)
            } else {
                sx.min(sy)
            };
            let dx = width as f32 - v[2] * scale;
            let dy = height as f32 - v[3] * scale;
            let x = if aspect.contains("xMin") {
                0.0
            } else if aspect.contains("xMax") {
                dx
            } else {
                dx * 0.5
            };
            let y = if aspect.contains("YMin") {
                0.0
            } else if aspect.contains("YMax") {
                dy
            } else {
                dy * 0.5
            };
            matrix = Matrix::translate(x, y)
                .multiply(Matrix::scale(scale, scale))
                .multiply(Matrix::translate(-v[0], -v[1]));
        }
    } else if natural_width > 0.0 && natural_height > 0.0 {
        matrix = Matrix::scale(width as f32 / natural_width, height as f32 / natural_height);
    }
    let mut painter = Painter {
        image: RasterImage {
            width,
            height,
            rgba: vec![0; width as usize * height as usize * 4],
        },
        work: MAX_WORK,
        points: 0,
        font: None,
        bold_font: None,
    };
    painter.element(&document, root, matrix, &Style::default(), 0)?;
    Ok(painter.image)
}

struct Painter {
    image: RasterImage,
    work: usize,
    points: usize,
    font: Option<FontArc>,
    bold_font: Option<FontArc>,
}
impl Painter {
    fn charge(&mut self, amount: usize) -> Result<()> {
        if amount > self.work {
            self.work = 0;
            return Err("SVG raster work limit exceeded".into());
        }
        self.work -= amount;
        Ok(())
    }
    fn element(
        &mut self,
        doc: &Document,
        id: NodeId,
        parent_matrix: Matrix,
        inherited: &Style,
        depth: usize,
    ) -> Result<()> {
        self.charge(1)?;
        if depth > 96 {
            return Err("SVG nesting limit exceeded".into());
        }
        if doc.namespace(id) != Some(Namespace::Svg) {
            return Ok(());
        }
        let Some(tag) = doc.tag(id) else {
            return Ok(());
        };
        if [
            "defs",
            "symbol",
            "clipPath",
            "mask",
            "filter",
            "linearGradient",
            "radialGradient",
            "script",
            "foreignObject",
            "title",
            "desc",
            "metadata",
            "style",
        ]
        .contains(&tag)
        {
            return Ok(());
        }
        let style = inherited.derive(doc, id);
        if style.hidden || style.opacity <= 0.0 {
            return Ok(());
        }
        let matrix = parent_matrix.multiply(transform(doc.attr(id, "transform").unwrap_or(""))?);
        if !matrix.valid() {
            return Err("SVG transform exceeds coordinate limits".into());
        }
        let attr = |key: &str, default: f32| doc.attr(id, key).and_then(length).unwrap_or(default);
        let mut paths = Vec::new();
        let mut fill = true;
        match tag {
            "rect" => {
                let x = attr("x", 0.0);
                let y = attr("y", 0.0);
                let w = attr("width", 0.0);
                let h = attr("height", 0.0);
                if w > 0.0 && h > 0.0 {
                    let rx = attr("rx", attr("ry", 0.0)).max(0.0).min(w * 0.5);
                    let ry = attr("ry", rx).max(0.0).min(h * 0.5);
                    let points = if rx == 0.0 || ry == 0.0 {
                        vec![
                            Point::new(x, y),
                            Point::new(x + w, y),
                            Point::new(x + w, y + h),
                            Point::new(x, y + h),
                        ]
                    } else {
                        rounded_rect(x, y, w, h, rx, ry)
                    };
                    paths.push(Subpath {
                        points,
                        closed: true,
                    });
                }
            }
            "circle" | "ellipse" => {
                let cx = attr("cx", 0.0);
                let cy = attr("cy", 0.0);
                let rx = attr(if tag == "circle" { "r" } else { "rx" }, 0.0);
                let ry = if tag == "circle" { rx } else { attr("ry", 0.0) };
                if rx > 0.0 && ry > 0.0 {
                    let count = (rx.max(ry) * matrix.magnitude() * 0.5).clamp(32.0, 256.0) as usize;
                    let points = (0..count)
                        .map(|i| {
                            let t = i as f32 / count as f32 * std::f32::consts::TAU;
                            Point::new(cx + rx * t.cos(), cy + ry * t.sin())
                        })
                        .collect();
                    paths.push(Subpath {
                        points,
                        closed: true,
                    });
                }
            }
            "line" => {
                fill = false;
                paths.push(Subpath {
                    points: vec![
                        Point::new(attr("x1", 0.0), attr("y1", 0.0)),
                        Point::new(attr("x2", 0.0), attr("y2", 0.0)),
                    ],
                    closed: false,
                });
            }
            "polyline" | "polygon" => {
                let values = numbers(doc.attr(id, "points").unwrap_or(""))?;
                if values.len() % 2 != 0 {
                    return Err("SVG points need coordinate pairs".into());
                }
                paths.push(Subpath {
                    points: values
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|v| Point::new(v[0], v[1]))
                        .collect(),
                    closed: tag == "polygon",
                });
            }
            "path" => paths = parse_path(doc.attr(id, "d").unwrap_or(""))?,
            "text" => {
                self.text(doc, id, matrix, &style)?;
                return Ok(());
            }
            _ => {}
        }
        for path in &mut paths {
            self.points = self.points.saturating_add(path.points.len());
            if self.points > MAX_POINTS {
                return Err("SVG geometry point limit exceeded".into());
            }
            for point in &mut path.points {
                *point = matrix.apply(*point);
                if !point.x.is_finite()
                    || !point.y.is_finite()
                    || point.x.abs() > MAX_COORDINATE
                    || point.y.abs() > MAX_COORDINATE
                {
                    return Err("SVG coordinate limit exceeded".into());
                }
            }
        }
        if fill && let Some(color) = style.fill_color() {
            self.fill(&paths, color, style.evenodd)?;
        }
        if let Some(color) = style.stroke_color() {
            let width = (style.stroke_width * matrix.magnitude()).min(4096.0);
            if width > 0.0 {
                for path in &paths {
                    for points in path.points.windows(2) {
                        self.stroke(points[0], points[1], width, color)?;
                    }
                    if path.closed && path.points.len() > 1 {
                        self.stroke(*path.points.last().unwrap(), path.points[0], width, color)?;
                    }
                }
            }
        }
        for child in &doc.nodes[id].children {
            self.element(doc, *child, matrix, &style, depth + 1)?;
        }
        Ok(())
    }

    fn blend(&mut self, x: usize, y: usize, color: Color, coverage: f32) {
        let alpha = (color.a as f32 / 255.0 * coverage).clamp(0.0, 1.0);
        if alpha <= 0.0 {
            return;
        }
        let index = (y * self.image.width as usize + x) * 4;
        let pixel = &mut self.image.rgba[index..index + 4];
        let old_alpha = pixel[3] as f32 / 255.0;
        let new_alpha = alpha + old_alpha * (1.0 - alpha);
        for (channel, value) in [color.r, color.g, color.b].iter().enumerate() {
            pixel[channel] = ((*value as f32 * alpha
                + pixel[channel] as f32 * old_alpha * (1.0 - alpha))
                / new_alpha)
                .round() as u8;
        }
        pixel[3] = (new_alpha * 255.0).round() as u8;
    }
    fn fill(&mut self, paths: &[Subpath], color: Color, evenodd: bool) -> Result<()> {
        if color.a == 0 {
            return Ok(());
        }
        let mut edges = Vec::new();
        let mut min_y = self.image.height as f32;
        let mut max_y = 0.0_f32;
        for path in paths {
            if path.points.len() < 3 {
                continue;
            }
            for i in 0..path.points.len() {
                let a = path.points[i];
                let b = path.points[(i + 1) % path.points.len()];
                if a.y == b.y {
                    continue;
                }
                min_y = min_y.min(a.y.min(b.y));
                max_y = max_y.max(a.y.max(b.y));
                edges.push((a, b));
            }
        }
        let y0 = min_y.floor().clamp(0.0, self.image.height as f32) as usize;
        let y1 = max_y.ceil().clamp(0.0, self.image.height as f32) as usize;
        if edges.is_empty() || y0 >= y1 {
            return Ok(());
        }
        let width = self.image.width as usize;
        // Every row sorts intersections for two subpixel samples. Counting
        // only edge visits underestimated dense crossing paths by log2(n).
        let sort_work = edges.len().saturating_mul(edges.len().ilog2() as usize + 1);
        let sample_work = edges.len().saturating_add(sort_work).saturating_add(width);
        self.charge((y1 - y0).saturating_mul(sample_work.saturating_mul(2).saturating_add(width)))?;
        let mut row = vec![0.0_f32; width];
        let mut crossings = Vec::with_capacity(edges.len());
        for y in y0..y1 {
            row.fill(0.0);
            for offset in [0.25, 0.75] {
                let scan = y as f32 + offset;
                crossings.clear();
                for (a, b) in &edges {
                    if (a.y <= scan && b.y > scan) || (b.y <= scan && a.y > scan) {
                        let x = a.x + (scan - a.y) * (b.x - a.x) / (b.y - a.y);
                        crossings.push((x, if b.y > a.y { 1_i32 } else { -1_i32 }));
                    }
                }
                crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut winding = 0_i32;
                let mut previous = 0.0_f32;
                for (x, direction) in &crossings {
                    if if evenodd {
                        winding % 2 != 0
                    } else {
                        winding != 0
                    } {
                        let left = previous.max(0.0).min(width as f32);
                        let right = x.max(0.0).min(width as f32);
                        for (index, coverage) in row
                            .iter_mut()
                            .enumerate()
                            .take(right.ceil() as usize)
                            .skip(left.floor() as usize)
                        {
                            *coverage += (right.min(index as f32 + 1.0) - left.max(index as f32))
                                .max(0.0)
                                * 0.5;
                        }
                    }
                    winding += direction;
                    previous = *x;
                }
            }
            for (x, coverage) in row.iter().enumerate() {
                if *coverage > 0.0 {
                    self.blend(x, y, color, coverage.min(1.0));
                }
            }
        }
        Ok(())
    }
    fn stroke(&mut self, a: Point, b: Point, width: f32, color: Color) -> Result<()> {
        if color.a == 0 {
            return Ok(());
        }
        let radius = width * 0.5;
        let x0 = (a.x.min(b.x) - radius - 1.0)
            .floor()
            .clamp(0.0, self.image.width as f32) as usize;
        let x1 = (a.x.max(b.x) + radius + 1.0)
            .ceil()
            .clamp(0.0, self.image.width as f32) as usize;
        let y0 = (a.y.min(b.y) - radius - 1.0)
            .floor()
            .clamp(0.0, self.image.height as f32) as usize;
        let y1 = (a.y.max(b.y) + radius + 1.0)
            .ceil()
            .clamp(0.0, self.image.height as f32) as usize;
        self.charge((x1 - x0).saturating_mul(y1 - y0))?;
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length = dx * dx + dy * dy;
        for y in y0..y1 {
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let t = if length > 0.0 {
                    ((px - a.x) * dx + (py - a.y) * dy) / length
                } else {
                    0.0
                }
                .clamp(0.0, 1.0);
                let distance = (px - a.x - t * dx).hypot(py - a.y - t * dy);
                let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
                if coverage > 0.0 {
                    self.blend(x, y, color, coverage);
                }
            }
        }
        Ok(())
    }
    fn text(&mut self, doc: &Document, id: NodeId, matrix: Matrix, style: &Style) -> Result<()> {
        let Some(color) = style.fill_color() else {
            return Ok(());
        };
        let Some(inverse) = matrix.inverse() else {
            return Ok(());
        };
        let mut visits = crate::dom::MAX_NODES * 2;
        let text = doc
            .text_content_projection_bounded(id, 4096, &mut visits)
            .map_err(|_| "SVG text length limit exceeded")?;
        let font = if style.bold {
            &mut self.bold_font
        } else {
            &mut self.font
        };
        if font.is_none() {
            let bytes: &[u8] = if style.bold {
                include_bytes!("../assets/DejaVuSans-Bold.ttf")
            } else {
                include_bytes!("../assets/DejaVuSans.ttf")
            };
            *font = Some(FontArc::try_from_slice(bytes).map_err(|_| "invalid bundled SVG font")?);
        }
        let font = font.as_ref().unwrap().clone();
        let scale =
            style.font_size * font.height_unscaled() / font.units_per_em().unwrap_or(2048.0);
        let scaled = font.as_scaled(PxScale::from(scale));
        let attr = |key: &str| doc.attr(id, key).and_then(length).unwrap_or(0.0);
        let mut x = attr("x") + attr("dx");
        let y = attr("y") + attr("dy");
        let advance: f32 = text
            .chars()
            .map(|c| scaled.h_advance(scaled.glyph_id(c)))
            .sum();
        if style.anchor == "middle" {
            x -= advance * 0.5;
        } else if style.anchor == "end" {
            x -= advance;
        }
        let mut previous = None;
        for ch in text.chars() {
            self.charge(1)?;
            let id = scaled.glyph_id(ch);
            if let Some(previous) = previous {
                x += scaled.kern(previous, id);
            }
            if let Some(outline) =
                scaled.outline_glyph(id.with_scale_and_position(scaled.scale(), point(x, y)))
            {
                let bounds = outline.px_bounds();
                let w = bounds.width() as usize;
                let h = bounds.height() as usize;
                if w * h > 1_048_576 {
                    return Err("SVG glyph size limit exceeded".into());
                }
                self.charge(w * h)?;
                let mut bitmap = vec![0.0_f32; w * h];
                outline.draw(|gx, gy, coverage| {
                    bitmap[gy as usize * w + gx as usize] = coverage;
                });
                let corners = [
                    matrix.apply(Point::new(bounds.min.x, bounds.min.y)),
                    matrix.apply(Point::new(bounds.max.x, bounds.min.y)),
                    matrix.apply(Point::new(bounds.min.x, bounds.max.y)),
                    matrix.apply(Point::new(bounds.max.x, bounds.max.y)),
                ];
                let x0 = corners
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::INFINITY, f32::min)
                    .floor()
                    .clamp(0.0, self.image.width as f32) as usize;
                let x1 = corners
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::NEG_INFINITY, f32::max)
                    .ceil()
                    .clamp(0.0, self.image.width as f32) as usize;
                let y0 = corners
                    .iter()
                    .map(|p| p.y)
                    .fold(f32::INFINITY, f32::min)
                    .floor()
                    .clamp(0.0, self.image.height as f32) as usize;
                let y1 = corners
                    .iter()
                    .map(|p| p.y)
                    .fold(f32::NEG_INFINITY, f32::max)
                    .ceil()
                    .clamp(0.0, self.image.height as f32) as usize;
                self.charge((x1 - x0).saturating_mul(y1 - y0))?;
                for py in y0..y1 {
                    for px in x0..x1 {
                        let local = inverse.apply(Point::new(px as f32 + 0.5, py as f32 + 0.5));
                        let gx = (local.x - bounds.min.x).floor() as i32;
                        let gy = (local.y - bounds.min.y).floor() as i32;
                        if gx >= 0 && gy >= 0 && (gx as usize) < w && (gy as usize) < h {
                            self.blend(px, py, color, bitmap[gy as usize * w + gx as usize]);
                        }
                    }
                }
            }
            x += scaled.h_advance(id);
            previous = Some(id);
        }
        Ok(())
    }
}

fn unit(text: &str) -> f32 {
    text.trim().parse::<f32>().unwrap_or(1.0).clamp(0.0, 1.0)
}
fn length(text: &str) -> Option<f32> {
    let text = text.trim();
    if text.ends_with('%') {
        return None;
    }
    let (number, factor) = if let Some(number) = text.strip_suffix("px") {
        (number, 1.0)
    } else if let Some(number) = text.strip_suffix("pt") {
        (number, 96.0 / 72.0)
    } else if let Some(number) = text.strip_suffix("in") {
        (number, 96.0)
    } else if let Some(number) = text.strip_suffix("cm") {
        (number, 96.0 / 2.54)
    } else if let Some(number) = text.strip_suffix("mm") {
        (number, 96.0 / 25.4)
    } else {
        (text, 1.0)
    };
    let value = number.trim().parse::<f32>().ok()? * factor;
    (value.is_finite() && value.abs() <= MAX_COORDINATE).then_some(value)
}

struct NumberReader<'a> {
    text: &'a str,
    pos: usize,
}
impl<'a> NumberReader<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }
    fn space(&mut self) {
        while self
            .text
            .as_bytes()
            .get(self.pos)
            .is_some_and(|c| c.is_ascii_whitespace() || *c == b',')
        {
            self.pos += 1;
        }
    }
    fn number(&mut self) -> Result<f32> {
        self.space();
        let start = self.pos;
        if matches!(self.text.as_bytes().get(self.pos), Some(b'+' | b'-')) {
            self.pos += 1;
        }
        let mut digits = 0;
        while self
            .text
            .as_bytes()
            .get(self.pos)
            .is_some_and(u8::is_ascii_digit)
        {
            self.pos += 1;
            digits += 1;
        }
        if self.text.as_bytes().get(self.pos) == Some(&b'.') {
            self.pos += 1;
            while self
                .text
                .as_bytes()
                .get(self.pos)
                .is_some_and(u8::is_ascii_digit)
            {
                self.pos += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Err(format!("invalid SVG number at byte {start}"));
        }
        if matches!(self.text.as_bytes().get(self.pos), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.text.as_bytes().get(self.pos), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            let exponent = self.pos;
            while self
                .text
                .as_bytes()
                .get(self.pos)
                .is_some_and(u8::is_ascii_digit)
            {
                self.pos += 1;
            }
            if self.pos == exponent {
                return Err("invalid SVG numeric exponent".into());
            }
        }
        let value = self.text[start..self.pos]
            .parse::<f32>()
            .map_err(|_| "invalid SVG number")?;
        if !value.is_finite() || value.abs() > MAX_COORDINATE {
            return Err("SVG number exceeds coordinate limits".into());
        }
        Ok(value)
    }
    fn pair(&mut self) -> Result<Point> {
        Ok(Point::new(self.number()?, self.number()?))
    }
    fn flag(&mut self) -> Result<bool> {
        self.space();
        match self.text.as_bytes().get(self.pos) {
            Some(b'0') => {
                self.pos += 1;
                Ok(false)
            }
            Some(b'1') => {
                self.pos += 1;
                Ok(true)
            }
            _ => Err("SVG arc flag must be 0 or 1".into()),
        }
    }
    fn done(&mut self) -> bool {
        self.space();
        self.pos >= self.text.len()
    }
}
fn numbers(text: &str) -> Result<Vec<f32>> {
    let mut reader = NumberReader::new(text);
    let mut result = Vec::new();
    while !reader.done() {
        if result.len() > MAX_POINTS * 2 {
            return Err("SVG number list limit exceeded".into());
        }
        result.push(reader.number()?);
    }
    Ok(result)
}
fn transform(text: &str) -> Result<Matrix> {
    let mut rest = text.trim();
    let mut result = Matrix::IDENTITY;
    let mut count = 0;
    while !rest.is_empty() {
        count += 1;
        if count > 64 {
            return Err("SVG transform list limit exceeded".into());
        }
        let open = rest.find('(').ok_or("invalid SVG transform")?;
        let close = rest[open + 1..]
            .find(')')
            .map(|i| i + open + 1)
            .ok_or("unterminated SVG transform")?;
        let name = rest[..open].trim();
        let args = numbers(&rest[open + 1..close])?;
        let next = match (name, args.as_slice()) {
            ("matrix", [a, b, c, d, e, f]) => Matrix {
                a: *a,
                b: *b,
                c: *c,
                d: *d,
                e: *e,
                f: *f,
            },
            ("translate", [x]) => Matrix::translate(*x, 0.0),
            ("translate", [x, y]) => Matrix::translate(*x, *y),
            ("scale", [x]) => Matrix::scale(*x, *x),
            ("scale", [x, y]) => Matrix::scale(*x, *y),
            ("rotate", [angle]) | ("rotate", [angle, _, _]) => {
                let angle = angle.to_radians();
                let rotation = Matrix {
                    a: angle.cos(),
                    b: angle.sin(),
                    c: -angle.sin(),
                    d: angle.cos(),
                    ..Matrix::IDENTITY
                };
                if args.len() == 3 {
                    Matrix::translate(args[1], args[2])
                        .multiply(rotation)
                        .multiply(Matrix::translate(-args[1], -args[2]))
                } else {
                    rotation
                }
            }
            ("skewX", [angle]) => Matrix {
                c: angle.to_radians().tan(),
                ..Matrix::IDENTITY
            },
            ("skewY", [angle]) => Matrix {
                b: angle.to_radians().tan(),
                ..Matrix::IDENTITY
            },
            _ => return Err(format!("unsupported or invalid SVG transform '{name}'")),
        };
        result = result.multiply(next);
        if !result.valid() {
            return Err("SVG transform exceeds coordinate limits".into());
        }
        rest = rest[close + 1..].trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
    }
    Ok(result)
}
fn rounded_rect(x: f32, y: f32, w: f32, h: f32, rx: f32, ry: f32) -> Vec<Point> {
    let mut points = Vec::new();
    for (cx, cy, start) in [
        (x + w - rx, y + ry, -90.0_f32),
        (x + w - rx, y + h - ry, 0.0),
        (x + rx, y + h - ry, 90.0),
        (x + rx, y + ry, 180.0),
    ] {
        for step in 0..=12 {
            let angle = (start + step as f32 * 7.5).to_radians();
            points.push(Point::new(cx + rx * angle.cos(), cy + ry * angle.sin()));
        }
    }
    points
}

fn parse_path(text: &str) -> Result<Vec<Subpath>> {
    let mut reader = NumberReader::new(text);
    let mut paths = Vec::new();
    let mut path = Subpath::default();
    let mut current = Point::default();
    let mut start = current;
    let mut previous_control = current;
    let mut previous_command = b'M';
    let mut command = b'M';
    let mut count = 0usize;
    let mut command_count = 0usize;
    while !reader.done() {
        command_count += 1;
        if command_count > MAX_POINTS {
            return Err("SVG path command limit exceeded".into());
        }
        let next = reader.text.as_bytes()[reader.pos];
        if count == 0 && !next.eq_ignore_ascii_case(&b'M') {
            return Err("SVG path must begin with moveto".into());
        }
        if next.is_ascii_alphabetic() {
            command = next;
            reader.pos += 1;
        } else if command.eq_ignore_ascii_case(&b'Z') {
            return Err("SVG path data after close needs a command".into());
        }
        let relative = command.is_ascii_lowercase();
        let origin = if relative { current } else { Point::default() };
        let endpoint;
        let previous_points = path.points.len();
        let instruction = command.to_ascii_uppercase();
        if path.points.is_empty() && instruction != b'M' && instruction != b'Z' {
            path.points.push(current);
        }
        match instruction {
            b'M' => {
                endpoint = reader.pair()?.add(origin);
                if !path.points.is_empty() {
                    paths.push(std::mem::take(&mut path));
                }
                current = endpoint;
                start = endpoint;
                path.points.push(endpoint);
                command = if relative { b'l' } else { b'L' };
            }
            b'L' => {
                endpoint = reader.pair()?.add(origin);
                path.points.push(endpoint);
                current = endpoint;
            }
            b'H' => {
                endpoint = Point::new(reader.number()? + origin.x, current.y);
                path.points.push(endpoint);
                current = endpoint;
            }
            b'V' => {
                endpoint = Point::new(current.x, reader.number()? + origin.y);
                path.points.push(endpoint);
                current = endpoint;
            }
            b'C' | b'S' => {
                let control1 = if command.eq_ignore_ascii_case(&b'C') {
                    reader.pair()?.add(origin)
                } else if previous_command.eq_ignore_ascii_case(&b'C')
                    || previous_command.eq_ignore_ascii_case(&b'S')
                {
                    Point::new(
                        2.0 * current.x - previous_control.x,
                        2.0 * current.y - previous_control.y,
                    )
                } else {
                    current
                };
                let control2 = reader.pair()?.add(origin);
                endpoint = reader.pair()?.add(origin);
                let steps = curve_steps(
                    current.distance(control1)
                        + control1.distance(control2)
                        + control2.distance(endpoint),
                );
                for step in 1..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    path.points.push(Point::new(
                        u * u * u * current.x
                            + 3.0 * u * u * t * control1.x
                            + 3.0 * u * t * t * control2.x
                            + t * t * t * endpoint.x,
                        u * u * u * current.y
                            + 3.0 * u * u * t * control1.y
                            + 3.0 * u * t * t * control2.y
                            + t * t * t * endpoint.y,
                    ));
                }
                previous_control = control2;
                current = endpoint;
            }
            b'Q' | b'T' => {
                let control = if command.eq_ignore_ascii_case(&b'Q') {
                    reader.pair()?.add(origin)
                } else if previous_command.eq_ignore_ascii_case(&b'Q')
                    || previous_command.eq_ignore_ascii_case(&b'T')
                {
                    Point::new(
                        2.0 * current.x - previous_control.x,
                        2.0 * current.y - previous_control.y,
                    )
                } else {
                    current
                };
                endpoint = reader.pair()?.add(origin);
                let steps = curve_steps(current.distance(control) + control.distance(endpoint));
                for step in 1..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    path.points.push(Point::new(
                        u * u * current.x + 2.0 * u * t * control.x + t * t * endpoint.x,
                        u * u * current.y + 2.0 * u * t * control.y + t * t * endpoint.y,
                    ));
                }
                previous_control = control;
                current = endpoint;
            }
            b'A' => {
                let rx = reader.number()?.abs();
                let ry = reader.number()?.abs();
                let rotation = reader.number()?;
                let large = reader.flag()?;
                let sweep = reader.flag()?;
                endpoint = reader.pair()?.add(origin);
                arc(
                    &mut path.points,
                    current,
                    endpoint,
                    rx,
                    ry,
                    rotation,
                    large,
                    sweep,
                );
                current = endpoint;
            }
            b'Z' => {
                path.closed = true;
                current = start;
                paths.push(std::mem::take(&mut path));
            }
            _ => {
                return Err(format!(
                    "unsupported SVG path command '{}'",
                    command as char
                ));
            }
        }
        previous_command = command;
        let added = match instruction {
            b'M' => 1,
            b'Z' => 0,
            _ => path.points.len().saturating_sub(previous_points),
        };
        count = count.saturating_add(added);
        if count > MAX_POINTS {
            return Err("SVG path geometry limit exceeded".into());
        }
    }
    if !path.points.is_empty() {
        paths.push(path);
    }
    if paths.iter().map(|path| path.points.len()).sum::<usize>() > MAX_POINTS {
        return Err("SVG path geometry limit exceeded".into());
    }
    Ok(paths)
}
fn curve_steps(length: f32) -> usize {
    (length / 8.0).ceil().clamp(8.0, 64.0) as usize
}
// Endpoint-to-center conversion follows SVG 2 Appendix B.2:
// https://www.w3.org/TR/SVG/implnote.html#ArcImplementationNotes
#[allow(clippy::too_many_arguments)]
fn arc(
    points: &mut Vec<Point>,
    start: Point,
    end: Point,
    mut rx: f32,
    mut ry: f32,
    rotation: f32,
    large: bool,
    sweep: bool,
) {
    if start.distance(end) < 1e-6 {
        return;
    }
    if rx <= 0.0 || ry <= 0.0 {
        points.push(end);
        return;
    }
    let phi = rotation.to_radians();
    let cos = phi.cos();
    let sin = phi.sin();
    let dx = (start.x - end.x) * 0.5;
    let dy = (start.y - end.y) * 0.5;
    let x = cos * dx + sin * dy;
    let y = -sin * dx + cos * dy;
    let lambda = x * x / (rx * rx) + y * y / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }
    let numerator = (rx * rx * ry * ry - rx * rx * y * y - ry * ry * x * x).max(0.0);
    let denominator = rx * rx * y * y + ry * ry * x * x;
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let factor = if denominator <= 0.0 {
        0.0
    } else {
        sign * (numerator / denominator).sqrt()
    };
    let cx = factor * rx * y / ry;
    let cy = -factor * ry * x / rx;
    let center = Point::new(
        cos * cx - sin * cy + (start.x + end.x) * 0.5,
        sin * cx + cos * cy + (start.y + end.y) * 0.5,
    );
    let first = ((y - cy) / ry).atan2((x - cx) / rx);
    let last = ((-y - cy) / ry).atan2((-x - cx) / rx);
    let mut delta = last - first;
    if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    }
    if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    }
    let steps = (delta.abs() * rx.max(ry) / 8.0).ceil().clamp(8.0, 128.0) as usize;
    for step in 1..=steps {
        let angle = first + delta * step as f32 / steps as f32;
        points.push(Point::new(
            center.x + cos * rx * angle.cos() - sin * ry * angle.sin(),
            center.y + sin * rx * angle.cos() + cos * ry * angle.sin(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dense_crossing_paths_charge_intersection_sorting_work() {
        let mut painter = Painter {
            image: RasterImage {
                width: 8,
                height: 8,
                rgba: vec![0; 8 * 8 * 4],
            },
            work: 2_000,
            points: 0,
            font: None,
            bold_font: None,
        };
        let path = Subpath {
            points: (0..64)
                .map(|index| Point::new((index % 8) as f32, if index % 2 == 0 { 0.0 } else { 8.0 }))
                .collect(),
            closed: true,
        };
        assert!(
            painter
                .fill(&[path], Color::BLACK, false)
                .unwrap_err()
                .contains("work limit")
        );
    }
    fn pixel(image: &RasterImage, x: usize, y: usize) -> [u8; 4] {
        image.rgba[(y * image.width as usize + x) * 4..(y * image.width as usize + x) * 4 + 4]
            .try_into()
            .unwrap()
    }
    #[test]
    fn shapes_transparency_and_viewbox() {
        let image = render("<svg width='100' height='100' viewBox='0 0 50 50'><rect x='5' y='5' width='20' height='20' fill='red'/><circle cx='35' cy='35' r='10' fill='#00ff00'/></svg>",None,None).unwrap();
        assert_eq!(pixel(&image, 20, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 70, 70), [0, 255, 0, 255]);
        assert_eq!(pixel(&image, 0, 0), [0, 0, 0, 0]);
    }
    #[test]
    fn adjusted_foreign_names_control_scaling_and_skip_definition_subtrees() {
        let image = render("<svg width=40 height=20 viewBox='0 0 10 10' preserveAspectRatio=none><rect width=10 height=10 fill=red /></svg>", None, None).unwrap();
        assert_eq!(pixel(&image, 1, 10), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 38, 10), [255, 0, 0, 255]);
        for tag in [
            "clipPath",
            "linearGradient",
            "radialGradient",
            "foreignObject",
        ] {
            let source = format!(
                "<svg width=10 height=10><{tag}><rect width=10 height=10 fill=red /></{tag}></svg>"
            );
            assert!(
                render(&source, None, None)
                    .unwrap()
                    .rgba
                    .iter()
                    .all(|&v| v == 0),
                "{tag}"
            );
        }
    }
    #[test]
    fn transforms_curves_and_evenodd_holes() {
        let image = render("<svg width='80' height='80'><g transform='translate(10 10)' fill='blue'><path fill-rule='evenodd' d='M0 0H60V60H0Z M20 20H40V40H20Z'/></g></svg>",None,None).unwrap();
        assert_eq!(pixel(&image, 15, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&image, 40, 40), [0, 0, 0, 0]);
        let curves = render("<svg width='80' height='80'><path d='M10 40 C10 0 70 0 70 40 Q40 80 10 40Z' fill='red' stroke='black' stroke-width='2'/></svg>",None,None).unwrap();
        assert_eq!(pixel(&curves, 40, 40), [255, 0, 0, 255]);
    }
    #[test]
    fn arc_paths_and_text_render() {
        let image = render("<svg width='100' height='100'><path d='M20 50 A30 30 0 1 0 80 50 A30 30 0 1 0 20 50Z' fill='green'/><text x='15' y='95' font-size='14'>Hello</text></svg>",None,None).unwrap();
        assert!(pixel(&image, 50, 50)[3] > 0);
        assert!(
            image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] > 0)
                .count()
                > 1000
        );
    }
    #[test]
    fn untrusted_svg_has_bounded_resources_and_no_external_content() {
        assert!(render("<svg width='999999' height='999999'/>", None, None).is_err());
        assert!(render(&"x".repeat(MAX_SOURCE + 1), None, None).is_err());
        assert!(render("<svg><path d='M NaN 0'/></svg>", None, None).is_err());
        assert!(
            render(
                "<svg><g transform='scale(1000000) scale(1000000)'/></svg>",
                None,
                None
            )
            .is_err()
        );
        let image = render("<svg width='10' height='10'><script>bad()</script><image href='file:///etc/passwd'/><foreignObject>ignored</foreignObject></svg>",None,None).unwrap();
        assert!(image.rgba.iter().all(|value| *value == 0));
    }

    #[test]
    fn malformed_path_data_and_repeated_close_commands_are_bounded() {
        assert!(parse_path(&format!("M0 0{}", "Z".repeat(MAX_POINTS + 1))).is_err());
        assert!(parse_path("L10 10").is_err());
        assert!(parse_path("M0 0 A10 10 0 2 0 10 10").is_err());
        assert!(parse_path("M0 0 A10 10 0 0110 10").is_ok());
        let after_close = parse_path("M0 0L10 0ZL20 20").unwrap();
        assert_eq!(after_close[1].points.len(), 2);
        let alphabet = b"012+-.,eEMmZzLlHhVvCcSsQqTtAa /()";
        let mut random = 0x671bd252_u32;
        for len in 0..512 {
            let mut text = String::from("M0 0");
            for _ in 0..len {
                random ^= random << 13;
                random ^= random >> 17;
                random ^= random << 5;
                text.push(alphabet[random as usize % alphabet.len()] as char);
            }
            let _ = parse_path(&text);
        }
    }

    #[test]
    fn painting_obeys_work_budget_and_rejects_deep_groups() {
        let mut painter = Painter {
            image: RasterImage {
                width: 8,
                height: 8,
                rgba: vec![0; 8 * 8 * 4],
            },
            work: 4,
            points: 0,
            font: None,
            bold_font: None,
        };
        let path = Subpath {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(8.0, 0.0),
                Point::new(8.0, 8.0),
                Point::new(0.0, 8.0),
            ],
            closed: true,
        };
        assert!(
            painter
                .fill(&[path], Color::BLACK, false)
                .unwrap_err()
                .contains("work limit")
        );
        let source = format!(
            "<svg>{}<rect width='10' height='10'/>{}</svg>",
            "<g>".repeat(100),
            "</g>".repeat(100)
        );
        assert!(
            render(&source, Some(20), Some(20))
                .unwrap_err()
                .contains("nesting")
        );
    }
}
