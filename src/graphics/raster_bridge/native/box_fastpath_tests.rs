//! Integer Color-box admission: independent literal pixels and work cuts.
use super::box_decoration_tests::decode_opacity_pixels;
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
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
fn square(x: f32, y: f32, width: f32, height: f32) -> RoundedBox {
    RoundedBox {
        rect: rect(x, y, width, height),
        radii: [[0.0; 2]; 4],
    }
}
fn decoration(shape: RoundedBox, cutout: Option<RoundedBox>, color: Color) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape,
        cutout,
        paint: BoxPaint::Color(color),
    }))
}
fn checked(
    target: Frame,
    phase: Frame,
    commands: &[DrawCommand],
    expected: &[u32],
) -> NativeScenePlan {
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let phases = [NativePhase {
        frame: phase,
        commands,
    }];
    let plan = plan_native_scene(target, &phases, &images, &fonts).unwrap();
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
        &images,
        phase.document_offset,
        phase.viewport_offset,
    );
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    assert_eq!(decode_opacity_pixels(plan.plan()), expected);
    assert!(fonts.cache.borrow().is_empty());
    plan
}
fn no_masks(plan: &NativeScenePlan, rectangles: usize, work: u64) {
    let s = plan.stats();
    assert_eq!(s.box_rectangles, rectangles);
    assert_eq!(s.box_loop_work, work);
    assert_eq!(
        (s.box_masks, s.mask_sources, s.coverage_bytes, s.row_entries),
        (0, 0, 0, 0)
    );
    assert!(plan.plan().input_bytes().is_empty());
    assert!(!plan.plan().has_glyphs());
}

#[test]
fn native_box_fastpath_integer_and_zero_axis_corners_translate_once() {
    let target = Frame::new(4, 3, 0xffffff);
    let expected = [
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000, 0xff0000, 0xffffff, 0xffffff,
        0xffffff, 0xffffff, 0xffffff,
    ];
    for radii in [
        [[0.0; 2]; 4],
        [[0.0, 999.0], [999.0, 0.0], [0.0, 0.0], [1.0, 0.0]],
    ] {
        for translated in [false, true] {
            let mut phase = target;
            let mut shape = if translated {
                square(0.25, 0.25, 2.0, 1.0)
            } else {
                square(1.0, 1.0, 2.0, 1.0)
            };
            shape.radii = radii;
            if translated {
                phase.document_offset = (0.75, 0.75);
            }
            let plan = checked(
                target,
                phase,
                &[decoration(shape, None, Color::rgb(255, 0, 0))],
                &expected,
            );
            no_masks(&plan, 1, 14);
            assert_eq!(plan.plan().draws().len(), 2);
            assert_eq!(plan.plan().draws()[1].kind(), DrawKind::Rectangle);
            assert_eq!(plan.plan().draws()[1].bounds(), (1, 1, 2, 1));
        }
    }
}

#[test]
fn native_box_fastpath_fractional_clip_keeps_integer_cell_membership() {
    let target = Frame::new(4, 2, 0xffffff);
    let mut phase = target;
    phase.caller_clip = Rect::new(0.5, 0.0, 2.5, 2.0);
    let plan = checked(
        target,
        phase,
        &[decoration(square(-1.0, 0.0, 5.0, 2.0), None, Color::BLACK)],
        &[0xffffff, 0, 0, 0xffffff, 0xffffff, 0, 0, 0xffffff],
    );
    no_masks(&plan, 1, 20);
    assert_eq!(plan.plan().draws()[1].bounds(), (1, 0, 2, 2));
    let target = Frame::new(6, 3, 0xffffff);
    let mut phase = target;
    phase.caller_clip = Rect::new(0.5, 0.25, 3.0, 1.5);
    let plan = checked(
        target,
        phase,
        &[decoration(square(0.0, 0.0, 6.0, 3.0), None, Color::BLACK)],
        &[
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0, 0, 0,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
        ],
    );
    no_masks(&plan, 1, 17);
    assert_eq!(plan.plan().draws()[1].bounds(), (1, 1, 3, 1));
}

#[test]
fn native_box_fastpath_fractional_outer_retains_literal_coverage_mask() {
    let target = Frame::new(2, 2, 0xffffff);
    let plan = checked(
        target,
        target,
        &[decoration(
            square(0.25, 0.25, 1.0, 1.0),
            None,
            Color::rgb(0, 0, 255),
        )],
        &[0x7070ff, 0xd0d0ff, 0xd0d0ff, 0xf0f0ff],
    );
    assert_eq!(plan.stats().box_rectangles, 0);
    assert_eq!(
        (
            plan.stats().box_masks,
            plan.stats().coverage_bytes,
            plan.stats().row_entries
        ),
        (1, 4, 2)
    );
    assert_eq!(plan.stats().box_loop_work, 90);
    assert_eq!(plan.plan().draws()[1].kind(), DrawKind::Glyph);
}

