use super::*;
use crate::graphics::{DrawCommand, Fonts, ImageStore, PaintBudget};
use std::sync::Arc;

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn color(code: char) -> u32 {
    match code {
        'W' => 0xffffff,
        'K' => 0,
        'R' => 0xff0000,
        'G' => 0x00ff00,
        'B' => 0x0000ff,
        'C' => 0x00ffff,
        'M' => 0xff00ff,
        'Y' => 0xffff00,
        'D' => 0x9f9f9f,
        'E' => 0x151515,
        'P' => 0xff8080,
        'Q' => 0x8080ff,
        _ => panic!("unknown literal pixel"),
    }
}

fn raster(rows: &[&str]) -> RasterImage {
    assert!(rows.iter().all(|row| row.len() == rows[0].len()));
    RasterImage {
        width: rows[0].len() as u32,
        height: rows.len() as u32,
        rgba: rows
            .iter()
            .flat_map(|row| row.chars())
            .flat_map(|c| {
                let p = color(c);
                [(p >> 16) as u8, (p >> 8) as u8, p as u8, 255]
            })
            .collect(),
    }
}

fn expected(canvas: &Canvas, rows: &[&str]) {
    assert_eq!(canvas.width as usize, rows[0].len());
    assert_eq!(canvas.height as usize, rows.len());
    let pixels: Vec<_> = rows.iter().flat_map(|row| row.chars().map(color)).collect();
    assert_eq!(canvas.pixels, pixels);
    assert!(!canvas.exhausted());
}

fn command(area: Rect, tile: Rect, repeat: [bool; 2]) -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: area,
        tile,
        repeat,
        radius: 0.0,
        key: "tile".into(),
    }
}

#[test]
fn background_image_independent_axis_modes_have_literal_pixels() {
    let image = raster(&["RB", "GY"]);
    let variants = [
        (
            [false, false],
            ["WWWWWWWW", "WWRRBBWW", "WWGGYYWW", "WWWWWWWW"],
        ),
        (
            [true, false],
            ["WWWWWWWW", "BBRRBBRR", "YYGGYYGG", "WWWWWWWW"],
        ),
        (
            [false, true],
            ["WWGGYYWW", "WWRRBBWW", "WWGGYYWW", "WWRRBBWW"],
        ),
        (
            [true, true],
            ["YYGGYYGG", "BBRRBBRR", "YYGGYYGG", "BBRRBBRR"],
        ),
    ];
    for (repeat, rows) in variants {
        let mut canvas = Canvas::new(8, 4).unwrap();
        canvas.background_image(
            rect(0.0, 0.0, 8.0, 4.0),
            rect(2.0, 1.0, 4.0, 2.0),
            0.0,
            &image,
            repeat,
        );
        expected(&canvas, &rows);
    }
}

#[test]
fn background_image_negative_fractional_tiles_and_scaling() {
    let image = raster(&["RB", "GY"]);
    for (repeat, rows) in [
        ([true; 2], ["BRBR", "YGYG", "BRBR"]),
        ([false; 2], ["BWWW", "YWWW", "WWWW"]),
    ] {
        let mut canvas = Canvas::new(4, 3).unwrap();
        canvas.background_image(
            rect(0.0, 0.0, 4.0, 3.0),
            rect(-0.75, -0.25, 2.0, 2.0),
            0.0,
            &image,
            repeat,
        );
        expected(&canvas, &rows);
    }
    let mut canvas = Canvas::new(7, 5).unwrap();
    canvas.background_image(
        rect(0.0, 0.0, 7.0, 5.0),
        rect(-1.0, -1.0, 6.0, 4.0),
        0.0,
        &raster(&["RGB", "CMY"]),
        [true; 2],
    );
    expected(
        &canvas,
        &["RGGBBRR", "CMMYYCC", "CMMYYCC", "RGGBBRR", "RGGBBRR"],
    );
    for (source, width, row) in [("RGB", 1.5, "GRBG"), ("RGBC", 3.0, "RBCR")] {
        let mut canvas = Canvas::new(4, 1).unwrap();
        canvas.background_image(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.0, 0.0, width, 1.0),
            0.0,
            &raster(&[source]),
            [true; 2],
        );
        expected(&canvas, &[row]);
    }
}

