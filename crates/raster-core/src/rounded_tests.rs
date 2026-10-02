use crate::rounded::{RoundedDisposition, RoundedShape};
use crate::{Frame, MAX_COORDINATE, MAX_MASK_COVERAGE_BYTES, Profile, Rect, reserved};
use std::cell::Cell;

fn shape(profile: Profile, frame: Frame, rect: Rect, clip: Rect, radius: f32) -> RoundedShape {
    match RoundedShape::prepare(profile, frame, rect, clip, radius).unwrap() {
        RoundedDisposition::Shape(shape) => shape,
        other => panic!("expected positive-radius coverage, got {other:?}"),
    }
}

fn full_shape(width: u32, height: u32, rect: Rect, radius: f32) -> RoundedShape {
    let frame = Frame::new(width, height, 0xffffff);
    shape(Profile::Probe, frame, rect, frame.caller_clip, radius)
}

#[test]
fn rounded_independent_two_by_two_corner_literal_and_radius_clamp() {
    // At every center dx=dy=0.5; trunc((1.5-sqrt(0.5))*255)=202.
    // Black at alpha255 over white would consequently be 0x353535; alpha128
    // gives effective alpha101 and 0x9a9a9a. This module returns coverage only.
    for radius in [1.0, 100.0, MAX_COORDINATE] {
        let mask = full_shape(2, 2, Rect::new(0.0, 0.0, 2.0, 2.0), radius)
            .materialize(|_| Ok(()))
            .unwrap();
        assert_eq!(mask.mask().coverage, &[202, 202, 202, 202]);
        assert_eq!(mask.row_origins(), &[0, 0]);
        assert_eq!(mask.origin(), (0, 0));
    }
}

#[test]
fn rounded_sub_half_radius_preserves_partial_interior_coverage() {
    let rect = Rect::new(0.0, 0.0, 1.0, 1.0);
    let quarter = full_shape(1, 1, rect, 0.25)
        .materialize(|_| Ok(()))
        .unwrap();
    assert_eq!(quarter.mask().coverage, &[191]);
    let half = full_shape(1, 1, rect, 0.5).materialize(|_| Ok(())).unwrap();
    assert_eq!(half.mask().coverage, &[255]);
    let subnormal = full_shape(1, 1, rect, f32::from_bits(1))
        .materialize(|_| Ok(()))
        .unwrap();
    assert_eq!(subnormal.mask().coverage, &[127]);
}

#[test]
fn rounded_fractional_clip_keeps_original_geometry_and_absolute_placement() {
    let frame = Frame::new(2, 1, 0xffffff);
    let rect = Rect::new(0.25, 0.0, 1.0, 1.0);
    let full = shape(Profile::Probe, frame, rect, frame.caller_clip, 0.25)
        .materialize(|_| Ok(()))
        .unwrap();
    assert_eq!(full.mask().coverage, &[191, 63]);
    let clipped = shape(
        Profile::Probe,
        frame,
        rect,
        Rect::new(1.0, 0.0, 1.0, 1.0),
        0.25,
    )
    .materialize(|_| Ok(()))
    .unwrap();
    assert_eq!(clipped.mask().coverage, &[63]);
    assert_eq!(clipped.row_origins(), &[1]);
    assert_eq!(clipped.origin(), (1, 0));
}

#[test]
fn rounded_negative_origin_and_nonzero_y_do_not_translate_twice() {
    let frame = Frame::new(2, 4, 0);
    let output = shape(
        Profile::Probe,
        frame,
        Rect::new(-0.25, 2.0, 1.0, 1.0),
        frame.caller_clip,
        0.25,
    )
    .materialize(|_| Ok(()))
    .unwrap();
    assert_eq!(output.mask().coverage, &[191]);
    assert_eq!(output.origin(), (0, 2));
    assert_eq!(output.origin_y(), 2);
    assert_eq!(output.row_origins(), &[0]);
}

#[test]
fn rounded_fractional_clip_tests_integer_origins_on_both_axes() {
    let frame = Frame::new(3, 3, 0);
    let output = shape(
        Profile::Probe,
        frame,
        Rect::new(0.0, 0.0, 3.0, 3.0),
        Rect::new(0.25, 0.25, 1.5, 1.5),
        0.25,
    )
    .materialize(|_| Ok(()))
    .unwrap();
    assert_eq!(output.origin(), (1, 1));
    assert_eq!(output.mask().coverage, &[191]);
    assert_eq!((output.mask().width, output.mask().height), (1, 1));
}

