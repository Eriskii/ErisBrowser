use super::*;
use crate::graphics::{
    Canvas, Color,
    gradient::{GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient},
};
use eris_raster_core::DrawKind;

fn rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}
fn gradient(bounds: BrowserRect, tile: BrowserRect, stops: &[GradientStop]) -> DrawCommand {
    DrawCommand::LinearGradient {
        repeat: [true, true],
        rect: bounds,
        tile,
        radius: 0.0,
        gradient: Arc::new(LinearGradient {
            direction: GradientDirection::Angle(90.0),
            stops: Arc::from(stops),
        }),
        current_color: Color::rgb(255, 0, 0),
        stop_scale: 1.0,
    }
}
fn stop(color: Color, percent: Option<f64>) -> GradientStop {
    GradientStop {
        color: GradientColor::Color(color),
        position: percent.map(|percent| GradientPosition { px: 0.0, percent }),
    }
}
fn ramp(bounds: BrowserRect) -> DrawCommand {
    gradient(
        bounds,
        bounds,
        &[
            stop(Color::BLACK, None),
            stop(Color::rgb(248, 248, 248), None),
        ],
    )
}
fn prepare(
    frame: Frame,
    commands: &[DrawCommand],
    images: &ImageStore,
) -> NativeResult<NativeScenePlan> {
    plan_native_scene(
        frame,
        &[NativePhase { frame, commands }],
        images,
        &Fonts::new(),
    )
}
fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
// Decode only public packed image/rectangle parameters. Expected pixels below
// remain literal; this is not shader execution or a candidate-evaluator oracle.
fn pixels(plan: &Plan) -> Vec<u32> {
    let frame = plan.frame();
    let mut out = vec![0; frame.width as usize * frame.height as usize];
    for (index, draw) in plan.draws().iter().enumerate() {
        let parameters = &plan.parameters()[index * PARAM_STRIDE..];
        let p = |n| word(parameters, n);
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
                    _ => panic!("fixture contains only opaque image/rectangle draws"),
                };
                assert_eq!(value >> 24, 255);
                out[(row * frame.width + column) as usize] = value & 0xffffff;
            }
        }
    }
    out
}

