//! Literal page geometry and pixels; no shader or graphics-driver execution.
use eris::{
    dom::NodeKind,
    graphics::{
        Canvas, Color, DrawCommand, Fonts,
        raster_bridge::native::{NativePhase, plan_native_scene},
    },
    layout::LayoutResult,
    page::Page,
};
use eris_raster_core::{DrawKind, Frame, PARAM_STRIDE, Plan, Profile};

pub const HTML: &str = include_str!("../../examples/vulkan-lines.html");
pub const WIDTH: u32 = 512;
pub const HEIGHT: u32 = 512;
const COLOR: u32 = 0xc02040;
// Frozen from binary32 layout arithmetic, before candidate planner execution.
const UNDERLINE_Y: [u32; 9] = [
    0x4207ae14, 0x4283d70a, 0x42c3d70a, 0x4301eb85, 0x4321eb85, 0x4341eb85, 0x4361eb85, 0x4380f5c3,
    0x4390f5c3,
];
const STRIKE_Y: [u32; 9] = [
    0x41d33333, 0x4269999a, 0x42b4cccd, 0x42f4cccd, 0x431a6666, 0x433a6666, 0x435a6666, 0x437a6666,
    0x438d3333,
];

fn check_color(color: Color) {
    assert_eq!((color.r, color.g, color.b, color.a), (192, 32, 64, 255));
}

// Keep the exact binary32 decimal values used by the independent oracle.
#[allow(clippy::excessive_precision)]
pub fn check_layout(page: &Page, layout: &LayoutResult) {
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert!(page.runtime.console.is_empty());
    assert!(page.images.is_empty());
    assert_eq!(page.title(), "Native line geometry");
    assert_eq!(layout.commands.len(), 27);
    assert!(layout.content_height <= HEIGHT as f32);
    for index in 0..9 {
        let id = page
            .document
            .query_selector(&format!("#line{index}"))
            .unwrap();
        assert_eq!(page.document.nodes[id].children.len(), 1);
        let text = page.document.nodes[id].children[0];
        let NodeKind::Text(value) = &page.document.nodes[text].kind else {
            panic!("retained nonbreaking-space Text");
        };
        assert_eq!(value.scalar(), Some("\u{a0}"));
        let DrawCommand::Text {
            x,
            y,
            text,
            size,
            color,
            bold,
            italic,
            monospace,
        } = &layout.commands[index * 3]
        else {
            panic!("each row begins with its original Text command");
        };
        assert_eq!((*x, *y), (16.0, (16 + index * 32) as f32));
        assert_eq!(text, "\u{a0}");
        assert_eq!(
            (*size, *bold, *italic, *monospace),
            (16.0, false, false, true)
        );
        check_color(*color);
        for (ordinal, expected_y) in [UNDERLINE_Y[index], STRIKE_Y[index]]
            .into_iter()
            .enumerate()
        {
            let DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } = &layout.commands[index * 3 + ordinal + 1]
            else {
                panic!("underline then strike-through remain original Lines");
            };
            assert_eq!((*x1, *x2, *width), (16.0, 25.6328125, 1.0));
            assert_eq!((y1.to_bits(), y2.to_bits()), (expected_y, expected_y));
            check_color(*color);
        }
    }
}

pub fn expected_pixels() -> Vec<u32> {
    let mut pixels = vec![0xffffff; (WIDTH * HEIGHT) as usize];
    for index in 0..9 {
        for top in [33 + index * 32, 26 + index * 32] {
            for y in top..top + 2 {
                pixels[y * WIDTH as usize + 16..y * WIDTH as usize + 26].fill(COLOR);
            }
        }
    }
    pixels
}

fn packed_rectangles(plan: &Plan) -> Vec<u32> {
    assert_eq!(plan.profile(), Profile::Native);
    assert_eq!(plan.draws().len(), 19);
    assert!(!plan.has_images());
    assert!(!plan.has_glyphs());
    assert!(plan.input_bytes().is_empty());
    assert_eq!(plan.group_scratch_bytes(), 0);
    assert_eq!(plan.invocations(), 526_592);
    let mut pixels = vec![0; (WIDTH * HEIGHT) as usize];
    for (index, draw) in plan.draws().iter().enumerate() {
        assert_eq!(draw.kind(), DrawKind::Rectangle);
        assert!(!draw.target_is_group());
        let expected = if index == 0 {
            (0, 0, WIDTH, HEIGHT, 0xffffff)
        } else {
            let row = (index - 1) / 2;
            let top = if (index - 1) % 2 == 0 { 33 } else { 26 } + row * 32;
            (16, top as u32, 10, 2, COLOR)
        };
        assert_eq!(
            draw.bounds(),
            (expected.0, expected.1, expected.2, expected.3)
        );
        let record = &plan.parameters()[index * PARAM_STRIDE..(index + 1) * PARAM_STRIDE];
        let word =
            |index: usize| u32::from_le_bytes(record[index * 4..index * 4 + 4].try_into().unwrap());
        assert_eq!((word(4), word(5)), (WIDTH, HEIGHT));
        assert_eq!((word(0), word(1), word(2), word(3)), draw.bounds());
        assert_eq!(word(6), 0xff000000 | expected.4);
        for y in expected.1..expected.1 + expected.3 {
            let start = (y * WIDTH + expected.0) as usize;
            pixels[start..start + expected.2 as usize].fill(expected.4);
        }
    }
    pixels
}

pub fn check_pixels_and_native(page: &Page, layout: &LayoutResult, fonts: &Fonts) {
    let expected = expected_pixels();
    assert_eq!(
        expected.iter().filter(|&&pixel| pixel == COLOR).count(),
        360
    );
    let frame = Frame::new(WIDTH, HEIGHT, 0xffffff);
    for _ in 0..2 {
        let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert_eq!(canvas.pixels, expected);
        let native = plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            &page.images,
            fonts,
        )
        .expect("eighteen short page Lines admit geometric work within unchanged limits");
        let stats = native.stats();
        assert_eq!(stats.phases, 1);
        assert_eq!(stats.bridge.original_commands, 27);
        assert_eq!(stats.bridge.lowered_commands, 27);
        assert_eq!(stats.cpu_pixel_upper_bound, 369);
        assert_eq!(stats.text_input_bytes, 18);
        assert_eq!(stats.text_input_scalars, 9);
        assert_eq!(stats.text.occurrences, 9);
        assert_eq!(stats.text.cold_requests, 1);
        assert_eq!(stats.text.paint_pixels_used, 9);
        assert_eq!(stats.mask_sources, 1);
        assert_eq!(stats.coverage_bytes, 0);
        assert_eq!(stats.row_entries, 0);
        assert_eq!(stats.rounded_masks, 0);
        assert_eq!(stats.box_masks, 0);
        assert_eq!(packed_rectangles(native.plan()), expected);
        assert!(native.plan().gpu_buffer_bytes() <= stats.gpu_buffer_upper_bound);
        assert!(stats.gpu_buffer_upper_bound <= 16 * 1024 * 1024);
    }
}
