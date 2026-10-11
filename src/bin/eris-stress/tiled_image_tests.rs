use super::*;

#[test]
fn tiled_geometry_checks_both_boxes_positive_tiles_and_finite_nonnegative_radius() {
    let area = Rect {
        x: -2.0,
        y: 0.5,
        width: 8.0,
        height: 6.0,
    };
    let tile = Rect {
        x: 1.0,
        y: -3.0,
        width: 4.0,
        height: 2.0,
    };
    assert!(background_image_invariants(area, tile, 1.25).is_ok());
    assert!(background_image_invariants(Rect::default(), tile, 0.0).is_ok());
    for field in 0..7 {
        let (mut area, mut tile, mut radius) = (area, tile, 1.25);
        match field {
            0 => area.x = f32::NAN,
            1 => area.width = -1.0,
            2 => tile.y = f32::INFINITY,
            3 => tile.width = 0.0,
            4 => tile.height = -1.0,
            5 => radius = -1.0,
            6 => radius = f32::NAN,
            _ => unreachable!(),
        }
        assert!(
            background_image_invariants(area, tile, radius).is_err(),
            "field {field}"
        );
    }
}
