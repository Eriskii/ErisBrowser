use super::*;

fn gradient() -> LinearGradient {
    LinearGradient {
        direction: GradientDirection::Angle(-45.0),
        stops: Arc::from([
            GradientStop {
                color: GradientColor::Color(Color::rgba(255, 0, 0, 128)),
                position: Some(GradientPosition {
                    px: -12.5,
                    percent: 25.0,
                }),
            },
            GradientStop {
                color: GradientColor::CurrentColor,
                position: None,
            },
            GradientStop {
                color: GradientColor::Color(Color::rgba(0, 0, 255, 0)),
                position: Some(GradientPosition {
                    px: 300.0,
                    percent: 150.0,
                }),
            },
        ]),
    }
}

// Authored from the wire contract, independently of Encoder's methods.
fn literal() -> Vec<u8> {
    let mut bytes = b"ERWB\x63\x00".to_vec();
    bytes.extend_from_slice(&(-45.0f64).to_le_bytes());
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&[0, 255, 0, 0, 128, 1]);
    bytes.extend_from_slice(&(-12.5f64).to_le_bytes());
    bytes.extend_from_slice(&25.0f64.to_le_bytes());
    bytes.extend_from_slice(&[1, 0, 0, 0, 0, 255, 0, 1]);
    bytes.extend_from_slice(&300.0f64.to_le_bytes());
    bytes.extend_from_slice(&150.0f64.to_le_bytes());
    bytes
}

fn retained(count: usize) -> usize {
    use std::mem::{align_of, size_of};
    count * size_of::<GradientStop>()
        + size_of::<LinearGradient>()
        + 4 * size_of::<usize>()
        + align_of::<GradientStop>()
        + align_of::<LinearGradient>()
}

fn reply(commands: Vec<DrawCommand>) -> Reply {
    let page = crate::page::Page::from_html(
        url::Url::parse("about:blank").unwrap(),
        "<p id=x>unchanged</p>",
        false,
    );
    Reply {
        navigation: None,
        snapshot: Some(Snapshot {
            generation: 7,
            processed_edit_sequence: 3,
            task_state: TaskState::Idle,
            layout: LayoutResult {
                commands,
                hit_regions: Vec::new(),
                node_visibility: vec![Visibility::Visible; page.document.nodes.len()],
                content_height: 24.0,
            },
            images: page.images,
            document: page.document,
            title: "gradient".into(),
            url: "about:blank".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        }),
    }
}

fn command() -> DrawCommand {
    DrawCommand::LinearGradient {
        rect: Rect {
            x: -1.5,
            y: 2.25,
            width: 20.0,
            height: 12.0,
        },
        tile: Rect {
            x: 0.5,
            y: 3.25,
            width: 16.0,
            height: 8.0,
        },
        radius: 2.5,
        gradient: Arc::new(gradient()),
        current_color: Color::rgba(12, 34, 56, 78),
        stop_scale: 1.5,
        repeat: [true, true],
    }
}

#[test]
fn gradient_wire_matches_literal_and_preserves_unresolved_stops() {
    let bytes = literal();
    assert_eq!(bytes.len(), 64);
    let mut remaining = retained(3);
    let mut decoder = Decoder::new(&bytes, 99).unwrap();
    let decoded = decode_gradient(&mut decoder, &mut remaining).unwrap();
    decoder.end().unwrap();
    assert_eq!(*decoded, gradient());
    assert_eq!(remaining, 0);
    let mut encoder = Encoder::new(99);
    encode_gradient(&mut encoder, &gradient(), &mut retained(3));
    assert_eq!(encoder.finish().unwrap(), bytes);
}

#[test]
fn gradient_wire_exact_and_one_short_retained_admissions_are_cumulative() {
    assert_eq!(MAX_GRADIENT_BYTES, 8 * 1024 * 1024);
    let bytes = literal();
    let charge = retained(3);
    let mut remaining = 2 * charge;
    let first = decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut remaining).unwrap();
    let second = decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut remaining).unwrap();
    assert_eq!(remaining, 0);
    assert!(!Arc::ptr_eq(&first, &second));
    assert!(!Arc::ptr_eq(&first.stops, &second.stops));
    let mut remaining = charge - 1;
    assert!(decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut remaining).is_err());
    assert_eq!(remaining, charge - 1);
    let mut remaining = charge;
    let mut encoder = Encoder::new(99);
    encode_gradient(&mut encoder, &gradient(), &mut remaining);
    assert_eq!(remaining, 0);
    encode_gradient(&mut encoder, &gradient(), &mut remaining);
    assert!(encoder.finish().is_err());
}

#[test]
fn gradient_wire_every_truncation_rejects_without_retained_admission() {
    let bytes = literal();
    for cut in 0..bytes.len() {
        let mut remaining = retained(3);
        if let Ok(mut decoder) = Decoder::new(&bytes[..cut], 99) {
            assert!(
                decode_gradient(&mut decoder, &mut remaining).is_err(),
                "cut {cut}"
            );
        }
        assert_eq!(remaining, retained(3), "cut {cut}");
    }
    let mut bytes = bytes;
    bytes.push(0);
    let mut decoder = Decoder::new(&bytes, 99).unwrap();
    decode_gradient(&mut decoder, &mut retained(3)).unwrap();
    assert!(decoder.end().is_err());
}

