//! Actual phase routing, malformed internal records, and unchanged old storage.
use super::*;
use crate::script::{MAX_STACK_UNITS, MAX_STEPS, parser::Parser, regexp};
use call_spread::{Arguments, prepay_entry, prepay_item, receive_prepaid};
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use std::mem::align_of;
use std::mem::size_of;

fn resource<T>(result: Result<T>) {
    match result {
        Err(error) => assert!(error.is_resource_limit(), "{error:?}"),
        Ok(_) => panic!("expected ResourceLimit"),
    }
}
fn expression(source: &str) -> (Rc<code::Unit>, code::ExprId) {
    let unit = Parser::program(source).unwrap();
    let code::Stmt::Expr(id) = unit.stmt(unit.body[0]) else {
        panic!("expression")
    };
    let id = *id;
    (unit, id)
}
fn frame(unit: Rc<code::Unit>, expression: code::ExprId) -> ExprFrame {
    ExprFrame {
        unit,
        expression,
        env: 0,
        reference: false,
        phase: Phase::Start,
    }
}
fn clean(runtime: &Runtime) {
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.calls,
            runtime.eval_depth,
            runtime.stack_units
        ),
        (0, 0, 0, 0)
    );
}
fn paid_values(runtime: &mut Runtime, items: Vec<Value>) -> Arguments {
    let before = runtime.steps;
    prepay_entry(runtime).unwrap();
    assert_eq!(before - runtime.steps, 8);
    let mut values = Arguments::empty_prepaid();
    for value in items {
        let before = runtime.steps;
        prepay_item(runtime).unwrap();
        assert_eq!(before - runtime.steps, 6);
        values
            .append_received(runtime, receive_prepaid(value))
            .unwrap();
    }
    values
}
fn refs(value: &Value) -> usize {
    let Value::Native(native) = value else {
        panic!("native")
    };
    Rc::strong_count(native)
}

#[test]
fn spread_start_entry_is_paid_after_old_tick_before_any_callee_evaluation() {
    for source in [
        "target(...source)",
        "holder.target(...source)",
        "new Target(...source)",
    ] {
        for (work, ok) in [(8, false), (9, true)] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            runtime
                .execute(
                    "var trace='';var holder={get target(){trace+='G';return function(){};}};",
                    &mut document,
                )
                .unwrap();
            let (unit, id) = expression(source);
            runtime.steps = work;
            let heap = runtime.allocated;
            let result = start(&mut runtime, frame(unit, id), &mut document);
            assert_eq!(result.is_ok(), ok);
            if !ok {
                resource(result);
                assert!(runtime.frames.is_empty());
            } else {
                assert!(result.unwrap().is_none());
                assert_eq!(runtime.frames.len(), 2);
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, heap));
            // Successful start only schedules; no callee/receiver is evaluated.
            // These direct-continuation fixtures restore their queued frames.
            runtime.frames.clear();
            runtime.eval_depth = 0;
            assert_eq!(
                runtime.execute("trace", &mut document).unwrap(),
                Value::String("".into())
            );
            clean(&runtime);
        }
    }
}

#[test]
fn reached_item_fee_schedules_only_after_six_prepaid_units() {
    for source in [
        "target(item,...source)",
        "target(...source,item)",
        "new Target(...source,item)",
    ] {
        for (work, ok) in [(5, false), (6, true)] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("");
            let (unit, id) = expression(source);
            let values = paid_values(&mut runtime, Vec::new());
            runtime.steps = work;
            let heap = runtime.allocated;
            let result = spread_arguments_next(
                &mut runtime,
                frame(unit, id),
                0,
                Value::Undefined,
                Value::Undefined,
                values,
                &mut document,
            );
            assert_eq!(result.is_ok(), ok);
            if !ok {
                resource(result);
                assert!(runtime.frames.is_empty());
            } else {
                assert!(result.unwrap().is_none());
                assert_eq!(runtime.frames.len(), 2);
                let Frame::Expression(parent) = &runtime.frames[0] else {
                    panic!("parent")
                };
                assert!(matches!(
                    parent.phase,
                    Phase::SpreadArguments { index: 0, .. }
                ));
                let Frame::Expression(child) = &runtime.frames[1] else {
                    panic!("child")
                };
                assert!(!matches!(
                    child.unit.expr(child.expression),
                    code::Expr::SpreadArgument(_)
                ));
            }
            assert_eq!((runtime.steps, runtime.allocated), (0, heap));
            runtime.frames.clear();
            runtime.eval_depth = 0;
            clean(&runtime);
        }
    }
}

