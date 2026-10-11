//! Throw diagnostics cannot execute metadata getters in scripts or host callbacks.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Page, TaskState},
};
use url::Url;
#[path = "support/throw_completion.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

fn check(page: &Page, fonts: &Fonts, red_x: u32, title: &str) {
    for width in [40u32, 48] {
        let layout = page.layout(width as f32, 16.0, fonts);
        let expected = oracle::expected(width, 16, red_x);
        let mut canvas = Canvas::new(width, 16).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        oracle::assert_pixels(&canvas.pixels, &expected, width);
        #[cfg(feature = "raster-bridge")]
        {
            use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
            let frame = eris_raster_core::Frame::new(width, 16, oracle::WHITE);
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
            page.document.query_selector("#box")
        );
        assert_eq!(
            layout.hit_test(21.0, 3.0),
            page.document.query_selector("#trap")
        );
        assert_eq!(page.title(), title);
        assert_eq!(page.task_state(), TaskState::Idle);
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    }
}
#[test]
fn throw_completion_preserves_original_values_and_pixels_through_host_callbacks() {
    let fonts = Fonts::new();
    let address = Url::parse("https://throw-completion.invalid/").unwrap();
    let mut disabled = Page::from_html(address.clone(), oracle::HTML, false);
    check(&disabled, &fonts, 2, "before");
    let target = disabled.document.query_selector("#box").unwrap();
    assert!(disabled.click(target).is_none());
    disabled.run_pending_tasks();
    check(&disabled, &fonts, 2, "before");
    assert!(disabled.runtime.console.is_empty());

    let mut page = Page::from_html(address, oracle::HTML, true);
    check(&page, &fonts, 6, oracle::INITIAL);
    assert_eq!(
        page.runtime.console,
        ["Uncaught microtask exception: uncaught exception: [object Object]"]
    );
    page.run_pending_tasks();
    check(&page, &fonts, 6, oracle::INITIAL);
    let target = page.document.query_selector("#box").unwrap();
    assert!(page.click(target).is_none());
    check(&page, &fonts, 10, oracle::CLICKED);
    assert_eq!(
        page.runtime.console,
        [
            "Uncaught microtask exception: uncaught exception: [object Object]",
            "Uncaught event listener exception: uncaught exception: [object Object]"
        ]
    );
}
