//! Frozen opaque BoxPaint::LinearGradient literals; no reference enters GPU encoding.
use crate::{Frame, Plan, Result, background_fixtures::witness::packed_pixels};
use eris::{
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore, Rect,
        box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
        gradient::{
            GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
        },
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    page::Page,
};
use std::sync::Arc;

pub const FIXTURE_NAMES: [&str; 6] = [
    "box-gradient-ramp-currentcolor-scale",
    "box-gradient-four-repeat-modes",
    "box-gradient-fractional-caller-clip",
    "box-gradient-document-fixed-restoration",
    "box-gradient-disjoint-zero-and-tiny",
    "box-gradient-page-padding-origin-content-clip",
];
pub const LITERAL_PIXELS: usize = 268;
pub const CONVERTED_FRAMES: usize = 12;
pub const COMPARED_BYTES: u64 = 2144;

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
fn rgb(value: u32) -> Color {
    Color::rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
fn stop(color: GradientColor, position: Option<GradientPosition>) -> GradientStop {
    GradientStop { color, position }
}
fn source(stops: &[GradientStop]) -> Arc<LinearGradient> {
    Arc::new(LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: stops.into(),
    })
}
fn hard() -> Arc<LinearGradient> {
    let middle = Some(GradientPosition {
        px: 0.0,
        percent: 50.0,
    });
    source(&[
        stop(GradientColor::Color(rgb(0xff0000)), middle),
        stop(GradientColor::Color(rgb(0x0000ff)), middle),
    ])
}
fn gradient(
    outer: Rect,
    tile: Rect,
    repeat: [bool; 2],
    gradient: Arc<LinearGradient>,
    current_color: Color,
    stop_scale: f32,
) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: square(outer),
        cutout: None,
        paint: BoxPaint::LinearGradient {
            tile,
            repeat,
            gradient,
            current_color,
            stop_scale,
        },
    }))
}
fn rows(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|pixel| match pixel {
            b'W' => 0xffffff,
            b'K' => 0x000000,
            b'R' => 0xff0000,
            b'B' => 0x0000ff,
            b'I' => 0x112233,
            b'A' => 0x1f1f1f,
            b'C' => 0x5d5d5d,
            b'D' => 0x9b9b9b,
            b'E' => 0xd9d9d9,
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

// Literal inventory: admitted boxes, B+G+M, generated sources, evaluation G, packing M.
// G and M also appear in the generated-gradient stats; they are not a second CPU debit.
fn prepare(
    name: &'static str,
    frame: Frame,
    commands: &[DrawCommand],
    expected: Vec<u32>,
    inventory: (usize, u64, usize, u64, u64),
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
        commands,
        fonts,
        &images,
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
        &images,
        fonts,
    )
    .map_err(|error| format!("{name} native refusal: {error}"))?;
    let stats = native.stats();
    let actual = (
        stats.box_gradients,
        stats.box_loop_work,
        stats.generated_gradient_sources,
        stats.gradient_evaluation_work,
        stats.gradient_preparation_work,
    );
    if actual != inventory || stats.generated_gradient_bytes as u64 != inventory.4 * 4 {
        return Err(format!(
            "{name} box/generated/work inventory actual={actual:?} expected={inventory:?}"
        ));
    }
    if stats.box_masks != 0
        || stats.box_background_images != 0
        || stats.mask_sources != 0
        || stats.coverage_bytes != 0
        || stats.row_entries != 0
        || native.plan().has_glyphs()
        || native.plan().group_scratch_bytes() != 0
        || stats.bridge.image_store_entries != 0
        || stats.bridge.referenced_sources != 0
        || stats.bridge.missing_images != 0
    {
        return Err(format!(
            "{name} unexpected mask/opacity/loaded-source inventory"
        ));
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
    let mut cases = Vec::with_capacity(FIXTURE_NAMES.len());
    let px = Some(GradientPosition {
        px: 1.0,
        percent: 0.0,
    });
    cases.push(prepare(
        FIXTURE_NAMES[0],
        Frame::new(4, 6, 0xffffff),
        &[
            gradient(
                rect(0.0, 0.0, 4.0, 2.0),
                rect(0.0, 0.0, 4.0, 2.0),
                [true, true],
                source(&[
                    stop(GradientColor::Color(Color::BLACK), None),
                    stop(GradientColor::Color(rgb(0xf8f8f8)), None),
                ]),
                Color::TRANSPARENT,
                1.0,
            ),
            gradient(
                rect(0.0, 2.0, 4.0, 2.0),
                rect(0.0, 2.0, 4.0, 2.0),
                [true, true],
                source(&[
                    stop(GradientColor::CurrentColor, px),
                    stop(GradientColor::Color(rgb(0x0000ff)), px),
                ]),
                rgb(0xff0000),
                2.0,
            ),
            gradient(
                rect(0.0, 4.0, 4.0, 2.0),
                rect(0.0, 4.0, 4.0, 2.0),
                [true, true],
                source(&[stop(GradientColor::CurrentColor, None)]),
                rgb(0x112233),
                16_000_000.0,
            ),
        ],
        rows(&["ACDE", "ACDE", "RRBB", "RRBB", "IIII", "IIII"]),
        (3, 180, 3, 84, 24),
        &fonts,
    )?);

    let repeated = hard();
    let mut commands = Vec::new();
    for (band, repeat) in [[false, false], [true, false], [false, true], [true, true]]
        .into_iter()
        .enumerate()
    {
        let y = (band * 4) as f32;
        commands.push(gradient(
            rect(0.0, y, 8.0, 4.0),
            rect(2.0, y + 1.0, 4.0, 2.0),
            repeat,
            Arc::clone(&repeated),
            Color::BLACK,
            1.0,
        ));
    }
    cases.push(prepare(
        FIXTURE_NAMES[1],
        Frame::new(8, 16, 0xffffff),
        &commands,
        rows(&[
            "WWWWWWWW", "WWRRBBWW", "WWRRBBWW", "WWWWWWWW", "WWWWWWWW", "BBRRBBRR", "BBRRBBRR",
            "WWWWWWWW", "WWRRBBWW", "WWRRBBWW", "WWRRBBWW", "WWRRBBWW", "BBRRBBRR", "BBRRBBRR",
            "BBRRBBRR", "BBRRBBRR",
        ]),
        (4, 704, 4, 344, 72),
        &fonts,
    )?);

    let mut clipped = Frame::new(6, 3, 0xffffff);
    clipped.caller_clip = eris_raster_core::Rect::new(0.5, 0.25, 3.0, 1.5);
    cases.push(prepare(
        FIXTURE_NAMES[2],
        clipped,
        &[gradient(
            rect(0.0, 0.0, 6.0, 3.0),
            rect(0.25, 0.25, 2.0, 2.0),
            [true, true],
            hard(),
            Color::BLACK,
            1.0,
        )],
        rows(&["WWWWWW", "WBRBWW", "WWWWWW"]),
        (1, 34, 1, 17, 3),
        &fonts,
    )?);

    let mut fixed = Frame::new(6, 3, 0xffffff);
    fixed.caller_clip = eris_raster_core::Rect::new(1.0, 0.0, 4.0, 3.0);
    fixed.document_offset = (1.0, 1.0);
    cases.push(prepare(
        FIXTURE_NAMES[3],
        fixed,
        &[
            DrawCommand::PushClip {
                rect: rect(0.0, 0.0, 1.0, 1.0),
            },
            DrawCommand::PushFixed,
            gradient(
                rect(0.0, 0.0, 4.0, 1.0),
                rect(0.5, 0.0, 2.0, 1.0),
                [false, false],
                hard(),
                Color::BLACK,
                1.0,
            ),
            DrawCommand::PopFixed,
            gradient(
                rect(0.0, 0.0, 4.0, 1.0),
                rect(0.0, 0.0, 2.0, 1.0),
                [true, true],
                hard(),
                Color::BLACK,
                1.0,
            ),
            DrawCommand::PopClip,
            gradient(
                rect(0.0, 1.0, 4.0, 1.0),
                rect(0.0, 1.0, 2.0, 1.0),
                [true, true],
                hard(),
                Color::BLACK,
                1.0,
            ),
        ],
        rows(&["WBWWWW", "WRWWWW", "WRBRBW"]),
        (3, 106, 3, 60, 6),
        &fonts,
    )?);

    cases.push(prepare(
        FIXTURE_NAMES[4],
        Frame::new(8, 4, 0xffffff),
        &[
            gradient(
                rect(0.0, 0.0, 8.0, 4.0),
                rect(20.0, 20.0, 4.0, 2.0),
                [false, false],
                hard(),
                Color::BLACK,
                1.0,
            ),
            gradient(
                rect(20.0, 20.0, 0.0, 0.0),
                rect(0.0, 0.0, 4.0, 2.0),
                [true, true],
                hard(),
                Color::BLACK,
                1.0,
            ),
            gradient(
                rect(0.0, 0.0, 4.0, 1.0),
                rect(0.5, 0.0, f32::from_bits(1), 1.0),
                [false, false],
                hard(),
                Color::BLACK,
                1.0,
            ),
            DrawCommand::Rect {
                rect: rect(7.0, 3.0, 1.0, 1.0),
                color: Color::BLACK,
                radius: 0.0,
            },
        ],
        rows(&["RWWWWWWW", "WWWWWWWW", "WWWWWWWW", "WWWWWWWK"]),
        (3, 118, 1, 29, 1),
        &fonts,
    )?);
    cases.push(page_content_clip(&fonts)?);
    if cases.iter().map(|case| case.name).collect::<Vec<_>>() != FIXTURE_NAMES
        || cases.iter().map(|case| case.expected.len()).sum::<usize>() != LITERAL_PIXELS
    {
        return Err("Box gradient fixture inventory".into());
    }
    Ok(cases)
}

fn page_gradient(commands: &[DrawCommand]) -> Result<()> {
    let mut boxes = commands.iter().filter_map(|command| match command {
        DrawCommand::BoxDecoration(value) => Some(value.as_ref()),
        _ => None,
    });
    let value = boxes.next().ok_or("Page gradient box missing")?;
    if boxes.next().is_some()
        || value.shape != square(rect(2.0, 2.0, 4.0, 2.0))
        || value.cutout.is_some()
    {
        return Err("Page gradient selected content geometry".into());
    }
    let middle = Some(GradientPosition {
        px: 0.0,
        percent: 50.0,
    });
    match &value.paint {
        BoxPaint::LinearGradient {
            tile,
            repeat,
            gradient,
            current_color,
            stop_scale,
        } if *tile == rect(3.0, 3.0, 4.0, 2.0)
            && *repeat == [false, false]
            && *current_color == rgb(0xff0000)
            && *stop_scale == 1.0
            && gradient.direction == GradientDirection::Angle(90.0)
            && gradient.stops.as_ref()
                == [
                    stop(GradientColor::CurrentColor, middle),
                    stop(GradientColor::Color(rgb(0x0000ff)), middle),
                ] =>
        {
            Ok(())
        }
        _ => Err("Page gradient lost padding tile/currentColor/stops/scale".into()),
    }
}
fn page_content_clip(fonts: &Fonts) -> Result<PreparedFixture> {
    let mut page = Page::try_from_html(
        "about:blank"
            .parse()
            .map_err(|error| format!("fixture URL: {error}"))?,
        include_str!("box_gradient_assets/page.html"),
        false,
    )
    .map_err(|error| error.to_string())?;
    let first = page.prepare_layout(8.0, 6.0, fonts);
    page_gradient(&first.commands)?;
    if !page.images.is_empty() || !page.diagnostics.is_empty() {
        return Err("Page gradient unexpectedly loaded resources or emitted diagnostics".into());
    }
    let expected = rows(&[
        "WWWWWWWW", "WWWWWWWW", "WWWWWWWW", "WWWRRBWW", "WWWWWWWW", "WWWWWWWW",
    ]);
    let initial = prepare(
        FIXTURE_NAMES[5],
        Frame::new(8, 6, 0xffffff),
        &first.commands,
        expected.clone(),
        (1, 66, 1, 39, 3),
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
    let readonly = page.layout(8.0, 6.0, fonts);
    page_gradient(&readonly.commands)?;
    if page
        .document
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect::<Vec<_>>()
        != graph
        || readonly
            .hit_regions
            .iter()
            .map(|hit| (hit.node, hit.rect, hit.fixed))
            .collect::<Vec<_>>()
            != hits
    {
        return Err("Page gradient read-only layout changed graph/hits".into());
    }
    prepare(
        FIXTURE_NAMES[5],
        Frame::new(8, 6, 0xffffff),
        &readonly.commands,
        expected,
        (1, 66, 1, 39, 3),
        fonts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_six_literal_canvas_and_packed_box_gradients_match_without_a_device() {
        let cases = fixtures().expect("all frozen Box gradients must admit and match");
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
