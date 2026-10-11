use super::super::{BoxDecoration, BoxPaint};
use super::*;
use crate::graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore};
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

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
fn ellipse() -> RoundedBox {
    RoundedBox {
        rect: rect(0.0, 0.0, 4.0, 2.0),
        radii: [[2.0, 1.0]; 4],
    }
}
fn window(width: u32, height: u32, shape: RoundedBox, cutout: Option<RoundedBox>) -> BoxMaskWindow {
    BoxMaskWindow::new(
        width,
        height,
        shape,
        cutout,
        rect(0.0, 0.0, width as f32, height as f32),
    )
    .unwrap()
}
fn funded(value: BoxMaskWindow) -> BoxMaskReservation {
    value.reserve(&mut 32_000_000).unwrap()
}
fn bytes(value: BoxMaskReservation) -> Vec<u8> {
    value
        .materialize(|_| Ok(()), |error| error)
        .unwrap()
        .into_tiles()
        .flat_map(|tile| tile.coverage().to_vec())
        .collect()
}

#[test]
fn translated_geometry_keeps_wider_domain_and_validates_before_empty() {
    let far = square(rect(2_000_000.0, 0.0, 1.0, 1.0));
    assert_eq!(funded(window(4, 2, far, None)).info().candidate_pixels, 0);
    for bad in [
        square(rect(f32::NAN, 0.0, 0.0, 0.0)),
        square(rect(16_000_000.0, 0.0, 1.0, 0.0)),
        square(rect(0.0, 0.0, -1.0, 0.0)),
        RoundedBox {
            rect: Rect::default(),
            radii: [[-1.0, 0.0]; 4],
        },
        RoundedBox {
            rect: Rect::default(),
            radii: [[f32::INFINITY, 0.0]; 4],
        },
    ] {
        assert!(matches!(
            BoxMaskWindow::new(4, 2, bad, None, Rect::default()),
            Err(BoxMaskError::InvalidGeometry)
        ));
        assert!(matches!(
            BoxMaskWindow::new(4, 2, square(Rect::default()), Some(bad), Rect::default()),
            Err(BoxMaskError::InvalidGeometry)
        ));
    }
    for (width, height) in [(0, 1), (1, 0), (1281, 1), (1, 1025)] {
        assert!(matches!(
            BoxMaskWindow::new(
                width,
                height,
                square(Rect::default()),
                None,
                Rect::default()
            ),
            Err(BoxMaskError::InvalidGeometry)
        ));
    }
    assert!(matches!(
        BoxMaskWindow::new(
            4,
            2,
            square(Rect::default()),
            None,
            rect(f32::NAN, 0.0, 0.0, 0.0)
        ),
        Err(BoxMaskError::InvalidGeometry)
    ));
}

#[test]
fn opaque_square_has_literal_work_storage_and_absolute_rows() {
    let mut remaining = 42;
    let reservation = window(4, 2, square(rect(0.0, 0.0, 4.0, 2.0)), None)
        .reserve(&mut remaining)
        .unwrap();
    assert_eq!(remaining, 0);
    let info = reservation.info();
    assert_eq!(
        (
            info.candidate_pixels,
            info.ambiguous_pixels,
            info.shape_count
        ),
        (8, 0, 1)
    );
    assert_eq!(
        (info.classification_work, info.materialization_work),
        (16, 26)
    );
    assert_eq!(
        (info.tile_count, info.coverage_bytes, info.row_entries),
        (1, 8, 2)
    );
    assert_eq!((info.cpu_payload_bytes, info.packed_input_bytes), (16, 40));
    let tiles: Vec<_> = reservation
        .materialize(|_| Ok(()), |error| error)
        .unwrap()
        .into_tiles()
        .collect();
    assert_eq!(tiles[0].row_origins(), &[0, 0]);
    assert_eq!(tiles[0].coverage(), &[255; 8]);
    assert_eq!((tiles[0].info().origin_x, tiles[0].info().origin_y), (0, 0));
}

#[test]
fn ellipse_coverage_is_literal_nine_of_sixteen_at_corners() {
    let mut remaining = 106;
    let reservation = window(4, 2, ellipse(), None)
        .reserve(&mut remaining)
        .unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(
        (
            reservation.info().candidate_pixels,
            reservation.info().ambiguous_pixels
        ),
        (8, 4)
    );
    assert_eq!(
        (
            reservation.info().classification_work,
            reservation.info().materialization_work
        ),
        (16, 90)
    );
    assert_eq!(bytes(reservation), [143, 255, 255, 143, 143, 255, 255, 143]);
}

#[test]
fn fractional_ring_uses_correlated_samples_instead_of_quantized_subtraction() {
    let cutout = square(rect(0.25, 0.25, 1.0, 1.0));
    let mut remaining = 162;
    let reservation = window(2, 2, square(rect(0.0, 0.0, 2.0, 2.0)), Some(cutout))
        .reserve(&mut remaining)
        .unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(
        (
            reservation.info().shape_count,
            reservation.info().ambiguous_pixels
        ),
        (2, 4)
    );
    assert_eq!(
        (
            reservation.info().classification_work,
            reservation.info().materialization_work
        ),
        (16, 146)
    );
    assert_eq!(bytes(reservation), [111, 207, 207, 239]);
    assert_eq!(bytes(funded(window(2, 2, cutout, Some(cutout)))), [0; 4]);
    assert_eq!(
        bytes(funded(window(
            2,
            2,
            square(rect(0.0, 0.0, 2.0, 2.0)),
            Some(square(rect(8.0, 8.0, 1.0, 1.0)))
        ))),
        [255; 4]
    );
}