#[test]
fn opaque_ramp_uses_literal_pixels_and_separate_generated_stats() {
    let frame = Frame::new(4, 2, 0xffffff);
    let commands = [ramp(rect(0.0, 0.0, 4.0, 2.0))];
    let result = prepare(frame, &commands, &ImageStore::new()).unwrap();
    let expected = [
        0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9, 0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9,
    ];
    assert_eq!(pixels(result.plan()), expected);
    let mut canvas = Canvas::new(4, 2).unwrap();
    canvas.paint(&commands, &Fonts::new(), &ImageStore::new(), 0.0, 0.0);
    assert_eq!(canvas.pixels, expected);
    assert!(!canvas.exhausted());
    let stats = result.stats();
    assert_eq!(
        (
            stats.generated_gradient_sources,
            stats.generated_gradient_bytes
        ),
        (1, 32)
    );
    assert_eq!(
        (
            stats.gradient_evaluation_work,
            stats.gradient_preparation_work,
            stats.cpu_pixel_upper_bound
        ),
        (32, 8, 40)
    );
    assert_eq!(
        (
            stats.bridge.unique_sources,
            stats.bridge.referenced_sources,
            stats.bridge.total_rgba_bytes,
            stats.bridge.missing_images
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(result.plan().draws()[1].kind(), DrawKind::Image);
    assert_eq!(result.plan().draws()[1].bounds(), (0, 0, 4, 2));
    assert!(result.retained_cpu_bytes().unwrap() < stats.preparation_peak_bytes);
}

#[test]
fn fractional_phase_and_clip_preserve_untrimmed_tile_phase() {
    let target = Frame::new(8, 4, 0xffffff);
    let mut phase = target;
    phase.document_offset = (0.25, -0.75);
    phase.caller_clip = Rect::new(1.25, 0.25, 5.5, 3.5);
    let command = gradient(
        rect(0.5, 1.0, 8.0, 4.0),
        rect(0.0, 1.0, 4.0, 2.0),
        &[
            stop(Color::rgb(255, 0, 0), Some(50.0)),
            stop(Color::rgb(0, 0, 255), Some(50.0)),
        ],
    );
    let result = plan_native_scene(
        target,
        &[NativePhase {
            frame: phase,
            commands: &[command],
        }],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    let white = 0xffffff;
    let expected = [
        white, white, white, white, white, white, white, white, white, white, 0x0000ff, 0x0000ff,
        0xff0000, 0xff0000, 0x0000ff, white, white, white, 0x0000ff, 0x0000ff, 0xff0000, 0xff0000,
        0x0000ff, white, white, white, 0x0000ff, 0x0000ff, 0xff0000, 0xff0000, 0x0000ff, white,
    ];
    assert_eq!(pixels(result.plan()), expected);
    assert_eq!(result.plan().draws()[1].bounds(), (2, 1, 5, 3));
    assert_eq!(
        (
            result.stats().generated_gradient_bytes,
            result.stats().gradient_evaluation_work,
            result.stats().gradient_preparation_work
        ),
        (60, 53, 15)
    );
}

#[test]
fn fixed_scope_escapes_document_clip_and_resolves_current_color() {
    let target = Frame::new(8, 4, 0xffffff);
    let mut phase = target;
    phase.document_offset = (0.0, -10.5);
    phase.viewport_offset = (1.0, 1.0);
    let mut g = gradient(
        rect(0.0, 0.0, 4.0, 2.0),
        rect(0.0, 0.0, 4.0, 2.0),
        &[
            GradientStop {
                color: GradientColor::CurrentColor,
                position: Some(GradientPosition {
                    px: 0.0,
                    percent: 50.0,
                }),
            },
            stop(Color::rgb(0, 0, 255), Some(50.0)),
        ],
    );
    if let DrawCommand::LinearGradient { gradient, .. } = &mut g {
        Arc::make_mut(gradient).direction = GradientDirection::Corner {
            right: true,
            bottom: false,
        };
    }
    let commands = [
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 1.0, 1.0),
        },
        DrawCommand::PushFixed,
        g,
        DrawCommand::PopFixed,
        DrawCommand::PopClip,
    ];
    let result = plan_native_scene(
        target,
        &[NativePhase {
            frame: phase,
            commands: &commands,
        }],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap();
    let w = 0xffffff;
    assert_eq!(
        pixels(result.plan()),
        [
            w, w, w, w, w, w, w, w, w, 0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, w, w, w, w,
            0xff0000, 0xff0000, 0xff0000, 0x0000ff, w, w, w, w, w, w, w, w, w, w, w
        ]
    );
    assert_eq!(result.plan().draws()[1].bounds(), (1, 1, 4, 2));
}

#[test]
fn every_opacity_scope_and_nonopaque_or_rounded_source_refuses_before_images() {
    let frame = Frame::new(8, 4, 0xffffff);
    let mut images = ImageStore::new();
    images.insert(
        "invalid-unvisited".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: Vec::new(),
        }),
    );
    for opacity in [0.0, 0.5, 1.0] {
        let commands = [
            DrawCommand::PushOpacity { opacity },
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 0.0, 0.0),
            },
            ramp(rect(0.0, 0.0, 4.0, 2.0)),
            DrawCommand::PopClip,
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            prepare(frame, &commands, &images).unwrap_err(),
            at(FallbackKind::UnsupportedGradient, 0, 2)
        );
    }
    for variant in 0..3 {
        let mut g = ramp(rect(100.0, 100.0, 4.0, 2.0));
        if let DrawCommand::LinearGradient {
            radius,
            gradient,
            current_color,
            ..
        } = &mut g
        {
            match variant {
                0 => *radius = 0.5,
                1 => {
                    Arc::make_mut(gradient).stops =
                        Arc::from([stop(Color::rgba(1, 2, 3, 254), None)])
                }
                _ => {
                    *current_color = Color::TRANSPARENT;
                    Arc::make_mut(gradient).stops = Arc::from([GradientStop {
                        color: GradientColor::CurrentColor,
                        position: None,
                    }]);
                }
            }
        }
        assert_eq!(
            prepare(frame, &[g], &images).unwrap_err(),
            at(FallbackKind::UnsupportedGradient, 0, 0)
        );
    }
}

