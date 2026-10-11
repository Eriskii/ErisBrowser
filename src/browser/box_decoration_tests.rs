use super::*;
use eris::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};

pub(super) fn command() -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: Rect {
                x: -1.5,
                y: 2.25,
                width: 8.0,
                height: 6.0,
            },
            radii: [[1.0, 2.0], [3.0, 1.5], [2.0, 1.0], [0.5, 2.5]],
        },
        cutout: Some(RoundedBox {
            rect: Rect {
                x: -0.5,
                y: 3.25,
                width: 5.0,
                height: 3.0,
            },
            radii: [[0.0, 1.0], [1.0, 0.5], [0.0, 0.0], [0.0, 1.5]],
        }),
        paint: BoxPaint::Image {
            tile: Rect {
                x: -3.0,
                y: 0.5,
                width: 4.0,
                height: 2.0,
            },
            repeat: [true, false],
            key: "Tile-π".into(),
        },
    }))
}

#[test]
fn box_decoration_zoom_scales_every_corner_cutout_and_tile_once() {
    let original = command();
    let BoxDecoration {
        shape,
        cutout,
        paint,
    } = unpack(scaled_command(&original, 2.0));
    assert_eq!(
        shape.rect,
        Rect {
            x: -3.0,
            y: 4.5,
            width: 16.0,
            height: 12.0
        }
    );
    assert_eq!(
        shape.radii,
        [[2.0, 4.0], [6.0, 3.0], [4.0, 2.0], [1.0, 5.0]]
    );
    let cutout = cutout.unwrap();
    assert_eq!(
        cutout.rect,
        Rect {
            x: -1.0,
            y: 6.5,
            width: 10.0,
            height: 6.0
        }
    );
    assert_eq!(
        cutout.radii,
        [[0.0, 2.0], [2.0, 1.0], [0.0, 0.0], [0.0, 3.0]]
    );
    let BoxPaint::Image { tile, repeat, key } = paint else {
        panic!()
    };
    assert_eq!(
        tile,
        Rect {
            x: -6.0,
            y: 1.0,
            width: 8.0,
            height: 4.0
        }
    );
    assert_eq!((repeat, key.as_str()), ([true, false], "Tile-π"));
    let BoxDecoration {
        shape,
        paint: BoxPaint::Image { key: old, .. },
        ..
    } = unpack(original)
    else {
        panic!()
    };
    assert_eq!(shape.rect.x, -1.5);
    assert_ne!(old.as_ptr(), key.as_ptr());
}

#[test]
fn box_decoration_gradient_zoom_retains_stops_and_scales_stop_lengths() {
    use eris::graphics::gradient::{GradientDirection, LinearGradient};
    let gradient = Arc::new(LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: vec![].into(),
    });
    let mut input = command();
    let BoxDecoration { paint, .. } = decoration_mut(&mut input);
    *paint = BoxPaint::LinearGradient {
        tile: Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        },
        repeat: [false, true],
        gradient: Arc::clone(&gradient),
        current_color: Color::BLACK,
        stop_scale: 1.25,
    };
    let BoxDecoration {
        paint:
            BoxPaint::LinearGradient {
                tile,
                repeat,
                gradient: after,
                current_color,
                stop_scale,
            },
        ..
    } = unpack(scaled_command(&input, 2.0))
    else {
        panic!()
    };
    assert_eq!(
        (tile.x, tile.y, tile.width, tile.height),
        (2.0, 4.0, 6.0, 8.0)
    );
    assert_eq!(
        (repeat, current_color, stop_scale),
        ([false, true], Color::BLACK, 2.5)
    );
    assert!(Arc::ptr_eq(&gradient, &after));
}

fn unpack(command: DrawCommand) -> BoxDecoration {
    let DrawCommand::BoxDecoration(payload) = command else {
        panic!()
    };
    Arc::unwrap_or_clone(payload)
}

fn decoration_mut(command: &mut DrawCommand) -> &mut BoxDecoration {
    let DrawCommand::BoxDecoration(payload) = command else {
        panic!()
    };
    Arc::make_mut(payload)
}