#[test]
fn truncated_and_overlapping_used_radii_are_never_renormalized() {
    let used = RoundedBox {
        rect: rect(0.0, 0.0, 1.0, 4.0),
        radii: [[2.0, 2.0], [0.0, 2.0], [0.0, 2.0], [2.0, 2.0]],
    };
    let mut remaining = 92;
    let reservation = window(1, 4, used, None).reserve(&mut remaining).unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(
        (
            reservation.info().candidate_pixels,
            reservation.info().ambiguous_pixels
        ),
        (4, 4)
    );
    assert_eq!(bytes(reservation), [95, 239, 239, 95]);
    let overlapping = RoundedBox {
        rect: rect(0.0, 0.0, 2.0, 2.0),
        radii: [[4.0; 2]; 4],
    };
    assert_eq!(bytes(funded(window(2, 2, overlapping, None))), [0; 4]);
}

#[test]
fn empty_candidates_pay_nothing_and_allocate_no_mask() {
    for shape in [square(Rect::default()), square(rect(8.0, 8.0, 1.0, 1.0))] {
        let mut remaining = 0;
        let reservation = window(4, 2, shape, None).reserve(&mut remaining).unwrap();
        assert_eq!(
            reservation.info(),
            BoxMaskInfo {
                candidate_pixels: 0,
                ambiguous_pixels: 0,
                shape_count: 1,
                classification_work: 0,
                materialization_work: 0,
                tile_count: 0,
                coverage_bytes: 0,
                row_entries: 0,
                cpu_payload_bytes: 0,
                packed_input_bytes: 0,
            }
        );
        let admitted = Cell::new(0);
        let result = reservation
            .materialize_with(
                |_| {
                    admitted.set(admitted.get() + 1);
                    Ok(())
                },
                |error| error,
                |_| -> Result<Vec<u8>, BoxMaskError> { panic!("no coverage allocation") },
                |_| -> Result<Vec<i32>, BoxMaskError> { panic!("no row allocation") },
                || panic!("no coverage evaluation"),
            )
            .unwrap();
        assert_eq!(admitted.get(), 1);
        assert_eq!(result.into_tiles().count(), 0);
    }
}

#[test]
fn upper_integer_clip_crops_storage_without_changing_candidate_tariff() {
    // Independent retained f32 cancellation witness: x+(1-x) is1+one ULP.
    // Helper receives this exact clip, without doing a second normalization.
    let shape = square(rect(f32::from_bits(0xbfff_ec57), 0.0, 4.0, 1.0));
    let value = BoxMaskWindow::new(2, 1, shape, None, rect(-2.0, 0.0, 3.0, 1.0)).unwrap();
    let mut remaining = 16;
    let reservation = value.reserve(&mut remaining).unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(
        (
            reservation.info().candidate_pixels,
            reservation.info().coverage_bytes
        ),
        (2, 1)
    );
    assert_eq!(
        (
            reservation.info().classification_work,
            reservation.info().materialization_work
        ),
        (10, 6)
    );
    assert_eq!(bytes(reservation), [255]);
    let value = BoxMaskWindow::new(
        4,
        2,
        square(rect(0.0, 0.0, 4.0, 2.0)),
        None,
        rect(0.5, 0.0, 2.5, 2.0),
    )
    .unwrap();
    let tiles: Vec<_> = funded(value)
        .materialize(|_| Ok(()), |error| error)
        .unwrap()
        .into_tiles()
        .collect();
    assert_eq!(tiles[0].row_origins(), &[1, 1]);
    assert_eq!(tiles[0].coverage(), &[255; 4]);
}