#[test]
fn background_image_half_open_centers_preserve_old_image_sampling() {
    let image = raster(&["RB"]);
    for (x, row) in [
        (0.25, "RBWW"),
        (0.5, "RBWW"),
        (0.75, "WRBW"),
        (-0.75, "BWWW"),
        (-1.5, "WWWW"),
    ] {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.background_image(
            rect(0.0, 0.0, 4.0, 2.0),
            rect(x, 0.5, 2.0, 1.0),
            0.0,
            &image,
            [false; 2],
        );
        expected(&canvas, &[row, "WWWW"]);
    }
    let mut canvas = Canvas::new(4, 1).unwrap();
    canvas.image(rect(0.5, 0.0, 2.0, 1.0), &image);
    expected(&canvas, &["RRBW"]);
}

#[test]
fn background_image_fractional_clip_and_rounded_paint_outline() {
    let mut canvas = Canvas::new(6, 4).unwrap();
    canvas.set_clip(rect(1.25, 0.25, 3.5, 2.5));
    canvas.background_image(
        rect(0.0, 0.0, 6.0, 4.0),
        rect(0.0, 0.0, 6.0, 4.0),
        0.0,
        &raster(&["RGB", "CMY"]),
        [false; 2],
    );
    expected(&canvas, &["WWWWWW", "WWGGBW", "WWMMYW", "WWWWWW"]);
    for radius in [2.0, 200.0] {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.background_image(
            rect(0.0, 0.0, 4.0, 4.0),
            rect(0.5, 0.5, 1.0, 1.0),
            radius,
            &raster(&["K"]),
            [true; 2],
        );
        expected(&canvas, &["DEED", "EKKE", "EKKE", "DEED"]);
    }
    let mut canvas = Canvas::new(4, 4).unwrap();
    canvas.background_image(
        rect(0.0, 0.0, 4.0, 4.0),
        rect(1.0, 1.0, 2.0, 2.0),
        2.0,
        &raster(&["R"]),
        [false; 2],
    );
    expected(&canvas, &["WWWW", "WRRW", "WRRW", "WWWW"]);
}

#[test]
fn background_image_source_alpha_and_isolated_opacity() {
    let image = RasterImage {
        width: 2,
        height: 1,
        rgba: vec![255, 0, 0, 128, 0, 0, 255, 0],
    };
    let mut canvas = Canvas::new(4, 1).unwrap();
    canvas.background_image(
        rect(0.0, 0.0, 4.0, 1.0),
        rect(0.0, 0.0, 2.0, 1.0),
        0.0,
        &image,
        [true; 2],
    );
    assert_eq!(canvas.pixels, [0xff7f7f, 0xffffff, 0xff7f7f, 0xffffff]);
    let mut images = ImageStore::new();
    images.insert("tile".into(), Arc::new(raster(&["RB"])));
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        DrawCommand::PushClip {
            rect: rect(1.0, 0.0, 2.0, 2.0),
        },
        command(
            rect(0.0, 0.0, 4.0, 2.0),
            rect(0.0, 0.0, 4.0, 2.0),
            [false; 2],
        ),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
        DrawCommand::Rect {
            rect: rect(3.0, 1.0, 1.0, 1.0),
            color: Color::BLACK,
            radius: 0.0,
        },
    ];
    let mut canvas = Canvas::new(4, 2).unwrap();
    canvas.paint(&commands, &Fonts::new(), &images, 0.0, 0.0);
    expected(&canvas, &["WPQW", "WPQK"]);
    let mut nested = vec![DrawCommand::PushOpacity { opacity: 0.5 }];
    nested.extend(commands[..5].iter().cloned());
    nested.push(DrawCommand::PopOpacity);
    let mut canvas = Canvas::new(4, 2).unwrap();
    canvas.paint(&nested, &Fonts::new(), &images, 0.0, 0.0);
    assert_eq!(
        canvas.pixels,
        [
            0xffffff, 0xffbfbf, 0xbfbfff, 0xffffff, 0xffffff, 0xffbfbf, 0xbfbfff, 0xffffff
        ]
    );
    assert!(!canvas.exhausted());
}

