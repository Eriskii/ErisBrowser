use eris::{
    dom::{Document, DomString, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/text-operations.html");

pub fn metadata(phase: usize) -> &'static str {
    ["Text ready", "Text changed"][phase]
}

fn data(doc: &Document, id: NodeId) -> &DomString {
    match &doc.nodes[id].kind {
        NodeKind::Text(data) => data,
        other => panic!("expected Text at {id}, got {other:?}"),
    }
}

fn units(doc: &Document, id: NodeId, expected: &[u16]) {
    assert_eq!(data(doc, id).units().collect::<Vec<_>>(), expected);
    assert!(doc.nodes[id].children.is_empty());
}

fn scalar(doc: &Document, id: NodeId, expected: &str) {
    units(doc, id, &expected.encode_utf16().collect::<Vec<_>>());
    assert_eq!(data(doc, id).scalar(), Some(expected));
}

fn pair(doc: &Document, parent: NodeId) -> [NodeId; 2] {
    let children: [NodeId; 2] = doc.nodes[parent].children.clone().try_into().unwrap();
    assert_ne!(children[0], children[1]);
    for id in children {
        assert!(matches!(doc.nodes[id].kind, NodeKind::Text(_)));
        assert_eq!(doc.nodes[id].parent, Some(parent));
        assert!(doc.nodes[id].children.is_empty());
    }
    children
}

fn aggregate(doc: &Document, parent: NodeId, expected: &[u16]) {
    let mut visits = 100_000;
    assert_eq!(
        doc.text_content_units_bounded(parent, expected.len(), &mut visits)
            .unwrap(),
        expected
    );
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

pub struct State {
    root: NodeId,
    nodes: usize,
    button: NodeId,
    base: NodeId,
    template: NodeId,
    content: NodeId,
    details: NodeId,
    summary: NodeId,
    summary_text: NodeId,
    parents: [NodeId; 7],
    pairs: [[NodeId; 2]; 7],
    detail_texts: Vec<NodeId>,
}

pub fn check(
    doc: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    phase: usize,
    previous: Option<&State>,
) -> State {
    assert!(phase < 2);
    let find = |id: &str| doc.query_selector(id).unwrap();
    let template = find("#template");
    let content = doc.template_contents(template).unwrap();
    assert!(matches!(
        doc.nodes[content].kind,
        NodeKind::DocumentFragment { host: Some(host) } if host == template
    ));
    assert_eq!(doc.nodes[content].parent, None);
    let parents = [
        find("#scalar"),
        find("#exact"),
        find("#heading"),
        find("#theme"),
        find("#field"),
        content,
        template,
    ];
    let pairs = parents.map(|parent| pair(doc, parent));
    let details = find("#disclosure");
    let summary = find("#summary");
    assert_eq!(doc.nodes[details].children.len(), phase + 3);
    assert_eq!(doc.nodes[details].children[phase + 2], summary);
    assert_eq!(doc.nodes[summary].parent, Some(details));
    assert_eq!(doc.first_summary(details), Some(summary));
    let detail_texts = doc.nodes[details].children[..phase + 2].to_vec();
    for &id in &detail_texts {
        assert_eq!(doc.nodes[id].parent, Some(details));
        assert!(doc.nodes[id].children.is_empty());
    }
    scalar(doc, detail_texts[0], "bef");
    scalar(doc, detail_texts[1], ["ore", "o"][phase]);
    if phase == 1 {
        scalar(doc, detail_texts[2], "re");
    }
    let summary_children = &doc.nodes[summary].children;
    assert_eq!(summary_children.len(), 1);
    let summary_text = summary_children[0];
    assert_eq!(doc.nodes[summary_text].parent, Some(summary));
    scalar(doc, summary_text, "kept summary");
    let state = State {
        root: doc.root,
        nodes: doc.nodes.len(),
        button: find("#change"),
        base: find("#fixed-base"),
        template,
        content,
        details,
        summary,
        summary_text,
        parents,
        pairs,
        detail_texts,
    };

    let strings = [
        ["Alpha ", ["beta", "gamma"][phase]],
        ["", ""], // The exact nonscalar pair is checked by literal units below.
        ["Text ", ["ready", "changed"][phase]],
        ["#probe{", ["background:green}", "background:blue}"][phase]],
        ["clean ", ["seed", "next"][phase]],
        ["con", ["tent", "tained"][phase]],
        ["out", ["side", "bound"][phase]],
    ];
    for (index, (&parent, children)) in parents.iter().zip(pairs).enumerate() {
        if index == 1 {
            continue;
        }
        for (id, expected) in children.into_iter().zip(strings[index]) {
            scalar(doc, id, expected);
        }
        let expected = strings[index].concat();
        aggregate(doc, parent, &expected.encode_utf16().collect::<Vec<_>>());
    }
    units(doc, pairs[1][0], &[65, 55357]);
    units(doc, pairs[1][1], &[56960, [66, 67][phase]]);
    aggregate(doc, parents[1], &[65, 55357, 56960, [66, 67][phase]]);
    for id in pairs[1] {
        assert!(data(doc, id).scalar().is_none());
        assert_eq!(data(doc, id).stored_bytes(), 4);
    }
    assert_ne!(pairs[5], pairs[6]);
    assert_eq!(
        doc.attr(state.base, "href"),
        Some("https://text.example/fixed/")
    );
    assert_eq!(doc.base_url().as_str(), "https://text.example/fixed/");
    assert_eq!(doc.title(), metadata(phase));
    assert_eq!(
        doc.attr(find("body"), "class"),
        Some(["ready", "clicked"][phase])
    );

    if let Some(old) = previous {
        assert_eq!(phase, 1);
        assert_eq!(state.root, old.root);
        assert_eq!(
            state.nodes,
            old.nodes + 1,
            "only the second details split creates a node"
        );
        assert_eq!(state.button, old.button);
        assert_eq!(state.base, old.base);
        assert_eq!(state.template, old.template);
        assert_eq!(state.content, old.content);
        assert_eq!(state.details, old.details);
        assert_eq!(state.summary, old.summary);
        assert_eq!(state.summary_text, old.summary_text);
        assert_eq!(state.parents, old.parents);
        assert_eq!(state.pairs, old.pairs);
        assert_eq!(&state.detail_texts[..2], old.detail_texts.as_slice());
        assert!(!old.detail_texts.contains(&state.detail_texts[2]));
        assert!(state.detail_texts[2] >= old.nodes);
    } else {
        assert_eq!(phase, 0);
    }

    // Literal per-node scalar glyph references preserve both Text boundaries.
    // Do not derive these expected strings from the observed Units payloads.
    let expected_exact = ["A�", ["�B", "�C"][phase]];
    let mut reference = Page::from_html(doc.url().clone(), HTML, false);
    reference.document = doc.clone();
    let count = reference.document.nodes.len();
    let bytes = reference.document.retained_bytes();
    for (id, expected) in pairs[1].into_iter().zip(expected_exact) {
        assert_eq!(expected.len(), 4);
        reference.document.set_text_content(id, expected);
        scalar(&reference.document, id, expected);
        assert_eq!(data(&reference.document, id).stored_bytes(), 4);
    }
    assert_eq!(reference.document.nodes.len(), count);
    assert_eq!(reference.document.retained_bytes(), bytes);
    assert_eq!(pair(&reference.document, parents[1]), pairs[1]);
    let actual = paint(layout, fonts, images);
    let expected = paint(
        &reference.layout(320.0, 240.0, fonts),
        fonts,
        &reference.images,
    );
    assert_eq!(
        actual.pixels, expected.pixels,
        "phase={phase}: scalar reference"
    );
    for x in [20, 70] {
        assert_eq!(actual.pixels[20 * 320 + x], [0x008000, 0x0000ff][phase]);
    }
    for (top, bottom) in [(100, 132), (135, 167)] {
        assert!(
            actual.pixels[top * 320..bottom * 320]
                .iter()
                .any(|pixel| *pixel != 0xffffff),
            "phase={phase}: nonempty glyph band"
        );
    }
    assert!(layout.commands.iter().any(|command| {
        matches!(command, DrawCommand::Text { text, .. } if text == ["clean seed", "clean next"][phase])
    }));
    units(doc, pairs[1][0], &[65, 55357]);
    units(doc, pairs[1][1], &[56960, [66, 67][phase]]);
    assert_eq!(pair(doc, parents[1]), pairs[1]);
    state
}
