use super::*;
use crate::graphics::{Canvas, Color, Rect as BrowserRect};
use eris_raster_core::DrawKind;

fn fill(x: f32, y: f32, width: f32, height: f32, color: Color, radius: f32) -> DrawCommand {
    DrawCommand::Rect {
        rect: BrowserRect {
            x,
            y,
            width,
            height,
        },
        color,
        radius,
    }
}
fn hidden() -> DrawCommand {
    fill(0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT, 0.0)
}
fn text(value: &str) -> DrawCommand {
    DrawCommand::Text {
        x: 1.0,
        y: 0.0,
        text: value.into(),
        size: 14.0,
        color: Color::BLACK,
        bold: false,
        italic: false,
        monospace: false,
    }
}
fn image(key: &str, x: f32) -> DrawCommand {
    DrawCommand::Image {
        rect: BrowserRect {
            x,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        key: key.into(),
    }
}
fn raster(color: [u8; 4]) -> Arc<RasterImage> {
    Arc::new(RasterImage {
        width: 1,
        height: 1,
        rgba: color.to_vec(),
    })
}
fn frame() -> Frame {
    Frame::new(64, 32, 0xffffff)
}
fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
fn parameter(plan: &Plan, draw: usize, index: usize) -> u32 {
    word(&plan.parameters()[draw * PARAM_STRIDE..], index)
}
fn single(commands: &[DrawCommand], target: Frame) -> NativeResult<NativeScenePlan> {
    plan_native_scene(
        target,
        &[NativePhase {
            frame: target,
            commands,
        }],
        &ImageStore::new(),
        &Fonts::new(),
    )
}

// A test-only decoder of the public packed plan format. It exercises adapter
// source IDs, absolute placements and draw order against the original Canvas;
// it is not shader execution or evidence about a graphics driver.
fn decode_pixels(plan: &Plan) -> Vec<u32> {
    let frame = plan.frame();
    let mut pixels = vec![0; frame.width as usize * frame.height as usize];
    let input = plan.input_bytes();
    for (draw_index, draw) in plan.draws().iter().enumerate() {
        let p = |field| parameter(plan, draw_index, field);
        let (x0, y0, width, height) = draw.bounds();
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let (source, alpha) = match draw.kind() {
                    DrawKind::Rectangle => (p(6), p(6) >> 24),
                    DrawKind::Image => {
                        let sx = word(input, (p(12) + x - x0) as usize);
                        let sy = word(input, (p(13) + y - y0) as usize);
                        let source = word(input, (p(8) + sy * p(9) + sx) as usize);
                        (source, source >> 24)
                    }
                    DrawKind::Glyph => {
                        let sy = i64::from(y) - i64::from(p(13) as i32);
                        if sy < 0 || sy >= i64::from(p(10)) {
                            continue;
                        }
                        let row_x = word(input, p(12) as usize + sy as usize) as i32;
                        let sx = i64::from(x) - i64::from(row_x);
                        if sx < 0 || sx >= i64::from(p(9)) {
                            continue;
                        }
                        let coverage = word(
                            input,
                            p(8) as usize + sy as usize * p(9) as usize + sx as usize,
                        );
                        assert!(coverage <= 255);
                        (p(6), (p(6) >> 24) * coverage / 255)
                    }
                };
                let pixel = &mut pixels[(y * frame.width + x) as usize];
                let mut blended = 0;
                for shift in [0, 8, 16] {
                    let channel = (((source >> shift) & 255) * alpha
                        + ((*pixel >> shift) & 255) * (255 - alpha)
                        + 127)
                        / 255;
                    blended |= channel << shift;
                }
                *pixel = blended;
            }
        }
    }
    pixels
}
fn canvas_pixels(
    target: Frame,
    phases: &[NativePhase<'_>],
    images: &ImageStore,
    fonts: &Fonts,
) -> Vec<u32> {
    let mut canvas = Canvas::new(target.width, target.height).unwrap();
    canvas.clear(Color::rgb(
        (target.clear >> 16) as u8,
        (target.clear >> 8) as u8,
        target.clear as u8,
    ));
    for phase in phases {
        let clip = phase.frame.caller_clip;
        canvas.set_clip(BrowserRect {
            x: clip.x,
            y: clip.y,
            width: clip.width,
            height: clip.height,
        });
        canvas.paint_with_viewport(
            phase.commands,
            fonts,
            images,
            phase.frame.document_offset,
            phase.frame.viewport_offset,
        );
        assert!(!canvas.exhausted());
    }
    canvas.pixels
}

#[test]
fn native_empty_scene_is_one_clear_and_one_conversion() {
    let target = Frame::new(400, 250, 0x123456);
    let result = plan_native_scene(target, &[], &ImageStore::new(), &Fonts::new()).unwrap();
    assert_eq!(result.stats().phases, 0);
    assert_eq!(result.plan().draws().len(), 1);
    assert_eq!(result.plan().profile(), Profile::Native);
    assert_eq!(result.plan().conversion_invocations(), 400 * 256);
    assert_eq!(
        result.plan().raster_invocations(),
        result.plan().conversion_invocations()
    );
    assert!(
        decode_pixels(result.plan())
            .iter()
            .all(|&pixel| pixel == 0x123456)
    );
}

#[test]
fn native_phase_frames_and_canonical_target_validate_before_hidden_commands() {
    let target = frame();
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let command = [hidden()];
    let mut bad = target;
    bad.document_offset = (1.0, 0.0);
    assert_eq!(
        plan_native_scene(bad, &[], &images, &fonts)
            .unwrap_err()
            .kind,
        FallbackKind::InvalidGeometry
    );
    for modified in [Frame::new(63, 32, target.clear), Frame::new(64, 32, 0)] {
        let error = plan_native_scene(
            target,
            &[NativePhase {
                frame: modified,
                commands: &command,
            }],
            &images,
            &fonts,
        )
        .unwrap_err();
        assert_eq!(
            (error.kind, error.phase_index, error.command_index),
            (FallbackKind::InvalidGeometry, Some(0), None)
        );
    }
    bad = target;
    bad.caller_clip.width = f32::NAN;
    assert_eq!(
        plan_native_scene(
            target,
            &[NativePhase {
                frame: bad,
                commands: &command
            }],
            &images,
            &fonts
        )
        .unwrap_err()
        .kind,
        FallbackKind::InvalidGeometry
    );
    let phases = [NativePhase {
        frame: target,
        commands: &[],
    }; MAX_NATIVE_PHASES];
    assert!(plan_native_scene(target, &phases, &images, &fonts).is_ok());
    let phases = [NativePhase {
        frame: target,
        commands: &[],
    }; MAX_NATIVE_PHASES + 1];
    assert_eq!(
        plan_native_scene(target, &phases, &images, &fonts)
            .unwrap_err()
            .kind,
        FallbackKind::CommandLimit
    );
}

#[test]
fn native_original_commands_and_cpu_allowance_are_global() {
    let small = Frame::new(1, 1, 0);
    let a = vec![hidden(); 128];
    let mut b = vec![hidden(); 128];
    assert_eq!(
        plan_native_scene(
            small,
            &[
                NativePhase {
                    frame: small,
                    commands: &a
                },
                NativePhase {
                    frame: small,
                    commands: &b
                }
            ],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap()
        .stats()
        .bridge
        .original_commands,
        256
    );
    b.push(hidden());
    assert_eq!(
        plan_native_scene(
            small,
            &[
                NativePhase {
                    frame: small,
                    commands: &a
                },
                NativePhase {
                    frame: small,
                    commands: &b
                }
            ],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err()
        .phase_index,
        Some(1)
    );
    let target = Frame::new(250, 200, 0);
    let a = vec![hidden(); 10];
    let mut b = vec![hidden(); 10];
    let accepted = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &a,
            },
            NativePhase {
                frame: target,
                commands: &b,
            },
        ],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(accepted.stats().cpu_pixel_upper_bound, 1_000_000);
    b.push(hidden());
    assert_eq!(
        plan_native_scene(
            target,
            &[
                NativePhase {
                    frame: target,
                    commands: &a
                },
                NativePhase {
                    frame: target,
                    commands: &b
                }
            ],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err()
        .kind,
        FallbackKind::CpuPaintBudget
    );
}

#[test]
fn native_text_input_and_expansion_caps_do_not_reset_at_phase_boundaries() {
    let target = frame();
    let mut first = text(&"x".repeat(2048));
    let mut second = first.clone();
    if let DrawCommand::Text { color, .. } = &mut first {
        *color = Color::TRANSPARENT;
    }
    if let DrawCommand::Text { color, .. } = &mut second {
        *color = Color::TRANSPARENT;
    }
    let a = [first];
    let mut b = [second];
    let accepted = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &a,
            },
            NativePhase {
                frame: target,
                commands: &b,
            },
        ],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(accepted.stats().text_input_scalars, 4096);
    assert_eq!(accepted.stats().text.cold_requests, 0);
    if let DrawCommand::Text { text, .. } = &mut b[0] {
        text.push('x');
    }
    let refused = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &a,
            },
            NativePhase {
                frame: target,
                commands: &b,
            },
        ],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap_err();
    assert_eq!(refused, at(FallbackKind::TextLimit, 1, 0));
    let a = [text(&"\u{200d}".repeat(128))];
    let mut b = [text(&"\u{200d}".repeat(128))];
    let exact = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &a,
            },
            NativePhase {
                frame: target,
                commands: &b,
            },
        ],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(exact.stats().text.occurrences, 256);
    assert_eq!(exact.stats().text.cold_requests, 1);
    assert_eq!(exact.stats().bridge.lowered_commands, 256);
    assert_eq!(exact.stats().mask_sources, 1);
    if let DrawCommand::Text { text, .. } = &mut b[0] {
        text.push('\u{200d}');
    }
    assert_eq!(
        plan_native_scene(
            target,
            &[
                NativePhase {
                    frame: target,
                    commands: &a
                },
                NativePhase {
                    frame: target,
                    commands: &b
                }
            ],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::CommandLimit, 1, 0)
    );
}

