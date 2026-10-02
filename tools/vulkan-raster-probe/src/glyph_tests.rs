use super::*;

fn glyph(source: u32, rows: u32, y: i32, alpha: u8) -> Command {
    Command::Glyph {
        source,
        rows,
        y,
        rgba: [200, 20, 40, alpha],
    }
}
fn words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| u32::from_le_bytes(*v))
        .collect()
}
fn empty_mask() -> SourceMask<'static> {
    SourceMask {
        width: 0,
        height: 0,
        coverage: &[],
    }
}

#[test]
fn glyph_packing_keeps_original_sources_and_signed_rows_in_separate_ranges() {
    let masks = [SourceMask {
        width: 2,
        height: 2,
        coverage: &[0, 1, 128, 255],
    }];
    let images = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[9, 8, 7, 6],
    }];
    let plan = plan_with_masks(
        Frame::new(4, 4, 0),
        &[glyph(0, 0, 1, 128)],
        &images,
        &masks,
        &[&[-1, 1]],
    )
    .unwrap();
    assert_eq!(
        words(plan.input_bytes()),
        [0x0609_0807, 0, 1, 128, 255, u32::MAX, 1]
    );
    assert_eq!(plan.draws()[1].bounds(), (0, 1, 3, 2));
    assert_eq!(plan.draws()[1].kind(), DrawKind::Glyph);
    assert_eq!(
        words(&plan.parameters()[PARAM_STRIDE..PARAM_STRIDE + 64]),
        [0, 1, 3, 2, 4, 4, 0x80c8_1428, 0, 1, 2, 2, 4, 5, 1, 2, 0]
    );
    assert_eq!((plan.gpu_buffer_bytes(), plan.invocations()), (668, 128));
    assert!(plan.has_input());
    assert!(plan.has_glyphs());
    assert!(!plan.has_images());
    assert!(
        plan.parameters()[PARAM_STRIDE + 64..]
            .iter()
            .all(|byte| *byte == 0)
    );
}

#[test]
fn glyph_addition_retains_literal_image_only_parameter_and_input_layout() {
    let source = SourceImage {
        width: 2,
        height: 1,
        rgba: &[1, 2, 3, 4, 5, 6, 7, 8],
    };
    let plan = plan_with_images(
        Frame::new(3, 1, 0),
        &[Command::Image {
            rect: Rect::new(0.0, 0.0, 3.0, 1.0),
            source: 0,
        }],
        &[source],
    )
    .unwrap();
    assert_eq!(
        words(plan.input_bytes()),
        [0x0401_0203, 0x0805_0607, 0, 0, 1, 0]
    );
    assert_eq!(
        words(&plan.parameters()[PARAM_STRIDE..PARAM_STRIDE + 64]),
        [0, 0, 3, 1, 3, 1, 0, 0, 0, 2, 1, 2, 2, 5, 3, 1]
    );
    assert_eq!(plan.gpu_buffer_bytes(), 560);
    assert!(plan.has_images() && plan.has_input());
    assert!(!plan.has_glyphs());
}

#[test]
fn glyph_and_image_draw_order_and_index_namespaces_remain_distinct() {
    let image = SourceImage {
        width: 1,
        height: 1,
        rgba: &[10, 20, 30, 255],
    };
    let mask = SourceMask {
        width: 1,
        height: 1,
        coverage: &[129],
    };
    let commands = [
        glyph(0, 0, 0, 128),
        Command::Image {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            source: 0,
        },
        glyph(0, 0, 0, 1),
    ];
    let plan = plan_with_masks(Frame::new(1, 1, 0), &commands, &[image], &[mask], &[&[0]]).unwrap();
    assert_eq!(
        plan.draws().iter().map(|d| d.kind()).collect::<Vec<_>>(),
        [
            DrawKind::Rectangle,
            DrawKind::Glyph,
            DrawKind::Image,
            DrawKind::Glyph
        ]
    );
    assert_eq!(words(plan.input_bytes()), [0xff0a_141e, 129, 0, 0, 0]);
    assert_eq!(plan.draws[1].image.unwrap().source_base, 1);
    assert_eq!(plan.draws[2].image.unwrap().source_base, 0);
    assert_eq!(plan.draws[3].image.unwrap().source_base, 1);
    assert!(plan.has_images() && plan.has_glyphs());
}

