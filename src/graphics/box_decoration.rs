//! Fixed-size elliptical coverage for self-contained box decorations.
//!
//! Coverage is a new-command contract: sixteen fixed subpixel samples, with
//! rings evaluated as outer AND NOT cutout at each sample. Old primitives keep
//! their established antialiasing and sampling when no decoration is active.
use super::{Canvas, Color, ImageStore, Rect, gradient::LinearGradient};
use std::sync::Arc;

#[cfg(feature = "raster-bridge")]
mod native;
#[cfg(feature = "raster-bridge")]
pub(crate) use native::{
    BoxCoreReservation, BoxCoreTiles, BoxCoreWindow, BoxMaskError, BoxMaskReservation, BoxMaskTile,
    BoxMaskTiles, BoxMaskWindow, BoxRectProof,
};

const MAX_COORDINATE: f64 = 16_000_000.0;
const SETUP_WORK: u64 = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundedBox {
    pub rect: Rect,
    /// Elliptical radii in top-left, top-right, bottom-right, bottom-left order.
    pub radii: [[f32; 2]; 4],
}

impl RoundedBox {
    /// Validate used geometry, without rescaling radii. Inset arcs may extend
    /// past an opposite edge, and all applicable corner constraints still apply.
    pub fn valid(&self) -> bool {
        let r = self.rect;
        [r.x, r.y, r.width, r.height]
            .into_iter()
            .all(|v| v.is_finite() && f64::from(v).abs() <= MAX_COORDINATE)
            && r.width >= 0.0
            && r.height >= 0.0
            && (f64::from(r.x) + f64::from(r.width)).abs() <= MAX_COORDINATE
            && (f64::from(r.y) + f64::from(r.height)).abs() <= MAX_COORDINATE
            && self
                .radii
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0.0 && f64::from(*v) <= MAX_COORDINATE)
    }

    /// Prepare an outer shape using the CSS common overlap factor. All original
    /// radii enter that factor before zero-axis corners become square.
    pub fn normalized(&self) -> Option<Self> {
        if !self.valid() {
            return None;
        }
        let [tl, tr, br, bl] = self.radii;
        let mut factor = 1.0f64;
        for (length, sum) in [
            (self.rect.width, f64::from(tl[0]) + f64::from(tr[0])),
            (self.rect.width, f64::from(bl[0]) + f64::from(br[0])),
            (self.rect.height, f64::from(tl[1]) + f64::from(bl[1])),
            (self.rect.height, f64::from(tr[1]) + f64::from(br[1])),
        ] {
            if sum > 0.0 {
                factor = factor.min(f64::from(length) / sum);
            }
        }
        let mut result = *self;
        for radius in &mut result.radii {
            if radius[0] == 0.0 || radius[1] == 0.0 {
                *radius = [0.0; 2];
            } else {
                radius[0] = (f64::from(radius[0]) * factor) as f32;
                radius[1] = (f64::from(radius[1]) * factor) as f32;
            }
        }
        Some(result)
    }

    /// Derive a used inner shape from an OUTER shape. Normalize outer radii
    /// once, then subtract the adjacent side insets. Do not normalize the
    /// result: a narrow residual box can retain a truncated ellipse arc.
    /// For content geometry sum border and padding first; do not call inset
    /// again on an already-derived padding shape.
    pub fn inset(&self, [top, right, bottom, left]: [f32; 4]) -> Option<Self> {
        if ![top, right, bottom, left]
            .into_iter()
            .all(|v| v.is_finite() && v >= 0.0 && f64::from(v) <= MAX_COORDINATE)
        {
            return None;
        }
        let outer = self.normalized()?;
        let mut result = Self {
            rect: Rect {
                x: (f64::from(outer.rect.x) + f64::from(left)) as f32,
                y: (f64::from(outer.rect.y) + f64::from(top)) as f32,
                width: (f64::from(outer.rect.width) - f64::from(left) - f64::from(right)).max(0.0)
                    as f32,
                height: (f64::from(outer.rect.height) - f64::from(top) - f64::from(bottom)).max(0.0)
                    as f32,
            },
            radii: outer.radii,
        };
        for (radius, inset) in result.radii.iter_mut().zip([
            [left, top],
            [right, top],
            [right, bottom],
            [left, bottom],
        ]) {
            radius[0] = (radius[0] - inset[0]).max(0.0);
            radius[1] = (radius[1] - inset[1]).max(0.0);
        }
        result.valid().then_some(result)
    }

    pub fn translated(self, dx: f32, dy: f32) -> Self {
        Self {
            rect: self.rect.translated(dx, dy),
            ..self
        }
    }

    pub fn scaled(self, scale: f32) -> Self {
        Self {
            rect: Rect {
                x: self.rect.x * scale,
                y: self.rect.y * scale,
                width: self.rect.width * scale,
                height: self.rect.height * scale,
            },
            radii: self.radii.map(|[x, y]| [x * scale, y * scale]),
        }
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        let left = f64::from(self.rect.x);
        let top = f64::from(self.rect.y);
        let right = left + f64::from(self.rect.width);
        let bottom = top + f64::from(self.rect.height);
        if x < left || y < top || x >= right || y >= bottom {
            return false;
        }
        for (index, [rx, ry]) in self.radii.into_iter().enumerate() {
            if rx == 0.0 || ry == 0.0 {
                continue;
            }
            let rx = f64::from(rx);
            let ry = f64::from(ry);
            let dx = if index == 0 || index == 3 {
                left + rx - x
            } else {
                x - (right - rx)
            };
            let dy = if index < 2 {
                top + ry - y
            } else {
                y - (bottom - ry)
            };
            if dx > 0.0 && dy > 0.0 {
                let nx = dx / rx;
                let ny = dy / ry;
                if nx * nx + ny * ny > 1.0 {
                    return false;
                }
            }
        }
        true
    }

    fn classify(&self, x: i32, y: i32) -> CoverageClass {
        let left = f64::from(x) + 0.125;
        let top = f64::from(y) + 0.125;
        let right = f64::from(x) + 0.875;
        let bottom = f64::from(y) + 0.875;
        if self.rect.width <= 0.0
            || self.rect.height <= 0.0
            || right < f64::from(self.rect.x)
            || bottom < f64::from(self.rect.y)
            || left >= f64::from(self.rect.x) + f64::from(self.rect.width)
            || top >= f64::from(self.rect.y) + f64::from(self.rect.height)
        {
            return CoverageClass::Outside;
        }
        // Each rounded shape is an intersection of convex corner regions and
        // its rectangle, even when inset arcs overlap. If the four vertices of
        // the sample square are inside, their convex hull contains all samples.
        if self.contains(left, top)
            && self.contains(right, top)
            && self.contains(right, bottom)
            && self.contains(left, bottom)
        {
            CoverageClass::Inside
        } else {
            CoverageClass::Partial
        }
    }
}

