//! Class storage uses the existing flat paged arena and Unit publication rules.
use super::*;
use crate::script::{MAX_STEPS, Native};
use std::mem::size_of;

fn budget() -> regexp::Budget {
    regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    }
}
fn constructor(unit: &mut Unit, ledger: &mut regexp::Budget) -> FunctionId {
    let mut function = Function::empty();
    function.arrow = false;
    function.constructable = true;
    function.class_kind = Some(ClassKind::DefaultDerived);
    unit.add_function(function, ledger).unwrap()
}
fn record(constructor: FunctionId, heritage: Option<ExprId>) -> Class {
    Class {
        name: None,
        heritage,
        constructor,
        elements: Vec::new(),
    }
}

#[test]
fn class_private_record_first_spare_and_next_page_cuts_preserve_ids() {
    for count in [0, 1, 128] {
        let work = match count {
            0 => 3,
            1 => 1,
            _ => 2,
        };
        let heap = match count {
            0 => 4 * size_of::<Vec<Class>>() + 32 + 128 * size_of::<Class>() + 32,
            1 => 0,
            _ => 128 * size_of::<Class>() + 32,
        };
        let mut rows = vec![(work - 1, heap, false), (work, heap, true)];
        if heap != 0 {
            rows.push((work, heap - 1, false));
        }
        for (available_work, available_heap, ok) in rows {
            let mut unit = Unit::empty(false);
            let mut ledger = budget();
            let function = constructor(&mut unit, &mut ledger);
            let mut first = None;
            for _ in 0..count {
                let id = unit.add_class(record(function, None), &mut ledger).unwrap();
                if first.is_none() {
                    first = Some((id, unit.class(id) as *const Class));
                }
            }
            ledger.steps = available_work;
            ledger.allocated = MAX_HEAP - available_heap;
            let result = unit.add_class(record(function, None), &mut ledger);
            if ok {
                assert_eq!(result.unwrap(), ClassId(count));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            assert_eq!(unit.classes.len(), count + usize::from(ok));
            if let Some((id, address)) = first {
                assert_eq!(unit.class(id) as *const Class, address);
            }
            assert_eq!(ledger.steps, 0);
            assert_eq!(
                ledger.allocated,
                if available_work < work {
                    MAX_HEAP - available_heap
                } else {
                    MAX_HEAP - available_heap + heap
                }
            );
        }
    }
}

#[test]
fn class_private_unit_publication_and_flat_drop_release_class_graph_owners() {
    let bytes = size_of::<Unit>() + 2 * size_of::<usize>();
    for (work, heap, ok) in [(0, bytes, false), (1, bytes - 1, false), (1, bytes, true)] {
        let mut unit = Unit::empty(false);
        let mut ledger = budget();
        let function = constructor(&mut unit, &mut ledger);
        let marker = Rc::new(Native {
            properties: None,
            name: "class-storage-marker".into(),
            receiver: Value::Undefined,
        });
        let weak = Rc::downgrade(&marker);
        let mut edge = unit
            .add_expr(Expr::Literal(Value::Native(marker)), &mut ledger)
            .unwrap();
        // Deep syntax edges remain IDs in one Unit; dropping them cannot recurse
        // through a chain of owning Class/Expr nodes or leave a child Unit cycle.
        for _ in 0..4096 {
            let class = unit
                .add_class(record(function, Some(edge)), &mut ledger)
                .unwrap();
            edge = unit.add_expr(Expr::Class(class), &mut ledger).unwrap();
        }
        ledger.steps = work;
        ledger.allocated = MAX_HEAP - heap;
        let result = unit.finish(&mut ledger);
        if ok {
            let unit = result.unwrap();
            assert!(weak.upgrade().is_some());
            assert_eq!(unit.compiled_storage, MAX_HEAP);
            assert!(matches!(unit.expr(edge), Expr::Class(_)));
            let retained = FunctionRef::new(&unit, function);
            drop(unit);
            assert!(weak.upgrade().is_some());
            drop(retained);
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert!(weak.upgrade().is_none());
        assert_eq!(ledger.steps, 0);
        assert_eq!(
            ledger.allocated,
            if work == 0 {
                MAX_HEAP - heap
            } else {
                MAX_HEAP - heap + bytes
            }
        );
    }
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        assert_eq!(size_of::<Class>(), 72);
        assert_eq!(size_of::<ClassElement>(), 32);
        assert_eq!(size_of::<Unit>(), 168);
        assert_eq!(bytes, 184);
    }
}
