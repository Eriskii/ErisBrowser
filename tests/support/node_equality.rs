use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/node-equality.html");

pub fn metadata(phase: usize) -> &'static str {
    ["Equality ready", "Equality done"][phase]
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

fn comment(doc: &Document, id: NodeId) {
    let NodeKind::Comment(data) = &doc.nodes[id].kind else {
        panic!("expected Comment at {id}");
    };
    assert_eq!(data.scalar(), None);
    assert_eq!(data.units().collect::<Vec<_>>(), [114, 97, 119, 55296]);
    assert_eq!(data.stored_bytes(), 8);
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
    let id = doc.nodes[parent].children[0];
    children(doc, parent, &[id]);
    id
}

fn detached_element(doc: &Document, id_value: &str) -> NodeId {
    let ids = doc
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(id, node)| {
            (node.parent.is_none()
                && matches!(node.kind, NodeKind::Element(_))
                && doc.attr(id, "id") == Some(id_value))
            .then_some(id)
        })
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 1);
    ids[0]
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

// Independent script-free literal reference; neither clone nor observed projection.
const REFERENCE_STYLE: &str = r#"
body{margin:0}.sample{position:absolute;top:0;width:40px;height:40px;background:red}
#probe{left:0}#done{left:50px}.ready .sample{background:green}.clicked .sample{background:blue}
#change{position:absolute;top:60px;left:0}
#live{position:absolute;top:100px;left:0;width:300px;font-size:24px;line-height:28px}
#result{position:absolute;top:160px;left:0;font-size:20px}#template{display:none}
"#;

fn reference_html(phase: usize) -> String {
    let (class, tail, status) = if phase == 0 {
        ("ready", " beta", "ready")
    } else {
        ("clicked", " gamma", "done")
    };
    format!(
        "<!doctype html><meta charset=utf-8><base href=https://equality.example/fixed/>\
         <title>{}</title><style>{REFERENCE_STYLE}</style><body class={class}>\
         <div id=probe class=sample></div><div id=done class=sample></div>\
         <button id=change type=button>Compare trees</button>\
         <div id=live data-note=base><span class=label>Alpha</span><span class=label>{tail}</span></div>\
         <div id=result>{status}</div>",
        metadata(phase)
    )
}

