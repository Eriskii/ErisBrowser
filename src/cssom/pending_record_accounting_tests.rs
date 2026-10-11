// Tests for the selected new private accounting/API seams, installed with production.
use super::*;
use crate::dom::{DomDataError, DomMutationBudget};
use std::{mem::size_of, sync::Arc};

const WORK: usize = 20_000_000;
fn parse(source: &str) -> InlineStyle {
    InlineStyle::parse(source, &mut WORK.clone()).unwrap()
}
fn budget(steps: usize, heap_limit: usize) -> DomMutationBudget {
    DomMutationBudget {
        steps,
        allocated: 0,
        heap_limit,
    }
}

#[test]
fn pending_record_shared_all_payload_has_conservative_per_reference_retention() {
    let mut style = parse("--a:initial;all:var(--a)");
    assert_eq!(style.declarations().len(), 87);
    let RecordValue::Pending(first) = &style.records[1].value else {
        panic!("pending all")
    };
    assert_eq!(first.shorthand.as_ref(), "all");
    assert_eq!(first.tokens, "var(--a)");
    for record in &style.records[1..] {
        let RecordValue::Pending(next) = &record.value else {
            panic!("pending longhand")
        };
        assert!(Arc::ptr_eq(first, next));
    }
    let arc_header = 2 * size_of::<usize>();
    let expected = arc_header
        + size_of::<InlineStyle>()
        + style.records.capacity() * size_of::<Record>()
        + style
            .records
            .iter()
            .map(|record| {
                record.name.capacity()
                    + match &record.value {
                        RecordValue::Tokens(tokens) => tokens.capacity(),
                        RecordValue::Pending(pending) => {
                            arc_header
                                + size_of::<PendingValue>()
                                + pending.tokens.capacity()
                                + arc_header
                                + pending.shorthand.len()
                        }
                    }
            })
            .sum::<usize>();
    assert_eq!(style.retained_bytes(), expected);
    // Replacing only the Arc identity with equal source content must not change getters.
    let same_content = Arc::new(PendingValue {
        shorthand: Arc::from("all"),
        tokens: "var(--a)".into(),
    });
    style.records[1].value = RecordValue::Pending(same_content);
    assert_eq!(
        style.get_property_value("all", &mut WORK.clone()).unwrap(),
        "var(--a)"
    );
}

#[test]
fn pending_record_checked_clone_prepays_copy_summary_and_failure_teardown() {
    let style = parse("margin:var(--m)!important");
    assert_eq!(style.logical_bytes(), 138);
    assert_eq!(style.release_work(), 143);
    assert_eq!(style.clone_work(), 1338);
    let retained = style.retained_bytes();
    let mut exact = budget(1338, retained);
    let cloned = style.try_clone_for_edit(&mut exact).unwrap();
    assert_eq!(exact.steps, 0);
    assert_eq!(exact.allocated, retained);
    assert_eq!(cloned, style);
    assert!(cloned.retained_bytes() <= retained);
    let mut short_work = budget(1337, retained);
    assert!(matches!(
        style.try_clone_for_edit(&mut short_work),
        Err(DomDataError::LimitExceeded)
    ));
    assert_eq!(short_work.allocated, 0);
    let mut short_heap = budget(1338, retained - 1);
    assert!(matches!(
        style.try_clone_for_edit(&mut short_heap),
        Err(DomDataError::LimitExceeded)
    ));
    assert_eq!(style.logical_bytes(), 138);
    assert_eq!(
        style
            .get_property_value("margin", &mut WORK.clone())
            .unwrap(),
        "var(--m)"
    );
}

#[test]
fn pending_record_prepared_exact_cuts_include_later_refusal_cleanup() {
    let style = parse("margin:var(--m)!important");
    let heap = 138 + 2 * size_of::<usize>() + size_of::<InlineStyle>();
    //461 serialize +143 flat block release allowance +2 fixed output/Arc work.
    let mut exact = budget(606, heap);
    let prepared = style.clone().prepare_commit(&mut exact).unwrap();
    assert_eq!(exact.steps, 0);
    assert_eq!(exact.allocated, heap);
    assert_eq!(prepared.serialized(), "margin: var(--m) !important;");
    assert_eq!(prepared.style().logical_bytes(), 138);
    let (retained, text) = prepared.into_parts();
    assert_eq!(text, "margin: var(--m) !important;");
    assert_eq!(retained.as_ref(), &style);
    let mut short_work = budget(605, heap);
    assert!(matches!(
        style.clone().prepare_commit(&mut short_work),
        Err(DomDataError::LimitExceeded)
    ));
    let mut short_heap = budget(606, heap - 1);
    assert!(matches!(
        style.clone().prepare_commit(&mut short_heap),
        Err(DomDataError::LimitExceeded)
    ));
    // A caller can refuse its later DOM admission and simply drop this prepared block.
    let prepared = style
        .clone()
        .prepare_commit(&mut budget(606, heap))
        .unwrap();
    drop(prepared);
    assert_eq!(style.logical_bytes(), 138);
}
