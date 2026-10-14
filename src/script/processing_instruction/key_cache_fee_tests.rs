//! Candidate private-API boundary; add only after the borrowed-key seam exists.
use super::key_cache_tests::{CASES, cached_keys};
use super::*;
use std::mem::size_of;

#[test]
fn cached_pi_function_fees_are_exact_and_one_short_refuses_the_callable() {
    let leaf =
        16 * (size_of::<PropertyKey>() + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let shared = size_of::<Option<AbortSlot>>()
        + size_of::<Option<f64>>()
        + 2 * size_of::<PropertyKey>()
        + leaf
        + size_of::<Native>();
    for case in &CASES {
        let heap = shared + case.heap_constant;
        for (work, bytes, success) in [
            (case.work, heap, true),
            (case.work - 1, heap, false),
            (case.work, heap - 1, false),
        ] {
            let mut runtime = Runtime::try_new().unwrap();
            let keys = cached_keys(&runtime);
            // Isolate this function factory: ordinary bootstrap has already
            // reserved its exact arena; the test prepares one spare slot.
            runtime.objects.try_reserve_exact(1).unwrap();
            let old_objects = runtime.objects.len();
            let old_registry = runtime.native_properties.clone();
            let old_global_count = runtime.environments[0].bindings.len();
            let old_order = runtime.next_global_order;
            runtime.steps = work;
            runtime.allocated = MAX_HEAP - bytes;
            let result =
                runtime.pi_function(case.suffix, case.display, case.arity, [&keys[0], &keys[1]]);
            assert_eq!(runtime.steps, 0);
            assert_eq!(runtime.objects.len(), old_objects + 1);
            assert_eq!(runtime.native_properties, old_registry);
            assert_eq!(runtime.environments[0].bindings.len(), old_global_count);
            assert_eq!(runtime.next_global_order, old_order);
            if success {
                let Value::Native(native) = result.unwrap() else {
                    panic!("native");
                };
                assert_eq!(runtime.allocated, MAX_HEAP);
                assert_eq!(native.properties.unwrap().get(), old_objects);
                assert_eq!(runtime.objects[old_objects].order, keys);
                assert_eq!(runtime.objects[old_objects].values.len(), 2);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
                if bytes < heap {
                    assert_eq!(runtime.allocated, MAX_HEAP + 1);
                } else {
                    let native_bytes = size_of::<Native>() + 32 + 7 + case.suffix.len();
                    assert_eq!(runtime.allocated, MAX_HEAP - native_bytes);
                }
            }
        }
    }
}
