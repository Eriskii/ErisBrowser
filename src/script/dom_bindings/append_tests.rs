// Normative expectations: DOM b2e32dc730eb0dc0cce1a431393fe4a17fda1d54.
use super::*;
use crate::dom::{Doctype, Node};

fn fresh() -> (Runtime, Document) {
    (
        Runtime::new(),
        Document::parse("<!doctype html><html><head></head><body></body></html>"),
    )
}

fn clean(runtime: &Runtime) {
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert_eq!(runtime.eval_depth, 0);
    assert_eq!(runtime.json_depth, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
}

fn check(source: &str) {
    check_with(source, |_, _| {}, |_, _| {});
}

fn check_with(
    source: &str,
    setup: impl Fn(&mut Runtime, &mut Document),
    post: impl Fn(&Runtime, &Document),
) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        for helper in [
            include_str!("../../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../../tests/upstream/test262/harness/sta.js"),
            r#"
            function hierarchy(f) {
                var error;
                try { f(); } catch (e) { error=e; }
                assert.sameValue(error instanceof DOMException,true);
                assert.sameValue(error.name,'HierarchyRequestError');
                assert.sameValue(error.code,3);
            }
            "#,
        ] {
            runtime.execute(helper, &mut doc).unwrap();
        }
        setup(&mut runtime, &mut doc);
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        result.unwrap_or_else(|error| panic!("strict={strict}: {error}"));
        clean(&runtime);
        post(&runtime, &doc);
    }
}

fn hierarchy_error(runtime: &Runtime, error: ScriptError) {
    assert!(!error.is_resource_limit());
    let ErrorKind::Thrown(Value::Object(id)) = &error.kind else {
        panic!("expected thrown DOMException, got {error:?}");
    };
    let bag = &runtime.objects[*id];
    assert_eq!(
        bag.prototype,
        Some(Value::Object(runtime.prototypes["DOMException"]))
    );
    for (name, expected) in [
        ("name", Value::String("HierarchyRequestError".into())),
        ("code", Value::Number(3.0)),
    ] {
        let PropertyValue::Data { value, .. } = &bag.values[&PropertyKey::from(name)].value else {
            panic!("exception field is not data");
        };
        assert_eq!(value, &expected);
    }
}

fn empty_document() -> Document {
    let mut doc = Document::parse("");
    for child in doc.nodes[doc.root].children.clone() {
        doc.remove_child(doc.root, child);
    }
    doc
}

fn doctype(doc: &mut Document) -> NodeId {
    doc.create_doctype(Doctype {
        name: "html".into(),
        public_id: None,
        system_id: None,
        force_quirks: false,
    })
}

#[test]
fn document_append_identity_metadata_and_brand_precede_conversion() {
    check(
        r#"
        var e=document.createElement('div'), f=document.createDocumentFragment();
        var append=document.append, hits=0, bad={toString:function(){hits++;throw 7;}};
        assert.sameValue(typeof append,'function');
        assert.sameValue(append,document.append);
        assert.sameValue(append===e.append,false);
        assert.sameValue(append===f.append,false);
        assert.sameValue(append.name,'append');assert.sameValue(append.length,0);
        var name=Object.getOwnPropertyDescriptor(append,'name');
        var length=Object.getOwnPropertyDescriptor(append,'length');
        assert.sameValue(name.writable,false);assert.sameValue(name.enumerable,false);assert.sameValue(name.configurable,true);
        assert.sameValue(length.writable,false);assert.sameValue(length.enumerable,false);assert.sameValue(length.configurable,true);
        Object.defineProperty(append,'name',{value:'renamed'});
        assert.sameValue(e.append.name,'append');assert.sameValue(f.append.name,'append');
        assert.sameValue(append.call(document),undefined);
        Object.defineProperty(append,'name',name);
        assert.throws(TypeError,function(){append.call(e,bad);});
        assert.throws(TypeError,function(){append.call(f,bad);});
        assert.throws(TypeError,function(){append.call({},bad);});
        assert.throws(TypeError,function(){e.append.call(document,bad);});
        assert.throws(TypeError,function(){f.append.call(document,bad);});
        assert.sameValue(hits,0);
    "#,
    );
}

