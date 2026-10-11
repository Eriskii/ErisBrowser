use super::*;

fn bg(rect: Rect, tile: Rect, repeat: [bool; 2]) -> Command {
    Command::BackgroundImage {
        rect,
        tile,
        repeat,
        source: 0,
    }
}
fn source() -> SourceImage<'static> {
    SourceImage {
        width: 2,
        height: 2,
        rgba: &[
            255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255,
        ],
    }
}
fn words(plan: &Plan, start: u32, count: u32) -> Vec<u32> {
    plan.input_bytes()[start as usize * 4..(start + count) as usize * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| u32::from_le_bytes(*word))
        .collect()
}
fn tables(plan: &Plan, index: usize) -> (Vec<u32>, Vec<u32>) {
    let draw = plan.draws()[index];
    let image = draw.image.unwrap();
    (
        words(plan, image.x_base, draw.width),
        words(plan, image.y_base, draw.height),
    )
}
fn geometry(frame: Frame, rect: Rect, tile: Rect, repeat: [bool; 2]) -> BackgroundImageGeometry {
    let state = scope::CoordinateState::new_for_profile(Profile::Native, frame).unwrap();
    BackgroundImageGeometry::new(frame, rect, tile, repeat, &state).unwrap()
}

#[test]
fn four_repeat_masks_have_literal_bounds_and_separate_work_tariffs() {
    let frame = Frame::new(8, 4, 0);
    let area = Rect::new(0., 0., 8., 4.);
    let tile = Rect::new(2., 1., 4., 2.);
    for (repeat, bounds, loop_work, lut_work) in [
        ([false, false], (2, 1, 4, 2), 32, 26),
        ([true, false], (0, 1, 8, 2), 72, 46),
        ([false, true], (2, 0, 4, 4), 72, 36),
        ([true, true], (0, 0, 8, 4), 168, 56),
    ] {
        let info = geometry(frame, area, tile, repeat);
        assert_eq!(info.bounds, Some(bounds));
        assert_eq!(info.loop_work, loop_work);
        assert_eq!(info.lut_work, lut_work);
    }
}

#[test]
fn four_masks_pack_literal_axis_indices_and_exact_existing_gpu_bytes() {
    for (repeat, x, y, input_bytes, gpu_bytes) in [
        ([false, false], vec![0, 0, 1, 1], vec![0, 1], 40, 808),
        (
            [true, false],
            vec![1, 1, 0, 0, 1, 1, 0, 0],
            vec![0, 1],
            56,
            824,
        ),
        ([false, true], vec![0, 0, 1, 1], vec![1, 0, 1, 0], 48, 816),
        (
            [true, true],
            vec![1, 1, 0, 0, 1, 1, 0, 0],
            vec![1, 0, 1, 0],
            64,
            832,
        ),
    ] {
        let plan = plan_with_images(
            Frame::new(8, 4, 0),
            &[bg(
                Rect::new(0., 0., 8., 4.),
                Rect::new(2., 1., 4., 2.),
                repeat,
            )],
            &[source()],
        )
        .unwrap();
        assert_eq!(plan.draws().len(), 2);
        assert_eq!(plan.draws()[1].kind(), DrawKind::Image);
        assert_eq!(tables(&plan, 1), (x, y));
        assert_eq!(plan.input_bytes().len(), input_bytes);
        assert_eq!(plan.gpu_buffer_bytes(), gpu_bytes);
        assert_eq!(plan.parameters().len(), 512);
        assert_eq!(plan.invocations(), 128);
        assert_eq!(
            words(&plan, 0, 4),
            [0xffff0000, 0xff0000ff, 0xff00ff00, 0xffffff00]
        );
    }
}

