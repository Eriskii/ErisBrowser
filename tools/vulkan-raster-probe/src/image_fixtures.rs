//! Fixed opaque-image inputs and independent expected pixels.
//!
//! Transcribed from the frozen oracle (SHA-256
//! ccd79c6ac60221ccaf463ce1ada56230c35e12039d3d8ca534e15019121c2ecf).
//! Expected pixels use literal rows or the direct integer band predicate below;
//! they never call a planner, source-sampling helper, Canvas or GPU code.

use crate::{
    Command, Frame, MAX_SOURCE_ENTRIES, Plan, Rect, Result, SourceImage, plan_with_images,
};

pub struct FixtureSource {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct ImageFixture {
    pub name: &'static str,
    pub frame: Frame,
    pub commands: Vec<Command>,
    pub sources: Vec<FixtureSource>,
    pub expected: Vec<u32>,
}

impl ImageFixture {
    pub fn plan(&self) -> Result<Plan> {
        if self.sources.len() > MAX_SOURCE_ENTRIES {
            return Err("source count budget".into());
        }
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(self.sources.len())
            .map_err(|_| "fixture source-view allocation".to_string())?;
        for source in &self.sources {
            sources.push(SourceImage {
                width: source.width,
                height: source.height,
                rgba: &source.rgba,
            });
        }
        plan_with_images(self.frame, &self.commands, &sources)
    }
}

fn color(value: u8) -> u32 {
    match value {
        b'.' => 0xffffff,
        b'Z' => 0x123456,
        b'A' => 0x010203,
        b'B' => 0xfefdfc,
        b'C' => 0xe01020,
        b'D' => 0x2040c0,
        b'E' => 0x80b020,
        b'F' => 0xa040d0,
        b'G' => 0x1765a9,
        b'H' => 0xb3d507,
        b'I' => 0xd77b23,
        b'J' => 0x2bd79d,
        b'K' => 0x9f35df,
        b'L' => 0x4d591b,
        b'M' => 0xc1a52f,
        b'N' => 0x6b8de3,
        b'O' => 0x53e7c9,
        b'P' => 0xe98347,
        _ => panic!("unknown image fixture color"),
    }
}

fn literal(rows: &[&str]) -> Vec<u32> {
    rows.iter().flat_map(|row| row.bytes()).map(color).collect()
}

// Source RGBA byte arrays are inputs, separate from the expected output rows.
// Special geometry uses exact f32 bit patterns, never decimal approximations.
pub fn fixtures() -> Vec<ImageFixture> {
    // Frozen fixture 01: image-upscale-2x2
    let fixture_01 = ImageFixture {
        name: "image-upscale-2x2",
        frame: Frame {
            width: 6,
            height: 6,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 6.0, 6.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(1.0, 1.0, 4.0, 4.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 2,
            height: 2,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20, 0x40,
                0xc0, 0xff,
            ],
        }],
        expected: literal(&["......", ".AABB.", ".AABB.", ".CCDD.", ".CCDD.", "......"]),
    };

