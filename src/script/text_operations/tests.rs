use super::*;

const CASES: &str = include_str!("../../../tests/fixtures/text-operations.js");
fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
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
        let source = format!("{CASES}\ntextOperationCases.{name}();");
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
    detached_surrogate_boundary,
    attached_surrogate_boundary_and_identity,
    zero_end_and_empty_make_distinct_nodes,
    whole_text_barriers_and_detached_runs,
    conversion_uses_fresh_data_and_parent,
    shrink_during_conversion_retains_callback_prefix,
    numeric_wrapping_required_argument_and_symbol,
    authentic_brand_precedes_offset_conversion,
    throwing_conversion_preserves_exact_identity_and_effects,
    saved_functions_survive_property_replacement_and_deletion,
    internal_slots_ignore_authored_public_getters,
    result_uses_default_text_prototype,
    whole_text_exact_pair_join_without_mutation,
);

#[test]
fn text_operations_bootstrap_admission() {
    let runtime = Runtime::uninitialized().finish_bootstrap().unwrap();
    println!(
        "TEXT_OPERATIONS_BOOTSTRAP steps={} allocated={} objects={} capacity={} native={} prototypes={}",
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
fn function_bags_are_distinct_ordinary_nonconstructible_and_realm_local() {
    let (mut runtime, mut doc) = fresh();
    assert_eq!(runtime.execute(r#"(function(){
      var split=Text.prototype.splitText,get=Object.getOwnPropertyDescriptor(Text.prototype,'wholeText').get;
      if(split===get)throw new Error('identity');
      for(var f of [split,get]){
        var n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');
        if(n.writable||n.enumerable||!n.configurable||l.writable||l.enumerable||!l.configurable||
           Object.getPrototypeOf(f)!==Function.prototype||Object.getOwnPropertyDescriptor(f,'prototype')!==undefined)
          throw new Error('function metadata');
        var caught=false;try{Reflect.construct(f,[]);}catch(e){caught=e instanceof TypeError;}
        if(!caught)throw new Error('constructibility');
      }
      Object.defineProperty(get,'name',{value:'changed'});
      if(split.name!=='splitText')throw new Error('shared bag');
      var text=new Text('x');text.wholeText='ignored';
      if(text.wholeText!=='x'||Object.prototype.hasOwnProperty.call(text,'wholeText'))throw new Error('readonly');
      var caught=false;try{(function(){'use strict';text.wholeText='no';})();}catch(e){caught=e instanceof TypeError;}
      if(!caught)throw new Error('strict readonly');return true;
    })()"#,&mut doc).unwrap(),Value::Bool(true));
    let (mut second, mut second_doc) = fresh();
    assert_eq!(
        second
            .execute(
                "Object.getOwnPropertyDescriptor(Text.prototype,'wholeText').get.name",
                &mut second_doc
            )
            .unwrap(),
        Value::String("get wholeText".into())
    );
    clean(&runtime);
    clean(&second);
}

fn setup(
    attached: bool,
    full_nodes: bool,
    full_children: bool,
) -> (Runtime, Document, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let parent = doc.create_document_fragment();
    let text = doc.create_text_node("A\u{1f680}B");
    if attached {
        let following = doc.create_comment("barrier");
        doc.append_child(parent, text);
        doc.append_child(parent, following);
    }
    if full_nodes {
        doc.nodes.shrink_to_fit();
    } else {
        doc.nodes.try_reserve_exact(1).unwrap();
    }
    if full_children {
        doc.nodes[parent].children.shrink_to_fit();
    } else {
        doc.nodes[parent].children.try_reserve_exact(1).unwrap();
    }
    (runtime, doc, text, parent)
}
fn units(doc: &Document, id: NodeId) -> Vec<u16> {
    current_text(doc, id).unwrap().units().collect()
}
fn state(doc: &Document, parent: NodeId) -> (String, usize, usize) {
    (
        format!("{doc:?}"),
        doc.nodes.capacity(),
        doc.nodes[parent].children.capacity(),
    )
}

#[test]
fn suffix_exact_and_one_short_admission_precedes_node_publication_and_real_growth() {
    for full in [false, true] {
        let (mut runtime, mut doc, id, _) = setup(false, full, false);
        let before = (runtime.steps, runtime.allocated);
        let fresh = runtime.split_text_suffix(id, 2, 4, &mut doc).unwrap();
        assert_eq!(units(&doc, fresh), [0xde80, 66]);
        assert_eq!(units(&doc, id), [65, 0xd83d, 0xde80, 66]);
        let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
        for (steps, bytes, success) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, id, parent) = setup(false, full, false);
            let before = state(&doc, parent);
            let count = doc.nodes.len();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            let result = runtime.split_text_suffix(id, 2, 4, &mut doc);
            if success {
                assert_eq!(result.unwrap(), count);
                assert_eq!(units(&doc, count), [0xde80, 66]);
                assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(state(&doc, parent), before);
            }
            clean(&runtime);
        }
    }
}

