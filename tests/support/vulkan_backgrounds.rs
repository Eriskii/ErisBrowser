//! Literal raster-background fixtures shared by integration tests and the GPU probe.
use eris::graphics::{
    Canvas, Color, DrawCommand, Fonts, ImageStore, RasterImage, Rect,
    raster_bridge::native::{NativePhase, NativeScenePlan, plan_native_scene},
};
use eris_raster_core::{DrawKind, Frame, PARAM_STRIDE, Plan, Profile};
use std::sync::Arc;

pub const FIXTURE_NAMES: [&str; 6] = [
    "background-four-repeat-modes",
    "background-negative-fractional-tile",
    "background-fractional-clip-scale",
    "background-alpha-nested-opacity",
    "background-document-fixed-restoration",
    "background-center-versus-old-image",
];
pub const LITERAL_PIXELS: usize = 222;
pub const CONVERTED_FRAMES: usize = 12;
pub const COMPARED_BYTES: u64 = 1776;

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn rgb(value: u32) -> Color {
    Color::rgba((value >> 16) as u8, (value >> 8) as u8, value as u8, 255)
}
fn background(rect: Rect, tile: Rect, repeat: [bool; 2], key: &str) -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect,
        tile,
        repeat,
        radius: 0.0,
        key: key.into(),
    }
}
fn source(width: u32, height: u32, rgba: &[u8]) -> Arc<RasterImage> {
    assert_eq!(rgba.len(), (width * height * 4) as usize);
    Arc::new(RasterImage {
        width,
        height,
        rgba: rgba.to_vec(),
    })
}
fn images() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert(
        "grid".into(),
        source(
            2,
            2,
            &[
                255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255,
            ],
        ),
    );
    images.insert(
        "clip".into(),
        source(
            3,
            2,
            &[
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 255, 255, 255, 255, 0, 255, 255,
                255, 255, 0, 255,
            ],
        ),
    );
    images.insert("rb".into(), source(2, 1, &[255, 0, 0, 255, 0, 0, 255, 255]));
    images.insert(
        "alpha".into(),
        source(2, 1, &[255, 0, 0, 128, 0, 0, 255, 0]),
    );
    images
}
fn rows(values: &[&str]) -> Vec<u32> {
    assert!(!values.is_empty());
    assert!(values.iter().all(|row| row.len() == values[0].len()));
    values
        .iter()
        .flat_map(|row| row.bytes())
        .map(|pixel| match pixel {
            b'.' => 0xffffff,
            b'R' => 0xff0000,
            b'G' => 0x00ff00,
            b'B' => 0x0000ff,
            b'Y' => 0xffff00,
            b'C' => 0x00ffff,
            b'M' => 0xff00ff,
            b'K' => 0x000000,
            b'P' => 0xff7f7f,
            b'Q' => 0xffbfbf,
            b'D' => 0xffdfdf,
            b'S' => 0xff8080,
            b'T' => 0x8080ff,
            b'U' => 0xbfbfff,
            _ => panic!("unknown literal color"),
        })
        .collect()
}

pub struct BackgroundFixture {
    pub name: &'static str,
    pub frame: Frame,
    pub commands: Vec<DrawCommand>,
    pub images: ImageStore,
    pub expected: Vec<u32>,
}
impl BackgroundFixture {
    pub fn canvas_pixels(&self, fonts: &Fonts) -> Vec<u32> {
        assert_eq!(
            self.expected.len(),
            (self.frame.width * self.frame.height) as usize
        );
        let mut canvas = Canvas::new(self.frame.width, self.frame.height).unwrap();
        canvas.clear(rgb(self.frame.clear));
        let c = self.frame.caller_clip;
        canvas.set_clip(rect(c.x, c.y, c.width, c.height));
        canvas.paint_with_viewport(
            &self.commands,
            fonts,
            &self.images,
            self.frame.document_offset,
            self.frame.viewport_offset,
        );
        assert!(!canvas.exhausted(), "{} exhausted Canvas", self.name);
        canvas.pixels
    }
    pub fn native(&self, fonts: &Fonts) -> Result<NativeScenePlan, String> {
        // Target is the whole framebuffer. Offsets/clip belong only to the phase.
        plan_native_scene(
            Frame::new(self.frame.width, self.frame.height, self.frame.clear),
            &[NativePhase {
                frame: self.frame,
                commands: &self.commands,
            }],
            &self.images,
            fonts,
        )
        .map_err(|error| error.to_string())
    }
}

