use super::*;

fn rect() -> Rect {
    Rect {
        x: -1.5,
        y: 2.25,
        width: 20.0,
        height: 12.0,
    }
}

fn command(repeat: [bool; 2], key: &str) -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: rect(),
        tile: Rect {
            x: 0.5,
            y: -3.25,
            width: 4.0,
            height: 2.0,
        },
        repeat,
        radius: 2.5,
        key: key.into(),
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
            generation: 73,
            processed_edit_sequence: 5,
            task_state: TaskState::Idle,
            layout: LayoutResult {
                commands,
                hit_regions: Vec::new(),
                node_visibility: vec![Visibility::Visible; page.document.nodes.len()],
                content_height: 24.0,
            },
            images: page.images,
            document: page.document,
            title: "tiled raster".into(),
            url: "about:blank".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        }),
    }
}

// Literal tag contract, independent of Encoder and the candidate branch.
fn literal(flags: u8, key: &str) -> Vec<u8> {
    let mut bytes = vec![12, flags];
    for number in [-1.5f32, 2.25, 20.0, 12.0, 0.5, -3.25, 4.0, 2.0, 2.5] {
        bytes.extend_from_slice(&number.to_le_bytes());
    }
    bytes.extend_from_slice(&(key.len() as u32).to_le_bytes());
    bytes.extend_from_slice(key.as_bytes());
    bytes
}

fn wire(raw: &[u8], count: u32) -> Vec<u8> {
    let mut bytes = encode_reply(&reply(Vec::new())).unwrap();
    let end = bytes.len();
    assert_eq!(&bytes[end - 4..], &[0; 4]);
    bytes[end - 4..].copy_from_slice(&count.to_le_bytes());
    bytes.extend_from_slice(raw);
    bytes
}

fn set_scalar(command: &mut DrawCommand, index: usize, value: f32) {
    let DrawCommand::BackgroundImage {
        rect, tile, radius, ..
    } = command
    else {
        panic!()
    };
    let field = match index {
        0 => &mut rect.x,
        1 => &mut rect.y,
        2 => &mut rect.width,
        3 => &mut rect.height,
        4 => &mut tile.x,
        5 => &mut tile.y,
        6 => &mut tile.width,
        7 => &mut tile.height,
        8 => radius,
        _ => panic!(),
    };
    *field = value;
}

#[test]
fn all_four_repeat_flags_match_literal_tag12_and_preserve_fields() {
    for (repeat, flags) in [
        ([false, false], 0),
        ([true, false], 1),
        ([false, true], 2),
        ([true, true], 3),
    ] {
        let expected = literal(flags, "Tile/é.png");
        assert_eq!(expected.len(), 53);
        let bytes = encode_reply(&reply(vec![command(repeat, "Tile/é.png")])).unwrap();
        assert!(bytes.ends_with(&expected));
        assert_eq!(bytes, wire(&expected, 1));
        let decoded = decode_reply(&wire(&expected, 1)).unwrap().snapshot.unwrap();
        let DrawCommand::BackgroundImage {
            rect,
            tile,
            radius,
            key,
            repeat: actual,
        } = &decoded.layout.commands[0]
        else {
            panic!()
        };
        assert_eq!(*actual, repeat);
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (-1.5, 2.25, 20.0, 12.0)
        );
        assert_eq!(
            (tile.x, tile.y, tile.width, tile.height),
            (0.5, -3.25, 4.0, 2.0)
        );
        assert_eq!(*radius, 2.5);
        assert_eq!(key, "Tile/é.png");
        assert!(decoded.images.is_empty());
        assert_eq!(
            (decoded.generation, decoded.processed_edit_sequence),
            (73, 5)
        );
        assert!(decoded.document.query_selector("#x").is_some());
    }
}