#[test]
fn native_box_fastpath_alpha_and_nested_opacity_keep_pixels_and_work() {
    let target = Frame::new(4, 1, 0xffffff);
    let shape = square(1.0, 0.0, 2.0, 1.0);
    let red = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 128,
    };
    let plan = checked(
        target,
        target,
        &[decoration(shape, None, red)],
        &[0xffffff, 0xff7f7f, 0xff7f7f, 0xffffff],
    );
    no_masks(&plan, 1, 14);
    let nested = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        DrawCommand::PushOpacity { opacity: 0.5 },
        decoration(shape, None, Color::rgb(0, 0, 255)),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let plan = checked(
        target,
        target,
        &nested,
        &[0xffffff, 0xbfbfff, 0xbfbfff, 0xffffff],
    );
    no_masks(&plan, 1, 14);
    assert_eq!(
        plan.plan()
            .draws()
            .iter()
            .filter(|d| d.kind() == DrawKind::GroupComposite)
            .count(),
        2
    );
    for commands in [
        vec![decoration(shape, None, Color::TRANSPARENT)],
        vec![
            DrawCommand::PushOpacity { opacity: 0.0 },
            decoration(shape, None, Color::BLACK),
            DrawCommand::PopOpacity,
        ],
    ] {
        let plan = checked(target, target, &commands, &[0xffffff; 4]);
        no_masks(&plan, 1, 14);
        assert!(plan.stats().cpu_pixel_upper_bound >= 14);
    }
}

#[test]
fn native_box_fastpath_fixed_scope_restores_document_clip() {
    let target = Frame::new(5, 2, 0xffffff);
    let mut phase = target;
    phase.document_offset = (1.0, 0.0);
    let commands = [
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 1.0, 1.0),
        },
        decoration(square(0.0, 0.0, 2.0, 1.0), None, Color::rgb(255, 0, 0)),
        DrawCommand::PushFixed,
        decoration(square(3.0, 0.0, 1.0, 1.0), None, Color::rgb(0, 0, 255)),
        DrawCommand::PopFixed,
        decoration(square(0.0, 0.0, 2.0, 1.0), None, Color::rgb(0, 255, 0)),
        DrawCommand::PopClip,
        DrawCommand::Rect {
            rect: rect(0.0, 1.0, 1.0, 1.0),
            color: Color::BLACK,
            radius: 0.0,
        },
    ];
    let plan = checked(
        target,
        phase,
        &commands,
        &[
            0xffffff, 0x00ff00, 0xffffff, 0x0000ff, 0xffffff, 0xffffff, 0, 0xffffff, 0xffffff,
            0xffffff,
        ],
    );
    no_masks(&plan, 3, 33);
}

#[test]
fn native_box_fastpath_full_viewport_uses_two_rectangles_without_mask_payload() {
    let target = Frame::new(1280, 1024, 0xffffff);
    let plan = checked(
        target,
        target,
        &[decoration(
            square(0.0, 0.0, 1280.0, 1024.0),
            None,
            Color::rgb(0, 0, 255),
        )],
        &vec![0x0000ff; 1_310_720],
    );
    no_masks(&plan, 1, 3_932_168);
    assert_eq!(plan.stats().cpu_pixel_upper_bound, 3_932_168);
    assert_eq!(plan.plan().draws().len(), 2);
    for draw in plan.plan().draws() {
        assert_eq!(draw.kind(), DrawKind::Rectangle);
        assert_eq!(draw.bounds(), (0, 0, 1280, 1024));
    }
    assert_eq!(plan.plan().invocations(), 3_932_160);
}

