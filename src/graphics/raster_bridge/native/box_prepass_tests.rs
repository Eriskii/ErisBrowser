//! Independent prepass funding and diagnostic-order regressions.
//! Literal allowances were frozen externally before execution.
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use crate::graphics::{Color, Rect as BrowserRect};
use std::sync::Arc;

fn box_command(width: f32, height: f32, radii: [[f32; 2]; 4]) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            radii,
        },
        cutout: None,
        paint: BoxPaint::Color(Color::BLACK),
    }))
}

#[test]
fn earlier_rect_debt_precedes_box_classification_and_materialization_reservation() {
    let target = Frame::new(4, 2, 0xffffff);
    let commands = [
        DrawCommand::Rect {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            color: Color::BLACK,
            radius: 0.0,
        },
        box_command(4.0, 2.0, [[2.0, 1.0]; 4]),
    ];
    let phases = [NativePhase {
        frame: target,
        commands: &commands,
    }];
    let counts = preflight(target, &phases).unwrap();
    assert_eq!(counts.original, 2);
    assert_eq!(counts.full_area_commands, 0);

    // P=8, A=4, S=1, M=8, R=2: stages 16 and 90.
    // The ordinary Rect costs 1 before either stage may start.
    for (initial, expected_remaining) in [(16, 15), (106, 89), (107, 0)] {
        let mut remaining = initial;
        let mut gradients = GradientReservation::new();
        let mut boxes = Vec::new();
        boxes.resize_with(2, || None);
        let result =
            geometry_pixel_work(target, &phases, &mut gradients, &mut boxes, &mut remaining);
        assert_eq!(remaining, expected_remaining);
        assert!(boxes[0].is_none());
        if initial == 107 {
            assert_eq!(result.unwrap(), 107);
            let BoxReservation::Mask(reservation) = boxes[1].as_ref().unwrap() else {
                panic!("ellipse must retain its funded mask reservation");
            };
            let info = reservation.info();
            assert_eq!(
                (
                    info.candidate_pixels,
                    info.ambiguous_pixels,
                    info.shape_count
                ),
                (8, 4, 1)
            );
            assert_eq!(
                (info.classification_work, info.materialization_work),
                (16, 90)
            );
            assert_eq!(
                (info.tile_count, info.coverage_bytes, info.row_entries),
                (1, 8, 2)
            );
        } else {
            assert_eq!(result.unwrap_err(), at(FallbackKind::CpuPaintBudget, 0, 1));
            assert!(boxes[1].is_none());
        }
    }
}

fn opacity_commands(first: DrawCommand, pairs: usize) -> Vec<DrawCommand> {
    let mut commands = vec![first];
    for _ in 0..pairs {
        commands.push(DrawCommand::PushOpacity { opacity: 1.0 });
        commands.push(DrawCommand::PopOpacity);
    }
    commands
}

#[test]
fn full_area_opacity_debt_is_global_before_any_box_window_preparation() {
    // A=264000, allowance=16*A=4224000. Keep Box first in original order.
    let target = Frame::new(1000, 264, 0xffffff);
    let images = ImageStore::new();
    let fonts = Fonts::new();
    let prepare = |commands: &[DrawCommand]| {
        plan_native_scene(
            target,
            &[NativePhase {
                frame: target,
                commands,
            }],
            &images,
            &fonts,
        )
    };

    let exact = opacity_commands(box_command(0.0, 0.0, [[0.0; 2]; 4]), 8);
    let plan = prepare(&exact).unwrap();
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 4_224_000);
    assert_eq!(plan.stats().box_masks, 0);
    assert_eq!(plan.stats().box_loop_work, 0);
    assert_eq!(plan.stats().mask_sources, 0);

    let no_remainder = opacity_commands(box_command(4.0, 2.0, [[2.0, 1.0]; 4]), 8);
    assert_eq!(
        prepare(&no_remainder).unwrap_err(),
        at(FallbackKind::CpuPaintBudget, 0, 0)
    );

    // The exact original oversized shape now selects the core route, but the
    // sixteen opacity markers leave no work for its first debit. All samples stay inside.
    // An extra balanced pair fails globally before visiting the first Box command.
    let mask_short = opacity_commands(box_command(1000.0, 264.0, [[0.125; 2]; 4]), 8);
    assert_eq!(
        prepare(&mask_short).unwrap_err(),
        at(FallbackKind::CpuPaintBudget, 0, 0)
    );
    // Separate forced-mask control retains the old cap-before-funding refusal.
    let mut forced_mask = box_command(1000.0, 264.0, [[0.125; 2]; 4]);
    let DrawCommand::BoxDecoration(decoration) = &mut forced_mask else {
        unreachable!();
    };
    Arc::make_mut(decoration).cutout = Some(RoundedBox {
        rect: BrowserRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
        radii: [[0.0; 2]; 4],
    });
    assert_eq!(
        prepare(&opacity_commands(forced_mask, 8)).unwrap_err(),
        at(FallbackKind::MaskPreparation, 0, 0)
    );
    let aggregate_short = opacity_commands(box_command(1000.0, 264.0, [[0.125; 2]; 4]), 9);
    assert_eq!(
        prepare(&aggregate_short).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None)
    );
}
