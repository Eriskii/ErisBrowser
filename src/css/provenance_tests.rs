use super::*;

fn sheet(text: impl Into<Arc<str>>, url: &Arc<::url::Url>) -> StyleSource {
    StyleSource::with_base(text, SourceBase::Stylesheet(url.clone()))
}

fn rules(source: &StyleSource) -> Vec<Rule> {
    let mut result = Vec::new();
    parse_source(
        &source.source,
        &source.layer,
        &source.base,
        100.0,
        100.0,
        &mut result,
        &mut ParseBudget::new(),
    );
    result
}

fn same_base(base: &SourceBase, expected: &Arc<::url::Url>) {
    let SourceBase::Stylesheet(actual) = base else {
        panic!("expected stylesheet provenance");
    };
    assert!(Arc::ptr_eq(actual, expected));
}

#[test]
fn document_wrappers_and_explicit_shared_source_handles() {
    assert!(matches!(SourceBase::default(), SourceBase::Document));
    let source = StyleSource::new("#a{color:red}");
    assert!(matches!(source.base, SourceBase::Document));
    assert!(source.layer.is_none());
    assert!(source.media.is_empty());
    let direct = parse_stylesheet("@layer x {#a{color:red}}", 100.0, 100.0);
    assert_eq!(direct.len(), 2);
    assert!(
        direct
            .iter()
            .all(|r| matches!(r.base, SourceBase::Document))
    );
    let layer = CascadeLayer::named(None, "outer").unwrap();
    let inherited = parse_stylesheet_in_layer("#a{color:red}", Some(layer.clone()), 100.0, 100.0);
    assert_eq!(inherited.len(), 2);
    for rule in &inherited {
        assert!(matches!(rule.base, SourceBase::Document));
        assert!(Arc::ptr_eq(rule.layer.as_ref().unwrap(), &layer));
    }
    let url = Arc::new(::url::Url::parse("https://example.test/css/final.css?Q=1").unwrap());
    let text: Arc<str> = "#a{color:blue}".into();
    let external = sheet(text.clone(), &url);
    let cloned = external.clone();
    assert!(Arc::ptr_eq(&external.source, &text));
    assert!(Arc::ptr_eq(&cloned.source, &text));
    same_base(&external.base, &url);
    same_base(&cloned.base, &url);
    same_base(&rules(&external)[0].base, &url);
}

#[test]
fn provenance_survives_conditional_and_layer_recursion_including_markers() {
    let url = Arc::new(::url::Url::parse("https://example.test/import/final.css").unwrap());
    let mut source = sheet(
        "@layer first,second;\
         @media (min-width:1px){@supports(color:red){@layer first{#a{color:red}}}}\
         @layer second{#a{color:blue}}\
         @container ignored{@layer ghost;#never{color:black}}\
         @media(min-width:999px){#no{color:purple}}",
        &url,
    );
    source.layer = CascadeLayer::named(None, "outer");
    let parsed = rules(&source);
    assert_eq!(parsed.len(), 8);
    assert_eq!(
        parsed
            .iter()
            .map(|r| r.selectors.join(","))
            .collect::<Vec<_>>(),
        ["", "", "", "", "#a", "", "#a", ""],
    );
    assert_eq!(
        parsed
            .iter()
            .map(|r| r.layer.as_ref().unwrap().name.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("outer"),
            Some("first"),
            Some("second"),
            Some("first"),
            Some("first"),
            Some("second"),
            Some("second"),
            Some("ghost")
        ],
    );
    assert_eq!(parsed[4].declarations[0].value, "red");
    assert_eq!(parsed[6].declarations[0].value, "blue");
    for rule in &parsed {
        same_base(&rule.base, &url);
    }
}

#[test]
fn adjacent_sources_keep_distinct_handles_and_independent_eof() {
    let first_url = Arc::new(::url::Url::parse("https://example.test/a/final.css").unwrap());
    let second_url = Arc::new(::url::Url::parse("https://example.test/b/final.css").unwrap());
    let first = sheet("#a{color:red}/* unclosed", &first_url);
    let mut second = sheet("#a{color:blue}", &second_url);
    let mut parsed = Vec::new();
    let mut budget = ParseBudget::new();
    for source in [&first, &second] {
        parse_source(
            &source.source,
            &source.layer,
            &source.base,
            100.0,
            100.0,
            &mut parsed,
            &mut budget,
        );
    }
    assert_eq!(parsed.len(), 2);
    same_base(&parsed[0].base, &first_url);
    same_base(&parsed[1].base, &second_url);
    assert_eq!(parsed[0].declarations[0].value, "red");
    assert_eq!(parsed[1].declarations[0].value, "blue");
    second.base = SourceBase::Document;
    same_base(&parsed[1].base, &second_url);
    let equal_url = Arc::new((*second_url).clone());
    assert!(!Arc::ptr_eq(&equal_url, &second_url));
    same_base(
        &rules(&sheet("#a{color:green}", &equal_url))[0].base,
        &equal_url,
    );
}

