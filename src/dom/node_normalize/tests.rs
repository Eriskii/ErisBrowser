use super::*;

const STEPS: usize = 100_000;

fn budget() -> DomMutationBudget {
    DomMutationBudget {
        steps: STEPS,
        allocated: 29,
        heap_limit: 29,
    }
}

fn text(doc: &mut Document, units: &[u16]) -> NodeId {
    doc.create_text_node_owned(DomString::from_units_owned(units.to_vec()).unwrap())
        .unwrap()
}

fn units(doc: &Document, id: NodeId) -> Vec<u16> {
    match &doc.nodes[id].kind {
        NodeKind::Text(data) => data.units().collect(),
        _ => panic!("expected Text"),
    }
}

fn merged_run() -> (Document, NodeId, [NodeId; 5]) {
    let mut doc = Document::parse("");
    let parent = doc.create_document_fragment();
    let survivor = text(&mut doc, &[65, 0xd83d, 0xde80, 66]);
    let empty = text(&mut doc, &[]);
    let tail = text(&mut doc, &[0xde80, 66]);
    let barrier = doc.create_comment_owned("stop".into()).unwrap();
    let following = text(&mut doc, &[90]);
    for child in [survivor, empty, tail, barrier, following] {
        doc.append_child(parent, child);
    }
    (doc, parent, [survivor, empty, tail, barrier, following])
}

#[test]
fn removes_only_tail_links_and_retains_every_payload_id_and_capacity() {
    let (mut doc, parent, [survivor, empty, tail, barrier, following]) = merged_run();
    let count = doc.nodes.len();
    let arena_capacity = doc.nodes.capacity();
    let child_capacity = doc.nodes[parent].children.capacity();
    let retained = doc.retained_bytes;
    let mut meter = budget();
    doc.remove_normalize_text_range(parent, 1, 3, Some(survivor), &mut meter)
        .unwrap();
    assert_eq!(doc.nodes[parent].children, [survivor, barrier, following]);
    assert_eq!(doc.nodes[survivor].parent, Some(parent));
    assert_eq!(doc.nodes[empty].parent, None);
    assert_eq!(doc.nodes[tail].parent, None);
    assert_eq!(units(&doc, survivor), [65, 0xd83d, 0xde80, 66]);
    assert_eq!(units(&doc, tail), [0xde80, 66]);
    assert!(units(&doc, empty).is_empty());
    assert_eq!(units(&doc, following), [90]);
    assert_eq!(doc.nodes.len(), count);
    assert_eq!(doc.nodes.capacity(), arena_capacity);
    assert_eq!(doc.nodes[parent].children.capacity(), child_capacity);
    assert_eq!(doc.retained_bytes, retained);
    assert_eq!(meter.allocated, 29);
    assert!(meter.steps < STEPS);
}

#[test]
fn empty_removal_does_not_merge_or_change_the_next_text() {
    let mut doc = Document::parse("");
    let parent = doc.create_document_fragment();
    let empty = text(&mut doc, &[]);
    let a = text(&mut doc, &[65]);
    let b = text(&mut doc, &[66]);
    for child in [empty, a, b] {
        doc.append_child(parent, child);
    }
    doc.remove_normalize_text_range(parent, 0, 1, None, &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[parent].children, [a, b]);
    assert_eq!(doc.nodes[empty].parent, None);
    assert_eq!(units(&doc, a), [65]);
    assert_eq!(units(&doc, b), [66]);
}

#[test]
fn every_insufficient_work_cut_preserves_the_already_merged_prefix() {
    let (mut measured_doc, parent, ids) = merged_run();
    let mut measured = budget();
    measured_doc
        .remove_normalize_text_range(parent, 1, 3, Some(ids[0]), &mut measured)
        .unwrap();
    let work = STEPS - measured.steps;
    assert!(work > 0);
    for available in 0..=work {
        let (mut doc, parent, [survivor, empty, tail, barrier, following]) = merged_run();
        let before = format!("{doc:?}");
        let capacity = doc.nodes[parent].children.capacity();
        let mut meter = budget();
        meter.steps = available;
        let result = doc.remove_normalize_text_range(parent, 1, 3, Some(survivor), &mut meter);
        assert_eq!(meter.steps, 0);
        assert_eq!(meter.allocated, 29);
        assert_eq!(doc.nodes[parent].children.capacity(), capacity);
        if available == work {
            assert_eq!(result, Ok(()));
            assert_eq!(doc.nodes[parent].children, [survivor, barrier, following]);
            assert_eq!(doc.nodes[empty].parent, None);
            assert_eq!(doc.nodes[tail].parent, None);
        } else {
            assert_eq!(result, Err(DomDataError::LimitExceeded));
            assert_eq!(format!("{doc:?}"), before);
            assert_eq!(doc.nodes[tail].parent, Some(parent));
        }
        assert_eq!(units(&doc, survivor), [65, 0xd83d, 0xde80, 66]);
        assert_eq!(units(&doc, tail), [0xde80, 66]);
    }
}