#[test]
fn complete_split_exact_and_one_short_preserve_inserted_suffix_on_late_refusal() {
    for (offset, prefix, tail, intermediate, final_delta) in [
        (1, vec![65], vec![0xd83d, 0xde80, 66], 5, 0),
        (2, vec![65, 0xd83d], vec![0xde80, 66], 4, 2),
    ] {
        for (attached, full_nodes, full_children) in [
            (false, false, false),
            (true, false, false),
            (true, true, true),
        ] {
            let (mut runtime, mut doc, id, _) = setup(attached, full_nodes, full_children);
            let before = (runtime.steps, runtime.allocated);
            runtime
                .text_operations_native(
                    "splitText",
                    Value::Node(id),
                    &[Value::Number(offset as f64)],
                    &mut doc,
                )
                .unwrap();
            let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
            for (steps, bytes, success) in [
                (work, heap, true),
                (work - 1, heap, false),
                (work, heap - 1, false),
            ] {
                let (mut runtime, mut doc, id, parent) = setup(attached, full_nodes, full_children);
                let (count, retained) = (doc.nodes.len(), doc.retained_bytes());
                let old_children = doc.nodes[parent].children.clone();
                runtime.steps = steps;
                runtime.allocated = MAX_HEAP - bytes;
                let result = runtime.text_operations_native(
                    "splitText",
                    Value::Node(id),
                    &[Value::Number(offset as f64)],
                    &mut doc,
                );
                assert_eq!(doc.nodes.len(), count + 1);
                assert_eq!(units(&doc, count), tail);
                assert_eq!(doc.nodes[count].parent, attached.then_some(parent));
                if attached {
                    assert_eq!(doc.nodes[parent].children, [id, count, old_children[1]]);
                }
                if success {
                    assert_eq!(result.unwrap(), Value::Node(count));
                    assert_eq!(units(&doc, id), prefix);
                    assert_eq!(doc.retained_bytes(), retained + final_delta);
                    assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert_eq!(units(&doc, id), [65, 0xd83d, 0xde80, 66]);
                    assert_eq!(doc.retained_bytes(), retained + intermediate);
                }
                clean(&runtime);
            }
        }
    }
}

