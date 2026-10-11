//! Independent opaque Box-gradient literals and admission boundaries.
use super::box_decoration_tests::decode_opacity_pixels;
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use crate::graphics::gradient::{
    GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
};
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
fn square(rect: BrowserRect) -> RoundedBox {
    RoundedBox {
        rect,
        radii: [[0.0; 2]; 4],
    }
}
fn stop(color: Color, percent: Option<f64>) -> GradientStop {
    GradientStop {
        color: GradientColor::Color(color),
        position: percent.map(|percent| GradientPosition { px: 0.0, percent }),
    }
}
fn gradient(stops: &[GradientStop]) -> Arc<LinearGradient> {
    Arc::new(LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: Arc::from(stops),
    })
}
fn ramp() -> Arc<LinearGradient> {
    gradient(&[
        stop(Color::BLACK, None),
        stop(Color::rgb(248, 248, 248), None),
    ])
}
fn hard() -> Arc<LinearGradient> {
    gradient(&[
        stop(Color::rgb(255, 0, 0), Some(50.0)),
        stop(Color::rgb(0, 0, 255), Some(50.0)),
    ])
}
fn red() -> Arc<LinearGradient> {
    gradient(&[stop(Color::rgb(255, 0, 0), None); 2])
}
fn boxed(
    area: BrowserRect,
    tile: BrowserRect,
    repeat: [bool; 2],
    gradient: Arc<LinearGradient>,
) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: square(area),
        cutout: None,
        paint: BoxPaint::LinearGradient {
            tile,
            repeat,
            gradient,
            current_color: Color::WHITE,
            stop_scale: 1.0,
        },
    }))
}
fn decoration(command: &mut DrawCommand) -> &mut BoxDecoration {
    let DrawCommand::BoxDecoration(value) = command else {
        panic!("expected Box gradient")
    };
    Arc::make_mut(value)
}
fn pixels(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|c| match c {
            b'W' => 0xffffff,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            b'K' => 0,
            b'A' => 0x1f1f1f,
            b'C' => 0x5d5d5d,
            b'D' => 0x9b9b9b,
            b'E' => 0xd9d9d9,
            b'I' => 0x112233,
            _ => panic!("unknown literal color"),
        })
        .collect()
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
fn route(plan: &NativeScenePlan, count: usize, work: u64, evaluation: u64, packing: u64) {
    let s = plan.stats();
    assert_eq!(s.box_gradients, count);
    assert_eq!(s.box_loop_work, work);
    assert_eq!(
        (s.gradient_evaluation_work, s.gradient_preparation_work),
        (evaluation, packing)
    );
    assert_eq!(
        (
            s.box_masks,
            s.box_rectangles,
            s.box_background_images,
            s.mask_sources,
            s.coverage_bytes,
            s.row_entries
        ),
        (0, 0, 0, 0, 0, 0)
    );
    assert!(!plan.plan().has_glyphs());
}

