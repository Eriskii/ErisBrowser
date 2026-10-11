//! Frozen ownership and admission cases; no candidate observations.
use super::*;
use crate::cssom::InlineStyle;
use std::sync::Arc;

const PENDING: &str = "--m:1px 2px 3px 4px;margin:var(--m)!important";
const PARTIAL: &str = "--m: 1px 2px 3px 4px; margin-top:  !important; margin-right:  !important; margin-bottom:  !important; margin-left: 9px;";
fn budget() -> DomMutationBudget {
    DomMutationBudget {
        steps: 20_000_000,
        allocated: 0,
        heap_limit: 64 * 1024 * 1024,
    }
}
fn prepared(source: &str, partial: bool) -> crate::cssom::PreparedInlineStyle {
    let mut work = 20_000_000;
    let mut style = InlineStyle::parse(source, &mut work).unwrap();
    if partial {
        assert!(
            style
                .set_property("margin-left", "9px", "", &mut work)
                .unwrap()
        );
    }
    style.prepare_commit(&mut budget()).unwrap()
}
fn fresh() -> (Document, NodeId) {
    let doc = Document::parse("<div></div>");
    let node = doc.query_selector("div").unwrap();
    (doc, node)
}
fn install(doc: &mut Document, node: NodeId, source: &str, partial: bool) {
    doc.commit_inline_style_checked(node, prepared(source, partial), &mut budget())
        .unwrap();
}
fn typed(doc: &Document, node: NodeId) -> &InlineStyle {
    doc.inline_style(node, &mut 20_000_000).unwrap().unwrap()
}
fn pair(style: &InlineStyle, name: &str) -> (String, String) {
    let mut work = 20_000_000;
    (
        style.get_property_value(name, &mut work).unwrap(),
        style
            .get_property_priority(name, &mut work)
            .unwrap()
            .to_owned(),
    )
}
fn map_charge() -> usize {
    // Independent selected fixed full-node envelope; no allocator measurement.
    16 * (std::mem::size_of::<NodeId>() + std::mem::size_of::<InlineStyleEntry>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}
fn sum_entries(doc: &Document) -> usize {
    doc.inline_styles
        .values()
        .map(|entry| entry.retained_cost)
        .sum()
}

#[test]
fn typed_pending_survives_lossy_text_and_a_later_typed_edit() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    assert_eq!(doc.attr(node, "style"), Some(PARTIAL));
    assert_eq!(
        pair(typed(&doc, node), "margin-top"),
        (String::new(), "important".into())
    );
    assert_eq!(
        pair(typed(&doc, node), "margin-left"),
        ("9px".into(), String::new())
    );
    let mut staged = typed(&doc, node).try_clone_for_edit(&mut budget()).unwrap();
    assert!(
        staged
            .set_property("--m", "5px 6px 7px 8px", "", &mut 20_000_000)
            .unwrap()
    );
    doc.commit_inline_style_checked(
        node,
        staged.prepare_commit(&mut budget()).unwrap(),
        &mut budget(),
    )
    .unwrap();
    let top = typed(&doc, node)
        .declarations()
        .find(|d| d.name == "margin-top")
        .unwrap();
    assert_eq!(top.value, "var(--m)");
    assert_eq!(top.pending_shorthand.map(|s| s.as_ref()), Some("margin"));
    assert!(top.important);
    assert_eq!(
        pair(typed(&doc, node), "margin-left"),
        ("9px".into(), String::new())
    );
}

#[test]
fn equal_text_external_write_invalidates_and_same_text_cssom_replacement_reparses() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    let text = doc.attr(node, "style").unwrap().to_owned();
    let released = doc.inline_style_bytes;
    let retained = doc.retained_bytes();
    doc.set_attr(node, "style", &text);
    assert_eq!(doc.attr(node, "style"), Some(PARTIAL));
    assert!(doc.inline_style(node, &mut 20_000_000).unwrap().is_none());
    assert_eq!(doc.inline_style_bytes, 0);
    assert_eq!(doc.retained_bytes(), retained - released);
    let parsed = InlineStyle::parse(&text, &mut 20_000_000).unwrap();
    assert_eq!(pair(&parsed, "margin-top"), (String::new(), String::new()));
    install(&mut doc, node, PENDING, true);
    let source = doc.attr(node, "style").unwrap().to_owned();
    doc.commit_inline_style_checked(node, prepared(&source, false), &mut budget())
        .unwrap();
    assert_eq!(
        doc.attr(node, "style"),
        Some("--m: 1px 2px 3px 4px; margin-left: 9px;")
    );
    assert_eq!(
        pair(typed(&doc, node), "margin-top"),
        (String::new(), String::new())
    );
}

