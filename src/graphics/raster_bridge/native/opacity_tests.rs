use super::*;

// Independent packed-plan interpretation with integer primitive blending and f32 group pop, used only by
// these bridge tests. Literal expected pixels below are the oracle; neither
// this decoder nor Canvas execution is evidence about a GPU driver.
fn decode_opacity_pixels(plan: &Plan) -> Vec<u32> {
    let frame = plan.frame();
    let mut pixels = vec![0; frame.width as usize * frame.height as usize];
    let mut scratch = vec![[0u64; 4]; plan.group_scratch_bytes() as usize / 8];
    assert_eq!(plan.group_scratch_bytes() % 8, 0);
    for (draw_index, draw) in plan.draws().iter().enumerate() {
        let p = |field| parameter(plan, draw_index, field);
        let index = |base: usize, x: u32, y: u32| {
            assert_eq!(p(base) % 2, 0);
            assert!(x >= p(base + 1) && y >= p(base + 2));
            assert!(x - p(base + 1) < p(base + 3));
            assert!(y - p(base + 2) < p(base + 4));
            (p(base) / 2 + (y - p(base + 2)) * p(base + 3) + x - p(base + 1)) as usize
        };
        let group = draw.target_is_group();
        assert_eq!(group, p(21) == 1);
        let (x0, y0, width, height) = draw.bounds();
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let root_index = (y * frame.width + x) as usize;
                let destination = if group {
                    scratch[index(16, x, y)]
                } else {
                    let rgb = pixels[root_index];
                    [
                        u64::from(rgb & 255) * 257,
                        u64::from((rgb >> 8) & 255) * 257,
                        u64::from((rgb >> 16) & 255) * 257,
                        65535,
                    ]
                };
                let next = match draw.kind() {
                    DrawKind::GroupClear => {
                        assert!(group);
                        [0; 4]
                    }
                    DrawKind::GroupComposite => {
                        let source = scratch[index(24, x, y)];
                        assert!(source.iter().all(|&channel| channel <= 65535));
                        assert!(source[..3].iter().all(|&channel| channel <= source[3]));
                        let opacity = match p(30) {
                            0 => {
                                let k = p(29);
                                assert!((1..256).contains(&k));
                                assert_eq!(p(31), 0, "grid ABI keeps reserved bits zero");
                                k as f32 / 256.0
                            }
                            1 => {
                                assert_eq!(p(29), 0, "raw opacity has no grid numerator");
                                let value = f32::from_bits(p(31));
                                assert!(value.is_finite() && value > 0.0 && value < 1.0);
                                value
                            }
                            flag => panic!("unknown opacity route {flag}"),
                        };
                        // Independent oracle: preserve Canvas's separate f32
                        // division, multiplication, subtraction and pop stages.
                        // Do not reproduce the shader's integer emulation here.
                        let inverse = 1.0 - source[3] as f32 / 65535.0 * opacity;
                        let mut next = [0; 4];
                        for (channel, next) in next.iter_mut().enumerate() {
                            let value = source[channel] as f32 * opacity
                                + destination[channel] as f32 * inverse;
                            *next = if group {
                                value.round() as u64
                            } else {
                                (value / 257.0).round() as u64 * 257
                            };
                            assert!(*next <= 65535);
                        }
                        next
                    }
                    kind => {
                        let input = plan.input_bytes();
                        let (source, alpha) = match kind {
                            DrawKind::Rectangle => (p(6), p(6) >> 24),
                            DrawKind::Image => {
                                let sx = word(input, (p(12) + x - x0) as usize);
                                let sy = word(input, (p(13) + y - y0) as usize);
                                let color = word(input, (p(8) + sy * p(9) + sx) as usize);
                                (color, color >> 24)
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
                            _ => unreachable!(),
                        };
                        let alpha = u64::from(alpha);
                        let mut next = [0; 4];
                        if group {
                            let alpha16 = alpha * 257;
                            for (channel, next) in next.iter_mut().enumerate().take(3) {
                                let color = u64::from((source >> (channel * 8)) & 255);
                                *next = (color * alpha16 + 127) / 255
                                    + (destination[channel] * (65535 - alpha16) + 32767) / 65535;
                            }
                            next[3] =
                                alpha16 + (destination[3] * (65535 - alpha16) + 32767) / 65535;
                        } else {
                            for (channel, next) in next.iter_mut().enumerate().take(3) {
                                let color = u64::from((source >> (channel * 8)) & 255);
                                *next = ((color * alpha
                                    + (destination[channel] / 257) * (255 - alpha)
                                    + 127)
                                    / 255)
                                    * 257;
                            }
                            next[3] = 65535;
                        }
                        next
                    }
                };
                if group {
                    scratch[index(16, x, y)] = next;
                } else {
                    pixels[root_index] =
                        ((next[2] / 257) << 16 | (next[1] / 257) << 8 | (next[0] / 257)) as u32;
                }
            }
        }
    }
    pixels
}

