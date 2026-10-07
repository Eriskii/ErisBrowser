use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore,
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    layout::LayoutResult,
};
use eris_raster_core::{DrawKind, Frame, PARAM_STRIDE, Plan, Profile};

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 200;
pub const PNG: &[u8] = include_bytes!("../../examples/vulkan-opacity.png");
pub const PUBLIC_HTML: &str = include_str!("../../examples/vulkan-opacity.html");

// The public scene leaves room for native browser chrome. This richer scene
// independently exercises an actual button and two overlapping siblings in
// the Page/worker viewport, where no browser chrome commands are present.
pub const BUTTON_HTML: &str = r#"<!doctype html><meta charset=utf-8>
<title id=heading>Native opacity 0.50</title><style>
body{margin:0}
#card{position:absolute;left:16px;top:16px;width:224px;height:128px;background:#ff0000;opacity:.5}
#blue{position:absolute;left:16px;top:16px;width:64px;height:48px;background:#0000ff}
#green{position:absolute;left:48px;top:40px;width:56px;height:40px;background:#00ff00}
#panel{position:absolute;left:136px;top:16px;width:64px;height:64px;background:#0000ff;opacity:.5}
#panel-green{position:absolute;left:16px;top:16px;width:32px;height:32px;background:#00ff00}
#label{position:absolute;left:16px;top:90px;font-size:16px;line-height:20px;color:white}
#image{position:absolute;left:96px;top:90px;width:16px;height:16px}
#rounded{position:absolute;left:128px;top:90px;width:24px;height:24px;background:#ffff00;border-radius:6px}
#change{position:absolute;left:16px;top:160px;width:160px;height:28px;border:0;padding:0;background:transparent;font-size:16px;line-height:20px}
</style><body data-phase=ready>
<div id=card><div id=blue></div><div id=green></div><div id=panel><div id=panel-green></div></div><span id=label>Hi</span><img id=image src=vulkan-opacity.png alt="two pixels"><div id=rounded></div></div>
<button id=change type=button>Toggle opacity</button>
<script>
var phase=0;var card=document.getElementById('card');
document.getElementById('change').addEventListener('click',function(){
phase=1-phase;card.style.opacity=phase?'0.75':'0.5';
document.body.setAttribute('data-phase',phase?'changed':'ready');
document.getElementById('heading').firstChild.data=phase?'Native opacity 0.75':'Native opacity 0.50';
});
</script>"#;

#[derive(Clone, Copy, Debug)]
pub enum Scene {
    Public,
    Button,
}
impl Scene {
    pub fn html(self) -> &'static str {
        match self {
            Self::Public => PUBLIC_HTML,
            Self::Button => BUTTON_HTML,
        }
    }
    pub fn hit(self) -> (f32, f32) {
        match self {
            Self::Public => (20.0, 20.0),
            Self::Button => (96.0, 174.0),
        }
    }
    fn nontext(self) -> usize {
        match self {
            Self::Public => 9,
            Self::Button => 11,
        }
    }
}

pub fn metadata(phase: usize) -> &'static str {
    ["Native opacity 0.50", "Native opacity 0.75"][phase]
}

pub struct State {
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    ids: Vec<NodeId>,
    pixels: Vec<u32>,
}

