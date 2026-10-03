use eris::{
    dom::{Document, DomString, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/node-normalize.html");

pub fn metadata(phase: usize) -> &'static str {
    ["Normalize ready", "Normalize done"][phase]
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
    parents: [NodeId; 8],
    runs: [[NodeId; 5]; 8],
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
    let details = find("#disclosure");
    let summary = find("#summary");
    let parents = [
        find("#scalar"),
        find("#exact"),
        find("#heading"),
        find("#theme"),
        find("#field"),
        content,
        template,
        details,
    ];
    let runs = if phase == 0 {
        assert!(previous.is_none());
        parents.map(|parent| doc.nodes[parent].children[..5].try_into().unwrap())
    } else {
        previous
            .expect("phase one requires initial retained IDs")
            .runs
    };
    for (index, (&parent, ids)) in parents.iter().zip(runs).enumerate() {
        let expected = if phase == 0 {
            ids.to_vec()
        } else {
            vec![ids[1]]
        };
        let mut children = expected;
        if index == 7 {
            children.push(summary);
        }
        assert_eq!(doc.nodes[parent].children, children);
        for (position, &id) in ids.iter().enumerate() {
            assert!(matches!(doc.nodes[id].kind, NodeKind::Text(_)));
            assert!(doc.nodes[id].children.is_empty());
            let attached = phase == 0 || position == 1;
            assert_eq!(doc.nodes[id].parent, attached.then_some(parent));
            for &other in &ids[..position] {
                assert_ne!(id, other);
            }
        }
        for position in [0, 2, 4] {
            scalar(doc, ids[position], "");
        }
    }
    let all_ids = runs.into_iter().flatten().collect::<Vec<_>>();
    for (index, id) in all_ids.iter().enumerate() {
        assert!(
            !all_ids[..index].contains(id),
            "separate original Text identities"
        );
    }
    assert_eq!(doc.nodes[summary].parent, Some(details));
    assert_eq!(doc.nodes[details].children[[5, 1][phase]], summary);
    assert_eq!(doc.first_summary(details), Some(summary));
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
        runs,
    };

    let strings = [
        ["Alpha ", "beta"],
        ["", ""], // Exact UTF-16 literals below cover this run.
        ["Normalize ", ["ready", "done"][phase]],
        ["#probe{", ["background:green}", "background:blue}"][phase]],
        ["clean ", ["seed", "next"][phase]],
        ["con", "tent"],
        ["out", "side"],
        ["be", "fore"],
    ];
    for (index, (&parent, ids)) in parents.iter().zip(runs).enumerate() {
        if index == 1 {
            continue;
        }
        let [left, right] = strings[index];
        let joined = [left, right].concat();
        scalar(doc, ids[1], if phase == 0 { left } else { &joined });
        scalar(doc, ids[3], right);
        // The summary is a non-Text barrier, retained after the details run.
        let expected = if index == 7 {
            [joined.as_str(), "kept summary"].concat()
        } else {
            joined
        };
        aggregate(doc, parent, &expected.encode_utf16().collect::<Vec<_>>());
    }
    if phase == 0 {
        units(doc, runs[1][1], &[65, 55357]);
        assert!(data(doc, runs[1][1]).scalar().is_none());
        assert_eq!(data(doc, runs[1][1]).stored_bytes(), 4);
    } else {
        units(doc, runs[1][1], &[65, 55357, 56960, 66]);
        scalar(doc, runs[1][1], "A🚀B");
        assert_eq!(data(doc, runs[1][1]).stored_bytes(), 6);
    }
    units(doc, runs[1][3], &[56960, 66]);
    assert!(data(doc, runs[1][3]).scalar().is_none());
    assert_eq!(data(doc, runs[1][3]).stored_bytes(), 4);
    aggregate(doc, parents[1], &[65, 55357, 56960, 66]);
    assert_eq!(
        doc.attr(state.base, "href"),
        Some("https://normalize.example/fixed/")
    );
    assert_eq!(doc.base_url().as_str(), "https://normalize.example/fixed/");
    assert_eq!(doc.title(), metadata(phase));
    assert_eq!(
        doc.attr(find("body"), "class"),
        Some(["ready", "clicked"][phase])
    );

    if let Some(old) = previous {
        assert_eq!(phase, 1);
        assert_eq!(state.root, old.root);
        assert_eq!(state.nodes, old.nodes, "normalization creates no nodes");
        assert_eq!(state.button, old.button);
        assert_eq!(state.base, old.base);
        assert_eq!(state.template, old.template);
        assert_eq!(state.content, old.content);
        assert_eq!(state.details, old.details);
        assert_eq!(state.summary, old.summary);
        assert_eq!(state.summary_text, old.summary_text);
        assert_eq!(state.parents, old.parents);
        assert_eq!(state.runs, old.runs);
    }

    // Before normalize, each unmatched half is rendered independently. After
    // normalize, the surviving single scalar Text contains the literal rocket.
    // Reference strings are authored literals, not projections of observed data.
    let mut reference = Page::from_html(doc.url().clone(), HTML, false);
    reference.document = doc.clone();
    let count = reference.document.nodes.len();
    let bytes = reference.document.retained_bytes();
    if phase == 0 {
        for (id, expected) in [(runs[1][1], "A�"), (runs[1][3], "�B")] {
            assert_eq!(expected.len(), 4);
            reference.document.set_text_content(id, expected);
            scalar(&reference.document, id, expected);
        }
    } else {
        reference.document.set_text_content(runs[1][1], "A🚀B");
        scalar(&reference.document, runs[1][1], "A🚀B");
        units(&reference.document, runs[1][3], &[56960, 66]);
        assert_eq!(reference.document.nodes[runs[1][3]].parent, None);
    }
    assert_eq!(reference.document.nodes.len(), count);
    assert_eq!(reference.document.retained_bytes(), bytes);
    assert_eq!(
        reference.document.nodes[parents[1]].children,
        doc.nodes[parents[1]].children
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
    // Reference construction must not mutate the observed exact or detached data.
    units(doc, runs[1][3], &[56960, 66]);
    assert_eq!(
        doc.nodes[runs[1][3]].parent,
        (phase == 0).then_some(parents[1])
    );
    state
}
