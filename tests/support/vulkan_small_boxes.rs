use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore,
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    layout::LayoutResult,
};
use eris_raster_core::{DrawKind, Frame, PARAM_STRIDE, Plan, Profile};

pub const HTML: &str = include_str!("../../examples/vulkan-small-boxes.html");
pub const WIDTH: u32 = 512;
pub const HEIGHT: u32 = 512;
pub const HIT: (f32, f32) = (20.0, 20.0);
const COLORS: [u32; 8] = [
    0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0x00ffff, 0xff00ff, 0x804000, 0x408080,
];

pub fn metadata(phase: usize) -> &'static str {
    ["Small boxes ready", "Small boxes changed"][phase]
}

pub struct State {
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    ids: Vec<NodeId>,
    pixels: Vec<u32>,
}

pub fn expected_pixels(phase: usize) -> Vec<u32> {
    assert!(phase < 2);
    let mut pixels = vec![0xffffff; 512 * 512];
    for row in 0..5 {
        for (column, color) in COLORS.iter().enumerate() {
            let color = if phase == 1 && row == 0 && column == 0 {
                0x0000ff
            } else {
                *color
            };
            for y in 16 + row * 20..28 + row * 20 {
                pixels[y * 512 + 16 + column * 28..y * 512 + 36 + column * 28].fill(color);
            }
        }
    }
    pixels
}

// This witness deliberately accepts only opaque Rectangle plan records. It
// checks packed positions/order/color against the literal scene and Canvas;
// it is not shader execution or a graphics-driver claim.
fn decode_rectangles(plan: &Plan) -> Vec<u32> {
    let mut pixels = vec![0; (WIDTH * HEIGHT) as usize];
    for (index, draw) in plan.draws().iter().enumerate() {
        assert_eq!(draw.kind(), DrawKind::Rectangle);
        assert!(!draw.target_is_group());
        let offset = index * PARAM_STRIDE + 6 * 4;
        let color = u32::from_le_bytes(plan.parameters()[offset..offset + 4].try_into().unwrap());
        assert_eq!(color >> 24, 255);
        let (x, y, width, height) = draw.bounds();
        assert!(x + width <= WIDTH && y + height <= HEIGHT);
        for row in y..y + height {
            let start = (row * WIDTH + x) as usize;
            pixels[start..start + width as usize].fill(color & 0xffffff);
        }
    }
    pixels
}

pub fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&State>,
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
    let NodeKind::Text(data) = &doc.nodes[title_text].kind else {
        panic!("retained title Text")
    };
    assert_eq!(data.scalar(), Some(metadata(phase)));
    let mut ids = vec![doc.root, body, title, title_text];
    for index in 0..40 {
        let id = doc.query_selector(&format!("#box{index:02}")).unwrap();
        assert_eq!(doc.nodes[id].parent, Some(body));
        assert!(doc.nodes[id].children.is_empty());
        assert!(!ids.contains(&id));
        ids.push(id);
    }
    assert_eq!(ids.len(), 44);
    assert_eq!(layout.hit_test(HIT.0, HIT.1), Some(ids[4]));
    assert!(layout.content_height <= HEIGHT as f32);
    assert_eq!(layout.commands.len(), 40);
    for (index, command) in layout.commands.iter().enumerate() {
        let DrawCommand::Rect {
            rect,
            color,
            radius,
        } = command
        else {
            panic!("forty visible Rect primitives")
        };
        let column = index % 8;
        let row = index / 8;
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (
                (16 + column * 28) as f32,
                (16 + row * 20) as f32,
                20.0,
                12.0
            )
        );
        assert_eq!(*radius, 0.0);
        let expected = if index == 0 && phase == 1 {
            0x0000ff
        } else {
            COLORS[column]
        };
        assert_eq!(
            (color.r, color.g, color.b, color.a),
            (
                (expected >> 16) as u8,
                (expected >> 8) as u8,
                expected as u8,
                255
            )
        );
    }
    let frame = Frame::new(WIDTH, HEIGHT, 0xffffff);
    let native = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &layout.commands,
        }],
        images,
        fonts,
    )
    .expect("actual loaded small boxes admit the Native plan");
    assert_eq!(native.stats().phases, 1);
    assert_eq!(native.stats().bridge.original_commands, 40);
    assert_eq!(native.stats().bridge.lowered_commands, 40);
    assert_eq!(native.stats().cpu_pixel_upper_bound, 9600);
    assert_eq!(native.stats().text.occurrences, 0);
    assert_eq!(native.stats().rounded_masks, 0);
    assert_eq!(native.stats().mask_sources, 0);
    assert_eq!(native.stats().row_entries, 0);
    let plan = native.plan();
    assert_eq!(plan.profile(), Profile::Native);
    assert_eq!(plan.draws().len(), 41);
    assert_eq!(plan.group_scratch_bytes(), 0);
    assert_eq!(plan.invocations(), 539_648);
    assert_eq!(plan.gpu_buffer_bytes(), 2_107_664);
    assert!(native.stats().gpu_buffer_upper_bound >= plan.gpu_buffer_bytes());
    let expected = expected_pixels(phase);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    assert_eq!(decode_rectangles(plan), expected);
    assert_eq!(expected[20 * 512 + 20], [0xff0000, 0x0000ff][phase]);
    assert_eq!(expected[20 * 512 + 40], 0xffffff);
    assert_eq!(expected[100 * 512 + 216], 0x408080);
    assert_eq!(expected[511 * 512 + 511], 0xffffff);
    let graph = doc
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect::<Vec<_>>();
    if let Some(previous) = previous {
        assert_eq!(ids, previous.ids);
        assert_eq!(graph, previous.graph);
        assert_eq!(
            expected
                .iter()
                .zip(&previous.pixels)
                .filter(|(left, right)| left != right)
                .count(),
            240
        );
    }
    State {
        graph,
        ids,
        pixels: expected,
    }
}
