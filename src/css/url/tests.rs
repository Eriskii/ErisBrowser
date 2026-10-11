use super::*;
use std::sync::Arc;

fn generous() -> UrlBudget {
    UrlBudget {
        work: 100_000_000,
        bytes: 16 * 1024 * 1024,
    }
}
fn decoded(value: &str) -> DecodedUrl {
    decode(value, &mut generous()).unwrap().unwrap()
}
fn document() -> ::url::Url {
    ::url::Url::parse("https://Document.test/Page/index.html?Old=1").unwrap()
}
fn absolute(value: &str, base: &SourceBase, document: &::url::Url) -> String {
    match resolve(&decoded(value), base, document, &mut generous()).unwrap() {
        ResolvedUrl::Absolute(value) => value.to_string(),
        other => panic!("expected absolute URL: {other:?}"),
    }
}

#[test]
fn complete_url_tokens_preserve_case_data_and_comments() {
    for (source, expected) in [
        ("url(Images/A.PNG?Q=Case#Frag)", "Images/A.PNG?Q=Case#Frag"),
        (" URL(  'Images/A B.PNG?Q=Case' ) ", "Images/A B.PNG?Q=Case"),
        ("/**/url(\"A/**/B\")/*end*/", "A/**/B"),
        ("url(A/**/B)", "A/**/B"),
        ("url(a;{b})", "a;{b}"),
        (r"u\72l(A\)B)", "A)B"),
        (r"\75rl(A\ B)", "A B"),
        ("url()", ""),
        ("url( \t )", ""),
        ("url(\"\")", ""),
        ("url(\" \" )", " "),
    ] {
        assert_eq!(decoded(source).as_str(), expected, "{source}");
    }
}

