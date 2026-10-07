use super::*;

const WORK: usize = 100_000;

fn budget() -> DomMutationBudget {
    DomMutationBudget {
        steps: WORK,
        allocated: 0,
        heap_limit: 8 * 1024 * 1024,
    }
}

fn text(doc: &mut Document, units: &[u16]) -> NodeId {
    doc.create_text_node_owned(DomString::from_units_owned(units.to_vec()).unwrap())
        .unwrap()
}

fn units(doc: &Document, id: NodeId) -> Vec<u16> {
    match &doc.nodes[id].kind {
        NodeKind::Text(data)
        | NodeKind::Comment(data)
        | NodeKind::ProcessingInstruction { data, .. } => data.units().collect(),
        _ => panic!("expected exact character data"),
    }
}

fn link(doc: &mut Document, parent: NodeId, child: NodeId) {
    doc.nodes[parent].children.push(child);
    doc.nodes[child].parent = Some(parent);
}

fn admit(doc: Document) -> Document {
    Document::from_snapshot(doc.nodes, doc.root, false, doc.mode).unwrap()
}

fn arena(doc: &mut Document, spare: usize) {
    doc.nodes = std::mem::take(&mut doc.nodes).into_boxed_slice().into_vec();
    assert_eq!(doc.nodes.capacity(), doc.nodes.len());
    if spare != 0 {
        doc.nodes.reserve_exact(spare);
    }
    assert!(doc.nodes.capacity() - doc.nodes.len() >= spare);
}

#[derive(Debug, PartialEq, Eq)]
struct State {
    logical: String,
    capacity: usize,
    children: Vec<usize>,
    payload_addresses: Vec<usize>,
}

fn state(doc: &Document) -> State {
    let mut pointers = Vec::new();
    for node in &doc.nodes {
        let data = match &node.kind {
            NodeKind::Text(data)
            | NodeKind::Comment(data)
            | NodeKind::ProcessingInstruction { data, .. } => Some(data),
            _ => None,
        };
        if let Some(data) = data {
            pointers.push(data.scalar().map_or_else(
                || data.raw_units().unwrap().as_ptr() as usize,
                |s| s.as_ptr() as usize,
            ));
        }
    }
    State {
        logical: format!("{doc:?}"),
        capacity: doc.nodes.capacity(),
        children: doc.nodes.iter().map(|n| n.children.capacity()).collect(),
        payload_addresses: pointers,
    }
}

#[test]
fn exact_leaves_are_fresh_and_keep_units_target_and_doctype_fields() {
    for (comment, expected) in [
        (false, vec![]),
        (false, vec![65, 55357, 56960, 66]),
        (false, vec![55296, 56320, 55296]),
        (true, vec![56320, 13, 10, 0]),
    ] {
        let mut doc = Document::parse("");
        let data = DomString::from_units_owned(expected.clone()).unwrap();
        let source = if comment {
            doc.create_comment_owned(data).unwrap()
        } else {
            doc.create_text_node_owned(data).unwrap()
        };
        let old = doc.nodes.len();
        let retained = doc.retained_bytes;
        let payload = match &doc.nodes[source].kind {
            NodeKind::Text(v) | NodeKind::Comment(v) => v.stored_bytes(),
            _ => unreachable!(),
        };
        let a = doc
            .clone_node_checked(source, false, &mut budget())
            .unwrap();
        let b = doc.clone_node_checked(source, true, &mut budget()).unwrap();
        assert_eq!((a, b, doc.nodes.len()), (old, old + 1, old + 2));
        assert_eq!(doc.retained_bytes, retained + 2 * payload);
        for id in [source, a, b] {
            assert_eq!(units(&doc, id), expected);
        }
        assert!(doc.nodes[a].parent.is_none() && doc.nodes[b].parent.is_none());
        assert!(doc.nodes[a].children.is_empty() && doc.nodes[b].children.is_empty());
        doc.replace_character_data(a, "changed".into()).unwrap();
        assert_eq!(units(&doc, source), expected);
        assert_eq!(units(&doc, b), expected);
    }
    let mut doc = Document::parse("");
    let pi = doc
        .create_processing_instruction_owned("Build".into(), "start".into())
        .unwrap();
    doc.replace_character_data(
        pi,
        DomString::from_nonscalar_units(vec![63, 62, 55296, 0]).unwrap(),
    )
    .unwrap();
    let copy = doc.clone_node_checked(pi, true, &mut budget()).unwrap();
    assert_eq!(units(&doc, copy), [63, 62, 55296, 0]);
    assert!(
        matches!(&doc.nodes[copy].kind, NodeKind::ProcessingInstruction { target, .. } if target == "Build")
    );
    for public in [None, Some(String::new()), Some("public".into())] {
        let id = doc.create_doctype(Doctype {
            name: "html".into(),
            public_id: public.clone(),
            system_id: Some("system".into()),
            force_quirks: true,
        });
        let copied = doc.clone_node_checked(id, false, &mut budget()).unwrap();
        let NodeKind::Doctype(d) = &doc.nodes[copied].kind else {
            panic!()
        };
        assert_eq!(
            (&d.name, &d.public_id, &d.system_id, d.force_quirks),
            (
                &"html".to_owned(),
                &public,
                &Some("system".to_owned()),
                true
            )
        );
        assert!(doc.nodes[copied].parent.is_none());
    }
}