#[test]
fn provenance_preserves_cascade_and_resolves_url_backgrounds_at_consumption() {
    let text = "@layer first {#a{color:red;--paint:url(../Images/Logo.png?Q=é#Mark)}}\
                @layer second {#a{color:blue;color:revert-layer}}\
                #a{background-image:var(--paint)}";
    let url = Arc::new(::url::Url::parse("https://example.test/css/final.css").unwrap());
    let doc = Document::parse("<div id=a></div>");
    let id = doc.query_selector("#a").unwrap();
    let document = compute_styles_from_sources(&doc, &[StyleSource::new(text)], 100.0, 100.0);
    let external = compute_styles_from_sources(&doc, &[sheet(text, &url)], 100.0, 100.0);
    assert_eq!(document[id].color, Color::rgb(255, 0, 0));
    assert_eq!(doc.base_url().as_str(), "about:blank");
    let Some(BackgroundImage::Url(document_image)) = &document[id].backgrounds.image_slots()[0]
    else {
        panic!("expected document URL image");
    };
    assert_eq!(
        document_image.as_ref(),
        &url::ResolvedUrl::Unresolved("../Images/Logo.png?Q=é#Mark".into())
    );
    let Some(BackgroundImage::Url(external_image)) = &external[id].backgrounds.image_slots()[0]
    else {
        panic!("expected external URL image");
    };
    let url::ResolvedUrl::Absolute(external_url) = external_image.as_ref() else {
        panic!("expected absolute stylesheet URL");
    };
    assert_eq!(
        external_url.as_str(),
        "https://example.test/Images/Logo.png?Q=%C3%A9#Mark"
    );
    let mut document_without_image = document[id].clone();
    let mut external_without_image = external[id].clone();
    document_without_image.backgrounds = BackgroundData::initial();
    external_without_image.backgrounds = BackgroundData::initial();
    assert_eq!(
        format!("{document_without_image:?}"),
        format!("{external_without_image:?}")
    );
    let mut work = 100_000;
    assert_eq!(
        valid_background_property("background-image", "url(../Images/Logo.png)", &mut work),
        Ok(true)
    );
}

#[test]
fn custom_property_tokens_remain_unresolved_through_inheritance() {
    let literal = "url(../Images/Logo.png?Q=é#Mark)";
    let declarations = parse_declarations(&format!("--paint:{literal}"));
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].value, literal);
    let mut retained = 0;
    let mut work = 100_000;
    let parent = compute_custom_properties(
        &Arc::new(BTreeMap::new()),
        BTreeMap::from([("--paint".into(), literal.into())]),
        &mut retained,
        &mut work,
    );
    let child = compute_custom_properties(&parent, BTreeMap::new(), &mut retained, &mut work);
    assert!(Arc::ptr_eq(&parent, &child));
    assert_eq!(child.get("--paint").unwrap().as_ref(), literal);
    assert_eq!(
        resolve_vars("var(--paint)", &child, 0, &mut work).as_deref(),
        Some(literal)
    );
}

#[test]
fn source_count_and_eight_mib_text_capacity_stay_separate_from_handles() {
    let url = Arc::new(::url::Url::parse("https://example.test/css/final.css").unwrap());
    let doc = Document::parse("<div id=a></div>");
    let id = doc.query_selector("#a").unwrap();
    let mut sources = vec![sheet("", &url); 256];
    sources[0] = sheet("#a{color:red}", &url);
    sources[255] = sheet("#a{color:blue}", &url);
    sources.push(sheet("#a{color:green}", &url));
    assert_eq!(
        compute_styles_from_sources(&doc, &sources, 100.0, 100.0)[id].color,
        Color::rgb(0, 0, 255)
    );
    let chunk = |tail: &str| {
        let mut text = " ".repeat(4 * 1024 * 1024 - tail.len());
        text.push_str(tail);
        sheet(text, &url)
    };
    let pair = [chunk("#a{color:red}"), chunk("#a{color:green}")];
    assert_eq!(pair[0].source.len() + pair[1].source.len(), MAX_STYLE_BYTES);
    assert_eq!(
        compute_styles_from_sources(&doc, &pair, 100.0, 100.0)[id].color,
        Color::rgb(0, 128, 0)
    );
}

#[test]
fn rule_count_and_parser_budgets_bound_shared_handles_without_url_copies() {
    let text: Arc<str> = "#a{color:red}".repeat(10_001).into();
    let url = Arc::new(::url::Url::parse("https://example.test/css/final.css").unwrap());
    let external = sheet(text.clone(), &url);
    let document = StyleSource::new(text);
    let mut result = Vec::new();
    let mut external_budget = ParseBudget::new();
    parse_source(
        &external.source,
        &external.layer,
        &external.base,
        100.0,
        100.0,
        &mut result,
        &mut external_budget,
    );
    assert_eq!(result.len(), 10_000);
    assert_eq!(Arc::strong_count(&url), 10_002);
    for rule in &result {
        same_base(&rule.base, &url);
    }
    let mut ordinary = Vec::new();
    let mut document_budget = ParseBudget::new();
    parse_source(
        &document.source,
        &document.layer,
        &document.base,
        100.0,
        100.0,
        &mut ordinary,
        &mut document_budget,
    );
    assert_eq!(ordinary.len(), 10_000);
    assert_eq!(external_budget.work, document_budget.work);
    assert_eq!(external_budget.names, document_budget.names);
    assert_eq!(external_budget.layers, document_budget.layers);
    assert_eq!(external_budget.declarations, document_budget.declarations);
    drop(result);
    assert_eq!(Arc::strong_count(&url), 2);
    drop(external);
    assert_eq!(Arc::strong_count(&url), 1);
}
