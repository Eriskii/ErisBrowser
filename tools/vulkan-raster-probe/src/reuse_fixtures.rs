//! Ordered native frames with literal, independently specified pixel references.
//! The first three frames deliberately have equal buffer sizes but different
//! clear colors, uniforms, image bytes, glyph coverage and row origins.
use eris_raster_core::{
    Command, Frame, Plan, Profile, Rect, Result, SourceImage, SourceMask,
    plan_with_masks_for_profile, rect,
};

pub struct Fixture {
    pub name: &'static str,
    pub plan: Plan,
    pub expected: Vec<u32>,
}

fn inputs(variant: usize) -> Result<Fixture> {
    let (name, clear, rectangle, image, coverage, origins, glyph_y, glyph_color, expected) =
        match variant {
            0 => (
                "inputs-a",
                0xffffff,
                rect(0.0, 0.0, 1.0, 1.0, 0xff0000),
                [0, 255, 0, 255, 0, 0, 255, 255],
                [255, 0],
                [1],
                1,
                [0, 0, 0, 128],
                vec![
                    0xff0000, 0x00ff00, 0x0000ff, 0xffffff, 0xffffff, 0x7f7f7f, 0xffffff, 0xffffff,
                    0xffffff, 0xffffff, 0xffffff, 0xffffff,
                ],
            ),
            1 => (
                "inputs-b",
                0x123456,
                Command::Rect {
                    rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                    rgba: [0, 0, 255, 128],
                    radius: 0.0,
                },
                [0xab, 0xcd, 0xef, 255, 0x10, 0x20, 0x30, 0],
                [0, 255],
                [1],
                1,
                [255, 0, 0, 255],
                vec![
                    0x091aab, 0xabcdef, 0x123456, 0x123456, 0x123456, 0x123456, 0xff0000, 0x123456,
                    0x123456, 0x123456, 0x123456, 0x123456,
                ],
            ),
            2 => (
                "inputs-c",
                0x010203,
                rect(3.0, 2.0, 1.0, 1.0, 0x987654),
                [0, 255, 255, 255, 0, 0, 0, 255],
                [255, 255],
                [0],
                1,
                [255, 255, 255, 255],
                vec![
                    0x010203, 0x00ffff, 0x000000, 0x010203, 0xffffff, 0xffffff, 0x010203, 0x010203,
                    0x010203, 0x010203, 0x010203, 0x987654,
                ],
            ),
            _ => return Err("reuse fixture variant".into()),
        };
    let commands = [
        rectangle,
        Command::Image {
            rect: Rect::new(1.0, 0.0, 2.0, 1.0),
            source: 0,
        },
        Command::Glyph {
            source: 0,
            rows: 0,
            y: glyph_y,
            rgba: glyph_color,
        },
    ];
    let plan = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(4, 3, clear),
        &commands,
        &[SourceImage {
            width: 2,
            height: 1,
            rgba: &image,
        }],
        &[SourceMask {
            width: 2,
            height: 1,
            coverage: &coverage,
        }],
        &[&origins],
    )?;
    Ok(Fixture {
        name,
        plan,
        expected,
    })
}

fn simple(
    name: &'static str,
    width: u32,
    height: u32,
    clear: u32,
    commands: &[Command],
    expected: Vec<u32>,
) -> Result<Fixture> {
    Ok(Fixture {
        name,
        plan: plan_with_masks_for_profile(
            Profile::Native,
            Frame::new(width, height, clear),
            commands,
            &[],
            &[],
            &[],
        )?,
        expected,
    })
}

pub fn fixtures() -> Result<Vec<Fixture>> {
    Ok(vec![
        inputs(0)?,
        inputs(1)?,
        inputs(2)?,
        simple(
            "rectangle-a",
            4,
            3,
            0x204060,
            &[rect(0.0, 0.0, 1.0, 1.0, 0xffffff)],
            vec![
                0xffffff, 0x204060, 0x204060, 0x204060, 0x204060, 0x204060, 0x204060, 0x204060,
                0x204060, 0x204060, 0x204060, 0x204060,
            ],
        )?,
        simple(
            "rectangle-b",
            4,
            3,
            0xa0b0c0,
            &[rect(3.0, 2.0, 1.0, 1.0, 0x102030)],
            vec![
                0xa0b0c0, 0xa0b0c0, 0xa0b0c0, 0xa0b0c0, 0xa0b0c0, 0xa0b0c0, 0xa0b0c0, 0xa0b0c0,
                0xa0b0c0, 0xa0b0c0, 0xa0b0c0, 0x102030,
            ],
        )?,
        simple("clear-a", 4, 3, 0xabcdef, &[], vec![0xabcdef; 12])?,
        // Entirely clipped commands must not preserve old target contents.
        simple(
            "hidden-clear",
            4,
            3,
            0x000000,
            &[rect(9.0, 9.0, 1.0, 1.0, 0xffffff)],
            vec![0x000000; 12],
        )?,
        simple("resize-width", 5, 3, 0x123456, &[], vec![0x123456; 15])?,
        simple("resize-height", 5, 4, 0x654321, &[], vec![0x654321; 20])?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_input_frames_have_equal_allocations_and_different_contents() {
        let cases = fixtures().unwrap();
        for case in &cases {
            assert_eq!(
                case.expected.len() as u32,
                case.plan.frame().width * case.plan.frame().height
            );
        }
        for pair in cases[..3].windows(2) {
            let (a, b) = (&pair[0].plan, &pair[1].plan);
            assert_eq!(a.gpu_buffer_bytes(), b.gpu_buffer_bytes());
            assert_eq!(a.input_bytes().len(), b.input_bytes().len());
            assert_eq!(a.parameters().len(), b.parameters().len());
            assert_ne!(a.input_bytes(), b.input_bytes());
            assert_ne!(a.parameters(), b.parameters());
            assert_ne!(pair[0].expected, pair[1].expected);
        }
        assert!(!cases[3].plan.has_input());
        assert_eq!(
            cases[5].plan.gpu_buffer_bytes(),
            cases[6].plan.gpu_buffer_bytes()
        );
        assert_eq!(cases[5].plan.draws().len(), cases[6].plan.draws().len());
    }
}
