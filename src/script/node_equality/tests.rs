use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/node-equality.js");

fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse("<html><head></head><body></body></html>"),
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
fn equal(runtime: &mut Runtime, doc: &Document, left: NodeId, right: NodeId) -> Result<Value> {
    runtime.node_is_equal_node(Value::Node(left), &[Value::Node(right)], doc)
}
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nnodeEqualityCases.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(
            result.unwrap(),
            Value::Bool(true),
            "{name}, strict={strict}"
        );
        clean(&runtime);
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata_and_inherited_identity,
    required_nullable_argument_without_coercion,
    authentic_receiver_and_argument_brands,
    exact_character_data_and_distinct_kinds,
    processing_instruction_target_and_mutated_data,
    element_namespaces_local_names_and_case,
    unordered_attributes_and_exact_scalar_values,
    foreign_attribute_namespace_side_map_matters,
    child_order_and_text_segmentation_are_structural,
    template_content_and_host_are_not_ordinary_children,
    connection_parent_and_document_identity_are_separate,
    live_changes_and_argument_expression_order,
    saved_method_shadow_replacement_and_deletion,
    internal_fields_ignore_public_shadows,
    authentic_alternate_prototype_does_not_change_equality,
);

#[test]
fn prior_equality_inventory_after_removing_later_configurable_members() {
    // Restoration changes creation order, so each mode uses a disposable realm.
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!(
            r#"{CASES}
        (function(){{
          const saved=Object.getOwnPropertyDescriptor(Node.prototype,'isConnected');
          if(!saved||typeof saved.get!=='function'||saved.set!==undefined||
             !saved.enumerable||!saved.configurable)throw new Error('connection descriptor');
          const position=Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
          if(!position||!position.configurable||typeof position.value!=='function')throw new Error('position descriptor');
          try{{
            if(!delete Node.prototype.compareDocumentPosition)throw new Error('position delete');
            if(!delete Node.prototype.isConnected)throw new Error('connection delete');
            if(nodeEqualityCases.represented_complete_key_order()!==true)throw new Error('prior equality inventory');
          }}finally{{
            Object.defineProperty(Node.prototype,'compareDocumentPosition',position);
            Object.defineProperty(Node.prototype,'isConnected',saved);}}
          const d=Object.getOwnPropertyDescriptor(Node.prototype,'isConnected');
          if(d.get!==saved.get||d.set!==undefined||d.enumerable!==saved.enumerable||
             d.configurable!==saved.configurable)throw new Error('connection restoration');
          const restoredPosition=Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition');
          if(restoredPosition.value!==position.value||restoredPosition.writable!==position.writable||
             restoredPosition.enumerable!==position.enumerable||restoredPosition.configurable!==position.configurable)
            throw new Error('position restoration');
          return true;
        }})()"#
        );
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        clean(&runtime);
    }
}

#[test]
fn equality_bootstrap_reports_actual_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NODE_EQUALITY_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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

#[test]
fn leaf_null_identity_and_mismatch_work_cuts_own_no_body_heap() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_text_node("same");
    let b = doc.create_text_node("same");
    let c = doc.create_comment("same");
    let before = state(&doc);
    for (receiver, args, expected) in [
        (Value::Node(a), vec![Value::Null], false),
        (Value::Document, vec![Value::Node(doc.root)], true),
        (Value::Node(a), vec![Value::Node(a)], true),
        (Value::Node(a), vec![Value::Node(b)], true),
        (Value::Node(a), vec![Value::Node(c)], false),
    ] {
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            runtime
                .node_is_equal_node(receiver.clone(), &args, &doc)
                .unwrap(),
            Value::Bool(expected)
        );
        let work = MAX_STEPS - runtime.steps;
        assert!(work > 0 && work < 200);
        for cut in 0..=work {
            runtime.steps = cut;
            runtime.allocated = MAX_HEAP;
            let result = runtime.node_is_equal_node(receiver.clone(), &args, &doc);
            if cut == work {
                assert_eq!(result.unwrap(), Value::Bool(expected));
            } else {
                assert!(result.unwrap_err().is_resource_limit(), "cut={cut}");
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            assert_eq!(state(&doc), before);
            clean(&runtime);
        }
    }
}