#[test]
fn native_box_gradient_ramp_currentcolor_scaled_stops_and_stop_count_are_literal() {
    let frame = Frame::new(4, 2, 0xffffff);
    let area = rect(0.0, 0.0, 4.0, 2.0);
    let mut commands = vec![boxed(area, area, [true; 2], ramp())];
    let BoxPaint::LinearGradient { current_color, .. } = &mut decoration(&mut commands[0]).paint
    else {
        unreachable!()
    };
    *current_color = Color::TRANSPARENT; // No stop references it.
    let plan = checked(
        frame,
        frame,
        &commands,
        &ImageStore::new(),
        &pixels(&["ACDE", "ACDE"]),
    );
    route(&plan, 1, 64, 32, 8);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 64);
    assert_eq!(
        (
            plan.stats().generated_gradient_sources,
            plan.stats().generated_gradient_bytes
        ),
        (1, 32)
    );
    assert_eq!(plan.plan().draws()[1].kind(), DrawKind::Image);
    assert_eq!(plan.plan().draws()[1].bounds(), (0, 0, 4, 2));
    let current = GradientStop {
        color: GradientColor::CurrentColor,
        position: None,
    };
    commands[0] = boxed(area, area, [true; 2], gradient(&[current]));
    let BoxPaint::LinearGradient { current_color, .. } = &mut decoration(&mut commands[0]).paint
    else {
        unreachable!()
    };
    *current_color = Color::rgb(17, 34, 51);
    let plan = checked(
        frame,
        frame,
        &commands,
        &ImageStore::new(),
        &pixels(&["IIII", "IIII"]),
    );
    route(&plan, 1, 52, 20, 8);
    let position = Some(GradientPosition {
        px: 1.0,
        percent: 0.0,
    });
    commands[0] = boxed(
        area,
        area,
        [true; 2],
        gradient(&[
            GradientStop {
                position,
                ..current
            },
            GradientStop {
                color: GradientColor::Color(Color::rgb(0, 0, 255)),
                position,
            },
        ]),
    );
    let BoxPaint::LinearGradient {
        current_color,
        stop_scale,
        ..
    } = &mut decoration(&mut commands[0]).paint
    else {
        unreachable!()
    };
    *current_color = Color::rgb(255, 0, 0);
    *stop_scale = 2.0;
    let plan = checked(
        frame,
        frame,
        &commands,
        &ImageStore::new(),
        &pixels(&["RRBB", "RRBB"]),
    );
    route(&plan, 1, 64, 32, 8);
    commands[0] = boxed(
        area,
        area,
        [true; 2],
        gradient(&[stop(Color::BLACK, None); 32]),
    );
    let plan = checked(frame, frame, &commands, &ImageStore::new(), &[0; 8]);
    route(&plan, 1, 216, 184, 8);
}

#[test]
fn native_box_gradient_repeat_fractional_clip_and_subnormal_windows_are_literal() {
    let frame = Frame::new(8, 4, 0xffffff);
    for (repeat, work, evaluation, packing, bounds, rows) in [
        (
            [false, false],
            144,
            64,
            8,
            (2, 1, 4, 2),
            ["WWWWWWWW", "WWRRBBWW", "WWRRBBWW", "WWWWWWWW"],
        ),
        (
            [true, false],
            176,
            88,
            16,
            (0, 1, 8, 2),
            ["WWWWWWWW", "BBRRBBRR", "BBRRBBRR", "WWWWWWWW"],
        ),
        ([false, true], 176, 88, 16, (2, 0, 4, 4), ["WWRRBBWW"; 4]),
        ([true, true], 208, 104, 32, (0, 0, 8, 4), ["BBRRBBRR"; 4]),
    ] {
        let command = boxed(
            rect(0.0, 0.0, 8.0, 4.0),
            rect(2.0, 1.0, 4.0, 2.0),
            repeat,
            hard(),
        );
        let plan = checked(frame, frame, &[command], &ImageStore::new(), &pixels(&rows));
        route(&plan, 1, work, evaluation, packing);
        assert_eq!(plan.stats().cpu_pixel_upper_bound, work);
        assert_eq!(plan.stats().generated_gradient_bytes, packing as usize * 4);
        assert_eq!(plan.plan().draws()[1].bounds(), bounds);
    }
    let target = Frame::new(6, 3, 0xffffff);
    let mut phase = target;
    phase.caller_clip = Rect::new(0.5, 0.25, 3.0, 1.5);
    for (repeat, work, evaluation, packing, rows) in [
        ([true, true], 34, 17, 3, ["WWWWWW", "WBRBWW", "WWWWWW"]),
        ([false, false], 44, 29, 1, ["WWWWWW", "WBWWWW", "WWWWWW"]),
    ] {
        let command = boxed(
            rect(0.0, 0.0, 6.0, 3.0),
            rect(0.25, 0.25, 2.0, 2.0),
            repeat,
            hard(),
        );
        let plan = checked(
            target,
            phase,
            &[command],
            &ImageStore::new(),
            &pixels(&rows),
        );
        route(&plan, 1, work, evaluation, packing);
    }
    let frame = Frame::new(4, 1, 0xffffff);
    let command = boxed(
        rect(0.0, 0.0, 4.0, 1.0),
        rect(0.5, 0.0, f32::from_bits(1), 1.0),
        [false; 2],
        hard(),
    );
    let plan = checked(
        frame,
        frame,
        &[command],
        &ImageStore::new(),
        &pixels(&["RWWW"]),
    );
    route(&plan, 1, 46, 29, 1);
    assert_eq!(plan.plan().draws()[1].bounds(), (0, 0, 1, 1));
}