fn checked_pixels(
    target: Frame,
    phases: &[NativePhase<'_>],
    images: &ImageStore,
    expected: &[u32],
) -> NativeScenePlan {
    let fonts = Fonts::new();
    let result = plan_native_scene(target, phases, images, &fonts).unwrap();
    assert_eq!(decode_opacity_pixels(result.plan()), expected);
    assert_eq!(canvas_pixels(target, phases, images, &fonts), expected);
    assert!(result.plan().gpu_buffer_bytes() <= result.stats().gpu_buffer_upper_bound);
    assert!(result.stats().gpu_buffer_upper_bound <= Profile::Native.max_gpu_buffer_bytes());
    result
}

fn composite_records(plan: &Plan) -> Vec<([u32; 3], bool)> {
    plan.draws()
        .iter()
        .enumerate()
        .filter(|(_, draw)| draw.kind() == DrawKind::GroupComposite)
        .map(|(index, draw)| {
            (
                [
                    parameter(plan, index, 29),
                    parameter(plan, index, 30),
                    parameter(plan, index, 31),
                ],
                draw.target_is_group(),
            )
        })
        .collect()
}

#[test]
fn native_full_opacity_preserves_eight_static_literals_and_threshold_neighbor() {
    // The first eight rows are transcribed from the separately held literal
    // candidates. Their bits and pixels are not generated by a planner/decoder.
    let cases = [
        (0x3dcccccd, 0x000000, 232, 0x171717),
        (0x3f333333, 0x000000, 5, 0x040404),
        (0x00000001, 0x334455, 255, 0x334455),
        (0x007fffff, 0x334455, 255, 0x334455),
        (0x00800000, 0x334455, 255, 0x334455),
        (0x33000000, 0x334455, 255, 0x334455),
        (0x33000001, 0x000000, 255, 0x000000),
        (0x3f7fffff, 0x000000, 5, 0x050505),
        (0x32ffffff, 0x334455, 255, 0x334455),
    ];
    for (bits, clear, gray, expected) in cases {
        let target = Frame::new(1, 1, clear);
        let commands = [
            DrawCommand::PushOpacity {
                opacity: f32::from_bits(bits),
            },
            fill(0.0, 0.0, 1.0, 1.0, Color::rgb(gray, gray, gray), 0.0),
            DrawCommand::PopOpacity,
        ];
        let result = checked_pixels(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands,
            }],
            &ImageStore::new(),
            &[expected],
        );
        assert_eq!(composite_records(result.plan()), [([0, 1, bits], false)]);
        assert_eq!(result.plan().group_scratch_bytes(), 8);
        assert_eq!(result.stats().cpu_pixel_upper_bound, 3);
        assert_eq!(result.plan().invocations(), 320);
        assert_eq!(
            result
                .plan()
                .draws()
                .iter()
                .map(|d| d.kind())
                .collect::<Vec<_>>(),
            [
                DrawKind::Rectangle,
                DrawKind::GroupClear,
                DrawKind::Rectangle,
                DrawKind::GroupComposite,
            ]
        );
    }
}

#[test]
fn native_full_opacity_nested_transparent_raw_and_grid_records_remain_distinct() {
    let target = Frame::new(1, 1, 0xffffff);
    let commands = [
        DrawCommand::PushOpacity {
            opacity: f32::from_bits(0x3dcccccd),
        },
        DrawCommand::PushOpacity {
            opacity: f32::from_bits(0x3f333333),
        },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(0, 0, 0, 128), 0.0),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0xf6f6f6],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 16);
    assert_eq!(
        composite_records(result.plan()),
        [([0, 1, 0x3f333333], true), ([0, 1, 0x3dcccccd], false)]
    );
    let mut mixed = commands.clone();
    mixed[1] = DrawCommand::PushOpacity { opacity: 0.5 };
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &mixed,
        }],
        &ImageStore::new(),
        &[0xf9f9f9],
    );
    assert_eq!(
        composite_records(result.plan()),
        [([128, 0, 0], true), ([0, 1, 0x3dcccccd], false)]
    );
}

