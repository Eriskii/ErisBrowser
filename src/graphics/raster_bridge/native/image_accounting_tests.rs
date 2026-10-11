//! Independent literal accounting and pixel oracles, authored from the existing
//! Canvas and Native planner before reading the image accounting candidate.
use super::*;

fn image_rect(key: &str, x: f32, y: f32, width: f32, height: f32) -> DrawCommand {
    DrawCommand::Image {
        rect: BrowserRect {
            x,
            y,
            width,
            height,
        },
        key: key.into(),
    }
}

fn image_clip(x: f32, y: f32, width: f32, height: f32) -> DrawCommand {
    DrawCommand::PushClip {
        rect: BrowserRect {
            x,
            y,
            width,
            height,
        },
    }
}

fn image_plan(
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

fn red_images() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert("red".into(), raster([255, 0, 0, 255]));
    images
}

fn identical_packed_plans(left: &Plan, right: &Plan) {
    assert_eq!(left.parameters(), right.parameters());
    assert_eq!(left.input_bytes(), right.input_bytes());
    assert_eq!(left.invocations(), right.invocations());
    assert_eq!(left.gpu_buffer_bytes(), right.gpu_buffer_bytes());
    assert_eq!(left.group_scratch_bytes(), right.group_scratch_bytes());
}

#[test]
fn image_accounting_literal_destination_bounds_and_pixels() {
    let target = Frame::new(3, 3, 0xffffff);
    let images = red_images();
    let fonts = Fonts::new();
    let cases = [
        ((1.0, 1.0, 2.0, 1.0), 2, vec![4usize, 5]),
        ((0.25, 0.25, 1.5, 1.5), 4, vec![0, 1, 3, 4]),
        ((-0.75, -0.25, 1.5, 1.5), 2, vec![0, 3]),
        ((2.75, 2.75, 1.0, 1.0), 1, vec![8]),
        ((-1.0, -1.0, 2.0, 2.0), 1, vec![0]),
        ((0.0, 0.0, 0.0, 1.0), 0, vec![]),
        ((0.0, 0.0, -1.0, 1.0), 0, vec![]),
        ((-2.0, 0.0, 1.0, 1.0), 0, vec![]),
        ((3.0, 0.0, 1.0, 1.0), 0, vec![]),
    ];
    for ((x, y, width, height), work, painted_indices) in cases {
        let commands = [image_rect("red", x, y, width, height)];
        let scene = image_plan(target, &commands, &images).unwrap();
        assert_eq!(scene.stats().cpu_pixel_upper_bound, work);
        assert_eq!(scene.stats().bridge.original_commands, 1);
        assert_eq!(scene.stats().bridge.referenced_sources, 1);
        assert_eq!(scene.stats().rounded_loop_work, 0);
        assert_eq!(scene.stats().coverage_bytes, 0);
        let mut expected = vec![0xffffff; 9];
        for index in painted_indices {
            expected[index] = 0xff0000;
        }
        assert_eq!(decode_pixels(scene.plan()), expected);
        assert_eq!(
            canvas_pixels(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &images,
                &fonts
            ),
            expected
        );
        // The packed rendering plan is unchanged relative to direct core input.
        let rgba = [255, 0, 0, 255];
        let core = eris_raster_core::plan_with_masks_for_profile(
            Profile::Native,
            target,
            &[Command::Image {
                rect: Rect::new(x, y, width, height),
                source: 0,
            }],
            &[SourceImage {
                width: 1,
                height: 1,
                rgba: &rgba,
            }],
            &[],
            &[],
        )
        .unwrap();
        identical_packed_plans(scene.plan(), &core);
    }
}

