use super::*;

fn shape() -> RoundedBox {
    RoundedBox {
        rect: Rect {
            x: -1.5,
            y: 2.25,
            width: 20.0,
            height: 12.0,
        },
        radii: [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]],
    }
}
fn hole() -> RoundedBox {
    RoundedBox {
        rect: Rect {
            x: 2.0,
            y: 3.0,
            width: 6.0,
            height: 4.0,
        },
        radii: [[0.5, 1.0], [1.5, 2.0], [2.5, 3.0], [3.5, 4.0]],
    }
}
fn tile() -> Rect {
    Rect {
        x: 0.5,
        y: -3.25,
        width: 4.0,
        height: 2.0,
    }
}
fn gradient() -> Arc<LinearGradient> {
    Arc::new(LinearGradient {
        direction: GradientDirection::Angle(-45.0),
        stops: Arc::from([
            GradientStop {
                color: GradientColor::Color(Color::rgba(1, 2, 3, 4)),
                position: Some(GradientPosition {
                    px: -12.5,
                    percent: 25.0,
                }),
            },
            GradientStop {
                color: GradientColor::CurrentColor,
                position: None,
            },
        ]),
    })
}
fn paint(kind: u8, flags: u8, key: &str) -> BoxPaint {
    let repeat = [flags & 1 != 0, flags & 2 != 0];
    match kind {
        0 => BoxPaint::Color(Color::rgba(5, 6, 7, 8)),
        1 => BoxPaint::Image {
            tile: tile(),
            repeat,
            key: key.into(),
        },
        2 => BoxPaint::LinearGradient {
            tile: tile(),
            repeat,
            gradient: gradient(),
            current_color: Color::rgba(12, 34, 56, 78),
            stop_scale: 1.5,
        },
        _ => panic!(),
    }
}
fn command(kind: u8, flags: u8, cutout: bool, key: &str) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(Decoration {
        shape: shape(),
        cutout: cutout.then(hole),
        paint: paint(kind, flags, key),
    }))
}
fn reply(commands: Vec<DrawCommand>) -> Reply {
    let page = crate::page::Page::from_html(
        url::Url::parse("about:blank").unwrap(),
        "<p id=kept>box</p>",
        false,
    );
    Reply {
        navigation: None,
        snapshot: Some(Snapshot {
            generation: 91,
            processed_edit_sequence: 7,
            task_state: TaskState::Idle,
            layout: LayoutResult {
                commands,
                hit_regions: Vec::new(),
                content_height: 24.0,
            },
            images: page.images,
            document: page.document,
            title: "box decoration".into(),
            url: "about:blank".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        }),
    }
}
fn floats(out: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
}
// Literal wire order authored before the production branch, never Encoder output.
fn literal(kind: u8, flags: u8, cutout: bool, key: &str) -> Vec<u8> {
    let mut out = vec![13];
    floats(
        &mut out,
        &[
            -1.5, 2.25, 20.0, 12.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
        ],
    );
    out.push(u8::from(cutout));
    if cutout {
        floats(
            &mut out,
            &[2.0, 3.0, 6.0, 4.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0],
        );
    }
    out.push(kind);
    if kind == 0 {
        out.extend_from_slice(&[5, 6, 7, 8]);
        return out;
    }
    out.push(flags);
    floats(&mut out, &[0.5, -3.25, 4.0, 2.0]);
    if kind == 1 {
        out.extend_from_slice(&(key.len() as u32).to_le_bytes());
        out.extend_from_slice(key.as_bytes());
    } else {
        out.extend_from_slice(&[12, 34, 56, 78]);
        floats(&mut out, &[1.5]);
        out.push(0);
        out.extend_from_slice(&(-45.0f64).to_le_bytes());
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&[0, 1, 2, 3, 4, 1]);
        out.extend_from_slice(&(-12.5f64).to_le_bytes());
        out.extend_from_slice(&25.0f64.to_le_bytes());
        out.extend_from_slice(&[1, 0]);
    }
    out
}
fn wire(raw: &[u8], count: u32) -> Vec<u8> {
    let mut out = encode_reply(&reply(Vec::new())).unwrap();
    let end = out.len();
    assert_eq!(&out[end - 4..], &[0; 4]);
    out[end - 4..].copy_from_slice(&count.to_le_bytes());
    out.extend_from_slice(raw);
    out
}
fn set_shape(shape: &mut RoundedBox, index: usize, value: f32) {
    match index {
        0 => shape.rect.x = value,
        1 => shape.rect.y = value,
        2 => shape.rect.width = value,
        3 => shape.rect.height = value,
        _ => shape.radii[(index - 4) / 2][(index - 4) % 2] = value,
    }
}
fn retained(count: usize) -> usize {
    use std::mem::{align_of, size_of};
    count * size_of::<GradientStop>()
        + size_of::<LinearGradient>()
        + 4 * size_of::<usize>()
        + align_of::<GradientStop>()
        + align_of::<LinearGradient>()
}

