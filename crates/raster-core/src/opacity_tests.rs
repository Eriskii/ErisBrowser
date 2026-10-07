//! Independent Native opacity planner checks. Literal pixels were authored and
//! peer-reviewed before implementation in numerical-source-ready.json under
//! /tmp/eris-next-rendering-assessment-1. This decoder checks public plan bytes;
//! it is not shader execution or a replacement for GPU/readback comparisons.
use super::*;

fn native(frame: Frame, commands: &[Command]) -> Result<Plan> {
    plan_with_masks_for_profile(Profile::Native, frame, commands, &[], &[], &[])
}

fn rgba(rect: Rect, color: [u8; 4]) -> Command {
    Command::Rect {
        rect,
        rgba: color,
        radius: 0.0,
    }
}

fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}

fn scratch_index(p: impl Fn(usize) -> u32, field: usize, x: u32, y: u32) -> usize {
    let (base, ox, oy, width, height) = (
        p(field),
        p(field + 1),
        p(field + 2),
        p(field + 3),
        p(field + 4),
    );
    assert_eq!(base % 2, 0);
    assert!(x >= ox && y >= oy && x - ox < width && y - oy < height);
    (base / 2 + (y - oy) * width + x - ox) as usize
}

// Arithmetic uses u64 here to independently check the shader's u32 envelope.
// Each record is logical R,G,B,A; storage packing is tested separately below.
fn over(source: u32, coverage: u32, destination: [u32; 4]) -> [u32; 4] {
    let a = u64::from((source >> 24) * coverage / 255) * 257;
    let mut out = [0; 4];
    for (index, shift) in [16, 8, 0, 24].into_iter().enumerate() {
        let component = if index == 3 {
            a
        } else {
            (u64::from((source >> shift) & 255) * a + 127) / 255
        };
        let numerator = u64::from(destination[index]) * (65535 - a) + 32767;
        assert!(numerator <= u64::from(u32::MAX));
        out[index] = (component + numerator / 65535).try_into().unwrap();
        assert!(out[index] <= 65535);
    }
    out
}

