use super::*;

fn fresh() -> (Runtime, Document) {
    (Runtime::try_new().unwrap(), Document::parse("<p>kept</p>"))
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
    assert_eq!(
        (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.json_depth
        ),
        (0, 0, 0, 0)
    );
}

fn authored(source: &str) {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        clean(&runtime);
    }
}

#[test]
fn for_in_snapshot_ordinary_typed_ordinary_prototype_transitions() {
    authored(
        r#"(function () {
  if (typeof Uint8Array !== 'function' || typeof Object.setPrototypeOf !== 'function') throw new Error('prerequisite');
  var tail = Object.create(null); tail.tail = 1;
  var view = new Uint8Array([7, 9]); view.note = 1;
  Object.setPrototypeOf(view, tail);
  var head = {head: 1}; Object.setPrototypeOf(head, view);
  var seen = '';
  for (var key in head) seen += key + '|';
  return seen === 'head|0|1|note|tail|' && view[0] === 7 && view[1] === 9;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_ordinary_negative_proof_survives_new_records_and_reentry() {
    authored(
        r#"(function () {
  if (typeof Uint8Array !== 'function' || typeof Int16Array !== 'function') throw new Error('prerequisite');
  var seed = new Uint8Array(1);
  var source = {first: 1, later: 2}, seen = '', inner = '', effects = 0, getterCalls = 0;
  for (var key in source) {
    seen += key + '|';
    if (key === 'first') {
      var created = new Uint8Array({length: 1, get 0() {
        effects++;
        var nested = {inside: 1};
        for (var nestedKey in nested) inner += nestedKey + '|';
        var other = new Int16Array(2); other[1] = 17;
        return other[1];
      }});
      Object.defineProperty(source, 'later', {get: function () { getterCalls++; throw new Error('enumeration got value'); }, enumerable: true, configurable: true});
      if (created[0] !== 17 || seed.length !== 1) throw new Error('construction result');
    }
  }
  return seen === 'first|later|' && inner === 'inside|' && effects === 1 && getterCalls === 0;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_live_deletion_redefinition_and_hidden_shadow() {
    authored(
        r#"(function () {
  if (typeof Uint8Array !== 'function' || typeof Object.defineProperty !== 'function') throw new Error('prerequisite');
  var seed = new Uint8Array(1), getterCalls = 0;
  var tail = {deleted: 10, hidden: 20};
  Object.defineProperty(tail, 'tail', {get: function () { getterCalls++; throw new Error('tail getter'); }, enumerable: true});
  var source = {first: 1, deleted: 2, hidden: 3}; Object.setPrototypeOf(source, tail);
  var seen = '';
  for (var key in source) {
    seen += key + '|';
    if (key === 'first') {
      delete source.deleted;
      Object.defineProperty(source, 'hidden', {get: function () { getterCalls++; throw new Error('hidden getter'); }, enumerable: false, configurable: true});
      source.addedAfterSnapshot = 4;
    }
  }
  return seen === 'first|deleted|tail|' && getterCalls === 0 && seed.length === 1 && source.addedAfterSnapshot === 4;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_prototype_is_read_after_body_replaces_it_with_typed_view() {
    authored(
        r#"(function () {
  if (typeof Uint8Array !== 'function' || typeof Object.setPrototypeOf !== 'function') throw new Error('prerequisite');
  var old = {obsolete: 1}, source = {first: 1}; Object.setPrototypeOf(source, old);
  var seed = new Uint8Array(1), chosen, seen = '';
  for (var key in source) {
    seen += key + '|';
    if (key === 'first') {
      chosen = new Uint8Array([23, 29]);
      var tail = Object.create(null); tail.tail = 1;
      Object.setPrototypeOf(chosen, tail);
      Object.setPrototypeOf(source, chosen);
    }
  }
  return seen === 'first|0|1|tail|' && chosen[1] === 29 && seed.length === 1;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_tracking_shrink_refreshes_each_index_and_releases_missing_shadow() {
    authored(
        r#"(function () {
  if (typeof ArrayBuffer.prototype.resize !== 'function' || typeof Uint8Array !== 'function') throw new Error('prerequisite');
  var buffer = new ArrayBuffer(3, {maxByteLength: 6}), view = new Uint8Array(buffer);
  view[0] = 11; view[1] = 22; view[2] = 33;
  var tail = Object.create(null); tail[1] = 77; tail.tail = 1;
  Object.setPrototypeOf(view, tail);
  var seen = '';
  for (var key in view) {
    seen += key + '|';
    if (key === '0') buffer.resize(1);
  }
  return seen === '0|1|tail|' && buffer.byteLength === 1 && view[0] === 11 && view[1] === undefined;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_fixed_view_oob_refreshes_each_index_and_prototype_proof() {
    authored(
        r#"(function () {
  if (typeof ArrayBuffer.prototype.resize !== 'function' || typeof Uint8Array !== 'function') throw new Error('prerequisite');
  var buffer = new ArrayBuffer(3, {maxByteLength: 6}), view = new Uint8Array(buffer, 0, 3);
  view[0] = 11; view[1] = 22; view[2] = 33;
  var tail = Object.create(null); tail[1] = 77; tail.tail = 1;
  Object.setPrototypeOf(view, tail);
  var seen = '';
  for (var key in view) {
    seen += key + '|';
    if (key === '0') buffer.resize(1);
  }
  return seen === '0|1|tail|' && buffer.byteLength === 1 && view[0] === undefined && view[1] === undefined;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_detached_view_indices_remain_live_and_ordinary_tail_is_reclassified() {
    authored(
        r#"(function () {
  if (typeof ArrayBuffer.prototype.transfer !== 'function' || typeof Uint8Array !== 'function') throw new Error('prerequisite');
  var buffer = new ArrayBuffer(3), view = new Uint8Array(buffer);
  view[0] = 11; view[1] = 22; view[2] = 33;
  var tail = Object.create(null); tail[1] = 77; tail.tail = 1;
  Object.setPrototypeOf(view, tail);
  var moved, seen = '';
  for (var key in view) {
    seen += key + '|';
    if (key === '0') moved = buffer.transfer();
  }
  var copied = new Uint8Array(moved);
  return seen === '0|1|tail|' && buffer.detached === true && view[0] === undefined && copied[0] === 11 && copied[1] === 22 && copied[2] === 33;
})()
"#,
    );
}

#[test]
fn for_in_snapshot_target_setter_reentry_keeps_current_identity_and_prior_effects() {
    authored(
        r#"(function () {
  if (typeof Uint8Array !== 'function' || typeof Object.defineProperty !== 'function') throw new Error('prerequisite');
  var seed = new Uint8Array(1), source = {first: 1, later: 2}, trace = '', getters = 0;
  var box = {set key(value) {
    trace += 'S' + value + '|';
    if (value === 'first') {
      var nested = {inner: 1};
      for (var nestedKey in nested) trace += nestedKey + '|';
      var fresh = new Uint8Array([31]);
      Object.defineProperty(source, 'later', {get: function () { getters++; throw new Error('getter'); }, enumerable: true, configurable: true});
      if (fresh[0] !== 31) throw new Error('fresh view');
    }
  }};
  for (box.key in source) trace += 'B|';
  return trace === 'Sfirst|inner|B|Slater|B|' && getters === 0 && seed.length === 1;
})()
"#,
    );
}

// These sidecars are deliberately impossible through public TypedArray Set.
// They distinguish authentic Some(0) from an ordinary no-record proof without
// constructing or modifying the private snapshot token or TypedArray record.
#[test]
fn for_in_snapshot_authentic_zero_states_never_gain_ordinary_numeric_sidecars() {
    for setup in [
        "new Uint8Array(0)",
        "var b=new ArrayBuffer(1);var v=new Uint8Array(b);b.transfer();v",
        "var b=new ArrayBuffer(2,{maxByteLength:4});var v=new Uint8Array(b,0,2);b.resize(0);v",
    ] {
        let (mut runtime, mut doc) = fresh();
        let target = runtime.execute(setup, &mut doc).unwrap();
        check_zero_state(&mut runtime, target);
        clean(&runtime);
    }

    let (mut runtime, mut doc) = fresh();
    runtime
        .execute(
            "var marker={};var caught=false;var source={get length(){throw marker;}};",
            &mut doc,
        )
        .unwrap();
    let first_fresh_id = runtime.objects.len();
    assert_eq!(
        runtime
            .execute(
                "try{new Uint8Array(source);}catch(e){caught=e===marker;}caught",
                &mut doc,
            )
            .unwrap(),
        Value::Bool(true)
    );
    // The real constructor publishes its fresh authentic shell before reading
    // length. That callback throws before backing initialization. Find the
    // retained shell by the existing public-to-script brand query, never by
    // forging a record or reaching into its private fields.
    let mut shells = Vec::new();
    for id in first_fresh_id..runtime.objects.len() {
        let value = Value::Object(id);
        if runtime.typed_array_is_view(&value).unwrap() {
            shells.push(value);
        }
    }
    assert_eq!(shells.len(), 1);
    check_zero_state(&mut runtime, shells.pop().unwrap());
    clean(&runtime);
}

fn check_zero_state(runtime: &mut Runtime, target: Value) {
    assert_eq!(runtime.typed_array_own_length(&target).unwrap(), Some(0));
    let Value::Object(id) = target else {
        panic!("authentic view shell")
    };
    runtime.objects[id].insert("0".into(), Value::Number(99.0));
    runtime.objects[id].insert("visible".into(), Value::Number(7.0));
    let (snapshot, keys) = runtime.for_in_snapshot(target.clone()).unwrap();
    assert_eq!(snapshot.value(), &target);
    assert_eq!(keys, vec![JsString::from("visible")]);
    let mut visited = VisitedNames::default();
    assert!(
        runtime
            .for_in_visit_snapshot(&mut visited, &snapshot, &"0".into())
            .unwrap()
            .is_none()
    );
    assert!(visited.buckets.is_empty());
    let descriptor = runtime
        .for_in_visit_snapshot(&mut visited, &snapshot, &"visible".into())
        .unwrap()
        .unwrap();
    assert!(descriptor.enumerable);
    assert!(visited.buckets[&7].contains_key(&JsString::from("visible")));
}

fn prepare_ordinary(records: bool) -> (Runtime, Document, Value) {
    let (mut runtime, mut doc) = fresh();
    if records {
        runtime
            .execute(
                "var keep=[];for(var i=0;i<8;i++)keep[i]=new Uint8Array(0);",
                &mut doc,
            )
            .unwrap();
    }
    let target = runtime
        .object_ordered([("aa".into(), Value::Number(1.0))])
        .unwrap();
    (runtime, doc, target)
}

// Admission tests use the actual existing operations for their measured base
// costs. Only the new selector/construction/use fees are fixed literals.
#[test]
fn for_in_snapshot_exact_one_short_fees_and_terminal_cleanup() {
    for records in [false, true] {
        let (mut runtime, _, target) = prepare_ordinary(records);
        runtime.steps = MAX_STEPS;
        let before_heap = runtime.allocated;
        let ordinary_keys = runtime.own_keys(&target).unwrap();
        let ordinary_work = MAX_STEPS - runtime.steps;
        let ordinary_heap = runtime.allocated - before_heap;

        let (mut runtime, _, target) = prepare_ordinary(records);
        runtime.steps = MAX_STEPS;
        let before_heap = runtime.allocated;
        let (snapshot, keys) = runtime.for_in_snapshot(target.clone()).unwrap();
        let snapshot_work = MAX_STEPS - runtime.steps;
        let snapshot_heap = runtime.allocated - before_heap;
        assert_eq!(keys, ordinary_keys);
        assert_eq!(snapshot.value(), &target);
        assert_eq!(snapshot_work, ordinary_work + if records { 12 } else { 4 });
        assert_eq!(snapshot_heap, ordinary_heap);
        assert!(snapshot_heap > 0);

        for (steps, bytes, success) in [
            (snapshot_work, snapshot_heap, true),
            (snapshot_work - 1, snapshot_heap, false),
            (snapshot_work, snapshot_heap - 1, false),
        ] {
            let (mut runtime, _, target) = prepare_ordinary(records);
            let object_count = runtime.objects.len();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - bytes;
            match runtime.for_in_snapshot(target) {
                Ok((_, result)) => {
                    assert!(success);
                    assert_eq!(result, ordinary_keys);
                    assert_eq!(runtime.steps, 0);
                    assert_eq!(runtime.allocated, MAX_HEAP);
                }
                Err(error) => {
                    assert!(!success);
                    assert!(error.is_resource_limit());
                }
            }
            assert_eq!(runtime.objects.len(), object_count);
            clean(&runtime);
        }
    }

    let key = JsString::from("aa");
    let (mut runtime, _, target) = prepare_ordinary(true);
    runtime.steps = MAX_STEPS;
    assert_eq!(runtime.typed_array_own_length(&target).unwrap(), None);
    let old_brand_work = MAX_STEPS - runtime.steps;
    assert!(old_brand_work > 4);
    runtime.steps = MAX_STEPS;
    let before_heap = runtime.allocated;
    let mut standalone = VisitedNames::default();
    assert!(
        runtime
            .for_in_visit(&mut standalone, &target, &key)
            .unwrap()
            .is_some()
    );
    let old_visit_work = MAX_STEPS - runtime.steps;
    let old_visit_heap = runtime.allocated - before_heap;

    let (mut runtime, _, target) = prepare_ordinary(true);
    let (snapshot, _) = runtime.for_in_snapshot(target).unwrap();
    runtime.steps = MAX_STEPS;
    let before_heap = runtime.allocated;
    let mut visited = VisitedNames::default();
    assert!(
        runtime
            .for_in_visit_snapshot(&mut visited, &snapshot, &key)
            .unwrap()
            .is_some()
    );
    let visit_work = MAX_STEPS - runtime.steps;
    let visit_heap = runtime.allocated - before_heap;
    assert_eq!(visit_work, old_visit_work - old_brand_work + 4);
    assert_eq!(visit_heap, old_visit_heap);
    assert_eq!(visited, standalone);

    // A duplicate returns before either descriptor reader: both routes have
    // identical reached work and no new storage, including no proof-use fee.
    let mut duplicate = visited.clone();
    runtime.steps = MAX_STEPS;
    let before_heap = runtime.allocated;
    assert!(
        runtime
            .for_in_visit(&mut duplicate, snapshot.value(), &key)
            .unwrap()
            .is_none()
    );
    let duplicate_work = MAX_STEPS - runtime.steps;
    runtime.steps = duplicate_work;
    assert!(
        runtime
            .for_in_visit_snapshot(&mut visited, &snapshot, &key)
            .unwrap()
            .is_none()
    );
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, before_heap);
    assert_eq!(visited, duplicate);

    for (steps, bytes, success) in [
        (visit_work, visit_heap, true),
        (visit_work - 1, visit_heap, false),
        (visit_work, visit_heap - 1, false),
        // Empty visited set pays tick1 + outer lookup1 before the 4-work
        // reader. This cut reaches that reader with only three work left.
        (5, visit_heap, false),
    ] {
        let (mut runtime, _, target) = prepare_ordinary(true);
        let (snapshot, _) = runtime.for_in_snapshot(target).unwrap();
        let mut visited = VisitedNames::default();
        runtime.steps = steps;
        runtime.allocated = MAX_HEAP - bytes;
        match runtime.for_in_visit_snapshot(&mut visited, &snapshot, &key) {
            Ok(Some(_)) => {
                assert!(success);
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert!(visited.buckets[&2].contains_key(&key));
            }
            Err(error) => {
                assert!(!success);
                assert!(error.is_resource_limit());
                assert!(visited.buckets.is_empty());
            }
            Ok(None) => panic!("existing descriptor disappeared"),
        }
        clean(&runtime);
    }

    // A snapshot taken before any records exist retains the unproved route.
    // Later constructors cannot invalidate correctness or retroactively choose
    // the negative-reader fee for that already captured object.
    let (mut runtime, mut doc, target) = prepare_ordinary(false);
    let (snapshot, _) = runtime.for_in_snapshot(target.clone()).unwrap();
    runtime.execute("new Uint8Array(1)", &mut doc).unwrap();
    runtime.steps = MAX_STEPS;
    let mut a = VisitedNames::default();
    runtime
        .for_in_visit(&mut a, &target, &key)
        .unwrap()
        .unwrap();
    let unchanged_work = MAX_STEPS - runtime.steps;
    runtime.steps = unchanged_work;
    let mut b = VisitedNames::default();
    runtime
        .for_in_visit_snapshot(&mut b, &snapshot, &key)
        .unwrap()
        .unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(a, b);
    clean(&runtime);

    terminal_cut_keeps_authored_effects();
}

fn terminal_cut_keeps_authored_effects() {
    let unit = parser::Parser::program(
        "try{for(var key in source){effects++;}}catch(e){caught++;}finally{finalized++;}effects",
    )
    .unwrap();
    let prepare = || {
        let (mut runtime, mut doc, target) = prepare_ordinary(true);
        let id = runtime.property_object(&target).unwrap();
        runtime.objects[id].insert("bb".into(), Value::Number(2.0));
        runtime.objects[id].insert("cc".into(), Value::Number(3.0));
        runtime.objects[id].prototype = None;
        runtime.define(1, "source", target, true).unwrap();
        runtime
            .execute("var effects=0,caught=0,finalized=0;", &mut doc)
            .unwrap();
        runtime.steps = MAX_STEPS;
        (runtime, doc)
    };
    let run = |runtime: &mut Runtime, doc: &mut Document| -> Result<Value> {
        match machine::evaluate_statements(runtime, &unit, machine::ListOwner::Program, 1, doc)? {
            Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
            _ => panic!("unexpected loop completion"),
        }
    };
    let (mut runtime, mut doc) = prepare();
    assert_eq!(run(&mut runtime, &mut doc).unwrap(), Value::Number(3.0));
    let work = MAX_STEPS - runtime.steps;
    clean(&runtime);
    let mut saw_partial_effect = false;
    for allowance in (0..work).step_by(7) {
        let (mut runtime, mut doc) = prepare();
        runtime.steps = allowance;
        assert!(run(&mut runtime, &mut doc).unwrap_err().is_resource_limit());
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        let Value::Number(effects) = runtime.lookup(1, "effects").unwrap().1 else {
            panic!("effect count")
        };
        if effects > 0.0 && effects < 3.0 {
            saw_partial_effect = true;
            assert_eq!(
                runtime.lookup(1, "finalized").unwrap().1,
                Value::Number(0.0)
            );
        }
        clean(&runtime);
        if saw_partial_effect {
            break;
        }
    }
    assert!(saw_partial_effect);
}
