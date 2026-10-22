use super::*;

fn command(repeat: [bool; 2]) -> DrawCommand {
    DrawCommand::LinearGradient {
        rect: Rect {
            x: -1.0,
            y: 2.0,
            width: 12.0,
            height: 6.0,
        },
        tile: Rect {
            x: 3.0,
            y: 4.0,
            width: 4.0,
            height: 2.0,
        },
        radius: 0.0,
        gradient: Arc::new(LinearGradient {
            direction: GradientDirection::Angle(90.0),
            stops: Arc::from([GradientStop {
                color: GradientColor::CurrentColor,
                position: None,
            }]),
        }),
        current_color: Color::rgba(12, 34, 56, 255),
        stop_scale: 1.5,
        repeat,
    }
}

fn reply(commands: Vec<DrawCommand>) -> Reply {
    let page = crate::page::Page::from_html(
        url::Url::parse("about:blank").unwrap(),
        "<p id=x>kept</p>",
        false,
    );
    Reply {
        navigation: None,
        snapshot: Some(Snapshot {
            generation: 91,
            processed_edit_sequence: 3,
            task_state: TaskState::Idle,
            layout: LayoutResult {
                commands,
                hit_regions: Vec::new(),
                node_visibility: vec![Visibility::Visible; page.document.nodes.len()],
                content_height: 6.0,
            },
            images: page.images,
            document: page.document,
            title: "background".into(),
            url: "about:blank".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        }),
    }
}

// Literal public wire contract, no Encoder helpers or command serialization.
fn literal(mask: Option<u8>) -> Vec<u8> {
    let mut bytes = vec![if mask.is_some() { 11 } else { 10 }];
    if let Some(mask) = mask {
        bytes.push(mask);
    }
    for value in [-1.0f32, 2.0, 12.0, 6.0, 3.0, 4.0, 4.0, 2.0, 0.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&[12, 34, 56, 255]);
    bytes.extend_from_slice(&1.5f32.to_le_bytes());
    bytes.push(0); // angle
    bytes.extend_from_slice(&90.0f64.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 0]); // currentColor, omitted position
    bytes
}

#[test]
fn repeating_gradient_keeps_literal_tag10_bytes_and_default_flags() {
    let bytes = encode_reply(&reply(vec![command([true, true])])).unwrap();
    assert_eq!(literal(None).len(), 60);
    assert!(bytes.ends_with(&literal(None)));
    let snapshot = decode_reply(&bytes).unwrap().snapshot.unwrap();
    assert!(matches!(
        snapshot.layout.commands[0],
        DrawCommand::LinearGradient {
            repeat: [true, true],
            ..
        }
    ));
}

#[test]
fn extended_gradient_flags_have_literal_masks_and_independent_payloads() {
    for (repeat, mask) in [([false, false], 0), ([true, false], 1), ([false, true], 2)] {
        let bytes = encode_reply(&reply(vec![command(repeat), command(repeat)])).unwrap();
        assert!(bytes.ends_with(&literal(Some(mask))));
        let snapshot = decode_reply(&bytes).unwrap().snapshot.unwrap();
        let DrawCommand::LinearGradient {
            repeat: actual,
            gradient: first,
            tile,
            ..
        } = &snapshot.layout.commands[0]
        else {
            panic!()
        };
        let DrawCommand::LinearGradient {
            gradient: second, ..
        } = &snapshot.layout.commands[1]
        else {
            panic!()
        };
        assert_eq!(*actual, repeat);
        assert_eq!(
            (tile.x, tile.y, tile.width, tile.height),
            (3.0, 4.0, 4.0, 2.0)
        );
        assert!(!Arc::ptr_eq(first, second));
        assert!(!Arc::ptr_eq(&first.stops, &second.stops));
    }
}

