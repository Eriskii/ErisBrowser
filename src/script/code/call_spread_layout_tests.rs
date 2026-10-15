//! Actual records and selected marker-page boundaries; no mirror definitions.
use super::*;
use crate::script::MAX_STEPS;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use std::mem::align_of;
use std::mem::size_of;
fn budget() -> regexp::Budget {
    regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    }
}
fn resource<T>(result: Result<T>) {
    match result {
        Err(e) => assert!(e.is_resource_limit(), "{e:?}"),
        Ok(_) => panic!("expected resource"),
    }
}

#[test]
fn spread_marker_append_uses_original_record_and_page_preflight() {
    for full in [false, true] {
        let work = if full { 2 } else { 1 };
        let heap = if full {
            128 * size_of::<Expr>() + 32
        } else {
            0
        };
        let mut rows = vec![(work - 1, heap, false), (work, heap, true)];
        if full {
            rows.push((work, heap - 1, false));
        }
        for (available_work, available_heap, ok) in rows {
            let mut unit = Unit::empty(false);
            let mut ledger = budget();
            let literal = unit
                .add_expr(Expr::Literal(Value::Number(3.0)), &mut ledger)
                .unwrap();
            if full {
                for _ in 1..128 {
                    unit.add_expr(Expr::Literal(Value::Null), &mut ledger)
                        .unwrap();
                }
            }
            let len = unit.expressions.len();
            let address = unit.expr(literal) as *const Expr;
            ledger.steps = available_work;
            ledger.allocated = MAX_HEAP - available_heap;
            let result = unit.add_expr(Expr::SpreadArgument(literal), &mut ledger);
            assert_eq!(result.is_ok(), ok);
            if ok {
                let id = result.unwrap();
                assert!(matches!(unit.expr(id),Expr::SpreadArgument(inner) if *inner==literal));
                assert_eq!(unit.expressions.len(), len + 1);
            } else {
                resource(result);
                assert_eq!(unit.expressions.len(), len);
            }
            assert_eq!(unit.expr(literal) as *const Expr, address);
            assert_eq!(ledger.steps, 0);
            assert_eq!(
                ledger.allocated,
                if available_work < work {
                    MAX_HEAP - available_heap
                } else if available_heap < heap {
                    MAX_HEAP + 1
                } else {
                    MAX_HEAP
                }
            );
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn actual_call_spread_code_layout_matches_both_held_mirrors() {
    assert_eq!((size_of::<Expr>(), align_of::<Expr>()), (48, 8));
    assert_eq!((size_of::<ExprId>(), align_of::<ExprId>()), (8, 8));
    assert_eq!(
        (size_of::<Vec<ExprId>>(), align_of::<Vec<ExprId>>()),
        (24, 8)
    );
    assert_eq!(
        (size_of::<PropertyName>(), align_of::<PropertyName>()),
        (16, 8)
    );
    assert_eq!(
        (size_of::<ObjectEntry>(), align_of::<ObjectEntry>()),
        (16, 8)
    );
    assert_eq!(128 * size_of::<Expr>() + 32, 6176);
}
