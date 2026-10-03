use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/node-position.html");

pub fn metadata(phase: usize) -> &'static str {
    ["Position ready", "Position done"][phase]
}

fn scalar(doc: &Document, id: NodeId, expected: &str) {
    let NodeKind::Text(data) = &doc.nodes[id].kind else {
        panic!("expected Text at {id}");
    };
    assert_eq!(data.scalar(), Some(expected));
    assert_eq!(
        data.units().collect::<Vec<_>>(),
        expected.encode_utf16().collect::<Vec<_>>()
    );
    assert!(doc.nodes[id].children.is_empty());
}

fn children(doc: &Document, parent: NodeId, expected: &[NodeId]) {
    assert_eq!(doc.nodes[parent].children, expected);
    for &id in expected {
        assert_eq!(doc.nodes[id].parent, Some(parent));
    }
}

fn only_child(doc: &Document, parent: NodeId) -> NodeId {
    assert_eq!(doc.nodes[parent].children.len(), 1);
    let child = doc.nodes[parent].children[0];
    children(doc, parent, &[child]);
    child
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

const REFERENCE_STYLE: &str = r#"
body{margin:0}.sample{position:absolute;top:0;width:40px;height:40px;background:red}
#probe{left:0}#done{left:50px}.ready .sample{background:green}.clicked .sample{background:blue}
#change{position:absolute;top:60px;left:0}
#pair{position:absolute;top:100px;left:0;width:300px;font-size:24px;line-height:28px}
#result{position:absolute;top:160px;left:0;font-size:20px}#template{display:none}
"#;

// These two literal scalar trees do not derive expected order from the observed DOM.
fn reference_html(phase: usize) -> String {
    let (class, spans, status) = if phase == 0 {
        (
            "ready",
            "<span id=left>Alpha</span><span id=right>Beta</span>",
            "ready",
        )
    } else {
        (
            "clicked",
            "<span id=right>Beta</span><span id=left>Alpha</span>",
            "done",
        )
    };
    format!(
        "<!doctype html><meta charset=utf-8><base href=https://position.example/fixed/>\
         <title>{}</title><style>{REFERENCE_STYLE}</style><body class={class}>\
         <div id=probe class=sample></div><div id=done class=sample></div>\
         <button id=change type=button>Reorder branches</button>\
         <div id=pair>{spans}</div><div id=result>{status}</div>",
        metadata(phase)
    )
}

pub struct State {
    nodes: usize,
    ids: [NodeId; 22],
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
    assert_eq!(previous.is_some(), phase == 1);
    let find = |selector: &str| doc.query_selector(selector).unwrap();
    let html = find("html");
    let body = find("body");
    let pair = find("#pair");
    let left = find("#left");
    let right = find("#right");
    let alpha = only_child(doc, left);
    let beta = only_child(doc, right);
    children(
        doc,
        pair,
        &if phase == 0 {
            [left, right]
        } else {
            [right, left]
        },
    );
    assert_eq!(doc.nodes[pair].parent, Some(body));
    assert_eq!(
        doc.nodes[body]
            .children
            .iter()
            .filter(|&&id| id == pair)
            .count(),
        1
    );
    assert_eq!(doc.nodes[html].parent, Some(doc.root));
    assert_eq!(doc.nodes[body].parent, Some(html));
    assert!(matches!(doc.nodes[doc.root].kind, NodeKind::Document));
    assert_eq!(doc.nodes[doc.root].parent, None);

    let template = find("#template");
    let ordinary = only_child(doc, template);
    let content = doc.template_contents(template).unwrap();
    let inside = only_child(doc, content);
    assert!(
        matches!(doc.nodes[content].kind, NodeKind::DocumentFragment { host: Some(id) } if id == template)
    );
    assert_eq!(doc.nodes[content].parent, None);
    assert_eq!(doc.nodes[template].parent, Some(body));
    let parking_ids = doc.nodes.iter().enumerate().filter_map(|(id, node)| {
        if node.parent.is_some() || !matches!(node.kind, NodeKind::DocumentFragment { host: None }) || node.children.len() != 1 { return None; }
        let child = node.children[0];
        matches!(&doc.nodes[child].kind, NodeKind::Text(data) if data.scalar() == Some("parked")).then_some(id)
    }).collect::<Vec<_>>();
    assert_eq!(parking_ids.len(), 1);
    let parking = parking_ids[0];
    let parked = only_child(doc, parking);
    let heading = find("#heading");
    let title = only_child(doc, heading);
    let result = find("#result");
    let status = only_child(doc, result);
    let base = find("#fixed-base");
    let state = State {
        nodes: doc.nodes.len(),
        ids: [
            doc.root,
            html,
            body,
            heading,
            title,
            base,
            find("#probe"),
            find("#done"),
            find("#change"),
            pair,
            left,
            alpha,
            right,
            beta,
            result,
            status,
            template,
            content,
            inside,
            ordinary,
            parking,
            parked,
        ],
    };
    for (index, id) in state.ids.iter().enumerate() {
        assert!(!state.ids[..index].contains(id), "distinct retained IDs");
    }
    if let Some(initial) = previous {
        assert_eq!(state.ids, initial.ids);
        assert_eq!(state.nodes, initial.nodes, "click creates no DOM nodes");
    }
    for (id, literal) in [
        (alpha, "Alpha"),
        (beta, "Beta"),
        (inside, "inside"),
        (ordinary, "ordinary"),
        (parked, "parked"),
        (title, metadata(phase)),
        (status, ["ready", "done"][phase]),
    ] {
        scalar(doc, id, literal);
    }
    for (id, tag) in [
        (pair, "div"),
        (left, "span"),
        (right, "span"),
        (template, "template"),
        (body, "body"),
        (html, "html"),
    ] {
        let NodeKind::Element(element) = &doc.nodes[id].kind else {
            panic!("expected element");
        };
        assert_eq!(element.tag, tag);
    }
    assert_eq!(doc.title(), metadata(phase));
    assert_eq!(doc.attr(body, "class"), Some(["ready", "clicked"][phase]));
    assert_eq!(
        doc.attr(base, "href"),
        Some("https://position.example/fixed/")
    );
    assert_eq!(doc.base_url().as_str(), "https://position.example/fixed/");

    let reference = Page::from_html(doc.url().clone(), &reference_html(phase), false);
    assert!(
        reference.diagnostics.is_empty(),
        "{:?}",
        reference.diagnostics
    );
    let expected_layout = reference.layout(320.0, 240.0, fonts);
    let actual = paint(layout, fonts, images);
    let expected = paint(&expected_layout, fonts, &reference.images);
    assert_eq!(
        actual.pixels, expected.pixels,
        "literal phase {phase} canvas"
    );
    for x in [20, 70] {
        assert_eq!(actual.pixels[20 * 320 + x], [0x008000, 0x0000ff][phase]);
    }
    assert!(
        (100..140).any(|y| actual.pixels[y * 320..y * 320 + 300]
            .iter()
            .any(|pixel| *pixel != 0xffffff)),
        "visible branch glyphs"
    );
    for literal in ["Alpha", "Beta", ["ready", "done"][phase]] {
        assert_eq!(
            layout
                .commands
                .iter()
                .filter(
                    |command| matches!(command, DrawCommand::Text { text, .. } if text == literal)
                )
                .count(),
            1
        );
    }
    let words = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } if text == "Alpha" || text == "Beta" => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        words,
        if phase == 0 {
            ["Alpha", "Beta"]
        } else {
            ["Beta", "Alpha"]
        }
    );
    state
}