pub fn fixtures() -> Vec<BackgroundFixture> {
    let mut commands = Vec::new();
    for (band, repeat) in [[false, false], [true, false], [false, true], [true, true]]
        .into_iter()
        .enumerate()
    {
        let y = (band * 4) as f32;
        commands.push(background(
            rect(0.0, y, 8.0, 4.0),
            rect(2.0, y + 1.0, 4.0, 2.0),
            repeat,
            "grid",
        ));
    }
    let repeated = BackgroundFixture {
        name: FIXTURE_NAMES[0],
        frame: Frame::new(8, 16, 0xffffff),
        commands,
        images: images(),
        // Four independent 8x4 bands: no-repeat, repeat-x, repeat-y, repeat.
        expected: rows(&[
            "........", "..RRBB..", "..GGYY..", "........", "........", "BBRRBBRR", "YYGGYYGG",
            "........", "..GGYY..", "..RRBB..", "..GGYY..", "..RRBB..", "YYGGYYGG", "BBRRBBRR",
            "YYGGYYGG", "BBRRBBRR",
        ]),
    };
    let negative = BackgroundFixture {
        name: FIXTURE_NAMES[1],
        frame: Frame::new(4, 6, 0xffffff),
        images: images(),
        commands: vec![
            background(
                rect(0.0, 0.0, 4.0, 3.0),
                rect(-0.75, -0.25, 2.0, 2.0),
                [true, true],
                "grid",
            ),
            background(
                rect(0.0, 3.0, 4.0, 3.0),
                rect(-0.75, 2.75, 2.0, 2.0),
                [false, false],
                "grid",
            ),
        ],
        expected: rows(&["BRBR", "YGYG", "BRBR", "B...", "Y...", "...."]),
    };
    let mut clipped_frame = Frame::new(6, 4, 0xffffff);
    clipped_frame.caller_clip = eris_raster_core::Rect::new(1.25, 0.25, 3.5, 2.5);
    let clipped = BackgroundFixture {
        name: FIXTURE_NAMES[2],
        frame: clipped_frame,
        images: images(),
        commands: vec![background(
            rect(0.0, 0.0, 6.0, 4.0),
            rect(0.0, 0.0, 6.0, 4.0),
            [false, false],
            "clip",
        )],
        expected: rows(&["......", "..GGB.", "..MMY.", "......"]),
    };
    let mut commands = Vec::new();
    for (row, key, depth) in [
        (0, "alpha", 0),
        (1, "alpha", 1),
        (2, "alpha", 2),
        (3, "rb", 1),
        (4, "rb", 2),
    ] {
        for _ in 0..depth {
            commands.push(DrawCommand::PushOpacity { opacity: 0.5 });
        }
        commands.push(background(
            rect(0.0, row as f32, 4.0, 1.0),
            rect(0.0, row as f32, 2.0, 1.0),
            [true, true],
            key,
        ));
        for _ in 0..depth {
            commands.push(DrawCommand::PopOpacity);
        }
    }
    let alpha = BackgroundFixture {
        name: FIXTURE_NAMES[3],
        frame: Frame::new(4, 5, 0xffffff),
        images: images(),
        commands,
        // Alpha128, then .5 and .5*.5; opaque sources under one/two .5 groups.
        expected: rows(&["P.P.", "Q.Q.", "D.D.", "STST", "QUQU"]),
    };
    let mut fixed_frame = Frame::new(6, 3, 0xffffff);
    fixed_frame.caller_clip = eris_raster_core::Rect::new(1.0, 0.0, 4.0, 3.0);
    fixed_frame.document_offset = (1.0, 1.0);
    fixed_frame.viewport_offset = (0.0, 0.0);
    let fixed = BackgroundFixture {
        name: FIXTURE_NAMES[4],
        frame: fixed_frame,
        images: images(),
        commands: vec![
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 1.0, 1.0),
            },
            DrawCommand::PushFixed,
            background(
                rect(0.0, 0.0, 4.0, 1.0),
                rect(0.5, 0.0, 2.0, 1.0),
                [false, false],
                "rb",
            ),
            DrawCommand::PopFixed,
            background(
                rect(0.0, 0.0, 4.0, 1.0),
                rect(0.0, 0.0, 2.0, 1.0),
                [true, true],
                "rb",
            ),
            DrawCommand::PopClip,
            background(
                rect(0.0, 1.0, 4.0, 1.0),
                rect(0.0, 1.0, 2.0, 1.0),
                [true, true],
                "rb",
            ),
        ],
        expected: rows(&[".B....", ".R....", ".RBRB."]),
    };
    let identity = BackgroundFixture {
        name: FIXTURE_NAMES[5],
        frame: Frame::new(4, 2, 0xffffff),
        images: images(),
        commands: vec![
            DrawCommand::Image {
                rect: rect(0.5, 0.0, 2.0, 1.0),
                key: "rb".into(),
            },
            background(
                rect(0.0, 1.0, 4.0, 1.0),
                rect(0.5, 1.0, 2.0, 1.0),
                [false, false],
                "rb",
            ),
        ],
        expected: rows(&["RRB.", "RB.."]),
    };
    vec![repeated, negative, clipped, alpha, fixed, identity]
}

