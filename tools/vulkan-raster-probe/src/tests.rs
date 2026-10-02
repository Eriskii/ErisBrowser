use super::*;

#[test]
fn independent_pixel_fixtures_agree_with_integer_plan() {
    for fixture in fixtures::fixtures() {
        let p = fixture.plan().unwrap();
        assert_eq!(
            fixture.expected.len(),
            (p.frame.width * p.frame.height) as usize
        );
        let mut actual = vec![0xdeadbeef; fixture.expected.len()];
        for d in &p.draws {
            assert_eq!(d.color >> 24, 255);
            for y in d.y..d.y + d.height {
                for x in d.x..d.x + d.width {
                    actual[(y * p.frame.width + x) as usize] = d.color & 0x00ff_ffff;
                }
            }
        }
        assert_eq!(actual, fixture.expected, "{}", fixture.name);
    }
}
#[test]
fn fractional_rectangle_and_clip_left_edges_are_deliberately_different() {
    let p = plan(Frame::new(8, 8, 0), &[rect(1.8, 2.8, 0.1, 0.1, 1)]).unwrap();
    assert_eq!(
        p.draws[1],
        Draw {
            x: 1,
            y: 2,
            width: 1,
            height: 1,
            color: 0xff00_0001,
            image: None
        }
    );
    let p = plan(
        Frame::new(8, 8, 0),
        &[
            Command::PushClip(Rect::new(1.8, 2.8, 1., 1.)),
            rect(0., 0., 8., 8., 1),
            Command::PopClip,
        ],
    )
    .unwrap();
    assert_eq!(
        p.draws[1],
        Draw {
            x: 2,
            y: 3,
            width: 1,
            height: 1,
            color: 0xff00_0001,
            image: None
        }
    );
}
#[test]
fn unsupported_content_is_rejected_even_after_hidden_or_empty_content() {
    let f = Frame::new(8, 8, 0);
    for cmd in [
        Command::Unsupported("text"),
        Command::Unsupported("image"),
        Command::Unsupported("opacity"),
        Command::Rect {
            rect: Rect::new(1., 1., 2., 2.),
            rgba: [0, 0, 0, 255],
            radius: 1.,
        },
    ] {
        assert!(plan(f, &[rect(0., 0., 8., 8., 1), cmd]).is_err());
    }
}
#[test]
fn invalid_nonfinite_coordinates_offsets_and_dimensions_are_rejected() {
    for v in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        MAX_COORDINATE + 1.,
    ] {
        for rect in [
            Rect::new(v, 0., 1., 1.),
            Rect::new(0., v, 1., 1.),
            Rect::new(0., 0., v, 1.),
            Rect::new(0., 0., 1., v),
        ] {
            assert!(
                plan(
                    Frame::new(8, 8, 0),
                    &[Command::PushClip(rect), Command::PopClip]
                )
                .is_err()
            );
            assert!(
                plan(
                    Frame::new(8, 8, 0),
                    &[Command::Rect {
                        rect,
                        rgba: [0, 0, 0, 255],
                        radius: 0.
                    }]
                )
                .is_err()
            );
        }
        let mut f = Frame::new(8, 8, 0);
        f.document_offset = (v, 0.);
        assert!(plan(f, &[]).is_err());
        f.document_offset = (0., 0.);
        f.viewport_offset = (0., v);
        assert!(plan(f, &[]).is_err());
        assert!(
            plan(
                Frame::new(8, 8, 0),
                &[Command::Rect {
                    rect: Rect::new(0., 0., 1., 1.),
                    rgba: [0, 0, 0, 255],
                    radius: v
                }]
            )
            .is_err()
        );
    }
    for (w, h) in [(0, 1), (1, 0), (321, 1), (1, 241), (u32::MAX, u32::MAX)] {
        assert!(plan(Frame::new(w, h, 0), &[]).is_err());
    }
    assert!(plan(Frame::new(1, 1, 0xff000000), &[]).is_err());
}
#[test]
fn typed_scope_balance_and_combined_depth_are_checked_atomically() {
    let f = Frame::new(8, 8, 0);
    for bad in [
        vec![Command::PopClip],
        vec![Command::PopFixed],
        vec![Command::PushFixed, Command::PopClip],
        vec![
            Command::PushClip(Rect::new(0., 0., 8., 8.)),
            Command::PopFixed,
        ],
        vec![Command::PushFixed],
    ] {
        assert!(plan(f, &bad).is_err());
    }
    let mut commands = Vec::new();
    for i in 0..MAX_SCOPES {
        commands.push(if i % 2 == 0 {
            Command::PushFixed
        } else {
            Command::PushClip(Rect::new(0., 0., 8., 8.))
        });
    }
    let mut too_deep = commands.clone();
    too_deep.push(Command::PushFixed);
    assert!(plan(f, &too_deep).is_err());
    for i in (0..MAX_SCOPES).rev() {
        commands.push(if i % 2 == 0 {
            Command::PopFixed
        } else {
            Command::PopClip
        });
    }
    assert!(plan(f, &commands).is_ok());
}
#[test]
fn commands_overdraw_invocations_and_explicit_buffers_stay_bounded() {
    let f = Frame::new(MAX_WIDTH, MAX_HEIGHT, 0);
    assert!(plan(f, &vec![rect(0., 0., 0., 0., 0); MAX_COMMANDS + 1]).is_err());
    assert!(plan(f, &vec![rect(0., 0., 320., 240., 1); 60]).is_err());
    let p = plan(f, &vec![rect(0., 0., 1., 1., 1); MAX_COMMANDS]).unwrap();
    assert_eq!(p.draws.len(), 257);
    assert_eq!(p.invocations, 320 * 240 + 256 * 64);
    assert_eq!(p.gpu_buffer_bytes, 320 * 240 * 8 + 257 * 256);
    assert!(p.gpu_buffer_bytes <= MAX_GPU_BUFFER_BYTES);
    assert_eq!(p.parameters().len(), 257 * 256);
}
#[test]
fn metadata_records_are_immutable_aligned_and_never_pixels() {
    let p = plan(
        Frame::new(8, 6, 0x112233),
        &[rect(1., 2., 3., 4., 0xabcdef)],
    )
    .unwrap();
    let b = p.parameters();
    assert_eq!(b.len(), 2 * 256);
    let read = |offset: usize| u32::from_le_bytes(b[offset..offset + 4].try_into().unwrap());
    assert_eq!((read(256), read(260), read(264), read(268)), (1, 2, 3, 4));
    assert_eq!((read(272), read(276), read(280)), (8, 6, 0xffab_cdef));
    assert!(b[32..256].iter().chain(&b[288..512]).all(|v| *v == 0));
}

