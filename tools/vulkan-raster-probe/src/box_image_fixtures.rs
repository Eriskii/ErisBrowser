//! Literal BoxPaint::Image frames, independent of Canvas, Native LUTs and GPU output.
use crate::{Frame, Plan, Result, background_fixtures::witness::packed_pixels};
use eris::{
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore, RasterImage, Rect,
        box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    page::Page,
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

pub const FIXTURE_NAMES: [&str; 6] = [
    "box-image-four-repeat-modes",
    "box-image-fractional-tile-and-clip",
    "box-image-source-alpha-nested-opacity",
    "box-image-document-fixed-restoration",
    "box-image-empty-no-repeat-and-missing",
    "box-image-page-padding-origin-content-clip",
];
pub const LITERAL_PIXELS: usize = 268;
pub const CONVERTED_FRAMES: usize = 12;
pub const COMPARED_BYTES: u64 = 2144;
const GRID_RGBA: [u8; 16] = [
    255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255,
];
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub struct PreparedFixture {
    pub name: &'static str,
    pub expected: Vec<u32>,
    pub plan: Plan,
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn square(rect: Rect) -> RoundedBox {
    RoundedBox {
        rect,
        radii: [[0.0; 2]; 4],
    }
}
fn image(outer: Rect, tile: Rect, repeat: [bool; 2], key: &str) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: square(outer),
        cutout: None,
        paint: BoxPaint::Image {
            tile,
            repeat,
            key: key.into(),
        },
    }))
}
fn source(key: &str, width: u32, height: u32, rgba: &[u8]) -> ImageStore {
    ImageStore::from([(
        key.into(),
        Arc::new(RasterImage {
            width,
            height,
            rgba: rgba.to_vec(),
        }),
    )])
}
fn rgb(value: u32) -> Color {
    Color::rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
fn rows(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|pixel| match pixel {
            b'W' => 0xffffff,
            b'K' => 0x000000,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            b'G' => 0x00ff00,
            b'Y' => 0xffff00,
            b'P' => 0xff7f7f,
            b'Q' => 0xffbfbf,
            b'D' => 0xffdfdf,
            _ => unreachable!("fixed literal palette"),
        })
        .collect()
}
fn compare(name: &str, route: &str, width: u32, actual: &[u32], expected: &[u32]) -> Result<()> {
    if actual.len() != expected.len() {
        return Err(format!("{name} {route} literal length"));
    }
    if let Some((at, (actual, expected))) = actual
        .iter()
        .zip(expected)
        .enumerate()
        .find(|(_, (actual, expected))| actual != expected)
    {
        return Err(format!(
            "{name} {route} pixel ({},{}) actual={actual:08x} expected={expected:08x}",
            at % width as usize,
            at / width as usize
        ));
    }
    Ok(())
}

// Literal inventory: admitted BoxImage commands, B+T+L work, missing image count.
fn prepare(
    name: &'static str,
    frame: Frame,
    commands: &[DrawCommand],
    images: &ImageStore,
    expected: Vec<u32>,
    inventory: (usize, u64, usize),
    fonts: &Fonts,
) -> Result<PreparedFixture> {
    if expected.len() != frame.width as usize * frame.height as usize {
        return Err(format!("{name} literal dimensions"));
    }
    let mut canvas = Canvas::new(frame.width, frame.height)?;
    canvas.clear(rgb(frame.clear));
    let clip = frame.caller_clip;
    canvas.set_clip(rect(clip.x, clip.y, clip.width, clip.height));
    canvas.paint_with_viewport(
        commands,
        fonts,
        images,
        frame.document_offset,
        frame.viewport_offset,
    );
    if canvas.exhausted() {
        return Err(format!("{name} Canvas exhausted"));
    }
    compare(name, "Canvas", frame.width, &canvas.pixels, &expected)?;
    let native = plan_native_scene(
        Frame::new(frame.width, frame.height, frame.clear),
        &[NativePhase { frame, commands }],
        images,
        fonts,
    )
    .map_err(|error| format!("{name} native refusal: {error}"))?;
    let stats = native.stats();
    if (
        stats.box_background_images,
        stats.box_loop_work,
        stats.bridge.missing_images,
    ) != inventory
    {
        return Err(format!(
            "{name} box image/work/missing inventory actual={}/{}/{} expected={inventory:?}",
            stats.box_background_images, stats.box_loop_work, stats.bridge.missing_images
        ));
    }
    if stats.box_masks != 0
        || stats.mask_sources != 0
        || stats.coverage_bytes != 0
        || stats.row_entries != 0
        || native.plan().has_glyphs()
        || stats.generated_gradient_sources != 0
        || stats.bridge.referenced_sources != 1
    {
        return Err(format!("{name} unexpected mask/generated/source inventory"));
    }
    if name == FIXTURE_NAMES[2] && native.plan().group_scratch_bytes() == 0 {
        return Err("image alpha fixture lost isolated GPU groups".into());
    }
    compare(
        name,
        "packed Native",
        frame.width,
        &packed_pixels(native.plan()),
        &expected,
    )?;
    Ok(PreparedFixture {
        name,
        expected,
        plan: native.into_plan(),
    })
}

