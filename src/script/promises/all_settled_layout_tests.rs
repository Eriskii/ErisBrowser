//! Actual production layouts checked against the retained complete mirrors.
//! Numeric expectations are measured only on x86_64 Linux, in four root runs.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use super::*;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn all_settled_actual_layout_matches_the_complete_four_configuration_mirrors() {
    use std::mem::{align_of, size_of};
    macro_rules! row {
        ($name:literal, $ty:ty, $bytes:literal, $alignment:literal) => {
            assert_eq!(
                (size_of::<$ty>(), align_of::<$ty>()),
                ($bytes, $alignment),
                $name
            );
            println!(
                "ALL_SETTLED_LAYOUT_ACTUAL {} bytes={} align={}",
                $name,
                size_of::<$ty>(),
                align_of::<$ty>()
            );
        };
    }
    row!("CallbackKind", CallbackKind, 16, 8);
    row!("CallbackRecord", CallbackRecord, 24, 8);
    row!("CallbackArena", Arena<CallbackRecord>, 32, 8);
    row!("State", State, 256, 8);
    row!("PromiseJob", PromiseJob, 128, 8);
    row!("OptionPromiseJob", Option<PromiseJob>, 128, 8);
    row!("PromiseJobArena", Arena<Option<PromiseJob>>, 32, 8);
    row!("QueueJob", QueueJob, 24, 8);
    row!("AllCapture", AllCapture, 72, 8);
    row!("AllCaptureArena", Arena<AllCapture>, 32, 8);
    row!("ReadyAllValues", ReadyAllValues, 48, 8);
    #[cfg(not(feature = "test262-host"))]
    row!("Runtime", Runtime, 1808, 16);
    #[cfg(feature = "test262-host")]
    row!("Runtime", Runtime, 1872, 16);
    row!("Value", Value, 24, 8);
    row!("ScriptObject", ScriptObject, 216, 8);
    row!("MicrotaskState", microtasks::State, 56, 8);
    row!("Capability", Capability, 72, 8);
    row!("ReactionPair", ReactionPair, 120, 8);
    row!("FinallyCapture", FinallyCapture, 48, 8);
    assert_eq!(size_of::<CallbackRecord>().div_ceil(size_of::<Value>()), 1);
    assert_eq!(32 + 8 * size_of::<CallbackRecord>(), 224);
    assert_eq!(size_of::<AllCapture>().div_ceil(size_of::<Value>()), 3);
    assert_eq!(2048usize.max(size_of::<Runtime>()), 2048);
}
