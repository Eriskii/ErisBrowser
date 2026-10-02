use super::*;

fn phased(target: Frame, phases: &[Phase<'_>]) -> Result<Plan> {
    plan_native_phases(target, phases, &[], &[], &[])
}

fn same_payload(a: &Plan, b: &Plan) {
    assert_eq!(a.profile(), b.profile());
    assert_eq!(a.draws(), b.draws());
    assert_eq!(a.parameters(), b.parameters());
    assert_eq!(a.input_bytes(), b.input_bytes());
    assert_eq!(a.invocations(), b.invocations());
    assert_eq!(a.conversion_invocations(), b.conversion_invocations());
    assert_eq!(a.conversion_buffer_bytes(), b.conversion_buffer_bytes());
    assert_eq!(a.gpu_buffer_bytes(), b.gpu_buffer_bytes());
}

#[test]
fn one_phase_preserves_native_payload_with_images_masks_and_offsets() {
    let target = Frame::new(8, 6, 0x123456);
    let mut frame = target;
    frame.caller_clip = Rect::new(1., 1., 6., 4.);
    frame.document_offset = (0.25, 1.);
    frame.viewport_offset = (2., 2.);
    let commands = [
        Command::Image {
            rect: Rect::new(0., 0., 5., 3.),
            source: 0,
        },
        Command::PushFixed,
        rect(0., 0., 2., 2., 0xabcdef),
        Command::PopFixed,
        Command::Glyph {
            source: 0,
            rows: 0,
            y: 1,
            rgba: [12, 34, 56, 128],
        },
    ];
    let images = [SourceImage {
        width: 2,
        height: 1,
        rgba: &[1, 2, 3, 4, 5, 6, 7, 8],
    }];
    let masks = [SourceMask {
        width: 2,
        height: 1,
        coverage: &[0, 202],
    }];
    let rows: [&[i32]; 1] = [&[2]];
    let old =
        plan_with_masks_for_profile(Profile::Native, frame, &commands, &images, &masks, &rows)
            .unwrap();
    let new = plan_native_phases(
        target,
        &[Phase {
            frame,
            commands: &commands,
        }],
        &images,
        &masks,
        &rows,
    )
    .unwrap();
    same_payload(&old, &new);
    assert_eq!(new.frame().caller_clip, target.caller_clip);
    assert_eq!(new.frame().document_offset, (0., 0.));
}

#[test]
fn phases_clear_once_preserve_draw_order_and_reserve_conversion_once() {
    let target = Frame::new(2, 1, 0x123456);
    let first = [rect(0., 0., 2., 1., 0xff0000)];
    let second = [rect(1., 0., 1., 1., 0x0000ff)];
    let p = phased(
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
    )
    .unwrap();
    assert_eq!(p.draws.len(), 3);
    assert_eq!(
        p.draws
            .iter()
            .map(|d| (d.bounds(), d.color))
            .collect::<Vec<_>>(),
        [
            ((0, 0, 2, 1), 0xff123456),
            ((0, 0, 2, 1), 0xffff0000),
            ((1, 0, 1, 1), 0xff0000ff),
        ]
    );
    assert_eq!(p.raster_invocations(), 192);
    assert_eq!(p.conversion_invocations(), 64);
    assert_eq!(p.invocations(), 256);
    assert_eq!(p.parameters().len(), 3 * PARAM_STRIDE);
}

#[test]
fn each_phase_fixed_scope_escapes_only_its_own_caller_clip_then_restores() {
    let target = Frame::new(12, 8, 0);
    let mut first = target;
    first.caller_clip = Rect::new(2., 1., 6., 5.);
    first.document_offset = (2., 1.);
    first.viewport_offset = (3., 2.);
    let mut second = target;
    second.caller_clip = Rect::new(8., 0., 4., 8.);
    second.document_offset = (8., 0.);
    second.viewport_offset = (9., 1.);
    let commands = [
        Command::PushClip(Rect::new(0., 0., 1., 1.)),
        Command::PushFixed,
        rect(0., 0., 8., 8., 1),
        Command::PopFixed,
        rect(0., 0., 8., 8., 2),
        Command::PopClip,
    ];
    let p = phased(
        target,
        &[
            Phase {
                frame: first,
                commands: &commands,
            },
            Phase {
                frame: second,
                commands: &commands,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        p.draws
            .iter()
            .skip(1)
            .map(|d| d.bounds())
            .collect::<Vec<_>>(),
        [(3, 2, 5, 4), (2, 1, 1, 1), (9, 1, 3, 7), (8, 0, 1, 1)]
    );
}

#[test]
fn scopes_cannot_be_repaired_by_a_later_phase() {
    let target = Frame::new(2, 2, 0);
    for (push, pop) in [
        (Command::PushFixed, Command::PopFixed),
        (
            Command::PushClip(Rect::new(0., 0., 1., 1.)),
            Command::PopClip,
        ),
    ] {
        assert_eq!(
            phased(
                target,
                &[
                    Phase {
                        frame: target,
                        commands: &[push]
                    },
                    Phase {
                        frame: target,
                        commands: &[pop]
                    }
                ]
            )
            .unwrap_err(),
            "unclosed scope"
        );
    }
    assert_eq!(
        phased(
            target,
            &[
                Phase {
                    frame: target,
                    commands: &[]
                },
                Phase {
                    frame: target,
                    commands: &[Command::PopFixed]
                }
            ]
        )
        .unwrap_err(),
        "mismatched fixed scope"
    );
    assert_eq!(
        phased(
            target,
            &[
                Phase {
                    frame: target,
                    commands: &[rect(0., 0., 1., 1., 1)]
                },
                Phase {
                    frame: target,
                    commands: &[Command::Unsupported("late phase")]
                }
            ]
        )
        .unwrap_err(),
        "unsupported late phase"
    );
}

#[test]
fn canonical_target_compatible_phase_frames_and_four_phase_ceiling_are_required() {
    let target = Frame::new(4, 3, 0x123456);
    let empty = Phase {
        frame: target,
        commands: &[],
    };
    assert!(phased(target, &[empty; MAX_NATIVE_PHASES]).is_ok());
    assert_eq!(
        phased(target, &[empty; 5]).unwrap_err(),
        "native phase budget"
    );
    for field in 0..3 {
        let mut bad = target;
        match field {
            0 => bad.caller_clip.width = 3.,
            1 => bad.document_offset.0 = 1.,
            _ => bad.viewport_offset.1 = 1.,
        }
        assert_eq!(
            phased(bad, &[]).unwrap_err(),
            "native target must have full clip and zero offsets"
        );
    }
    for field in 0..3 {
        let mut bad = target;
        match field {
            0 => bad.width += 1,
            1 => bad.height += 1,
            _ => bad.clear += 1,
        }
        assert_eq!(
            phased(
                target,
                &[Phase {
                    frame: bad,
                    commands: &[]
                }]
            )
            .unwrap_err(),
            "native phase target mismatch"
        );
    }
    let mut invalid = target;
    invalid.caller_clip.x = f32::NAN;
    let malformed = [SourceImage {
        width: 0,
        height: 1,
        rgba: &[],
    }];
    assert_eq!(
        plan_native_phases(
            target,
            &[
                empty,
                Phase {
                    frame: invalid,
                    commands: &[]
                }
            ],
            &malformed,
            &[],
            &[]
        )
        .unwrap_err(),
        "invalid or over-limit rectangle"
    );
}

#[test]
fn zero_phases_are_clear_only_but_still_validate_unused_sources() {
    let target = Frame::new(65, 2, 0xabcdef);
    let old = plan_with_masks_for_profile(Profile::Native, target, &[], &[], &[], &[]).unwrap();
    let p = phased(target, &[]).unwrap();
    same_payload(&old, &p);
    assert_eq!(p.draws.len(), 1);
    assert_eq!(p.draws[0].color, 0xffabcdef);
    let malformed = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[],
    }];
    assert_eq!(
        plan_native_phases(target, &[], &malformed, &[], &[]).unwrap_err(),
        "exact source RGBA length"
    );
    let too_many = [SourceMask {
        width: 0,
        height: 0,
        coverage: &[],
    }; 257];
    assert_eq!(
        plan_native_phases(target, &[], &[], &too_many, &[]).unwrap_err(),
        "combined source entry budget"
    );
    let row = [0; 1024];
    let rows: [&[i32]; 65] = [&row; 65];
    assert_eq!(
        plan_native_phases(target, &[], &[], &[], &rows).unwrap_err(),
        "row entry budget"
    );
}

#[test]
fn original_commands_are_globally_bounded_even_when_no_phase_emits_a_draw() {
    let target = Frame::new(1, 1, 0);
    let commands = [rect(0., 0., 0., 0., 0); 65];
    let phase = Phase {
        frame: target,
        commands: &commands[..64],
    };
    let p = phased(target, &[phase; 4]).unwrap();
    assert_eq!(p.draws.len(), 1);
    assert_eq!(
        phased(
            target,
            &[
                Phase {
                    frame: target,
                    commands: &commands
                },
                phase,
                phase,
                phase
            ]
        )
        .unwrap_err(),
        "command budget"
    );
}

#[test]
fn every_phase_shares_the_four_million_work_ceiling() {
    let target = Frame::new(1000, 1000, 0);
    let commands = [rect(0., 0., 1000., 1000., 1)];
    let phase = Phase {
        frame: target,
        commands: &commands,
    };
    let p = phased(target, &[phase, phase]).unwrap();
    assert_eq!(p.invocations(), 4_000_000);
    assert_eq!(p.raster_invocations(), 3_000_000);
    assert_eq!(
        phased(
            target,
            &[
                phase,
                phase,
                Phase {
                    frame: target,
                    commands: &[rect(0., 0., 1., 1., 2)]
                }
            ]
        )
        .unwrap_err(),
        "GPU invocation budget"
    );
}

#[test]
fn global_source_packing_lookup_tables_and_storage_are_not_reset_per_phase() {
    let target = Frame::new(3, 1, 0);
    let commands = [
        Command::Image {
            rect: Rect::new(0., 0., 3., 1.),
            source: 0,
        },
        Command::Glyph {
            source: 0,
            rows: 0,
            y: 0,
            rgba: [1, 2, 3, 255],
        },
    ];
    let images = [SourceImage {
        width: 2,
        height: 1,
        rgba: &[1, 2, 3, 4, 5, 6, 7, 8],
    }];
    let masks = [SourceMask {
        width: 1,
        height: 1,
        coverage: &[202],
    }];
    let rows: [&[i32]; 1] = [&[1]];
    let p = plan_native_phases(
        target,
        &[Phase {
            frame: target,
            commands: &commands,
        }; 4],
        &images,
        &masks,
        &rows,
    )
    .unwrap();
    let flat = commands.repeat(4);
    let old = plan_with_masks_for_profile(Profile::Native, target, &flat, &images, &masks, &rows)
        .unwrap();
    same_payload(&old, &p);
    // Four source/coverage/row words packed once, plus four 3+1 lookup tables.
    assert_eq!(p.input.len(), 80);
    assert_eq!(p.draws.len(), 9);
    assert_eq!(p.gpu_buffer_bytes(), 2668);
    assert_eq!(p.invocations(), 640);
    for (index, draw) in p.draws.iter().skip(1).step_by(2).enumerate() {
        let image = draw.image.unwrap();
        assert_eq!(image.source_base, 0);
        assert_eq!(image.x_base, 4 + index as u32 * 4);
    }
}

#[test]
fn shared_draft_applies_native_lut_and_byte_refusal_to_prior_phase_work() {
    // The public command/work bounds can preclude reaching the structural LUT
    // and byte maxima. Seed only these private counters to test their cutpoints.
    let frame = Frame::new(1, 1, 0);
    let sources = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[0, 0, 0, 255],
    }];
    let command = [Command::Image {
        rect: Rect::new(0., 0., 1., 1.),
        source: 0,
    }];
    let mut draft = PlannerDraft::new(Profile::Native, frame, 2, &sources, &[], &[]).unwrap();
    draft.lut_words = Profile::Native.max_lut_entries() - 2;
    draft
        .append(
            &command,
            scope::CoordinateState::new_for_profile(Profile::Native, frame).unwrap(),
        )
        .unwrap();
    assert_eq!(draft.lut_words, Profile::Native.max_lut_entries());
    assert_eq!(
        draft
            .append(
                &command,
                scope::CoordinateState::new_for_profile(Profile::Native, frame).unwrap()
            )
            .unwrap_err(),
        "LUT entry budget"
    );

    let mut draft = PlannerDraft::new(Profile::Native, frame, 1, &sources, &[], &[]).unwrap();
    draft
        .append(
            &command,
            scope::CoordinateState::new_for_profile(Profile::Native, frame).unwrap(),
        )
        .unwrap();
    draft.source_words = Profile::Native.max_gpu_buffer_bytes() as usize / 4;
    assert_eq!(draft.finish().unwrap_err(), "GPU buffer budget");
}