#[test]
fn native_late_structural_failure_has_original_location_and_no_font_effect() {
    let fonts = Fonts::new();
    let target = frame();
    let a = [text("AB")];
    let b = [
        image("missing", 0.0),
        DrawCommand::PushOpacity { opacity: 0.0 },
    ];
    let error = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &a,
            },
            NativePhase {
                frame: target,
                commands: &b,
            },
        ],
        &ImageStore::new(),
        &fonts,
    )
    .unwrap_err();
    assert_eq!(error, at(FallbackKind::UnsupportedOpacity, 1, 1));
    assert_eq!(fonts.cache.borrow().len(), 0);
    let open = [DrawCommand::PushFixed];
    let close = [DrawCommand::PopFixed];
    let error = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: target,
                commands: &open,
            },
            NativePhase {
                frame: target,
                commands: &close,
            },
        ],
        &ImageStore::new(),
        &fonts,
    )
    .unwrap_err();
    assert_eq!(
        error,
        NativeFallback::new(FallbackKind::InvalidScope, Some(0), None)
    );
}

#[test]
fn native_invalid_hidden_geometry_is_not_omitted() {
    let target = frame();
    for bad in [f32::NAN, f32::INFINITY, MAX_COORDINATE + 1.0] {
        let input = [fill(0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT, bad)];
        assert_eq!(
            single(&input, target).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
        let input = [fill(bad, 0.0, 0.0, 0.0, Color::TRANSPARENT, 0.0)];
        assert_eq!(
            single(&input, target).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    let input = [DrawCommand::Line {
        x1: -MAX_COORDINATE,
        x2: MAX_COORDINATE,
        y1: 0.0,
        y2: 0.0,
        width: 0.0,
        color: Color::TRANSPARENT,
    }];
    assert_eq!(
        single(&input, target).unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 0)
    );
}

#[test]
fn native_rounded_literals_and_typed_zero_path_preserve_pixels() {
    let target = Frame::new(2, 2, 0xffffff);
    let corner = [fill(0.0, 0.0, 2.0, 2.0, Color::BLACK, 1.0)];
    let result = single(&corner, target).unwrap();
    assert_eq!(decode_pixels(result.plan()), vec![0x353535; 4]);
    assert_eq!(result.stats().rounded_loop_work, 4);
    assert_eq!(result.stats().rounded_masks, 1);
    assert_eq!(result.stats().text.occurrences, 0);
    let quarter = [fill(0.25, 0.0, 1.0, 1.0, Color::BLACK, 0.25)];
    assert_eq!(
        decode_pixels(single(&quarter, Frame::new(2, 1, 0xffffff)).unwrap().plan()),
        [0x404040, 0xc0c0c0]
    );
    for radius in [-1.0, -0.0, 0.0] {
        let flat = [fill(0.0, 0.0, 2.0, 2.0, Color::BLACK, radius)];
        let result = single(&flat, target).unwrap();
        assert_eq!(result.stats().rounded_masks, 0);
        assert_eq!(result.plan().draws()[1].kind(), DrawKind::Rectangle);
        assert_eq!(decode_pixels(result.plan()), vec![0; 4]);
    }
}

#[test]
fn native_font_rounded_interleaving_reuses_font_id_across_phases() {
    let target = frame();
    let a = [text("A"), fill(20.0, 1.0, 2.0, 2.0, Color::BLACK, 1.0)];
    let b = [
        text("A"),
        fill(24.0, 1.0, 2.0, 2.0, Color::BLACK, 1.0),
        text("B"),
    ];
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let phases = [
        NativePhase {
            frame: target,
            commands: &a,
        },
        NativePhase {
            frame: target,
            commands: &b,
        },
    ];
    let result = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(result.stats().text.cold_requests, 2);
    assert_eq!(result.stats().text.occurrences, 3);
    assert_eq!(result.stats().rounded_masks, 2);
    assert_eq!(result.stats().mask_sources, 4);
    assert_eq!(
        parameter(result.plan(), 1, 8),
        parameter(result.plan(), 3, 8)
    );
    assert!(parameter(result.plan(), 2, 8) > parameter(result.plan(), 1, 8));
    assert!(parameter(result.plan(), 4, 8) > parameter(result.plan(), 2, 8));
    assert!(parameter(result.plan(), 5, 8) > parameter(result.plan(), 4, 8));
    assert_eq!(
        decode_pixels(result.plan()),
        canvas_pixels(target, &phases, &images, &fonts)
    );
}

#[test]
fn native_page_fixed_clip_and_chrome_order_match_original_canvas() {
    let target = Frame::new(64, 40, 0xffffff);
    let mut page_frame = target;
    page_frame.caller_clip = Rect::new(0.0, 8.0, 64.0, 24.0);
    page_frame.document_offset = (1.0, -1.0);
    page_frame.viewport_offset = (0.0, 8.0);
    let page = [
        DrawCommand::PushClip {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 2.0,
            },
        },
        DrawCommand::PushFixed,
        text("A"),
        fill(60.0, -10.0, 4.0, 40.0, Color::rgb(255, 0, 0), 0.0),
        DrawCommand::PopFixed,
        DrawCommand::PopClip,
        DrawCommand::Image {
            rect: BrowserRect {
                x: 3.0,
                y: 10.0,
                width: 3.0,
                height: 3.0,
            },
            key: "one".into(),
        },
    ];
    let overlay = [fill(
        19.5,
        10.0,
        5.0,
        7.0,
        Color::rgba(30, 40, 50, 180),
        3.0,
    )];
    let mut overlay_frame = target;
    overlay_frame.caller_clip = page_frame.caller_clip;
    let mut title = text("To");
    if let DrawCommand::Text { italic, color, .. } = &mut title {
        *italic = true;
        *color = Color::rgb(3, 4, 5);
    }
    let chrome = [
        fill(0.0, 0.0, 64.0, 8.0, Color::rgb(20, 30, 40), 0.0),
        title,
    ];
    let phases = [
        NativePhase {
            frame: page_frame,
            commands: &page,
        },
        NativePhase {
            frame: overlay_frame,
            commands: &overlay,
        },
        NativePhase {
            frame: target,
            commands: &chrome,
        },
    ];
    let mut images = ImageStore::new();
    images.insert("one".into(), raster([100, 110, 120, 127]));
    let fonts = Fonts::new();
    let result = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    let decoded = decode_pixels(result.plan());
    assert_eq!(decoded, canvas_pixels(target, &phases, &images, &fonts));
    assert_eq!(decoded[9 * 64 + 63], 0xff0000);
    assert_eq!(decoded[33 * 64 + 63], 0xffffff);
    assert_eq!(decoded[63], 0x141e28);
}

#[test]
fn native_image_identity_order_is_shared_and_insertion_independent() {
    let target = Frame::new(4, 2, 0xffffff);
    let shared = raster([10, 20, 30, 128]);
    let other = raster([1, 2, 3, 255]);
    let commands = [image("z", 0.0), image("alias", 1.0), image("missing", 2.0)];
    let next = [image("a", 3.0)];
    let phases = [
        NativePhase {
            frame: target,
            commands: &commands,
        },
        NativePhase {
            frame: target,
            commands: &next,
        },
    ];
    let make = |reverse| {
        let mut map = ImageStore::new();
        let entries = [
            ("z", shared.clone()),
            ("alias", shared.clone()),
            ("a", other.clone()),
        ];
        for index in if reverse { [2, 1, 0] } else { [0, 1, 2] } {
            map.insert(entries[index].0.into(), entries[index].1.clone());
        }
        map
    };
    let first = make(false);
    let second = make(true);
    let before = Arc::strong_count(&shared);
    let result = plan_native_scene(target, &phases, &first, &Fonts::new()).unwrap();
    let repeated = plan_native_scene(target, &phases, &second, &Fonts::new()).unwrap();
    assert_eq!(Arc::strong_count(&shared), before);
    assert_eq!(result.stats().bridge.unique_sources, 2);
    assert_eq!(result.stats().bridge.referenced_sources, 2);
    assert_eq!(result.stats().bridge.missing_images, 1);
    assert_eq!(result.stats().bridge.total_rgba_bytes, 8);
    assert_eq!(result.plan().parameters(), repeated.plan().parameters());
    assert_eq!(result.plan().input_bytes(), repeated.plan().input_bytes());
    assert_eq!(word(result.plan().input_bytes(), 0), 0x800a141e);
    assert_eq!(word(result.plan().input_bytes(), 1), 0xff010203);
}

#[test]
fn native_unused_images_and_sparse_map_capacity_are_admitted_before_fonts() {
    let target = frame();
    let commands = [text("A")];
    let mut images = ImageStore::new();
    images.insert(
        "unused".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![],
        }),
    );
    assert_eq!(
        plan_native_scene(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands
            }],
            &images,
            &Fonts::new()
        )
        .unwrap_err()
        .kind,
        FallbackKind::InvalidImage
    );
    let mut sparse = ImageStore::with_capacity(1024);
    sparse.insert("one".into(), raster([0, 0, 0, 255]));
    assert_eq!(
        plan_native_scene(target, &[], &sparse, &Fonts::new())
            .unwrap_err()
            .kind,
        FallbackKind::ImageStoreLimit
    );
}