#[test]
fn resumed_ordinary_output_and_all_retained_handles_clean_up_on_first_append_or_next_item_refusal()
{
    for work in [7, 8] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let (unit, id) = expression("target(0,item,...later)");
        let function = Runtime::native("Object.is", Value::Undefined);
        let receiver = Runtime::native("Object.is", Value::Null);
        let output = Runtime::native("Object.is", Value::Bool(true));
        let arguments = paid_values(&mut runtime, vec![Value::Number(0.0)]);
        let before = runtime.steps;
        prepay_item(&mut runtime).unwrap();
        assert_eq!(before - runtime.steps, 6);
        let mut resumed = frame(unit, id);
        resumed.phase = Phase::SpreadArguments {
            index: 1,
            function: function.clone(),
            receiver: receiver.clone(),
            arguments,
        };
        runtime.steps = work;
        runtime.allocated = MAX_HEAP;
        resource(step(
            &mut runtime,
            resumed,
            Some(Output::Value(output.clone())),
            &mut document,
        ));
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        assert_eq!((refs(&function), refs(&receiver), refs(&output)), (1, 1, 1));
        clean(&runtime);
    }
}

#[test]
fn completed_storage_transfers_without_fee_and_existing_dispatch_refusals_release_it() {
    let mut runtime = Runtime::try_new().unwrap();
    let marker = Runtime::native("Object.is", Value::Undefined);
    let values = paid_values(&mut runtime, vec![marker.clone()]);
    let before = (runtime.steps, runtime.allocated);
    let moved = values.into_values();
    assert_eq!((runtime.steps, runtime.allocated), before);
    assert_eq!(moved.len(), 1);
    assert_eq!(refs(&marker), 2);
    drop(moved);
    assert_eq!(refs(&marker), 1);

    for construct in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let function = runtime
            .execute(
                "var trace='';(function Target(){trace+='T';})",
                &mut document,
            )
            .unwrap();
        let (unit, id) = expression(if construct {
            "new Target(...source)"
        } else {
            "target(...source)"
        });
        let marker = Runtime::native("Object.is", Value::Undefined);
        let arguments = paid_values(&mut runtime, vec![marker.clone()]);
        if construct {
            // Existing IsConstructor tick succeeds; existing stack guard refuses.
            runtime.steps = 1;
            runtime.stack_units = MAX_STACK_UNITS;
        } else {
            // Existing frame growth needs work1; no transfer fee is introduced.
            runtime.frames = Vec::new();
            runtime.steps = 0;
        }
        let heap = runtime.allocated;
        resource(spread_arguments_next(
            &mut runtime,
            frame(unit, id),
            1,
            function,
            Value::Undefined,
            arguments,
            &mut document,
        ));
        assert_eq!(
            (runtime.steps, runtime.allocated, refs(&marker)),
            (0, heap, 1)
        );
        assert!(runtime.frames.is_empty());
        assert_eq!(
            runtime.stack_units,
            if construct { MAX_STACK_UNITS } else { 0 }
        );
        runtime.stack_units = 0;
        assert_eq!(
            runtime.execute("trace", &mut document).unwrap(),
            Value::String("".into())
        );
        clean(&runtime);
    }
}