#[test]
fn box_color_wire_has_explicit_raw_corners_and_optional_cutout() {
    for cutout in [false, true] {
        let raw = literal(0, 0, cutout, "");
        assert_eq!(raw.len(), if cutout { 103 } else { 55 });
        let encoded = encode_reply(&reply(vec![command(0, 0, cutout, "")])).unwrap();
        assert_eq!(encoded, wire(&raw, 1));
        let decoded = decode_reply(&encoded).unwrap();
        assert_eq!(encode_reply(&decoded).unwrap(), encoded);
        let snapshot = decoded.snapshot.unwrap();
        assert_eq!(
            (snapshot.generation, snapshot.processed_edit_sequence),
            (91, 7)
        );
        assert!(snapshot.document.query_selector("#kept").is_some());
        let DrawCommand::BoxDecoration(payload) = &snapshot.layout.commands[0] else {
            panic!()
        };
        let Decoration {
            shape: actual,
            cutout: inner,
            paint: BoxPaint::Color(color),
        } = payload.as_ref()
        else {
            panic!()
        };
        assert_eq!(
            actual.radii,
            [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]]
        );
        assert_eq!(*color, Color::rgba(5, 6, 7, 8));
        assert_eq!(inner.is_some(), cutout);
        if let Some(inner) = inner {
            assert_eq!(inner.radii, hole().radii);
        }
    }
}

#[test]
fn box_image_four_masks_and_opaque_missing_keys_roundtrip() {
    for flags in 0..4 {
        for key in [
            "",
            "Tile/é.png",
            "https://missing.invalid/A",
            "../not-opened",
            "nul\0key",
        ] {
            let raw = literal(1, flags, false, key);
            assert_eq!(raw.len(), 72 + key.len());
            let bytes = encode_reply(&reply(vec![command(1, flags, false, key)])).unwrap();
            assert_eq!(bytes, wire(&raw, 1));
            let decoded = decode_reply(&bytes).unwrap();
            assert_eq!(encode_reply(&decoded).unwrap(), bytes);
            assert!(decoded.snapshot.unwrap().images.is_empty());
        }
    }
}

#[test]
fn box_gradient_four_masks_preserve_signed_stops_and_independent_arcs() {
    for flags in 0..4 {
        let original = command(2, flags, true, "");
        let DrawCommand::BoxDecoration(payload) = &original else {
            panic!()
        };
        let Decoration {
            paint:
                BoxPaint::LinearGradient {
                    gradient: original_gradient,
                    ..
                },
            ..
        } = payload.as_ref()
        else {
            panic!()
        };
        let original_gradient = Arc::clone(original_gradient);
        let raw = literal(2, flags, true, "");
        assert_eq!(raw.len(), 161);
        let bytes = encode_reply(&reply(vec![original])).unwrap();
        assert_eq!(bytes, wire(&raw, 1));
        let decoded = decode_reply(&bytes).unwrap();
        assert_eq!(encode_reply(&decoded).unwrap(), bytes);
        let snapshot = decoded.snapshot.unwrap();
        let DrawCommand::BoxDecoration(payload) = &snapshot.layout.commands[0] else {
            panic!()
        };
        let Decoration {
            paint:
                BoxPaint::LinearGradient {
                    gradient,
                    repeat,
                    current_color,
                    stop_scale,
                    ..
                },
            ..
        } = payload.as_ref()
        else {
            panic!()
        };
        assert_eq!(**gradient, *original_gradient);
        assert!(!Arc::ptr_eq(gradient, &original_gradient));
        assert!(!Arc::ptr_eq(&gradient.stops, &original_gradient.stops));
        assert_eq!(*repeat, [flags & 1 != 0, flags & 2 != 0]);
        assert_eq!(*current_color, Color::rgba(12, 34, 56, 78));
        assert_eq!(*stop_scale, 1.5);
    }
}

