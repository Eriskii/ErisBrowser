use super::*;
use crate::graphics::{Canvas, PaintBudget};

fn area(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn hard() -> LinearGradient {
    LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: Arc::from([
            GradientStop {
                color: GradientColor::Color(Color::rgb(255, 0, 0)),
                position: Some(GradientPosition {
                    px: 0.0,
                    percent: 50.0,
                }),
            },
            GradientStop {
                color: GradientColor::Color(Color::rgb(0, 0, 255)),
                position: Some(GradientPosition {
                    px: 0.0,
                    percent: 50.0,
                }),
            },
        ]),
    }
}
fn pixels(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|c| match c {
            b'W' => 0xffffff,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            _ => panic!("invalid literal pixel"),
        })
        .collect()
}

#[test]
fn repeat_axes_have_independent_literal_strips() {
    let paint = area(0.0, 0.0, 12.0, 6.0);
    let tile = area(4.0, 2.0, 4.0, 2.0);
    for (repeat, rows) in [
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
        let mut canvas = Canvas::new(12, 6).unwrap();
        canvas.linear_gradient_with_repeat(paint, tile, 0.0, &hard(), Color::BLACK, 1.0, repeat);
        assert!(!canvas.exhausted());
        assert_eq!(canvas.pixels, pixels(&rows), "{repeat:?}");
    }
}

#[test]
fn nonrepeat_uses_half_open_pixel_centers_at_fractional_and_negative_edges() {
    for (x, row) in [
        (0.25, "RBWW"),
        (0.5, "RBWW"),
        (0.75, "WRBW"),
        (-0.75, "BWWW"),
        (-1.5, "WWWW"),
    ] {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.linear_gradient_with_repeat(
            area(0.0, 0.0, 4.0, 2.0),
            area(x, 0.5, 2.0, 1.0),
            0.0,
            &hard(),
            Color::BLACK,
            1.0,
            [false, false],
        );
        assert_eq!(canvas.pixels, pixels(&[row, "WWWW"]), "x={x}");
    }
    let mut canvas = Canvas::new(4, 2).unwrap();
    canvas.clip = area(1.25, 0.0, 2.5, 2.0);
    canvas.linear_gradient_with_repeat(
        area(0.0, 0.0, 4.0, 2.0),
        area(0.75, 0.5, 2.0, 1.0),
        0.0,
        &hard(),
        Color::BLACK,
        1.0,
        [false, false],
    );
    assert_eq!(canvas.pixels, pixels(&["WWBW", "WWWW"]));
}

#[test]
fn nonrepeat_retains_original_rounded_paint_outline_and_transparent_holes() {
    let mut canvas = Canvas::new(12, 6).unwrap();
    canvas.linear_gradient_with_repeat(
        area(0.0, 0.0, 12.0, 6.0),
        area(4.0, 2.0, 4.0, 2.0),
        3.0,
        &hard(),
        Color::BLACK,
        1.0,
        [false, false],
    );
    assert_eq!(
        canvas.pixels,
        pixels(&[
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWRRBBWWWW",
            "WWWWRRBBWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW"
        ])
    );
}

#[test]
fn nonrepeat_preserves_positive_subnormal_tile_at_an_exact_pixel_center() {
    let gradient = LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: Arc::from([GradientStop {
            color: GradientColor::Color(Color::rgb(255, 0, 0)),
            position: None,
        }]),
    };
    let tiny = f32::from_bits(1);
    for (origin, row) in [(0.5, "RWWW"), (1.5, "WRWW"), (2.5, "WWRW")] {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.linear_gradient_with_repeat(
            area(0.0, 0.0, 4.0, 2.0),
            area(origin, 0.5, tiny, tiny),
            0.0,
            &gradient,
            Color::BLACK,
            1.0,
            [false, false],
        );
        assert_eq!(canvas.pixels, pixels(&[row, "WWWW"]));
    }
}

