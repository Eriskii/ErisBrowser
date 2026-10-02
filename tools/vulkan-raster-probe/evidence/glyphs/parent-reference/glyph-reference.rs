//! Immutable-parent CPU reference harness. No GPU/planner or new Fonts API.
#![forbid(unsafe_code)]
#[path = "../../../font_inventory.rs"]
mod font_inventory;
use eris::graphics::{Canvas, Color, Fonts};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn paint(fixture: &font_inventory::FontFixture, fonts: &Fonts) -> Result<Vec<u32>, String> {
    let f = &fixture.frame;
    if f.width == 0 || f.height == 0 || f.width > 320 || f.height > 240 || f.clear >> 24 != 0 {
        return Err("invalid frozen frame".into());
    }
    let mut canvas = Canvas::new(f.width, f.height)?;
    canvas.clear(Color::rgb(
        (f.clear >> 16) as u8,
        (f.clear >> 8) as u8,
        f.clear as u8,
    ));
    canvas.set_clip(f.caller_clip);
    canvas.paint_with_viewport(
        &fixture.commands,
        fonts,
        &fixture.images,
        f.document_offset,
        f.viewport_offset,
    );
    if canvas.exhausted() || canvas.pixels.iter().any(|p| p >> 24 != 0) {
        return Err("reference paint exhausted or target high byte set".into());
    }
    Ok(canvas.pixels)
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 || args[1] != "--output-dir" {
        return Err("usage: glyph-reference --output-dir NEW_DIRECTORY".into());
    }
    let out = PathBuf::from(&args[2]);
    // create_dir deliberately rejects existing paths, including prior partial results.
    fs::create_dir(&out).map_err(|e| format!("create fresh output directory: {e}"))?;
    let fixtures = font_inventory::fixtures();
    if fixtures.len() != 14 {
        return Err("frozen font inventory count changed".into());
    }
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0usize;
    for fixture in fixtures {
        if !fixture
            .name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !names.insert(fixture.name)
        {
            return Err("invalid or repeated frozen name".into());
        }
        let fonts = Fonts::new();
        let cold = paint(&fixture, &fonts)?;
        let warm = paint(&fixture, &fonts)?;
        if cold != warm {
            return Err(format!("cold/warm parent disagreement: {}", fixture.name));
        }
        let bytes = cold.len().checked_mul(4).ok_or("output size overflow")?;
        let path = out.join(format!("{}.rgb", fixture.name));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        for pixel in cold {
            file.write_all(&pixel.to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        file.sync_all().map_err(|e| e.to_string())?;
        total = total.checked_add(bytes).ok_or("total overflow")?;
        println!(
            "REFERENCE name={} width={} height={} bytes={} encoding=u32le-00RRGGBB cold_warm=equal exhausted=false",
            fixture.name, fixture.frame.width, fixture.frame.height, bytes
        );
    }
    println!(
        "REFERENCE_COMPLETE cases=14 bytes={total} parent=5ad2cfd3582f92f8368a0a0fa2fd3bb7dd7af2f0"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
