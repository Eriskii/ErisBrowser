//! Fixed Color BoxDecoration inventory, frozen before candidate execution.
//! Literal arrays are the oracle; Canvas and packed Native are checked against them.
use crate::{Frame, Plan, Result, background_fixtures::witness::packed_pixels};
use eris::{
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore, Rect,
        box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    page::Page,
};
use std::sync::Arc;

pub const FIXTURE_NAMES: [&str; 10] = [
    "ellipse-opaque-blue",
    "correlated-fractional-ring",
    "used-truncated-left-ellipse",
    "fixed-fractional-mask-and-document-clip",
    "source-alpha-over-yellow",
    "nested-half-opacity-blue",
    "identical-fractional-cutout",
    "fractional-caller-clip",
    "two-mask-absolute-seam",
    "page-translucent-rounded-ring",
];
pub const LITERAL_PIXELS: usize = 2756;
pub const CONVERTED_FRAMES: usize = 20;
pub const COMPARED_BYTES: u64 = 22048;

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
fn shape(rect: Rect, radii: [[f32; 2]; 4]) -> RoundedBox {
    RoundedBox { rect, radii }
}
fn square(rect: Rect) -> RoundedBox {
    shape(rect, [[0.0; 2]; 4])
}
fn decoration(shape: RoundedBox, cutout: Option<RoundedBox>, color: Color) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape,
        cutout,
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
        .find(|(_, (a, b))| a != b)
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
    masks: usize,
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
    // Frame offsets and the caller clip belong to the phase exactly once.
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
    if native.stats().box_masks != masks {
        return Err(format!(
            "{name} box mask count actual={} expected={masks}",
            native.stats().box_masks
        ));
    }
    if name == "two-mask-absolute-seam"
        && (native.stats().coverage_bytes != 2560 || native.stats().row_entries != 4)
    {
        return Err("seam mask storage/row inventory".into());
    }
    if name == "nested-half-opacity-blue" && native.plan().group_scratch_bytes() == 0 {
        return Err("nested fixture lost isolated GPU groups".into());
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
    let red = Color::rgb(255, 0, 0);
    let ellipse = shape(rect(0.0, 0.0, 4.0, 2.0), [[2.0, 1.0]; 4]);
    let mut cases = Vec::with_capacity(FIXTURE_NAMES.len());
    cases.push(prepare(
        FIXTURE_NAMES[0],
        Frame::new(4, 2, 0xffffff),
        vec![decoration(ellipse, None, blue)],
        vec![
            0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff, 0x7070ff, 0x0000ff, 0x0000ff, 0x7070ff,
        ],
        1,
        &fonts,
    )?);
    cases.push(prepare(
        FIXTURE_NAMES[1],
        Frame::new(2, 2, 0xffffff),
        vec![decoration(
            square(rect(0.0, 0.0, 2.0, 2.0)),
            Some(square(rect(0.25, 0.25, 1.0, 1.0))),
            Color::BLACK,
        )],
        vec![0x909090, 0x303030, 0x303030, 0x101010],
        1,
        &fonts,
    )?);
    cases.push(prepare(
        FIXTURE_NAMES[2],
        Frame::new(1, 4, 0xffffff),
        vec![decoration(
            shape(
                rect(0.0, 0.0, 1.0, 4.0),
                [[2.0, 2.0], [0.0, 2.0], [0.0, 2.0], [2.0, 2.0]],
            ),
            None,
            blue,
        )],
        vec![0xa0a0ff, 0x1010ff, 0x1010ff, 0xa0a0ff],
        1,
        &fonts,
    )?);
    let mut fixed = Frame::new(4, 2, 0xffffff);
    fixed.document_offset = (1.0, 0.0);
    cases.push(prepare(
        FIXTURE_NAMES[3],
        fixed,
        vec![
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 1.0, 1.0),
            },
            decoration(square(rect(0.0, 0.0, 2.0, 1.0)), None, red),
            DrawCommand::PushFixed,
            decoration(square(rect(2.25, 0.25, 1.0, 1.0)), None, blue),
            DrawCommand::PopFixed,
            decoration(
                square(rect(0.0, 0.0, 2.0, 1.0)),
                None,
                Color::rgb(0, 255, 0),
            ),
            DrawCommand::PopClip,
            DrawCommand::Rect {
                rect: rect(0.0, 1.0, 1.0, 1.0),
                color: Color::BLACK,
                radius: 0.0,
            },
        ],
        vec![
            0xffffff, 0x00ff00, 0x7070ff, 0xd0d0ff, 0xffffff, 0x000000, 0xd0d0ff, 0xf0f0ff,
        ],
        3,
        &fonts,
    )?);
    cases.push(prepare(
        FIXTURE_NAMES[4],
        Frame::new(4, 2, 0xffff00),
        vec![decoration(ellipse, None, Color::rgba(255, 0, 0, 128))],
        vec![
            0xffb800, 0xff7f00, 0xff7f00, 0xffb800, 0xffb800, 0xff7f00, 0xff7f00, 0xffb800,
        ],
        1,
        &fonts,
    )?);
    cases.push(prepare(
        FIXTURE_NAMES[5],
        Frame::new(4, 2, 0xffffff),
        vec![
            DrawCommand::PushOpacity { opacity: 0.5 },
            DrawCommand::PushOpacity { opacity: 0.5 },
            decoration(ellipse, None, blue),
            DrawCommand::PopOpacity,
            DrawCommand::PopOpacity,
        ],
        vec![
            0xdbdbff, 0xbfbfff, 0xbfbfff, 0xdbdbff, 0xdbdbff, 0xbfbfff, 0xbfbfff, 0xdbdbff,
        ],
        1,
        &fonts,
    )?);
    let identical = square(rect(0.25, 0.25, 1.0, 1.0));
    cases.push(prepare(
        FIXTURE_NAMES[6],
        Frame::new(2, 2, 0xffffff),
        vec![decoration(identical, Some(identical), Color::BLACK)],
        vec![0xffffff; 4],
        1,
        &fonts,
    )?);
    let mut clipped = Frame::new(4, 2, 0xffffff);
    clipped.caller_clip = eris_raster_core::Rect::new(0.5, 0.0, 2.5, 2.0);
    cases.push(prepare(
        FIXTURE_NAMES[7],
        clipped,
        vec![decoration(
            square(rect(0.0, 0.0, 4.0, 2.0)),
            None,
            Color::BLACK,
        )],
        vec![
            0xffffff, 0x000000, 0x000000, 0xffffff, 0xffffff, 0x000000, 0x000000, 0xffffff,
        ],
        1,
        &fonts,
    )?);
    // Literal sparse rows: no geometry/sampling helper computes an expectation.
    let mut seam = vec![0x000000; 2560];
    for at in [0, 1279, 1280, 2559] {
        seam[at] = 0x404040;
    }
    for at in [1023, 1024] {
        seam[at] = 0x909090;
    }
    for at in [2303, 2304] {
        seam[at] = 0x303030;
    }
    cases.push(prepare(
        FIXTURE_NAMES[8],
        Frame::new(1280, 2, 0xffffff),
        vec![decoration(
            square(rect(0.25, 0.0, 1279.5, 2.0)),
            Some(square(rect(1023.25, 0.25, 1.5, 1.0))),
            Color::BLACK,
        )],
        seam,
        2,
        &fonts,
    )?);
    cases.push(page_ring(&fonts)?);
    if cases.iter().map(|f| f.name).collect::<Vec<_>>() != FIXTURE_NAMES
        || cases.iter().map(|f| f.expected.len()).sum::<usize>() != LITERAL_PIXELS
    {
        return Err("native Color box literal inventory".into());
    }
    Ok(cases)
}

