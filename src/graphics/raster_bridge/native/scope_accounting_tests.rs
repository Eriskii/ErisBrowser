use super::*;

fn clip(x: f32, y: f32, width: f32, height: f32) -> DrawCommand {
    DrawCommand::PushClip {
        rect: BrowserRect {
            x,
            y,
            width,
            height,
        },
    }
}

fn free_scopes() -> [DrawCommand; 4] {
    [
        clip(0.0, 0.0, 2000.0, 2000.0),
        DrawCommand::PushFixed,
        DrawCommand::PopFixed,
        DrawCommand::PopClip,
    ]
}

fn same_plan(left: &Plan, right: &Plan) {
    assert_eq!(left.parameters(), right.parameters());
    assert_eq!(left.input_bytes(), right.input_bytes());
    assert_eq!(left.invocations(), right.invocations());
    assert_eq!(left.gpu_buffer_bytes(), right.gpu_buffer_bytes());
    assert_eq!(left.group_scratch_bytes(), right.group_scratch_bytes());
}

#[test]
fn native_scope_accounting_preserves_fixed_escape_restore_and_phase_pixels() {
    let target = Frame::new(8, 4, 0xffffff);
    let mut first = target;
    first.document_offset = (-3.0, 0.0);
    first.viewport_offset = (1.0, 0.0);
    let a = [
        clip(0.0, 0.0, 0.0, 0.0),
        DrawCommand::PushFixed,
        fill(0.0, 0.0, 2.0, 2.0, Color::rgb(255, 0, 0), 0.0),
        DrawCommand::PopFixed,
        fill(0.0, 0.0, 8.0, 4.0, Color::rgb(0, 0, 255), 0.0),
        DrawCommand::PopClip,
        fill(3.0, 2.0, 2.0, 1.0, Color::rgb(0, 255, 0), 0.0),
    ];
    let mut second = target;
    second.caller_clip = Rect::new(4.0, 0.0, 2.0, 4.0);
    second.viewport_offset = (4.0, 0.0);
    let b = [
        DrawCommand::PushFixed,
        fill(0.0, 0.0, 4.0, 1.0, Color::rgb(0, 0, 255), 0.0),
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
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let scene = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    let expected = [
        0xffffff, 0xff0000, 0xff0000, 0xffffff, 0x0000ff, 0x0000ff, 0xffffff, 0xffffff, 0xffffff,
        0xff0000, 0xff0000, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x00ff00, 0x00ff00,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
    ];
    assert_eq!(scene.stats().bridge.original_commands, 10);
    // The clipped rectangle is omitted during existing lowering, not erased from input admission.
    assert_eq!(scene.stats().bridge.lowered_commands, 9);
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 8); // 4 + 0 + 2 + 2 loop pixels
    assert_eq!(decode_pixels(scene.plan()), expected);
    assert_eq!(canvas_pixels(target, &phases, &images, &fonts), expected);
    let flat = [
        fill(1.0, 0.0, 2.0, 2.0, Color::rgb(255, 0, 0), 0.0),
        fill(0.0, 2.0, 2.0, 1.0, Color::rgb(0, 255, 0), 0.0),
        fill(4.0, 0.0, 2.0, 1.0, Color::rgb(0, 0, 255), 0.0),
    ];
    same_plan(scene.plan(), single(&flat, target).unwrap().plan());
}

#[test]
fn native_scope_accounting_keeps_original_depth_and_validation_boundaries() {
    let target = Frame::new(1000, 1000, 0xffffff);
    let mut commands = Vec::new();
    for _ in 0..64 {
        commands.extend(free_scopes());
    }
    let exact = single(&commands, target).unwrap();
    assert_eq!(exact.stats().bridge.original_commands, 256);
    assert_eq!(exact.stats().bridge.lowered_commands, 256);
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 0);
    commands.push(hidden());
    assert_eq!(
        single(&commands, target).unwrap_err().kind,
        FallbackKind::CommandLimit
    );

    let mut nested = Vec::new();
    for _ in 0..16 {
        nested.extend([clip(0.0, 0.0, 1000.0, 1000.0), DrawCommand::PushFixed]);
    }
    for _ in 0..16 {
        nested.extend([DrawCommand::PopFixed, DrawCommand::PopClip]);
    }
    let exact = single(&nested, target).unwrap();
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 0);
    assert_eq!(exact.stats().bridge.lowered_commands, 64);
    nested.insert(32, DrawCommand::PushFixed);
    nested.insert(33, DrawCommand::PopFixed);
    assert_eq!(
        single(&nested, target).unwrap_err(),
        at(FallbackKind::ScopeLimit, 0, 32)
    );
    for malformed in [
        vec![DrawCommand::PushFixed, DrawCommand::PopClip],
        vec![clip(0.0, 0.0, 1.0, 1.0), DrawCommand::PopFixed],
        vec![DrawCommand::PopClip],
        vec![DrawCommand::PushFixed],
    ] {
        assert_eq!(
            single(&malformed, target).unwrap_err().kind,
            FallbackKind::InvalidScope
        );
    }
    let invalid = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        clip(f32::NAN, 0.0, 0.0, 0.0),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
    ];
    assert_eq!(
        single(&invalid, target).unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 1)
    );
}