#[test]
fn box_unknown_paint_cutout_and_all_repeat_high_bits_reject() {
    for value in 2..=255u8 {
        let mut raw = literal(0, 0, false, "");
        raw[49] = value;
        assert!(decode_reply(&wire(&raw, 1)).is_err());
    }
    for value in 3..=255u8 {
        let mut raw = literal(0, 0, false, "");
        raw[50] = value;
        assert!(decode_reply(&wire(&raw, 1)).is_err());
    }
    for kind in [1, 2] {
        for value in 4..=255u8 {
            let mut raw = literal(kind, 0, false, "x");
            raw[51] = value;
            assert!(decode_reply(&wire(&raw, 1)).is_err());
        }
    }
}

#[test]
fn box_all_payload_truncations_and_trailing_bytes_reject() {
    for kind in 0..3 {
        for cutout in [false, true] {
            let raw = literal(kind, 3, cutout, "é");
            for cut in 0..raw.len() {
                assert!(
                    decode_reply(&wire(&raw[..cut], 1)).is_err(),
                    "kind {kind} cutout {cutout} cut {cut}"
                );
            }
            let mut raw = raw;
            raw.push(0);
            assert!(decode_reply(&wire(&raw, 1)).is_err());
        }
    }
}

#[test]
fn box_all_shape_fields_reject_nonfinite_and_raw_bounds_on_both_sides() {
    for inner in [false, true] {
        for index in 0..12 {
            for value in [
                f32::NAN,
                f32::INFINITY,
                f32::NEG_INFINITY,
                16_000_001.0,
                -16_000_001.0,
            ] {
                let mut raw = literal(0, 0, true, "");
                let at = if inner { 50 } else { 1 } + index * 4;
                raw[at..at + 4].copy_from_slice(&value.to_le_bytes());
                assert!(decode_reply(&wire(&raw, 1)).is_err());
                let mut value_command = command(0, 0, true, "");
                let DrawCommand::BoxDecoration(payload) = &mut value_command else {
                    panic!()
                };
                let Decoration { shape, cutout, .. } = Arc::get_mut(payload).unwrap();
                set_shape(
                    if inner {
                        cutout.as_mut().unwrap()
                    } else {
                        shape
                    },
                    index,
                    value,
                );
                assert!(encode_reply(&reply(vec![value_command])).is_err());
            }
        }
    }
}

#[test]
fn box_negative_dimensions_radii_and_out_of_bound_endpoints_reject() {
    for inner in [false, true] {
        for index in 2..12 {
            let mut raw = literal(0, 0, true, "");
            let at = if inner { 50 } else { 1 } + index * 4;
            raw[at..at + 4].copy_from_slice(&(-1.0f32).to_le_bytes());
            assert!(decode_reply(&wire(&raw, 1)).is_err());
            let mut value = command(0, 0, true, "");
            let DrawCommand::BoxDecoration(payload) = &mut value else {
                panic!()
            };
            let Decoration { shape, cutout, .. } = Arc::get_mut(payload).unwrap();
            set_shape(
                if inner {
                    cutout.as_mut().unwrap()
                } else {
                    shape
                },
                index,
                -1.0,
            );
            assert!(encode_reply(&reply(vec![value])).is_err());
        }
    }
    for index in [0, 1] {
        let mut raw = literal(0, 0, false, "");
        raw[1 + index * 4..5 + index * 4].copy_from_slice(&16_000_000.0f32.to_le_bytes());
        assert!(decode_reply(&wire(&raw, 1)).is_err());
        let mut value = command(0, 0, false, "");
        let DrawCommand::BoxDecoration(payload) = &mut value else {
            panic!()
        };
        let Decoration { shape, .. } = Arc::get_mut(payload).unwrap();
        set_shape(shape, index, 16_000_000.0);
        assert!(encode_reply(&reply(vec![value])).is_err());
    }
    let mut value = command(0, 0, true, "");
    let DrawCommand::BoxDecoration(payload) = &mut value else {
        panic!()
    };
    let Decoration { shape, cutout, .. } = Arc::get_mut(payload).unwrap();
    shape.rect = Rect {
        x: -16_000_000.0,
        y: 16_000_000.0,
        width: -0.0,
        height: 0.0,
    };
    shape.radii = [[-0.0, f32::from_bits(1)]; 4];
    *cutout = Some(RoundedBox {
        rect: Rect {
            x: 100.0,
            y: 100.0,
            width: 2.0,
            height: 3.0,
        },
        radii: [[16_000_000.0; 2]; 4],
    });
    let decoded = decode_reply(&encode_reply(&reply(vec![value])).unwrap()).unwrap();
    let snapshot = decoded.snapshot.unwrap();
    let DrawCommand::BoxDecoration(payload) = &snapshot.layout.commands[0] else {
        panic!()
    };
    let Decoration { shape, cutout, .. } = payload.as_ref();
    assert_eq!(shape.rect.width.to_bits(), 0x8000_0000);
    assert_eq!(shape.radii[0][0].to_bits(), 0x8000_0000);
    assert_eq!(shape.radii[0][1].to_bits(), 1);
    assert_eq!(cutout.unwrap().rect.x, 100.0);
}