fn decode(plan: &Plan) -> Vec<u32> {
    let frame = plan.frame();
    let mut pixels = vec![0xdeadbeef; frame.width as usize * frame.height as usize];
    assert_eq!(plan.group_scratch_bytes() % 8, 0);
    let mut scratch = vec![[0xdead; 4]; plan.group_scratch_bytes() as usize / 8];
    for (draw_index, draw) in plan.draws().iter().enumerate() {
        let record = &plan.parameters()[draw_index * PARAM_STRIDE..(draw_index + 1) * PARAM_STRIDE];
        let p = |index| word(record, index);
        let (x0, y0, width, height) = draw.bounds();
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let root = (y * frame.width + x) as usize;
                match draw.kind() {
                    DrawKind::GroupClear => {
                        assert!(draw.target_is_group());
                        assert_eq!(p(21), 1);
                        scratch[scratch_index(p, 16, x, y)] = [0; 4];
                    }
                    DrawKind::GroupComposite => {
                        let source = scratch[scratch_index(p, 24, x, y)];
                        assert!(source[..3].iter().all(|&channel| channel <= source[3]));
                        // Deliberately use Canvas's separate f32 stages, not the
                        // shader's integer emulation, opaque or tiny fast paths.
                        let opacity = match p(30) {
                            0 => {
                                assert!((1..256).contains(&p(29)));
                                assert_eq!(p(31), 0);
                                p(29) as f32 / 256.0
                            }
                            1 => {
                                assert_eq!(p(29), 0);
                                assert!((1..0x3f80_0000).contains(&p(31)));
                                f32::from_bits(p(31))
                            }
                            _ => panic!("unknown opacity parameter route"),
                        };
                        let inverse = 1.0 - source[3] as f32 / 65535.0 * opacity;
                        if draw.target_is_group() {
                            assert_eq!(p(21), 1);
                            let index = scratch_index(p, 16, x, y);
                            for (c, source_channel) in source.into_iter().enumerate() {
                                scratch[index][c] = (source_channel as f32 * opacity
                                    + scratch[index][c] as f32 * inverse)
                                    .round()
                                    as u32;
                            }
                            assert!(scratch[index][3] <= 65535);
                            assert!(scratch[index][..3].iter().all(|&c| c <= scratch[index][3]));
                        } else {
                            assert_eq!(p(21), 0);
                            let mut color = 0;
                            for (c, shift) in [16, 8, 0].into_iter().enumerate() {
                                let d = ((pixels[root] >> shift) & 255) * 257;
                                let channel = source[c] as f32 * opacity + d as f32 * inverse;
                                let next = (channel / 257.0).round() as u32;
                                assert!(next <= 255);
                                color |= next << shift;
                            }
                            pixels[root] = color;
                        }
                    }
                    kind => {
                        let (source, coverage) = match kind {
                            DrawKind::Rectangle => (p(6), 255),
                            DrawKind::Image => {
                                let sx = word(plan.input_bytes(), (p(12) + x - x0) as usize);
                                let sy = word(plan.input_bytes(), (p(13) + y - y0) as usize);
                                (
                                    word(plan.input_bytes(), (p(8) + sy * p(9) + sx) as usize),
                                    255,
                                )
                            }
                            DrawKind::Glyph => {
                                let sy = i64::from(y) - i64::from(p(13) as i32);
                                if sy < 0 || sy >= i64::from(p(10)) {
                                    continue;
                                }
                                let ox =
                                    word(plan.input_bytes(), p(12) as usize + sy as usize) as i32;
                                let sx = i64::from(x) - i64::from(ox);
                                if sx < 0 || sx >= i64::from(p(9)) {
                                    continue;
                                }
                                (
                                    p(6),
                                    word(
                                        plan.input_bytes(),
                                        p(8) as usize + sy as usize * p(9) as usize + sx as usize,
                                    ),
                                )
                            }
                            _ => unreachable!(),
                        };
                        if draw.target_is_group() {
                            assert_eq!(p(21), 1);
                            let index = scratch_index(p, 16, x, y);
                            scratch[index] = over(source, coverage, scratch[index]);
                        } else {
                            assert!(record[64..].iter().all(|&b| b == 0));
                            let a = (source >> 24) * coverage / 255;
                            let mut color = 0;
                            for shift in [0, 8, 16] {
                                let c = (((source >> shift) & 255) * a
                                    + ((pixels[root] >> shift) & 255) * (255 - a)
                                    + 127)
                                    / 255;
                                color |= c << shift;
                            }
                            pixels[root] = color;
                        }
                    }
                }
            }
        }
    }
    pixels
}

#[test]
fn opacity_tie_and_overlapping_children_match_frozen_pixels() {
    let tie = native(
        Frame::new(3, 1, 0),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 1., 1., 0x410000),
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&tie), [0x210000, 0, 0]);
    assert_eq!(tie.group_scratch_bytes(), 8);
    assert_eq!(tie.invocations(), 320);
    assert_eq!(tie.gpu_buffer_bytes(), 1316);
    assert_eq!(
        tie.draws().iter().map(|d| d.kind()).collect::<Vec<_>>(),
        [
            DrawKind::Rectangle,
            DrawKind::GroupClear,
            DrawKind::Rectangle,
            DrawKind::GroupComposite,
        ]
    );
    let overlap = native(
        Frame::new(8, 2, 0xffffff),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 6., 2., 0xff0000),
            rect(2., 0., 4., 2., 0x0000ff),
            Command::PopOpacity,
            rect(7., 0., 1., 2., 0x00ff00),
        ],
    )
    .unwrap();
    assert_eq!(
        decode(&overlap),
        [
            0xff8080, 0xff8080, 0x8080ff, 0x8080ff, 0x8080ff, 0x8080ff, 0xffffff, 0x00ff00,
            0xff8080, 0xff8080, 0x8080ff, 0x8080ff, 0x8080ff, 0x8080ff, 0xffffff, 0x00ff00,
        ]
    );
}

