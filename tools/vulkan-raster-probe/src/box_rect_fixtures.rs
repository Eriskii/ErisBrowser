//! Integer Color boxes: independently frozen pixels, then Canvas and Native checks.
//! This inventory is separate from the ten rounded/masked box fixtures.
use crate::{Frame, Plan, Result, background_fixtures::witness::packed_pixels};
use eris::graphics::{
    Canvas, Color, DrawCommand, Fonts, ImageStore, Rect,
    box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
    raster_bridge::native::{NativePhase, plan_native_scene},
};
use std::sync::Arc;

pub const FIXTURE_NAMES: [&str; 5] = [
    "integer-box-full-viewport",
    "integer-box-fractional-clip",
    "integer-box-zero-axis-alpha-order",
    "integer-box-document-fixed-restoration",
    "integer-box-nested-alpha-opacity",
];
pub const LITERAL_PIXELS: usize = 1_310_766;
pub const CONVERTED_FRAMES: usize = 10;
pub const COMPARED_BYTES: u64 = 10_486_128;

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
fn decoration(shape: RoundedBox, color: Color) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape,
        cutout: None,
        paint: BoxPaint::Color(color),
    }))
}
fn rgb(value: u32) -> Color {
    Color::rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
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
fn prepare(
    name: &'static str,
    frame: Frame,
    commands: Vec<DrawCommand>,
    expected: Vec<u32>,
    expected_rectangles_and_work: (usize, u64),
    fonts: &Fonts,
) -> Result<PreparedFixture> {
    if expected.len() != frame.width as usize * frame.height as usize {
        return Err(format!("{name} literal dimensions"));
    }
    let images = ImageStore::new();
    let mut canvas = Canvas::new(frame.width, frame.height)?;
    canvas.clear(rgb(frame.clear));
    let clip = frame.caller_clip;
    canvas.set_clip(rect(clip.x, clip.y, clip.width, clip.height));
    canvas.paint_with_viewport(
        &commands,
        fonts,
        &images,
        frame.document_offset,
        frame.viewport_offset,
    );
    if canvas.exhausted() {
        return Err(format!("{name} Canvas exhausted"));
    }
    compare(name, "Canvas", frame.width, &canvas.pixels, &expected)?;
    // The target is canonical; document/fixed offsets and caller clip are phase state.
    let native = plan_native_scene(
        Frame::new(frame.width, frame.height, frame.clear),
        &[NativePhase {
            frame,
            commands: &commands,
        }],
        &images,
        fonts,
    )
    .map_err(|error| format!("{name} native refusal: {error}"))?;
    let stats = native.stats();
    let (rectangles, work) = expected_rectangles_and_work;
    if stats.box_rectangles != rectangles || stats.box_loop_work != work {
        return Err(format!(
            "{name} rectangle/work inventory actual={}/{} expected={rectangles}/{work}",
            stats.box_rectangles, stats.box_loop_work
        ));
    }
    if stats.box_masks != 0
        || stats.mask_sources != 0
        || stats.coverage_bytes != 0
        || stats.row_entries != 0
        || native.plan().has_glyphs()
    {
        return Err(format!("{name} unexpected mask or row payload"));
    }
    if name == FIXTURE_NAMES[4] && native.plan().group_scratch_bytes() == 0 {
        return Err("nested square fixture lost isolated GPU groups".into());
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
    let blue = Color::rgb(0, 0, 255);
    let red_half = Color::rgba(255, 0, 0, 128);
    let lime = Color::rgb(0, 255, 0);
    let mut cases = Vec::with_capacity(FIXTURE_NAMES.len());
    // One full viewport was larger than the existing mask coverage cap. The
    // literal is constant; this path must admit without any coverage allocation.
    cases.push(prepare(
        FIXTURE_NAMES[0],
        Frame::new(1280, 1024, 0xffffff),
        vec![decoration(square(rect(0.0, 0.0, 1280.0, 1024.0)), blue)],
        vec![0x0000ff; 1_310_720],
        (1, 3_932_168),
        &fonts,
    )?);

    let mut clipped = Frame::new(5, 3, 0xffffff);
    clipped.caller_clip = eris_raster_core::Rect::new(0.25, 0.75, 3.5, 1.75);
    cases.push(prepare(
        FIXTURE_NAMES[1],
        clipped,
        vec![decoration(square(rect(0.0, 0.0, 5.0, 3.0)), Color::BLACK)],
        vec![
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x000000, 0x000000,
            0x000000, 0xffffff, 0xffffff, 0x000000, 0x000000, 0x000000, 0xffffff,
        ],
        (1, 26),
        &fonts,
    )?);

    let mut translated = Frame::new(4, 2, 0xffffff);
    translated.document_offset = (0.25, 0.75);
    cases.push(prepare(
        FIXTURE_NAMES[2],
        translated,
        vec![
            decoration(
                RoundedBox {
                    rect: rect(-0.25, -0.75, 4.0, 2.0),
                    // A zero axis makes each of these used corners square.
                    radii: [[0.0, 50.0], [50.0, 0.0], [0.0, 7.0], [8.0, 0.0]],
                },
                blue,
            ),
            decoration(square(rect(-0.25, -0.75, 2.0, 2.0)), red_half),
            DrawCommand::Rect {
                rect: rect(2.75, -0.75, 1.0, 1.0),
                color: lime,
                radius: 0.0,
            },
        ],
        vec![
            0x80007f, 0x80007f, 0x0000ff, 0x00ff00, 0x80007f, 0x80007f, 0x0000ff, 0x0000ff,
        ],
        (2, 52),
        &fonts,
    )?);

    let mut fixed = Frame::new(5, 3, 0xffffff);
    fixed.document_offset = (1.0, 1.0);
    cases.push(prepare(
        FIXTURE_NAMES[3],
        fixed,
        vec![
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 2.0, 1.0),
            },
            decoration(square(rect(0.0, 0.0, 4.0, 2.0)), blue),
            DrawCommand::PushFixed,
            decoration(square(rect(3.0, 0.0, 2.0, 1.0)), Color::BLACK),
            DrawCommand::PopFixed,
            decoration(square(rect(0.0, 0.0, 1.0, 1.0)), red_half),
            DrawCommand::PopClip,
            DrawCommand::Rect {
                rect: rect(0.0, 1.0, 1.0, 1.0),
                color: lime,
                radius: 0.0,
            },
        ],
        vec![
            0xffffff, 0xffffff, 0xffffff, 0x000000, 0x000000, 0xffffff, 0x80007f, 0x0000ff,
            0xffffff, 0xffffff, 0xffffff, 0x00ff00, 0xffffff, 0xffffff, 0xffffff,
        ],
        (3, 39),
        &fonts,
    )?);

    cases.push(prepare(
        FIXTURE_NAMES[4],
        Frame::new(4, 2, 0xffffff),
        vec![
            DrawCommand::PushOpacity { opacity: 0.5 },
            DrawCommand::PushOpacity { opacity: 0.5 },
            decoration(square(rect(0.0, 0.0, 4.0, 2.0)), blue),
            decoration(square(rect(0.0, 0.0, 2.0, 2.0)), red_half),
            DrawCommand::PopOpacity,
            DrawCommand::PopOpacity,
        ],
        vec![
            0xdfbfdf, 0xdfbfdf, 0xbfbfff, 0xbfbfff, 0xdfbfdf, 0xdfbfdf, 0xbfbfff, 0xbfbfff,
        ],
        (2, 52),
        &fonts,
    )?);
    if cases.len() != FIXTURE_NAMES.len()
        || cases.iter().map(|case| case.expected.len()).sum::<usize>() != LITERAL_PIXELS
    {
        return Err("integer Color box fixture inventory".into());
    }
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_literal_canvas_and_packed_rect_fixtures_match_without_a_device() {
        let cases = fixtures().expect("all frozen integer Color boxes must admit and match");
        assert_eq!(
            cases.iter().map(|case| case.name).collect::<Vec<_>>(),
            FIXTURE_NAMES
        );
        assert_eq!(
            cases.iter().map(|case| case.expected.len()).sum::<usize>(),
            1_310_766
        );
    }
}
