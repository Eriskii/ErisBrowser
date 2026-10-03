use eris::{
    dom::{Document, NodeId, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore},
    layout::LayoutResult,
    page::Page,
};

pub const HTML: &str = include_str!("../fixtures/node-predicates.html");

pub fn metadata(phase: usize) -> &'static str {
    ["Predicates ready", "Predicates done"][phase]
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

fn only_child(doc: &Document, parent: NodeId) -> NodeId {
    assert_eq!(doc.nodes[parent].children.len(), 1);
    let id = doc.nodes[parent].children[0];
    assert_eq!(doc.nodes[id].parent, Some(parent));
    id
}

fn paint(layout: &LayoutResult, fonts: &Fonts, images: &ImageStore) -> Canvas {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas
}

// Independent literal markup: the reference has no script and does not clone
// or project the observed Document. Hidden/detached bookkeeping is checked below.
const REFERENCE_STYLE: &str = r#"
body{margin:0}.sample{position:absolute;top:0;width:40px;height:40px;background:red}
#probe{left:0}#done{left:50px}.ready .sample{background:green}.clicked .sample{background:blue}
#change{position:absolute;top:60px;left:0}
#left,#right{position:absolute;top:100px;width:150px;font-size:24px;line-height:28px}
#left{left:0}#right{left:160px}#result{position:absolute;top:160px;left:0;font-size:20px}
#old,#template{display:none}
"#;

fn reference_html(phase: usize) -> String {
    let (class, left, right, status) = if phase == 0 {
        ("ready", "<span id=branch>Same node</span>", "", "ready")
    } else {
        ("clicked", "", "<span id=branch>Same node</span>", "done")
    };
    format!(
        "<!doctype html><meta charset=utf-8><title>{}</title><style>{REFERENCE_STYLE}</style>\
         <body class={class}><div id=probe class=sample></div><div id=done class=sample></div>\
         <button id=change type=button>Move subtree</button>\
         <div id=left>{left}</div><div id=right>{right}</div><div id=result>{status}</div>",
        metadata(phase)
    )
}

pub struct State {
    nodes: usize,
    ids: [NodeId; 18],
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
    let left = find("#left");
    let right = find("#right");
    let branch = find("#branch");
    let branch_text = only_child(doc, branch);
    let old = previous.map_or_else(|| find("#old"), |state| state.ids[10]);
    let old_leaf = only_child(doc, old);
    let old_text = only_child(doc, old_leaf);
    let template = find("#template");
    let content = doc.template_contents(template).unwrap();
    assert!(matches!(
        doc.nodes[content].kind,
        NodeKind::DocumentFragment { host: Some(host) } if host == template
    ));
    assert_eq!(doc.nodes[content].parent, None);
    let inert = only_child(doc, content);
    let inert_text = only_child(doc, inert);
    let ordinary = only_child(doc, template);
    let heading = find("#heading");
    let title_text = only_child(doc, heading);
    let result = find("#result");
    let result_text = only_child(doc, result);
    let button = find("#change");
    let state = State {
        nodes: doc.nodes.len(),
        ids: [
            doc.root,
            button,
            heading,
            title_text,
            result,
            result_text,
            left,
            right,
            branch,
            branch_text,
            old,
            old_leaf,
            old_text,
            template,
            content,
            inert,
            inert_text,
            ordinary,
        ],
    };
    for (index, id) in state.ids.iter().enumerate() {
        assert!(!state.ids[..index].contains(id), "distinct retained IDs");
    }
    if let Some(initial) = previous {
        assert_eq!(state.ids, initial.ids);
        assert_eq!(state.nodes, initial.nodes, "click creates no DOM nodes");
    }
    if phase == 0 {
        assert_eq!(doc.nodes[left].children, [branch, old]);
        assert!(doc.nodes[right].children.is_empty());
    } else {
        assert!(doc.nodes[left].children.is_empty());
        assert_eq!(doc.nodes[right].children, [branch]);
        assert!(doc.query_selector("#old").is_none());
        assert!(doc.query_selector("#old-leaf").is_none());
    }
    assert_eq!(doc.nodes[branch].parent, Some([left, right][phase]));
    assert_eq!(doc.nodes[old].parent, (phase == 0).then_some(left));
    for (id, literal) in [
        (branch_text, "Same node"),
        (old_text, "retained"),
        (inert_text, "content"),
        (ordinary, "ordinary"),
        (title_text, metadata(phase)),
        (result_text, ["ready", "done"][phase]),
    ] {
        scalar(doc, id, literal);
    }
    assert_eq!(doc.attr(old, "id"), Some("old"));
    assert_eq!(doc.attr(old_leaf, "id"), Some("old-leaf"));
    assert_eq!(doc.attr(inert, "id"), Some("inert"));
    assert!(doc.query_selector("#inert").is_none());
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
    for (side, x) in [0, 160].into_iter().enumerate() {
        let ink = (100..140).any(|y| {
            actual.pixels[y * 320 + x..y * 320 + x + 150]
                .iter()
                .any(|pixel| *pixel != 0xffffff)
        });
        assert_eq!(ink, side == phase, "only the expected column paints Text");
    }
    // Inline layout emits words separately; each literal word must paint once.
    for literal in ["Same", "node"] {
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
    assert!(layout.commands.iter().any(|command| {
        matches!(command, DrawCommand::Text { text, .. } if text == ["ready", "done"][phase])
    }));
    state
}