#[test]
fn empty_windows_keep_no_generated_payload_or_sampling_debit() {
    let frame = Frame::new(8, 4, 0xffffff);
    for bounds in [rect(0.0, 0.0, 0.0, 2.0), rect(-20.0, 0.0, 4.0, 2.0)] {
        let g = gradient(
            bounds,
            rect(-0.5, -0.25, 4.0, 2.0),
            &[stop(Color::BLACK, None)],
        );
        let result = prepare(frame, &[g], &ImageStore::new()).unwrap();
        assert_eq!(
            (
                result.stats().generated_gradient_sources,
                result.stats().generated_gradient_bytes,
                result.stats().gradient_evaluation_work,
                result.stats().gradient_preparation_work
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(pixels(result.plan()), [0xffffff; 32]);
    }
}

#[test]
fn generated_and_loaded_sources_share_caps_but_keep_stats_distinct() {
    let frame = Frame::new(512, 1, 0xffffff);
    let mut images = ImageStore::new();
    let image = Arc::new(RasterImage {
        width: 512,
        height: 511,
        rgba: vec![255; 512 * 511 * 4],
    });
    images.insert("a".into(), Arc::clone(&image));
    images.insert("alias".into(), image);
    let result = prepare(frame, &[ramp(rect(0.0, 0.0, 512.0, 1.0))], &images).unwrap();
    assert_eq!(
        (
            result.stats().bridge.image_store_entries,
            result.stats().bridge.unique_sources,
            result.stats().bridge.referenced_sources
        ),
        (2, 1, 0)
    );
    assert_eq!(result.stats().bridge.total_rgba_bytes, 1_046_528);
    assert_eq!(result.stats().generated_gradient_bytes, 2_048);
    assert_eq!(
        result.stats().bridge.total_rgba_bytes + result.stats().generated_gradient_bytes,
        1_048_576
    );
    let bigger = Frame::new(513, 1, 0xffffff);
    assert!(prepare(bigger, &[ramp(rect(0.0, 0.0, 513.0, 1.0))], &images).is_err());
}

#[test]
fn unused_loaded_source_slots_still_limit_generated_fragments() {
    let frame = Frame::new(4, 1, 0xffffff);
    let mut images = ImageStore::new();
    for i in 0..255 {
        images.insert(
            i.to_string(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
            }),
        );
    }
    let command = ramp(rect(0.0, 0.0, 4.0, 1.0));
    let result = prepare(frame, std::slice::from_ref(&command), &images).unwrap();
    assert_eq!(result.stats().bridge.unique_sources, 255);
    assert_eq!(result.stats().generated_gradient_sources, 1);
    images.insert(
        "last".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        }),
    );
    assert!(prepare(frame, &[command], &images).is_err());
}

#[test]
fn exact_cpu_admission_and_one_extra_pixel_share_all_phase_work() {
    let target = Frame::new(128, 128, 0xffffff);
    let stops = [stop(Color::BLACK, None); 32];
    let mut commands = Vec::new();
    for _ in 0..6 {
        commands.push(gradient(
            rect(0.0, 0.0, 128.0, 128.0),
            rect(0.0, 0.0, 128.0, 128.0),
            &stops,
        ));
    }
    commands.push(gradient(
        rect(0.0, 0.0, 128.0, 117.0),
        rect(0.0, 0.0, 128.0, 117.0),
        &stops,
    ));
    commands.push(gradient(
        rect(0.0, 0.0, 126.0, 92.0),
        rect(0.0, 0.0, 126.0, 92.0),
        &stops,
    ));
    let phases = [
        NativePhase {
            frame: target,
            commands: &commands[..4],
        },
        NativePhase {
            frame: target,
            commands: &commands[4..],
        },
    ];
    let result = plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap();
    assert_eq!(
        (
            result.stats().generated_gradient_sources,
            result.stats().generated_gradient_bytes
        ),
        (8, 499_488)
    );
    assert_eq!(
        (
            result.stats().gradient_evaluation_work,
            result.stats().gradient_preparation_work,
            result.stats().cpu_pixel_upper_bound
        ),
        (875_128, 124_872, 1_000_000)
    );
    let extra = [DrawCommand::Rect {
        rect: rect(0.0, 0.0, 1.0, 1.0),
        color: Color::BLACK,
        radius: 0.0,
    }];
    let phases = [
        phases[0],
        phases[1],
        NativePhase {
            frame: target,
            commands: &extra,
        },
    ];
    assert_eq!(
        plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new())
            .unwrap_err()
            .kind,
        FallbackKind::CpuPaintBudget
    );
}

