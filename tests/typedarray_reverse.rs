//! A literal buffer-reverse page through direct rendering and real worker IPC.
use eris::{
    dom::{Document, NodeId},
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

const HTML: &str = include_str!("../examples/typedarray-reverse.html");
const TITLES: [&str; 3] = ["Buffer reverse", "Buffer reversed", "Buffer reversed"];
const EXPECTED: [&str; 3] = [
    "Buffer: 99 11 22 33 44 88\nSelected: 11 22 33 44\nByte length: 12\nSame view: ready",
    "Buffer: 99 44 33 22 11 88\nSelected: 44 33 22 11\nByte length: 12\nSame view: true",
    "Buffer: 99 11 22 33 44 88\nSelected: 11 22 33 44\nByte length: 12\nSame view: true",
];
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new() -> Self {
        let directory = loop {
            let path = std::env::temp_dir().join(format!(
                "eris-reverse-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture: {error}"),
            }
        };
        let path = directory.join("index.html");
        fs::write(&path, HTML).unwrap();
        Self {
            address: Url::from_file_path(path).unwrap().to_string(),
            directory,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

struct State {
    click: NodeId,
    output_pixels: Vec<u32>,
}

fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&State>,
) -> State {
    assert_eq!(doc.title(), TITLES[phase]);
    let button = doc.query_selector("#change").unwrap();
    let result = doc.query_selector("#result").unwrap();
    assert_eq!(doc.text_content(result), EXPECTED[phase]);
    let hit = layout
        .hit_regions
        .iter()
        .find(|hit| hit.node == button)
        .unwrap();
    let click = layout.hit_test(hit.rect.x + 1.0, hit.rect.y + 1.0).unwrap();
    assert_eq!(click, button);
    let output = layout
        .hit_regions
        .iter()
        .find(|hit| hit.node == result)
        .unwrap()
        .rect;
    assert!(output.width > 2.0 && output.height > 2.0);
    assert!(output.x >= 0.0 && output.y >= 0.0);
    assert!(output.x + output.width <= 640.0 && output.y + output.height <= 480.0);
    let mut lines: Vec<(f32, String)> = Vec::new();
    for command in &layout.commands {
        if let DrawCommand::Text { x, y, text, .. } = command
            && *x >= output.x
            && *x < output.x + output.width
            && *y >= output.y
            && *y < output.y + output.height
        {
            if let Some((top, line)) = lines.last_mut()
                && *top == *y
            {
                line.push_str(text);
            } else {
                lines.push((*y, text.clone()));
            }
        }
    }
    assert_eq!(
        lines
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>(),
        EXPECTED[phase].split('\n').collect::<Vec<_>>()
    );
    let mut canvas = Canvas::new(640, 480).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert!(canvas.pixels.contains(&0x245bcc));
    let mut output_pixels = Vec::new();
    for y in output.y.ceil() as usize + 1..(output.y + output.height).floor() as usize - 1 {
        let left = output.x.ceil() as usize + 1;
        let right = (output.x + output.width).floor() as usize - 1;
        output_pixels.extend_from_slice(&canvas.pixels[y * 640 + left..y * 640 + right]);
    }
    assert!(output_pixels.contains(&0x172033));
    if let Some(before) = previous {
        assert_eq!(click, before.click);
        assert_ne!(output_pixels, before.output_pixels);
    }
    State {
        click,
        output_pixels,
    }
}

#[test]
fn typedarray_reverse_page_renders_shared_bytes_before_and_after_click() {
    let fixture = Fixture::new();
    let mut page = Page::load(&fixture.address, true).unwrap();
    let fonts = Fonts::new();
    let mut previous: Option<State> = None;
    for phase in 0..3 {
        if let Some(before) = &previous {
            assert!(page.click(before.click).is_none());
        }
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        assert!(
            page.runtime.console.is_empty(),
            "{:?}",
            page.runtime.console
        );
        let layout = page.layout(640.0, 480.0, &fonts);
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
fn typedarray_reverse_confined_worker_renders_shared_bytes_before_and_after_click() {
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
        227,
    )
    .expect("real worker requires fully enforced Linux Landlock ABI 6");
    let reply = client
        .exchange(Command::Load { navigation }, || false)
        .unwrap();
    assert!(reply.snapshot.is_none() && reply.navigation.is_none());
    let fonts = Fonts::new();
    let mut previous: Option<State> = None;
    for (phase, title) in TITLES.iter().enumerate() {
        if let Some(before) = &previous {
            let reply = client
                .exchange(Command::Click { node: before.click }, || false)
                .unwrap();
            assert!(reply.snapshot.is_none() && reply.navigation.is_none());
        }
        let reply = client
            .exchange(
                Command::Render {
                    width: 640.0,
                    height: 480.0,
                },
                || false,
            )
            .unwrap();
        assert!(reply.navigation.is_none());
        let snapshot = reply.snapshot.unwrap();
        assert_eq!(snapshot.generation, 227);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        assert_eq!(snapshot.title, *title);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|line| line.starts_with("Page process ")
                    || line.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        previous = Some(check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
    drop(client);
}
