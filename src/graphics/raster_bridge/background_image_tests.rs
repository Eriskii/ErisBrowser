use super::*;
use crate::graphics::{Canvas, Color, Fonts, Rect as BrowserRect};

fn rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}

fn background(repeat: [bool; 2], key: &str) -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        tile: rect(-0.75, 0.5, 2.0, 1.0),
        repeat,
        radius: 0.0,
        key: key.into(),
    }
}

fn malformed_images() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert(
        "present".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: Vec::new(),
        }),
    );
    images
}

// The two Probe routes still reject every BackgroundImage. Native now
// admits square corners, so preserve these whole-frame refusal witnesses with
// an explicit rounded clone; original Probe inputs and Canvas literals stay exact.
fn rounded_native(commands: &[DrawCommand]) -> Vec<DrawCommand> {
    commands
        .iter()
        .cloned()
        .map(|mut command| {
            if let DrawCommand::BackgroundImage { radius, .. } = &mut command {
                *radius = 1.0;
            }
            command
        })
        .collect()
}

fn assert_three_refuse(commands: &[DrawCommand], images: &ImageStore, index: usize) {
    let frame = Frame::new(4, 2, 0xffffff);
    let fonts = Fonts::new();
    let expected = FallbackReason {
        kind: FallbackKind::UnsupportedBackgroundImage,
        command_index: Some(index),
    };
    assert_eq!(
        plan_display_list(commands, images, frame).unwrap_err(),
        expected
    );
    assert_eq!(
        plan_display_list_with_fonts(commands, images, frame, &fonts).unwrap_err(),
        expected
    );
    assert_eq!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &rounded_native(commands)
            }],
            images,
            &fonts,
        )
        .unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::UnsupportedBackgroundImage,
            phase_index: Some(0),
            command_index: Some(index),
        }
    );
    assert_eq!(expected.category(), "unsupported-background-image");
    assert!(fonts.cache.borrow().is_empty());
}

#[test]
fn background_image_all_repeat_modes_and_missing_keys_refuse_before_source_validation() {
    for repeat in [[false, false], [true, false], [false, true], [true, true]] {
        for key in ["missing", "present"] {
            let commands = [
                DrawCommand::PushFixed,
                background(repeat, key),
                DrawCommand::PopFixed,
            ];
            let images = malformed_images();
            let references = Arc::strong_count(&images["present"]);
            assert_three_refuse(&commands, &images, 1);
            assert_eq!(Arc::strong_count(&images["present"]), references);
            assert!(images["present"].rgba.is_empty());
        }
    }
}

#[test]
fn background_image_hidden_and_degenerate_commands_are_not_elided_by_bridges() {
    for bounds in [rect(100.0, 100.0, 1.0, 1.0), rect(0.0, 0.0, 0.0, 0.0)] {
        let command = DrawCommand::BackgroundImage {
            rect: bounds,
            tile: rect(0.0, 0.0, 0.0, 0.0),
            repeat: [false, false],
            radius: 0.0,
            key: "missing".into(),
        };
        let commands = [
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 0.0, 0.0),
            },
            command,
            DrawCommand::PopClip,
        ];
        assert_three_refuse(&commands, &malformed_images(), 1);
    }
}

#[test]
fn background_image_native_refusal_reports_original_phase_inside_suppressed_scopes() {
    let frame = Frame::new(4, 2, 0xffffff);
    let first = [DrawCommand::Rect {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        color: Color::BLACK,
        radius: 0.0,
    }];
    let hidden = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 0.0, 0.0),
        },
        background([true, true], "missing"),
        DrawCommand::PopClip,
        DrawCommand::PopOpacity,
    ];
    let phases = [
        native::NativePhase {
            frame,
            commands: &first,
        },
        native::NativePhase {
            frame,
            commands: &rounded_native(&hidden),
        },
        native::NativePhase {
            frame,
            commands: &first,
        },
    ];
    let error =
        native::plan_native_scene(frame, &phases, &malformed_images(), &Fonts::new()).unwrap_err();
    assert_eq!(
        error,
        native::NativeFallback {
            kind: FallbackKind::UnsupportedBackgroundImage,
            phase_index: Some(1),
            command_index: Some(2),
        }
    );
    assert_eq!(
        error.to_string(),
        "unsupported-background-image at phase 1 original command 2"
    );
}

#[test]
fn background_image_fonts_and_native_refuse_before_preceding_text_materializes() {
    let frame = Frame::new(32, 24, 0xffffff);
    let commands = [
        DrawCommand::Text {
            x: 1.0,
            y: 1.0,
            text: "not rasterized".into(),
            size: 12.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        },
        background([true, false], "missing"),
    ];
    let fonts = Fonts::new();
    let images = malformed_images();
    assert_eq!(
        plan_display_list_with_fonts(&commands, &images, frame, &fonts).unwrap_err(),
        FallbackReason {
            kind: FallbackKind::UnsupportedBackgroundImage,
            command_index: Some(1)
        }
    );
    assert!(fonts.cache.borrow().is_empty());
    assert_eq!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &rounded_native(&commands)
            }],
            &images,
            &fonts
        )
        .unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::UnsupportedBackgroundImage,
            phase_index: Some(0),
            command_index: Some(1),
        }
    );
    assert!(fonts.cache.borrow().is_empty());
    // The primitive bridge's existing earlier unsupported-text diagnostic stays exact.
    assert_eq!(
        plan_display_list(&commands, &images, frame).unwrap_err(),
        FallbackReason {
            kind: FallbackKind::UnsupportedText,
            command_index: Some(0)
        }
    );
}