#[test]
fn box_paint_tiles_and_gradient_scale_validate_before_variable_payload() {
    for kind in [1, 2] {
        for index in 0..4 {
            for number in [f32::NAN, f32::INFINITY, 16_000_001.0] {
                let mut raw = literal(kind, 0, false, "");
                raw[52 + index * 4..56 + index * 4].copy_from_slice(&number.to_le_bytes());
                assert!(decode_reply(&wire(&raw, 1)).is_err());
                let mut value = command(kind, 0, false, "");
                let DrawCommand::BoxDecoration(payload) = &mut value else {
                    panic!()
                };
                let Decoration { paint, .. } = Arc::get_mut(payload).unwrap();
                let t = match paint {
                    BoxPaint::Image { tile, .. } | BoxPaint::LinearGradient { tile, .. } => tile,
                    _ => panic!(),
                };
                match index {
                    0 => t.x = number,
                    1 => t.y = number,
                    2 => t.width = number,
                    _ => t.height = number,
                };
                assert!(encode_reply(&reply(vec![value])).is_err());
            }
        }
    }
    for kind in [1, 2] {
        for index in [2, 3] {
            for number in [-1.0f32, -0.0, 0.0] {
                let mut raw = literal(kind, 0, false, "");
                raw[52 + index * 4..56 + index * 4].copy_from_slice(&number.to_le_bytes());
                raw.truncate(68);
                assert_ne!(
                    decode_reply(&wire(&raw, 1)).err().unwrap(),
                    "truncated IPC message"
                );
            }
        }
    }
    for number in [-1.0f32, -0.0, 0.0, f32::NAN, f32::INFINITY, 16_000_001.0] {
        let mut raw = literal(2, 0, false, "");
        raw[72..76].copy_from_slice(&number.to_le_bytes());
        raw.truncate(76);
        assert_ne!(
            decode_reply(&wire(&raw, 1)).err().unwrap(),
            "truncated IPC message"
        );
        let mut value = command(2, 0, false, "");
        let DrawCommand::BoxDecoration(payload) = &mut value else {
            panic!()
        };
        let Decoration {
            paint: BoxPaint::LinearGradient { stop_scale, .. },
            ..
        } = Arc::get_mut(payload).unwrap()
        else {
            panic!()
        };
        *stop_scale = number;
        assert!(encode_reply(&reply(vec![value])).is_err());
    }
    for kind in [1, 2] {
        let mut value = command(kind, 0, false, "");
        let DrawCommand::BoxDecoration(payload) = &mut value else {
            panic!()
        };
        let Decoration { paint, .. } = Arc::get_mut(payload).unwrap();
        let t = match paint {
            BoxPaint::Image { tile, .. } | BoxPaint::LinearGradient { tile, .. } => tile,
            _ => panic!(),
        };
        t.width = f32::from_bits(1);
        t.height = f32::from_bits(1);
        assert!(decode_reply(&encode_reply(&reply(vec![value])).unwrap()).is_ok());
    }
}

#[test]
fn box_keys_reject_utf8_and_length_errors_before_copy() {
    for length in [16_777_217u32, u32::MAX] {
        let mut raw = literal(1, 0, false, "");
        raw[68..72].copy_from_slice(&length.to_le_bytes());
        assert_eq!(
            decode_reply(&wire(&raw, 1)).err().unwrap(),
            "IPC collection limit exceeded"
        );
    }
    let mut raw = literal(1, 0, false, "x");
    raw[72] = 0xff;
    assert_eq!(
        decode_reply(&wire(&raw, 1)).err().unwrap(),
        "invalid IPC UTF-8"
    );
    let mut raw = literal(1, 0, false, "");
    raw[68..72].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        decode_reply(&wire(&raw, 1)).err().unwrap(),
        "truncated IPC message"
    );
}

