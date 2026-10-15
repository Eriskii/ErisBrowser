//! Source-derived installer cuts; fixture setup is manual Rust, not author code.
use crate::script::{
    AbortSlot, Binding, MAX_HEAP, MAX_STEPS, Native, Property, PropertyKey, PropertyValue, Runtime,
    Value,
};
use std::rc::Rc;

fn helper_bytes() -> usize {
    use std::mem::size_of;
    let a = size_of::<Option<AbortSlot>>();
    let d = size_of::<Option<f64>>();
    let k = size_of::<PropertyKey>();
    let n = size_of::<Native>();
    let bpp = 16 * (k + size_of::<Property>()) + 32 * size_of::<usize>() + 64;
    let bsb = 16 * (size_of::<String>() + size_of::<Binding>()) + 32 * size_of::<usize>() + 64;
    // Excludes the new State header and the eleven initial ScriptObject arena
    // slots. Their charges belong to uninitialized/reserve_bootstrap_objects.
    11 * a + 11 * d + 32 * k + 11 * bpp + 10 * n + 4 * bsb + 11450
}

fn empty_arenas(runtime: &Runtime) {
    macro_rules! empty {
        ($field:ident) => {
            assert!(runtime.promises.$field.records.is_empty());
            assert_eq!(runtime.promises.$field.records.capacity(), 0);
            assert_eq!(runtime.promises.$field.admitted_capacity, 0);
        };
    }
    empty!(promises);
    empty!(gates);
    empty!(callbacks);
    empty!(captures);
    empty!(jobs);
    empty!(finally_captures);
    empty!(all_captures);
    assert!(runtime.microtasks.queue.is_empty());
    assert_eq!(runtime.microtasks.queue.capacity(), 0);
    assert_eq!(runtime.microtasks.admitted_capacity, 0);
}

fn coherent_intrinsics(runtime: &Runtime) {
    let intrinsics = runtime.promises.intrinsics.as_ref().unwrap();
    let global = &runtime.environments[0].bindings["Promise"];
    assert!(global.mutable && global.global_property && global.deletable);
    assert!(!global.enumerable);
    let Value::Native(global_native) = &global.value else {
        panic!("Promise global is not the installed native");
    };
    let Value::Native(original_native) = &intrinsics.constructor else {
        panic!("private constructor is not the installed native");
    };
    assert!(Rc::ptr_eq(global_native, original_native));
    assert_eq!(original_native.name, "Promise");
    assert_eq!(original_native.receiver, Value::Window);
    let owner = original_native.properties.unwrap().get();
    let prototype = &runtime.objects[owner].values[&PropertyKey::from("prototype")];
    assert!(matches!(
        &prototype.value,
        PropertyValue::Data { value: Value::Object(id), .. } if *id == intrinsics.prototype
    ));
    let constructor =
        &runtime.objects[intrinsics.prototype].values[&PropertyKey::from("constructor")];
    let PropertyValue::Data {
        value: Value::Native(back_reference),
        ..
    } = &constructor.value
    else {
        panic!("prototype.constructor is not the original native");
    };
    assert!(Rc::ptr_eq(back_reference, original_native));
    assert!(!runtime.native_properties.contains_key("Promise"));
    assert!(!runtime.prototypes.contains_key("Promise"));
}

fn replay_fixture() -> Runtime {
    let mut runtime = Runtime::try_new().unwrap();
    empty_arenas(&runtime);
    coherent_intrinsics(&runtime);
    assert_eq!(runtime.objects.len(), 752);
    assert!(runtime.environments[0].bindings.remove("Promise").is_some());
    assert!(runtime.promises.intrinsics.take().is_some());
    // Inert Rust fixture setup: retain all old bags, reserve eleven additional
    // slots and then reset counters. This does not resume author callbacks,
    // assert allocation rollback, or alter the production admission policy.
    runtime.objects.try_reserve_exact(11).unwrap();
    assert!(runtime.objects.capacity() >= runtime.objects.len() + 11);
    assert_eq!(runtime.environments[0].bindings.len(), 211);
    runtime.steps = 0;
    runtime.allocated = 0;
    runtime
}

