//! Host checkpoints must settle mutations before default actions and layout.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Page, TaskState},
};
use url::Url;
#[path = "support/microtasks.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

fn page(html: &str, scripts: bool) -> Page {
    Page::from_html(
        Url::parse("https://microtask.invalid/").unwrap(),
        html,
        scripts,
    )
}
fn check(page: &Page, fonts: &Fonts, red_x: u32, title: &str, state: TaskState) {
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
        for (selector, x) in [("#box", red_x), ("#link", 20)] {
            assert_eq!(
                layout.hit_test(x as f32 + 1.0, 3.0),
                page.document.query_selector(selector)
            );
        }
        assert_eq!(page.title(), title);
        assert_eq!(page.task_state(), state);
    }
}
fn failure_diagnostic(page: &Page) {
    assert_eq!(page.diagnostics.len(), 1, "{:?}", page.diagnostics);
    assert!(page.diagnostics[0].starts_with("script:"));
    assert!(page.diagnostics[0].contains("script instruction limit exceeded"));
}
#[test]
fn microtask_script_readiness_listener_and_default_action_order_has_literal_pixels() {
    let fonts = Fonts::new();
    let mut page = page(oracle::NORMAL, true);
    check(&page, &fonts, 6, oracle::INITIAL, TaskState::Idle);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let target = page.document.query_selector("#box").unwrap();
    assert!(page.click(target).is_none());
    check(&page, &fonts, 10, oracle::CLICKED, TaskState::Idle);
    let link = page.document.query_selector("#link").unwrap();
    let navigation = page.click(link).unwrap();
    assert_eq!(navigation.address, "https://microtask.invalid/chosen.html");
    assert!(navigation.form_body.is_none());
    assert_eq!(page.document.attr(link, "href"), Some("chosen.html"));
    check(&page, &fonts, 10, oracle::LINK, TaskState::Idle);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}
#[test]
fn microtask_fatal_job_stays_inert_across_layout_tasks_and_sync_input_until_new_realm() {
    let fonts = Fonts::new();
    let mut failed = page(oracle::FAILURE, true);
    check(&failed, &fonts, 6, "queued", TaskState::Suspended);
    failure_diagnostic(&failed);
    for _ in 0..2 {
        failed.run_pending_tasks();
        check(&failed, &fonts, 6, "queued", TaskState::Suspended);
        failure_diagnostic(&failed);
    }
    let target = failed.document.query_selector("#box").unwrap();
    assert!(failed.click(target).is_none());
    failed.run_pending_tasks();
    check(&failed, &fonts, 6, "clicked", TaskState::Suspended);
    failure_diagnostic(&failed);
    drop(failed);
    let fresh = page(oracle::NORMAL, true);
    check(&fresh, &fonts, 6, oracle::INITIAL, TaskState::Idle);
    assert!(fresh.diagnostics.is_empty());
    let mut disabled = page(oracle::FAILURE, false);
    check(&disabled, &fonts, 2, "before", TaskState::Idle);
    let target = disabled.document.query_selector("#box").unwrap();
    assert!(disabled.click(target).is_none());
    disabled.run_pending_tasks();
    check(&disabled, &fonts, 2, "before", TaskState::Idle);
    assert!(disabled.diagnostics.is_empty());
}
