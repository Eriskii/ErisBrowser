use super::*;

const STEPS: usize = 100_000;

fn budget() -> DomMutationBudget {
    DomMutationBudget {
        steps: STEPS,
        allocated: 0,
        heap_limit: 8 * 1024 * 1024,
    }
}

fn text(doc: &mut Document, units: &[u16]) -> NodeId {
    doc.create_text_node_owned(DomString::from_units_owned(units.to_vec()).unwrap())
        .unwrap()
}

fn units(doc: &Document, id: NodeId) -> Vec<u16> {
    doc.text_operation_data(id).unwrap().units().collect()
}

fn insertion_case(growth: bool) -> (Document, NodeId, NodeId, NodeId, NodeId) {
    let mut doc = Document::parse("");
    let parent = doc.create_document_fragment();
    let original = text(&mut doc, &[97, 98]);
    let right = text(&mut doc, &[33]);
    doc.append_child(parent, original);
    doc.append_child(parent, right);
    let fresh = text(&mut doc, &[98]);
    doc.nodes[parent].children = std::mem::take(&mut doc.nodes[parent].children)
        .into_boxed_slice()
        .into_vec();
    if !growth {
        doc.nodes[parent].children.reserve_exact(1);
    }
    assert_eq!(
        doc.nodes[parent].children.len() == doc.nodes[parent].children.capacity(),
        growth
    );
    (doc, parent, original, fresh, right)
}

#[test]
fn insertion_keeps_source_data_and_places_suffix_before_existing_neighbor() {
    for growth in [false, true] {
        let (mut doc, parent, original, fresh, right) = insertion_case(growth);
        let retained = doc.retained_bytes;
        let count = doc.nodes.len();
        doc.insert_fresh_text_after(original, fresh, &mut budget())
            .unwrap();
        assert_eq!(doc.nodes[parent].children, [original, fresh, right]);
        assert_eq!(doc.nodes[fresh].parent, Some(parent));
        assert_eq!(units(&doc, original), [97, 98]);
        assert_eq!(units(&doc, fresh), [98]);
        assert_eq!(doc.nodes.len(), count);
        assert_eq!(doc.retained_bytes, retained);
        // Prefix replacement is a later caller-owned stage, not part of insert.
        assert_eq!(
            doc.whole_text_units_bounded(original, 10, &mut budget()),
            Ok(vec![97, 98, 98, 33])
        );
    }
}

#[test]
fn detached_original_leaves_created_suffix_detached_without_allocating() {
    let mut doc = Document::parse("");
    let original = text(&mut doc, &[0xD800, 120]);
    let fresh = text(&mut doc, &[120]);
    let before = format!("{doc:?}");
    let mut meter = budget();
    doc.insert_fresh_text_after(original, fresh, &mut meter)
        .unwrap();
    assert_eq!(format!("{doc:?}"), before);
    assert_eq!(meter.allocated, 0);
    assert!(meter.steps < STEPS);
}

#[test]
fn insertion_exact_and_one_short_admission_preserves_detached_refusal_prefix() {
    for growth in [false, true] {
        let (mut doc, _, original, fresh, _) = insertion_case(growth);
        let mut measured = budget();
        doc.insert_fresh_text_after(original, fresh, &mut measured)
            .unwrap();
        let work = STEPS - measured.steps;
        let mut limits = vec![
            (work, measured.allocated, true),
            (work - 1, measured.allocated, false),
        ];
        if measured.allocated > 0 {
            limits.push((work, measured.allocated - 1, false));
        }
        for (steps, heap_limit, success) in limits {
            let (mut doc, parent, original, fresh, right) = insertion_case(growth);
            let before = format!("{doc:?}");
            let capacity = doc.nodes[parent].children.capacity();
            let mut meter = DomMutationBudget {
                steps,
                allocated: 29,
                heap_limit: 29 + heap_limit,
            };
            let result = doc.insert_fresh_text_after(original, fresh, &mut meter);
            if success {
                assert_eq!(result, Ok(()));
                assert_eq!(meter.steps, 0);
                assert_eq!(meter.allocated, 29 + measured.allocated);
                assert_eq!(doc.nodes[parent].children, [original, fresh, right]);
            } else {
                assert_eq!(result, Err(DomDataError::LimitExceeded));
                assert_eq!(format!("{doc:?}"), before);
                assert_eq!(doc.nodes[parent].children.capacity(), capacity);
                assert_eq!(doc.nodes[fresh].parent, None);
                assert!(meter.steps < steps || meter.allocated > meter.heap_limit);
            }
        }
    }
}

