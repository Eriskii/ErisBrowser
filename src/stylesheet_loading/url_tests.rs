use super::*;

fn cached(request: &str, final_url: &str, source: &str, loader: &mut Loader) -> Arc<Url> {
    let url = Arc::new(Url::parse(final_url).unwrap());
    loader.cache.insert(
        (request.into(), "UTF-8"),
        Ok(Sheet {
            source: source.into(),
            url: url.clone(),
            embedded: false,
            encoding: encoding_rs::UTF_8,
        }),
    );
    url
}

fn fixed(source: &StyleSource) -> &Arc<Url> {
    match &source.base {
        SourceBase::Stylesheet(url) => url,
        SourceBase::Document => panic!("expected external stylesheet context"),
    }
}

#[test]
fn redirected_parent_and_import_keep_separate_final_bases_across_segments() {
    let document = Url::parse("https://example.test/docs/page.html").unwrap();
    let mut loader = Loader::new(&document, encoding_rs::UTF_8);
    let parent = cached(
        "https://example.test/start.css",
        "https://example.test/assets/v1/main.css",
        "@layer foundation; @import '../../theme/import.css' layer(theme); p{color:red}",
        &mut loader,
    );
    let child = cached(
        "https://example.test/theme/import.css",
        "https://example.test/cdn/theme/final.css",
        "p{background:blue}",
        &mut loader,
    );
    let sources = loader
        .external(
            &mut Fetcher::for_document(&document),
            &Url::parse("https://example.test/start.css").unwrap(),
            &mut Vec::new(),
        )
        .unwrap();
    let parent_parts: Vec<_> = sources
        .iter()
        .filter(|s| s.source.contains("foundation") || s.source.contains("color:red"))
        .collect();
    assert_eq!(parent_parts.len(), 2);
    for source in parent_parts {
        assert_eq!(
            fixed(source).as_str(),
            "https://example.test/assets/v1/main.css"
        );
        assert!(Arc::ptr_eq(fixed(source), &parent));
    }
    let imported = sources
        .iter()
        .find(|s| s.source.contains("background:blue"))
        .unwrap();
    assert_eq!(
        fixed(imported).as_str(),
        "https://example.test/cdn/theme/final.css"
    );
    assert!(Arc::ptr_eq(fixed(imported), &child));
    assert!(imported.layer.is_some());
    assert_eq!(loader.cache.len(), 2);
}

#[test]
fn inline_root_is_document_context_but_imports_stay_fixed() {
    let document = Url::parse("https://example.test/docs/page.html").unwrap();
    let loading_base = Url::parse("https://example.test/first/").unwrap();
    let mut loader = Loader::new(&document, encoding_rs::UTF_8);
    let child = cached(
        "https://example.test/first/theme.css",
        "https://example.test/final/theme.css",
        "p{color:blue}",
        &mut loader,
    );
    let sources = loader
        .inline(
            &mut Fetcher::for_document(&document),
            "@layer base; @import 'theme.css'; p{background:red}",
            &loading_base,
            &mut Vec::new(),
        )
        .unwrap();
    for source in &sources {
        if source.source.contains("color:blue") {
            assert!(Arc::ptr_eq(fixed(source), &child));
        } else {
            assert!(matches!(source.base, SourceBase::Document));
        }
    }
    assert!(sources.iter().any(|s| s.source.contains("background:red")));
    assert_eq!(loading_base.as_str(), "https://example.test/first/");
    // The returned context is a marker, not the inline import-loading base.
    // Page may resolve it against a later document base without refetching imports.
    assert_eq!(loader.cache.len(), 1);
}