#[test]
fn native_box_fastpath_keeps_true_rounding_and_every_cutout_on_masks() {
    let target = Frame::new(4, 2, 0xffffff);
    let shape = RoundedBox {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        radii: [[2.0, 1.0]; 4],
    };
    let plan = checked(
        target,
        target,
        &[decoration(shape, None, Color::rgb(0, 0, 255))],
        &[
            0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff, 0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff,
        ],
    );
    assert_eq!(
        (
            plan.stats().box_rectangles,
            plan.stats().box_masks,
            plan.stats().box_loop_work
        ),
        (0, 1, 106)
    );
    let target = Frame::new(2, 2, 0xffffff);
    let plan = checked(
        target,
        target,
        &[decoration(
            square(0.0, 0.0, 2.0, 2.0),
            Some(square(0.25, 0.25, 1.0, 1.0)),
            Color::BLACK,
        )],
        &[0x909090, 0x303030, 0x303030, 0x101010],
    );
    assert_eq!(
        (
            plan.stats().box_rectangles,
            plan.stats().box_masks,
            plan.stats().box_loop_work
        ),
        (0, 1, 162)
    );
    let target = Frame::new(2, 1, 0xffffff);
    for cutout in [square(0.0, 0.0, 0.0, 0.0), square(20.0, 0.0, 1.0, 1.0)] {
        let plan = checked(
            target,
            target,
            &[decoration(
                square(0.0, 0.0, 2.0, 1.0),
                Some(cutout),
                Color::rgb(0, 0, 255),
            )],
            &[0x0000ff; 2],
        );
        assert_eq!(
            (plan.stats().box_rectangles, plan.stats().box_masks),
            (0, 1)
        );
    }
}

#[test]
fn native_box_fastpath_empty_eligible_commands_are_counted_without_draws() {
    let target = Frame::new(4, 3, 0xffffff);
    for shape in [square(1.0, 1.0, 0.0, 1.0), square(5.0, 0.0, 2.0, 1.0)] {
        let plan = checked(
            target,
            target,
            &[decoration(shape, None, Color::BLACK)],
            &[0xffffff; 12],
        );
        no_masks(&plan, 1, 0);
        assert_eq!(plan.stats().cpu_pixel_upper_bound, 0);
        assert_eq!(plan.plan().draws().len(), 1);
    }
}

#[test]
fn native_box_fastpath_exact_prepass_debit_is_atomic_and_keeps_original_index() {
    let target = Frame::new(4, 3, 0xffffff);
    let commands = [decoration(square(1.0, 1.0, 2.0, 1.0), None, Color::BLACK)];
    let phases = [NativePhase {
        frame: target,
        commands: &commands,
    }];
    preflight(target, &phases).unwrap();
    for initial in [13, 14] {
        let mut remaining = initial;
        let mut gradients = GradientReservation::new();
        let mut boxes = vec![None];
        let result =
            geometry_pixel_work(target, &phases, &mut gradients, &mut boxes, &mut remaining);
        if initial == 14 {
            assert_eq!(result.unwrap(), 14);
            assert_eq!(remaining, 0);
            assert!(matches!(
                boxes[0].as_ref(),
                Some(BoxReservation::Rectangle { work: 14 })
            ));
        } else {
            assert_eq!(result.unwrap_err(), at(FallbackKind::CpuPaintBudget, 0, 0));
            assert_eq!(remaining, 13);
            assert!(boxes[0].is_none());
        }
    }
}

#[test]
fn native_box_fastpath_validates_hidden_shape_and_refuses_other_paints_first() {
    let target = Frame::new(4, 3, 0xffffff);
    let images = ImageStore::new();
    let fonts = Fonts::new();
    let eligible = decoration(square(0.0, 0.0, 1.0, 1.0), None, Color::BLACK);
    for value in [f32::NAN, f32::INFINITY, -1.0, 1_000_001.0] {
        let mut invalid = square(5.0, 0.0, 0.0, 1.0);
        invalid.radii[0][0] = value;
        let commands = [
            eligible.clone(),
            DrawCommand::PushOpacity { opacity: 0.0 },
            decoration(invalid, None, Color::TRANSPARENT),
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
            at(FallbackKind::InvalidGeometry, 0, 2)
        );
    }
    let paint = BoxPaint::Image {
        tile: rect(0.0, 0.0, 1.0, 1.0),
        repeat: [true, true],
        key: "missing".into(),
    };
    let gradient = Arc::new(crate::graphics::gradient::LinearGradient {
        direction: crate::graphics::gradient::GradientDirection::Angle(0.0),
        stops: Arc::from([crate::graphics::gradient::GradientStop {
            color: crate::graphics::gradient::GradientColor::Color(Color::BLACK),
            position: None,
        }]),
    });
    for paint in [
        paint,
        BoxPaint::LinearGradient {
            tile: rect(0.0, 0.0, 1.0, 1.0),
            repeat: [true, true],
            gradient,
            current_color: Color::BLACK,
            stop_scale: 1.0,
        },
    ] {
        let commands = [
            eligible.clone(),
            DrawCommand::PushOpacity { opacity: 0.0 },
            DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: square(5.0, 0.0, 0.0, 1.0),
                cutout: None,
                paint,
            })),
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
            at(FallbackKind::UnsupportedBoxDecoration, 0, 2)
        );
    }
}
