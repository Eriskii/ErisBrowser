//! Test-only representation adapters and independent singleton admission tests.
use super::*;

pub(super) enum BucketKeys<'a> {
    Single(std::iter::Once<&'a JsString>),
    Tree(std::collections::btree_map::Keys<'a, JsString, ()>),
}
impl<'a> Iterator for BucketKeys<'a> {
    type Item = &'a JsString;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(keys) => keys.next(),
            Self::Tree(keys) => keys.next(),
        }
    }
}
impl NameBucket {
    // Unmetered test setup only. First retained identity survives duplicates.
    pub(super) fn test_insert(&mut self, key: JsString) {
        match self {
            Self::Single(old) if *old == key => {}
            Self::Single(old) => {
                let mut tree = BTreeMap::new();
                tree.insert(old.clone(), ());
                tree.insert(key, ());
                *self = Self::Tree(tree);
            }
            Self::Tree(tree) => {
                tree.entry(key).or_insert(());
            }
        }
    }
    pub(super) fn keys(&self) -> BucketKeys<'_> {
        match self {
            Self::Single(key) => BucketKeys::Single(std::iter::once(key)),
            Self::Tree(tree) => BucketKeys::Tree(tree.keys()),
        }
    }
    pub(super) fn get_key_value(&self, key: &JsString) -> Option<(&JsString, &())> {
        match self {
            Self::Single(old) if old == key => Some((old, &())),
            Self::Single(_) => None,
            Self::Tree(tree) => tree.get_key_value(key),
        }
    }
    pub(super) fn contains_key(&self, key: &JsString) -> bool {
        self.get_key_value(key).is_some()
    }
    pub(super) fn is_empty(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Tree(tree) => tree.is_empty(),
        }
    }
}

fn fresh() -> (Runtime, Document) {
    let mut runtime = Runtime::try_new().unwrap();
    runtime.steps = MAX_STEPS;
    (runtime, Document::parse("<p>kept</p>"))
}
fn object(runtime: &mut Runtime, keys: &[JsString]) -> Value {
    runtime
        .object_ordered(keys.iter().cloned().map(|key| (key, Value::Bool(true))))
        .unwrap()
}
fn reset(runtime: &mut Runtime) {
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
}
fn spent(runtime: &Runtime) -> (usize, usize) {
    (MAX_STEPS - runtime.steps, runtime.allocated)
}
fn retain<'a>(visited: &'a VisitedNames, key: &JsString) -> &'a JsString {
    visited.buckets[&key.len()].get_key_value(key).unwrap().0
}
fn no_reader(_: &mut Runtime, _: &Value, _: &JsString) -> Result<Option<Property>> {
    panic!("duplicate must not dispatch a descriptor reader")
}
fn refused_reader(_: &mut Runtime, _: &Value, _: &JsString) -> Result<Option<Property>> {
    Err(ScriptError::resource("private descriptor refusal"))
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

#[test]
fn singleton_first_name_and_equal_utf16_keep_first_payload() {
    for units in [vec![], vec![0xd800], vec![0xd800; 512]] {
        let (mut runtime, _) = fresh();
        let key = JsString::from(units);
        let target = object(&mut runtime, std::slice::from_ref(&key));
        let mut visited = VisitedNames::default();
        reset(&mut runtime);
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &key)
                .unwrap()
                .is_some()
        );
        assert_eq!(spent(&runtime), (31, VISITED_BUCKET_BYTES));
        assert!(matches!(visited.buckets[&key.len()], NameBucket::Single(_)));
        let pointer = key.units().as_ptr();
        assert_eq!(retain(&visited, &key).units().as_ptr(), pointer);
        let equal = JsString::from(key.units().to_vec());
        let before = visited.clone();
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .for_in_visit_using(&mut visited, &target, &equal, no_reader)
                .unwrap()
                .is_none()
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(visited, before);
        assert_eq!(retain(&visited, &key).units().as_ptr(), pointer);
    }
}