#[test]
fn exact_edges_touching_clips_and_fixed_escape_empty_ancestors() {
    let p = plan(
        Frame::new(8, 8, 0),
        &[
            rect(0.2, 0.2, 0.1, 0.1, 1),
            rect(2., 2., 1., 1., 2),
            rect(2., 2., 1.001, 1.001, 3),
        ],
    )
    .unwrap();
    assert_eq!(
        p.draws[1],
        Draw {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            color: 0xff00_0001,
            image: None
        }
    );
    assert_eq!(
        p.draws[2],
        Draw {
            x: 2,
            y: 2,
            width: 1,
            height: 1,
            color: 0xff00_0002,
            image: None
        }
    );
    assert_eq!(
        p.draws[3],
        Draw {
            x: 2,
            y: 2,
            width: 2,
            height: 2,
            color: 0xff00_0003,
            image: None
        }
    );
    let p = plan(
        Frame::new(8, 8, 0),
        &[
            Command::PushClip(Rect::new(0., 0., 2., 2.)),
            rect(2., 0., 1., 1., 1),
            Command::PushClip(Rect::new(20., 20., 1., 1.)),
            rect(0., 0., 8., 8., 1),
            Command::PushFixed,
            rect(0., 0., 1., 1., 2),
            Command::PopFixed,
            rect(0., 0., 8., 8., 3),
            Command::PopClip,
            Command::PopClip,
        ],
    )
    .unwrap();
    assert_eq!(p.draws.len(), 2);
    assert_eq!(
        p.draws[1],
        Draw {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            color: 0xff00_0002,
            image: None
        }
    );
}

fn image(rect: Rect, source: u32) -> Command {
    Command::Image { rect, source }
}
fn input_word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
// This interpreter checks serialized metadata/arena wiring only. It is not the
// pixel oracle: expected pixels are the separately frozen literal/band fixtures.
fn interpret(p: &Plan) -> Vec<u32> {
    let mut output = vec![0xdeadbeef; (p.frame.width * p.frame.height) as usize];
    for (draw, record) in p
        .draws
        .iter()
        .zip(p.parameters.as_chunks::<PARAM_STRIDE>().0)
    {
        let read = |index| input_word(record, index);
        let (x, y, width, height) = (read(0), read(1), read(2), read(3));
        assert_eq!((read(4), read(5)), (p.frame.width, p.frame.height));
        assert_eq!(read(7), 0);
        let used = if draw.kind() == DrawKind::Image {
            64
        } else {
            32
        };
        assert!(record[used..].iter().all(|v| *v == 0));
        for dy in 0..height {
            for dx in 0..width {
                let color = match draw.kind() {
                    DrawKind::Glyph => {
                        panic!("legacy image interpreter does not accept glyph plans")
                    }
                    DrawKind::Rectangle => read(6),
                    DrawKind::Image => {
                        let (source, sw, sh, count) = (read(8), read(9), read(10), read(11));
                        assert_eq!(count, sw * sh);
                        assert_eq!((read(14), read(15)), (width, height));
                        let sx = input_word(&p.input, (read(12) + dx) as usize);
                        let sy = input_word(&p.input, (read(13) + dy) as usize);
                        assert!(sx < sw && sy < sh);
                        input_word(&p.input, (source + sy * sw + sx) as usize)
                    }
                };
                let pixel = &mut output[((y + dy) * p.frame.width + x + dx) as usize];
                let alpha = color >> 24;
                let channel = |shift: u32| -> u32 {
                    let source = (color >> shift) & 255u32;
                    let destination = (*pixel >> shift) & 255u32;
                    (source * alpha + destination * (255 - alpha) + 127) / 255
                };
                *pixel = (channel(16) << 16) | (channel(8) << 8) | channel(0);
            }
        }
    }
    output
}