#[test]
fn element_case_and_attribute_namespace_do_not_alias_style() {
    let (mut doc, html) = fresh();
    install(&mut doc, html, PENDING, true);
    doc.set_attr(html, "STYLE", "color:blue");
    assert!(doc.inline_style(html, &mut 20_000_000).unwrap().is_none());
    let svg = doc.create_element_ns(Namespace::Svg, "svg");
    install(&mut doc, svg, PENDING, true);
    let kept = Arc::clone(&doc.inline_styles[&svg].block);
    doc.set_attr(svg, "STYLE", "color:blue");
    doc.set_attr_ns(svg, AttributeNamespace::XLink, "xlink:href", "#target");
    assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&svg].block));
    assert_eq!(doc.attr(svg, "style"), Some(PARTIAL));
    doc.set_attr(svg, "style", "color:blue");
    assert!(doc.inline_style(svg, &mut 20_000_000).unwrap().is_none());
}

#[test]
fn detach_reparent_and_parent_text_replacement_keep_the_node_state() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    let kept = Arc::clone(&doc.inline_styles[&node].block);
    let original = doc.nodes[node].parent.unwrap();
    doc.remove_child(original, node);
    assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&node].block));
    let other = doc.create_element("section");
    doc.append_child(original, other);
    doc.append_child(other, node);
    assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&node].block));
    doc.set_text_content(other, "replacement");
    assert_eq!(doc.nodes[node].parent, None);
    assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&node].block));
}

#[test]
fn clone_node_copies_attributes_but_semantic_document_clone_keeps_typed_state() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    for deep in [false, true] {
        let copy = doc.clone_node_checked(node, deep, &mut budget()).unwrap();
        assert_eq!(doc.attr(copy, "style"), Some(PARTIAL));
        assert!(doc.inline_style(copy, &mut 20_000_000).unwrap().is_none());
        assert_eq!(
            pair(
                &InlineStyle::parse(doc.attr(copy, "style").unwrap(), &mut 20_000_000).unwrap(),
                "margin-top"
            ),
            (String::new(), String::new())
        );
    }
    let mut copy = doc.clone();
    assert_eq!(copy.inline_style_bytes, doc.inline_style_bytes);
    assert_eq!(copy.retained_bytes(), doc.retained_bytes());
    assert!(Arc::ptr_eq(
        &copy.inline_styles[&node].block,
        &doc.inline_styles[&node].block
    ));
    copy.set_attr(node, "style", "color:blue");
    assert!(copy.inline_styles.is_empty());
    assert_eq!(
        pair(typed(&doc, node), "margin-top"),
        (String::new(), "important".into())
    );
}

#[test]
fn ui_projection_has_no_typed_map_and_subtracts_only_its_cached_retention() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    let kept = Arc::clone(&doc.inline_styles[&node].block);
    let strong = Arc::strong_count(&kept);
    let view = doc.clone_for_ui_snapshot();
    assert!(view.inline_styles.is_empty());
    assert_eq!(view.inline_style_bytes, 0);
    assert_eq!(
        view.retained_bytes(),
        doc.retained_bytes() - doc.inline_style_bytes
    );
    assert_eq!(view.attr(node, "style"), Some(PARTIAL));
    assert_eq!(Arc::strong_count(&kept), strong);
    assert_eq!(
        pair(typed(&doc, node), "margin-top"),
        (String::new(), "important".into())
    );
    let rebuilt = Document::from_snapshot(view.nodes, view.root, false, view.mode).unwrap();
    assert!(rebuilt.inline_styles.is_empty());
    assert_eq!(rebuilt.inline_style_bytes, 0);
}

#[test]
fn removal_and_unrelated_writes_update_only_the_selected_retained_entry() {
    let (mut doc, node) = fresh();
    let second = doc.create_element("p");
    install(&mut doc, node, PENDING, true);
    install(&mut doc, second, "color:red", false);
    let kept = Arc::clone(&doc.inline_styles[&node].block);
    let second_bytes = doc.inline_styles[&second].retained_cost;
    doc.set_attr(node, "title", "unchanged style");
    doc.remove_attr(node, "missing");
    assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&node].block));
    let old_attr = 5 + doc.attr(node, "style").unwrap().len();
    let old_entry = doc.inline_styles[&node].retained_cost;
    let before = doc.retained_bytes();
    // This is also the exact Document API used by Page's author-policy removal.
    doc.remove_attr(node, "style");
    assert_eq!(doc.attr(node, "style"), None);
    assert_eq!(doc.inline_style_bytes, second_bytes);
    assert_eq!(doc.inline_style_bytes, sum_entries(&doc));
    assert_eq!(doc.retained_bytes(), before - old_attr - old_entry);
    assert!(doc.inline_styles.contains_key(&second));
}