fn page_ring(fonts: &Fonts) -> Result<PreparedFixture> {
    let page = Page::from_html(
        "about:blank".parse().map_err(|_| "fixture URL")?,
        "<!doctype html><style>html,body{margin:0;padding:0;background:white}#ring{width:8px;height:8px;border:2px solid rgba(255,0,0,.5);border-radius:2px;background:blue}</style><div id=\"ring\"></div>",
        false,
    );
    let layout = page.layout(12.0, 12.0, fonts);
    if !page.images.is_empty() || !page.diagnostics.is_empty() {
        return Err("Page ring unexpected resource/diagnostic".into());
    }
    let boxes = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BoxDecoration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    let outer = shape(rect(0.0, 0.0, 12.0, 12.0), [[2.0; 2]; 4]);
    if boxes.len() != 2
        || boxes[0].shape != outer
        || boxes[0].cutout.is_some()
        || !matches!(&boxes[0].paint, BoxPaint::Color(c) if *c == Color::rgb(0, 0, 255))
        || boxes[1].shape != outer
        || boxes[1].cutout != Some(square(rect(2.0, 2.0, 8.0, 8.0)))
        || !matches!(&boxes[1].paint, BoxPaint::Color(c) if *c == Color::rgba(255, 0, 0, 128))
    {
        return Err("Page ring lost exact Color shape/cutout route".into());
    }
    let rows = [
        "ABCCCCCCCCBA",
        "BCCCCCCCCCCB",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "CCDDDDDDDDCC",
        "BCCCCCCCCCCB",
        "ABCCCCCCCCBA",
    ];
    let expected = rows
        .iter()
        .flat_map(|row| row.bytes())
        .map(|pixel| match pixel {
            b'A' => 0xb283d0,
            b'B' => 0x800988,
            b'C' => 0x80007f,
            b'D' => 0x0000ff,
            _ => unreachable!("fixed literal palette"),
        })
        .collect();
    prepare(
        FIXTURE_NAMES[9],
        Frame::new(12, 12, 0xffffff),
        layout.commands,
        expected,
        2,
        fonts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ten_literal_canvas_and_packed_native_fixtures_match_without_a_device() {
        let cases = fixtures().expect("all frozen Native Color box fixtures must admit and match");
        assert_eq!(cases.len(), 10);
        assert_eq!(
            cases.iter().map(|case| case.name).collect::<Vec<_>>(),
            FIXTURE_NAMES
        );
        assert_eq!(
            cases.iter().map(|case| case.expected.len()).sum::<usize>(),
            2756
        );
    }
}