pub fn fixtures() -> Result<Vec<PreparedFixture>> {
    let fonts = Fonts::new();
    let grid = source("grid", 2, 2, &GRID_RGBA);
    let mut cases = Vec::with_capacity(FIXTURE_NAMES.len());
    let mut commands = Vec::new();
    for (band, repeat) in [[false, false], [true, false], [false, true], [true, true]]
        .into_iter()
        .enumerate()
    {
        let y = (band * 4) as f32;
        commands.push(image(
            rect(0.0, y, 8.0, 4.0),
            rect(2.0, y + 1.0, 4.0, 2.0),
            repeat,
            "grid",
        ));
    }
    cases.push(prepare(
        FIXTURE_NAMES[0],
        Frame::new(8, 16, 0xffffff),
        &commands,
        &grid,
        rows(&[
            "WWWWWWWW", "WWRRBBWW", "WWGGYYWW", "WWWWWWWW", "WWWWWWWW", "BBRRBBRR", "YYGGYYGG",
            "WWWWWWWW", "WWGGYYWW", "WWRRBBWW", "WWGGYYWW", "WWRRBBWW", "YYGGYYGG", "BBRRBBRR",
            "YYGGYYGG", "BBRRBBRR",
        ]),
        (4, 796, 0),
        &fonts,
    )?);

    let mut clipped = Frame::new(6, 4, 0xffffff);
    clipped.caller_clip = eris_raster_core::Rect::new(1.25, 0.25, 3.5, 2.5);
    cases.push(prepare(
        FIXTURE_NAMES[1],
        clipped,
        &[image(
            rect(0.0, 0.0, 6.0, 4.0),
            rect(-0.75, -0.25, 2.0, 2.0),
            [true, true],
            "grid",
        )],
        &grid,
        rows(&["WWWWWW", "WWYGYW", "WWBRBW", "WWWWWW"]),
        (1, 86, 0),
        &fonts,
    )?);

    let alpha = source("alpha", 2, 1, &[255, 0, 0, 128, 0, 0, 255, 0]);
    let mut commands = Vec::new();
    for depth in 0..3 {
        for _ in 0..depth {
            commands.push(DrawCommand::PushOpacity { opacity: 0.5 });
        }
        commands.push(image(
            rect(0.0, depth as f32, 4.0, 1.0),
            rect(0.0, depth as f32, 2.0, 1.0),
            [true, false],
            "alpha",
        ));
        for _ in 0..depth {
            commands.push(DrawCommand::PopOpacity);
        }
    }
    cases.push(prepare(
        FIXTURE_NAMES[2],
        Frame::new(4, 3, 0xffffff),
        &commands,
        &alpha,
        rows(&["PWPW", "QWQW", "DWDW"]),
        (3, 201, 0),
        &fonts,
    )?);

    let mut fixed = Frame::new(6, 4, 0xffffff);
    fixed.caller_clip = eris_raster_core::Rect::new(0.25, 0.25, 5.5, 3.5);
    fixed.document_offset = (1.0, 1.0);
    cases.push(prepare(
        FIXTURE_NAMES[3],
        fixed,
        &[
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 3.0, 2.0),
            },
            image(
                rect(0.0, 0.0, 4.0, 2.0),
                rect(0.0, 0.0, 2.0, 2.0),
                [true, true],
                "grid",
            ),
            DrawCommand::PushFixed,
            image(
                rect(4.0, 1.0, 2.0, 2.0),
                rect(4.0, 1.0, 2.0, 2.0),
                [false, false],
                "grid",
            ),
            DrawCommand::PopFixed,
            DrawCommand::Rect {
                rect: rect(0.0, 0.0, 1.0, 1.0),
                color: Color::BLACK,
                radius: 0.0,
            },
            DrawCommand::PopClip,
            DrawCommand::Rect {
                rect: rect(0.0, 2.0, 1.0, 1.0),
                color: Color::rgb(0, 255, 0),
                radius: 0.0,
            },
        ],
        &grid,
        rows(&["WWWWWW", "WKBRRB", "WGYGGY", "WGWWWW"]),
        (2, 142, 0),
        &fonts,
    )?);

    cases.push(prepare(
        FIXTURE_NAMES[4],
        Frame::new(8, 4, 0xffffff),
        &[
            image(
                rect(0.0, 0.0, 8.0, 4.0),
                rect(20.0, 20.0, 4.0, 2.0),
                [false, false],
                "grid",
            ),
            image(
                rect(0.0, 0.0, 8.0, 4.0),
                rect(2.0, 1.0, 4.0, 2.0),
                [false, false],
                "missing",
            ),
            DrawCommand::Rect {
                rect: rect(0.0, 0.0, 1.0, 1.0),
                color: Color::BLACK,
                radius: 0.0,
            },
        ],
        &grid,
        rows(&["KWWWWWWW", "WWWWWWWW", "WWWWWWWW", "WWWWWWWW"]),
        (2, 202, 1),
        &fonts,
    )?);
    cases.push(page_content_clip(&fonts)?);
    if cases.iter().map(|case| case.name).collect::<Vec<_>>() != FIXTURE_NAMES
        || cases.iter().map(|case| case.expected.len()).sum::<usize>() != LITERAL_PIXELS
    {
        return Err("Box image fixture inventory".into());
    }
    Ok(cases)
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Result<Self> {
        for _ in 0..16 {
            let path = std::env::temp_dir().join(format!(
                "eris-box-image-probe-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("fixture directory: {error}")),
            }
        }
        Err("fixture directory collision limit".into())
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn page_image_key(commands: &[DrawCommand]) -> Result<&str> {
    let mut boxes = commands.iter().filter_map(|command| match command {
        DrawCommand::BoxDecoration(value) => Some(value.as_ref()),
        _ => None,
    });
    let value = boxes.next().ok_or("Page image box missing")?;
    if boxes.next().is_some()
        || value.shape != square(rect(2.0, 2.0, 4.0, 2.0))
        || value.cutout.is_some()
    {
        return Err("Page image selected content geometry".into());
    }
    match &value.paint {
        BoxPaint::Image { tile, repeat, key }
            if *tile == rect(3.0, 3.0, 4.0, 2.0) && *repeat == [false, false] =>
        {
            Ok(key)
        }
        _ => Err("Page image lost independent padding positioning tile".into()),
    }
}
fn page_content_clip(fonts: &Fonts) -> Result<PreparedFixture> {
    let directory = Directory::new()?;
    let html = directory.0.join("page.html");
    let png = directory.0.join("tile.png");
    fs::write(&html, include_bytes!("box_image_assets/page.html")).map_err(|e| e.to_string())?;
    fs::write(&png, include_bytes!("box_image_assets/tile.png")).map_err(|e| e.to_string())?;
    // Page parses this existing filesystem path into a file URL itself.
    let mut page = Page::load(html.to_str().ok_or("fixture path UTF8")?, false)?;
    let first = page.prepare_layout(8.0, 6.0, fonts);
    let key = page_image_key(&first.commands)?.to_string();
    let raster = page
        .images
        .get(&key)
        .cloned()
        .ok_or("Page decoded image missing")?;
    if raster.width != 2 || raster.height != 2 || raster.rgba != GRID_RGBA {
        return Err("Page image literal PNG decode".into());
    }
    let expected = rows(&[
        "WWWWWWWW", "WWWWWWWW", "WWWWWWWW", "WWWRRBWW", "WWWWWWWW", "WWWWWWWW",
    ]);
    let initial = prepare(
        FIXTURE_NAMES[5],
        Frame::new(8, 6, 0xffffff),
        &first.commands,
        &page.images,
        expected.clone(),
        (1, 61, 0),
        fonts,
    )?;
    drop(initial);
    let graph = page
        .document
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect::<Vec<_>>();
    let hits = first
        .hit_regions
        .iter()
        .map(|hit| (hit.node, hit.rect, hit.fixed))
        .collect::<Vec<_>>();
    fs::remove_file(&png).map_err(|e| e.to_string())?;
    let cached = page.prepare_layout(8.0, 6.0, fonts);
    if page_image_key(&cached.commands)? != key
        || !page
            .images
            .get(&key)
            .is_some_and(|image| Arc::ptr_eq(image, &raster))
        || page
            .document
            .nodes
            .iter()
            .map(|node| (node.parent, node.children.clone()))
            .collect::<Vec<_>>()
            != graph
        || cached
            .hit_regions
            .iter()
            .map(|hit| (hit.node, hit.rect, hit.fixed))
            .collect::<Vec<_>>()
            != hits
        || !page.diagnostics.is_empty()
    {
        return Err("Page cached image/key/graph/hits changed after removing source file".into());
    }
    prepare(
        FIXTURE_NAMES[5],
        Frame::new(8, 6, 0xffffff),
        &cached.commands,
        &page.images,
        expected,
        (1, 61, 0),
        fonts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_six_literal_canvas_and_packed_box_images_match_without_a_device() {
        let cases = fixtures().expect("all frozen Box images must admit and match");
        assert_eq!(
            cases.iter().map(|case| case.name).collect::<Vec<_>>(),
            FIXTURE_NAMES
        );
        assert_eq!(
            cases.iter().map(|case| case.expected.len()).sum::<usize>(),
            268
        );
    }
}