#[test]
fn lookup_prepays_literal_empty_one_and_split_boundary_search_work() {
    for (count, fee) in [(0, 8), (1, 10), (11, 30), (12, 32)] {
        let (mut doc, node) = fresh();
        for n in 0..count {
            let target = if n == 0 {
                node
            } else {
                doc.create_element("p")
            };
            install(&mut doc, target, "color:red", false);
        }
        let mut exact = fee;
        assert_eq!(
            doc.inline_style(node, &mut exact).unwrap().is_some(),
            count != 0
        );
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert!(matches!(
            doc.inline_style(node, &mut short),
            Err(DomDataError::LimitExceeded)
        ));
        assert_eq!(short, 0);
    }
}

#[test]
fn fresh_commit_has_literal_106_work_and_exact_metadata_heap_admission() {
    let heap = map_charge()
        + 16 * (2 * std::mem::size_of::<String>())
        + 32 * std::mem::size_of::<usize>()
        + 64
        + 5;
    for (work, limit, success) in [
        (105, heap, false),
        (106, heap - 1, false),
        (106, heap, true),
    ] {
        let (mut doc, node) = fresh();
        let before = doc.retained_bytes();
        let staged = prepared("color:red", false);
        let text = staged.serialized().to_owned();
        let typed_bytes = staged.style().retained_bytes() + map_charge();
        let mut charge = DomMutationBudget {
            steps: work,
            allocated: 0,
            heap_limit: limit,
        };
        let result = doc.commit_inline_style_checked(node, staged, &mut charge);
        assert_eq!(result.is_ok(), success);
        if success {
            assert_eq!(charge.steps, 0);
            assert_eq!(charge.allocated, heap);
            assert_eq!(doc.attr(node, "style"), Some(text.as_str()));
            assert_eq!(doc.inline_style_bytes, typed_bytes);
            assert_eq!(doc.retained_bytes(), before + 5 + text.len() + typed_bytes);
        } else {
            assert!(matches!(result, Err(DomDataError::LimitExceeded)));
            assert_eq!(doc.attr(node, "style"), None);
            assert!(doc.inline_styles.is_empty());
            assert_eq!(doc.inline_style_bytes, 0);
            assert_eq!(doc.retained_bytes(), before);
        }
    }
}

#[test]
fn retained_exact_and_one_short_refusal_preserve_attribute_and_typed_identity() {
    for spare in [false, true] {
        let (mut doc, node) = fresh();
        install(&mut doc, node, PENDING, true);
        let kept = Arc::clone(&doc.inline_styles[&node].block);
        let old_text = doc.attr(node, "style").unwrap().to_owned();
        let old_charge = doc.inline_styles[&node].retained_cost;
        let source = format!("--extra:{};color:red", "x".repeat(2048));
        let replacement = prepared(&source, false);
        let new_charge = replacement.style().retained_bytes() + map_charge();
        let text = replacement.serialized().to_owned();
        let old_total = 5 + old_text.len() + old_charge;
        let new_total = 5 + text.len() + new_charge;
        assert!(new_total > old_total);
        // Exact succeeds; one occupied byte more must refuse atomically.
        doc.retained_bytes = MAX_DOM_BYTES + usize::from(!spare) - new_total + old_total;
        assert!(doc.retained_bytes <= MAX_DOM_BYTES);
        let before = doc.retained_bytes();
        let result = doc.commit_inline_style_checked(node, replacement, &mut budget());
        if spare {
            assert!(result.is_ok());
            assert_eq!(doc.retained_bytes(), MAX_DOM_BYTES);
            assert_eq!(doc.attr(node, "style"), Some(text.as_str()));
        } else {
            assert!(matches!(result, Err(DomDataError::LimitExceeded)));
            assert_eq!(doc.retained_bytes(), before);
            assert_eq!(doc.attr(node, "style"), Some(old_text.as_str()));
            assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&node].block));
            assert_eq!(doc.inline_style_bytes, old_charge);
        }
        assert_eq!(doc.inline_style_bytes, sum_entries(&doc));
    }
}

#[test]
fn relevant_invalidation_short_work_preserves_identity_and_exact_work_discards_it() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    let kept = Arc::clone(&doc.inline_styles[&node].block);
    let text = doc.attr(node, "style").unwrap().to_owned();
    let fee = 558 + typed(&doc, node).release_work();
    let retained = doc.retained_bytes();
    let mut short = fee - 1;
    assert!(matches!(
        doc.prepay_inline_style_invalidation(node, "style", None, &mut short),
        Err(DomDataError::LimitExceeded)
    ));
    assert!(Arc::ptr_eq(&kept, &doc.inline_styles[&node].block));
    assert_eq!(doc.retained_bytes(), retained);
    assert_eq!(doc.attr(node, "style"), Some(text.as_str()));
    let mut exact = fee;
    doc.prepay_inline_style_invalidation(node, "STYLE", None, &mut exact)
        .unwrap();
    assert_eq!(exact, 0);
    doc.set_attr(node, "STYLE", &text);
    assert!(doc.inline_styles.is_empty());
    assert_eq!(doc.inline_style_bytes, 0);
}

