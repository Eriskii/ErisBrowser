use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/node-position.js");

fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse("<!doctype html><html><head></head><body></body></html>"),
    )
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn state(doc: &Document) -> (String, usize, Vec<usize>) {
    (
        format!("{doc:?}"),
        doc.nodes.capacity(),
        doc.nodes.iter().map(|n| n.children.capacity()).collect(),
    )
}
fn compare(runtime: &mut Runtime, doc: &Document, a: NodeId, b: NodeId) -> Result<Value> {
    runtime.node_compare_document_position(Value::Node(a), &[Value::Node(b)], doc)
}
fn execute(runtime: &mut Runtime, doc: &mut Document, source: &str, strict: bool) -> Result<Value> {
    let result = if strict {
        runtime.execute_strict(source, doc)
    } else {
        runtime.execute(source, doc)
    };
    clean(runtime);
    result
}
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nnodePositionCases.{name}({strict});");
        assert_eq!(
            execute(&mut runtime, &mut doc, &source, strict).unwrap(),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata_and_cached_method,
    authentic_receiver_brand_without_public_properties,
    required_nonnullable_argument_without_coercion,
    identity_document_alias_and_structural_equality,
    ancestor_masks_at_different_depths,
    preorder_uses_child_positions_not_creation_order,
    moving_existing_siblings_changes_current_order,
    disconnected_masks_reversal_and_stable_existing_nodes,
    fragments_splice_children_but_keep_detached_identity,
    template_content_and_ordinary_children_have_separate_roots,
    namespace_visibility_and_exact_data_do_not_decide_position,
    private_links_ignore_public_fields_and_extra_values,
    argument_expressions_finish_before_read_and_throw_prefix_survives,
    own_shadows_prototype_replacement_deletion_and_saved_calls,
    authentic_alternate_prototype_retains_node_brand,
);

#[test]
fn prior_position_inventory_after_removing_configurable_clone() {
    // Restoration changes creation order, so each mode uses a disposable realm.
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!(
            r#"{CASES}
        (function(){{
          const saved=Object.getOwnPropertyDescriptor(Node.prototype,'cloneNode');
          if(!saved||typeof saved.value!=='function'||!saved.writable||!saved.enumerable||!saved.configurable)
            throw new Error('clone descriptor');
          try{{
            if(!delete Node.prototype.cloneNode)throw new Error('clone delete');
            if(nodePositionCases.represented_complete_key_order({strict})!==true)throw new Error('prior position inventory');
          }}finally{{Object.defineProperty(Node.prototype,'cloneNode',saved);}}
          const restored=Object.getOwnPropertyDescriptor(Node.prototype,'cloneNode');
          if(restored.value!==saved.value||restored.writable!==saved.writable||
             restored.enumerable!==saved.enumerable||restored.configurable!==saved.configurable)
            throw new Error('clone restoration');
          return true;
        }})()"#
        );
        assert_eq!(
            execute(&mut runtime, &mut doc, &source, strict).unwrap(),
            Value::Bool(true)
        );
    }
}

#[test]
fn position_bootstrap_reports_actual_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_POSITION_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
        runtime.steps,
        runtime.allocated,
        runtime.objects.len(),
        runtime.objects.capacity(),
        runtime.native_properties.len(),
        runtime.prototypes.len()
    );
    assert_eq!(runtime.objects.len(), dom_prototypes::BOOTSTRAP_OBJECTS);
    assert!(runtime.steps > 0 && runtime.allocated < MAX_HEAP);
    clean(&runtime);
}

fn link(doc: &mut Document, parent: NodeId, child: NodeId) {
    assert_eq!(doc.nodes[child].parent, None);
    doc.nodes[parent].children.push(child);
    doc.nodes[child].parent = Some(parent);
}
fn siblings(doc: &mut Document, width: usize) -> (NodeId, NodeId, NodeId) {
    assert!(width >= 2);
    let parent = doc.create_element("div");
    let a = doc.create_text_node("a");
    let b = doc.create_text_node("b");
    link(doc, parent, a);
    for _ in 2..width {
        let child = doc.create_comment("");
        link(doc, parent, child);
    }
    link(doc, parent, b);
    assert_eq!(doc.nodes[parent].children.len(), width);
    (parent, a, b)
}