#[test]
fn prior_tags_zero_through_eleven_keep_their_literal_payloads() {
    let gradient = |repeat| DrawCommand::LinearGradient {
        rect: rect(),
        tile: rect(),
        radius: 0.0,
        gradient: Arc::new(LinearGradient {
            direction: GradientDirection::Angle(90.0),
            stops: Arc::from([GradientStop {
                color: GradientColor::CurrentColor,
                position: None,
            }]),
        }),
        current_color: Color::rgba(1, 2, 3, 4),
        stop_scale: 1.0,
        repeat,
    };
    let commands = vec![
        DrawCommand::PushFixed,
        DrawCommand::PushOpacity { opacity: 0.5 },
        DrawCommand::PushClip { rect: rect() },
        DrawCommand::Rect {
            rect: rect(),
            color: Color::rgba(1, 2, 3, 4),
            radius: 0.0,
        },
        DrawCommand::Text {
            x: 1.0,
            y: 2.0,
            text: "Hi".into(),
            size: 12.0,
            color: Color::rgba(1, 2, 3, 4),
            bold: true,
            italic: false,
            monospace: true,
        },
        DrawCommand::Image {
            rect: rect(),
            key: "x".into(),
        },
        DrawCommand::Line {
            x1: 1.0,
            y1: 2.0,
            x2: 3.0,
            y2: 4.0,
            color: Color::rgba(1, 2, 3, 4),
            width: 1.0,
        },
        gradient([true, true]),
        gradient([true, false]),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
        DrawCommand::PopFixed,
    ];
    let mut expected = vec![6, 8];
    expected.extend_from_slice(&0.5f32.to_le_bytes());
    let floats = |out: &mut Vec<u8>, values: &[f32]| {
        for number in values {
            out.extend_from_slice(&number.to_le_bytes());
        }
    };
    let bounds = [-1.5, 2.25, 20.0, 12.0];
    expected.push(0);
    floats(&mut expected, &bounds);
    expected.push(2);
    floats(&mut expected, &bounds);
    expected.extend_from_slice(&[1, 2, 3, 4]);
    floats(&mut expected, &[0.0]);
    expected.push(3);
    floats(&mut expected, &[1.0, 2.0]);
    expected.extend_from_slice(&[2, 0, 0, 0, b'H', b'i']);
    floats(&mut expected, &[12.0]);
    expected.extend_from_slice(&[1, 2, 3, 4, 1, 0, 1]);
    expected.push(4);
    floats(&mut expected, &bounds);
    expected.extend_from_slice(&[1, 0, 0, 0, b'x']);
    expected.push(5);
    floats(&mut expected, &[1.0, 2.0, 3.0, 4.0]);
    expected.extend_from_slice(&[1, 2, 3, 4]);
    floats(&mut expected, &[1.0]);
    for tag in [10, 11] {
        expected.push(tag);
        if tag == 11 {
            expected.push(1);
        }
        floats(&mut expected, &bounds);
        floats(&mut expected, &bounds);
        floats(&mut expected, &[0.0]);
        expected.extend_from_slice(&[1, 2, 3, 4]);
        floats(&mut expected, &[1.0]);
        expected.push(0);
        expected.extend_from_slice(&90.0f64.to_le_bytes());
        expected.extend_from_slice(&[1, 0, 0, 0, 1, 0]);
    }
    expected.extend_from_slice(&[1, 9, 7]);
    assert_eq!(encode_reply(&reply(commands)).unwrap(), wire(&expected, 12));
    assert_eq!(
        decode_reply(&wire(&expected, 12))
            .unwrap()
            .snapshot
            .unwrap()
            .layout
            .commands
            .len(),
        12
    );
}

#[test]
fn every_truncation_unknown_flag_and_trailing_byte_is_rejected() {
    let raw = literal(0, "x");
    let complete = wire(&raw, 1);
    let start = complete.len() - raw.len();
    for cut in start..complete.len() {
        assert!(decode_reply(&complete[..cut]).is_err(), "cut {cut}");
    }
    for flags in 4..=u8::MAX {
        let mut invalid = raw.clone();
        invalid[1] = flags;
        assert!(decode_reply(&wire(&invalid, 1)).is_err(), "flags {flags}");
    }
    let mut invalid = raw;
    invalid[0] = 13;
    assert!(decode_reply(&wire(&invalid, 1)).is_err());
    let mut trailing = complete;
    trailing.push(0);
    assert!(decode_reply(&trailing).is_err());
}

#[test]
fn all_nine_scalars_enforce_finite_geometry_limits_on_both_sides() {
    for index in 0..9 {
        for value in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            16_000_001.0,
            -16_000_001.0,
        ] {
            let mut raw = literal(3, "x");
            raw[2 + index * 4..6 + index * 4].copy_from_slice(&value.to_le_bytes());
            assert!(
                decode_reply(&wire(&raw, 1)).is_err(),
                "field {index} {value}"
            );
            let mut invalid = command([true, true], "x");
            set_scalar(&mut invalid, index, value);
            assert!(encode_reply(&reply(vec![invalid])).is_err());
        }
    }
    for index in [0, 1, 4, 5] {
        for value in [-16_000_000.0, 16_000_000.0] {
            let mut valid = command([false, false], "x");
            set_scalar(&mut valid, index, value);
            assert!(decode_reply(&encode_reply(&reply(vec![valid])).unwrap()).is_ok());
        }
    }
}