#[test]
fn rounded_clip_edges_on_both_sides_of_an_integer_remain_distinct() {
    let frame = Frame::new(3, 1, 0);
    let rect = Rect::new(0.0, 0.0, 3.0, 1.0);
    let below = f32::from_bits(1.0f32.to_bits() - 1);
    let above = f32::from_bits(1.0f32.to_bits() + 1);
    for (edge, expected_width) in [(below, 1), (1.0, 1), (above, 2)] {
        let output = shape(
            Profile::Probe,
            frame,
            rect,
            Rect::new(0.0, 0.0, edge, 1.0),
            0.25,
        )
        .materialize(|_| Ok(()))
        .unwrap();
        assert_eq!(output.mask().width, expected_width);
        assert!(output.mask().coverage.iter().all(|&v| v == 191));
    }
    for (edge, expected_x) in [(below, 1), (1.0, 1), (above, 2)] {
        let output = shape(
            Profile::Probe,
            frame,
            rect,
            Rect::new(edge, 0.0, 3.0 - edge, 1.0),
            0.25,
        )
        .materialize(|_| Ok(()))
        .unwrap();
        assert_eq!(output.origin().0, expected_x);
    }
}

#[test]
fn rounded_loop_work_precedes_blend_upper_clip_rejection() {
    // Independent f32 subtraction/addition example, not renderer output:
    // 1 - x = bits 0x403ff62c; x + that = 1 + one ulp.
    let frame = Frame::new(2, 1, 0);
    let rect = Rect::new(f32::from_bits(0xbfffec57), 0.0, 4.0, 1.0);
    let clip = Rect::new(-2.0, 0.0, 3.0, 1.0);
    let prepared = shape(Profile::Probe, frame, rect, clip, 0.25);
    assert_eq!(prepared.info().loop_work, 2);
    assert_eq!(prepared.info().coverage_bytes, 1);
    let output = prepared
        .materialize(|shape| {
            assert_eq!(shape.info().loop_work, 2);
            Ok(())
        })
        .unwrap();
    assert_eq!(output.mask().coverage, &[191]);
    assert!(matches!(
        RoundedShape::prepare(Profile::Probe, frame, rect, clip, 0.0).unwrap(),
        RoundedDisposition::ZeroRadius { loop_work: 2 }
    ));
}

#[test]
fn rounded_zero_negative_and_clamped_zero_take_typed_rectangle_path() {
    let frame = Frame::new(3, 1, 0);
    for radius in [0.0, -0.0, -0.25, -MAX_COORDINATE] {
        let disposition = RoundedShape::prepare(
            Profile::Probe,
            frame,
            Rect::new(0.0, 0.0, 3.0, 1.0),
            Rect::new(0.25, 0.0, 1.5, 1.0),
            radius,
        )
        .unwrap();
        assert!(matches!(
            disposition,
            RoundedDisposition::ZeroRadius { loop_work: 1 }
        ));
        assert_eq!(disposition.loop_work(), 1);
    }
    let disposition = RoundedShape::prepare(
        Profile::Probe,
        frame,
        Rect::new(0.0, 0.0, f32::from_bits(1), 1.0),
        frame.caller_clip,
        1.0,
    )
    .unwrap();
    assert!(matches!(
        disposition,
        RoundedDisposition::ZeroRadius { loop_work: 1 }
    ));
}

#[test]
fn rounded_empty_and_offscreen_geometry_do_not_become_zero_radius() {
    let frame = Frame::new(2, 2, 0);
    for rect in [
        Rect::new(0.0, 0.0, 0.0, 1.0),
        Rect::new(0.0, 0.0, 1.0, -1.0),
        Rect::new(3.0, 0.0, 1.0, 1.0),
        Rect::new(-3.0, -3.0, 1.0, 1.0),
        Rect::new(0.1, 0.1, 0.1, 0.1),
    ] {
        // The tiny positive rectangle still reaches a pixel under a full clip.
        let clip = if rect.x == 0.1 {
            Rect::new(0.1, 0.1, 0.1, 0.1)
        } else {
            frame.caller_clip
        };
        let disposition = RoundedShape::prepare(Profile::Probe, frame, rect, clip, 0.0).unwrap();
        assert!(matches!(
            disposition,
            RoundedDisposition::Empty { loop_work: 0 }
        ));
        assert_eq!(disposition.loop_work(), 0);
    }
}

