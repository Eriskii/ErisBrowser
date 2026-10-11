//! Independent box-image literals and shared resource limits; no GPU required.
use super::box_decoration_tests::decode_opacity_pixels;
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use crate::graphics::{Canvas, Color, Rect as BrowserRect};
use eris_raster_core::DrawKind;

fn rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}
fn square(area: BrowserRect) -> RoundedBox {
    RoundedBox {
        rect: area,
        radii: [[0.0; 2]; 4],
    }
}
fn background(outer: BrowserRect, tile: BrowserRect, repeat: [bool; 2], key: &str) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: square(outer),
        cutout: None,
        paint: BoxPaint::Image {
            tile,
            repeat,
            key: key.into(),
        },
    }))
}
fn raster(width: u32, height: u32, rgba: &[u8]) -> Arc<RasterImage> {
    Arc::new(RasterImage {
        width,
        height,
        rgba: rgba.to_vec(),
    })
}
fn images() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert(
        "grid".into(),
        raster(
            2,
            2,
            &[
                255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255,
            ],
        ),
    );
    images.insert("rb".into(), raster(2, 1, &[255, 0, 0, 255, 0, 0, 255, 255]));
    images.insert(
        "alpha".into(),
        raster(2, 1, &[255, 0, 0, 128, 0, 0, 255, 0]),
    );
    images
}
fn pixels(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|c| match c {
            b'W' => 0xffffff,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            b'G' => 0x00ff00,
            b'Y' => 0xffff00,
            b'P' => 0xff7f7f,
            b'Q' => 0xffbfbf,
            b'D' => 0xffdfdf,
            b'S' => 0xff8080,
            b'T' => 0x8080ff,
            b'U' => 0xbfbfff,
            _ => panic!("unknown literal color"),
        })
        .collect()
}
fn checked(
    target: Frame,
    phase: Frame,
    commands: &[DrawCommand],
    images: &ImageStore,
    expected: &[u32],
) -> NativeScenePlan {
    let fonts = Fonts::new();
    let plan = plan_native_scene(
        target,
        &[NativePhase {
            frame: phase,
            commands,
        }],
        images,
        &fonts,
    )
    .unwrap();
    let mut canvas = Canvas::new(target.width, target.height).unwrap();
    canvas.clear(Color::rgb(
        (target.clear >> 16) as u8,
        (target.clear >> 8) as u8,
        target.clear as u8,
    ));
    canvas.set_clip(browser_rect(phase.caller_clip));
    canvas.paint_with_viewport(
        commands,
        &fonts,
        images,
        phase.document_offset,
        phase.viewport_offset,
    );
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    assert_eq!(decode_opacity_pixels(plan.plan()), expected);
    assert!(fonts.cache.borrow().is_empty());
    plan
}
fn image_route(plan: &NativeScenePlan, count: usize, work: u64) {
    let s = plan.stats();
    assert_eq!(s.box_background_images, count);
    assert_eq!(s.box_loop_work, work);
    assert_eq!(
        (
            s.box_masks,
            s.box_rectangles,
            s.mask_sources,
            s.coverage_bytes,
            s.row_entries
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(
        (s.generated_gradient_sources, s.generated_gradient_bytes),
        (0, 0)
    );
    assert!(!plan.plan().has_glyphs());
}
fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset * 4..offset * 4 + 4].try_into().unwrap())
}

#[test]
fn native_box_image_four_repeat_modes_keep_literal_luts_and_box_classification() {
    let frame = Frame::new(8, 4, 0xffffff);
    let cases = [
        (
            [false, false],
            130,
            (2, 1, 4, 2),
            ["WWWWWWWW", "WWRRBBWW", "WWGGYYWW", "WWWWWWWW"],
        ),
        (
            [true, false],
            190,
            (0, 1, 8, 2),
            ["WWWWWWWW", "BBRRBBRR", "YYGGYYGG", "WWWWWWWW"],
        ),
        (
            [false, true],
            180,
            (2, 0, 4, 4),
            ["WWGGYYWW", "WWRRBBWW", "WWGGYYWW", "WWRRBBWW"],
        ),
        (
            [true, true],
            296,
            (0, 0, 8, 4),
            ["YYGGYYGG", "BBRRBBRR", "YYGGYYGG", "BBRRBBRR"],
        ),
    ];
    for (repeat, work, bounds, rows) in cases {
        let command = background(
            rect(0.0, 0.0, 8.0, 4.0),
            rect(2.0, 1.0, 4.0, 2.0),
            repeat,
            "grid",
        );
        let plan = checked(frame, frame, &[command], &images(), &pixels(&rows));
        image_route(&plan, 1, work);
        assert_eq!(plan.stats().cpu_pixel_upper_bound, work);
        assert_eq!(plan.plan().draws()[1].kind(), DrawKind::Image);
        assert_eq!(plan.plan().draws()[1].bounds(), bounds);
        let p = &plan.plan().parameters()[PARAM_STRIDE..];
        let xs: Vec<_> = (0..bounds.2)
            .map(|x| word(plan.plan().input_bytes(), (word(p, 12) + x) as usize))
            .collect();
        let ys: Vec<_> = (0..bounds.3)
            .map(|y| word(plan.plan().input_bytes(), (word(p, 13) + y) as usize))
            .collect();
        assert_eq!(
            xs,
            if repeat[0] {
                vec![1, 1, 0, 0, 1, 1, 0, 0]
            } else {
                vec![0, 0, 1, 1]
            }
        );
        assert_eq!(
            ys,
            if repeat[1] {
                vec![1, 0, 1, 0]
            } else {
                vec![0, 1]
            }
        );
    }
}