#[test]
fn glyph_reached_ids_and_height_validate_before_alpha_or_clip_culling() {
    let masks = [SourceMask {
        width: 1,
        height: 1,
        coverage: &[255],
    }];
    for clip in [Rect::new(0.0, 0.0, 1.0, 1.0), Rect::new(0.0, 0.0, 0.0, 0.0)] {
        let mut frame = Frame::new(1, 1, 0);
        frame.caller_clip = clip;
        for alpha in [0, 255] {
            assert!(
                plan_with_masks(frame, &[glyph(u32::MAX, 0, 0, alpha)], &[], &masks, &[&[0]])
                    .is_err()
            );
            assert!(
                plan_with_masks(frame, &[glyph(0, u32::MAX, 0, alpha)], &[], &masks, &[&[0]])
                    .is_err()
            );
            assert!(plan_with_masks(frame, &[glyph(0, 0, 0, alpha)], &[], &masks, &[&[]]).is_err());
        }
    }
}

#[test]
fn glyph_unused_sources_and_row_tables_are_validated_without_a_draw() {
    let f = Frame::new(1, 1, 0);
    for mask in [
        SourceMask {
            width: 0,
            height: 1,
            coverage: &[],
        },
        SourceMask {
            width: 1,
            height: 0,
            coverage: &[],
        },
        SourceMask {
            width: 1,
            height: 1,
            coverage: &[],
        },
        SourceMask {
            width: 0,
            height: 0,
            coverage: &[0],
        },
    ] {
        assert!(plan_with_masks(f, &[], &[], &[mask], &[]).is_err());
    }
    assert!(
        plan_with_masks(
            f,
            &[],
            &[SourceImage {
                width: 1,
                height: 1,
                rgba: &[]
            }],
            &[empty_mask()],
            &[&[]]
        )
        .is_err()
    );
    let too_tall = vec![0; 1025];
    assert!(plan_with_masks(f, &[], &[], &[], &[&too_tall]).is_err());
    let tables: Vec<&[i32]> = vec![&[]; 257];
    assert!(plan_with_masks(f, &[], &[], &[], &tables).is_err());
}

#[test]
fn glyph_empty_canonical_sources_count_but_create_no_gpu_input() {
    let f = Frame::new(1, 1, 0x102030);
    let p = plan_with_masks(
        f,
        &[glyph(0, 0, i32::MIN, 255)],
        &[],
        &[empty_mask()],
        &[&[]],
    )
    .unwrap();
    assert_eq!(p.draws().len(), 1);
    assert_eq!(p.gpu_buffer_bytes(), 264);
    assert!(!p.has_input() && !p.has_images() && !p.has_glyphs());
    let tables: Vec<&[i32]> = vec![&[]; 256];
    assert!(plan_with_masks(f, &[], &[], &vec![empty_mask(); 256], &tables).is_ok());
    assert!(plan_with_masks(f, &[], &[], &vec![empty_mask(); 257], &[]).is_err());
    let tiny = SourceImage {
        width: 1,
        height: 1,
        rgba: &[0; 4],
    };
    assert!(plan_with_masks(f, &[], &vec![tiny; 255], &[empty_mask()], &[]).is_ok());
    assert!(plan_with_masks(f, &[], &vec![tiny; 256], &[empty_mask()], &[]).is_err());
}

#[test]
fn glyph_mask_axis_area_and_aggregate_coverage_caps_are_distinct() {
    let f = Frame::new(1, 1, 0);
    let bytes = vec![0; 262_144];
    let full = SourceMask {
        width: 1024,
        height: 256,
        coverage: &bytes,
    };
    assert_eq!(mask_storage(&[full], &[]).unwrap(), (262_144, 0));
    assert!(plan_with_masks(f, &[], &[], &[full], &[]).is_ok());
    let extra = SourceMask {
        width: 1,
        height: 1,
        coverage: &[0],
    };
    assert!(mask_storage(&[full, extra], &[]).is_err());
    let axis = vec![0; 1025];
    assert!(
        mask_storage(
            &[SourceMask {
                width: 1025,
                height: 1,
                coverage: &axis
            }],
            &[]
        )
        .is_err()
    );
    let area = vec![0; 1024 * 257];
    assert!(
        mask_storage(
            &[SourceMask {
                width: 1024,
                height: 257,
                coverage: &area
            }],
            &[]
        )
        .is_err()
    );
}