#[test]
fn box_key_exact_and_one_short_share_existing_sender_and_receiver_counters() {
    let mut e = Encoder::new(99);
    let mut keys = 3;
    let mut gradients = MAX_GRADIENT_BYTES;
    e.budget_string("x", &mut keys);
    encode_box_decoration(
        &mut e,
        shape(),
        None,
        &paint(1, 0, "é"),
        &mut keys,
        &mut gradients,
    );
    assert_eq!(keys, 0);
    let bytes = e.finish().unwrap();
    let mut d = Decoder::new(&bytes, 99).unwrap();
    let mut keys = 3;
    assert_eq!(d.budget_string(&mut keys).unwrap(), "x");
    assert_eq!(d.byte().unwrap(), 13);
    decode_box_decoration(&mut d, &mut keys, &mut gradients).unwrap();
    d.end().unwrap();
    assert_eq!(keys, 0);
    let mut e = Encoder::new(99);
    let mut keys = 1;
    encode_box_decoration(
        &mut e,
        shape(),
        None,
        &paint(1, 0, "é"),
        &mut keys,
        &mut gradients,
    );
    assert!(e.finish().is_err());
    assert_eq!(keys, 1);
    let bytes = [b"ERWA\x63".as_slice(), literal(1, 0, false, "é").as_slice()].concat();
    let mut d = Decoder::new(&bytes, 99).unwrap();
    assert_eq!(d.byte().unwrap(), 13);
    assert!(decode_box_decoration(&mut d, &mut keys, &mut gradients).is_err());
    assert_eq!(keys, 1);
}

#[test]
fn box_image_keys_share_full_cap_with_alias_old_and_tiled_commands() {
    let large = "k".repeat(16_777_213);
    let mut value = reply(vec![
        DrawCommand::Image {
            rect: tile(),
            key: "b".into(),
        },
        DrawCommand::BackgroundImage {
            rect: tile(),
            tile: tile(),
            repeat: [true, true],
            radius: 0.0,
            key: "c".into(),
        },
        command(1, 3, false, &large),
    ]);
    value.snapshot.as_mut().unwrap().images.insert(
        "a".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
        }),
    );
    let mut bytes = encode_reply(&value).unwrap();
    let decoded = decode_reply(&bytes).unwrap();
    assert_eq!(decoded.snapshot.unwrap().images.len(), 1);
    let at = bytes.len() - large.len() - 4;
    bytes[at..at + 4].copy_from_slice(&16_777_214u32.to_le_bytes());
    bytes.push(b'k');
    assert!(decode_reply(&bytes).is_err());
    let DrawCommand::BoxDecoration(payload) =
        &mut value.snapshot.as_mut().unwrap().layout.commands[2]
    else {
        panic!()
    };
    let Decoration {
        paint: BoxPaint::Image { key, .. },
        ..
    } = Arc::get_mut(payload).unwrap()
    else {
        panic!()
    };
    key.push('k');
    assert!(encode_reply(&value).is_err());
}