#[test]
fn native_box_image_fractional_tiles_clips_and_subnormal_centers_are_literal() {
    let frame = Frame::new(4, 3, 0xffffff);
    for (repeat, work, rows) in [
        ([true, true], 136, ["BRBR", "YGYG", "BRBR"]),
        ([false, false], 63, ["BWWW", "YWWW", "WWWW"]),
    ] {
        let command = background(
            rect(0.0, 0.0, 4.0, 3.0),
            rect(-0.75, -0.25, 2.0, 2.0),
            repeat,
            "grid",
        );
        let plan = checked(frame, frame, &[command], &images(), &pixels(&rows));
        image_route(&plan, 1, work);
    }
    let frame = Frame::new(6, 3, 0xffffff);
    let mut phase = frame;
    phase.caller_clip = Rect::new(0.5, 0.25, 3.0, 1.5);
    for (repeat, work, rows) in [
        ([true, true], 61, ["WWWWWW", "WYGYWW", "WWWWWW"]),
        ([false, false], 39, ["WWWWWW", "WYWWWW", "WWWWWW"]),
    ] {
        let command = background(
            rect(0.0, 0.0, 6.0, 3.0),
            rect(0.25, 0.25, 2.0, 2.0),
            repeat,
            "grid",
        );
        let plan = checked(frame, phase, &[command], &images(), &pixels(&rows));
        image_route(&plan, 1, work);
    }
    let frame = Frame::new(4, 1, 0xffffff);
    let command = background(
        rect(0.0, 0.0, 4.0, 1.0),
        rect(0.5, 0.0, f32::from_bits(1), 1.0),
        [false, false],
        "grid",
    );
    let plan = checked(frame, frame, &[command], &images(), &pixels(&["GWWW"]));
    image_route(&plan, 1, 41);
    let frame = Frame::new(4, 2, 0xffffff);
    let commands = [
        DrawCommand::Image {
            rect: rect(0.5, 0.0, 2.0, 1.0),
            key: "rb".into(),
        },
        background(
            rect(0.0, 1.0, 4.0, 1.0),
            rect(0.5, 1.0, 2.0, 1.0),
            [false, false],
            "rb",
        ),
    ];
    let plan = checked(
        frame,
        frame,
        &commands,
        &images(),
        &pixels(&["RRBW", "RBWW"]),
    );
    image_route(&plan, 1, 47);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 50);
}

#[test]
fn native_box_image_alpha_and_nested_opacity_preserve_original_rgba16_rounding() {
    let frame = Frame::new(4, 5, 0xffffff);
    let mut commands = Vec::new();
    for (row, key, depth) in [
        (0, "alpha", 0),
        (1, "alpha", 1),
        (2, "alpha", 2),
        (3, "rb", 1),
        (4, "rb", 2),
    ] {
        for _ in 0..depth {
            commands.push(DrawCommand::PushOpacity { opacity: 0.5 });
        }
        commands.push(background(
            rect(0.0, row as f32, 4.0, 1.0),
            rect(0.0, row as f32, 2.0, 1.0),
            [true, true],
            key,
        ));
        for _ in 0..depth {
            commands.push(DrawCommand::PopOpacity);
        }
    }
    let plan = checked(
        frame,
        frame,
        &commands,
        &images(),
        &pixels(&["PWPW", "QWQW", "DWDW", "STST", "QUQU"]),
    );
    image_route(&plan, 5, 360);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 600);
}

