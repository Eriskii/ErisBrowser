//! Loaded alpha and rasterized external SVG use the same fitted image path.
use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts},
    page::Page,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 4, 0, 0, 0, 2, 8, 6, 0,
    0, 0, 127, 168, 125, 99, 0, 0, 0, 21, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 208, 0, 194,
    12, 12, 64, 18, 132, 209, 5, 0, 178, 150, 9, 249, 244, 19, 30, 46, 0, 0, 0, 0, 73, 69, 78, 68,
    174, 66, 96, 130,
];
const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="2"><rect width="2" height="2" fill="red"/><rect x="2" width="2" height="2" fill="lime"/></svg>"##;
const HTML: &str = "<!doctype html><style>body{margin:0}img{display:block;width:4px;height:4px;object-fit:cover;background:blue}</style><img id=alpha width=40 height=20 src=alpha.png><img id=vector width=40 height=20 src=vector.svg><div style='width:4px;height:1px;background:black'></div>";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "eris-object-fit-sources-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => {
                    let fixture = Self(path);
                    fs::write(fixture.0.join("index.html"), HTML).unwrap();
                    fs::write(fixture.0.join("alpha.png"), PNG).unwrap();
                    fs::write(fixture.0.join("vector.svg"), SVG).unwrap();
                    return fixture;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("fixture directory: {error}"),
            }
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn object_fit_loaded_alpha_and_external_svg_preserve_background_and_sampling() {
    let fixture = Fixture::new();
    let url = Url::from_file_path(fixture.0.join("index.html")).unwrap();
    let page = Page::load(url.as_str(), false).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert_eq!(page.images.len(), 2);
    let alpha = page.images.get("alpha.png").unwrap();
    let vector = page.images.get("vector.svg").unwrap();
    assert_eq!((alpha.width, alpha.height), (4, 2));
    assert_eq!((vector.width, vector.height), (4, 2));
    assert_eq!(&alpha.rgba[..8], &[255, 0, 0, 128, 255, 0, 0, 128]);
    assert_eq!(&alpha.rgba[8..16], &[0, 0, 255, 0, 0, 0, 255, 0]);
    assert_eq!(
        &vector.rgba[..16],
        &[
            255, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255
        ]
    );
    for name in ["#alpha", "#vector"] {
        let id = page.document.query_selector(name).unwrap();
        assert_eq!(page.document.attr(id, "data-eris-natural-width"), Some("4"));
        assert_eq!(
            page.document.attr(id, "data-eris-natural-height"),
            Some("2")
        );
        assert_eq!(page.document.attr(id, "width"), Some("40"));
    }
    let fonts = Fonts::new();
    let layout = page.layout(12.0, 12.0, &fonts);
    let destinations = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Image { rect, key } => {
                Some((key.as_str(), rect.x, rect.y, rect.width, rect.height))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        destinations,
        vec![
            ("alpha.png", -2.0, 0.0, 8.0, 4.0),
            ("vector.svg", -2.0, 4.0, 8.0, 4.0)
        ]
    );
    let mut canvas = Canvas::new(12, 12).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    // Literal resulting colors, with red alpha128 over blue = #80007f.
    for y in 0..12 {
        for x in 0..12 {
            let expected = match (x, y) {
                (0..=1, 0..=3) => 0x80007f,
                (2..=3, 0..=3) => 0x0000ff,
                (0..=1, 4..=7) => 0xff0000,
                (2..=3, 4..=7) => 0x00ff00,
                (0..=3, 8) => 0x000000,
                _ => 0xffffff,
            };
            assert_eq!(canvas.pixels[y * 12 + x], expected, "{x},{y}");
        }
    }
}