#[test]
fn nonrepeat_precharges_exact_bounded_window_work_before_paint() {
    let paint = area(0.0, 0.0, 12.0, 6.0);
    let tile = area(4.0, 2.0, 4.0, 2.0);
    // Nondefault preparation:16 fixed units +8 stops +8 pixels*(3 sampling+2 axis checks).
    for (available, exhausted) in [(64, false), (63, true)] {
        let mut canvas = Canvas::new(12, 6).unwrap();
        canvas.budget = Some(PaintBudget {
            pixels: available,
            glyphs: 0,
        });
        canvas.linear_gradient_with_repeat(
            paint,
            tile,
            0.0,
            &hard(),
            Color::BLACK,
            1.0,
            [false, false],
        );
        assert_eq!(canvas.exhausted(), exhausted);
        if exhausted {
            assert!(canvas.pixels.iter().all(|&p| p == 0xffffff));
        } else {
            assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0);
        }
    }
    let mut canvas = Canvas::new(12, 6).unwrap();
    canvas.budget = Some(PaintBudget {
        pixels: 224,
        glyphs: 0,
    });
    canvas.linear_gradient_with_repeat(paint, tile, 0.0, &hard(), Color::BLACK, 1.0, [true, true]);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0); // Existing default tariff remains8+72*3.
}

#[cfg(feature = "raster-bridge")]
#[test]
fn nonrepeat_opaque_preparation_has_no_transparent_padding_or_double_offset() {
    let gradient = hard();
    let paint = area(0.0, 0.0, 12.0, 6.0);
    let prepared = OpaqueGradient::new_with_repeat(
        12,
        6,
        paint,
        area(4.0, 2.0, 4.0, 2.0),
        paint,
        &gradient,
        Color::BLACK,
        1.0,
        [false, false],
    )
    .unwrap()
    .unwrap();
    let info = prepared.info();
    assert_eq!(
        (info.x, info.y, info.width, info.height, info.loop_work),
        (4, 2, 4, 2, 64)
    );
    assert_eq!(
        prepared.materialize().unwrap(),
        [
            255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255
        ]
        .repeat(2)
    );
    assert!(
        OpaqueGradient::new_with_repeat(
            12,
            6,
            paint,
            area(-4.0, 0.0, 2.0, 2.0),
            paint,
            &gradient,
            Color::BLACK,
            1.0,
            [false, false]
        )
        .unwrap()
        .is_none()
    );
}

#[cfg(feature = "raster-bridge")]
#[test]
fn nonrepeat_prepared_pixels_match_canvas_with_fractional_clips_angles_and_axes() {
    for angle in [0.0, 37.0, 90.0, 180.0, 270.0] {
        for x in [-0.75, 0.25, 0.5, 0.75, 4.25] {
            for repeat in [[false, false], [false, true], [true, false]] {
                let mut gradient = hard();
                gradient.direction = GradientDirection::Angle(angle);
                let paint = area(0.0, 0.0, 12.0, 6.0);
                let tile = area(x, 0.25, 3.5, 2.75);
                let clip = area(0.25, 0.0, 10.5, 5.5);
                let mut canvas = Canvas::new(12, 6).unwrap();
                canvas.clip = clip;
                canvas.linear_gradient_with_repeat(
                    paint,
                    tile,
                    0.0,
                    &gradient,
                    Color::BLACK,
                    1.0,
                    repeat,
                );
                let prepared = OpaqueGradient::new_with_repeat(
                    12,
                    6,
                    paint,
                    tile,
                    clip,
                    &gradient,
                    Color::BLACK,
                    1.0,
                    repeat,
                )
                .unwrap();
                let mut expected = vec![0xffffff; 72];
                if let Some(prepared) = prepared {
                    let info = prepared.info();
                    let bytes = prepared.materialize().unwrap();
                    for (i, rgba) in bytes.as_chunks::<4>().0.iter().enumerate() {
                        assert_eq!(rgba[3], 255);
                        let x = info.x + i as u32 % info.width;
                        let y = info.y + i as u32 / info.width;
                        expected[(y * 12 + x) as usize] =
                            u32::from_be_bytes([0, rgba[0], rgba[1], rgba[2]]);
                    }
                }
                assert_eq!(canvas.pixels, expected, "{angle}/{x}/{repeat:?}");
            }
        }
    }
}
