//! Actual types must match the independently held1.88/1.99 prospective mirror.
#[cfg(target_pointer_width = "64")]
use super::*;

#[cfg(target_pointer_width = "64")]
#[test]
fn finally_actual_capture_layout_matches_the_held_mirror_without_widening_queue_slots() {
    use std::mem::{align_of, size_of};
    macro_rules! row {
        ($name:literal, $ty:ty, $bytes:literal) => {
            assert_eq!((size_of::<$ty>(), align_of::<$ty>()), ($bytes, 8), $name);
            println!(
                "FINALLY_LAYOUT_ACTUAL {} bytes={} align={}",
                $name,
                size_of::<$ty>(),
                align_of::<$ty>()
            );
        };
    }
    row!("State", State, 256);
    row!("FinallyCapture", FinallyCapture, 48);
    row!("FinallyCaptureArena", Arena<FinallyCapture>, 32);
    row!("FinallyCaptureId", FinallyCaptureId, 8);
    row!("CallbackKind", CallbackKind, 16);
    row!("CallbackRecord", CallbackRecord, 24);
    row!("PromiseJob", PromiseJob, 128);
    row!("QueueJob", QueueJob, 24);
    row!("Value", Value, 24);
    row!("ScriptObject", ScriptObject, 216);
    row!("MicrotaskState", microtasks::State, 56);
    println!(
        "FINALLY_LAYOUT_ACTUAL Runtime bytes={} align={}",
        size_of::<Runtime>(),
        align_of::<Runtime>()
    );
}