#[test]
fn suffix_compaction_pays_for_actual_moved_ids() {
    let mut charged = Vec::new();
    for followers in [0, 3, 40] {
        let mut doc = Document::parse("");
        let parent = doc.create_document_fragment();
        let a = text(&mut doc, &[65]);
        let b = text(&mut doc, &[66]);
        doc.append_child(parent, a);
        doc.append_child(parent, b);
        let mut expected = vec![a];
        for _ in 0..followers {
            let comment = doc.create_comment_owned("barrier".into()).unwrap();
            doc.append_child(parent, comment);
            expected.push(comment);
        }
        let mut meter = budget();
        doc.remove_normalize_text_range(parent, 1, 2, Some(a), &mut meter)
            .unwrap();
        assert_eq!(doc.nodes[parent].children, expected);
        assert_eq!(doc.nodes[b].parent, None);
        assert_eq!(meter.allocated, 29);
        charged.push(STEPS - meter.steps);
    }
    assert_eq!(charged[1] - charged[0], 6);
    assert_eq!(charged[2] - charged[0], 80);
}

#[test]
fn local_range_and_parent_failures_leave_all_data_and_links_unchanged() {
    for scenario in 0..8 {
        let (mut doc, parent, ids) = merged_run();
        let (mut owner, mut start, mut end) = (parent, 1, 3);
        match scenario {
            0 => owner = usize::MAX,
            1 => owner = ids[0],
            2 => start = end,
            3 => start = end + 1,
            4 => end = usize::MAX,
            5 => doc.nodes[parent].parent = Some(doc.root),
            6 => doc.nodes[parent].kind = NodeKind::Document,
            7 => start = 0,
            _ => unreachable!(),
        }
        let before = format!("{doc:?}");
        let mut meter = budget();
        assert_eq!(
            doc.remove_normalize_text_range(owner, start, end, Some(ids[0]), &mut meter),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(format!("{doc:?}"), before);
        assert_eq!(meter.allocated, 29);
        assert!(meter.steps < STEPS);
    }
}

#[test]
fn reached_ids_backlinks_leaf_kinds_and_survivor_are_checked_before_writes() {
    // Duplicate uniqueness is a caller proof, not an assertion provided by this
    // helper. Test the independent local checks that the helper does promise.
    for scenario in 0..9 {
        let (mut doc, parent, [survivor, empty, tail, barrier, following]) = merged_run();
        let mut kept = Some(survivor);
        match scenario {
            0 => doc.nodes[parent].children[2] = usize::MAX,
            1 => doc.nodes[parent].children[2] = parent,
            2 => doc.nodes[parent].children[2] = doc.root,
            3 => doc.nodes[parent].children[2] = survivor,
            4 => doc.nodes[parent].children[2] = barrier,
            5 => doc.nodes[tail].children.push(following),
            6 => doc.nodes[tail].parent = None,
            7 => doc.nodes[survivor].children.push(empty),
            8 => kept = Some(tail),
            _ => unreachable!(),
        }
        let before = format!("{doc:?}");
        assert_eq!(
            doc.remove_normalize_text_range(parent, 1, 3, kept, &mut budget()),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(format!("{doc:?}"), before);
    }
}

#[test]
fn removal_refusal_keeps_prior_empty_detachment_and_payload_replacement() {
    let mut doc = Document::parse("");
    let parent = doc.create_document_fragment();
    let empty = text(&mut doc, &[]);
    let a = text(&mut doc, &[65]);
    let b = text(&mut doc, &[66]);
    for child in [empty, a, b] {
        doc.append_child(parent, child);
    }
    doc.remove_normalize_text_range(parent, 0, 1, None, &mut budget())
        .unwrap();
    doc.replace_character_data(a, "AB".into()).unwrap();
    let retained = doc.retained_bytes;
    let mut meter = budget();
    meter.steps = 0;
    assert_eq!(
        doc.remove_normalize_text_range(parent, 1, 2, Some(a), &mut meter),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(doc.nodes[parent].children, [a, b]);
    assert_eq!(doc.nodes[empty].parent, None);
    assert_eq!(doc.nodes[b].parent, Some(parent));
    assert_eq!(units(&doc, a), [65, 66]);
    assert_eq!(units(&doc, b), [66]);
    assert_eq!(doc.retained_bytes, retained);
    assert_eq!(meter.allocated, 29);
}

#[test]
fn document_host_text_and_full_arena_need_no_new_node_or_heap() {
    let mut doc = Document::parse("");
    let root = doc.root;
    let a = text(&mut doc, &[65, 66]);
    let b = text(&mut doc, &[66]);
    doc.append_child(root, a);
    doc.append_child(root, b);
    let start = doc.nodes[root].children.len() - 1;
    doc.nodes.resize_with(MAX_NODES, || Node {
        parent: None,
        children: Vec::new(),
        kind: NodeKind::Text(DomString::default()),
    });
    let capacity = doc.nodes.capacity();
    let retained = doc.retained_bytes;
    let mut meter = budget();
    doc.remove_normalize_text_range(root, start, start + 1, Some(a), &mut meter)
        .unwrap();
    assert_eq!(doc.nodes.len(), MAX_NODES);
    assert_eq!(doc.nodes.capacity(), capacity);
    assert_eq!(doc.nodes[root].children.last(), Some(&a));
    assert_eq!(doc.nodes[b].parent, None);
    assert_eq!(units(&doc, b), [66]);
    assert_eq!(doc.retained_bytes, retained);
    assert_eq!(meter.allocated, meter.heap_limit);
}

#[test]
fn admitted_max_depth_is_preserved_without_an_ancestor_walk() {
    let mut doc = Document::parse("");
    let mut parent = doc.root;
    for _ in 0..MAX_DEPTH - 1 {
        let child = doc.create_element("div");
        doc.append_child(parent, child);
        parent = child;
    }
    let a = text(&mut doc, &[65, 66]);
    let b = text(&mut doc, &[66]);
    // Raw append has a stricter depth check than snapshot admission. Build the
    // reciprocal boundary edges explicitly, then validate the complete snapshot.
    doc.nodes[parent].children.extend([a, b]);
    doc.nodes[a].parent = Some(parent);
    doc.nodes[b].parent = Some(parent);
    assert_eq!(doc.nodes[parent].children, [a, b]);
    for leaf in [a, b] {
        let mut cursor = leaf;
        let mut depth = 0;
        while let Some(ancestor) = doc.nodes[cursor].parent {
            depth += 1;
            assert!(depth <= MAX_DEPTH);
            cursor = ancestor;
        }
        assert_eq!(cursor, doc.root);
        assert_eq!(depth, MAX_DEPTH);
    }
    // The existing snapshot validator admits root depth0 plus256 edges.
    let mut doc = Document::from_snapshot(doc.nodes.clone(), doc.root, false, doc.mode).unwrap();
    let mut meter = budget();
    doc.remove_normalize_text_range(parent, 1, 2, Some(a), &mut meter)
        .unwrap();
    assert_eq!(doc.nodes[parent].children, [a]);
    assert_eq!(doc.nodes[a].parent, Some(parent));
    assert_eq!(doc.nodes[b].parent, None);
    assert_eq!(units(&doc, b), [66]);
    assert_eq!(meter.allocated, 29);
}

#[test]
fn ordinary_template_children_and_content_remain_independent() {
    let mut doc = Document::parse("<template></template>");
    let host = doc.query_selector("template").unwrap();
    let content = doc.template_contents(host).unwrap();
    let a = text(&mut doc, &[65, 66]);
    let b = text(&mut doc, &[66]);
    let x = text(&mut doc, &[88, 89]);
    let y = text(&mut doc, &[89]);
    doc.append_child(host, a);
    doc.append_child(host, b);
    doc.append_child(content, x);
    doc.append_child(content, y);
    doc.remove_normalize_text_range(host, 1, 2, Some(a), &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[host].children, [a]);
    assert_eq!(doc.nodes[content].children, [x, y]);
    doc.remove_normalize_text_range(content, 1, 2, Some(x), &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[content].children, [x]);
    assert_eq!(doc.nodes[host].children, [a]);
    assert_eq!(doc.template_contents(host), Some(content));
    assert_eq!(doc.nodes[b].parent, None);
    assert_eq!(doc.nodes[y].parent, None);
    assert_eq!(units(&doc, b), [66]);
    assert_eq!(units(&doc, y), [89]);
}

#[test]
fn text_removal_preserves_frozen_base_summary_cache_and_details_state() {
    let mut doc = Document::parse(
        "<head><base href='/assets/'></head><body><details open name=g>lead<summary>label</summary></details></body>",
    );
    doc.initialize_url(url::Url::parse("https://example.test/page").unwrap());
    let details = doc.query_selector("details").unwrap();
    let summary = doc.query_selector("summary").unwrap();
    let lead = doc.nodes[details].children[0];
    let tail = text(&mut doc, &[]);
    doc.append_child(details, tail);
    doc.nodes[details].children.swap(1, 2);
    assert_eq!(doc.nodes[details].children, [lead, tail, summary]);
    assert_eq!(doc.first_summary(details), Some(summary));
    let metadata = |doc: &Document| {
        format!(
            "{:?}",
            (
                &doc.first_base,
                &doc.details_groups,
                &doc.details_toggles,
                &doc.details_trackers,
                &doc.details_active,
                doc.details_sequence,
                &doc.details_summaries,
            )
        )
    };
    let before = metadata(&doc);
    doc.remove_normalize_text_range(details, 1, 2, Some(lead), &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[details].children, [lead, summary]);
    assert_eq!(metadata(&doc), before);
    assert_eq!(doc.first_summary(details), Some(summary));
    assert_eq!(doc.base_url().as_str(), "https://example.test/assets/");
    assert_eq!(doc.nodes[tail].parent, None);
    assert!(units(&doc, tail).is_empty());
}
