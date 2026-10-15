//! Actual production sizes must match the root-executed prospective mirror.
//! This observes type layout only; it is not a stack or RSS measurement.
#[cfg(target_pointer_width = "64")]
use super::*;

#[cfg(target_pointer_width = "64")]
#[test]
fn all_actual_private_layout_matches_four_configuration_prospective_mirror() {
    use std::mem::{align_of, size_of};
    macro_rules! row {
        ($name:literal, $ty:ty, $bytes:literal, $align:literal) => {
            assert_eq!(
                (size_of::<$ty>(), align_of::<$ty>()),
                ($bytes, $align),
                $name
            );
            println!(
                "PROMISE_ALL_LAYOUT_ACTUAL {} bytes={} align={}",
                $name,
                size_of::<$ty>(),
                align_of::<$ty>()
            );
        };
    }
    row!("Value", Value, 24, 8);
    row!("AllCaptureId", AllCaptureId, 8, 8);
    row!("AllCapture", AllCapture, 72, 8);
    row!("AllCaptureArena", Arena<AllCapture>, 32, 8);
    row!("ReadyAllValues", ReadyAllValues, 48, 8);
    row!("AllIterator", AllIterator, 56, 8);
    row!("CallbackKind", CallbackKind, 16, 8);
    row!("CallbackRecord", CallbackRecord, 24, 8);
    row!("PromiseJob", PromiseJob, 128, 8);
    row!("OptionPromiseJob", Option<PromiseJob>, 128, 8);
    row!("QueueJob", QueueJob, 24, 8);
    row!("State", State, 256, 8);
    #[cfg(not(feature = "test262-host"))]
    row!("Runtime", Runtime, 1808, 16);
    #[cfg(feature = "test262-host")]
    row!("Runtime", Runtime, 1872, 16);
    row!("ScriptObject", ScriptObject, 216, 8);
    row!("ArrayValuesRow", Vec<Value>, 24, 8);
    row!("ArrayPropertiesRow", usize, 8, 8);
    row!("ArrayHolesRow", BTreeSet<usize>, 24, 8);
    row!("ArrayLengthRow", array_properties::ArrayLength, 8, 4);
    row!("AbortSlotOption", Option<AbortSlot>, 16, 8);
    row!("DateOption", Option<f64>, 16, 8);
    row!("PropertyKey", PropertyKey, 16, 8);
    row!("Property", Property, 56, 8);
    row!("Native", Native, 56, 8);
    assert_eq!(size_of::<AllCapture>().div_ceil(size_of::<Value>()), 3);
    assert_eq!(size_of::<ScriptObject>().div_ceil(size_of::<Value>()), 9);
    assert!(size_of::<Runtime>() < 2048);
}
