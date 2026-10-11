use eris::{
    dom::{Document, NodeId},
    graphics::{Canvas, Fonts, ImageStore},
    layout::{HitAction, LayoutResult},
};

pub const HTML: &str = include_str!("../../examples/linear-gradient.html");
pub const WIDTH: u32 = 96;
pub const HEIGHT: u32 = 64;
type Graph = Vec<(Option<NodeId>, Vec<NodeId>)>;
type Hit = (NodeId, HitAction, f32, f32, f32, f32, bool);

pub struct State {
    pub click: NodeId,
    graph: Graph,
    hits: Vec<Hit>,
    ids: Vec<NodeId>,
    pixels: Vec<u32>,
}

pub fn metadata(phase: usize) -> &'static str {
    ["Gradient ready", "Gradient changed"][phase]
}

fn fill(pixels: &mut [u32], x: usize, y: usize, width: usize, height: usize, color: u32) {
    for row in y..y + height {
        pixels[row * WIDTH as usize + x..row * WIDTH as usize + x + width].fill(color);
    }
}

fn expected(phase: usize) -> Vec<u32> {
    let mut pixels = vec![0xffffff; (WIDTH * HEIGHT) as usize];
    for (column, color) in [0x1f1f1f, 0x5d5d5d, 0x9b9b9b, 0xd9d9d9]
        .into_iter()
        .enumerate()
    {
        fill(&mut pixels, 8 + column, 8, 1, 16, color);
    }
    for (column, color) in [0xf92020, 0xfb6060, 0xfc9f9f, 0xfedfdf]
        .into_iter()
        .enumerate()
    {
        fill(&mut pixels, 8 + column, 32, 1, 16, color);
    }
    fill(
        &mut pixels,
        24,
        8,
        16,
        16,
        if phase == 0 { 0xff0000 } else { 0x00ff00 },
    );
    fill(
        &mut pixels,
        40,
        8,
        16,
        16,
        if phase == 0 { 0x0000ff } else { 0xffff00 },
    );
    fill(&mut pixels, 24, 32, 32, 16, 0x00ff00);
    fill(&mut pixels, 64, 8, 16, 16, 0x00ff00);
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
    assert_eq!(doc.title(), metadata(phase));
    assert!(
        images.is_empty(),
        "gradients need no generated image resources"
    );
    let body = doc.query_selector("body").unwrap();
    assert_eq!(doc.attr(body, "data-supports"), Some("yes"));
    assert_eq!(
        doc.attr(body, "data-phase"),
        Some(["ready", "changed"][phase])
    );
    assert_eq!(
        doc.attr(body, "data-style-kept"),
        Some(["ready", "yes"][phase])
    );
    let mut ids = Vec::new();
    for (name, x, y, width) in [
        ("stripe", 8.0, 8.0, 4.0),
        ("switch", 24.0, 8.0, 32.0),
        ("alpha", 8.0, 32.0, 4.0),
        ("parent", 24.0, 32.0, 32.0),
        ("inherited", 24.0, 32.0, 32.0),
        ("reset", 64.0, 8.0, 16.0),
    ] {
        let id = doc.query_selector(&format!("#{name}")).unwrap();
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == id)
            .unwrap();
        assert_eq!(
            (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
            (x, y, width, 16.0),
            "{name}"
        );
        ids.push(id);
    }
    let click = layout.hit_test(25.0, 9.0).unwrap();
    assert_eq!(click, ids[1]);
    let graph = doc
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect();
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
        .collect();
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected(phase));
    if let Some(previous) = previous {
        assert_eq!(graph, previous.graph);
        assert_eq!(hits, previous.hits);
        assert_eq!(ids, previous.ids);
        assert_ne!(canvas.pixels, previous.pixels);
    }
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::{
            FallbackKind,
            native::{NativePhase, plan_native_scene},
        };
        let frame = eris_raster_core::Frame::new(WIDTH, HEIGHT, 0xffffff);
        let refusal = plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            images,
            fonts,
        )
        .unwrap_err();
        assert_eq!(refusal.kind, FallbackKind::UnsupportedGradient);
        assert!(refusal.command_index.is_some());
    }
    State {
        click,
        graph,
        hits,
        ids,
        pixels: canvas.pixels,
    }
}
