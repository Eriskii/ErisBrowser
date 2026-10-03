use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/node-normalize.js");

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse(""))
}
fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}
fn independent(name: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = format!("{CASES}\nnodeNormalizeCases.{name}();");
        let result = if strict {
            runtime.execute_strict(&source, &mut doc)
        } else {
            runtime.execute(&source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{name} strict={strict}");
        clean(&runtime);
    }
}
macro_rules! cases {($($name:ident),* $(,)?)=>{$(#[test]fn $name(){independent(stringify!($name));})*};}
cases!(
    metadata,
    represented_complete_key_order,
    receiver_is_not_a_descendant,
    only_empty_descendants_are_removed,
    first_nonempty_survives_with_detached_data,
    exact_pair_repair_keeps_removed_units,
    exact_units_do_not_unicode_normalize,
    barriers_and_nested_subtrees,
    detached_fragment_is_idempotent,
    document_descendants_without_detached_arena_scan,
    template_content_is_not_an_ordinary_child,
    extra_arguments_are_not_converted,
    authentic_brands_and_saved_leaf_calls,
    saved_method_survives_replacement_and_deletion,
    internal_slots_ignore_authored_getters,
    removed_nodes_reuse_their_original_data,
);

#[test]
fn normalize_bootstrap_reports_actual_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "NORMALIZE_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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

fn text(doc: &mut Document, values: &[u16]) -> NodeId {
    doc.create_text_node_owned(DomString::from_units_owned(values.to_vec()).unwrap())
        .unwrap()
}
fn units(doc: &Document, id: NodeId) -> Vec<u16> {
    text_data(doc, id).unwrap().units().collect()
}
fn pair(first: &[u16], second: &[u16]) -> (Runtime, Document, NodeId, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let parent = doc.create_document_fragment();
    let a = text(&mut doc, first);
    let b = text(&mut doc, second);
    doc.append_child(parent, a);
    doc.append_child(parent, b);
    doc.nodes.shrink_to_fit();
    doc.nodes[parent].children.shrink_to_fit();
    (runtime, doc, parent, a, b)
}
fn state(doc: &Document, parent: NodeId) -> (String, usize, usize) {
    (
        format!("{doc:?}"),
        doc.nodes.capacity(),
        doc.nodes[parent].children.capacity(),
    )
}

#[test]
fn complete_exact_and_one_short_preserve_real_publication_prefixes() {
    for (a, b, expected, delta) in [
        (vec![65], vec![66], vec![65, 66], 1),
        (vec![0xd83d], vec![0xde80], vec![0xd83d, 0xde80], 2),
        (
            vec![0xd800],
            vec![0x4e00, 0x4e01],
            vec![0xd800, 0x4e00, 0x4e01],
            4,
        ),
    ] {
        let (mut runtime, mut doc, parent, _, _) = pair(&a, &b);
        let before = (runtime.steps, runtime.allocated);
        runtime
            .node_normalize(Value::Node(parent), &mut doc)
            .unwrap();
        let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
        for (steps, bytes, success, work_short) in [
            (work, heap, true, false),
            (work - 1, heap, false, true),
            (work, heap - 1, false, false),
        ] {
            let (mut runtime, mut doc, parent, first, last) = pair(&a, &b);
            let count = doc.nodes.len();
            let retained = doc.retained_bytes();
            let original = state(&doc, parent);
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.node_normalize(Value::Node(parent), &mut doc);
            assert_eq!(doc.nodes.len(), count);
            assert_eq!(doc.nodes.capacity(), original.1);
            assert_eq!(doc.nodes[parent].children.capacity(), original.2);
            assert_eq!(units(&doc, last), b);
            if success || work_short {
                // Last work refusal is traversal completion after the entire
                // mutation, not a fabricated rollback of the tail removal.
                assert_eq!(units(&doc, first), expected);
                assert_eq!(doc.nodes[parent].children, [first]);
                assert_eq!(doc.nodes[last].parent, None);
                assert_eq!(doc.retained_bytes(), retained + delta);
            } else {
                assert_eq!(state(&doc, parent), original);
            }
            if success {
                assert_eq!(result.unwrap(), Value::Undefined);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                if work_short {
                    assert_eq!(runtime.steps, 0);
                } else {
                    assert!(runtime.allocated > MAX_HEAP);
                }
            }
            clean(&runtime);
        }
    }
}

fn stages() -> (Runtime, Document, NodeId, [NodeId; 6]) {
    let (runtime, mut doc, parent, a, b) = pair(&[65, 0xd83d], &[0xde80, 66]);
    let empty = text(&mut doc, &[]);
    let comment = doc.create_comment("barrier");
    let c = text(&mut doc, &[67]);
    let d = text(&mut doc, &[68]);
    // Test-owned topology uses the ordinary raw operation; prepend the empty
    // leaf directly with consistent links before this operation is measured.
    doc.nodes[parent].children.insert(0, empty);
    doc.nodes[empty].parent = Some(parent);
    doc.append_child(parent, comment);
    doc.append_child(parent, c);
    doc.append_child(parent, d);
    (runtime, doc, parent, [empty, a, b, comment, c, d])
}

#[test]
fn every_work_cut_preserves_empty_merge_remove_and_later_run_stages() {
    let (mut runtime, mut doc, parent, _) = stages();
    let before = runtime.steps;
    runtime
        .node_normalize(Value::Node(parent), &mut doc)
        .unwrap();
    let cost = before - runtime.steps;
    let mut seen = [false; 6];
    for steps in 0..cost {
        let (mut runtime, mut doc, parent, [empty, a, b, comment, c, d]) = stages();
        let count = doc.nodes.len();
        let retained = doc.retained_bytes();
        runtime.steps = steps;
        assert!(
            runtime
                .node_normalize(Value::Node(parent), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(doc.nodes.len(), count);
        assert!(units(&doc, empty).is_empty());
        assert_eq!(units(&doc, b), [0xde80, 66]);
        assert_eq!(units(&doc, d), [68]);
        let first = units(&doc, a);
        let second = units(&doc, c);
        let phase = if doc.nodes[empty].parent.is_some() {
            0
        } else if first == [65, 0xd83d] {
            1
        } else if doc.nodes[b].parent.is_some() {
            2
        } else if second == [67] {
            3
        } else if doc.nodes[d].parent.is_some() {
            4
        } else {
            5
        };
        seen[phase] = true;
        let expected_children = match phase {
            0 => vec![empty, a, b, comment, c, d],
            1 | 2 => vec![a, b, comment, c, d],
            3 | 4 => vec![a, comment, c, d],
            5 => vec![a, comment, c],
            _ => unreachable!(),
        };
        assert_eq!(doc.nodes[parent].children, expected_children, "cut {steps}");
        assert_eq!(
            first,
            if phase < 2 {
                vec![65, 0xd83d]
            } else {
                vec![65, 0xd83d, 0xde80, 66]
            }
        );
        assert_eq!(second, if phase < 4 { vec![67] } else { vec![67, 68] });
        assert_eq!(
            doc.retained_bytes(),
            retained + usize::from(phase >= 2) * 2 + usize::from(phase >= 4)
        );
        for id in expected_children {
            assert_eq!(doc.nodes[id].parent, Some(parent));
        }
        clean(&runtime);
    }
    assert!(
        seen.into_iter().all(|value| value),
        "all six literal stages must be reached"
    );
}

#[test]
fn singleton_keeps_identity_but_pays_full_canonical_copy() {
    for input in [vec![65, 0xd83d, 0xde80], vec![0xd800, 120, 0xdc00]] {
        let (mut runtime, mut doc) = fresh();
        let parent = doc.create_document_fragment();
        let id = text(&mut doc, &input);
        doc.append_child(parent, id);
        let retained = doc.retained_bytes();
        let count = doc.nodes.len();
        let pointer = match text_data(&doc, id).unwrap().scalar() {
            Some(value) => value.as_ptr() as usize,
            None => text_data(&doc, id).unwrap().raw_units().unwrap().as_ptr() as usize,
        };
        let before = runtime.allocated;
        runtime
            .node_normalize(Value::Node(parent), &mut doc)
            .unwrap();
        let next = match text_data(&doc, id).unwrap().scalar() {
            Some(value) => value.as_ptr() as usize,
            None => text_data(&doc, id).unwrap().raw_units().unwrap().as_ptr() as usize,
        };
        assert_ne!(
            pointer, next,
            "replacement is allocated while old payload is still borrowed"
        );
        assert_eq!(units(&doc, id), input);
        assert_eq!(doc.nodes[parent].children, [id]);
        assert_eq!((doc.nodes.len(), doc.retained_bytes()), (count, retained));
        assert!(runtime.allocated - before >= text_data(&doc, id).unwrap().stored_bytes());
        clean(&runtime);
    }
}

#[test]
fn current_payload_admission_cannot_borrow_credit_from_detached_tails() {
    let (mut runtime, mut doc, parent, a, b) = pair(&[0xd83d], &[0xde80]);
    while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 1 {
        let amount = (crate::dom::MAX_DOM_BYTES - 1 - doc.retained_bytes()).min(8 * 1024 * 1024);
        doc.create_comment_owned("x".repeat(amount).into()).unwrap();
    }
    let count = doc.nodes.len();
    let capacity = doc.nodes.capacity();
    assert!(
        runtime
            .node_normalize(Value::Node(parent), &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(doc.retained_bytes(), crate::dom::MAX_DOM_BYTES - 1);
    assert_eq!((doc.nodes.len(), doc.nodes.capacity()), (count, capacity));
    assert_eq!(doc.nodes[parent].children, [a, b]);
    assert_eq!(units(&doc, a), [0xd83d]);
    assert_eq!(units(&doc, b), [0xde80]);
    clean(&runtime);
}

#[test]
fn full_node_arena_needs_no_slot_or_growth_and_keeps_detached_data() {
    let (mut runtime, mut doc, parent, a, b) = pair(&[65], &[66]);
    while doc.nodes.len() < crate::dom::MAX_NODES {
        doc.create_comment("");
    }
    doc.nodes.shrink_to_fit();
    let capacity = doc.nodes.capacity();
    runtime
        .node_normalize(Value::Node(parent), &mut doc)
        .unwrap();
    assert_eq!(doc.nodes.len(), crate::dom::MAX_NODES);
    assert_eq!(doc.nodes.capacity(), capacity);
    assert_eq!(doc.nodes[parent].children, [a]);
    assert_eq!(doc.nodes[b].parent, None);
    assert_eq!(units(&doc, a), [65, 66]);
    assert_eq!(units(&doc, b), [66]);
    clean(&runtime);
}

#[test]
fn leaf_and_small_subtree_costs_ignore_unrelated_arena_and_payloads() {
    let mut costs = Vec::new();
    for unrelated in [0, 1000] {
        let (mut runtime, mut doc, parent, leaf, _) = pair(&[65], &[66]);
        for _ in 0..unrelated {
            doc.create_comment("unrelated payload");
        }
        let empty = doc.create_document_fragment();
        for id in [leaf, empty, parent] {
            let before = (runtime.steps, runtime.allocated);
            runtime.node_normalize(Value::Node(id), &mut doc).unwrap();
            costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        }
        clean(&runtime);
    }
    assert_eq!(&costs[..3], &costs[3..]);
    assert_eq!(costs[0].1, 0);
    assert_eq!(costs[1].1, 0);
    assert!(costs[2].1 > 0);
}

#[test]
fn scratch_growth_and_duplicates_have_measured_prepaid_boundaries() {
    for old in [0, 10, 11, 31] {
        let mut witness = Runtime::new();
        let mut reached: BTreeMap<_, _> = (0..old).map(|id| (id, ())).collect();
        let before = (witness.steps, witness.allocated);
        witness.normalize_mark(&mut reached, old).unwrap();
        let (work, heap) = (before.0 - witness.steps, witness.allocated - before.1);
        for (steps, available, success) in [(work, heap, true), (work - 1, heap, false)]
            .into_iter()
            .chain((heap > 0).then_some((work, heap.saturating_sub(1), false)))
        {
            let mut runtime = Runtime::new();
            let mut reached: BTreeMap<_, _> = (0..old).map(|id| (id, ())).collect();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - available;
            let result = runtime.normalize_mark(&mut reached, old);
            assert_eq!(reached.len(), old + usize::from(success));
            if success {
                result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
        }
        let mut runtime = Runtime::new();
        let before = reached.clone();
        assert_eq!(
            runtime
                .normalize_mark(&mut reached, old)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(reached, before);
    }
    for full in [false, true] {
        fn frames(full: bool) -> Vec<NormalizeFrame> {
            let mut frames = vec![
                NormalizeFrame { parent: 1, next: 0 },
                NormalizeFrame { parent: 2, next: 0 },
            ];
            if full {
                frames.shrink_to_fit();
            } else {
                frames.try_reserve_exact(1).unwrap();
            }
            frames
        }
        let mut runtime = Runtime::new();
        let mut measured = frames(full);
        let before = (runtime.steps, runtime.allocated);
        runtime
            .normalize_push(&mut measured, NormalizeFrame { parent: 3, next: 0 })
            .unwrap();
        let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
        for (steps, available, success) in [(work, heap, true), (work - 1, heap, false)]
            .into_iter()
            .chain((heap > 0).then_some((work, heap.saturating_sub(1), false)))
        {
            let mut runtime = Runtime::new();
            let mut cursor = frames(full);
            let capacity = cursor.capacity();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - available;
            let result = runtime.normalize_push(&mut cursor, NormalizeFrame { parent: 3, next: 0 });
            assert_eq!(cursor.len(), 2 + usize::from(success));
            if success {
                result.unwrap();
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(cursor.capacity(), capacity);
            }
        }
    }
}

#[test]
fn malformed_links_duplicates_and_cycles_refuse_without_current_run_publication() {
    for case in 0..6 {
        let (mut runtime, mut doc, parent, a, b) = pair(&[65], &[66]);
        match case {
            0 => doc.nodes[parent].children[1] = usize::MAX,
            1 => doc.nodes[b].parent = None,
            2 => doc.nodes[parent].children.push(b),
            3 => doc.nodes[parent].children[1] = a,
            4 => doc.nodes[a].children.push(b),
            5 => {
                doc.nodes[parent].children.push(parent);
                doc.nodes[parent].parent = Some(parent);
            }
            _ => unreachable!(),
        }
        let before = state(&doc, parent);
        assert_eq!(
            runtime
                .node_normalize(Value::Node(parent), &mut doc)
                .unwrap_err()
                .name(),
            "TypeError",
            "case{case}"
        );
        assert_eq!(state(&doc, parent), before, "case{case}");
        clean(&runtime);
    }
    let (mut runtime, mut doc) = fresh();
    let a = doc.create_element("div");
    let b = doc.create_element("span");
    doc.nodes[a].children.push(b);
    doc.nodes[b].parent = Some(a);
    doc.nodes[b].children.push(a);
    doc.nodes[a].parent = Some(b);
    let before = format!("{doc:?}");
    assert_eq!(
        runtime
            .node_normalize(Value::Node(a), &mut doc)
            .unwrap_err()
            .name(),
        "TypeError"
    );
    assert_eq!(format!("{doc:?}"), before);
    clean(&runtime);
}

#[test]
fn authentic_document_aliases_leaves_invalid_ids_and_alternate_prototypes() {
    for alias in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let a = text(&mut doc, &[65]);
        let b = text(&mut doc, &[66]);
        doc.append_child(doc.root, a);
        doc.append_child(doc.root, b);
        let receiver = if alias {
            Value::Node(doc.root)
        } else {
            Value::Document
        };
        runtime.node_normalize(receiver, &mut doc).unwrap();
        assert_eq!(units(&doc, a), [65, 66]);
        assert_eq!(doc.nodes[b].parent, None);
        clean(&runtime);
    }
    let (mut runtime, mut doc) = fresh();
    for receiver in [
        Value::Node(usize::MAX),
        Value::Node(doc.nodes.len()),
        Value::Null,
        Value::Object(0),
    ] {
        let before = runtime.allocated;
        assert_eq!(
            runtime
                .node_normalize(receiver, &mut doc)
                .unwrap_err()
                .name(),
            "TypeError"
        );
        assert_eq!(runtime.allocated, before);
    }
    assert_eq!(
        runtime
            .execute(
                r#"(function(){
      function Other(){};Other.prototype=Object.create(null);
      var t=Reflect.construct(Text,['\ud800'],Other),f=Node.prototype.normalize;
      if(t instanceof Text||Object.getPrototypeOf(t)!==Other.prototype)throw new Error('override');
      if(f.call(t)!==undefined)throw new Error('saved leaf call');
      var g=Object.getOwnPropertyDescriptor(CharacterData.prototype,'data').get;
      if(g.call(t).charCodeAt(0)!==55296)throw new Error('exact leaf');return true;
    })()"#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    clean(&runtime);
}

#[test]
fn depth_edge_is_admitted_with_real_cursor_growth_and_no_ancestor_walk() {
    let (mut runtime, mut doc) = fresh();
    let root = doc.create_document_fragment();
    let mut parent = root;
    // Snapshot admission uses edge depth. Raw append's separate conservative
    // builder bound would refuse this final edge before normalize was reached.
    for _ in 1..crate::dom::MAX_DEPTH {
        let next = doc.create_element("div");
        doc.nodes[parent].children.push(next);
        doc.nodes[next].parent = Some(parent);
        parent = next;
    }
    let a = text(&mut doc, &[65]);
    let b = text(&mut doc, &[66]);
    for id in [a, b] {
        doc.nodes[parent].children.push(id);
        doc.nodes[id].parent = Some(parent);
    }
    let mut depth = 0;
    let mut cursor = Some(a);
    while let Some(id) = cursor {
        cursor = doc.nodes[id].parent;
        if cursor.is_some() {
            depth += 1;
        }
    }
    assert_eq!(depth, crate::dom::MAX_DEPTH);
    let mut doc = Document::from_snapshot(
        doc.nodes,
        doc.root,
        false,
        crate::dom::DocumentMode::NoQuirks,
    )
    .unwrap();
    assert_eq!(doc.nodes[parent].children, [a, b]);
    runtime.node_normalize(Value::Node(root), &mut doc).unwrap();
    assert_eq!(doc.nodes[parent].children, [a]);
    assert_eq!(units(&doc, a), [65, 66]);
    assert_eq!(doc.nodes[b].parent, None);
    clean(&runtime);
}

#[test]
fn terminal_normalize_refusal_keeps_prefix_and_skips_catch_finally() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let parent = doc.create_document_fragment();
        let mut leaves = Vec::new();
        for _ in 0..2000 {
            let id = text(&mut doc, &[]);
            doc.append_child(parent, id);
            leaves.push(id);
        }
        runtime
            .execute("var target,effect=0,caught=false,finalized=false", &mut doc)
            .unwrap();
        runtime.environments[0]
            .bindings
            .get_mut("target")
            .unwrap()
            .value = Value::Node(parent);
        let source =
            "try{target.normalize(effect=1);}catch(e){caught=true;}finally{finalized=true;}";
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
        assert_eq!(
            runtime.environments[0].bindings["caught"].value,
            Value::Bool(false)
        );
        assert_eq!(
            runtime.environments[0].bindings["finalized"].value,
            Value::Bool(false)
        );
        let remaining = doc.nodes[parent].children.len();
        assert!(remaining > 0 && remaining < leaves.len());
        for (index, id) in leaves.iter().enumerate() {
            assert!(units(&doc, *id).is_empty());
            assert_eq!(
                doc.nodes[*id].parent,
                if index < leaves.len() - remaining {
                    None
                } else {
                    Some(parent)
                }
            );
        }
        clean(&runtime);
    }
}

#[test]
fn saved_method_call_admission_unwinds_temporary_native_name_and_is_realm_local() {
    fn ready() -> (Runtime, Document, Value, Value) {
        let (mut runtime, mut doc) = fresh();
        let method = runtime
            .execute("Node.prototype.normalize", &mut doc)
            .unwrap();
        let receiver = Value::Node(doc.create_text_node("leaf"));
        (runtime, doc, method, receiver)
    }
    let (mut runtime, mut doc, method, receiver) = ready();
    let before = (runtime.steps, runtime.allocated);
    runtime
        .call(method, Vec::new(), receiver, &mut doc)
        .unwrap();
    let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
    for (steps, available, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (8, heap, false),
    ] {
        let (mut runtime, mut doc, method, receiver) = ready();
        let before = format!("{doc:?}");
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - available;
        let result = runtime.call(method, Vec::new(), receiver, &mut doc);
        if success {
            assert_eq!(result.unwrap(), Value::Undefined);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(format!("{doc:?}"), before);
        clean(&runtime);
    }
    let (mut first, mut a) = fresh();
    let (mut second, mut b) = fresh();
    first
        .execute(
            "Object.defineProperty(Node.prototype.normalize,'name',{value:'changed'})",
            &mut a,
        )
        .unwrap();
    assert_eq!(
        second
            .execute("Node.prototype.normalize.name", &mut b)
            .unwrap(),
        Value::String("normalize".into())
    );
    clean(&first);
    clean(&second);
}
