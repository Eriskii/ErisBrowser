use super::*;

#[test]
fn flat_slot_admission_prepays_both_scans_at_exact_work_boundary() {
    let mut exact = 112; // 16*3 authored slots +64, independent of availability.
    assert!(admit_slots(0, 3, &mut exact));
    assert_eq!(exact, 0);
    let mut short = 111;
    assert!(!admit_slots(0, 3, &mut short));
    assert_eq!(short, 0);
}

#[test]
fn flat_slot_metadata_includes_full_none_slots_and_never_wraps() {
    let three_slots = 3 * size_of::<Option<LoadedBackground<'_>>>();
    let base = PASS_METADATA - three_slots - 64;
    let mut work = 112;
    assert!(admit_slots(base, 3, &mut work));
    let mut work = 112;
    assert!(!admit_slots(base + 1, 3, &mut work));
    assert_eq!(work, 112); // Metadata refusal precedes scan work and allocation.
    assert!(!admit_slots(usize::MAX, 3, &mut work));
    assert!(!admit_slots(0, usize::MAX, &mut work));
    assert_eq!(work, 112);
}