#[test]
fn native_box_gradient_fixed_restore_raw_phase_and_empty_windows_keep_work() {
    let target = Frame::new(6, 3, 0xffffff);
    let mut phase = target;
    phase.caller_clip = Rect::new(1.0, 0.0, 4.0, 3.0);
    phase.document_offset = (1.0, 1.0);
    let commands = [
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 1.0, 1.0),
        },
        DrawCommand::PushFixed,
        boxed(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.5, 0.0, 2.0, 1.0),
            [false; 2],
            hard(),
        ),
        DrawCommand::PopFixed,
        boxed(
            rect(0.0, 0.0, 4.0, 1.0),
            rect(0.0, 0.0, 2.0, 1.0),
            [true; 2],
            hard(),
        ),
        DrawCommand::PopClip,
        boxed(
            rect(0.0, 1.0, 4.0, 1.0),
            rect(0.0, 1.0, 2.0, 1.0),
            [true; 2],
            hard(),
        ),
    ];
    let plan = checked(
        target,
        phase,
        &commands,
        &ImageStore::new(),
        &pixels(&["WBWWWW", "WRWWWW", "WRBRBW"]),
    );
    route(&plan, 3, 106, 60, 6);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 106);
    let mut phase = target;
    phase.document_offset = (0.75, 0.75);
    let mut command = boxed(
        rect(0.25, 0.25, 4.0, 1.0),
        rect(-0.25, -0.75, 2.0, 1.0),
        [true; 2],
        hard(),
    );
    decoration(&mut command).shape.radii = [[9.0, 0.0], [0.0, 9.0], [7.0, 0.0], [0.0, 7.0]];
    let plan = checked(
        target,
        phase,
        &[command],
        &ImageStore::new(),
        &pixels(&["WWWWWW", "WBRBRW", "WWWWWW"]),
    );
    route(&plan, 1, 40, 20, 4);
    let frame = Frame::new(8, 4, 0xffffff);
    let commands = [
        boxed(
            rect(0.0, 0.0, 8.0, 4.0),
            rect(20.0, 20.0, 4.0, 2.0),
            [false; 2],
            hard(),
        ),
        boxed(
            rect(20.0, 20.0, 0.0, 0.0),
            rect(0.0, 0.0, 4.0, 2.0),
            [true; 2],
            hard(),
        ),
    ];
    let plan = checked(frame, frame, &commands, &ImageStore::new(), &[0xffffff; 32]);
    route(&plan, 2, 72, 0, 0);
    assert_eq!(
        (
            plan.stats().generated_gradient_sources,
            plan.stats().generated_gradient_bytes
        ),
        (0, 0)
    );
    assert_eq!(plan.plan().draws().len(), 1);
}

