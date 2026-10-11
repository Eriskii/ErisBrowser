use super::*;

fn words(plan: &Plan, start: u32, count: u32) -> Vec<u32> {
    plan.input_bytes()[start as usize * 4..(start + count) as usize * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| u32::from_le_bytes(*bytes))
        .collect()
}

#[test]
fn screen_image_ignores_document_and_fixed_offsets_but_obeys_current_clip() {
    let bytes = [255; 32];
    let source = SourceImage {
        width: 4,
        height: 2,
        rgba: &bytes,
    };
    let mut frame = Frame::new(8, 8, 0);
    frame.document_offset = (-100_000.25, 100_000.5);
    frame.viewport_offset = (0.125, -0.375);
    frame.caller_clip = Rect::new(1.5, 2.5, 4.0, 3.0);
    let plan = plan_with_images(
        frame,
        &[
            Command::ScreenImage {
                x: 1,
                y: 2,
                source: 0,
            },
            Command::PushFixed,
            Command::ScreenImage {
                x: 1,
                y: 2,
                source: 0,
            },
            Command::PopFixed,
        ],
        &[source],
    )
    .unwrap();
    assert_eq!(plan.draws.len(), 3);
    for draw in &plan.draws[1..] {
        assert_eq!(draw.bounds(), (2, 3, 3, 1));
        let image = draw.image.unwrap();
        assert_eq!(words(&plan, image.x_base, 3), [1, 2, 3]);
        assert_eq!(words(&plan, image.y_base, 1), [1]);
    }
}

#[test]
fn screen_image_negative_origin_crops_exact_source_indices() {
    let bytes = [255; 48];
    let source = SourceImage {
        width: 4,
        height: 3,
        rgba: &bytes,
    };
    let plan = plan_with_images(
        Frame::new(4, 4, 0),
        &[Command::ScreenImage {
            x: -1,
            y: -1,
            source: 0,
        }],
        &[source],
    )
    .unwrap();
    let draw = plan.draws[1];
    assert_eq!(draw.bounds(), (0, 0, 3, 2));
    let image = draw.image.unwrap();
    assert_eq!(words(&plan, image.x_base, 3), [1, 2, 3]);
    assert_eq!(words(&plan, image.y_base, 2), [1, 2]);
}

#[test]
fn screen_image_validates_hidden_source_ids_and_integer_coordinates() {
    let mut frame = Frame::new(4, 4, 0);
    frame.caller_clip = Rect::new(0.0, 0.0, 0.0, 0.0);
    assert_eq!(
        plan_with_images(
            frame,
            &[Command::ScreenImage {
                x: 0,
                y: 0,
                source: 0
            },],
            &[]
        )
        .unwrap_err(),
        "source ID outside validated table"
    );
    let bytes = [255; 4];
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &bytes,
    };
    for (x, y) in [(i32::MAX, 0), (0, i32::MIN)] {
        assert!(
            plan_with_images(
                frame,
                &[Command::ScreenImage { x, y, source: 0 },],
                &[source]
            )
            .is_err()
        );
    }
}