#[test]
fn escapes_apply_css_unicode_replacement_and_string_continuations() {
    for (source, expected) in [
        (r"url(\41 b\000043D)", "AbCD"),
        (r"url(\0 \D800 \110000 )", "\u{fffd}\u{fffd}\u{fffd}"),
        ("url(\0)", "\u{fffd}"),
        ("url(\"A\0B\")", "A\u{fffd}B"),
        ("url(\"A\\\nB\\\r\nC\\\x0cD\")", "ABCD"),
        ("url(\"\\41\r\nB\")", "AB"),
        (r#"url("A\\B")"#, "A\\B"),
        ("url(\"é雪\")", "é雪"),
        ("url(\"\u{1}\")", "\u{1}"),
    ] {
        assert_eq!(decoded(source).as_str(), expected, "{source:?}");
    }
}

#[test]
fn malformed_or_unsupported_complete_values_are_not_eof_repaired() {
    for source in [
        "",
        "a.png",
        "'a.png'",
        "src(a.png)",
        "url (a)",
        "u/**/rl(a)",
        "url(a) url(b)",
        "url(a),url(b)",
        "url(\"a\" type(image/png))",
        "url(\"a\") trailing",
        "url(a",
        "url(\"a\"",
        "url(\"a)",
        "url(a\\)",
        "url(a b)",
        "url(a(b)",
        "url(a\"b)",
        "url(a\u{1}b)",
        "url(a\u{7f}b)",
        "url(a\\\nb)",
        "url(\"a\nb\")",
        "url(/*x*/\"a\")",
        "url(a)/*",
        "url(a)]",
    ] {
        assert_eq!(decode(source, &mut generous()), Ok(None), "{source:?}");
    }
}

#[test]
fn empty_and_local_fragment_values_never_become_document_requests() {
    let base = SourceBase::Stylesheet(Arc::new(
        ::url::Url::parse("https://Sheet.test/css/main.css").unwrap(),
    ));
    for source in ["url()", "url('')"] {
        assert_eq!(
            resolve(&decoded(source), &base, &document(), &mut generous()),
            Ok(ResolvedUrl::Empty)
        );
    }
    for (source, expected) in [
        ("url(#Icon)", "#Icon"),
        (r"url(\23 AbC)", "#AbC"),
        ("url('#é')", "#é"),
    ] {
        assert_eq!(
            resolve(&decoded(source), &base, &document(), &mut generous()),
            Ok(ResolvedUrl::LocalFragment(expected.into()))
        );
    }
    assert_eq!(
        absolute("url(other.svg#Icon)", &base, &document()),
        "https://sheet.test/css/other.svg#Icon"
    );
}

#[test]
fn relative_urls_use_the_selected_base_and_document_markers_stay_dynamic() {
    let sheet = SourceBase::Stylesheet(Arc::new(
        ::url::Url::parse("https://Sheet.test/css/nested/main.css").unwrap(),
    ));
    let first = document();
    let later = ::url::Url::parse("https://Changed.test/Elsewhere/").unwrap();
    let value = "url(../Images/A.PNG?Q=Case#Frag)";
    assert_eq!(
        absolute(value, &SourceBase::Document, &first),
        "https://document.test/Images/A.PNG?Q=Case#Frag"
    );
    assert_eq!(
        absolute(value, &SourceBase::Document, &later),
        "https://changed.test/Images/A.PNG?Q=Case#Frag"
    );
    for doc in [&first, &later] {
        assert_eq!(
            absolute(value, &sheet, doc),
            "https://sheet.test/css/Images/A.PNG?Q=Case#Frag"
        );
    }
    // Already resolved values can be handed to a child without re-resolving.
    let resolved = resolve(&decoded(value), &sheet, &first, &mut generous()).unwrap();
    let inherited = resolved;
    assert!(
        matches!(inherited, ResolvedUrl::Absolute(url) if url.as_str() == "https://sheet.test/css/Images/A.PNG?Q=Case#Frag")
    );
}

#[test]
fn url_parsing_uses_default_utf8_and_does_not_impose_fetch_policy() {
    for (source, expected) in [
        (
            "url('asset?é=雪')",
            "https://document.test/Page/asset?%C3%A9=%E9%9B%AA",
        ),
        (
            "url('https://User:Pass@Example.test:443/Case?Q=A#Frag')",
            "https://User:Pass@example.test/Case?Q=A#Frag",
        ),
        ("url(data:text/plain,Case)", "data:text/plain,Case"),
        ("url(file:///tmp/A.PNG)", "file:///tmp/A.PNG"),
        ("url(custom:Case)", "custom:Case"),
    ] {
        assert_eq!(
            absolute(source, &SourceBase::Document, &document()),
            expected
        );
    }
    let opaque = ::url::Url::parse("about:blank").unwrap();
    assert_eq!(
        resolve(
            &decoded("url(A)"),
            &SourceBase::Document,
            &opaque,
            &mut generous()
        ),
        Ok(ResolvedUrl::Unresolved("A".into()))
    );
    assert_eq!(
        resolve(
            &decoded("url('http://[invalid')"),
            &SourceBase::Document,
            &document(),
            &mut generous()
        ),
        Ok(ResolvedUrl::Unresolved("http://[invalid".into()))
    );
}

#[test]
fn decoder_has_literal_exact_and_one_short_work_and_storage_admission() {
    // url(A):6 bytes; tokenizer13 + decoder80 =93 work. Output18 bytes
    // plus the String header are admitted before the sole decoded allocation.
    let storage = 18 + std::mem::size_of::<String>();
    let mut exact = UrlBudget {
        work: 93,
        bytes: storage,
    };
    assert_eq!(decode("url(A)", &mut exact).unwrap().unwrap().as_str(), "A");
    assert_eq!(exact, UrlBudget { work: 0, bytes: 0 });
    let mut short_work = UrlBudget {
        work: 92,
        bytes: storage,
    };
    assert_eq!(decode("url(A)", &mut short_work), Err(UrlError::Limit));
    assert_eq!(short_work.work, 0);
    let mut short_bytes = UrlBudget {
        work: 93,
        bytes: storage - 1,
    };
    assert_eq!(decode("url(A)", &mut short_bytes), Err(UrlError::Limit));
    assert_eq!(short_bytes.bytes, 0);
    let mut shared = UrlBudget {
        work: 186,
        bytes: storage * 2,
    };
    assert!(decode("url(A)", &mut shared).unwrap().is_some());
    assert!(decode("url(B)", &mut shared).unwrap().is_some());
    assert_eq!(shared, UrlBudget { work: 0, bytes: 0 });
}

#[test]
fn raw_source_boundary_and_nul_expansion_are_bounded() {
    let exact = format!("url({})", "A".repeat(4091));
    assert_eq!(exact.len(), 4096);
    assert_eq!(decoded(&exact).as_str().len(), 4091);
    assert_eq!(
        decode(&(exact + " "), &mut generous()),
        Err(UrlError::Limit)
    );
    let value = format!("url({})", "\0".repeat(4091));
    assert_eq!(decoded(&value).as_str(), "\u{fffd}".repeat(4091));
}

#[test]
fn resolver_has_literal_dispatch_and_parse_admission_boundaries() {
    let empty = decoded("url()");
    let mut empty_budget = UrlBudget { work: 8, bytes: 0 };
    assert_eq!(
        resolve(
            &empty,
            &SourceBase::Document,
            &document(),
            &mut empty_budget
        ),
        Ok(ResolvedUrl::Empty)
    );
    assert_eq!(empty_budget, UrlBudget { work: 0, bytes: 0 });
    let fragment = decoded("url(#X)");
    let fragment_storage = 2 + std::mem::size_of::<String>();
    let mut fragment_budget = UrlBudget {
        work: 24,
        bytes: fragment_storage,
    };
    assert_eq!(
        resolve(
            &fragment,
            &SourceBase::Document,
            &document(),
            &mut fragment_budget
        ),
        Ok(ResolvedUrl::LocalFragment("#X".into()))
    );
    assert_eq!(fragment_budget, UrlBudget { work: 0, bytes: 0 });
    // T=1,B=10:8+32*(B+T+1)+16*(T+1)^2=456.
    let value = decoded("url(a)");
    let base = ::url::Url::parse("https://x/").unwrap();
    let storage = 4864 + std::mem::size_of::<::url::Url>();
    let mut exact = UrlBudget {
        work: 456,
        bytes: storage,
    };
    assert_eq!(
        resolve(&value, &SourceBase::Document, &base, &mut exact).unwrap(),
        ResolvedUrl::Absolute(::url::Url::parse("https://x/a").unwrap())
    );
    assert_eq!(exact, UrlBudget { work: 0, bytes: 0 });
    for (work, bytes) in [(455, storage), (456, storage - 1)] {
        assert_eq!(
            resolve(
                &value,
                &SourceBase::Document,
                &base,
                &mut UrlBudget { work, bytes }
            ),
            Err(UrlError::Limit)
        );
    }
}

#[test]
fn pure_decoding_does_not_enable_url_background_acceptance() {
    for value in ["url(A.png)", "url('A.png')", "url(#Icon)"] {
        assert!(decode(value, &mut generous()).unwrap().is_some());
        assert!(!crate::css::supports_property("background-image", value));
        assert!(!crate::css::supports_property("background", value));
    }
}