#[test]
fn native_images_fonts_and_rounded_masks_share_source_capacity() {
    let target = Frame::new(2, 2, 0xffffff);
    let mut images = ImageStore::new();
    for n in 0..255 {
        images.insert(format!("{n:03}"), raster([0, 0, 0, 255]));
    }
    let a = [text("\u{200d}")];
    let b = [fill(0.0, 0.0, 1.0, 1.0, Color::BLACK, 0.25)];
    let first = plan_native_scene(
        target,
        &[NativePhase {
            frame: target,
            commands: &a,
        }],
        &images,
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(
        first.stats().bridge.unique_sources + first.stats().mask_sources,
        256
    );
    assert_eq!(
        plan_native_scene(
            target,
            &[
                NativePhase {
                    frame: target,
                    commands: &a
                },
                NativePhase {
                    frame: target,
                    commands: &b
                }
            ],
            &images,
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::MaskPreparation, 1, 0)
    );
}

#[test]
fn native_rounded_coverage_global_limit_refuses_later_phase() {
    let target = Frame::new(512, 512, 0xffffff);
    let block = [fill(0.0, 0.0, 512.0, 256.0, Color::BLACK, 1.0)];
    let tiny = [fill(0.0, 0.0, 1.0, 1.0, Color::BLACK, 0.25)];
    let phases = [
        NativePhase {
            frame: target,
            commands: &block,
        },
        NativePhase {
            frame: target,
            commands: &block,
        },
    ];
    let exact = plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap();
    assert_eq!(exact.stats().coverage_bytes, 262_144);
    assert_eq!(exact.stats().rounded_masks, 2);
    let phases = [
        phases[0],
        phases[1],
        NativePhase {
            frame: target,
            commands: &tiny,
        },
    ];
    assert_eq!(
        plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap_err(),
        at(FallbackKind::MaskPreparation, 2, 0)
    );
}

#[test]
fn native_row_budget_is_global_even_for_narrow_coverage() {
    let target = Frame::new(1, 1024, 0xffffff);
    let half = vec![fill(0.0, 0.0, 1.0, 1024.0, Color::BLACK, 0.25); 32];
    let one = [half[0].clone()];
    let phases = [
        NativePhase {
            frame: target,
            commands: &half,
        },
        NativePhase {
            frame: target,
            commands: &half,
        },
    ];
    let exact = plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap();
    assert_eq!(exact.stats().row_entries, 65_536);
    assert_eq!(exact.stats().coverage_bytes, 65_536);
    let phases = [
        phases[0],
        phases[1],
        NativePhase {
            frame: target,
            commands: &one,
        },
    ];
    assert_eq!(
        plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap_err(),
        at(FallbackKind::GlyphRowLimit, 2, 0)
    );
}

#[test]
fn native_remaining_text_work_is_not_reset_after_earlier_phase() {
    let target = Frame::new(250, 200, 0);
    let first = vec![hidden(); 20];
    let second = [text("\u{200d}")];
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
                }
            ],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::CpuPaintBudget, 1, 0)
    );
}

