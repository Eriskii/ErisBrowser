//! Native core admission controls frozen before production or execution.
//! Pixel/proof/allocation oracles live in the separate public/helper groups.
use super::*;
use crate::graphics::Color;
use crate::graphics::box_decoration::{BoxCoreReservation, BoxCoreWindow, RoundedBox};
use std::sync::Arc;

fn rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}
fn shape(area: BrowserRect, radii: [[f32; 2]; 4]) -> RoundedBox {
    RoundedBox { rect: area, radii }
}
fn primary() -> RoundedBox {
    shape(rect(0.0, 0.0, 512.0, 513.0), [[1.0; 2]; 4])
}
fn color_box(outer: RoundedBox, cutout: Option<RoundedBox>, color: Color) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: outer,
        cutout,
        paint: BoxPaint::Color(color),
    }))
}
fn ordinary(area: BrowserRect, color: Color) -> DrawCommand {
    DrawCommand::Rect {
        rect: area,
        color,
        radius: 0.0,
    }
}
fn reservation(frame: Frame, outer: RoundedBox, work: u64) -> BoxCoreReservation {
    let mut remaining = work;
    let result = BoxCoreWindow::new(
        frame.width,
        frame.height,
        outer,
        None,
        rect(0.0, 0.0, frame.width as f32, frame.height as f32),
    )
    .unwrap()
    .expect("selected oversized core")
    .reserve(&mut remaining)
    .unwrap();
    assert_eq!(remaining, 0);
    result
}
fn primary_reservation() -> BoxCoreReservation {
    reservation(Frame::new(512, 513, 0xffffff), primary(), 791_114)
}
fn ledger(work: u64) -> Ledger {
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
        cpu_pixels: work,
        cpu_limit: work,
        gpu_upper: 0,
        rounded_loop_work: 0,
        box_loop_work: 0,
    }
}
fn prepare(target: Frame, frame: Frame, commands: &[DrawCommand]) -> NativeResult<NativeScenePlan> {
    plan_native_scene(
        target,
        &[NativePhase { frame, commands }],
        &ImageStore::new(),
        &Fonts::new(),
    )
}

#[test]
fn native_box_core_complete_batch_obeys_six_shared_exact_limits_atomically() {
    let reservation = primary_reservation();
    assert_eq!(PARAM_STRIDE, 256);
    // Four masks: E2046/R1024 =>12280 input bytes. Five paint commands
    // plus clear use1536 parameter bytes. GPU allowance is not enlarged.
    for boundary in 0..6 {
        let mut exact = ledger(791_114);
        match boundary {
            0 => exact.operations = 252,
            1 => exact.sources = 252,
            2 => exact.coverage = 260_098,
            3 => exact.rows = 64_512,
            4 => exact.row_tables = 252,
            _ => exact.targets = 16_763_400,
        }
        let mut over = exact;
        match boundary {
            0 => over.operations += 1,
            1 => over.sources += 1,
            2 => over.coverage += 1,
            3 => over.rows += 1,
            4 => over.row_tables += 1,
            _ => over.targets += 1,
        }
        exact.box_core(&reservation, 2, 7).unwrap();
        match boundary {
            0 => assert_eq!(exact.operations, 256),
            1 => assert_eq!(exact.sources, 256),
            2 => assert_eq!(exact.coverage, 262_144),
            3 => assert_eq!(exact.rows, 65_536),
            4 => assert_eq!(exact.row_tables, 256),
            _ => assert_eq!(exact.gpu_upper, 16_777_216),
        }
        assert_eq!(exact.box_loop_work, 791_114);
        assert_eq!(
            exact.cpu_pixels, 791_114,
            "prepaid geometry is not charged twice"
        );
        assert_eq!(exact.masks, 4);
        let before = over;
        let kind = match boundary {
            0 | 4 => FallbackKind::CommandLimit,
            1 | 2 => FallbackKind::MaskPreparation,
            3 => FallbackKind::GlyphRowLimit,
            _ => FallbackKind::PlannerLimit,
        };
        assert_eq!(
            over.box_core(&reservation, 2, 7).unwrap_err(),
            at(kind, 2, 7)
        );
        assert_eq!(
            over, before,
            "entire batch remains unpublished at boundary {boundary}"
        );
    }
}

