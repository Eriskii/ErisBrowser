//! Literal Line work/pixel controls frozen independently before producer authoring.
//! The packed-plan decoder is a CPU test helper, not shader or GPU execution.
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};

fn line(geometry: [f32; 5], color: Color) -> DrawCommand {
    let [x1, y1, x2, y2, width] = geometry;
    DrawCommand::Line {
        x1,
        y1,
        x2,
        y2,
        width,
        color,
    }
}

fn literal_pixels(size: usize, entries: &[(usize, u32)]) -> Vec<u32> {
    let mut pixels = vec![0xffffff; size];
    for &(index, pixel) in entries {
        pixels[index] = pixel;
    }
    pixels
}

fn checked_scene(
    target: Frame,
    phases: &[NativePhase<'_>],
    work: u64,
    pixels: &[u32],
) -> NativeScenePlan {
    let images = ImageStore::new();
    let fonts = Fonts::new();
    let scene = plan_native_scene(target, phases, &images, &fonts).unwrap();
    assert_eq!(scene.stats().cpu_pixel_upper_bound, work);
    assert_eq!(decode_pixels(scene.plan()), pixels);
    assert_eq!(canvas_pixels(target, phases, &images, &fonts), pixels);
    assert_eq!(
        (
            scene.stats().rounded_masks,
            scene.stats().rounded_loop_work,
            scene.stats().mask_sources,
            scene.stats().coverage_bytes,
            scene.stats().row_entries,
            scene.stats().text_input_bytes
        ),
        (0, 0, 0, 0, 0, 0),
    );
    scene
}

fn same_packed_plan(left: &Plan, right: &Plan) {
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

#[test]
fn native_line_accounting_forty_small_lines_are_960_pixels() {
    let target = Frame::new(512, 512, 0xffffff);
    let commands: Vec<_> = (0..40)
        .map(|y| line([0.0, y as f32, 24.0, y as f32, 1.0], Color::rgb(255, 0, 0)))
        .collect();
    let mut expected = vec![0xffffff; 512 * 512];
    for y in 0..40 {
        expected[y * 512..y * 512 + 24].fill(0xff0000);
    }
    // Forty disjoint 24x1 rectangles: 960 visits; old 40*512*512 = 10,485,760.
    let scene = checked_scene(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        960,
        &expected,
    );
    assert_eq!(scene.stats().bridge.original_commands, 40);
    assert_eq!(scene.stats().bridge.lowered_commands, 40);
    assert_eq!(scene.plan().draws().len(), 41);
}

#[test]
// Keep the exact binary32 decimal values used by the independent oracle.
#[allow(clippy::excessive_precision)]
fn native_line_accounting_literal_geometry_and_rect_plans() {
    struct Case {
        name: &'static str,
        line: [f32; 5],
        rect: [f32; 4],
        work: u64,
        red: &'static [usize],
    }
    let cases = [
        Case {
            name: "horizontal",
            line: [1.0, 1.0, 3.0, 1.0, 1.0],
            rect: [1.0, 1.0, 2.0, 1.0],
            work: 2,
            red: &[5, 6],
        },
        Case {
            name: "reversed",
            line: [3.0, 1.0, 1.0, 1.0, 1.0],
            rect: [1.0, 1.0, 2.0, 1.0],
            work: 2,
            red: &[5, 6],
        },
        Case {
            name: "vertical",
            line: [1.0, 1.0, 1.0, 3.0, 1.0],
            rect: [1.0, 1.0, 1.0, 2.0],
            work: 2,
            red: &[5, 9],
        },
        Case {
            name: "diagonal_bounds",
            line: [1.0, 1.0, 3.0, 3.0, 0.0],
            rect: [1.0, 1.0, 2.0, 2.0],
            work: 4,
            red: &[5, 6, 9, 10],
        },
        Case {
            name: "point_width",
            line: [1.0, 1.0, 1.0, 1.0, 2.0],
            rect: [1.0, 1.0, 2.0, 2.0],
            work: 4,
            red: &[5, 6, 9, 10],
        },
        Case {
            name: "minimum_width",
            line: [1.0, 1.0, 2.0, 1.0, 2.0],
            rect: [1.0, 1.0, 2.0, 2.0],
            work: 4,
            red: &[5, 6, 9, 10],
        },
        Case {
            name: "negative_width",
            line: [1.0, 1.0, 3.0, 2.0, -2.0],
            rect: [1.0, 1.0, 2.0, 1.0],
            work: 2,
            red: &[5, 6],
        },
        Case {
            name: "zero_height",
            line: [1.0, 1.0, 3.0, 1.0, 0.0],
            rect: [1.0, 1.0, 2.0, 0.0],
            work: 0,
            red: &[],
        },
        Case {
            name: "negative_point_width",
            line: [1.0, 1.0, 1.0, 1.0, -1.0],
            rect: [1.0, 1.0, 0.0, 0.0],
            work: 0,
            red: &[],
        },
        Case {
            name: "fractional",
            line: [0.25, 0.25, 1.75, 1.75, 0.25],
            rect: [0.25, 0.25, 1.5, 1.5],
            work: 4,
            red: &[0, 1, 4, 5],
        },
        Case {
            name: "offscreen",
            line: [5.0, 0.0, 7.0, 1.0, 0.0],
            rect: [5.0, 0.0, 2.0, 1.0],
            work: 0,
            red: &[],
        },
        Case {
            name: "partly_visible",
            line: [-1.0, -1.0, 1.0, 1.0, 0.0],
            rect: [-1.0, -1.0, 2.0, 2.0],
            work: 1,
            red: &[0],
        },
        Case {
            name: "f32_cancel_to_zero",
            line: [-500000.0, 0.0, 0.015625, 1.0, 0.0],
            rect: [-500000.0, 0.0, 500000.0, 1.0],
            work: 0,
            red: &[],
        },
        Case {
            name: "f32_one_ulp_visible",
            line: [-500000.0, 0.0, 0.03125, 1.0, 0.0],
            rect: [-500000.0, 0.0, 500000.03125, 1.0],
            work: 1,
            red: &[0],
        },
    ];
    let target = Frame::new(4, 4, 0xffffff);
    for case in cases {
        let commands = [line(case.line, Color::rgb(255, 0, 0))];
        let entries: Vec<_> = case.red.iter().map(|&index| (index, 0xff0000)).collect();
        let expected = literal_pixels(16, &entries);
        let scene = checked_scene(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands,
            }],
            case.work,
            &expected,
        );
        assert_eq!(scene.stats().bridge.original_commands, 1, "{}", case.name);
        let [x, y, width, height] = case.rect;
        let ordinary = single(
            &[fill(x, y, width, height, Color::rgb(255, 0, 0), 0.0)],
            target,
        )
        .unwrap();
        same_packed_plan(scene.plan(), ordinary.plan());
    }
    // Overlapping primitives make the preserved draw order observable.
    let commands = [
        line([0.0, 0.0, 2.0, 2.0, 0.0], Color::rgb(255, 0, 0)),
        line([1.0, 0.0, 3.0, 0.0, 1.0], Color::rgb(0, 0, 255)),
        fill(1.0, 1.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0),
    ];
    let expected = literal_pixels(
        16,
        &[
            (0, 0xff0000),
            (4, 0xff0000),
            (1, 0x0000ff),
            (2, 0x0000ff),
            (5, 0x00ff00),
        ],
    );
    let scene = checked_scene(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        7,
        &expected,
    );
    let ordinary = single(
        &[
            fill(0.0, 0.0, 2.0, 2.0, Color::rgb(255, 0, 0), 0.0),
            fill(1.0, 0.0, 2.0, 1.0, Color::rgb(0, 0, 255), 0.0),
            fill(1.0, 1.0, 1.0, 1.0, Color::rgb(0, 255, 0), 0.0),
        ],
        target,
    )
    .unwrap();
    same_packed_plan(scene.plan(), ordinary.plan());
}

