//! Independently frozen rounded Color pixels using APIs available on the parent.
//! CPU Canvas and packed-plan interpretation only; no GPU/device/window opens.
#![cfg(feature = "raster-bridge")]

use eris::{
    graphics::{
        Canvas, Color, DrawCommand, Fonts, ImageStore, Rect,
        box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
        raster_bridge::native::{NativePhase, NativeScenePlan, plan_native_scene},
    },
    page::Page,
};
use eris_raster_core::{DrawKind, Frame, Profile, Rect as CoreRect};
use std::sync::Arc;
use url::Url;
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

const PAGE: &str = include_str!("fixtures/rounded-core-page.html");
const SEVEN_ROW: &[u8] = include_bytes!("fixtures/rounded-core-seven-tile-row.rgba");

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn command(area: Rect, radii: [[f32; 2]; 4], alpha: u8) -> DrawCommand {
    DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
        shape: RoundedBox { rect: area, radii },
        cutout: None,
        paint: BoxPaint::Color(Color::rgba(255, 0, 0, alpha)),
    }))
}
fn corners(width: u32, height: u32, inside: u32, corner: u32) -> Vec<u32> {
    let mut expected = vec![inside; width as usize * height as usize];
    for (x, y) in [
        (0, 0),
        (width - 1, 0),
        (0, height - 1),
        (width - 1, height - 1),
    ] {
        expected[(y * width + x) as usize] = corner;
    }
    expected
}
fn pixels(name: &str, width: u32, actual: &[u32], expected: &[u32]) {
    assert_eq!(actual.len(), expected.len(), "{name} frame length");
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "{name} pixel ({},{}): actual={:06x} expected={:06x}",
            index % width as usize,
            index / width as usize,
            actual[index],
            expected[index]
        );
    }
}
fn canvas(frame: Frame, commands: &[DrawCommand], fonts: &Fonts, images: &ImageStore) -> Vec<u32> {
    let mut canvas = Canvas::new(frame.width, frame.height).unwrap();
    canvas.clear(Color::WHITE);
    let clip = frame.caller_clip;
    canvas.set_clip(rect(clip.x, clip.y, clip.width, clip.height));
    canvas.paint_with_viewport(
        commands,
        fonts,
        images,
        frame.document_offset,
        frame.viewport_offset,
    );
    assert!(!canvas.exhausted());
    canvas.pixels
}
fn check(name: &str, frame: Frame, commands: &[DrawCommand], expected: &[u32]) -> NativeScenePlan {
    let fonts = Fonts::new();
    let images = ImageStore::new();
    // This complete software pixel control executes before the parent fails admission.
    pixels(
        name,
        frame.width,
        &canvas(frame, commands, &fonts, &images),
        expected,
    );
    let result = plan_native_scene(
        Frame::new(frame.width, frame.height, 0xffffff),
        &[NativePhase { frame, commands }],
        &images,
        &fonts,
    )
    .expect("frozen oversized Color must admit within unchanged Native limits");
    pixels(
        name,
        frame.width,
        &witness::packed_pixels(result.plan()),
        expected,
    );
    assert_eq!(result.plan().profile(), Profile::Native);
    assert_eq!(result.stats().box_rectangles, 0);
    assert_eq!(result.stats().rounded_masks, 0);
    assert_eq!(result.stats().generated_gradient_sources, 0);
    assert_eq!(result.stats().bridge.original_commands, commands.len());
    assert!(result.plan().gpu_buffer_bytes() <= result.stats().gpu_buffer_upper_bound);
    result
}
fn inventory(
    result: &NativeScenePlan,
    masks: usize,
    coverage: usize,
    rows: usize,
    work: u64,
    lowered: usize,
) {
    let stats = result.stats();
    assert_eq!(
        (
            stats.box_masks,
            stats.mask_sources,
            stats.coverage_bytes,
            stats.row_entries
        ),
        (masks, masks, coverage, rows)
    );
    assert_eq!(stats.box_loop_work, work);
    assert_eq!(stats.bridge.lowered_commands, lowered);
}

