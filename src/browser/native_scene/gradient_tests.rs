use super::*;

#[test]
fn gradient_native_zoom_copy_shares_payload_and_matches_literal_geometry() {
    let original = super::super::gradient_tests::command();
    let commands = [original];
    let scaled = scaled_page(&commands, 2.0).unwrap();
    assert!(matches!(scaled, Cow::Owned(_)));
    let DrawCommand::LinearGradient {
        rect,
        tile,
        radius,
        gradient,
        current_color,
        stop_scale,
    } = &scaled[0]
    else {
        panic!()
    };
    assert_eq!(
        *rect,
        Rect {
            x: -2.0,
            y: 4.0,
            width: 40.0,
            height: 24.0
        }
    );
    assert_eq!(
        *tile,
        Rect {
            x: 2.0,
            y: 6.0,
            width: 32.0,
            height: 16.0
        }
    );
    assert_eq!((*radius, *stop_scale), (4.0, 2.5));
    assert_eq!(*current_color, Color::rgba(12, 34, 56, 78));
    let DrawCommand::LinearGradient {
        gradient: before, ..
    } = &commands[0]
    else {
        panic!()
    };
    assert!(Arc::ptr_eq(gradient, before));
    let unscaled = scaled_page(&commands, 1.0005).unwrap();
    assert!(matches!(unscaled, Cow::Borrowed(_)));
}