#[test]
fn host_stored_long_fields_and_namespace_annotations_are_not_reparsed() {
    let mut doc = Document::parse("");
    let source = doc.create_element("div");
    let tag = format!("MiXeD:{}!", "x".repeat(300));
    let value = "value".repeat(900);
    let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
        panic!()
    };
    el.tag = tag.clone();
    el.attrs.insert("UPPER name".into(), value.clone());
    el.attrs.insert("empty".into(), String::new());
    let foreign = doc.create_element_ns(Namespace::Svg, "linearGradient");
    for name in [
        "xlink:actuate",
        "xlink:arcrole",
        "xlink:href",
        "xlink:role",
        "xlink:show",
        "xlink:title",
        "xlink:type",
        "xml:lang",
        "xml:space",
        "xmlns",
        "xmlns:xlink",
    ] {
        doc.set_attr_ns(
            foreign,
            AttributeNamespace::from_qualified_name(name).unwrap(),
            name,
            "kept",
        );
    }
    let mut doc = admit(doc);
    let before = format!("{:?}", doc.nodes);
    let a = doc
        .clone_node_checked(source, false, &mut budget())
        .unwrap();
    let NodeKind::Element(el) = &doc.nodes[a].kind else {
        panic!()
    };
    assert_eq!(el.tag, tag);
    assert_eq!(el.attrs.get("UPPER name"), Some(&value));
    assert_eq!(el.attrs.get("empty").map(String::as_str), Some(""));
    let b = doc
        .clone_node_checked(foreign, false, &mut budget())
        .unwrap();
    let (NodeKind::Element(original), NodeKind::Element(copied)) =
        (&doc.nodes[foreign].kind, &doc.nodes[b].kind)
    else {
        panic!()
    };
    assert_eq!(copied.namespace, Namespace::Svg);
    assert_eq!(copied.tag, "linearGradient");
    assert_eq!(copied.attrs, original.attrs);
    assert_eq!(copied.attr_namespaces, original.attr_namespaces);
    assert_eq!(copied.attr_namespaces.len(), 11);
    assert_eq!(format!("{:?}", &doc.nodes[..a]), before);
    let id = doc.create_doctype(Doctype {
        name: "html".into(),
        public_id: None,
        system_id: None,
        force_quirks: false,
    });
    let NodeKind::Doctype(d) = &mut doc.nodes[id].kind else {
        panic!()
    };
    d.name = "n".repeat(300);
    d.public_id = Some("p".repeat(4097));
    d.system_id = Some("s".repeat(4099));
    let mut doc = admit(doc);
    let copy = doc.clone_node_checked(id, false, &mut budget()).unwrap();
    let NodeKind::Doctype(d) = &doc.nodes[copy].kind else {
        panic!()
    };
    assert_eq!(d.name, "n".repeat(300));
    assert_eq!(d.public_id, Some("p".repeat(4097)));
    assert_eq!(d.system_id, Some("s".repeat(4099)));
}