#[test]
fn image_frozen_pixels_and_independently_counted_protocol_are_exact() {
    // Frozen arithmetic protocol, not counters obtained from the planner.
    let expected_counts = [
        (2, 128, 848, 144),
        (2, 128, 716, 84),
        (2, 128, 720, 64),
        (2, 128, 736, 96),
        (2, 128, 680, 60),
        (2, 128, 680, 64),
        (3, 192, 1044, 96),
        (3, 192, 928, 60),
        (2, 128, 592, 32),
        (5, 320, 1568, 96),
        (6, 384, 1956, 160),
        (2, 153600, 617160, 307200),
        (1, 64, 352, 48),
        (2, 128, 564, 16),
    ];
    let fixtures = image_fixtures::fixtures();
    assert_eq!(fixtures.len(), expected_counts.len());
    for (fixture, (draws, invocations, bytes, compared)) in fixtures.iter().zip(expected_counts) {
        let p = fixture.plan().unwrap();
        assert_eq!(
            (p.draws.len(), p.invocations, p.gpu_buffer_bytes),
            (draws, invocations, bytes),
            "{}",
            fixture.name
        );
        assert_eq!(fixture.expected.len() * 4, compared);
        assert_eq!(interpret(&p), fixture.expected, "{}", fixture.name);
        assert_eq!(
            p.gpu_buffer_bytes,
            u64::from(p.frame.width) * u64::from(p.frame.height) * 8
                + p.parameters.len() as u64
                + p.input.len() as u64
        );
        assert_eq!(
            p.has_images(),
            p.draws.iter().any(|d| d.kind() == DrawKind::Image)
        );
    }
}

#[test]
fn image_plan_owns_sources_reuses_ids_and_validates_even_unused_entries() {
    let mut a = [1, 2, 3, 255];
    let b = [254, 253, 252, 255];
    let c = [224, 16, 32, 255];
    let commands = [
        image(Rect::new(0., 0., 1., 1.), 1),
        image(Rect::new(1., 0., 1., 1.), 1),
    ];
    let p = plan_with_images(
        Frame::new(2, 1, 0),
        &commands,
        &[
            SourceImage {
                width: 1,
                height: 1,
                rgba: &a,
            },
            SourceImage {
                width: 1,
                height: 1,
                rgba: &b,
            },
            SourceImage {
                width: 1,
                height: 1,
                rgba: &c,
            },
        ],
    )
    .unwrap();
    a.fill(0);
    assert_eq!(
        &p.input[..12],
        &[3, 2, 1, 255, 252, 253, 254, 255, 32, 16, 224, 255]
    );
    assert_eq!(p.input.len(), (3 + 2 + 2) * 4);
    assert_eq!(p.draws[1].image.unwrap().source_base, 1);
    assert_eq!(p.draws[2].image.unwrap().source_base, 1);
    assert_eq!(interpret(&p), [0xfefdfc, 0xfefdfc]);
    let with_unused_transparent = plan_with_images(
        Frame::new(2, 1, 0),
        &commands,
        &[
            SourceImage {
                width: 1,
                height: 1,
                rgba: &a,
            },
            SourceImage {
                width: 1,
                height: 1,
                rgba: &b,
            },
        ],
    )
    .unwrap(); // Source 0 is unused and valid transparent RGBA.
    assert_eq!(input_word(with_unused_transparent.input_bytes(), 0), 0);
    assert_eq!(interpret(&with_unused_transparent), [0xfefdfc, 0xfefdfc]);
    let sources = [SourceImage {
        width: 1,
        height: 1,
        rgba: &b,
    }];
    let empty = plan_with_images(
        Frame::new(2, 1, 0x123456),
        &[image(Rect::new(8., 8., 1., 1.), 0)],
        &sources,
    )
    .unwrap();
    assert!(!empty.has_images());
    assert!(empty.input_bytes().is_empty());
    assert_eq!(empty.gpu_buffer_bytes(), 2 * 8 + 256);
    assert_eq!(interpret(&empty), [0x123456; 2]);
}

