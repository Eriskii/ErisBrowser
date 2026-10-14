//! Native Color-only admission: independent literal pixels frozen before implementation execution.
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use crate::graphics::{Canvas, Color, Rect as BrowserRect};
use eris_raster_core::DrawKind;

fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
fn parameter(plan: &Plan, draw: usize, index: usize) -> u32 {
    word(&plan.parameters()[draw * PARAM_STRIDE..], index)
}
// Independent packed-plan interpretation with integer primitive blending and f32 group pop, used only by
// these bridge tests. Literal expected pixels below are the oracle; neither
// this decoder nor Canvas execution is evidence about a GPU driver.
pub(super) fn decode_opacity_pixels(plan: &Plan) -> Vec<u32> {
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
// Positive subpixel radii retain the mask route while all quarter-grid samples
// and classifier extrema remain inside: P/A0/S1 and every old tariff are exact.
fn mask_square(x: f32, y: f32, width: f32, height: f32) -> RoundedBox {
    RoundedBox {
        rect: rect(x, y, width, height),
        radii: [[0.125; 2]; 4],
    }
}
fn ellipse() -> RoundedBox {
    RoundedBox {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        radii: [[2.0, 1.0]; 4],
    }
}
fn decoration(shape: RoundedBox, cutout: Option<RoundedBox>, color: Color) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape,
        cutout,
        paint: BoxPaint::Color(color),
    }))
}
fn ordinary(bounds: BrowserRect, color: Color) -> DrawCommand {
    DrawCommand::Rect {
        rect: bounds,
        color,
        radius: 0.0,
    }
}
fn checked(
    target: Frame,
    phase: Frame,
    commands: &[DrawCommand],
    expected: &[u32],
) -> NativeScenePlan {
    let phases = [NativePhase {
        frame: phase,
        commands,
    }];
    let fonts = Fonts::new();
    let images = ImageStore::new();
    let native = plan_native_scene(target, &phases, &images, &fonts).unwrap();
    assert_eq!(decode_opacity_pixels(native.plan()), expected);
    assert_eq!(canvas_pixels(target, &phases, &images, &fonts), expected);
    assert!(fonts.cache.borrow().is_empty());
    assert!(native.plan().gpu_buffer_bytes() <= native.stats().gpu_buffer_upper_bound);
    native
}
fn glyph_coverage(plan: &Plan) -> Vec<Vec<u32>> {
    plan.draws()
        .iter()
        .enumerate()
        .filter(|(_, draw)| draw.kind() == DrawKind::Glyph)
        .map(|(i, _)| {
            let base = parameter(plan, i, 8) as usize;
            let cells = (parameter(plan, i, 9) * parameter(plan, i, 10)) as usize;
            (0..cells)
                .map(|j| word(plan.input_bytes(), base + j))
                .collect()
        })
        .collect()
}

#[test]
fn native_box_color_ellipse_has_literal_glyph_bytes_and_pixels() {
    let frame = Frame::new(4, 2, 0xffffff);
    let result = checked(
        frame,
        frame,
        &[decoration(ellipse(), None, Color::rgb(0, 0, 255))],
        &[
            0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff, 0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff,
        ],
    );
    assert_eq!(
        glyph_coverage(result.plan()),
        [vec![143, 255, 255, 143, 143, 255, 255, 143]]
    );
    assert_eq!(
        (
            result.stats().box_masks,
            result.stats().coverage_bytes,
            result.stats().row_entries
        ),
        (1, 8, 2)
    );
    assert_eq!(result.stats().box_loop_work, 106);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 106);
    assert_eq!(result.stats().rounded_masks, 0);
}

#[test]
fn native_box_color_ring_samples_are_correlated_before_quantization() {
    let frame = Frame::new(2, 2, 0xffffff);
    let cutout = square(0.25, 0.25, 1.0, 1.0);
    let result = checked(
        frame,
        frame,
        &[decoration(
            square(0.0, 0.0, 2.0, 2.0),
            Some(cutout),
            Color::BLACK,
        )],
        &[0x909090, 0x303030, 0x303030, 0x101010],
    );
    assert_eq!(glyph_coverage(result.plan()), [vec![111, 207, 207, 239]]);
    assert_eq!(result.stats().box_loop_work, 162);
    checked(
        frame,
        frame,
        &[decoration(cutout, Some(cutout), Color::BLACK)],
        &[0xffffff; 4],
    );
}

