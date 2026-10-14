//! Oversized rounded Color fixtures from preimplementation literal frames.
//! References reach Canvas/packed/GPU comparators, never the GPU encoder.
use crate::{Frame, Plan, Result, background_fixtures::witness::packed_pixels};
use eris::{
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore, Rect,
        box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    page::Page,
};
use eris_raster_core::Profile;
use std::sync::Arc;

pub const FIXTURE_NAMES: [&str; 10] = [
    "core-radius1",
    "core-alpha128",
    "core-opacity-half",
    "core-alpha128-opacity-half",
    "core-suppressed",
    "core-full-viewport",
    "core-fractional-clip",
    "core-seven-tiles",
    "old-cap-new-admission",
    "core-page-padding-box",
];
pub const LITERAL_PIXELS: usize = 3_675_652;
pub const CONVERTED_FRAMES: usize = 20;
pub const COMPARED_BYTES: u64 = 29_405_216;
const SEVEN_ROW: &[u8; 5120] = include_bytes!("box_core_fixtures/seven-tile-row.rgba");
const PAGE: &str = include_str!("box_core_fixtures/page.html");

#[derive(Clone, Copy)]
pub struct Metrics {
    pub masks: usize,
    pub coverage: usize,
    pub rows: usize,
    pub box_work: u64,
    pub cpu_work: u64,
    pub lowered: usize,
    pub exact_invocations: Option<u64>,
}
pub struct PreparedFixture {
    pub name: &'static str,
    pub expected: Vec<u32>,
    pub plan: Plan,
    pub metrics: Metrics,
}
fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn decoration(area: Rect, radii: [[f32; 2]; 4], color: Color) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox { rect: area, radii },
        cutout: None,
        paint: BoxPaint::Color(color),
    }))
}
fn corners(width: usize, height: usize, inside: u32, corner: u32) -> Vec<u32> {
    let mut pixels = vec![inside; width * height];
    for index in [0, width - 1, width * (height - 1), width * height - 1] {
        pixels[index] = corner;
    }
    pixels
}
fn compare(name: &str, route: &str, width: u32, actual: &[u32], expected: &[u32]) -> Result<()> {
    if actual.len() != expected.len() {
        return Err(format!("{name} {route} literal length"));
    }
    if let Some(at) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        return Err(format!(
            "{name} {route} pixel ({},{}) actual={:08x} expected={:08x}",
            at % width as usize,
            at / width as usize,
            actual[at],
            expected[at]
        ));
    }
    Ok(())
}
fn prepare(
    name: &'static str,
    frame: Frame,
    commands: &[DrawCommand],
    expected: Vec<u32>,
    metrics: Metrics,
    fonts: &Fonts,
) -> Result<PreparedFixture> {
    if expected.len() != frame.width as usize * frame.height as usize {
        return Err(format!("{name} literal dimensions"));
    }
    let images = ImageStore::new();
    let mut canvas = Canvas::new(frame.width, frame.height)?;
    canvas.clear(Color::WHITE);
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
        Frame::new(frame.width, frame.height, 0xffffff),
        &[NativePhase { frame, commands }],
        &images,
        fonts,
    )
    .map_err(|error| format!("{name} Native refusal: {error}"))?;
    let stats = native.stats();
    if stats.box_cores != 1
        || stats.box_rectangles != 0
        || stats.box_masks != metrics.masks
        || stats.mask_sources != metrics.masks
        || stats.coverage_bytes != metrics.coverage
        || stats.row_entries != metrics.rows
        || stats.box_loop_work != metrics.box_work
        || stats.cpu_pixel_upper_bound != metrics.cpu_work
        || stats.bridge.original_commands != commands.len()
        || stats.bridge.lowered_commands != metrics.lowered
        || stats.rounded_masks != 0
        || stats.generated_gradient_sources != 0
        || stats.box_background_images != 0
        || stats.box_gradients != 0
    {
        return Err(format!("{name} rounded-core inventory mismatch: {stats:?}"));
    }
    if native.plan().profile() != Profile::Native
        || native.plan().gpu_buffer_bytes() > stats.gpu_buffer_upper_bound
        || metrics
            .exact_invocations
            .is_some_and(|n| native.plan().invocations() != n)
    {
        return Err(format!("{name} Native plan bounds/work mismatch"));
    }
    if matches!(name, "core-opacity-half" | "core-alpha128-opacity-half")
        && native.plan().group_scratch_bytes() == 0
    {
        return Err(format!("{name} lost isolated GPU opacity group"));
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
        metrics,
    })
}

