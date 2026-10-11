use super::*;

fn table(start: u32, count: u32, origin: f32, extent: f32, source: u32) -> Vec<u32> {
    let mut bytes = Vec::new();
    assert_eq!(
        append_table(&mut bytes, start, count, origin, extent, source).unwrap(),
        0
    );
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| u32::from_le_bytes(*b))
        .collect()
}

fn words(plan: &Plan, start: u32, count: u32) -> Vec<u32> {
    let begin = start as usize * 4;
    plan.input_bytes()[begin..begin + count as usize * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| u32::from_le_bytes(*b))
        .collect()
}

#[test]
fn identity_lut_contains_every_source_row_including_125() {
    let actual = table(0, 240, 0.0, 240.0, 240);
    assert_eq!(actual, (0..240).collect::<Vec<u32>>());
    assert_eq!(&actual[124..127], &[124, 125, 126]);
}

#[test]
fn identity_lut_translation_clip_and_clamps_have_literal_indices() {
    assert_eq!(table(0, 5, 0.5, 4.0, 4), [0, 0, 1, 2, 3]);
    assert_eq!(table(1, 3, -0.25, 4.0, 4), [1, 2, 3]);
    assert_eq!(table(125, 3, 123.0, 240.0, 240), [2, 3, 4]);
    assert_eq!(table(0, 3, 1.0, 1.0, 1), [0, 0, 0]);
    assert_eq!(table(4, 2, 0.0, 4.0, 4), [3, 3]);
}

#[test]
fn mixed_axis_plans_publish_exact_identity_and_scaled_lookup_tables() {
    for (width, height, frame, rect, x_expected, y_expected) in [
        (
            240,
            2,
            Frame::new(240, 4, 0),
            Rect::new(0.0, 0.0, 240.0, 4.0),
            (0..240).collect::<Vec<u32>>(),
            vec![0, 0, 1, 1],
        ),
        (
            2,
            240,
            Frame::new(4, 240, 0),
            Rect::new(0.0, 0.0, 4.0, 240.0),
            vec![0, 0, 1, 1],
            (0..240).collect::<Vec<u32>>(),
        ),
    ] {
        let bytes = vec![255; width as usize * height as usize * 4];
        let source = SourceImage {
            width,
            height,
            rgba: &bytes,
        };
        let plan =
            plan_with_images(frame, &[Command::Image { rect, source: 0 }], &[source]).unwrap();
        assert_eq!(plan.draws.len(), 2);
        let draw = plan.draws[1];
        let image = draw.image.unwrap();
        assert_eq!(words(&plan, image.x_base, draw.width), x_expected);
        assert_eq!(words(&plan, image.y_base, draw.height), y_expected);
        assert_eq!(plan.input.len(), 2896);
        assert_eq!(plan.gpu_buffer_bytes(), 11088);
        assert_eq!(plan.invocations(), 3840);
    }
}

#[test]
fn clipped_negative_identity_origin_keeps_draw_geometry_and_lut_bounds() {
    let bytes = [255u8; 48];
    let source = SourceImage {
        width: 4,
        height: 3,
        rgba: &bytes,
    };
    let mut frame = Frame::new(4, 4, 0);
    frame.caller_clip = Rect::new(0.5, 0.5, 3.0, 2.0);
    let plan = plan_with_images(
        frame,
        &[Command::Image {
            rect: Rect::new(-0.25, -0.25, 4.0, 3.0),
            source: 0,
        }],
        &[source],
    )
    .unwrap();
    let draw = plan.draws[1];
    assert_eq!((draw.x, draw.y, draw.width, draw.height), (1, 1, 3, 2));
    let image = draw.image.unwrap();
    assert_eq!(words(&plan, image.x_base, 3), [1, 2, 3]);
    assert_eq!(words(&plan, image.y_base, 2), [1, 2]);
}

#[test]
fn scaled_lut_thresholds_and_source_validation_remain_unchanged() {
    assert_eq!(table(0, 2, 0.0, f32::from_bits(0x3fff_ffff), 4), [0, 2]);
    assert_eq!(table(0, 2, 0.0, 2.0, 4), [0, 2]);
    assert_eq!(table(0, 3, 0.0, f32::from_bits(0x4000_0001), 4), [0, 1, 3]);
    assert_eq!(table(0, 3, 0.25, 1.5, 4), [0, 2, 3]);
    let short = [0u8; 15];
    assert_eq!(
        validate_source_images(&[SourceImage {
            width: 2,
            height: 2,
            rgba: &short
        }])
        .unwrap_err(),
        "exact source RGBA length"
    );
    assert_eq!(
        validate_source_images(&[SourceImage {
            width: 0,
            height: 1,
            rgba: &[]
        }])
        .unwrap_err(),
        "source dimensions"
    );
}