#[test]
fn oversized_rounded_color_keeps_alpha_and_opacity_pixels_at_partition_seams() {
    let frame = Frame::new(512, 513, 0xffffff);
    // Source-alpha and isolated-opacity arithmetic is independently frozen.
    for (name, alpha, opacity, inside, corner) in [
        ("opaque", 255, None, 0xff0000, 0xff3030),
        ("alpha128", 128, None, 0xff7f7f, 0xff9898),
        ("half-opacity", 255, Some(0.5), 0xff8080, 0xff9898),
        ("alpha128-half-opacity", 128, Some(0.5), 0xffbfbf, 0xffcccc),
        ("alpha0", 0, None, 0xffffff, 0xffffff),
        ("opacity0", 255, Some(0.0), 0xffffff, 0xffffff),
    ] {
        let mut commands = Vec::new();
        if let Some(opacity) = opacity {
            commands.push(DrawCommand::PushOpacity { opacity });
        }
        commands.push(command(rect(0.0, 0.0, 512.0, 513.0), [[1.0; 2]; 4], alpha));
        if opacity.is_some() {
            commands.push(DrawCommand::PopOpacity);
        }
        let expected = corners(512, 513, inside, corner);
        let result = check(name, frame, &commands, &expected);
        let scopes = if opacity.is_some() { 2 } else { 0 };
        inventory(&result, 4, 2_046, 1_024, 791_114, 5 + scopes);
        assert_eq!(
            result.stats().cpu_pixel_upper_bound,
            791_114 + scopes as u64 * 262_656
        );
        if opacity == Some(0.5) {
            assert!(result.plan().group_scratch_bytes() > 0);
        }
        if opacity.is_none() && alpha != 0 {
            assert_eq!(
                result
                    .plan()
                    .draws()
                    .iter()
                    .filter(|d| d.kind() == DrawKind::Glyph)
                    .count(),
                4
            );
            // The second Rectangle is the extracted core; the first is clear.
            assert_eq!(
                result
                    .plan()
                    .draws()
                    .iter()
                    .filter(|d| d.kind() == DrawKind::Rectangle)
                    .count(),
                2
            );
            assert_eq!(result.plan().invocations(), 811_008);
        }
    }
}

#[test]
fn one_fullviewport_rounded_color_stays_below_the_existing_invocation_cap() {
    // A direct one-Box scene. A real Page's extra fullviewport canvas paint can
    // still exceed the unchanged invocation cap and is not claimed admitted.
    let commands = [command(rect(0.0, 0.0, 1280.0, 1024.0), [[1.0; 2]; 4], 255)];
    let result = check(
        "fullviewport",
        Frame::new(1280, 1024, 0xffffff),
        &commands,
        &corners(1280, 1024, 0xff0000, 0xff3030),
    );
    inventory(&result, 6, 4_604, 2_048, 3_938_888, 7);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 3_938_888);
    assert_eq!(result.stats().gpu_buffer_upper_bound, 10_514_432);
    assert_eq!(result.plan().invocations(), 3_969_024);
    assert_eq!(
        result
            .plan()
            .draws()
            .iter()
            .filter(|d| d.kind() == DrawKind::Glyph)
            .count(),
        6
    );
}

#[test]
fn oversized_used_ellipses_keep_seven_absolute_edge_tiles_without_renormalization() {
    assert_eq!(SEVEN_ROW.len(), 1280 * 4);
    let row: Vec<u32> = SEVEN_ROW
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            assert_eq!(p[3], 255);
            u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2])
        })
        .collect();
    let mut expected = vec![0xff0000; 1280 * 205];
    expected[..1280].copy_from_slice(&row);
    expected[1280 * 204..].copy_from_slice(&row);
    let commands = [command(
        rect(0.0, 0.0, 1280.0, 205.0),
        [[1100.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1100.0, 1.0]],
        255,
    )];
    let result = check(
        "seven-tiles",
        Frame::new(1280, 205, 0xffffff),
        &commands,
        &expected,
    );
    inventory(&result, 7, 226_063, 613, 1_032_096, 8);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 1_032_096);
    let bounds: Vec<_> = result
        .plan()
        .draws()
        .iter()
        .filter(|d| d.kind() == DrawKind::Glyph)
        .map(|d| d.bounds())
        .collect();
    assert_eq!(
        bounds,
        [
            (0, 0, 1024, 1),
            (1024, 0, 256, 1),
            (0, 204, 1024, 1),
            (1024, 204, 256, 1),
            (0, 1, 1024, 203),
            (1024, 1, 76, 203),
            (1279, 1, 1, 203),
        ]
    );
}