#[test]
fn opacity_nested_pop_keeps_rgba16_rounding_and_disjoint_regions() {
    let plan = native(
        Frame::new(4, 1, 0xffffff),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 3., 1., 0xff0000),
            Command::PushOpacity(0.5),
            rect(1., 0., 1., 1., 0x0000ff),
            Command::PopOpacity,
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&plan), [0xff8080, 0xbf80bf, 0xff8080, 0xffffff]);
    assert_eq!(plan.group_scratch_bytes(), 32);
    let clears: Vec<_> = plan
        .draws()
        .iter()
        .enumerate()
        .filter(|(_, d)| d.kind() == DrawKind::GroupClear)
        .collect();
    assert_eq!(clears.len(), 2);
    let mut ranges = Vec::new();
    for (index, draw) in clears {
        let record = &plan.parameters()[index * PARAM_STRIDE..];
        let base = word(record, 16);
        let (_, _, w, h) = draw.bounds();
        ranges.push((base, base + w * h * 2));
    }
    ranges.sort();
    assert!(ranges[0].1 <= ranges[1].0);
    assert_eq!(ranges[1].1 as u64 * 4, plan.group_scratch_bytes());
}

#[test]
fn opacity_near_half_grid_values_keep_root_and_nested_rounding() {
    // Additive literals authored before the first candidate execution. No
    // candidate plan or CPU painter supplied these expected channels.
    for (k, root, nested) in [
        (127, 0x200000, 0xc080bf),
        (128, 0x210000, 0xbf80bf),
        (129, 0x210000, 0xbf80c0),
    ] {
        let value = k as f32 / 256.;
        let direct = native(
            Frame::new(1, 1, 0),
            &[
                Command::PushOpacity(value),
                rect(0., 0., 1., 1., 0x410000),
                Command::PopOpacity,
            ],
        )
        .unwrap();
        assert_eq!(decode(&direct), [root]);
        let paired = native(
            Frame::new(1, 1, 0xffffff),
            &[
                Command::PushOpacity(0.5),
                rect(0., 0., 1., 1., 0xff0000),
                Command::PushOpacity(value),
                rect(0., 0., 1., 1., 0x0000ff),
                Command::PopOpacity,
                Command::PopOpacity,
            ],
        )
        .unwrap();
        assert_eq!(decode(&paired), [nested]);
    }
}

#[test]
fn opacity_image_and_mask_alpha_remain_primitive_inputs() {
    let images = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[255, 0, 0, 128],
    }];
    let image = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(3, 1, 0),
        &[
            Command::PushOpacity(0.25),
            rect(0., 0., 2., 1., 0xffffff),
            Command::Image {
                rect: Rect::new(1., 0., 1., 1.),
                source: 0,
            },
            Command::PopOpacity,
        ],
        &images,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(decode(&image), [0x404040, 0x402020, 0]);
    let masks = [SourceMask {
        width: 2,
        height: 1,
        coverage: &[0, 128],
    }];
    let mask = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(3, 1, 0),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 2., 1., 0xffffff),
            Command::Glyph {
                source: 0,
                rows: 0,
                y: 0,
                rgba: [255, 0, 0, 128],
            },
            Command::PopOpacity,
        ],
        &[],
        &masks,
        &[&[0]],
    )
    .unwrap();
    assert_eq!(decode(&mask), [0x808080, 0x806060, 0]);
}

#[test]
fn opacity_fixed_escape_and_fractional_caller_clip_keep_absolute_bounds() {
    let mut frame = Frame::new(6, 2, 0xffffff);
    frame.document_offset = (0., -5.);
    let plan = native(
        frame,
        &[
            Command::PushOpacity(0.5),
            rect(0., 5., 5., 2., 0xff0000),
            Command::PushClip(Rect::new(0., 5., 1., 1.)),
            Command::PushFixed,
            rect(3., 0., 1., 1., 0x0000ff),
            Command::PopFixed,
            rect(0., 5., 1., 1., 0x00ff00),
            Command::PopClip,
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(
        decode(&plan),
        [
            0x80ff80, 0xff8080, 0xff8080, 0x8080ff, 0xff8080, 0xffffff, 0xff8080, 0xff8080,
            0xff8080, 0xff8080, 0xff8080, 0xffffff,
        ]
    );
    assert_eq!(plan.group_scratch_bytes(), 80);
    let mut fractional = Frame::new(3, 1, 0xffffff);
    fractional.caller_clip = Rect::new(0.5, 0., 1., 1.);
    let plan = native(
        fractional,
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 3., 1., 0xff0000),
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&plan), [0xffffff, 0xff8080, 0xffffff]);
    assert_eq!(plan.group_scratch_bytes(), 8);
    assert_eq!(plan.draws()[1].bounds(), (1, 0, 1, 1));
}