#[test]
fn variadic_conversion_finishes_before_moves_and_keeps_author_side_effects() {
    check(
        r#"
        var old=document.createElement('div'), target=document.createElement('div');
        var child=document.createElement('i');old.appendChild(child);
        var order='', a={toString:function(){order+='a';assert.sameValue(child.parentNode,old);return 'A';}};
        var b={toString:function(){order+='b';assert.sameValue(child.parentNode,old);target.textContent='live';return 'B';}};
        assert.sameValue(target.append(child,a,b),undefined);
        assert.sameValue(order,'ab');assert.sameValue(target.textContent,'liveAB');
        assert.sameValue(target.childNodes[1],child);assert.sameValue(old.childNodes.length,0);
        var moved=document.createElement('b');old.appendChild(moved);
        var sentinel={}, later=0;
        var first={toString:function(){target.setAttribute('side','kept');return 'first';}};
        var fail={toString:function(){throw sentinel;}};
        var last={toString:function(){later++;return 'last';}};
        try {target.append(moved,first,fail,last);assert(false);}catch(error){assert.sameValue(error,sentinel);}
        assert.sameValue(moved.parentNode,old);assert.sameValue(target.getAttribute('side'),'kept');
        assert.sameValue(target.textContent,'liveAB');assert.sameValue(later,0);
    "#,
    );
}

#[test]
fn zero_arguments_differ_from_empty_undefined_and_failed_symbol_conversion() {
    check(
        r#"
        var root=document.documentElement, parent=root.parentNode;
        assert.sameValue(document.append(),undefined);assert.sameValue(root.parentNode,parent);
        hierarchy(function(){document.append('');});
        hierarchy(function(){document.append(undefined);});
        assert.sameValue(document.documentElement,root);
        var e=document.createElement('div'), old=document.createElement('aside'), child=document.createElement('i');
        old.appendChild(child);var later=0,bad={toString:function(){later++;return 'x';}};
        assert.throws(TypeError,function(){e.append(child,Symbol(),bad);});
        assert.sameValue(child.parentNode,old);assert.sameValue(e.childNodes.length,0);assert.sameValue(later,0);
    "#,
    );
}

#[test]
fn duplicate_nodes_and_fragments_follow_sequential_move_order() {
    check(
        r#"
        var p=document.createElement('div'), a=document.createElement('a'), b=document.createElement('b');
        p.append(a,b,a);assert.sameValue(p.childNodes.length,2);
        assert.sameValue(p.childNodes[0],b);assert.sameValue(p.childNodes[1],a);
        var f=document.createDocumentFragment();f.append(a,b);
        p.append(a,f);assert.sameValue(f.childNodes.length,0);
        assert.sameValue(p.childNodes[0],a);assert.sameValue(p.childNodes[1],b);
        f.append(a,b);p.append(f,a);assert.sameValue(f.childNodes.length,0);
        assert.sameValue(p.childNodes[0],b);assert.sameValue(p.childNodes[1],a);
        f.append(a,b);p.append(f,f);assert.sameValue(f.childNodes.length,0);
        assert.sameValue(p.childNodes.length,2);assert.sameValue(p.childNodes[0],a);assert.sameValue(p.childNodes[1],b);
    "#,
    );
}

#[test]
fn document_existing_root_is_not_excluded_but_duplicate_arguments_move_it_first() {
    check_with(
        r#"
        var root=document.documentElement,parent=root.parentNode;
        hierarchy(function(){document.append(root);});
        hierarchy(function(){root.appendChild.call(document,root);});
        assert.sameValue(root.parentNode,parent);assert.sameValue(document.documentElement,root);
        assert.sameValue(document.append(root,root),undefined);
        assert.sameValue(root.parentNode,parent);assert.sameValue(document.documentElement,root);
        assert.sameValue(parent,document);
    "#,
        |_, _| {},
        |_, doc| {
            // Document has no public childNodes getter in this bounded runtime.
            let children = &doc.nodes[doc.root].children;
            assert_eq!(children.len(), 2);
            assert!(matches!(doc.nodes[children[0]].kind, NodeKind::Doctype(_)));
            assert_eq!(children[1], document_element(doc).unwrap());
            assert_eq!(doc.nodes[children[1]].parent, Some(doc.root));
        },
    );
}

