use super::*;
use crate::graphics::gradient::{
    GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
};
use crate::graphics::{Canvas, ImageStore};
use std::sync::Arc;

fn image() -> Arc<LinearGradient> {
    Arc::new(LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: [Color::rgb(255, 0, 0), Color::rgb(0, 0, 255)]
            .map(|color| GradientStop {
                color: GradientColor::Color(color),
                position: Some(GradientPosition {
                    px: 0.0,
                    percent: 50.0,
                }),
            })
            .into(),
    })
}
fn explicit(width: Length, height: Length) -> BackgroundSize {
    BackgroundSize::Explicit { width, height }
}
fn position(x: Length, y: Length) -> BackgroundPosition {
    BackgroundPosition {
        x: PositionComponent {
            from_end: false,
            offset: x,
        },
        y: PositionComponent {
            from_end: false,
            offset: y,
        },
    }
}
fn center() -> BackgroundPosition {
    position(Length::Percent(50.0), Length::Percent(50.0))
}
fn prepare(source: &str, width: f32, height: f32) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(source);
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width, height);
    (doc, styles)
}
fn paint(result: &LayoutResult, width: u32, height: u32) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(
        &result.commands,
        &Fonts::new(),
        &ImageStore::new(),
        0.0,
        0.0,
    );
    assert!(!canvas.exhausted());
    canvas.pixels
}
fn rows(input: &[&str]) -> Vec<u32> {
    input
        .iter()
        .flat_map(|s| s.bytes())
        .map(|c| match c {
            b'W' => 0xffffff,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            b'G' => 0x00ff00,
            _ => panic!("literal palette"),
        })
        .collect()
}
fn gradients(result: &LayoutResult) -> Vec<(Rect, Rect, [bool; 2])> {
    result
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::LinearGradient {
                rect, tile, repeat, ..
            } => Some((*rect, *tile, *repeat)),
            DrawCommand::BoxDecoration(value) => match &value.paint {
                crate::graphics::box_decoration::BoxPaint::LinearGradient {
                    tile, repeat, ..
                } => Some((value.shape.rect, *tile, *repeat)),
                _ => None,
            },
            _ => None,
        })
        .collect()
}
fn hits(result: &LayoutResult) -> Vec<(NodeId, HitAction, Rect, bool)> {
    result
        .hit_regions
        .iter()
        .map(|h| (h.node, h.action, h.rect, h.fixed))
        .collect()
}

#[test]
fn gradient_sizes_use_the_positioning_area_without_an_intrinsic_ratio() {
    let area = rect(2.0, 3.0, 12.0, 6.0);
    for (size, expected) in [
        (BackgroundSize::default(), rect(2.0, 3.0, 12.0, 6.0)),
        (BackgroundSize::Cover, rect(2.0, 3.0, 12.0, 6.0)),
        (BackgroundSize::Contain, rect(2.0, 3.0, 12.0, 6.0)),
        (
            explicit(Length::Px(4.0), Length::Auto),
            rect(2.0, 3.0, 4.0, 6.0),
        ),
        (
            explicit(Length::Auto, Length::Px(2.0)),
            rect(2.0, 3.0, 12.0, 2.0),
        ),
        (
            explicit(Length::Percent(25.0), Length::Percent(50.0)),
            rect(2.0, 3.0, 3.0, 3.0),
        ),
        (
            explicit(
                Length::Calc {
                    px: -2.0,
                    percent: 50.0,
                    has_percent: true,
                },
                Length::Px(2.0),
            ),
            rect(2.0, 3.0, 4.0, 2.0),
        ),
    ] {
        assert_eq!(
            background_tile(area, size, BackgroundPosition::default()),
            Some(expected)
        );
    }
}

