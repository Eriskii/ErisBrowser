//! Literal adapter oracles frozen before Native BackgroundImage implementation.
use super::*;
use crate::graphics::{Canvas, Color};
use eris_raster_core::DrawKind;

fn rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}
fn background(paint: BrowserRect, tile: BrowserRect, repeat: [bool; 2], key: &str) -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: paint,
        tile,
        repeat,
        radius: 0.0,
        key: key.into(),
    }
}
fn image(width: u32, height: u32, rgba: Vec<u8>) -> Arc<RasterImage> {
    Arc::new(RasterImage {
        width,
        height,
        rgba,
    })
}
fn pattern() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert(
        "p".into(),
        image(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255,
            ],
        ),
    );
    images
}
fn single(
    target: Frame,
    commands: &[DrawCommand],
    images: &ImageStore,
) -> NativeResult<NativeScenePlan> {
    plan_native_scene(
        target,
        &[NativePhase {
            frame: target,
            commands,
        }],
        images,
        &Fonts::new(),
    )
}
fn colors(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| {
            row.bytes().map(|value| match value {
                b'R' => 0xff0000,
                b'B' => 0x0000ff,
                b'G' => 0x00ff00,
                b'Y' => 0xffff00,
                b'W' => 0xffffff,
                _ => panic!("unknown literal pixel"),
            })
        })
        .collect()
}
fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
fn parameter(plan: &Plan, draw: usize, field: usize) -> u32 {
    word(&plan.parameters()[draw * PARAM_STRIDE..], field)
}
// Read public packed RGBA/LUT words only. The expected pixels never use the
// candidate geometry helper; this does not execute a GPU shader.
fn pixels(plan: &Plan) -> Vec<u32> {
    let frame = plan.frame();
    let mut output = vec![0; frame.width as usize * frame.height as usize];
    for (index, draw) in plan.draws().iter().enumerate() {
        let p = |n| parameter(plan, index, n);
        let (x, y, width, height) = draw.bounds();
        for row in y..y + height {
            for column in x..x + width {
                let value = match draw.kind() {
                    DrawKind::Rectangle => p(6),
                    DrawKind::Image => {
                        let sx = word(plan.input_bytes(), (p(12) + column - x) as usize);
                        let sy = word(plan.input_bytes(), (p(13) + row - y) as usize);
                        word(plan.input_bytes(), (p(8) + sy * p(9) + sx) as usize)
                    }
                    _ => panic!("literal packed decoder excludes groups/text"),
                };
                let alpha = value >> 24;
                let at = (row * frame.width + column) as usize;
                let previous = output[at];
                let mut blended = 0;
                for shift in [0, 8, 16] {
                    blended |= ((((value >> shift) & 255) * alpha
                        + ((previous >> shift) & 255) * (255 - alpha)
                        + 127)
                        / 255)
                        << shift;
                }
                output[at] = blended;
            }
        }
    }
    output
}
fn canvas(target: Frame, phases: &[NativePhase<'_>], images: &ImageStore) -> Vec<u32> {
    let mut out = Canvas::new(target.width, target.height).unwrap();
    out.clear(Color::WHITE);
    for phase in phases {
        out.set_clip(browser_rect(phase.frame.caller_clip));
        out.paint_with_viewport(
            phase.commands,
            &Fonts::new(),
            images,
            phase.frame.document_offset,
            phase.frame.viewport_offset,
        );
        assert!(!out.exhausted());
    }
    out.pixels
}
fn same_plan(a: &Plan, b: &Plan) {
    assert_eq!(a.parameters(), b.parameters());
    assert_eq!(a.input_bytes(), b.input_bytes());
    assert_eq!(a.invocations(), b.invocations());
    assert_eq!(a.gpu_buffer_bytes(), b.gpu_buffer_bytes());
    assert_eq!(a.group_scratch_bytes(), b.group_scratch_bytes());
}

#[test]
fn native_background_four_repeat_masks_have_literal_pixels_luts_and_work() {
    let target = Frame::new(8, 4, 0xffffff);
    let images = pattern();
    let cases = [
        (
            [false, false],
            58,
            (2, 1, 4, 2),
            ["WWWWWWWW", "WWRRBBWW", "WWGGYYWW", "WWWWWWWW"],
        ),
        (
            [true, false],
            118,
            (0, 1, 8, 2),
            ["WWWWWWWW", "BBRRBBRR", "YYGGYYGG", "WWWWWWWW"],
        ),
        (
            [false, true],
            108,
            (2, 0, 4, 4),
            ["WWGGYYWW", "WWRRBBWW", "WWGGYYWW", "WWRRBBWW"],
        ),
        (
            [true, true],
            224,
            (0, 0, 8, 4),
            ["YYGGYYGG", "BBRRBBRR", "YYGGYYGG", "BBRRBBRR"],
        ),
    ];
    for (repeat, work, bounds, rows) in cases {
        let commands = [background(
            rect(0.0, 0.0, 8.0, 4.0),
            rect(2.0, 1.0, 4.0, 2.0),
            repeat,
            "p",
        )];
        let result = single(target, &commands, &images).unwrap();
        assert_eq!(result.stats().cpu_pixel_upper_bound, work);
        assert_eq!(result.plan().draws()[1].bounds(), bounds);
        assert_eq!(pixels(result.plan()), colors(&rows));
        assert_eq!(
            canvas(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &images
            ),
            colors(&rows)
        );
        let xs: Vec<_> = (0..bounds.2)
            .map(|x| {
                word(
                    result.plan().input_bytes(),
                    (parameter(result.plan(), 1, 12) + x) as usize,
                )
            })
            .collect();
        let ys: Vec<_> = (0..bounds.3)
            .map(|y| {
                word(
                    result.plan().input_bytes(),
                    (parameter(result.plan(), 1, 13) + y) as usize,
                )
            })
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
        assert_eq!(
            (
                result.stats().generated_gradient_sources,
                result.stats().generated_gradient_bytes
            ),
            (0, 0)
        );
        assert!(result.retained_cpu_bytes().unwrap() < result.stats().preparation_peak_bytes);
    }
}

#[test]
fn native_background_negative_fractional_phase_has_independent_literal_rows() {
    let target = Frame::new(4, 3, 0xffffff);
    let images = pattern();
    for (repeat, work, rows) in [
        ([true, true], 104, ["BRBR", "YGYG", "BRBR"]),
        ([false, false], 31, ["BWWW", "YWWW", "WWWW"]),
    ] {
        let commands = [background(
            rect(0.0, 0.0, 4.0, 3.0),
            rect(-0.75, -0.25, 2.0, 2.0),
            repeat,
            "p",
        )];
        let result = single(target, &commands, &images).unwrap();
        assert_eq!(result.stats().cpu_pixel_upper_bound, work);
        assert_eq!(pixels(result.plan()), colors(&rows));
        assert_eq!(
            canvas(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &images
            ),
            colors(&rows)
        );
    }
}

#[test]
fn native_background_fractional_clip_and_subnormal_nonrepeat_use_pixel_centers() {
    let target = Frame::new(6, 4, 0xffffff);
    let mut phase = target;
    phase.caller_clip = Rect::new(1.25, 0.25, 3.5, 2.5);
    let mut images = ImageStore::new();
    images.insert(
        "p".into(),
        image(
            3,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 255, 255, 255, 255, 0, 255, 255,
                255, 255, 0, 255,
            ],
        ),
    );
    let commands = [background(
        rect(0.0, 0.0, 6.0, 4.0),
        rect(0.0, 0.0, 6.0, 4.0),
        [false, false],
        "p",
    )];
    let phases = [NativePhase {
        frame: phase,
        commands: &commands,
    }];
    let result = plan_native_scene(target, &phases, &images, &Fonts::new()).unwrap();
    let expected = vec![
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x00ff00,
        0x00ff00, 0x0000ff, 0xffffff, 0xffffff, 0xffffff, 0xff00ff, 0xff00ff, 0xffff00, 0xffffff,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
    ];
    assert_eq!(result.stats().cpu_pixel_upper_bound, 49);
    assert_eq!(result.plan().draws()[1].bounds(), (2, 1, 3, 2));
    assert_eq!(pixels(result.plan()), expected);
    assert_eq!(canvas(target, &phases, &images), expected);
    let target = Frame::new(4, 1, 0xffffff);
    let images = pattern();
    let commands = [background(
        rect(0.0, 0.0, 4.0, 1.0),
        rect(0.5, 0.0, f32::from_bits(1), 1.0),
        [false, false],
        "p",
    )];
    let result = single(target, &commands, &images).unwrap();
    assert_eq!(result.stats().cpu_pixel_upper_bound, 25);
    assert_eq!(result.plan().draws()[1].bounds(), (0, 0, 1, 1));
    assert_eq!(pixels(result.plan()), colors(&["GWWW"]));
}