#[test]
fn oversized_rounded_color_fractional_clip_keeps_integer_origin_membership() {
    let frame = Frame {
        caller_clip: CoreRect::new(0.5, 0.25, 513.0, 513.0),
        ..Frame::new(514, 514, 0xffffff)
    };
    let commands = [command(rect(0.0, 0.0, 514.0, 514.0), [[1.0; 2]; 4], 255)];
    let mut expected = vec![0xffffff; 514 * 514];
    for y in 1..514 {
        for x in 1..514 {
            expected[y * 514 + x] = 0xff0000;
        }
    }
    expected[514 * 514 - 1] = 0xff3030;
    let result = check("fractional-clip", frame, &commands, &expected);
    inventory(&result, 2, 1_025, 513, 791_073, 3);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 791_073);
}

#[test]
fn page_selected_rounded_background_admits_after_literal_geometry_and_software_controls() {
    let page = Page::from_html(
        Url::parse("https://rounded-core.invalid/").unwrap(),
        PAGE,
        false,
    );
    let fonts = Fonts::new();
    let layout = page.layout(512.0, 513.0, &fonts);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert_eq!(page.title(), "Rounded Color core");
    assert!(page.images.is_empty());
    assert_eq!(
        layout.commands.len(),
        2,
        "white canvas plus selected rounded Color"
    );
    match &layout.commands[0] {
        DrawCommand::Rect {
            rect: area,
            color,
            radius,
        } => {
            assert_eq!(*area, rect(0.0, 0.0, 512.0, 513.0));
            assert_eq!(*color, Color::WHITE);
            assert_eq!(*radius, 0.0);
        }
        other => panic!("unexpected canvas paint: {other:?}"),
    }
    match &layout.commands[1] {
        DrawCommand::BoxDecoration(value) => {
            assert_eq!(
                value.shape,
                RoundedBox {
                    rect: rect(0.0, 0.0, 512.0, 513.0),
                    radii: [[1.0; 2]; 4]
                }
            );
            assert_eq!(value.cutout, None);
            match &value.paint {
                BoxPaint::Color(color) => assert_eq!(*color, Color::rgb(255, 0, 0)),
                _ => panic!("wrong selected paint"),
            }
        }
        other => panic!("padding-box clip must produce BoxDecoration: {other:?}"),
    }
    assert_eq!(
        layout.hit_test(2.0, 2.0),
        page.document.query_selector("#board")
    );
    let frame = Frame::new(512, 513, 0xffffff);
    let expected = corners(512, 513, 0xff0000, 0xff3030);
    pixels(
        "Page software",
        512,
        &canvas(frame, &layout.commands, &fonts, &page.images),
        &expected,
    );
    let result = plan_native_scene(
        frame,
        &[NativePhase {
            frame,
            commands: &layout.commands,
        }],
        &page.images,
        &fonts,
    )
    .expect("selected Page rounded Color must admit after software and shape controls");
    pixels(
        "Page packed Native",
        512,
        &witness::packed_pixels(result.plan()),
        &expected,
    );
    inventory(&result, 4, 2_046, 1_024, 791_114, 6);
    assert_eq!(result.stats().bridge.original_commands, 2);
    assert_eq!(result.stats().box_rectangles, 0);
    assert_eq!(result.stats().cpu_pixel_upper_bound, 1_053_770);
    assert_eq!(result.plan().invocations(), 1_077_248);
}