#[test]
fn background_image_refusal_preserves_earlier_invalid_and_cheap_store_guards() {
    let frame = Frame::new(4, 2, 0xffffff);
    let commands = [
        DrawCommand::Rect {
            rect: rect(f32::NAN, 0.0, 1.0, 1.0),
            color: Color::BLACK,
            radius: 0.0,
        },
        background([false, false], "missing"),
    ];
    let images = ImageStore::new();
    let expected = FallbackReason {
        kind: FallbackKind::InvalidGeometry,
        command_index: Some(0),
    };
    assert_eq!(
        plan_display_list(&commands, &images, frame).unwrap_err(),
        expected
    );
    assert_eq!(
        plan_display_list_with_fonts(&commands, &images, frame, &Fonts::new()).unwrap_err(),
        expected
    );
    assert_eq!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &commands
            }],
            &images,
            &Fonts::new()
        )
        .unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::InvalidGeometry,
            phase_index: Some(0),
            command_index: Some(0)
        }
    );
    let commands = [background([true, true], "missing")];
    let mut sparse = ImageStore::new();
    sparse.reserve(MAX_STORE_CAPACITY + 1);
    assert!(sparse.capacity() > MAX_STORE_CAPACITY);
    let expected = FallbackReason {
        kind: FallbackKind::ImageStoreLimit,
        command_index: None,
    };
    assert_eq!(
        plan_display_list(&commands, &sparse, frame).unwrap_err(),
        expected
    );
    assert_eq!(
        plan_display_list_with_fonts(&commands, &sparse, frame, &Fonts::new()).unwrap_err(),
        expected
    );
    // Native still checks command structure first. This valid square background
    // now reaches the unchanged image-table capacity guard.
    assert_eq!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &commands
            }],
            &sparse,
            &Fonts::new()
        )
        .unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::ImageStoreLimit,
            phase_index: None,
            command_index: None
        }
    );
}

#[test]
fn background_image_whole_list_cpu_repaint_keeps_before_and_after_commands() {
    let mut images = ImageStore::new();
    images.insert(
        "pixels".into(),
        Arc::new(RasterImage {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
        }),
    );
    let commands = [
        DrawCommand::Rect {
            rect: rect(0.0, 0.0, 4.0, 2.0),
            color: Color::rgb(0, 255, 0),
            radius: 0.0,
        },
        DrawCommand::BackgroundImage {
            rect: rect(0.0, 0.0, 4.0, 1.0),
            tile: rect(0.5, 0.0, 2.0, 1.0),
            repeat: [false, false],
            radius: 0.0,
            key: "pixels".into(),
        },
        DrawCommand::Rect {
            rect: rect(3.0, 1.0, 1.0, 1.0),
            color: Color::BLACK,
            radius: 0.0,
        },
    ];
    assert_three_refuse(&commands, &images, 1);
    let mut canvas = Canvas::new(4, 2).unwrap();
    canvas.paint(&commands, &Fonts::new(), &images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    // Direct whole-list software witness, not a claim that a window was acquired.
    assert_eq!(
        canvas.pixels,
        [
            0xff0000, 0x0000ff, 0x00ff00, 0x00ff00, 0x00ff00, 0x00ff00, 0x00ff00, 0x000000
        ]
    );
}

#[test]
fn background_image_addition_does_not_refuse_old_image_and_rectangle_plans() {
    let frame = Frame::new(4, 2, 0xffffff);
    let mut images = ImageStore::new();
    images.insert(
        "old".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        }),
    );
    let commands = [
        DrawCommand::Rect {
            rect: rect(0.0, 0.0, 4.0, 2.0),
            color: Color::WHITE,
            radius: 0.0,
        },
        DrawCommand::Image {
            rect: rect(1.0, 0.0, 1.0, 1.0),
            key: "old".into(),
        },
    ];
    let ordinary = plan_display_list(&commands, &images, frame).unwrap();
    let fonts = Fonts::new();
    let text = plan_display_list_with_fonts(&commands, &images, frame, &fonts).unwrap();
    let native = native::plan_native_scene(
        frame,
        &[native::NativePhase {
            frame,
            commands: &commands,
        }],
        &images,
        &fonts,
    )
    .unwrap();
    for stats in [ordinary.stats(), text.stats(), &native.stats().bridge] {
        assert_eq!(
            (
                stats.original_commands,
                stats.image_store_entries,
                stats.unique_sources,
                stats.referenced_sources,
                stats.total_rgba_bytes,
                stats.referenced_rgba_bytes,
                stats.missing_images
            ),
            (2, 1, 1, 1, 4, 4, 0)
        );
    }
}