/// Shared command payload keeps the display-list enum compact. Cloning a
/// command shares this allocation; painting borrows its geometry and source.
#[derive(Clone, Debug)]
pub struct BoxDecoration {
    pub shape: RoundedBox,
    pub cutout: Option<RoundedBox>,
    pub paint: BoxPaint,
}

#[derive(Clone, Debug)]
pub enum BoxPaint {
    Color(Color),
    Image {
        tile: Rect,
        repeat: [bool; 2],
        key: String,
    },
    LinearGradient {
        tile: Rect,
        repeat: [bool; 2],
        gradient: Arc<LinearGradient>,
        current_color: Color,
        stop_scale: f32,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CoverageClass {
    Outside,
    Inside,
    Partial,
}

#[derive(Clone, Copy)]
pub(super) struct ShapeMask {
    outer: RoundedBox,
    cutout: Option<RoundedBox>,
}

impl ShapeMask {
    fn classify(self, x: i32, y: i32) -> CoverageClass {
        let outer = self.outer.classify(x, y);
        if outer == CoverageClass::Outside {
            return CoverageClass::Outside;
        }
        match self.cutout.map(|shape| shape.classify(x, y)) {
            Some(CoverageClass::Inside) => CoverageClass::Outside,
            None | Some(CoverageClass::Outside) => outer,
            Some(CoverageClass::Partial) => CoverageClass::Partial,
        }
    }

    fn coverage(self, x: i32, y: i32) -> u8 {
        match self.classify(x, y) {
            CoverageClass::Outside => 0,
            CoverageClass::Inside => 255,
            CoverageClass::Partial => {
                let mut hits = 0u16;
                for sy in [0.125, 0.375, 0.625, 0.875] {
                    for sx in [0.125, 0.375, 0.625, 0.875] {
                        let px = f64::from(x) + sx;
                        let py = f64::from(y) + sy;
                        if self.outer.contains(px, py)
                            && self.cutout.is_none_or(|shape| !shape.contains(px, py))
                        {
                            hits += 1;
                        }
                    }
                }
                (255 * hits / 16) as u8
            }
        }
    }
}

fn valid_tile(tile: Rect) -> bool {
    [tile.x, tile.y, tile.width, tile.height]
        .into_iter()
        .all(|v| v.is_finite() && f64::from(v).abs() <= MAX_COORDINATE)
        && tile.width > 0.0
        && tile.height > 0.0
}

impl Canvas {
    // The no-mask branch returns the original coverage verbatim. In particular
    // gradient channels remain unquantized until their established composition.
    pub(super) fn box_coverage(&self, x: i32, y: i32, coverage: u8) -> u8 {
        match self.box_mask {
            None => coverage,
            Some(mask) => (u16::from(coverage) * u16::from(mask.coverage(x, y)) / 255) as u8,
        }
    }

    pub fn box_decoration(
        &mut self,
        shape: RoundedBox,
        cutout: Option<RoundedBox>,
        paint: &BoxPaint,
        images: &ImageStore,
    ) {
        self.box_decoration_offset(shape, cutout, paint, images, (0.0, 0.0));
    }

    pub(super) fn box_decoration_offset(
        &mut self,
        shape: RoundedBox,
        cutout: Option<RoundedBox>,
        paint: &BoxPaint,
        images: &ImageStore,
        (dx, dy): (f32, f32),
    ) {
        // This is a single-command hook, never a new scope. Every early return
        // and quota refusal leaves old primitives and opacity pops unmasked.
        self.box_mask = None;
        let shape = shape.translated(dx, dy);
        let cutout = cutout.map(|shape| shape.translated(dx, dy));
        if self.paint_exhausted
            || self.suppressed()
            || !shape.valid()
            || cutout.is_some_and(|shape| !shape.valid())
            || shape.rect.width <= 0.0
            || shape.rect.height <= 0.0
        {
            return;
        }
        match paint {
            BoxPaint::Color(color) if color.a == 0 => return,
            BoxPaint::Image { tile, key, .. } => {
                if !valid_tile(tile.translated(dx, dy)) || !images.contains_key(key) {
                    return;
                }
            }
            BoxPaint::LinearGradient {
                tile,
                gradient,
                stop_scale,
                ..
            } => {
                if !valid_tile(tile.translated(dx, dy))
                    || !gradient.valid()
                    || !stop_scale.is_finite()
                    || *stop_scale <= 0.0
                    || f64::from(*stop_scale) > MAX_COORDINATE
                {
                    return;
                }
            }
            BoxPaint::Color(_) => {}
        }
        let visible = shape.rect.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return;
        }
        let x0 = visible.x.floor().max(self.clip.x.ceil()).max(0.0) as i32;
        let y0 = visible.y.floor().max(self.clip.y.ceil()).max(0.0) as i32;
        let x1 = (visible.x + visible.width).ceil().min(self.width as f32) as i32;
        let y1 = (visible.y + visible.height).ceil().min(self.height as f32) as i32;
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let pixels = (x1 - x0) as u64 * (y1 - y0) as u64;
        let shapes = 1 + u64::from(cutout.is_some());
        // One unit is a fixed-size classification or point-membership query,
        // not a machine instruction. Prepay each of the two classifications;
        // only ambiguous candidates reserve the sixteen membership samples.
        // This window conservatively includes later no-repeat/transparent holes.
        if !self.consume_pixels(SETUP_WORK + pixels * shapes) {
            return;
        }
        let mask = ShapeMask {
            outer: shape,
            cutout,
        };
        let mut ambiguous = 0u64;
        for y in y0..y1 {
            for x in x0..x1 {
                ambiguous += u64::from(mask.classify(x, y) == CoverageClass::Partial);
            }
        }
        if !self.consume_pixels(pixels * shapes + ambiguous * 16 * shapes) {
            return;
        }
        // Existing primitives keep their own source/paint/opacity tariffs and
        // scalar radius zero prevents double antialiasing of this outline.
        self.box_mask = Some(mask);
        match paint {
            BoxPaint::Color(color) => self.rect(shape.rect, *color, 0.0),
            BoxPaint::Image { tile, repeat, key } => {
                if let Some(image) = images.get(key) {
                    self.background_image(shape.rect, tile.translated(dx, dy), 0.0, image, *repeat);
                }
            }
            BoxPaint::LinearGradient {
                tile,
                repeat,
                gradient,
                current_color,
                stop_scale,
            } => {
                self.linear_gradient_with_repeat(
                    shape.rect,
                    tile.translated(dx, dy),
                    0.0,
                    gradient,
                    *current_color,
                    *stop_scale,
                    *repeat,
                );
            }
        }
        self.box_mask = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::gradient::{GradientColor, GradientDirection, GradientStop};
    use crate::graphics::{DrawCommand, Fonts, PaintBudget, RasterImage};

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
    fn shape(rect: Rect, radii: [[f32; 2]; 4]) -> RoundedBox {
        RoundedBox { rect, radii }
    }
    fn square(rect: Rect) -> RoundedBox {
        shape(rect, [[0.0; 2]; 4])
    }
    fn solid(shape: RoundedBox, color: Color) -> DrawCommand {
        DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape,
            cutout: None,
            paint: BoxPaint::Color(color),
        }))
    }
    fn constant_gradient(color: Color) -> Arc<LinearGradient> {
        Arc::new(LinearGradient {
            direction: GradientDirection::Angle(90.0),
            stops: Arc::from([GradientStop {
                color: GradientColor::Color(color),
                position: None,
            }]),
        })
    }
    fn paint(width: u32, height: u32, commands: &[DrawCommand], images: &ImageStore) -> Canvas {
        let mut canvas = Canvas::new(width, height).unwrap();
        canvas.paint(commands, &Fonts::new(), images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert!(canvas.box_mask.is_none());
        canvas
    }

    #[test]
    fn geometry_validation_is_finite_bounded_and_allows_empty_shapes() {
        let valid = square(rect(-16_000_000.0, 0.0, 16_000_000.0, 0.0));
        assert!(valid.valid());
        for bad in [
            square(rect(16_000_000.0, 0.0, 1.0, 1.0)),
            square(rect(0.0, 0.0, -1.0, 1.0)),
            square(rect(f32::NAN, 0.0, 1.0, 1.0)),
            shape(rect(0.0, 0.0, 1.0, 1.0), [[f32::INFINITY, 1.0]; 4]),
            shape(rect(0.0, 0.0, 1.0, 1.0), [[-1.0, 1.0]; 4]),
            shape(rect(0.0, 0.0, 1.0, 1.0), [[16_000_002.0, 1.0]; 4]),
        ] {
            assert!(!bad.valid());
            assert!(bad.normalized().is_none());
        }
    }

    #[test]
    fn overlap_normalization_uses_one_factor_and_zero_axis_is_square() {
        let source = shape(rect(0.0, 0.0, 12.0, 8.0), [[8.0, 6.0]; 4]);
        assert_eq!(source.normalized().unwrap().radii, [[16.0 / 3.0, 4.0]; 4]);
        let zero = shape(
            rect(0.0, 0.0, 10.0, 10.0),
            [[100.0, 0.0], [1.0, 1.0], [0.0, 0.0], [0.0, 0.0]],
        );
        assert_eq!(
            zero.normalized().unwrap().radii,
            [
                [0.0, 0.0],
                [10.0 / 101.0, 10.0 / 101.0],
                [0.0, 0.0],
                [0.0, 0.0]
            ]
        );
        assert_eq!(
            square(rect(0.0, 0.0, 0.0, 0.0)).normalized().unwrap().radii,
            [[0.0; 2]; 4]
        );
    }

    #[test]
    fn asymmetric_insets_keep_truncated_inner_ellipses() {
        let outer = shape(rect(0.0, 0.0, 20.0, 16.0), [[6.0; 2]; 4]);
        let padding = outer.inset([1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(padding.rect, rect(4.0, 1.0, 14.0, 12.0));
        assert_eq!(
            padding.radii,
            [[2.0, 5.0], [4.0, 5.0], [4.0, 3.0], [2.0, 3.0]]
        );
        let narrow = shape(rect(0.0, 0.0, 10.0, 10.0), [[5.0; 2]; 4])
            .inset([0.0, 9.0, 0.0, 0.0])
            .unwrap();
        assert_eq!(narrow.rect, rect(0.0, 0.0, 1.0, 10.0));
        assert_eq!(
            narrow.radii,
            [[5.0, 5.0], [0.0, 5.0], [0.0, 5.0], [5.0, 5.0]]
        );
        assert!(!narrow.contains(0.5, 0.5));
        assert!(narrow.contains(0.5, 4.5));
        assert!(outer.inset([-1.0, 0.0, 0.0, 0.0]).is_none());
        assert!(outer.inset([f32::NAN, 0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn translate_and_scale_preserve_corner_order_and_do_not_normalize() {
        let source = shape(
            rect(1.0, 2.0, 3.0, 4.0),
            [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]],
        );
        assert_eq!(source.translated(-2.0, 3.0).rect, rect(-1.0, 5.0, 3.0, 4.0));
        assert_eq!(source.translated(-2.0, 3.0).radii, source.radii);
        let scaled = source.scaled(2.0);
        assert_eq!(scaled.rect, rect(2.0, 4.0, 6.0, 8.0));
        assert_eq!(
            scaled.radii,
            [[2.0, 4.0], [6.0, 8.0], [10.0, 12.0], [14.0, 16.0]]
        );
        assert!(!source.translated(f32::INFINITY, 0.0).valid());
        assert!(!source.scaled(-1.0).valid());
    }

    #[test]
    fn four_by_four_ellipse_and_circle_counts_are_literal() {
        let ellipse = shape(rect(0.0, 0.0, 4.0, 2.0), [[2.0, 1.0]; 4]);
        let canvas = paint(4, 2, &[solid(ellipse, Color::BLACK)], &ImageStore::new());
        assert_eq!(
            canvas.pixels,
            [0x707070, 0, 0, 0x707070, 0x707070, 0, 0, 0x707070]
        );
        let circle = shape(rect(0.0, 0.0, 4.0, 4.0), [[1.0, 1.0]; 4]);
        let canvas = paint(4, 4, &[solid(circle, Color::BLACK)], &ImageStore::new());
        assert_eq!(
            canvas.pixels,
            [
                0x303030, 0, 0, 0x303030, 0, 0, 0, 0, 0, 0, 0, 0, 0x303030, 0, 0, 0x303030
            ]
        );
    }

    #[test]
    fn fractional_rect_and_correlated_cutout_use_the_same_sample_grid() {
        let fractional = square(rect(0.25, 0.25, 1.0, 1.0));
        let canvas = paint(2, 2, &[solid(fractional, Color::BLACK)], &ImageStore::new());
        assert_eq!(canvas.pixels, [0x707070, 0xd0d0d0, 0xd0d0d0, 0xf0f0f0]);
        let ring = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape: square(rect(0.0, 0.0, 2.0, 2.0)),
            cutout: Some(fractional),
            paint: BoxPaint::Color(Color::BLACK),
        }));
        assert_eq!(
            paint(2, 2, &[ring], &ImageStore::new()).pixels,
            [0x909090, 0x303030, 0x303030, 0x101010]
        );
        let empty_ring = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape: fractional,
            cutout: Some(fractional),
            paint: BoxPaint::Color(Color::BLACK),
        }));
        assert_eq!(
            paint(2, 2, &[empty_ring], &ImageStore::new()).pixels,
            [0xffffff; 4]
        );
    }

    #[test]
    fn color_raster_and_gradient_share_the_literal_elliptical_outline() {
        let shape = shape(rect(0.0, 0.0, 4.0, 2.0), [[2.0, 1.0]; 4]);
        let mut images = ImageStore::new();
        images.insert(
            "black".into(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
            }),
        );
        for source in [
            BoxPaint::Color(Color::BLACK),
            BoxPaint::Image {
                tile: rect(0.0, 0.0, 1.0, 1.0),
                repeat: [true, true],
                key: "black".into(),
            },
            BoxPaint::LinearGradient {
                tile: rect(0.0, 0.0, 1.0, 1.0),
                repeat: [true, true],
                gradient: constant_gradient(Color::BLACK),
                current_color: Color::BLACK,
                stop_scale: 1.0,
            },
        ] {
            let command = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape,
                cutout: None,
                paint: source,
            }));
            assert_eq!(
                paint(4, 2, &[command], &images).pixels,
                [0x707070, 0, 0, 0x707070, 0x707070, 0, 0, 0x707070]
            );
        }
    }

    #[test]
    fn image_sampling_and_nonrepeat_holes_remain_center_based() {
        let mut images = ImageStore::new();
        images.insert(
            "rb".into(),
            Arc::new(RasterImage {
                width: 2,
                height: 1,
                rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
            }),
        );
        for (repeat, expected) in [
            ([false, false], [0xff0000, 0x0000ff, 0xffffff, 0xffffff]),
            ([true, false], [0xff0000, 0x0000ff, 0xff0000, 0x0000ff]),
        ] {
            let command = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: square(rect(0.0, 0.0, 4.0, 1.0)),
                cutout: None,
                paint: BoxPaint::Image {
                    tile: rect(0.25, 0.0, 2.0, 1.0),
                    repeat,
                    key: "rb".into(),
                },
            }));
            assert_eq!(paint(4, 1, &[command], &images).pixels, expected);
        }
    }

    #[test]
    fn alpha_and_nested_groups_apply_shape_coverage_once() {
        let ellipse = shape(rect(0.0, 0.0, 4.0, 2.0), [[2.0, 1.0]; 4]);
        assert_eq!(
            paint(
                4,
                2,
                &[solid(ellipse, Color::rgba(255, 0, 0, 128))],
                &ImageStore::new()
            )
            .pixels,
            [
                0xffb8b8, 0xff7f7f, 0xff7f7f, 0xffb8b8, 0xffb8b8, 0xff7f7f, 0xff7f7f, 0xffb8b8
            ]
        );
        let canvas = paint(
            4,
            2,
            &[
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PushOpacity { opacity: 0.5 },
                solid(ellipse, Color::rgb(255, 0, 0)),
                DrawCommand::PopOpacity,
                DrawCommand::PopOpacity,
            ],
            &ImageStore::new(),
        );
        assert_eq!(
            canvas.pixels,
            [
                0xffdbdb, 0xffbfbf, 0xffbfbf, 0xffdbdb, 0xffdbdb, 0xffbfbf, 0xffbfbf, 0xffdbdb
            ]
        );
    }

    #[test]
    fn fixed_clip_restoration_and_following_primitive_do_not_reuse_mask() {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.paint_with_viewport(
            &[
                DrawCommand::PushClip {
                    rect: rect(0.0, 0.0, 1.0, 1.0),
                },
                solid(square(rect(0.0, 0.0, 2.0, 1.0)), Color::rgb(255, 0, 0)),
                DrawCommand::PushFixed,
                solid(square(rect(3.0, 0.0, 1.0, 1.0)), Color::rgb(0, 0, 255)),
                DrawCommand::PopFixed,
                DrawCommand::PopClip,
                DrawCommand::Rect {
                    rect: rect(0.0, 1.0, 1.0, 1.0),
                    color: Color::BLACK,
                    radius: 0.0,
                },
            ],
            &Fonts::new(),
            &ImageStore::new(),
            (1.0, 0.0),
            (0.0, 0.0),
        );
        assert_eq!(
            canvas.pixels,
            [
                0xffffff, 0xff0000, 0xffffff, 0x0000ff, 0xffffff, 0, 0xffffff, 0xffffff
            ]
        );
        assert!(canvas.box_mask.is_none());
        assert!(!canvas.exhausted());
    }

    #[test]
    fn classifier_and_sample_fees_are_prepaid_at_literal_boundaries() {
        let shape = square(rect(0.25, 0.25, 0.5, 0.5));
        for allowance in [0, 8, 9, 25, 26, 27] {
            let mut canvas = Canvas::new(1, 1).unwrap();
            canvas.budget = Some(PaintBudget {
                pixels: allowance,
                glyphs: 0,
            });
            canvas.box_decoration(
                shape,
                None,
                &BoxPaint::Color(Color::BLACK),
                &ImageStore::new(),
            );
            assert_eq!(
                canvas.pixels,
                [if allowance == 27 { 0xc0c0c0 } else { 0xffffff }]
            );
            assert_eq!(canvas.exhausted(), allowance != 27);
            assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0);
            assert!(canvas.box_mask.is_none());
        }
        let mut canvas = Canvas::new(1, 1).unwrap();
        canvas.budget = Some(PaintBudget {
            pixels: 11,
            glyphs: 0,
        });
        canvas.box_decoration(
            square(rect(0.0, 0.0, 1.0, 1.0)),
            None,
            &BoxPaint::Color(Color::BLACK),
            &ImageStore::new(),
        );
        assert_eq!(canvas.pixels, [0]);
        assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0);
        assert!(!canvas.exhausted());
    }

    #[test]
    fn empty_missing_transparent_and_suppressed_commands_leave_no_mask_or_layer() {
        let images = ImageStore::new();
        let shape = square(rect(0.0, 0.0, 2.0, 2.0));
        let missing = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape,
            cutout: None,
            paint: BoxPaint::Image {
                tile: shape.rect,
                repeat: [true, true],
                key: "missing".into(),
            },
        }));
        let canvas = paint(
            2,
            2,
            &[
                solid(square(rect(0.0, 0.0, 0.0, 2.0)), Color::BLACK),
                solid(shape, Color::TRANSPARENT),
                missing,
                DrawCommand::PushOpacity { opacity: 0.0 },
                solid(shape, Color::BLACK),
                DrawCommand::PopOpacity,
            ],
            &images,
        );
        assert_eq!(canvas.pixels, [0xffffff; 4]);
        assert_eq!(canvas.layer_allocated, 0);
        assert!(canvas.layers.is_empty());
    }

    #[test]
    fn arbitrary_overlapping_corner_constraints_are_all_applied() {
        let overlapping = shape(rect(0.0, 0.0, 2.0, 2.0), [[4.0, 4.0]; 4]);
        assert!(overlapping.valid());
        assert!(!overlapping.contains(0.5, 0.5));
        assert!(!overlapping.contains(1.5, 1.5));
        assert_eq!(
            paint(
                2,
                2,
                &[solid(overlapping, Color::BLACK)],
                &ImageStore::new()
            )
            .pixels,
            [0xffffff; 4]
        );
    }
}