#[test]
fn box_gradients_share_exact_retained_admission_with_old_payloads() {
    let charge = retained(2);
    let mut e = Encoder::new(99);
    let mut left = 2 * charge;
    let mut keys = 0;
    encode_gradient(&mut e, &gradient(), &mut left);
    encode_box_decoration(
        &mut e,
        shape(),
        None,
        &paint(2, 0, ""),
        &mut keys,
        &mut left,
    );
    assert_eq!(left, 0);
    let bytes = e.finish().unwrap();
    let mut d = Decoder::new(&bytes, 99).unwrap();
    let mut left = 2 * charge;
    decode_gradient(&mut d, &mut left).unwrap();
    assert_eq!(d.byte().unwrap(), 13);
    decode_box_decoration(&mut d, &mut keys, &mut left).unwrap();
    assert_eq!(left, 0);
    d.end().unwrap();
    let mut e = Encoder::new(99);
    let mut left = charge - 1;
    encode_box_decoration(
        &mut e,
        shape(),
        None,
        &paint(2, 0, ""),
        &mut keys,
        &mut left,
    );
    assert!(e.finish().is_err());
    assert_eq!(left, charge - 1);
    let bytes = [b"ERWA\x63".as_slice(), literal(2, 0, false, "").as_slice()].concat();
    let mut d = Decoder::new(&bytes, 99).unwrap();
    assert_eq!(d.byte().unwrap(), 13);
    assert!(decode_box_decoration(&mut d, &mut keys, &mut left).is_err());
    assert_eq!(left, charge - 1);
    let count = MAX_GRADIENT_BYTES / charge + 1;
    let commands = (0..count)
        .map(|i| {
            if i % 2 == 0 {
                command(2, 0, false, "")
            } else {
                DrawCommand::LinearGradient {
                    rect: tile(),
                    tile: tile(),
                    radius: 0.0,
                    gradient: gradient(),
                    current_color: Color::rgb(0, 0, 0),
                    stop_scale: 1.0,
                    repeat: [true, true],
                }
            }
        })
        .collect();
    assert!(encode_reply(&reply(commands)).is_err());
}

#[test]
fn box_gradient_malformed_counts_tags_and_nonfinite_positions_reject() {
    for count in [0u32, 33, u32::MAX] {
        let mut raw = literal(2, 0, false, "");
        raw[85..89].copy_from_slice(&count.to_le_bytes());
        assert!(decode_reply(&wire(&raw, 1)).is_err());
    }
    for (at, value) in [(76, 2), (89, 2), (94, 2), (111, 2), (112, 2)] {
        let mut raw = literal(2, 0, false, "");
        raw[at] = value;
        assert!(decode_reply(&wire(&raw, 1)).is_err());
    }
    for at in [77, 95, 103] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut raw = literal(2, 0, false, "");
            raw[at..at + 8].copy_from_slice(&value.to_le_bytes());
            assert!(decode_reply(&wire(&raw, 1)).is_err());
        }
    }
    let mut value = command(2, 0, false, "");
    let DrawCommand::BoxDecoration(payload) = &mut value else {
        panic!()
    };
    let Decoration {
        paint: BoxPaint::LinearGradient { gradient, .. },
        ..
    } = Arc::get_mut(payload).unwrap()
    else {
        panic!()
    };
    *gradient = Arc::new(LinearGradient {
        direction: GradientDirection::Angle(0.0),
        stops: Arc::from([]),
    });
    assert!(encode_reply(&reply(vec![value])).is_err());
}

#[test]
fn box_commands_obey_existing_scope_balance_and_depth_without_new_scope() {
    let mut raw = vec![6, 8];
    floats(&mut raw, &[0.5]);
    raw.push(0);
    floats(&mut raw, &[0.0, 0.0, 10.0, 10.0]);
    raw.extend(literal(0, 0, true, ""));
    raw.extend([1, 9, 7]);
    let decoded = decode_reply(&wire(&raw, 7)).unwrap();
    assert_eq!(decoded.snapshot.unwrap().layout.commands.len(), 7);
    for closing in [[9, 1, 7], [1, 7, 9], [1, 9, 9]] {
        let mut bad = raw.clone();
        let end = bad.len();
        bad[end - 3..].copy_from_slice(&closing);
        assert!(decode_reply(&wire(&bad, 7)).is_err());
    }
    for depth in [128usize, 129] {
        let mut raw = vec![6; depth];
        raw.extend(literal(0, 0, false, ""));
        raw.extend(vec![7; depth]);
        assert_eq!(
            decode_reply(&wire(&raw, (depth * 2 + 1) as u32)).is_ok(),
            depth == 128
        );
    }
}

#[test]
fn box_extension_preserves_the_old_tag12_payload_exactly() {
    let old = DrawCommand::BackgroundImage {
        rect: shape().rect,
        tile: tile(),
        repeat: [true, false],
        radius: 2.5,
        key: "x".into(),
    };
    let mut raw = vec![12, 1];
    floats(
        &mut raw,
        &[-1.5, 2.25, 20.0, 12.0, 0.5, -3.25, 4.0, 2.0, 2.5],
    );
    raw.extend([1, 0, 0, 0, b'x']);
    assert_eq!(encode_reply(&reply(vec![old])).unwrap(), wire(&raw, 1));
    assert!(decode_reply(&wire(&raw, 1)).is_ok());
}
