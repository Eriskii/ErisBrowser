use super::*;

fn command(key: String) -> DrawCommand {
    let mut value = super::super::tiled_image_tests::command();
    let DrawCommand::BackgroundImage { key: old, .. } = &mut value else {
        panic!()
    };
    *old = key;
    value
}

#[test]
fn native_tiled_zoom_copies_literal_geometry_but_unit_zoom_borrows() {
    let input = [command("Tile-π".into())];
    for zoom in [1.0, 1.0005] {
        let Cow::Borrowed(borrowed) = scaled_page(&input, zoom).unwrap() else {
            panic!()
        };
        assert_eq!(borrowed.as_ptr(), input.as_ptr());
    }
    let Cow::Owned(output) = scaled_page(&input, 1.5).unwrap() else {
        panic!()
    };
    let DrawCommand::BackgroundImage {
        rect,
        tile,
        repeat,
        radius,
        key,
    } = &output[0]
    else {
        panic!()
    };
    assert_eq!(
        *rect,
        Rect {
            x: -2.25,
            y: 3.375,
            width: 12.0,
            height: 9.0
        }
    );
    assert_eq!(
        *tile,
        Rect {
            x: -4.5,
            y: 0.75,
            width: 6.0,
            height: 3.0
        }
    );
    assert_eq!(
        (*repeat, *radius, key.as_str()),
        ([true, false], 1.875, "Tile-π")
    );
    let DrawCommand::BackgroundImage { key: old, .. } = &input[0] else {
        panic!()
    };
    assert_ne!(old.as_ptr(), key.as_ptr());
}

#[test]
fn tiled_and_ordinary_zoom_keys_share_existing_exact_byte_admission() {
    let at_individual = command("é".repeat(2048));
    assert!(scaled_page(&[at_individual], 1.5).is_ok()); // 4096 UTF-8 bytes
    let over_individual = command(format!("{}x", "é".repeat(2048)));
    assert!(
        scaled_page(&[over_individual], 1.5)
            .unwrap_err()
            .contains("image key budget")
    );
    let mut mixed = Vec::new();
    for _ in 0..8 {
        mixed.push(command("b".repeat(4096)));
        mixed.push(DrawCommand::Image {
            rect: Rect::default(),
            key: "i".repeat(4096),
        });
    }
    assert!(scaled_page(&mixed, 1.5).is_ok()); // 65536 shared bytes
    mixed.push(command(String::new()));
    assert!(scaled_page(&mixed, 1.5).is_ok());
    mixed.push(command("x".into()));
    assert!(
        scaled_page(&mixed, 1.5)
            .unwrap_err()
            .contains("image key budget")
    );
    assert_eq!(mixed.len(), 18);
}