#[test]
fn failed_document_batch_retains_the_temporary_fragment_and_removed_root() {
    check_with(
        r#"
        var root=document.documentElement,parent=root.parentNode;
        hierarchy(function(){document.append(root,'x');});
        assert.sameValue(document.documentElement,null);
        assert.sameValue(parent,document);
        var temp=root.parentNode;assert.sameValue(temp.nodeType,11);
        assert.sameValue(temp.parentNode,null);assert.sameValue(temp.childNodes.length,2);
        assert.sameValue(temp.childNodes[0],root);assert.sameValue(temp.childNodes[1].nodeType,3);
        assert.sameValue(temp.childNodes[1].textContent,'x');
    "#,
        |_, _| {},
        |_, doc| {
            let children = &doc.nodes[doc.root].children;
            assert_eq!(children.len(), 1);
            assert!(matches!(doc.nodes[children[0]].kind, NodeKind::Doctype(_)));
            assert_eq!(doc.nodes[children[0]].parent, Some(doc.root));
            assert_eq!(document_element(doc), None);
        },
    );
    check(
        r#"
        var root=document.documentElement, left=document.createElement('div'),right=document.createElement('div');
        var a=document.createElement('a'),b=document.createElement('b');left.appendChild(a);right.appendChild(b);
        hierarchy(function(){document.append(a,b);});
        assert.sameValue(document.documentElement,root);
        assert.sameValue(left.childNodes.length,0);assert.sameValue(right.childNodes.length,0);
        var temp=a.parentNode;assert.sameValue(temp.nodeType,11);assert.sameValue(temp,b.parentNode);
        assert.sameValue(temp.childNodes[0],a);assert.sameValue(temp.childNodes[1],b);
    "#,
    );
}

#[test]
fn single_and_multi_fragment_failures_have_different_retained_ownership() {
    check(
        r#"
        var f=document.createDocumentFragment(),empty=document.createDocumentFragment();
        var a=document.createElement('a'),b=document.createElement('b');f.append(a,b);
        hierarchy(function(){document.append(f);});
        assert.sameValue(a.parentNode,f);assert.sameValue(b.parentNode,f);assert.sameValue(f.childNodes.length,2);
        hierarchy(function(){document.append(f,empty);});
        assert.sameValue(f.childNodes.length,0);assert.sameValue(empty.childNodes.length,0);
        var temp=a.parentNode;assert.sameValue(temp===f,false);assert.sameValue(temp.nodeType,11);
        assert.sameValue(temp,b.parentNode);assert.sameValue(temp.childNodes[0],a);assert.sameValue(temp.childNodes[1],b);
        var text=document.createTextNode(''),textFragment=document.createDocumentFragment();textFragment.append(text);
        hierarchy(function(){document.append(textFragment);});
        assert.sameValue(text.parentNode,textFragment);assert.sameValue(textFragment.childNodes.length,1);
        assert.sameValue(document.append(empty),undefined);
    "#,
    );
}

