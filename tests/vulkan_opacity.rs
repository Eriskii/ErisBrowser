//! Loaded Page and confined-worker opacity witnesses; these do not execute GPU code.
#![cfg(feature = "raster-bridge")]

#[path = "support/vulkan_opacity.rs"]
mod witness;

use eris::{graphics::Fonts, page::Page};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new(scene: witness::Scene) -> Self {
        let directory = loop {
            let candidate = std::env::temp_dir().join(format!(
                "eris-vulkan-opacity-{}-{}",
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
        fs::write(&path, scene.html()).unwrap();
        fs::write(directory.join("vulkan-opacity.png"), witness::PNG).unwrap();
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

#[test]
fn loaded_opacity_click_retains_ids_and_matches_native_canvas() {
    for scene in [witness::Scene::Public, witness::Scene::Button] {
        let fixture = Fixture::new(scene);
        let mut page = Page::load(&fixture.address, true).unwrap();
        let fonts = Fonts::new();
        let mut previous = None;
        let mut clicked = None;
        for phase in 0..2 {
            if let Some(node) = clicked {
                assert!(page.click(node).is_none());
            }
            assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
            assert!(
                page.runtime.console.is_empty(),
                "{:?}",
                page.runtime.console
            );
            assert_eq!(page.title(), witness::metadata(phase));
            let layout = page.layout(witness::WIDTH as f32, witness::HEIGHT as f32, &fonts);
            let (x, y) = scene.hit();
            clicked = Some(layout.hit_test(x, y).unwrap());
            previous = Some(witness::check(
                &page.document,
                &layout,
                &page.images,
                &fonts,
                scene,
                phase,
                previous.as_ref(),
            ));
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_opacity_click_retains_ids_and_matches_native_canvas() {
    use eris::{
        page::Navigation,
        worker::{Command, WorkerClient},
    };
    use std::path::Path;
    for scene in [witness::Scene::Public, witness::Scene::Button] {
        let fixture = Fixture::new(scene);
        let navigation = Navigation::get(&fixture.address);
        let mut client = WorkerClient::spawn_at(
            Path::new(env!("CARGO_BIN_EXE_eris-browser")),
            true,
            &navigation,
            217,
        )
        .expect("real worker requires fully enforced Linux Landlock ABI 6");
        let reply = client
            .exchange(Command::Load { navigation }, || false)
            .unwrap();
        assert!(reply.snapshot.is_none());
        assert!(reply.navigation.is_none());
        let fonts = Fonts::new();
        let mut previous = None;
        let mut clicked = None;
        for phase in 0..2 {
            if let Some(node) = clicked {
                let reply = client.exchange(Command::Click { node }, || false).unwrap();
                assert!(reply.snapshot.is_none());
                assert!(reply.navigation.is_none());
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
            assert_eq!(snapshot.generation, 217);
            assert_eq!(snapshot.processed_edit_sequence, 0);
            assert!(
                snapshot
                    .diagnostics
                    .iter()
                    .all(|message| message.starts_with("Page process ")
                        || message.starts_with("Resource broker ")),
                "{:?}",
                snapshot.diagnostics
            );
            assert_eq!(snapshot.title, witness::metadata(phase));
            let (x, y) = scene.hit();
            clicked = Some(snapshot.layout.hit_test(x, y).unwrap());
            previous = Some(witness::check(
                &snapshot.document,
                &snapshot.layout,
                &snapshot.images,
                &fonts,
                scene,
                phase,
                previous.as_ref(),
            ));
        }
        drop(client); // Worker and broker cleanup precede fixture removal.
    }
}
