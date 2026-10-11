//! Literal getter tariffs frozen before the next production edit.
use super::*;
use crate::dom::{DomDataError, DomMutationBudget};
use std::mem::size_of;

fn parse(source: &str) -> InlineStyle {
    let mut work = 20_000_000;
    InlineStyle::parse(source, &mut work).unwrap()
}
fn budget(steps: usize, heap_limit: usize) -> DomMutationBudget {
    DomMutationBudget {
        steps,
        allocated: 0,
        heap_limit,
    }
}

#[test]
fn direct_var_shorthand_getter_exact677_and_short676_are_atomic() {
    let style = parse(
        "margin-top:var(--x);margin-right:var(--x);margin-bottom:var(--x);margin-left:var(--x)",
    );
    let before = style.clone();
    assert_eq!(style.len(), 4);
    assert_eq!(style.logical_bytes(), 94);
    // N50, p6, K4: Q=7+(1+4*38)+4*57=388. V=4*(8+1)=36.
    // Complete prepayment is388+(8*36+1)=677, before token inspection.
    let mut exact = 677;
    assert_eq!(
        style.get_property_value("margin", &mut exact),
        Ok(String::new())
    );
    assert_eq!(exact, 0);
    let mut short = 676;
    assert_eq!(
        style.get_property_value("margin", &mut short),
        Err(Error::Limit)
    );
    assert_eq!(short, 0);
    assert_eq!(style, before);
    // Direct longhand copy and priority are separate unchanged paths.
    let mut direct = 127; // p11:12+44+62 + value-copy9.
    assert_eq!(
        style.get_property_value("margin-left", &mut direct),
        Ok("var(--x)".into())
    );
    assert_eq!(direct, 0);
    let mut priority = 388;
    assert_eq!(style.get_property_priority("margin", &mut priority), Ok(""));
    assert_eq!(priority, 0);
    let mut serialization = 189; // 2*L94+1, no pending metadata.
    assert_eq!(
        style.serialize(&mut serialization).unwrap(),
        "margin-top: var(--x); margin-right: var(--x); margin-bottom: var(--x); margin-left: var(--x);"
    );
    assert_eq!(serialization, 0);
    assert_eq!(style, before);
}

#[test]
fn literal_getter_exact517_preserves_original_pending_budget_paths() {
    let literal = parse("margin:1px");
    let literal_before = literal.clone();
    assert_eq!(literal.logical_bytes(), 74);
    // Same Q388; V=4*(3+1)=16. New paid scan+synthesis is129.
    let mut exact = 517;
    assert_eq!(
        literal.get_property_value("margin", &mut exact),
        Ok("1px".into())
    );
    assert_eq!(exact, 0);
    let mut short = 516;
    assert_eq!(
        literal.get_property_value("margin", &mut short),
        Err(Error::Limit)
    );
    assert_eq!(short, 0);
    assert_eq!(literal, literal_before);

    let pending = parse("margin:var(--m)!important");
    let before = pending.clone();
    assert_eq!(pending.logical_bytes(), 138);
    assert_eq!(pending.release_work(), 143);
    assert_eq!(pending.clone_work(), 1338);
    // Original source reconstruction returns before the new ordinary guard.
    let mut getter = 536; // Q388 + source-check139 + returned-copy9.
    assert_eq!(
        pending.get_property_value("margin", &mut getter),
        Ok("var(--m)".into())
    );
    assert_eq!(getter, 0);
    let mut short_getter = 535;
    assert_eq!(
        pending.get_property_value("margin", &mut short_getter),
        Err(Error::Limit)
    );
    assert_eq!(short_getter, 0);
    let mut serializer = 461;
    assert_eq!(
        pending.serialize(&mut serializer).unwrap(),
        "margin: var(--m) !important;"
    );
    assert_eq!(serializer, 0);
    assert_eq!(pending.serialize(&mut 460), Err(Error::Limit));

    let retained = pending.retained_bytes();
    let mut copy_budget = budget(1338, retained);
    let cloned = pending.try_clone_for_edit(&mut copy_budget).unwrap();
    assert_eq!((copy_budget.steps, copy_budget.allocated), (0, retained));
    assert_eq!(cloned, pending);
    let mut short_copy = budget(1337, retained);
    assert!(matches!(
        pending.try_clone_for_edit(&mut short_copy),
        Err(DomDataError::LimitExceeded)
    ));
    assert_eq!((short_copy.steps, short_copy.allocated), (0, 0));

    let prepared_heap = 138 + 2 * size_of::<usize>() + size_of::<InlineStyle>();
    let mut prepared_budget = budget(606, prepared_heap);
    let prepared = pending
        .clone()
        .prepare_commit(&mut prepared_budget)
        .unwrap();
    assert_eq!(
        (prepared_budget.steps, prepared_budget.allocated),
        (0, prepared_heap)
    );
    assert_eq!(prepared.serialized(), "margin: var(--m) !important;");
    assert_eq!(prepared.style(), &pending);
    let mut short_prepared = budget(605, prepared_heap);
    assert!(matches!(
        pending.clone().prepare_commit(&mut short_prepared),
        Err(DomDataError::LimitExceeded)
    ));
    assert_eq!(short_prepared.steps, 0);
    assert_eq!(pending, before);
}
