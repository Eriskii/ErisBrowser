use crate::script::{AbortSlot, Binding, MAX_HEAP, Native, Property, PropertyKey, Runtime};
use std::mem::size_of;

const INSTALL_WORK: usize = 1_103;

fn before_install() -> Runtime {
    let mut runtime = Runtime::uninitialized();
    // Execute the real preceding initialization. The first new installer debit
    // then refuses, before retaining a key or publishing an AggregateError bag.
    runtime.steps = 99_694;
    assert!(
        runtime
            .initialize_bootstrap()
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
    assert!(runtime.objects.capacity() >= 754);
    publication(&runtime, 752, false, false, false);
    runtime
}

fn publication(runtime: &Runtime, objects: usize, prototype: bool, native: bool, global: bool) {
    assert_eq!(runtime.objects.len(), objects);
    assert_eq!(runtime.prototypes.len(), 25 + usize::from(prototype));
    assert_eq!(runtime.native_properties.len(), 299 + usize::from(native));
    assert_eq!(
        runtime.environments[0].bindings.len(),
        212 + usize::from(global)
    );
    assert_eq!(runtime.prototypes.contains_key("AggregateError"), prototype);
    assert_eq!(
        runtime.native_properties.contains_key("AggregateError"),
        native
    );
    assert_eq!(
        runtime.environments[0]
            .bindings
            .contains_key("AggregateError"),
        global
    );
    assert_eq!(runtime.next_global_order, 212 + u64::from(global));
    assert_eq!(runtime.calls, 0);
    assert!(runtime.frames.is_empty());
}

fn node<K, V>() -> usize {
    16 * (size_of::<K>() + size_of::<V>()) + 32 * size_of::<usize>() + 64
}

fn metadata_heap() -> usize {
    2 * size_of::<Option<AbortSlot>>()
        + 2 * size_of::<Option<f64>>()
        + 6 * size_of::<PropertyKey>()
        + 2 * node::<PropertyKey, Property>()
        + size_of::<Native>()
        + 2_542
}

fn through_prototype_heap() -> usize {
    metadata_heap() + 3 * node::<&'static str, usize>()
}

fn through_native_heap() -> usize {
    through_prototype_heap() + 14 + 4 * node::<String, usize>() + 32
}

fn install_heap() -> usize {
    through_native_heap() + 14 + 4 * node::<String, Binding>() + 32
}

#[test]
fn aggregate_installer_work_admission_precedes_bags_and_external_publication() {
    // The thresholds come from the selected installer ledger, not a measured
    // candidate. Reached private bags may remain after refusal; external maps
    // advance only after their own charged publication stage.
    for (budget, objects, prototype, native, global) in [
        (0, 752, false, false, false),
        (18, 752, false, false, false),
        (419, 752, false, false, false),
        (427, 752, false, false, false),
        (428, 753, false, false, false),
        (436, 753, false, false, false),
        (437, 754, false, false, false),
        (565, 754, false, false, false),
        (700, 754, false, false, false),
        (701, 754, true, false, false),
        (901, 754, true, false, false),
        (902, 754, true, true, false),
        (1_102, 754, true, true, false),
        (1_103, 754, true, true, true),
    ] {
        let mut runtime = before_install();
        let initial_heap = runtime.allocated;
        let capacity = runtime.objects.capacity();
        runtime.steps = budget;
        let result = runtime.install_aggregate_error_intrinsics();
        if global {
            result.unwrap();
            assert_eq!(runtime.allocated - initial_heap, install_heap());
        } else {
            assert!(result.unwrap_err().is_resource_limit(), "budget={budget}");
            assert!(runtime.allocated >= initial_heap);
            assert!(runtime.allocated - initial_heap <= install_heap());
        }
        assert_eq!(runtime.steps, 0, "budget={budget}");
        assert_eq!(runtime.objects.capacity(), capacity);
        publication(&runtime, objects, prototype, native, global);
    }
}

#[test]
fn aggregate_installer_heap_admission_precedes_each_new_publication() {
    let header = 72 + size_of::<Option<AbortSlot>>() + size_of::<Option<f64>>();
    let order = 3 * size_of::<PropertyKey>();
    let pool = 652;
    let prototype_bag = pool + header + order;
    let constructor_bag = prototype_bag + header + order;
    for (available, spent, objects, prototype, native, success) in [
        (87, 271, 752, false, false, false),
        (pool + header - 1, 427, 752, false, false, false),
        (prototype_bag - 1, 428, 752, false, false, false),
        (prototype_bag + header - 1, 436, 753, false, false, false),
        (constructor_bag - 1, 437, 753, false, false, false),
        (through_prototype_heap() - 1, 701, 754, false, false, false),
        (through_native_heap() - 1, 902, 754, true, false, false),
        (install_heap() - 1, 1_103, 754, true, true, false),
        (install_heap(), 1_103, 754, true, true, true),
    ] {
        let mut runtime = before_install();
        let capacity = runtime.objects.capacity();
        runtime.steps = INSTALL_WORK;
        runtime.allocated = MAX_HEAP - available;
        let result = runtime.install_aggregate_error_intrinsics();
        if success {
            result.unwrap();
            assert_eq!(runtime.allocated, MAX_HEAP);
        } else {
            assert!(
                result.unwrap_err().is_resource_limit(),
                "available={available}"
            );
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
        }
        assert_eq!(runtime.steps, INSTALL_WORK - spent);
        assert_eq!(runtime.objects.capacity(), capacity);
        publication(&runtime, objects, prototype, native, success);
    }
}