#[test]
fn opacity_unit_zero_and_empty_scopes_preserve_existing_targets() {
    let draws = [
        rgba(Rect::new(0., 0., 1., 1.), [1, 0, 0, 128]),
        rgba(Rect::new(0., 0., 1., 1.), [0, 0, 0, 1]),
    ];
    let bare = native(Frame::new(1, 1, 0), &draws).unwrap();
    let unit = native(
        Frame::new(1, 1, 0),
        &[
            Command::PushOpacity(1.),
            draws[0],
            draws[1],
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&unit), [0x010000]);
    assert_eq!(unit.group_scratch_bytes(), 0);
    assert_eq!(unit.parameters(), bare.parameters());
    assert_eq!(unit.draws(), bare.draws());
    let nested_unit = native(
        Frame::new(1, 1, 0xffffff),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 1., 1., 0xff0000),
            Command::PushOpacity(1.),
            rect(0., 0., 1., 1., 0x0000ff),
            Command::PopOpacity,
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&nested_unit), [0x8080ff]);
    assert_eq!(nested_unit.group_scratch_bytes(), 8);
    assert_eq!(
        nested_unit
            .draws()
            .iter()
            .filter(|d| d.kind() == DrawKind::GroupClear)
            .count(),
        1
    );
    let zero = native(
        Frame::new(2, 1, 0xffffff),
        &[
            Command::PushOpacity(0.),
            rgba(Rect::new(0., 0., 2., 1.), [255, 0, 0, 128]),
            Command::PopOpacity,
            rect(1., 0., 1., 1., 0x00ff00),
        ],
    )
    .unwrap();
    assert_eq!(decode(&zero), [0xffffff, 0x00ff00]);
    assert_eq!(zero.group_scratch_bytes(), 0);
    for opacity in [0., -0., 0.5, 1.] {
        let empty = native(
            Frame::new(1, 1, 0),
            &[Command::PushOpacity(opacity), Command::PopOpacity],
        )
        .unwrap();
        assert_eq!(empty.group_scratch_bytes(), 0);
        assert_eq!(empty.draws().len(), 1);
    }
}

#[test]
fn opacity_grid_records_stay_exact_and_all_finite_partial_values_admit() {
    let baseline = native(
        Frame::new(1, 1, 0),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 1., 1., 0xffffff),
            Command::PopOpacity,
        ],
    )
    .unwrap();
    for k in 1u32..256 {
        let p = native(
            Frame::new(1, 1, 0),
            &[
                Command::PushOpacity(k as f32 / 256.),
                rect(0., 0., 1., 1., 0xffffff),
                Command::PopOpacity,
            ],
        )
        .unwrap();
        assert_eq!(p.group_scratch_bytes(), 8);
        // Every old record byte remains exact; only the old numerator varies.
        let mut expected = baseline.parameters().to_vec();
        let numerator_offset = (p.draws().len() - 1) * PARAM_STRIDE + 29 * 4;
        expected[numerator_offset..numerator_offset + 4].copy_from_slice(&k.to_le_bytes());
        assert_eq!(p.parameters(), expected);
        assert_eq!(p.draws().last().unwrap().opacity_numerator(), Some(k));
        assert_eq!(
            p.draws().last().unwrap().opacity_bits(),
            Some((k as f32 / 256.).to_bits())
        );
        assert_eq!(
            word(
                p.parameters(),
                (p.draws().len() - 1) * PARAM_STRIDE / 4 + 29
            ),
            k
        );
    }
    for bits in [
        1,
        0x007f_ffff,
        0x0080_0000,
        0x3300_0000,
        0x3300_0001,
        0x3dcc_cccd,
        0x3f00_0001,
        0x3f7f_ffff,
    ] {
        let opacity = f32::from_bits(bits);
        for commands in [
            vec![Command::PushOpacity(opacity), Command::PopOpacity],
            vec![
                Command::PushOpacity(0.),
                Command::PushOpacity(opacity),
                Command::PopOpacity,
                Command::PopOpacity,
            ],
        ] {
            let p = native(Frame::new(1, 1, 0), &commands).unwrap();
            assert_eq!(p.group_scratch_bytes(), 0);
            assert_eq!(p.draws().len(), 1);
        }
    }
    for opacity in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.01] {
        for prefix in [vec![], vec![Command::PushOpacity(0.)]] {
            let mut commands = prefix.clone();
            commands.extend([Command::PushOpacity(opacity), Command::PopOpacity]);
            if !prefix.is_empty() {
                commands.push(Command::PopOpacity);
            }
            assert_eq!(
                native(Frame::new(1, 1, 0), &commands).unwrap_err(),
                "invalid opacity"
            );
        }
    }
}