#[test]
fn brand_arity_and_nonnullable_argument_debits_precede_tree_work() {
    let (mut runtime, mut doc) = fresh();
    let leaf = doc.create_text_node("");
    let original = state(&doc);
    for (receiver, args, work) in [
        (Value::Null, vec![Value::Node(leaf)], 16),
        (Value::Node(usize::MAX), vec![], 16),
        (Value::Node(leaf), vec![], 20),
        (Value::Node(leaf), vec![Value::Null], 32),
        (Value::Node(leaf), vec![Value::Undefined], 32),
        (Value::Node(leaf), vec![Value::Bool(false)], 32),
        (Value::Node(leaf), vec![Value::Node(doc.nodes.len())], 32),
    ] {
        for available in [work - 1, work] {
            runtime.steps = available;
            runtime.allocated = MAX_HEAP;
            let error = runtime
                .node_compare_document_position(receiver.clone(), &args, &doc)
                .unwrap_err();
            if available == work {
                assert_eq!(error.name(), "TypeError");
            } else {
                assert!(error.is_resource_limit());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
            clean(&runtime);
        }
    }
}

#[test]
fn self_checks_local_shape_and_canonical_document_without_ancestor_audit() {
    let (mut runtime, mut doc) = fresh();
    let leaf = doc.create_text_node("");
    let parent = doc.create_element("div");
    // No parent walk follows a valid self comparison; this is not a global validator.
    doc.nodes[parent].parent = Some(usize::MAX);
    for (receiver, argument) in [
        (Value::Node(parent), Value::Node(parent)),
        (Value::Node(leaf), Value::Node(leaf)),
        (Value::Document, Value::Node(doc.root)),
        (Value::Node(doc.root), Value::Document),
    ] {
        for available in [65, 66] {
            runtime.steps = available;
            runtime.allocated = MAX_HEAP;
            let result = runtime.node_compare_document_position(
                receiver.clone(),
                std::slice::from_ref(&argument),
                &doc,
            );
            if available == 66 {
                assert_eq!(result.unwrap(), Value::Number(0.0));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        }
    }
    for case in 0..5 {
        let (_, mut doc) = fresh();
        let id = match case {
            0 => {
                doc.nodes[doc.root].kind = NodeKind::Text("false root".into());
                doc.nodes[doc.root].children.clear();
                doc.root
            }
            1 => {
                let id = doc.create_element("div");
                doc.nodes[id].kind = NodeKind::Document;
                id
            }
            2 => {
                let id = doc.create_document_fragment();
                doc.nodes[id].parent = Some(doc.root);
                id
            }
            3 => {
                let id = doc.create_text_node("");
                doc.nodes[id].children.push(doc.root);
                id
            }
            _ => {
                doc.nodes[doc.root].parent = Some(usize::MAX);
                doc.root
            }
        };
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            compare(&mut runtime, &doc, id, id).unwrap_err().name(),
            "TypeError"
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(state(&doc), original);
    }
    clean(&runtime);
}

#[test]
fn mixed_depth_literal_matrices_cover_all_pairs_before_and_after_reordering() {
    // Copied from root/literal-order-matrices.json, independently reviewed by layout.
    const BEFORE: [[u8; 8]; 8] = [
        [0, 20, 20, 20, 20, 20, 20, 20],
        [10, 0, 20, 20, 20, 4, 4, 4],
        [10, 10, 0, 4, 4, 4, 4, 4],
        [10, 10, 2, 0, 20, 4, 4, 4],
        [10, 10, 2, 10, 0, 4, 4, 4],
        [10, 2, 2, 2, 2, 0, 20, 4],
        [10, 2, 2, 2, 2, 10, 0, 4],
        [10, 2, 2, 2, 2, 2, 2, 0],
    ];
    const AFTER: [[u8; 8]; 8] = [
        [0, 20, 20, 20, 20, 20, 20, 20],
        [10, 0, 20, 20, 20, 2, 2, 2],
        [10, 10, 0, 4, 4, 2, 2, 2],
        [10, 10, 2, 0, 20, 2, 2, 2],
        [10, 10, 2, 10, 0, 2, 2, 2],
        [10, 4, 4, 4, 4, 0, 20, 2],
        [10, 4, 4, 4, 4, 10, 0, 2],
        [10, 4, 4, 4, 4, 4, 4, 0],
    ];
    let (mut runtime, mut doc) = fresh();
    let ids: [NodeId; 8] = std::array::from_fn(|_| doc.create_element("div"));
    for (p, c) in [(0, 1), (1, 2), (1, 3), (3, 4), (0, 5), (5, 6), (0, 7)] {
        link(&mut doc, ids[p], ids[c]);
    }
    runtime.allocated = MAX_HEAP;
    for (phase, matrix) in [BEFORE, AFTER].iter().enumerate() {
        if phase == 1 {
            doc.nodes[ids[0]].children = vec![ids[7], ids[5], ids[1]];
        }
        let original = state(&doc);
        for (i, &a) in ids.iter().enumerate() {
            for (j, &b) in ids.iter().enumerate() {
                runtime.steps = MAX_STEPS;
                assert_eq!(
                    compare(&mut runtime, &doc, a, b).unwrap(),
                    Value::Number(f64::from(matrix[i][j])),
                    "phase={phase}, pair={i}/{j}"
                );
            }
        }
        assert_eq!(state(&doc), original);
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    clean(&runtime);
}

#[test]
fn small_work_cuts_leave_tree_capacities_and_heap_unchanged() {
    for geometry in 0..4 {
        let (mut runtime, mut doc) = fresh();
        let (parent, a, b) = siblings(&mut doc, 2);
        let (a, b, expected) = match geometry {
            0 => {
                let other = doc.create_text_node("");
                (b, other, 37.0)
            }
            1 => (a, b, 4.0),
            2 => (parent, b, 20.0),
            _ => (a, a, 0.0),
        };
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            compare(&mut runtime, &doc, a, b).unwrap(),
            Value::Number(expected)
        );
        let work = MAX_STEPS - runtime.steps;
        assert!(work < 500);
        for available in 0..=work {
            runtime.steps = available;
            let result = compare(&mut runtime, &doc, a, b);
            if available == work {
                assert_eq!(result.unwrap(), Value::Number(expected));
            } else {
                assert!(
                    result.unwrap_err().is_resource_limit(),
                    "geometry={geometry},cut={available}"
                );
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), original);
            clean(&runtime);
        }
    }
}

#[test]
fn both_complete_paths_validate_before_disconnected_order_and_bound_cycles() {
    for case in 0..11 {
        let (mut runtime, mut doc) = fresh();
        let p = doc.create_element("div");
        let leaf = doc.create_text_node("");
        let other = doc.create_text_node("");
        link(&mut doc, p, leaf);
        match case {
            0 => doc.nodes[p].children.clear(),
            1 => doc.nodes[p].children.push(leaf),
            2 => doc.nodes[leaf].parent = Some(usize::MAX),
            3 => {
                let wrong = doc.create_comment("");
                doc.nodes[leaf].parent = Some(wrong);
            }
            4 => {
                doc.nodes[p].parent = Some(doc.root);
                doc.nodes[doc.root].children.push(p);
                doc.nodes[doc.root].kind = NodeKind::Element(match doc.nodes[p].kind.clone() {
                    NodeKind::Element(e) => e,
                    _ => unreachable!(),
                });
            }
            5 => doc.nodes[p].kind = NodeKind::Document,
            6 => {
                doc.nodes[p].kind = NodeKind::DocumentFragment { host: None };
                doc.nodes[p].parent = Some(doc.root);
            }
            7 => doc.nodes[leaf].children.push(other),
            8 => {
                doc.nodes[p].parent = Some(p);
                doc.nodes[p].children.push(p);
            }
            9 => {
                let q = doc.create_element("div");
                link(&mut doc, q, p);
                link(&mut doc, p, q);
            }
            _ => {
                // The false canonical root is intermediate, not an endpoint.
                // An endpoint-only check could incorrectly return 35/37 here.
                let root = doc.root;
                let outer = doc.create_element("section");
                doc.nodes[root].kind = doc.nodes[outer].kind.clone();
                link(&mut doc, root, p);
                link(&mut doc, outer, root);
            }
        }
        let original = state(&doc);
        for (a, b) in [(other, leaf), (leaf, other)] {
            runtime.steps = MAX_STEPS;
            runtime.allocated = MAX_HEAP;
            assert_eq!(
                compare(&mut runtime, &doc, a, b).unwrap_err().name(),
                "TypeError",
                "case={case},a={a}"
            );
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(state(&doc), original);
            clean(&runtime);
        }
    }
    // An unrelated ID is compared but not dereferenced during selected membership.
    let (mut runtime, mut doc) = fresh();
    let (p, a, b) = siblings(&mut doc, 2);
    doc.nodes[p].children.insert(1, usize::MAX);
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        compare(&mut runtime, &doc, a, b).unwrap(),
        Value::Number(4.0)
    );
    assert_eq!(runtime.allocated, MAX_HEAP);
}