#[test]
fn native_preparation_bound_includes_reference_and_fits_existing_ui_reserve() {
    let target = Frame::new(1280, 1024, 0xffffff);
    let without = preparation_peak_bytes(target, false).unwrap();
    let with = preparation_peak_bytes(target, true).unwrap();
    assert_eq!(with - without, 1280 * 1024 * 4);
    assert!(with <= 16 * 1024 * 1024);
    assert!(without > 4_718_592 + 262_144 + 262_144 + 1_048_592);
    assert!(preparation_peak_bytes(Frame::new(1281, 1024, 0), false).is_err());
    let prepared = single(
        &[text("AV"), fill(20.0, 1.0, 2.0, 2.0, Color::BLACK, 1.0)],
        frame(),
    )
    .unwrap();
    assert_eq!(
        prepared.stats().preparation_peak_bytes,
        preparation_peak_bytes(frame(), false).unwrap()
    );
    assert!(prepared.retained_cpu_bytes().unwrap() < prepared.stats().preparation_peak_bytes);
    assert!(prepared.plan().gpu_buffer_bytes() <= prepared.stats().gpu_buffer_upper_bound);
    let core_bytes = prepared.plan().retained_cpu_bytes().unwrap();
    assert_eq!(
        prepared.into_plan().retained_cpu_bytes().unwrap(),
        core_bytes
    );
}