#[test]
fn opacity_raw_values_keep_source_rounding_and_literal_root_pixels() {
    for (bits, source, clear, expected) in [
        (0x3dcc_cccd, 0xe8e8e8, 0, 0x171717),
        (0x3f33_3333, 0x050505, 0, 0x040404),
        (0x0000_0001, 0xffffff, 0x334455, 0x334455),
        (0x007f_ffff, 0xffffff, 0x334455, 0x334455),
        (0x0080_0000, 0xffffff, 0x334455, 0x334455),
        (0x32ff_ffff, 0xffffff, 0x334455, 0x334455),
        (0x3300_0000, 0xffffff, 0x334455, 0x334455),
        (0x3300_0001, 0xffffff, 0, 0),
        (0x3f7f_ffff, 0x050505, 0, 0x050505),
    ] {
        let p = native(
            Frame::new(1, 1, clear),
            &[
                Command::PushOpacity(f32::from_bits(bits)),
                rect(0., 0., 1., 1., source),
                Command::PopOpacity,
            ],
        )
        .unwrap();
        assert_eq!(decode(&p), [expected], "bits={bits:08x}");
        assert_eq!(p.group_scratch_bytes(), 8);
        assert_eq!(p.draws().len(), 4);
        for draw in &p.draws()[..3] {
            assert_eq!(draw.opacity_bits(), None);
            assert_eq!(draw.opacity_numerator(), None);
        }
        let composite = p.draws()[3];
        assert_eq!(composite.opacity_bits(), Some(bits));
        assert_eq!(composite.opacity_numerator(), None);
        let record = &p.parameters()[3 * PARAM_STRIDE..4 * PARAM_STRIDE];
        assert_eq!(
            [
                word(record, 28),
                word(record, 29),
                word(record, 30),
                word(record, 31)
            ],
            [1, 0, 1, bits]
        );
    }
}