#[test]
fn image_strict_sources_reject_malformed_and_hidden_input() {
    let a = [1, 2, 3, 255];
    let truncated = [1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3];
    let trailing = [1, 2, 3, 255, 0];
    let first_alpha = [1, 2, 3, 0, 254, 253, 252, 255];
    let last_alpha = [1, 2, 3, 255, 254, 253, 252, 254];
    let large = a.repeat(262145);
    let malformed = [
        (0, 1, &[][..]),
        (1, 0, &[][..]),
        (u32::MAX, u32::MAX, &a[..]),
        (2, 2, &truncated[..]),
        (1, 1, &trailing[..]),
        (262145, 1, &large[..]),
    ];
    let f = Frame::new(4, 3, 0xffffff);
    for (width, height, rgba) in malformed {
        let source = SourceImage {
            width,
            height,
            rgba,
        };
        assert!(plan_with_images(f, &[image(Rect::new(0., 0., 1., 1.), 0)], &[source]).is_err());
    }
    // Retain the original hidden inputs. Valid alpha is now supported; malformed
    // length, invalid source ID and excessive bytes still reject in all contexts.
    let hidden = [
        (2, 1, &first_alpha[..], 0, true),
        (2, 1, &last_alpha[..], 0, true),
        (2, 2, &truncated[..], 0, false),
        (1, 1, &a[..], 1, false),
        (262145, 1, &large[..], 0, false),
    ];
    for (width, height, rgba, id, supported) in hidden {
        let source = SourceImage {
            width,
            height,
            rgba,
        };
        for commands in [
            vec![image(Rect::new(0., 0., 0., 1.), id)],
            vec![image(Rect::new(20., 20., 1., 1.), id)],
            vec![
                Command::PushClip(Rect::new(20., 20., 1., 1.)),
                image(Rect::new(0., 0., 4., 3.), id),
                Command::PopClip,
            ],
        ] {
            let result = plan_with_images(f, &commands, &[source]);
            if supported {
                let p = result.unwrap();
                assert_eq!(p.draws.len(), 1);
                assert!(p.input_bytes().is_empty());
                assert_eq!(interpret(&p), [0xffffff; 12]);
            } else {
                assert!(result.is_err());
            }
        }
    }
    assert!(
        plan_with_images(
            f,
            &[image(Rect::new(0., 0., 1., 1.), 1)],
            &[SourceImage {
                width: 1,
                height: 1,
                rgba: &a
            }]
        )
        .is_err()
    );
    let half = a.repeat(131073);
    let source = SourceImage {
        width: 131073,
        height: 1,
        rgba: &half,
    };
    assert_eq!(
        plan_with_images(f, &[], &[source, source]).unwrap_err(),
        "source byte budget"
    );
    let sources = vec![
        SourceImage {
            width: 1,
            height: 1,
            rgba: &a
        };
        257
    ];
    assert_eq!(
        plan_with_images(f, &[], &sources).unwrap_err(),
        "source entry budget"
    );
}

#[test]
fn image_geometry_scopes_and_unsupported_commands_refuse_whole_plan() {
    let a = [1, 2, 3, 255];
    let sources = [SourceImage {
        width: 1,
        height: 1,
        rgba: &a,
    }];
    let f = Frame::new(4, 3, 0xffffff);
    for r in [
        Rect::new(f32::NAN, 0., 1., 1.),
        Rect::new(0., 0., 1., f32::INFINITY),
        Rect::new(1_000_001., 0., 1., 1.),
    ] {
        assert!(plan_with_images(f, &[image(r, 0)], &sources).is_err());
    }
    let mut offset = f;
    offset.document_offset = (f32::NEG_INFINITY, 0.);
    assert!(plan_with_images(offset, &[image(Rect::new(0., 0., 1., 1.), 0)], &sources).is_err());
    for suffix in [
        Command::Rect {
            rect: Rect::new(0., 0., 1., 1.),
            rgba: a,
            radius: 1.,
        },
        Command::Unsupported("opacity"),
        Command::Unsupported("text"),
    ] {
        assert!(
            plan_with_images(f, &[image(Rect::new(0., 0., 1., 1.), 0), suffix], &sources).is_err()
        );
    }
    for commands in [
        vec![Command::PushFixed, Command::PopClip],
        vec![Command::PopFixed],
        vec![Command::PushFixed],
    ] {
        assert!(plan_with_images(f, &commands, &sources).is_err());
    }
    let mut too_deep = vec![Command::PushFixed; 33];
    too_deep.extend([Command::PopFixed; 33]);
    assert_eq!(
        plan_with_images(f, &too_deep, &sources).unwrap_err(),
        "scope budget"
    );
    assert_eq!(
        plan_with_images(f, &vec![image(Rect::new(0., 0., 0., 1.), 0); 257], &sources).unwrap_err(),
        "command budget"
    );
}

#[test]
fn image_exact_gpu_byte_and_invocation_caps_accept_and_next_unit_rejects() {
    let a = [1, 2, 3, 255];
    let bytes = a.repeat(262012);
    let f = Frame::new(1, 1, 0xffffff);
    let command = image(Rect::new(0., 0., 1., 1.), 0);
    let p = plan_with_images(
        f,
        &[command],
        &[SourceImage {
            width: 262012,
            height: 1,
            rgba: &bytes,
        }],
    )
    .unwrap();
    assert_eq!(p.gpu_buffer_bytes(), 1_048_576);
    assert_eq!(p.input.len(), 1_048_048 + 8);
    assert_eq!(interpret(&p), [0x010203]);
    let bytes = a.repeat(262013);
    assert_eq!(
        plan_with_images(
            f,
            &[command],
            &[SourceImage {
                width: 262013,
                height: 1,
                rgba: &bytes
            }]
        )
        .unwrap_err(),
        "GPU buffer budget"
    );
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &a,
    };
    let mut commands = vec![image(Rect::new(0., 0., 320., 240.), 0); 51];
    commands.push(image(Rect::new(0., 0., 80., 80.), 0));
    let f = Frame::new(320, 240, 0xffffff);
    let p = plan_with_images(f, &commands, &[source]).unwrap();
    assert_eq!(p.invocations(), 4_000_000);
    assert_eq!(p.draws.len(), 53);
    assert_eq!(p.input.len(), (1 + 28_720) * 4);
    assert_eq!(p.gpu_buffer_bytes(), 742_852);
    assert!(interpret(&p).iter().all(|v| *v == 0x010203));
    commands.push(command);
    assert_eq!(
        plan_with_images(f, &commands, &[source]).unwrap_err(),
        "GPU invocation budget"
    );
}