#[test]
fn native_scope_accounting_keeps_full_rect_image_line_charges_across_phases() {
    let target = Frame::new(250, 200, 0);
    let primitives = [
        fill(0.0, 0.0, 250.0, 200.0, Color::TRANSPARENT, 0.0),
        image("missing", 0.0),
        DrawCommand::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            color: Color::TRANSPARENT,
            width: 0.0,
        },
    ];
    for primitive in primitives {
        let mut first = Vec::new();
        for _ in 0..10 {
            first.extend(free_scopes());
            first.push(primitive.clone());
        }
        let mut second = first.clone();
        let fonts = Fonts::new();
        let images = ImageStore::new();
        let exact = plan_native_scene(
            target,
            &[
                NativePhase {
                    frame: target,
                    commands: &first,
                },
                NativePhase {
                    frame: target,
                    commands: &second,
                },
            ],
            &images,
            &fonts,
        )
        .unwrap();
        assert_eq!(exact.stats().bridge.original_commands, 100);
        assert_eq!(exact.stats().cpu_pixel_upper_bound, 1_000_000);
        second.push(primitive);
        assert_eq!(
            plan_native_scene(
                target,
                &[
                    NativePhase {
                        frame: target,
                        commands: &first
                    },
                    NativePhase {
                        frame: target,
                        commands: &second
                    },
                ],
                &images,
                &fonts
            )
            .unwrap_err()
            .kind,
            FallbackKind::CpuPaintBudget
        );
    }
}

#[test]
fn native_scope_accounting_retains_opacity_area_even_for_empty_zero_and_unit() {
    let target = Frame::new(1000, 1000, 0);
    for opacity in [0.0, 0.5, 1.0] {
        let mut first = Vec::new();
        for _ in 0..4 {
            first.extend(free_scopes());
            first.extend([
                DrawCommand::PushOpacity { opacity },
                DrawCommand::PopOpacity,
            ]);
        }
        let mut second = first.clone();
        let fonts = Fonts::new();
        let images = ImageStore::new();
        let exact = plan_native_scene(
            target,
            &[
                NativePhase {
                    frame: target,
                    commands: &first,
                },
                NativePhase {
                    frame: target,
                    commands: &second,
                },
            ],
            &images,
            &fonts,
        )
        .unwrap();
        assert_eq!(exact.stats().bridge.original_commands, 48);
        assert_eq!(exact.stats().bridge.lowered_commands, 48);
        assert_eq!(exact.stats().cpu_pixel_upper_bound, 16_000_000);
        assert_eq!(exact.plan().group_scratch_bytes(), 0);
        second.extend([
            DrawCommand::PushOpacity { opacity },
            DrawCommand::PopOpacity,
        ]);
        assert_eq!(
            plan_native_scene(
                target,
                &[
                    NativePhase {
                        frame: target,
                        commands: &first
                    },
                    NativePhase {
                        frame: target,
                        commands: &second
                    },
                ],
                &images,
                &fonts
            )
            .unwrap_err()
            .kind,
            FallbackKind::CpuPaintBudget
        );
    }
}

