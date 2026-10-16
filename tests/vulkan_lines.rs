//! Real loaded Page and complete native-plan comparison, without GPU execution.
#![cfg(feature = "raster-bridge")]

#[path = "support/vulkan_lines.rs"]
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
    fn new() -> Self {
        let directory = loop {
            let candidate = std::env::temp_dir().join(format!(
                "eris-native-lines-{}-{}",
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
        fs::write(&path, witness::HTML).unwrap();
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
fn native_line_loaded_decorations_admit_exact_plan_and_literal_pixels() {
    let fixture = Fixture::new();
    let page = Page::load(&fixture.address, false).unwrap();
    let fonts = Fonts::new();
    let layout = page.layout(witness::WIDTH as f32, witness::HEIGHT as f32, &fonts);
    witness::check_layout(&page, &layout);
    witness::check_pixels_and_native(&page, &layout, &fonts);
}
