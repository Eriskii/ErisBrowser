//! Loaded image fitting through direct Page rendering and confined worker IPC.
#[path = "support/object_fit_position.rs"]
mod witness;

use eris::{graphics::Fonts, page::Page};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new() -> Self {
        let directory = loop {
            let path = std::env::temp_dir().join(format!(
                "eris-object-fit-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture directory: {error}"),
            }
        };
        let path = directory.join("index.html");
        fs::write(&path, witness::HTML).unwrap();
        fs::write(directory.join("object-fit-position.png"), witness::PNG).unwrap();
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

#[test]
fn object_fit_page_preserves_boxes_hits_and_graph_across_style_click() {
    let fixture = Fixture::new();
    let mut page = Page::load(&fixture.address, true).unwrap();
    let fonts = Fonts::new();
    let mut previous: Option<witness::State> = None;
    for phase in 0..2 {
        if let Some(before) = &previous {
            assert!(page.click(before.click).is_none());
        }
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        assert!(
            page.runtime.console.is_empty(),
            "{:?}",
            page.runtime.console
        );
        let layout = page.layout(witness::WIDTH as f32, witness::HEIGHT as f32, &fonts);
        previous = Some(witness::check(
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
fn object_fit_confined_worker_preserves_boxes_hits_and_graph_across_style_click() {
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
        231,
    )
    .expect("real worker requires fully enforced Linux Landlock ABI 6");
    let reply = client
        .exchange(Command::Load { navigation }, || false)
        .unwrap();
    assert!(reply.snapshot.is_none() && reply.navigation.is_none());
    let fonts = Fonts::new();
    let mut previous: Option<witness::State> = None;
    for phase in 0..2 {
        if let Some(before) = &previous {
            let reply = client
                .exchange(Command::Click { node: before.click }, || false)
                .unwrap();
            assert!(reply.snapshot.is_none() && reply.navigation.is_none());
        }
        let reply = client
            .exchange(
                Command::Render {
                    width: witness::WIDTH as f32,
                    height: witness::HEIGHT as f32,
                },
                || false,
            )
            .unwrap();
        assert!(reply.navigation.is_none());
        let snapshot = reply.snapshot.unwrap();
        assert_eq!(snapshot.generation, 231);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        assert_eq!(snapshot.title, witness::metadata(phase));
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|message| message.starts_with("Page process ")
                    || message.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        previous = Some(witness::check(
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