#[test]
fn background_image_fixed_and_document_offsets_restore_both_boxes() {
    let mut images = ImageStore::new();
    images.insert("tile".into(), Arc::new(raster(&["RB"])));
    let commands = [
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 1.0, 1.0),
        },
        DrawCommand::PushFixed,
        command(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.5, 0.0, 2.0, 1.0),
            [false; 2],
        ),
        DrawCommand::PopFixed,
        command(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.0, 0.0, 2.0, 1.0),
            [true; 2],
        ),
        DrawCommand::PopClip,
        command(
            rect(0.0, 1.0, 4.0, 1.0),
            rect(0.0, 1.0, 2.0, 1.0),
            [true; 2],
        ),
    ];
    let mut canvas = Canvas::new(6, 3).unwrap();
    canvas.set_clip(rect(1.0, 0.0, 4.0, 3.0));
    canvas.paint_with_viewport(&commands, &Fonts::new(), &images, (1.0, 1.0), (0.0, 0.0));
    expected(&canvas, &["WBWWWW", "WRWWWW", "WRBRBW"]);
    assert_eq!(canvas.clip, rect(1.0, 0.0, 4.0, 3.0));
}

#[test]
fn background_image_subnormal_huge_and_rounded_modulo_seams() {
    let area = rect(0.0, 0.0, 3.0, 2.0);
    for (repeat, rows) in [([true; 2], ["RRR", "RRR"]), ([false; 2], ["RWW", "WWW"])] {
        let mut canvas = Canvas::new(3, 2).unwrap();
        canvas.background_image(
            area,
            rect(0.5, 0.5, f32::from_bits(1), f32::from_bits(1)),
            0.0,
            &raster(&["R"]),
            repeat,
        );
        expected(&canvas, &rows);
    }
    for extent in [0.25, 1_048_576.0] {
        let mut canvas = Canvas::new(3, 2).unwrap();
        canvas.background_image(
            area,
            rect(0.0, 0.0, extent, extent),
            0.0,
            &raster(&["RB", "GY"]),
            [true; 2],
        );
        expected(&canvas, &["RRR", "RRR"]);
    }
    let mut canvas = Canvas::new(1, 1).unwrap();
    canvas.background_image(
        rect(0.0, 0.0, 1.0, 1.0),
        rect(1.0, 0.0, f32::MAX, 1.0),
        0.0,
        &raster(&["RB"]),
        [true; 2],
    );
    expected(&canvas, &["B"]);
}

#[test]
fn background_image_invalid_geometry_and_short_sources_skip() {
    let area = rect(0.0, 0.0, 2.0, 2.0);
    let good = raster(&["R"]);
    let mut cases = Vec::new();
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for axis in 0..4 {
            let mut fields = [0.0, 0.0, 2.0, 2.0];
            fields[axis] = invalid;
            let bad = rect(fields[0], fields[1], fields[2], fields[3]);
            cases.push((bad, area, 0.0));
            cases.push((area, bad, 0.0));
        }
        cases.push((area, area, invalid));
    }
    for value in [0.0, -1.0] {
        cases.push((rect(0.0, 0.0, value, 2.0), area, 0.0));
        cases.push((area, rect(0.0, 0.0, value, 2.0), 0.0));
        cases.push((area, rect(0.0, 0.0, 2.0, value), 0.0));
    }
    cases.push((area, area, -1.0));
    cases.push((area, rect(f32::MAX, 0.0, f32::MAX, 1.0), 0.0));
    for (rect, tile, radius) in cases {
        let mut canvas = Canvas::new(2, 2).unwrap();
        canvas.background_image(rect, tile, radius, &good, [true; 2]);
        expected(&canvas, &["WW", "WW"]);
    }
    for (width, height, bytes) in [(0, 1, 4), (1, 0, 4), (1, 1, 3), (u32::MAX, u32::MAX, 4)] {
        let mut canvas = Canvas::new(2, 2).unwrap();
        canvas.background_image(
            area,
            area,
            0.0,
            &RasterImage {
                width,
                height,
                rgba: vec![0; bytes],
            },
            [true; 2],
        );
        expected(&canvas, &["WW", "WW"]);
    }
    // Direct trusted raster storage permits trailing bytes like existing Image;
    // IPC continues to require exact raster payload lengths.
    let mut padded = good;
    padded.rgba.extend([1, 2, 3, 4]);
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.background_image(area, area, 0.0, &padded, [true; 2]);
    expected(&canvas, &["RR", "RR"]);
}