#[test]
fn deep_order_and_segmentation_use_fresh_links_and_keep_source() {
    let mut doc = Document::parse("");
    let root = doc.create_document_fragment();
    let a = text(&mut doc, &[65]);
    let empty = text(&mut doc, &[]);
    let b = doc.create_element("b");
    let nested = text(&mut doc, &[67]);
    let comment = doc.create_comment("mark");
    link(&mut doc, b, nested);
    for id in [empty, b, comment, a] {
        link(&mut doc, root, id);
    }
    let old = doc.nodes.len();
    let prefix = format!("{:?}", doc.nodes);
    let copy = doc.clone_node_checked(root, true, &mut budget()).unwrap();
    assert_eq!(copy, old);
    assert_eq!(doc.nodes.len(), old + 6);
    assert_eq!(
        doc.nodes[copy].children,
        [old + 1, old + 2, old + 4, old + 5]
    );
    assert_eq!(doc.nodes[old + 2].children, [old + 3]);
    assert!(units(&doc, old + 1).is_empty());
    assert_eq!(units(&doc, old + 3), [67]);
    assert_eq!(units(&doc, old + 4), [109, 97, 114, 107]);
    assert_eq!(units(&doc, old + 5), [65]);
    for id in [old + 1, old + 2, old + 4, old + 5] {
        assert_eq!(doc.nodes[id].parent, Some(copy));
    }
    assert_eq!(doc.nodes[old + 3].parent, Some(old + 2));
    assert_eq!(format!("{:?}", &doc.nodes[..old]), prefix);
    let shallow = doc.clone_node_checked(root, false, &mut budget()).unwrap();
    assert!(doc.nodes[shallow].children.is_empty());
}

fn templates() -> (Document, NodeId, NodeId, NodeId) {
    let mut doc = Document::parse("");
    let outer = doc.create_element("template");
    let inner = doc.create_element("template");
    let oc = doc.template_contents(outer).unwrap();
    let ic = doc.template_contents(inner).unwrap();
    let ordinary = text(&mut doc, &[79]);
    let inside = text(&mut doc, &[65, 55296]);
    let comment = doc.create_comment("I");
    link(&mut doc, outer, ordinary);
    link(&mut doc, oc, inner);
    link(&mut doc, inner, comment);
    link(&mut doc, ic, inside);
    (admit(doc), outer, oc, ic)
}

#[test]
fn shallow_and_deep_templates_have_fresh_distinct_hosted_graphs() {
    let (mut doc, source, _, _) = templates();
    let old = doc.nodes.len();
    let prefix = format!("{:?}", doc.nodes);
    let a = doc
        .clone_node_checked(source, false, &mut budget())
        .unwrap();
    assert_eq!(a, old);
    assert_eq!(doc.nodes.len(), old + 2);
    assert_eq!(doc.template_contents(a), Some(old + 1));
    assert!(doc.nodes[a].children.is_empty() && doc.nodes[old + 1].children.is_empty());
    assert!(matches!(doc.nodes[old+1].kind,NodeKind::DocumentFragment{host:Some(id)} if id==a));
    let b = doc.nodes.len();
    assert_eq!(doc.clone_node_checked(source, true, &mut budget()), Ok(b));
    assert_eq!(doc.nodes.len(), b + 7);
    assert_eq!(doc.template_contents(b), Some(b + 1));
    assert_eq!(doc.nodes[b + 1].children, [b + 2]);
    assert_eq!(doc.template_contents(b + 2), Some(b + 3));
    assert_eq!(doc.nodes[b + 3].children, [b + 4]);
    assert_eq!(doc.nodes[b + 2].children, [b + 5]);
    assert_eq!(doc.nodes[b].children, [b + 6]);
    assert_eq!(units(&doc, b + 4), [65, 55296]);
    assert_eq!(units(&doc, b + 5), [73]);
    assert_eq!(units(&doc, b + 6), [79]);
    assert_eq!(doc.nodes[b + 4].parent, Some(b + 3));
    assert_eq!(doc.nodes[b + 5].parent, Some(b + 2));
    assert_eq!(doc.nodes[b + 3].parent, None);
    assert!(matches!(doc.nodes[b+3].kind,NodeKind::DocumentFragment{host:Some(id)} if id==b+2));
    assert_eq!(format!("{:?}", &doc.nodes[..old]), prefix);
}