#[test]
fn disconnected_direction_uses_original_operand_ids_across_root_moves() {
    let (mut runtime, mut doc) = fresh();
    let left = doc.create_element("div");
    let right = doc.create_element("div");
    let a = doc.create_text_node("a");
    let b = doc.create_text_node("b");
    // Opposite root order makes a root-ID tie-break observably wrong.
    link(&mut doc, right, a);
    link(&mut doc, left, b);
    for moved in [false, true] {
        if moved {
            doc.nodes[right].children.clear();
            doc.nodes[a].parent = None;
            let root = doc.root;
            link(&mut doc, root, a);
        }
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert!(a < b && left < right);
        assert_eq!(
            compare(&mut runtime, &doc, a, b).unwrap(),
            Value::Number(37.0)
        );
        assert_eq!(
            compare(&mut runtime, &doc, b, a).unwrap(),
            Value::Number(35.0)
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(state(&doc), original);
    }
    clean(&runtime);
}

fn chain(doc: &mut Document, root: NodeId, depth: usize) -> Vec<NodeId> {
    let mut ids = vec![root];
    for _ in 0..depth {
        let id = doc.create_element("span");
        link(doc, *ids.last().unwrap(), id);
        ids.push(id);
    }
    let mut cursor = *ids.last().unwrap();
    let mut edges = 0;
    while let Some(parent) = doc.nodes[cursor].parent {
        assert!(doc.nodes[parent].children.contains(&cursor));
        cursor = parent;
        edges += 1;
    }
    assert_eq!((cursor, edges), (root, depth));
    ids
}
fn validated(doc: Document) -> Document {
    Document::from_snapshot(
        doc.nodes,
        doc.root,
        false,
        crate::dom::DocumentMode::NoQuirks,
    )
    .unwrap()
}

#[test]
fn six_geometries_measure_exact_and_one_short_work_with_256_edge_admission() {
    for geometry in 0..6 {
        let (mut runtime, mut doc) = fresh();
        let (a, b, expected, reverse, predicted) = match geometry {
            0 => {
                let a = doc.create_text_node("");
                let b = doc.create_text_node("");
                (a, b, 37.0, 35.0, 118)
            }
            1 => {
                let (_, a, b) = siblings(&mut doc, 2);
                (a, b, 4.0, 2.0, 249)
            }
            2 | 3 => {
                let root = doc.create_element("div");
                let ids = chain(&mut doc, root, 256);
                if geometry == 2 {
                    (ids[0], ids[256], 20.0, 10.0, 11138)
                } else {
                    (ids[255], ids[256], 20.0, 10.0, 17003)
                }
            }
            4 => {
                let ra = doc.create_element("div");
                let a = chain(&mut doc, ra, 256)[256];
                let rb = doc.create_element("div");
                let b = chain(&mut doc, rb, 256)[256];
                (a, b, 37.0, 35.0, 17014)
            }
            _ => {
                let root = doc.create_element("div");
                let a = chain(&mut doc, root, 256)[256];
                let b = chain(&mut doc, root, 256)[256];
                assert_eq!(doc.nodes[root].children.len(), 2);
                (a, b, 4.0, 2.0, 22179)
            }
        };
        let doc = validated(doc);
        let original = state(&doc);
        for (a, b, expected) in [(a, b, expected), (b, a, reverse)] {
            runtime.steps = MAX_STEPS;
            runtime.allocated = MAX_HEAP;
            assert_eq!(
                compare(&mut runtime, &doc, a, b).unwrap(),
                Value::Number(expected)
            );
            let work = MAX_STEPS - runtime.steps;
            assert_eq!(work, predicted, "geometry={geometry}");
            for available in [work - 1, work] {
                runtime.steps = available;
                let result = compare(&mut runtime, &doc, a, b);
                if available == work {
                    assert_eq!(result.unwrap(), Value::Number(expected));
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                }
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                assert_eq!(state(&doc), original);
                clean(&runtime);
            }
        }
    }
}

#[test]
fn a_257th_host_edge_is_rejected_on_either_complete_walk() {
    let (mut runtime, mut doc) = fresh();
    let root = doc.create_element("div");
    let ids = chain(&mut doc, root, 256);
    let other = doc.create_text_node("separate");
    let mut doc = validated(doc);
    let extra = doc.create_text_node("overdeep");
    link(&mut doc, ids[256], extra);
    let mut cursor = extra;
    let mut depth = 0;
    while let Some(parent) = doc.nodes[cursor].parent {
        cursor = parent;
        depth += 1;
    }
    assert_eq!((cursor, depth), (root, 257));
    let original = state(&doc);
    for (a, b) in [(other, extra), (extra, other)] {
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            compare(&mut runtime, &doc, a, b).unwrap_err().name(),
            "TypeError"
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn final_sibling_scan_and_result_have_separate_exact_admission() {
    let (mut runtime, mut doc) = fresh();
    let width = 1024;
    let (p, a, b) = siblings(&mut doc, width);
    let original = state(&doc);
    let prefix = 208 + 8 * width;
    let scan = 1 + 6 * width;
    let work = 221 + 14 * width;
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        compare(&mut runtime, &doc, a, b).unwrap(),
        Value::Number(4.0)
    );
    assert_eq!(MAX_STEPS - runtime.steps, work);
    runtime.steps = MAX_STEPS;
    assert_eq!(
        runtime.position_branch_order(p, a, b, &doc).unwrap(),
        Value::Number(4.0)
    );
    assert_eq!(MAX_STEPS - runtime.steps, 13 + 6 * width);
    for available in [
        prefix + 8 + scan - 1,
        work - 4,
        work - 3,
        work - 2,
        work - 1,
        work,
    ] {
        runtime.steps = available;
        let result = compare(&mut runtime, &doc, a, b);
        if available == work {
            assert_eq!(result.unwrap(), Value::Number(4.0));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
    // Direct helper invalid-input coverage does not add a production injection seam.
    for duplicate in [false, true] {
        let (_, mut doc) = fresh();
        let (p, a, b) = siblings(&mut doc, 2);
        if duplicate {
            doc.nodes[p].children.push(a);
        } else {
            doc.nodes[p].children.pop();
        }
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .position_branch_order(p, a, b, &doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
}

#[test]
fn all_represented_leaf_kinds_and_empty_document_use_identity_and_parent_links() {
    let (mut runtime, mut doc) = fresh();
    let text = doc.create_text_node("text");
    let comment = doc.create_comment("comment");
    let pi = doc.create_processing_instruction("target", "data");
    let doctype = doc
        .nodes
        .iter()
        .position(|n| matches!(n.kind, NodeKind::Doctype(_)))
        .unwrap();
    let root = doc.root;
    for id in [text, comment, pi] {
        link(&mut doc, root, id);
    }
    runtime.allocated = MAX_HEAP;
    for id in [root, doctype, text, comment, pi] {
        runtime.steps = MAX_STEPS;
        assert_eq!(
            compare(&mut runtime, &doc, id, id).unwrap(),
            Value::Number(0.0)
        );
        if id != root {
            assert_eq!(
                compare(&mut runtime, &doc, root, id).unwrap(),
                Value::Number(20.0)
            );
            assert_eq!(
                compare(&mut runtime, &doc, id, root).unwrap(),
                Value::Number(10.0)
            );
        }
    }
    for id in doc.nodes[root].children.clone() {
        doc.nodes[id].parent = None;
    }
    doc.nodes[root].children.clear();
    runtime.steps = 66;
    assert_eq!(
        runtime
            .node_compare_document_position(Value::Document, &[Value::Node(root)], &doc)
            .unwrap(),
        Value::Number(0.0)
    );
    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
    clean(&runtime);
}

#[test]
fn unrelated_arena_and_unvisited_payloads_do_not_change_body_cost() {
    let mut costs = Vec::new();
    for noise in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let (_, a, b) = siblings(&mut doc, 2);
        if noise {
            for _ in 0..1000 {
                doc.create_comment("");
            }
            doc.create_comment(&"x".repeat(MAX_STRING + 1));
        }
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            compare(&mut runtime, &doc, a, b).unwrap(),
            Value::Number(4.0)
        );
        costs.push((MAX_STEPS - runtime.steps, runtime.allocated));
        clean(&runtime);
    }
    assert_eq!(costs, vec![(249, MAX_HEAP); 2]);
}

#[test]
fn scoped_dispatch_is_paid_without_repricing_existing_routes() {
    let (mut runtime, mut doc) = fresh();
    let leaf = doc.create_text_node("");
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
        .node_data_call_preflight("DOM.Node.compareDocumentPosition")
        .unwrap();
    assert_eq!((MAX_STEPS - runtime.steps, runtime.allocated), (36, 64));
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert_eq!(
        runtime
            .node_data_native(
                "compareDocumentPosition",
                Value::Node(leaf),
                &[Value::Node(leaf)],
                &mut doc
            )
            .unwrap(),
        Value::Number(0.0)
    );
    assert_eq!(
        (MAX_STEPS - runtime.steps, runtime.allocated),
        (24 + 66, MAX_HEAP)
    );
    for (name, expected) in [
        ("getRootNode", 12 + 16),
        ("isEqualNode", 23 + 16),
        ("hasChildNodes", 14 + 16),
        ("normalize", 10 + 16),
        ("isSameNode", 11 + 16),
        ("contains", 9 + 16),
        ("getIsConnected", 24 + 16),
        ("getNodeValue", 9),
        ("setNodeValue", 9),
        ("getTextContent", 9),
        ("setTextContent", 9),
    ] {
        runtime.steps = MAX_STEPS;
        assert_eq!(
            runtime
                .node_data_native(name, Value::Null, &[], &mut doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(MAX_STEPS - runtime.steps, expected, "{name}");
    }
    clean(&runtime);
}

#[test]
fn saved_native_exact_and_one_short_boundaries_preserve_metadata_and_cleanup() {
    fn ready() -> (Runtime, Document, NodeId, NodeId, Value) {
        let (mut runtime, mut doc) = fresh();
        let (_, a, b) = siblings(&mut doc, 2);
        let method = runtime
            .execute("Node.prototype.compareDocumentPosition", &mut doc)
            .unwrap();
        (runtime, doc, a, b, method)
    }
    let (mut measured, mut doc, a, b, method) = ready();
    let before = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .call(method, vec![Value::Node(b)], Value::Node(a), &mut doc)
            .unwrap(),
        Value::Number(4.0)
    );
    let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
    assert!(heap >= 64);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (8, heap, false),
    ] {
        let (mut runtime, mut doc, a, b, method) = ready();
        let original = state(&doc);
        let bag = runtime.property_object(&method).unwrap();
        let metadata = format!("{:?}", runtime.objects[bag].values);
        let objects = (runtime.objects.len(), runtime.objects.capacity());
        let Value::Native(native) = &method else {
            panic!("native")
        };
        let saved = native.clone();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(
            method.clone(),
            vec![Value::Node(b)],
            Value::Node(a),
            &mut doc,
        );
        if success {
            assert_eq!(result.unwrap(), Value::Number(4.0));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        let Value::Native(native) = &method else {
            panic!("native")
        };
        assert!(Rc::ptr_eq(native, &saved));
        assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
        assert_eq!((runtime.objects.len(), runtime.objects.capacity()), objects);
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

const TERMINAL_SETUP: &str = "var a,b,effect=0,caught=false,finalized=false,completed=false;var compare=Node.prototype.compareDocumentPosition;";
const TERMINAL_CALL: &str =
    "try{compare.call(a,b,effect=1);completed=true;}catch(e){caught=true;}finally{finalized=true;}";

fn bind_pair(runtime: &mut Runtime, a: NodeId, b: NodeId) {
    for (name, id) in [("a", a), ("b", b)] {
        runtime.environments[0]
            .bindings
            .get_mut(name)
            .unwrap()
            .value = Value::Node(id);
    }
}
fn terminal_flags(runtime: &Runtime) {
    assert_eq!(
        runtime.environments[0].bindings["effect"].value,
        Value::Number(1.0)
    );
    for name in ["caught", "finalized", "completed"] {
        assert_eq!(
            runtime.environments[0].bindings[name].value,
            Value::Bool(false),
            "{name}"
        );
    }
    clean(runtime);
}

#[test]
fn first_second_and_final_scan_refusals_share_terminal_execution_budget() {
    for strict in [false, true] {
        // Match the complete public program to overbound its non-body prefix.
        let (mut baseline, mut small) = fresh();
        let (_, sa, sb) = siblings(&mut small, 2);
        execute(&mut baseline, &mut small, TERMINAL_SETUP, strict).unwrap();
        bind_pair(&mut baseline, sa, sb);
        execute(&mut baseline, &mut small, TERMINAL_CALL, strict).unwrap();
        let overhead = (MAX_STEPS - baseline.steps).checked_sub(249).unwrap();
        assert!(overhead >= 60);
        for stage in 0..3 {
            let (mut runtime, mut doc) = fresh();
            let width = if stage == 2 { 8000 } else { 26000 };
            let (parent, first, last) = siblings(&mut doc, width);
            let (a, b) = if stage == 2 {
                (first, last)
            } else {
                let other = doc.create_text_node("separate");
                if stage == 0 {
                    (first, other)
                } else {
                    (other, first)
                }
            };
            if stage == 2 {
                runtime.steps = MAX_STEPS;
                assert_eq!(
                    runtime.position_root_and_depth(a, &doc).unwrap(),
                    (parent, 1)
                );
                assert_eq!(
                    runtime.position_root_and_depth(b, &doc).unwrap(),
                    (parent, 1)
                );
                assert_eq!(MAX_STEPS - runtime.steps, 64106);
                // Setup/phase bookkeeping plus BOTH walks fit, including an
                // overbound for the complete small program's non-body work.
                assert!(64208 + 8 + overhead < MAX_STEPS);
                assert!(112221 + overhead > MAX_STEPS);
                runtime.steps = MAX_STEPS - 64208;
                assert!(
                    runtime
                        .position_branch_order(parent, a, b, &doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(runtime.steps, 0);
            } else {
                assert!(1 + 4 * width > MAX_STEPS);
                runtime.steps = MAX_STEPS;
                if stage == 1 {
                    assert_eq!(runtime.position_root_and_depth(a, &doc).unwrap(), (a, 0));
                    assert_eq!(MAX_STEPS - runtime.steps, 24);
                }
                assert!(
                    runtime
                        .position_root_and_depth(first, &doc)
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(runtime.steps, 0);
            }
            execute(&mut runtime, &mut doc, TERMINAL_SETUP, strict).unwrap();
            bind_pair(&mut runtime, a, b);
            let original = state(&doc);
            assert!(
                execute(&mut runtime, &mut doc, TERMINAL_CALL, strict)
                    .unwrap_err()
                    .is_resource_limit(),
                "stage={stage}"
            );
            assert_eq!(runtime.steps, 0);
            terminal_flags(&runtime);
            assert_eq!(state(&doc), original);
        }
    }
}

#[test]
fn exhaustion_in_an_ignored_extra_expression_precedes_native_invocation() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let a = doc.create_text_node("same");
        execute(&mut runtime, &mut doc, TERMINAL_SETUP, strict).unwrap();
        bind_pair(&mut runtime, a, a);
        let original = state(&doc);
        let source = "function exhaust(){effect=1;while(true){}}try{compare.call(a,b,exhaust());completed=true;}catch(e){caught=true;}finally{finalized=true;}";
        assert!(
            execute(&mut runtime, &mut doc, source, strict)
                .unwrap_err()
                .is_resource_limit()
        );
        terminal_flags(&runtime);
        assert_eq!(state(&doc), original);
    }
}

#[test]
fn two_live_realms_keep_separate_cached_method_metadata() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    a.execute("var saved=Node.prototype.compareDocumentPosition;Object.defineProperty(saved,'name',{value:'A'});",&mut da).unwrap();
    assert_eq!(
        b.execute("Node.prototype.compareDocumentPosition.name", &mut db)
            .unwrap(),
        Value::String("compareDocumentPosition".into())
    );
    b.execute("var saved=Node.prototype.compareDocumentPosition;Object.defineProperty(saved,'name',{value:'B'});",&mut db).unwrap();
    for (runtime, doc, label) in [(&mut a, &mut da, "A"), (&mut b, &mut db, "B")] {
        assert_eq!(
            runtime
                .execute("saved===Node.prototype.compareDocumentPosition", doc)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            runtime.execute("saved.name", doc).unwrap(),
            Value::String(label.into())
        );
        clean(runtime);
    }
}
