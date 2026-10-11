use super::*;

pub(super) fn command() -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: Rect {
            x: -1.5,
            y: 2.25,
            width: 8.0,
            height: 6.0,
        },
        tile: Rect {
            x: -3.0,
            y: 0.5,
            width: 4.0,
            height: 2.0,
        },
        repeat: [true, false],
        radius: 1.25,
        key: "Tile-π".into(),
    }
}

#[test]
fn tiled_image_zoom_scales_both_boxes_once_and_preserves_resource_and_axes() {
    for repeat in [[true, true], [true, false], [false, true], [false, false]] {
        let mut original = command();
        let DrawCommand::BackgroundImage { repeat: flags, .. } = &mut original else {
            panic!()
        };
        *flags = repeat;
        let scaled = scaled_command(&original, 2.0);
        let DrawCommand::BackgroundImage {
            rect,
            tile,
            repeat: after,
            radius,
            key,
        } = scaled
        else {
            panic!()
        };
        assert_eq!(
            rect,
            Rect {
                x: -3.0,
                y: 4.5,
                width: 16.0,
                height: 12.0
            }
        );
        assert_eq!(
            tile,
            Rect {
                x: -6.0,
                y: 1.0,
                width: 8.0,
                height: 4.0
            }
        );
        assert_eq!(radius, 2.5);
        assert_eq!(after, repeat);
        assert_eq!(key, "Tile-π");
        let DrawCommand::BackgroundImage {
            rect,
            tile,
            repeat: before,
            radius,
            key: old_key,
        } = original
        else {
            panic!()
        };
        assert_eq!(
            (rect.x, rect.y, tile.x, tile.y, radius),
            (-1.5, 2.25, -3.0, 0.5, 1.25)
        );
        assert_eq!(before, repeat);
        assert_eq!(old_key, key);
        assert_ne!(old_key.as_ptr(), key.as_ptr());
    }
}
