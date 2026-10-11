use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::{HitAction, LayoutResult},
};

pub const HTML: &str = include_str!("../../examples/object-fit-position.html");
pub const PNG: &[u8] = include_bytes!("../../examples/object-fit-position.png");
pub const WIDTH: u32 = 128;
pub const HEIGHT: u32 = 128;
pub const HIT: (f32, f32) = (12.0, 12.0);
const NAMES: [&str; 9] = [
    "fill", "contain", "cover", "none", "scale", "small", "top", "two", "calc",
];
const TOP: [u32; 4] = [0xff0000, 0x00ff00, 0x0000ff, 0xffff00];
const BOTTOM: [u32; 4] = [0x00ffff, 0xff00ff, 0xff8000, 0x8000ff];
const BOXES: [(u32, u32, u32); 9] = [
    (8, 8, 22),
    (40, 8, 22),
    (72, 8, 22),
    (8, 40, 22),
    (40, 40, 22),
    (72, 40, 10),
    (8, 72, 22),
    (40, 72, 22),
    (72, 72, 22),
];
type Links = Vec<(Option<NodeId>, Vec<NodeId>)>;
type Hit = (NodeId, HitAction, f32, f32, f32, f32, bool);

pub struct State {
    pub click: NodeId,
    graph: Links,
    ids: Vec<NodeId>,
    hits: Vec<Hit>,
    pixels: Vec<u32>,
}

pub fn metadata(phase: usize) -> &'static str {
    ["Object fit ready", "Object fit changed"][phase]
}

fn solid(pixels: &mut [u32], x: u32, y: u32, width: u32, height: u32, color: u32) {
    assert!(x + width <= WIDTH && y + height <= HEIGHT);
    for row in y..y + height {
        pixels[(row * WIDTH + x) as usize..(row * WIDTH + x + width) as usize].fill(color);
    }
}

fn stripes(
    pixels: &mut [u32],
    origin: (u32, u32),
    stripe: (u32, u32),
    top: &[u32],
    bottom: &[u32],
) {
    assert_eq!(top.len(), bottom.len());
    for (index, (&a, &b)) in top.iter().zip(bottom).enumerate() {
        let x = origin.0 + index as u32 * stripe.0;
        solid(pixels, x, origin.1, stripe.0, stripe.1, a);
        solid(pixels, x, origin.1 + stripe.1, stripe.0, stripe.1, b);
    }
}

pub fn expected_pixels(phase: usize) -> Vec<u32> {
    assert!(phase < 2);
    let mut pixels = vec![0xffffff; (WIDTH * HEIGHT) as usize];
    for (x, y, side) in BOXES {
        solid(&mut pixels, x, y, side, side, 0x000000);
        solid(&mut pixels, x + 1, y + 1, side - 2, side - 2, 0xd0d0d0);
    }
    solid(&mut pixels, 0, 104, 104, 6, 0x123456);
    // Literal visible color rectangles; no fit, position, clipping, or sampling
    // implementation is called to construct these expectations.
    if phase == 0 {
        stripes(&mut pixels, (11, 11), (4, 8), &TOP, &BOTTOM);
    } else {
        stripes(&mut pixels, (15, 19), (2, 2), &TOP, &BOTTOM);
    }
    stripes(&mut pixels, (43, 15), (4, 4), &TOP, &BOTTOM);
    stripes(
        &mut pixels,
        (75, 11),
        (8, 8),
        &[0x00ff00, 0x0000ff],
        &[0xff00ff, 0xff8000],
    );
    stripes(&mut pixels, (15, 49), (2, 2), &TOP, &BOTTOM);
    stripes(&mut pixels, (47, 49), (2, 2), &TOP, &BOTTOM);
    stripes(&mut pixels, (75, 44), (1, 1), &TOP, &BOTTOM);
    stripes(&mut pixels, (11, 75), (4, 4), &TOP, &BOTTOM);
    stripes(&mut pixels, (45, 87), (2, 2), &TOP, &BOTTOM);
    stripes(&mut pixels, (79, 83), (2, 2), &TOP, &BOTTOM);
    pixels
}