#[test]
fn native_line_accounting_fractional_clip_and_translation() {
    let target = Frame::new(4, 4, 0xffffff);
    let rows = [
        (
            [0.0, 0.0, 3.0, 3.0, 0.0],
            Rect::new(0.25, 0.25, 1.5, 1.5),
            (0.0, 0.0),
            1,
            vec![5],
        ),
        (
            [0.25, 0.25, 1.25, 0.25, 0.5],
            Rect::new(0.0, 0.0, 4.0, 4.0),
            (0.5, 0.5),
            4,
            vec![0, 1, 4, 5],
        ),
        (
            [2.0, 1.0, 4.0, 1.0, 1.0],
            Rect::new(0.0, 0.0, 4.0, 4.0),
            (-1.0, 1.0),
            2,
            vec![9, 10],
        ),
    ];
    for (geometry, clip, offset, work, red) in rows {
        let mut frame = target;
        frame.caller_clip = clip;
        frame.document_offset = offset;
        let commands = [line(geometry, Color::rgb(255, 0, 0))];
        let entries: Vec<_> = red.into_iter().map(|index| (index, 0xff0000)).collect();
        checked_scene(
            target,
            &[NativePhase {
                frame,
                commands: &commands,
            }],
            work,
            &literal_pixels(16, &entries),
        );
    }
}

