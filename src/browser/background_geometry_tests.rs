use super::*;

#[test]
fn background_repeat_zoom_preserves_each_axis_and_shared_stops() {
    for repeat in [[true, true], [true, false], [false, true], [false, false]] {
        let mut original = super::gradient_tests::command();
        let DrawCommand::LinearGradient { repeat: flags, .. } = &mut original else {
            panic!()
        };
        *flags = repeat;
        let scaled = scaled_command(&original, 1.5);
        let DrawCommand::LinearGradient {
            rect,
            tile,
            radius,
            repeat: after,
            gradient,
            stop_scale,
            ..
        } = scaled
        else {
            panic!()
        };
        assert_eq!(after, repeat);
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (-1.5, 3.0, 30.0, 18.0)
        );
        assert_eq!(
            (tile.x, tile.y, tile.width, tile.height),
            (1.5, 4.5, 24.0, 12.0)
        );
        assert_eq!((radius, stop_scale), (3.0, 1.875));
        let DrawCommand::LinearGradient {
            gradient: before,
            repeat: original_flags,
            ..
        } = original
        else {
            panic!()
        };
        assert_eq!(original_flags, repeat);
        assert!(Arc::ptr_eq(&gradient, &before));
        assert!(Arc::ptr_eq(&gradient.stops, &before.stops));
    }
}