#[test]
fn image_accounting_fractional_clip_charges_rejected_edge_pixels() {
    let target = Frame::new(3, 3, 0xffffff);
    let images = red_images();
    let fonts = Fonts::new();
    for nested in [false, true] {
        let mut input = target;
        let mut commands = Vec::new();
        if nested {
            commands.push(image_clip(0.25, 0.25, 1.5, 1.5));
        } else {
            input.caller_clip = Rect::new(0.25, 0.25, 1.5, 1.5);
        }
        commands.push(image_rect("red", 0.0, 0.0, 3.0, 3.0));
        if nested {
            commands.push(DrawCommand::PopClip);
        }
        let phases = [NativePhase {
            frame: input,
            commands: &commands,
        }];
        let scene = plan_native_scene(target, &phases, &images, &fonts).unwrap();
        // Image loops over four integer origins; blend rejects three. The
        // rectangle accounting rule would charge only one and is wrong here.
        assert_eq!(scene.stats().cpu_pixel_upper_bound, 4);
        let expected = [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000, 0xffffff, 0xffffff, 0xffffff,
            0xffffff,
        ];
        assert_eq!(decode_pixels(scene.plan()), expected);
        assert_eq!(canvas_pixels(target, &phases, &images, &fonts), expected);
    }
    let mut input = target;
    input.caller_clip = Rect::new(-2.0, -2.0, 3.0, 3.0);
    let commands = [image_rect("red", -2.0, -2.0, 4.0, 4.0)];
    let phases = [NativePhase {
        frame: input,
        commands: &commands,
    }];
    let scene = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 1);
    let expected = [
        0xff0000, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
    ];
    assert_eq!(decode_pixels(scene.plan()), expected);
    assert_eq!(canvas_pixels(target, &phases, &images, &fonts), expected);
}

#[test]
fn image_accounting_fixed_escape_restore_and_phase_reset() {
    let target = Frame::new(8, 4, 0xffffff);
    let mut first = target;
    first.document_offset = (-3.0, 0.0);
    first.viewport_offset = (1.0, 0.0);
    let a = [
        image_clip(0.0, 0.0, 0.0, 0.0),
        DrawCommand::PushFixed,
        image_rect("red", 0.0, 0.0, 2.0, 2.0),
        DrawCommand::PopFixed,
        image_rect("blue", 0.0, 0.0, 8.0, 4.0),
        DrawCommand::PopClip,
        image_rect("green", 3.0, 2.0, 2.0, 1.0),
    ];
    let mut second = target;
    second.caller_clip = Rect::new(4.0, 0.0, 2.0, 4.0);
    second.viewport_offset = (4.0, 0.0);
    let b = [
        DrawCommand::PushFixed,
        image_rect("blue", 0.0, 0.0, 4.0, 1.0),
        DrawCommand::PopFixed,
    ];
    let phases = [
        NativePhase {
            frame: first,
            commands: &a,
        },
        NativePhase {
            frame: second,
            commands: &b,
        },
    ];
    let mut images = red_images();
    images.insert("green".into(), raster([0, 255, 0, 255]));
    images.insert("blue".into(), raster([0, 0, 255, 255]));
    let fonts = Fonts::new();
    let scene = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 8); // 4 + 0 + 2 + 2
    assert_eq!(scene.stats().bridge.original_commands, 10);
    let expected = [
        0xffffff, 0xff0000, 0xff0000, 0xffffff, 0x0000ff, 0x0000ff, 0xffffff, 0xffffff, 0xffffff,
        0xff0000, 0xff0000, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x00ff00, 0x00ff00,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
    ];
    assert_eq!(decode_pixels(scene.plan()), expected);
    assert_eq!(canvas_pixels(target, &phases, &images, &fonts), expected);
}

#[test]
fn image_accounting_opacity_does_not_discount_destination_work() {
    let target = Frame::new(3, 3, 0xffffff);
    let images = red_images();
    let fonts = Fonts::new();
    for (opacity, pixel) in [(0.0, 0xffffff), (0.5, 0xff8080), (1.0, 0xff0000)] {
        let commands = [
            DrawCommand::PushOpacity { opacity },
            image_clip(0.25, 0.25, 1.5, 1.5),
            image_rect("red", 0.0, 0.0, 3.0, 3.0),
            DrawCommand::PopClip,
            DrawCommand::PopOpacity,
        ];
        let scene = image_plan(target, &commands, &images).unwrap();
        assert_eq!(scene.stats().cpu_pixel_upper_bound, 22); // two full-area markers + four image loop pixels
        assert_eq!(scene.stats().bridge.original_commands, 5);
        let mut expected = [0xffffff; 9];
        expected[4] = pixel;
        assert_eq!(
            canvas_pixels(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &images,
                &fonts
            ),
            expected
        );
        if opacity == 0.5 {
            assert!(
                scene
                    .plan()
                    .draws()
                    .iter()
                    .any(|draw| draw.kind() == DrawKind::GroupComposite)
            );
            assert!(scene.plan().group_scratch_bytes() > 0);
        }
        assert!(scene.plan().gpu_buffer_bytes() <= scene.stats().gpu_buffer_upper_bound);
    }
}