#[test]
fn native_line_accounting_phases_restore_fixed_and_clip_state() {
    let target = Frame::new(8, 4, 0xffffff);
    let page = [
        DrawCommand::PushClip {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        },
        DrawCommand::PushFixed,
        line([0.0, 0.0, 2.0, 2.0, 0.0], Color::rgb(255, 0, 0)),
        DrawCommand::PopFixed,
        line([0.0, 0.0, 8.0, 4.0, 0.0], Color::rgb(0, 0, 255)),
        DrawCommand::PopClip,
        line([3.0, 2.0, 5.0, 2.0, 1.0], Color::rgb(0, 255, 0)),
    ];
    let chrome = [
        DrawCommand::PushFixed,
        line([0.0, 0.0, 4.0, 0.0, 1.0], Color::rgb(0, 0, 255)),
        DrawCommand::PopFixed,
    ];
    let mut first = target;
    first.document_offset = (-3.0, 0.0);
    first.viewport_offset = (1.0, 0.0);
    let mut second = target;
    second.caller_clip = Rect::new(4.0, 0.0, 2.0, 4.0);
    second.viewport_offset = (4.0, 0.0);
    let expected = literal_pixels(
        32,
        &[
            (1, 0xff0000),
            (2, 0xff0000),
            (9, 0xff0000),
            (10, 0xff0000),
            (16, 0x00ff00),
            (17, 0x00ff00),
            (4, 0x0000ff),
            (5, 0x0000ff),
        ],
    );
    let scene = checked_scene(
        target,
        &[
            NativePhase {
                frame: first,
                commands: &page,
            },
            NativePhase {
                frame: second,
                commands: &chrome,
            },
        ],
        8,
        &expected,
    );
    assert_eq!(scene.stats().phases, 2);
    assert_eq!(scene.stats().bridge.original_commands, 10);
}

#[test]
fn native_line_accounting_transparency_and_zero_opacity_keep_debt() {
    let target = Frame::new(4, 4, 0xffffff);
    for (alpha, pixel) in [(0, 0xffffff), (128, 0xff7f7f)] {
        let commands = [line(
            [0.25, 0.25, 1.75, 1.75, 0.0],
            Color::rgba(255, 0, 0, alpha),
        )];
        let expected = literal_pixels(16, &[(0, pixel), (1, pixel), (4, pixel), (5, pixel)]);
        checked_scene(
            target,
            &[NativePhase {
                frame: target,
                commands: &commands,
            }],
            4,
            &expected,
        );
    }
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        line([0.25, 0.25, 1.75, 1.75, 0.0], Color::rgb(255, 0, 0)),
        DrawCommand::PopOpacity,
    ];
    // The two opacity markers retain 2*16; hidden Line geometry retains 4.
    let scene = checked_scene(
        target,
        &[NativePhase {
            frame: target,
            commands: &commands,
        }],
        36,
        &[0xffffff; 16],
    );
    assert_eq!(scene.plan().group_scratch_bytes(), 0);
}

#[test]
fn native_line_accounting_exact_shared_cpu_boundary_and_one_over() {
    let target = Frame::new(512, 512, 0xffffff);
    let half = vec![line([0.0, 0.0, 512.0, 512.0, 0.0], Color::TRANSPARENT); 8];
    let phases = [NativePhase {
        frame: target,
        commands: &half,
    }; 2];
    let scene = checked_scene(target, &phases, 4_194_304, &vec![0xffffff; 512 * 512]);
    assert_eq!(scene.stats().bridge.original_commands, 16);
    let one = [line([0.0, 0.0, 1.0, 1.0, 0.0], Color::TRANSPARENT)];
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
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None),
    );
}

