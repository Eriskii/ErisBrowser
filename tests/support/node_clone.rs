use eris::{
    dom::{Document, Namespace, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/node-clone.html");

pub fn metadata(phase: usize) -> &'static str {
    ["Clone ready", "Clone done"][phase]
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

fn element(doc: &Document, id: NodeId, tag: &str, attrs: &[(&str, &str)]) {
    let NodeKind::Element(value) = &doc.nodes[id].kind else {
        panic!("expected Element at {id}");
    };
    assert_eq!(value.namespace, Namespace::Html);
    assert_eq!(value.tag, tag);
    assert_eq!(value.attrs.len(), attrs.len());
    for &(name, expected) in attrs {
        assert_eq!(value.attrs.get(name).map(String::as_str), Some(expected));
    }
    assert!(value.attr_namespaces.is_empty());
    if tag != "template" {
        assert!(value.template_contents.is_none());
    }
}

fn text(doc: &Document, id: NodeId, scalar: &str, tail: Option<u16>) {
    let NodeKind::Text(data) = &doc.nodes[id].kind else {
        panic!("expected Text at {id}");
    };
    let expected = scalar.encode_utf16().chain(tail).collect::<Vec<_>>();
    assert_eq!(data.units().collect::<Vec<_>>(), expected);
    assert_eq!(data.scalar(), tail.is_none().then_some(scalar));
    assert!(doc.nodes[id].children.is_empty());
}

fn graph(doc: &Document, root: NodeId, copied: bool) -> [NodeId; 10] {
    assert_eq!(doc.nodes[root].children.len(), 4);
    let [label, comment, template, pi] = doc.nodes[root].children[..] else {
        panic!("four literal root children");
    };
    let visible = only_child(doc, label);
    let content = doc.template_contents(template).unwrap();
    let bold = only_child(doc, content);
    let inert = only_child(doc, bold);
    let ordinary = only_child(doc, template);
    children(doc, root, &[label, comment, template, pi]);
    element(
        doc,
        root,
        "div",
        &[
            ("id", if copied { "copy" } else { "card" }),
            ("class", "card"),
        ],
    );
    element(doc, label, "span", &[("id", "label")]);
    element(doc, template, "template", &[("id", "card-template")]);
    element(doc, bold, "b", &[]);
    text(doc, visible, if copied { "Copy" } else { "Alpha" }, None);
    text(doc, inert, "inert", Some(0xdc00));
    text(doc, ordinary, "ordinary", Some(0xd800));
    assert!(matches!(doc.nodes[content].kind,
        NodeKind::DocumentFragment { host: Some(host) } if host == template));
    assert_eq!(doc.nodes[content].parent, None);
    let NodeKind::Comment(data) = &doc.nodes[comment].kind else {
        panic!("expected literal Comment");
    };
    assert_eq!(
        data.units().collect::<Vec<_>>(),
        [0x6d, 0x61, 0x72, 0x6b, 0xd800]
    );
    assert_eq!(data.scalar(), None);
    assert!(doc.nodes[comment].children.is_empty());
    let NodeKind::ProcessingInstruction { target, data } = &doc.nodes[pi].kind else {
        panic!("expected literal PI");
    };
    assert_eq!(target, "probe");
    assert_eq!(
        data.units().collect::<Vec<_>>(),
        [0x73, 0x65, 0x65, 0x64, 0x3f, 0x3e, 0xd800]
    );
    assert_eq!(data.scalar(), None);
    assert!(doc.nodes[pi].children.is_empty());
    let ids = [
        root, label, visible, comment, template, content, bold, inert, ordinary, pi,
    ];
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[..index].contains(id), "ten distinct graph identities");
    }
    ids
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

const REFERENCE_STYLE: &str = r#"
body{margin:0}.sample{position:absolute;top:0;left:0;width:40px;height:40px;background:red}
.ready .sample{background:green}.clicked .sample{background:blue}
#change{position:absolute;top:60px;left:0}
.card{position:absolute;top:100px;left:0;width:140px;height:36px;background:green;font-size:24px;line-height:28px}
#copy{left:150px;background:blue}#result{position:absolute;top:160px;left:0;font-size:20px}template{display:none}
"#;

// Literal visible trees, independent of the copied DOM and its serialization.
fn reference_html(phase: usize) -> String {
    let (class, status, copy) = if phase == 0 {
        ("ready", "ready", "")
    } else {
        (
            "clicked",
            "done",
            "<div id=copy class=card><span id=label>Copy</span></div>",
        )
    };
    format!(
        "<!doctype html><meta charset=utf-8><base href=https://clone.example/fixed/>\
         <title>{}</title><style>{REFERENCE_STYLE}</style><body class={class}>\
         <div id=probe class=sample></div><button id=change type=button>Duplicate card</button>\
         <div id=card class=card><span id=label>Alpha</span></div>\
         <div id=result>{status}</div>{copy}",
        metadata(phase)
    )
}

pub struct State {
    nodes: usize,
    stable: Vec<NodeId>,
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
    let source = find("#card");
    let originals = graph(doc, source, false);
    let heading = find("#heading");
    let title = only_child(doc, heading);
    let result = find("#result");
    let status = only_child(doc, result);
    let base = find("#fixed-base");
    let mut stable = vec![
        doc.root,
        html,
        body,
        base,
        heading,
        title,
        find("#probe"),
        find("#change"),
        result,
        status,
    ];
    stable.extend(originals);
    for (index, id) in stable.iter().enumerate() {
        assert!(
            !stable[..index].contains(id),
            "twenty distinct preserved identities"
        );
    }
    assert_eq!(stable.len(), 20);
    assert!(matches!(doc.nodes[doc.root].kind, NodeKind::Document));
    assert_eq!(doc.nodes[doc.root].parent, None);
    assert_eq!(doc.nodes[html].parent, Some(doc.root));
    assert_eq!(doc.nodes[body].parent, Some(html));
    assert_eq!(doc.nodes[source].parent, Some(body));
    assert_eq!(
        doc.nodes[body]
            .children
            .iter()
            .filter(|&&id| id == source)
            .count(),
        1
    );
    if let Some(initial) = previous {
        assert_eq!(stable, initial.stable);
        assert_eq!(
            doc.nodes.len(),
            initial.nodes + 10,
            "exactly ten fresh clone nodes"
        );
        let copy = find("#copy");
        let copies = graph(doc, copy, true);
        let mut added = copies.to_vec();
        added.sort_unstable();
        assert_eq!(
            added,
            (initial.nodes..initial.nodes + 10).collect::<Vec<_>>()
        );
        assert!(copies.iter().all(|id| !originals.contains(id)));
        assert_eq!(doc.nodes[copy].parent, Some(body));
        assert_eq!(
            doc.nodes[body]
                .children
                .iter()
                .filter(|&&id| id == copy)
                .count(),
            1
        );
    } else {
        assert!(doc.query_selector("#copy").is_none());
    }
    text(doc, title, metadata(phase), None);
    text(doc, status, ["ready", "done"][phase], None);
    assert_eq!(doc.title(), metadata(phase));
    assert_eq!(doc.attr(body, "class"), Some(["ready", "clicked"][phase]));
    assert_eq!(doc.attr(base, "href"), Some("https://clone.example/fixed/"));
    assert_eq!(doc.base_url().as_str(), "https://clone.example/fixed/");

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
        "literal clone phase {phase} canvas"
    );
    assert_eq!(actual.pixels[20 * 320 + 20], [0x008000, 0x0000ff][phase]);
    assert_eq!(actual.pixels[130 * 320 + 130], 0x008000);
    assert_eq!(actual.pixels[130 * 320 + 280], [0xffffff, 0x0000ff][phase]);
    for (left, word) in if phase == 0 {
        vec![(0, "Alpha")]
    } else {
        vec![(0, "Alpha"), (150, "Copy")]
    } {
        assert!(
            (100..128).any(|y| actual.pixels[y * 320 + left..y * 320 + left + 100].contains(&0)),
            "nonempty {word} glyphs"
        );
    }
    let words = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } if text == "Alpha" || text == "Copy" => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        words,
        if phase == 0 {
            vec!["Alpha"]
        } else {
            vec!["Alpha", "Copy"]
        }
    );
    for literal in [metadata(phase), "inert", "ordinary", "seed"] {
        assert!(
            !layout.commands.iter().any(
                |command| matches!(command, DrawCommand::Text { text, .. } if text == literal)
            ),
            "hidden text {literal}"
        );
    }
    State {
        nodes: doc.nodes.len(),
        stable,
    }
}
