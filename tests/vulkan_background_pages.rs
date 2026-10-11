//! Loaded CSS backgrounds through direct Page and real confined worker frames.
#![cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

use eris::{
    dom::{Document, NodeId},
    graphics::raster_bridge::native::{NativePhase, plan_native_scene},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};
use eris_raster_core::Frame;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

const WIDTH: u32 = 96;
const HEIGHT: u32 = 64;
const HTML: &str = include_str!("../examples/vulkan-backgrounds.html");
const PNG: &[u8] = include_bytes!("../examples/background-tile.png");
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new() -> Self {
        let directory = loop {
            let path = std::env::temp_dir().join(format!(
                "eris-native-background-pages-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("fixture directory: {error}"),
            }
        };
        fs::write(directory.join("index.html"), HTML).unwrap();
        fs::write(directory.join("background-tile.png"), PNG).unwrap();
        let address = Url::from_file_path(directory.join("index.html"))
            .unwrap()
            .to_string();
        Self { directory, address }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

// Handwritten color rows, independent of layout, LUT generation and rendering.
fn expected(phase: usize, scroll: usize) -> Vec<u32> {
    let mut pixels = vec![0xffffff; WIDTH as usize * HEIGHT as usize];
    let rows = if phase == 0 {
        [
            "RRRRGGGGRRRRGGGGRRRRGGGGRRRRGGGG",
            "BBBBYYYYBBBBYYYYBBBBYYYYBBBBYYYY",
            "RRRRGGGGRRRRGGGGRRRRGGGGRRRRGGGG",
            "BBBBYYYYBBBBYYYYBBBBYYYYBBBBYYYY",
        ]
    } else {
        [
            "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
            "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
            "EEEEEEEEEEEEEEEEEEEEEEEERRRRGGGG",
            "EEEEEEEEEEEEEEEEEEEEEEEEBBBBYYYY",
        ]
    };
    let color = |value| match value {
        b'R' => 0xff0000,
        b'G' => 0x00ff00,
        b'B' => 0x0000ff,
        b'Y' => 0xffff00,
        b'E' => 0xeeeeee,
        _ => panic!("literal color"),
    };
    for (band, row) in rows.iter().enumerate() {
        assert_eq!(row.len(), 32);
        for y in (8 + band * 4)..(12 + band * 4) {
            for (x, value) in row.bytes().enumerate() {
                pixels[(y - scroll) * WIDTH as usize + 8 + x] = color(value);
            }
        }
    }
    for (y, row) in [
        "RRGGRRGG", "RRGGRRGG", "BBYYBBYY", "BBYYBBYY", "RRGGRRGG", "RRGGRRGG", "BBYYBBYY",
        "BBYYBBYY",
    ]
    .iter()
    .enumerate()
    {
        for (x, value) in row.bytes().enumerate() {
            pixels[(8 + y) * WIDTH as usize + 56 + x] = color(value);
        }
    }
    pixels
}

#[derive(Debug, PartialEq)]
struct State {
    click: NodeId,
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    hits: Vec<(NodeId, f32, f32, f32, f32, bool)>,
}

fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
) -> State {
    let click = doc.query_selector("#board").unwrap();
    assert_eq!(layout.hit_test(12.0, 12.0), Some(click));
    let keys = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BackgroundImage { key, radius, .. } => {
                assert_eq!(*radius, 0.0);
                Some(key)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], keys[1]);
    let image = &images[keys[0]];
    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(
        image.rgba,
        [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255
        ]
    );
    for scroll in [0usize, 4] {
        let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint_with_viewport(
            &layout.commands,
            fonts,
            images,
            (0.0, -(scroll as f32)),
            (0.0, 0.0),
        );
        assert!(!canvas.exhausted());
        let expected = expected(phase, scroll);
        assert_eq!(canvas.pixels, expected, "phase={phase} scroll={scroll}");
        let target = Frame::new(WIDTH, HEIGHT, 0xffffff);
        let mut frame = target;
        frame.document_offset = (0.0, -(scroll as f32));
        let native = plan_native_scene(
            target,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            images,
            fonts,
        )
        .expect("CSS backgrounds must admit native rendering");
        assert_eq!(native.stats().bridge.referenced_sources, 1);
        assert_eq!(native.stats().bridge.referenced_rgba_bytes, 16);
        assert_eq!(native.stats().generated_gradient_sources, 0);
        assert_eq!(witness::packed_pixels(native.plan()), expected);
    }
    State {
        click,
        graph: doc
            .nodes
            .iter()
            .map(|node| (node.parent, node.children.clone()))
            .collect(),
        hits: layout
            .hit_regions
            .iter()
            .map(|hit| {
                (
                    hit.node,
                    hit.rect.x,
                    hit.rect.y,
                    hit.rect.width,
                    hit.rect.height,
                    hit.fixed,
                )
            })
            .collect(),
    }
}