#[test]
fn directly_copied_hosted_fragment_has_no_source_host() {
    let (mut doc, _, content, _) = templates();
    let old = doc.nodes.len();
    let copy = doc
        .clone_node_checked(content, true, &mut budget())
        .unwrap();
    assert_eq!(copy, old);
    assert!(matches!(
        doc.nodes[copy].kind,
        NodeKind::DocumentFragment { host: None }
    ));
    assert_eq!(doc.nodes[copy].parent, None);
    assert_eq!(doc.nodes[copy].children, [old + 1]);
    assert_eq!(doc.nodes[old + 1].parent, Some(copy));
    assert_ne!(
        doc.template_contents(old + 1),
        doc.template_contents(doc.nodes[content].children[0])
    );
    let empty = doc
        .clone_node_checked(content, false, &mut budget())
        .unwrap();
    assert!(doc.nodes[empty].children.is_empty());
}

fn depth_case(templates: bool) -> (Document, NodeId, NodeId) {
    let mut doc = Document::parse("");
    let source = doc.create_element("div");
    let mut current = source;
    let mut depth = 0;
    while depth < MAX_DEPTH {
        if templates {
            let template = doc.create_element("template");
            link(&mut doc, current, template);
            current = doc.template_contents(template).unwrap();
            depth += 2;
        } else {
            let next = doc.create_element("div");
            link(&mut doc, current, next);
            current = next;
            depth += 1;
        }
    }
    assert_eq!(depth, 256);
    (admit(doc), source, current)
}

#[test]
fn relative_ordinary_and_template_depth_256_is_admitted_and_257_refused() {
    for templates in [false, true] {
        let (mut doc, source, last) = depth_case(templates);
        let old = doc.nodes.len();
        assert_eq!(doc.clone_node_checked(source, true, &mut budget()), Ok(old));
        assert_eq!(doc.nodes.len(), old + 257);
        let (mut bad, source, last_again) = depth_case(templates);
        assert_eq!(last, last_again);
        let extra = text(&mut bad, &[120]);
        link(&mut bad, last_again, extra);
        let before = state(&bad);
        let mut meter = budget();
        assert_eq!(
            bad.clone_node_checked(source, true, &mut meter),
            Err(DomDataError::LimitExceeded)
        );
        assert!(meter.steps > 0);
        assert_eq!(state(&bad), before);
        let leaf = bad.clone_node_checked(extra, true, &mut budget()).unwrap();
        assert_eq!(units(&bad, leaf), [120]);
    }
}

#[test]
fn reached_malformed_edges_are_refused_before_publication() {
    for variant in 0..10 {
        let mut doc = Document::parse("");
        let root = doc.create_element("div");
        let child = doc.create_element("span");
        link(&mut doc, root, child);
        let other = text(&mut doc, &[65]);
        let mut source = root;
        match variant {
            0 => source = usize::MAX,
            1 => doc.nodes[root].children[0] = usize::MAX,
            2 => doc.nodes[child].parent = None,
            3 => doc.nodes[root].children.push(child),
            4 => {
                doc.nodes[root].children = vec![root];
                doc.nodes[root].parent = Some(root);
            }
            5 => {
                doc.nodes[child].children.push(root);
                doc.nodes[root].parent = Some(child);
            }
            6 => doc.nodes[child].kind = NodeKind::Document,
            7 => doc.nodes[child].kind = NodeKind::DocumentFragment { host: None },
            8 => {
                doc.nodes[child].kind = NodeKind::Doctype(Doctype {
                    name: "x".into(),
                    public_id: None,
                    system_id: None,
                    force_quirks: false,
                })
            }
            9 => {
                doc.nodes[child].kind = NodeKind::Text("bad".into());
                doc.nodes[child].children.push(other);
            }
            _ => unreachable!(),
        }
        let before = state(&doc);
        assert_eq!(
            doc.clone_node_checked(source, true, &mut budget()),
            Err(DomDataError::InvalidNode),
            "variant={variant}"
        );
        assert_eq!(state(&doc), before);
    }
}