#[cfg(feature = "raster-bridge")]
fn check_native(layout: &LayoutResult, images: &ImageStore, fonts: &Fonts, expected: &[u32]) {
    use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
    use eris_raster_core::{DrawKind, Frame, PARAM_STRIDE, Profile};
    fn word(bytes: &[u8], index: usize) -> u32 {
        u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
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
    .expect("bounded opaque image/rectangle scene admits the Native plan");
    let plan = native.plan();
    assert_eq!(plan.profile(), Profile::Native);
    assert_eq!(native.stats().bridge.unique_sources, 1);
    assert_eq!(plan.group_scratch_bytes(), 0);
    let mut actual = vec![0; expected.len()];
    // Packed-plan checking on the CPU; this does not execute a GPU shader or
    // verify an acquired texture. All fixture draws are opaque and unrounded.
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
        }
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                let color = match draw.kind() {
                    DrawKind::Rectangle => p(6),
                    DrawKind::Image => {
                        assert_eq!((p(9), p(10)), (8, 4));
                        let sx = word(plan.input_bytes(), (p(12) + x - x0) as usize);
                        let sy = word(plan.input_bytes(), (p(13) + y - y0) as usize);
                        assert!(sx < 8 && sy < 4);
                        word(plan.input_bytes(), (p(8) + sy * p(9) + sx) as usize)
                    }
                    _ => panic!("fixture has only opaque rectangle and image draws"),
                };
                assert_eq!(color >> 24, 255);
                actual[(y * WIDTH + x) as usize] = color & 0xffffff;
            }
        }
    }
    assert_eq!(actual, expected);
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
    assert_eq!(doc.title(), metadata(phase));
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
    let title = doc.query_selector("#heading").unwrap();
    assert_eq!(doc.nodes[title].children.len(), 1);
    let title_text = doc.nodes[title].children[0];
    let NodeKind::Text(text) = &doc.nodes[title_text].kind else {
        panic!("retained title Text")
    };
    assert_eq!(text.scalar(), Some(metadata(phase)));
    let board = doc.query_selector("#board").unwrap();
    let after = doc.query_selector("#after").unwrap();
    let mut ids = vec![doc.root, body, title, title_text, board, after];
    assert_eq!(images.len(), 1);
    let image = images.values().next().unwrap();
    assert_eq!((image.width, image.height), (8, 4));
    let mut expected_rgba = Vec::new();
    for row in [&TOP, &TOP, &BOTTOM, &BOTTOM] {
        for color in row {
            for _ in 0..2 {
                expected_rgba.extend_from_slice(&[
                    (color >> 16) as u8,
                    (color >> 8) as u8,
                    *color as u8,
                    255,
                ]);
            }
        }
    }
    assert_eq!(image.rgba, expected_rgba);
    for (name, (x, y, side)) in NAMES.into_iter().zip(BOXES) {
        let id = doc.query_selector(&format!("#{name}")).unwrap();
        assert_eq!(doc.nodes[id].parent, Some(board));
        assert!(doc.nodes[id].children.is_empty());
        assert_eq!(doc.attr(id, "width"), Some("80"));
        assert_eq!(doc.attr(id, "height"), Some("40"));
        assert_eq!(doc.attr(id, "data-eris-natural-width"), Some("8"));
        assert_eq!(doc.attr(id, "data-eris-natural-height"), Some("4"));
        assert!(!ids.contains(&id));
        ids.push(id);
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == id)
            .unwrap();
        assert_eq!(
            (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
            (x as f32, y as f32, side as f32, side as f32)
        );
        assert!(!hit.fixed);
        assert_eq!(layout.hit_test(x as f32 + 1.0, y as f32 + 1.0), Some(id));
    }
    for (id, expected) in [
        (board, (0.0, 0.0, 104.0, 104.0)),
        (after, (0.0, 104.0, 104.0, 6.0)),
    ] {
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == id)
            .unwrap();
        assert_eq!(
            (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
            expected
        );
    }
    assert_eq!(ids.len(), 15);
    let click = ids[6];
    assert_eq!(layout.hit_test(HIT.0, HIT.1), Some(click));
    assert!(layout.content_height <= HEIGHT as f32);
    let image_commands = layout
        .commands
        .iter()
        .filter_map(|command| {
            if let DrawCommand::Image { rect, key } = command {
                assert!(std::sync::Arc::ptr_eq(images.get(key).unwrap(), image));
                Some((rect.x, rect.y, rect.width, rect.height))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut destinations = vec![
        (11.0, 11.0, 16.0, 16.0),
        (43.0, 15.0, 16.0, 8.0),
        (67.0, 11.0, 32.0, 16.0),
        (15.0, 49.0, 8.0, 4.0),
        (47.0, 49.0, 8.0, 4.0),
        (75.0, 44.0, 4.0, 2.0),
        (11.0, 75.0, 16.0, 8.0),
        (45.0, 87.0, 8.0, 4.0),
        (79.0, 83.0, 8.0, 4.0),
    ];
    if phase == 1 {
        destinations[0] = (15.0, 19.0, 8.0, 4.0);
    }
    assert_eq!(image_commands, destinations);
    let expected = expected_pixels(phase);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    #[cfg(feature = "raster-bridge")]
    check_native(layout, images, fonts, &expected);
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
    if let Some(previous) = previous {
        assert_eq!(ids, previous.ids);
        assert_eq!(graph, previous.graph);
        assert_eq!(hits, previous.hits);
        assert_eq!(click, previous.click);
        assert_ne!(expected, previous.pixels);
    }
    State {
        click,
        graph,
        ids,
        hits,
        pixels: expected,
    }
}
