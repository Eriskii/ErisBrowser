//! Native gradient admission through direct Page and confined worker IPC.
#![cfg(feature = "raster-bridge")]
#[path = "support/vulkan_gradients.rs"]
mod witness;

use eris::{graphics::Fonts, page::Page};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

// The predecessor wrapper retains this source with only false substituted.
const EXPECT_NATIVE: bool = true;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new() -> Self {
        let directory = loop {
            let path = std::env::temp_dir().join(format!(
                "eris-vulkan-gradients-{}-{}",
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
fn vulkan_gradients_page_preserves_boxes_hits_and_graph_across_style_click() {
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
            EXPECT_NATIVE,
        ));
    }
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn vulkan_gradients_confined_worker_preserves_boxes_hits_and_graph_across_style_click() {
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
        243,
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
        assert_eq!(snapshot.generation, 243);
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
            EXPECT_NATIVE,
        ));
    }
    drop(client);
}

#[test]
fn vulkan_gradients_direct_literals_cover_sampling_scopes_and_native_records() {
    let fonts = Fonts::new();
    let fixtures = witness::fixtures();
    assert_eq!(fixtures.len(), 3);
    assert_eq!(fixtures.iter().map(|f| f.expected.len()).sum::<usize>(), 76);
    for fixture in fixtures {
        assert_eq!(
            fixture.canvas_pixels(&fonts),
            fixture.expected,
            "{}",
            fixture.name
        );
        let native = fixture.native(&fonts);
        if EXPECT_NATIVE {
            let native = native.expect("frozen opaque gradient literal admits Native");
            assert_eq!(
                witness::packed_pixels(native.plan()),
                fixture.expected,
                "{}",
                fixture.name
            );
        } else {
            assert!(native.unwrap_err().starts_with("unsupported-gradient"));
        }
    }
}