#[test]
fn native_box_core_uses_the_existing_mask_ledger_and_preserves_prior_admission() {
    let reservation = primary_reservation();
    let mut paid = ledger(791_114);
    paid.operations = 2; // prior paint plus the original core command
    // Synthetic cumulative resource state through the existing common seam.
    paid.mask(
        260_098,
        64_512,
        true,
        false,
        at(FallbackKind::MaskPreparation, 0, 0),
    )
    .unwrap();
    paid.box_core(&reservation, 0, 1).unwrap();
    assert_eq!(
        (
            paid.sources,
            paid.masks,
            paid.coverage,
            paid.rows,
            paid.row_tables
        ),
        (5, 5, 262_144, 65_536, 5)
    );
    assert_eq!(paid.operations, 6);
    assert_eq!(paid.box_loop_work, 791_114);
    let before = paid;
    assert_eq!(
        paid.box_core(&reservation, 0, 2).unwrap_err(),
        at(FallbackKind::GlyphRowLimit, 0, 2)
    );
    assert_eq!(paid, before);
}

#[test]
fn native_box_core_zero_and_seven_tile_batches_count_the_rect_exactly_once() {
    for (frame, outer, work, k, coverage, rows) in [
        (
            Frame::new(512, 513, 0xffffff),
            shape(rect(-2.0, -2.0, 516.0, 517.0), [[1.0; 2]; 4]),
            787_980,
            0,
            0,
            0,
        ),
        (
            Frame::new(1280, 205, 0xffffff),
            shape(
                rect(0.0, 0.0, 1280.0, 205.0),
                [[1100.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1100.0, 1.0]],
            ),
            1_032_096,
            7,
            226_063,
            613,
        ),
    ] {
        let reservation = reservation(frame, outer, work);
        let mut paid = ledger(work);
        paid.check_gpu(at(FallbackKind::PlannerLimit, 0, 0))
            .unwrap();
        paid.box_core(&reservation, 0, 0).unwrap();
        assert_eq!(
            (paid.operations, paid.sources, paid.masks, paid.row_tables),
            (1 + k, k, k, k)
        );
        assert_eq!(
            (
                paid.coverage,
                paid.rows,
                paid.cpu_pixels,
                paid.box_loop_work
            ),
            (coverage, rows, work, work)
        );
        if k == 0 {
            assert_eq!(paid.gpu_upper, 512);
        }
        let commands = [color_box(outer, None, Color::BLACK)];
        let result = prepare(frame, frame, &commands).unwrap();
        assert_eq!(
            (
                result.stats().box_cores,
                result.stats().box_rectangles,
                result.stats().box_masks
            ),
            (1, 0, k)
        );
        assert_eq!(result.stats().bridge.lowered_commands, 1 + k);
    }
}

#[test]
fn native_box_core_original_order_cpu_debt_precedes_both_prepaid_stages() {
    let target = Frame::new(512, 513, 0xffffff);
    let commands = [
        ordinary(rect(0.0, 0.0, 1.0, 1.0), Color::BLACK),
        color_box(primary(), None, Color::BLACK),
    ];
    let phases = [NativePhase {
        frame: target,
        commands: &commands,
    }];
    assert_eq!(preflight(target, &phases).unwrap().original, 2);
    for (initial, left) in [(262_668, 262_667), (791_114, 528_445), (791_115, 0)] {
        let mut remaining = initial;
        let mut gradients = GradientReservation::new();
        let mut boxes = Vec::new();
        boxes.resize_with(2, || None);
        let result =
            geometry_pixel_work(target, &phases, &mut gradients, &mut boxes, &mut remaining);
        assert_eq!(remaining, left);
        assert!(boxes[0].is_none());
        if initial == 791_115 {
            assert_eq!(result.unwrap(), 791_115);
            let BoxReservation::Core {
                reservation,
                raw_core,
            } = boxes[1].as_ref().unwrap()
            else {
                panic!("oversized Color must retain its funded core reservation");
            };
            assert_eq!(*raw_core, rect(1.0, 1.0, 510.0, 511.0));
            assert_eq!(
                (
                    reservation.info().classification_work,
                    reservation.info().materialization_work
                ),
                (262_668, 528_446)
            );
        } else {
            assert_eq!(result.unwrap_err(), at(FallbackKind::CpuPaintBudget, 0, 1));
            assert!(boxes[1].is_none());
        }
    }
}