#[test]
fn native_old_probe_refusals_are_preserved() {
    let rounded = [fill(0.0, 0.0, 2.0, 2.0, Color::BLACK, 1.0)];
    assert_eq!(
        plan_display_list(&rounded, &ImageStore::new(), frame())
            .unwrap_err()
            .kind,
        FallbackKind::UnsupportedRoundedRect
    );
    assert_eq!(
        super::super::plan_display_list_with_fonts(
            &rounded,
            &ImageStore::new(),
            frame(),
            &Fonts::new()
        )
        .unwrap_err()
        .kind,
        FallbackKind::UnsupportedRoundedRect
    );
    assert_eq!(
        plan_display_list(&[text("A")], &ImageStore::new(), frame())
            .unwrap_err()
            .kind,
        FallbackKind::UnsupportedText
    );
    assert_eq!(
        plan_display_list(&[], &ImageStore::new(), Frame::new(400, 250, 0))
            .unwrap_err()
            .kind,
        FallbackKind::ViewportLimit
    );
    assert!(single(&rounded, frame()).is_ok());
}

#[test]
fn native_warm_cpu_font_cache_does_not_change_scene_bytes_or_charges() {
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let target = frame();
    let commands = [text("AV To"), fill(40.0, 1.0, 2.0, 2.0, Color::BLACK, 1.0)];
    let phases = [NativePhase {
        frame: target,
        commands: &commands,
    }];
    let cold = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    let reference = canvas_pixels(target, &phases, &images, &fonts);
    let warm = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(cold.stats(), warm.stats());
    assert_eq!(cold.plan().parameters(), warm.plan().parameters());
    assert_eq!(cold.plan().input_bytes(), warm.plan().input_bytes());
    assert_eq!(decode_pixels(warm.plan()), reference);
}