fn pair() -> (Runtime, Document, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let a = doc.create_document_fragment();
    let b = doc.create_document_fragment();
    for parent in [a, b] {
        let child = doc.create_element("div");
        let text = doc.create_text_node("same");
        doc.set_attr(child, "data-k", "value");
        doc.append_child(parent, child);
        doc.append_child(child, text);
    }
    (runtime, doc, a, b)
}

#[test]
fn paired_tree_measured_work_cuts_and_heap_boundary_preserve_every_dom_capacity() {
    let (mut runtime, doc, a, b) = pair();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(true));
    let (work, heap) = (MAX_STEPS - runtime.steps, runtime.allocated);
    assert!(heap > 0 && work < 5000);
    let before = state(&doc);
    for cut in 0..=work {
        runtime.steps = cut;
        runtime.allocated = MAX_HEAP - heap;
        let result = equal(&mut runtime, &doc, a, b);
        if cut == work {
            assert_eq!(result.unwrap(), Value::Bool(true));
            assert_eq!(runtime.allocated, MAX_HEAP);
        } else {
            assert!(result.unwrap_err().is_resource_limit(), "cut={cut}");
        }
        assert_eq!(runtime.steps, 0);
        assert_eq!(state(&doc), before);
        clean(&runtime);
    }
    runtime.steps = work;
    runtime.allocated = MAX_HEAP - heap + 1;
    assert!(
        equal(&mut runtime, &doc, a, b)
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(runtime.allocated > MAX_HEAP);
    assert_eq!(state(&doc), before);
}

