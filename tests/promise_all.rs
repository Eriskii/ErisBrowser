//! Promise.all preserves input order and runs reactions at host checkpoints.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Page, TaskState},
};
use url::Url;
#[path = "support/promise_all.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

fn check(page: &Page, fonts: &Fonts, red_x: u32, title: &str) {
    for width in [320u32, 300] {
        let layout = page.layout(width as f32, 240.0, fonts);
        let expected = oracle::expected(width, 240, red_x);
        let mut canvas = Canvas::new(width, 240).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        oracle::assert_pixels(&canvas.pixels, &expected, width);
        #[cfg(feature = "raster-bridge")]
        {
            use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
            let frame = eris_raster_core::Frame::new(width, 240, oracle::WHITE);
            let scene = plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &layout.commands,
                }],
                &page.images,
                fonts,
            )
            .unwrap();
            oracle::assert_pixels(&witness::packed_pixels(scene.plan()), &expected, width);
        }
        assert_eq!(
            layout.hit_test(red_x as f32 + 1.0, 3.0),
            page.document.query_selector("#red")
        );
        assert_eq!(
            layout.hit_test(21.0, 3.0),
            page.document.query_selector("#blue")
        );
        assert_eq!(page.title(), title);
        assert_eq!(page.task_state(), TaskState::Idle);
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    }
}
#[test]
fn promise_all_collects_in_order_and_drains_before_the_next_host_listener() {
    let fonts = Fonts::new();
    let address = Url::parse("https://promise-all.invalid/").unwrap();
    let mut disabled = Page::from_html(address.clone(), oracle::HTML, false);
    check(&disabled, &fonts, 2, "base");
    let target = disabled.document.query_selector("#red").unwrap();
    assert!(disabled.click(target).is_none());
    disabled.run_pending_tasks();
    check(&disabled, &fonts, 2, "base");
    assert!(disabled.runtime.console.is_empty());

    let mut page = Page::from_html(address, oracle::HTML, true);
    check(&page, &fonts, 12, oracle::INITIAL);
    assert!(page.runtime.console.is_empty());
    page.run_pending_tasks();
    check(&page, &fonts, 12, oracle::INITIAL);
    assert!(page.runtime.console.is_empty());
    let target = page.document.query_selector("#red").unwrap();
    assert!(page.click(target).is_none());
    check(&page, &fonts, 16, oracle::CLICKED);
    assert!(page.runtime.console.is_empty());
    page.run_pending_tasks();
    check(&page, &fonts, 16, oracle::CLICKED);
    assert!(page.runtime.console.is_empty());
}