#[test]
fn position_uses_signed_remaining_space_and_physical_end_offsets() {
    let area = rect(0.0, 0.0, 12.0, 6.0);
    let size = explicit(Length::Px(4.0), Length::Px(2.0));
    for (position, expected) in [
        (center(), rect(4.0, 2.0, 4.0, 2.0)),
        (
            position(Length::Percent(25.0), Length::Percent(75.0)),
            rect(2.0, 3.0, 4.0, 2.0),
        ),
        (
            position(Length::Px(-1.0), Length::Px(0.25)),
            rect(-1.0, 0.25, 4.0, 2.0),
        ),
        (
            position(
                Length::Calc {
                    px: 1.0,
                    percent: 25.0,
                    has_percent: true,
                },
                Length::Percent(100.0),
            ),
            rect(3.0, 4.0, 4.0, 2.0),
        ),
        (
            BackgroundPosition {
                x: PositionComponent {
                    from_end: true,
                    offset: Length::Px(2.0),
                },
                y: PositionComponent {
                    from_end: true,
                    offset: Length::Px(1.0),
                },
            },
            rect(6.0, 3.0, 4.0, 2.0),
        ),
    ] {
        assert_eq!(background_tile(area, size, position), Some(expected));
    }
    assert_eq!(
        background_tile(area, explicit(Length::Px(16.0), Length::Px(2.0)), center()),
        Some(rect(-2.0, 2.0, 16.0, 2.0))
    );
}

#[test]
fn zero_clamped_or_unrepresentable_tiles_are_omitted_without_distortion() {
    let area = rect(0.0, 0.0, 12.0, 6.0);
    for length in [
        Length::Px(0.0),
        Length::Calc {
            px: -20.0,
            percent: 50.0,
            has_percent: true,
        },
        Length::Px(f32::INFINITY),
        Length::Px(f32::NAN),
        Length::Px(1_000_001.0),
        Length::Fr(1.0),
    ] {
        assert_eq!(
            background_tile(area, explicit(length, Length::Px(2.0)), center()),
            None
        );
    }
    assert_eq!(
        background_tile(
            area,
            explicit(Length::Px(4.0), Length::Px(2.0)),
            position(Length::Px(1_000_001.0), Length::Px(0.0))
        ),
        None
    );
    // Explicit sizes need no positive positioning-area dimensions.
    assert_eq!(
        background_tile(
            rect(1.0, 2.0, 0.0, 0.0),
            explicit(Length::Px(4.0), Length::Px(2.0)),
            BackgroundPosition::default()
        ),
        Some(rect(1.0, 2.0, 4.0, 2.0))
    );
    assert_eq!(
        background_tile(
            rect(1.0, 2.0, 0.0, 0.0),
            BackgroundSize::default(),
            BackgroundPosition::default()
        ),
        None
    );
    let tiny = f32::MIN_POSITIVE;
    assert_eq!(
        background_tile(
            area,
            explicit(Length::Px(tiny), Length::Px(2.0)),
            BackgroundPosition::default()
        )
        .unwrap()
        .width
        .to_bits(),
        tiny.to_bits()
    );
}

#[test]
fn repeat_axes_and_position_preserve_complete_literal_pixels_and_hits() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:12px;height:6px;background:white}</style><div id=a></div>",
        12.0,
        6.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let baseline = layout(&doc, &styles, 12.0, 6.0, &Fonts::new());
    styles[a].background_image = Some(BackgroundImage::Gradient(image()));
    styles[a].background_size = explicit(Length::Px(4.0), Length::Px(2.0));
    styles[a].background_position = center();
    for (repeat, expected) in [
        (
            [false, false],
            [
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
                "WWWWRRBBWWWW",
                "WWWWRRBBWWWW",
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
            ],
        ),
        (
            [true, false],
            [
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
                "RRBBRRBBRRBB",
                "RRBBRRBBRRBB",
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
            ],
        ),
        ([false, true], ["WWWWRRBBWWWW"; 6]),
        ([true, true], ["RRBBRRBBRRBB"; 6]),
    ] {
        styles[a].background_repeat = repeat;
        let result = layout(&doc, &styles, 12.0, 6.0, &Fonts::new());
        assert_eq!(
            gradients(&result),
            [(rect(0.0, 0.0, 12.0, 6.0), rect(4.0, 2.0, 4.0, 2.0), repeat)]
        );
        assert_eq!(paint(&result, 12, 6), rows(&expected));
        assert_eq!(hits(&result), hits(&baseline));
    }
    styles[a].background_repeat = [false; 2];
    styles[a].background_size = explicit(Length::Px(0.0), Length::Px(2.0));
    let result = layout(&doc, &styles, 12.0, 6.0, &Fonts::new());
    assert!(gradients(&result).is_empty());
    assert_eq!(paint(&result, 12, 6), vec![0xffffff; 72]);
    assert_eq!(hits(&result), hits(&baseline));
}