#[test]
fn external_prefix_truncation_can_reuse_released_typed_bytes() {
    let (mut doc, node) = fresh();
    install(&mut doc, node, PENDING, true);
    let old_len = doc.attr(node, "style").unwrap().len();
    let credit = doc.inline_style_bytes;
    doc.retained_bytes = MAX_DOM_BYTES - 3;
    let capacity = old_len + credit + 3;
    let replacement = "x".repeat(capacity + 2);
    doc.set_attr(node, "style", &replacement);
    assert_eq!(doc.attr(node, "style").unwrap().len(), capacity);
    assert!(doc.inline_styles.is_empty());
    assert_eq!(doc.inline_style_bytes, 0);
    assert_eq!(doc.retained_bytes(), MAX_DOM_BYTES);
}

#[test]
fn repeated_replacement_and_removal_keep_cached_total_equal_to_entries() {
    let (mut doc, node) = fresh();
    let second = doc.create_element("p");
    let baseline = doc.retained_bytes();
    for round in 0..8 {
        install(
            &mut doc,
            node,
            if round % 2 == 0 { PENDING } else { "color:red" },
            round % 2 == 0,
        );
        install(&mut doc, second, "padding:1px 2px 3px 4px", false);
        assert_eq!(doc.inline_style_bytes, sum_entries(&doc));
        doc.remove_attr(second, "style");
        assert_eq!(doc.inline_style_bytes, sum_entries(&doc));
    }
    doc.remove_attr(node, "style");
    assert!(doc.inline_styles.is_empty());
    assert_eq!(doc.inline_style_bytes, 0);
    assert_eq!(doc.retained_bytes(), baseline);
}

#[test]
fn attribute_slot_limit_and_invalid_owner_refuse_without_publication() {
    let (mut doc, node) = fresh();
    for index in 0..1024 {
        doc.set_attr(node, &format!("a{index}"), "");
    }
    let before = doc.retained_bytes();
    let result = doc.commit_inline_style_checked(node, prepared("color:red", false), &mut budget());
    assert!(matches!(result, Err(DomDataError::LimitExceeded)));
    assert_eq!(doc.retained_bytes(), before);
    assert!(doc.inline_styles.is_empty());
    assert_eq!(doc.attr(node, "style"), None);
    for owner in [doc.root, usize::MAX] {
        assert!(matches!(
            doc.commit_inline_style_checked(owner, prepared("color:red", false), &mut budget()),
            Err(DomDataError::InvalidNode)
        ));
        assert_eq!(doc.retained_bytes(), before);
        assert_eq!(doc.inline_style_bytes, 0);
    }
}

// Additive absent-owner invalidation fee: literal counts, no observations.
#[test]
fn absent_typed_owner_prepays_both_lookup_and_external_credit_searches() {
    for (count, fee) in [(0, 8), (1, 12), (11, 52), (12, 56)] {
        let (mut doc, node) = fresh();
        for _ in 0..count {
            let other = doc.create_element("p");
            install(&mut doc, other, "color:red", false);
        }
        let retained = doc.retained_bytes();
        let typed_bytes = doc.inline_style_bytes;
        let mut short = fee - 1;
        assert!(matches!(
            doc.prepay_inline_style_invalidation(node, "style", None, &mut short),
            Err(DomDataError::LimitExceeded)
        ));
        assert_eq!(short, 0);
        assert_eq!(doc.attr(node, "style"), None);
        assert_eq!(doc.retained_bytes(), retained);
        assert_eq!(doc.inline_style_bytes, typed_bytes);
        assert_eq!(doc.inline_styles.len(), count);
        let mut exact = fee;
        doc.prepay_inline_style_invalidation(node, "style", None, &mut exact)
            .unwrap();
        assert_eq!(exact, 0);
        doc.set_attr(node, "style", "color:blue");
        assert_eq!(doc.attr(node, "style"), Some("color:blue"));
        assert_eq!(doc.retained_bytes(), retained + 15);
        assert_eq!(doc.inline_style_bytes, typed_bytes);
        assert_eq!(doc.inline_styles.len(), count);
        assert!(!doc.inline_styles.contains_key(&node));
        assert_eq!(doc.inline_style_bytes, sum_entries(&doc));
    }
}
