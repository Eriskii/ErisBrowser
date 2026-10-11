use super::*;
use eris::graphics::gradient::{GradientPosition, GradientStop};

fn encode(command: &DrawCommand) -> Result<Vec<u8>, String> {
    let mut bytes = Bytes::new()?;
    commands(&mut bytes, std::slice::from_ref(command))?;
    Ok(bytes.0)
}

#[test]
fn gradient_identity_binds_every_paint_field_and_keeps_arc_addresses_out() {
    let base = super::super::gradient_tests::command();
    let before = encode(&base).unwrap();
    for field in 0..13 {
        let mut value = base.clone();
        let DrawCommand::LinearGradient {
            rect,
            tile,
            radius,
            gradient,
            current_color,
            stop_scale,
            ..
        } = &mut value
        else {
            panic!()
        };
        match field {
            0 => rect.x += 0.25,
            1 => tile.y += 0.25,
            2 => tile.width += 1.0,
            3 => *radius += 0.5,
            4 => current_color.a ^= 1,
            5 => *stop_scale += 0.25,
            6 => {
                Arc::make_mut(gradient).direction = GradientDirection::Corner {
                    right: false,
                    bottom: false,
                }
            }
            7 => Arc::make_mut(gradient).direction = GradientDirection::Angle(-45.5),
            8 => {
                Arc::make_mut(&mut Arc::make_mut(gradient).stops)[0].color =
                    GradientColor::Color(Color::rgba(12, 34, 56, 78))
            }
            9 => {
                Arc::make_mut(&mut Arc::make_mut(gradient).stops)[0]
                    .position
                    .as_mut()
                    .unwrap()
                    .px += 0.25
            }
            10 => {
                Arc::make_mut(&mut Arc::make_mut(gradient).stops)[0]
                    .position
                    .as_mut()
                    .unwrap()
                    .percent += 1.0
            }
            11 => Arc::make_mut(&mut Arc::make_mut(gradient).stops)[0].position = None,
            _ => {
                Arc::make_mut(gradient).stops = Arc::from([GradientStop {
                    color: GradientColor::CurrentColor,
                    position: Some(GradientPosition {
                        px: 2.5,
                        percent: 25.0,
                    }),
                }])
            }
        }
        assert_ne!(encode(&value).unwrap(), before, "field {field}");
    }
    let other = super::super::gradient_tests::command();
    assert_eq!(encode(&other).unwrap(), before);
    let DrawCommand::LinearGradient { gradient: a, .. } = base else {
        panic!()
    };
    let DrawCommand::LinearGradient { gradient: b, .. } = other else {
        panic!()
    };
    assert!(!Arc::ptr_eq(&a, &b));
}

#[test]
fn gradient_identity_rejects_nonfinite_metadata_and_old_rectangle_bytes_stay_exact() {
    let mut value = super::super::gradient_tests::command();
    if let DrawCommand::LinearGradient { gradient, .. } = &mut value {
        Arc::make_mut(gradient).direction = GradientDirection::Angle(f64::NAN);
    }
    assert!(encode(&value).is_err());
    let rect = DrawCommand::Rect {
        rect: Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        },
        color: Color::rgba(5, 6, 7, 8),
        radius: 9.0,
    };
    let mut expected = vec![1, 0, 0, 0, 6];
    for n in [1.0f32, 2.0, 3.0, 4.0] {
        expected.extend_from_slice(&n.to_le_bytes());
    }
    expected.extend_from_slice(&[5, 6, 7, 8]);
    expected.extend_from_slice(&9.0f32.to_le_bytes());
    assert_eq!(encode(&rect).unwrap(), expected);
}