#[test]
fn native_box_core_full_scene_cpu_allowance_keeps_transparent_rect_debt() {
    let frame = Frame::new(512, 513, 0xffffff);
    let mut commands = vec![color_box(primary(), None, Color::BLACK)];
    for _ in 0..12 {
        commands.push(ordinary(rect(0.0, 0.0, 512.0, 513.0), Color::TRANSPARENT));
    }
    commands.push(ordinary(rect(0.0, 0.0, 505.0, 513.0), Color::TRANSPARENT));
    commands.push(ordinary(rect(0.0, 0.0, 445.0, 1.0), Color::TRANSPARENT));
    //791114 +12*262656 +259065 +445 =4202496 =16*frame_area.
    let result = prepare(frame, frame, &commands).unwrap();
    assert_eq!(result.stats().cpu_pixel_upper_bound, 4_202_496);
    assert_eq!(result.stats().box_loop_work, 791_114);
    assert_eq!(
        (
            result.stats().box_cores,
            result.stats().box_masks,
            result.stats().box_rectangles
        ),
        (1, 4, 0)
    );
    assert_eq!(result.stats().bridge.lowered_commands, 19);
    commands.push(ordinary(rect(0.0, 0.0, 1.0, 1.0), Color::TRANSPARENT));
    assert_eq!(
        prepare(frame, frame, &commands).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
}

#[test]
fn native_box_core_preserves_small_mask_route_and_suppressed_reservations() {
    let small = Frame::new(512, 512, 0xffffff);
    let commands = [color_box(
        shape(rect(0.0, 0.0, 512.0, 512.0), [[1.0; 2]; 4]),
        None,
        Color::BLACK,
    )];
    let old = prepare(small, small, &commands).unwrap();
    assert_eq!(
        (
            old.stats().box_cores,
            old.stats().box_masks,
            old.stats().coverage_bytes,
            old.stats().row_entries
        ),
        (0, 1, 262_144, 512)
    );
    assert_eq!(old.stats().box_loop_work, 1_049_160);
    let frame = Frame::new(512, 513, 0xffffff);
    for opacity in [false, true] {
        let commands = if opacity {
            vec![
                DrawCommand::PushOpacity { opacity: 0.0 },
                color_box(primary(), None, Color::BLACK),
                DrawCommand::PopOpacity,
            ]
        } else {
            vec![color_box(primary(), None, Color::TRANSPARENT)]
        };
        let result = prepare(frame, frame, &commands).unwrap();
        assert_eq!(
            (
                result.stats().box_cores,
                result.stats().box_rectangles,
                result.stats().box_masks
            ),
            (1, 0, 4)
        );
        assert_eq!(
            (
                result.stats().coverage_bytes,
                result.stats().row_entries,
                result.stats().box_loop_work
            ),
            (2_046, 1_024, 791_114)
        );
        assert_eq!(
            result.stats().cpu_pixel_upper_bound,
            if opacity { 1_316_426 } else { 791_114 }
        );
    }
}

#[test]
fn native_box_core_raw_offset_proof_and_unselected_shapes_keep_refusals() {
    let target = Frame::new(512, 513, 0xffffff);
    for fixed in [false, true] {
        let mut phase = target;
        phase.document_offset = (3.0, 5.0);
        phase.viewport_offset = (7.0, 9.0);
        let (x, y) = if fixed { (-7.0, -9.0) } else { (-3.0, -5.0) };
        let paint = color_box(
            shape(rect(x, y, 512.0, 513.0), [[1.0; 2]; 4]),
            None,
            Color::BLACK,
        );
        let commands = if fixed {
            vec![DrawCommand::PushFixed, paint, DrawCommand::PopFixed]
        } else {
            vec![paint]
        };
        let result = prepare(target, phase, &commands).unwrap();
        assert_eq!(result.stats().box_cores, 1);
        assert_eq!(result.stats().box_loop_work, 791_114);
    }
    let zero = shape(rect(0.0, 0.0, 0.0, 0.0), [[0.0; 2]; 4]);
    let mut fractional_phase = target;
    fractional_phase.document_offset = (0.25, 0.75);
    let mut limit_phase = target;
    limit_phase.document_offset = (-1_000_000.0, 0.0);
    let full = Frame::new(1280, 1024, 0xffffff);
    for (frame, phase, outer, cutout) in [
        (target, target, primary(), Some(zero)),
        (
            target,
            fractional_phase,
            shape(rect(-0.25, -0.75, 512.0, 513.0), [[1.0; 2]; 4]),
            None,
        ),
        (
            target,
            limit_phase,
            shape(rect(1_000_000.0, 0.0, 512.0, 513.0), [[1.0; 2]; 4]),
            None,
        ),
        (
            target,
            target,
            shape(rect(0.0, 0.0, 512.0, 513.0), [[256.0, 256.5]; 4]),
            None,
        ),
        (
            full,
            full,
            shape(rect(0.0, 0.0, 1280.0, 1024.0), [[400.0; 2]; 4]),
            None,
        ),
    ] {
        let commands = [color_box(outer, cutout, Color::TRANSPARENT)];
        assert_eq!(
            prepare(frame, phase, &commands).unwrap_err(),
            at(FallbackKind::MaskPreparation, 0, 0)
        );
    }
    let commands = [color_box(
        shape(rect(1_000_001.0, 0.0, 512.0, 513.0), [[1.0; 2]; 4]),
        None,
        Color::TRANSPARENT,
    )];
    assert_eq!(
        prepare(target, target, &commands).unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 0)
    );
}

