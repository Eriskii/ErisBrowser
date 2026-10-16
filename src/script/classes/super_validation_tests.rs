//! Class superclass validation must precede bound argument forwarding.
use super::*;

#[test]
fn class_private_super_validation_checks_bound_nonconstructor_without_heap_or_calls() {
    for (name, work, success, ordinary_error) in [
        ("bound", 2, false, false),
        ("bound", 3, false, true),
        ("Ordinary", 1, false, false),
        ("Ordinary", 2, true, false),
    ] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("");
        runtime.execute(
            "var reached=0; var target=()=>{reached++;}; var bound=target.bind(null,{marker:1},{marker:2}); function Ordinary(){reached++;}",
            &mut doc,
        ).unwrap();
        let superclass = runtime.environments[0].bindings[name].value.clone();
        let Value::Function(bound_id) = runtime.environments[0].bindings["bound"].value else {
            panic!("bound function");
        };
        let retained = runtime.functions[bound_id].bound.as_ref().unwrap();
        let arguments = retained.arguments.as_ptr();
        assert_eq!(retained.arguments.len(), 2);
        let Value::Function(arrow_id) = retained.target else {
            panic!("arrow target");
        };
        assert!(runtime.functions[arrow_id].code.arrow);
        assert!(!runtime.functions[arrow_id].code.constructable);
        let before = (
            runtime.functions.len(),
            runtime.objects.len(),
            runtime.environments.len(),
        );
        let guards = (
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units,
            runtime.frames.len(),
        );
        runtime.steps = work;
        runtime.allocated = MAX_HEAP; // Zero available heap; validation owns no allocation.
        let result = runtime.class_validate_superclass(&superclass);
        if success {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            if ordinary_error {
                assert_eq!(error.name(), "TypeError");
                assert!(!error.is_resource_limit() && !error.is_unsupported());
            } else {
                assert!(error.is_resource_limit());
            }
        }
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!(
            (
                runtime.functions.len(),
                runtime.objects.len(),
                runtime.environments.len()
            ),
            before
        );
        assert_eq!(
            (
                runtime.calls,
                runtime.eval_depth,
                runtime.stack_units,
                runtime.frames.len()
            ),
            guards
        );
        let retained = runtime.functions[bound_id].bound.as_ref().unwrap();
        assert_eq!(retained.arguments.as_ptr(), arguments);
        assert_eq!(retained.arguments.len(), 2);
        assert_eq!(
            runtime.environments[0].bindings["reached"].value,
            Value::Number(0.0)
        );
        assert_eq!(runtime.environments[0].bindings[name].value, superclass);
    }
}