#[test]
fn late_unsupported_gradient_preserves_original_phase_and_command() {
    let target = Frame::new(4, 2, 0xffffff);
    let first = [ramp(rect(0.0, 0.0, 4.0, 2.0))];
    let second = [
        DrawCommand::PushOpacity { opacity: 1.0 },
        ramp(rect(0.0, 0.0, 4.0, 2.0)),
        DrawCommand::PopOpacity,
    ];
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
        plan_native_scene(target, &phases, &ImageStore::new(), &Fonts::new()).unwrap_err(),
        at(FallbackKind::UnsupportedGradient, 1, 1)
    );
}

#[test]
fn scaled_length_stops_repeat_without_scaling_the_framebuffer_window() {
    let frame = Frame::new(8, 1, 0xffffff);
    let mut command = gradient(
        rect(0.0, 0.0, 8.0, 1.0),
        rect(0.0, 0.0, 4.0, 1.0),
        &[
            GradientStop {
                color: GradientColor::Color(Color::rgb(255, 0, 0)),
                position: Some(GradientPosition {
                    px: 1.0,
                    percent: 0.0,
                }),
            },
            GradientStop {
                color: GradientColor::Color(Color::rgb(0, 0, 255)),
                position: Some(GradientPosition {
                    px: 1.0,
                    percent: 0.0,
                }),
            },
        ],
    );
    if let DrawCommand::LinearGradient { stop_scale, .. } = &mut command {
        *stop_scale = 2.0;
    }
    let result = prepare(frame, &[command], &ImageStore::new()).unwrap();
    assert_eq!(
        pixels(result.plan()),
        [
            0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff
        ]
    );
    assert_eq!(result.plan().draws()[1].bounds(), (0, 0, 8, 1));
}

#[test]
fn generated_ledger_admits_exact_limits_and_refuses_transactionally() {
    let base = Ledger {
        operations: 0,
        sources: 0,
        masks: 0,
        rows: 0,
        row_tables: 0,
        coverage: 0,
        image_bytes: 0,
        image_lut_words: 0,
        targets: 0,
        cpu_pixels: 0,
        cpu_limit: 1_000_000,
        gpu_upper: 0,
        rounded_loop_work: 0,
    };
    let failure = at(FallbackKind::PlannerLimit, 2, 7);
    for boundary in 0..4 {
        let mut ledger = base;
        match boundary {
            0 => ledger.sources = 255,
            1 => ledger.image_bytes = 1_048_572,
            2 => ledger.image_lut_words = 589_822,
            _ => ledger.targets = 16_777_216 - 256 - 12,
        }
        ledger.gradient(1, 1, failure).unwrap();
        match boundary {
            0 => assert_eq!(ledger.sources, 256),
            1 => assert_eq!(ledger.image_bytes, 1_048_576),
            2 => assert_eq!(ledger.image_lut_words, 589_824),
            _ => assert_eq!(ledger.gpu_upper, 16_777_216),
        }
        let before = ledger;
        assert!(ledger.gradient(1, 1, failure).is_err());
        assert_eq!(
            ledger, before,
            "no ledger mutation on refusal at boundary {boundary}"
        );
    }
}