#[test]
fn negative_fractional_origins_repeat_without_phase_reconstruction() {
    let frame = Frame::new(4, 3, 0);
    let area = Rect::new(0., 0., 4., 3.);
    let tile = Rect::new(-0.75, -0.25, 2., 2.);
    for (repeat, bounds, x, y, t, l) in [
        (
            [true; 2],
            (0, 0, 4, 3),
            vec![1, 0, 1, 0],
            vec![0, 1, 0],
            68,
            36,
        ),
        ([false; 2], (0, 0, 1, 2), vec![1], vec![0, 1], 14, 17),
    ] {
        let info = geometry(frame, area, tile, repeat);
        assert_eq!(
            (info.bounds, info.loop_work, info.lut_work),
            (Some(bounds), t, l)
        );
        let plan = plan_with_images(frame, &[bg(area, tile, repeat)], &[source()]).unwrap();
        assert_eq!(plan.draws()[1].bounds(), bounds);
        assert_eq!(tables(&plan, 1), (x, y));
    }
}

#[test]
fn fractional_clip_is_integer_containment_while_tile_is_center_containment() {
    let mut frame = Frame::new(6, 4, 0);
    frame.caller_clip = Rect::new(1.25, 0.25, 3.5, 2.5);
    let area = Rect::new(0., 0., 6., 4.);
    let info = geometry(frame, area, area, [false; 2]);
    assert_eq!(
        (info.bounds, info.loop_work, info.lut_work),
        (Some((2, 1, 3, 2)), 26, 23)
    );
    let bytes = [255; 24];
    let plan = plan_with_images(
        frame,
        &[bg(area, area, [false; 2])],
        &[SourceImage {
            width: 3,
            height: 2,
            rgba: &bytes,
        }],
    )
    .unwrap();
    assert_eq!(tables(&plan, 1), (vec![1, 1, 2], vec![0, 1]));
    for (origin, bounds, x) in [
        (0.25, Some((0, 0, 2, 1)), vec![0, 1]),
        (0.5, Some((0, 0, 2, 1)), vec![0, 1]),
        (0.75, Some((1, 0, 2, 1)), vec![0, 1]),
        (-0.75, Some((0, 0, 1, 1)), vec![1]),
        (-1.5, None, vec![]),
    ] {
        let frame = Frame::new(4, 2, 0);
        let area = Rect::new(0., 0., 4., 2.);
        let tile = Rect::new(origin, 0.5, 2., 1.);
        assert_eq!(geometry(frame, area, tile, [false; 2]).bounds, bounds);
        let bytes = [255; 8];
        let plan = plan_with_images(
            frame,
            &[bg(area, tile, [false; 2])],
            &[SourceImage {
                width: 2,
                height: 1,
                rgba: &bytes,
            }],
        )
        .unwrap();
        if bounds.is_some() {
            assert_eq!(tables(&plan, 1), (x, vec![0]));
        } else {
            assert_eq!(plan.draws().len(), 1);
            assert!(plan.input_bytes().is_empty());
        }
    }
}

#[test]
fn subnormal_nonrepeat_tile_keeps_exact_center_and_rejects_adjacent_centers() {
    let frame = Frame::new(2, 2, 0);
    let area = Rect::new(0., 0., 2., 2.);
    let tiny = f32::from_bits(1);
    let tile = Rect::new(0.5, 0.5, tiny, tiny);
    let info = geometry(frame, area, tile, [false; 2]);
    assert_eq!(
        (info.bounds, info.loop_work, info.lut_work),
        (Some((0, 0, 1, 1)), 11, 14)
    );
    let plan = plan_with_images(frame, &[bg(area, tile, [false; 2])], &[source()]).unwrap();
    assert_eq!(tables(&plan, 1), (vec![0], vec![0]));
    for origin in [f32::from_bits(0x3eff_ffff), f32::from_bits(0x3f00_0001)] {
        let info = geometry(frame, area, Rect::new(origin, 0.5, tiny, tiny), [false; 2]);
        assert_eq!((info.bounds, info.loop_work, info.lut_work), (None, 0, 0));
    }
    let repeat = plan_with_images(frame, &[bg(area, tile, [true; 2])], &[source()]).unwrap();
    assert_eq!(tables(&repeat, 1), (vec![0, 0], vec![0, 0]));
}