#[test]
fn singleton_utf16_distinctions_and_both_promotion_orders() {
    for units in [
        vec![vec![0xd800], vec![0], vec![0xdc00], vec![0xfffd]],
        vec![vec![0], vec![0xd800], vec![0xfffd], vec![0xdc00]],
        vec![vec![0xd800, 0xdc00], vec![0xdc00, 0xd800]],
        vec![
            vec![0xd800; 512],
            [vec![0xd800; 511], vec![0xdc00]].concat(),
        ],
    ] {
        let keys: Vec<_> = units.into_iter().map(JsString::from).collect();
        let (mut runtime, _) = fresh();
        let target = object(&mut runtime, &keys);
        let mut visited = VisitedNames::default();
        for (i, key) in keys.iter().enumerate() {
            reset(&mut runtime);
            assert!(
                runtime
                    .for_in_visit(&mut visited, &target, key)
                    .unwrap()
                    .is_some()
            );
            if i == 1 {
                assert_eq!(spent(&runtime), (64 + 2 * key.len(), VISITED_NAME_BYTES));
            }
            assert_eq!(retain(&visited, key).units().as_ptr(), key.units().as_ptr());
            if i == 0 {
                assert!(matches!(visited.buckets[&key.len()], NameBucket::Single(_)));
            } else {
                assert!(matches!(visited.buckets[&key.len()], NameBucket::Tree(_)));
            }
            for earlier in &keys[..=i] {
                assert_eq!(
                    retain(&visited, earlier).units().as_ptr(),
                    earlier.units().as_ptr()
                );
            }
        }
        let before = visited.clone();
        runtime.allocated = MAX_HEAP;
        for key in &keys {
            let equal = JsString::from(key.units().to_vec());
            assert!(
                runtime
                    .for_in_visit_using(&mut visited, &target, &equal, no_reader)
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(visited, before);
        assert_eq!(runtime.allocated, MAX_HEAP);
    }
}

#[test]
fn singleton_duplicate_missing_and_failed_reader_never_promote() {
    let (mut runtime, _) = fresh();
    let aa = JsString::from("aa");
    let bb = JsString::from("bb");
    let target = object(&mut runtime, std::slice::from_ref(&aa));
    let missing = object(&mut runtime, &[]);
    let mut visited = VisitedNames::default();
    runtime
        .for_in_visit(&mut visited, &target, &aa)
        .unwrap()
        .unwrap();
    let before = visited.clone();
    for duplicate in [false, true] {
        reset(&mut runtime);
        runtime.allocated = MAX_HEAP;
        let key = if duplicate { &aa } else { &bb };
        let value = if duplicate {
            runtime.for_in_visit_using(&mut visited, &missing, key, no_reader)
        } else {
            runtime.for_in_visit(&mut visited, &missing, key)
        };
        assert!(value.unwrap().is_none());
        assert_eq!(MAX_STEPS - runtime.steps, 9);
        assert_eq!(runtime.allocated, MAX_HEAP);
        assert_eq!(visited, before);
    }
    reset(&mut runtime);
    assert!(
        runtime
            .for_in_visit_using(&mut visited, &target, &bb, refused_reader)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(visited, before);
    assert_eq!(runtime.allocated, 0);
    // A different missing length keeps the exact outer-only path.
    runtime.steps = 4;
    runtime.allocated = MAX_HEAP;
    assert!(
        runtime
            .for_in_visit(&mut visited, &missing, &"long".into())
            .unwrap()
            .is_none()
    );
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    assert_eq!(visited, before);
}

#[test]
fn singleton_promotion_all_stage_cuts_retain_original_single() {
    for (first, second) in [("aa", "bb"), ("bb", "aa")] {
        let prepare = || {
            let (mut runtime, doc) = fresh();
            let old = JsString::from(first);
            let new = JsString::from(second);
            let target = object(&mut runtime, &[old.clone(), new.clone()]);
            let mut visited = VisitedNames::default();
            runtime
                .for_in_visit(&mut visited, &target, &old)
                .unwrap()
                .unwrap();
            (runtime, doc, target, old, new, visited)
        };
        // 4 outer +2 enum +3 equality +59 complete promotion. Reader costs0.
        for steps in [0, 3, 5, 8, 9, 67, 68] {
            let (mut runtime, _, target, old, new, mut visited) = prepare();
            let before = visited.clone();
            let object_count = runtime.objects.len();
            let id = runtime.property_object(&target).unwrap();
            let order = runtime.objects[id].order.clone();
            runtime.steps = steps;
            runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES;
            let result = runtime.for_in_visit(&mut visited, &target, &new);
            if steps == 68 {
                assert!(result.unwrap().is_some());
                assert_eq!(runtime.steps, 0);
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert!(matches!(visited.buckets[&2], NameBucket::Tree(_)));
                assert_eq!(
                    retain(&visited, &new).units().as_ptr(),
                    new.units().as_ptr()
                );
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                assert_eq!(visited, before);
            }
            assert_eq!(
                retain(&visited, &old).units().as_ptr(),
                old.units().as_ptr()
            );
            assert_eq!(runtime.objects.len(), object_count);
            assert_eq!(runtime.objects[id].order, order);
            clean(&runtime);
        }
        let (mut runtime, _, target, old, new, mut visited) = prepare();
        let before = visited.clone();
        runtime.steps = 68;
        runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES + 1;
        assert!(
            runtime
                .for_in_visit(&mut visited, &target, &new)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(visited, before);
        assert_eq!(
            retain(&visited, &old).units().as_ptr(),
            old.units().as_ptr()
        );
    }
}

#[test]
fn singleton_hidden_names_shadow_but_deleted_names_remain_eligible() {
    let source = r#"(function(){
        var tail={aa:1,bb:2,cc:3,tail:4}, middle=Object.create(tail), calls=0;
        Object.defineProperty(middle,'aa',{get:function(){calls++;throw 'getter';},enumerable:false});
        Object.defineProperty(middle,'bb',{get:function(){calls++;throw 'getter';},enumerable:false});
        var source=Object.create(middle);source.first=1;source.cc=2;
        var seen='';for(var key in source){seen+=key+'|';if(key==='first')delete source.cc;}
        return seen==='first|cc|tail|'&&calls===0;
    })()"#;
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        clean(&runtime);
    }
}

#[test]
fn singleton_general_tree_path_preserves_growth_and_cuts() {
    for count in [2, 10, 11, 12, 70, 71, 72] {
        let keys: Vec<JsString> = (0..=count).map(|i| format!("p{i:03}").into()).collect();
        let (mut runtime, _) = fresh();
        let target = object(&mut runtime, &keys);
        let mut seed = VisitedNames::default();
        for key in &keys[..count] {
            runtime
                .for_in_visit(&mut seed, &target, key)
                .unwrap()
                .unwrap();
        }
        assert!(matches!(seed.buckets[&4], NameBucket::Tree(_)));
        let key = &keys[count];
        let mut visited = seed.clone();
        reset(&mut runtime);
        runtime
            .for_in_visit(&mut visited, &target, key)
            .unwrap()
            .unwrap();
        let (comparisons, nodes) = tree_bound(count);
        let work = 4 + 2 + (1 + nodes + comparisons * 5) + (1 + (nodes + 1) * 24);
        assert_eq!(spent(&runtime), (work, VISITED_NAME_BYTES));
        for heap_cut in [false, true] {
            let mut visited = seed.clone();
            runtime.steps = if heap_cut { work } else { work - 1 };
            runtime.allocated = if heap_cut {
                MAX_HEAP - VISITED_NAME_BYTES + 1
            } else {
                0
            };
            assert!(
                runtime
                    .for_in_visit(&mut visited, &target, key)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(visited, seed);
        }
        for earlier in &keys[..count] {
            assert_eq!(
                retain(&seed, earlier).units().as_ptr(),
                earlier.units().as_ptr()
            );
        }
    }
}

#[test]
fn singleton_outer_search_and_cumulative_storage_are_independent() {
    let (mut runtime, _) = fresh();
    let keys: Vec<JsString> = ["aa", "bb", "cc"].into_iter().map(JsString::from).collect();
    let target = object(&mut runtime, &keys);
    let mut visited = VisitedNames::default();
    reset(&mut runtime);
    runtime
        .for_in_visit(&mut visited, &target, &keys[0])
        .unwrap()
        .unwrap();
    assert_eq!(spent(&runtime), (31, VISITED_BUCKET_BYTES));
    runtime.steps = 68;
    runtime.allocated = MAX_HEAP - VISITED_NAME_BYTES;
    runtime
        .for_in_visit(&mut visited, &target, &keys[1])
        .unwrap()
        .unwrap();
    assert_eq!(runtime.steps, 0);
    assert_eq!(runtime.allocated, MAX_HEAP);
    let before = visited.clone();
    runtime.steps = 63;
    assert!(
        runtime
            .for_in_visit(&mut visited, &target, &keys[2])
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(visited, before);
    // A fresh independent set cannot reuse the previous outer allocation.
    let mut other = VisitedNames::default();
    runtime.steps = 31;
    assert!(
        runtime
            .for_in_visit(&mut other, &target, &keys[0])
            .unwrap_err()
            .is_resource_limit()
    );
    assert!(other.buckets.is_empty());
    println!(
        "SINGLETON_STORAGE bucket={} tree={} visited={} inner_allowance={} outer_allowance={}",
        std::mem::size_of::<NameBucket>(),
        std::mem::size_of::<BTreeMap<JsString, ()>>(),
        std::mem::size_of::<VisitedNames>(),
        VISITED_NAME_BYTES,
        VISITED_BUCKET_BYTES
    );
}

#[test]
fn singleton_machine_interruptions_keep_prior_target_and_body_effects() {
    let unit = parser::Parser::program(
        "try{for(box.key in source){seen++;}}catch(e){caught++;}finally{finalized++;}seen",
    )
    .unwrap();
    let prepare = || {
        let (mut runtime, mut doc) = fresh();
        runtime.execute("var seen=0,assigned=0,caught=0,finalized=0;var source={aa:1,bb:2,long:3};var box={set key(v){assigned++;}}",&mut doc).unwrap();
        runtime.steps = MAX_STEPS;
        (runtime, doc)
    };
    let run = |runtime: &mut Runtime, doc: &mut Document| -> Result<Value> {
        match machine::evaluate_statements(runtime, &unit, machine::ListOwner::Program, 1, doc)? {
            Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
            _ => panic!("normal completion"),
        }
    };
    let (mut measured, mut doc) = prepare();
    assert_eq!(run(&mut measured, &mut doc).unwrap(), Value::Number(3.0));
    let used = MAX_STEPS - measured.steps;
    let mut after_first = false;
    for allowance in (0..used).step_by(7) {
        let (mut runtime, mut doc) = prepare();
        runtime.steps = allowance;
        assert!(run(&mut runtime, &mut doc).unwrap_err().is_resource_limit());
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        let Value::Number(seen) = runtime.lookup(1, "seen").unwrap().1 else {
            panic!("seen")
        };
        if seen < 3.0 {
            assert_eq!(
                runtime.lookup(1, "finalized").unwrap().1,
                Value::Number(0.0)
            );
        }
        if seen == 1.0 && runtime.lookup(1, "assigned").unwrap().1 == Value::Number(1.0) {
            after_first = true;
        }
        clean(&runtime);
    }
    assert!(after_first);
}
