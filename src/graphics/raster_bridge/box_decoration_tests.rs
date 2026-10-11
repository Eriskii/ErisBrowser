//! Independent refusal oracles held before the adapter implementation.
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use crate::graphics::gradient::{GradientDirection, LinearGradient};
use crate::graphics::{Color, Fonts, Rect as BrowserRect};

fn rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}
fn shape() -> RoundedBox {
    RoundedBox {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        radii: [[1.0, 0.5]; 4],
    }
}
fn color() -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: shape(),
        cutout: None,
        paint: BoxPaint::Color(Color::BLACK),
    }))
}
fn image(key: &str) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: shape(),
        cutout: None,
        paint: BoxPaint::Image {
            tile: rect(-0.75, 0.5, 2.0, 1.0),
            repeat: [true, false],
            key: key.into(),
        },
    }))
}
fn poison() -> ImageStore {
    let mut store = ImageStore::new();
    store.insert(
        "bad".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: Vec::new(),
        }),
    );
    store
}
fn all_routes(commands: &[DrawCommand], images: &ImageStore, kind: FallbackKind, index: usize) {
    let frame = Frame::new(4, 2, 0xffffff);
    let fonts = Fonts::new();
    let expected = FallbackReason {
        kind,
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
            &[native::NativePhase { frame, commands }],
            images,
            &fonts
        )
        .unwrap_err(),
        native::NativeFallback {
            kind,
            phase_index: Some(0),
            command_index: Some(index)
        }
    );
    assert!(fonts.cache.borrow().is_empty());
}

#[test]
fn box_decoration_all_paints_refuse_without_materializing_payloads() {
    let invalid_gradient = Arc::new(LinearGradient {
        direction: GradientDirection::Angle(f64::NAN),
        stops: Arc::from([]),
    });
    let commands = [
        color(),
        image("bad"),
        image("missing"),
        DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape: shape(),
            cutout: None,
            paint: BoxPaint::LinearGradient {
                tile: rect(f32::NAN, 0.0, 0.0, 0.0),
                repeat: [false, true],
                gradient: invalid_gradient.clone(),
                current_color: Color::WHITE,
                stop_scale: f32::NAN,
            },
        })),
    ];
    let images = poison();
    let image_refs = Arc::strong_count(&images["bad"]);
    let gradient_refs = Arc::strong_count(&invalid_gradient);
    for command in &commands {
        all_routes(
            std::slice::from_ref(command),
            &images,
            FallbackKind::UnsupportedBoxDecoration,
            0,
        );
    }
    assert_eq!(Arc::strong_count(&images["bad"]), image_refs);
    assert_eq!(Arc::strong_count(&invalid_gradient), gradient_refs);
    assert_eq!(
        FallbackReason {
            kind: FallbackKind::UnsupportedBoxDecoration,
            command_index: Some(0)
        }
        .to_string(),
        "unsupported-box-decoration at original command 0"
    );
}

#[test]
fn box_decoration_hidden_empty_offscreen_and_fixed_commands_still_refuse() {
    for bounds in [
        rect(0.0, 0.0, 0.0, 0.0),
        rect(100.0, 100.0, 4.0, 2.0),
        rect(0.0, 0.0, 4.0, 2.0),
    ] {
        let commands = [
            DrawCommand::PushFixed,
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 0.0, 0.0),
            },
            DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: RoundedBox {
                    rect: bounds,
                    radii: [[0.0; 2]; 4],
                },
                cutout: None,
                paint: BoxPaint::Color(Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                }),
            })),
            DrawCommand::PopClip,
            DrawCommand::PopFixed,
        ];
        all_routes(
            &commands,
            &poison(),
            FallbackKind::UnsupportedBoxDecoration,
            2,
        );
    }
}

#[test]
fn box_decoration_cutout_covering_outer_shape_does_not_elide_refusal() {
    for cutout in [
        None,
        Some(shape()),
        Some(RoundedBox {
            rect: rect(-2.0, -2.0, 8.0, 6.0),
            radii: [[0.0; 2]; 4],
        }),
    ] {
        let command = DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape: shape(),
            cutout,
            paint: BoxPaint::Color(Color::BLACK),
        }));
        all_routes(
            &[command],
            &poison(),
            FallbackKind::UnsupportedBoxDecoration,
            0,
        );
    }
}

#[test]
fn box_decoration_invalid_outer_geometry_has_original_invalid_geometry_location() {
    for field in 0..12 {
        for value in [f32::NAN, f32::INFINITY, 1_000_001.0] {
            let mut outer = shape();
            match field {
                0 => outer.rect.x = value,
                1 => outer.rect.y = value,
                2 => outer.rect.width = value,
                3 => outer.rect.height = value,
                _ => outer.radii[(field - 4) / 2][(field - 4) % 2] = value,
            }
            let commands = [
                DrawCommand::PushFixed,
                DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                    shape: outer,
                    cutout: None,
                    paint: BoxPaint::Color(Color::BLACK),
                })),
                DrawCommand::PopFixed,
            ];
            all_routes(&commands, &poison(), FallbackKind::InvalidGeometry, 1);
        }
    }
    for field in 2..12 {
        let mut outer = shape();
        match field {
            2 => outer.rect.width = -1.0,
            3 => outer.rect.height = -1.0,
            _ => outer.radii[(field - 4) / 2][(field - 4) % 2] = -1.0,
        }
        all_routes(
            &[DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: outer,
                cutout: None,
                paint: BoxPaint::Color(Color::BLACK),
            }))],
            &poison(),
            FallbackKind::InvalidGeometry,
            0,
        );
    }
}