#[test]
fn native_background_keeps_old_image_edge_sampling_separate() {
    let target = Frame::new(4, 1, 0xffffff);
    let mut images = ImageStore::new();
    images.insert(
        "p".into(),
        image(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]),
    );
    let old = [DrawCommand::Image {
        rect: rect(0.5, 0.0, 2.0, 1.0),
        key: "p".into(),
    }];
    let new = [background(
        rect(0.0, 0.0, 4.0, 1.0),
        rect(0.5, 0.0, 2.0, 1.0),
        [false, false],
        "p",
    )];
    let a = single(target, &old, &images).unwrap();
    let b = single(target, &new, &images).unwrap();
    assert_eq!(a.stats().cpu_pixel_upper_bound, 3);
    assert_eq!(b.stats().cpu_pixel_upper_bound, 31);
    assert_eq!(pixels(a.plan()), colors(&["RRBW"]));
    assert_eq!(pixels(b.plan()), colors(&["RBWW"]));
    assert_eq!(
        canvas(
            target,
            &[NativePhase {
                frame: target,
                commands: &old
            }],
            &images
        ),
        colors(&["RRBW"])
    );
    assert_eq!(
        canvas(
            target,
            &[NativePhase {
                frame: target,
                commands: &new
            }],
            &images
        ),
        colors(&["RBWW"])
    );
}