#[test]
fn native_background_page_click_warm_cache_fixed_scroll_and_rounded_fallback() {
    let fixture = Fixture::new();
    let mut page = Page::load(&fixture.address, true).unwrap();
    let fonts = Fonts::new();
    let layout = page.prepare_layout(WIDTH as f32, HEIGHT as f32, &fonts);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let before = check(&page.document, &layout, &page.images, &fonts, 0);
    fs::remove_file(fixture.directory.join("background-tile.png")).unwrap();
    assert!(page.click(before.click).is_none());
    assert_eq!(page.title(), "Positioned image background");
    let layout = page.prepare_layout(WIDTH as f32, HEIGHT as f32, &fonts);
    assert_eq!(
        check(&page.document, &layout, &page.images, &fonts, 1),
        before
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert!(page.runtime.console.is_empty());
    page.document.set_attr(
        before.click,
        "style",
        "background-repeat:no-repeat;background-position:right bottom;border-radius:4px",
    );
    let rounded = page.prepare_layout(WIDTH as f32, HEIGHT as f32, &fonts);
    assert!(
        rounded
            .commands
            .iter()
            .any(|c| matches!(c, DrawCommand::BackgroundImage { radius, .. } if *radius == 4.0))
    );
    let frame = Frame::new(WIDTH, HEIGHT, 0xffffff);
    let refusal = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &rounded.commands,
        }],
        &page.images,
        &fonts,
    )
    .unwrap_err();
    assert_eq!(refusal.category(), "unsupported-background-image");
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn native_background_confined_worker_click_preserves_pixels_graph_and_image_snapshot() {
    use eris::{
        page::Navigation,
        worker::{Command, WorkerClient},
    };
    use std::path::Path;
    let fixture = Fixture::new();
    let navigation = Navigation::get(&fixture.address);
    let mut client = WorkerClient::spawn_at(
        Path::new(env!("CARGO_BIN_EXE_eris-browser")),
        true,
        &navigation,
        257,
    )
    .expect("requires enforced Landlock ABI 6");
    let reply = client
        .exchange(Command::Load { navigation }, || false)
        .unwrap();
    assert!(reply.snapshot.is_none() && reply.navigation.is_none());
    let fonts = Fonts::new();
    let mut previous: Option<State> = None;
    for phase in 0..2 {
        if let Some(before) = &previous {
            fs::remove_file(fixture.directory.join("background-tile.png")).unwrap();
            let reply = client
                .exchange(Command::Click { node: before.click }, || false)
                .unwrap();
            assert!(reply.snapshot.is_none() && reply.navigation.is_none());
        }
        let reply = client
            .exchange(
                Command::Render {
                    width: WIDTH as f32,
                    height: HEIGHT as f32,
                },
                || false,
            )
            .unwrap();
        assert!(reply.navigation.is_none());
        let snapshot = reply.snapshot.unwrap();
        assert_eq!(snapshot.generation, 257);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        assert_eq!(
            snapshot.title,
            if phase == 0 {
                "Repeated image background"
            } else {
                "Positioned image background"
            }
        );
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|message| message.starts_with("Page process ")
                    || message.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        let next = check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
        );
        if let Some(before) = &previous {
            assert_eq!(&next, before);
        }
        previous = Some(next);
    }
}