#[test]
fn box_decoration_invalid_cutout_is_checked_even_under_empty_clip() {
    for field in 0..12 {
        let mut cutout = shape();
        match field {
            0 => cutout.rect.x = f32::NAN,
            1 => cutout.rect.y = f32::NEG_INFINITY,
            2 => cutout.rect.width = -1.0,
            3 => cutout.rect.height = 1_000_001.0,
            _ => cutout.radii[(field - 4) / 2][(field - 4) % 2] = -1.0,
        }
        let commands = [
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 0.0, 0.0),
            },
            DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: shape(),
                cutout: Some(cutout),
                paint: BoxPaint::Color(Color::BLACK),
            })),
            DrawCommand::PopClip,
        ];
        all_routes(&commands, &poison(), FallbackKind::InvalidGeometry, 1);
    }
}

#[test]
fn box_decoration_adapter_coordinate_bound_is_inclusive_and_tighter_than_cpu() {
    for sign in [-1.0, 1.0] {
        let edge = RoundedBox {
            rect: rect(
                sign * 1_000_000.0,
                sign * 1_000_000.0,
                1_000_000.0,
                1_000_000.0,
            ),
            radii: [[1_000_000.0; 2]; 4],
        };
        assert!(edge.valid());
        all_routes(
            &[DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: edge,
                cutout: None,
                paint: BoxPaint::Color(Color::BLACK),
            }))],
            &poison(),
            FallbackKind::UnsupportedBoxDecoration,
            0,
        );
        let beyond = RoundedBox {
            rect: rect(sign * 1_000_001.0, 0.0, 1.0, 1.0),
            radii: [[0.0; 2]; 4],
        };
        assert!(beyond.valid());
        all_routes(
            &[DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: beyond,
                cutout: None,
                paint: BoxPaint::Color(Color::BLACK),
            }))],
            &poison(),
            FallbackKind::InvalidGeometry,
            0,
        );
    }
}

#[test]
fn box_decoration_preceding_text_and_rounded_masks_remain_cold() {
    let frame = Frame::new(32, 24, 0xffffff);
    let commands = [
        DrawCommand::Text {
            x: 1.0,
            y: 1.0,
            text: "cold font".into(),
            size: 12.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        },
        DrawCommand::Rect {
            rect: rect(0.0, 0.0, 4.0, 4.0),
            radius: 1.0,
            color: Color::BLACK,
        },
        color(),
    ];
    let fonts = Fonts::new();
    let images = poison();
    // Probe with fonts still preserves its earlier unsupported rounded rectangle.
    assert_eq!(
        plan_display_list_with_fonts(&commands, &images, frame, &fonts).unwrap_err(),
        FallbackReason {
            kind: FallbackKind::UnsupportedRoundedRect,
            command_index: Some(1)
        }
    );
    let text_then_box = [commands[0].clone(), color()];
    assert_eq!(
        plan_display_list_with_fonts(&text_then_box, &images, frame, &fonts).unwrap_err(),
        FallbackReason {
            kind: FallbackKind::UnsupportedBoxDecoration,
            command_index: Some(1)
        }
    );
    assert_eq!(
        plan_display_list(&text_then_box, &images, frame).unwrap_err(),
        FallbackReason {
            kind: FallbackKind::UnsupportedText,
            command_index: Some(0)
        }
    );
    assert_eq!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &commands
            }],
            &images,
            &fonts
        )
        .unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::UnsupportedBoxDecoration,
            phase_index: Some(0),
            command_index: Some(2)
        }
    );
    assert!(fonts.cache.borrow().is_empty());
}

#[test]
fn box_decoration_native_reports_phase_and_index_under_zero_opacity() {
    let frame = Frame::new(4, 2, 0xffffff);
    let before = [DrawCommand::Rect {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        color: Color::WHITE,
        radius: 0.0,
    }];
    let hidden = [
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PushFixed,
        image("missing"),
        DrawCommand::PopFixed,
        DrawCommand::PopOpacity,
    ];
    let phases = [
        native::NativePhase {
            frame,
            commands: &before,
        },
        native::NativePhase {
            frame,
            commands: &hidden,
        },
        native::NativePhase {
            frame,
            commands: &before,
        },
    ];
    let error = native::plan_native_scene(frame, &phases, &poison(), &Fonts::new()).unwrap_err();
    assert_eq!(
        error,
        native::NativeFallback {
            kind: FallbackKind::UnsupportedBoxDecoration,
            phase_index: Some(1),
            command_index: Some(2)
        }
    );
    assert_eq!(
        error.to_string(),
        "unsupported-box-decoration at phase 1 original command 2"
    );
}

