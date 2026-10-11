//! Literal image identity regressions, separate from the frozen image fixtures.
//!
//! Source bytes and expected pixels are authored independently. No expectation
//! calls a planner, image sampler, Canvas, or GPU implementation.

use crate::{
    Command, Frame, Rect,
    image_fixtures::{FixtureSource, ImageFixture},
    rect,
};

fn color(symbol: u8) -> u32 {
    match symbol {
        b'.' => 0xffffff,
        b'R' => 0xff0000,
        b'G' => 0x00ff00,
        b'B' => 0x0000ff,
        b'Y' => 0xffff00,
        b'C' => 0x00ffff,
        b'M' => 0xff00ff,
        b'O' => 0xff8000,
        b'K' => 0x000000,
        b'A' => 0x010203,
        b'D' => 0x2040c0,
        _ => panic!("unknown identity fixture color"),
    }
}

fn rows(values: &[&str]) -> Vec<u32> {
    values
        .iter()
        .flat_map(|row| row.bytes())
        .map(color)
        .collect()
}

pub fn fixtures() -> Vec<ImageFixture> {
    // Source125 must remain destination125 on both exact240-pixel axes.
    // Input construction writes whole literal stripes; the separate expected
    // predicate expresses their final integer-coordinate intersection.
    let mut cross = [128, 128, 128, 255].repeat(240 * 240);
    for x in 0..240 {
        cross[(125 * 240 + x) * 4..(125 * 240 + x + 1) * 4].copy_from_slice(&[255, 0, 0, 255]);
    }
    for y in 0..240 {
        cross[(y * 240 + 125) * 4..(y * 240 + 126) * 4].copy_from_slice(&[0, 255, 0, 255]);
    }
    cross[(125 * 240 + 125) * 4..(125 * 240 + 126) * 4].copy_from_slice(&[0, 0, 255, 255]);
    let exact = ImageFixture {
        name: "image-identity-240-cross-125",
        frame: Frame::new(240, 240, 0xffffff),
        commands: vec![Command::Image {
            rect: Rect::new(0.0, 0.0, 240.0, 240.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 240,
            height: 240,
            rgba: cross,
        }],
        expected: (0..240)
            .flat_map(|y| {
                (0..240).map(move |x| match (x, y) {
                    (125, 125) => 0x0000ff,
                    (125, _) => 0x00ff00,
                    (_, 125) => 0xff0000,
                    _ => 0x808080,
                })
            })
            .collect(),
    };

    // The clip exposes source x125..133 at framebuffer x5..13. Its fractional
    // lower edge rejects y2; the fractional image origin selects source rows
    // 1,2,3 at y3,4,5. Clipping must not reset the original image origin.
    let mut clipped_source = Vec::new();
    for background in [
        [128, 128, 128, 255],
        [0, 255, 255, 255],
        [255, 0, 255, 255],
        [255, 128, 0, 255],
    ] {
        clipped_source.extend_from_slice(&background.repeat(240));
    }
    for y in 0..4 {
        clipped_source[(y * 240 + 125) * 4..(y * 240 + 129) * 4].copy_from_slice(&[
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ]);
    }
    let clipped = ImageFixture {
        name: "image-identity-fractional-clip-original-origin",
        frame: Frame::new(20, 8, 0xffffff),
        commands: vec![
            Command::PushClip(Rect::new(4.5, 2.5, 9.0, 3.0)),
            Command::Image {
                rect: Rect::new(-120.0, 1.5, 240.0, 4.0),
                source: 0,
            },
            Command::PopClip,
            rect(16.0, 6.0, 2.0, 1.0, 0x000000),
        ],
        sources: vec![FixtureSource {
            width: 240,
            height: 4,
            rgba: clipped_source,
        }],
        expected: rows(&[
            "....................",
            "....................",
            "....................",
            ".....RGBYCCCCC......",
            ".....RGBYMMMMM......",
            ".....RGBYOOOOO......",
            "................KK..",
            "....................",
        ]),
    };

    // Only Y is identity: source width2 still upscales to destination width4.
    // Source row125 stays framebuffer row125; horizontal white one-pixel
    // borders surround the ordinary repeated AADD pattern. Keep the frame
    // within the existing Probe240-pixel height cap.
    let mut vertical_source = [1, 2, 3, 255, 32, 64, 192, 255].repeat(240);
    vertical_source[125 * 8..126 * 8].copy_from_slice(&[255, 0, 0, 255, 0, 255, 0, 255]);
    let vertical = ImageFixture {
        name: "image-identity-y-only-with-x-upscale",
        frame: Frame::new(6, 240, 0xffffff),
        commands: vec![Command::Image {
            rect: Rect::new(1.0, 0.0, 4.0, 240.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 2,
            height: 240,
            rgba: vertical_source,
        }],
        expected: (0..240)
            .flat_map(|y| {
                let row = match y {
                    125 => ".RRGG.",
                    _ => ".AADD.",
                };
                row.bytes().map(color)
            })
            .collect(),
    };

    vec![exact, clipped, vertical]
}