// Test-only interpretation copied from the established opacity witness.
// Literal fixture rows are independent; this inspects packed records only.
fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
fn parameter(plan: &Plan, draw: usize, index: usize) -> u32 {
    word(&plan.parameters()[draw * PARAM_STRIDE..], index)
}
pub fn packed_pixels(plan: &Plan) -> Vec<u32> {
    assert_eq!(plan.profile(), Profile::Native);
    let frame = plan.frame();
    let mut pixels = vec![0; frame.width as usize * frame.height as usize];
    let mut scratch = vec![[0u64; 4]; plan.group_scratch_bytes() as usize / 8];
    assert_eq!(plan.group_scratch_bytes() % 8, 0);
    for (draw_index, draw) in plan.draws().iter().enumerate() {
        let p = |field| parameter(plan, draw_index, field);
        let index = |base: usize, x: u32, y: u32| {
            assert_eq!(p(base) % 2, 0);
            assert!(x >= p(base + 1) && y >= p(base + 2));
            assert!(x - p(base + 1) < p(base + 3));
            assert!(y - p(base + 2) < p(base + 4));
            (p(base) / 2 + (y - p(base + 2)) * p(base + 3) + x - p(base + 1)) as usize
        };
        let group = draw.target_is_group();
        assert_eq!(group, p(21) == 1);
        let (x0, y0, width, height) = draw.bounds();
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let root_index = (y * frame.width + x) as usize;
                let destination = if group {
                    scratch[index(16, x, y)]
                } else {
                    let rgb = pixels[root_index];
                    [
                        u64::from(rgb & 255) * 257,
                        u64::from((rgb >> 8) & 255) * 257,
                        u64::from((rgb >> 16) & 255) * 257,
                        65535,
                    ]
                };
                let next = match draw.kind() {
                    DrawKind::GroupClear => {
                        assert!(group);
                        [0; 4]
                    }
                    DrawKind::GroupComposite => {
                        let source = scratch[index(24, x, y)];
                        assert!(source.iter().all(|&channel| channel <= 65535));
                        assert!(source[..3].iter().all(|&channel| channel <= source[3]));
                        let opacity = match p(30) {
                            0 => {
                                let k = p(29);
                                assert!((1..256).contains(&k));
                                assert_eq!(p(31), 0, "grid ABI keeps reserved bits zero");
                                k as f32 / 256.0
                            }
                            1 => {
                                assert_eq!(p(29), 0, "raw opacity has no grid numerator");
                                let value = f32::from_bits(p(31));
                                assert!(value.is_finite() && value > 0.0 && value < 1.0);
                                value
                            }
                            flag => panic!("unknown opacity route {flag}"),
                        };
                        // Independent oracle: preserve Canvas's separate f32
                        // division, multiplication, subtraction and pop stages.
                        // Do not reproduce the shader's integer emulation here.
                        let inverse = 1.0 - source[3] as f32 / 65535.0 * opacity;
                        let mut next = [0; 4];
                        for (channel, next) in next.iter_mut().enumerate() {
                            let value = source[channel] as f32 * opacity
                                + destination[channel] as f32 * inverse;
                            *next = if group {
                                value.round() as u64
                            } else {
                                (value / 257.0).round() as u64 * 257
                            };
                            assert!(*next <= 65535);
                        }
                        next
                    }
                    kind => {
                        let input = plan.input_bytes();
                        let (source, alpha) = match kind {
                            DrawKind::Rectangle => (p(6), p(6) >> 24),
                            DrawKind::Image => {
                                let sx = word(input, (p(12) + x - x0) as usize);
                                let sy = word(input, (p(13) + y - y0) as usize);
                                let color = word(input, (p(8) + sy * p(9) + sx) as usize);
                                (color, color >> 24)
                            }
                            DrawKind::Glyph => {
                                let sy = i64::from(y) - i64::from(p(13) as i32);
                                if sy < 0 || sy >= i64::from(p(10)) {
                                    continue;
                                }
                                let row_x = word(input, p(12) as usize + sy as usize) as i32;
                                let sx = i64::from(x) - i64::from(row_x);
                                if sx < 0 || sx >= i64::from(p(9)) {
                                    continue;
                                }
                                let coverage = word(
                                    input,
                                    p(8) as usize + sy as usize * p(9) as usize + sx as usize,
                                );
                                assert!(coverage <= 255);
                                (p(6), (p(6) >> 24) * coverage / 255)
                            }
                            _ => unreachable!(),
                        };
                        let alpha = u64::from(alpha);
                        let mut next = [0; 4];
                        if group {
                            let alpha16 = alpha * 257;
                            for (channel, next) in next.iter_mut().enumerate().take(3) {
                                let color = u64::from((source >> (channel * 8)) & 255);
                                *next = (color * alpha16 + 127) / 255
                                    + (destination[channel] * (65535 - alpha16) + 32767) / 65535;
                            }
                            next[3] =
                                alpha16 + (destination[3] * (65535 - alpha16) + 32767) / 65535;
                        } else {
                            for (channel, next) in next.iter_mut().enumerate().take(3) {
                                let color = u64::from((source >> (channel * 8)) & 255);
                                *next = ((color * alpha
                                    + (destination[channel] / 257) * (255 - alpha)
                                    + 127)
                                    / 255)
                                    * 257;
                            }
                            next[3] = 65535;
                        }
                        next
                    }
                };
                if group {
                    scratch[index(16, x, y)] = next;
                } else {
                    pixels[root_index] =
                        ((next[2] / 257) << 16 | (next[1] / 257) << 8 | (next[0] / 257)) as u32;
                }
            }
        }
    }
    pixels
}