#[test]
fn native_full_opacity_positive_subnormal_keeps_validation_and_materialized_costs() {
    let tiny = f32::from_bits(1);
    let target = Frame::new(1, 1, 0x334455);
    let bad = [
        DrawCommand::PushOpacity { opacity: tiny },
        fill(f32::NAN, 0.0, 0.0, 0.0, Color::TRANSPARENT, 0.0),
        DrawCommand::PopOpacity,
    ];
    assert_eq!(
        single(&bad, target).unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 1)
    );
    let bad = [
        DrawCommand::PushOpacity { opacity: tiny },
        DrawCommand::PopFixed,
    ];
    assert_eq!(
        single(&bad, target).unwrap_err(),
        at(FallbackKind::InvalidScope, 0, 1)
    );
    let target = Frame::new(512, 512, 0xffffff);
    let group = [
        DrawCommand::PushOpacity { opacity: tiny },
        fill(0.0, 0.0, 512.0, 512.0, Color::BLACK, 0.0),
        DrawCommand::PopOpacity,
    ];
    let mut commands = Vec::new();
    for _ in 0..4 {
        commands.extend(group.clone());
    }
    let result = single(&commands, target).unwrap();
    assert_eq!(result.plan().group_scratch_bytes(), 8_388_608);
    assert_eq!(result.plan().invocations(), 3_670_016);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 3_145_728);
    assert_eq!(
        composite_records(result.plan()),
        vec![([0, 1, 1], false); 4]
    );
    commands.extend(group);
    assert_eq!(
        single(&commands, target).unwrap_err().kind,
        FallbackKind::PlannerLimit
    );
}

#[test]
fn native_opacity_literal_half_tie_and_nested_rounding() {
    let target = Frame::new(3, 1, 0);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgb(65, 0, 0), 0.0),
        DrawCommand::PopOpacity,
    ];
    let first = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0x210000, 0, 0],
    );
    assert_eq!(first.plan().group_scratch_bytes(), 8);
    assert_eq!(first.stats().cpu_pixel_upper_bound, 7); // two full-area markers + one pixel
    assert_eq!(first.stats().bridge.original_commands, 3);
    assert_eq!(first.stats().bridge.lowered_commands, 3);
    assert_eq!(
        first.stats().gpu_buffer_upper_bound,
        3 * 4 + first.plan().conversion_buffer_bytes() + 4 * PARAM_STRIDE as u64 + 8
    );
    let target = Frame::new(4, 1, 0xffffff);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 3.0, 1.0, Color::rgb(255, 0, 0), 0.0),
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(1.0, 0.0, 1.0, 1.0, Color::rgb(0, 0, 255), 0.0),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let nested = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0xff8080, 0xbf80bf, 0xff8080, 0xffffff],
    );
    assert_eq!(nested.plan().group_scratch_bytes(), 32);
}

#[test]
fn native_opacity_unit_and_zero_keep_destination_and_following_pixels() {
    let target = Frame::new(1, 1, 0);
    let commands = [
        DrawCommand::PushOpacity { opacity: 1.0 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(1, 0, 0, 128), 0.0),
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(0, 0, 0, 1), 0.0),
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0x010000],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 0);
    let nested_unit = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 1.0, 1.0, Color::BLACK, 0.0),
        DrawCommand::PushOpacity { opacity: 1.0 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(1, 0, 0, 128), 0.0),
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(0, 0, 0, 1), 0.0),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &nested_unit,
        }],
        &ImageStore::new(),
        &[0],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 8);
    for opacity in [0.0, -0.0] {
        let target = Frame::new(2, 1, 0xffffff);
        let commands = [
            DrawCommand::PushOpacity { opacity },
            fill(0.0, 0.0, 2.0, 1.0, Color::rgba(255, 0, 0, 128), 0.0),
            DrawCommand::PopOpacity,
            fill(1.0, 0.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0),
        ];
        let result = checked_pixels(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands,
            }],
            &ImageStore::new(),
            &[0xffffff, 0x00ff00],
        );
        assert_eq!(result.plan().group_scratch_bytes(), 0);
        assert_eq!(result.stats().cpu_pixel_upper_bound, 7); // two full-area markers + 2 + 1 pixels
    }
}

