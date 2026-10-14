//! Frozen independent helper proof, quota and allocation witnesses.
//! No expected pixel or fee is derived from candidate execution.
use super::*;
use std::cell::Cell;
use std::mem::{align_of, size_of};

const SEVEN_ROW: &[u8] = include_bytes!("core-seven-tile-row.bin");
fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn shape(width: f32, height: f32, radius: f32) -> RoundedBox {
    RoundedBox {
        rect: rect(0.0, 0.0, width, height),
        radii: [[radius; 2]; 4],
    }
}
fn selected(width: u32, height: u32, outer: RoundedBox, clip: Rect) -> BoxCoreWindow {
    BoxCoreWindow::new(width, height, outer, None, clip)
        .unwrap()
        .expect("frozen oversized selection")
}
fn primary() -> BoxCoreWindow {
    selected(
        512,
        513,
        shape(512.0, 513.0, 1.0),
        rect(0.0, 0.0, 512.0, 513.0),
    )
}
fn seven() -> BoxCoreWindow {
    selected(
        1280,
        205,
        RoundedBox {
            rect: rect(0.0, 0.0, 1280.0, 205.0),
            radii: [[1100.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1100.0, 1.0]],
        },
        rect(0.0, 0.0, 1280.0, 205.0),
    )
}
fn funded(window: BoxCoreWindow, work: u64) -> BoxCoreReservation {
    let mut remaining = work;
    let reservation = window.reserve(&mut remaining).unwrap();
    assert_eq!(remaining, 0);
    reservation
}
fn bounds(reservation: &BoxCoreReservation) -> Vec<(i32, i32, u32, u32)> {
    reservation
        .tiles()
        .map(|tile| (tile.origin_x, tile.origin_y, tile.width, tile.height))
        .collect()
}
fn allocated<T>(count: usize) -> Result<Vec<T>, BoxMaskError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| BoxMaskError::AllocationFailure)?;
    Ok(result)
}
fn projection(width: u32, height: u32, reservation: BoxCoreReservation) -> (Vec<u8>, Vec<u8>) {
    let core = reservation.core_rect();
    let mut coverage = vec![0u8; width as usize * height as usize];
    let mut owners = vec![0u8; coverage.len()];
    for y in core.y as u32..(core.y + core.height) as u32 {
        for x in core.x as u32..(core.x + core.width) as u32 {
            let index = (y * width + x) as usize;
            coverage[index] = 255;
            owners[index] += 1;
        }
    }
    let tiles = reservation.materialize(|_| Ok(()), |error| error).unwrap();
    for tile in tiles.into_tiles() {
        let info = tile.info();
        assert_eq!(tile.coverage().len(), info.coverage_bytes);
        assert_eq!(tile.row_origins(), vec![info.origin_x; info.row_entries]);
        for y in 0..info.height {
            for x in 0..info.width {
                let index = (info.origin_y as u32 + y) * width + info.origin_x as u32 + x;
                coverage[index as usize] = tile.coverage()[(y * info.width + x) as usize];
                owners[index as usize] += 1;
            }
        }
    }
    (coverage, owners)
}

#[test]
fn core_primary_partition_keeps_literal_corner_coverage_and_original_candidate_tariff() {
    let reservation = funded(primary(), 791_114);
    assert_eq!(reservation.core_rect(), rect(1.0, 1.0, 510.0, 511.0));
    let info = reservation.info();
    assert_eq!(
        (
            info.candidate_pixels,
            info.ambiguous_pixels,
            info.shape_count
        ),
        (262_656, 4, 1)
    );
    assert_eq!(
        (info.classification_work, info.materialization_work),
        (262_668, 528_446)
    );
    assert_eq!(
        (info.tile_count, info.coverage_bytes, info.row_entries),
        (4, 2_046, 1_024)
    );
    assert_eq!(
        (info.cpu_payload_bytes, info.packed_input_bytes),
        (6_142, 12_280)
    );
    assert_eq!(
        bounds(&reservation),
        [
            (0, 0, 512, 1),
            (0, 512, 512, 1),
            (0, 1, 1, 511),
            (511, 1, 1, 511),
        ]
    );
    let (actual, owners) = projection(512, 513, reservation);
    let mut expected = vec![255; 512 * 513];
    for index in [0, 511, 512 * 512, 512 * 513 - 1] {
        expected[index] = 207;
    }
    assert_eq!(actual, expected);
    assert!(owners.into_iter().all(|count| count == 1));
}