#[test]
fn padding_positioning_and_border_paint_are_independent() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:8px;height:4px;padding:1px;border:1px solid lime;background:white}</style><div id=a></div>",
        12.0,
        8.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let baseline = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    styles[a].background_image = Some(BackgroundImage::Gradient(image()));
    styles[a].background_size = explicit(Length::Px(4.0), Length::Px(2.0));
    styles[a].background_position = center();
    styles[a].background_repeat = [false; 2];
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert_eq!(
        gradients(&result),
        [(
            rect(0.0, 0.0, 12.0, 8.0),
            rect(4.0, 3.0, 4.0, 2.0),
            [false; 2]
        )]
    );
    assert_eq!(
        paint(&result, 12, 8),
        rows(&[
            "GGGGGGGGGGGG",
            "GWWWWWWWWWWG",
            "GWWWWWWWWWWG",
            "GWWWRRBBWWWG",
            "GWWWRRBBWWWG",
            "GWWWWWWWWWWG",
            "GWWWWWWWWWWG",
            "GGGGGGGGGGGG"
        ])
    );
    assert_eq!(hits(&result), hits(&baseline));
}

#[test]
fn propagated_canvas_uses_body_geometry_and_root_color_on_root_padding() {
    let (doc, styles) = prepare(
        "<style>html{margin:0;width:8px;height:4px;box-sizing:border-box;padding:1px;border:1px solid transparent;color:red;background-size:99px 99px}body{margin:0;width:2px;height:2px;color:blue;background-image:linear-gradient(to right,currentColor 50%,lime 50%);background-size:2px 1px;background-position:right bottom;background-repeat:no-repeat}</style>",
        12.0,
        8.0,
    );
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert_eq!(
        gradients(&result),
        [(
            rect(0.0, 0.0, 12.0, 8.0),
            rect(5.0, 2.0, 2.0, 1.0),
            [false; 2]
        )]
    );
    assert_eq!(
        paint(&result, 12, 8),
        rows(&[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWRGWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW"
        ])
    );
}

#[test]
fn inline_size_and_position_resolve_once_against_the_complete_virtual_strip() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0;width:4px;font-size:1px;line-height:2px}i{display:inline-block;width:2px;height:2px;vertical-align:top}</style><span id=a><i></i><i></i><i></i></span>",
        4.0,
        4.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let baseline = layout(&doc, &styles, 4.0, 4.0, &Fonts::new());
    styles[a].background_image = Some(BackgroundImage::Gradient(image()));
    styles[a].background_size = explicit(Length::Px(2.0), Length::Px(2.0));
    styles[a].background_position = position(Length::Percent(75.0), Length::Percent(0.0));
    styles[a].background_repeat = [false; 2];
    let result = layout(&doc, &styles, 4.0, 4.0, &Fonts::new());
    assert_eq!(
        gradients(&result).iter().map(|g| g.1).collect::<Vec<_>>(),
        [rect(3.0, 1.05, 2.0, 2.0), rect(-1.0, 3.05, 2.0, 2.0)]
    );
    assert_eq!(
        paint(&result, 4, 4),
        [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000,
            0xffffff, 0xffffff, 0xffffff, 0xffc0c0, 0x0000ff, 0xffffff, 0xffffff, 0xffffff,
        ]
    );
    assert_eq!(hits(&result), hits(&baseline));
}

#[test]
fn table_rows_and_late_stretched_cells_resolve_final_percentage_and_calc_sizes() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}table{width:14px}tr{height:6px}td{width:4px;height:2px;padding:0;vertical-align:bottom}i{display:block;width:1px;height:1px}</style><table><tr id=r><td id=a><i id=c></i></td><td></td></tr></table>",
        14.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let r = doc.query_selector("#r").unwrap();
    let baseline = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    for id in [a, r] {
        styles[id].background_image = Some(BackgroundImage::Gradient(image()));
        styles[id].background_size = explicit(Length::Percent(50.0), Length::Percent(50.0));
        styles[id].background_position = center();
        styles[id].background_repeat = [false; 2];
    }
    let result = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    let g = gradients(&result);
    assert!(g.contains(&(
        rect(2.0, 2.0, 10.0, 6.0),
        rect(4.5, 3.5, 5.0, 3.0),
        [false; 2]
    )));
    assert!(g.contains(&(
        rect(2.0, 2.0, 4.0, 6.0),
        rect(3.0, 3.5, 2.0, 3.0),
        [false; 2]
    )));
    assert_eq!(hits(&result), hits(&baseline));
    // Measured2px cell gives a zero used height; final6px cell revives it to1px.
    styles[a].background_size = explicit(
        Length::Px(2.0),
        Length::Calc {
            px: -2.0,
            percent: 50.0,
            has_percent: true,
        },
    );
    let result = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    assert!(gradients(&result).contains(&(
        rect(2.0, 2.0, 4.0, 6.0),
        rect(3.0, 4.5, 2.0, 1.0),
        [false; 2]
    )));
    assert_eq!(hits(&result), hits(&baseline));
}