#[test]
fn image_source_command_and_scope_caps_accept_exact_boundaries() {
    let a = [1, 2, 3, 255];
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &a,
    };
    let f = Frame::new(1, 1, 0x123456);
    let sources = vec![source; 256];
    let p = plan_with_images(f, &[image(Rect::new(0., 0., 1., 1.), 255)], &sources).unwrap();
    assert_eq!(p.draws[1].image.unwrap().source_base, 255);
    assert_eq!(p.input.len(), (256 + 2) * 4);
    assert_eq!(interpret(&p), [0x010203]);
    let p = plan_with_images(
        f,
        &vec![image(Rect::new(0., 0., 0., 1.), 0); 256],
        &[source],
    )
    .unwrap();
    assert_eq!(p.draws.len(), 1);
    assert!(!p.has_images());
    assert_eq!(interpret(&p), [0x123456]);
    let mut commands = vec![Command::PushFixed; 32];
    commands.extend([Command::PopFixed; 32]);
    let p = plan_with_images(f, &commands, &[source]).unwrap();
    assert_eq!(interpret(&p), [0x123456]);
    assert_eq!(MAX_LUT_ENTRIES, 143360); // Structural bound, not a reachable quota oracle.
}

// Scalar and table expectations transcribed from the frozen preimplementation
// oracle ccd79c6a…; no planner/Canvas output was used to generate them.
#[test]
fn image_scalar_sampling_matches_nine_frozen_float_edge_oracles() {
    // lower-clamp
    for (destination, expected) in [(-1, 0), (0, 0), (1, 2)] {
        assert_eq!(sample_index(destination, 0.25, 1.5, 4), expected);
    }
    // upper-clamp-independent-of-coverage
    for (destination, expected) in [(0, 0), (1, 1), (2, 2), (3, 2)] {
        assert_eq!(sample_index(destination, 0.0, 2.0, 3), expected);
    }
    // one-texel-source
    for (destination, expected) in [(-2, 0), (0, 0), (1, 0), (2, 0), (4, 0)] {
        assert_eq!(sample_index(destination, -0.5, 3.0, 1), expected);
    }
    // threshold-one-ulp-smaller-denominator
    assert_eq!(sample_index(1, 0.0, f32::from_bits(0x3fffffff), 4), 2);
    // threshold-exact-denominator
    assert_eq!(sample_index(1, 0.0, 2.0, 4), 2);
    // threshold-one-ulp-larger-denominator
    assert_eq!(sample_index(1, 0.0, f32::from_bits(0x40000001), 4), 1);
    // subnormal-finite-ratio
    assert_eq!(
        sample_index(0, f32::from_bits(0x80000001), f32::from_bits(0x00000002), 2),
        1
    );
    // positive-intermediate-infinity-clamps
    assert_eq!(sample_index(1, 0.0, f32::from_bits(0x00000001), 2), 1);
    // negative-intermediate-infinity-clamps
    assert_eq!(sample_index(-1, 0.0, f32::from_bits(0x00000001), 2), 0);
}