#[test]
fn core_asymmetric_used_radii_reach_all_seven_tiles_without_changing_absolute_samples() {
    let reservation = funded(seven(), 1_032_096);
    assert_eq!(reservation.core_rect(), rect(1100.0, 1.0, 179.0, 203.0));
    let info = reservation.info();
    assert_eq!(
        (info.candidate_pixels, info.ambiguous_pixels),
        (262_400, 1_138)
    );
    assert_eq!(
        (info.classification_work, info.materialization_work),
        (262_412, 769_684)
    );
    assert_eq!(
        (info.tile_count, info.coverage_bytes, info.row_entries),
        (7, 226_063, 613)
    );
    assert_eq!(
        (info.cpu_payload_bytes, info.packed_input_bytes),
        (228_515, 906_704)
    );
    assert_eq!(
        bounds(&reservation),
        [
            (0, 0, 1024, 1),
            (1024, 0, 256, 1),
            (0, 204, 1024, 1),
            (1024, 204, 256, 1),
            (0, 1, 1024, 203),
            (1024, 1, 76, 203),
            (1279, 1, 1, 203),
        ]
    );
    assert_eq!(SEVEN_ROW.len(), 1280);
    let mut expected = vec![255; 1280 * 205];
    expected[..1280].copy_from_slice(SEVEN_ROW);
    expected[1280 * 204..].copy_from_slice(SEVEN_ROW);
    let (actual, owners) = projection(1280, 205, reservation);
    assert_eq!(actual, expected);
    assert!(owners.into_iter().all(|count| count == 1));
}

#[test]
fn core_fractional_clip_and_all_visible_core_have_exact_nonoverlapping_storage() {
    let reservation = funded(
        selected(
            514,
            514,
            shape(514.0, 514.0, 1.0),
            rect(0.5, 0.25, 513.0, 513.0),
        ),
        791_073,
    );
    assert_eq!(reservation.core_rect(), rect(1.0, 1.0, 512.0, 512.0));
    let info = reservation.info();
    assert_eq!((info.candidate_pixels, info.ambiguous_pixels), (263_169, 1));
    assert_eq!(
        (info.classification_work, info.materialization_work),
        (263_181, 527_892)
    );
    assert_eq!(
        (info.tile_count, info.coverage_bytes, info.row_entries),
        (2, 1_025, 513)
    );
    assert_eq!(bounds(&reservation), [(1, 513, 513, 1), (513, 1, 1, 512)]);
    let (actual, owners) = projection(514, 514, reservation);
    for y in 0..514 {
        for x in 0..514 {
            let visible = x != 0 && y != 0;
            assert_eq!(owners[y * 514 + x], u8::from(visible));
            let expected = if !visible {
                0
            } else if x == 513 && y == 513 {
                207
            } else {
                255
            };
            assert_eq!(actual[y * 514 + x], expected);
        }
    }
    let outer = RoundedBox {
        rect: rect(-2.0, -2.0, 516.0, 517.0),
        radii: [[1.0; 2]; 4],
    };
    let reservation = funded(
        selected(512, 513, outer, rect(0.0, 0.0, 512.0, 513.0)),
        787_980,
    );
    assert_eq!(reservation.core_rect(), rect(0.0, 0.0, 512.0, 513.0));
    let info = reservation.info();
    assert_eq!((info.candidate_pixels, info.ambiguous_pixels), (262_656, 0));
    assert_eq!(
        (info.classification_work, info.materialization_work),
        (262_668, 525_312)
    );
    assert_eq!(
        (info.tile_count, info.coverage_bytes, info.row_entries),
        (0, 0, 0)
    );
    let calls = Cell::new(0);
    let result = reservation
        .materialize_with(
            |_| Ok(()),
            |error| error,
            |_| {
                calls.set(calls.get() + 1);
                allocated::<u8>(0)
            },
            |_| {
                calls.set(calls.get() + 1);
                allocated::<i32>(0)
            },
            || calls.set(calls.get() + 1),
        )
        .unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(result.into_tiles().count(), 0);
}