#[test]
fn promise_installer_exact1570_and_one_short1569_precede_global_cache_publication() {
    for (work, accepted) in [(1570, true), (1569, false)] {
        let mut runtime = replay_fixture();
        let capacity = runtime.objects.capacity();
        runtime.steps = work;
        let result = runtime.install_promise_intrinsics();
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.capacity(), capacity);
        assert_eq!(runtime.native_properties.len(), 299);
        assert_eq!(runtime.prototypes.len(), 25);
        empty_arenas(&runtime);
        if accepted {
            assert_eq!(runtime.allocated, helper_bytes());
            assert_eq!(runtime.objects.len(), 763);
            assert_eq!(runtime.environments[0].bindings.len(), 212);
            coherent_intrinsics(&runtime);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert!(runtime.allocated <= helper_bytes());
            assert_eq!(runtime.environments[0].bindings.len(), 211);
            assert!(!runtime.environments[0].bindings.contains_key("Promise"));
            assert!(runtime.promises.intrinsics.is_none());
            // Already appended private metadata bags are intentionally not
            // required to roll back after an initialization refusal.
        }
        assert!(runtime.frames.is_empty());
        assert!(runtime.console.is_empty());
    }
}

#[test]
fn promise_installer_exact_helper_heap_and_one_short_keep_refused_realm_unpublished() {
    let bytes = helper_bytes();
    for (allowance, accepted) in [(bytes, true), (bytes - 1, false)] {
        let mut runtime = replay_fixture();
        let capacity = runtime.objects.capacity();
        runtime.steps = 1570;
        runtime.allocated = MAX_HEAP - allowance;
        let result = runtime.install_promise_intrinsics();
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(runtime.steps, 0);
        assert_eq!(runtime.objects.capacity(), capacity);
        assert_eq!(runtime.native_properties.len(), 299);
        assert_eq!(runtime.prototypes.len(), 25);
        empty_arenas(&runtime);
        if accepted {
            assert_eq!(runtime.allocated, MAX_HEAP);
            assert_eq!(runtime.objects.len(), 763);
            assert_eq!(runtime.environments[0].bindings.len(), 212);
            coherent_intrinsics(&runtime);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
            assert_eq!(runtime.environments[0].bindings.len(), 211);
            assert!(!runtime.environments[0].bindings.contains_key("Promise"));
            assert!(runtime.promises.intrinsics.is_none());
        }
        assert!(runtime.frames.is_empty());
        assert!(runtime.console.is_empty());
    }
}

#[test]
fn promise_full_raw99973_and_public_reset_preserve_empty_paid_arenas() {
    for (work, accepted, remaining) in [(99_972, false, 0), (99_973, true, 0), (100_000, true, 27)]
    {
        let mut runtime = Runtime::uninitialized();
        empty_arenas(&runtime);
        assert!(runtime.promises.intrinsics.is_none());
        runtime.steps = work;
        match runtime.finish_bootstrap() {
            Ok(runtime) => {
                assert!(accepted);
                assert_eq!(runtime.steps, remaining);
                assert_eq!(
                    (runtime.objects.len(), runtime.objects.capacity()),
                    (752, 752)
                );
                assert_eq!(runtime.environments[0].bindings.len(), 212);
                assert_eq!(runtime.native_properties.len(), 299);
                assert_eq!(runtime.prototypes.len(), 25);
                assert!(runtime.allocated < MAX_HEAP);
                empty_arenas(&runtime);
                coherent_intrinsics(&runtime);
                assert!(runtime.frames.is_empty());
                assert!(runtime.console.is_empty());
            }
            Err(error) => {
                assert!(!accepted);
                assert!(error.is_resource_limit());
            }
        }
    }
    let runtime = Runtime::try_new().unwrap();
    assert_eq!(runtime.steps, MAX_STEPS);
    empty_arenas(&runtime);
    coherent_intrinsics(&runtime);
}