#[test]
fn image_all_642_frozen_lookup_entries_and_origins_are_preserved() {
    struct Tables {
        origin: (u32, u32),
        x: Vec<u32>,
        y: Vec<u32>,
    }
    let oracle: &[(&str, Vec<Tables>)] = &[
        (
            "image-upscale-2x2",
            vec![Tables {
                origin: (1, 1),
                x: vec![0, 0, 1, 1],
                y: vec![0, 0, 1, 1],
            }],
        ),
        (
            "image-three-columns-to-five",
            vec![Tables {
                origin: (1, 1),
                x: vec![0, 0, 1, 1, 2],
                y: vec![0],
            }],
        ),
        (
            "image-downscale-distinct-4x4",
            vec![Tables {
                origin: (1, 1),
                x: vec![0, 2],
                y: vec![0, 2],
            }],
        ),
        (
            "image-clip-preserves-original-origin",
            vec![Tables {
                origin: (3, 1),
                x: vec![1, 1, 2, 2],
                y: vec![0],
            }],
        ),
        (
            "image-negative-document-offset",
            vec![Tables {
                origin: (0, 0),
                x: vec![1, 1, 2, 2],
                y: vec![0, 1],
            }],
        ),
        (
            "image-negative-fractional-origin",
            vec![Tables {
                origin: (0, 0),
                x: vec![0, 1, 1],
                y: vec![0, 1, 1],
            }],
        ),
        (
            "image-tiny-and-fractional-clip",
            vec![
                Tables {
                    origin: (0, 0),
                    x: vec![0],
                    y: vec![0],
                },
                Tables {
                    origin: (1, 1),
                    x: vec![2],
                    y: vec![2],
                },
            ],
        ),
        (
            "image-exact-and-one-ulp-translated-edges",
            vec![
                Tables {
                    origin: (0, 0),
                    x: vec![0],
                    y: vec![0],
                },
                Tables {
                    origin: (2, 0),
                    x: vec![0, 1],
                    y: vec![0, 1],
                },
            ],
        ),
        (
            "image-f32-endpoint-addition-rounds-back",
            vec![Tables {
                origin: (2, 0),
                x: vec![0],
                y: vec![0],
            }],
        ),
        (
            "image-rectangle-order-and-source-reuse",
            vec![
                Tables {
                    origin: (0, 0),
                    x: vec![0, 0, 0, 1, 1, 1],
                    y: vec![0, 0, 0, 0],
                },
                Tables {
                    origin: (2, 0),
                    x: vec![0, 1],
                    y: vec![0, 0, 0],
                },
                Tables {
                    origin: (0, 3),
                    x: vec![0, 0, 1, 1],
                    y: vec![0],
                },
            ],
        ),
        (
            "image-fixed-escape-and-nested-restoration",
            vec![
                Tables {
                    origin: (1, 1),
                    x: vec![0, 0, 1, 1],
                    y: vec![0, 0],
                },
                Tables {
                    origin: (4, 2),
                    x: vec![1],
                    y: vec![0],
                },
                Tables {
                    origin: (6, 3),
                    x: vec![0],
                    y: vec![0],
                },
                Tables {
                    origin: (4, 2),
                    x: vec![1],
                    y: vec![0],
                },
                Tables {
                    origin: (1, 3),
                    x: vec![0, 1],
                    y: vec![0],
                },
            ],
        ),
        (
            "image-dispatch-tail-and-clear-border",
            vec![Tables {
                origin: (0, 0),
                x: [vec![0; 160], vec![1; 159]].concat(),
                y: [vec![0; 120], vec![1; 119]].concat(),
            }],
        ),
        ("image-valid-empty-and-hidden", vec![]),
        (
            "image-subnormal-origin-and-lost-extent",
            vec![Tables {
                origin: (0, 0),
                x: vec![1],
                y: vec![0],
            }],
        ),
    ];
    let fixtures = image_fixtures::fixtures();
    assert_eq!(oracle.len(), fixtures.len());
    let mut total = 0;
    for (fixture, (name, tables)) in fixtures.iter().zip(oracle) {
        assert_eq!(fixture.name, *name);
        let p = fixture.plan().unwrap();
        let draws: Vec<_> = p
            .draws
            .iter()
            .filter(|d| d.kind() == DrawKind::Image)
            .collect();
        assert_eq!(draws.len(), tables.len(), "{name}");
        for (draw, expected) in draws.iter().zip(tables) {
            let params = draw.image.unwrap();
            assert_eq!((draw.x, draw.y), expected.origin, "{name}");
            assert_eq!(
                (draw.width as usize, draw.height as usize),
                (expected.x.len(), expected.y.len())
            );
            for (base, table) in [(params.x_base, &expected.x), (params.y_base, &expected.y)] {
                for (i, value) in table.iter().enumerate() {
                    assert_eq!(
                        input_word(&p.input, base as usize + i),
                        *value,
                        "{name} entry {i}"
                    );
                }
                total += table.len();
            }
        }
    }
    assert_eq!(total, 642);
}

fn transparent_rect(rect: Rect) -> Command {
    Command::Rect {
        rect,
        rgba: [101, 202, 77, 0],
        radius: 0.,
    }
}

#[test]
fn alpha_frozen_literals_and_counter_contracts_match_serialized_plans() {
    // Nine preimplementation tuples from the independent counter freeze 8bad259e.
    // Literal pixel targets come from the separately authored fixture module.
    let expected = [
        (6, 2, 12, 768, 3168, 48),
        (6, 2, 3, 192, 944, 48),
        (4, 1, 10, 640, 2644, 16),
        (7, 1, 65, 4160, 16696, 28),
        (4, 3, 4, 256, 1132, 48),
        (5, 3, 2, 128, 672, 60),
        (5, 3, 6, 384, 1680, 60),
        (2, 2, 2, 128, 576, 16),
        (3, 2, 1, 64, 304, 24),
    ];
    let fixtures = alpha_fixtures::fixtures();
    assert_eq!(fixtures.len(), expected.len());
    assert_eq!(alpha_fixtures::EXPECTED_COUNTERS.len(), expected.len());
    for ((fixture, counters), (width, height, draws, invocations, bytes, compared)) in fixtures
        .iter()
        .zip(alpha_fixtures::EXPECTED_COUNTERS)
        .zip(expected)
    {
        assert_eq!(fixture.name, counters.name);
        assert_eq!((counters.width, counters.height), (width, height));
        assert_eq!(
            (
                counters.draws,
                counters.invocations,
                counters.gpu_buffer_bytes
            ),
            (draws, invocations, bytes)
        );
        assert_eq!(counters.compared_bytes, compared as u64);
        let p = fixture.plan().unwrap();
        assert_eq!((p.frame.width, p.frame.height), (width, height));
        assert_eq!(
            (p.draws.len(), p.invocations, p.gpu_buffer_bytes),
            (draws, invocations, bytes),
            "{}",
            fixture.name
        );
        assert_eq!(fixture.expected.len() * 4, compared);
        let actual = interpret(&p);
        assert_eq!(actual, fixture.expected, "{}", fixture.name);
        assert!(actual.iter().all(|word| word >> 24 == 0));
        assert_eq!(p.draws[0].color, 0xff00_0000 | p.frame.clear);
        assert_eq!(
            p.gpu_buffer_bytes,
            u64::from(width) * u64::from(height) * 8
                + p.parameters.len() as u64
                + p.input.len() as u64
        );
    }
}