#[test]
fn core_selection_is_strictly_individually_oversized_and_invalid_geometry_stays_observable() {
    for (width, height) in [(512, 512), (1280, 204), (1, 1)] {
        assert!(
            BoxCoreWindow::new(
                width,
                height,
                shape(width as f32, height as f32, 1.0),
                None,
                rect(0.0, 0.0, width as f32, height as f32)
            )
            .unwrap()
            .is_none()
        );
    }
    let clip = rect(0.0, 0.0, 512.0, 513.0);
    let outer = shape(512.0, 513.0, 1.0);
    for cutout in [
        shape(0.0, 0.0, 0.0),
        outer,
        RoundedBox {
            rect: rect(2000.0, 0.0, 1.0, 1.0),
            radii: [[0.0; 2]; 4],
        },
    ] {
        assert!(
            BoxCoreWindow::new(512, 513, outer, Some(cutout), clip)
                .unwrap()
                .is_none()
        );
    }
    for area in [
        rect(0.25, 0.0, 512.0, 513.0),
        rect(0.0, 0.0, 511.5, 513.0),
        rect(0.0, 0.0, 0.0, 513.0),
        rect(2000.0, 0.0, 512.0, 513.0),
    ] {
        assert!(
            BoxCoreWindow::new(
                512,
                513,
                RoundedBox {
                    rect: area,
                    ..outer
                },
                None,
                clip
            )
            .unwrap()
            .is_none()
        );
    }
    let no_core = RoundedBox {
        radii: [[256.0, 256.5]; 4],
        ..outer
    };
    assert!(
        BoxCoreWindow::new(512, 513, no_core, None, clip)
            .unwrap()
            .is_none()
    );
    assert!(
        BoxCoreWindow::new(
            1280,
            1024,
            shape(1280.0, 1024.0, 400.0),
            None,
            rect(0.0, 0.0, 1280.0, 1024.0)
        )
        .unwrap()
        .is_none()
    );
    for (width, height) in [(0, 513), (512, 0), (1281, 513), (512, 1025)] {
        assert!(matches!(
            BoxCoreWindow::new(width, height, outer, None, clip),
            Err(BoxMaskError::InvalidGeometry)
        ));
    }
    for bad in [f32::NAN, f32::INFINITY, -1.0, 16_000_002.0] {
        let mut invalid = outer;
        invalid.radii[0][0] = bad;
        assert!(matches!(
            BoxCoreWindow::new(512, 513, invalid, None, clip),
            Err(BoxMaskError::InvalidGeometry)
        ));
        assert!(matches!(
            BoxCoreWindow::new(512, 513, outer, Some(invalid), clip),
            Err(BoxMaskError::InvalidGeometry)
        ));
    }
    for bad in [f32::NAN, f32::INFINITY, 16_000_002.0] {
        assert!(matches!(
            BoxCoreWindow::new(512, 513, outer, None, rect(bad, 0.0, 0.0, 0.0)),
            Err(BoxMaskError::InvalidGeometry)
        ));
    }
}

#[test]
fn core_both_work_stages_precede_queries_samples_and_payloads_at_exact_cuts() {
    for (budget, remaining_expected, proofs, classes, success) in [
        (262_667, 262_667, 0, 0, false),
        (262_668, 0, 4, 2_046, false),
        (791_113, 528_445, 4, 2_046, false),
        (791_114, 0, 4, 2_046, true),
    ] {
        let proof_count = Cell::new(0);
        let classify_count = Cell::new(0);
        let mut remaining = budget;
        let result = primary().reserve_with(
            &mut remaining,
            || proof_count.set(proof_count.get() + 1),
            || classify_count.set(classify_count.get() + 1),
        );
        assert_eq!(remaining, remaining_expected);
        assert_eq!((proof_count.get(), classify_count.get()), (proofs, classes));
        assert_eq!(result.is_ok(), success);
        if !success {
            assert!(matches!(result, Err(BoxMaskError::WorkLimit)));
        }
    }
}

#[test]
fn core_defensive_hull_refusal_is_funded_before_any_candidate_classification() {
    // Test-only corruption of immutable metadata. No production constructor
    // exposes this state; it pins the defensive membership failure boundary.
    let mut window = primary();
    window.core.x = -1;
    let mut remaining = 791_114;
    let proofs = Cell::new(0);
    let classes = Cell::new(0);
    let result = window.reserve_with(
        &mut remaining,
        || proofs.set(proofs.get() + 1),
        || classes.set(classes.get() + 1),
    );
    assert!(matches!(result, Err(BoxMaskError::MaskLimit)));
    assert_eq!((remaining, proofs.get(), classes.get()), (528_446, 4, 0));
}