#[test]
fn glyph_row_table_entry_total_is_bounded_including_unused_tables() {
    let row = [0; 1024];
    let mut tables: Vec<&[i32]> = vec![&row; 64];
    assert_eq!(mask_storage(&[], &tables).unwrap(), (0, 65_536));
    assert!(plan_with_masks(Frame::new(1, 1, 0), &[], &[], &[], &tables).is_ok());
    tables.push(&[0]);
    assert!(mask_storage(&[], &tables).is_err());
}

#[test]
fn glyph_exact_combined_gpu_byte_ceiling_counts_all_mask_and_row_words() {
    // 1023*255 coverage cells +255 used rows +894 unused rows =262014 words.
    // At 1x1, two targets (8 bytes) +two records (512) leave exactly 1048056.
    let pixels = vec![1; 1023 * 255];
    let mask = SourceMask {
        width: 1023,
        height: 255,
        coverage: &pixels,
    };
    let rows = [0; 255];
    let spare = [0; 895];
    let f = Frame::new(1, 1, 0);
    let p = plan_with_masks(
        f,
        &[glyph(0, 0, 0, 255)],
        &[],
        &[mask],
        &[&rows, &spare[..894]],
    )
    .unwrap();
    assert_eq!(p.input_bytes().len(), 1_048_056);
    assert_eq!(p.gpu_buffer_bytes(), MAX_GPU_BUFFER_BYTES);
    assert_eq!(p.draws().len(), 2);
    assert!(plan_with_masks(f, &[glyph(0, 0, 0, 255)], &[], &[mask], &[&rows, &spare]).is_err());
}

#[test]
fn glyph_repeated_dispatches_share_masks_but_pay_full_invocation_work() {
    let data = vec![255; 320 * 240];
    let mask = SourceMask {
        width: 320,
        height: 240,
        coverage: &data,
    };
    let rows = [0; 240];
    let f = Frame::new(320, 240, 0);
    let p = plan_with_masks(f, &vec![glyph(0, 0, 0, 255); 51], &[], &[mask], &[&rows]).unwrap();
    assert_eq!(p.invocations(), 3_993_600);
    assert_eq!(p.input_bytes().len(), (320 * 240 + 240) * 4);
    assert_eq!(p.gpu_buffer_bytes(), 935_872);
    assert!(plan_with_masks(f, &vec![glyph(0, 0, 0, 255); 52], &[], &[mask], &[&rows]).is_err());
}

#[test]
fn glyph_fractional_clip_uses_integer_origins_and_both_upper_edges() {
    let mask = SourceMask {
        width: 3,
        height: 2,
        coverage: &[255; 6],
    };
    let mut f = Frame::new(4, 3, 0);
    f.caller_clip = Rect::new(0.5, 0.5, 2.0, 1.5);
    let p = plan_with_masks(f, &[glyph(0, 0, 0, 128)], &[], &[mask], &[&[0, 0]]).unwrap();
    assert_eq!(p.draws()[1].bounds(), (1, 1, 2, 1));
}

#[test]
fn glyph_signed_extremes_and_negative_y_clip_without_overflow() {
    let mask = SourceMask {
        width: 2,
        height: 3,
        coverage: &[255; 6],
    };
    let rows = [i32::MIN, i32::MAX, -1];
    let f = Frame::new(2, 2, 0);
    let p = plan_with_masks(f, &[glyph(0, 0, -2, 255)], &[], &[mask], &[&rows]).unwrap();
    assert_eq!(p.draws()[1].bounds(), (0, 0, 1, 1));
    assert_eq!(
        words(&p.parameters()[PARAM_STRIDE..PARAM_STRIDE + 64])[13],
        (-2i32) as u32
    );
    for y in [i32::MIN, i32::MAX] {
        let p = plan_with_masks(f, &[glyph(0, 0, y, 255)], &[], &[mask], &[&rows]).unwrap();
        assert_eq!(p.draws().len(), 1);
        assert!(!p.has_input());
    }
    for x in [i32::MIN, i32::MAX] {
        let p = plan_with_masks(f, &[glyph(0, 0, 0, 255)], &[], &[mask], &[&[x; 3]]).unwrap();
        assert_eq!(p.draws().len(), 1);
    }
}