pub fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    scene: Scene,
    phase: usize,
    previous: Option<&State>,
) -> State {
    assert!(phase < 2);
    assert_eq!(previous.is_some(), phase == 1);
    let find = |selector| doc.query_selector(selector).unwrap();
    let body = find("body");
    assert_eq!(
        doc.attr(body, "data-phase"),
        Some(["ready", "changed"][phase])
    );
    let title = find("#heading");
    assert_eq!(doc.nodes[title].children.len(), 1);
    let title_text = doc.nodes[title].children[0];
    let NodeKind::Text(data) = &doc.nodes[title_text].kind else {
        panic!("retained title Text");
    };
    assert_eq!(data.scalar(), Some(metadata(phase)));
    let mut ids = vec![doc.root, body, title, title_text];
    for selector in [
        "#change",
        "#panel",
        "#panel-green",
        "#label",
        "#image",
        "#rounded",
    ] {
        ids.push(find(selector));
    }
    if matches!(scene, Scene::Button) {
        ids.extend([find("#card"), find("#blue"), find("#green")]);
    }
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[..index].contains(id));
    }
    let graph = doc
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect();
    let (x, y) = scene.hit();
    let change = find("#change");
    let hit = layout.hit_test(x, y).unwrap();
    match scene {
        Scene::Public => assert_eq!(hit, change),
        Scene::Button => {
            // Buttons use ordinary child layout. Hit testing returns the Text
            // node; the real Click dispatched above bubbles to its button.
            assert_eq!(doc.nodes[change].children, [hit]);
            assert_eq!(doc.nodes[hit].parent, Some(change));
            let NodeKind::Text(label) = &doc.nodes[hit].kind else {
                panic!("button's retained literal Text is the real hit target");
            };
            assert_eq!(label.scalar(), Some("Toggle opacity"));
        }
    }
    assert!(
        layout.content_height <= HEIGHT as f32,
        "no scrollbar prerequisite"
    );
    assert_eq!(
        doc.attr(find("#image"), "data-eris-natural-width"),
        Some("2")
    );
    assert_eq!(
        doc.attr(find("#image"), "data-eris-natural-height"),
        Some("1")
    );
    assert_eq!(images.len(), 1);
    let image = images.values().next().unwrap();
    assert_eq!((image.width, image.height), (2, 1));
    assert_eq!(image.rgba, [255, 255, 255, 128, 0, 0, 255, 255]);
    let nontext = layout
        .commands
        .iter()
        .filter(|command| !matches!(command, DrawCommand::Text { .. }))
        .count();
    assert_eq!(nontext, scene.nontext());
    let opacities = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::PushOpacity { opacity } => Some(*opacity),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(opacities, [[0.5, 0.5], [0.75, 0.5]][phase]);
    assert!(layout.commands.iter().any(|command| matches!(command,
        DrawCommand::Rect { radius, .. } if *radius == 6.0)));
    assert!(layout.commands.iter().any(|command| matches!(command,
        DrawCommand::Text { text, .. } if text == "Hi")));
    assert_eq!(
        layout
            .commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Image { .. }))
            .count(),
        1
    );

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
    .expect("actual loaded display list takes the Native route");
    let plan = native.plan();
    assert_eq!(plan.profile(), Profile::Native);
    assert_eq!(plan.group_scratch_bytes(), 262_144);
    assert_eq!(
        plan.draws()
            .iter()
            .filter(|draw| draw.kind() == DrawKind::GroupClear)
            .count(),
        2
    );
    assert_eq!(
        plan.draws()
            .iter()
            .filter(|draw| draw.kind() == DrawKind::GroupComposite)
            .count(),
        2
    );
    let weights = plan
        .draws()
        .iter()
        .enumerate()
        .filter(|(_, draw)| draw.kind() == DrawKind::GroupComposite)
        .map(|(index, _)| parameter(plan, index, 29))
        .collect::<Vec<_>>();
    assert_eq!(weights, [[128, 128], [128, 192]][phase]);
    assert!(
        plan.draws()
            .iter()
            .any(|draw| draw.target_is_group() && draw.kind() == DrawKind::Image)
    );
    assert!(
        plan.draws()
            .iter()
            .any(|draw| draw.target_is_group() && draw.kind() == DrawKind::Glyph)
    );
    assert!(native.stats().rounded_masks > 0);
    assert!(native.stats().gpu_buffer_upper_bound >= plan.gpu_buffer_bytes());
    assert!(native.stats().gpu_buffer_upper_bound <= Profile::Native.max_gpu_buffer_bytes());
    assert!(native.stats().cpu_pixel_upper_bound <= u64::from(WIDTH) * u64::from(HEIGHT) * 16);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    // Packed-plan interpretation is a CPU oracle, not a GPU execution claim.
    assert_eq!(
        decode_opacity_pixels(plan),
        canvas.pixels,
        "full viewport Canvas reference"
    );
    let sample = |x: usize, y: usize| canvas.pixels[y * WIDTH as usize + x];
    for (x, y, expected) in [
        (20, 20, [0xff8080, 0xff4040]),
        (156, 36, [0xbf80bf, 0x9f409f]),
        (176, 56, [0xbfbf80, 0x9f9f40]),
        (114, 110, [0xffc0c0, 0xffa0a0]),
        (124, 110, [0x8080ff, 0x4040ff]),
        (156, 118, [0xffff80, 0xffff40]),
        (300, 190, [0xffffff, 0xffffff]),
    ] {
        assert_eq!(
            sample(x, y),
            expected[phase],
            "literal pixel ({x},{y}), {scene:?}, phase {phase}"
        );
    }
    if matches!(scene, Scene::Button) {
        assert_eq!(sample(40, 40), [0x8080ff, 0x4040ff][phase]);
        assert_eq!(sample(72, 64), [0x80ff80, 0x40ff40][phase]);
    }
    if let Some(previous) = previous {
        assert_eq!(ids, previous.ids);
        assert_eq!(
            graph, previous.graph,
            "every original ID and edge retained, no click-created nodes"
        );
        assert_ne!(
            canvas.pixels, previous.pixels,
            "click changes visible opacity"
        );
    }
    State {
        graph,
        ids,
        pixels: canvas.pixels,
    }
}

// Test-only independent integer interpretation of the packed Native ABI.
// The same decoder is used by the focused native bridge opacity tests; this
// copy lets real loaded Page/IPC display lists meet the full Canvas oracle.
fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
fn parameter(plan: &Plan, draw: usize, index: usize) -> u32 {
    word(&plan.parameters()[draw * PARAM_STRIDE..], index)
}
fn decode_opacity_pixels(plan: &Plan) -> Vec<u32> {
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
                        assert_eq!(source[3], 65535);
                        let k = u64::from(p(29));
                        assert!((1..256).contains(&k));
                        let mut next = [0; 4];
                        for (channel, next) in next.iter_mut().enumerate() {
                            let numerator = source[channel] * k + destination[channel] * (256 - k);
                            *next = if group {
                                (numerator + 128) / 256
                            } else {
                                ((numerator + 32896) / 65792) * 257
                            };
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