#[test]
fn native_normal_desktop_page_and_rounded_chrome_fit_the_closed_profile() {
    let target = Frame::new(1180, 880, 0xffffff);
    let mut page_frame = target;
    page_frame.caller_clip = Rect::new(0.0, 76.0, 1180.0, 779.0);
    page_frame.document_offset = (0.0, 76.0);
    page_frame.viewport_offset = (0.0, 76.0);
    let page = [
        fill(0.0, 0.0, 1180.0, 779.0, Color::WHITE, 0.0),
        text("Page"),
    ];
    let mut address = text("eris:welcome");
    if let DrawCommand::Text { x, y, color, .. } = &mut address {
        *x = 192.0;
        *y = 38.0;
        *color = Color::WHITE;
    }
    let chrome = [
        fill(0.0, 0.0, 1180.0, 76.0, Color::rgb(22, 26, 36), 0.0),
        fill(181.0, 31.0, 984.0, 34.0, Color::rgb(38, 45, 61), 7.0),
        address,
        fill(0.0, 855.0, 1180.0, 25.0, Color::rgb(22, 26, 36), 0.0),
    ];
    let result = plan_native_scene(
        target,
        &[
            NativePhase {
                frame: page_frame,
                commands: &page,
            },
            NativePhase {
                frame: target,
                commands: &chrome,
            },
        ],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(result.plan().profile(), Profile::Native);
    assert_eq!(result.stats().phases, 2);
    assert_eq!(result.stats().rounded_masks, 1);
    assert_eq!(result.stats().rounded_loop_work, 984 * 34);
    assert!(result.stats().text.occurrences > 0);
    assert!(result.plan().gpu_buffer_bytes() < 16 * 1024 * 1024);
    assert!(result.plan().invocations() <= 4_000_000);
    assert!(preparation_peak_bytes(target, true).unwrap() <= 16 * 1024 * 1024);
}

#[test]
fn native_wide_address_bar_admits_real_and_maximum_viewport_without_extra_pixel_work() {
    for (width, height) in [(1275, 764), (1280, 1024)] {
        let target = Frame::new(width, height, 0xffffff);
        let bar_width = width - 196;
        let commands = [fill(
            181.0,
            31.0,
            bar_width as f32,
            34.0,
            Color::rgb(38, 45, 61),
            7.0,
        )];
        let result = single(&commands, target).unwrap();
        let stats = result.stats();
        assert_eq!(stats.bridge.original_commands, 1);
        assert_eq!(stats.bridge.lowered_commands, 2);
        assert_eq!(stats.rounded_masks, 2);
        assert_eq!(stats.mask_sources, 2);
        assert_eq!(stats.row_entries, 68);
        assert_eq!(stats.coverage_bytes, bar_width as usize * 34);
        assert_eq!(stats.rounded_loop_work, u64::from(bar_width) * 34);
        assert_eq!(
            stats.cpu_pixel_upper_bound,
            u64::from(width) * u64::from(height)
        );
        let draws = result.plan().draws();
        assert_eq!(draws.len(), 3);
        assert_eq!(draws[1].bounds(), (181, 31, 1024, 34));
        assert_eq!(draws[2].bounds(), (1205, 31, bar_width - 1024, 34));
        let full_dispatch = u64::from(width.div_ceil(8)) * u64::from(height.div_ceil(8)) * 64;
        let rounded_dispatch = u64::from(bar_width.div_ceil(8)) * 5 * 64;
        assert_eq!(
            result.plan().invocations(),
            full_dispatch * 2 + rounded_dispatch
        );
        assert_eq!(
            result.plan().input_bytes().len(),
            4 * (bar_width as usize * 34 + 68)
        );
        assert!(preparation_peak_bytes(target, true).unwrap() <= 16 * 1024 * 1024);
        assert_eq!(
            decode_pixels(result.plan()),
            canvas_pixels(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &ImageStore::new(),
                &Fonts::new()
            )
        );
    }
}

#[test]
fn native_wide_translucent_seam_uses_independent_integer_targets() {
    let target = Frame::new(1079, 1, 0xffffff);
    let black = fill(0.0, 0.0, 1079.0, 1.0, Color::rgba(0, 0, 0, 128), 0.25);
    let first = single(std::slice::from_ref(&black), target).unwrap();
    // coverage191, alpha=floor(128*191/255)=95, white→160 per channel.
    assert_eq!(decode_pixels(first.plan()), vec![0xa0a0a0; 1079]);
    let commands = [
        black,
        fill(0.0, 0.0, 1079.0, 1.0, Color::rgb(255, 0, 0), 0.25),
    ];
    let result = single(&commands, target).unwrap();
    // Red alpha191 over160 gives round((255*191+160*64)/255)=231;
    // other channels round(160*64/255)=40. Both seams have one blend/pass.
    assert_eq!(decode_pixels(result.plan()), vec![0xe72828; 1079]);
    assert_eq!(result.stats().bridge.lowered_commands, 4);
    let draws = result.plan().draws();
    assert_eq!(draws[1].bounds(), (0, 0, 1024, 1));
    assert_eq!(draws[2].bounds(), (1024, 0, 55, 1));
    assert_eq!(draws[3].bounds(), draws[1].bounds());
    assert_eq!(draws[4].bounds(), draws[2].bounds());
}

#[test]
fn native_wide_tiles_preserve_fractional_fixed_and_interleaved_phase_order() {
    let target = Frame::new(1280, 80, 0xffffff);
    let mut page_frame = target;
    page_frame.document_offset = (-0.25, 3.5);
    page_frame.viewport_offset = (0.25, 1.25);
    let commands = [
        DrawCommand::PushClip {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 1060.5,
                height: 50.0,
            },
        },
        fill(
            -0.25,
            0.25,
            1200.0,
            5.25,
            Color::rgba(20, 70, 130, 180),
            7.0,
        ),
        DrawCommand::PushFixed,
        fill(0.25, 8.25, 1270.0, 6.5, Color::rgba(180, 30, 90, 110), 7.0),
        image("mark", 1023.5),
        text("A"),
        DrawCommand::PopFixed,
        fill(1000.25, 4.0, 80.0, 5.0, Color::rgba(5, 210, 80, 140), 0.25),
        DrawCommand::PopClip,
    ];
    let overlay = [
        fill(1023.25, 8.0, 3.0, 5.0, Color::rgba(220, 150, 40, 90), 0.25),
        text("B"),
    ];
    let phases = [
        NativePhase {
            frame: page_frame,
            commands: &commands,
        },
        NativePhase {
            frame: target,
            commands: &overlay,
        },
    ];
    let mut images = ImageStore::new();
    images.insert("mark".into(), raster([25, 190, 250, 150]));
    let fonts = Fonts::new();
    let result = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(result.stats().rounded_masks, 6);
    assert_eq!(
        decode_pixels(result.plan()),
        canvas_pixels(target, &phases, &images, &fonts)
    );
}