#[test]
fn native_box_color_used_radii_are_not_renormalized() {
    let frame = Frame::new(1, 4, 0xffffff);
    let used = RoundedBox {
        rect: rect(0.0, 0.0, 1.0, 4.0),
        radii: [[2.0, 2.0], [0.0, 2.0], [0.0, 2.0], [2.0, 2.0]],
    };
    let result = checked(
        frame,
        frame,
        &[decoration(used, None, Color::rgb(0, 0, 255))],
        &[0xa0a0ff, 0x1010ff, 0x1010ff, 0xa0a0ff],
    );
    assert_eq!(glyph_coverage(result.plan()), [vec![95, 239, 239, 95]]);
    assert_eq!(result.stats().box_loop_work, 92);
    let frame = Frame::new(2, 2, 0xffffff);
    let overlap = RoundedBox {
        rect: rect(0.0, 0.0, 2.0, 2.0),
        radii: [[4.0; 2]; 4],
    };
    checked(
        frame,
        frame,
        &[decoration(overlap, None, Color::BLACK)],
        &[0xffffff; 4],
    );
}

#[test]
fn native_box_color_fractional_caller_clip_and_negative_origin_are_literal() {
    let target = Frame::new(4, 2, 0xffffff);
    let mut phase = target;
    phase.caller_clip = Rect::new(0.5, 0.0, 2.5, 2.0);
    checked(
        target,
        phase,
        &[decoration(
            square(-0.25, -0.25, 4.25, 2.25),
            None,
            Color::BLACK,
        )],
        &[
            0xffffff, 0x000000, 0x000000, 0xffffff, 0xffffff, 0x000000, 0x000000, 0xffffff,
        ],
    );
}

#[test]
fn native_box_color_fixed_scope_restores_document_clip_and_ordinary_paint() {
    let target = Frame::new(4, 2, 0xffffff);
    let mut phase = target;
    phase.document_offset = (1.0, 0.0);
    let commands = [
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 1.0, 1.0),
        },
        decoration(square(0.0, 0.0, 2.0, 1.0), None, Color::rgb(255, 0, 0)),
        DrawCommand::PushFixed,
        decoration(square(2.25, 0.25, 1.0, 1.0), None, Color::rgb(0, 0, 255)),
        DrawCommand::PopFixed,
        decoration(square(0.0, 0.0, 2.0, 1.0), None, Color::rgb(0, 255, 0)),
        DrawCommand::PopClip,
        ordinary(rect(0.0, 1.0, 1.0, 1.0), Color::BLACK),
    ];
    checked(
        target,
        phase,
        &commands,
        &[
            0xffffff, 0x00ff00, 0x7070ff, 0xd0d0ff, 0xffffff, 0x000000, 0xd0d0ff, 0xf0f0ff,
        ],
    );
}

#[test]
fn native_box_color_source_alpha_and_order_remain_gpu_blend_inputs() {
    let frame = Frame::new(4, 2, 0xffff00);
    checked(
        frame,
        frame,
        &[decoration(
            ellipse(),
            None,
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 128,
            },
        )],
        &[
            0xffb800, 0xff7f00, 0xff7f00, 0xffb800, 0xffb800, 0xff7f00, 0xff7f00, 0xffb800,
        ],
    );
    let frame = Frame::new(2, 1, 0xffffff);
    let commands = [
        decoration(
            square(0.0, 0.0, 2.0, 1.0),
            None,
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 128,
            },
        ),
        decoration(
            square(0.0, 0.0, 1.0, 1.0),
            None,
            Color {
                r: 0,
                g: 0,
                b: 255,
                a: 128,
            },
        ),
        ordinary(rect(1.0, 0.0, 1.0, 1.0), Color::rgb(0, 255, 0)),
    ];
    checked(frame, frame, &commands, &[0x7f3fbf, 0x00ff00]);
}

#[test]
fn native_box_color_nested_opacity_keeps_rgba16_group_rounding() {
    let frame = Frame::new(4, 2, 0xffffff);
    let commands = [
        DrawCommand::PushOpacity { opacity: 0.5 },
        DrawCommand::PushOpacity { opacity: 0.5 },
        decoration(ellipse(), None, Color::rgb(0, 0, 255)),
        DrawCommand::PopOpacity,
        DrawCommand::PopOpacity,
    ];
    let result = checked(
        frame,
        frame,
        &commands,
        &[
            0xdbdbff, 0xbfbfff, 0xbfbfff, 0xdbdbff, 0xdbdbff, 0xbfbfff, 0xbfbfff, 0xdbdbff,
        ],
    );
    assert_eq!(result.stats().box_loop_work, 106);
    assert_eq!(
        result
            .plan()
            .draws()
            .iter()
            .filter(|draw| draw.kind() == DrawKind::GroupComposite)
            .count(),
        2
    );
}

