use super::*;
fn sheet(text: &str, base: &str) -> StyleSource {
    StyleSource::with_base(
        text,
        SourceBase::Stylesheet(Arc::new(::url::Url::parse(base).unwrap())),
    )
}
fn document(text: &str) -> Document {
    let mut doc = Document::parse(text);
    doc.initialize_url(::url::Url::parse("https://document.test/root/index.html").unwrap());
    doc
}
fn url_value(style: &ComputedStyle) -> &Arc<url::ResolvedUrl> {
    match style.backgrounds.image_slots()[0].as_ref().unwrap() {
        BackgroundImage::Url(value) => value,
        BackgroundImage::Gradient(_) => panic!("expected URL"),
    }
}
fn absolute(style: &ComputedStyle) -> &str {
    match url_value(style).as_ref() {
        url::ResolvedUrl::Absolute(value) => value.as_str(),
        other => panic!("expected absolute URL: {other:?}"),
    }
}
#[test]
fn selected_rule_inline_and_rollback_keep_actual_consuming_base() {
    let doc = document(
        "<div id=a></div><div id=b style='background-image:url(Tile.PNG)'></div><div id=c></div>",
    );
    let sources = [
        sheet(
            "@layer low,high;@layer low{#a,#c{background-image:url(Tile.PNG)}}",
            "https://a.test/final/main.css",
        ),
        sheet(
            "@layer high{#a{background-image:url(Other.PNG);background-image:revert-layer}}#c{background-image:url(Tile.PNG)}",
            "https://b.test/import/end.css",
        ),
    ];
    let styles = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    for (id, expected) in [
        ("#a", "https://a.test/final/Tile.PNG"),
        ("#b", "https://document.test/root/Tile.PNG"),
        ("#c", "https://b.test/import/Tile.PNG"),
    ] {
        assert_eq!(absolute(&styles[doc.query_selector(id).unwrap()]), expected);
    }
}
#[test]
fn vars_pending_shorthand_and_computed_inheritance_distinguish_origins() {
    let doc = document("<main id=p><div id=i></div><div id=v></div></main><div id=s></div>");
    let sources = [
        sheet(
            "#p{--tile:url(Tile.PNG);background-image:var(--tile)}",
            "https://a.test/a/main.css",
        ),
        sheet(
            "#i{background-image:inherit}#v{background-image:var(--tile)}#s{--tile:url(Tile.PNG);background:var(--tile) center / 8px 4px no-repeat red}",
            "https://b.test/b/end.css",
        ),
    ];
    let styles = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    let at = |id| &styles[doc.query_selector(id).unwrap()];
    assert_eq!(absolute(at("#p")), "https://a.test/a/Tile.PNG");
    assert_eq!(absolute(at("#i")), "https://a.test/a/Tile.PNG");
    assert!(Arc::ptr_eq(url_value(at("#p")), url_value(at("#i"))));
    assert_eq!(absolute(at("#v")), "https://b.test/b/Tile.PNG");
    assert_eq!(absolute(at("#s")), "https://b.test/b/Tile.PNG");
    assert_eq!(at("#s").backgrounds.repeats()[0], [false, false]);
    assert_eq!(at("#s").background_color, Color::rgb(255, 0, 0));
}
#[test]
fn admitted_empty_local_unresolved_and_invalid_values_have_distinct_states() {
    let doc = document(
        "<div id=e></div><div id=l></div><div id=u></div><div id=bad></div><div id=reset></div>",
    );
    let sources = [StyleSource::new(
        "#e{background-image:linear-gradient(red);background-image:url()}#l{background-image:url(#Icon)}#u{background-image:url('http://[invalid')}#bad{background-image:url(Kept.PNG);background-image:url(A B)}#reset{background-image:url(A);all:initial}",
    )];
    let styles = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    let at = |id| &styles[doc.query_selector(id).unwrap()];
    assert_eq!(url_value(at("#e")).as_ref(), &url::ResolvedUrl::Empty);
    assert_eq!(
        url_value(at("#l")).as_ref(),
        &url::ResolvedUrl::LocalFragment("#Icon".into())
    );
    assert_eq!(
        url_value(at("#u")).as_ref(),
        &url::ResolvedUrl::Unresolved("http://[invalid".into())
    );
    assert_eq!(absolute(at("#bad")), "https://document.test/root/Kept.PNG");
    assert!(at("#reset").backgrounds.image_slots()[0].is_none());
}
#[test]
fn token_payload_survives_declaration_post_var_and_supports_routes() {
    for value in ["url(A/**/B.PNG)", r"u\72l(A\)B.PNG)", "url('A;!{B}.PNG')"] {
        assert!(supports_declaration("background-image", value), "{value}");
        assert!(
            supports_matches(&format!("(background-image:{value})")),
            "{value}"
        );
        assert!(supports_property("background-image", value), "{value}");
    }
    let doc = document("<div id=a></div><div id=b></div><div id=c></div>");
    let sources = [StyleSource::new(
        r"#a{background-image:url(A/**/B.PNG)}#b{--tile:url(A/**/B.PNG);background-image:var(--tile)}#c{background-image:u\72l(A\)B.PNG)}",
    )];
    let styles = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    for (id, expected) in [
        ("#a", "https://document.test/root/A/**/B.PNG"),
        ("#b", "https://document.test/root/A/**/B.PNG"),
        ("#c", "https://document.test/root/A)B.PNG"),
    ] {
        assert_eq!(absolute(&styles[doc.query_selector(id).unwrap()]), expected);
    }
    for value in ["url(A);color:red", "url(A)!important", "url(A),,url(B)"] {
        assert!(!supports_declaration("background-image", value));
    }
}
#[test]
fn document_sources_share_storage_but_external_origins_do_not() {
    let doc = document("<div id=a></div><div id=b></div><div id=c></div><div id=d></div>");
    let sources = [
        StyleSource::new("#a{background-image:url(A)}#b{background-image:url(A)}"),
        sheet("#c{background-image:url(A)}", "https://x.test/a.css"),
        sheet("#d{background-image:url(A)}", "https://x.test/b.css"),
    ];
    let styles = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    let at = |id| &styles[doc.query_selector(id).unwrap()];
    assert!(Arc::ptr_eq(url_value(at("#a")), url_value(at("#b"))));
    assert!(!Arc::ptr_eq(url_value(at("#c")), url_value(at("#d"))));
    assert_eq!(absolute(at("#c")), absolute(at("#d")));
}
#[test]
fn document_base_recomputation_does_not_rebase_external_urls() {
    let mut doc = document("<div id=a></div><div id=b></div>");
    let sources = [
        StyleSource::new("#a{background-image:url(A)}"),
        sheet("#b{background-image:url(A)}", "https://s.test/style.css"),
    ];
    let before = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    doc.set_url(::url::Url::parse("https://changed.test/else/index.html").unwrap());
    let after = compute_styles_from_sources(&doc, &sources, 80.0, 60.0);
    let a = doc.query_selector("#a").unwrap();
    let b = doc.query_selector("#b").unwrap();
    assert_eq!(absolute(&before[a]), "https://document.test/root/A");
    assert_eq!(absolute(&after[a]), "https://changed.test/else/A");
    assert_eq!(absolute(&before[b]), "https://s.test/A");
    assert_eq!(absolute(&after[b]), "https://s.test/A");
}
