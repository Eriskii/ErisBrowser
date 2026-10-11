use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore,
        raster_bridge::{
            FallbackKind,
            native::{NativePhase, plan_native_scene},
        },
    },
    layout::LayoutResult,
};
use eris_raster_core::{
    Command, DrawKind, Frame, PARAM_STRIDE, Phase, Plan, Profile, Rect, SourceImage,
    plan_native_phases,
};

pub const HTML: &str = include_str!("../../examples/vulkan-small-images.html");
pub const PNG: &[u8] = include_bytes!("../../examples/vulkan-small-images.png");
pub const WIDTH: u32 = 512;
pub const HEIGHT: u32 = 512;
pub const HIT: (f32, f32) = (20.0, 20.0);
const RGBA: [u8; 16] = [
    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
];

pub fn metadata(phase: usize) -> &'static str {
    ["Small images ready", "Small images wider"][phase]
}

pub struct State {
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    ids: Vec<NodeId>,
    pixels: Vec<u32>,
}

pub fn expected_pixels(phase: usize) -> Vec<u32> {
    assert!(phase < 2);
    let mut pixels = vec![0xffffff; 512 * 512];
    for index in 0..40 {
        let x = 16 + index % 8 * 32;
        let y = 16 + index / 8 * 32;
        let half_width = if index == 0 && phase == 1 { 16 } else { 12 };
        // Independent literal quadrants, not the renderer's sampling formula.
        for row in y..y + 12 {
            pixels[row * 512 + x..row * 512 + x + half_width].fill(0xff0000);
            pixels[row * 512 + x + half_width..row * 512 + x + half_width * 2].fill(0x00ff00);
        }
        for row in y + 12..y + 24 {
            pixels[row * 512 + x..row * 512 + x + half_width].fill(0x0000ff);
            pixels[row * 512 + x + half_width..row * 512 + x + half_width * 2].fill(0xffff00);
        }
    }
    pixels
}

fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}

// Decode only the opaque clear and Image records accepted by this witness.
// This is packed-plan validation on the CPU, not shader or acquired-texture execution.
fn decode_images(plan: &Plan) -> Vec<u32> {
    let mut pixels = vec![0; (WIDTH * HEIGHT) as usize];
    for (index, draw) in plan.draws().iter().enumerate() {
        assert!(!draw.target_is_group());
        let p = |field| {
            word(
                &plan.parameters()[index * PARAM_STRIDE..(index + 1) * PARAM_STRIDE],
                field,
            )
        };
        let (x0, y0, width, height) = draw.bounds();
        assert!(x0 + width <= WIDTH && y0 + height <= HEIGHT);
        if index == 0 {
            assert_eq!(draw.kind(), DrawKind::Rectangle);
            assert_eq!(draw.bounds(), (0, 0, WIDTH, HEIGHT));
            assert_eq!(p(6), 0xffffffff);
        } else {
            assert_eq!(draw.kind(), DrawKind::Image);
            assert_eq!((p(9), p(10)), (2, 2));
        }
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let color = if index == 0 {
                    p(6)
                } else {
                    let sx = word(plan.input_bytes(), (p(12) + x - x0) as usize);
                    let sy = word(plan.input_bytes(), (p(13) + y - y0) as usize);
                    assert!(sx < 2 && sy < 2);
                    word(plan.input_bytes(), (p(8) + sy * p(9) + sx) as usize)
                };
                assert_eq!(color >> 24, 255);
                pixels[(y * WIDTH + x) as usize] = color & 0xffffff;
            }
        }
    }
    pixels
}