#[test]
fn native_box_color_transparent_and_zero_opacity_keep_conservative_reservations() {
    let frame = Frame::new(4, 2, 0xffffff);
    for commands in [
        vec![decoration(ellipse(), None, Color::TRANSPARENT)],
        vec![
            DrawCommand::PushOpacity { opacity: 0.0 },
            decoration(ellipse(), None, Color::BLACK),
            DrawCommand::PopOpacity,
        ],
    ] {
        let result = checked(frame, frame, &commands, &[0xffffff; 8]);
        assert_eq!(
            (
                result.stats().box_masks,
                result.stats().coverage_bytes,
                result.stats().row_entries
            ),
            (1, 8, 2)
        );
        assert_eq!(result.stats().box_loop_work, 106);
        assert!(result.stats().cpu_pixel_upper_bound >= 106);
    }
}

#[test]
fn native_box_color_raw_invalid_coordinates_precede_visibility_and_alpha() {
    let frame = Frame::new(4, 2, 0xffffff);
    for field in 0..12 {
        for value in [f32::NAN, f32::INFINITY, 1_000_001.0] {
            let mut shape = ellipse();
            match field {
                0 => shape.rect.x = value,
                1 => shape.rect.y = value,
                2 => shape.rect.width = value,
                3 => shape.rect.height = value,
                _ => shape.radii[(field - 4) / 2][(field - 4) % 2] = value,
            }
            let commands = [
                DrawCommand::PushOpacity { opacity: 0.0 },
                decoration(shape, None, Color::TRANSPARENT),
                DrawCommand::PopOpacity,
            ];
            let result = plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &commands,
                }],
                &ImageStore::new(),
                &Fonts::new(),
            );
            assert_eq!(result.unwrap_err(), at(FallbackKind::InvalidGeometry, 0, 1));
        }
    }
    let mut phase = frame;
    phase.document_offset = (1_000_000.0, 0.0);
    checked(
        frame,
        phase,
        &[decoration(
            square(1_000_000.0, 0.0, 1.0, 1.0),
            None,
            Color::BLACK,
        )],
        &[0xffffff; 8],
    );
}

#[test]
fn native_box_color_strip_widths_keep_absolute_rows_and_exact_storage_work() {
    for (width, masks, rows, work) in [(1024, 1, 1, 4105), (1025, 2, 2, 4110), (1280, 2, 2, 5130)] {
        let frame = Frame::new(width, 1, 0xffffff);
        let result = checked(
            frame,
            frame,
            &[decoration(
                mask_square(0.0, 0.0, width as f32, 1.0),
                None,
                Color::BLACK,
            )],
            &vec![0; width as usize],
        );
        assert_eq!(
            (result.stats().box_masks, result.stats().row_entries),
            (masks, rows)
        );
        assert_eq!(result.stats().coverage_bytes, width as usize);
        assert_eq!(result.stats().box_loop_work, work);
        let glyphs: Vec<_> = result
            .plan()
            .draws()
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind() == DrawKind::Glyph)
            .collect();
        assert_eq!(glyphs.len(), masks);
        for (tile, (draw, _)) in glyphs.into_iter().enumerate() {
            assert_eq!(
                word(
                    result.plan().input_bytes(),
                    parameter(result.plan(), draw, 12) as usize
                ) as i32,
                if tile == 0 { 0 } else { 1024 }
            );
            assert_eq!(parameter(result.plan(), draw, 13), 0);
        }
    }
}