#[test]
fn repeated_cached_external_sources_share_one_url_without_new_string_storage() {
    let document = Url::parse("https://example.test/page").unwrap();
    let request = Url::parse("https://example.test/start.css").unwrap();
    let mut loader = Loader::new(&document, encoding_rs::UTF_8);
    let owner = cached(
        request.as_str(),
        "https://example.test/final.css",
        "p{color:red}",
        &mut loader,
    );
    let before = loader.url_storage;
    let first = loader
        .external(
            &mut Fetcher::for_document(&document),
            &request,
            &mut Vec::new(),
        )
        .unwrap();
    let second = loader
        .external(
            &mut Fetcher::for_document(&document),
            &request,
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(loader.url_storage, before);
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert!(Arc::ptr_eq(fixed(&first[0]), &owner));
    assert!(Arc::ptr_eq(fixed(&second[0]), &owner));
    assert_eq!(loader.segments, 2);
}

#[test]
fn shared_url_admission_prepays_exact_string_storage_and_four_work() {
    let document = Url::parse("https://example.test/page").unwrap();
    let final_url = Url::parse("https://example.test/a.css").unwrap();
    // Literal URL bytes: https:// (8) + example.test (12) + /a.css (6).
    assert_eq!(final_url.as_str().len(), 26);
    for (work_left, bytes_left, succeeds) in [(4, 33, true), (3, 33, false), (4, 32, false)] {
        let mut loader = Loader::new(&document, encoding_rs::UTF_8);
        loader.url_work = MAX_URL_BYTES - work_left;
        loader.url_storage = MAX_URL_BYTES - bytes_left;
        let storage_before = loader.url_storage;
        let result = loader.retain_sheet_url(final_url.clone(), 7);
        if succeeds {
            assert_eq!(result.unwrap().as_str(), "https://example.test/a.css");
            assert_eq!(loader.url_work, MAX_URL_BYTES);
            assert_eq!(loader.url_storage, MAX_URL_BYTES);
        } else {
            assert!(result.is_err());
            assert_eq!(loader.url_storage, storage_before);
            assert!(loader.cache.is_empty());
        }
    }
}

#[test]
fn segment_base_publication_is_paid_before_output_is_moved() {
    let document = Url::parse("https://example.test/page").unwrap();
    let owner = Arc::new(Url::parse("https://example.test/style.css").unwrap());
    let sheet = Sheet {
        source: "".into(),
        url: owner.clone(),
        embedded: false,
        encoding: encoding_rs::UTF_8,
    };
    for (remaining, succeeds) in [(4, true), (3, false)] {
        let mut loader = Loader::new(&document, encoding_rs::UTF_8);
        loader.url_work = MAX_URL_BYTES - remaining;
        let mut sources = Vec::new();
        let mut output = "p{color:red}".to_owned();
        let result = loader.flush_source(&mut sources, &mut output, &StyleSource::new(""), &sheet);
        assert_eq!(result.is_ok(), succeeds);
        assert_eq!(loader.url_work, MAX_URL_BYTES);
        assert_eq!(loader.url_storage, 0);
        if succeeds {
            assert!(output.is_empty());
            assert_eq!(sources.len(), 1);
            assert!(Arc::ptr_eq(fixed(&sources[0]), &owner));
        } else {
            assert_eq!(output, "p{color:red}");
            assert!(sources.is_empty());
        }
    }
}

#[test]
fn unchanged_inline_text_capacity_does_not_pay_fixed_headers_from_text_bytes() {
    let document = Url::parse("https://example.test/page").unwrap();
    let mut loader = Loader::new(&document, encoding_rs::UTF_8);
    let source = " ".repeat(4 * 1024 * 1024);
    let first = loader
        .inline(
            &mut Fetcher::for_document(&document),
            &source,
            &document,
            &mut Vec::new(),
        )
        .unwrap();
    let second = loader
        .inline(
            &mut Fetcher::for_document(&document),
            &source,
            &document,
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(loader.decoded, 8 * 1024 * 1024);
    assert_eq!(loader.expanded, 8 * 1024 * 1024);
    assert_eq!(
        first.iter().map(|s| s.source.len()).sum::<usize>(),
        4 * 1024 * 1024
    );
    assert_eq!(
        second.iter().map(|s| s.source.len()).sum::<usize>(),
        4 * 1024 * 1024
    );
    assert!(
        first
            .iter()
            .chain(&second)
            .all(|s| matches!(s.base, SourceBase::Document))
    );
}