#[test]
fn native_box_gradient_validates_scale_and_refuses_hidden_coverage_alpha_and_groups() {
    let frame = Frame::new(4, 2, 0xffffff);
    let area = rect(0.0, 0.0, 4.0, 2.0);
    let empty = rect(20.0, 20.0, 0.0, 0.0);
    let mut valid = boxed(
        area,
        area,
        [true; 2],
        gradient(&[stop(Color::rgb(255, 0, 0), None)]),
    );
    let BoxPaint::LinearGradient { stop_scale, .. } = &mut decoration(&mut valid).paint else {
        unreachable!()
    };
    *stop_scale = 16_000_000.0;
    let plan = checked(
        frame,
        frame,
        std::slice::from_ref(&valid),
        &ImageStore::new(),
        &[0xff0000; 8],
    );
    route(&plan, 1, 52, 20, 8);
    let mut poison = ImageStore::new();
    let source = Arc::new(RasterImage {
        width: 1,
        height: 1,
        rgba: Vec::new(),
    });
    poison.insert("bad".into(), source.clone());
    let refs = Arc::strong_count(&source);
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY, 16_000_001.0] {
        let mut command = valid.clone();
        decoration(&mut command).shape.rect = empty;
        let BoxPaint::LinearGradient { stop_scale, .. } = &mut decoration(&mut command).paint
        else {
            unreachable!()
        };
        *stop_scale = scale;
        assert_eq!(
            prepare(frame, &[command], &poison).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    let invalid_gradients = [
        gradient(&[]),
        gradient(&[stop(Color::BLACK, None); 33]),
        Arc::new(LinearGradient {
            direction: GradientDirection::Angle(f64::NAN),
            stops: Arc::from([stop(Color::BLACK, None)]),
        }),
        gradient(&[GradientStop {
            color: GradientColor::Color(Color::BLACK),
            position: Some(GradientPosition {
                px: f64::INFINITY,
                percent: 0.0,
            }),
        }]),
    ];
    for value in invalid_gradients {
        assert_eq!(
            prepare(frame, &[boxed(empty, area, [true; 2], value)], &poison).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    for tile in [
        rect(0.0, 0.0, 0.0, 1.0),
        rect(f32::NAN, 0.0, 1.0, 1.0),
        rect(0.0, 0.0, 1_000_001.0, 1.0),
    ] {
        assert_eq!(
            prepare(frame, &[boxed(empty, tile, [true; 2], ramp())], &poison).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    for invalid_radius in [f32::NAN, -1.0, 1_000_001.0] {
        let mut command = boxed(empty, area, [true; 2], ramp());
        decoration(&mut command).shape.radii[0][0] = invalid_radius;
        assert_eq!(
            prepare(frame, &[command], &poison).unwrap_err(),
            at(FallbackKind::InvalidGeometry, 0, 0)
        );
    }
    // An ineligible later Box must precede an earlier Color classification
    // which would exceed the mask cap, and the malformed source payload.
    let large = Frame::new(1000, 264, 0xffffff);
    let earlier = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: rect(0.0, 0.0, 1000.0, 264.0),
            radii: [[0.125; 2]; 4],
        },
        cutout: None,
        paint: BoxPaint::Color(Color::BLACK),
    }));
    for variant in 0..7 {
        let mut command = boxed(empty, area, [true; 2], ramp());
        let payload = decoration(&mut command);
        match variant {
            0 => payload.shape.rect.x = 20.25,
            1 => payload.shape.rect.height = 0.25,
            2 => payload.shape.radii[0] = [0.125; 2],
            3 => payload.cutout = Some(square(rect(0.0, 0.0, 0.0, 0.0))),
            4 => payload.cutout = Some(payload.shape),
            5 | 6 => {
                let BoxPaint::LinearGradient {
                    gradient,
                    current_color,
                    ..
                } = &mut payload.paint
                else {
                    unreachable!()
                };
                *gradient = if variant == 5 {
                    Arc::new(LinearGradient {
                        direction: GradientDirection::Angle(90.0),
                        stops: Arc::from([
                            stop(Color::TRANSPARENT, Some(-100.0)),
                            stop(Color::BLACK, Some(-100.0)),
                        ]),
                    })
                } else {
                    Arc::new(LinearGradient {
                        direction: GradientDirection::Angle(90.0),
                        stops: Arc::from([GradientStop {
                            color: GradientColor::CurrentColor,
                            position: None,
                        }]),
                    })
                };
                *current_color = Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 128,
                };
            }
            _ => unreachable!(),
        }
        let fonts = Fonts::new();
        assert_eq!(
            plan_native_scene(
                large,
                &[NativePhase {
                    frame: large,
                    commands: &[earlier.clone(), command]
                }],
                &poison,
                &fonts
            )
            .unwrap_err(),
            at(FallbackKind::UnsupportedBoxDecoration, 0, 1)
        );
        assert!(fonts.cache.borrow().is_empty());
    }
    for opacity in [0.0, 0.5, 1.0] {
        let commands = [
            DrawCommand::PushOpacity { opacity },
            DrawCommand::PushFixed,
            boxed(empty, area, [true; 2], ramp()),
            DrawCommand::PopFixed,
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            prepare(frame, &commands, &poison).unwrap_err(),
            at(FallbackKind::UnsupportedBoxDecoration, 0, 2)
        );
    }
    assert_eq!(Arc::strong_count(&source), refs);
    assert!(source.rgba.is_empty());
}