#[test]
fn native_opacity_fixed_escape_uses_caller_clip_and_restores_document_offset() {
    let target = Frame::new(6, 2, 0xffffff);
    let mut phase = target;
    phase.document_offset = (0.0, -5.0);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 5.0, 5.0, 2.0, Color::rgb(255, 0, 0), 0.0),
        DrawCommand::PushClip {
            rect: BrowserRect {
                x: 0.0,
                y: 5.0,
                width: 1.0,
                height: 1.0,
            },
        },
        DrawCommand::PushFixed,
        fill(3.0, 0.0, 1.0, 1.0, Color::rgb(0, 0, 255), 0.0),
        DrawCommand::PopFixed,
        fill(0.0, 5.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: phase,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[
            0x80ff80, 0xff8080, 0xff8080, 0x8080ff, 0xff8080, 0xffffff, 0xff8080, 0xff8080,
            0xff8080, 0xff8080, 0xff8080, 0xffffff,
        ],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 80);
    let mut outside = commands.clone();
    outside[4] = fill(5.0, 0.0, 1.0, 1.0, Color::rgb(0, 0, 255), 0.0);
    let outside = checked_pixels(
        target,
        &[NativePhase {
            frame: phase,
            commands: &outside,
        }],
        &ImageStore::new(),
        &[
            0x80ff80, 0xff8080, 0xff8080, 0xff8080, 0xff8080, 0x8080ff, 0xff8080, 0xff8080,
            0xff8080, 0xff8080, 0xff8080, 0xffffff,
        ],
    );
    assert_eq!(outside.plan().group_scratch_bytes(), 96);
}

#[test]
fn native_opacity_fractional_clip_and_separate_phase_restore_root_target() {
    let target = Frame::new(3, 1, 0xffffff);
    let mut clipped = target;
    clipped.caller_clip = Rect::new(0.5, 0.0, 1.0, 1.0);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 3.0, 1.0, Color::rgb(255, 0, 0), 0.0),
        DrawCommand::PopOpacity,
    ];
    let overlay = [fill(2.0, 0.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0)];
    let result = checked_pixels(
        target,
        &[
            NativePhase {
                frame: clipped,
                commands: &commands,
            },
            NativePhase {
                frame: target,
                commands: &overlay,
            },
        ],
        &ImageStore::new(),
        &[0xffffff, 0xff8080, 0x00ff00],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 8);
}

#[test]
fn native_opacity_translucent_image_preserves_exact_source_and_order() {
    let target = Frame::new(3, 1, 0);
    let mut images = ImageStore::new();
    images.insert("red-half".into(), raster([255, 0, 0, 128]));
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.25 },
        fill(0.0, 0.0, 2.0, 1.0, Color::WHITE, 0.0),
        image("red-half", 1.0),
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &images,
        &[0x404040, 0x402020, 0],
    );
    assert_eq!(result.stats().bridge.referenced_sources, 1);
    assert_eq!(result.stats().bridge.referenced_rgba_bytes, 4);
    assert!(
        result
            .plan()
            .draws()
            .iter()
            .any(|d| d.kind() == DrawKind::Image && d.target_is_group())
    );
}

#[test]
fn native_opacity_text_and_rounded_masks_remain_inside_opaque_backing() {
    let target = frame();
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 64.0, 32.0, Color::WHITE, 0.0),
        fill(32.0, 2.0, 20.0, 20.0, Color::rgba(255, 0, 0, 128), 4.0),
        text("A"),
        DrawCommand::PopOpacity,
    ];
    let fonts = Fonts::new();
    let phases = [NativePhase {
        frame: target,
        commands: &commands,
    }];
    let result = plan_native_scene(target, &phases, &ImageStore::new(), &fonts).unwrap();
    assert!(result.stats().text.occurrences > 0);
    assert!(result.stats().rounded_masks > 0);
    let decoded = decode_opacity_pixels(result.plan());
    assert_eq!(
        decoded,
        canvas_pixels(target, &phases, &ImageStore::new(), &fonts)
    );
    assert!(decoded.iter().any(|&p| p != 0xffffff));
    assert_eq!(result.plan().group_scratch_bytes(), 64 * 32 * 8);
    assert!(
        result
            .plan()
            .draws()
            .iter()
            .filter(|d| d.kind() == DrawKind::Glyph)
            .all(|d| d.target_is_group())
    );
}

