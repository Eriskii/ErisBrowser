// Independently authored storage/consumer witnesses integrated for foundation validation.
// The original unexecuted draft remains in the preparation directory.
use super::*;

fn exact(units: &[u16]) -> DomString {
    DomString::from_units_owned(units.to_vec()).unwrap()
}

fn data(doc: &Document, id: NodeId) -> &DomString {
    match &doc.nodes[id].kind {
        NodeKind::Text(data)
        | NodeKind::Comment(data)
        | NodeKind::ProcessingInstruction { data, .. } => data,
        _ => panic!("expected CharacterData"),
    }
}

#[test]
fn independent_canonical_transitions_preserve_node_and_retained_accounting() {
    let mut doc = Document::parse("<p></p>");
    let parent = doc.query_selector("p").unwrap();
    let baseline = doc.retained_bytes();
    let id = doc.create_text_node_owned("é".into()).unwrap();
    doc.append_child(parent, id);
    let count = doc.nodes.len();
    assert_eq!(data(&doc, id).scalar(), Some("é"));
    assert_eq!(doc.retained_bytes(), baseline + 2);

    doc.replace_character_data(id, exact(&[0xd800, 0x78]))
        .unwrap();
    assert_eq!(data(&doc, id).raw_units(), Some(&[0xd800, 0x78][..]));
    assert_eq!(data(&doc, id).scalar(), None);
    assert_eq!(doc.retained_bytes(), baseline + 4);

    doc.replace_character_data(id, exact(&[0xd83e, 0xdd80]))
        .unwrap();
    assert_eq!(data(&doc, id).scalar(), Some("🦀"));
    assert_eq!(data(&doc, id).raw_units(), None);
    assert_eq!(doc.retained_bytes(), baseline + 4);
    doc.replace_character_data(id, exact(&[])).unwrap();
    assert_eq!(data(&doc, id).scalar(), Some(""));
    assert_eq!(data(&doc, id).raw_units(), None);
    assert_eq!(doc.retained_bytes(), baseline);
    assert_eq!(doc.nodes.len(), count);
    assert_eq!(doc.nodes[parent].children, [id]);
    assert_eq!(doc.nodes[id].parent, Some(parent));
}

#[test]
fn independent_descendant_aggregation_pairs_across_nodes_and_ignored_data() {
    let mut doc = Document::parse("<div></div>");
    let parent = doc.query_selector("div").unwrap();
    let high = doc.create_text_node_owned(exact(&[0xd83e])).unwrap();
    let ignored = doc
        .create_comment_owned("not part of text content".into())
        .unwrap();
    let nested = doc.create_element("span");
    let empty = doc.create_text_node_owned("".into()).unwrap();
    let low = doc.create_text_node_owned(exact(&[0xdd80])).unwrap();
    let pi = doc
        .create_processing_instruction_owned("probe".into(), "ignored".into())
        .unwrap();
    for id in [high, ignored, nested, pi] {
        doc.append_child(parent, id);
    }
    for id in [empty, low] {
        doc.append_child(nested, id);
    }
    assert_eq!(
        doc.text_content_units_bounded(parent, 2, &mut 128).unwrap(),
        [0xd83e, 0xdd80]
    );
    assert_eq!(
        doc.text_content_scalar_bounded(parent, 4, &mut 128)
            .unwrap(),
        "🦀"
    );
    assert_eq!(data(&doc, high).raw_units(), Some(&[0xd83e][..]));
    assert_eq!(data(&doc, low).raw_units(), Some(&[0xdd80][..]));
    assert!(matches!(
        doc.text_content_units_bounded(parent, 1, &mut 128),
        Err(DomDataError::LimitExceeded)
    ));
    assert!(matches!(
        doc.text_content_scalar_bounded(parent, 3, &mut 128),
        Err(DomDataError::LimitExceeded)
    ));
    assert!(matches!(
        doc.text_content_units_bounded(parent, 2, &mut 0),
        Err(DomDataError::LimitExceeded)
    ));
}