#[test]
fn negative_sizes_radius_and_nonpositive_tiles_reject_before_key_copy() {
    for (index, value) in [
        (2, -1.0f32),
        (3, -1.0),
        (6, -1.0),
        (7, -1.0),
        (8, -1.0),
        (6, 0.0),
        (6, -0.0),
        (7, 0.0),
        (7, -0.0),
    ] {
        let mut raw = literal(0, "x");
        raw[2 + index * 4..6 + index * 4].copy_from_slice(&value.to_le_bytes());
        // The claimed key is absent: geometry must be refused first.
        raw.truncate(38);
        assert_ne!(
            decode_reply(&wire(&raw, 1)).err().unwrap(),
            "truncated IPC message"
        );
        let mut invalid = command([false, false], "x");
        set_scalar(&mut invalid, index, value);
        assert!(encode_reply(&reply(vec![invalid])).is_err());
    }
}

#[test]
fn positive_subnormal_tiles_and_signed_zero_inert_rectangles_stay_legal() {
    let mut valid = command([false, true], "");
    for index in [2, 3, 8] {
        set_scalar(&mut valid, index, -0.0);
    }
    for index in [6, 7] {
        set_scalar(&mut valid, index, f32::from_bits(1));
    }
    let decoded = decode_reply(&encode_reply(&reply(vec![valid])).unwrap())
        .unwrap()
        .snapshot
        .unwrap();
    let DrawCommand::BackgroundImage {
        rect,
        tile,
        radius,
        key,
        ..
    } = &decoded.layout.commands[0]
    else {
        panic!()
    };
    assert_eq!(
        (
            rect.width.to_bits(),
            rect.height.to_bits(),
            radius.to_bits()
        ),
        (0x8000_0000, 0x8000_0000, 0x8000_0000)
    );
    assert_eq!((tile.width.to_bits(), tile.height.to_bits()), (1, 1));
    assert!(key.is_empty());
}

#[test]
fn keys_require_utf8_bounded_lengths_and_complete_payloads() {
    for length in [16_777_217u32, u32::MAX] {
        let mut raw = literal(0, "");
        raw[38..42].copy_from_slice(&length.to_le_bytes());
        assert_eq!(
            decode_reply(&wire(&raw, 1)).err().unwrap(),
            "IPC collection limit exceeded"
        );
    }
    let mut raw = literal(0, "x");
    raw[42] = 0xff;
    assert_eq!(
        decode_reply(&wire(&raw, 1)).err().unwrap(),
        "invalid IPC UTF-8"
    );
    let mut raw = literal(0, "");
    raw[38..42].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        decode_reply(&wire(&raw, 1)).err().unwrap(),
        "truncated IPC message"
    );
    for key in [
        "",
        "https://unavailable.invalid/Tile.PNG",
        "../../not-opened",
        "nul\0key",
    ] {
        let decoded = decode_reply(&wire(&literal(1, key), 1))
            .unwrap()
            .snapshot
            .unwrap();
        let DrawCommand::BackgroundImage { key: actual, .. } = &decoded.layout.commands[0] else {
            panic!()
        };
        assert_eq!(actual, key);
        assert!(decoded.images.is_empty());
    }
}

#[test]
fn sender_key_helper_checks_admission_before_copy_and_shares_remaining_bytes() {
    let mut exact = Encoder::new(99);
    let mut remaining = 2;
    exact.budget_string("é", &mut remaining);
    assert_eq!(remaining, 0);
    assert_eq!(exact.bytes, b"ERWC\x63\x02\0\0\0\xc3\xa9");
    let before = exact.bytes.clone();
    exact.budget_string("x", &mut remaining);
    assert_eq!(remaining, 0);
    assert_eq!(exact.bytes, before);
    assert!(exact.finish().is_err());
    let mut short = Encoder::new(99);
    let mut remaining = 1;
    short.budget_string("é", &mut remaining);
    assert_eq!(remaining, 1);
    assert_eq!(short.bytes, b"ERWC\x63");
    assert!(short.finish().is_err());
}