#[test]
fn native_line_accounting_empty_inner_loops_keep_structural_caps() {
    let target = Frame::new(1, 1024, 0xffffff);
    let mut frame = target;
    frame.caller_clip = Rect::new(0.25, 0.0, 0.25, 1024.0);
    // Positive geometry and 1024 possible y rows, but ceil(.25)==ceil(.5)==1:
    // no inner pixel is visited. The 256-command and 1024-height caps still bind.
    for color in [Color::TRANSPARENT, Color::BLACK] {
        let mut commands = vec![line([0.0, 0.0, 1.0, 1024.0, 0.0], color); 256];
        let scene = checked_scene(
            target,
            &[NativePhase {
                frame,
                commands: &commands,
            }],
            0,
            &[0xffffff; 1024],
        );
        assert_eq!(scene.stats().bridge.original_commands, 256);
        let quarters = [NativePhase {
            frame,
            commands: &commands[..64],
        }; 4];
        let shared = checked_scene(target, &quarters, 0, &[0xffffff; 1024]);
        assert_eq!(shared.stats().bridge.original_commands, 256);
        assert_eq!(shared.stats().phases, 4);
        commands.push(line([0.0, 0.0, 1.0, 1024.0, 0.0], color));
        assert_eq!(
            plan_native_scene(
                target,
                &[NativePhase {
                    frame,
                    commands: &commands
                }],
                &ImageStore::new(),
                &Fonts::new()
            )
            .unwrap_err(),
            NativeFallback::new(FallbackKind::CommandLimit, Some(0), None),
        );
    }
    let empty: [DrawCommand; 0] = [];
    assert_eq!(
        plan_native_scene(
            target,
            &[NativePhase {
                frame,
                commands: &empty
            }; 5],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        NativeFallback::new(FallbackKind::CommandLimit, None, None),
    );
    assert_eq!(
        single(&[], Frame::new(1, 1025, 0xffffff)).unwrap_err(),
        NativeFallback::new(FallbackKind::ViewportLimit, None, None),
    );
}

#[test]
fn native_line_accounting_validates_hidden_geometry_before_debit() {
    let target = Frame::new(512, 512, 0xffffff);
    let excessive = vec![line([0.0, 0.0, 512.0, 512.0, 0.0], Color::TRANSPARENT); 17];
    let mut invalid = Vec::new();
    for index in 0..5 {
        let mut geometry = [0.0, 0.0, 1.0, 1.0, 1.0];
        geometry[index] = f32::NAN;
        invalid.push(geometry);
    }
    invalid.extend([
        [0.0, 0.0, 1.0, 1.0, f32::INFINITY],
        [0.0, 0.0, 1.0, 1.0, 1_000_001.0],
        [-1_000_000.0, 0.0, 1_000_000.0, 1.0, 0.0],
    ]);
    for geometry in invalid {
        let later = [
            DrawCommand::PushOpacity { opacity: 0.0 },
            DrawCommand::PushClip {
                rect: BrowserRect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
            },
            line(geometry, Color::TRANSPARENT),
            DrawCommand::PopClip,
            DrawCommand::PopOpacity,
        ];
        assert_eq!(
            plan_native_scene(
                target,
                &[
                    NativePhase {
                        frame: target,
                        commands: &excessive
                    },
                    NativePhase {
                        frame: target,
                        commands: &later
                    },
                ],
                &ImageStore::new(),
                &Fonts::new()
            )
            .unwrap_err(),
            at(FallbackKind::InvalidGeometry, 1, 2),
        );
    }
    let mut malformed = excessive;
    malformed.push(DrawCommand::PopFixed);
    assert_eq!(
        single(&malformed, target).unwrap_err(),
        at(FallbackKind::InvalidScope, 0, 17)
    );
}

#[test]
fn native_line_accounting_original_order_controls_box_budget_location() {
    let target = Frame::new(512, 512, 0xffffff);
    let one_pixel_box = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            radii: [[0.0; 2]; 4],
        },
        cutout: None,
        paint: BoxPaint::Color(Color::BLACK),
    }));
    let mut lines_first = vec![line([0.0, 0.0, 512.0, 512.0, 0.0], Color::TRANSPARENT); 16];
    lines_first.push(one_pixel_box.clone());
    // Full Lines consume 4,194,304 before the Box's existing 8+3*1 debit.
    assert_eq!(
        single(&lines_first, target).unwrap_err(),
        at(FallbackKind::CpuPaintBudget, 0, 16)
    );
    let mut box_first = vec![one_pixel_box];
    box_first.extend(lines_first.into_iter().take(16));
    // The Box now pays 11 first; the last Line fails the unindexed geometry debit.
    assert_eq!(
        single(&box_first, target).unwrap_err(),
        NativeFallback::new(FallbackKind::CpuPaintBudget, None, None),
    );
}
