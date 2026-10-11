use super::*;
use crate::graphics::{
    Color, Fonts, Rect as BrowserRect,
    gradient::{GradientColor, GradientDirection, GradientStop, LinearGradient},
};

fn gradient() -> DrawCommand {
    DrawCommand::LinearGradient {
        repeat: [true, true],
        rect: BrowserRect {
            x: 0.0,
            y: 0.0,
            width: 24.0,
            height: 16.0,
        },
        tile: BrowserRect {
            x: 1.0,
            y: 2.0,
            width: 20.0,
            height: 12.0,
        },
        radius: 3.0,
        gradient: Arc::new(LinearGradient {
            direction: GradientDirection::Angle(90.0),
            stops: Arc::from([GradientStop {
                color: GradientColor::Color(Color::BLACK),
                position: None,
            }]),
        }),
        current_color: Color::WHITE,
        stop_scale: 1.0,
    }
}
fn images() -> ImageStore {
    // Validation must never progress as far as source-sized preparation.
    let mut store = ImageStore::new();
    store.insert(
        "malformed-but-unvisited".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: Vec::new(),
        }),
    );
    store
}

#[test]
fn gradient_plain_and_font_bridges_refuse_original_command_before_source_preparation() {
    let commands = [DrawCommand::PushFixed, gradient(), DrawCommand::PopFixed];
    let frame = Frame::new(32, 24, 0xffffff);
    let images = images();
    let expected = FallbackReason {
        kind: FallbackKind::UnsupportedGradient,
        command_index: Some(1),
    };
    assert_eq!(
        plan_display_list(&commands, &images, frame).unwrap_err(),
        expected
    );
    assert_eq!(
        plan_display_list_with_fonts(&commands, &images, frame, &Fonts::new()).unwrap_err(),
        expected
    );
    assert_eq!(expected.category(), "unsupported-gradient");
    assert!(images["malformed-but-unvisited"].rgba.is_empty());
}

#[test]
fn gradient_native_refusal_keeps_phase_and_index_even_when_clipped_and_suppressed() {
    let target = Frame::new(32, 24, 0xffffff);
    let first = [DrawCommand::Rect {
        rect: BrowserRect {
            x: 0.0,
            y: 0.0,
            width: 3.0,
            height: 2.0,
        },
        color: Color::WHITE,
        radius: 0.0,
    }];
    let hidden = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PushClip {
            rect: BrowserRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        },
        gradient(),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
    ];
    let phases = [
        native::NativePhase {
            frame: target,
            commands: &first,
        },
        native::NativePhase {
            frame: target,
            commands: &hidden,
        },
    ];
    let error = native::plan_native_scene(target, &phases, &images(), &Fonts::new()).unwrap_err();
    assert_eq!(
        error,
        native::NativeFallback {
            kind: FallbackKind::UnsupportedGradient,
            phase_index: Some(1),
            command_index: Some(2)
        }
    );
    assert_eq!(
        error.to_string(),
        "unsupported-gradient at phase 1 original command 2"
    );
}

#[test]
fn gradient_refusal_does_not_reclassify_preceding_invalid_input_or_old_frames() {
    let frame = Frame::new(32, 24, 0xffffff);
    let malformed = DrawCommand::Rect {
        rect: BrowserRect {
            x: f32::NAN,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        color: Color::WHITE,
        radius: 0.0,
    };
    let commands = [malformed, gradient()];
    assert_eq!(
        plan_display_list(&commands, &ImageStore::new(), frame)
            .unwrap_err()
            .kind,
        FallbackKind::InvalidGeometry
    );
    let phases = [native::NativePhase {
        frame,
        commands: &commands,
    }];
    let error =
        native::plan_native_scene(frame, &phases, &ImageStore::new(), &Fonts::new()).unwrap_err();
    assert_eq!(
        (error.kind, error.command_index),
        (FallbackKind::InvalidGeometry, Some(0))
    );
    let ordinary = [DrawCommand::Rect {
        rect: BrowserRect {
            x: 1.0,
            y: 1.0,
            width: 2.0,
            height: 2.0,
        },
        color: Color::BLACK,
        radius: 0.0,
    }];
    assert!(plan_display_list(&ordinary, &ImageStore::new(), frame).is_ok());
    assert!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &ordinary
            }],
            &ImageStore::new(),
            &Fonts::new()
        )
        .is_ok()
    );
}
