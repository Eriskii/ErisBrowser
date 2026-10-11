//! Independent opaque-gradient literals shared by CPU integration and GPU probe.
use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore, Rect,
        gradient::{
            GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
        },
        raster_bridge::native::{NativePhase, NativeScenePlan, plan_native_scene},
    },
    layout::{HitAction, LayoutResult},
};
use eris_raster_core::{DrawKind, Frame, PARAM_STRIDE, Plan, Profile};
use std::sync::Arc;

pub const HTML: &str = include_str!("../../examples/vulkan-gradients.html");
pub const WIDTH: u32 = 96;
pub const HEIGHT: u32 = 64;
pub const HIT: (f32, f32) = (30.0, 14.0);

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn color(rgb: u32) -> Color {
    Color {
        r: (rgb >> 16) as u8,
        g: (rgb >> 8) as u8,
        b: rgb as u8,
        a: 255,
    }
}
fn stop(rgb: u32, position: Option<GradientPosition>) -> GradientStop {
    GradientStop {
        color: GradientColor::Color(color(rgb)),
        position,
    }
}
fn hard(direction: GradientDirection) -> LinearGradient {
    let middle = Some(GradientPosition {
        px: 0.0,
        percent: 50.0,
    });
    LinearGradient {
        direction,
        stops: Arc::from([stop(0xff0000, middle), stop(0x0000ff, middle)]),
    }
}
fn gradient(
    rect: Rect,
    tile: Rect,
    source: LinearGradient,
    current_color: Color,
    stop_scale: f32,
) -> DrawCommand {
    DrawCommand::LinearGradient {
        repeat: [true, true],
        rect,
        tile,
        radius: 0.0,
        gradient: Arc::new(source),
        current_color,
        stop_scale,
    }
}
fn rectangle(rect: Rect, rgb: u32) -> DrawCommand {
    DrawCommand::Rect {
        rect,
        color: color(rgb),
        radius: 0.0,
    }
}
fn rows(values: &[&str]) -> Vec<u32> {
    values
        .iter()
        .flat_map(|row| row.bytes())
        .map(|c| match c {
            b'.' => 0xffffff,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            b'G' => 0x00ff00,
            b'K' => 0x000000,
            b'M' => 0xff00ff,
            _ => panic!("unknown literal pixel"),
        })
        .collect()
}

pub struct GradientFixture {
    pub name: &'static str,
    pub frame: Frame,
    pub commands: Vec<DrawCommand>,
    pub expected: Vec<u32>,
}
impl GradientFixture {
    pub fn canvas_pixels(&self, fonts: &Fonts) -> Vec<u32> {
        let mut canvas = Canvas::new(self.frame.width, self.frame.height).unwrap();
        canvas.clear(color(self.frame.clear));
        let c = self.frame.caller_clip;
        canvas.set_clip(rect(c.x, c.y, c.width, c.height));
        canvas.paint_with_viewport(
            &self.commands,
            fonts,
            &ImageStore::new(),
            self.frame.document_offset,
            self.frame.viewport_offset,
        );
        assert!(!canvas.exhausted());
        canvas.pixels
    }
    pub fn native(&self, fonts: &Fonts) -> Result<NativeScenePlan, String> {
        plan_native_scene(
            Frame::new(self.frame.width, self.frame.height, self.frame.clear),
            &[NativePhase {
                frame: self.frame,
                commands: &self.commands,
            }],
            &ImageStore::new(),
            fonts,
        )
        .map_err(|error| error.to_string())
    }
}

