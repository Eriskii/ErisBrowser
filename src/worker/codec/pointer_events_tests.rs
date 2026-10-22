use super::*;

// Independently written wire fields: no Encoder is used to construct fixtures.
fn string(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
    bytes.extend_from_slice(text.as_bytes());
}

fn prefix(visibility: u8, hits: u32) -> Vec<u8> {
    let mut bytes = b"ERWC\x63".to_vec();
    bytes.extend_from_slice(&7u64.to_le_bytes());
    bytes.extend_from_slice(&3u64.to_le_bytes());
    bytes.push(0); // idle
    string(&mut bytes, "pointer");
    string(&mut bytes, "about:blank");
    bytes.extend_from_slice(&0f64.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes()); // diagnostics
    bytes.extend_from_slice(&0u32.to_le_bytes()); // root
    bytes.extend_from_slice(&[0, 0]); // scripting, no quirks
    string(&mut bytes, "UTF-8");
    bytes.push(0); // no frozen base
    bytes.extend_from_slice(&1u32.to_le_bytes()); // one document node
    bytes.push(0); // no parent
    bytes.extend_from_slice(&0u32.to_le_bytes()); // no children
    bytes.push(0); // document node kind
    bytes.extend_from_slice(&1u32.to_le_bytes()); // visibility count
    bytes.push(visibility);
    bytes.extend_from_slice(&0u32.to_le_bytes()); // rasters
    bytes.extend_from_slice(&0u32.to_le_bytes()); // image aliases
    bytes.extend_from_slice(&4f32.to_le_bytes()); // content height
    bytes.extend_from_slice(&hits.to_le_bytes());
    bytes
}

fn wire(values: &[u8]) -> Vec<u8> {
    let mut bytes = prefix(0, values.len() as u32);
    for (index, value) in values.iter().enumerate() {
        bytes.extend_from_slice(&0u32.to_le_bytes()); // node
        for scalar in [index as f32 * 3.0, 0.0, 2.0, 2.0] {
            bytes.extend_from_slice(&scalar.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0, *value]); // fixed, action, pointer events
    }
    bytes.extend_from_slice(&0u32.to_le_bytes()); // commands
    bytes
}

fn decode(bytes: &[u8]) -> Result<Snapshot> {
    let mut decoder = Decoder::new(bytes, 99)?;
    let snapshot = decode_snapshot(&mut decoder)?;
    decoder.end()?;
    Ok(snapshot)
}