#[test]
fn template_pairs_and_namespace_annotations_have_local_validation() {
    for variant in 0..9 {
        let (mut doc, source, content, _) = templates();
        match variant {
            0 => {
                let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
                    panic!()
                };
                el.template_contents = None;
            }
            1 => {
                let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
                    panic!()
                };
                el.template_contents = Some(usize::MAX);
            }
            2 => doc.nodes[content].kind = NodeKind::Text("bad".into()),
            3 => {
                doc.nodes[content].kind = NodeKind::DocumentFragment {
                    host: Some(doc.root),
                }
            }
            4 => doc.nodes[content].parent = Some(source),
            5 => {
                let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
                    panic!()
                };
                el.tag = "div".into();
            }
            6 => {
                let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
                    panic!()
                };
                el.attr_namespaces
                    .insert("xlink:href".into(), AttributeNamespace::XLink);
            }
            7 => {
                let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
                    panic!()
                };
                el.attrs.insert("xml:lang".into(), "x".into());
                el.attr_namespaces
                    .insert("xml:lang".into(), AttributeNamespace::XLink);
            }
            8 => {
                let NodeKind::Element(el) = &mut doc.nodes[source].kind else {
                    panic!()
                };
                for i in 0..1025 {
                    el.attrs.insert(i.to_string(), String::new());
                }
            }
            _ => unreachable!(),
        }
        let before = state(&doc);
        assert_eq!(
            doc.clone_node_checked(source, false, &mut budget()),
            Err(DomDataError::InvalidNode),
            "variant={variant}"
        );
        assert_eq!(state(&doc), before);
    }
}

#[test]
fn shallow_unreached_edges_and_old_leaf_ancestry_are_outside_the_selected_check() {
    let mut doc = Document::parse("");
    let source = doc.create_element("div");
    doc.nodes[source].children.push(usize::MAX);
    let copy = doc
        .clone_node_checked(source, false, &mut budget())
        .unwrap();
    assert!(doc.nodes[copy].children.is_empty());
    assert_eq!(
        doc.clone_node_checked(source, true, &mut budget()),
        Err(DomDataError::InvalidNode)
    );
    let leaf = text(&mut doc, &[55296]);
    doc.nodes[leaf].parent = Some(usize::MAX);
    let copy = doc.clone_node_checked(leaf, true, &mut budget()).unwrap();
    assert_eq!(units(&doc, copy), [55296]);
    assert_eq!(doc.nodes[copy].parent, None);
    let (mut doc, template, content, _) = templates();
    doc.nodes[content].children.push(usize::MAX);
    assert!(
        doc.clone_node_checked(template, false, &mut budget())
            .is_ok()
    );
    assert_eq!(
        doc.clone_node_checked(template, true, &mut budget()),
        Err(DomDataError::InvalidNode)
    );
}

fn details_graph() -> (Document, NodeId) {
    let mut doc = Document::parse("");
    let root = doc.create_element("div");
    let first = doc.create_element("details");
    let template = doc.create_element("template");
    let content = doc.template_contents(template).unwrap();
    let inside = doc.create_element("details");
    let ordinary = doc.create_element("details");
    let last = doc.create_element("details");
    for id in [first, inside, ordinary, last] {
        doc.set_attr(id, "name", "g");
        doc.set_attr(id, "open", "");
    }
    for id in [first, template, last] {
        link(&mut doc, root, id);
    }
    link(&mut doc, content, inside);
    link(&mut doc, template, ordinary);
    let doc = admit(doc);
    for id in [first, inside, ordinary, last] {
        assert_eq!(doc.attr(id, "open"), Some(""));
    }
    assert_eq!(doc.details_name_bytes, 4);
    assert!(doc.details_toggles.is_empty());
    (doc, root)
}

#[test]
fn template_hook_order_and_closed_to_closed_tasks_have_literal_sequence() {
    let (mut doc, source) = details_graph();
    let b = doc.nodes.len();
    let old = format!("{:?}", doc.nodes);
    let old_groups = doc.details_groups.clone();
    let retained = doc.retained_bytes;
    assert_eq!(doc.clone_node_checked(source, true, &mut budget()), Ok(b));
    assert_eq!(doc.nodes.len(), b + 7);
    assert_eq!(doc.nodes[b].children, [b + 1, b + 2, b + 6]);
    assert_eq!(doc.template_contents(b + 2), Some(b + 3));
    assert_eq!(doc.nodes[b + 3].children, [b + 4]);
    assert_eq!(doc.nodes[b + 2].children, [b + 5]);
    assert_eq!(doc.attr(b + 1, "open"), Some(""));
    assert_eq!(doc.attr(b + 4, "open"), Some(""));
    assert_eq!(doc.attr(b + 5, "open"), None);
    assert_eq!(doc.attr(b + 6, "open"), None);
    assert_eq!(doc.details_sequence, 6);
    assert_eq!(doc.details_name_bytes, 8);
    assert_eq!(doc.retained_bytes, retained + 67);
    let expected = [
        (0, b + 1, true),
        (1, b + 4, true),
        (3, b + 5, false),
        (5, b + 6, false),
    ];
    assert_eq!(doc.details_toggles.len(), 4);
    assert_eq!(doc.details_trackers.len(), 4);
    for (seq, node, new_open) in expected {
        assert_eq!(
            doc.details_toggles.get(&seq),
            Some(&DetailsToggle {
                node,
                old_open: false,
                new_open
            })
        );
        let tracker = doc.details_trackers.get(&node).unwrap();
        assert_eq!((tracker.task, tracker.old_open), (seq, false));
    }
    let mut groups = old_groups;
    groups.insert((b, "g".into()), b + 1);
    groups.insert((b + 3, "g".into()), b + 4);
    assert_eq!(doc.details_groups, groups);
    assert_eq!(format!("{:?}", &doc.nodes[..b]), old);
}