#[test]
fn native_scope_accounting_preserves_shared_glyph_allowance_and_late_reserve() {
    let target = Frame::new(250, 200, 0xffffff);
    let full = fill(0.0, 0.0, 250.0, 200.0, Color::TRANSPARENT, 0.0);
    let first = vec![full.clone(); 19];
    let plain = [text("AVAV")];
    let mut scoped = Vec::new();
    for _ in 0..9 {
        scoped.extend(free_scopes());
    }
    scoped.push(plain[0].clone());
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let baseline = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &first,
            },
            NativePhase {
                frame: target,
                commands: &plain,
            },
        ],
        &images,
        &fonts,
    )
    .unwrap();
    let phases = [
        NativePhase {
            frame: target,
            commands: &first,
        },
        NativePhase {
            frame: target,
            commands: &scoped,
        },
    ];
    let result = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(result.stats().text.occurrences, 4);
    assert_eq!(result.stats().text.cold_requests, 2);
    assert!(result.stats().text.paint_pixels_used > 0);
    assert_eq!(result.stats().text, baseline.stats().text);
    assert_eq!(
        result.stats().text.paint_pixels_used + result.stats().text.paint_pixels_remaining,
        50_000
    );
    assert_eq!(
        result.stats().cpu_pixel_upper_bound,
        950_000 + result.stats().text.paint_pixels_used
    );
    same_plan(result.plan(), baseline.plan());
    let pixels = decode_pixels(result.plan());
    assert!(pixels.iter().any(|&pixel| pixel != 0xffffff));
    assert_eq!(pixels, canvas_pixels(target, &phases, &images, &fonts));
    let exhausted = vec![full; 20];
    for (phases, phase_index) in [
        (
            [
                NativePhase {
                    frame: target,
                    commands: &exhausted,
                },
                NativePhase {
                    frame: target,
                    commands: &scoped,
                },
            ],
            1,
        ),
        (
            [
                NativePhase {
                    frame: target,
                    commands: &scoped,
                },
                NativePhase {
                    frame: target,
                    commands: &exhausted,
                },
            ],
            0,
        ),
    ] {
        assert_eq!(
            plan_native_scene(target, &phases, &images, &fonts).unwrap_err(),
            at(FallbackKind::CpuPaintBudget, phase_index, 36)
        );
    }
}

#[test]
fn native_scope_accounting_loaded_page_keeps_clipped_red_and_escaping_fixed_blue() {
    const HTML: &str = "<!doctype html><style>body{margin:0}.clip{overflow:hidden;width:32px;height:32px}#outer{position:absolute;left:16px;top:16px}#red{width:64px;height:64px;background:red}#fixed{position:fixed;left:64px;top:16px;width:16px;height:16px;background:blue}</style><div id=outer class=clip><div class=clip><div class=clip><div class=clip><div class=clip><div class=clip><div class=clip><div class=clip><div id=red></div><div id=fixed></div></div></div></div></div></div></div></div></div>";
    let page = crate::page::Page::from_html(
        url::Url::parse("https://scope-accounting.example/scene").unwrap(),
        HTML,
        false,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let fonts = Fonts::new();
    let layout = page.layout(512.0, 512.0, &fonts);
    assert_eq!(layout.commands.len(), 20);
    let count =
        |matches: fn(&DrawCommand) -> bool| layout.commands.iter().filter(|c| matches(c)).count();
    assert_eq!(count(|c| matches!(c, DrawCommand::PushClip { .. })), 8);
    assert_eq!(count(|c| matches!(c, DrawCommand::PopClip)), 8);
    assert_eq!(count(|c| matches!(c, DrawCommand::PushFixed)), 1);
    assert_eq!(count(|c| matches!(c, DrawCommand::PopFixed)), 1);
    assert_eq!(count(|c| matches!(c, DrawCommand::Rect { .. })), 2);
    // Historical all-command reservation; scope and Rect refinements preserve these pixels.
    let old_bound = 262_144 * layout.commands.len() as u64;
    assert_eq!(old_bound, 5_242_880);
    assert!(old_bound > 4_194_304);
    let target = Frame::new(512, 512, 0xffffff);
    let phases = [NativePhase {
        frame: target,
        commands: &layout.commands,
    }];
    let scene = plan_native_scene(target, &phases, &page.images, &fonts).unwrap();
    assert_eq!(scene.stats().bridge.original_commands, 20);
    assert_eq!(scene.stats().bridge.lowered_commands, 20);
    assert_eq!(scene.stats().cpu_pixel_upper_bound, 1_280); // 32*32 + 16*16
    assert_eq!(scene.plan().draws().len(), 3);
    assert_eq!(scene.plan().draws()[1].bounds(), (16, 16, 32, 32));
    assert_eq!(scene.plan().draws()[2].bounds(), (64, 16, 16, 16));
    let mut expected = vec![0xffffff; 512 * 512];
    for y in 16..48 {
        expected[y * 512 + 16..y * 512 + 48].fill(0xff0000);
    }
    for y in 16..32 {
        expected[y * 512 + 64..y * 512 + 80].fill(0x0000ff);
    }
    assert_eq!(expected[20 * 512 + 20], 0xff0000);
    assert_eq!(expected[20 * 512 + 50], 0xffffff);
    assert_eq!(expected[20 * 512 + 68], 0x0000ff);
    assert_eq!(expected[20 * 512 + 80], 0xffffff);
    assert_eq!(decode_pixels(scene.plan()), expected);
    assert_eq!(
        canvas_pixels(target, &phases, &page.images, &fonts),
        expected
    );
}