#[test]
fn rounded_invalid_scalars_are_rejected_even_when_geometry_is_hidden() {
    let frame = Frame::new(2, 2, 0);
    let hidden = Rect::new(0.0, 0.0, 0.0, 0.0);
    let valid = Rect::new(0.0, 0.0, 1.0, 1.0);
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for rect in [
            Rect::new(invalid, 0.0, 1.0, 1.0),
            Rect::new(0.0, invalid, 1.0, 1.0),
            Rect::new(0.0, 0.0, invalid, 1.0),
            Rect::new(0.0, 0.0, 1.0, invalid),
        ] {
            assert!(RoundedShape::prepare(Profile::Probe, frame, rect, hidden, 1.0).is_err());
            assert!(RoundedShape::prepare(Profile::Probe, frame, hidden, rect, 1.0).is_err());
        }
        assert!(RoundedShape::prepare(Profile::Probe, frame, valid, hidden, invalid).is_err());
    }
    for radius in [MAX_COORDINATE + 1.0, -MAX_COORDINATE - 1.0] {
        assert!(RoundedShape::prepare(Profile::Probe, frame, hidden, hidden, radius).is_err());
    }
    for rect in [
        Rect::new(2.0 * MAX_COORDINATE + 1.0, 0.0, 0.0, 1.0),
        Rect::new(0.0, -2.0 * MAX_COORDINATE - 1.0, 1.0, 0.0),
        Rect::new(0.0, 0.0, MAX_COORDINATE + 1.0, 0.0),
        Rect::new(0.0, 0.0, 0.0, -MAX_COORDINATE - 1.0),
    ] {
        assert!(RoundedShape::prepare(Profile::Probe, frame, rect, hidden, 1.0).is_err());
    }
}

#[test]
fn rounded_translation_sum_bound_accepts_valid_hidden_extremes() {
    let frame = Frame::new(1, 1, 0);
    for coordinate in [-2.0 * MAX_COORDINATE, 2.0 * MAX_COORDINATE] {
        let result = RoundedShape::prepare(
            Profile::Probe,
            frame,
            Rect::new(coordinate, coordinate, MAX_COORDINATE, MAX_COORDINATE),
            frame.caller_clip,
            MAX_COORDINATE,
        )
        .unwrap();
        assert!(matches!(result, RoundedDisposition::Empty { .. }));
    }
}

#[test]
fn rounded_profile_viewport_checks_run_before_culling() {
    let empty = Rect::new(0.0, 0.0, 0.0, 0.0);
    for (width, height) in [(0, 1), (1, 0), (321, 240), (320, 241)] {
        assert!(
            RoundedShape::prepare(
                Profile::Probe,
                Frame::new(width, height, 0),
                empty,
                empty,
                1.0,
            )
            .is_err()
        );
    }
    assert!(
        RoundedShape::prepare(
            Profile::Native,
            Frame::new(1280, 1024, 0),
            empty,
            empty,
            1.0,
        )
        .is_ok()
    );
    assert!(
        RoundedShape::prepare(
            Profile::Native,
            Frame::new(1281, 1024, 0),
            empty,
            empty,
            1.0,
        )
        .is_err()
    );
}

#[test]
fn rounded_existing_mask_axis_area_and_storage_bounds_are_shared() {
    let frame = Frame::new(1280, 1024, 0);
    let exact = shape(
        Profile::Native,
        frame,
        Rect::new(0.0, 0.0, 512.0, 512.0),
        frame.caller_clip,
        1.0,
    );
    assert_eq!(exact.info().coverage_bytes, MAX_MASK_COVERAGE_BYTES);
    assert_eq!(exact.info().row_entries, 512);
    assert_eq!(exact.info().cpu_payload_bytes, 262_144 + 4 * 512);
    assert_eq!(exact.info().packed_input_bytes, 4 * (262_144 + 512));
    assert_eq!(exact.info().loop_work, 262_144);
    for rect in [
        Rect::new(0.0, 0.0, 1025.0, 1.0),
        Rect::new(0.0, 0.0, 1024.0, 257.0),
        Rect::new(0.0, 0.0, 513.0, 512.0),
    ] {
        assert!(
            RoundedShape::prepare(Profile::Native, frame, rect, frame.caller_clip, 1.0).is_err()
        );
    }
    let axis = shape(
        Profile::Native,
        frame,
        Rect::new(0.0, 0.0, 1024.0, 1.0),
        frame.caller_clip,
        1.0,
    );
    assert_eq!(axis.info().width, 1024);
    // Ordinary zero-radius rectangles need no mask and do not acquire its cap.
    assert!(matches!(
        RoundedShape::prepare(
            Profile::Native,
            frame,
            Rect::new(0.0, 0.0, 1280.0, 1024.0),
            frame.caller_clip,
            0.0,
        )
        .unwrap(),
        RoundedDisposition::ZeroRadius { .. }
    ));
}