#[test]
fn flex_and_grid_reflow_use_final_stretched_positioning_dimensions() {
    for parent in [
        "display:flex;width:12px;height:6px",
        "display:grid;width:12px;height:6px;grid-template-columns:12px;grid-template-rows:6px",
    ] {
        let (doc, mut styles) = prepare(
            &format!(
                "<style>body{{margin:0}}main{{{parent}}}#a{{flex:1}}</style><main><div id=a></div></main>"
            ),
            12.0,
            6.0,
        );
        let a = doc.query_selector("#a").unwrap();
        let baseline = layout(&doc, &styles, 12.0, 6.0, &Fonts::new());
        assert_eq!(
            baseline
                .hit_regions
                .iter()
                .find(|h| h.node == a)
                .unwrap()
                .rect,
            rect(0.0, 0.0, 12.0, 6.0)
        );
        styles[a].background_image = Some(BackgroundImage::Gradient(image()));
        styles[a].background_size = explicit(Length::Percent(50.0), Length::Percent(50.0));
        styles[a].background_position = center();
        styles[a].background_repeat = [false; 2];
        let result = layout(&doc, &styles, 12.0, 6.0, &Fonts::new());
        assert_eq!(
            gradients(&result),
            [(
                rect(0.0, 0.0, 12.0, 6.0),
                rect(3.0, 1.5, 6.0, 3.0),
                [false; 2]
            )]
        );
        assert_eq!(hits(&result), hits(&baseline));
    }
}

#[test]
fn transparent_rounded_interiors_keep_no_repeat_and_zero_size_holes() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:12px;height:8px;box-sizing:border-box;border:2px solid red;border-radius:3px}</style><div id=a></div>",
        12.0,
        8.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(image()));
    styles[a].background_repeat = [false; 2];
    for width in [2.0, 0.0] {
        styles[a].background_size = explicit(Length::Px(width), Length::Px(2.0));
        let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
        let pixels = paint(&result, 12, 8);
        assert_eq!(pixels[4 * 12 + 6], 0xffffff); // interior hole
        assert_eq!(pixels[6], 0xff0000); // top border
        if width == 0.0 {
            assert!(gradients(&result).is_empty());
        }
    }
    // An opaque backing color remains sufficient for the old rounded shortcut.
    styles[a].background_color = Color::rgb(0, 255, 0);
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert_eq!(paint(&result, 12, 8)[4 * 12 + 6], 0x00ff00);
    // This pins interior holes, not the pre-existing general rounded-ring corners.
}

#[test]
fn omitted_positioned_gradient_keeps_the_rounded_interior_transparent() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:12px;height:8px;box-sizing:border-box;border:2px solid red;border-radius:3px}</style><div id=a></div>",
        12.0,
        8.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let baseline = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    styles[a].background_image = Some(BackgroundImage::Gradient(image()));
    // Default size and repeat alone cannot prove coverage: adding a valid large
    // position to the 2px padding origin crosses the bounded command extent.
    styles[a].background_position = position(Length::Px(1_000_000.0), Length::Px(0.0));
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert!(gradients(&result).is_empty());
    let pixels = paint(&result, 12, 8);
    assert_eq!(pixels[4 * 12 + 6], 0xffffff);
    assert_eq!(pixels[6], 0xff0000);
    assert_eq!(hits(&result), hits(&baseline));
    styles[a].background_color = Color::rgb(0, 255, 0);
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert!(gradients(&result).is_empty());
    assert_eq!(paint(&result, 12, 8)[4 * 12 + 6], 0x00ff00);
}