#[test]
fn native_background_mixed_aliases_share_loaded_source_identity_and_missing_stats() {
    let target = Frame::new(4, 2, 0xffffff);
    let mut images = ImageStore::new();
    let red = image(1, 1, vec![255, 0, 0, 255]);
    images.insert("red".into(), red.clone());
    images.insert("alias".into(), red.clone());
    images.insert("unused".into(), image(1, 1, vec![0, 0, 255, 255]));
    let before = Arc::strong_count(&red);
    let commands = [
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
    let result = single(target, &commands, &images).unwrap();
    let stats = result.stats().bridge;
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
    assert_eq!(result.stats().cpu_pixel_upper_bound, 59);
    assert_eq!(Arc::strong_count(&red), before);
    assert_eq!(pixels(result.plan()), colors(&["RRWW", "WWWW"]));
}

#[test]
fn native_background_rounded_preflight_stays_cold_inside_hidden_missing_scopes() {
    let target = Frame::new(8, 4, 0xffffff);
    let mut images = ImageStore::new();
    images.insert("malformed".into(), image(1, 1, Vec::new()));
    for radius in [-1.0, 1.0, 2.0] {
        let fonts = Fonts::new();
        let mut rounded = background(
            rect(100.0, 100.0, 0.0, 0.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "missing",
        );
        if let DrawCommand::BackgroundImage { radius: r, .. } = &mut rounded {
            *r = radius;
        }
        let commands = [
            DrawCommand::Text {
                x: 0.0,
                y: 0.0,
                text: "cold".into(),
                size: 12.0,
                color: Color::BLACK,
                bold: false,
                italic: false,
                monospace: false,
            },
            DrawCommand::PushOpacity { opacity: 0.0 },
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 0.0, 0.0),
            },
            rounded,
            DrawCommand::PopClip,
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            plan_native_scene(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &images,
                &fonts
            )
            .unwrap_err(),
            at(FallbackKind::UnsupportedBackgroundImage, 0, 3)
        );
        assert!(fonts.cache.borrow().is_empty());
    }
}