#[test]
fn image_accounting_missing_transparent_and_aliased_sources_keep_per_command_cost() {
    let target = Frame::new(4, 2, 0xffffff);
    let mut images = red_images();
    images.insert("alias".into(), Arc::clone(&images["red"]));
    images.insert("transparent".into(), raster([255, 0, 0, 0]));
    let commands = [
        image_rect("red", 0.0, 0.0, 1.0, 1.0),
        image_rect("alias", 1.0, 0.0, 1.0, 1.0),
        image_rect("transparent", 2.0, 0.0, 1.0, 1.0),
        image_rect("missing", 3.0, 0.0, 1.0, 1.0),
        image_rect("red", 0.0, 0.0, 1.0, 1.0),
    ];
    let scene = image_plan(target, &commands, &images).unwrap();
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 5);
    let stats = scene.stats().bridge;
    assert_eq!(stats.image_store_entries, 3);
    assert_eq!(stats.unique_sources, 2);
    assert_eq!(stats.referenced_sources, 2);
    assert_eq!(stats.total_rgba_bytes, 8);
    assert_eq!(stats.referenced_rgba_bytes, 8);
    assert_eq!(stats.missing_images, 1);
    let expected = [
        0xff0000, 0xff0000, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
    ];
    assert_eq!(decode_pixels(scene.plan()), expected);
    assert_eq!(
        canvas_pixels(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands
            }],
            &images,
            &Fonts::new()
        ),
        expected
    );
    assert!(Arc::ptr_eq(&images["red"], &images["alias"]));
}

#[test]
fn image_accounting_minimum_budget_exact_and_one_pixel_over() {
    let target = Frame::new(250, 250, 0xffffff);
    let images = red_images();
    let mut commands = vec![image_rect("red", 0.0, 0.0, 250.0, 250.0); 16];
    let exact = image_plan(target, &commands, &images).unwrap();
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 1_000_000);
    assert_eq!(decode_pixels(exact.plan()), vec![0xff0000; 62_500]);
    commands.push(image_rect("red", 0.0, 0.0, 1.0, 1.0));
    assert_eq!(
        image_plan(target, &commands, &images).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
    // Removing exactly one previous destination pixel admits the split remainder plus the final one-pixel command.
    // That final store remains present and metered.
    commands[15] = image_rect("red", 0.0, 0.0, 250.0, 249.0);
    commands.insert(16, image_rect("red", 0.0, 249.0, 249.0, 1.0));
    let exact = image_plan(target, &commands, &images).unwrap();
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 1_000_000);
}

#[test]
fn image_accounting_scaled_budget_is_shared_across_phases_even_for_missing_sources() {
    let target = Frame::new(1000, 1000, 0xffffff);
    let first = vec![image_rect("missing", 0.0, 0.0, 1000.0, 1000.0); 8];
    let mut second = first.clone();
    let images = ImageStore::new();
    let fonts = Fonts::new();
    let phases = [
        NativePhase {
            frame: target,
            commands: &first,
        },
        NativePhase {
            frame: target,
            commands: &second,
        },
    ];
    let exact = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 16_000_000);
    assert_eq!(exact.stats().bridge.missing_images, 16);
    second.push(image_rect("missing", 0.0, 0.0, 1.0, 1.0));
    let phases = [
        NativePhase {
            frame: target,
            commands: &first,
        },
        NativePhase {
            frame: target,
            commands: &second,
        },
    ];
    assert_eq!(
        plan_native_scene(target, &phases, &images, &fonts).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
}

