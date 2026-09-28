use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts},
    page::Page,
};
use url::Url;

fn page(source: &str) -> Page {
    Page::from_html(
        Url::parse("https://example.test/base/page").unwrap(),
        source,
        true,
    )
}
#[test]
fn script_click_flows_through_dom_style_layout_and_pixels() {
    let mut p = page(
        "<style>body{margin:0}#box{width:100px;height:50px;background:red}</style><div id=box></div><button id=go>Change</button><script>document.getElementById('go').addEventListener('click', function(){document.getElementById('box').style.backgroundColor='blue';});</script>",
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let fonts = Fonts::new();
    let mut c = Canvas::new(200, 100).unwrap();
    c.paint(
        &p.layout(200.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(c.pixels[20 * 200 + 20], 0xff0000);
    p.click(p.document.query_selector("#go").unwrap());
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    c.clear(Color::WHITE);
    c.paint(
        &p.layout(200.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(c.pixels[20 * 200 + 20], 0x0000ff);
}

#[test]
fn namespace_demo_rebuilds_svg_with_a_prototype_switch_and_paints_each_palette() {
    use eris::dom::Namespace;

    let mut p = Page::from_html(
        Url::parse("https://example.test/namespaces.html").unwrap(),
        include_str!("../examples/namespaces.html"),
        true,
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let initial_svg = p.document.query_selector("#scene").unwrap();
    let initial_key = format!("eris-inline-svg:{initial_svg}");
    let initial_image = &p.images[&initial_key];
    assert_eq!((initial_image.width, initial_image.height), (760, 240));
    // The 380:160 viewBox meets a 760:240 viewport with transparent side bands.
    // This checks that the adjusted preserveAspectRatio attribute reaches SVG paint.
    let side_pixel = (120 * 760 + 40) * 4;
    assert_eq!(&initial_image.rgba[side_pixel..side_pixel + 4], &[0; 4]);
    let center_pixel = (120 * 760 + 215) * 4;
    assert_eq!(
        &initial_image.rgba[center_pixel..center_pixel + 4],
        &[0x5b, 0xd4, 0xba, 255]
    );

    let button = p.document.query_selector("#advance").unwrap();
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(960, 900).unwrap();
    for (step, name, fill, rgb) in [
        (1, "Amber", "#ffc46b", 0xffc46b),
        (2, "Iris", "#b7a5f5", 0xb7a5f5),
        (3, "Seafoam", "#5bd4ba", 0x5bd4ba),
    ] {
        assert!(p.click(button).is_none());
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let svg = p.document.query_selector("#scene").unwrap();
        let accent = p.document.query_selector("#accent").unwrap();
        assert_ne!(svg, initial_svg);
        assert_eq!(p.document.namespace(svg), Some(Namespace::Svg));
        assert_eq!(p.document.namespace(accent), Some(Namespace::Svg));
        assert_eq!(p.document.attr(svg, "viewBox"), Some("0 0 380 160"));
        assert_eq!(p.document.attr(svg, "viewbox"), None);
        assert_eq!(
            p.document.attr(svg, "preserveAspectRatio"),
            Some("xMidYMid meet")
        );
        assert_eq!(p.document.attr(accent, "fill"), Some(fill));
        for (selector, expected) in [
            ("#state", format!("{name} / update {step}")),
            ("#namespace", Namespace::Svg.uri().to_owned()),
            ("#viewport", "0 0 380 160 / xMidYMid meet".to_owned()),
            (
                "#runtime",
                format!(
                    "SceneCycle instance: true / prototype method / switch case {}",
                    step % 3
                ),
            ),
        ] {
            assert_eq!(
                p.document
                    .text_content(p.document.query_selector(selector).unwrap()),
                expected
            );
        }
        assert!(!p.images.contains_key(&initial_key));
        assert_eq!(p.images.len(), 1, "removed SVG rasters must be released");
        canvas.clear(Color::WHITE);
        canvas.paint(
            &p.layout(960.0, 900.0, &fonts).commands,
            &fonts,
            &p.images,
            0.0,
            0.0,
        );
        assert!(
            canvas.pixels.iter().filter(|pixel| **pixel == rgb).count() > 8_000,
            "the {name} SVG palette must reach the page canvas"
        );
    }
}

#[test]
fn form_get_serializes_successful_controls_and_clicks_do_not_submit_text_inputs() {
    let mut p = page(
        "<form action=/search><input name=q value='hello world'><input type=checkbox name=on checked><input type=checkbox name=off><input name=no disabled value=bad><textarea name=body>α&amp;β</textarea><button name=go value=yes>Go</button></form>",
    );
    let input = p.document.query_selector("input").unwrap();
    assert!(p.click(input).is_none());
    let button = p.document.query_selector("button").unwrap();
    let target = p.click(button).unwrap();
    let pairs = Url::parse(&target.address)
        .unwrap()
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect::<Vec<_>>();
    assert_eq!(
        pairs,
        vec![
            ("q".into(), "hello world".into()),
            ("on".into(), "on".into()),
            ("body".into(), "α&β".into()),
            ("go".into(), "yes".into())
        ]
    );
    let unchecked = p.document.query_selector("input[name=off]").unwrap();
    p.click(unchecked);
    assert!(p.document.attr(unchecked, "checked").is_some());
}
#[test]
fn cancelled_links_and_forms_stay_in_document() {
    let mut p = page(
        "<a href='/next' onclick='event.preventDefault()'>Link</a><form action='/send' onsubmit='event.preventDefault()'><button>Send</button></form>",
    );
    assert!(p.click(p.document.query_selector("a").unwrap()).is_none());
    assert!(
        p.click(p.document.query_selector("button").unwrap())
            .is_none()
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
}
#[test]
fn scripts_disabled_and_template_content_inert() {
    let p = Page::from_html(
        Url::parse("https://example.test").unwrap(),
        "<p id=x>original</p><script>document.getElementById('x').textContent='changed';</script><template><h1>hidden template</h1></template>",
        false,
    );
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "original"
    );
    let result = p.layout(400.0, 200.0, &Fonts::new());
    assert!(
        !result
            .commands
            .iter()
            .any(|c| matches!(c,DrawCommand::Text{text,..} if text.contains("template")))
    );
}

#[test]
fn noscript_parsing_and_rendering_follow_the_page_scripting_flag() {
    let source = "<!doctype html><body><p id=main>Page</p><noscript><p id=fallback>Fallback content</p><script>document.getElementById('main').textContent='wrong';</script></noscript>";
    let enabled = Page::from_html(Url::parse("https://example.test/").unwrap(), source, true);
    let disabled = Page::from_html(Url::parse("https://example.test/").unwrap(), source, false);
    assert!(enabled.document.query_selector("#fallback").is_none());
    assert!(disabled.document.query_selector("#fallback").is_some());
    assert_eq!(
        enabled
            .document
            .text_content(enabled.document.query_selector("#main").unwrap()),
        "Page"
    );
    let fonts = Fonts::new();
    let has_fallback = |p: &Page| {
        p.layout(400.0, 200.0, &fonts).commands.iter().any(
            |command| matches!(command,DrawCommand::Text{text,..} if text.contains("Fallback")),
        )
    };
    assert!(!has_fallback(&enabled));
    assert!(has_fallback(&disabled));
    assert!(enabled.diagnostics.is_empty(), "{:?}", enabled.diagnostics);
}

#[test]
fn json_demo_reads_a_textarea_runs_callbacks_and_updates_the_document() {
    let mut page = Page::from_html(
        Url::parse("https://example.test/standards.html").unwrap(),
        include_str!("../examples/standards.html"),
        true,
    );
    let button = page.document.query_selector("#format").unwrap();
    assert!(page.click(button).is_none());
    let output = page.document.query_selector("#result").unwrap();
    assert_eq!(
        page.document.text_content(output),
        "{\n  \"title\": \"Custom engine\",\n  \"count\": 14,\n  \"items\": [\n    2,\n    4,\n    6\n  ]\n}"
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}
#[test]
fn file_resource_loading_preserves_stylesheet_order_and_script_events() {
    let path = std::env::temp_dir().join(format!("eris-pipeline-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("site.css"), "#box{background:red}").unwrap();
    std::fs::write(
        path.join("main.js"),
        "document.getElementById('box').textContent='loaded';",
    )
    .unwrap();
    std::fs::write(path.join("index.html"),"<link rel=stylesheet href=site.css><style>#box{background:blue;width:100px;height:30px}body{margin:0}</style><div id=box>start</div><script src=main.js></script>").unwrap();
    let p = Page::load(path.join("index.html").to_str().unwrap(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#box").unwrap()),
        "loaded"
    );
    let fonts = Fonts::new();
    let mut c = Canvas::new(200, 100).unwrap();
    c.paint(
        &p.layout(200.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(c.pixels[25 * 200 + 80], 0x0000ff);
    std::fs::remove_dir_all(path).unwrap();
}
#[test]
fn local_file_scope_follows_canonical_paths() {
    use eris::net::{Fetcher, ResourceKind};
    let path = std::env::temp_dir().join(format!("eris-scope-{}", std::process::id()));
    std::fs::create_dir_all(path.join("allowed")).unwrap();
    std::fs::write(path.join("allowed/index.html"), "ok").unwrap();
    std::fs::write(path.join("outside.txt"), "outside").unwrap();
    let url = Url::from_file_path(path.join("allowed/index.html")).unwrap();
    let mut fetcher = Fetcher::for_document(&url);
    assert!(fetcher.fetch(&url, None, ResourceKind::Document).is_ok());
    assert!(
        fetcher
            .fetch(
                &url.join("../outside.txt").unwrap(),
                Some(&url),
                ResourceKind::Image
            )
            .is_err()
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(path.join("outside.txt"), path.join("allowed/link.txt"))
            .unwrap();
        assert!(
            fetcher
                .fetch(
                    &url.join("link.txt").unwrap(),
                    Some(&url),
                    ResourceKind::Image
                )
                .is_err()
        );
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn template_scripts_styles_metadata_and_resources_are_inert() {
    let path = std::env::temp_dir().join(format!("eris-template-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("index.html"), "<body><p id=x>original</p><template><meta http-equiv='Content-Security-Policy' content=\"default-src 'none'\"><link rel=stylesheet href=missing.css><style>p{color:red}</style><script>document.getElementById('x').textContent='unsafe';</script><script src=missing.js></script><img src=missing.png><svg width=10 height=10><rect width=10 height=10 fill=red /></svg></template><script>document.getElementById('x').textContent='active';</script>").unwrap();
    let p = Page::load(path.join("index.html").to_str().unwrap(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert!(p.scripts_enabled);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "active"
    );
    assert!(p.stylesheets().is_empty());
    assert!(p.images.is_empty());
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn from_html_enforces_meta_csp_before_scripts_and_styles() {
    let p = page(
        "<meta http-equiv='content-security-policy' content=\"script-src 'none'\"><style>p{color:red}</style><p id=x style='color:blue'>original</p><script>document.getElementById('x').textContent='unsafe';</script>",
    );
    let paragraph = p.document.query_selector("#x").unwrap();
    assert!(!p.scripts_enabled);
    assert_eq!(p.document.text_content(paragraph), "original");
    assert!(p.document.attr(paragraph, "style").is_none());
    assert!(p.stylesheets().is_empty());
    assert_eq!(p.diagnostics.len(), 1);
    assert!(p.load_ms > 0.0);
}

#[test]
fn disabled_control_ancestors_block_events_and_submission_except_first_legend() {
    let mut p = page(
        "<p id=x>original</p><form action=/search><fieldset disabled><legend><input id=legend type=checkbox name=legend value=yes></legend><input id=locked type=checkbox name=locked checked><button id=disabled><span id=child onclick=\"document.getElementById('x').textContent='unsafe'\">Blocked</span></button><legend><input name=later value=bad></legend></fieldset><div id=plain>Not a submitter</div><button id=submit>Go</button></form><div inert><a id=inert href=/unsafe>Inert</a></div>",
    );
    assert!(
        p.click(p.document.query_selector("#child").unwrap())
            .is_none()
    );
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "original"
    );
    let locked = p.document.query_selector("#locked").unwrap();
    assert!(p.click(locked).is_none());
    assert!(p.document.attr(locked, "checked").is_some());
    let legend = p.document.query_selector("#legend").unwrap();
    p.click(legend);
    assert!(p.document.attr(legend, "checked").is_some());
    assert!(
        p.click(p.document.query_selector("#plain").unwrap())
            .is_none()
    );
    assert!(
        p.click(p.document.query_selector("#inert").unwrap())
            .is_none()
    );
    let target = p
        .click(p.document.query_selector("#submit").unwrap())
        .unwrap();
    assert_eq!(
        Url::parse(&target.address).unwrap().query(),
        Some("legend=yes")
    );
}

#[test]
fn form_select_serializes_enabled_selected_options_and_limits_output() {
    let mut p = page(
        "<form><select name=choice multiple><optgroup disabled><option selected value=bad>Bad</option></optgroup><optgroup><option selected value=one>One</option><option selected value=two>Two</option><option disabled selected value=bad2>Bad</option></optgroup></select><button>Go</button></form>",
    );
    let button = p.document.query_selector("button").unwrap();
    assert_eq!(
        Url::parse(&p.click(button).unwrap().address)
            .unwrap()
            .query(),
        Some("choice=one&choice=two")
    );
    let input = p.document.create_element("input");
    p.document.set_attr(input, "name", "large");
    p.document
        .set_attr(input, "value", &"a".repeat(1024 * 1024 + 1));
    p.document
        .append_child(p.document.query_selector("form").unwrap(), input);
    assert!(p.click(button).is_none());
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.contains("submission byte budget"))
    );
}

#[test]
fn inline_svg_reaches_pixels_and_removed_nodes_release_rasters() {
    let mut p = page(
        "<style>body{margin:0}</style><div id=holder><svg width=20 height=20 viewBox='0 0 20 20'><rect width=20 height=20 fill='#12843c'/></svg></div>",
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let svg = p.document.query_selector("svg").unwrap();
    let key = format!("eris-inline-svg:{svg}");
    assert!(p.images.contains_key(&key));
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(80, 60).unwrap();
    canvas.paint(
        &p.layout(80.0, 60.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(canvas.pixels[10 * 80 + 10], 0x12843c);
    let holder = p.document.query_selector("#holder").unwrap();
    p.document.set_text_content(holder, "removed");
    p.refresh_inline_svg();
    assert!(!p.images.contains_key(&key));
}

#[test]
fn repeated_external_images_share_rasters_and_resource_budget() {
    let path = std::env::temp_dir().join(format!("eris-svg-resources-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("image.svg"),
        "<svg width='12' height='8'><rect width='12' height='8' fill='#285fac'/></svg>",
    )
    .unwrap();
    let mut html = "<style>body{margin:0}img{display:block}</style>".to_owned();
    for n in 0..80 {
        html.push_str(if n % 2 == 0 {
            "<img src='image.svg'>"
        } else {
            "<img src='./image.svg'>"
        });
    }
    std::fs::write(path.join("index.html"), html).unwrap();
    let p = Page::load(path.join("index.html").to_str().unwrap(), false).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.images.len(), 2);
    assert!(std::sync::Arc::ptr_eq(
        p.images.get("image.svg").unwrap(),
        p.images.get("./image.svg").unwrap()
    ));
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(80, 60).unwrap();
    canvas.paint(
        &p.layout(80.0, 60.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(canvas.pixels[4 * 80 + 6], 0x285fac);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn post_navigation_uses_action_query_and_submitter_overrides() {
    let mut p = page(
        "<form action='/default?old=1' method=get enctype='multipart/form-data'><input name=q value='one two'><button id=post formaction='/posted?keep=1' formmethod=PoSt formenctype='application/x-www-form-urlencoded' name=go value=yes>Post</button><button id=get>Get</button></form>",
    );
    let post = p
        .click(p.document.query_selector("#post").unwrap())
        .unwrap();
    assert_eq!(post.address, "https://example.test/posted?keep=1");
    assert_eq!(post.form_body.as_deref(), Some("q=one+two&go=yes"));
    let get = p.click(p.document.query_selector("#get").unwrap()).unwrap();
    assert_eq!(get.address, "https://example.test/default?q=one+two");
    assert!(get.form_body.is_none());
}

#[test]
fn post_rejects_unsupported_encoding_downgrade_and_encoded_payload_growth() {
    for attributes in [
        "method=post enctype='multipart/form-data'",
        "method=post enctype='text/plain'",
        "method=post action='http://example.test/unsafe'",
    ] {
        let mut p = page(&format!(
            "<form {attributes}><input name=q value=test><button>Send</button></form>"
        ));
        assert!(
            p.click(p.document.query_selector("button").unwrap())
                .is_none()
        );
        assert!(!p.diagnostics.is_empty());
    }
    let mut p = page("<form method=post><input name=q><button>Send</button></form>");
    let input = p.document.query_selector("input").unwrap();
    p.document.set_attr(input, "value", &"&".repeat(400_000));
    assert!(
        p.click(p.document.query_selector("button").unwrap())
            .is_none()
    );
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.contains("submission byte budget"))
    );
}

#[test]
fn navigation_post_rejects_local_and_builtin_schemes_before_loading() {
    use eris::page::Navigation;
    for address in [
        "file:///etc/passwd",
        "data:text/html,hello",
        "about:blank",
        "eris:home",
    ] {
        let navigation = Navigation {
            address: address.into(),
            form_body: Some("q=test".into()),
        };
        assert!(Page::load_navigation(&navigation, false).is_err());
    }
    assert_eq!(Navigation::get("https://example.test").form_body, None);
}

#[test]
fn urlencoded_forms_normalize_line_breaks_for_get_and_post() {
    for method in ["get", "post"] {
        let mut p = page(&format!(
            "<form method={method}><textarea name=body>one\ntwo\rthree\r\nfour</textarea><button>Send</button></form>"
        ));
        let navigation = p
            .click(p.document.query_selector("button").unwrap())
            .unwrap();
        let encoded = navigation.form_body.unwrap_or_else(|| {
            Url::parse(&navigation.address)
                .unwrap()
                .query()
                .unwrap()
                .to_owned()
        });
        assert_eq!(encoded, "body=one%0D%0Atwo%0D%0Athree%0D%0Afour");
    }
}

#[test]
fn fragment_identifiers_decode_utf8_without_form_plus_rules() {
    let p = page("<p id='café'>one</p><p id='a+b'>two</p>");
    let id = eris::page::find_fragment(&p.document, "caf%C3%A9").unwrap();
    assert_eq!(p.document.attr(id, "id"), Some("café"));
    let id = eris::page::find_fragment(&p.document, "a+b").unwrap();
    assert_eq!(p.document.attr(id, "id"), Some("a+b"));
}

#[test]
fn authoritative_control_edit_policy_blocks_inert_disabled_and_readonly_nodes() {
    let p = page(
        "<input id=a><input id=b readonly><fieldset disabled><input id=c></fieldset><template><input id=d></template><input id=e type=HIDDEN><input id=f type=PASSWORD><input id=g type=CHECKBOX><textarea id=h></textarea>",
    );
    for name in ["a", "f", "h"] {
        assert!(
            p.can_edit_control(p.document.query_selector(&format!("#{name}")).unwrap()),
            "{name}"
        );
    }
    for name in ["b", "c", "d", "e", "g"] {
        assert!(
            !p.can_edit_control(p.document.query_selector(&format!("#{name}")).unwrap()),
            "{name}"
        );
    }
    assert!(!p.can_edit_control(usize::MAX));
}