#[test]
fn native_box_color_shared_mask_area_cap_is_not_per_strip() {
    let frame = Frame::new(1280, 204, 0xffffff);
    let result = checked(
        frame,
        frame,
        &[decoration(
            mask_square(0.0, 0.0, 1280.0, 204.0),
            None,
            Color::BLACK,
        )],
        &vec![0; 261_120],
    );
    assert_eq!(
        (
            result.stats().box_masks,
            result.stats().coverage_bytes,
            result.stats().row_entries
        ),
        (2, 261_120, 408)
    );
    assert_eq!(result.stats().box_loop_work, 1_044_896);
    let frame = Frame::new(1280, 205, 0xffffff);
    let commands = [decoration(
        mask_square(0.0, 0.0, 1280.0, 205.0),
        None,
        Color::BLACK,
    )];
    // Same original oversized input: the new core route admits its exact pixels.
    let result = checked(frame, frame, &commands, &vec![0; 262_400]);
    assert_eq!(
        (
            result.stats().box_cores,
            result.stats().box_masks,
            result.stats().coverage_bytes,
            result.stats().row_entries
        ),
        (1, 6, 2_966, 410)
    );
    assert_eq!(result.stats().box_rectangles, 0);
    assert_eq!(result.stats().box_loop_work, 790_588);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 790_588);
    assert_eq!(result.stats().bridge.lowered_commands, 7);

    // Separate forced-mask control: even an empty cutout excludes core extraction.
    let commands = [decoration(
        mask_square(0.0, 0.0, 1280.0, 205.0),
        Some(square(0.0, 0.0, 0.0, 0.0)),
        Color::BLACK,
    )];
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &ImageStore::new(),
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::MaskPreparation, 0, 0)
    );
}

#[test]
fn native_box_color_loaded_sources_and_aliases_share_the_source_limit() {
    let frame = Frame::new(1, 1, 0xffffff);
    let commands = [decoration(
        mask_square(0.0, 0.0, 1.0, 1.0),
        None,
        Color::BLACK,
    )];
    let mut images = ImageStore::new();
    for i in 0..255 {
        images.insert(
            format!("image-{i}"),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 0, 255],
            }),
        );
    }
    let result = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &commands,
        }],
        &images,
        &Fonts::new(),
    )
    .unwrap();
    assert_eq!(result.stats().box_masks, 1);
    assert_eq!(decode_opacity_pixels(result.plan()), [0]);
    let alias = images["image-0"].clone();
    images.insert("alias".into(), alias);
    assert!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &images,
            &Fonts::new()
        )
        .is_ok()
    );
    images.remove("alias");
    images.insert(
        "new".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        }),
    );
    assert_eq!(
        plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &commands
            }],
            &images,
            &Fonts::new()
        )
        .unwrap_err(),
        at(FallbackKind::MaskPreparation, 0, 0)
    );
}

#[test]
fn native_box_color_reservation_has_literal_two_stage_cpu_cuts() {
    use crate::graphics::box_decoration::{BoxMaskError, BoxMaskWindow};
    for (initial, expected_left) in [(15, 15), (16, 0), (105, 89)] {
        let mut remaining = initial;
        let window = BoxMaskWindow::new(4, 2, ellipse(), None, rect(0.0, 0.0, 4.0, 2.0)).unwrap();
        assert!(matches!(
            window.reserve(&mut remaining),
            Err(BoxMaskError::WorkLimit)
        ));
        assert_eq!(remaining, expected_left);
    }
    let mut remaining = 106;
    let reservation = BoxMaskWindow::new(4, 2, ellipse(), None, rect(0.0, 0.0, 4.0, 2.0))
        .unwrap()
        .reserve(&mut remaining)
        .unwrap();
    assert_eq!(remaining, 0);
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
    let mut ledger = box_ledger();
    ledger.cpu_pixels = 106;
    ledger.cpu_limit = 106;
    ledger.box_mask(&reservation, 2, 7).unwrap();
    assert_eq!(
        ledger.cpu_pixels, 106,
        "mask materialization does not debit geometry twice"
    );
    assert_eq!(ledger.box_loop_work, 106);
}

fn box_ledger() -> Ledger {
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
        cpu_pixels: 0,
        cpu_limit: 1_000_000,
        gpu_upper: 0,
        rounded_loop_work: 0,
        box_loop_work: 0,
    }
}