#[test]
fn opacity_tiny_identity_keeps_nested_regions_draws_and_all_admission_charges() {
    let make = |inner| {
        native(
            Frame::new(1, 1, 0xffffff),
            &[
                Command::PushOpacity(0.5),
                rgba(Rect::new(0., 0., 1., 1.), [255, 0, 0, 128]),
                Command::PushOpacity(inner),
                rect(0., 0., 1., 1., 0xffffff),
                Command::PopOpacity,
                Command::PopOpacity,
            ],
        )
        .unwrap()
    };
    let visible = make(0.7);
    for bits in [1, 0x007f_ffff, 0x0080_0000, 0x32ff_ffff, 0x3300_0000] {
        let tiny = make(f32::from_bits(bits));
        assert_eq!(decode(&tiny), [0xffbfbf]);
        assert_eq!(tiny.group_scratch_bytes(), 16);
        assert_eq!(tiny.group_scratch_bytes(), visible.group_scratch_bytes());
        assert_eq!(tiny.invocations(), visible.invocations());
        assert_eq!(tiny.gpu_buffer_bytes(), visible.gpu_buffer_bytes());
        assert_eq!(tiny.input_bytes(), visible.input_bytes());
        assert_eq!(tiny.parameters().len(), visible.parameters().len());
        assert_eq!(
            tiny.draws()
                .iter()
                .map(|d| (d.kind(), d.bounds()))
                .collect::<Vec<_>>(),
            visible
                .draws()
                .iter()
                .map(|d| (d.kind(), d.bounds()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            tiny.draws()
                .iter()
                .filter_map(|d| d.opacity_bits())
                .collect::<Vec<_>>(),
            [bits, 0x3f00_0000]
        );
    }
    // A non-grid inner source rounds to 900 in RGBA16 before the outer pop.
    let nested = native(
        Frame::new(1, 1, 0),
        &[
            Command::PushOpacity(0.5),
            Command::PushOpacity(f32::from_bits(0x3f33_3333)),
            rect(0., 0., 1., 1., 0x050505),
            Command::PopOpacity,
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&nested), [0x020202]);
}

#[test]
fn opacity_transparent_regions_admit_independent_first_paints_and_fixed_holes() {
    let frame = Frame::new(4, 1, 0xffffff);
    let disjoint_backing = [
        Command::PushOpacity(0.5),
        rect(0., 0., 2., 1., 0xff0000),
        rect(1., 0., 2., 1., 0x0000ff),
        Command::PopOpacity,
    ];
    let disjoint = native(frame, &disjoint_backing).unwrap();
    assert_eq!(decode(&disjoint), [0xff8080, 0x8080ff, 0x8080ff, 0xffffff]);
    assert_eq!(disjoint.group_scratch_bytes(), 24);
    let partial_first = [
        Command::PushOpacity(0.5),
        rgba(Rect::new(0., 0., 1., 1.), [255, 0, 0, 128]),
        rect(0., 0., 1., 1., 0xffffff),
        Command::PopOpacity,
    ];
    assert_eq!(
        decode(&native(frame, &partial_first).unwrap()),
        [0xffffff; 4]
    );
    let child_first = [
        Command::PushOpacity(0.5),
        Command::PushOpacity(0.5),
        rect(0., 0., 1., 1., 0xffffff),
        Command::PopOpacity,
        Command::PopOpacity,
    ];
    let child = native(frame, &child_first).unwrap();
    assert_eq!(decode(&child), [0xffffff; 4]);
    assert_eq!(child.group_scratch_bytes(), 16);
    let escaped_backing = [
        Command::PushClip(Rect::new(0., 0., 1., 1.)),
        Command::PushOpacity(0.5),
        rect(0., 0., 4., 1., 0xff0000),
        Command::PushFixed,
        rect(3., 0., 1., 1., 0x0000ff),
        Command::PopFixed,
        Command::PopOpacity,
        Command::PopClip,
    ];
    let escaped = native(frame, &escaped_backing).unwrap();
    assert_eq!(decode(&escaped), [0xff8080, 0xffffff, 0xffffff, 0x8080ff]);
    assert_eq!(escaped.group_scratch_bytes(), 32);
    let image = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[255, 0, 0, 128],
    }];
    let image_only = plan_with_masks_for_profile(
        Profile::Native,
        frame,
        &[
            Command::PushOpacity(0.5),
            Command::Image {
                rect: Rect::new(0., 0., 1., 1.),
                source: 0,
            },
            Command::PopOpacity,
        ],
        &image,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        decode(&image_only),
        [0xffbfbf, 0xffffff, 0xffffff, 0xffffff]
    );
    assert_eq!(image_only.group_scratch_bytes(), 8);
}

#[test]
fn opacity_transparent_rounding_keeps_both_root_and_nested_counterexamples() {
    let root = native(
        Frame::new(1, 1, 0x222222),
        &[
            Command::PushOpacity(192.0 / 256.0),
            rgba(Rect::new(0., 0., 1., 1.), [0, 0, 0, 55]),
            Command::PopOpacity,
        ],
    )
    .unwrap();
    // Ideal rational source-over would produce1d1d1d. Canvas's final f32
    // division is just below28.5 and must retain1c1c1c.
    assert_eq!(decode(&root), [0x1c1c1c]);
    assert_eq!(root.group_scratch_bytes(), 8);

    let nested = native(
        Frame::new(1, 1, 0x2c2c2c),
        &[
            Command::PushOpacity(127.0 / 256.0),
            rect(0., 0., 1., 1., 0x979797),
            Command::PushOpacity(0.5),
            rgba(Rect::new(0., 0., 1., 1.), [0, 0, 0, 38]),
            Command::PopOpacity,
            Command::PopOpacity,
        ],
    )
    .unwrap();
    // The inner channel stores35916 (a half tie), not ideal35915. That
    // one RGBA16 level survives the outer pop:92 rather than91.
    assert_eq!(decode(&nested), [0x5c5c5c]);
    assert_eq!(nested.group_scratch_bytes(), 16);
}

