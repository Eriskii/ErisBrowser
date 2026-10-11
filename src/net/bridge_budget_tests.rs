use super::*;
use std::cell::Cell;

thread_local! { static CALLS: Cell<usize> = const { Cell::new(0) }; }

fn refused(_: &Url, _: Option<&Url>, _: ResourceKind, _: Option<&str>) -> Result<Resource, String> {
    CALLS.set(CALLS.get() + 1);
    Err("fixture network refusal".into())
}

#[test]
fn bridge_attempt_limit_counts_errors_and_survives_moves_without_resetting_direct_fields() {
    std::thread::spawn(|| {
        use_page_fetch_bridge(refused);
        let mut fetcher = Fetcher::default();
        let target = Url::parse("https://example.test/image.png").unwrap();
        let original = fetcher.accounting_snapshot();
        for _ in 0..24 {
            assert_eq!(
                fetcher
                    .fetch(&target, None, ResourceKind::Image)
                    .unwrap_err(),
                "fixture network refusal"
            );
        }
        let mut moved = fetcher;
        for _ in 0..24 {
            assert_eq!(
                moved.fetch(&target, None, ResourceKind::Image).unwrap_err(),
                "fixture network refusal"
            );
        }
        for _ in 0..4 {
            assert_eq!(
                moved.fetch(&target, None, ResourceKind::Image).unwrap_err(),
                "page resource or time budget exceeded"
            );
        }
        assert_eq!(CALLS.get(), 48);
        let after = moved.accounting_snapshot();
        assert_eq!(after.count, original.count);
        assert_eq!(after.total, original.total);
        assert_eq!(after.started, original.started);
        assert_eq!(after.bridge_attempts, 48);
    })
    .join()
    .unwrap();
}

#[test]
fn rejected_form_before_bridge_does_not_spend_an_attempt() {
    std::thread::spawn(|| {
        use_page_fetch_bridge(refused);
        let mut fetcher = Fetcher::default();
        assert!(
            fetcher
                .fetch_document(&Url::parse("file:///tmp/example").unwrap(), Some("a=b"))
                .is_err()
        );
        assert_eq!(CALLS.get(), 0);
        assert_eq!(fetcher.accounting_snapshot().bridge_attempts, 0);
    })
    .join()
    .unwrap();
}