#[test]
fn native_box_image_raw_tile_and_effective_square_follow_document_fixed_scopes() {
    let frame = Frame::new(6, 3, 0xffffff);
    let mut phase = frame;
    phase.caller_clip = Rect::new(1.0, 0.0, 4.0, 3.0);
    phase.document_offset = (1.0, 1.0);
    let commands = [
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 1.0, 1.0),
        },
        DrawCommand::PushFixed,
        background(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.5, 0.0, 2.0, 1.0),
            [false, false],
            "rb",
        ),
        DrawCommand::PopFixed,
        background(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.0, 0.0, 2.0, 1.0),
            [true, true],
            "rb",
        ),
        DrawCommand::PopClip,
        background(
            rect(0.0, 1.0, 4.0, 1.0),
            rect(0.0, 1.0, 2.0, 1.0),
            [true, true],
            "rb",
        ),
    ];
    let plan = checked(
        frame,
        phase,
        &commands,
        &images(),
        &pixels(&["WBWWWW", "WRWWWW", "WRBRBW"]),
    );
    image_route(&plan, 3, 150);
    phase = frame;
    phase.document_offset = (0.75, 0.75);
    let command = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: rect(0.25, 0.25, 4.0, 1.0),
            radii: [[0.0, 99.0], [99.0, 0.0], [0.0, 0.0], [1.0, 0.0]],
        },
        cutout: None,
        paint: BoxPaint::Image {
            tile: rect(-0.25, -0.75, 2.0, 1.0),
            repeat: [true, true],
            key: "rb".into(),
        },
    }));
    let plan = checked(
        frame,
        phase,
        &[command],
        &images(),
        &pixels(&["WWWWWW", "WBRBRW", "WWWWWW"]),
    );
    image_route(&plan, 1, 72);
}

#[test]
fn native_box_image_missing_nonrepeat_empty_and_suppressed_keep_policy_debits() {
    let frame = Frame::new(8, 4, 0xffffff);
    for (tile, repeat, key, work, missing) in [
        (rect(2.0, 1.0, 4.0, 2.0), [true, true], "missing", 296, 1),
        (rect(20.0, 20.0, 4.0, 2.0), [false, false], "grid", 72, 0),
        (
            rect(20.0, 20.0, 4.0, 2.0),
            [false, false],
            "https://missing.invalid/a",
            72,
            1,
        ),
    ] {
        let command = background(rect(0.0, 0.0, 8.0, 4.0), tile, repeat, key);
        let plan = checked(frame, frame, &[command], &images(), &[0xffffff; 32]);
        image_route(&plan, 1, work);
        assert_eq!(plan.stats().cpu_pixel_upper_bound, work);
        assert_eq!(plan.stats().bridge.missing_images, missing);
        assert_eq!(plan.plan().draws().len(), 1);
    }
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        background(
            rect(0.0, 0.0, 8.0, 4.0),
            rect(2.0, 1.0, 4.0, 2.0),
            [true, true],
            "missing",
        ),
        DrawCommand::PopOpacity,
    ];
    let plan = checked(frame, frame, &commands, &ImageStore::new(), &[0xffffff; 32]);
    image_route(&plan, 1, 296);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 360);
    let empty = background(
        rect(0.0, 0.0, 0.0, 1.0),
        rect(0.0, 0.0, 1.0, 1.0),
        [true, true],
        "missing",
    );
    let plan = checked(frame, frame, &[empty], &ImageStore::new(), &[0xffffff; 32]);
    image_route(&plan, 1, 0);
    assert_eq!(plan.stats().bridge.missing_images, 1);
}