#[test]
fn independent_scalar_projection_rejects_unpaired_aggregate_without_changing_units() {
    let mut doc = Document::parse("<p></p>");
    let parent = doc.query_selector("p").unwrap();
    let low = doc.create_text_node_owned(exact(&[0xdc00])).unwrap();
    let high = doc.create_text_node_owned(exact(&[0xd800])).unwrap();
    doc.append_child(parent, low);
    doc.append_child(parent, high);
    let before = doc.retained_bytes();
    assert_eq!(
        doc.text_content_units_bounded(parent, 2, &mut 128).unwrap(),
        [0xdc00, 0xd800]
    );
    assert!(matches!(
        doc.text_content_scalar_bounded(parent, 16, &mut 128),
        Err(DomDataError::InvalidData)
    ));
    assert_eq!(doc.text_content(parent), "��"); // explicitly display-only legacy projection
    assert_eq!(doc.retained_bytes(), before);
    assert_eq!(data(&doc, low).units().collect::<Vec<_>>(), [0xdc00]);
    assert_eq!(data(&doc, high).units().collect::<Vec<_>>(), [0xd800]);
}

#[test]
fn independent_css_child_aggregation_carries_pair_across_skipped_nodes() {
    let mut doc = Document::parse("<style></style>");
    let style = doc.query_selector("style").unwrap();
    let prefix = doc.create_text_node_owned(".".into()).unwrap();
    let high = doc.create_text_node_owned(exact(&[0xd83e])).unwrap();
    let comment = doc.create_comment_owned("ignored".into()).unwrap();
    let nested = doc.create_element("span");
    let excluded = doc
        .create_text_node_owned("must-not-enter-css".into())
        .unwrap();
    doc.append_child(nested, excluded);
    let low = doc.create_text_node_owned(exact(&[0xdd80])).unwrap();
    let suffix = doc.create_text_node_owned("{color:red}".into()).unwrap();
    for id in [prefix, high, comment, nested, low, suffix] {
        doc.append_child(style, id);
    }
    assert_eq!(
        doc.child_text_content_bounded(style, &mut 128, &mut 128)
            .as_deref(),
        Some(".🦀{color:red}")
    );
    assert_eq!(data(&doc, high).units().collect::<Vec<_>>(), [0xd83e]);
    assert_eq!(data(&doc, low).units().collect::<Vec<_>>(), [0xdd80]);
}

#[test]
fn independent_clone_snapshot_and_detached_payloads_preserve_exact_data() {
    let mut doc = Document::parse("<p></p>");
    let parent = doc.query_selector("p").unwrap();
    let text = doc.create_text_node_owned(exact(&[0x41, 0xd800])).unwrap();
    let detached = doc.create_comment_owned(exact(&[0xdc00, 0])).unwrap();
    let pi = doc
        .create_processing_instruction_owned("probe".into(), "".into())
        .unwrap();
    doc.replace_character_data(pi, exact(&[0xdfff, 0x3f, 0x3e]))
        .unwrap();
    doc.append_child(parent, text);
    doc.append_child(parent, pi);
    let copy = doc.clone();
    let rebuilt = Document::from_snapshot(
        doc.nodes.clone(),
        doc.root,
        doc.scripting_enabled(),
        doc.mode(),
    )
    .unwrap();
    for kept in [&copy, &rebuilt] {
        assert_eq!(kept.retained_bytes(), doc.retained_bytes());
        assert_eq!(data(kept, text).units().collect::<Vec<_>>(), [0x41, 0xd800]);
        assert_eq!(
            data(kept, detached).units().collect::<Vec<_>>(),
            [0xdc00, 0]
        );
        assert_eq!(
            data(kept, pi).units().collect::<Vec<_>>(),
            [0xdfff, 0x3f, 0x3e]
        );
        assert!(
            matches!(&kept.nodes[pi].kind, NodeKind::ProcessingInstruction { target, .. } if target == "probe")
        );
        assert_eq!(kept.nodes[detached].parent, None);
        assert_eq!(kept.nodes[parent].children, [text, pi]);
    }
    doc.replace_character_data(text, "changed".into()).unwrap();
    assert_eq!(
        data(&copy, text).units().collect::<Vec<_>>(),
        [0x41, 0xd800]
    );
    assert_eq!(
        data(&rebuilt, text).units().collect::<Vec<_>>(),
        [0x41, 0xd800]
    );
}