#[test]
fn alpha_only_old_rejections_are_supported_without_discarding_source_alpha() {
    // The old unsupported-content case was a valid empty transparent rectangle.
    let p = plan(
        Frame::new(8, 8, 0),
        &[
            rect(0., 0., 8., 8., 1),
            Command::Rect {
                rect: Rect::new(20., 20., 0., 0.),
                rgba: [0, 0, 0, 0],
                radius: 0.,
            },
        ],
    )
    .unwrap();
    assert_eq!(p.draws.len(), 2);
    assert_eq!(interpret(&p), [1; 64]);

    // These exact source bytes previously rejected solely for valid alpha.
    let first_alpha = [1, 2, 3, 0, 254, 253, 252, 255];
    let last_alpha = [1, 2, 3, 255, 254, 253, 252, 254];
    for (rgba, words, pixels) in [
        (
            &first_alpha,
            [0x0001_0203, 0xfffe_fdfc],
            [0xffffff, 0xfefdfc],
        ),
        (
            &last_alpha,
            [0xff01_0203, 0xfefe_fdfc],
            [0x010203, 0xfefdfc],
        ),
    ] {
        let p = plan_with_images(
            Frame::new(2, 1, 0xffffff),
            &[image(Rect::new(0., 0., 2., 1.), 0)],
            &[SourceImage {
                width: 2,
                height: 1,
                rgba,
            }],
        )
        .unwrap();
        assert_eq!(p.draws.len(), 2);
        assert_eq!([input_word(&p.input, 0), input_word(&p.input, 1)], words);
        assert_eq!(interpret(&p), pixels);
    }
}

#[test]
fn transparent_rectangles_validate_geometry_and_radius_before_elision() {
    let f = Frame::new(4, 3, 0x123456);
    for value in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        MAX_COORDINATE + 1.,
    ] {
        for invalid in [
            Rect::new(value, 0., 0., 0.),
            Rect::new(0., value, 0., 0.),
            Rect::new(0., 0., value, 0.),
            Rect::new(0., 0., 0., value),
        ] {
            let command = transparent_rect(invalid);
            assert!(plan(f, &[command]).is_err());
            assert!(
                plan(
                    f,
                    &[
                        Command::PushClip(Rect::new(20., 20., 1., 1.)),
                        command,
                        Command::PopClip,
                    ]
                )
                .is_err()
            );
        }
    }
    for radius in [1., -1., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for geometry in [Rect::new(0., 0., 4., 3.), Rect::new(20., 20., 0., 0.)] {
            assert!(
                plan(
                    f,
                    &[Command::Rect {
                        rect: geometry,
                        rgba: [101, 202, 77, 0],
                        radius,
                    }]
                )
                .is_err()
            );
        }
    }
    let p = plan(f, &[transparent_rect(Rect::new(0., 0., 4., 3.))]).unwrap();
    assert_eq!(p.draws.len(), 1);
    assert_eq!(p.invocations, 64);
    assert_eq!(p.gpu_buffer_bytes, 4 * 3 * 8 + 256);
    assert_eq!(interpret(&p), [0x123456; 12]);
}

#[test]
fn transparent_commands_cannot_bypass_typed_scopes_counts_or_clear_validation() {
    let f = Frame::new(1, 1, 0x123456);
    let zero = transparent_rect(Rect::new(0., 0., 1., 1.));
    for bad in [
        vec![zero, Command::PopClip],
        vec![zero, Command::PopFixed],
        vec![Command::PushFixed, zero, Command::PopClip],
        vec![
            Command::PushClip(Rect::new(0., 0., 1., 1.)),
            zero,
            Command::PopFixed,
        ],
        vec![zero, Command::PushFixed],
        vec![Command::PushClip(Rect::new(0., 0., 1., 1.)), zero],
        vec![zero, Command::Unsupported("opacity")],
    ] {
        assert!(plan(f, &bad).is_err());
    }
    let p = plan(f, &vec![zero; MAX_COMMANDS]).unwrap();
    assert_eq!(p.draws.len(), 1);
    assert_eq!(interpret(&p), [0x123456]);
    assert_eq!(
        plan(f, &vec![zero; MAX_COMMANDS + 1]).unwrap_err(),
        "command budget"
    );
    for depth in [MAX_SCOPES, MAX_SCOPES + 1] {
        let mut commands = vec![Command::PushFixed; depth];
        commands.push(zero);
        commands.extend(vec![Command::PopFixed; depth]);
        let result = plan(f, &commands);
        if depth == MAX_SCOPES {
            assert_eq!(interpret(&result.unwrap()), [0x123456]);
        } else {
            assert_eq!(result.unwrap_err(), "scope budget");
        }
    }
    for clear in [0x0100_0000, 0xff00_0000, u32::MAX] {
        assert_eq!(
            plan(Frame::new(1, 1, clear), &[zero]).unwrap_err(),
            "clear color must be packed RGB"
        );
    }
}

