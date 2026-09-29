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
            for y in d.y..d.y + d.height {
                for x in d.x..d.x + d.width {
                    actual[(y * p.frame.width + x) as usize] = d.color;
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
            color: 1
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
            color: 1
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
            rect: Rect::new(20., 20., 0., 0.),
            rgba: [0, 0, 0, 0],
            radius: 0.,
        },
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
    assert_eq!((read(272), read(276), read(280)), (8, 6, 0xabcdef));
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
            color: 1
        }
    );
    assert_eq!(
        p.draws[2],
        Draw {
            x: 2,
            y: 2,
            width: 1,
            height: 1,
            color: 2
        }
    );
    assert_eq!(
        p.draws[3],
        Draw {
            x: 2,
            y: 2,
            width: 2,
            height: 2,
            color: 3
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
            color: 2
        }
    );
}