#[test]
fn independent_layout_projection_matches_replacement_glyphs_without_mutating_snapshot() {
    use crate::graphics::{Canvas, DrawCommand, Fonts, ImageStore};
    let mut exact_doc = Document::parse("<p style='margin:0'>placeholder</p>");
    let parent = exact_doc.query_selector("p").unwrap();
    let text = exact_doc.nodes[parent].children[0];
    exact_doc
        .replace_character_data(text, exact(&[0x41, 0xd800, 0x42, 0xdc00]))
        .unwrap();
    let snapshot = exact_doc.clone();
    let mut displayed = exact_doc.clone();
    displayed
        .replace_character_data(text, "A�B�".into())
        .unwrap();
    let fonts = Fonts::new();
    let mut canvases = Vec::new();
    for doc in [&exact_doc, &snapshot, &displayed] {
        let styles = crate::css::compute_styles(doc, &doc.stylesheets(), 200.0, 80.0);
        let result = crate::layout::layout(doc, &styles, 200.0, 80.0, &fonts);
        let output = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>();
        assert_eq!(output, "A�B�");
        let mut canvas = Canvas::new(200, 80).unwrap();
        canvas.paint(&result.commands, &fonts, &ImageStore::new(), 0.0, 0.0);
        assert!(!canvas.exhausted());
        canvases.push(canvas.pixels);
    }
    assert_eq!(canvases[0], canvases[1]);
    assert_eq!(canvases[0], canvases[2]);
    assert_eq!(
        data(&exact_doc, text).units().collect::<Vec<_>>(),
        [0x41, 0xd800, 0x42, 0xdc00]
    );
    assert_eq!(
        data(&snapshot, text).units().collect::<Vec<_>>(),
        [0x41, 0xd800, 0x42, 0xdc00]
    );
    assert_eq!(data(&displayed, text).scalar(), Some("A�B�"));
}