#[test]
fn pointer_events_codec_literal_values_and_mixed_record_stride_round_trip() {
    for (values, expected) in [
        (vec![0], vec![PointerEvents::Auto]),
        (vec![1], vec![PointerEvents::None]),
        (vec![1, 0], vec![PointerEvents::None, PointerEvents::Auto]),
    ] {
        let bytes = wire(&values);
        let offset = prefix(0, values.len() as u32).len();
        assert_eq!(bytes.len(), offset + 23 * values.len() + 4);
        for (index, value) in values.iter().enumerate() {
            assert_eq!(bytes[offset + 23 * index + 22], *value);
        }
        let snapshot = decode(&bytes).unwrap();
        assert_eq!(
            snapshot
                .layout
                .hit_regions
                .iter()
                .map(|hit| hit.pointer_events)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(snapshot.layout.hit_regions.len(), values.len());
        for (index, hit) in snapshot.layout.hit_regions.iter().enumerate() {
            assert_eq!(hit.node, 0);
            assert_eq!(hit.action, HitAction::Node);
            assert!(!hit.fixed);
            assert_eq!(
                (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
                (index as f32 * 3.0, 0.0, 2.0, 2.0)
            );
        }
        let mut encoder = Encoder::new(99);
        encode_snapshot(&mut encoder, &snapshot);
        assert_eq!(encoder.finish().unwrap(), bytes);
    }
}

#[test]
fn pointer_events_codec_unknown_bytes_all_truncations_and_trailing_bytes_refuse() {
    for value in [2, 255] {
        assert!(
            matches!(decode(&wire(&[value])), Err(error) if error == "unknown IPC pointer events")
        );
    }
    let bytes = wire(&[1, 0]);
    for cut in 0..bytes.len() {
        assert!(decode(&bytes[..cut]).is_err(), "cut={cut}");
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(matches!(decode(&trailing), Err(error) if error == "trailing IPC bytes"));
}

#[test]
fn pointer_events_codec_none_preserves_prior_field_refusal_order() {
    let offset = prefix(0, 1).len();
    for (at, replacement, message) in [
        (
            offset,
            1u32.to_le_bytes().to_vec(),
            "IPC collection limit exceeded",
        ),
        (
            offset + 4,
            f32::NAN.to_le_bytes().to_vec(),
            "invalid IPC geometry",
        ),
        (
            offset + 12,
            (-1f32).to_le_bytes().to_vec(),
            "negative IPC rectangle size",
        ),
        (offset + 20, vec![2], "invalid IPC boolean"),
        (offset + 21, vec![2], "invalid IPC hit action"),
    ] {
        let mut bytes = wire(&[1]);
        bytes[at..at + replacement.len()].copy_from_slice(&replacement);
        assert!(
            matches!(decode(&bytes), Err(error) if error == message),
            "{message}"
        );
    }
    // Hidden-node authentication still precedes geometry and the new byte.
    let mut hidden = prefix(1, 1);
    hidden.extend_from_slice(&0u32.to_le_bytes());
    assert!(matches!(decode(&hidden), Err(error) if error == "IPC hit targets hidden node"));
}

#[test]
fn pointer_events_codec_zero_hits_old_headers_and_count_limit_keep_exact_boundaries() {
    assert!(decode(&wire(&[])).unwrap().layout.hit_regions.is_empty());
    assert!(
        matches!(decode(&prefix(0, MAX_COMMANDS as u32 + 1)), Err(error) if error == "IPC collection limit exceeded")
    );
    for magic in [b"ERWB", b"ERWA", b"ERW9", b"ERW8"] {
        let mut bytes = wire(&[1]);
        bytes[..4].copy_from_slice(magic);
        assert!(
            matches!(decode(&bytes), Err(error) if error == "invalid IPC version/message kind")
        );
    }
}

#[test]
fn pointer_events_codec_none_keeps_generated_summary_action_authentication() {
    let page = crate::page::Page::from_html(
        url::Url::parse("https://pointer.invalid/").unwrap(),
        "<details id=d>contents</details><p id=p>ordinary</p>",
        false,
    );
    let layout = page.layout(80.0, 60.0, &crate::graphics::Fonts::new());
    let mut reply = Reply {
        navigation: None,
        snapshot: Some(Snapshot {
            generation: 1,
            processed_edit_sequence: 0,
            task_state: TaskState::Idle,
            layout,
            images: page.images,
            document: page.document,
            title: "pointer".into(),
            url: "https://pointer.invalid/".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        }),
    };
    let snapshot = reply.snapshot.as_mut().unwrap();
    let ordinary = snapshot.document.query_selector("#p").unwrap();
    let hit = snapshot
        .layout
        .hit_regions
        .iter_mut()
        .find(|hit| hit.action == HitAction::DefaultSummary)
        .unwrap();
    hit.pointer_events = PointerEvents::None;
    let decoded = decode_reply(&encode_reply(&reply).unwrap()).unwrap();
    assert!(
        decoded
            .snapshot
            .unwrap()
            .layout
            .hit_regions
            .iter()
            .any(|hit| hit.action == HitAction::DefaultSummary
                && hit.pointer_events == PointerEvents::None)
    );
    reply
        .snapshot
        .as_mut()
        .unwrap()
        .layout
        .hit_regions
        .iter_mut()
        .find(|hit| hit.action == HitAction::DefaultSummary)
        .unwrap()
        .node = ordinary;
    assert!(
        matches!(decode_reply(&encode_reply(&reply).unwrap()), Err(error) if error == "invalid IPC hit action")
    );
}
