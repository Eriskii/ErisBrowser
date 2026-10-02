use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{Canvas, Color, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const EXACT_HTML: &str = include_str!("../fixtures/node-data.html");
pub const EFFECTS_HTML: &str = include_str!("../fixtures/node-data-effects.html");

fn retained_element(doc: &Document, name: &str) -> NodeId {
    let matches: Vec<_> = (0..doc.nodes.len())
        .filter(|&id| doc.attr(id, "id") == Some(name))
        .collect();
    assert_eq!(matches.len(), 1, "unique retained #{name}");
    matches[0]
}
fn single_text(doc: &Document, host: NodeId) -> NodeId {
    assert_eq!(doc.nodes[host].children.len(), 1);
    let id = doc.nodes[host].children[0];
    assert!(matches!(doc.nodes[id].kind, NodeKind::Text(_)));
    assert_eq!(doc.nodes[id].parent, Some(host));
    id
}
fn exact_units(doc: &Document, id: NodeId, expected: &[u16]) {
    let data = match &doc.nodes[id].kind {
        NodeKind::Text(data) | NodeKind::Comment(data) => data,
        other => panic!("expected character data, got {other:?}"),
    };
    assert_eq!(data.units().collect::<Vec<_>>(), expected);
}
fn samples(doc: &Document, canvas: &Canvas, phase: usize) {
    assert!(!canvas.exhausted());
    let body = doc.query_selector("body").unwrap();
    assert_eq!(doc.attr(body, "class"), Some(["ready", "clicked"][phase]));
    let status = doc.query_selector("#result").unwrap();
    assert_eq!(doc.text_content(status), ["ready", "6"][phase]);
    let color = [0x008000, 0x0000ff][phase];
    for x in [20, 70, 120, 170, 220, 270] {
        assert_eq!(canvas.pixels[20 * 320 + x], color, "phase={phase}, x={x}");
    }
}
fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

pub struct ExactState {
    exact: NodeId,
    value: NodeId,
    status: NodeId,
    branch: NodeId,
}

pub fn check_exact(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&ExactState>,
) -> ExactState {
    use eris::graphics::DrawCommand;
    let exact_host = doc.query_selector("#exact").unwrap();
    let value_host = doc.query_selector("#value").unwrap();
    let status_host = doc.query_selector("#result").unwrap();
    let state = ExactState {
        exact: single_text(doc, exact_host),
        value: single_text(doc, value_host),
        status: single_text(doc, status_host),
        branch: retained_element(doc, "old-branch"),
    };
    let units = [[65, 55296, 66, 56320], [67, 56320, 68, 55296]][phase];
    let value_units = [[86, 55296], [87, 56320]][phase];
    exact_units(doc, state.exact, &units);
    exact_units(doc, state.value, &value_units);
    assert_eq!(doc.nodes[state.branch].parent, None);
    assert_eq!(doc.nodes[state.branch].children.len(), 2);
    let children = &doc.nodes[state.branch].children;
    assert_eq!(doc.nodes[children[0]].parent, Some(state.branch));
    assert_eq!(doc.nodes[children[1]].parent, Some(state.branch));
    assert!(matches!(doc.nodes[children[0]].kind, NodeKind::Text(_)));
    assert!(matches!(doc.nodes[children[1]].kind, NodeKind::Comment(_)));
    exact_units(doc, children[0], &[111, 108, 100, 55296]);
    exact_units(doc, children[1], &[56320]);
    if let Some(old) = previous {
        assert_eq!(phase, 1);
        assert_ne!(
            state.exact, old.exact,
            "container assignment creates a fresh Text"
        );
        assert_eq!(doc.nodes[old.exact].parent, None);
        exact_units(doc, old.exact, &[65, 55296, 66, 56320]);
        assert_eq!(
            (state.value, state.status, state.branch),
            (old.value, old.status, old.branch)
        );
    } else {
        assert_eq!(phase, 0);
    }
    let projected = [["A�B�", "V�"], ["C�D�", "W�"]][phase];
    for text in projected {
        assert!(layout.commands.iter().any(|command| {
            matches!(command, DrawCommand::Text { text: value, .. } if value == text)
        }));
    }
    // Preserve the actual tree but supply independently literal scalar glyph
    // input in a script-disabled reference. Never decode expectations from data.
    let mut reference = Page::from_html(doc.url().clone(), EXACT_HTML, false);
    reference.document = doc.clone();
    let bytes = reference.document.retained_bytes();
    let count = reference.document.nodes.len();
    for (id, text) in [(state.exact, projected[0]), (state.value, projected[1])] {
        let parent = reference.document.nodes[id].parent;
        reference.document.set_text_content(id, text);
        let NodeKind::Text(data) = &reference.document.nodes[id].kind else {
            panic!()
        };
        assert_eq!(data.scalar(), Some(text));
        assert_eq!(reference.document.nodes[id].parent, parent);
    }
    assert_eq!(reference.document.retained_bytes(), bytes);
    assert_eq!(reference.document.nodes.len(), count);
    let actual = paint(layout, fonts, images);
    let expected = paint(
        &reference.layout(320.0, 240.0, fonts),
        fonts,
        &reference.images,
    );
    assert_eq!(
        actual.pixels, expected.pixels,
        "phase={phase}: scalar literal pixels"
    );
    samples(doc, &actual, phase);
    for (top, bottom) in [(100, 132), (135, 167)] {
        assert!(
            actual.pixels[top * 320..bottom * 320]
                .iter()
                .any(|pixel| *pixel != 0xffffff)
        );
    }
    exact_units(doc, state.exact, &units);
    exact_units(doc, state.value, &value_units);
    state
}

pub struct EffectsState {
    visible: NodeId,
    field: NodeId,
    status: NodeId,
    summary: NodeId,
    summary_text: NodeId,
}

pub fn check_effects(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&EffectsState>,
) -> EffectsState {
    use eris::graphics::DrawCommand;
    let expected = ["ready", "changed"][phase];
    assert_eq!(doc.title(), expected);
    let visible = doc.query_selector("#visible").unwrap();
    let field = doc.query_selector("#field").unwrap();
    let status = doc.query_selector("#result").unwrap();
    let disclosure = doc.query_selector("#disclosure").unwrap();
    let summary = retained_element(doc, "old-summary");
    let state = EffectsState {
        visible: single_text(doc, visible),
        field: single_text(doc, field),
        status: single_text(doc, status),
        summary,
        summary_text: single_text(doc, summary),
    };
    assert_eq!(doc.text_content(visible), expected);
    assert_eq!(doc.text_content(field), expected);
    assert_eq!(doc.text_content(summary), "Old summary");
    if let Some(old) = previous {
        assert_eq!(phase, 1);
        for (before, after) in [(old.visible, state.visible), (old.field, state.field)] {
            assert_ne!(before, after);
            assert_eq!(doc.nodes[before].parent, None);
            assert_eq!(doc.text_content(before), "ready");
        }
        assert_eq!(
            (state.status, state.summary, state.summary_text),
            (old.status, old.summary, old.summary_text)
        );
        assert_eq!(doc.nodes[summary].parent, None);
        assert_eq!(doc.first_summary(disclosure), None);
        assert_eq!(doc.base_url(), doc.url());
        assert!(!layout.commands.iter().any(|command| {
            matches!(command, DrawCommand::Text { text, .. } if text == "Hidden replacement")
        }));
    } else {
        assert_eq!(phase, 0);
        assert_eq!(doc.first_summary(disclosure), Some(summary));
        assert_eq!(doc.nodes[summary].parent, Some(disclosure));
        assert_eq!(doc.base_url().as_str(), "https://second.example/b/");
    }
    let detached = retained_element(doc, "detached-details");
    let detached_summary = retained_element(doc, "detached-summary");
    let peer = doc.query_selector("#peer").unwrap();
    assert_eq!(doc.nodes[detached].parent, None);
    assert_eq!(doc.nodes[detached_summary].parent, Some(detached));
    assert!(doc.attr(detached, "open").is_some());
    assert!(doc.attr(peer, "open").is_some());
    assert_eq!(doc.text_content(detached_summary), "retained summary");
    let color = [Color::rgb(0, 128, 0), Color::rgb(0, 0, 255)][phase];
    assert!(layout.commands.iter().any(|command| {
        matches!(command, DrawCommand::Text { text, color: actual, .. } if text == expected && *actual == color)
    }));
    let canvas = paint(layout, fonts, images);
    samples(doc, &canvas, phase);
    state
}