#[test]
fn native_box_gradient_atomic_fee_and_shared_million_cut_precede_generation() {
    let frame = Frame::new(4, 2, 0xffffff);
    let area = rect(0.0, 0.0, 4.0, 2.0);
    let commands = [boxed(area, area, [true; 2], ramp())];
    let phases = [NativePhase {
        frame,
        commands: &commands,
    }];
    preflight(frame, &phases).unwrap();
    for initial in [63, 64] {
        let mut remaining = initial;
        let mut gradients = GradientReservation::new();
        let mut boxes = vec![None];
        let result =
            geometry_pixel_work(frame, &phases, &mut gradients, &mut boxes, &mut remaining);
        if initial == 64 {
            assert_eq!(result.unwrap(), 64);
            assert_eq!(remaining, 0);
            assert!(matches!(
                boxes[0].as_ref(),
                Some(BoxReservation::LinearGradient { work: 64 })
            ));
            assert!(gradients.windows[0].is_some());
            assert_eq!(
                (
                    gradients.sources,
                    gradients.bytes,
                    gradients.evaluation,
                    gradients.packing
                ),
                (1, 32, 32, 8)
            );
        } else {
            assert_eq!(result.unwrap_err(), at(FallbackKind::CpuPaintBudget, 0, 0));
            assert_eq!(remaining, 63);
            assert!(boxes[0].is_none());
            assert!(gradients.windows[0].is_none());
            assert_eq!(
                (
                    gradients.sources,
                    gradients.bytes,
                    gradients.evaluation,
                    gradients.packing
                ),
                (0, 0, 0, 0)
            );
        }
    }
    let frame = Frame::new(128, 128, 0xffffff);
    let area = rect(0.0, 0.0, 128.0, 128.0);
    let mut commands = vec![boxed(area, area, [true; 2], red()); 10];
    commands.push(DrawCommand::Rect {
        rect: area,
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    commands.push(DrawCommand::Rect {
        rect: rect(0.0, 0.0, 104.0, 4.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    let plan = checked(
        frame,
        frame,
        &commands,
        &ImageStore::new(),
        &vec![0xff0000; 16_384],
    );
    route(&plan, 10, 983_200, 491_600, 163_840);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 1_000_000);
    assert_eq!(plan.stats().generated_gradient_bytes, 655_360);
    commands.push(DrawCommand::Rect {
        rect: rect(0.0, 0.0, 1.0, 1.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
    });
    assert_eq!(
        prepare(frame, &commands, &ImageStore::new()).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
}

#[test]
fn native_box_gradient_shared_arc_keeps_windows_and_loaded_statistics_separate() {
    let frame = Frame::new(4, 3, 0xffffff);
    let shared = ramp();
    let area0 = rect(0.0, 0.0, 4.0, 1.0);
    let area1 = rect(0.0, 1.0, 4.0, 1.0);
    let area2 = rect(0.0, 2.0, 4.0, 1.0);
    let commands = [
        DrawCommand::LinearGradient {
            rect: area0,
            tile: area0,
            repeat: [true; 2],
            radius: 0.0,
            gradient: shared.clone(),
            current_color: Color::WHITE,
            stop_scale: 1.0,
        },
        boxed(area1, area1, [true; 2], shared.clone()),
        boxed(area2, area2, [true; 2], shared.clone()),
    ];
    let refs = Arc::strong_count(&shared);
    let mut images = ImageStore::new();
    let image = Arc::new(RasterImage {
        width: 1,
        height: 1,
        rgba: vec![0, 0, 255, 255],
    });
    images.insert("blue".into(), image.clone());
    images.insert("alias".into(), image.clone());
    let image_refs = Arc::strong_count(&image);
    let plan = checked(
        frame,
        frame,
        &commands,
        &images,
        &pixels(&["ACDE", "ACDE", "ACDE"]),
    );
    route(&plan, 2, 80, 60, 12);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 104);
    assert_eq!(
        (
            plan.stats().generated_gradient_sources,
            plan.stats().generated_gradient_bytes
        ),
        (3, 48)
    );
    let s = &plan.stats().bridge;
    assert_eq!(
        (
            s.unique_sources,
            s.referenced_sources,
            s.total_rgba_bytes,
            s.referenced_rgba_bytes,
            s.missing_images
        ),
        (1, 0, 4, 0, 0)
    );
    assert_eq!(Arc::strong_count(&shared), refs);
    assert_eq!(Arc::strong_count(&image), image_refs);
    assert!(plan.retained_cpu_bytes().unwrap() < plan.stats().preparation_peak_bytes);
}

#[test]
fn native_box_gradient_shared_source_bytes_slots_and_large_window_remain_bounded() {
    let frame = Frame::new(512, 512, 0xffffff);
    let area = rect(0.0, 0.0, 512.0, 512.0);
    let command = boxed(area, area, [true; 2], red());
    let plan = checked(
        frame,
        frame,
        std::slice::from_ref(&command),
        &ImageStore::new(),
        &vec![0xff0000; 262_144],
    );
    route(&plan, 1, 1_572_880, 786_440, 262_144);
    assert_eq!(
        (
            plan.stats().generated_gradient_sources,
            plan.stats().generated_gradient_bytes
        ),
        (1, 1_048_576)
    );
    let mut images = ImageStore::new();
    images.insert(
        "extra".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        }),
    );
    assert_eq!(
        prepare(frame, &[command], &images).unwrap_err(),
        at(FallbackKind::PlannerLimit, 0, 0)
    );
    let frame = Frame::new(4, 2, 0xffffff);
    let area = rect(0.0, 0.0, 1.0, 1.0);
    let command = boxed(area, area, [true; 2], red());
    let mut images = ImageStore::new();
    for index in 0..255 {
        images.insert(
            format!("loaded{index}"),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
            }),
        );
    }
    let plan = checked(
        frame,
        frame,
        std::slice::from_ref(&command),
        &images,
        &pixels(&["RWWW", "WWWW"]),
    );
    route(&plan, 1, 22, 11, 1);
    assert_eq!(plan.stats().bridge.unique_sources, 255);
    assert_eq!(plan.stats().generated_gradient_sources, 1);
    images.insert(
        "last".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        }),
    );
    assert_eq!(
        prepare(frame, &[command], &images).unwrap_err(),
        at(FallbackKind::PlannerLimit, 0, 0)
    );
    let frame = Frame::new(1280, 1024, 0xffffff);
    let area = rect(0.0, 0.0, 1280.0, 1024.0);
    let command = boxed(area, area, [true; 2], red());
    assert_eq!(
        prepare(frame, &[command], &ImageStore::new()).unwrap_err(),
        at(FallbackKind::PlannerLimit, 0, 0)
    );
}
