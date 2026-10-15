//! Actual production parser union sizes, qualified by held x86_64 mirrors.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use super::*;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use std::mem::{align_of, size_of};
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn actual_call_spread_parser_layout_matches_both_held_mirrors() {
    assert_eq!(
        (
            size_of::<expressions::Frame>(),
            align_of::<expressions::Frame>()
        ),
        (72, 8)
    );
    assert_eq!((size_of::<Kind>(), align_of::<Kind>()), (112, 8));
    assert_eq!(
        (size_of::<Transition>(), align_of::<Transition>()),
        (224, 8)
    );
    assert_eq!((size_of::<Output>(), align_of::<Output>()), (88, 8));
    assert_eq!(4 * size_of::<Kind>() + 32, 480);
    assert_eq!(MAX_HEAP / size_of::<Kind>(), 74898);
}
