use super::*;

fn fixture() -> Runtime {
    let mut runtime = Runtime::new();
    for name in [
        "bucketZ",
        "bucketA",
        "01",
        "10",
        "2",
        "4294967294",
        "4294967295",
    ] {
        runtime
            .store_window_property(
                &name.into(),
                Property::data(Value::Number(1.0), true, true, true),
            )
            .unwrap();
    }
    runtime
        .store_window_property(
            &JsString::from(vec![0xd800, 65]),
            Property::data(Value::Number(2.0), true, true, true),
        )
        .unwrap();
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    runtime
}

fn snapshot(runtime: &mut Runtime) -> (Vec<JsString>, usize, usize) {
    runtime.steps = MAX_STEPS;
    runtime.allocated = 0;
    let keys = runtime.window_own_keys().unwrap();
    (keys, MAX_STEPS - runtime.steps, runtime.allocated)
}

#[test]
fn dense_buckets_match_sparse_sort_and_literal_numeric_creation_order() {
    let mut runtime = fixture();
    let order = runtime.next_global_order;
    let (dense, dense_work, dense_bytes) = snapshot(&mut runtime);
    assert_eq!(
        &dense[..3],
        &[JsString::from("2"), "10".into(), "4294967294".into()]
    );
    assert_eq!(
        &dense[dense.len() - 5..],
        &[
            JsString::from("bucketZ"),
            "bucketA".into(),
            "01".into(),
            "4294967295".into(),
            JsString::from(vec![0xd800, 65]),
        ]
    );
    runtime.next_global_order = u64::MAX;
    let (sparse, sparse_work, sparse_bytes) = snapshot(&mut runtime);
    assert_eq!(dense, sparse);
    assert!(
        dense_work < sparse_work,
        "dense={dense_work}, sparse={sparse_work}"
    );
    assert_eq!(
        dense_bytes - sparse_bytes,
        order as usize * std::mem::size_of::<Option<JsString>>()
    );
    runtime.next_global_order = order;
    runtime
        .window_write_data(&"bucketZ".into(), Value::Number(9.0))
        .unwrap();
    assert_eq!(snapshot(&mut runtime).0, dense);
    assert!(
        runtime
            .delete_property(Value::Window, &"bucketZ".into())
            .unwrap()
    );
    runtime
        .store_window_property(
            &"bucketZ".into(),
            Property::data(Value::Number(10.0), true, true, true),
        )
        .unwrap();
    let moved = snapshot(&mut runtime).0;
    assert_eq!(moved.last(), Some(&JsString::from("bucketZ")));
    assert_eq!(moved.len(), dense.len());
    let original_without: Vec<_> = dense
        .into_iter()
        .filter(|key| key != &JsString::from("bucketZ"))
        .collect();
    assert_eq!(&moved[..moved.len() - 1], original_without.as_slice());
}

#[test]
fn dense_and_sparse_exact_work_and_heap_cuts_preserve_window_state() {
    for sparse in [false, true] {
        let mut runtime = fixture();
        if sparse {
            runtime.next_global_order = u64::MAX;
        }
        let count = runtime.environments[0].bindings.len();
        let order = runtime.next_global_order;
        let (expected, work, bytes) = snapshot(&mut runtime);
        assert!(work > 0 && bytes > 0);
        runtime.steps = work;
        runtime.allocated = MAX_HEAP - bytes;
        assert_eq!(runtime.window_own_keys().unwrap(), expected);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.allocated, MAX_HEAP);

        runtime.steps = work - 1;
        runtime.allocated = 0;
        assert!(runtime.window_own_keys().unwrap_err().is_resource_limit());
        assert_eq!(runtime.environments[0].bindings.len(), count);
        assert_eq!(runtime.next_global_order, order);
        assert_eq!(runtime.global_non_scalar.len(), 1);

        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP - bytes + 1;
        assert!(runtime.window_own_keys().unwrap_err().is_resource_limit());
        assert_eq!(runtime.environments[0].bindings.len(), count);
        assert_eq!(runtime.next_global_order, order);
        assert_eq!(snapshot(&mut runtime).0, expected);
    }
}

#[test]
fn long_scalar_and_non_scalar_keys_survive_both_ordering_routes() {
    let mut runtime = fixture();
    let long = format!("long{}", "x".repeat(2048));
    let long_key = JsString::from(long.as_str());
    runtime
        .store_window_property(&long_key, Property::data(Value::Null, true, false, true))
        .unwrap();
    let order = runtime.next_global_order;
    let dense = snapshot(&mut runtime).0;
    assert_eq!(dense.last(), Some(&long_key));
    assert_eq!(dense.iter().filter(|key| *key == &long_key).count(), 1);
    runtime.next_global_order = u64::MAX;
    let sparse = snapshot(&mut runtime).0;
    assert_eq!(dense, sparse);
    runtime.next_global_order = order;
    assert!(
        !runtime
            .window_reflected_property(&long_key)
            .unwrap()
            .unwrap()
            .enumerable
    );
}
