use super::*;
use eris_raster_core::rounded::{RoundedDisposition, RoundedShape};

fn same_plan(left: &Plan, right: &Plan) {
    assert_eq!(left.parameters(), right.parameters());
    assert_eq!(left.input_bytes(), right.input_bytes());
    assert_eq!(left.invocations(), right.invocations());
    assert_eq!(left.gpu_buffer_bytes(), right.gpu_buffer_bytes());
    assert_eq!(left.group_scratch_bytes(), right.group_scratch_bytes());
    let records = |plan: &Plan| {
        plan.draws()
            .iter()
            .map(|draw| {
                (
                    draw.kind(),
                    draw.bounds(),
                    draw.target_is_group(),
                    draw.opacity_bits(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(records(left), records(right));
}

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

#[test]
fn native_rect_accounting_uses_preblend_bounds_and_preserves_fractional_pixels() {
    let target = Frame::new(3, 3, 0xffffff);
    let cases = [
        (
            (1.0, 1.0, 2.0, 1.0),
            2,
            vec![
                0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000, 0xff0000, 0xffffff, 0xffffff,
                0xffffff,
            ],
        ),
        (
            (0.25, 0.25, 1.5, 1.5),
            4,
            vec![
                0xff0000, 0xff0000, 0xffffff, 0xff0000, 0xff0000, 0xffffff, 0xffffff, 0xffffff,
                0xffffff,
            ],
        ),
        (
            (-1.0, -1.0, 2.0, 2.0),
            1,
            vec![
                0xff0000, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
                0xffffff,
            ],
        ),
        ((0.0, 0.0, 0.0, 1.0), 0, vec![0xffffff; 9]),
        ((0.0, 0.0, -1.0, 1.0), 0, vec![0xffffff; 9]),
        ((4.0, 0.0, 1.0, 1.0), 0, vec![0xffffff; 9]),
    ];
    for ((x, y, width, height), work, expected) in cases {
        for radius in [0.0, -0.25] {
            let commands = [fill(x, y, width, height, Color::rgb(255, 0, 0), radius)];
            let prepared = single(&commands, target).unwrap();
            assert_eq!(prepared.stats().cpu_pixel_upper_bound, work);
            assert_eq!(prepared.stats().rounded_masks, 0);
            assert_eq!(decode_pixels(prepared.plan()), expected);
            assert_eq!(
                canvas_pixels(
                    target,
                    &[NativePhase {
                        frame: target,
                        commands: &commands
                    }],
                    &ImageStore::new(),
                    &Fonts::new()
                ),
                expected
            );
            // An independent core admission has no browser CPU reservation.
            // Its packed plan remains identical for these already supported Rects.
            let core = eris_raster_core::plan_with_masks_for_profile(
                Profile::Native,
                target,
                &[Command::Rect {
                    rect: Rect::new(x, y, width, height),
                    rgba: [255, 0, 0, 255],
                    radius: 0.0,
                }],
                &[],
                &[],
                &[],
            )
            .unwrap();
            same_plan(prepared.plan(), &core);
        }
    }
    let transparent = [fill(0.25, 0.25, 1.5, 1.5, Color::TRANSPARENT, 0.5)];
    let prepared = single(&transparent, target).unwrap();
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 4);
    assert_eq!(decode_pixels(prepared.plan()), vec![0xffffff; 9]);

    // Preserve the raw helper's literal loop2/coverage1 trap. Native normalizes
    // this caller clip to x[0,1), so its separate expected debit is only1.
    let target = Frame::new(2, 1, 0);
    let rect = Rect::new(f32::from_bits(0xbfffec57), 0.0, 4.0, 1.0);
    let raw_clip = Rect::new(-2.0, 0.0, 3.0, 1.0);
    let RoundedDisposition::Shape(shape) =
        RoundedShape::prepare(Profile::Native, target, rect, raw_clip, 0.25).unwrap()
    else {
        panic!("raw rounded shape")
    };
    assert_eq!(shape.info().loop_work, 2);
    assert_eq!(shape.info().coverage_bytes, 1);
    let mut native_frame = target;
    native_frame.caller_clip = raw_clip;
    let commands = [fill(
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        Color::WHITE,
        0.25,
    )];
    let prepared = plan_native_scene(
        target,
        &[NativePhase {
            frame: native_frame,
            commands: &commands,
        }],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 1);
    assert_eq!(prepared.stats().rounded_loop_work, 1);
    assert_eq!(prepared.stats().coverage_bytes, 1);
    assert_eq!(decode_pixels(prepared.plan()), [0xbfbfbf, 0]);
    assert_eq!(
        canvas_pixels(
            target,
            &[NativePhase {
                frame: native_frame,
                commands: &commands
            }],
            &ImageStore::new(),
            &Fonts::new()
        ),
        [0xbfbfbf, 0]
    );
}

#[test]
fn native_rect_accounting_tracks_scopes_phases_and_validates_hidden_tails() {
    let target = Frame::new(8, 4, 0xffffff);
    let mut first = target;
    first.document_offset = (-3.0, 0.0);
    first.viewport_offset = (1.0, 0.0);
    let page = [
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
    let overlay = [
        DrawCommand::PushFixed,
        fill(0.0, 0.0, 4.0, 1.0, Color::rgb(0, 0, 255), 0.0),
        DrawCommand::PopFixed,
    ];
    let phases = [
        NativePhase {
            frame: first,
            commands: &page,
        },
        NativePhase {
            frame: second,
            commands: &overlay,
        },
    ];
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let prepared = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 8); //4+0+2+2
    assert_eq!(prepared.stats().bridge.original_commands, 10);
    let mut expected = vec![0xffffff; 32];
    for index in [1, 2, 9, 10] {
        expected[index] = 0xff0000;
    }
    expected[4] = 0x0000ff;
    expected[5] = 0x0000ff;
    expected[16] = 0x00ff00;
    expected[17] = 0x00ff00;
    assert_eq!(decode_pixels(prepared.plan()), expected);
    assert_eq!(canvas_pixels(target, &phases, &images, &fonts), expected);
    let flat = [
        fill(1.0, 0.0, 2.0, 2.0, Color::rgb(255, 0, 0), 0.0),
        fill(0.0, 2.0, 2.0, 1.0, Color::rgb(0, 255, 0), 0.0),
        fill(4.0, 0.0, 2.0, 1.0, Color::rgb(0, 0, 255), 0.0),
    ];
    same_plan(prepared.plan(), single(&flat, target).unwrap().plan());
    for invalid in [
        fill(0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT, f32::NAN),
        fill(f32::INFINITY, 0.0, 0.0, 0.0, Color::TRANSPARENT, 0.0),
        image("hidden", f32::NAN),
    ] {
        let commands = [
            DrawCommand::PushOpacity { opacity: 0.0 },
            clip(0.0, 0.0, 0.0, 0.0),
            invalid,
            DrawCommand::PopClip,
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            single(&commands, target).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 2)
        );
    }
    let malformed = [clip(0.0, 0.0, 0.0, 0.0), hidden(), DrawCommand::PopFixed];
    assert_eq!(
        single(&malformed, target).unwrap_err(),
        at(FallbackKind::InvalidScope, 0, 2)
    );
    let zero = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        fill(0.0, 0.0, 2.0, 2.0, Color::WHITE, 0.0),
        DrawCommand::PopOpacity,
    ];
    let prepared = single(&zero, target).unwrap();
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 68); //Two32-pixel markers +4 Rect pixels.
    assert_eq!(prepared.plan().group_scratch_bytes(), 0);
    assert_eq!(decode_pixels(prepared.plan()), vec![0xffffff; 32]);
}

#[test]
fn native_rect_accounting_keeps_exact_cpu_limit_and_one_pixel_refusal() {
    let target = Frame::new(512, 512, 0xffffff);
    let full = fill(0.0, 0.0, 512.0, 512.0, Color::TRANSPARENT, 0.0);
    let mut commands = vec![full; 16];
    let prepared = single(&commands, target).unwrap();
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 4_194_304);
    assert_eq!(prepared.stats().bridge.original_commands, 16);
    assert_eq!(decode_pixels(prepared.plan()), vec![0xffffff; 512 * 512]);
    commands.push(fill(0.0, 0.0, 1.0, 1.0, Color::TRANSPARENT, 0.0));
    assert_eq!(
        single(&commands, target).unwrap_err().kind,
        FallbackKind::CpuPaintBudget
    );
    // Image, Line and opacity markers deliberately retain full-frame fees.
    let other = [
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
    for command in other {
        assert_eq!(
            single(&[command], target)
                .unwrap()
                .stats()
                .cpu_pixel_upper_bound,
            262_144
        );
    }
    for opacity in [0.0, 0.3, 1.0, f32::from_bits(1)] {
        let commands = [
            DrawCommand::PushOpacity { opacity },
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            single(&commands, target)
                .unwrap()
                .stats()
                .cpu_pixel_upper_bound,
            524_288
        );
    }
}

#[test]
fn native_rect_accounting_shares_exact_remaining_glyph_work_between_phases() {
    let target = Frame::new(250, 200, 0xffffff);
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let text_only = [text("A")];
    let measured = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &text_only,
            },
            NativePhase {
                frame: target,
                commands: &text_only,
            },
        ],
        &images,
        &fonts,
    )
    .unwrap();
    let glyph_work = measured.stats().text.paint_pixels_used;
    assert!(glyph_work > 0 && glyph_work < 50_000);
    assert_eq!(measured.stats().text.occurrences, 2);
    let remainder = 50_000 - glyph_work;
    let mut first = vec![fill(0.0, 0.0, 250.0, 200.0, Color::TRANSPARENT, 0.0); 19];
    first.push(fill(
        0.0,
        0.0,
        250.0,
        (remainder / 250) as f32,
        Color::TRANSPARENT,
        0.0,
    ));
    first.push(fill(
        0.0,
        0.0,
        (remainder % 250) as f32,
        1.0,
        Color::TRANSPARENT,
        0.0,
    ));
    first.push(text("A"));
    let phases = [
        NativePhase {
            frame: target,
            commands: &first,
        },
        NativePhase {
            frame: target,
            commands: &text_only,
        },
    ];
    let exact = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 1_000_000);
    assert_eq!(exact.stats().text.paint_pixels_used, glyph_work);
    assert_eq!(exact.stats().text.paint_pixels_remaining, 0);
    assert_eq!(exact.stats().text.cold_requests, 1);
    assert_eq!(decode_pixels(exact.plan()), decode_pixels(measured.plan()));
    assert!(decode_pixels(exact.plan()).iter().any(|&p| p != 0xffffff));
    assert_eq!(
        decode_pixels(exact.plan()),
        canvas_pixels(target, &phases, &images, &fonts)
    );
    first.insert(0, fill(0.0, 0.0, 1.0, 1.0, Color::TRANSPARENT, 0.0));
    let phases = [
        NativePhase {
            frame: target,
            commands: &first,
        },
        NativePhase {
            frame: target,
            commands: &text_only,
        },
    ];
    assert_eq!(
        plan_native_scene(target, &phases, &images, &fonts).unwrap_err(),
        at(FallbackKind::CpuPaintBudget, 1, 0)
    );
}