#[test]
fn old_image_and_screen_image_tables_remain_distinct_and_byte_exact() {
    let bytes = [255; 8];
    let image = SourceImage {
        width: 2,
        height: 1,
        rgba: &bytes,
    };
    let plan = plan_with_images(
        Frame::new(4, 1, 0),
        &[
            Command::Image {
                rect: Rect::new(0.5, 0., 2., 1.),
                source: 0,
            },
            Command::ScreenImage {
                x: 1,
                y: 0,
                source: 0,
            },
            bg(
                Rect::new(0., 0., 4., 1.),
                Rect::new(0.5, 0., 2., 1.),
                [false; 2],
            ),
        ],
        &[image],
    )
    .unwrap();
    assert_eq!(plan.draws()[1].bounds(), (0, 0, 3, 1));
    assert_eq!(tables(&plan, 1), (vec![0, 0, 1], vec![0]));
    assert_eq!(plan.draws()[2].bounds(), (1, 0, 2, 1));
    assert_eq!(tables(&plan, 2), (vec![0, 1], vec![0]));
    assert_eq!(plan.draws()[3].bounds(), (0, 0, 2, 1));
    assert_eq!(tables(&plan, 3), (vec![0, 1], vec![0]));
    assert_eq!(plan.input_bytes().len(), 48);
    assert_eq!(plan.parameters().len(), 1024);
    assert_eq!(plan.gpu_buffer_bytes(), 1104);
}

#[test]
fn identity_and_mixed_scaled_axes_have_frozen_center_indices() {
    let bytes = vec![255; 240 * 2 * 4];
    for (width, height, frame, area, x, y) in [
        (
            240,
            2,
            Frame::new(240, 4, 0),
            Rect::new(0., 0., 240., 4.),
            (0..240).collect::<Vec<u32>>(),
            vec![0, 0, 1, 1],
        ),
        (
            2,
            240,
            Frame::new(4, 240, 0),
            Rect::new(0., 0., 4., 240.),
            vec![0, 0, 1, 1],
            (0..240).collect::<Vec<u32>>(),
        ),
    ] {
        let plan = plan_with_images(
            frame,
            &[bg(area, area, [false; 2])],
            &[SourceImage {
                width,
                height,
                rgba: &bytes,
            }],
        )
        .unwrap();
        assert_eq!(tables(&plan, 1), (x, y));
    }
    for (dimension, extent, x) in [(3, 1.5, vec![1, 0, 2, 1]), (4, 3.0, vec![0, 2, 3, 0])] {
        let bytes = vec![255; dimension as usize * 4];
        let plan = plan_with_images(
            Frame::new(4, 1, 0),
            &[bg(
                Rect::new(0., 0., 4., 1.),
                Rect::new(0., 0., extent, 1.),
                [true; 2],
            )],
            &[SourceImage {
                width: dimension,
                height: 1,
                rgba: &bytes,
            }],
        )
        .unwrap();
        assert_eq!(tables(&plan, 1), (x, vec![0]));
    }
}

#[test]
fn scope_offsets_translate_paint_and_tile_once_and_restore_after_fixed() {
    let mut frame = Frame::new(6, 3, 0);
    frame.caller_clip = Rect::new(1., 0., 4., 3.);
    frame.document_offset = (1., 1.);
    let state = scope::CoordinateState::new(frame).unwrap();
    let before = (state.clip(), state.offset());
    let area = Rect::new(0., 0., 4., 2.);
    let tile = Rect::new(0., 0., 2., 2.);
    let info = BackgroundImageGeometry::new(frame, area, tile, [true; 2], &state).unwrap();
    assert_eq!(info.bounds, Some((1, 1, 4, 2)));
    assert_eq!((state.clip(), state.offset()), before);
    let plan = plan_with_images(
        frame,
        &[
            bg(area, tile, [true; 2]),
            Command::PushClip(Rect::new(0., 0., 1., 1.)),
            Command::PushFixed,
            bg(area, tile, [true; 2]),
            Command::PopFixed,
            bg(area, tile, [true; 2]),
            Command::PopClip,
        ],
        &[source()],
    )
    .unwrap();
    assert_eq!(plan.draws()[1].bounds(), (1, 1, 4, 2));
    assert_eq!(tables(&plan, 1), (vec![0, 1, 0, 1], vec![0, 1]));
    assert_eq!(plan.draws()[2].bounds(), (1, 0, 3, 2));
    assert_eq!(tables(&plan, 2), (vec![1, 0, 1], vec![0, 1]));
    assert_eq!(plan.draws()[3].bounds(), (1, 1, 1, 1));
    assert_eq!(tables(&plan, 3), (vec![0], vec![0]));
}

