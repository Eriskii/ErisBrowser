//! Root's two pinned x86-64 mirror measurements supply these layout assertions.
//! This test measures actual production types; it does not execute Promise code.
#[cfg(target_pointer_width = "64")]
use super::*;

#[cfg(target_pointer_width = "64")]
#[test]
fn actual_promise_private_type_layouts_match_held_mirror() {
    use std::mem::{align_of, size_of};
    macro_rules! row {
        ($name:literal, $ty:ty, $bytes:literal) => {
            assert_eq!((size_of::<$ty>(), align_of::<$ty>()), ($bytes, 8), $name);
            println!(
                "PROMISE_LAYOUT_ACTUAL {} bytes={} align={}",
                $name,
                size_of::<$ty>(),
                align_of::<$ty>()
            );
        };
    }
    row!("Value", Value, 24);
    row!("ScriptObject", ScriptObject, 216);
    row!("CurrentMicrotaskState", microtasks::State, 56);
    row!("ProposedMicrotaskState", microtasks::State, 56);
    row!("State", State, 192);
    row!("Intrinsics", Intrinsics, 32);
    row!("PromiseRecord", PromiseRecord, 80);
    row!("ReactionPair", ReactionPair, 120);
    row!("Capability", Capability, 72);
    row!("ResolveGate", ResolveGate, 16);
    row!("CallbackRecord", CallbackRecord, 24);
    row!("CapabilityCapture", CapabilityCapture, 48);
    row!("PromiseJob", PromiseJob, 128);
    row!("JobSlot", Option<PromiseJob>, 128);
    row!("QueueJob", QueueJob, 24);
    row!("PromiseArena", Arena<PromiseRecord>, 32);
    row!("PromiseId", PromiseId, 8);
    row!("OptionPromiseId", Option<PromiseId>, 16);
}