    // Frozen fixture 02: image-three-columns-to-five
    let fixture_02 = ImageFixture {
        name: "image-three-columns-to-five",
        frame: Frame {
            width: 7,
            height: 3,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 7.0, 3.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(1.0, 1.0, 5.0, 1.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 3,
            height: 1,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff,
            ],
        }],
        expected: literal(&[".......", ".AABBC.", "......."]),
    };

    // Frozen fixture 03: image-downscale-distinct-4x4
    let fixture_03 = ImageFixture {
        name: "image-downscale-distinct-4x4",
        frame: Frame {
            width: 4,
            height: 4,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 4.0, 4.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(1.0, 1.0, 2.0, 2.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 4,
            height: 4,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20, 0x40,
                0xc0, 0xff, 0x80, 0xb0, 0x20, 0xff, 0xa0, 0x40, 0xd0, 0xff, 0x17, 0x65, 0xa9, 0xff,
                0xb3, 0xd5, 0x07, 0xff, 0xd7, 0x7b, 0x23, 0xff, 0x2b, 0xd7, 0x9d, 0xff, 0x9f, 0x35,
                0xdf, 0xff, 0x4d, 0x59, 0x1b, 0xff, 0xc1, 0xa5, 0x2f, 0xff, 0x6b, 0x8d, 0xe3, 0xff,
                0x53, 0xe7, 0xc9, 0xff, 0xe9, 0x83, 0x47, 0xff,
            ],
        }],
        expected: literal(&["....", ".AC.", ".IK.", "...."]),
    };

    // Frozen fixture 04: image-clip-preserves-original-origin
    let fixture_04 = ImageFixture {
        name: "image-clip-preserves-original-origin",
        frame: Frame {
            width: 8,
            height: 3,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 8.0, 3.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::PushClip(Rect::new(3.0, 1.0, 4.0, 1.0)),
            Command::Image {
                rect: Rect::new(1.0, 1.0, 6.0, 1.0),
                source: 0,
            },
            Command::PopClip,
        ],
        sources: vec![FixtureSource {
            width: 3,
            height: 1,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff,
            ],
        }],
        expected: literal(&["........", "...BBCC.", "........"]),
    };

    // Frozen fixture 05: image-negative-document-offset
    let fixture_05 = ImageFixture {
        name: "image-negative-document-offset",
        frame: Frame {
            width: 5,
            height: 3,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 5.0, 3.0),
            document_offset: (-2.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(0.0, 0.0, 6.0, 2.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 3,
            height: 2,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20, 0x40,
                0xc0, 0xff, 0x80, 0xb0, 0x20, 0xff, 0xa0, 0x40, 0xd0, 0xff,
            ],
        }],
        expected: literal(&["BBCC.", "EEFF.", "....."]),
    };

    // Frozen fixture 06: image-negative-fractional-origin
    let fixture_06 = ImageFixture {
        name: "image-negative-fractional-origin",
        frame: Frame {
            width: 4,
            height: 4,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 4.0, 4.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(-0.5, -0.5, 3.0, 3.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 2,
            height: 2,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20, 0x40,
                0xc0, 0xff,
            ],
        }],
        expected: literal(&["ABB.", "CDD.", "CDD.", "...."]),
    };

    // Frozen fixture 07: image-tiny-and-fractional-clip
    let fixture_07 = ImageFixture {
        name: "image-tiny-and-fractional-clip",
        frame: Frame {
            width: 6,
            height: 4,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 6.0, 4.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::Image {
                rect: Rect::new(0.25, 0.25, 0.125, 0.125),
                source: 0,
            },
            Command::PushClip(Rect::new(0.25, 0.25, 1.5, 1.5)),
            Command::Image {
                rect: Rect::new(0.25, 0.25, 0.125, 0.125),
                source: 0,
            },
            Command::Image {
                rect: Rect::new(0.25, 0.25, 1.5, 1.5),
                source: 1,
            },
            Command::PopClip,
        ],
        sources: vec![
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0x01, 0x02, 0x03, 0xff],
            },
            FixtureSource {
                width: 4,
                height: 4,
                rgba: vec![
                    0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20,
                    0x40, 0xc0, 0xff, 0x80, 0xb0, 0x20, 0xff, 0xa0, 0x40, 0xd0, 0xff, 0x17, 0x65,
                    0xa9, 0xff, 0xb3, 0xd5, 0x07, 0xff, 0xd7, 0x7b, 0x23, 0xff, 0x2b, 0xd7, 0x9d,
                    0xff, 0x9f, 0x35, 0xdf, 0xff, 0x4d, 0x59, 0x1b, 0xff, 0xc1, 0xa5, 0x2f, 0xff,
                    0x6b, 0x8d, 0xe3, 0xff, 0x53, 0xe7, 0xc9, 0xff, 0xe9, 0x83, 0x47, 0xff,
                ],
            },
        ],
        expected: literal(&["A.....", ".K....", "......", "......"]),
    };

    // Frozen fixture 08: image-exact-and-one-ulp-translated-edges
    let fixture_08 = ImageFixture {
        name: "image-exact-and-one-ulp-translated-edges",
        frame: Frame {
            width: 5,
            height: 3,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 5.0, 3.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::Image {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                source: 0,
            },
            Command::Image {
                rect: Rect::new(
                    2.0,
                    0.0,
                    f32::from_bits(0x3f80_0002),
                    f32::from_bits(0x3f80_0001),
                ),
                source: 0,
            },
        ],
        sources: vec![FixtureSource {
            width: 2,
            height: 2,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20, 0x40,
                0xc0, 0xff,
            ],
        }],
        expected: literal(&["A.AB.", "..CD.", "....."]),
    };

    // Frozen fixture 09: image-f32-endpoint-addition-rounds-back
    let fixture_09 = ImageFixture {
        name: "image-f32-endpoint-addition-rounds-back",
        frame: Frame {
            width: 4,
            height: 2,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 4.0, 2.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(2.0, 0.0, f32::from_bits(0x3f80_0001), 1.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 2,
            height: 1,
            rgba: vec![0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff],
        }],
        expected: literal(&["..A.", "...."]),
    };

    // Frozen fixture 10: image-rectangle-order-and-source-reuse
    let fixture_10 = ImageFixture {
        name: "image-rectangle-order-and-source-reuse",
        frame: Frame {
            width: 6,
            height: 4,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 6.0, 4.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::Image {
                rect: Rect::new(0.0, 0.0, 6.0, 4.0),
                source: 0,
            },
            Command::Rect {
                rect: Rect::new(1.0, 1.0, 4.0, 2.0),
                rgba: [128, 176, 32, 255],
                radius: 0.0,
            },
            Command::Image {
                rect: Rect::new(2.0, 0.0, 2.0, 3.0),
                source: 1,
            },
            Command::Image {
                rect: Rect::new(0.0, 3.0, 4.0, 1.0),
                source: 0,
            },
        ],
        sources: vec![
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff],
            },
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0xe0, 0x10, 0x20, 0xff, 0x20, 0x40, 0xc0, 0xff],
            },
        ],
        expected: literal(&["AACDBB", "AECDEB", "AECDEB", "AABBBB"]),
    };

    // Frozen fixture 11: image-fixed-escape-and-nested-restoration
    let fixture_11 = ImageFixture {
        name: "image-fixed-escape-and-nested-restoration",
        frame: Frame {
            width: 8,
            height: 5,
            clear: 0xffffff,
            caller_clip: Rect::new(1.0, 1.0, 6.0, 3.0),
            document_offset: (-2.0, -1.0),
            viewport_offset: (1.0, 1.0),
        },
        commands: vec![
            Command::PushClip(Rect::new(20.0, 20.0, 1.0, 1.0)),
            Command::Image {
                rect: Rect::new(2.0, 1.0, 8.0, 5.0),
                source: 4,
            },
            Command::PushFixed,
            Command::Image {
                rect: Rect::new(0.0, 0.0, 4.0, 2.0),
                source: 0,
            },
            Command::PushClip(Rect::new(3.0, 1.0, 1.0, 1.0)),
            Command::Image {
                rect: Rect::new(-1.0, -1.0, 8.0, 6.0),
                source: 1,
            },
            Command::PushFixed,
            Command::Image {
                rect: Rect::new(5.0, 2.0, 1.0, 1.0),
                source: 2,
            },
            Command::PopFixed,
            Command::Image {
                rect: Rect::new(-1.0, -1.0, 8.0, 6.0),
                source: 3,
            },
            Command::PopClip,
            Command::PopFixed,
            Command::Image {
                rect: Rect::new(2.0, 1.0, 8.0, 5.0),
                source: 4,
            },
            Command::PopClip,
            Command::Image {
                rect: Rect::new(3.0, 4.0, 2.0, 1.0),
                source: 5,
            },
        ],
        sources: vec![
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff],
            },
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0x17, 0x65, 0xa9, 0xff, 0xb3, 0xd5, 0x07, 0xff],
            },
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0x80, 0xb0, 0x20, 0xff],
            },
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0xd7, 0x7b, 0x23, 0xff, 0x2b, 0xd7, 0x9d, 0xff],
            },
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0x9f, 0x35, 0xdf, 0xff],
            },
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0xe0, 0x10, 0x20, 0xff, 0x20, 0x40, 0xc0, 0xff],
            },
        ],
        expected: literal(&["........", ".AABB...", ".AABJ...", ".CD...E.", "........"]),
    };

    // Frozen fixture 12: image-dispatch-tail-and-clear-border
    let fixture_12 = ImageFixture {
        name: "image-dispatch-tail-and-clear-border",
        frame: Frame {
            width: 320,
            height: 240,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 320.0, 240.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![Command::Image {
            rect: Rect::new(0.0, 0.0, 319.0, 239.0),
            source: 0,
        }],
        sources: vec![FixtureSource {
            width: 2,
            height: 2,
            rgba: vec![
                0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff, 0xe0, 0x10, 0x20, 0xff, 0x20, 0x40,
                0xc0, 0xff,
            ],
        }],
        // Independently authored integer bands; no source sampling or LUTs.
        expected: (0..240)
            .flat_map(|y| {
                (0..320).map(move |x| {
                    if x == 319 || y == 239 {
                        0xffffff
                    } else if y < 120 && x < 160 {
                        0x010203
                    } else if y < 120 {
                        0xfefdfc
                    } else if x < 160 {
                        0xe01020
                    } else {
                        0x2040c0
                    }
                })
            })
            .collect(),
    };

    // Frozen fixture 13: image-valid-empty-and-hidden
    let fixture_13 = ImageFixture {
        name: "image-valid-empty-and-hidden",
        frame: Frame {
            width: 4,
            height: 3,
            clear: 0x123456,
            caller_clip: Rect::new(0.0, 0.0, 4.0, 3.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::Image {
                rect: Rect::new(0.0, 0.0, 0.0, 2.0),
                source: 0,
            },
            Command::Image {
                rect: Rect::new(0.0, 0.0, -1.0, 2.0),
                source: 0,
            },
            Command::Image {
                rect: Rect::new(8.0, 8.0, 1.0, 1.0),
                source: 0,
            },
            Command::PushClip(Rect::new(8.0, 8.0, 1.0, 1.0)),
            Command::Image {
                rect: Rect::new(0.0, 0.0, 4.0, 3.0),
                source: 0,
            },
            Command::PopClip,
        ],
        sources: vec![FixtureSource {
            width: 1,
            height: 1,
            rgba: vec![0x01, 0x02, 0x03, 0xff],
        }],
        expected: literal(&["ZZZZ", "ZZZZ", "ZZZZ"]),
    };

    // Frozen fixture 14: image-subnormal-origin-and-lost-extent
    let fixture_14 = ImageFixture {
        name: "image-subnormal-origin-and-lost-extent",
        frame: Frame {
            width: 2,
            height: 2,
            clear: 0xffffff,
            caller_clip: Rect::new(0.0, 0.0, 2.0, 2.0),
            document_offset: (0.0, 0.0),
            viewport_offset: (0.0, 0.0),
        },
        commands: vec![
            Command::Image {
                rect: Rect::new(
                    f32::from_bits(0x8000_0001),
                    0.0,
                    f32::from_bits(0x0000_0002),
                    1.0,
                ),
                source: 0,
            },
            Command::Image {
                rect: Rect::new(1.0, 1.0, f32::from_bits(0x0000_0001), 1.0),
                source: 1,
            },
        ],
        sources: vec![
            FixtureSource {
                width: 2,
                height: 1,
                rgba: vec![0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0xff],
            },
            FixtureSource {
                width: 1,
                height: 1,
                rgba: vec![0xe0, 0x10, 0x20, 0xff],
            },
        ],
        expected: literal(&["B.", ".."]),
    };

    vec![
        fixture_01, fixture_02, fixture_03, fixture_04, fixture_05, fixture_06, fixture_07,
        fixture_08, fixture_09, fixture_10, fixture_11, fixture_12, fixture_13, fixture_14,
    ]
}
