//! Proof metadata literals frozen before extracting the existing rectangle proof.
use super::*;
use eris_raster_core::{BackgroundImageGeometry, Frame, Rect as CoreRect, scope::CoordinateState};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn square(rect: Rect) -> RoundedBox {
    RoundedBox {
        rect,
        radii: [[0.0; 2]; 4],
    }
}

#[test]
fn proof_exposes_outer_candidates_and_separate_box_and_color_tariffs() {
    let cases = [
        (
            4,
            2,
            rect(0.0, 0.0, 4.0, 2.0),
            rect(0.0, 0.0, 4.0, 2.0),
            8,
            24,
            32,
        ),
        (
            6,
            3,
            rect(0.0, 0.0, 6.0, 3.0),
            rect(0.5, 0.25, 3.0, 1.5),
            3,
            14,
            17,
        ),
        (
            1280,
            1024,
            rect(0.0, 0.0, 1280.0, 1024.0),
            rect(0.0, 0.0, 1280.0, 1024.0),
            1_310_720,
            2_621_448,
            3_932_168,
        ),
    ];
    for (width, height, area, clip, pixels, box_work, color_work) in cases {
        let proof = BoxMaskWindow::rect_proof(width, height, square(area), None, clip)
            .unwrap()
            .unwrap();
        assert_eq!(proof.candidate_pixels(), pixels);
        assert_eq!(proof.classification_work(), box_work);
        assert_eq!(
            BoxMaskWindow::rect_work(width, height, square(area), None, clip),
            Ok(Some(color_work))
        );
    }
}

#[test]
fn proof_and_color_wrapper_preserve_empty_ineligible_and_invalid_results() {
    let clip = rect(0.0, 0.0, 4.0, 2.0);
    let empty = square(rect(20.0, 0.0, 4.0, 2.0));
    let proof = BoxMaskWindow::rect_proof(4, 2, empty, None, clip)
        .unwrap()
        .unwrap();
    assert_eq!(
        (proof.candidate_pixels(), proof.classification_work()),
        (0, 0)
    );
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, empty, None, clip),
        Ok(Some(0))
    );
    let fractional = square(rect(0.25, 0.0, 4.0, 2.0));
    assert!(
        BoxMaskWindow::rect_proof(4, 2, fractional, None, clip)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, fractional, None, clip),
        Ok(None)
    );
    // A valid empty/disjoint cutout remains ineligible even if its mask is empty.
    assert!(
        BoxMaskWindow::rect_proof(4, 2, empty, Some(empty), clip)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, empty, Some(empty), clip),
        Ok(None)
    );
    let mut invalid = empty;
    invalid.radii[0][0] = f32::NAN;
    assert!(matches!(
        BoxMaskWindow::rect_proof(4, 2, invalid, None, clip),
        Err(BoxMaskError::InvalidGeometry)
    ));
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, invalid, None, clip),
        Err(BoxMaskError::InvalidGeometry)
    );
}

#[test]
fn image_loop_narrowing_never_replaces_the_original_box_candidate_allowance() {
    let frame = Frame::new(8, 4, 0xffffff);
    let state = CoordinateState::new_for_profile(Profile::Native, frame).unwrap();
    let area = rect(0.0, 0.0, 8.0, 4.0);
    let proof = BoxMaskWindow::rect_proof(8, 4, square(area), None, area)
        .unwrap()
        .unwrap();
    assert_eq!(proof.candidate_pixels(), 32);
    assert_eq!(proof.classification_work(), 72);
    for (repeat, loop_work, lut_work, total) in [
        ([false, false], 32, 26, 130),
        ([true, false], 72, 46, 190),
        ([false, true], 72, 36, 180),
        ([true, true], 168, 56, 296),
    ] {
        let image = BackgroundImageGeometry::new(
            frame,
            CoreRect::new(0.0, 0.0, 8.0, 4.0),
            CoreRect::new(2.0, 1.0, 4.0, 2.0),
            repeat,
            &state,
        )
        .unwrap();
        assert_eq!((image.loop_work, image.lut_work), (loop_work, lut_work));
        assert_eq!(
            proof.classification_work() + image.loop_work + image.lut_work,
            total
        );
    }
    let disjoint = BackgroundImageGeometry::new(
        frame,
        CoreRect::new(0.0, 0.0, 8.0, 4.0),
        CoreRect::new(20.0, 20.0, 4.0, 2.0),
        [false, false],
        &state,
    )
    .unwrap();
    assert_eq!(disjoint.bounds, None);
    assert_eq!((disjoint.loop_work, disjoint.lut_work), (0, 0));
    assert_eq!(
        proof.classification_work() + disjoint.loop_work + disjoint.lut_work,
        72
    );
}
