use super::*;
use crate::graphics::Canvas;

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn gray() -> LinearGradient {
    LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: Arc::from([
            GradientStop {
                color: GradientColor::Color(Color::BLACK),
                position: None,
            },
            GradientStop {
                color: GradientColor::Color(Color::rgb(248, 248, 248)),
                position: None,
            },
        ]),
    }
}

#[test]
fn prepared_opaque_gradient_has_literal_channels_and_prepaid_work() {
    let gradient = gray();
    let bounds = rect(0.0, 0.0, 4.0, 2.0);
    let prepared = OpaqueGradient::new(4, 2, bounds, bounds, bounds, &gradient, Color::BLACK, 1.0)
        .unwrap()
        .unwrap();
    let info = prepared.info();
    assert_eq!((info.x, info.y, info.width, info.height), (0, 0, 4, 2));
    assert_eq!(info.loop_work, 32); // 8*3 sampling units + 4*2 stop units.
    assert_eq!(info.rgba_bytes(), 32);
    assert_eq!(
        prepared.materialize().unwrap(),
        [
            31, 31, 31, 255, 93, 93, 93, 255, 155, 155, 155, 255, 217, 217, 217, 255,
        ]
        .repeat(2)
    );
}

#[test]
fn prepared_opaque_gradient_preserves_fractional_clip_and_tile_phase() {
    let gradient = gray();
    let paint = rect(-0.25, -0.25, 6.0, 3.0);
    let tile = rect(0.0, 0.0, 4.0, 2.0);
    let clip = rect(1.25, 0.25, 3.25, 1.5);
    let prepared = OpaqueGradient::new(6, 4, paint, tile, clip, &gradient, Color::BLACK, 1.0)
        .unwrap()
        .unwrap();
    let info = prepared.info();
    assert_eq!((info.x, info.y, info.width, info.height), (2, 1, 3, 1));
    assert_eq!(info.loop_work, 17);
    let bytes = prepared.materialize().unwrap();
    assert_eq!(
        bytes,
        [155, 155, 155, 255, 217, 217, 217, 255, 31, 31, 31, 255]
    );
    let mut canvas = Canvas::new(6, 4).unwrap();
    canvas.clip = clip;
    canvas.linear_gradient(paint, tile, 0.0, &gradient, Color::BLACK, 1.0);
    assert_eq!(&canvas.pixels[8..11], &[0x9b9b9b, 0xd9d9d9, 0x1f1f1f]);
}

#[test]
fn prepared_opaque_gradient_rejects_hidden_transparency_and_malformed_geometry() {
    let bounds = rect(0.0, 0.0, 4.0, 2.0);
    let empty = rect(0.0, 0.0, 0.0, 0.0);
    let gradient = LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: Arc::from([GradientStop {
            color: GradientColor::CurrentColor,
            position: None,
        }]),
    };
    assert!(matches!(
        OpaqueGradient::new(
            4,
            2,
            bounds,
            bounds,
            empty,
            &gradient,
            Color::TRANSPARENT,
            1.0
        ),
        Err(OpaqueGradientError::Unsupported)
    ));
    assert!(
        OpaqueGradient::new(4, 2, bounds, bounds, empty, &gradient, Color::BLACK, 1.0)
            .unwrap()
            .is_none()
    );
    for tile in [rect(f32::NAN, 0.0, 4.0, 2.0), rect(0.0, 0.0, 0.0, 2.0)] {
        assert!(matches!(
            OpaqueGradient::new(4, 2, bounds, tile, bounds, &gradient, Color::BLACK, 1.0),
            Err(OpaqueGradientError::Geometry)
        ));
    }
    assert!(matches!(
        OpaqueGradient::new(4, 2, bounds, bounds, bounds, &gradient, Color::BLACK, 0.0),
        Err(OpaqueGradientError::Geometry)
    ));
}

#[test]
fn prepared_opaque_gradient_matches_canvas_across_angles_and_fractional_positions() {
    for angle in [0.0, 37.0, 90.0, 180.0, 270.0, 381.5] {
        for origin in [-7.25, -0.125, 0.0, 1.75] {
            let mut gradient = gray();
            gradient.direction = GradientDirection::Angle(angle);
            let paint = rect(origin, 0.25, 16.0, 8.0);
            let tile = rect(origin, -0.75, 3.25, 2.5);
            let clip = rect(0.25, 0.125, 12.5, 7.5);
            let prepared =
                OpaqueGradient::new(16, 10, paint, tile, clip, &gradient, Color::BLACK, 1.0)
                    .unwrap()
                    .unwrap();
            let info = prepared.info();
            let bytes = prepared.materialize().unwrap();
            let mut canvas = Canvas::new(16, 10).unwrap();
            canvas.clip = clip;
            canvas.linear_gradient(paint, tile, 0.0, &gradient, Color::BLACK, 1.0);
            for y in 0..info.height {
                for x in 0..info.width {
                    let at = ((y * info.width + x) * 4) as usize;
                    let pixel = u32::from(bytes[at]) << 16
                        | u32::from(bytes[at + 1]) << 8
                        | u32::from(bytes[at + 2]);
                    assert_eq!(
                        pixel,
                        canvas.pixels[((info.y + y) * 16 + info.x + x) as usize]
                    );
                }
            }
        }
    }
}
