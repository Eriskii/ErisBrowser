//! Independent whole-page pixels for separated table spacing.
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
const GREEN: u32 = 0x00ff00;
const BLACK: u32 = 0;

fn expected(rectangles: &[(u32, u32, u32, u32, u32)]) -> Vec<u32> {
    let mut pixels = vec![WHITE; (WIDTH * HEIGHT) as usize];
    for &(x, y, w, h, color) in rectangles {
        assert!(x + w <= WIDTH && y + h <= HEIGHT);
        for row in y..y + h {
            for column in x..x + w {
                pixels[(row * WIDTH + column) as usize] = color;
            }
        }
    }
    pixels
}
fn check(body: &str, table: &str, rectangles: &[(u32, u32, u32, u32, u32)]) {
    let html = format!(
        "<!doctype html><style>html,body{{margin:0;padding:0;background:white}}body{{{body}}}table{{position:absolute;left:2px;top:2px;box-sizing:content-box;width:20px;margin:0;padding:0;border:0;background:yellow;{table}}}tr{{background:transparent}}td{{margin:0;padding:0;border:0;height:4px}}#c,#d{{height:6px}}#a{{background:red}}#b{{background:blue}}#c{{background:lime}}#d{{background:black}}</style><table id=board><tr><td id=a></td><td id=b></td></tr><tr><td id=c></td><td id=d></td></tr></table>"
    );
    let page = Page::from_html(
        Url::parse("https://spacing.invalid/").unwrap(),
        &html,
        false,
    );
    let fonts = Fonts::new();
    let layout = page.layout(WIDTH as f32, HEIGHT as f32, &fonts);
    let literal = expected(rectangles);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, literal, "table CSS: {table}");
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
        let frame = eris_raster_core::Frame::new(WIDTH, HEIGHT, WHITE);
        let scene = plan_native_scene(
            frame,
            &[NativePhase {
                frame,
                commands: &layout.commands,
            }],
            &page.images,
            &fonts,
        )
        .unwrap();
        assert_eq!(witness::packed_pixels(scene.plan()), literal);
    }
}
const UNEQUAL: &[(u32, u32, u32, u32, u32)] = &[
    (2, 2, 20, 19, YELLOW),
    (4, 5, 7, 4, RED),
    (13, 5, 7, 4, BLUE),
    (4, 12, 7, 6, GREEN),
    (13, 12, 7, 6, BLACK),
];
#[test]
fn unequal_axes_include_outer_and_inner_gaps() {
    check("", "border-spacing:2px 3px", UNEQUAL);
}
#[test]
fn zero_removes_outer_and_inner_gaps() {
    check(
        "",
        "border-spacing:0",
        &[
            (2, 2, 20, 10, YELLOW),
            (2, 2, 10, 4, RED),
            (12, 2, 10, 4, BLUE),
            (2, 6, 10, 6, GREEN),
            (12, 6, 10, 6, BLACK),
        ],
    );
}
#[test]
fn inherited_relative_spacing_uses_parent_computed_lengths() {
    check(
        "font-size:10px;border-spacing:.2em calc(.5em - 2px)",
        "font-size:20px;border-spacing:inherit",
        UNEQUAL,
    );
}
#[test]
fn final_negative_calculation_clamps_without_rejecting_the_pair() {
    check(
        "",
        "border-spacing:calc(1px - 3px) calc(2px + 1px)",
        &[
            (2, 2, 20, 19, YELLOW),
            (2, 5, 10, 4, RED),
            (12, 5, 10, 4, BLUE),
            (2, 12, 10, 6, GREEN),
            (12, 12, 10, 6, BLACK),
        ],
    );
}