pub fn fixtures() -> Vec<GradientFixture> {
    let ramp_rect = rect(0.0, 0.0, 4.0, 1.0);
    let px_rect = rect(0.0, 1.0, 4.0, 1.0);
    let current_rect = rect(0.0, 2.0, 4.0, 1.0);
    let px = Some(GradientPosition {
        px: 1.0,
        percent: 0.0,
    });
    let middle = Some(GradientPosition {
        px: 0.0,
        percent: 50.0,
    });
    let ramp = GradientFixture {
        name: "gradient-ramp-px-scale-current-color",
        frame: Frame::new(4, 3, 0xffffff),
        commands: vec![
            gradient(
                ramp_rect,
                ramp_rect,
                LinearGradient {
                    direction: GradientDirection::Angle(90.0),
                    stops: Arc::from([stop(0x000000, None), stop(0xf8f8f8, None)]),
                },
                color(0xff00ff),
                1.0,
            ),
            gradient(
                px_rect,
                px_rect,
                LinearGradient {
                    direction: GradientDirection::Angle(90.0),
                    stops: Arc::from([stop(0xff0000, px), stop(0x0000ff, px)]),
                },
                color(0xff00ff),
                2.0,
            ),
            gradient(
                current_rect,
                current_rect,
                LinearGradient {
                    direction: GradientDirection::Angle(90.0),
                    stops: Arc::from([
                        GradientStop {
                            color: GradientColor::CurrentColor,
                            position: middle,
                        },
                        stop(0x0000ff, middle),
                    ]),
                },
                color(0x00ff00),
                1.0,
            ),
        ],
        // Literal pixel centers 1/8,3/8,5/8,7/8 of 248, then two hard edges.
        expected: vec![
            0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff,
            0x00ff00, 0x00ff00, 0x0000ff, 0x0000ff,
        ],
    };
    let mut repeated_frame = Frame::new(8, 4, 0xffffff);
    repeated_frame.document_offset = (0.25, -0.75);
    let repeated = GradientFixture {
        name: "gradient-repeated-corner-fractional-clip",
        frame: repeated_frame,
        commands: vec![
            DrawCommand::PushClip {
                rect: rect(1.0, 1.0, 5.5, 3.5),
            },
            gradient(
                rect(0.5, 1.0, 8.0, 4.0),
                rect(0.0, 1.0, 4.0, 2.0),
                hard(GradientDirection::Corner {
                    right: true,
                    bottom: false,
                }),
                color(0xff00ff),
                1.0,
            ),
            DrawCommand::PopClip,
            rectangle(rect(0.75, 3.75, 1.0, 1.0), 0x000000),
        ],
        // Local corner edge x/4=y/2; no admitted center lies on that edge.
        expected: rows(&["........", "..RBRRR.", "..BBRBB.", ".KRBRRR."]),
    };
    let mut fixed_frame = Frame::new(8, 4, 0xffffff);
    fixed_frame.document_offset = (0.25, -0.75);
    fixed_frame.viewport_offset = (0.5, 0.5);
    let fixed = GradientFixture {
        name: "gradient-document-fixed-fractional-restoration",
        frame: fixed_frame,
        commands: vec![
            DrawCommand::PushClip {
                rect: rect(0.0, 1.0, 2.0, 2.0),
            },
            gradient(
                rect(0.0, 1.0, 4.0, 2.0),
                rect(0.0, 1.0, 4.0, 2.0),
                hard(GradientDirection::Angle(90.0)),
                color(0xff00ff),
                1.0,
            ),
            DrawCommand::PushFixed,
            gradient(
                rect(3.0, 0.0, 2.0, 2.0),
                rect(3.0, 0.0, 2.0, 2.0),
                LinearGradient {
                    direction: GradientDirection::Angle(180.0),
                    stops: Arc::from([GradientStop {
                        color: GradientColor::CurrentColor,
                        position: None,
                    }]),
                },
                color(0x00ff00),
                1.0,
            ),
            DrawCommand::PopFixed,
            rectangle(rect(0.75, 1.75, 1.0, 1.0), 0x000000),
            DrawCommand::PopClip,
            rectangle(rect(6.75, 3.75, 1.0, 1.0), 0xff00ff),
        ],
        // Fixed .5 origins cover the original floor/ceil loop (three pixels).
        expected: rows(&["...GGG..", ".KBGGG..", ".RBGGG..", ".......M"]),
    };
    vec![ramp, repeated, fixed]
}

fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}

/// CPU inspection of opaque packed records, not GPU or acquired-surface work.
pub fn packed_pixels(plan: &Plan) -> Vec<u32> {
    assert_eq!(plan.profile(), Profile::Native);
    let frame = plan.frame();
    let mut pixels = vec![0; (frame.width * frame.height) as usize];
    for (index, draw) in plan.draws().iter().enumerate() {
        assert!(!draw.target_is_group());
        let p = |field| {
            word(
                &plan.parameters()[index * PARAM_STRIDE..(index + 1) * PARAM_STRIDE],
                field,
            )
        };
        let (x0, y0, width, height) = draw.bounds();
        assert!(x0 + width <= frame.width && y0 + height <= frame.height);
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let rgba = match draw.kind() {
                    DrawKind::Rectangle => p(6),
                    DrawKind::Image => {
                        let sx = word(plan.input_bytes(), (p(12) + x - x0) as usize);
                        let sy = word(plan.input_bytes(), (p(13) + y - y0) as usize);
                        assert!(sx < p(9) && sy < p(10));
                        word(plan.input_bytes(), (p(8) + sy * p(9) + sx) as usize)
                    }
                    _ => panic!("opaque witness only accepts rectangle/image records"),
                };
                assert_eq!(rgba >> 24, 255);
                pixels[(y * frame.width + x) as usize] = rgba & 0xffffff;
            }
        }
    }
    pixels
}