#[test]
fn partition_admits_whole_area_and_preserves_absolute_seam_coordinates() {
    for (width, count, last_width) in [(1024, 1, 1024), (1025, 2, 1), (1280, 2, 256)] {
        let reservation = funded(window(
            1280,
            1,
            square(rect(0.0, 0.0, width as f32, 1.0)),
            None,
        ));
        assert_eq!(
            (
                reservation.info().tile_count,
                reservation.info().row_entries
            ),
            (count, count)
        );
        let tiles: Vec<_> = reservation
            .materialize(|_| Ok(()), |error| error)
            .unwrap()
            .into_tiles()
            .collect();
        assert_eq!(tiles.last().unwrap().info().width, last_width);
        if count == 2 {
            assert_eq!(tiles[1].row_origins(), &[1024]);
        }
        assert!(
            tiles
                .iter()
                .all(|tile| tile.coverage().iter().all(|value| *value == 255))
        );
    }
    let edge = square(rect(0.25, 0.0, 1024.5, 1.0));
    let tiles: Vec<_> = funded(window(1280, 1, edge, None))
        .materialize(|_| Ok(()), |error| error)
        .unwrap()
        .into_tiles()
        .collect();
    assert_eq!(tiles.len(), 2);
    assert_eq!(tiles[0].row_origins(), &[0]);
    assert_eq!(tiles[0].coverage()[0], 191);
    assert!(tiles[0].coverage()[1..].iter().all(|value| *value == 255));
    assert_eq!(tiles[1].row_origins(), &[1024]);
    assert_eq!(tiles[1].coverage(), &[191]);
    let reservation = funded(window(
        1280,
        1024,
        square(rect(0.0, 0.0, 1280.0, 204.0)),
        None,
    ));
    assert_eq!(
        (
            reservation.info().coverage_bytes,
            reservation.info().row_entries
        ),
        (261_120, 408)
    );
    assert!(matches!(
        BoxMaskWindow::new(
            1280,
            1024,
            square(rect(0.0, 0.0, 1280.0, 205.0)),
            None,
            rect(0.0, 0.0, 1280.0, 1024.0)
        ),
        Err(BoxMaskError::MaskLimit)
    ));
}

#[test]
fn classification_stages_are_prepaid_at_exact_and_one_short_boundaries() {
    for (budget, classified, remaining, success) in [
        (15, 0, 15, false),
        (16, 8, 0, false),
        (105, 8, 89, false),
        (106, 8, 0, true),
    ] {
        let count = Cell::new(0);
        let mut work = budget;
        let result =
            window(4, 2, ellipse(), None).reserve_with(&mut work, || count.set(count.get() + 1));
        assert_eq!(count.get(), classified);
        assert_eq!(work, remaining);
        assert_eq!(result.is_ok(), success);
        if !success {
            assert!(matches!(result, Err(BoxMaskError::WorkLimit)));
        }
    }
}

#[test]
fn all_reserves_and_ledger_admission_precede_any_sample_or_publication() {
    for fail in 0..=4 {
        let events = RefCell::new(Vec::new());
        let calls = Cell::new(0);
        let evaluated = Cell::new(0);
        let reservation = funded(window(1280, 1, square(rect(0.0, 0.0, 1025.0, 1.0)), None));
        let result = reservation.materialize_with(
            |_| {
                events.borrow_mut().push("admit");
                if fail == 0 {
                    Err(BoxMaskError::MaskLimit)
                } else {
                    Ok(())
                }
            },
            |error| error,
            |n| {
                events.borrow_mut().push("coverage");
                calls.set(calls.get() + 1);
                if calls.get() == fail {
                    Err(BoxMaskError::AllocationFailure)
                } else {
                    reserve_vec::<u8>(n)
                }
            },
            |n| {
                events.borrow_mut().push("rows");
                calls.set(calls.get() + 1);
                if calls.get() == fail {
                    Err(BoxMaskError::AllocationFailure)
                } else {
                    reserve_vec::<i32>(n)
                }
            },
            || evaluated.set(evaluated.get() + 1),
        );
        assert!(result.is_err());
        assert_eq!(evaluated.get(), 0);
        assert_eq!(events.borrow().first(), Some(&"admit"));
        assert_eq!(calls.get(), fail);
    }
    let events = RefCell::new(Vec::new());
    let evaluated = Cell::new(0);
    let reservation = funded(window(1280, 1, square(rect(0.0, 0.0, 1025.0, 1.0)), None));
    let tiles = reservation
        .materialize_with(
            |_| {
                events.borrow_mut().push("admit");
                Ok(())
            },
            |error| error,
            |n| {
                events.borrow_mut().push("coverage");
                reserve_vec::<u8>(n)
            },
            |n| {
                events.borrow_mut().push("rows");
                reserve_vec::<i32>(n)
            },
            || {
                assert_eq!(events.borrow().len(), 5);
                evaluated.set(evaluated.get() + 1);
            },
        )
        .unwrap();
    assert_eq!(
        *events.borrow(),
        ["admit", "coverage", "rows", "coverage", "rows"]
    );
    assert_eq!(evaluated.get(), 1025);
    assert_eq!(tiles.into_tiles().count(), 2);
}

#[test]
fn literal_coverage_and_unchanged_canvas_source_alpha_agree() {
    for (color, clear, expected) in [
        (
            Color::rgb(0, 0, 255),
            Color::WHITE,
            [
                0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff, 0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff,
            ],
        ),
        (
            Color::rgba(255, 0, 0, 128),
            Color::rgb(255, 255, 0),
            [
                0xffb800, 0xff7f00, 0xff7f00, 0xffb800, 0xffb800, 0xff7f00, 0xff7f00, 0xffb800,
            ],
        ),
    ] {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.clear(clear);
        canvas.paint(
            &[DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
                shape: ellipse(),
                cutout: None,
                paint: BoxPaint::Color(color),
            }))],
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels, expected);
        assert!(!canvas.exhausted());
        let coverage = bytes(funded(window(4, 2, ellipse(), None)));
        assert_eq!(coverage, [143, 255, 255, 143, 143, 255, 255, 143]);
    }
}
