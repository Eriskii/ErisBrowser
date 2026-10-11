use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};

fn command() -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox {
            rect: rect(1.0, 2.0, 8.0, 6.0),
            radii: [[1.0, 2.0]; 4],
        },
        cutout: Some(RoundedBox {
            rect: rect(2.0, 3.0, 4.0, 3.0),
            radii: [[0.0, 1.0]; 4],
        }),
        paint: BoxPaint::Image {
            tile: rect(-2.0, 0.5, 2.0, 1.0),
            repeat: [false, true],
            key: "tile".into(),
        },
    }))
}

#[test]
fn box_decoration_translation_moves_outer_inner_and_sampling_phase() {
    let mut input = command();
    translate(&mut input, 3.0, -4.0);
    let BoxDecoration {
        shape,
        cutout,
        paint: BoxPaint::Image { tile, repeat, key },
    } = unpack(input)
    else {
        panic!()
    };
    assert_eq!(shape.rect, rect(4.0, -2.0, 8.0, 6.0));
    assert_eq!(shape.radii, [[1.0, 2.0]; 4]);
    assert_eq!(cutout.unwrap().rect, rect(5.0, -1.0, 4.0, 3.0));
    assert_eq!(tile, rect(1.0, -3.5, 2.0, 1.0));
    assert_eq!((repeat, key.as_str()), ([false, true], "tile"));
}

#[test]
fn box_decoration_unrepresentable_phase_is_omitted_without_clamping() {
    let mut input = command();
    translate(&mut input, 1_000_010.0, 0.0);
    let BoxDecoration { shape, .. } = unpack(input);
    assert_eq!(shape.rect, Rect::default());
}

#[test]
fn box_decoration_translation_rejects_far_edge_overflow_atomically() {
    for move_inner in [false, true] {
        let mut input = command();
        let BoxDecoration { shape, cutout, .. } = decoration_mut(&mut input);
        let wide = Rect {
            x: 0.0,
            y: 0.0,
            width: 16_000_000.0,
            height: 1.0,
        };
        if move_inner {
            cutout.as_mut().unwrap().rect = wide;
        } else {
            shape.rect = wide;
        }
        let original_cutout = *cutout;
        translate(&mut input, 1.0, 0.0);
        let BoxDecoration {
            shape,
            cutout,
            paint: BoxPaint::Image { tile, .. },
        } = unpack(input)
        else {
            panic!()
        };
        assert_eq!(shape.rect, Rect::default());
        assert_eq!(cutout, original_cutout);
        assert_eq!(tile, rect(-2.0, 0.5, 2.0, 1.0));
        assert!(shape.valid());
        assert!(cutout.unwrap().valid());
    }
}

use std::sync::Arc;

fn unpack(command: DrawCommand) -> BoxDecoration {
    let DrawCommand::BoxDecoration(payload) = command else {
        panic!()
    };
    Arc::unwrap_or_clone(payload)
}

fn decoration_mut(command: &mut DrawCommand) -> &mut BoxDecoration {
    let DrawCommand::BoxDecoration(payload) = command else {
        panic!()
    };
    Arc::make_mut(payload)
}

#[test]
fn shared_box_decoration_translation_preserves_the_original_fragment() {
    let original = command();
    let mut moved = original.clone();
    let (DrawCommand::BoxDecoration(a), DrawCommand::BoxDecoration(b)) = (&original, &moved) else {
        panic!()
    };
    assert!(Arc::ptr_eq(a, b));
    translate(&mut moved, 3.0, 4.0);
    let (DrawCommand::BoxDecoration(a), DrawCommand::BoxDecoration(b)) = (&original, &moved) else {
        panic!()
    };
    assert!(!Arc::ptr_eq(a, b));
    assert_eq!(a.shape.rect, rect(1.0, 2.0, 8.0, 6.0));
    assert_eq!(b.shape.rect, rect(4.0, 6.0, 8.0, 6.0));
    let (
        BoxPaint::Image {
            tile: before,
            key: old,
            ..
        },
        BoxPaint::Image {
            tile: after,
            key: new,
            ..
        },
    ) = (&a.paint, &b.paint)
    else {
        panic!()
    };
    assert_eq!(*before, rect(-2.0, 0.5, 2.0, 1.0));
    assert_eq!(*after, rect(1.0, 4.5, 2.0, 1.0));
    assert_eq!(old, new);
    assert_ne!(old.as_ptr(), new.as_ptr());
}
