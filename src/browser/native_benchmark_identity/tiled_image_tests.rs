use super::*;

fn command() -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: Rect {
            x: -0.0,
            y: 1.0,
            width: 2.0,
            height: 3.0,
        },
        tile: Rect {
            x: -2.0,
            y: 0.5,
            width: 4.0,
            height: 2.0,
        },
        repeat: [true, false],
        radius: 1.25,
        key: "é".into(),
    }
}

fn encoded(command: &DrawCommand) -> Result<Vec<u8>, String> {
    let mut bytes = Bytes::new()?;
    commands(&mut bytes, std::slice::from_ref(command))?;
    Ok(bytes.0)
}

#[test]
fn tiled_identity_has_literal_tag_twelve_and_all_geometry_key_bytes() {
    let expected = [
        1, 0, 0, 0, 12, 1, 0, 0, 0, 128, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 192, 0,
        0, 0, 63, 0, 0, 128, 64, 0, 0, 0, 64, 0, 0, 160, 63, 2, 0, 0, 0, 195, 169,
    ];
    assert_eq!(encoded(&command()).unwrap(), expected);
    for (repeat, mask) in [
        ([false, false], 0),
        ([true, false], 1),
        ([false, true], 2),
        ([true, true], 3),
    ] {
        let mut value = command();
        let DrawCommand::BackgroundImage { repeat: flags, .. } = &mut value else {
            panic!()
        };
        *flags = repeat;
        let mut oracle = expected;
        oracle[5] = mask;
        assert_eq!(encoded(&value).unwrap(), oracle);
    }
    for field in 0..10 {
        let mut value = command();
        let DrawCommand::BackgroundImage {
            rect,
            tile,
            radius,
            key,
            ..
        } = &mut value
        else {
            panic!()
        };
        match field {
            0 => rect.x = 0.0, // signed zero remains part of exact identity
            1 => rect.y = 2.0,
            2 => rect.width = 3.0,
            3 => rect.height = 4.0,
            4 => tile.x = -1.0,
            5 => tile.y = 1.5,
            6 => tile.width = 5.0,
            7 => tile.height = 3.0,
            8 => *radius = 2.25,
            9 => key.push('x'),
            _ => unreachable!(),
        }
        assert_ne!(encoded(&value).unwrap(), expected, "field {field}");
    }
}

#[test]
fn tiled_identity_refuses_invalid_geometry_and_preserves_existing_output_cap() {
    for field in 0..7 {
        let mut value = command();
        let DrawCommand::BackgroundImage {
            rect, tile, radius, ..
        } = &mut value
        else {
            panic!()
        };
        match field {
            0 => rect.x = f32::NAN,
            1 => rect.width = -1.0,
            2 => tile.x = f32::INFINITY,
            3 => tile.width = 0.0,
            4 => tile.height = -1.0,
            5 => *radius = -1.0,
            6 => *radius = f32::NAN,
            _ => unreachable!(),
        }
        assert!(encoded(&value).is_err(), "field {field}");
    }
    let mut value = command();
    let DrawCommand::BackgroundImage { key, .. } = &mut value else {
        panic!()
    };
    *key = "k".repeat(65490); // fixed prefix 46 + key = existing 65536-byte cap
    assert_eq!(encoded(&value).unwrap().len(), 65536);
    let DrawCommand::BackgroundImage { key, .. } = &mut value else {
        panic!()
    };
    key.push('k');
    assert!(encoded(&value).unwrap_err().contains("64 KiB"));
}