#[test]
fn gradient_tile_translation_is_bounded_without_shifting_repetition_phase() {
    let command = |tile| DrawCommand::LinearGradient {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        tile,
        radius: 0.0,
        gradient: image(),
        current_color: Color::BLACK,
        stop_scale: 1.0,
        repeat: [true; 2],
    };
    for (tile, dx, dy, expected) in [
        (
            rect(1.0, 2.0, 4.0, 2.0),
            3.0,
            5.0,
            Some(rect(4.0, 7.0, 4.0, 2.0)),
        ),
        (
            rect(999_998.0, 0.0, 4.0, 2.0),
            2.0,
            0.0,
            Some(rect(1_000_000.0, 0.0, 4.0, 2.0)),
        ),
        (rect(999_998.0, 0.0, 4.0, 2.0), 3.0, 0.0, None),
        (
            rect(-999_998.0, 0.0, 4.0, 2.0),
            -2.0,
            0.0,
            Some(rect(-1_000_000.0, 0.0, 4.0, 2.0)),
        ),
        (rect(-999_998.0, 0.0, 4.0, 2.0), -3.0, 0.0, None),
        (rect(0.0, 999_998.0, 4.0, 2.0), 0.0, 3.0, None),
        (rect(0.0, -999_998.0, 4.0, 2.0), 0.0, -3.0, None),
        // Do not let f32 addition round an out-of-bound origin back to the limit.
        (rect(1_000_000.0, 0.0, 4.0, 2.0), 0.01, 0.0, None),
        (rect(0.0, 0.0, 4.0, 2.0), f32::NAN, 0.0, None),
    ] {
        let mut moved = command(tile);
        translate(&mut moved, dx, dy);
        let DrawCommand::LinearGradient { tile, .. } = moved else {
            panic!("gradient")
        };
        assert_eq!(tile, expected.unwrap_or_default());
    }
    let mut pending = command(Rect {
        x: 1.0,
        y: 2.0,
        width: -1.0,
        height: 2.0,
    });
    translate(&mut pending, 3.0, 5.0);
    assert!(
        matches!(&pending, DrawCommand::LinearGradient { tile, .. } if *tile == Rect { x: 4.0, y: 7.0, width: -1.0, height: 2.0 })
    );
    translate(&mut pending, 1_000_000.0, 0.0);
    assert!(
        matches!(&pending, DrawCommand::LinearGradient { tile, .. } if *tile == Rect::default())
    );
    translate(&mut pending, -1_000_000.0, 0.0);
    assert!(matches!(&pending, DrawCommand::LinearGradient { tile, .. } if tile.width == 0.0));
}

#[test]
fn table_placement_omits_out_of_range_tiles_and_preserves_exact_boundary_phase() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}table{width:14px}tr{height:6px}td{width:4px;height:2px;padding:0}</style><table><tr><td id=a></td><td></td></tr></table>",
        14.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let baseline = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    assert_eq!(paint(&baseline, 14, 10), vec![0xffffff; 140]);
    styles[a].background_image = Some(BackgroundImage::Gradient(image()));
    styles[a].background_size = explicit(Length::Px(4.0), Length::Px(2.0));
    styles[a].background_position = position(Length::Px(1_000_000.0), Length::Px(0.0));
    let omitted = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    assert!(gradients(&omitted).is_empty());
    assert_eq!(paint(&omitted, 14, 10), vec![0xffffff; 140]);
    assert_eq!(hits(&omitted), hits(&baseline));
    styles[a].background_position = position(Length::Px(999_998.0), Length::Px(0.0));
    let boundary = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    assert_eq!(
        gradients(&boundary),
        [(
            rect(2.0, 2.0, 4.0, 6.0),
            rect(1_000_000.0, 2.0, 4.0, 2.0),
            [true; 2]
        )]
    );
    assert_eq!(
        paint(&boundary, 14, 10),
        rows(&[
            "WWWWWWWWWWWWWW",
            "WWWWWWWWWWWWWW",
            "WWBBRRWWWWWWWW",
            "WWBBRRWWWWWWWW",
            "WWBBRRWWWWWWWW",
            "WWBBRRWWWWWWWW",
            "WWBBRRWWWWWWWW",
            "WWBBRRWWWWWWWW",
            "WWWWWWWWWWWWWW",
            "WWWWWWWWWWWWWW",
        ])
    );
    assert_eq!(hits(&boundary), hits(&baseline));
}