#[test]
fn owned_units_admission_uses_payload_bytes_and_preserves_refused_state() {
    let mut doc = Document::parse("");
    let before = doc.retained_bytes();
    let exact_limit = DomString::from_units_owned(vec![0xd800; MAX_TEXT / 2]).unwrap();
    let pointer = exact_limit.raw_units().unwrap().as_ptr();
    let id = doc.create_text_node_owned(exact_limit).unwrap();
    assert_eq!(data(&doc, id).raw_units().unwrap().as_ptr(), pointer);
    assert_eq!(doc.retained_bytes(), before + MAX_TEXT);
    let oversized = DomString::from_units_owned(vec![0xdc00; MAX_TEXT / 2 + 1]).unwrap();
    let nodes = doc.nodes.len();
    let retained = doc.retained_bytes();
    assert_eq!(
        doc.create_comment_owned(oversized.clone()),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(
        doc.replace_character_data(id, oversized),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!((doc.nodes.len(), doc.retained_bytes()), (nodes, retained));
    assert_eq!(data(&doc, id).raw_units().unwrap().as_ptr(), pointer);
    let empty = doc.create_comment_owned(DomString::default()).unwrap();
    assert_eq!(doc.retained_bytes(), retained);
    assert_eq!(data(&doc, empty).scalar(), Some(""));
    // PI's existing creation cap includes the scalar target, not just its data.
    let data_limit = DomString::from_units_owned(vec![0xd800; MAX_TEXT / 2]).unwrap();
    assert_eq!(
        doc.create_processing_instruction_owned("x".into(), data_limit),
        Err(DomDataError::LimitExceeded)
    );
    let rebuilt = Document::from_snapshot(doc.nodes.clone(), doc.root, false, doc.mode()).unwrap();
    assert_eq!(rebuilt.retained_bytes(), doc.retained_bytes());
}

#[test]
fn presentation_serialization_pairs_adjacent_data_but_markup_ends_the_pair() {
    let mut doc = Document::parse("<div></div>");
    let parent = doc.query_selector("div").unwrap();
    let high = doc.create_text_node_owned(exact(&[0xd834])).unwrap();
    let low = doc.create_text_node_owned(exact(&[0xdd1e])).unwrap();
    doc.append_child(parent, high);
    doc.append_child(parent, low);
    assert_eq!(doc.outer_html(parent), "<div>𝄞</div>");
    let comment = doc.create_comment_owned("x".into()).unwrap();
    doc.append_child(parent, comment);
    doc.append_child(parent, low);
    assert_eq!(doc.outer_html(parent), "<div>�<!--x-->�</div>");
    assert_eq!(doc.text_content(parent), "𝄞");
    assert_eq!(data(&doc, high).raw_units().unwrap(), [0xd834]);
    assert_eq!(data(&doc, low).raw_units().unwrap(), [0xdd1e]);
    let pi = doc
        .create_processing_instruction_owned("probe".into(), exact(&[0xd800, 63]))
        .unwrap();
    assert_eq!(doc.outer_html(pi), "<?probe �??>");
    assert_eq!(data(&doc, pi).raw_units().unwrap(), [0xd800, 63]);
}

#[test]
fn presentation_serialization_stops_at_first_unfitting_scalar() {
    let mut writer = SerializedText {
        output: "x".repeat(MAX_TEXT - 1),
        ..SerializedText::default()
    };
    writer.push_units("é".encode_utf16());
    push_serialized(&mut writer, "z</p>");
    assert_eq!(writer.finish(), "x".repeat(MAX_TEXT - 1));
}

#[test]
fn aggregate_projection_limits_and_invalid_traversal_remain_explicit() {
    let mut doc = Document::parse("<p></p>");
    let parent = doc.query_selector("p").unwrap();
    let child = doc.create_text_node_owned(exact(&[0xd800])).unwrap();
    doc.append_child(parent, child);
    assert_eq!(
        doc.text_content_projection_bounded(parent, 3, &mut 3)
            .unwrap(),
        "�"
    );
    assert_eq!(
        doc.text_content_projection_bounded(parent, 2, &mut 3),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(
        doc.text_content_scalar_bounded(parent, 3, &mut 3),
        Err(DomDataError::InvalidData)
    );
    assert_eq!(
        doc.text_content_units_bounded(parent, 1, &mut 2),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(
        doc.text_content_units_bounded(usize::MAX, 1, &mut 3),
        Err(DomDataError::InvalidNode)
    );
    // Public arena mutation cannot make a cyclic/duplicate walk allocate without
    // bound or turn a malformed snapshot into successful aggregate text.
    doc.nodes[parent].children.push(child);
    assert_eq!(
        doc.text_content_units_bounded(parent, 4, &mut 10),
        Err(DomDataError::InvalidData)
    );
    doc.nodes[parent].children.truncate(1);
    doc.nodes[child].children.push(child);
    assert_eq!(
        doc.text_content_scalar_bounded(parent, 4, &mut 3),
        Err(DomDataError::InvalidData)
    );
    assert_eq!(
        doc.text_content_units_bounded(child, 4, &mut 1),
        Err(DomDataError::InvalidData)
    );
}

#[test]
fn dom_string_reports_actual_node_header_delta() {
    // Exact previous scalar field shapes, retained only to measure the typed
    // arena reservation delta on the compiler used for the candidate.
    #[allow(dead_code)]
    enum ScalarKind {
        Document,
        DocumentFragment { host: Option<NodeId> },
        Element(Element),
        Text(String),
        Comment(String),
        Doctype(Doctype),
        ProcessingInstruction { target: String, data: String },
    }
    #[allow(dead_code)]
    struct ScalarNode {
        parent: Option<NodeId>,
        children: Vec<NodeId>,
        kind: ScalarKind,
    }
    println!(
        "DOM_STRING_HEADERS old_string={} new_dom_string={} old_kind={} new_kind={} old_node={} new_node={}",
        std::mem::size_of::<String>(),
        std::mem::size_of::<DomString>(),
        std::mem::size_of::<ScalarKind>(),
        std::mem::size_of::<NodeKind>(),
        std::mem::size_of::<ScalarNode>(),
        std::mem::size_of::<Node>()
    );
    assert!(std::mem::size_of::<Node>() >= std::mem::size_of::<DomString>());
}