#[test]
fn every_insufficient_insertion_work_prefix_is_visible_without_link_mutation() {
    let (mut doc, _, original, fresh, _) = insertion_case(true);
    let mut measured = budget();
    doc.insert_fresh_text_after(original, fresh, &mut measured)
        .unwrap();
    for steps in 0..STEPS - measured.steps {
        let (mut doc, _, original, fresh, _) = insertion_case(true);
        let before = format!("{doc:?}");
        let mut meter = budget();
        meter.steps = steps;
        assert_eq!(
            doc.insert_fresh_text_after(original, fresh, &mut meter),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(meter.steps, 0);
        assert_eq!(format!("{doc:?}"), before);
    }
}

#[test]
fn insertion_rejects_invalid_local_ids_identity_membership_and_leaf_links() {
    for scenario in 0..10 {
        let (mut doc, parent, original, fresh, right) = insertion_case(false);
        let mut source = original;
        let mut suffix = fresh;
        match scenario {
            0 => source = usize::MAX,
            1 => suffix = usize::MAX,
            2 => suffix = source,
            3 => doc.nodes[fresh].parent = Some(parent),
            4 => doc.nodes[parent].children.retain(|&id| id != original),
            5 => doc.nodes[parent].children.push(original),
            6 => doc.nodes[parent].children.push(usize::MAX),
            7 => doc.nodes[original].children.push(right),
            8 => doc.nodes[original].parent = Some(right),
            9 => doc.nodes[fresh].children.push(right),
            _ => unreachable!(),
        }
        let before = format!("{doc:?}");
        assert_eq!(
            doc.insert_fresh_text_after(source, suffix, &mut budget()),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(format!("{doc:?}"), before);
    }
}

#[test]
fn bounded_ancestor_cycle_refusal_keeps_the_created_suffix_detached() {
    let (mut doc, parent, original, fresh, _) = insertion_case(false);
    let element = doc.create_element("div");
    doc.nodes[parent].kind = doc.nodes[element].kind.clone();
    doc.nodes[parent].parent = Some(parent);
    let before = format!("{doc:?}");
    let mut meter = budget();
    assert_eq!(
        doc.insert_fresh_text_after(original, fresh, &mut meter),
        Err(DomDataError::LimitExceeded)
    );
    assert!(meter.steps < STEPS);
    assert_eq!(doc.nodes[fresh].parent, None);
    assert_eq!(format!("{doc:?}"), before);
}

#[test]
fn structurally_admitted_document_text_uses_internal_insertion_without_hierarchy_check() {
    let mut doc = Document::parse("");
    let original = text(&mut doc, &[65, 66]);
    doc.append_child(doc.root, original);
    let mut doc =
        Document::from_snapshot(doc.nodes, doc.root, false, DocumentMode::NoQuirks).unwrap();
    let fresh = text(&mut doc, &[66]);
    doc.insert_fresh_text_after(original, fresh, &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[fresh].parent, Some(doc.root));
    let children = &doc.nodes[doc.root].children;
    assert_eq!(&children[children.len() - 2..], &[original, fresh]);
    assert_eq!(
        doc.whole_text_units_bounded(original, 3, &mut budget()),
        Ok(vec![65, 66, 66])
    );
}

fn deep_text(edges: usize) -> (Document, NodeId) {
    let mut doc = Document::parse("");
    // Direct setup models from_snapshot's edge-depth rule rather than raw append.
    let mut parent = doc.root;
    for _ in 1..edges {
        let child = doc.create_element("div");
        doc.nodes[parent].children.push(child);
        doc.nodes[child].parent = Some(parent);
        parent = child;
    }
    let original = text(&mut doc, &[120]);
    doc.nodes[parent].children.push(original);
    doc.nodes[original].parent = Some(parent);
    (doc, original)
}

#[test]
fn insertion_preserves_accepted_maximum_edge_depth_and_refuses_deeper_host_graph() {
    let (doc, original) = deep_text(MAX_DEPTH);
    let mut doc =
        Document::from_snapshot(doc.nodes, doc.root, false, DocumentMode::NoQuirks).unwrap();
    let parent = doc.nodes[original].parent;
    let fresh = text(&mut doc, &[]);
    doc.insert_fresh_text_after(original, fresh, &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[fresh].parent, parent);
    assert!(Document::from_snapshot(doc.nodes, doc.root, false, DocumentMode::NoQuirks).is_ok());
    let (mut doc, original) = deep_text(MAX_DEPTH + 1);
    let fresh = text(&mut doc, &[]);
    let before = format!("{doc:?}");
    assert_eq!(
        doc.insert_fresh_text_after(original, fresh, &mut budget()),
        Err(DomDataError::LimitExceeded)
    );
    assert_eq!(format!("{doc:?}"), before);
}

#[test]
fn template_lists_stay_separate_and_invalid_host_association_refuses() {
    let mut doc = Document::parse("<template></template>");
    let host = doc.query_selector("template").unwrap();
    let fragment = doc.template_contents(host).unwrap();
    let ordinary = text(&mut doc, &[111]);
    let content = text(&mut doc, &[105]);
    doc.append_child(host, ordinary);
    doc.append_child(fragment, content);
    let fresh = text(&mut doc, &[33]);
    doc.insert_fresh_text_after(content, fresh, &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[fragment].children, [content, fresh]);
    assert_eq!(doc.nodes[host].children, [ordinary]);
    assert_eq!(
        doc.whole_text_units_bounded(content, 2, &mut budget()),
        Ok(vec![105, 33])
    );
    assert_eq!(
        doc.whole_text_units_bounded(ordinary, 1, &mut budget()),
        Ok(vec![111])
    );
    if let NodeKind::Element(el) = &mut doc.nodes[host].kind {
        el.template_contents = None;
    }
    let extra = text(&mut doc, &[]);
    let before = format!("{doc:?}");
    assert_eq!(
        doc.insert_fresh_text_after(content, extra, &mut budget()),
        Err(DomDataError::InvalidNode)
    );
    assert_eq!(format!("{doc:?}"), before);
}

#[test]
fn fresh_text_does_not_unfreeze_base_or_invalidate_summary_ids_and_details_state() {
    let mut doc = Document::parse(
        "<head><base href='/assets/'></head><body><details open name=g>lead<summary>label</summary></details></body>",
    );
    doc.initialize_url(url::Url::parse("https://example.test/page").unwrap());
    let details = doc.query_selector("details").unwrap();
    let summary = doc.query_selector("summary").unwrap();
    let original = doc.nodes[details].children[0];
    assert_eq!(doc.first_summary(details), Some(summary));
    let state = format!(
        "{:?}",
        (
            &doc.first_base,
            &doc.details_groups,
            &doc.details_toggles,
            &doc.details_trackers,
            &doc.details_active,
            doc.details_sequence,
            &doc.details_summaries
        )
    );
    let fresh = text(&mut doc, &[100]);
    doc.insert_fresh_text_after(original, fresh, &mut budget())
        .unwrap();
    assert_eq!(doc.nodes[details].children, [original, fresh, summary]);
    assert_eq!(doc.first_summary(details), Some(summary));
    assert_eq!(doc.base_url().as_str(), "https://example.test/assets/");
    assert_eq!(
        format!(
            "{:?}",
            (
                &doc.first_base,
                &doc.details_groups,
                &doc.details_toggles,
                &doc.details_trackers,
                &doc.details_active,
                doc.details_sequence,
                &doc.details_summaries
            )
        ),
        state
    );
}

#[test]
fn whole_text_includes_empty_text_but_stops_at_comment_pi_and_element() {
    let mut doc = Document::parse("");
    let parent = doc.create_document_fragment();
    let a = text(&mut doc, &[97]);
    let empty = text(&mut doc, &[]);
    let b = text(&mut doc, &[98]);
    let comment = doc.create_comment_owned("ignored".into()).unwrap();
    let c = text(&mut doc, &[99]);
    let pi = doc
        .create_processing_instruction_owned("ok".into(), "ignored".into())
        .unwrap();
    let d = text(&mut doc, &[100]);
    let element = doc.create_element("span");
    let nested = text(&mut doc, &[110]);
    doc.append_child(element, nested);
    let e = text(&mut doc, &[101]);
    for child in [a, empty, b, comment, c, pi, d, element, e] {
        doc.append_child(parent, child);
    }
    for id in [a, empty, b] {
        assert_eq!(
            doc.whole_text_units_bounded(id, 2, &mut budget()),
            Ok(vec![97, 98])
        );
    }
    for (id, expected) in [(c, 99), (d, 100), (nested, 110), (e, 101)] {
        assert_eq!(
            doc.whole_text_units_bounded(id, 1, &mut budget()),
            Ok(vec![expected])
        );
    }
}

fn exact_run() -> (Document, NodeId) {
    let mut doc = Document::parse("");
    let parent = doc.create_document_fragment();
    let mut first = None;
    for data in [&[65, 0xD83D][..], &[], &[0xDE80, 0xD800], &[66]] {
        let id = text(&mut doc, data);
        first.get_or_insert(id);
        doc.append_child(parent, id);
    }
    (doc, first.unwrap())
}

#[test]
fn whole_text_joins_exact_units_without_rewriting_storage_or_snapshot_values() {
    let (doc, id) = exact_run();
    let before = format!("{doc:?}");
    assert_eq!(
        doc.whole_text_units_bounded(id, 5, &mut budget()),
        Ok(vec![65, 0xD83D, 0xDE80, 0xD800, 66])
    );
    assert_eq!(format!("{doc:?}"), before);
    assert_eq!(units(&doc, id), [65, 0xD83D]);
    let snapshot =
        Document::from_snapshot(doc.nodes.clone(), doc.root, false, DocumentMode::NoQuirks)
            .unwrap();
    assert_eq!(
        snapshot.whole_text_units_bounded(id, 5, &mut budget()),
        Ok(vec![65, 0xD83D, 0xDE80, 0xD800, 66])
    );
}

#[test]
fn whole_text_exact_limit_and_measured_budget_cuts_never_mutate() {
    let (doc, id) = exact_run();
    let before = format!("{doc:?}");
    let mut measured = budget();
    let expected = doc.whole_text_units_bounded(id, 5, &mut measured).unwrap();
    let work = STEPS - measured.steps;
    for (maximum, steps, heap, success) in [
        (5, work, measured.allocated, true),
        (4, work, measured.allocated, false),
        (5, work - 1, measured.allocated, false),
        (5, work, measured.allocated - 1, false),
    ] {
        let mut meter = DomMutationBudget {
            steps,
            allocated: 0,
            heap_limit: heap,
        };
        let result = doc.whole_text_units_bounded(id, maximum, &mut meter);
        if success {
            assert_eq!(result, Ok(expected.clone()));
            assert_eq!(meter.steps, 0);
        } else {
            assert_eq!(result, Err(DomDataError::LimitExceeded));
        }
        assert_eq!(format!("{doc:?}"), before);
    }
}

#[test]
fn whole_text_cost_does_not_scan_unrelated_arena_or_element_descendants() {
    let (mut doc, id) = exact_run();
    let mut before = budget();
    let expected = doc.whole_text_units_bounded(id, 5, &mut before).unwrap();
    for _ in 0..64 {
        text(&mut doc, &[0xD800, 0xD800, 0xD800]);
    }
    let mut after = budget();
    assert_eq!(
        doc.whole_text_units_bounded(id, 5, &mut after),
        Ok(expected)
    );
    assert_eq!(
        (after.steps, after.allocated),
        (before.steps, before.allocated)
    );
    let empty = text(&mut doc, &[]);
    assert_eq!(
        doc.whole_text_units_bounded(empty, 0, &mut budget()),
        Ok(Vec::new())
    );
}

#[test]
fn whole_text_rejects_invalid_local_receiver_selected_links_and_boundary_ids() {
    for scenario in 0..7 {
        let (mut doc, parent, original, _, right) = insertion_case(false);
        let mut source = original;
        match scenario {
            0 => source = usize::MAX,
            1 => source = parent,
            2 => doc.nodes[parent].children.retain(|&id| id != original),
            3 => doc.nodes[parent].children.push(original),
            4 => doc.nodes[right].parent = None,
            5 => doc.nodes[parent].children[1] = usize::MAX,
            6 => doc.nodes[right].children.push(original),
            _ => unreachable!(),
        }
        let before = format!("{doc:?}");
        assert_eq!(
            doc.whole_text_units_bounded(source, 100, &mut budget()),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(format!("{doc:?}"), before);
    }
}