#[test]
fn core_aggregate_admission_runs_once_before_payload_allocation() {
    let reservation = funded(primary(), 791_114);
    let admits = Cell::new(0);
    let reserves = Cell::new(0);
    let evaluations = Cell::new(0);
    let result = reservation.materialize_with(
        |shape| {
            admits.set(admits.get() + 1);
            assert_eq!(shape.info().tile_count, 4);
            Err(BoxMaskError::MaskLimit)
        },
        |error| error,
        |n| {
            reserves.set(reserves.get() + 1);
            allocated::<u8>(n)
        },
        |n| {
            reserves.set(reserves.get() + 1);
            allocated::<i32>(n)
        },
        || evaluations.set(evaluations.get() + 1),
    );
    assert!(matches!(result, Err(BoxMaskError::MaskLimit)));
    assert_eq!((admits.get(), reserves.get(), evaluations.get()), (1, 0, 0));
}

#[test]
fn core_all_fourteen_reserves_succeed_before_the_first_coverage_sample() {
    for fail_at in 1..=14 {
        let reservation = funded(seven(), 1_032_096);
        let calls = Cell::new(0);
        let evaluated = Cell::new(0);
        let result = reservation.materialize_with(
            |_| Ok(()),
            |error| error,
            |n| {
                calls.set(calls.get() + 1);
                if calls.get() == fail_at {
                    Err(BoxMaskError::AllocationFailure)
                } else {
                    allocated::<u8>(n)
                }
            },
            |n| {
                calls.set(calls.get() + 1);
                if calls.get() == fail_at {
                    Err(BoxMaskError::AllocationFailure)
                } else {
                    allocated::<i32>(n)
                }
            },
            || evaluated.set(evaluated.get() + 1),
        );
        assert!(matches!(result, Err(BoxMaskError::AllocationFailure)));
        assert_eq!((calls.get(), evaluated.get()), (fail_at, 0));
    }
    // Malformed private allocation-hook results also refuse before any sample.
    for bad in 0..4 {
        let evaluated = Cell::new(0);
        let result = funded(primary(), 791_114).materialize_with(
            |_| Ok(()),
            |error| error,
            |n| match bad {
                0 => Ok(Vec::new()),
                1 => Ok(vec![0]),
                _ => allocated::<u8>(n),
            },
            |n| match bad {
                2 => Ok(Vec::new()),
                3 => Ok(vec![0]),
                _ => allocated::<i32>(n),
            },
            || evaluated.set(evaluated.get() + 1),
        );
        assert!(matches!(result, Err(BoxMaskError::AllocationFailure)));
        assert_eq!(evaluated.get(), 0);
    }
    let reservation = funded(seven(), 1_032_096);
    let calls = Cell::new(0);
    let evaluated = Cell::new(0);
    let tiles = reservation
        .materialize_with(
            |_| Ok(()),
            |error| error,
            |n| {
                calls.set(calls.get() + 1);
                allocated::<u8>(n)
            },
            |n| {
                calls.set(calls.get() + 1);
                allocated::<i32>(n)
            },
            || {
                assert_eq!(calls.get(), 14);
                evaluated.set(evaluated.get() + 1);
            },
        )
        .unwrap();
    assert_eq!(
        (calls.get(), evaluated.get(), tiles.into_tiles().count()),
        (14, 226_063, 7)
    );
}

#[test]
fn core_actual_type_layout_is_observed_without_guessing_stack_or_allocator_usage() {
    macro_rules! observe {
        ($ty:ty) => {
            println!(
                "rounded-core-type {} size={} align={}",
                stringify!($ty),
                size_of::<$ty>(),
                align_of::<$ty>()
            );
        };
    }
    observe!(BoxCoreWindow);
    observe!(BoxCoreReservation);
    observe!(BoxCoreTiles);
    observe!(BoxMaskInfo);
    observe!(BoxMaskTile);
    observe!(PixelWindow);
    // This is an actual-type observation for root's final receipt, not a fixed
    // ABI, compiler frame, allocator overhead or RSS bound.
    assert_eq!(size_of::<BoxCoreReservation>(), size_of::<BoxCoreWindow>());
}