pub struct State {
    nodes: usize,
    ids: [NodeId; 29],
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
    let live = find("#live");
    let peer = detached_element(doc, "live");
    assert_eq!(doc.nodes[live].children.len(), 3);
    assert_eq!(doc.nodes[peer].children.len(), 3);
    let [first, second, live_comment]: [NodeId; 3] =
        doc.nodes[live].children[..].try_into().unwrap();
    let [peer_first, peer_second, peer_comment]: [NodeId; 3] =
        doc.nodes[peer].children[..].try_into().unwrap();
    children(doc, live, &[first, second, live_comment]);
    children(doc, peer, &[peer_first, peer_second, peer_comment]);
    let first_text = only_child(doc, first);
    let second_text = only_child(doc, second);
    let peer_first_text = only_child(doc, peer_first);
    let peer_second_text = only_child(doc, peer_second);
    assert_eq!(doc.nodes[live].parent, Some(find("body")));
    assert_eq!(doc.nodes[peer].parent, None);
    for parent in [live, peer] {
        let NodeKind::Element(element) = &doc.nodes[parent].kind else {
            panic!("branch element");
        };
        assert_eq!(element.tag, "div");
        assert_eq!(element.attrs.len(), 2);
        assert_eq!(doc.attr(parent, "id"), Some("live"));
        assert_eq!(doc.attr(parent, "data-note"), Some("base"));
    }
    for span in [first, second, peer_first, peer_second] {
        let NodeKind::Element(element) = &doc.nodes[span].kind else {
            panic!("span element");
        };
        assert_eq!(element.tag, "span");
        assert_eq!(element.attrs.len(), 1);
        assert_eq!(doc.attr(span, "class"), Some("label"));
    }
    let template = find("#template");
    let peer_template = detached_element(doc, "template");
    let content = doc.template_contents(template).unwrap();
    let peer_content = doc.template_contents(peer_template).unwrap();
    let content_text = only_child(doc, content);
    let peer_content_text = only_child(doc, peer_content);
    let ordinary = only_child(doc, template);
    let peer_ordinary = only_child(doc, peer_template);
    for (fragment, host) in [(content, template), (peer_content, peer_template)] {
        assert!(
            matches!(doc.nodes[fragment].kind, NodeKind::DocumentFragment { host: Some(id) } if id == host)
        );
        assert_eq!(doc.nodes[fragment].parent, None);
    }
    assert_eq!(doc.nodes[peer_template].parent, None);
    let loose_ids = doc.nodes.iter().enumerate().filter_map(|(id, node)| {
        if !matches!(node.kind, NodeKind::DocumentFragment { host: None }) || node.children.len() != 1 { return None; }
        let child = node.children[0];
        matches!(&doc.nodes[child].kind, NodeKind::Text(data) if data.scalar() == Some("inside")).then_some(id)
    }).collect::<Vec<_>>();
    assert_eq!(loose_ids.len(), 1);
    let loose = loose_ids[0];
    let loose_text = only_child(doc, loose);
    assert_eq!(doc.nodes[loose].parent, None);
    let heading = find("#heading");
    let title_text = only_child(doc, heading);
    let result = find("#result");
    let result_text = only_child(doc, result);
    let fixed_base = find("#fixed-base");
    let state = State {
        nodes: doc.nodes.len(),
        ids: [
            doc.root,
            find("#change"),
            heading,
            title_text,
            result,
            result_text,
            fixed_base,
            live,
            first,
            first_text,
            second,
            second_text,
            live_comment,
            peer,
            peer_first,
            peer_first_text,
            peer_second,
            peer_second_text,
            peer_comment,
            template,
            content,
            content_text,
            ordinary,
            peer_template,
            peer_content,
            peer_content_text,
            peer_ordinary,
            loose,
            loose_text,
        ],
    };
    for (index, id) in state.ids.iter().enumerate() {
        assert!(!state.ids[..index].contains(id), "independent retained IDs");
    }
    if let Some(initial) = previous {
        assert_eq!(state.ids, initial.ids);
        assert_eq!(state.nodes, initial.nodes, "click creates no DOM nodes");
    }
    for (id, literal) in [
        (first_text, "Alpha"),
        (peer_first_text, "Alpha"),
        (second_text, [" beta", " gamma"][phase]),
        (peer_second_text, [" beta", " gamma"][phase]),
        (content_text, "inside"),
        (peer_content_text, "different"),
        (ordinary, "ordinary"),
        (peer_ordinary, "ordinary"),
        (loose_text, "inside"),
        (title_text, metadata(phase)),
        (result_text, ["ready", "done"][phase]),
    ] {
        scalar(doc, id, literal);
    }
    comment(doc, live_comment);
    comment(doc, peer_comment);
    assert_eq!(doc.base_url().as_str(), "https://equality.example/fixed/");
    assert_eq!(
        doc.attr(fixed_base, "href"),
        Some("https://equality.example/fixed/")
    );
    assert_eq!(doc.title(), metadata(phase));
    assert_eq!(
        doc.attr(find("body"), "class"),
        Some(["ready", "clicked"][phase])
    );

    let reference = Page::from_html(doc.url().clone(), &reference_html(phase), false);
    assert!(
        reference.diagnostics.is_empty(),
        "{:?}",
        reference.diagnostics
    );
    let expected_layout = reference.layout(320.0, 240.0, fonts);
    let actual = paint(layout, fonts, images);
    let expected = paint(&expected_layout, fonts, &reference.images);
    assert_eq!(actual.pixels, expected.pixels, "literal phase {phase} page");
    for x in [20, 70] {
        assert_eq!(actual.pixels[20 * 320 + x], [0x008000, 0x0000ff][phase]);
    }
    assert!(
        (100..140).any(|y| actual.pixels[y * 320..y * 320 + 300]
            .iter()
            .any(|pixel| *pixel != 0xffffff)),
        "visible branch glyphs"
    );
    for literal in ["Alpha", ["beta", "gamma"][phase], ["ready", "done"][phase]] {
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
    let absent = ["gamma", "beta"][phase];
    assert!(
        !layout
            .commands
            .iter()
            .any(|command| matches!(command, DrawCommand::Text { text, .. } if text == absent))
    );
    state
}