#[test]
fn doctype_validation_during_fragment_assembly_preserves_only_the_completed_prefix() {
    check_with(
        r#"
        var root=document.documentElement,parent=root.parentNode,dt=fixtureDoctype;
        assert.sameValue(dt.nodeType,10);
        var holder=document.createElement('div');holder.innerHTML='<!--kept-->';
        var comment=holder.firstChild;assert.sameValue(comment.nodeType,8);
        hierarchy(function(){document.append(dt,comment);});assert.sameValue(comment.parentNode,holder);
        hierarchy(function(){document.append(comment,dt);});
        var temp=comment.parentNode;assert.sameValue(temp.nodeType,11);assert.sameValue(temp.childNodes.length,1);
        assert.sameValue(temp.childNodes[0],comment);assert.sameValue(holder.childNodes.length,0);
        assert.sameValue(dt.parentNode,parent);assert.sameValue(root.parentNode,parent);
        hierarchy(function(){document.append(dt,dt);});assert.sameValue(dt.parentNode,parent);
        hierarchy(function(){holder.append(dt);});
        hierarchy(function(){document.createDocumentFragment().append(dt);});
        assert.sameValue(dt.parentNode,parent);
    "#,
        |runtime, doc| {
            // Bind the actual parsed node; no unsupported Document getter or
            // fabricated Node(root) wrapper is used to obtain it in JavaScript.
            let children = &doc.nodes[doc.root].children;
            assert_eq!(children.len(), 2);
            let dt = children[0];
            assert!(matches!(doc.nodes[dt].kind, NodeKind::Doctype(_)));
            runtime
                .define(0, "fixtureDoctype", Value::Node(dt), false)
                .unwrap();
        },
        |_, doc| {
            let children = &doc.nodes[doc.root].children;
            assert_eq!(children.len(), 2);
            assert!(matches!(doc.nodes[children[0]].kind, NodeKind::Doctype(_)));
            assert_eq!(children[1], document_element(doc).unwrap());
            assert!(
                children
                    .iter()
                    .all(|child| doc.nodes[*child].parent == Some(doc.root))
            );
        },
    );
}

#[test]
fn single_and_multi_cycles_preserve_the_normative_mutation_prefix() {
    check(
        r#"
        var old=document.createElement('aside'),target=document.createElement('div'),child=document.createElement('i');
        old.appendChild(target);target.appendChild(child);
        hierarchy(function(){target.append(target);});
        hierarchy(function(){child.append(target);});
        assert.sameValue(target.parentNode,old);assert.sameValue(child.parentNode,target);
        hierarchy(function(){target.append(child,target);});
        var temp=child.parentNode;assert.sameValue(temp.nodeType,11);
        assert.sameValue(target.parentNode,temp);assert.sameValue(old.childNodes.length,0);
        assert.sameValue(target.childNodes.length,0);assert.sameValue(temp.childNodes[0],child);assert.sameValue(temp.childNodes[1],target);
        var a=document.createElement('a');old.appendChild(a);
        hierarchy(function(){target.append(a,document);});
        assert.sameValue(old.childNodes.length,0);assert.sameValue(a.parentNode.nodeType,11);
        assert.sameValue(a.parentNode.childNodes.length,1);
        var template=document.createElement('template');
        hierarchy(function(){template.content.append(template);});
        assert.sameValue(template.parentNode,null);assert.sameValue(template.content.childNodes.length,0);
    "#,
    );
}

#[test]
fn checked_append_enforces_the_supported_node_kind_matrix() {
    // Each row has a fresh tree; element counts cannot leak between rows.
    for parent_kind in 0..4 {
        for child_kind in 0..7 {
            let mut doc = empty_document();
            let mut runtime = Runtime::new();
            let parent = match parent_kind {
                0 => doc.root,
                1 => doc.create_element("div"),
                2 => doc.create_document_fragment(),
                _ => doc.create_text_node("parent"),
            };
            let child = match child_kind {
                0 => doc.create_element("i"),
                1 => doc.create_document_fragment(),
                2 => doc.create_text_node(""),
                3 => doc.create_comment("comment"),
                4 => doc.create_processing_instruction("target", "data"),
                5 => doctype(&mut doc),
                _ => doc.root,
            };
            let admitted = match parent_kind {
                0 => matches!(child_kind, 0 | 1 | 3 | 4 | 5),
                1 | 2 => child_kind <= 4,
                _ => false,
            };
            let result = runtime.dom_checked_append(parent, child, &mut doc);
            if admitted {
                result.unwrap();
                if child_kind == 1 {
                    assert!(doc.nodes[parent].children.is_empty());
                    assert_eq!(doc.nodes[child].parent, None);
                } else {
                    assert_eq!(doc.nodes[parent].children, [child]);
                    assert_eq!(doc.nodes[child].parent, Some(parent));
                }
            } else {
                hierarchy_error(&runtime, result.unwrap_err());
                assert!(doc.nodes[parent].children.is_empty());
                assert_eq!(doc.nodes[child].parent, None);
            }
            clean(&runtime);
        }
    }
}