// The independent literal core plan must already fit the unchanged Native caps
// on the predecessor. It isolates the bridge's CPU reservation from other caps.
fn literal_plan(phase: usize) -> Plan {
    let commands = (0..40)
        .map(|index| Command::Image {
            rect: Rect::new(
                (16 + index % 8 * 32) as f32,
                (16 + index / 8 * 32) as f32,
                if index == 0 && phase == 1 { 32.0 } else { 24.0 },
                24.0,
            ),
            source: 0,
        })
        .collect::<Vec<_>>();
    let frame = Frame::new(WIDTH, HEIGHT, 0xffffff);
    plan_native_phases(
        frame,
        &[Phase {
            frame,
            commands: &commands,
        }],
        &[SourceImage {
            width: 2,
            height: 2,
            rgba: &RGBA,
        }],
        &[],
        &[],
    )
    .expect("literal image scene fits the existing complete Native core limits")
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
    assert_eq!(images.len(), 1);
    let image = images.values().next().unwrap();
    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(image.rgba, RGBA);
    let body = doc.query_selector("body").unwrap();
    assert_eq!(
        doc.attr(body, "data-phase"),
        Some(["ready", "wider"][phase])
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
        let id = doc.query_selector(&format!("#icon{index:02}")).unwrap();
        assert_eq!(doc.nodes[id].parent, Some(body));
        assert!(doc.nodes[id].children.is_empty());
        assert_eq!(doc.attr(id, "data-eris-natural-width"), Some("2"));
        assert_eq!(doc.attr(id, "data-eris-natural-height"), Some("2"));
        assert!(!ids.contains(&id));
        ids.push(id);
    }
    assert_eq!(ids.len(), 44);
    assert_eq!(layout.hit_test(HIT.0, HIT.1), Some(ids[4]));
    assert!(layout.content_height <= HEIGHT as f32);
    assert_eq!(layout.commands.len(), 40);
    for (index, command) in layout.commands.iter().enumerate() {
        let DrawCommand::Image { rect, key } = command else {
            panic!("forty visible Image primitives")
        };
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (
                (16 + index % 8 * 32) as f32,
                (16 + index / 8 * 32) as f32,
                if index == 0 && phase == 1 { 32.0 } else { 24.0 },
                24.0
            )
        );
        assert!(std::sync::Arc::ptr_eq(images.get(key).unwrap(), image));
    }
    let expected = expected_pixels(phase);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    let core = literal_plan(phase);
    assert_eq!(core.profile(), Profile::Native);
    assert_eq!(core.draws().len(), 41);
    assert_eq!(core.group_scratch_bytes(), 0);
    assert_eq!(core.invocations(), [547_328, 547_520][phase]);
    assert_eq!(core.gpu_buffer_bytes(), [2_115_360, 2_115_392][phase]);
    assert_eq!(core.input_bytes().len(), [7_696, 7_728][phase]);
    assert_eq!(decode_images(&core), expected);
    let frame = Frame::new(WIDTH, HEIGHT, 0xffffff);
    let result = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &layout.commands,
        }],
        images,
        fonts,
    );
    if expect_native {
        let native = result.expect("actual loaded icon grid admits the Native plan");
        let stats = native.stats();
        assert_eq!(stats.phases, 1);
        assert_eq!(stats.bridge.original_commands, 40);
        assert_eq!(stats.bridge.lowered_commands, 40);
        assert_eq!(stats.bridge.image_store_entries, 1);
        assert_eq!(stats.bridge.unique_sources, 1);
        assert_eq!(stats.bridge.referenced_sources, 1);
        assert_eq!(stats.bridge.total_rgba_bytes, 16);
        assert_eq!(stats.bridge.referenced_rgba_bytes, 16);
        assert_eq!(stats.cpu_pixel_upper_bound, [23_040, 23_232][phase]);
        assert_eq!(stats.text.occurrences, 0);
        assert_eq!(stats.rounded_masks, 0);
        assert_eq!(stats.mask_sources, 0);
        assert_eq!(stats.row_entries, 0);
        assert!(stats.gpu_buffer_upper_bound >= core.gpu_buffer_bytes());
        let plan = native.plan();
        assert_eq!(plan.profile(), Profile::Native);
        assert_eq!(plan.draws(), core.draws());
        assert_eq!(plan.parameters(), core.parameters());
        assert_eq!(plan.input_bytes(), core.input_bytes());
        assert_eq!(plan.invocations(), core.invocations());
        assert_eq!(plan.gpu_buffer_bytes(), core.gpu_buffer_bytes());
        assert_eq!(plan.group_scratch_bytes(), 0);
        assert_eq!(decode_images(plan), expected);
        println!(
            "small-images phase={phase} native=admitted commands=40 sources=1 cpu={} invocations={} gpu_bytes={} exact_pixels=262144",
            stats.cpu_pixel_upper_bound,
            plan.invocations(),
            plan.gpu_buffer_bytes()
        );
    } else {
        let refusal = result.expect_err("unchanged predecessor must refuse the whole icon scene");
        assert_eq!(refusal.kind, FallbackKind::CpuPaintBudget);
        assert_eq!(refusal.phase_index, None);
        assert_eq!(refusal.command_index, None);
        println!(
            "small-images phase={phase} native=cpu-paint-budget commands=40 sources=1 old_cpu=10485760 limit=4194304 core_invocations={} core_gpu_bytes={} exact_pixels=262144",
            core.invocations(),
            core.gpu_buffer_bytes()
        );
    }
    assert_eq!(expected[20 * 512 + 20], 0xff0000);
    assert_eq!(expected[20 * 512 + 30], [0x00ff00, 0xff0000][phase]);
    assert_eq!(expected[20 * 512 + 44], [0xffffff, 0x00ff00][phase]);
    assert_eq!(expected[32 * 512 + 20], 0x0000ff);
    assert_eq!(expected[32 * 512 + 44], [0xffffff, 0xffff00][phase]);
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
                .filter(|(a, b)| a != b)
                .count(),
            288
        );
    }
    State {
        graph,
        ids,
        pixels: expected,
    }
}