#[test]
fn native_box_image_aliases_share_sources_with_both_old_image_commands() {
    let frame = Frame::new(4, 2, 0xffffff);
    let mut images = ImageStore::new();
    let red = raster(1, 1, &[255, 0, 0, 255]);
    images.insert("red".into(), red.clone());
    images.insert("alias".into(), red.clone());
    images.insert("unused".into(), raster(1, 1, &[0, 0, 255, 255]));
    let before = Arc::strong_count(&red);
    let mut commands = vec![
        DrawCommand::Image {
            rect: rect(0.0, 0.0, 1.0, 1.0),
            key: "red".into(),
        },
        background(
            rect(1.0, 0.0, 1.0, 1.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "alias",
        ),
        background(
            rect(2.0, 0.0, 1.0, 1.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "missing",
        ),
    ];
    let plan = checked(frame, frame, &commands, &images, &pixels(&["RRWW", "WWWW"]));
    image_route(&plan, 2, 78);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 79);
    let stats = plan.stats().bridge;
    assert_eq!(
        (
            stats.image_store_entries,
            stats.unique_sources,
            stats.referenced_sources,
            stats.total_rgba_bytes,
            stats.referenced_rgba_bytes,
            stats.missing_images
        ),
        (3, 2, 1, 8, 4, 1)
    );
    assert_eq!(stats.lowered_commands, 2);
    assert_eq!(Arc::strong_count(&red), before);
    commands.push(DrawCommand::BackgroundImage {
        rect: rect(3.0, 0.0, 1.0, 1.0),
        tile: rect(0.0, 0.0, 1.0, 1.0),
        repeat: [true, true],
        radius: 0.0,
        key: "alias".into(),
    });
    let plan = checked(frame, frame, &commands, &images, &pixels(&["RRWR", "WWWW"]));
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 108);
    assert_eq!(
        (
            plan.stats().bridge.unique_sources,
            plan.stats().bridge.referenced_sources
        ),
        (2, 1)
    );
    assert_eq!(Arc::strong_count(&red), before);
}

#[test]
fn native_box_image_atomic_fee_and_shared_million_cut_precede_packing() {
    let frame = Frame::new(8, 4, 0xffffff);
    let commands = [background(
        rect(0.0, 0.0, 8.0, 4.0),
        rect(2.0, 1.0, 4.0, 2.0),
        [false, false],
        "missing",
    )];
    let phases = [NativePhase {
        frame,
        commands: &commands,
    }];
    preflight(frame, &phases).unwrap();
    for initial in [129, 130] {
        let mut remaining = initial;
        let mut gradients = GradientReservation::new();
        let mut boxes = vec![None];
        let result =
            geometry_pixel_work(frame, &phases, &mut gradients, &mut boxes, &mut remaining);
        if initial == 130 {
            assert_eq!(result.unwrap(), 130);
            assert_eq!(remaining, 0);
            assert!(matches!(
                boxes[0].as_ref(),
                Some(BoxReservation::BackgroundImage { work: 130 })
            ));
        } else {
            assert_eq!(result.unwrap_err(), at(FallbackKind::CpuPaintBudget, 0, 0));
            assert_eq!(remaining, 129);
            assert!(boxes[0].is_none());
        }
    }
    let frame = Frame::new(128, 128, 0xffffff);
    let mut commands = vec![
        background(
            rect(0.0, 0.0, 128.0, 128.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "missing"
        );
        8
    ];
    for _ in 0..4 {
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 128.0, 128.0),
            color: Color::TRANSPARENT,
            radius: 0.0,
        });
    }
    commands.push(DrawCommand::Rect {
        rect: rect(0.0, 0.0, 67.0, 128.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    let plan = checked(
        frame,
        frame,
        &commands,
        &ImageStore::new(),
        &vec![0xffffff; 16_384],
    );
    image_route(&plan, 8, 925_888);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 1_000_000);
    assert_eq!(plan.stats().bridge.missing_images, 8);
    commands.push(DrawCommand::Rect {
        rect: rect(0.0, 0.0, 1.0, 1.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
}

#[test]
fn native_box_image_ineligibility_precedes_earlier_color_mask_and_source_preparation() {
    let frame = Frame::new(1000, 264, 0xffffff);
    let earlier = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: rect(0.0, 0.0, 1000.0, 264.0),
            radii: [[0.125; 2]; 4],
        },
        cutout: None,
        paint: BoxPaint::Color(Color::BLACK),
    }));
    let mut store = ImageStore::new();
    store.insert("poison".into(), raster(1, 1, &[]));
    for variant in 0..5 {
        let mut shape = square(rect(2000.0, 0.0, 0.0, 1.0));
        let mut cutout = None;
        match variant {
            0 => shape.rect.x = 2000.25,
            1 => shape.rect.height = 0.5,
            2 => shape.radii[0] = [1.0, 1.0],
            3 => cutout = Some(square(rect(0.0, 0.0, 0.0, 0.0))),
            _ => cutout = Some(shape),
        }
        let commands = [
            earlier.clone(),
            DrawCommand::PushOpacity { opacity: 0.0 },
            DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape,
                cutout,
                paint: BoxPaint::Image {
                    tile: rect(0.0, 0.0, 1.0, 1.0),
                    repeat: [true, true],
                    key: "missing".into(),
                },
            })),
            DrawCommand::PopOpacity,
        ];
        let fonts = Fonts::new();
        assert_eq!(
            plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &commands
                }],
                &store,
                &fonts
            )
            .unwrap_err(),
            at(FallbackKind::UnsupportedBoxDecoration, 0, 2)
        );
        assert!(fonts.cache.borrow().is_empty());
    }
}