#[test]
fn box_decoration_command_counts_preserve_full_input_limit_precedence() {
    let frame = Frame::new(4, 2, 0xffffff);
    let plain = DrawCommand::Rect {
        rect: rect(0.0, 0.0, 0.0, 0.0),
        color: Color::BLACK,
        radius: 0.0,
    };
    let mut commands = vec![plain.clone(); 256];
    commands[255] = color();
    all_routes(
        &commands,
        &poison(),
        FallbackKind::UnsupportedBoxDecoration,
        255,
    );
    commands.push(plain);
    let images = poison();
    let fonts = Fonts::new();
    let expected = FallbackReason {
        kind: FallbackKind::CommandLimit,
        command_index: None,
    };
    assert_eq!(
        plan_display_list(&commands, &images, frame).unwrap_err(),
        expected
    );
    assert_eq!(
        plan_display_list_with_fonts(&commands, &images, frame, &fonts).unwrap_err(),
        expected
    );
    let phases = [
        native::NativePhase {
            frame,
            commands: &commands[..1],
        },
        native::NativePhase {
            frame,
            commands: &commands[1..],
        },
    ];
    assert_eq!(
        native::plan_native_scene(frame, &phases, &images, &fonts).unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::CommandLimit,
            phase_index: Some(1),
            command_index: None
        }
    );
    assert!(fonts.cache.borrow().is_empty());
}

#[test]
fn box_decoration_preserves_earlier_geometry_scope_and_cheap_store_guards() {
    all_routes(
        &[
            DrawCommand::Rect {
                rect: rect(f32::NAN, 0.0, 1.0, 1.0),
                color: Color::BLACK,
                radius: 0.0,
            },
            color(),
        ],
        &poison(),
        FallbackKind::InvalidGeometry,
        0,
    );
    all_routes(
        &[DrawCommand::PopFixed, color()],
        &poison(),
        FallbackKind::InvalidScope,
        0,
    );
    let frame = Frame::new(4, 2, 0xffffff);
    let commands = [color()];
    let mut images = ImageStore::new();
    images.reserve(MAX_STORE_CAPACITY + 1);
    assert!(images.capacity() > MAX_STORE_CAPACITY);
    let fonts = Fonts::new();
    let expected = FallbackReason {
        kind: FallbackKind::ImageStoreLimit,
        command_index: None,
    };
    assert_eq!(
        plan_display_list(&commands, &images, frame).unwrap_err(),
        expected
    );
    assert_eq!(
        plan_display_list_with_fonts(&commands, &images, frame, &fonts).unwrap_err(),
        expected
    );
    // Native's existing structural pass precedes the cheap store guard.
    assert_eq!(
        native::plan_native_scene(
            frame,
            &[native::NativePhase {
                frame,
                commands: &commands
            }],
            &images,
            &fonts
        )
        .unwrap_err(),
        native::NativeFallback {
            kind: FallbackKind::UnsupportedBoxDecoration,
            phase_index: Some(0),
            command_index: Some(0)
        }
    );
}

#[test]
fn box_decoration_keys_and_alias_payloads_are_not_prepared_on_refusal() {
    let mut images = poison();
    let alias = images["bad"].clone();
    images.insert("alias".into(), alias.clone());
    let key = "x".repeat(4097);
    let commands = [
        DrawCommand::Image {
            rect: rect(0.0, 0.0, 1.0, 1.0),
            key: "alias".into(),
        },
        image(&key),
    ];
    let refs = Arc::strong_count(&alias);
    all_routes(
        &commands,
        &images,
        FallbackKind::UnsupportedBoxDecoration,
        1,
    );
    assert_eq!(Arc::strong_count(&alias), refs);
    assert_eq!(images.len(), 2);
    assert!(alias.rgba.is_empty());
}

#[test]
fn box_decoration_addition_keeps_old_routes_and_dynamic_preparation_size_accounting() {
    let frame = Frame::new(4, 2, 0xffffff);
    let commands = [
        DrawCommand::Rect {
            rect: rect(0.0, 0.0, 4.0, 2.0),
            color: Color::WHITE,
            radius: 0.0,
        },
        DrawCommand::Image {
            rect: rect(0.0, 0.0, 1.0, 1.0),
            key: "old".into(),
        },
    ];
    let mut images = ImageStore::new();
    images.insert(
        "old".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        }),
    );
    let fonts = Fonts::new();
    let probe = plan_display_list(&commands, &images, frame).unwrap();
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
    for stats in [probe.stats(), text.stats(), &native.stats().bridge] {
        assert_eq!(
            (
                stats.original_commands,
                stats.unique_sources,
                stats.referenced_sources,
                stats.total_rgba_bytes,
                stats.missing_images
            ),
            (2, 1, 1, 4, 0)
        );
    }
    let without = native::preparation_peak_bytes(frame, false).unwrap();
    let with = native::preparation_peak_bytes(frame, true).unwrap();
    assert_eq!(with - without, 32);
    assert_eq!(native.stats().preparation_peak_bytes, without);
    assert!(without >= 64 * std::mem::size_of::<DrawCommand>() + 16 * 1024);
}
