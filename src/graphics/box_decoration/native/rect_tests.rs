//! Pure rectangle proof: literal expectations frozen before its implementation.
use super::*;

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn square(area: Rect) -> RoundedBox {
    RoundedBox {
        rect: area,
        radii: [[0.0; 2]; 4],
    }
}

#[test]
fn square_and_zero_axis_corners_reserve_reference_work_without_mask_limits() {
    let clip = rect(0.0, 0.0, 4.0, 2.0);
    let mut shape = square(clip);
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, shape, None, clip),
        Ok(Some(32))
    );
    shape.radii = [[0.0, 5.0], [4.0, 0.0], [0.0, 0.0], [0.0, 8.0]];
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, shape, None, clip),
        Ok(Some(32))
    );
    let full = rect(0.0, 0.0, 1280.0, 1024.0);
    assert_eq!(
        BoxMaskWindow::rect_work(1280, 1024, square(full), None, full),
        Ok(Some(3_932_168))
    );
    // The old mask route is unchanged, including its fixed payload cap.
    assert!(matches!(
        BoxMaskWindow::new(1280, 1024, square(full), None, full),
        Err(BoxMaskError::MaskLimit)
    ));
}

#[test]
fn fractional_clips_keep_integer_origin_and_f32_endpoint_membership() {
    let outer = square(rect(0.0, 0.0, 6.0, 3.0));
    assert_eq!(
        BoxMaskWindow::rect_work(6, 3, outer, None, rect(0.5, 0.25, 3.0, 1.5)),
        Ok(Some(17))
    );
    // Immediately below/above right edge 3: integer x=3 joins only above.
    for (right, work) in [(0x403f_ffff, 17), (0x4040_0001, 20)] {
        assert_eq!(
            BoxMaskWindow::rect_work(
                6,
                3,
                outer,
                None,
                rect(0.0, 0.0, f32::from_bits(right), 1.0)
            ),
            Ok(Some(work))
        );
    }
    // Both additions reconstruct right=3 in f32. The lower ceil differs:
    // below1 includes x=1,2; above1 includes only x=2.
    for (left, work) in [(0x3f7f_ffff, 14), (0x3f80_0001, 11)] {
        assert_eq!(
            BoxMaskWindow::rect_work(6, 3, outer, None, rect(f32::from_bits(left), 0.0, 2.0, 1.0)),
            Ok(Some(work))
        );
    }
}

#[test]
fn translated_integer_and_empty_windows_are_proved_without_changing_raw_geometry() {
    let clip = rect(0.0, 0.0, 4.0, 2.0);
    let raw = square(rect(-0.25, -0.75, 4.0, 2.0));
    assert_eq!(BoxMaskWindow::rect_work(4, 2, raw, None, clip), Ok(None));
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, raw.translated(0.25, 0.75), None, clip),
        Ok(Some(32))
    );
    assert_eq!(raw.rect, rect(-0.25, -0.75, 4.0, 2.0));
    for area in [
        rect(0.0, 0.0, 0.0, 2.0),
        rect(20.0, 0.0, 4.0, 2.0),
        rect(2_000_000.0, 0.0, 4.0, 2.0),
    ] {
        assert_eq!(
            BoxMaskWindow::rect_work(4, 2, square(area), None, clip),
            Ok(Some(0))
        );
    }
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, square(clip), None, rect(0.0, 0.0, 0.0, 2.0)),
        Ok(Some(0))
    );
}

#[test]
fn fractional_rounded_and_cutout_shapes_keep_the_old_mask_route() {
    let clip = rect(0.0, 0.0, 4.0, 2.0);
    let outer = square(clip);
    let mut rounded = outer;
    rounded.radii[0] = [1.0, 1.0];
    assert_eq!(
        BoxMaskWindow::rect_work(4, 2, rounded, None, clip),
        Ok(None)
    );
    for area in [
        rect(0.25, 0.0, 4.0, 2.0),
        rect(0.0, 0.25, 4.0, 2.0),
        rect(0.0, 0.0, 3.5, 2.0),
        rect(0.0, 0.0, 4.0, 1.5),
    ] {
        assert_eq!(
            BoxMaskWindow::rect_work(4, 2, square(area), None, clip),
            Ok(None)
        );
    }
    for cutout in [
        outer,
        square(rect(0.0, 0.0, 0.0, 0.0)),
        square(rect(20.0, 0.0, 1.0, 1.0)),
    ] {
        assert_eq!(
            BoxMaskWindow::rect_work(4, 2, outer, Some(cutout), clip),
            Ok(None)
        );
    }
    let endpoint = square(rect(f32::from_bits(0xbfff_ec57), 0.0, 4.0, 1.0));
    let clip = rect(-2.0, 0.0, 3.0, 1.0);
    assert_eq!(
        BoxMaskWindow::rect_work(2, 1, endpoint, None, clip),
        Ok(None)
    );
    let mut remaining = 16;
    let old = BoxMaskWindow::new(2, 1, endpoint, None, clip)
        .unwrap()
        .reserve(&mut remaining)
        .unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(
        (old.info().candidate_pixels, old.info().coverage_bytes),
        (2, 1)
    );
}

#[test]
fn invalid_geometry_is_rejected_before_empty_or_ineligible_proofs() {
    let clip = rect(0.0, 0.0, 4.0, 2.0);
    let hidden = square(rect(20.0, 0.0, 0.0, 2.0));
    for (width, height) in [(0, 2), (4, 0), (1281, 2), (4, 1025)] {
        assert_eq!(
            BoxMaskWindow::rect_work(width, height, hidden, None, clip),
            Err(BoxMaskError::InvalidGeometry)
        );
    }
    for value in [f32::NAN, f32::INFINITY, 16_000_002.0] {
        for field in 0..4 {
            let mut invalid_clip = clip;
            match field {
                0 => invalid_clip.x = value,
                1 => invalid_clip.y = value,
                2 => invalid_clip.width = value,
                _ => invalid_clip.height = value,
            }
            assert_eq!(
                BoxMaskWindow::rect_work(4, 2, hidden, None, invalid_clip),
                Err(BoxMaskError::InvalidGeometry)
            );
        }
    }
    for value in [f32::NAN, f32::INFINITY, -1.0, 16_000_002.0] {
        for corner in 0..4 {
            for axis in 0..2 {
                let mut invalid = hidden;
                invalid.radii[corner][axis] = value;
                assert_eq!(
                    BoxMaskWindow::rect_work(4, 2, invalid, None, clip),
                    Err(BoxMaskError::InvalidGeometry)
                );
                assert_eq!(
                    BoxMaskWindow::rect_work(4, 2, hidden, Some(invalid), clip),
                    Err(BoxMaskError::InvalidGeometry)
                );
            }
        }
    }
    for invalid in [
        square(rect(16_000_000.0, 0.0, 1.0, 1.0)),
        square(rect(0.0, 0.0, -1.0, 1.0)),
        square(rect(0.0, 0.0, 1.0, -1.0)),
    ] {
        assert_eq!(
            BoxMaskWindow::rect_work(4, 2, invalid, None, clip),
            Err(BoxMaskError::InvalidGeometry)
        );
    }
}
