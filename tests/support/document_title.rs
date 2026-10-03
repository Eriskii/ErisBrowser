use eris::{
    dom::{Document, DomString, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/document-title.html");
const STATUS_UNITS: [[u16; 2]; 3] = [[76, 55296], [66, 56320], [80, 55296]];
const SCALAR_REFERENCE: [&str; 3] = ["L�", "B�", "P�"];

// These expectations come from literal inputs, never observed engine output.
pub fn metadata(phase: usize, url: &str, worker: bool) -> String {
    match phase {
        0 => "A\u{fffd} B\u{00a0}\u{000b}".into(),
        1 => url.into(),
        2 => {
            let mut title = "A".repeat(511);
            title.push('🚀');
            if !worker {
                title.push('\u{fffd}');
            }
            title
        }
        _ => panic!("invalid phase"),
    }
}

fn raw_title(phase: usize) -> Vec<u16> {
    match phase {
        0 => vec![32, 9, 65, 55296, 13, 10, 32, 66, 160, 11, 32],
        1 => vec![32, 9, 13, 10, 12, 32],
        2 => {
            let mut units = vec![65; 511];
            units.extend([55357, 56960, 55296]);
            units
        }
        _ => panic!("invalid phase"),
    }
}

fn data(doc: &Document, id: NodeId) -> &DomString {
    match &doc.nodes[id].kind {
        NodeKind::Text(data) | NodeKind::Comment(data) => data,
        other => panic!("expected Text/Comment at {id}, got {other:?}"),
    }
}

fn exact(doc: &Document, id: NodeId, expected: &[u16]) {
    assert_eq!(data(doc, id).units().collect::<Vec<_>>(), expected);
}

fn single_text(doc: &Document, parent: NodeId) -> NodeId {
    let children = &doc.nodes[parent].children;
    assert_eq!(children.len(), 1);
    let id = children[0];
    assert!(matches!(doc.nodes[id].kind, NodeKind::Text(_)));
    assert_eq!(doc.nodes[id].parent, Some(parent));
    assert!(doc.nodes[id].children.is_empty());
    id
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

pub struct TitleState {
    subject: NodeId,
    button: NodeId,
    status: NodeId,
    status_text: NodeId,
    branch: NodeId,
    branch_children: [NodeId; 2],
    title_texts: Vec<NodeId>,
}

pub fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&TitleState>,
) -> TitleState {
    assert!(phase < 3);
    let subject = doc.query_selector("#subject").unwrap();
    let status = doc.query_selector("#status").unwrap();
    let retained: Vec<_> = (0..doc.nodes.len())
        .filter(|&id| doc.attr(id, "id") == Some("old-title-branch"))
        .collect();
    assert_eq!(retained.len(), 1);
    let branch = retained[0];
    let branch_children: [NodeId; 2] = doc.nodes[branch].children.clone().try_into().unwrap();
    let mut title_texts = previous.map_or_else(Vec::new, |old| old.title_texts.clone());
    assert_eq!(title_texts.len(), phase);
    let current = single_text(doc, subject);
    assert!(!title_texts.contains(&current));
    title_texts.push(current);
    let state = TitleState {
        subject,
        button: doc.query_selector("#change").unwrap(),
        status,
        status_text: single_text(doc, status),
        branch,
        branch_children,
        title_texts,
    };
    for (old_phase, &id) in state.title_texts.iter().enumerate() {
        exact(doc, id, &raw_title(old_phase));
        assert_eq!(
            doc.nodes[id].parent,
            (old_phase == phase).then_some(subject)
        );
        assert!(doc.nodes[id].children.is_empty());
    }
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
    exact(doc, branch_children[0], &[111, 108, 100, 55296]);
    exact(doc, branch_children[1], &[56320]);
    exact(doc, state.status_text, &STATUS_UNITS[phase]);
    if let Some(old) = previous {
        assert_eq!(state.subject, old.subject);
        assert_eq!(state.button, old.button);
        assert_eq!(state.status, old.status);
        assert_eq!(state.status_text, old.status_text);
        assert_eq!(state.branch, old.branch);
        assert_eq!(state.branch_children, old.branch_children);
    } else {
        assert_eq!(phase, 0);
    }

    let scalar = SCALAR_REFERENCE[phase];
    assert!(
        layout
            .commands
            .iter()
            .any(|command| { matches!(command, DrawCommand::Text { text, .. } if text == scalar) })
    );
    let mut reference = Page::from_html(doc.url().clone(), HTML, false);
    reference.document = doc.clone();
    let count = reference.document.nodes.len();
    let bytes = reference.document.retained_bytes();
    assert_eq!(data(doc, state.status_text).stored_bytes(), scalar.len());
    reference
        .document
        .set_text_content(state.status_text, scalar);
    assert_eq!(
        data(&reference.document, state.status_text).scalar(),
        Some(scalar)
    );
    assert_eq!(reference.document.nodes.len(), count);
    assert_eq!(reference.document.retained_bytes(), bytes);
    assert_eq!(single_text(&reference.document, status), state.status_text);
    let actual = paint(layout, fonts, images);
    let expected = paint(
        &reference.layout(320.0, 240.0, fonts),
        fonts,
        &reference.images,
    );
    assert_eq!(
        actual.pixels, expected.pixels,
        "phase={phase}: literal glyph reference"
    );
    let body = doc.query_selector("body").unwrap();
    assert_eq!(
        doc.attr(body, "class"),
        Some(["ready", "empty", "long"][phase])
    );
    for x in [20, 70, 120, 170, 220, 270] {
        assert_eq!(
            actual.pixels[20 * 320 + x],
            [0x008000, 0x0000ff, 0x800080][phase]
        );
    }
    assert!(
        actual.pixels[105 * 320..140 * 320]
            .iter()
            .any(|pixel| *pixel != 0xffffff)
    );
    exact(doc, state.status_text, &STATUS_UNITS[phase]);
    exact(doc, current, &raw_title(phase));
    assert_eq!(single_text(doc, subject), current);
    state
}