#[test]
fn sparse_page_vacancy_occupied_duplicate_and_full_page_count_boundaries() {
    let (mut runtime, _) = fresh();
    for count in [0, 10, 11, 80] {
        let seed: BTreeMap<usize, u64> = (0..count).map(|key| (key, 1)).collect();
        let id = count * PAGE_BITS;
        let mut pages = seed.clone();
        runtime.steps = MAX_STEPS;
        runtime.allocated = 0;
        runtime.equality_mark(&mut pages, id).unwrap();
        let (work, heap) = (MAX_STEPS - runtime.steps, runtime.allocated);
        assert_eq!(pages.len(), count + 1);
        assert_eq!(heap == 0, count == 10);
        for (steps, bytes, success) in [(work, heap, true), (work - 1, heap, false)] {
            let mut pages = seed.clone();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.equality_mark(&mut pages, id);
            if success {
                result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(pages, seed);
            }
        }
        if heap > 0 {
            let mut pages = seed.clone();
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - heap + 1;
            assert!(
                runtime
                    .equality_mark(&mut pages, id)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(pages, seed);
            assert!(runtime.allocated > MAX_HEAP);
        }
    }
    let mut left = BTreeMap::new();
    let mut right = BTreeMap::new();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime.equality_mark(&mut left, 7).unwrap();
    runtime.equality_mark(&mut right, 7).unwrap();
    let heap = runtime.allocated;
    runtime.equality_mark(&mut left, 8).unwrap();
    assert_eq!(runtime.allocated, heap);
    let before = left.clone();
    assert_eq!(
        runtime.equality_mark(&mut left, 7).unwrap_err().name(),
        "TypeError"
    );
    assert_eq!(left, before);
    assert_ne!(left, right);
    let mut all: BTreeMap<usize, u64> = (0..MAX_PAGES).map(|page| (page, 1)).collect();
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    runtime
        .equality_mark(&mut all, crate::dom::MAX_NODES - 1)
        .unwrap();
    assert_eq!(all.len(), MAX_PAGES);
    assert_eq!(runtime.allocated, MAX_HEAP);
    let before = all.clone();
    assert_eq!(
        runtime
            .equality_mark(&mut all, crate::dom::MAX_NODES)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    assert_eq!(all, before);
}

#[test]
fn cursor_actual_growth_and_reuse_cutpoints_precede_push() {
    let (mut runtime, _) = fresh();
    let frame = EqualityFrame {
        left: 1,
        right: 2,
        next: 0,
    };
    for length in [0, 1, 8] {
        let mut measured = Vec::with_capacity(length);
        measured.resize(length, frame);
        runtime.steps = MAX_STEPS;
        runtime.allocated = 0;
        runtime.equality_push(&mut measured, frame).unwrap();
        let (work, heap) = (MAX_STEPS - runtime.steps, runtime.allocated);
        assert!(heap > 0);
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let mut frames = Vec::with_capacity(length);
            frames.resize(length, frame);
            let capacity = frames.capacity();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.equality_push(&mut frames, frame);
            if success {
                result.unwrap();
                assert_eq!(frames.len(), length + 1);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!((frames.len(), frames.capacity()), (length, capacity));
            }
        }
    }
    let mut frames = Vec::with_capacity(2);
    frames.push(frame);
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    runtime.equality_push(&mut frames, frame).unwrap();
    assert_eq!(runtime.allocated, MAX_HEAP);
}

#[test]
fn doctype_ids_and_stored_processing_instruction_data_use_literal_fields() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_doctype(crate::dom::Doctype {
        name: "html".into(),
        public_id: None,
        system_id: None,
        force_quirks: false,
    });
    let b = doc.create_doctype(crate::dom::Doctype {
        name: "html".into(),
        public_id: Some("".into()),
        system_id: Some("".into()),
        force_quirks: true,
    });
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(true));
    for field in 0..3 {
        let old = doc.nodes[b].kind.clone();
        let NodeKind::Doctype(d) = &mut doc.nodes[b].kind else {
            unreachable!()
        };
        match field {
            0 => d.name = "HTML".into(),
            1 => d.public_id = Some("id".into()),
            _ => d.system_id = Some("id".into()),
        }
        assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
        doc.nodes[b].kind = old;
    }
    let pi = doc
        .create_processing_instruction_owned("Build".into(), "".into())
        .unwrap();
    let peer = doc
        .create_processing_instruction_owned("Build".into(), "".into())
        .unwrap();
    for id in [pi, peer] {
        doc.replace_character_data(
            id,
            DomString::from_units_owned(vec![63, 62, 0xd800]).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(
        equal(&mut runtime, &doc, pi, peer).unwrap(),
        Value::Bool(true)
    );
    if let NodeKind::ProcessingInstruction { target, .. } = &mut doc.nodes[peer].kind {
        *target = "build".into();
    }
    assert_eq!(
        equal(&mut runtime, &doc, pi, peer).unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn canonical_payloads_and_host_strings_above_js_limit_need_no_output_allocation() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_text_node("");
    let b = doc.create_text_node("");
    for (left, right, expected) in [
        (vec![0xd800], vec![0xd800], true),
        (vec![0xd800], vec![0xfffd], false),
        (vec![0xd800, 0xdc00], vec![0xd800, 0xdc00], true),
        (vec![65, 0xd800], vec![65, 0xdc00], false),
    ] {
        doc.replace_character_data(a, DomString::from_units_owned(left).unwrap())
            .unwrap();
        doc.replace_character_data(b, DomString::from_units_owned(right).unwrap())
            .unwrap();
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert_eq!(
            equal(&mut runtime, &doc, a, b).unwrap(),
            Value::Bool(expected)
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
    doc.replace_character_data(a, "x".repeat(MAX_STRING + 1).into())
        .unwrap();
    doc.replace_character_data(b, "x".repeat(MAX_STRING + 2).into())
        .unwrap();
    runtime.steps = MAX_STEPS;
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
    doc.replace_character_data(b, "x".repeat(MAX_STRING + 1).into())
        .unwrap();
    let original = state(&doc);
    runtime.steps = MAX_STEPS;
    assert!(
        equal(&mut runtime, &doc, a, b)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(state(&doc), original);
}

#[test]
fn attribute_iteration_crosses_tree_nodes_and_checks_namespace_annotations() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_element_ns(crate::dom::Namespace::Svg, "a");
    let b = doc.create_element_ns(crate::dom::Namespace::Svg, "a");
    for i in 0..25 {
        doc.set_attr(a, &format!("data-{i:02}"), "same");
    }
    for i in (0..25).rev() {
        doc.set_attr(b, &format!("data-{i:02}"), "same");
    }
    for key in [
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
        for id in [a, b] {
            doc.set_attr_ns(
                id,
                AttributeNamespace::from_qualified_name(key).unwrap(),
                key,
                "v",
            );
        }
    }
    runtime.steps = MAX_STEPS;
    runtime.allocated = MAX_HEAP;
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(true));
    assert_eq!(runtime.allocated, MAX_HEAP);
    doc.set_attr(b, "data-24", "last");
    runtime.steps = MAX_STEPS;
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
    doc.set_attr(b, "data-24", "same");
    doc.remove_attr(b, "xlink:href");
    doc.set_attr(b, "xlink:href", "v");
    runtime.steps = MAX_STEPS;
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
}

#[test]
fn consumed_malformed_namespace_metadata_refuses_but_unreached_tail_is_not_audit() {
    for case in 0..4 {
        let (mut runtime, mut doc) = fresh();
        let a = doc.create_element("div");
        let b = doc.create_element("div");
        for id in [a, b] {
            doc.set_attr(id, "a", "same");
            doc.set_attr(id, "xlink:href", "v");
            let NodeKind::Element(el) = &mut doc.nodes[id].kind else {
                unreachable!()
            };
            let (name, ns) = match case {
                0 => ("xlink:href", AttributeNamespace::Xml),
                1 => ("absent", AttributeNamespace::XLink),
                2 => ("zz", AttributeNamespace::XLink),
                _ => ("a", AttributeNamespace::XLink),
            };
            el.attr_namespaces.insert(name.into(), ns);
        }
        let original = state(&doc);
        assert_eq!(
            equal(&mut runtime, &doc, a, b).unwrap_err().name(),
            "TypeError",
            "case={case}"
        );
        assert_eq!(state(&doc), original);
        doc.set_attr(b, "a", "different");
        runtime.steps = MAX_STEPS;
        // Malformed metadata ahead of or on a is consumed before its value;
        // only a later dangling/incorrect annotation is safely unreached.
        if case < 3 {
            assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
        }
    }
}

#[test]
fn reached_duplicates_backlinks_cycles_and_wrong_child_kinds_refuse() {
    for case in 0..10 {
        let (mut runtime, mut doc) = fresh();
        let a = doc.create_element("div");
        let b = doc.create_element("div");
        let x = doc.create_element("span");
        let y = doc.create_element("span");
        doc.append_child(a, x);
        doc.append_child(b, y);
        match case {
            0 => doc.nodes[a].children[0] = usize::MAX,
            1 => doc.nodes[x].parent = None,
            2 => {
                doc.nodes[a].children.push(x);
                let z = doc.create_element("span");
                doc.append_child(b, z);
            }
            3 => {
                doc.nodes[b].children.push(y);
                let z = doc.create_element("span");
                doc.append_child(a, z);
            }
            4 => {
                doc.nodes[x].children.push(a);
                doc.nodes[a].parent = Some(x);
                doc.nodes[y].children.push(b);
                doc.nodes[b].parent = Some(y);
            }
            5 => doc.nodes[x].kind = NodeKind::Document,
            6 => doc.nodes[x].kind = NodeKind::DocumentFragment { host: None },
            7 => {
                doc.nodes[x].kind = NodeKind::Doctype(crate::dom::Doctype {
                    name: "html".into(),
                    public_id: None,
                    system_id: None,
                    force_quirks: false,
                })
            }
            8 => {
                doc.nodes[x].kind = NodeKind::Text("leaf".into());
                doc.nodes[x].children.push(a);
            }
            9 => doc.nodes[y].parent = Some(a),
            _ => unreachable!(),
        }
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        assert_eq!(
            equal(&mut runtime, &doc, a, b).unwrap_err().name(),
            "TypeError",
            "case={case}"
        );
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn local_self_fastpath_and_reached_mismatch_do_not_certify_unvisited_graph() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_element("div");
    let b = doc.create_element("span");
    doc.nodes[a].children.push(usize::MAX);
    doc.nodes[b].children.push(usize::MAX);
    runtime.allocated = MAX_HEAP;
    assert_eq!(equal(&mut runtime, &doc, a, a).unwrap(), Value::Bool(true));
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
    assert_eq!(runtime.allocated, MAX_HEAP);
    runtime.allocated = 0;
    doc.nodes[b].kind = doc.nodes[a].kind.clone();
    assert_eq!(
        equal(&mut runtime, &doc, a, b).unwrap_err().name(),
        "TypeError"
    );
    runtime.allocated = MAX_HEAP;
    let leaf = doc.create_text_node("x");
    doc.nodes[leaf].children.push(a);
    assert_eq!(
        equal(&mut runtime, &doc, leaf, leaf).unwrap_err().name(),
        "TypeError"
    );
    let root = doc.root;
    doc.nodes[root].children.clear();
    doc.nodes[root].kind = NodeKind::Text("false root".into());
    assert_eq!(
        equal(&mut runtime, &doc, root, root).unwrap_err().name(),
        "TypeError"
    );
    assert_eq!(runtime.allocated, MAX_HEAP);
    clean(&runtime);
}

#[test]
fn ancestor_operands_can_overlap_across_sides_without_false_duplicate_error() {
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_element("div");
    let b = doc.create_element("div");
    let leaf = doc.create_text_node("x");
    doc.append_child(a, b);
    doc.append_child(b, leaf);
    let original = state(&doc);
    // B is the right root and later a left-side child. Separate reached sets
    // permit it; the child-count/kind mismatch returns an ordinary false.
    assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(false));
    assert_eq!(equal(&mut runtime, &doc, b, a).unwrap(), Value::Bool(false));
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

fn deep_pair(layout: usize) -> (Document, NodeId, NodeId) {
    let mut doc = Document::parse("");
    let mut left = Vec::new();
    let mut right = Vec::new();
    let start = doc.nodes.len();
    let make = |doc: &mut Document, index: usize| {
        if index == 0 {
            doc.create_document_fragment()
        } else if index == crate::dom::MAX_DEPTH {
            doc.create_text_node("x")
        } else {
            doc.create_element("div")
        }
    };
    if layout == 0 {
        for side in [&mut left, &mut right] {
            for index in 0..=crate::dom::MAX_DEPTH {
                side.push(make(&mut doc, index));
            }
        }
    } else {
        for index in 0..=crate::dom::MAX_DEPTH {
            if layout == 2 {
                while doc.nodes.len() < start + 128 * index {
                    doc.create_comment("");
                }
            }
            left.push(make(&mut doc, index));
            if layout == 2 {
                while doc.nodes.len() < start + 128 * index + 64 {
                    doc.create_comment("");
                }
            }
            right.push(make(&mut doc, index));
        }
    }
    for side in [&left, &right] {
        for edge in side.windows(2) {
            doc.nodes[edge[0]].children.push(edge[1]);
            doc.nodes[edge[1]].parent = Some(edge[0]);
        }
        let mut current = *side.last().unwrap();
        let mut depth = 0;
        while let Some(parent) = doc.nodes[current].parent {
            depth += 1;
            current = parent;
        }
        assert_eq!((current, depth), (side[0], crate::dom::MAX_DEPTH));
    }
    let doc = Document::from_snapshot(
        doc.nodes,
        doc.root,
        false,
        crate::dom::DocumentMode::NoQuirks,
    )
    .unwrap();
    (doc, left[0], right[0])
}

#[test]
fn accepted_dense_256_edge_pairs_fit_with_measured_exact_and_short_boundaries() {
    for layout in [0, 1] {
        let (mut runtime, _) = fresh();
        let (doc, a, b) = deep_pair(layout);
        let original = state(&doc);
        runtime.steps = MAX_STEPS;
        runtime.allocated = 0;
        assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(true));
        let (work, heap) = (MAX_STEPS - runtime.steps, runtime.allocated);
        println!("NODE_EQUALITY_DEPTH layout={layout} work={work} heap={heap}");
        assert!(work < 75000 && heap < 20000);
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = equal(&mut runtime, &doc, a, b);
            if success {
                assert_eq!(result.unwrap(), Value::Bool(true));
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(state(&doc), original);
            clean(&runtime);
        }
    }
}

#[test]
fn sparse_depth_refuses_fixed_work_and_a_257th_edge_is_invalid() {
    let (mut runtime, _) = fresh();
    let (doc, a, b) = deep_pair(2);
    let original = state(&doc);
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    assert!(
        equal(&mut runtime, &doc, a, b)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
    assert_eq!(state(&doc), original);
    clean(&runtime);
    let (mut doc, a, b) = deep_pair(0);
    for root in [a, b] {
        let mut last = root;
        while let Some(&next) = doc.nodes[last].children.first() {
            last = next;
        }
        let element = doc.create_element("div");
        doc.nodes[last].kind = doc.nodes[element].kind.clone();
        let extra = doc.create_text_node("x");
        doc.nodes[last].children.push(extra);
        doc.nodes[extra].parent = Some(last);
    }
    let original = state(&doc);
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    assert_eq!(
        equal(&mut runtime, &doc, a, b).unwrap_err().name(),
        "TypeError"
    );
    assert_eq!(state(&doc), original);
    clean(&runtime);
}

#[test]
fn unrelated_arena_payload_does_not_add_traversal_or_storage_fees() {
    let mut costs = Vec::new();
    for extra in [false, true] {
        let (mut runtime, mut doc, a, b) = pair();
        if extra {
            for _ in 0..1000 {
                doc.create_comment("");
            }
            doc.create_comment(&"z".repeat(MAX_STRING + 1));
        }
        runtime.steps = MAX_STEPS;
        runtime.allocated = 0;
        assert_eq!(equal(&mut runtime, &doc, a, b).unwrap(), Value::Bool(true));
        costs.push((MAX_STEPS - runtime.steps, runtime.allocated));
    }
    assert_eq!(costs[0], costs[1]);
}

#[test]
fn terminal_equality_budget_preserves_argument_effects_and_suppresses_handlers() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let a = doc.create_text_node(&"x".repeat(26000));
        let b = doc.create_text_node(&"x".repeat(26000));
        runtime
            .execute(
                "var left,right,effect=0,caught=false,finalized=false;",
                &mut doc,
            )
            .unwrap();
        runtime.environments[0]
            .bindings
            .get_mut("left")
            .unwrap()
            .value = Value::Node(a);
        runtime.environments[0]
            .bindings
            .get_mut("right")
            .unwrap()
            .value = Value::Node(b);
        let original = state(&doc);
        let source = "try{left.isEqualNode((effect=1,right));}catch(e){caught=true;}finally{finalized=true;}";
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(
            runtime.environments[0].bindings["effect"].value,
            Value::Number(1.0)
        );
        for name in ["caught", "finalized"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Bool(false)
            );
        }
        assert_eq!(state(&doc), original);
        clean(&runtime);
    }
}

#[test]
fn saved_native_invocation_cuts_preserve_metadata_dom_and_stack_cleanup() {
    fn ready() -> (Runtime, Document, NodeId, NodeId, Value) {
        let (mut runtime, mut doc, a, b) = pair();
        let method = runtime
            .execute("Node.prototype.isEqualNode", &mut doc)
            .unwrap();
        (runtime, doc, a, b, method)
    }
    let (mut measured, mut doc, a, b, method) = ready();
    let before = (measured.steps, measured.allocated);
    assert_eq!(
        measured
            .call(method, vec![Value::Node(b)], Value::Node(a), &mut doc)
            .unwrap(),
        Value::Bool(true)
    );
    let (work, heap) = (before.0 - measured.steps, measured.allocated - before.1);
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
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(method, vec![Value::Node(b)], Value::Node(a), &mut doc);
        if success {
            assert_eq!(result.unwrap(), Value::Bool(true));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(state(&doc), original);
        assert_eq!(format!("{:?}", runtime.objects[bag].values), metadata);
        clean(&runtime);
    }
}

#[test]
fn two_live_realms_have_separate_mutable_equality_metadata() {
    let (mut a, mut da) = fresh();
    let (mut b, mut db) = fresh();
    a.execute(
        "Object.defineProperty(Node.prototype.isEqualNode,'name',{value:'realmA'})",
        &mut da,
    )
    .unwrap();
    assert_eq!(
        b.execute("Node.prototype.isEqualNode.name", &mut db)
            .unwrap(),
        Value::String("isEqualNode".into())
    );
    b.execute(
        "Object.defineProperty(Node.prototype.isEqualNode,'name',{value:'realmB'})",
        &mut db,
    )
    .unwrap();
    assert_eq!(
        a.execute("Node.prototype.isEqualNode.name", &mut da)
            .unwrap(),
        Value::String("realmA".into())
    );
    assert_eq!(
        b.execute("Node.prototype.isEqualNode.name", &mut db)
            .unwrap(),
        Value::String("realmB".into())
    );
    for (runtime, doc) in [(&mut a, &mut da), (&mut b, &mut db)] {
        assert_eq!(
            runtime
                .execute("(new Text('x')).isEqualNode(new Text('x'))", doc)
                .unwrap(),
            Value::Bool(true)
        );
        clean(runtime);
    }
}