pub fn fixtures() -> Result<Vec<PreparedFixture>> {
    let fonts = Fonts::new();
    let red = Color::rgb(255, 0, 0);
    let mut cases = Vec::with_capacity(FIXTURE_NAMES.len());
    for (index, alpha, opacity, inside, corner) in [
        (0, 255, None, 0xff0000, 0xff3030),
        (1, 128, None, 0xff7f7f, 0xff9898),
        (2, 255, Some(0.5), 0xff8080, 0xff9898),
        (3, 128, Some(0.5), 0xffbfbf, 0xffcccc),
        (4, 255, Some(0.0), 0xffffff, 0xffffff),
    ] {
        let mut commands = Vec::new();
        if let Some(opacity) = opacity {
            commands.push(DrawCommand::PushOpacity { opacity });
        }
        commands.push(decoration(
            rect(0.0, 0.0, 512.0, 513.0),
            [[1.0; 2]; 4],
            Color::rgba(255, 0, 0, alpha),
        ));
        if opacity.is_some() {
            commands.push(DrawCommand::PopOpacity);
        }
        cases.push(prepare(
            FIXTURE_NAMES[index],
            Frame::new(512, 513, 0xffffff),
            &commands,
            corners(512, 513, inside, corner),
            Metrics {
                masks: 4,
                coverage: 2_046,
                rows: 1_024,
                box_work: 791_114,
                cpu_work: if opacity.is_some() {
                    1_316_426
                } else {
                    791_114
                },
                lowered: if opacity.is_some() { 7 } else { 5 },
                exact_invocations: if opacity.is_none() {
                    Some(811_008)
                } else {
                    None
                },
            },
            &fonts,
        )?);
    }
    cases.push(prepare(
        FIXTURE_NAMES[5],
        Frame::new(1280, 1024, 0xffffff),
        &[decoration(
            rect(0.0, 0.0, 1280.0, 1024.0),
            [[1.0; 2]; 4],
            red,
        )],
        corners(1280, 1024, 0xff0000, 0xff3030),
        Metrics {
            masks: 6,
            coverage: 4_604,
            rows: 2_048,
            box_work: 3_938_888,
            cpu_work: 3_938_888,
            lowered: 7,
            exact_invocations: Some(3_969_024),
        },
        &fonts,
    )?);
    let frame = Frame {
        caller_clip: eris_raster_core::Rect::new(0.5, 0.25, 513.0, 513.0),
        ..Frame::new(514, 514, 0xffffff)
    };
    let mut clipped = vec![0xffffff; 514 * 514];
    for y in 1..514 {
        for x in 1..514 {
            clipped[y * 514 + x] = 0xff0000;
        }
    }
    clipped[514 * 514 - 1] = 0xff3030;
    cases.push(prepare(
        FIXTURE_NAMES[6],
        frame,
        &[decoration(rect(0.0, 0.0, 514.0, 514.0), [[1.0; 2]; 4], red)],
        clipped,
        Metrics {
            masks: 2,
            coverage: 1_025,
            rows: 513,
            box_work: 791_073,
            cpu_work: 791_073,
            lowered: 3,
            exact_invocations: None,
        },
        &fonts,
    )?);
    let row = SEVEN_ROW
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            if p[3] != 255 {
                return Err("seven-tile literal alpha".into());
            }
            Ok(u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2]))
        })
        .collect::<Result<Vec<u32>>>()?;
    let mut seven = vec![0xff0000; 1280 * 205];
    seven[..1280].copy_from_slice(&row);
    seven[1280 * 204..].copy_from_slice(&row);
    cases.push(prepare(
        FIXTURE_NAMES[7],
        Frame::new(1280, 205, 0xffffff),
        &[decoration(
            rect(0.0, 0.0, 1280.0, 205.0),
            [[1100.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1100.0, 1.0]],
            red,
        )],
        seven,
        Metrics {
            masks: 7,
            coverage: 226_063,
            rows: 613,
            box_work: 1_032_096,
            cpu_work: 1_032_096,
            lowered: 8,
            exact_invocations: None,
        },
        &fonts,
    )?);
    cases.push(prepare(
        FIXTURE_NAMES[8],
        Frame::new(1280, 205, 0xffffff),
        &[decoration(
            rect(0.0, 0.0, 1280.0, 205.0),
            [[0.125; 2]; 4],
            Color::BLACK,
        )],
        vec![0x000000; 1280 * 205],
        Metrics {
            masks: 6,
            coverage: 2_966,
            rows: 410,
            box_work: 790_588,
            cpu_work: 790_588,
            lowered: 7,
            exact_invocations: None,
        },
        &fonts,
    )?);

    // A real Page uses the exact held HTML. No URL resource or script is needed.
    let page = Page::from_html(
        "https://rounded-core.invalid/"
            .parse()
            .map_err(|_| "Page URL")?,
        PAGE,
        false,
    );
    let layout = page.layout(512.0, 513.0, &fonts);
    if !page.diagnostics.is_empty()
        || page.title() != "Rounded Color core"
        || !page.images.is_empty()
        || layout.commands.len() != 2
        || layout.hit_test(2.0, 2.0) != page.document.query_selector("#board")
        || page.document.query_selector("#board").is_none()
    {
        return Err("selected Page title/resource/command/hit controls".into());
    }
    if !matches!(&layout.commands[0], DrawCommand::Rect { rect: area, color, radius }
        if *area == rect(0.0, 0.0, 512.0, 513.0) && *color == Color::WHITE && *radius == 0.0)
    {
        return Err("selected Page canvas geometry".into());
    }
    let expected_shape = RoundedBox {
        rect: rect(0.0, 0.0, 512.0, 513.0),
        radii: [[1.0; 2]; 4],
    };
    if !matches!(&layout.commands[1], DrawCommand::BoxDecoration(value)
        if value.shape == expected_shape
        && value.cutout.is_none() && matches!(&value.paint, BoxPaint::Color(color) if *color == red))
    {
        return Err("selected Page rounded Color geometry".into());
    }
    cases.push(prepare(
        FIXTURE_NAMES[9],
        Frame::new(512, 513, 0xffffff),
        &layout.commands,
        corners(512, 513, 0xff0000, 0xff3030),
        Metrics {
            masks: 4,
            coverage: 2_046,
            rows: 1_024,
            box_work: 791_114,
            cpu_work: 1_053_770,
            lowered: 6,
            exact_invocations: Some(1_077_248),
        },
        &fonts,
    )?);
    if cases.iter().map(|case| case.name).collect::<Vec<_>>() != FIXTURE_NAMES
        || cases.iter().map(|case| case.expected.len()).sum::<usize>() != LITERAL_PIXELS
    {
        return Err("rounded-core fixture inventory".into());
    }
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ten_frozen_rounded_core_frames_match_canvas_and_packed_native_without_a_device() {
        let cases = fixtures().expect("all frozen rounded-core fixtures must admit and match");
        assert_eq!(cases.len(), 10);
        assert_eq!(
            cases.iter().map(|c| c.expected.len()).sum::<usize>(),
            3_675_652
        );
        assert_eq!(CONVERTED_FRAMES, 20);
        assert_eq!(COMPARED_BYTES, 29_405_216);
    }
}