pub fn metadata(phase: usize) -> &'static str {
    ["Native gradients ready", "Native gradients changed"][phase]
}
type Hit = (NodeId, HitAction, f32, f32, f32, f32, bool);
pub struct State {
    pub click: NodeId,
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    ids: Vec<NodeId>,
    hits: Vec<Hit>,
    pixels: Vec<u32>,
}
pub fn expected_page_pixels(phase: usize) -> Vec<u32> {
    assert!(phase < 2);
    let mut result = vec![0xffffff; (WIDTH * HEIGHT) as usize];
    for y in 8..16 {
        result[y * WIDTH as usize + 8..y * WIDTH as usize + 12]
            .copy_from_slice(&[0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9]);
    }
    for y in 8..24 {
        result[y * WIDTH as usize + 24..y * WIDTH as usize + 40].fill(if phase == 0 {
            0xff0000
        } else {
            0x00ff00
        });
        result[y * WIDTH as usize + 40..y * WIDTH as usize + 56].fill(if phase == 0 {
            0x0000ff
        } else {
            0xffff00
        });
        result[y * WIDTH as usize + 64..y * WIDTH as usize + 80].fill(0xff00ff);
    }
    result
}
pub fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&State>,
    expect_native: bool,
) -> State {
    assert!(phase < 2);
    assert_eq!(previous.is_some(), phase == 1);
    assert!(images.is_empty());
    let body = doc.query_selector("body").unwrap();
    assert_eq!(
        doc.attr(body, "data-phase"),
        Some(["ready", "changed"][phase])
    );
    let title = doc.query_selector("#heading").unwrap();
    assert_eq!(doc.nodes[title].children.len(), 1);
    let title_text = doc.nodes[title].children[0];
    let NodeKind::Text(text) = &doc.nodes[title_text].kind else {
        panic!("retained title text")
    };
    assert_eq!(text.scalar(), Some(metadata(phase)));
    let mut ids = vec![doc.root, body, title, title_text];
    for name in ["board", "ramp", "switch", "fixed"] {
        let id = doc.query_selector(&format!("#{name}")).unwrap();
        assert!(!ids.contains(&id));
        ids.push(id);
    }
    let named_boxes = [
        (0.0, 0.0, 96.0, 64.0, false),
        (8.0, 8.0, 4.0, 8.0, false),
        (24.0, 8.0, 32.0, 16.0, false),
        (64.0, 8.0, 16.0, 16.0, true),
    ];
    for (&node, expected) in ids[4..].iter().zip(named_boxes) {
        let hits = layout
            .hit_regions
            .iter()
            .filter(|hit| hit.node == node)
            .collect::<Vec<_>>();
        assert_eq!(hits.len(), 1);
        let hit = hits[0];
        assert_eq!(hit.action, HitAction::Node);
        assert_eq!(
            (
                hit.rect.x,
                hit.rect.y,
                hit.rect.width,
                hit.rect.height,
                hit.fixed
            ),
            expected
        );
    }
    let hits = layout
        .hit_regions
        .iter()
        .map(|hit| {
            (
                hit.node,
                hit.action,
                hit.rect.x,
                hit.rect.y,
                hit.rect.width,
                hit.rect.height,
                hit.fixed,
            )
        })
        .collect::<Vec<_>>();
    let click = ids[6];
    assert_eq!(layout.hit_test(HIT.0, HIT.1), Some(click));
    assert_eq!(
        layout
            .commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::LinearGradient { .. }))
            .count(),
        3
    );
    let expected = expected_page_pixels(phase);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    let frame = Frame::new(WIDTH, HEIGHT, 0xffffff);
    let native = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &layout.commands,
        }],
        images,
        fonts,
    );
    if expect_native {
        let native = native.expect("opaque square-corner Page gradients admit Native");
        assert_eq!(packed_pixels(native.plan()), expected);
    } else {
        assert_eq!(native.unwrap_err().category(), "unsupported-gradient");
    }
    let graph = doc
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect::<Vec<_>>();
    if let Some(before) = previous {
        assert_eq!(graph, before.graph);
        assert_eq!(ids, before.ids);
        assert_eq!(hits, before.hits);
        assert_eq!(click, before.click);
        assert_eq!(
            before
                .pixels
                .iter()
                .zip(&expected)
                .filter(|(a, b)| a != b)
                .count(),
            512
        );
    }
    State {
        click,
        graph,
        ids,
        hits,
        pixels: expected,
    }
}