#[test]
fn gradient_wire_rejects_bad_tags_counts_and_all_nonfinite_fields() {
    for (offset, value) in [(5, 2), (18, 2), (23, 2), (40, 2), (41, 2)] {
        let mut bytes = literal();
        bytes[offset] = value;
        let mut remaining = MAX_GRADIENT_BYTES;
        assert!(decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut remaining).is_err());
        assert_eq!(remaining, MAX_GRADIENT_BYTES);
    }
    for count in [0u32, 33, u32::MAX] {
        let mut bytes = literal();
        bytes[14..18].copy_from_slice(&count.to_le_bytes());
        let mut remaining = MAX_GRADIENT_BYTES;
        assert!(decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut remaining).is_err());
        assert_eq!(remaining, MAX_GRADIENT_BYTES);
    }
    for offset in [6, 24, 32, 48, 56] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut bytes = literal();
            bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
            let mut remaining = MAX_GRADIENT_BYTES;
            assert!(
                decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut remaining).is_err()
            );
            assert_eq!(remaining, MAX_GRADIENT_BYTES);
        }
    }
}

#[test]
fn gradient_wire_corner_single_and_maximum_stop_count_are_valid() {
    for count in [1, 32] {
        let value = LinearGradient {
            direction: GradientDirection::Corner {
                right: false,
                bottom: true,
            },
            stops: vec![
                GradientStop {
                    color: GradientColor::CurrentColor,
                    position: Some(GradientPosition {
                        px: -0.0,
                        percent: -150.0
                    })
                };
                count
            ]
            .into(),
        };
        let mut e = Encoder::new(99);
        encode_gradient(&mut e, &value, &mut retained(count));
        let bytes = e.finish().unwrap();
        assert_eq!(&bytes[5..8], &[1, 0, 1]);
        let result =
            decode_gradient(&mut Decoder::new(&bytes, 99).unwrap(), &mut retained(count)).unwrap();
        assert_eq!(*result, value);
        assert_eq!(
            result.stops[0].position.unwrap().px.to_bits(),
            (-0.0f64).to_bits()
        );
        let mut invalid = bytes;
        invalid[6] = 2;
        assert!(
            decode_gradient(
                &mut Decoder::new(&invalid, 99).unwrap(),
                &mut retained(count)
            )
            .is_err()
        );
    }
}

#[test]
fn gradient_wire_rejects_invalid_sender_metadata_and_keeps_timing_reader() {
    let mut value = gradient();
    value.stops = Arc::from([]);
    let mut e = Encoder::new(99);
    let mut remaining = MAX_GRADIENT_BYTES;
    encode_gradient(&mut e, &value, &mut remaining);
    assert!(e.finish().is_err());
    let mut e = Encoder::new(99);
    e.f64(-45.0);
    let bytes = e.finish().unwrap();
    assert!(Decoder::new(&bytes, 99).unwrap().f64().is_err());
    assert_eq!(
        Decoder::new(&bytes, 99).unwrap().gradient_f64().unwrap(),
        -45.0
    );
}

#[test]
fn gradient_snapshot_preserves_geometry_scopes_and_independent_data_without_images() {
    let expected_gradient = gradient();
    let original = command();
    let r = reply(vec![
        DrawCommand::PushFixed,
        DrawCommand::PushOpacity { opacity: 0.5 },
        original.clone(),
        original.clone(),
        DrawCommand::PopOpacity,
        DrawCommand::PopFixed,
    ]);
    let decoded = decode_reply(&encode_reply(&r).unwrap())
        .unwrap()
        .snapshot
        .unwrap();
    assert!(decoded.images.is_empty());
    assert_eq!(decoded.generation, 7);
    assert!(decoded.document.query_selector("#x").is_some());
    let DrawCommand::LinearGradient {
        rect,
        tile,
        radius,
        gradient,
        current_color,
        stop_scale,
        repeat,
    } = &decoded.layout.commands[2]
    else {
        panic!("gradient command missing")
    };
    assert_eq!(*repeat, [true, true]);
    assert_eq!(
        *rect,
        Rect {
            x: -1.5,
            y: 2.25,
            width: 20.0,
            height: 12.0
        }
    );
    assert_eq!(
        *tile,
        Rect {
            x: 0.5,
            y: 3.25,
            width: 16.0,
            height: 8.0
        }
    );
    assert_eq!(
        (*radius, *current_color, *stop_scale),
        (2.5, Color::rgba(12, 34, 56, 78), 1.5)
    );
    assert_eq!(gradient.as_ref(), &expected_gradient);
    let DrawCommand::LinearGradient {
        gradient: other, ..
    } = &decoded.layout.commands[3]
    else {
        panic!()
    };
    assert!(!Arc::ptr_eq(gradient, other));
    assert!(!Arc::ptr_eq(&gradient.stops, &other.stops));
}

#[test]
fn gradient_snapshot_rejects_invalid_geometry_before_stop_allocation() {
    for field in 0..7 {
        let mut value = command();
        let DrawCommand::LinearGradient {
            rect,
            tile,
            radius,
            stop_scale,
            ..
        } = &mut value
        else {
            unreachable!()
        };
        match field {
            0 => tile.width = 0.0,
            1 => tile.height = -1.0,
            2 => *radius = -1.0,
            3 => *stop_scale = 0.0,
            4 => *stop_scale = f32::NAN,
            5 => rect.x = f32::INFINITY,
            _ => tile.x = f32::NEG_INFINITY,
        }
        assert!(
            decode_reply(&encode_reply(&reply(vec![value])).unwrap()).is_err(),
            "field {field}"
        );
    }
    let mut value = command();
    if let DrawCommand::LinearGradient { rect, .. } = &mut value {
        rect.width = 0.0;
    }
    assert!(decode_reply(&encode_reply(&reply(vec![value])).unwrap()).is_ok());
}