#[test]
fn background_image_prepays_exact_candidate_work_and_recovers() {
    let area = rect(0.0, 0.0, 2.0, 2.0);
    for (repeat, cost) in [
        ([false; 2], 20),
        ([true, false], 24),
        ([false, true], 24),
        ([true; 2], 28),
    ] {
        for allowance in [cost - 1, cost] {
            let mut canvas = Canvas::new(2, 2).unwrap();
            canvas.budget = Some(PaintBudget {
                pixels: allowance,
                glyphs: 1,
            });
            canvas.background_image(area, area, 0.0, &raster(&["R"]), repeat);
            assert_eq!(
                canvas.pixels,
                vec![
                    if allowance == cost {
                        0xff0000
                    } else {
                        0xffffff
                    };
                    4
                ]
            );
            assert_eq!(canvas.exhausted(), allowance < cost);
            assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0);
            canvas.paint(
                &[DrawCommand::Rect {
                    rect: area,
                    color: Color::BLACK,
                    radius: 0.0,
                }],
                &Fonts::new(),
                &ImageStore::new(),
                0.0,
                0.0,
            );
            expected(&canvas, &["KK", "KK"]);
        }
    }
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.budget = Some(PaintBudget {
        pixels: 11,
        glyphs: 1,
    });
    canvas.background_image(
        area,
        rect(0.5, 0.5, f32::from_bits(1), f32::from_bits(1)),
        0.0,
        &raster(&["R"]),
        [false; 2],
    );
    expected(&canvas, &["RW", "WW"]);
    assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0);
}

#[test]
fn background_image_transparent_candidates_cost_work_without_layers() {
    let area = rect(0.0, 0.0, 2.0, 2.0);
    let transparent = RasterImage {
        width: 1,
        height: 1,
        rgba: vec![255, 0, 0, 0],
    };
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.budget = Some(PaintBudget {
        pixels: 28,
        glyphs: 1,
    });
    canvas.background_image(area, area, 0.0, &transparent, [true; 2]);
    expected(&canvas, &["WW", "WW"]);
    assert_eq!(canvas.budget.as_ref().unwrap().pixels, 0);
    let mut images = ImageStore::new();
    images.insert("tile".into(), Arc::new(transparent));
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.paint(
        &[
            DrawCommand::PushOpacity { opacity: 0.5 },
            command(area, area, [true; 2]),
            DrawCommand::PopOpacity,
        ],
        &Fonts::new(),
        &images,
        0.0,
        0.0,
    );
    expected(&canvas, &["WW", "WW"]);
    assert_eq!(canvas.layer_allocated, 0);
}

#[test]
fn background_image_layer_allocation_failure_discards_pending_group() {
    let area = rect(0.0, 0.0, 2.0, 2.0);
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.begin_opacity(0.5, area);
    canvas.budget = Some(PaintBudget {
        pixels: 31,
        glyphs: 1,
    });
    // 28 sampling units leave three; the first visible pixel needs four layer units.
    canvas.background_image(area, area, 0.0, &raster(&["R"]), [true; 2]);
    assert!(canvas.exhausted());
    assert_eq!(canvas.pixels, [0xffffff; 4]);
    assert_eq!(canvas.layer_allocated, 0);
    assert!(canvas.end_opacity());
    assert!(canvas.layers.is_empty());
}

#[test]
fn background_image_missing_key_and_suppressed_group_are_inert() {
    let area = rect(0.0, 0.0, 2.0, 2.0);
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.paint(
        &[command(area, area, [true; 2])],
        &Fonts::new(),
        &ImageStore::new(),
        0.0,
        0.0,
    );
    expected(&canvas, &["WW", "WW"]);
    let mut images = ImageStore::new();
    images.insert("tile".into(), Arc::new(raster(&["R"])));
    canvas.paint(
        &[
            DrawCommand::PushOpacity { opacity: 0.0 },
            command(area, area, [true; 2]),
            DrawCommand::PopOpacity,
        ],
        &Fonts::new(),
        &images,
        0.0,
        0.0,
    );
    expected(&canvas, &["WW", "WW"]);
    assert_eq!(canvas.layer_allocated, 0);
}
