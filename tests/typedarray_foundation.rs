//! Literal TypedArray results across a loaded Page and a confined worker click.
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

const HTML: &str =
    include_str!("conformance/typedarray-foundation-browser/typedarray-foundation.html");
const REFERENCES: [&str; 2] = [
    include_str!("conformance/typedarray-foundation-browser/reference-initial.html"),
    include_str!("conformance/typedarray-foundation-browser/reference-after.html"),
];
const REFERENCE_URL: &str = "https://example.test/typedarray-foundation.html";
const TITLES: [&str; 2] = ["TypedArray ready", "TypedArray changed"];
const WORDS: [&str; 2] = ["Ready", "Changed"];
const STATUS: [&str; 2] = ["2,1 | 5,258,0 | detached", "1,2 | 3,258,513 | detached"];
const WIDTH: u32 = 320;
const HEIGHT: u32 = 160;
const HIT: (f32, f32) = (20.0, 20.0);
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    address: String,
}

impl Fixture {
    fn new() -> Self {
        let directory = loop {
            let candidate = std::env::temp_dir().join(format!(
                "eris-typedarray-foundation-{}-{}",
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
    let change = find("#change");
    let word = find("#word");
    let status = find("#status");
    assert_eq!(doc.attr(body, "data-phase"), Some(["0", "1"][phase]));
    let ids = vec![
        body,
        only_text(doc, find("title"), TITLES[phase]),
        change,
        word,
        only_text(doc, word, WORDS[phase]),
        status,
        only_text(doc, status, STATUS[phase]),
    ];
    assert_eq!(ids.len(), 7);
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[..index].contains(id));
    }
    assert!(doc.nodes[change].children.is_empty());
    let click = layout.hit_test(HIT.0, HIT.1).unwrap();
    assert_eq!(click, change, "real hit on the empty marker div");
    assert!(layout.content_height <= HEIGHT as f32);

    // Normal inline layout emits separate word and space commands. Rebuild
    // each fixed visible band in command order, retaining every literal space.
    for (top, expected) in [(16.0, WORDS[phase]), (72.0, STATUS[phase])] {
        let mut visible = String::new();
        for command in &layout.commands {
            if let DrawCommand::Text { y, text, .. } = command
                && *y >= top
                && *y < top + 32.0
            {
                visible.push_str(text);
            }
        }
        assert_eq!(visible, expected, "literal display text at y={top}");
    }

    // References are frozen script-free literal HTML, never reconstructed
    // from the candidate DOM or pixels. They have no network resources.
    let reference = Page::from_html(Url::parse(REFERENCE_URL).unwrap(), REFERENCES[phase], false);
    assert!(
        reference.diagnostics.is_empty(),
        "{:?}",
        reference.diagnostics
    );
    assert!(reference.runtime.console.is_empty());
    assert!(reference.images.is_empty());
    let expected_layout = reference.layout(WIDTH as f32, HEIGHT as f32, fonts);
    let actual = paint(layout, fonts, images);
    let expected = paint(&expected_layout, fonts, &reference.images);
    assert_eq!(
        actual.pixels, expected.pixels,
        "literal phase {phase} canvas"
    );
    let marker = [0x008000, 0x0000ff][phase];
    for y in 16..48 {
        assert!(
            actual.pixels[y * WIDTH as usize + 16..y * WIDTH as usize + 80]
                .iter()
                .all(|pixel| *pixel == marker)
        );
    }
    assert_eq!(actual.pixels[20 * WIDTH as usize + 20], marker);
    assert_eq!(actual.pixels[140 * WIDTH as usize + 300], 0xffffff);
    // Independent ink checks keep a matching blank/reference failure from
    // satisfying full-canvas equality.
    for (left, top, right) in [(96, 16, 240), (16, 72, 304)] {
        assert!((top..top + 32).any(|y| {
            actual.pixels[y * WIDTH as usize + left..y * WIDTH as usize + right]
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
        assert_eq!(ids, previous.ids, "same seven nodes after the real click");
        assert_eq!(doc.nodes.len(), previous.graph.len());
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
fn typedarray_foundation_loaded_page_preserves_views_across_real_click() {
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
fn typedarray_foundation_confined_worker_preserves_views_across_real_click() {
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
        220,
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
        assert_eq!(snapshot.generation, 220);
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
    drop(client); // Reap worker and broker before removing the owned fixture.
}
