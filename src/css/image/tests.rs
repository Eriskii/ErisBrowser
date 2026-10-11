use super::*;
fn base() -> ::url::Url {
    ::url::Url::parse("https://x/").unwrap()
}
fn arc_reservation() -> usize {
    let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<url::ResolvedUrl>())
        .unwrap();
    layout.pad_to_align().size() + 32
}
fn first_bytes() -> usize {
    4984 + arc_reservation()
        + std::mem::size_of::<UrlEntry>()
        + std::mem::size_of::<String>()
        + std::mem::size_of::<::url::Url>()
}
#[test]
fn first_miss_and_repeated_hit_have_literal_work_and_shared_identity() {
    let mut pool = ImagePool::default();
    let mut work = 579;
    let first = pool
        .url("url(a)", WinningSource::Document, &[], &base(), &mut work)
        .unwrap()
        .unwrap();
    assert_eq!(work, 0);
    assert_eq!(pool.urls.charged_bytes, first_bytes());
    let mut work = 24;
    let same = pool
        .url("url(a)", WinningSource::Document, &[], &base(), &mut work)
        .unwrap()
        .unwrap();
    assert_eq!(work, 0);
    assert!(Arc::ptr_eq(&first, &same));
    assert_eq!(pool.urls.charged_bytes, first_bytes());
    for short in 0..579 {
        let mut p = ImagePool::default();
        let mut work = short;
        assert_eq!(
            p.url("url(a)", WinningSource::Document, &[], &base(), &mut work),
            Err(Error::Limit)
        );
        assert!(p.urls.entries.is_empty());
        assert_eq!(work, 0);
    }
}
#[test]
fn exact_and_short_shared_storage_preserve_old_gradient_reservation() {
    let bytes = first_bytes();
    for short in [false, true] {
        let mut p = ImagePool::default();
        p.urls.charged_bytes = MAX_IMAGE_BYTES - bytes + usize::from(short);
        let result = p.url("url(a)", WinningSource::Document, &[], &base(), &mut 579);
        if short {
            assert_eq!(result, Err(Error::Limit));
            assert!(p.urls.entries.is_empty());
        } else {
            assert!(result.unwrap().is_some());
        }
        assert_eq!(p.urls.charged_bytes, MAX_IMAGE_BYTES);
    }
    let parsed = match gradient::parse_image(
        "linear-gradient(red,blue)",
        [16.0, 16.0, 80.0, 60.0],
        &mut 100_000,
    )
    .unwrap()
    .unwrap()
    {
        gradient::ParsedImage::Gradient(p) => p,
        _ => panic!("gradient"),
    };
    let mut old = gradient::GradientPool::default();
    let mut old_work = 424;
    let _ = old.intern(&parsed, &mut old_work).unwrap().unwrap();
    let mut mixed = ImagePool::default();
    let mut work = 424;
    let a = mixed.gradient(&parsed, &mut work).unwrap().unwrap();
    assert_eq!(work, old_work);
    assert_eq!(mixed.gradients.bytes(), old.bytes());
    mixed.urls.charged_bytes = MAX_IMAGE_BYTES - mixed.gradients.bytes();
    let b = mixed.gradient(&parsed, &mut 100_000).unwrap().unwrap();
    assert!(Arc::ptr_eq(&a, &b));
}
#[test]
fn empty_fragment_and_growth_moves_have_literal_charges() {
    for (raw, total) in [("url()", 120), ("url(#X)", 158)] {
        let mut p = ImagePool::default();
        let mut work = total;
        assert!(
            p.url(raw, WinningSource::Document, &[], &base(), &mut work)
                .unwrap()
                .is_some()
        );
        assert_eq!(work, 0);
    }
    for (raw, total) in [
        ("url(e)", 603 + 4 * std::mem::size_of::<UrlEntry>()),
        ("url(0)", 615 + 8 * std::mem::size_of::<UrlEntry>()),
    ] {
        let mut p = ImagePool::default();
        for key in ["url(a)", "url(b)", "url(c)", "url(d)"] {
            p.url(key, WinningSource::Document, &[], &base(), &mut 100_000)
                .unwrap()
                .unwrap();
        }
        assert_eq!(p.urls.admitted_capacity, 4);
        let before = p.urls.charged_bytes;
        let mut work = total;
        p.url(raw, WinningSource::Document, &[], &base(), &mut work)
            .unwrap()
            .unwrap();
        assert_eq!(work, 0);
        assert_eq!(p.urls.admitted_capacity, 8);
        assert_eq!(
            p.urls.charged_bytes - before,
            4984 + arc_reservation()
                + 8 * std::mem::size_of::<UrlEntry>()
                + std::mem::size_of::<String>()
                + std::mem::size_of::<::url::Url>()
        );
    }
}
#[test]
fn failed_calls_keep_debits_without_publishing_partial_entries() {
    let mut p = ImagePool::default();
    let mut work = 100_000;
    assert!(
        p.url("url(a", WinningSource::Document, &[], &base(), &mut work)
            .unwrap()
            .is_none()
    );
    assert!(p.urls.entries.is_empty());
    assert!(p.urls.charged_bytes > 0);
    let charged = p.urls.charged_bytes;
    assert_eq!(
        p.url("url(a)", WinningSource::Rule(99), &[], &base(), &mut work),
        Err(Error::Invalid)
    );
    assert_eq!(p.urls.charged_bytes, charged);
    let unresolved = p
        .url(
            "url('http://[invalid')",
            WinningSource::Document,
            &[],
            &base(),
            &mut work,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        unresolved.as_ref(),
        &url::ResolvedUrl::Unresolved("http://[invalid".into())
    );
}
