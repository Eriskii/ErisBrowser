//! Nine independently frozen opaque-target source-over alpha fixtures.
//!
//! Literal source/geometry/RGB expectations: SHA-256
//! a72c5dfc2331b61da90a4efc852592180eeee90021e8f267fab7dbe838aafe33.
//! Manual counters: SHA-256
//! 8bad259e2b0c7f2139a0089170279b928f6724c45b607a9ce6987ea748b3028f.
//! Expected pixels and counters never call a blend, sampling, planning,
//! Canvas or GPU helper. Alpha-zero rectangles remain in the inputs:
//! validate their geometry, then omit their draw. Reached images still
//! dispatch when their source alpha is zero. Clear is always opaque.

use crate::image_fixtures::{FixtureSource, ImageFixture};
use crate::{Command, Frame, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpectedCounters {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub draws: usize,
    pub invocations: u64,
    pub gpu_buffer_bytes: u64,
    pub compared_bytes: u64,
}

// Transcribed independently counted protocol tuples, including clear.
pub const EXPECTED_COUNTERS: [ExpectedCounters; 9] = [
    ExpectedCounters {
        name: "alpha-rectangle-six-boundaries",
        width: 6,
        height: 2,
        draws: 12,
        invocations: 768,
        gpu_buffer_bytes: 3168,
        compared_bytes: 48,
    },
    ExpectedCounters {
        name: "alpha-image-six-boundaries",
        width: 6,
        height: 2,
        draws: 3,
        invocations: 192,
        gpu_buffer_bytes: 944,
        compared_bytes: 48,
    },
    ExpectedCounters {
        name: "alpha-ordered-mixed-draws",
        width: 4,
        height: 1,
        draws: 10,
        invocations: 640,
        gpu_buffer_bytes: 2644,
        compared_bytes: 16,
    },
    ExpectedCounters {
        name: "alpha-one-repeated-rounding",
        width: 7,
        height: 1,
        draws: 65,
        invocations: 4160,
        gpu_buffer_bytes: 16696,
        compared_bytes: 28,
    },
    ExpectedCounters {
        name: "alpha-subpixel-and-fractional-clip",
        width: 4,
        height: 3,
        draws: 4,
        invocations: 256,
        gpu_buffer_bytes: 1132,
        compared_bytes: 48,
    },
    ExpectedCounters {
        name: "alpha-image-negative-fractional-original-origin",
        width: 5,
        height: 3,
        draws: 2,
        invocations: 128,
        gpu_buffer_bytes: 672,
        compared_bytes: 60,
    },
    ExpectedCounters {
        name: "alpha-fixed-reset-and-restoration",
        width: 5,
        height: 3,
        draws: 6,
        invocations: 384,
        gpu_buffer_bytes: 1680,
        compared_bytes: 60,
    },
    ExpectedCounters {
        name: "alpha-transparent-hidden-rgb",
        width: 2,
        height: 2,
        draws: 2,
        invocations: 128,
        gpu_buffer_bytes: 576,
        compared_bytes: 16,
    },
    ExpectedCounters {
        name: "alpha-empty-clip-still-clears-opaque-target",
        width: 3,
        height: 2,
        draws: 1,
        invocations: 64,
        gpu_buffer_bytes: 304,
        compared_bytes: 24,
    },
];

fn alpha_rect(rect: Rect, rgba: [u8; 4]) -> Command {
    Command::Rect {
        rect,
        rgba,
        radius: 0.0,
    }
}

pub fn fixtures() -> Vec<ImageFixture> {
    // Frozen fixture 01: alpha-rectangle-six-boundaries
    let fixture_01 = ImageFixture {
        name: "alpha-rectangle-six-boundaries",
        frame: Frame {
            width: 6,
            height: 2,
            clear: 0x00ff11,
            caller_clip: Rect::new(0.0, 0.0, 6.0, 2.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            alpha_rect(Rect::new(0.0, 1.0, 6.0, 1.0), [10, 20, 30, 255]),
            alpha_rect(Rect::new(0.0, 0.0, 1.0, 1.0), [255, 0, 129, 0]),
            alpha_rect(Rect::new(1.0, 0.0, 1.0, 1.0), [255, 0, 129, 1]),
            alpha_rect(Rect::new(2.0, 0.0, 1.0, 1.0), [255, 0, 129, 127]),
            alpha_rect(Rect::new(3.0, 0.0, 1.0, 1.0), [255, 0, 129, 128]),
            alpha_rect(Rect::new(4.0, 0.0, 1.0, 1.0), [255, 0, 129, 254]),
            alpha_rect(Rect::new(5.0, 0.0, 1.0, 1.0), [255, 0, 129, 255]),
            alpha_rect(Rect::new(0.0, 1.0, 1.0, 1.0), [200, 40, 90, 0]),
            alpha_rect(Rect::new(1.0, 1.0, 1.0, 1.0), [200, 40, 90, 1]),
            alpha_rect(Rect::new(2.0, 1.0, 1.0, 1.0), [200, 40, 90, 127]),
            alpha_rect(Rect::new(3.0, 1.0, 1.0, 1.0), [200, 40, 90, 128]),
            alpha_rect(Rect::new(4.0, 1.0, 1.0, 1.0), [200, 40, 90, 254]),
            alpha_rect(Rect::new(5.0, 1.0, 1.0, 1.0), [200, 40, 90, 255]),
        ],
        sources: vec![],
        expected: vec![
            0x00ff11, 0x01fe11, 0x7f8049, 0x807f49, 0xfe0181, 0xff0081, 0x0a141e, 0x0b141e,
            0x691e3c, 0x691e3c, 0xc7285a, 0xc8285a,
        ],
    };

    // Frozen fixture 02: alpha-image-six-boundaries
    let fixture_02 = ImageFixture {
        name: "alpha-image-six-boundaries",
        frame: Frame {
            width: 6,
            height: 2,
            clear: 0x00ff11,
            caller_clip: Rect::new(0.0, 0.0, 6.0, 2.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            alpha_rect(Rect::new(0.0, 1.0, 6.0, 1.0), [10, 20, 30, 255]),
            Command::Image {
                rect: Rect::new(0.0, 0.0, 6.0, 2.0),
                source: 0,
            },
        ],
        sources: vec![FixtureSource {
            width: 6,
            height: 2,
            rgba: vec![
                0xff, 0x00, 0x81, 0x00, 0xff, 0x00, 0x81, 0x01, 0xff, 0x00, 0x81, 0x7f, 0xff, 0x00,
                0x81, 0x80, 0xff, 0x00, 0x81, 0xfe, 0xff, 0x00, 0x81, 0xff, 0xc8, 0x28, 0x5a, 0x00,
                0xc8, 0x28, 0x5a, 0x01, 0xc8, 0x28, 0x5a, 0x7f, 0xc8, 0x28, 0x5a, 0x80, 0xc8, 0x28,
                0x5a, 0xfe, 0xc8, 0x28, 0x5a, 0xff,
            ],
        }],
        expected: vec![
            0x00ff11, 0x01fe11, 0x7f8049, 0x807f49, 0xfe0181, 0xff0081, 0x0a141e, 0x0b141e,
            0x691e3c, 0x691e3c, 0xc7285a, 0xc8285a,
        ],
    };

    // Frozen fixture 03: alpha-ordered-mixed-draws
    let fixture_03 = ImageFixture {
        name: "alpha-ordered-mixed-draws",
        frame: Frame {
            width: 4,
            height: 1,
            clear: 0x000000,
            caller_clip: Rect::new(0.0, 0.0, 4.0, 1.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            alpha_rect(Rect::new(0.0, 0.0, 1.0, 1.0), [255, 0, 0, 128]),
            Command::Image {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                source: 0,
            },
            Command::Image {
                rect: Rect::new(1.0, 0.0, 1.0, 1.0),
                source: 0,
            },
            alpha_rect(Rect::new(1.0, 0.0, 1.0, 1.0), [255, 0, 0, 128]),
            alpha_rect(Rect::new(2.0, 0.0, 1.0, 1.0), [255, 0, 0, 128]),
            Command::Image {
                rect: Rect::new(2.0, 0.0, 1.0, 1.0),
                source: 1,
            },
            alpha_rect(Rect::new(2.0, 0.0, 1.0, 1.0), [255, 0, 0, 127]),
            alpha_rect(Rect::new(2.0, 0.0, 1.0, 1.0), [0, 255, 0, 0]),
            Command::Image {
                rect: Rect::new(3.0, 0.0, 1.0, 1.0),
                source: 2,
            },
            Command::Image {
                rect: Rect::new(3.0, 0.0, 1.0, 1.0),
                source: 2,
            },
        ],
        sources: vec![
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0x00, 0x00, 0xff, 0x80],
            },
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0x00, 0x00, 0xff, 0xff],
            },
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0xff, 0xff, 0xff, 0x80],
            },
        ],
        expected: vec![0x400080, 0x800040, 0x7f0080, 0xc0c0c0],
    };

    // Frozen fixture 04: alpha-one-repeated-rounding
    let fixture_04 = ImageFixture {
        name: "alpha-one-repeated-rounding",
        frame: Frame {
            width: 7,
            height: 1,
            clear: 0x000000,
            caller_clip: Rect::new(0.0, 0.0, 7.0, 1.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            alpha_rect(Rect::new(0.0, 0.0, 7.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(1.0, 0.0, 6.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(2.0, 0.0, 5.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(2.0, 0.0, 5.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(3.0, 0.0, 4.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(3.0, 0.0, 4.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(3.0, 0.0, 4.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(3.0, 0.0, 4.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(4.0, 0.0, 3.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(5.0, 0.0, 2.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
            alpha_rect(Rect::new(6.0, 0.0, 1.0, 1.0), [255, 255, 255, 1]),
        ],
        sources: vec![],
        expected: vec![
            0x010101, 0x020202, 0x040404, 0x080808, 0x101010, 0x202020, 0x404040,
        ],
    };

    // Frozen fixture 05: alpha-subpixel-and-fractional-clip
    let fixture_05 = ImageFixture {
        name: "alpha-subpixel-and-fractional-clip",
        frame: Frame {
            width: 4,
            height: 3,
            clear: 0x000000,
            caller_clip: Rect::new(0.0, 0.0, 4.0, 3.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            alpha_rect(Rect::new(0.25, 0.25, 0.125, 0.125), [255, 0, 0, 128]),
            Command::PushClip(Rect::new(0.25, 0.25, 1.5, 1.5)),
            alpha_rect(Rect::new(0.25, 0.25, 0.125, 0.125), [0, 0, 255, 128]),
            alpha_rect(Rect::new(0.0, 0.0, 4.0, 3.0), [0, 0, 255, 128]),
            Command::Image {
                rect: Rect::new(0.0, 0.0, 4.0, 3.0),
                source: 0,
            },
            Command::PopClip,
        ],
        sources: vec![FixtureSource {
            width: 1,
            height: 1,
            rgba: vec![0x00, 0xff, 0x00, 0x7f],
        }],
        expected: vec![
            0x800000, 0x000000, 0x000000, 0x000000, 0x000000, 0x007f40, 0x000000, 0x000000,
            0x000000, 0x000000, 0x000000, 0x000000,
        ],
    };

    // Frozen fixture 06: alpha-image-negative-fractional-original-origin
    let fixture_06 = ImageFixture {
        name: "alpha-image-negative-fractional-original-origin",
        frame: Frame {
            width: 5,
            height: 3,
            clear: 0x000000,
            caller_clip: Rect::new(0.0, 0.0, 5.0, 3.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::PushClip(Rect::new(0.25, 0.25, 3.5, 1.5)),
            Command::Image {
                rect: Rect::new(-0.5, 0.25, 4.5, 2.0),
                source: 0,
            },
            Command::PopClip,
        ],
        sources: vec![FixtureSource {
            width: 3,
            height: 2,
            rgba: vec![
                0xff, 0x00, 0x00, 0x80, 0x00, 0xff, 0x00, 0x7f, 0x00, 0x00, 0xff, 0xfe, 0xff, 0xff,
                0x00, 0x01, 0xff, 0x00, 0xff, 0x00, 0xff, 0xff, 0xff, 0xff,
            ],
        }],
        expected: vec![
            0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x007f00, 0x007f00,
            0x0000fe, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
        ],
    };

    // Frozen fixture 07: alpha-fixed-reset-and-restoration
    let fixture_07 = ImageFixture {
        name: "alpha-fixed-reset-and-restoration",
        frame: Frame {
            width: 5,
            height: 3,
            clear: 0x000000,
            caller_clip: Rect::new(1.0, 0.0, 4.0, 3.0),
            document_offset: (-1.0, 0.0),
            viewport_offset: (1.0, 0.0),
        },
        commands: vec![
            Command::PushClip(Rect::new(20.0, 20.0, 1.0, 1.0)),
            alpha_rect(Rect::new(1.0, 0.0, 5.0, 3.0), [255, 0, 0, 128]),
            Command::PushFixed,
            alpha_rect(Rect::new(0.0, 0.0, 2.0, 2.0), [255, 0, 0, 128]),
            Command::PushClip(Rect::new(1.0, 1.0, 1.0, 1.0)),
            Command::Image {
                rect: Rect::new(0.0, 0.0, 3.0, 2.0),
                source: 0,
            },
            Command::PushFixed,
            alpha_rect(Rect::new(3.0, 2.0, 1.0, 1.0), [0, 255, 0, 127]),
            Command::PopFixed,
            alpha_rect(Rect::new(0.0, 0.0, 3.0, 2.0), [0, 255, 0, 127]),
            Command::PopClip,
            Command::PopFixed,
            alpha_rect(Rect::new(1.0, 0.0, 5.0, 3.0), [255, 255, 255, 255]),
            Command::PopClip,
            Command::Image {
                rect: Rect::new(2.0, 2.0, 1.0, 1.0),
                source: 1,
            },
        ],
        sources: vec![
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0x00, 0x00, 0xff, 0x80],
            },
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0xff, 0xff, 0xff, 0x01],
            },
        ],
        expected: vec![
            0x000000, 0x800000, 0x800000, 0x000000, 0x000000, 0x000000, 0x800000, 0x207f40,
            0x000000, 0x000000, 0x000000, 0x010101, 0x000000, 0x000000, 0x007f00,
        ],
    };

    // Frozen fixture 08: alpha-transparent-hidden-rgb
    let fixture_08 = ImageFixture {
        name: "alpha-transparent-hidden-rgb",
        frame: Frame {
            width: 2,
            height: 2,
            clear: 0x123456,
            caller_clip: Rect::new(0.0, 0.0, 2.0, 2.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::Image {
                rect: Rect::new(0.0, 0.0, 2.0, 2.0),
                source: 0,
            },
            alpha_rect(Rect::new(0.0, 0.0, 2.0, 2.0), [101, 202, 77, 0]),
        ],
        sources: vec![FixtureSource {
            width: 2,
            height: 2,
            rgba: vec![
                0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0xff, 0xff,
                0xff, 0x00,
            ],
        }],
        expected: vec![0x123456, 0x123456, 0x123456, 0x123456],
    };

    // Frozen fixture 09: alpha-empty-clip-still-clears-opaque-target
    let fixture_09 = ImageFixture {
        name: "alpha-empty-clip-still-clears-opaque-target",
        frame: Frame {
            width: 3,
            height: 2,
            clear: 0x123456,
            caller_clip: Rect::new(10.0, 10.0, 1.0, 1.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            alpha_rect(Rect::new(0.0, 0.0, 3.0, 2.0), [255, 0, 0, 0]),
            Command::Image {
                rect: Rect::new(0.0, 0.0, 3.0, 2.0),
                source: 0,
            },
            Command::PushClip(Rect::new(0.0, 0.0, 3.0, 2.0)),
            Command::PopClip,
        ],
        sources: vec![FixtureSource {
            width: 1,
            height: 1,
            rgba: vec![0x09, 0x08, 0x07, 0x00],
        }],
        expected: vec![0x123456, 0x123456, 0x123456, 0x123456, 0x123456, 0x123456],
    };

    vec![
        fixture_01, fixture_02, fixture_03, fixture_04, fixture_05, fixture_06, fixture_07,
        fixture_08, fixture_09,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MAX_COMMANDS, MAX_COORDINATE, MAX_GPU_BUFFER_BYTES, MAX_HEIGHT, MAX_INVOCATIONS,
        MAX_SCOPES, MAX_SOURCE_ENTRIES, MAX_SOURCE_RGBA_BYTES, MAX_WIDTH, PARAM_STRIDE,
    };

    fn bounded_rect(rect: Rect) {
        assert!(
            [rect.x, rect.y, rect.width, rect.height]
                .into_iter()
                .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
        );
    }

    // Structural checks only: no candidate planner, blend or sampling oracle.
    #[test]
    fn alpha_fixture_inputs_and_literal_outputs_are_bounded() {
        let fixtures = fixtures();
        assert_eq!(fixtures.len(), EXPECTED_COUNTERS.len());
        let mut names = std::collections::BTreeSet::new();
        let mut total_pixels = 0;
        let mut total_commands = 0;
        let mut total_source_bytes = 0;
        for (fixture, counter) in fixtures.iter().zip(EXPECTED_COUNTERS) {
            assert!(names.insert(fixture.name));
            assert_eq!(fixture.name, counter.name);
            assert_eq!(
                (fixture.frame.width, fixture.frame.height),
                (counter.width, counter.height)
            );
            assert!((1..=MAX_WIDTH).contains(&fixture.frame.width));
            assert!((1..=MAX_HEIGHT).contains(&fixture.frame.height));
            bounded_rect(fixture.frame.caller_clip);
            for offset in [fixture.frame.document_offset, fixture.frame.viewport_offset] {
                assert!(
                    [offset.0, offset.1]
                        .into_iter()
                        .all(|v| v.is_finite() && v.abs() <= MAX_COORDINATE)
                );
            }
            assert!(fixture.frame.clear <= 0xffffff);
            assert_eq!(
                fixture.expected.len(),
                (counter.width * counter.height) as usize
            );
            assert!(fixture.expected.iter().all(|pixel| *pixel <= 0xffffff));
            assert!(fixture.commands.len() <= MAX_COMMANDS);
            assert!(fixture.sources.len() <= MAX_SOURCE_ENTRIES);
            let mut source_bytes = 0;
            for source in &fixture.sources {
                assert!(source.width > 0 && source.height > 0);
                assert_eq!(
                    source.rgba.len() as u64,
                    u64::from(source.width) * u64::from(source.height) * 4
                );
                source_bytes += source.rgba.len();
            }
            assert!(source_bytes <= MAX_SOURCE_RGBA_BYTES);
            let mut scopes = Vec::new();
            for command in &fixture.commands {
                match command {
                    Command::Rect { rect, radius, .. } => {
                        bounded_rect(*rect);
                        assert_eq!(*radius, 0.0);
                    }
                    Command::Image { rect, source } => {
                        bounded_rect(*rect);
                        assert!((*source as usize) < fixture.sources.len());
                    }
                    Command::PushClip(rect) => {
                        bounded_rect(*rect);
                        scopes.push(false);
                    }
                    Command::PushFixed => scopes.push(true),
                    Command::PopClip => assert_eq!(scopes.pop(), Some(false)),
                    Command::PopFixed => assert_eq!(scopes.pop(), Some(true)),
                    Command::Unsupported(_) => panic!("unsupported command in positive fixture"),
                }
                assert!(scopes.len() <= MAX_SCOPES);
            }
            assert!(scopes.is_empty());
            total_pixels += fixture.expected.len();
            total_commands += fixture.commands.len();
            total_source_bytes += source_bytes;
        }
        assert_eq!(total_pixels, 87);
        assert_eq!(total_commands, 119);
        assert_eq!(total_source_bytes, 116);
        // Independent rectangle/image inputs share the same frozen literal
        // output at all six alpha boundaries; no expected pixels are derived.
        assert_eq!(fixtures[0].expected, fixtures[1].expected);
    }

    #[test]
    fn alpha_fixture_frozen_counter_arithmetic() {
        // Hand-counted visible-command counts and arena sizes from the
        // separate frozen audit, not from interpreting command geometry.
        let visible_commands = [11, 2, 9, 64, 3, 1, 5, 1, 0];
        let packed_source_bytes = [0u64, 48, 12, 0, 4, 24, 8, 16, 0];
        let lookup_words = [0u64, 8, 10, 0, 2, 4, 4, 4, 0];
        let mut draws = 0;
        let mut invocations = 0;
        let mut bytes = 0;
        let mut compared = 0;
        for (index, counter) in EXPECTED_COUNTERS.iter().enumerate() {
            assert_eq!(counter.draws, 1 + visible_commands[index]);
            assert!(counter.width <= 8 && counter.height <= 8);
            assert_eq!(counter.invocations, counter.draws as u64 * 64);
            assert_eq!(
                counter.compared_bytes,
                u64::from(counter.width) * u64::from(counter.height) * 4
            );
            let arena_bytes = packed_source_bytes[index] + lookup_words[index] * 4;
            assert_eq!(
                counter.gpu_buffer_bytes,
                counter.compared_bytes * 2
                    + counter.draws as u64 * PARAM_STRIDE as u64
                    + arena_bytes
            );
            assert!(counter.invocations <= MAX_INVOCATIONS);
            assert!(counter.gpu_buffer_bytes <= MAX_GPU_BUFFER_BYTES);
            draws += counter.draws;
            invocations += counter.invocations;
            bytes += counter.gpu_buffer_bytes;
            compared += counter.compared_bytes;
        }
        assert_eq!(draws, 105);
        assert_eq!(invocations, 6720);
        assert_eq!(bytes, 27816);
        assert_eq!(compared, 348);
    }
}
