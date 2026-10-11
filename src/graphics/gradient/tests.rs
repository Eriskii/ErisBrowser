use super::*;
use crate::graphics::{Canvas, DrawCommand, Fonts, ImageStore, PaintBudget};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn stop(color: Color, percent: Option<f64>) -> GradientStop {
    GradientStop {
        color: GradientColor::Color(color),
        position: percent.map(|percent| GradientPosition { px: 0.0, percent }),
    }
}

fn gradient(direction: GradientDirection, stops: &[GradientStop]) -> LinearGradient {
    LinearGradient {
        direction,
        stops: Arc::from(stops),
    }
}

fn gray() -> LinearGradient {
    gradient(
        GradientDirection::Angle(90.0),
        &[
            stop(Color::BLACK, None),
            stop(Color::rgb(248, 248, 248), None),
        ],
    )
}

fn command(rect: Rect, tile: Rect, gradient: LinearGradient) -> DrawCommand {
    DrawCommand::LinearGradient {
        rect,
        tile,
        radius: 0.0,
        gradient: Arc::new(gradient),
        current_color: Color::BLACK,
        stop_scale: 1.0,
    }
}

#[test]
fn pixel_centers_interpolate_legacy_srgb_without_endpoint_quantization() {
    let mut canvas = Canvas::new(4, 2).unwrap();
    let bounds = rect(0.0, 0.0, 4.0, 2.0);
    canvas.linear_gradient(bounds, bounds, 0.0, &gray(), Color::BLACK, 1.0);
    // Independent fractions (1,3,5,7)/8 multiplied by 248.
    assert_eq!(
        canvas.pixels,
        [0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9].repeat(2)
    );
}

#[test]
fn premultiplied_transparent_endpoint_has_no_dark_fringe() {
    let mut canvas = Canvas::new(4, 1).unwrap();
    let bounds = rect(0.0, 0.0, 4.0, 1.0);
    let source = gradient(
        GradientDirection::Angle(90.0),
        &[
            stop(Color::rgb(248, 0, 0), None),
            stop(Color::TRANSPARENT, None),
        ],
    );
    canvas.linear_gradient(bounds, bounds, 0.0, &source, Color::BLACK, 1.0);
    // Premultiplied red remains 248 when unpremultiplied; composite with
    // alpha (7,5,3,1)/8 over white before rounding the final channels.
    assert_eq!(canvas.pixels, [0xf92020, 0xfb6060, 0xfc9f9f, 0xfedfdf]);
}

#[test]
fn cardinal_directions_and_turns_have_exact_orientation() {
    for (angle, expected) in [
        (90.0, [0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9]),
        (450.0, [0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9]),
        (-90.0, [0xd9d9d9, 0x9b9b9b, 0x5d5d5d, 0x1f1f1f]),
    ] {
        let mut canvas = Canvas::new(4, 1).unwrap();
        let mut source = gray();
        source.direction = GradientDirection::Angle(angle);
        let bounds = rect(0.0, 0.0, 4.0, 1.0);
        canvas.linear_gradient(bounds, bounds, 0.0, &source, Color::BLACK, 1.0);
        assert_eq!(canvas.pixels, expected);
    }
    for (angle, expected) in [
        (180.0, [0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9]),
        (0.0, [0xd9d9d9, 0x9b9b9b, 0x5d5d5d, 0x1f1f1f]),
    ] {
        let mut canvas = Canvas::new(1, 4).unwrap();
        let mut source = gray();
        source.direction = GradientDirection::Angle(angle);
        let bounds = rect(0.0, 0.0, 1.0, 4.0);
        canvas.linear_gradient(bounds, bounds, 0.0, &source, Color::BLACK, 1.0);
        assert_eq!(canvas.pixels, expected);
    }
}

#[test]
fn corner_direction_depends_on_aspect_ratio() {
    let mut canvas = Canvas::new(4, 2).unwrap();
    let bounds = rect(0.0, 0.0, 4.0, 2.0);
    let source = gradient(
        GradientDirection::Corner {
            right: true,
            bottom: false,
        },
        &[
            stop(Color::rgb(255, 0, 0), Some(50.0)),
            stop(Color::rgb(0, 0, 255), Some(50.0)),
        ],
    );
    canvas.linear_gradient(bounds, bounds, 0.0, &source, Color::BLACK, 1.0);
    // t=(1+x/4-y/2)/2 at pixel centers. None lies on the boundary.
    assert_eq!(
        canvas.pixels,
        [
            0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, 0xff0000, 0xff0000, 0xff0000, 0x0000ff
        ]
    );
}