#[test]
fn rounded_large_original_geometry_is_cropped_before_mask_storage_admission() {
    let frame = Frame::new(3, 2, 0);
    let prepared = shape(
        Profile::Probe,
        frame,
        Rect::new(-1.0, -1.0, MAX_COORDINATE, MAX_COORDINATE),
        frame.caller_clip,
        0.25,
    );
    let output = prepared.materialize(|_| Ok(())).unwrap();
    assert_eq!(output.mask().coverage, &[191; 6]);
    assert_eq!(output.row_origins(), &[0, 0]);
    assert_eq!(output.info().cpu_payload_bytes, 14);
    assert_eq!(output.info().packed_input_bytes, 32);
}

#[test]
fn rounded_preflight_failure_precedes_both_allocations_and_pixel_work() {
    let prepared = full_shape(2, 2, Rect::new(0.0, 0.0, 2.0, 2.0), 1.0);
    let called = Cell::new(0);
    let result = prepared.materialize_with(
        |shape| {
            called.set(called.get() + 1);
            assert_eq!(shape.info().loop_work, 4);
            assert_eq!(shape.info().cpu_payload_bytes, 12);
            assert_eq!(shape.info().packed_input_bytes, 24);
            Err("caller cumulative budget".into())
        },
        |_| panic!("coverage reserve after refused preflight"),
        |_| panic!("row reserve after refused preflight"),
    );
    assert_eq!(result.unwrap_err(), "caller cumulative budget");
    assert_eq!(called.get(), 1);
    // A later independent caller can still materialize this immutable shape.
    assert_eq!(
        prepared.materialize(|_| Ok(())).unwrap().mask().coverage,
        &[202; 4]
    );
}

#[test]
fn rounded_each_fallible_reserve_failure_returns_no_partial_mask() {
    let prepared = full_shape(2, 2, Rect::new(0.0, 0.0, 2.0, 2.0), 1.0);
    let phase = Cell::new(0);
    let first = prepared.materialize_with(
        |_| {
            phase.set(1);
            Ok(())
        },
        |count| {
            assert_eq!(phase.get(), 1);
            assert_eq!(count, 4);
            phase.set(2);
            Err("coverage reserve refused".into())
        },
        |_| panic!("second reserve after failed first reserve"),
    );
    assert_eq!(first.unwrap_err(), "coverage reserve refused");
    assert_eq!(phase.get(), 2);
    let second = prepared.materialize_with(
        |_| {
            phase.set(3);
            Ok(())
        },
        |count| {
            assert_eq!(phase.get(), 3);
            phase.set(4);
            reserved::<u8>(count)
        },
        |count| {
            assert_eq!(phase.get(), 4);
            assert_eq!(count, 2);
            phase.set(5);
            Err("row reserve refused".into())
        },
    );
    assert_eq!(second.unwrap_err(), "row reserve refused");
    assert_eq!(phase.get(), 5);
}

#[test]
fn rounded_one_short_caller_work_and_storage_refuse_before_reserve() {
    let prepared = full_shape(2, 2, Rect::new(0.0, 0.0, 2.0, 2.0), 1.0);
    for (work, cpu, packed) in [(3, 12, 24), (4, 11, 24), (4, 12, 23)] {
        let result = prepared.materialize_with(
            |shape| {
                let info = shape.info();
                if info.loop_work > work
                    || info.cpu_payload_bytes > cpu
                    || info.packed_input_bytes > packed
                {
                    Err("one short".into())
                } else {
                    Ok(())
                }
            },
            |_| panic!("refused allowance reached coverage allocation"),
            |_| panic!("refused allowance reached row allocation"),
        );
        assert_eq!(result.unwrap_err(), "one short");
    }
    let result = prepared
        .materialize(|shape| {
            assert_eq!(shape.info().loop_work, 4);
            assert_eq!(shape.info().cpu_payload_bytes, 12);
            assert_eq!(shape.info().packed_input_bytes, 24);
            Ok(())
        })
        .unwrap();
    assert_eq!(result.mask().coverage, &[202; 4]);
}

#[test]
fn rounded_zero_coverage_corners_still_count_and_remain_in_mask() {
    let prepared = full_shape(8, 8, Rect::new(0.0, 0.0, 8.0, 8.0), 4.0);
    let output = prepared.materialize(|_| Ok(())).unwrap();
    // Corner distance sqrt(3.5^2+3.5^2)>4.5, hence coverage zero.
    assert_eq!(output.mask().coverage[0], 0);
    assert_eq!(output.mask().coverage[7], 0);
    assert_eq!(output.mask().coverage[56], 0);
    assert_eq!(output.mask().coverage[63], 0);
    assert_eq!(output.mask().coverage[3 * 8 + 3], 255);
    assert_eq!(output.info().loop_work, 64);
    assert_eq!(output.info().coverage_bytes, 64);
    assert_eq!(output.row_origins(), &[0; 8]);
}