#[test]
fn native_opacity_unbacked_sources_and_nongrid_values_admit_whole_scene() {
    let target = Frame::new(4, 1, 0xffffff);
    let ordinary = [fill(3.0, 0.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0)];
    let mut images = ImageStore::new();
    images.insert("opaque".into(), raster([255, 0, 0, 255]));
    for body in [
        vec![
            fill(0.0, 0.0, 2.0, 1.0, Color::BLACK, 0.0),
            fill(1.0, 0.0, 2.0, 1.0, Color::WHITE, 0.0),
        ],
        vec![image("opaque", 0.0)],
        vec![fill(0.0, 0.0, 2.0, 1.0, Color::BLACK, 0.25)],
    ] {
        let mut commands = vec![DrawCommand::PushOpacity { opacity: 0.5 }];
        commands.extend(body);
        commands.push(DrawCommand::PopOpacity);
        let phases = [
            NativePhase {
                frame: target,
                commands: &ordinary,
            },
            NativePhase {
                frame: target,
                commands: &commands,
            },
        ];
        let fonts = Fonts::new();
        let result = plan_native_scene(target, &phases, &images, &fonts).unwrap();
        let decoded = decode_opacity_pixels(result.plan());
        assert_eq!(decoded, canvas_pixels(target, &phases, &images, &fonts));
        assert_ne!(decoded[0], 0xffffff);
        assert_eq!(decoded[3], 0x00ff00);
        assert!(result.plan().group_scratch_bytes() > 0);
    }
    for prefix in [None, Some(0.0)] {
        let mut commands = Vec::new();
        if let Some(opacity) = prefix {
            commands.push(DrawCommand::PushOpacity { opacity });
        }
        commands.extend([
            DrawCommand::PushOpacity { opacity: 0.1 },
            fill(0.0, 0.0, 1.0, 1.0, Color::BLACK, 0.0),
            DrawCommand::PopOpacity,
        ]);
        if prefix.is_some() {
            commands.push(DrawCommand::PopOpacity);
        }
        let expected = if prefix.is_some() {
            [0xffffff; 4]
        } else {
            [0xe6e6e6, 0xffffff, 0xffffff, 0xffffff]
        };
        let result = checked_pixels(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands,
            }],
            &ImageStore::new(),
            &expected,
        );
        assert_eq!(
            result.plan().group_scratch_bytes(),
            if prefix.is_some() { 0 } else { 8 }
        );
    }
    // The old probe path still refuses even typed zero/unit opacity scopes.
    for opacity in [0.0, 0.1, 0.5, 0.7, 1.0, f32::from_bits(1)] {
        let commands = [
            DrawCommand::PushOpacity { opacity },
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            plan_display_list(&commands, &ImageStore::new(), target)
                .unwrap_err()
                .kind,
            FallbackKind::UnsupportedOpacity
        );
    }
}