#[test]
fn native_rect_accounting_preserves_rounded_tiles_masks_and_single_debit() {
    let target = Frame::new(1079, 1, 0xffffff);
    let commands = [fill(0.0, 0.0, 1079.0, 1.0, Color::rgba(0, 0, 0, 128), 0.25)];
    let prepared = single(&commands, target).unwrap();
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 1079);
    assert_eq!(prepared.stats().rounded_loop_work, 1079);
    assert_eq!(prepared.stats().rounded_masks, 2);
    assert_eq!(prepared.stats().mask_sources, 2);
    assert_eq!(prepared.stats().coverage_bytes, 1079);
    assert_eq!(prepared.stats().row_entries, 2);
    assert_eq!(prepared.stats().bridge.lowered_commands, 2);
    assert_eq!(prepared.plan().draws()[1].bounds(), (0, 0, 1024, 1));
    assert_eq!(prepared.plan().draws()[2].bounds(), (1024, 0, 55, 1));
    assert_eq!(prepared.plan().input_bytes().len(), 4 * (1079 + 2));
    assert_eq!(decode_pixels(prepared.plan()), vec![0xa0a0a0; 1079]);
    assert_eq!(
        canvas_pixels(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands
            }],
            &ImageStore::new(),
            &Fonts::new()
        ),
        vec![0xa0a0a0; 1079]
    );
    let target = Frame::new(1280, 1024, 0xffffff);
    let exact = [fill(0.0, 0.0, 512.0, 512.0, Color::BLACK, 1.0)];
    let accepted = single(&exact, target).unwrap();
    assert_eq!(accepted.stats().cpu_pixel_upper_bound, 262_144);
    assert_eq!(accepted.stats().coverage_bytes, 262_144);
    assert_eq!(accepted.stats().rounded_loop_work, 262_144);
    let too_large = [fill(0.0, 0.0, 513.0, 512.0, Color::BLACK, 1.0)];
    assert_eq!(
        single(&too_large, target).unwrap_err(),
        at(FallbackKind::MaskPreparation, 0, 0)
    );
}

