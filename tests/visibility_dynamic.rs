use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Page, TaskState},
};
use url::Url;
#[path = "support/visibility_dynamic.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

fn check(page: &Page, fonts: &Fonts, shown: bool, title: &str) {
    for width in [320u32, 300] {
        let layout = page.layout(width as f32, 240.0, fonts);
        let expected = oracle::expected(width, 240, shown);
        let mut canvas = Canvas::new(width, 240).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        oracle::assert_pixels(&canvas.pixels, &expected, width);
        #[cfg(feature = "raster-bridge")]
        {
            use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
            let frame = eris_raster_core::Frame::new(width, 240, 0xffffff);
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
        let box_node = page.document.query_selector("#box").unwrap();
        assert_eq!(
            layout.hit_test(1.0, 1.0),
            page.document.query_selector("#toggle")
        );
        assert_eq!(
            layout.hit_test(1.0, 9.0),
            page.document.query_selector("#after")
        );
        assert_eq!(layout.hit_test(5.0, 5.0) == Some(box_node), shown);
        assert_eq!(page.title(), title);
        assert_eq!(page.task_state(), TaskState::Idle);
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    }
}

#[test]
fn visibility_dynamic_hiding_preserves_space_and_visible_child_event_bubbling() {
    let fonts = Fonts::new();
    for scripts in [false, true] {
        let mut page = Page::from_html(
            Url::parse("https://visibility-dynamic.invalid/").unwrap(),
            oracle::HTML,
            scripts,
        );
        check(&page, &fonts, false, "hidden");
        let toggle = page.document.query_selector("#toggle").unwrap();
        assert!(page.click(toggle).is_none());
        check(
            &page,
            &fonts,
            scripts,
            if scripts { "shownB" } else { "hidden" },
        );
        assert!(page.click(toggle).is_none());
        check(
            &page,
            &fonts,
            false,
            if scripts { "hiddenB" } else { "hidden" },
        );
        page.run_pending_tasks();
        check(
            &page,
            &fonts,
            false,
            if scripts { "hiddenB" } else { "hidden" },
        );
    }
}