#[test]
fn old_active_tasks_frozen_base_and_cold_summary_cache_survive_clone() {
    let (mut doc, source) = details_graph();
    let first = doc.nodes[source].children[0];
    let summary = doc.create_element("summary");
    link(&mut doc, first, summary);
    assert_eq!(doc.first_summary(first), Some(summary));
    let active_node = doc.create_element("details");
    doc.set_attr(active_node, "open", "");
    let active = doc.peek_details_toggle().unwrap();
    doc.begin_details_toggle(active.id).unwrap();
    let pending_node = doc.create_element("details");
    doc.set_attr(pending_node, "open", "");
    let body = doc.query_selector("body").unwrap();
    let base = doc.create_element("base");
    doc.set_attr(base, "href", "https://first.test/a/");
    doc.append_child(body, base);
    doc.initialize_url(url::Url::parse("https://origin.test/page").unwrap());
    let detached = doc.create_element("base");
    doc.set_attr(detached, "href", "https://detached.test/b/");
    link(&mut doc, source, detached);
    let old_base = (
        doc.url.clone(),
        doc.first_base.clone(),
        doc.base_tracking,
        doc.encoding.name(),
        doc.encoding_declaration.map(|encoding| encoding.name()),
    );
    let old_href = doc.base_href_bytes;
    let old_tasks = doc.details_toggles.clone();
    let old_trackers = format!("{:?}", doc.details_trackers);
    let old_summaries = doc.details_summaries.borrow().clone();
    let old_count = doc.nodes.len();
    let old_prefix = format!("{:?}", doc.nodes);
    let copy = doc.clone_node_checked(source, true, &mut budget()).unwrap();
    assert_eq!(doc.details_active, Some(active));
    assert!(doc.details_toggle_running());
    for (key, value) in &old_tasks {
        assert_eq!(doc.details_toggles.get(key), Some(value));
    }
    let old_tracker_rows: BTreeMap<_, _> = doc
        .details_trackers
        .iter()
        .filter(|(id, _)| **id < old_count)
        .map(|(id, v)| (*id, *v))
        .collect();
    assert_eq!(format!("{old_tracker_rows:?}"), old_trackers);
    assert_eq!(
        (
            doc.url.clone(),
            doc.first_base.clone(),
            doc.base_tracking,
            doc.encoding.name(),
            doc.encoding_declaration.map(|encoding| encoding.name())
        ),
        old_base
    );
    assert_eq!(doc.base_url().as_str(), "https://first.test/a/");
    assert_eq!(
        doc.base_href_bytes,
        old_href + "https://detached.test/b/".len()
    );
    assert_eq!(*doc.details_summaries.borrow(), old_summaries);
    let new_details = doc.nodes[copy].children[0];
    let new_summary = doc.nodes[new_details].children[0];
    assert_ne!(new_summary, summary);
    assert!(!doc.details_summaries.borrow().contains_key(&new_details));
    assert_eq!(doc.first_summary(new_details), Some(new_summary));
    assert_eq!(doc.first_summary(first), Some(summary));
    assert_eq!(format!("{:?}", &doc.nodes[..old_count]), old_prefix);
}