#[test]
fn checked_document_append_keeps_doctype_before_unique_element_and_allows_comments_pi() {
    let mut doc = empty_document();
    let mut runtime = Runtime::new();
    let parent = doc.root;
    let before = doc.create_comment("before");
    let dt = doctype(&mut doc);
    let middle = doc.create_processing_instruction("mid", "");
    let element = doc.create_element("html");
    let after = doc.create_comment("after");
    for child in [before, dt, middle, element, after] {
        runtime.dom_checked_append(parent, child, &mut doc).unwrap();
    }
    let expected = vec![before, dt, middle, element, after];
    assert_eq!(doc.nodes[parent].children, expected);
    let other_element = doc.create_element("other");
    let other_dt = doctype(&mut doc);
    for child in [element, dt, other_element, other_dt] {
        let error = runtime
            .dom_checked_append(parent, child, &mut doc)
            .unwrap_err();
        hierarchy_error(&runtime, error);
        assert_eq!(doc.nodes[parent].children, expected);
    }
    doc.remove_child(parent, dt);
    let error = runtime
        .dom_checked_append(parent, dt, &mut doc)
        .unwrap_err();
    hierarchy_error(&runtime, error); // A doctype cannot be appended after an element.
    assert_eq!(doc.nodes[dt].parent, None);
    doc.remove_child(parent, element);
    runtime.dom_checked_append(parent, dt, &mut doc).unwrap();
    let fragment = doc.create_document_fragment();
    doc.append_child(fragment, element);
    runtime
        .dom_checked_append(parent, fragment, &mut doc)
        .unwrap();
    assert_eq!(
        doc.nodes[parent].children,
        [before, middle, after, dt, element]
    );
    assert!(doc.nodes[fragment].children.is_empty());
    clean(&runtime);
}