#[test]
fn opacity_transparent_parent_alpha_and_unpainted_union_holes_survive_pop() {
    let nested = native(
        Frame::new(1, 1, 0xffffff),
        &[
            Command::PushOpacity(0.5),
            Command::PushOpacity(0.5),
            rgba(Rect::new(0., 0., 1., 1.), [0, 0, 0, 128]),
            Command::PopOpacity,
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&nested), [0xdfdfdf]);
    assert_eq!(nested.group_scratch_bytes(), 16);
    let holes = native(
        Frame::new(3, 1, 0x0a0a0a),
        &[
            Command::PushOpacity(0.5),
            rect(0., 0., 1., 1., 0xff0000),
            rect(2., 0., 1., 1., 0x0000ff),
            Command::PopOpacity,
        ],
    )
    .unwrap();
    assert_eq!(decode(&holes), [0x850505, 0x0a0a0a, 0x050585]);
    assert_eq!(holes.group_scratch_bytes(), 24);
    assert_eq!(holes.draws()[1].bounds(), (0, 0, 3, 1));
    assert_eq!(holes.draws().last().unwrap().bounds(), (0, 0, 3, 1));
}

#[test]
fn opacity_transparent_glyph_coverage_and_zero_alpha_image_keep_exact_pixels() {
    let glyph = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(3, 1, 0xffffff),
        &[
            Command::PushOpacity(0.5),
            Command::Glyph {
                source: 0,
                rows: 0,
                y: 0,
                rgba: [255, 0, 0, 128],
            },
            Command::PopOpacity,
        ],
        &[],
        &[SourceMask {
            width: 2,
            height: 1,
            coverage: &[0, 128],
        }],
        &[&[0]],
    )
    .unwrap();
    assert_eq!(decode(&glyph), [0xffffff, 0xffdfdf, 0xffffff]);
    assert_eq!(glyph.group_scratch_bytes(), 16);
    assert!(glyph.has_glyphs());
    let image = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(2, 1, 0x123456),
        &[
            Command::PushOpacity(0.5),
            Command::Image {
                rect: Rect::new(0., 0., 1., 1.),
                source: 0,
            },
            Command::PopOpacity,
        ],
        &[SourceImage {
            width: 1,
            height: 1,
            rgba: &[255, 0, 255, 0],
        }],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(decode(&image), [0x123456; 2]);
    // Conservative geometry and uploads remain even when image pixels are
    // transparent. Only the compositor's alpha-zero identity skips the write.
    assert_eq!(image.group_scratch_bytes(), 8);
    assert!(image.draws().iter().any(|d| d.kind() == DrawKind::Image));
}

#[test]
fn opacity_zero_cannot_hide_invalid_source_ids_geometry_or_tail_commands() {
    for hidden in [
        Command::Image {
            rect: Rect::new(0., 0., 1., 1.),
            source: 0,
        },
        Command::Glyph {
            source: 0,
            rows: 0,
            y: 0,
            rgba: [0, 0, 0, 255],
        },
        rect(f32::NAN, 0., 1., 1., 0),
        Command::Unsupported("late unsupported"),
    ] {
        assert!(
            native(
                Frame::new(1, 1, 0),
                &[Command::PushOpacity(0.), hidden, Command::PopOpacity,]
            )
            .is_err()
        );
    }
    let malformed = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[],
    }];
    assert!(
        plan_with_masks_for_profile(
            Profile::Native,
            Frame::new(1, 1, 0),
            &[Command::PushOpacity(0.), Command::PopOpacity,],
            &malformed,
            &[],
            &[]
        )
        .is_err()
    );
}

#[test]
fn opacity_combined_scope_depth_and_crossed_pops_are_rejected() {
    let frame = Frame::new(2, 1, 0);
    let mut pushes = Vec::new();
    let mut pops = Vec::new();
    for index in 0..MAX_SCOPES {
        let (push, pop) = match index % 3 {
            0 => (Command::PushOpacity(1.), Command::PopOpacity),
            1 => (
                Command::PushClip(Rect::new(0., 0., 2., 1.)),
                Command::PopClip,
            ),
            _ => (Command::PushFixed, Command::PopFixed),
        };
        pushes.push(push);
        pops.push(pop);
    }
    let mut exact = pushes.clone();
    exact.push(rect(0., 0., 1., 1., 1));
    exact.extend(pops.iter().rev().copied());
    assert_eq!(decode(&native(frame, &exact).unwrap()), [1, 0]);
    pushes.push(Command::PushOpacity(1.));
    assert!(native(frame, &pushes).is_err());
    for commands in [
        vec![Command::PopOpacity],
        vec![Command::PushOpacity(0.5)],
        vec![Command::PushOpacity(0.), Command::PopFixed],
        vec![
            Command::PushFixed,
            Command::PushOpacity(1.),
            Command::PopFixed,
            Command::PopOpacity,
        ],
    ] {
        assert!(native(frame, &commands).is_err());
    }
}