#[test]
fn native_box_image_validation_and_key_command_source_limits_stay_shared() {
    let frame = Frame::new(4, 2, 0xffffff);
    let empty = rect(0.0, 0.0, 0.0, 0.0);
    let tile = rect(0.0, 0.0, 1.0, 1.0);
    for bad in [
        rect(0.0, 0.0, 0.0, 1.0),
        rect(0.0, 0.0, -1.0, 1.0),
        rect(f32::NAN, 0.0, 1.0, 1.0),
        rect(0.0, 0.0, 1_000_001.0, 1.0),
    ] {
        let commands = [background(empty, bad, [true, true], "missing")];
        assert_eq!(
            plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &commands
                }],
                &ImageStore::new(),
                &Fonts::new()
            )
            .unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    for bad in [f32::NAN, -1.0, 1_000_001.0] {
        let mut shape = square(empty);
        shape.radii[0][0] = bad;
        let commands = [DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape,
            cutout: None,
            paint: BoxPaint::Image {
                tile,
                repeat: [true, true],
                key: "missing".into(),
            },
        }))];
        assert_eq!(
            plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &commands
                }],
                &ImageStore::new(),
                &Fonts::new()
            )
            .unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    let too_long = background(empty, tile, [true, true], &"x".repeat(4097));
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &[too_long]
            }],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::KeyBudget, 0, 0)
    );
    let long = "x".repeat(4096);
    let mut commands = Vec::new();
    for _ in 0..6 {
        commands.push(DrawCommand::Image {
            rect: empty,
            key: long.clone(),
        });
    }
    for _ in 0..6 {
        commands.push(DrawCommand::BackgroundImage {
            rect: empty,
            tile,
            repeat: [true, true],
            radius: 0.0,
            key: long.clone(),
        });
    }
    for _ in 0..4 {
        commands.push(background(empty, tile, [true, true], &long));
    }
    let plan = checked(frame, frame, &commands, &ImageStore::new(), &[0xffffff; 8]);
    image_route(&plan, 4, 0);
    commands.push(background(empty, tile, [true, true], "x"));
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::KeyBudget, 0, 16)
    );
    let mut commands = vec![background(empty, tile, [true, true], "missing"); 256];
    let plan = checked(frame, frame, &commands, &ImageStore::new(), &[0xffffff; 8]);
    image_route(&plan, 256, 0);
    commands.push(background(empty, tile, [true, true], "missing"));
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        NativeFallback::new(FallbackKind::CommandLimit, Some(0), None)
    );
    let commands = [background(empty, tile, [true, true], "missing")];
    let mut store = ImageStore::new();
    store.insert("full".into(), raster(512, 512, &vec![255; 1_048_576]));
    let plan = checked(frame, frame, &commands, &store, &[0xffffff; 8]);
    assert_eq!(plan.stats().bridge.total_rgba_bytes, 1_048_576);
    store.insert("extra".into(), raster(1, 1, &[0, 0, 0, 255]));
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &store,
            &Fonts::new()
        )
        .unwrap_err(),
        NativeFallback::new(FallbackKind::InvalidImage, None, None)
    );
}

#[test]
fn native_box_image_full_viewport_uses_loaded_pixel_and_luts_without_mask() {
    let frame = Frame::new(1280, 1024, 0xffffff);
    let mut store = ImageStore::new();
    store.insert("blue".into(), raster(1, 1, &[0, 0, 255, 255]));
    let command = background(
        rect(0.0, 0.0, 1280.0, 1024.0),
        rect(0.0, 0.0, 1.0, 1.0),
        [true, true],
        "blue",
    );
    let plan = checked(frame, frame, &[command], &store, &vec![0x0000ff; 1_310_720]);
    image_route(&plan, 1, 9_184_280);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 9_184_280);
    assert_eq!(plan.plan().draws().len(), 2);
    assert_eq!(plan.plan().draws()[1].kind(), DrawKind::Image);
    assert_eq!(plan.plan().draws()[1].bounds(), (0, 0, 1280, 1024));
    assert_eq!(plan.plan().invocations(), 3_932_160);
    assert_eq!(plan.stats().bridge.total_rgba_bytes, 4);
    assert_eq!(plan.plan().input_bytes().len(), 9_220);
}