#[test]
fn malformed_tile_and_source_are_rejected_even_with_empty_clip_or_opacity() {
    let mut frame = Frame::new(2, 2, 0);
    frame.caller_clip = Rect::new(0., 0., 0., 0.);
    let area = Rect::new(0., 0., 0., 0.);
    for tile in [
        Rect::new(0., 0., 0., 1.),
        Rect::new(0., 0., -1., 1.),
        Rect::new(f32::NAN, 0., 1., 1.),
        Rect::new(0., 0., 1., f32::INFINITY),
        Rect::new(1_000_001., 0., 1., 1.),
    ] {
        assert!(plan_with_images(frame, &[bg(area, tile, [true; 2])], &[source()]).is_err());
    }
    let valid = bg(area, Rect::new(0., 0., 1., 1.), [true; 2]);
    assert_eq!(
        plan_with_images(frame, &[valid], &[]).unwrap_err(),
        "source ID outside validated table"
    );
    let invalid = SourceImage {
        width: 2,
        height: 2,
        rgba: &[0; 15],
    };
    assert_eq!(
        plan_with_images(frame, &[valid], &[invalid]).unwrap_err(),
        "exact source RGBA length"
    );
    let commands = [
        Command::PushOpacity(0.),
        bg(area, Rect::new(0., 0., 0., 1.), [true; 2]),
        Command::PopOpacity,
    ];
    assert!(
        plan_with_masks_for_profile(Profile::Native, frame, &commands, &[source()], &[], &[])
            .is_err()
    );
}

#[test]
fn empty_valid_paint_and_large_translated_tile_do_not_allocate_input() {
    let frame = Frame::new(2, 2, 0);
    for area in [
        Rect::new(0., 0., 0., 2.),
        Rect::new(0., 0., -1., 2.),
        Rect::new(5., 5., 1., 1.),
    ] {
        let info = geometry(frame, area, Rect::new(0., 0., 1., 1.), [true; 2]);
        assert_eq!((info.bounds, info.loop_work, info.lut_work), (None, 0, 0));
        let plan = plan_with_images(
            frame,
            &[bg(area, Rect::new(0., 0., 1., 1.), [true; 2])],
            &[source()],
        )
        .unwrap();
        assert_eq!(plan.draws().len(), 1);
        assert!(plan.input_bytes().is_empty());
    }
    let mut shifted = frame;
    shifted.document_offset = (1_000_000., 0.);
    let info = geometry(
        shifted,
        Rect::new(-1_000_000., 0., 2., 2.),
        Rect::new(1_000_000., 0., 1., 1.),
        [true; 2],
    );
    assert_eq!(
        (info.bounds, info.loop_work, info.lut_work),
        (Some((0, 0, 2, 2)), 28, 24)
    );
}

#[test]
fn shared_lut_admission_accepts_exact_cap_and_rejects_one_word_over() {
    let frame = Frame::new(8, 4, 0);
    let images = [source()];
    for (profile, exact) in [(Profile::Probe, 143_348), (Profile::Native, 589_812)] {
        for (start, accepted) in [(exact, true), (exact + 1, false)] {
            let mut draft = PlannerDraft::new(profile, frame, 1, &images, &[], &[]).unwrap();
            draft.lut_words = start;
            let state = scope::CoordinateState::new_for_profile(profile, frame).unwrap();
            let result = draft.append(
                &[bg(
                    Rect::new(0., 0., 8., 4.),
                    Rect::new(0., 0., 1., 1.),
                    [true; 2],
                )],
                state,
            );
            assert_eq!(result.is_ok(), accepted);
            if accepted {
                assert_eq!(draft.lut_words, exact + 12);
            } else {
                assert_eq!(result.unwrap_err(), "LUT entry budget");
                assert_eq!(draft.lut_words, start);
            }
        }
    }
}