#[test]
fn every_work_cut_retains_only_original_detached_or_inserted_suffix_phase() {
    let (mut runtime, mut doc, id, _) = setup(true, false, false);
    let before = runtime.steps;
    runtime.split_text(id, 2, &mut doc).unwrap();
    let work = before - runtime.steps;
    let mut phases = [false; 3];
    for steps in 0..work {
        let (mut runtime, mut doc, id, parent) = setup(true, false, false);
        let (count, retained) = (doc.nodes.len(), doc.retained_bytes());
        let original = state(&doc, parent);
        let old_children = doc.nodes[parent].children.clone();
        runtime.steps = steps;
        assert!(
            runtime
                .split_text(id, 2, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.steps, 0);
        assert_eq!(units(&doc, id), [65, 0xd83d, 0xde80, 66]);
        match doc.nodes.len() - count {
            0 => {
                phases[0] = true;
                assert_eq!(state(&doc, parent), original);
            }
            1 => {
                assert_eq!(units(&doc, count), [0xde80, 66]);
                assert_eq!(doc.retained_bytes(), retained + 4);
                match doc.nodes[count].parent {
                    None => {
                        phases[1] = true;
                        assert_eq!(doc.nodes[parent].children, old_children);
                    }
                    Some(p) => {
                        phases[2] = true;
                        assert_eq!(p, parent);
                        assert_eq!(doc.nodes[parent].children, [id, count, old_children[1]]);
                    }
                }
            }
            n => panic!("unexpected split publication count {n}"),
        }
        clean(&runtime);
    }
    assert_eq!(phases, [true, true, true]);
}

#[test]
fn canonical_suffix_expansion_and_prefix_replacement_have_distinct_retained_totals() {
    let (mut runtime, mut doc) = fresh();
    let id = doc
        .create_text_node_owned(DomString::from_units_owned(vec![0xd800, 0x4e00, 0x4e01]).unwrap())
        .unwrap();
    doc.nodes.try_reserve_exact(1).unwrap();
    let retained = doc.retained_bytes();
    let fresh = runtime.split_text_suffix(id, 1, 3, &mut doc).unwrap();
    assert_eq!(doc.retained_bytes(), retained + 6);
    assert_eq!(
        current_text(&doc, fresh).unwrap().scalar(),
        Some("\u{4e00}\u{4e01}")
    );
    assert_eq!(units(&doc, id), [0xd800, 0x4e00, 0x4e01]);
    runtime.split_text_prefix(id, 1, &mut doc).unwrap();
    assert_eq!(doc.retained_bytes(), retained + 2);
    assert_eq!(units(&doc, id), [0xd800]);
    clean(&runtime);
}

#[test]
fn genuine_brand_precedes_conversion_for_invalid_ids_and_other_character_data() {
    let (mut runtime, mut doc) = fresh();
    let input = runtime
        .execute(
            "var textConversions=0;({valueOf(){textConversions++;return 0;}})",
            &mut doc,
        )
        .unwrap();
    let comment = doc.create_comment("x");
    let pi = doc.create_processing_instruction("ok", "x");
    for receiver in [
        Value::Document,
        Value::Node(doc.root),
        Value::Node(comment),
        Value::Node(pi),
        Value::Node(doc.nodes.len()),
        Value::Node(usize::MAX),
        Value::Null,
    ] {
        for method in ["splitText", "getWholeText"] {
            assert_eq!(
                runtime
                    .text_operations_native(
                        method,
                        receiver.clone(),
                        std::slice::from_ref(&input),
                        &mut doc
                    )
                    .unwrap_err()
                    .name(),
                "TypeError"
            );
        }
    }
    assert_eq!(
        runtime.environments[0].bindings["textConversions"].value,
        Value::Number(0.0)
    );
    clean(&runtime);
}

fn whole_setup() -> (Runtime, Document, NodeId, NodeId) {
    let (runtime, mut doc) = fresh();
    let parent = doc.create_document_fragment();
    let first = doc
        .create_text_node_owned(DomString::from_units_owned(vec![0xd800]).unwrap())
        .unwrap();
    let empty = doc.create_text_node("");
    let last = doc
        .create_text_node_owned(DomString::from_units_owned(vec![0xdc00, 65]).unwrap())
        .unwrap();
    for id in [first, empty, last] {
        doc.append_child(parent, id);
    }
    (runtime, doc, empty, parent)
}

#[test]
fn whole_text_exact_one_short_includes_separate_rc_copy_and_leaves_dom_untouched() {
    let (mut runtime, mut doc, id, _) = whole_setup();
    let before = (runtime.steps, runtime.allocated);
    let expected = runtime
        .text_operations_native("getWholeText", Value::Node(id), &[], &mut doc)
        .unwrap();
    assert_eq!(expected, Value::String(vec![0xd800, 0xdc00, 65].into()));
    let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (32, heap, false),
    ] {
        let (mut runtime, mut doc, id, parent) = whole_setup();
        let original = state(&doc, parent);
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.text_operations_native("getWholeText", Value::Node(id), &[], &mut doc);
        if success {
            assert_eq!(result.unwrap(), expected);
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            if steps < work {
                assert_eq!(runtime.steps, 0);
            } else {
                assert!(runtime.allocated > MAX_HEAP);
            }
        }
        assert_eq!(state(&doc, parent), original);
        clean(&runtime);
    }
}

#[test]
fn whole_text_cost_ignores_unrelated_arena_and_payloads() {
    let mut costs = Vec::new();
    for unrelated in [0, 1000] {
        let (mut runtime, mut doc, id, parent) = whole_setup();
        for _ in 0..unrelated {
            doc.create_comment("unrelated payload");
        }
        let original = state(&doc, parent);
        let before = (runtime.steps, runtime.allocated);
        assert_eq!(
            runtime.whole_text(id, &doc).unwrap(),
            Value::String(vec![0xd800, 0xdc00, 65].into())
        );
        costs.push((before.0 - runtime.steps, runtime.allocated - before.1));
        assert_eq!(state(&doc, parent), original);
        clean(&runtime);
    }
    assert_eq!(costs[0], costs[1]);
}

