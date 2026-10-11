use super::*;
use eris::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};

fn command() -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: Rect {
                x: -0.0,
                y: 1.0,
                width: 2.0,
                height: 3.0,
            },
            radii: [[0.0; 2]; 4],
        },
        cutout: None,
        paint: BoxPaint::Color(Color::rgba(1, 2, 3, 4)),
    }))
}
fn encoded(value: &DrawCommand) -> Result<Vec<u8>, String> {
    let mut out = Bytes::new()?;
    commands(&mut out, std::slice::from_ref(value))?;
    Ok(out.0)
}
#[test]
fn box_decoration_identity_has_literal_tag_and_all_shape_bits() {
    // Count, tag13, four raw f32 coordinates, eight radii, no cutout,
    // color subtype0 and RGBA. Signed zero is intentionally preserved.
    let expected = [
        1, 0, 0, 0, 13, 0, 0, 0, 128, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2,
        3, 4,
    ];
    assert_eq!(encoded(&command()).unwrap(), expected);
    for i in 0..8 {
        let mut value = command();
        let BoxDecoration { shape, .. } = decoration_mut(&mut value);
        shape.radii[i / 2][i % 2] = 0.25;
        assert_ne!(encoded(&value).unwrap(), expected);
        let BoxDecoration { shape, .. } = decoration_mut(&mut value);
        shape.radii[i / 2][i % 2] = f32::NAN;
        assert!(encoded(&value).is_err());
    }
    let mut value = command();
    let BoxDecoration { shape, cutout, .. } = decoration_mut(&mut value);
    *cutout = Some(*shape);
    assert_ne!(encoded(&value).unwrap(), expected);
}

fn decoration_mut(command: &mut DrawCommand) -> &mut BoxDecoration {
    let DrawCommand::BoxDecoration(payload) = command else {
        panic!()
    };
    Arc::make_mut(payload)
}
