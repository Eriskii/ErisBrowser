use super::*;

#[test]
fn adapter_host_dispatch_preserves_legacy_selection_and_rejects_disabled_opt_in() {
    assert_eq!(host_protocol(b"ERJS1\0\0"), Ok(false));
    assert_eq!(host_protocol(b"ERJS"), Ok(false));
    assert_eq!(host_protocol(b"ERJS9"), Ok(false));
    #[cfg(feature = "test262-host")]
    assert_eq!(host_protocol(b"ERJS2"), Ok(true));
    #[cfg(not(feature = "test262-host"))]
    assert!(host_protocol(b"ERJS2").is_err());
}
