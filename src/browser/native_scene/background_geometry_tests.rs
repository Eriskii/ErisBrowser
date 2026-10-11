use super::*;

#[test]
fn native_background_zoom_preserves_repeat_axes_without_payload_copy() {
    for repeat in [[true, true], [true, false], [false, true], [false, false]] {
        let mut original = super::super::gradient_tests::command();
        let DrawCommand::LinearGradient { repeat: flags, .. } = &mut original else {
            panic!()
        };
        *flags = repeat;
        let commands = [original];
        let scaled = scaled_page(&commands, 1.5).unwrap();
        let DrawCommand::LinearGradient {
            rect,
            tile,
            repeat: after,
            gradient,
            stop_scale,
            ..
        } = &scaled[0]
        else {
            panic!()
        };
        assert_eq!(*after, repeat);
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (-1.5, 3.0, 30.0, 18.0)
        );
        assert_eq!(
            (tile.x, tile.y, tile.width, tile.height),
            (1.5, 4.5, 24.0, 12.0)
        );
        assert_eq!(*stop_scale, 1.875);
        let DrawCommand::LinearGradient {
            gradient: before, ..
        } = &commands[0]
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(gradient, before));
        assert!(matches!(
            scaled_page(&commands, 1.0).unwrap(),
            Cow::Borrowed(_)
        ));
    }
}
