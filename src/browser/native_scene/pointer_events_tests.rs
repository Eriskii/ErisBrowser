use super::*;

fn paint_phases(browser: &Browser, scene: &Scene<'_>) -> Canvas {
    let mut canvas = Canvas::new(scene.target.width, scene.target.height).unwrap();
    canvas.clear(Color::WHITE);
    let empty = ImageStore::new();
    let images = browser.snapshot.as_ref().map_or(&empty, |s| &s.images);
    for phase in scene.phases() {
        let clip = phase.frame.caller_clip;
        canvas.set_clip(Rect {
            x: clip.x,
            y: clip.y,
            width: clip.width,
            height: clip.height,
        });
        canvas.paint_with_viewport(
            phase.commands,
            &browser.fonts,
            images,
            phase.frame.document_offset,
            phase.frame.viewport_offset,
        );
        assert!(!canvas.exhausted());
    }
    canvas
}

#[test]
fn pointer_events_native_none_preserves_complete_focus_outline_pixels() {
    for (zoom, fixed) in [(1.0, false), (1.25, true)] {
        let mut browser = super::super::tests::editing_browser("<input id=field value=base>");
        let snapshot = browser.snapshot.as_mut().unwrap();
        let node = snapshot.document.query_selector("#field").unwrap();
        snapshot.layout.content_height = 2000.0;
        snapshot.layout.hit_regions = vec![eris::layout::HitRegion {
            node,
            action: HitAction::Node,
            pointer_events: PointerEvents::Auto,
            rect: Rect {
                x: 24.0,
                y: 104.0,
                width: 80.0,
                height: 24.0,
            },
            fixed,
        }];
        browser.zoom = zoom;
        browser.scroll = 40.0;
        browser.status = "pointer focus".into();
        let size = PhysicalSize::new(1180, 880);
        let unfocused = browser.paint_canvas(size).unwrap().pixels;
        browser.focused = Some(node);
        let focused = browser.paint_canvas(size).unwrap().pixels;
        assert_ne!(
            focused, unfocused,
            "fixture must actually paint a focus outline"
        );
        browser.snapshot.as_mut().unwrap().layout.hit_regions[0].pointer_events =
            PointerEvents::None;
        assert_eq!(browser.paint_canvas(size).unwrap().pixels, focused);
        let scene = build(&browser, size).unwrap();
        assert_eq!(paint_phases(&browser, &scene).pixels, focused);
        assert_eq!(browser.focused, Some(node));
    }
}
