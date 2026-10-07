//! Literal Number text and Canvas expectations across Page and worker clicks.
use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

const HTML: &str = include_str!("../examples/number-format.html");
const REFERENCES: [&str; 2] = [
    include_str!("fixtures/number-format-positive.html"),
    include_str!("fixtures/number-format-negative.html"),
];
const TITLES: [&str; 2] = ["Positive amount", "Negative amount"];
const VALUES: [[&str; 3]; 2] = [
    [
        "1000000000000000.2",
        "[1000000000000000.2]",
        "1000000000000000.2",
    ],
    [
        "-1000000000000000.2",
        "[-1000000000000000.2]",
        "-1000000000000000.2",
    ],
];
const WIDTH: u32 = 640;
const HEIGHT: u32 = 240;
const HIT: (f32, f32) = (32.0, 200.0);
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new() -> Self {
        let directory = loop {
            let candidate = std::env::temp_dir().join(format!(
                "eris-number-format-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture directory: {error}"),
            }
        };
        let path = directory.join("index.html");
        fs::write(&path, HTML).unwrap();
        Self {
            directory,
            address: Url::from_file_path(path).unwrap().to_string(),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

struct State {
    ids: Vec<NodeId>,
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    pixels: Vec<u32>,
    click: NodeId,
}

fn only_text(doc: &Document, parent: NodeId, expected: &str) -> NodeId {
    assert_eq!(doc.nodes[parent].children.len(), 1);
    let id = doc.nodes[parent].children[0];
    assert_eq!(doc.nodes[id].parent, Some(parent));
    assert!(doc.nodes[id].children.is_empty());
    let NodeKind::Text(data) = &doc.nodes[id].kind else {
        panic!("expected retained Text at {id}");
    };
    assert_eq!(data.scalar(), Some(expected));
    assert_eq!(
        data.units().collect::<Vec<_>>(),
        expected.encode_utf16().collect::<Vec<_>>()
    );
    id
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

fn check(
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
    assert_eq!(doc.title(), TITLES[phase]);
    let find = |selector| doc.query_selector(selector).unwrap();
    let body = find("body");
    assert_eq!(
        doc.attr(body, "data-phase"),
        Some(["positive", "negative"][phase])
    );
    let mut ids = vec![doc.root, body, find("#swatch")];
    for (selector, text) in [
        ("#title", TITLES[phase]),
        ("#heading", "Number display"),
        ("#plain", VALUES[phase][0]),
        ("#export", VALUES[phase][1]),
        ("#saved", VALUES[phase][2]),
        ("#state", TITLES[phase]),
        ("#change", "Change sign"),
    ] {
        let element = find(selector);
        ids.push(element);
        ids.push(only_text(doc, element, text));
    }
    assert_eq!(ids.len(), 17);
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[..index].contains(id));
    }
    let button = find("#change");
    let button_text = doc.nodes[button].children[0];
    let click = layout.hit_test(HIT.0, HIT.1).unwrap();
    assert!(
        click == button || click == button_text,
        "button hit or its Text"
    );
    assert!(layout.content_height <= HEIGHT as f32);
    // These amounts contain no spaces, so each must be one visible Text run.
    for (literal, count) in [(VALUES[phase][0], 2), (VALUES[phase][1], 1)] {
        assert_eq!(
            layout
                .commands
                .iter()
                .filter(|command| {
                    matches!(command, DrawCommand::Text { text, .. } if text == literal)
                })
                .count(),
            count
        );
    }
    // Parse independent literal HTML with scripts disabled, never an observed
    // subtree or output-derived string. Compare the complete CPU Canvas.
    let reference = Page::from_html(doc.url().clone(), REFERENCES[phase], false);
    assert!(
        reference.diagnostics.is_empty(),
        "{:?}",
        reference.diagnostics
    );
    assert!(reference.runtime.console.is_empty());
    let expected_layout = reference.layout(WIDTH as f32, HEIGHT as f32, fonts);
    let actual = paint(layout, fonts, images);
    let expected = paint(&expected_layout, fonts, &reference.images);
    assert_eq!(
        actual.pixels, expected.pixels,
        "literal phase {phase} canvas"
    );
    assert_eq!(
        actual.pixels[200 * WIDTH as usize + 200],
        [0x008000, 0x0000ff][phase]
    );
    assert_eq!(actual.pixels[230 * WIDTH as usize + 620], 0xffffff);
    // Independent text-ink witness rejects a blank canvas even if both layouts
    // stopped painting text. The three rows all occupy this fixed region.
    for top in [56, 96, 136] {
        assert!((top..top + 24).any(|y| {
            actual.pixels[y * WIDTH as usize + 208..y * WIDTH as usize + 600]
                .iter()
                .any(|pixel| *pixel != 0xffffff)
        }));
    }
    let graph = doc
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect();
    if let Some(previous) = previous {
        assert_eq!(ids, previous.ids, "same elements and Text after real click");
        assert_eq!(
            graph, previous.graph,
            "click creates or detaches no DOM nodes"
        );
        assert_ne!(
            actual.pixels, previous.pixels,
            "event changes visible output"
        );
    }
    State {
        ids,
        graph,
        pixels: actual.pixels,
        click,
    }
}

#[test]
fn number_format_loaded_page_changes_literal_text_and_pixels_after_click() {
    let fixture = Fixture::new();
    let mut page = Page::load(&fixture.address, true).unwrap();
    let fonts = Fonts::new();
    let mut previous: Option<State> = None;
    for (phase, title) in TITLES.iter().enumerate() {
        if let Some(previous) = &previous {
            assert!(page.click(previous.click).is_none());
        }
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        assert!(
            page.runtime.console.is_empty(),
            "{:?}",
            page.runtime.console
        );
        assert_eq!(page.title(), *title);
        let layout = page.layout(WIDTH as f32, HEIGHT as f32, &fonts);
        previous = Some(check(
            &page.document,
            &layout,
            &page.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn number_format_confined_worker_changes_literal_text_and_pixels_after_click() {
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
        219,
    )
    .expect("real worker requires fully enforced Linux Landlock ABI 6");
    let reply = client
        .exchange(Command::Load { navigation }, || false)
        .unwrap();
    assert!(reply.snapshot.is_none());
    assert!(reply.navigation.is_none());
    let fonts = Fonts::new();
    let mut previous: Option<State> = None;
    for (phase, title) in TITLES.iter().enumerate() {
        if let Some(previous) = &previous {
            let reply = client
                .exchange(
                    Command::Click {
                        node: previous.click,
                    },
                    || false,
                )
                .unwrap();
            assert!(reply.snapshot.is_none());
            assert!(reply.navigation.is_none());
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
        assert_eq!(snapshot.generation, 219);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        assert!(
            snapshot.diagnostics.iter().all(|line| {
                line.starts_with("Page process ") || line.starts_with("Resource broker ")
            }),
            "{:?}",
            snapshot.diagnostics
        );
        assert_eq!(snapshot.title, *title);
        previous = Some(check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
    drop(client); // Reap worker/broker before the owned fixture is removed.
}