#[test]
fn glyph_absolute_origins_ignore_offsets_but_obey_fixed_clip_restore() {
    let mask = SourceMask {
        width: 2,
        height: 1,
        coverage: &[255; 2],
    };
    let mut f = Frame::new(4, 3, 0);
    f.document_offset = (100.0, -100.0);
    f.viewport_offset = (500.0, 500.0);
    let commands = [
        Command::PushClip(Rect::new(-100.0, 100.0, 2.0, 2.0)),
        glyph(0, 0, 1, 255),
        Command::PushFixed,
        glyph(0, 0, 1, 255),
        Command::PopFixed,
        glyph(0, 0, 1, 255),
        Command::PopClip,
    ];
    let p = plan_with_masks(f, &commands, &[], &[mask], &[&[1]]).unwrap();
    assert_eq!(p.draws()[1].bounds(), (1, 1, 1, 1));
    assert_eq!(p.draws()[2].bounds(), (1, 1, 2, 1));
    assert_eq!(p.draws()[3].bounds(), (1, 1, 1, 1));
}

#[test]
fn glyph_shared_scope_state_preserves_typed_nesting_and_float_order() {
    let mut f = Frame::new(8, 4, 0);
    f.caller_clip = Rect::new(1.0, 0.5, 5.0, 3.0);
    f.document_offset = (0.0, -1.0);
    f.viewport_offset = (2.0, 1.0);
    let mut state = scope::CoordinateState::new(f).unwrap();
    assert_eq!(state.clip(), f.caller_clip);
    assert!(!state.apply(&glyph(0, 0, 0, 255)).unwrap());
    state
        .apply(&Command::PushClip(Rect::new(2.0, 1.0, 1.0, 2.0)))
        .unwrap();
    assert_eq!(state.clip(), Rect::new(2.0, 0.5, 1.0, 1.5));
    state.apply(&Command::PushFixed).unwrap();
    assert_eq!((state.clip(), state.offset()), (f.caller_clip, (2.0, 1.0)));
    state.apply(&Command::PopFixed).unwrap();
    assert_eq!(state.offset(), (0.0, -1.0));
    assert_eq!(state.clip(), Rect::new(2.0, 0.5, 1.0, 1.5));
    assert!(state.finish().is_err());
    state.apply(&Command::PopClip).unwrap();
    assert!(state.finish().is_ok());
    assert!(state.apply(&Command::PopFixed).is_err());
    let mut state = scope::CoordinateState::new(f).unwrap();
    for _ in 0..MAX_SCOPES {
        state.apply(&Command::PushFixed).unwrap();
    }
    assert!(
        state
            .apply(&Command::PushClip(Rect::new(0.0, 0.0, 1.0, 1.0)))
            .is_err()
    );
}

#[test]
fn glyph_zero_coverage_is_not_scanned_to_suppress_dispatch() {
    let mask = SourceMask {
        width: 2,
        height: 1,
        coverage: &[0, 0],
    };
    let f = Frame::new(2, 1, 0);
    let p = plan_with_masks(f, &[glyph(0, 0, 0, 255)], &[], &[mask], &[&[0]]).unwrap();
    assert_eq!(p.draws().len(), 2);
    assert!(p.has_input());
    let p = plan_with_masks(f, &[glyph(0, 0, 0, 0)], &[], &[mask], &[&[0]]).unwrap();
    assert_eq!(p.draws().len(), 1);
    assert!(!p.has_input());
}

#[test]
fn glyph_returned_plan_owns_coverage_and_rows_without_source_lifetimes() {
    let mut coverage = [7, 9];
    let mut rows = [-1];
    let p = plan_with_masks(
        Frame::new(2, 1, 0),
        &[glyph(0, 0, 0, 255)],
        &[],
        &[SourceMask {
            width: 2,
            height: 1,
            coverage: &coverage,
        }],
        &[&rows],
    )
    .unwrap();
    coverage.fill(255);
    rows.fill(999);
    assert_eq!(words(p.input_bytes()), [7, 9, u32::MAX]);
    assert_eq!(p.draws()[1].bounds(), (0, 0, 1, 1));
}