#[test]
fn aggregate_key_cap_includes_aliases_old_images_and_tiled_images() {
    assert_eq!(MAX_STRING, 16_777_216);
    let key = "x".repeat(16_777_214);
    let mut value = reply(vec![
        DrawCommand::Image {
            rect: rect(),
            key: "b".into(),
        },
        command([true, true], &key),
    ]);
    value.snapshot.as_mut().unwrap().images.insert(
        "a".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        }),
    );
    let mut bytes = encode_reply(&value).unwrap();
    let decoded = decode_reply(&bytes).unwrap().snapshot.unwrap();
    assert_eq!(decoded.images.len(), 1);
    let DrawCommand::BackgroundImage { key: actual, .. } = &decoded.layout.commands[1] else {
        panic!()
    };
    assert_eq!(actual.len(), 16_777_214);
    drop(decoded);
    let length_at = bytes.len() - key.len() - 4;
    bytes[length_at..length_at + 4].copy_from_slice(&16_777_215u32.to_le_bytes());
    bytes.push(b'x');
    assert_eq!(
        decode_reply(&bytes).err().unwrap(),
        "IPC collection limit exceeded"
    );
    let DrawCommand::BackgroundImage { key, .. } =
        &mut value.snapshot.as_mut().unwrap().layout.commands[1]
    else {
        panic!()
    };
    key.push('x');
    assert!(encode_reply(&value).is_err());
    // The individual cap still applies even when a caller offers more budget.
    let mut encoder = Encoder::new(99);
    let mut remaining = 16_777_218;
    let oversized = "x".repeat(16_777_217);
    encoder.budget_string(&oversized, &mut remaining);
    assert_eq!(remaining, 16_777_218);
    assert_eq!(encoder.bytes, b"ERWC\x63");
    assert!(encoder.finish().is_err());
}

#[test]
fn tiled_images_preserve_typed_scope_balance_and_depth_limit() {
    let commands = vec![
        DrawCommand::PushFixed,
        DrawCommand::PushOpacity { opacity: 0.5 },
        DrawCommand::PushClip { rect: rect() },
        command([false, false], "missing"),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
        DrawCommand::PopFixed,
    ];
    let decoded = decode_reply(&encode_reply(&reply(commands)).unwrap())
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(decoded.layout.commands.len(), 7);
    assert!(matches!(
        decoded.layout.commands[3],
        DrawCommand::BackgroundImage { .. }
    ));
    for closes in [[9, 1, 7], [1, 7, 9], [1, 9, 9]] {
        let mut bytes = encode_reply(&reply(vec![
            DrawCommand::PushFixed,
            DrawCommand::PushOpacity { opacity: 0.5 },
            DrawCommand::PushClip { rect: rect() },
            command([true, false], "missing"),
            DrawCommand::PopClip,
            DrawCommand::PopOpacity,
            DrawCommand::PopFixed,
        ]))
        .unwrap();
        let end = bytes.len();
        bytes[end - 3..].copy_from_slice(&closes);
        assert!(decode_reply(&bytes).is_err());
    }
    for depth in [128, 129] {
        let commands = (0..depth)
            .map(|_| DrawCommand::PushFixed)
            .chain(std::iter::once(command([true, false], "missing")))
            .chain((0..depth).map(|_| DrawCommand::PopFixed))
            .collect();
        assert_eq!(
            decode_reply(&encode_reply(&reply(commands)).unwrap()).is_ok(),
            depth == 128
        );
    }
}

#[test]
fn loaded_aliases_share_pixels_while_missing_tiled_keys_remain_unavailable() {
    let mut value = reply(vec![
        command([true, false], "a"),
        command([false, true], "b"),
        command([false, false], "missing"),
    ]);
    let raster = Arc::new(RasterImage {
        width: 2,
        height: 1,
        rgba: vec![255, 0, 0, 255, 0, 255, 0, 128],
    });
    let images = &mut value.snapshot.as_mut().unwrap().images;
    images.insert("a".into(), raster.clone());
    images.insert("b".into(), raster);
    let decoded = decode_reply(&encode_reply(&value).unwrap())
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(decoded.images.len(), 2);
    assert!(Arc::ptr_eq(&decoded.images["a"], &decoded.images["b"]));
    assert_eq!(decoded.images["a"].rgba, [255, 0, 0, 255, 0, 255, 0, 128]);
    assert!(!decoded.images.contains_key("missing"));
    assert_eq!(decoded.layout.commands.len(), 3);
}
