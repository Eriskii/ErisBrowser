//! Principal background boxes through direct Page loading, preparation and CSSOM.
#[path = "support/background_boxes.rs"]
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
    fn new(paint: witness::Paint) -> Self {
        let directory = loop {
            let path = std::env::temp_dir().join(format!(
                "eris-background-boxes-{}-{}",
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
        fs::write(&path, paint.html()).unwrap();
        if paint == witness::Paint::Image {
            fs::write(directory.join("pattern.png"), witness::PNG).unwrap();
        }
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

fn run(paint: witness::Paint) {
    let fixture = Fixture::new(paint);
    let mut page = Page::load(&fixture.address, true).unwrap();
    let fonts = Fonts::new();
    let mut previous: Option<witness::Identity> = None;
    let mut removed = false;
    for changed in [false, true] {
        if changed {
            assert!(page.click(previous.as_ref().unwrap().click).is_none());
        }
        for width in [32, 20, 32] {
            let layout = page.prepare_layout(width as f32, witness::HEIGHT as f32, &fonts);
            assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
            assert!(
                page.runtime.console.is_empty(),
                "{:?}",
                page.runtime.console
            );
            previous = Some(witness::check(
                &page.document,
                &layout,
                &page.images,
                &fonts,
                witness::Case {
                    paint,
                    width,
                    changed,
                },
                previous.as_ref(),
            ));
            if paint == witness::Paint::Image && !removed {
                fs::remove_file(fixture.directory.join("pattern.png")).unwrap();
                removed = true;
            }
        }
    }
}

#[test]
fn gradient_origin_clip_mutation_and_resize_preserve_principal_pixels_hits_and_nodes() {
    run(witness::Paint::Gradient);
}

#[test]
fn prepared_url_origin_clip_mutation_and_resize_preserve_principal_pixels_hits_and_nodes() {
    run(witness::Paint::Image);
}
