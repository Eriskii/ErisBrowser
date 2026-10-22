use super::tests::editing_browser;
use super::*;

fn node(browser: &Browser, selector: &str) -> NodeId {
    browser
        .snapshot
        .as_ref()
        .unwrap()
        .document
        .query_selector(selector)
        .unwrap()
}

fn acknowledgement(browser: &Browser, sequence: u64, pointer_events: PointerEvents) -> Snapshot {
    let previous = browser.snapshot.as_ref().unwrap();
    let document = previous.document.clone();
    let styles = eris::css::compute_styles(&document, &document.stylesheets(), 1180.0, 739.0);
    let mut layout = eris::layout::layout(&document, &styles, 1180.0, 739.0, &browser.fonts);
    for hit in &mut layout.hit_regions {
        hit.pointer_events = pointer_events;
    }
    Snapshot {
        generation: previous.generation,
        processed_edit_sequence: sequence,
        task_state: previous.task_state,
        document,
        layout,
        images: eris::graphics::ImageStore::new(),
        title: previous.title.clone(),
        url: previous.url.clone(),
        diagnostics: Vec::new(),
        load_ms: 0.0,
    }
}

#[test]
fn pointer_events_browser_skips_only_ineligible_points_with_fixed_scroll_and_zoom() {
    let mut browser =
        editing_browser("<button id=normal>normal</button><button id=fixed>fixed</button>");
    let normal = node(&browser, "#normal");
    let fixed = node(&browser, "#fixed");
    browser.snapshot.as_mut().unwrap().layout.hit_regions = vec![
        eris::layout::HitRegion {
            node: normal,
            action: HitAction::Node,
            pointer_events: PointerEvents::Auto,
            rect: Rect {
                x: 20.0,
                y: 420.0,
                width: 20.0,
                height: 20.0,
            },
            fixed: false,
        },
        eris::layout::HitRegion {
            node: fixed,
            action: HitAction::Node,
            pointer_events: PointerEvents::None,
            rect: Rect {
                x: 20.0,
                y: 20.0,
                width: 20.0,
                height: 20.0,
            },
            fixed: true,
        },
    ];
    browser.zoom = 2.0;
    browser.scroll = 800.0;
    browser.cursor = (60.0, TOOLBAR + 60.0);
    assert_eq!(browser.hit(), Some(normal));
    browser.snapshot.as_mut().unwrap().layout.hit_regions[1].pointer_events = PointerEvents::Auto;
    assert_eq!(browser.hit(), Some(fixed));
    browser.cursor.0 = 80.0;
    assert_eq!(browser.hit(), None, "right edges are excluded");
    browser.cursor = (60.0, TOOLBAR - 1.0);
    assert_eq!(browser.hit(), None);
    browser.cursor = (60.0, TOOLBAR + 60.0);
    browser.snapshot.as_mut().unwrap().generation = 2;
    assert_eq!(browser.hit(), None, "stale generation remains ineligible");
    browser.snapshot.as_mut().unwrap().generation = 1;
    for hit in &mut browser.snapshot.as_mut().unwrap().layout.hit_regions {
        hit.pointer_events = PointerEvents::None;
    }
    assert_eq!(browser.hit(), None);
    assert_eq!(
        browser.snapshot.as_ref().unwrap().layout.hit_regions.len(),
        2
    );
}

#[test]
fn pointer_events_browser_none_keeps_offscreen_tab_and_default_summary_keyboard_activation() {
    let mut browser = editing_browser(
        "<style>#field{position:absolute;top:2000px}</style><input id=field value=base><button id=button>go</button><details id=details>contents</details>",
    );
    let field = node(&browser, "#field");
    let button = node(&browser, "#button");
    let details = node(&browser, "#details");
    for hit in &mut browser.snapshot.as_mut().unwrap().layout.hit_regions {
        hit.pointer_events = PointerEvents::None;
    }
    browser.visible_nodes = visible_layout_nodes(browser.snapshot.as_ref().unwrap());
    for id in [field, button, details] {
        assert!(browser.visible_nodes[id]);
    }
    browser.tab_focus();
    assert_eq!(browser.focused, Some(field));
    assert!(browser.editable(field));
    assert_eq!(browser.scroll, 0.0);
    browser.tab_focus();
    assert_eq!(browser.focused, Some(button));
    browser.key(Key::Named(NamedKey::Enter), None);
    assert!(
        matches!(browser.tx.state.lock().unwrap().pending.pop_front(), Some(Request::Click { generation: 1, node }) if node == button)
    );
    browser.tab_focus();
    assert_eq!(browser.focused, Some(details));
    browser.key(Key::Named(NamedKey::Space), None);
    assert!(
        matches!(browser.tx.state.lock().unwrap().pending.pop_front(), Some(Request::DefaultSummary { generation: 1, node }) if node == details)
    );
    browser.modifiers = ModifiersState::SHIFT;
    browser.tab_focus();
    assert_eq!(browser.focused, Some(button));
}

#[test]
fn pointer_events_browser_snapshot_none_preserves_pending_edit_and_old_ack_cannot_replace_it() {
    let mut browser = editing_browser("<input id=field value=base>");
    let field = node(&browser, "#field");
    browser.focus_input(field);
    browser.input_value = "pending".into();
    browser.edit_sequence = 4;
    let pending = acknowledgement(&browser, 0, PointerEvents::None);
    browser.accept_snapshot(pending);
    assert_eq!(browser.focused, Some(field));
    assert_eq!(browser.input_value, "pending");
    assert!(browser.editable(field));
    let accepted = acknowledgement(&browser, 4, PointerEvents::None);
    browser.accept_snapshot(accepted);
    assert_eq!(browser.focused, Some(field));
    assert_eq!(browser.input_value, "base");
    let stale = acknowledgement(&browser, 3, PointerEvents::Auto);
    browser.accept_snapshot(stale);
    assert_eq!(
        browser.snapshot.as_ref().unwrap().processed_edit_sequence,
        4
    );
    assert!(
        browser
            .snapshot
            .as_ref()
            .unwrap()
            .layout
            .hit_regions
            .iter()
            .all(|hit| hit.pointer_events == PointerEvents::None)
    );
    assert!(browser.tx.state.lock().unwrap().pending.is_empty());
}

#[test]
fn pointer_events_browser_fragment_navigation_keeps_none_geometry() {
    let mut browser = editing_browser("<div id=target></div>");
    let target = node(&browser, "#target");
    let snapshot = browser.snapshot.as_mut().unwrap();
    snapshot.layout.content_height = 3000.0;
    snapshot.layout.hit_regions = vec![eris::layout::HitRegion {
        node: target,
        action: HitAction::Node,
        pointer_events: PointerEvents::None,
        rect: Rect {
            x: 0.0,
            y: 1230.0,
            width: 8.0,
            height: 8.0,
        },
        fixed: false,
    }];
    browser.jump_to("target");
    assert_eq!(browser.scroll, 1230.0);
    browser.scroll = 0.0;
    browser.snapshot.as_mut().unwrap().layout.hit_regions[0].fixed = true;
    browser.jump_to("target");
    assert_eq!(browser.scroll, 0.0);
}