#[test]
fn native_box_color_two_strip_shared_ledger_cuts_are_transactional() {
    use crate::graphics::box_decoration::BoxMaskWindow;
    let reservation = BoxMaskWindow::new(
        1025,
        1,
        square(0.0, 0.0, 1025.0, 1.0),
        None,
        rect(0.0, 0.0, 1025.0, 1.0),
    )
    .unwrap()
    .reserve(&mut 4110)
    .unwrap();
    // Existing parameter ABI has256 bytes/record. Two operations +clear make768.
    assert_eq!(PARAM_STRIDE, 256);
    for boundary in 0..6 {
        let mut exact = box_ledger();
        match boundary {
            0 => exact.sources = 254,
            1 => exact.coverage = 261_119,
            2 => exact.rows = 65_534,
            3 => exact.row_tables = 254,
            4 => exact.operations = 255,
            _ => exact.targets = 16_777_216 - 4_108 - 768,
        }
        let mut over = exact;
        match boundary {
            0 => over.sources += 1,
            1 => over.coverage += 1,
            2 => over.rows += 1,
            3 => over.row_tables += 1,
            4 => over.operations += 1,
            _ => over.targets += 1,
        }
        exact.box_mask(&reservation, 2, 7).unwrap();
        match boundary {
            0 => assert_eq!(exact.sources, 256),
            1 => assert_eq!(exact.coverage, 262_144),
            2 => assert_eq!(exact.rows, 65_536),
            3 => assert_eq!(exact.row_tables, 256),
            4 => assert_eq!(exact.operations, 256),
            _ => assert_eq!(exact.gpu_upper, 16_777_216),
        }
        assert_eq!(exact.box_loop_work, 4110);
        let before = over;
        let kind = match boundary {
            0 | 1 => FallbackKind::MaskPreparation,
            2 => FallbackKind::GlyphRowLimit,
            3 | 4 => FallbackKind::CommandLimit,
            _ => FallbackKind::PlannerLimit,
        };
        assert_eq!(
            over.box_mask(&reservation, 2, 7).unwrap_err(),
            at(kind, 2, 7)
        );
        assert_eq!(
            over, before,
            "no partial strip admission at boundary {boundary}"
        );
    }
}

#[test]
fn native_box_color_shares_rows_coverage_and_sources_with_existing_masks() {
    use crate::graphics::box_decoration::BoxMaskWindow;
    let reservation = BoxMaskWindow::new(
        1,
        1,
        square(0.0, 0.0, 1.0, 1.0),
        None,
        rect(0.0, 0.0, 1.0, 1.0),
    )
    .unwrap()
    .reserve(&mut 13)
    .unwrap();
    let mut ledger = box_ledger();
    // Existing font/rounded mask admission uses this common source/coverage ledger.
    ledger
        .mask(
            262_143,
            65_535,
            true,
            false,
            at(FallbackKind::MaskPreparation, 0, 0),
        )
        .unwrap();
    ledger.box_mask(&reservation, 0, 1).unwrap();
    assert_eq!(
        (ledger.sources, ledger.masks, ledger.coverage, ledger.rows),
        (2, 2, 262_144, 65_536)
    );
    let before = ledger;
    assert_eq!(
        ledger.box_mask(&reservation, 0, 2).unwrap_err(),
        at(FallbackKind::GlyphRowLimit, 0, 2)
    );
    assert_eq!(ledger, before);
}

#[test]
fn native_box_color_full_scene_cpu_limit_has_an_exact_one_pixel_cut() {
    // Frame area64000 gives the unchanged1,024,000 CPU allowance.
    // Three1000x64 masks cost3*256072; one999x63 costs251819.
    // Their sum1,020,035 plus one ordinary793x5 rectangle is exactly1,024,000.
    let frame = Frame::new(1000, 64, 0xffffff);
    let full = decoration(mask_square(0.0, 0.0, 1000.0, 64.0), None, Color::BLACK);
    let mut commands = vec![
        full.clone(),
        full.clone(),
        full,
        decoration(mask_square(0.0, 0.0, 999.0, 63.0), None, Color::BLACK),
        ordinary(rect(0.0, 0.0, 793.0, 5.0), Color::BLACK),
    ];
    let native = checked(frame, frame, &commands, &vec![0; 64_000]);
    assert_eq!(native.stats().cpu_pixel_upper_bound, 1_024_000);
    assert_eq!(native.stats().box_loop_work, 1_020_035);
    assert_eq!(native.stats().coverage_bytes, 254_937);
    commands.push(ordinary(rect(0.0, 0.0, 1.0, 1.0), Color::BLACK));
    let error = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &commands,
        }],
        &ImageStore::new(),
        &Fonts::new(),
    )
    .unwrap_err();
    assert_eq!(
        error,
        NativeFallback {
            kind: FallbackKind::CpuPaintBudget,
            phase_index: None,
            command_index: None
        }
    );
}