#[test]
fn native_box_core_later_raw_and_image_eligibility_checks_still_precede_color_work() {
    let frame = Frame::new(512, 513, 0xffffff);
    let head = color_box(primary(), None, Color::BLACK);
    let image = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: primary(),
        cutout: None,
        paint: BoxPaint::Image {
            tile: rect(0.0, 0.0, 1.0, 1.0),
            repeat: [true; 2],
            key: "missing".into(),
        },
    }));
    assert_eq!(
        prepare(frame, frame, &[head.clone(), image]).unwrap_err(),
        at(FallbackKind::UnsupportedBoxDecoration, 0, 1)
    );
    assert_eq!(
        prepare(
            frame,
            frame,
            &[
                head.clone(),
                ordinary(rect(f32::NAN, 0.0, 1.0, 1.0), Color::TRANSPARENT)
            ]
        )
        .unwrap_err(),
        at(FallbackKind::InvalidGeometry, 0, 1)
    );
    let mut images = ImageStore::new();
    images.insert(
        "poison".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: Vec::new(),
        }),
    );
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &[head]
            }],
            &images,
            &Fonts::new()
        )
        .unwrap_err(),
        NativeFallback::new(FallbackKind::InvalidImage, None, None)
    );
}

#[test]
fn native_box_core_adapter_types_are_observed_without_guessed_stack_sizes() {
    use std::mem::{align_of, size_of};
    println!(
        "BOX_CORE_NATIVE BoxReservation bytes={} align={}",
        size_of::<BoxReservation>(),
        align_of::<BoxReservation>()
    );
    println!(
        "BOX_CORE_NATIVE OptionalBoxReservation bytes={} align={}",
        size_of::<Option<BoxReservation>>(),
        align_of::<Option<BoxReservation>>()
    );
    println!(
        "BOX_CORE_NATIVE NativeSceneStats bytes={} align={}",
        size_of::<NativeSceneStats>(),
        align_of::<NativeSceneStats>()
    );
    assert!(size_of::<Option<BoxReservation>>() >= size_of::<BoxReservation>());
}