#[test]
fn original_single_stream_validation_precedence_is_preserved() {
    let malformed = [SourceImage {
        width: 0,
        height: 1,
        rgba: &[],
    }];
    let many = [rect(0., 0., 0., 0., 0); 257];
    let check = |frame, commands: &[Command]| {
        plan_with_masks_for_profile(Profile::Probe, frame, commands, &malformed, &[], &[])
            .unwrap_err()
    };
    assert_eq!(
        check(Frame::new(321, 1, u32::MAX), &many),
        "viewport budget"
    );
    assert_eq!(
        check(Frame::new(1, 1, u32::MAX), &many),
        "clear color must be packed RGB"
    );
    let mut frame = Frame::new(1, 1, 0);
    frame.caller_clip.x = f32::NAN;
    assert_eq!(check(frame, &many), "command budget");
    assert_eq!(check(frame, &[]), "invalid or over-limit rectangle");
    frame = Frame::new(1, 1, 0);
    frame.document_offset.0 = f32::INFINITY;
    assert_eq!(check(frame, &[]), "invalid offset");
    assert_eq!(
        check(Frame::new(1, 1, 0), &[Command::Unsupported("last")]),
        "source dimensions"
    );
}

#[test]
fn retained_cpu_bytes_use_actual_capacities_and_check_overflow() {
    let mut p = phased(Frame::new(1, 1, 0), &[]).unwrap();
    let initial = p.retained_cpu_bytes().unwrap();
    p.draws.try_reserve_exact(7).unwrap();
    p.parameters.try_reserve_exact(19).unwrap();
    p.input.try_reserve_exact(23).unwrap();
    assert_eq!(
        p.retained_cpu_bytes().unwrap(),
        std::mem::size_of::<Plan>()
            + p.draws.capacity() * std::mem::size_of::<Draw>()
            + p.parameters.capacity()
            + p.input.capacity()
    );
    assert!(p.retained_cpu_bytes().unwrap() > initial);
    for capacities in [(usize::MAX, 0, 0), (0, usize::MAX, 0), (0, 0, usize::MAX)] {
        assert_eq!(
            Plan::retained_capacity_bytes(capacities.0, capacities.1, capacities.2).unwrap_err(),
            "retained plan CPU byte overflow"
        );
    }
    let peak = native_planner_metadata_peak_bytes().unwrap();
    let draw_overlap =
        (MAX_COMMANDS + 1) * (std::mem::size_of::<PendingDraw>() + std::mem::size_of::<Draw>());
    let scopes = MAX_NATIVE_PHASES * MAX_SCOPES * std::mem::size_of::<scope::Scope>();
    assert!(peak >= std::mem::size_of::<Plan>() + draw_overlap + scopes);
}
