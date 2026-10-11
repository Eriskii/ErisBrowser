//! Literal whole-frame background geometry oracle, independent of layout helpers.
use eris::{
    dom::{Document, NodeId},
    graphics::{Canvas, Fonts, ImageStore},
    layout::{HitAction, LayoutResult},
};

pub const HTML: &str = include_str!("../../examples/background-geometry.html");
pub const WIDTH: u32 = 72;
pub const HEIGHT: u32 = 52;
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
    ["Background geometry ready", "Background geometry changed"][phase]
}
fn rows(pixels: &mut [u32], x: usize, y: usize, values: &[&str]) {
    for (dy, row) in values.iter().enumerate() {
        for (dx, value) in row.bytes().enumerate() {
            pixels[(y + dy) * WIDTH as usize + x + dx] = match value {
                b'W' => 0xffffff,
                b'R' => 0xff0000,
                b'B' => 0x0000ff,
                b'G' => 0x00ff00,
                b'Y' => 0xffff00,
                _ => panic!("literal color"),
            };
        }
    }
}
fn expected(phase: usize) -> Vec<u32> {
    let mut pixels = vec![0xffffff; (WIDTH * HEIGHT) as usize];
    if phase == 0 {
        rows(&mut pixels, 4, 6, &["WWWWRRBBWWWW"; 2]);
    } else {
        rows(&mut pixels, 4, 4, &["RRRBBBRRRBBB"; 2]);
    }
    rows(&mut pixels, 20, 7, &["WWRRBBWWWWWW"; 2]);
    rows(&mut pixels, 36, 6, &["RRRRRRBBBBBB"; 2]);
    rows(&mut pixels, 52, 6, &["RRBBRRBBRRBB"; 2]);
    rows(&mut pixels, 4, 14, &["WWWWRRBBWWWW"; 6]);
    rows(&mut pixels, 20, 18, &["WWWWWWRRBBWW"; 2]);
    rows(&mut pixels, 36, 17, &["WWWWWWRRBBWW"; 2]);
    rows(&mut pixels, 52, 16, &["WWWWRRBBWWWW"; 2]);
    rows(&mut pixels, 4, 24, &["WWWWRRBBWWWW"; 6]);
    rows(&mut pixels, 20, 26, &["RRRRRRBBBBBB"; 2]);
    rows(&mut pixels, 36, 24, &["RRRRRRBBBBBB"; 6]);
    rows(&mut pixels, 52, 24, &["RRRRRRBBBBBB"; 6]);
    rows(&mut pixels, 4, 34, &["YYYYYYYYYYYY"; 6]);
    rows(
        &mut pixels,
        20,
        34,
        &[if phase == 0 {
            "GGGGGGGGGGGG"
        } else {
            "RRRRRRBBBBBB"
        }; 6],
    );
    rows(&mut pixels, 36, 36, &["WWWWRRBBWWWW"; 2]);
    rows(&mut pixels, 52, 36, &["RRRRRRBB"; 2]);
    for y in 44..46 {
        pixels[y * WIDTH as usize..y * WIDTH as usize + 64].fill(0x00ff00);
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
    assert_eq!(doc.title(), metadata(phase));
    assert!(
        images.is_empty(),
        "semantic gradients retain no page image assets"
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
    let names = [
        "switch", "percent", "oversize", "repeatx", "repeaty", "edge3", "edge4", "calc", "autow",
        "autoh", "cover", "contain", "zero", "reset", "padded",
    ];
    let mut ids = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let node = doc.query_selector(&format!("#{name}")).unwrap();
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == node)
            .unwrap();
        assert_eq!(
            (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
            (
                4.0 + 16.0 * (i % 4) as f32,
                4.0 + 10.0 * (i / 4) as f32,
                12.0,
                6.0
            ),
            "{name}"
        );
        ids.push(node);
    }
    let following = doc.query_selector("#following").unwrap();
    let hit = layout
        .hit_regions
        .iter()
        .find(|hit| hit.node == following)
        .unwrap();
    assert_eq!(
        (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
        (0.0, 44.0, 64.0, 2.0)
    );
    ids.push(following);
    let click = layout.hit_test(5.0, 5.0).unwrap();
    assert_eq!(click, ids[0]);
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
    let expected = expected(phase);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    if let Some(previous) = previous {
        assert_eq!(graph, previous.graph);
        assert_eq!(hits, previous.hits);
        assert_eq!(ids, previous.ids);
        assert_eq!(click, previous.click);
        assert_ne!(canvas.pixels, previous.pixels);
    }
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
        let frame = eris_raster_core::Frame::new(WIDTH, HEIGHT, 0xffffff);
        let native = plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            images,
            fonts,
        )
        .expect("opaque square-corner positioned gradients admit Native");
        assert_eq!(packed_pixels(native.plan()), expected);
    }
    State {
        click,
        graph,
        hits,
        ids,
        pixels: canvas.pixels,
    }
}

// Interpret only existing opaque packed rectangle/image records; expected pixels
// above are independent literal rows, never derived from the renderer's geometry.
#[cfg(feature = "raster-bridge")]
fn packed_pixels(plan: &eris_raster_core::Plan) -> Vec<u32> {
    use eris_raster_core::{DrawKind, PARAM_STRIDE, Profile};
    fn word(bytes: &[u8], index: usize) -> u32 {
        u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    }
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
                    _ => panic!("opaque fixture accepts only rectangle/image records"),
                };
                assert_eq!(rgba >> 24, 255);
                pixels[(y * frame.width + x) as usize] = rgba & 0xffffff;
            }
        }
    }
    pixels
}