#[test]
fn opacity_regions_are_retained_across_siblings_and_phases_without_ledger_reset() {
    let first = [
        Command::PushOpacity(0.5),
        rect(0., 0., 1., 1., 0xff0000),
        Command::PopOpacity,
    ];
    let second = [
        Command::PushOpacity(0.25),
        rect(2., 0., 2., 1., 0x0000ff),
        Command::PopOpacity,
    ];
    let target = Frame::new(8, 1, 0);
    let plan = plan_native_phases(
        target,
        &[
            Phase {
                frame: target,
                commands: &first,
            },
            Phase {
                frame: target,
                commands: &second,
            },
        ],
        &[],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(plan.group_scratch_bytes(), 24);
    assert_eq!(decode(&plan), [0x800000, 0, 0x000040, 0x000040, 0, 0, 0, 0]);
    assert_eq!(plan.draws().len(), 7);
    assert_eq!(plan.invocations(), 512);
    assert_eq!(plan.gpu_buffer_bytes(), 2120);
    assert!(
        plan_native_phases(
            target,
            &[
                Phase {
                    frame: target,
                    commands: &[Command::PushOpacity(0.5)]
                },
                Phase {
                    frame: target,
                    commands: &[Command::PopOpacity]
                },
            ],
            &[],
            &[],
            &[]
        )
        .is_err()
    );
}

#[test]
fn opacity_group_clear_and_pop_use_the_global_exact_work_boundary() {
    let frame = Frame::new(1280, 1024, 0);
    let commands = [
        Command::PushOpacity(0.5),
        rect(0., 0., 640., 712., 0xff0000),
        Command::PopOpacity,
        Command::PushOpacity(0.5),
        rect(0., 0., 160., 24., 0x0000ff),
        Command::PopOpacity,
    ];
    let exact = native(frame, &commands).unwrap();
    assert_eq!(exact.invocations(), 4_000_000);
    assert_eq!(exact.group_scratch_bytes(), 3_676_160);
    assert_eq!(exact.gpu_buffer_bytes(), 14_163_728);
    let mut over = commands.to_vec();
    over.push(rect(0., 0., 1., 1., 0));
    assert_eq!(native(frame, &over).unwrap_err(), "GPU invocation budget");
    for bits in [1, 0x3300_0000, 0x3dcc_cccd] {
        let mut raw = commands;
        raw[0] = Command::PushOpacity(f32::from_bits(bits));
        raw[3] = Command::PushOpacity(f32::from_bits(bits));
        let p = native(frame, &raw).unwrap();
        assert_eq!(p.invocations(), 4_000_000);
        assert_eq!(p.group_scratch_bytes(), 3_676_160);
        assert_eq!(p.gpu_buffer_bytes(), 14_163_728);
        let mut over = raw.to_vec();
        over.push(rect(0., 0., 1., 1., 0));
        assert_eq!(native(frame, &over).unwrap_err(), "GPU invocation budget");
    }
}

#[test]
fn opacity_keeps_probe_refusal_and_old_native_uniform_bytes() {
    let frame = Frame::new(3, 1, 0);
    for value in [0., -0., 0.5, 1., 0.1, f32::from_bits(1), f32::NAN] {
        assert_eq!(
            plan(frame, &[Command::PushOpacity(value), Command::PopOpacity]).unwrap_err(),
            "unsupported opacity profile"
        );
    }
    let commands = [rect(0., 0., 1., 1., 0xff0000)];
    let old = plan(frame, &commands).unwrap();
    let current = native(frame, &commands).unwrap();
    assert_eq!(current.group_scratch_bytes(), 0);
    assert_eq!(old.draws(), current.draws());
    assert_eq!(old.parameters(), current.parameters());
    assert_eq!(old.input_bytes(), current.input_bytes());
    for bytes in current.parameters().chunks_exact(PARAM_STRIDE) {
        assert!(bytes[64..].iter().all(|&b| b == 0));
    }
}