#[test]
fn native_rect_accounting_loaded_forty_boxes_have_literal_pixels_and_small_fees() {
    let page = crate::page::Page::from_html(
        url::Url::parse("https://rect-accounting.example/boxes").unwrap(),
        include_str!("../../../../examples/vulkan-small-boxes.html"),
        false,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let fonts = Fonts::new();
    let layout = page.layout(512.0, 512.0, &fonts);
    assert_eq!(layout.commands.len(), 40);
    assert!(
        layout
            .commands
            .iter()
            .all(|c| matches!(c, DrawCommand::Rect { radius, .. } if *radius == 0.0))
    );
    let target = Frame::new(512, 512, 0xffffff);
    let phases = [NativePhase {
        frame: target,
        commands: &layout.commands,
    }];
    let prepared = plan_native_scene(target, &phases, &page.images, &fonts).unwrap();
    assert_eq!(prepared.stats().bridge.original_commands, 40);
    assert_eq!(prepared.stats().bridge.lowered_commands, 40);
    assert_eq!(prepared.stats().cpu_pixel_upper_bound, 9600);
    assert_eq!(prepared.plan().draws().len(), 41);
    assert_eq!(prepared.plan().invocations(), 539_648);
    assert_eq!(prepared.plan().gpu_buffer_bytes(), 2_107_664);
    let colors = [
        0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0x00ffff, 0xff00ff, 0x804000, 0x408080,
    ];
    let mut expected = vec![0xffffff; 512 * 512];
    for row in 0..5 {
        for (col, color) in colors.iter().enumerate() {
            for y in 16 + row * 20..28 + row * 20 {
                expected[y * 512 + 16 + col * 28..y * 512 + 36 + col * 28].fill(*color);
            }
        }
    }
    assert_eq!(decode_pixels(prepared.plan()), expected);
    assert_eq!(
        canvas_pixels(target, &phases, &page.images, &fonts),
        expected
    );
}