#[test]
fn forged_spread_markers_are_resource_errors_in_value_reference_and_ordinary_parent_contexts() {
    for context in 0..6 {
        let mut budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let mut unit = code::Unit::empty(false);
        let literal = unit
            .add_expr(code::Expr::Literal(Value::Number(1.0)), &mut budget)
            .unwrap();
        let marker = unit
            .add_expr(code::Expr::SpreadArgument(literal), &mut budget)
            .unwrap();
        let root = match context {
            0 | 1 => marker,
            2 => unit
                .add_expr(code::Expr::Call(literal, vec![marker]), &mut budget)
                .unwrap(),
            3 => unit
                .add_expr(code::Expr::New(literal, vec![marker]), &mut budget)
                .unwrap(),
            4 => {
                let nested = unit
                    .add_expr(code::Expr::SpreadArgument(marker), &mut budget)
                    .unwrap();
                unit.add_expr(code::Expr::CallSpread(literal, vec![nested]), &mut budget)
                    .unwrap()
            }
            _ => unit
                .add_expr(code::Expr::Update(marker, 1.0, false), &mut budget)
                .unwrap(),
        };
        let unit = unit.finish(&mut budget).unwrap();
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        if context == 1 {
            resource(evaluate_reference(
                &mut runtime,
                &unit,
                root,
                0,
                &mut document,
            ));
        } else {
            resource(evaluate(&mut runtime, &unit, root, 0, &mut document));
        }
        clean(&runtime);
    }
}

#[test]
fn ordinary_argument_storage_retains_exact_original_tariff() {
    for n in [0, 1, 9] {
        let bytes = if n == 0 {
            0
        } else {
            32 + n * size_of::<Value>()
        };
        let mut runtime = Runtime::try_new().unwrap();
        runtime.steps = n;
        runtime.allocated = MAX_HEAP - bytes;
        let values = reserve_values(&mut runtime, n, false).unwrap();
        assert!(values.is_empty());
        assert!(values.capacity() >= n);
        assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP));
        if n != 0 {
            runtime.steps = n - 1;
            runtime.allocated = 0;
            resource(reserve_values(&mut runtime, n, false));
            assert_eq!((runtime.steps, runtime.allocated), (0, 0));
            runtime.steps = n;
            runtime.allocated = MAX_HEAP - (bytes - 1);
            resource(reserve_values(&mut runtime, n, false));
            assert_eq!((runtime.steps, runtime.allocated), (0, MAX_HEAP + 1));
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn actual_call_spread_machine_layout_matches_both_held_mirrors() {
    assert_eq!((size_of::<Phase>(), align_of::<Phase>()), (96, 8));
    assert_eq!((size_of::<ExprFrame>(), align_of::<ExprFrame>()), (128, 8));
    assert_eq!((size_of::<Frame>(), align_of::<Frame>()), (184, 8));
    assert_eq!(
        (
            size_of::<statements::Frame>(),
            align_of::<statements::Frame>()
        ),
        (184, 8)
    );
    assert_eq!(
        (size_of::<calls::Frame>(), align_of::<calls::Frame>()),
        (112, 8)
    );
    assert_eq!((size_of::<Arguments>(), align_of::<Arguments>()), (32, 8));
    assert_eq!((size_of::<Value>(), align_of::<Value>()), (24, 8));
    assert_eq!((size_of::<Reference>(), align_of::<Reference>()), (56, 8));
    assert_eq!(INITIAL_FRAMES * size_of::<Frame>() + 32, 1504);
    assert_eq!(MAX_FRAMES, 45590);
    #[cfg(not(feature = "test262-host"))]
    assert_eq!((size_of::<Runtime>(), align_of::<Runtime>()), (1808, 16));
    #[cfg(feature = "test262-host")]
    assert_eq!((size_of::<Runtime>(), align_of::<Runtime>()), (1872, 16));
}
