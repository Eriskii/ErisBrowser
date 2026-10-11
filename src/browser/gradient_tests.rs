use super::*;
use eris::graphics::gradient::{
    GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
};

pub(super) fn command() -> DrawCommand {
    DrawCommand::LinearGradient {
        rect: Rect {
            x: -1.0,
            y: 2.0,
            width: 20.0,
            height: 12.0,
        },
        tile: Rect {
            x: 1.0,
            y: 3.0,
            width: 16.0,
            height: 8.0,
        },
        radius: 2.0,
        gradient: Arc::new(LinearGradient {
            direction: GradientDirection::Corner {
                right: true,
                bottom: false,
            },
            stops: Arc::from([
                GradientStop {
                    color: GradientColor::CurrentColor,
                    position: Some(GradientPosition {
                        px: 2.5,
                        percent: 25.0,
                    }),
                },
                GradientStop {
                    color: GradientColor::Color(Color::BLACK),
                    position: None,
                },
            ]),
        }),
        current_color: Color::rgba(12, 34, 56, 78),
        stop_scale: 1.25,
    }
}

#[test]
fn gradient_zoom_scales_both_boxes_radius_and_px_multiplier_once() {
    let original = command();
    let scaled = scaled_command(&original, 2.0);
    let DrawCommand::LinearGradient {
        rect,
        tile,
        radius,
        gradient,
        current_color,
        stop_scale,
    } = scaled
    else {
        panic!()
    };
    assert_eq!(
        rect,
        Rect {
            x: -2.0,
            y: 4.0,
            width: 40.0,
            height: 24.0
        }
    );
    assert_eq!(
        tile,
        Rect {
            x: 2.0,
            y: 6.0,
            width: 32.0,
            height: 16.0
        }
    );
    assert_eq!((radius, stop_scale), (4.0, 2.5));
    assert_eq!(current_color, Color::rgba(12, 34, 56, 78));
    assert_eq!(
        gradient.stops[0].position,
        Some(GradientPosition {
            px: 2.5,
            percent: 25.0
        })
    );
    let DrawCommand::LinearGradient {
        gradient: before,
        stop_scale,
        ..
    } = original
    else {
        panic!()
    };
    assert!(Arc::ptr_eq(&gradient, &before));
    assert!(Arc::ptr_eq(&gradient.stops, &before.stops));
    assert_eq!(stop_scale, 1.25);
}
