//! Whole-page row-background pixels specified independently of table layout.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::Page,
};
use url::Url;

#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

const WIDTH: u32 = 40;
const HEIGHT: u32 = 32;
const WHITE: u32 = 0xffffff;
const YELLOW: u32 = 0xffff00;
const RED: u32 = 0xff0000;
const BLUE: u32 = 0x0000ff;

fn compare(actual: &[u32], expected: &[u32]) {
    assert_eq!(actual.len(), expected.len());
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "pixel ({},{}) actual={:06x} expected={:06x}",
            index % WIDTH as usize,
            index / WIDTH as usize,
            actual[index],
            expected[index]
        );
    }
}
fn check(width: u32, body: &str, rectangles: &[(u32, u32, u32, u32, u32)]) {
    let html = format!(
        "<!doctype html><style>html,body{{margin:0;padding:0;background:white}}table{{position:absolute;left:2px;top:2px;box-sizing:content-box;width:{width}px;margin:0;padding:0;border:0;border-spacing:2px 1px;background:yellow}}tr{{height:2px}}td{{box-sizing:content-box;height:2px;margin:0;padding:0;border:0;background:transparent;vertical-align:top}}</style><table>{body}</table>"
    );
    let page = Page::from_html(
        Url::parse("https://row-paint.invalid/").unwrap(),
        &html,
        false,
    );
    let fonts = Fonts::new();
    let layout = page.layout(WIDTH as f32, HEIGHT as f32, &fonts);
    let mut expected = vec![WHITE; (WIDTH * HEIGHT) as usize];
    for &(x, y, w, h, color) in rectangles {
        assert!(x + w <= WIDTH && y + h <= HEIGHT);
        for row in y..y + h {
            for column in x..x + w {
                expected[(row * WIDTH + column) as usize] = color;
            }
        }
    }
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    compare(&canvas.pixels, &expected);
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
        let frame = eris_raster_core::Frame::new(WIDTH, HEIGHT, WHITE);
        let native = plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            &page.images,
            &fonts,
        )
        .unwrap();
        compare(&witness::packed_pixels(native.plan()), &expected);
    }
}

#[test]
fn table_background_remains_visible_between_real_empty_cells() {
    check(
        14,
        "<tr style='background:red'><td></td><td></td></tr><tr style='background:blue'><td></td><td></td></tr>",
        &[
            (2, 2, 14, 7, YELLOW),
            (4, 3, 4, 2, RED),
            (10, 3, 4, 2, RED),
            (4, 6, 4, 2, BLUE),
            (10, 6, 4, 2, BLUE),
        ],
    );
}

#[test]
fn rowspans_retain_the_originating_row_background_through_the_vertical_gap() {
    check(
        14,
        "<tr style='background:red'><td rowspan=2></td><td></td></tr><tr style='background:blue'><td></td></tr>",
        &[
            (2, 2, 14, 7, YELLOW),
            (4, 3, 4, 5, RED),
            (10, 3, 4, 2, RED),
            (10, 6, 4, 2, BLUE),
        ],
    );
}

#[test]
fn colspan_background_is_continuous_across_its_internal_track_gap() {
    check(
        20,
        "<tr style='background:red'><td colspan=2></td><td></td></tr><tr style='background:blue'><td></td><td></td><td></td></tr>",
        &[
            (2, 2, 20, 7, YELLOW),
            (4, 3, 10, 2, RED),
            (16, 3, 4, 2, RED),
            (4, 6, 4, 2, BLUE),
            (10, 6, 4, 2, BLUE),
            (16, 6, 4, 2, BLUE),
        ],
    );
}

#[test]
fn missing_slots_and_an_empty_row_do_not_paint_invented_cells() {
    check(
        14,
        "<tr style='background:red'><td></td></tr><tr style='background:blue'><td></td><td></td></tr><tr style='background:lime'></tr>",
        &[
            (2, 2, 14, 10, YELLOW),
            (4, 3, 4, 2, RED),
            (4, 6, 4, 2, BLUE),
            (10, 6, 4, 2, BLUE),
        ],
    );
}

#[test]
fn row_gradient_phase_uses_final_tracks_when_fixed_cells_overflow() {
    check(
        10,
        "<tr style='background:linear-gradient(90deg,red 50%,blue 50%)'><td style='width:6px'></td><td style='width:6px'></td></tr>",
        &[
            (2, 2, 10, 4, YELLOW),
            (4, 3, 6, 2, RED),
            (12, 3, 6, 2, BLUE),
        ],
    );
}
