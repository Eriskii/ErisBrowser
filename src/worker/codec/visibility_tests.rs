use super::*;

// Minimal one-node DOM, authored directly from the snapshot wire field order.
fn prefix(visibility_count: usize, values: &[u8]) -> Vec<u8> {
    let mut e = Encoder::new(99);
    e.u64(7);
    e.u64(3);
    e.byte(0);
    e.string("visibility");
    e.string("about:blank");
    e.f64(0.0);
    e.u32(0); // diagnostics
    e.u32(0); // root
    e.boolean(false);
    e.byte(0); // no quirks
    e.string("UTF-8");
    e.boolean(false); // frozen base
    e.u32(1); // DOM nodes
    e.boolean(false); // parent
    e.u32(0); // children
    e.byte(0); // document node
    e.u32(visibility_count);
    e.raw(values);
    e.finish().unwrap()
}

fn complete(value: u8) -> Vec<u8> {
    let mut bytes = prefix(1, &[value]);
    bytes.extend_from_slice(&0u32.to_le_bytes()); // rasters
    bytes.extend_from_slice(&0u32.to_le_bytes()); // image aliases
    bytes.extend_from_slice(&0f32.to_le_bytes()); // content height
    bytes.extend_from_slice(&0u32.to_le_bytes()); // hits
    bytes.extend_from_slice(&0u32.to_le_bytes()); // commands
    bytes
}

#[test]
fn visibility_codec_roundtrip_preserves_each_exact_byte_and_rejects_old_magic() {
    for (value, visibility) in [(0, Visibility::Visible), (1, Visibility::Hidden)] {
        let bytes = complete(value);
        assert_eq!(&bytes[..4], b"ERWB");
        let mut decoder = Decoder::new(&bytes, 99).unwrap();
        let snapshot = decode_snapshot(&mut decoder).unwrap();
        decoder.end().unwrap();
        assert_eq!(snapshot.layout.node_visibility, [visibility]);
        assert_eq!(snapshot.document.nodes.len(), 1);
        let mut encoder = Encoder::new(99);
        encode_snapshot(&mut encoder, &snapshot);
        assert_eq!(encoder.finish().unwrap(), bytes);

        let reply = Reply {
            navigation: None,
            snapshot: Some(snapshot),
        };
        let encoded = encode_reply(&reply).unwrap();
        assert_eq!(
            decode_reply(&encoded)
                .unwrap()
                .snapshot
                .unwrap()
                .layout
                .node_visibility,
            [visibility]
        );
        for cut in 0..encoded.len() {
            assert!(decode_reply(&encoded[..cut]).is_err(), "cut={cut}");
        }
        let mut old = encoded;
        old[..4].copy_from_slice(b"ERWA");
        assert!(
            matches!(decode_reply(&old), Err(error) if error == "invalid IPC version/message kind")
        );
    }
}

#[test]
fn visibility_codec_refuses_count_and_byte_faults_before_later_payloads() {
    for (count, values, message) in [
        (
            0,
            &[][..],
            "IPC visibility count differs from DOM node count",
        ),
        (
            2,
            &[][..],
            "IPC visibility count differs from DOM node count",
        ),
        (MAX_NODES + 1, &[][..], "IPC collection limit exceeded"),
        (1, &[][..], "truncated IPC message"),
        (1, &[2][..], "unknown IPC visibility"),
        (1, &[255][..], "unknown IPC visibility"),
    ] {
        let bytes = prefix(count, values);
        let mut decoder = Decoder::new(&bytes, 99).unwrap();
        assert!(matches!(decode_snapshot(&mut decoder), Err(error) if error == message));
    }
}

#[test]
fn visibility_codec_sender_refuses_short_and_long_node_metadata() {
    for count in [0, 2] {
        let bytes = complete(0);
        let mut decoder = Decoder::new(&bytes, 99).unwrap();
        let mut snapshot = decode_snapshot(&mut decoder).unwrap();
        snapshot.layout.node_visibility = vec![Visibility::Visible; count];
        let reply = Reply {
            navigation: None,
            snapshot: Some(snapshot),
        };
        assert!(
            matches!(encode_reply(&reply), Err(error) if error == "IPC output budget exceeded")
        );
    }
}

#[test]
fn visibility_codec_rejects_hidden_hit_targets_in_both_directions() {
    let mut bytes = prefix(1, &[1]);
    bytes.extend_from_slice(&0u32.to_le_bytes()); // rasters
    bytes.extend_from_slice(&0u32.to_le_bytes()); // image aliases
    bytes.extend_from_slice(&0f32.to_le_bytes()); // content height
    bytes.extend_from_slice(&1u32.to_le_bytes()); // hits
    bytes.extend_from_slice(&0u32.to_le_bytes()); // hidden root; no geometry supplied
    let mut decoder = Decoder::new(&bytes, 99).unwrap();
    assert!(
        matches!(decode_snapshot(&mut decoder), Err(error) if error == "IPC hit targets hidden node")
    );

    let bytes = complete(1);
    let mut decoder = Decoder::new(&bytes, 99).unwrap();
    let mut snapshot = decode_snapshot(&mut decoder).unwrap();
    snapshot.layout.hit_regions.push(HitRegion {
        node: 0,
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        fixed: false,
        action: HitAction::Node,
    });
    let mut encoder = Encoder::new(99);
    encode_snapshot(&mut encoder, &snapshot);
    assert!(matches!(encoder.finish(), Err(error) if error == "IPC output budget exceeded"));
}