#[test]
fn native_opacity_hidden_inputs_and_mixed_scope_limits_are_fully_validated() {
    let target = frame();
    let fonts = Fonts::new();
    for opacity in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.1] {
        let commands = [
            text("A"),
            DrawCommand::PushOpacity { opacity },
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            plan_native_scene(
                target,
                &[NativePhase {
                    frame: target,
                    commands: &commands
                }],
                &ImageStore::new(),
                &fonts
            )
            .unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 1)
        );
    }
    assert_eq!(fonts.cache.borrow().len(), 0);
    let bad = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        fill(f32::NAN, 0.0, 0.0, 0.0, Color::TRANSPARENT, 0.0),
        DrawCommand::PopOpacity,
    ];
    assert_eq!(
        single(&bad, target).unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 1)
    );
    let bad = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PopFixed,
    ];
    assert_eq!(
        single(&bad, target).unwrap_err(),
        at(FallbackKind::InvalidScope, 0, 1)
    );
    let mut opens = Vec::new();
    let mut closes = Vec::new();
    for i in 0..MAX_SCOPES {
        let (open, close) = match i % 3 {
            0 => (
                DrawCommand::PushOpacity { opacity: 1.0 },
                DrawCommand::PopOpacity,
            ),
            1 => (DrawCommand::PushFixed, DrawCommand::PopFixed),
            _ => (
                DrawCommand::PushClip {
                    rect: BrowserRect {
                        x: 0.0,
                        y: 0.0,
                        width: 64.0,
                        height: 32.0,
                    },
                },
                DrawCommand::PopClip,
            ),
        };
        opens.push(open);
        closes.push(close);
    }
    let mut exact = opens.clone();
    exact.extend(closes.into_iter().rev());
    assert_eq!(
        single(&exact, target).unwrap().plan().group_scratch_bytes(),
        0
    );
    opens.push(DrawCommand::PushOpacity { opacity: 1.0 });
    assert_eq!(
        single(&opens, target).unwrap_err(),
        at(FallbackKind::ScopeLimit, 0, MAX_SCOPES)
    );
    let mut bad_images = ImageStore::new();
    bad_images.insert(
        "broken".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0; 3],
        }),
    );
    let hidden_image = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        image("broken", 0.0),
        DrawCommand::PopOpacity,
    ];
    assert_eq!(
        plan_native_scene(
            target,
            &[NativePhase {
                frame: target,
                commands: &hidden_image
            }],
            &bad_images,
            &fonts
        )
        .unwrap_err()
        .kind,
        FallbackKind::InvalidImage
    );
}

#[test]
fn native_opacity_does_not_drop_original_command_or_cpu_admission_costs() {
    let target = Frame::new(1, 1, 0xffffff);
    let mut commands = Vec::new();
    for _ in 0..128 {
        commands.extend([
            DrawCommand::PushOpacity { opacity: 0.0 },
            DrawCommand::PopOpacity,
        ]);
    }
    let exact = single(&commands, target).unwrap();
    assert_eq!(exact.stats().bridge.original_commands, 256);
    assert_eq!(exact.stats().bridge.lowered_commands, 256);
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 256);
    commands.push(hidden());
    assert_eq!(
        single(&commands, target).unwrap_err().kind,
        FallbackKind::CommandLimit
    );
    let large = Frame::new(1000, 1000, 0);
    let mut commands = Vec::new();
    for _ in 0..8 {
        commands.extend([
            DrawCommand::PushOpacity { opacity: 0.0 },
            DrawCommand::PopOpacity,
        ]);
    }
    let exact = single(&commands, large).unwrap();
    assert_eq!(exact.stats().cpu_pixel_upper_bound, 16_000_000);
    commands.extend([
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PopOpacity,
    ]);
    assert_eq!(
        single(&commands, large).unwrap_err().kind,
        FallbackKind::CpuPaintBudget
    );
}

#[test]
fn native_opacity_group_clear_and_pop_keep_the_four_million_work_limit() {
    let target = Frame::new(512, 512, 0xffffff);
    let mut commands = Vec::new();
    let group = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 512.0, 512.0, Color::BLACK, 0.0),
        DrawCommand::PopOpacity,
    ];
    for _ in 0..4 {
        commands.extend(group.clone());
    }
    let result = single(&commands, target).unwrap();
    // Four complete group clear/paint/pop triples plus root clear/conversion.
    assert_eq!(result.plan().invocations(), 3_670_016);
    assert_eq!(result.plan().group_scratch_bytes(), 8_388_608);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 3_145_728);
    assert!(result.stats().gpu_buffer_upper_bound <= 16 * 1024 * 1024);
    commands.extend(group);
    // Five triples still fit the original CPU allowance and command limits,
    // but 17 padded dispatches would require 4,456,448 invocations.
    assert_eq!(
        single(&commands, target).unwrap_err().kind,
        FallbackKind::PlannerLimit
    );
}

#[test]
fn native_opacity_transparent_pop_keeps_root_and_nested_f32_rounding_literals() {
    let root = Frame::new(1, 1, 0x222222);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.75 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(0, 0, 0, 55), 0.0),
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        root,
        &[NativePhase {
            frame: root,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0x1c1c1c],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 8);
    // Ideal rational source-over would yield29 instead of this literal28.
    let root = Frame::new(1, 1, 0x2c2c2c);
    let commands = [
        DrawCommand::PushOpacity {
            opacity: 127.0 / 256.0,
        },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgb(151, 151, 151), 0.0),
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(0, 0, 0, 38), 0.0),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        root,
        &[NativePhase {
            frame: root,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0x5c5c5c],
    );
    // The separately rounded inner parent value35916 makes the final channel92.
    assert_eq!(result.plan().group_scratch_bytes(), 16);
}