#[test]
fn checked_append_depth_limit_is_terminal_and_leaves_the_current_move_unapplied() {
    let mut doc = empty_document();
    let mut parent = doc.create_element("outer");
    for _ in 1..crate::dom::MAX_DEPTH {
        let next = doc.create_element("div");
        doc.append_child(parent, next);
        assert_eq!(doc.nodes[next].parent, Some(parent));
        parent = next;
    }
    let source = doc.create_element("aside");
    let child = doc.create_element("i");
    doc.append_child(source, child);
    let mut runtime = Runtime::new();
    assert!(
        runtime
            .dom_checked_append(parent, child, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(doc.nodes[child].parent, Some(source));
    assert_eq!(doc.nodes[source].children, [child]);
    assert!(doc.nodes[parent].children.is_empty());
    clean(&runtime);
}

#[test]
fn checked_move_exact_and_one_short_work_heap_admission_precede_tree_changes() {
    let mut initial = empty_document();
    let source = initial.create_element("aside");
    let target = initial.create_element("main");
    let child = initial.create_element("i");
    initial.append_child(source, child);
    let mut baseline = Runtime::new();
    let steps = baseline.steps;
    let allocated = baseline.allocated;
    baseline
        .dom_checked_append(target, child, &mut initial.clone())
        .unwrap();
    let work = steps - baseline.steps;
    let heap = baseline.allocated - allocated;
    assert!(work > 0 && heap > 0);
    for (work_short, heap_short) in [(false, false), (true, false), (false, true)] {
        let mut doc = initial.clone();
        let mut runtime = Runtime::new();
        runtime.steps = work - usize::from(work_short);
        runtime.allocated = MAX_HEAP - heap + usize::from(heap_short);
        let result = runtime.dom_checked_append(target, child, &mut doc);
        if work_short || heap_short {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(doc.nodes[source].children, [child]);
            assert_eq!(doc.nodes[child].parent, Some(source));
            assert!(doc.nodes[target].children.is_empty());
        } else {
            result.unwrap();
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert!(doc.nodes[source].children.is_empty());
            assert_eq!(doc.nodes[target].children, [child]);
            assert_eq!(doc.nodes[child].parent, Some(target));
        }
        clean(&runtime);
    }
}

#[test]
fn terminal_second_move_keeps_the_first_node_in_the_temporary_fragment() {
    let mut initial = empty_document();
    let source = initial.create_element("aside");
    let target = initial.create_element("main");
    let first = initial.create_element("i");
    let large = initial.create_element("section");
    let empty = initial.create_document_fragment();
    initial.append_child(source, first);
    initial.append_child(source, large);
    for _ in 0..1024 {
        let child = initial.create_element("b");
        initial.append_child(large, child);
    }
    // Calibrate a completed small operation on exactly the same initial arena.
    // This includes more work than its first move, without guessing helper fees.
    // Both arenas have one test-owned fragment slot. This witness isolates
    // move traversal; separately tested paid growth must not enter calibration.
    initial.nodes.try_reserve_exact(1).unwrap();
    let mut calibration_doc = initial.clone();
    calibration_doc.nodes.try_reserve_exact(1).unwrap();
    let mut calibration = Runtime::new();
    let before = calibration.steps;
    calibration
        .dom_native(
            "Element.append",
            Value::Node(target),
            &[Value::Node(first), Value::Node(empty)],
            &mut calibration_doc,
        )
        .unwrap();
    let budget = before - calibration.steps + 1;
    assert!(
        budget < 1024,
        "small-operation budget no longer isolates the large traversal"
    );
    let mut doc = initial;
    let mut runtime = Runtime::new();
    runtime.steps = budget;
    assert!(
        runtime
            .dom_native(
                "Element.append",
                Value::Node(target),
                &[Value::Node(first), Value::Node(large)],
                &mut doc
            )
            .unwrap_err()
            .is_resource_limit()
    );
    let temporary = doc.nodes[first].parent.expect("first move completed");
    assert!(matches!(
        doc.nodes[temporary].kind,
        NodeKind::DocumentFragment { host: None }
    ));
    assert_eq!(doc.nodes[temporary].children, [first]);
    assert_eq!(doc.nodes[temporary].parent, None);
    assert_eq!(doc.nodes[large].parent, Some(source));
    assert_eq!(doc.nodes[source].children, [large]);
    assert_eq!(doc.nodes[large].children.len(), 1024);
    assert!(doc.nodes[target].children.is_empty());
    clean(&runtime);
}

#[test]
fn all_text_nodes_materialize_before_any_argument_moves() {
    let mut doc = empty_document();
    let source = doc.create_element("aside");
    let target = doc.create_element("main");
    let child = doc.create_element("i");
    doc.append_child(source, child);
    // Detached empty comments are valid arena entries and consume no text bytes.
    doc.nodes.resize_with(crate::dom::MAX_NODES - 1, || Node {
        parent: None,
        children: Vec::new(),
        kind: NodeKind::Comment(String::new().into()),
    });
    // One test-owned Text slot isolates the second-string node-cap refusal.
    // Actual full-buffer work/heap refusal has separate focused coverage.
    doc.nodes.try_reserve_exact(1).unwrap();
    let mut runtime = Runtime::new();
    let error = runtime
        .dom_native(
            "Element.append",
            Value::Node(target),
            &[
                Value::Node(child),
                Value::String("a".into()),
                Value::String("b".into()),
            ],
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(doc.nodes.len(), crate::dom::MAX_NODES);
    assert!(matches!(&doc.nodes.last().unwrap().kind, NodeKind::Text(text) if text == "a"));
    assert_eq!(doc.nodes.last().unwrap().parent, None);
    assert_eq!(doc.nodes[source].children, [child]);
    assert_eq!(doc.nodes[child].parent, Some(source));
    assert!(doc.nodes[target].children.is_empty());
    clean(&runtime);
}

#[test]
fn exhausted_exception_storage_is_terminal_not_a_catchable_hierarchy_success() {
    let (mut runtime, mut doc) = fresh();
    let root = document_element(&doc).unwrap();
    let parent = doc.root;
    let before = doc.nodes[parent].children.clone();
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .dom_checked_append(parent, root, &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(doc.nodes[parent].children, before);
    assert_eq!(doc.nodes[root].parent, Some(parent));
    clean(&runtime);
}

#[test]
fn public_conversion_resource_stop_keeps_author_mutation_and_unwinds_without_catch() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let source = r#"
            var old=document.createElement('aside'), target=document.createElement('main');
            var child=document.createElement('i');old.appendChild(child);
            var caught=false,finalized=false,completed=false;
            var bad={toString:function(){target.setAttribute('side','kept');while(true){}}};
            try {target.append(child,bad);completed=true;}catch(error){caught=true;}finally{finalized=true;}
        "#;
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert!(result.unwrap_err().is_resource_limit());
        for name in ["caught", "finalized", "completed"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Bool(false)
            );
        }
        let node = |name: &str| match &runtime.environments[0].bindings[name].value {
            Value::Node(id) => *id,
            _ => panic!("expected node binding {name}"),
        };
        let (old, target, child) = (node("old"), node("target"), node("child"));
        assert_eq!(doc.attr(target, "side"), Some("kept"));
        assert!(doc.nodes[target].children.is_empty());
        assert_eq!(doc.nodes[old].children, [child]);
        assert_eq!(doc.nodes[child].parent, Some(old));
        clean(&runtime);
    }
}

#[test]
fn failed_document_append_updates_base_url_before_a_later_root_restore() {
    check(
        r#"
        var root=document.documentElement, fallback=document.URL;
        var base=document.createElement('base');
        base.setAttribute('href','https://append.example/assets/');
        document.head.append(base);
        assert.sameValue(document.baseURI,'https://append.example/assets/');
        hierarchy(function(){document.append(root,'x');});
        assert.sameValue(document.documentElement,null);
        assert.sameValue(root.parentNode.nodeType,11);
        assert.sameValue(document.URL,fallback);
        assert.sameValue(document.baseURI,fallback);
        assert.sameValue(document.append(root),undefined);
        assert.sameValue(document.documentElement,root);
        assert.sameValue(root.parentNode,document);
        assert.sameValue(base.parentNode,document.head);
        assert.sameValue(document.baseURI,'https://append.example/assets/');
    "#,
    );
}

#[test]
fn failed_document_append_keeps_details_group_changes_from_fragment_assembly() {
    // HTML details insertion steps apply in detached trees as well. The
    // inserted open member closes when its new tree already has an open peer.
    check(
        r#"
        var connected=document.createElement('details');
        connected.setAttribute('name','append-group');connected.open=true;
        document.body.append(connected);
        var a=document.createElement('details'),b=document.createElement('details');
        a.setAttribute('name','append-group');b.setAttribute('name','append-group');
        a.open=true;b.open=true;
        assert.sameValue(a.open,true);assert.sameValue(b.open,true);
        hierarchy(function(){document.append(a,b);});
        var temporary=a.parentNode;
        assert.sameValue(temporary.nodeType,11);assert.sameValue(b.parentNode,temporary);
        assert.sameValue(temporary.childNodes[0],a);assert.sameValue(temporary.childNodes[1],b);
        assert.sameValue(a.open,true);assert.sameValue(b.open,false);
        assert.sameValue(connected.open,true);
        document.body.append(a);
        assert.sameValue(a.parentNode,document.body);assert.sameValue(a.open,false);
        assert.sameValue(connected.open,true);assert.sameValue(b.open,false);
        assert.sameValue(temporary.childNodes.length,1);assert.sameValue(temporary.childNodes[0],b);
    "#,
    );
}