#[test]
fn sequence_overflow_and_specific_metadata_collisions_are_atomic() {
    for collision in 0..4 {
        let mut doc = Document::parse("");
        let source = doc.create_element("details");
        doc.set_attr(source, "name", "g");
        doc.set_attr(source, "open", "");
        while let Some(task) = doc.peek_details_toggle() {
            doc.begin_details_toggle(task.id).unwrap();
            doc.finish_details_toggle(task.id);
        }
        let fresh = doc.nodes.len();
        let seq = doc.details_sequence;
        match collision {
            0 => {
                doc.details_groups.insert((fresh, "g".into()), source);
            }
            1 => {
                doc.details_trackers.insert(
                    fresh,
                    DetailsToggleTracker {
                        task: seq,
                        old_open: true,
                    },
                );
            }
            2 => {
                doc.details_toggles.insert(
                    seq,
                    DetailsToggle {
                        node: source,
                        old_open: true,
                        new_open: false,
                    },
                );
            }
            3 => {
                doc.details_active = Some(DetailsToggleTask {
                    id: seq,
                    event: DetailsToggle {
                        node: source,
                        old_open: true,
                        new_open: false,
                    },
                })
            }
            _ => unreachable!(),
        }
        let before = state(&doc);
        assert_eq!(
            doc.clone_node_checked(source, false, &mut budget()),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(state(&doc), before);
    }
    let mut doc = Document::parse("");
    let source = doc.create_element("details");
    doc.set_attr(source, "open", "");
    // Private sequence arithmetic, not a claim to have dispatched u64::MAX tasks.
    doc.details_sequence = u64::MAX - 1;
    let copy = doc
        .clone_node_checked(source, false, &mut budget())
        .unwrap();
    assert_eq!(doc.details_sequence, u64::MAX);
    assert_eq!(doc.details_toggles.get(&(u64::MAX - 1)).unwrap().node, copy);
    let before = state(&doc);
    assert_eq!(
        doc.clone_node_checked(source, false, &mut budget()),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(state(&doc), before);
    let leaf = text(&mut doc, &[65]);
    assert!(doc.clone_node_checked(leaf, false, &mut budget()).is_ok());
}

fn cut_case(hooks: bool, growth: bool) -> (Document, NodeId) {
    let mut doc = Document::parse("");
    let source = if hooks {
        let id = doc.create_element("details");
        doc.set_attr(id, "name", "g");
        doc.set_attr(id, "open", "");
        id
    } else {
        text(&mut doc, &[55296, 65])
    };
    arena(&mut doc, usize::from(!growth));
    (doc, source)
}

#[test]
fn every_small_work_cut_keeps_full_state_and_all_old_capacities() {
    for hooks in [false, true] {
        let (mut measured, source) = cut_case(hooks, true);
        let mut meter = budget();
        let copy = measured
            .clone_node_checked(source, false, &mut meter)
            .unwrap();
        if hooks {
            assert_eq!(measured.attr(copy, "open"), Some(""));
            assert_eq!(measured.details_toggles.len(), 2);
        } else {
            assert_eq!(units(&measured, copy), [55296, 65]);
        }
        let cost = WORK - meter.steps;
        assert!(cost > 0);
        for steps in 0..cost {
            let (mut doc, source) = cut_case(hooks, true);
            let before = state(&doc);
            let mut limit = budget();
            limit.steps = steps;
            assert_eq!(
                doc.clone_node_checked(source, false, &mut limit),
                Err(DomDataError::LimitExceeded),
                "hooks={hooks}, steps={steps}"
            );
            assert_eq!(limit.steps, 0);
            assert_eq!(state(&doc), before);
        }
    }
}

#[test]
fn exact_and_one_short_heap_and_work_preserve_absolute_admission() {
    for hooks in [false, true] {
        for growth in [false, true] {
            let (mut measured, source) = cut_case(hooks, growth);
            let mut meter = budget();
            measured
                .clone_node_checked(source, false, &mut meter)
                .unwrap();
            let work = WORK - meter.steps;
            let heap = meter.allocated;
            let expected = format!("{measured:?}");
            for (steps, available, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
            ] {
                let (mut doc, source) = cut_case(hooks, growth);
                let before = state(&doc);
                let mut limit = DomMutationBudget {
                    steps,
                    allocated: 37,
                    heap_limit: 37 + available,
                };
                let result = doc.clone_node_checked(source, false, &mut limit);
                if success {
                    assert!(result.is_ok());
                    assert_eq!(limit.steps, 0);
                    assert_eq!(limit.allocated, 37 + heap);
                    assert_eq!(format!("{doc:?}"), expected);
                } else {
                    assert_eq!(result, Err(DomDataError::LimitExceeded));
                    assert_eq!(state(&doc), before);
                    if steps < work {
                        assert_eq!(limit.steps, 0);
                    } else {
                        assert!(limit.allocated > limit.heap_limit);
                    }
                }
            }
        }
    }
}

#[test]
fn real_node_cap_counts_empty_results_and_shallow_template_content() {
    for (template, slots, success) in [
        (false, 1, true),
        (false, 0, false),
        (true, 1, false),
        (true, 2, true),
    ] {
        let mut doc = Document::parse("");
        let source = if template {
            doc.create_element("template")
        } else {
            doc.create_comment("")
        };
        while doc.nodes.len() < MAX_NODES - slots {
            doc.nodes.push(Node {
                parent: None,
                children: Vec::new(),
                kind: NodeKind::Comment("".into()),
            });
        }
        let len = doc.nodes.len();
        let capacity = doc.nodes.capacity();
        let retained = doc.retained_bytes;
        let result = doc.clone_node_checked(source, false, &mut budget());
        assert_eq!(result.is_ok(), success);
        if success {
            assert_eq!(doc.nodes.len(), MAX_NODES);
            assert_eq!(result, Ok(len));
            if template {
                assert_eq!(doc.template_contents(len), Some(len + 1));
            }
        } else {
            assert_eq!(result, Err(DomDataError::LimitExceeded));
            assert_eq!(doc.nodes.len(), len);
            assert_eq!(doc.nodes.capacity(), capacity);
            assert_eq!(doc.retained_bytes, retained);
        }
    }
}

#[test]
fn retained_counter_boundaries_credit_only_removed_copy_attributes() {
    // Explicit private counter arithmetic; this small setup does not pretend to
    // contain a reconstructed 32 MiB source payload.
    for available in [66, 67] {
        let (mut doc, source) = details_graph();
        doc.retained_bytes = MAX_DOM_BYTES - available;
        let before = state(&doc);
        let result = doc.clone_node_checked(source, true, &mut budget());
        if available == 67 {
            assert!(result.is_ok());
            assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        } else {
            assert_eq!(result, Err(DomDataError::LimitExceeded));
            assert_eq!(state(&doc), before);
        }
    }
    let mut doc = Document::parse("");
    let source = text(&mut doc, &[65]);
    doc.retained_bytes = MAX_DOM_BYTES;
    let before = state(&doc);
    assert_eq!(
        doc.clone_node_checked(source, false, &mut budget()),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(state(&doc), before);
}

#[test]
fn unrelated_arena_and_groups_do_not_add_leaf_scratch_but_real_growth_is_paid() {
    fn fixture(large: bool, growth: bool) -> (Document, NodeId) {
        let mut doc = Document::parse("");
        let source = text(&mut doc, &[55296, 65]);
        if large {
            for _ in 0..1024 {
                doc.create_comment("");
            }
            for i in 0..32 {
                let id = doc.create_element("details");
                doc.set_attr(id, "name", &format!("g{i}"));
                doc.set_attr(id, "open", "");
            }
        }
        arena(&mut doc, usize::from(!growth));
        (doc, source)
    }
    let mut costs = Vec::new();
    for large in [false, true] {
        let (mut doc, source) = fixture(large, false);
        let mut meter = budget();
        let copy = doc.clone_node_checked(source, false, &mut meter).unwrap();
        assert_eq!(units(&doc, copy), [55296, 65]);
        costs.push((WORK - meter.steps, meter.allocated));
    }
    assert_eq!(costs[0], costs[1]);
    let (mut doc, source) = fixture(true, true);
    let old = doc.nodes.len();
    let mut meter = budget();
    doc.clone_node_checked(source, false, &mut meter).unwrap();
    assert_eq!(WORK - meter.steps, costs[0].0 + 1 + 2 * old);
    assert_eq!(meter.allocated, costs[0].1 + (old + 1) * size_of::<Node>());
    let (mut refused, source) = fixture(true, true);
    let before = state(&refused);
    let mut limit = budget();
    limit.heap_limit = meter.allocated - 1;
    assert_eq!(
        refused.clone_node_checked(source, false, &mut limit),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(state(&refused), before);
}