#[test]
fn final_arena_gpu_cap_is_exact_and_source_cap_stays_shared() {
    let area = Rect::new(0., 0., 1., 1.);
    for (width, accepted) in [(262_012, true), (262_013, false)] {
        let bytes = vec![255; width as usize * 4];
        let result = plan_with_images(
            Frame::new(1, 1, 0),
            &[bg(area, area, [true; 2])],
            &[SourceImage {
                width,
                height: 1,
                rgba: &bytes,
            }],
        );
        assert_eq!(result.is_ok(), accepted);
        if accepted {
            let plan = result.unwrap();
            assert_eq!(plan.gpu_buffer_bytes(), 1_048_576);
            assert_eq!(tables(&plan, 1), (vec![131_006], vec![0]));
        } else {
            assert_eq!(result.unwrap_err(), "GPU buffer budget");
        }
    }
    for (width, accepted) in [(262_144, true), (262_145, false)] {
        let bytes = vec![255; width as usize * 4];
        let result = plan_with_masks_for_profile(
            Profile::Native,
            Frame::new(1, 1, 0),
            &[bg(area, area, [true; 2])],
            &[SourceImage {
                width,
                height: 1,
                rgba: &bytes,
            }],
            &[],
            &[],
        );
        assert_eq!(result.is_ok(), accepted);
        if accepted {
            assert_eq!(result.unwrap().gpu_buffer_bytes(), 1_049_372);
        } else {
            assert_eq!(result.unwrap_err(), "source byte budget");
        }
    }
}

#[test]
fn alpha_sources_and_native_groups_keep_original_packed_input_and_image_kind() {
    let bytes = [255, 0, 0, 128, 0, 0, 255, 0];
    let area = Rect::new(0., 0., 4., 1.);
    let plan = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(4, 1, 0xffffff),
        &[
            Command::PushOpacity(0.5),
            bg(area, Rect::new(0., 0., 2., 1.), [true; 2]),
            Command::PopOpacity,
        ],
        &[SourceImage {
            width: 2,
            height: 1,
            rgba: &bytes,
        }],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(words(&plan, 0, 2), [0x80ff0000, 0x000000ff]);
    let index = plan
        .draws()
        .iter()
        .position(|d| d.kind() == DrawKind::Image)
        .unwrap();
    assert!(plan.draws()[index].target_is_group());
    assert_eq!(tables(&plan, index), (vec![0, 1, 0, 1], vec![0]));
    assert!(plan.draws().iter().any(
        |d| d.kind() == DrawKind::GroupComposite && d.opacity_bits() == Some(0.5f32.to_bits())
    ));
}

#[test]
fn public_geometry_checks_arbitrary_frame_even_when_coordinate_state_is_small() {
    let frame = Frame::new(2, 2, 0);
    let state = scope::CoordinateState::new(frame).unwrap();
    let area = Rect::new(0., 0., 2., 2.);
    for (width, height) in [
        (0, 1),
        (1, 0),
        (u32::MAX, 1),
        (1, u32::MAX),
        (1281, 1),
        (1, 1025),
    ] {
        let target = Frame::new(width, height, 0);
        assert_eq!(
            BackgroundImageGeometry::new(target, area, area, [true; 2], &state).unwrap_err(),
            "viewport budget"
        );
    }
    let info =
        BackgroundImageGeometry::new(Frame::new(1280, 1024, 0), area, area, [true; 2], &state)
            .unwrap();
    assert_eq!(
        (info.bounds, info.loop_work, info.lut_work),
        (Some((0, 0, 2, 2)), 28, 24)
    );
}