#[test]
fn native_wide_group_shares_sources_and_original_operation_allowance() {
    let target = Frame::new(1025, 1, 0xffffff);
    let bar = fill(0.0, 0.0, 1025.0, 1.0, Color::BLACK, 0.25);
    let commands = [bar.clone()];
    let phases = [NativePhase {
        frame: target,
        commands: &commands,
    }];
    let mut images = ImageStore::new();
    for index in 0..254 {
        images.insert(format!("{index:03}"), raster([0, 0, 0, 255]));
    }
    let fonts = Fonts::new();
    let exact = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(
        exact.stats().bridge.unique_sources + exact.stats().mask_sources,
        256
    );
    images.insert("last".into(), raster([0, 0, 0, 255]));
    assert_eq!(
        plan_native_scene(target, &phases, &images, &fonts).unwrap_err(),
        at(FallbackKind::MaskPreparation, 0, 0)
    );
    let mut commands = vec![hidden(); 254];
    commands.push(bar.clone());
    let exact = single(&commands, target).unwrap();
    assert_eq!(exact.stats().bridge.original_commands, 255);
    assert_eq!(exact.stats().bridge.lowered_commands, 2);
    commands.insert(0, hidden());
    assert_eq!(
        single(&commands, target).unwrap_err(),
        at(FallbackKind::CommandLimit, 0, 255)
    );
}