#[test]
fn descending_and_omitted_positions_follow_fixup_order() {
    let bounds = rect(0.0, 0.0, 100.0, 1.0);
    for (positions, expected) in [
        (
            [Some(20.0), None, None, Some(80.0)],
            [20.0, 40.0, 60.0, 80.0],
        ),
        (
            [Some(-50.0), None, None, Some(100.0)],
            [-50.0, 0.0, 50.0, 100.0],
        ),
        (
            [Some(75.0), Some(25.0), None, None],
            [75.0, 75.0, 87.5, 100.0],
        ),
        (
            [Some(150.0), None, None, None],
            [150.0, 150.0, 150.0, 150.0],
        ),
    ] {
        let source = gradient(
            GradientDirection::Angle(90.0),
            &positions.map(|p| stop(Color::BLACK, p)),
        );
        let resolved = ResolvedGradient::new(&source, bounds, Color::BLACK, 1.0).unwrap();
        assert_eq!(
            resolved.stops[..4]
                .iter()
                .map(|s| s.position)
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn coincident_stop_uses_last_color_at_the_boundary() {
    let source = gradient(
        GradientDirection::Angle(90.0),
        &[
            stop(Color::rgb(255, 0, 0), Some(50.0)),
            stop(Color::rgb(0, 255, 0), Some(50.0)),
            stop(Color::rgb(0, 0, 255), Some(50.0)),
        ],
    );
    let resolved =
        ResolvedGradient::new(&source, rect(0.0, 0.0, 4.0, 1.0), Color::BLACK, 1.0).unwrap();
    assert_eq!(resolved.sample(1.5, 0.5), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(resolved.sample(2.0, 0.5), [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(resolved.sample(2.5, 0.5), [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn background_tile_repeats_under_borders_without_stretching() {
    let mut canvas = Canvas::new(8, 2).unwrap();
    canvas.linear_gradient(
        rect(0.0, 0.0, 8.0, 2.0),
        rect(2.0, 0.0, 4.0, 2.0),
        0.0,
        &gray(),
        Color::BLACK,
        1.0,
    );
    assert_eq!(
        canvas.pixels,
        [
            0x9b9b9b, 0xd9d9d9, 0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9, 0x1f1f1f, 0x5d5d5d
        ]
        .repeat(2)
    );
}

#[test]
fn clipping_keeps_original_tile_phase() {
    let mut canvas = Canvas::new(6, 1).unwrap();
    canvas.set_clip(rect(2.25, 0.0, 2.0, 1.0));
    canvas.linear_gradient(
        rect(0.0, 0.0, 6.0, 1.0),
        rect(0.0, 0.0, 4.0, 1.0),
        0.0,
        &gray(),
        Color::BLACK,
        1.0,
    );
    assert_eq!(
        canvas.pixels,
        [0xffffff, 0xffffff, 0xffffff, 0xd9d9d9, 0x1f1f1f, 0xffffff]
    );
}

#[test]
fn single_current_color_resolves_at_the_consuming_element() {
    let source = gradient(
        GradientDirection::Angle(123.0),
        &[GradientStop {
            color: GradientColor::CurrentColor,
            position: Some(GradientPosition {
                px: -100.0,
                percent: 900.0,
            }),
        }],
    );
    let bounds = rect(0.0, 0.0, 4.0, 2.0);
    for color in [Color::rgb(255, 0, 0), Color::rgb(0, 255, 0)] {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.linear_gradient(bounds, bounds, 0.0, &source, color, 1.0);
        assert_eq!(canvas.pixels, vec![color.packed(); 8]);
    }
}

#[test]
fn zoom_scales_px_stops_and_keeps_percent_stops_relative() {
    let source = gradient(
        GradientDirection::Angle(90.0),
        &[
            GradientStop {
                color: GradientColor::Color(Color::BLACK),
                position: Some(GradientPosition {
                    px: 1.0,
                    percent: 25.0,
                }),
            },
            stop(Color::WHITE, Some(100.0)),
        ],
    );
    let resolved =
        ResolvedGradient::new(&source, rect(0.0, 0.0, 8.0, 2.0), Color::BLACK, 2.0).unwrap();
    assert_eq!(resolved.stops[0].position, 4.0);
    assert_eq!(resolved.stops[1].position, 8.0);
}

#[test]
fn display_list_translates_document_and_fixed_tiles_separately() {
    let bounds = rect(0.0, 0.0, 4.0, 1.0);
    let mut canvas = Canvas::new(6, 3).unwrap();
    canvas.paint_with_viewport(
        &[
            command(bounds, bounds, gray()),
            DrawCommand::PushFixed,
            command(bounds, bounds, gray()),
            DrawCommand::PopFixed,
        ],
        &Fonts::new(),
        &ImageStore::new(),
        (2.0, 1.0),
        (0.0, 2.0),
    );
    assert!(!canvas.exhausted());
    assert_eq!(
        canvas.pixels,
        [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9, 0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9,
            0xffffff, 0xffffff,
        ]
    );
}

#[test]
fn opacity_composites_gradient_as_one_group() {
    let bounds = rect(0.0, 0.0, 4.0, 1.0);
    let mut canvas = Canvas::new(4, 1).unwrap();
    let source = gradient(
        GradientDirection::Angle(90.0),
        &[
            stop(Color::rgb(248, 0, 0), None),
            stop(Color::TRANSPARENT, None),
        ],
    );
    canvas.paint(
        &[
            DrawCommand::PushOpacity { opacity: 0.5 },
            command(bounds, bounds, source),
            DrawCommand::PopOpacity,
        ],
        &Fonts::new(),
        &ImageStore::new(),
        0.0,
        0.0,
    );
    // Original alpha fractions divided by two; red still remains 248.
    assert_eq!(canvas.pixels, [0xfc8f8f, 0xfdafaf, 0xfecfcf, 0xffefef]);
    assert!(!canvas.exhausted());
}

#[test]
fn malformed_gradient_geometry_and_nonfinite_positions_are_inert() {
    let bounds = rect(0.0, 0.0, 4.0, 1.0);
    for source in [
        gradient(
            GradientDirection::Angle(f64::NAN),
            &[stop(Color::BLACK, None)],
        ),
        gradient(GradientDirection::Angle(0.0), &[]),
        gradient(
            GradientDirection::Angle(0.0),
            &vec![stop(Color::BLACK, None); 33],
        ),
        gradient(
            GradientDirection::Angle(0.0),
            &[stop(Color::BLACK, Some(f64::INFINITY))],
        ),
    ] {
        let mut canvas = Canvas::new(4, 1).unwrap();
        canvas.linear_gradient(bounds, bounds, 0.0, &source, Color::BLACK, 1.0);
        assert_eq!(canvas.pixels, [0xffffff; 4]);
    }
    for tile in [
        rect(0.0, 0.0, 0.0, 1.0),
        rect(f32::NAN, 0.0, 4.0, 1.0),
        rect(0.0, 0.0, 4.0, -1.0),
    ] {
        let mut canvas = Canvas::new(4, 1).unwrap();
        canvas.linear_gradient(bounds, tile, 0.0, &gray(), Color::BLACK, 1.0);
        assert_eq!(canvas.pixels, [0xffffff; 4]);
    }
}

#[test]
fn paint_work_is_reserved_before_any_gradient_pixel_is_written() {
    let bounds = rect(0.0, 0.0, 4.0, 1.0);
    // Two stops reserve eight resolution units, and four pixels each reserve
    // two binary-search units plus one evaluation/composite unit: total 20.
    for allowance in [19, 20] {
        let mut canvas = Canvas::new(4, 1).unwrap();
        canvas.budget = Some(PaintBudget {
            pixels: allowance,
            glyphs: 0,
        });
        canvas.linear_gradient(bounds, bounds, 0.0, &gray(), Color::BLACK, 1.0);
        if allowance == 19 {
            assert!(canvas.exhausted());
            assert_eq!(canvas.pixels, [0xffffff; 4]);
        } else {
            assert!(!canvas.exhausted());
            assert_eq!(canvas.pixels, [0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9]);
        }
    }
}
