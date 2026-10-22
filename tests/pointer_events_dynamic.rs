use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Page, TaskState},
};
use url::Url;
#[path = "support/pointer_events_dynamic.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

fn check(
    page: &Page,
    fonts: &Fonts,
    main: &str,
    title: &str,
    baselines: &mut [Option<oracle::Geometry>; 2],
) -> oracle::Targets {
    let mut targets = None;
    for (index, width) in oracle::WIDTHS.into_iter().enumerate() {
        let layout = page.layout(width as f32, oracle::HEIGHT as f32, fonts);
        let expected = oracle::expected(width);
        let mut canvas = Canvas::new(width, oracle::HEIGHT).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        oracle::assert_pixels(&canvas.pixels, &expected, width);
        #[cfg(feature = "raster-bridge")]
        {
            use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
            let frame = eris_raster_core::Frame::new(width, oracle::HEIGHT, 0xffffff);
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
        oracle::assert_geometry(&layout, &mut baselines[index]);
        let current = oracle::assert_targets(&page.document, &layout, main);
        if let Some(previous) = targets {
            assert_eq!(current, previous, "target IDs changed with viewport width");
        }
        targets = Some(current);
        assert_eq!(page.title(), title);
        assert_eq!(page.task_state(), TaskState::Idle);
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    }
    targets.unwrap()
}

#[test]
fn pointer_events_dynamic_keeps_pixels_geometry_and_auto_child_event_ancestors() {
    let fonts = Fonts::new();
    for scripts in [false, true] {
        let mut page = Page::from_html(
            Url::parse("https://pointer-events-dynamic.invalid/").unwrap(),
            oracle::HTML,
            scripts,
        );
        let mut baselines = [None, None];
        let mut targets: Option<oracle::Targets> = None;
        for (action, main, title) in oracle::PHASES {
            let click = match action {
                oracle::Action::Initial => None,
                oracle::Action::ClickToggle => Some(targets.unwrap().toggle),
                oracle::Action::ClickMain => Some(targets.unwrap().main),
                oracle::Action::ClickChild => Some(targets.unwrap().child),
                oracle::Action::DrainTasks => {
                    page.run_pending_tasks();
                    None
                }
            };
            if let Some(node) = click {
                assert!(page.click(node).is_none());
            }
            targets = Some(check(
                &page,
                &fonts,
                if scripts { main } else { "over" },
                if scripts { title } else { "auto" },
                &mut baselines,
            ));
        }
    }
}