#[test]
fn transparent_source_structure_is_checked_even_if_unused_empty_or_clipped() {
    let pixel = [9, 8, 7, 0];
    let truncated = [9, 8, 7];
    let trailing = [9, 8, 7, 0, 0];
    let excessive = pixel.repeat(MAX_SOURCE_RGBA_BYTES / 4 + 1);
    let f = Frame::new(2, 2, 0x123456);
    for (width, height, rgba) in [
        (0, 1, &[][..]),
        (1, 0, &[][..]),
        (u32::MAX, u32::MAX, &pixel[..]),
        (1, 1, &truncated[..]),
        (1, 1, &trailing[..]),
        (262145, 1, &excessive[..]),
    ] {
        let source = SourceImage {
            width,
            height,
            rgba,
        };
        for commands in [
            vec![],
            vec![image(Rect::new(0., 0., 0., 1.), 0)],
            vec![image(Rect::new(20., 20., 1., 1.), 0)],
            vec![
                Command::PushClip(Rect::new(20., 20., 1., 1.)),
                image(Rect::new(0., 0., 2., 2.), 0),
                Command::PopClip,
            ],
        ] {
            assert!(plan_with_images(f, &commands, &[source]).is_err());
        }
    }
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &pixel,
    };
    assert_eq!(
        plan_with_images(f, &[], &vec![source; MAX_SOURCE_ENTRIES + 1]).unwrap_err(),
        "source entry budget"
    );
    let exact = pixel.repeat(MAX_SOURCE_RGBA_BYTES / 4);
    let source = SourceImage {
        width: 262144,
        height: 1,
        rgba: &exact,
    };
    let p = plan_with_images(f, &[], &[source]).unwrap();
    assert!(!p.has_images());
    assert_eq!(interpret(&p), [0x123456; 4]);
    assert_eq!(
        plan_with_images(f, &[], &[source, source]).unwrap_err(),
        "source byte budget"
    );
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &pixel,
    };
    assert!(plan_with_images(f, &[image(Rect::new(0., 0., 0., 1.), 1)], &[source]).is_err());
    assert!(plan_with_images(f, &[image(Rect::new(f32::NAN, 0., 1., 1.), 0)], &[source]).is_err());
}

#[test]
fn fully_transparent_images_keep_dispatch_arena_and_exact_resource_caps() {
    let rgba = [255, 0, 0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 255, 255, 255, 0];
    let p = plan_with_images(
        Frame::new(2, 2, 0x123456),
        &[image(Rect::new(0., 0., 2., 2.), 0)],
        &[SourceImage {
            width: 2,
            height: 2,
            rgba: &rgba,
        }],
    )
    .unwrap();
    assert_eq!(
        (p.draws.len(), p.invocations, p.gpu_buffer_bytes),
        (2, 128, 576)
    );
    assert_eq!(p.input.len(), 32);
    assert!(p.has_images());
    assert_eq!(interpret(&p), [0x123456; 4]);

    let pixel = [1, 2, 3, 0];
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &pixel,
    };
    let mut commands = vec![image(Rect::new(0., 0., 320., 240.), 0); 51];
    commands.push(image(Rect::new(0., 0., 80., 80.), 0));
    let f = Frame::new(320, 240, 0x123456);
    let p = plan_with_images(f, &commands, &[source]).unwrap();
    assert_eq!(
        (p.invocations(), p.draws.len(), p.gpu_buffer_bytes()),
        (4_000_000, 53, 742_852)
    );
    commands.push(image(Rect::new(0., 0., 1., 1.), 0));
    assert_eq!(
        plan_with_images(f, &commands, &[source]).unwrap_err(),
        "GPU invocation budget"
    );

    let f = Frame::new(1, 1, 0x123456);
    let command = image(Rect::new(0., 0., 1., 1.), 0);
    for width in [262012, 262013] {
        let bytes = pixel.repeat(width as usize);
        let result = plan_with_images(
            f,
            &[command],
            &[SourceImage {
                width,
                height: 1,
                rgba: &bytes,
            }],
        );
        if width == 262012 {
            let p = result.unwrap();
            assert_eq!(p.gpu_buffer_bytes(), MAX_GPU_BUFFER_BYTES);
            assert_eq!(interpret(&p), [0x123456]);
        } else {
            assert_eq!(result.unwrap_err(), "GPU buffer budget");
        }
    }
}
