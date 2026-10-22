use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Page, TaskState},
};
use url::Url;
#[path = "support/supports_plain.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

#[test]
fn supports_plain_condition_matches_literal_pixels_direct() {
    let fonts = Fonts::new();
    let page = Page::from_html(
        Url::parse("https://supports-plain.invalid/").unwrap(),
        oracle::HTML,
        false,
    );
    let layout = page.layout(oracle::WIDTH as f32, oracle::HEIGHT as f32, &fonts);
    let mut canvas = Canvas::new(oracle::WIDTH, oracle::HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    oracle::assert_pixels(&canvas.pixels);
    assert_eq!(page.task_state(), TaskState::Idle);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
        let frame = eris_raster_core::Frame::new(oracle::WIDTH, oracle::HEIGHT, 0xffffff);
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
        oracle::assert_pixels(&witness::packed_pixels(scene.plan()));
    }
}