#[test]
fn extended_gradient_accepts_known_mask3_but_encoder_canonicalizes_to_tag10() {
    let mut bytes = encode_reply(&reply(vec![command([false, false])])).unwrap();
    let offset = bytes.len() - literal(Some(0)).len();
    bytes[offset + 1] = 3;
    let decoded = decode_reply(&bytes).unwrap();
    let canonical = encode_reply(&decoded).unwrap();
    assert!(canonical.ends_with(&literal(None)));
    assert_eq!(canonical.len() + 1, bytes.len());
}

#[test]
fn extended_gradient_rejects_every_unknown_flag_and_truncated_payload() {
    let bytes = encode_reply(&reply(vec![command([false, false])])).unwrap();
    let offset = bytes.len() - literal(Some(0)).len();
    for mask in 4..=u8::MAX {
        let mut invalid = bytes.clone();
        invalid[offset + 1] = mask;
        assert!(decode_reply(&invalid).is_err(), "mask {mask}");
    }
    for cut in offset..bytes.len() {
        assert!(decode_reply(&bytes[..cut]).is_err(), "cut {cut}");
    }
    let mut invalid = bytes;
    invalid.push(0);
    assert!(decode_reply(&invalid).is_err());
}

#[test]
fn extended_gradient_retains_geometry_stop_validation_and_shared_byte_admission() {
    let bytes = encode_reply(&reply(vec![command([false, false])])).unwrap();
    let offset = bytes.len() - literal(Some(0)).len();
    for relative in [2usize, 18, 26, 30, 34, 42] {
        let mut invalid = bytes.clone();
        invalid[offset + relative..offset + relative + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_reply(&invalid).is_err());
    }
    for count in [0u32, 33, u32::MAX] {
        let mut invalid = bytes.clone();
        invalid[offset + 55..offset + 59].copy_from_slice(&count.to_le_bytes());
        assert!(decode_reply(&invalid).is_err());
    }
    // The extra mask is wire metadata; retained payload cost is unchanged.
    let raw = literal(Some(0));
    let mut gradient_wire = b"ERWC\x63".to_vec();
    gradient_wire.extend_from_slice(&raw[46..]);
    let charge = std::mem::size_of::<GradientStop>()
        + std::mem::size_of::<LinearGradient>()
        + 4 * std::mem::size_of::<usize>()
        + std::mem::align_of::<GradientStop>()
        + std::mem::align_of::<LinearGradient>();
    let mut budget = charge - 1;
    assert!(decode_gradient(&mut Decoder::new(&gradient_wire, 99).unwrap(), &mut budget).is_err());
    assert_eq!(budget, charge - 1);
    let mut budget = 2 * charge;
    for _ in 0..2 {
        decode_gradient(&mut Decoder::new(&gradient_wire, 99).unwrap(), &mut budget).unwrap();
    }
    assert_eq!(budget, 0);
}

#[test]
fn mixed_repeat_modes_keep_scope_order_and_document_identity() {
    let commands = vec![
        DrawCommand::PushFixed,
        DrawCommand::PushOpacity { opacity: 0.5 },
        command([true, true]),
        command([true, false]),
        command([false, true]),
        command([false, false]),
        DrawCommand::PopOpacity,
        DrawCommand::PopFixed,
    ];
    let decoded = decode_reply(&encode_reply(&reply(commands)).unwrap())
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(
        (decoded.generation, decoded.processed_edit_sequence),
        (91, 3)
    );
    assert!(decoded.document.query_selector("#x").is_some());
    assert!(decoded.images.is_empty());
    for (command, expected) in decoded.layout.commands[2..6].iter().zip([
        [true, true],
        [true, false],
        [false, true],
        [false, false],
    ]) {
        let DrawCommand::LinearGradient { repeat, .. } = command else {
            panic!()
        };
        assert_eq!(*repeat, expected);
    }
    assert!(matches!(
        decoded.layout.commands[6],
        DrawCommand::PopOpacity
    ));
    assert!(matches!(decoded.layout.commands[7], DrawCommand::PopFixed));
}
