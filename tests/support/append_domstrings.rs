use eris::{
    dom::{Document, DomString, NodeId, NodeKind},
    graphics::{Canvas, Color, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/append-domstrings.html");

const LIVE_UNITS: [[[u16; 2]; 3]; 2] = [
    [[65, 55296], [66, 56320], [86, 55296]],
    [[67, 56320], [68, 55296], [87, 56320]],
];
const SCALAR_REFERENCE: [[&str; 3]; 2] = [["A�", "B�", "V�"], ["C�", "D�", "W�"]];

fn data(doc: &Document, id: NodeId) -> &DomString {
    match &doc.nodes[id].kind {
        NodeKind::Text(data) | NodeKind::Comment(data) => data,
        other => panic!("expected Text/Comment at {id}, got {other:?}"),
    }
}

fn exact_units(doc: &Document, id: NodeId, expected: &[u16]) {
    assert_eq!(data(doc, id).units().collect::<Vec<_>>(), expected);
}

fn text_children(doc: &Document, parent: NodeId, count: usize) -> Vec<NodeId> {
    let children = doc.nodes[parent].children.clone();
    assert_eq!(children.len(), count);
    for &id in &children {
        assert!(matches!(doc.nodes[id].kind, NodeKind::Text(_)));
        assert_eq!(doc.nodes[id].parent, Some(parent));
        assert!(doc.nodes[id].children.is_empty());
    }
    children
}

fn retained_element(doc: &Document, name: &str) -> NodeId {
    let matching: Vec<_> = (0..doc.nodes.len())
        .filter(|&id| doc.attr(id, "id") == Some(name))
        .collect();
    assert_eq!(matching.len(), 1, "unique retained #{name}");
    matching[0]
}

fn failure_fragment(doc: &Document, expected: &[u16]) -> (NodeId, NodeId) {
    let matching: Vec<_> = doc
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(id, node)| {
            if node.parent.is_some()
                || !matches!(node.kind, NodeKind::DocumentFragment { host: None })
                || node.children.len() != 1
            {
                return None;
            }
            let text = node.children[0];
            let NodeKind::Text(data) = &doc.nodes[text].kind else {
                return None;
            };
            data.units()
                .eq(expected.iter().copied())
                .then_some((id, text))
        })
        .collect();
    assert_eq!(matching.len(), 1, "one retained failure with literal units");
    let (fragment, text) = matching[0];
    assert_eq!(doc.nodes[text].parent, Some(fragment));
    assert!(doc.nodes[text].children.is_empty());
    (fragment, text)
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

pub struct AppendState {
    document: NodeId,
    document_children: Vec<NodeId>,
    root: NodeId,
    button: NodeId,
    live: [NodeId; 3],
    status: NodeId,
    branch: NodeId,
    branch_children: [NodeId; 2],
    pair: [NodeId; 3],
    failure: (NodeId, NodeId),
}

pub fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&AppendState>,
) -> AppendState {
    assert!(phase < 2);
    let exact = doc.query_selector("#exact").unwrap();
    let value = doc.query_selector("#value").unwrap();
    let status = doc.query_selector("#result").unwrap();
    let pair_host = doc.query_selector("#pair-value").unwrap();
    let root = doc.query_selector("html").unwrap();
    let pair: [NodeId; 3] = doc.nodes[pair_host].children.clone().try_into().unwrap();
    let branch = retained_element(doc, "old-branch");
    let branch_children: [NodeId; 2] = doc.nodes[branch].children.clone().try_into().unwrap();
    let live = text_children(doc, exact, 2);
    assert_ne!(live[0], live[1], "append arguments are distinct Texts");
    let state = AppendState {
        document: doc.root,
        document_children: doc.nodes[doc.root].children.clone(),
        root,
        button: doc.query_selector("#change").unwrap(),
        live: [live[0], live[1], text_children(doc, value, 1)[0]],
        status: text_children(doc, status, 1)[0],
        branch,
        branch_children,
        pair,
        failure: failure_fragment(doc, &[if phase == 0 { 55296 } else { 56320 }]),
    };
    assert!(matches!(doc.nodes[state.document].kind, NodeKind::Document));
    assert_eq!(doc.nodes[root].parent, Some(state.document));
    let elements: Vec<_> = state
        .document_children
        .iter()
        .copied()
        .filter(|&id| matches!(doc.nodes[id].kind, NodeKind::Element(_)))
        .collect();
    assert_eq!(elements, [root], "restored unique document element");
    for (&id, expected) in state.live.iter().zip(&LIVE_UNITS[phase]) {
        exact_units(doc, id, expected);
        assert_eq!(data(doc, id).stored_bytes(), 4);
    }
    let mut visits = 100_000;
    assert_eq!(
        doc.text_content_units_bounded(exact, 4, &mut visits)
            .unwrap(),
        if phase == 0 {
            [65, 55296, 66, 56320]
        } else {
            [67, 56320, 68, 55296]
        }
    );

    assert_eq!(doc.nodes[branch].parent, None);
    assert!(matches!(
        doc.nodes[branch_children[0]].kind,
        NodeKind::Text(_)
    ));
    assert!(matches!(
        doc.nodes[branch_children[1]].kind,
        NodeKind::Comment(_)
    ));
    for id in branch_children {
        assert_eq!(doc.nodes[id].parent, Some(branch));
    }
    exact_units(doc, branch_children[0], &[111, 108, 100, 55296]);
    exact_units(doc, branch_children[1], &[56320]);

    assert_ne!(pair[0], pair[2]);
    assert!(matches!(doc.nodes[pair[0]].kind, NodeKind::Text(_)));
    assert!(matches!(doc.nodes[pair[1]].kind, NodeKind::Comment(_)));
    assert!(matches!(doc.nodes[pair[2]].kind, NodeKind::Text(_)));
    for id in pair {
        assert_eq!(doc.nodes[id].parent, Some(pair_host));
    }
    exact_units(doc, pair[0], &[55296]);
    exact_units(doc, pair[1], &[101, 120, 99, 108, 117, 100, 101, 100]);
    exact_units(doc, pair[2], &[56320]);
    let mut visits = 100_000;
    assert_eq!(
        doc.text_content_units_bounded(pair_host, 2, &mut visits)
            .unwrap(),
        [55296, 56320]
    );
    assert_eq!(doc.nodes[state.failure.0].children, [state.failure.1]);
    assert_eq!(doc.nodes[state.failure.0].parent, None);
    assert_eq!(doc.nodes[state.failure.1].parent, Some(state.failure.0));

    if let Some(old) = previous {
        assert_eq!(phase, 1);
        assert_eq!(state.document, old.document);
        assert_eq!(state.document_children, old.document_children);
        assert_eq!(state.root, old.root);
        assert_eq!(state.button, old.button);
        assert_eq!(state.status, old.status);
        assert_eq!(state.branch, old.branch);
        assert_eq!(state.branch_children, old.branch_children);
        assert_eq!(state.pair, old.pair);
        for (&id, expected) in old.live.iter().zip(&LIVE_UNITS[0]) {
            assert!(!state.live.contains(&id));
            assert_eq!(doc.nodes[id].parent, None);
            exact_units(doc, id, expected);
        }
        assert_ne!(state.failure.0, old.failure.0);
        assert_ne!(state.failure.1, old.failure.1);
        assert_eq!(failure_fragment(doc, &[55296]), old.failure);
        assert_eq!(doc.nodes[old.failure.0].parent, None);
        assert_eq!(doc.nodes[old.failure.0].children, [old.failure.1]);
        assert_eq!(doc.nodes[old.failure.1].parent, Some(old.failure.0));
        exact_units(doc, old.failure.1, &[55296]);
    } else {
        assert_eq!(phase, 0);
    }

    // Preserve exact node boundaries. Expected scalar glyph strings are literal,
    // never obtained by projecting the observed nonscalar payloads.
    let mut reference = Page::from_html(doc.url().clone(), HTML, false);
    reference.document = doc.clone();
    let count = reference.document.nodes.len();
    let bytes = reference.document.retained_bytes();
    for (&id, expected) in state.live.iter().zip(SCALAR_REFERENCE[phase]) {
        let parent = reference.document.nodes[id].parent;
        assert_eq!(expected.len(), 4);
        reference.document.set_text_content(id, expected);
        assert_eq!(data(&reference.document, id).scalar(), Some(expected));
        assert_eq!(data(&reference.document, id).stored_bytes(), 4);
        assert_eq!(reference.document.nodes[id].parent, parent);
    }
    assert_eq!(reference.document.nodes.len(), count);
    assert_eq!(reference.document.retained_bytes(), bytes);
    assert_eq!(
        reference.document.nodes[exact].children,
        [state.live[0], state.live[1]]
    );
    let actual = paint(layout, fonts, images);
    let expected = paint(
        &reference.layout(320.0, 240.0, fonts),
        fonts,
        &reference.images,
    );
    assert_eq!(
        actual.pixels, expected.pixels,
        "phase={phase}: literal scalar reference"
    );
    let body = doc.query_selector("body").unwrap();
    assert_eq!(doc.attr(body, "class"), Some(["ready", "clicked"][phase]));
    assert_eq!(doc.text_content(status), ["ready", "6"][phase]);
    let color = [0x008000, 0x0000ff][phase];
    for x in [20, 70, 120, 170, 220, 270] {
        assert_eq!(actual.pixels[20 * 320 + x], color, "phase={phase}, x={x}");
    }
    for (top, bottom) in [(100, 132), (135, 167)] {
        assert!(
            actual.pixels[top * 320..bottom * 320]
                .iter()
                .any(|pixel| *pixel != 0xffffff)
        );
    }
    for (&id, expected) in state.live.iter().zip(&LIVE_UNITS[phase]) {
        exact_units(doc, id, expected);
    }
    assert_eq!(doc.nodes[exact].children, [state.live[0], state.live[1]]);
    assert_eq!(doc.nodes[value].children, [state.live[2]]);
    state
}