#[test]
fn image_accounting_tiny_images_keep_original_command_limit() {
    let target = Frame::new(512, 512, 0xffffff);
    let images = red_images();
    let mut commands = (0..256)
        .map(|index| {
            image_rect(
                "red",
                (index % 16 * 2) as f32,
                (index / 16 * 2) as f32,
                1.0,
                1.0,
            )
        })
        .collect::<Vec<_>>();
    let scene = image_plan(target, &commands, &images).unwrap();
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 256);
    assert_eq!(scene.stats().bridge.original_commands, 256);
    assert_eq!(scene.stats().bridge.unique_sources, 1);
    let mut expected = vec![0xffffff; 512 * 512];
    for y in 0..16 {
        for x in 0..16 {
            expected[y * 2 * 512 + x * 2] = 0xff0000;
        }
    }
    assert_eq!(decode_pixels(scene.plan()), expected);
    assert_eq!(
        canvas_pixels(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands
            }],
            &images,
            &Fonts::new()
        ),
        expected
    );
    commands.push(image_rect("red", 600.0, 0.0, 1.0, 1.0));
    assert_eq!(
        image_plan(target, &commands, &images).unwrap_err().kind,
        FallbackKind::CommandLimit
    );
}

#[test]
fn image_accounting_hidden_images_do_not_bypass_input_and_source_validation() {
    let target = Frame::new(8, 8, 0xffffff);
    let images = red_images();
    let invalid = [
        image_clip(0.0, 0.0, 0.0, 0.0),
        image_rect("red", f32::NAN, 0.0, 1.0, 1.0),
        DrawCommand::PopClip,
    ];
    assert_eq!(
        image_plan(target, &invalid, &images).unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 1)
    );
    let mismatch = [
        DrawCommand::PushFixed,
        image_rect("red", 100.0, 0.0, 1.0, 1.0),
        DrawCommand::PopClip,
    ];
    assert_eq!(
        image_plan(target, &mismatch, &images).unwrap_err(),
        at(FallbackKind::InvalidScope, 0, 2)
    );
    let huge_key = "x".repeat(4097);
    let command = [image_rect(&huge_key, 100.0, 0.0, 1.0, 1.0)];
    assert_eq!(
        image_plan(target, &command, &images).unwrap_err().kind,
        FallbackKind::KeyBudget
    );
    let mut malformed = ImageStore::new();
    malformed.insert(
        "bad".into(),
        Arc::new(RasterImage {
            width: 2,
            height: 2,
            rgba: vec![255; 4],
        }),
    );
    let command = [image_rect("bad", 100.0, 0.0, 1.0, 1.0)];
    assert_eq!(
        image_plan(target, &command, &malformed).unwrap_err().kind,
        FallbackKind::InvalidImage
    );
    let mut bad_target = target;
    bad_target.width = 1281;
    assert_eq!(
        image_plan(bad_target, &[], &images).unwrap_err().kind,
        FallbackKind::ViewportLimit
    );
}

#[test]
fn image_accounting_preserves_other_command_fees_and_gpu_limit() {
    let target = Frame::new(4, 2, 0xffffff);
    let images = red_images();
    let commands = [
        DrawCommand::PushOpacity { opacity: 1.0 },
        image_rect("red", 0.0, 0.0, 1.0, 1.0),
        DrawCommand::PopOpacity,
        DrawCommand::Line {
            x1: 0.0,
            y1: 1.0,
            x2: 1.0,
            y2: 1.0,
            color: Color::BLACK,
            width: 1.0,
        },
        fill(2.0, 0.0, 2.0, 1.0, Color::TRANSPARENT, 0.0),
    ];
    let scene = image_plan(target, &commands, &images).unwrap();
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 27); // 2*8 opacity + 1 image + 8 line + 2 rect
    let target = Frame::new(1280, 1024, 0xffffff);
    let commands = vec![image_rect("red", 0.0, 0.0, 1280.0, 1024.0); 3];
    // CPU work is below its existing limit, but packed GPU invocations retain
    // their independent limit and still reject this scene.
    assert_eq!(
        image_plan(target, &commands, &images).unwrap_err().kind,
        FallbackKind::PlannerLimit
    );
}