#[test]
fn native_opacity_transparent_parent_holes_and_fixed_escape_keep_literal_pixels() {
    let target = Frame::new(1, 1, 0xffffff);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgba(0, 0, 0, 128), 0.0),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0xdfdfdf],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 16);
    let target = Frame::new(5, 1, 0x0a0a0a);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        fill(0.0, 0.0, 1.0, 1.0, Color::rgb(255, 0, 0), 0.0),
        DrawCommand::PushClip {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        },
        DrawCommand::PushFixed,
        fill(4.0, 0.0, 1.0, 1.0, Color::rgb(0, 0, 255), 0.0),
        DrawCommand::PopFixed,
        fill(0.0, 0.0, 5.0, 1.0, Color::rgb(0, 255, 0), 0.0),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
        fill(2.0, 0.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0),
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &ImageStore::new(),
        &[0x850505, 0x0a0a0a, 0x00ff00, 0x0a0a0a, 0x050585],
    );
    assert_eq!(result.plan().group_scratch_bytes(), 40);
}

#[test]
fn native_opacity_image_only_alpha_and_zero_source_keep_root_channels() {
    let target = Frame::new(3, 1, 0x222222);
    let mut images = ImageStore::new();
    images.insert(
        "alpha-strip".into(),
        Arc::new(RasterImage {
            width: 3,
            height: 1,
            rgba: vec![0, 0, 0, 55, 255, 0, 0, 128, 0, 0, 255, 0],
        }),
    );
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.75 },
        DrawCommand::Image {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 3.0,
                height: 1.0,
            },
            key: "alpha-strip".into(),
        },
        DrawCommand::PopOpacity,
    ];
    let result = checked_pixels(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        &images,
        &[0x1c1c1c, 0x751515, 0x222222],
    );
    assert_eq!(result.stats().bridge.referenced_sources, 1);
    assert_eq!(result.stats().bridge.referenced_rgba_bytes, 12);
    assert_eq!(result.plan().group_scratch_bytes(), 24);
    assert_eq!(
        result
            .plan()
            .draws()
            .iter()
            .filter(|d| d.kind() == DrawKind::Image && d.target_is_group())
            .count(),
        1
    );
}

#[test]
fn native_opacity_rounded_only_and_glyph_only_groups_match_full_canvas() {
    let target = frame();
    let images = ImageStore::new();
    let fonts = Fonts::new();
    for (primitive, is_text) in [
        (
            fill(0.0, 0.0, 20.0, 20.0, Color::rgba(255, 0, 0, 128), 6.0),
            false,
        ),
        (text("A"), true),
    ] {
        let commands = [
            DrawCommand::PushOpacity { opacity: 0.5 },
            primitive,
            DrawCommand::PopOpacity,
        ];
        let phases = [NativePhase {
            frame: target,
            commands: &commands,
        }];
        let result = plan_native_scene(target, &phases, &images, &fonts).unwrap();
        let pixels = decode_opacity_pixels(result.plan());
        assert_eq!(pixels, canvas_pixels(target, &phases, &images, &fonts));
        assert!(pixels.iter().any(|&p| p != 0xffffff));
        assert_eq!(pixels[0], 0xffffff);
        assert!(result.plan().group_scratch_bytes() > 0);
        assert!(
            result
                .plan()
                .draws()
                .iter()
                .filter(|d| d.kind() == DrawKind::Glyph)
                .all(|d| d.target_is_group())
        );
        if is_text {
            assert_eq!(result.stats().text.occurrences, 1);
            assert_eq!(result.stats().rounded_masks, 0);
            assert!(
                pixels
                    .iter()
                    .all(|&p| (p & 255) == ((p >> 8) & 255) && (p & 255) == ((p >> 16) & 255))
            );
        } else {
            assert_eq!(result.stats().rounded_masks, 1);
            assert_eq!(result.stats().text.occurrences, 0);
            assert_eq!(pixels[10 * 64 + 10], 0xffbfbf);
            assert_eq!(result.plan().group_scratch_bytes(), 3200);
        }
    }
}