fn fill_nodes(doc: &mut Document, count: usize) {
    doc.nodes.resize_with(count, || crate::dom::Node {
        parent: None,
        children: Vec::new(),
        kind: NodeKind::Comment(DomString::default()),
    });
}
#[test]
fn numeric_callback_consumes_current_node_or_byte_allowance_before_outer_split() {
    for bytes in [false, true] {
        let (mut runtime, mut doc, id, parent) = setup(true, false, false);
        let input=runtime.execute("var splitEffect=0;({valueOf(){splitEffect++;document.createTextNode('callback');return 2;}})",&mut doc).unwrap();
        if bytes {
            while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 8 {
                let amount =
                    (crate::dom::MAX_DOM_BYTES - 8 - doc.retained_bytes()).min(8 * 1024 * 1024);
                let added = doc.create_text_node(&"x".repeat(amount));
                assert_ne!(added, doc.root);
            }
        } else {
            fill_nodes(&mut doc, MAX_NODES - 1);
        }
        doc.nodes.try_reserve_exact(1).unwrap();
        let (count, retained) = (doc.nodes.len(), doc.retained_bytes());
        let children = doc.nodes[parent].children.clone();
        assert!(
            runtime
                .text_operations_native("splitText", Value::Node(id), &[input], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            runtime.environments[0].bindings["splitEffect"].value,
            Value::Number(1.0)
        );
        assert_eq!(doc.nodes.len(), count + 1);
        assert_eq!(doc.retained_bytes(), retained + 8);
        assert_eq!(
            units(&doc, count),
            "callback".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(doc.nodes[parent].children, children);
        assert_eq!(units(&doc, id), [65, 0xd83d, 0xde80, 66]);
        clean(&runtime);
    }
}

#[test]
fn empty_suffix_still_requires_a_fresh_node_and_last_slot_is_usable() {
    for spare in [false, true] {
        let (mut runtime, mut doc, id, _) = setup(false, false, false);
        fill_nodes(&mut doc, MAX_NODES - usize::from(spare));
        if spare {
            doc.nodes.try_reserve_exact(1).unwrap();
        }
        let count = doc.nodes.len();
        let retained = doc.retained_bytes();
        let result = runtime.split_text(id, 4, &mut doc);
        if spare {
            assert_eq!(result.unwrap(), Value::Node(count));
            assert!(units(&doc, count).is_empty());
            assert_eq!(doc.nodes.len(), count + 1);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(doc.nodes.len(), count);
        }
        assert_eq!(doc.retained_bytes(), retained);
        assert_eq!(units(&doc, id), [65, 0xd83d, 0xde80, 66]);
        clean(&runtime);
    }
}

#[test]
fn split_does_not_credit_future_truncation_to_suffix_admission() {
    let (mut runtime, mut doc, id, parent) = setup(true, false, false);
    while doc.retained_bytes() < crate::dom::MAX_DOM_BYTES - 3 {
        let amount = (crate::dom::MAX_DOM_BYTES - 3 - doc.retained_bytes()).min(8 * 1024 * 1024);
        let new = doc.create_text_node(&"x".repeat(amount));
        assert_ne!(new, doc.root);
    }
    doc.nodes.try_reserve_exact(1).unwrap();
    let count = doc.nodes.len();
    let children = doc.nodes[parent].children.clone();
    assert!(
        runtime
            .split_text(id, 2, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(doc.nodes.len(), count);
    assert_eq!(doc.retained_bytes(), crate::dom::MAX_DOM_BYTES - 3);
    assert_eq!(doc.nodes[parent].children, children);
    assert_eq!(units(&doc, id), [65, 0xd83d, 0xde80, 66]);
    clean(&runtime);
}

#[test]
fn terminal_numeric_conversion_preserves_effect_and_skips_catch_finally() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = "var text=new Text('old'),effect=0,caught=false,finalized=false;try{text.splitText({valueOf(){effect++;text.data='callback';while(true){};}});}catch(e){caught=true;}finally{finalized=true;}";
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        for (name, value) in [
            ("effect", Value::Number(1.0)),
            ("caught", Value::Bool(false)),
            ("finalized", Value::Bool(false)),
        ] {
            assert_eq!(runtime.environments[0].bindings[name].value, value);
        }
        let Value::Node(id) = runtime.environments[0].bindings["text"].value else {
            panic!()
        };
        assert_eq!(current_text(&doc, id).unwrap().scalar(), Some("callback"));
        clean(&runtime);
    }
}

#[test]
fn saved_getter_invocation_refusals_unwind_the_native_name_copy() {
    fn ready() -> (Runtime, Document, Value, Value) {
        let (mut runtime, mut doc) = fresh();
        let get = runtime
            .execute(
                "Object.getOwnPropertyDescriptor(Text.prototype,'wholeText').get",
                &mut doc,
            )
            .unwrap();
        let node = Value::Node(doc.create_text_node("exact"));
        (runtime, doc, get, node)
    }
    let (mut runtime, mut doc, get, node) = ready();
    let before = (runtime.steps, runtime.allocated);
    let expected = runtime.call(get, Vec::new(), node, &mut doc).unwrap();
    let (work, heap) = (before.0 - runtime.steps, runtime.allocated - before.1);
    for (steps, bytes, success) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
        (8, heap, false),
    ] {
        let (mut runtime, mut doc, get, node) = ready();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        let result = runtime.call(get, Vec::new(), node, &mut doc);
        if success {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        clean(&runtime);
    }
}