#[test]
fn native_wide_coverage_total_is_global_across_phases() {
    let target = Frame::new(1025, 256, 0xffffff);
    let wide = [fill(0.0, 0.0, 1025.0, 255.0, Color::BLACK, 0.25)];
    let rest = [
        fill(0.0, 0.0, 3.0, 256.0, Color::BLACK, 0.25),
        fill(0.0, 0.0, 1.0, 1.0, Color::BLACK, 0.25),
    ];
    let phases = [
        NativePhase {
            frame: target,
            commands: &wide,
        },
        NativePhase {
            frame: target,
            commands: &rest,
        },
    ];
    let exact = plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap();
    assert_eq!(exact.stats().coverage_bytes, 262_144);
    assert_eq!(exact.stats().rounded_masks, 4);
    assert_eq!(exact.stats().row_entries, 767);
    let extra = [rest[1].clone()];
    let phases = [
        phases[0],
        phases[1],
        NativePhase {
            frame: target,
            commands: &extra,
        },
    ];
    assert_eq!(
        plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap_err(),
        at(FallbackKind::MaskPreparation, 2, 0)
    );
}

fn partition_ledger() -> Ledger {
    Ledger {
        operations: 1,
        sources: 0,
        masks: 0,
        rows: 0,
        row_tables: 0,
        coverage: 0,
        image_bytes: 0,
        image_lut_words: 0,
        targets: 0,
        cpu_pixels: 1025,
        cpu_limit: 1_000_000,
        gpu_upper: 0,
        rounded_loop_work: 0,
    }
}

#[test]
fn native_wide_group_preflight_is_atomic_at_each_global_cutpoint() {
    let target = Frame::new(1025, 1, 0);
    let RoundedTilesDisposition::Tiles(shape) =
        RoundedTiles::prepare_native(target, target.caller_clip, target.caller_clip, 0.25).unwrap()
    else {
        panic!("partition");
    };
    let mut exact = partition_ledger();
    exact.rounded(&shape, 2, 3).unwrap();
    assert_eq!(exact.operations, 2);
    assert_eq!(exact.sources, 2);
    assert_eq!(exact.rows, 2);
    assert_eq!(exact.coverage, 1025);
    assert_eq!(exact.rounded_loop_work, 1025);
    assert_eq!(exact.cpu_pixels, 1025); // original nontext allowance paid once
    assert_eq!(exact.gpu_upper, 4108 + 3 * 256);
    for field in 0..7 {
        let mut fits = partition_ledger();
        match field {
            0 => fits.operations = 255,
            1 => fits.sources = 254,
            2 => fits.rows = MAX_ROW_ENTRIES - 2,
            3 => fits.row_tables = 254,
            4 => fits.coverage = MAX_MASK_COVERAGE_BYTES - 1025,
            5 => fits.targets = Profile::Native.max_gpu_buffer_bytes() - exact.gpu_upper,
            6 => fits.rounded_loop_work = u64::MAX - 1025,
            _ => unreachable!(),
        }
        let mut fails = fits;
        match field {
            0 => fails.operations += 1,
            1 => fails.sources += 1,
            2 => fails.rows += 1,
            3 => fails.row_tables += 1,
            4 => fails.coverage += 1,
            5 => fails.targets += 1,
            6 => fails.rounded_loop_work += 1,
            _ => unreachable!(),
        }
        fits.rounded(&shape, 2, 3).unwrap();
        let before = fails;
        let result =
            shape.materialize(|shape| fails.rounded(shape, 2, 3).map_err(|e| e.to_string()));
        assert!(result.is_err(), "boundary {field}");
        assert_eq!(fails, before, "partial group ledger at boundary {field}");
    }
}