#[test]
fn native_background_invalid_geometry_and_key_are_checked_before_hidden_elision() {
    let target = Frame::new(8, 4, 0xffffff);
    let images = ImageStore::new();
    for tile in [
        rect(0.0, 0.0, 0.0, 1.0),
        rect(0.0, 0.0, -1.0, 1.0),
        rect(f32::NAN, 0.0, 1.0, 1.0),
        rect(0.0, 0.0, f32::MAX, 1.0),
    ] {
        let commands = [background(
            rect(0.0, 0.0, 0.0, 0.0),
            tile,
            [false, false],
            "missing",
        )];
        assert_eq!(
            single(target, &commands, &images).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    for radius in [f32::NAN, f32::INFINITY, 1_000_001.0] {
        let mut command = background(
            rect(0.0, 0.0, 0.0, 0.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "missing",
        );
        if let DrawCommand::BackgroundImage { radius: r, .. } = &mut command {
            *r = radius;
        }
        assert_eq!(
            single(target, &[command], &images).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    let commands = [background(
        rect(0.0, 0.0, 0.0, 0.0),
        rect(0.0, 0.0, 1.0, 1.0),
        [true, true],
        &"x".repeat(4097),
    )];
    assert_eq!(
        single(target, &commands, &images).unwrap_err(),
        at(FallbackKind::KeyBudget, 0, 0)
    );
}

#[test]
fn native_background_missing_and_suppressed_sources_keep_conservative_work() {
    let target = Frame::new(8, 4, 0xffffff);
    let command = background(
        rect(0.0, 0.0, 8.0, 4.0),
        rect(2.0, 1.0, 4.0, 2.0),
        [true, true],
        "missing",
    );
    let result = single(target, std::slice::from_ref(&command), &ImageStore::new()).unwrap();
    assert_eq!(result.stats().cpu_pixel_upper_bound, 224);
    assert_eq!(result.stats().bridge.missing_images, 1);
    assert_eq!(result.plan().draws().len(), 1);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        command,
        DrawCommand::PopOpacity,
    ];
    let result = single(target, &commands, &ImageStore::new()).unwrap();
    assert_eq!(result.stats().cpu_pixel_upper_bound, 288);
    assert_eq!(result.stats().bridge.missing_images, 1);
    let mut malformed = ImageStore::new();
    malformed.insert("unreferenced".into(), image(1, 1, Vec::new()));
    assert_eq!(
        single(target, &commands, &malformed).unwrap_err(),
        NativeFallback::new(FallbackKind::InvalidImage, None, None)
    );
}

#[test]
fn native_background_cpu_admission_exact_million_and_one_over_precedes_packing() {
    let target = Frame::new(128, 128, 0xffffff);
    let mut commands = vec![
        background(
            rect(0.0, 0.0, 128.0, 128.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "missing"
        );
        12
    ];
    commands.push(DrawCommand::Rect {
        rect: rect(0.0, 0.0, 70.0, 64.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    // 12*(81928 candidate work +1032 LUT upper bound)+4480 old Rect pixels.
    let result = single(target, &commands, &ImageStore::new()).unwrap();
    assert_eq!(result.stats().cpu_pixel_upper_bound, 1_000_000);
    assert_eq!(result.stats().bridge.missing_images, 12);
    commands.push(DrawCommand::Rect {
        rect: rect(0.0, 0.0, 1.0, 1.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    assert_eq!(
        single(target, &commands, &ImageStore::new()).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
}

#[test]
fn native_background_key_command_source_and_store_caps_stay_shared() {
    let target = Frame::new(4, 2, 0xffffff);
    let long = "x".repeat(4096);
    let empty = rect(0.0, 0.0, 0.0, 0.0);
    let tile = rect(0.0, 0.0, 1.0, 1.0);
    let mut commands = vec![background(empty, tile, [true, true], &long); 16];
    assert!(single(target, &commands, &ImageStore::new()).is_ok());
    commands.push(background(empty, tile, [true, true], "x"));
    assert_eq!(
        single(target, &commands, &ImageStore::new()).unwrap_err(),
        at(FallbackKind::KeyBudget, 0, 16)
    );
    let mut commands = vec![background(empty, tile, [true, true], "x"); 256];
    assert!(single(target, &commands, &ImageStore::new()).is_ok());
    commands.push(background(empty, tile, [true, true], "x"));
    assert_eq!(
        single(target, &commands, &ImageStore::new()).unwrap_err(),
        NativeFallback::new(FallbackKind::CommandLimit, Some(0), None)
    );
    let mut images = ImageStore::new();
    images.insert("full".into(), image(512, 512, vec![255; 1_048_576]));
    assert_eq!(
        single(target, &[], &images)
            .unwrap()
            .stats()
            .bridge
            .total_rgba_bytes,
        1_048_576
    );
    images.insert("extra".into(), image(1, 1, vec![255; 4]));
    assert_eq!(
        single(target, &[], &images).unwrap_err(),
        NativeFallback::new(FallbackKind::InvalidImage, None, None)
    );
    let mut sparse = ImageStore::new();
    sparse.reserve(MAX_STORE_CAPACITY + 1);
    assert_eq!(
        single(target, &[], &sparse).unwrap_err(),
        NativeFallback::new(FallbackKind::ImageStoreLimit, None, None)
    );
}

#[test]
fn native_background_fixed_restore_and_phase_reset_preserve_raw_tile_offsets() {
    let target = Frame::new(6, 3, 0xffffff);
    let mut first = target;
    first.document_offset = (1.0, 1.0);
    first.caller_clip = Rect::new(1.0, 0.0, 4.0, 3.0);
    let mut second = target;
    second.document_offset = (4.0, 2.0);
    let mut images = ImageStore::new();
    images.insert("r".into(), image(1, 1, vec![255, 0, 0, 255]));
    images.insert("b".into(), image(1, 1, vec![0, 0, 255, 255]));
    let commands = [
        background(
            rect(0.0, 0.0, 1.0, 1.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "r",
        ),
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 0.0, 0.0),
        },
        DrawCommand::PushFixed,
        background(
            rect(1.0, 0.0, 1.0, 1.0),
            rect(1.0, 0.0, 1.0, 1.0),
            [true, true],
            "b",
        ),
        DrawCommand::PopFixed,
        background(
            rect(0.0, 0.0, 6.0, 3.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "b",
        ),
        DrawCommand::PopClip,
        background(
            rect(0.0, 1.0, 4.0, 1.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "r",
        ),
        background(
            rect(1.0, 1.0, 1.0, 1.0),
            rect(0.0, 0.0, 1.0, 1.0),
            [true, true],
            "b",
        ),
    ];
    let last = [background(
        rect(0.0, 0.0, 1.0, 1.0),
        rect(0.0, 0.0, 1.0, 1.0),
        [true, true],
        "b",
    )];
    let phases = [
        NativePhase {
            frame: first,
            commands: &commands,
        },
        NativePhase {
            frame: second,
            commands: &last,
        },
    ];
    let result = plan_native_scene(target, &phases, &images, &Fonts::new()).unwrap();
    assert_eq!(result.stats().cpu_pixel_upper_bound, 172);
    assert_eq!(
        pixels(result.plan()),
        colors(&["WBWWWW", "WRWWWW", "WRBRBW"])
    );
    assert_eq!(
        canvas(target, &phases, &images),
        colors(&["WBWWWW", "WRWWWW", "WRBRBW"])
    );
}

#[test]
fn native_background_source_alpha_and_nested_opacity_reuse_core_groups() {
    let target = Frame::new(4, 1, 0xffffff);
    let bytes = vec![255, 0, 0, 128, 0, 0, 255, 0];
    let mut images = ImageStore::new();
    images.insert("p".into(), image(2, 1, bytes.clone()));
    for (depth, work, pixel) in [(0, 56, 0xff7f7f), (1, 64, 0xffbfbf), (2, 72, 0xffdfdf)] {
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.5 }; depth];
        commands.push(background(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.0, 0.0, 2.0, 1.0),
            [true, true],
            "p",
        ));
        commands.extend(vec![DrawCommand::PopOpacity; depth]);
        let result = single(target, &commands, &images).unwrap();
        assert_eq!(result.stats().cpu_pixel_upper_bound, work);
        let expected = vec![pixel, 0xffffff, pixel, 0xffffff];
        assert_eq!(
            canvas(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &images
            ),
            expected
        );
        if depth == 0 {
            assert_eq!(pixels(result.plan()), expected);
        }
        let mut core = vec![Command::PushOpacity(0.5); depth];
        core.push(Command::BackgroundImage {
            rect: Rect::new(0.0, 0.0, 4.0, 1.0),
            tile: Rect::new(0.0, 0.0, 2.0, 1.0),
            repeat: [true, true],
            source: 0,
        });
        core.extend(vec![Command::PopOpacity; depth]);
        let direct = eris_raster_core::plan_native_phases(
            target,
            &[Phase {
                frame: target,
                commands: &core,
            }],
            &[SourceImage {
                width: 2,
                height: 1,
                rgba: &bytes,
            }],
            &[],
            &[],
        )
        .unwrap();
        same_plan(result.plan(), &direct);
    }
}
