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

fn acknowledgement(browser: &Browser, sequence: u64, hidden: bool) -> Snapshot {
    let previous = browser.snapshot.as_ref().unwrap();
    let mut document = previous.document.clone();
    let field = document.query_selector("#field").unwrap();
    document.set_attr(
        field,
        "style",
        if hidden {
            "visibility:hidden"
        } else {
            "visibility:visible"
        },
    );
    let styles = eris::css::compute_styles(&document, &document.stylesheets(), 1180.0, 739.0);
    let layout = eris::layout::layout(&document, &styles, 1180.0, 739.0, &browser.fonts);
    Snapshot {
        generation: previous.generation,
        processed_edit_sequence: sequence,
        task_state: previous.task_state,
        layout,
        images: eris::graphics::ImageStore::new(),
        document,
        title: previous.title.clone(),
        url: previous.url.clone(),
        diagnostics: Vec::new(),
        load_ms: 0.0,
    }
}

#[test]
fn visibility_browser_visible_child_does_not_focus_hidden_link_or_suppress_click() {
    let mut browser = editing_browser(
        "<style>body{margin:0}#parent{visibility:hidden}#child{visibility:visible}#far{position:absolute;top:2000px}</style><a id=parent href='#target'><span id=child>visible</span></a><input id=far><button id=last>last</button>",
    );
    let parent = node(&browser, "#parent");
    let child = node(&browser, "#child");
    let far = node(&browser, "#far");
    let last = node(&browser, "#last");
    let snapshot = browser.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.layout.node_visibility[parent], Visibility::Hidden);
    assert_eq!(snapshot.layout.node_visibility[child], Visibility::Visible);
    let hit = snapshot
        .layout
        .hit_regions
        .iter()
        .find(|hit| snapshot.document.nodes[hit.node].parent == Some(child))
        .unwrap();
    let target = hit.node;
    let rect = hit.rect;
    assert!(!browser.visible_nodes[parent]);
    assert!(browser.visible_nodes[child]);
    browser.focus_input(parent);
    assert_eq!(browser.focused, None);
    browser.cursor = (
        rect.x + rect.width / 2.0,
        TOOLBAR + rect.y + rect.height / 2.0,
    );
    browser.click();
    assert_eq!(browser.focused, None);
    let request = browser.tx.state.lock().unwrap().pending.pop_front();
    assert!(matches!(request, Some(Request::Click { generation: 1, node }) if node == target));
    browser.tab_focus();
    assert_eq!(browser.focused, Some(far));
    browser.tab_focus();
    assert_eq!(browser.focused, Some(last));
    browser.key(Key::Named(NamedKey::Enter), None);
    let request = browser.tx.state.lock().unwrap().pending.pop_front();
    assert!(matches!(request, Some(Request::Click { generation: 1, node }) if node == last));
    browser.modifiers = ModifiersState::SHIFT;
    browser.tab_focus();
    assert_eq!(browser.focused, Some(far));
    assert_eq!(browser.scroll, 0.0);
}

#[test]
fn visibility_browser_accepted_hidden_snapshot_revokes_pending_edit_and_stale_one_cannot() {
    let mut browser = editing_browser("<input id=field value=base>");
    let field = node(&browser, "#field");
    browser.focus_input(field);
    assert!(browser.editable(field));
    browser.input_value = "unacknowledged".into();
    browser.edit_sequence = 4;
    let hidden = acknowledgement(&browser, 0, true);
    browser.accept_snapshot(hidden);
    assert_eq!(browser.focused, None);
    assert!(!browser.editable(field));
    assert!(browser.input_value.is_empty());
    browser.insert_text("must not queue");
    assert!(!browser.tx.has_pending());

    let shown = acknowledgement(&browser, 4, false);
    browser.accept_snapshot(shown);
    browser.tab_focus();
    assert_eq!(browser.focused, Some(field));
    let stale = acknowledgement(&browser, 3, true);
    browser.accept_snapshot(stale);
    assert_eq!(browser.focused, Some(field));
    assert!(browser.editable(field));
    assert_eq!(
        browser.snapshot.as_ref().unwrap().layout.node_visibility[field],
        Visibility::Visible
    );
}

#[test]
fn visibility_browser_metadata_does_not_replace_reachability_or_revive_missing_summary() {
    let mut browser = editing_browser(
        "<details id=details><span id=child>child</span></details><input id=field>",
    );
    let field = node(&browser, "#field");
    let details = node(&browser, "#details");
    let child = node(&browser, "#child");
    let snapshot = browser.snapshot.as_mut().unwrap();
    snapshot.layout.node_visibility.fill(Visibility::Visible);
    snapshot.layout.hit_regions = vec![eris::layout::HitRegion {
        node: child,
        action: HitAction::Node,
        rect: Rect {
            x: 0.0,
            y: 2000.0,
            width: 8.0,
            height: 8.0,
        },
        fixed: false,
    }];
    let mask = visible_layout_nodes(snapshot);
    assert!(
        !mask[field],
        "visible metadata alone cannot invent layout reachability"
    );
    assert!(
        !mask[details],
        "a details control without an actual default-summary hit stays ineligible"
    );
    assert!(mask[child], "offscreen geometry remains reachable");
    snapshot.layout.node_visibility.pop();
    assert!(
        visible_layout_nodes(snapshot)
            .iter()
            .all(|visible| !visible)
    );
}
