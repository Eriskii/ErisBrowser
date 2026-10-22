//! Literal page/bridge witnesses; fake bridges do not prove process isolation.
use super::*;
use crate::graphics::DrawCommand;
use std::{cell::RefCell, collections::BTreeMap};

thread_local! {
    static SOURCE: RefCell<String> = const { RefCell::new(String::new()) };
    static REQUESTS: RefCell<Vec<(String, ResourceKind)>> = const { RefCell::new(Vec::new()) };
}

fn bridge(
    url: &Url,
    initiator: Option<&Url>,
    kind: ResourceKind,
    form: Option<&str>,
) -> Result<net::Resource, String> {
    assert!(form.is_none());
    REQUESTS.with(|r| r.borrow_mut().push((url.to_string(), kind)));
    if kind == ResourceKind::Document {
        assert!(initiator.is_none());
        return Ok(net::Resource {
            url: Url::parse("https://page.test/final/page").unwrap(),
            bytes: SOURCE.with(|s| s.borrow().as_bytes().to_vec()),
            content_type: "text/html".into(),
            headers: BTreeMap::new(),
            status: 200,
            origin_clean: true,
            decoded_image: None,
        });
    }
    assert_eq!(kind, ResourceKind::Image);
    assert_eq!(initiator.unwrap().as_str(), "https://page.test/final/page");
    if url.path().ends_with("missing.png") {
        return Err("fixture refused".into());
    }
    Ok(net::Resource {
        url: url.clone(),
        bytes: Vec::new(),
        content_type: String::new(),
        headers: BTreeMap::new(),
        status: 200,
        origin_clean: false,
        decoded_image: Some(RasterImage {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 255, 0, 255],
        }),
    })
}

fn with_page(source: &str, run: impl FnOnce(&mut Page) + Send + 'static) {
    let source = source.to_owned();
    std::thread::spawn(move || {
        SOURCE.with(|s| *s.borrow_mut() = source);
        net::use_page_fetch_bridge(bridge);
        let mut page = Page::load("https://page.test/start", false).unwrap();
        run(&mut page);
    })
    .join()
    .unwrap();
}

fn requests() -> Vec<String> {
    REQUESTS.with(|r| r.borrow().iter().map(|(u, _)| u.clone()).collect())
}

fn keys(result: &LayoutResult) -> Vec<&str> {
    result
        .commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::BackgroundImage { key, .. } => Some(key.as_str()),
            DrawCommand::BoxDecoration(value) => match &value.paint {
                crate::graphics::box_decoration::BoxPaint::Image { key, .. } => Some(key.as_str()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn prepare_shares_img_cache_taint_and_preserves_dom_and_readonly_layout() {
    with_page(
        "<style>body{margin:0}#a,#b{width:8px;height:3px;background-image:url(tile.png#one)}</style><img src='tile.png'><div id=a></div><div id=b style='background-image:url(./tile.png#two)'></div>",
        |page| {
            let fonts = Fonts::new();
            assert!(keys(&page.layout(100.0, 80.0, &fonts)).is_empty());
            let nodes = page.document.nodes.len();
            let result = page.prepare_layout(100.0, 80.0, &fonts);
            let selected = keys(&result);
            assert_eq!(selected.len(), 2);
            assert_eq!(selected[0], selected[1]);
            assert!(Arc::ptr_eq(
                &page.images[selected[0]],
                &page.images["tile.png"]
            ));
            assert_eq!(page.image_origin_clean(selected[0]), Some(false));
            assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
            assert_eq!(page.document.nodes.len(), nodes);
            for id in ["#a", "#b"] {
                let id = page.document.query_selector(id).unwrap();
                assert!(page.document.attr(id, "data-eris-natural-width").is_none());
            }
            let _ = page.prepare_layout(100.0, 80.0, &fonts);
            assert_eq!(keys(&page.layout(100.0, 80.0, &fonts)).len(), 2);
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/final/tile.png"
                ]
            );
        },
    );
}

#[test]
fn new_urls_are_loaded_after_completed_style_base_and_media_changes() {
    with_page(
        "<base id=base href='/a/'><style>#x{width:8px;height:3px;background-image:url(Tile.PNG)}@media(min-width:200px){#x{background-image:url(Wide.PNG)}}</style><div id=x></div>",
        |page| {
            let fonts = Fonts::new();
            assert_eq!(keys(&page.prepare_layout(100.0, 80.0, &fonts)).len(), 1);
            let base = page.document.query_selector("#base").unwrap();
            page.document.set_attr(base, "href", "/b/");
            assert!(keys(&page.layout(100.0, 80.0, &fonts)).is_empty());
            assert_eq!(keys(&page.prepare_layout(100.0, 80.0, &fonts)).len(), 1);
            assert_eq!(keys(&page.prepare_layout(240.0, 80.0, &fonts)).len(), 1);
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/a/Tile.PNG",
                    "https://page.test/b/Tile.PNG",
                    "https://page.test/b/Wide.PNG"
                ]
            );
        },
    );
}

#[test]
fn unavailable_urls_never_revive_losers_refetch_or_grow_diagnostics() {
    with_page(
        "<style>html{background-image:url(missing.png)}body{background-image:url(body.png)}#x{height:4px;background-image:url(older.png);background-image:url(missing.png)}</style><div id=x></div><div style='background-image:url()'></div><div style='background-image:url(#local)'></div>",
        |page| {
            let fonts = Fonts::new();
            page.prepare_layout(80.0, 60.0, &fonts);
            let count = page.diagnostics.len();
            for _ in 0..4 {
                page.prepare_layout(80.0, 60.0, &fonts);
            }
            assert_eq!(page.diagnostics.len(), count);
            let urls = requests();
            assert_eq!(
                urls.iter().filter(|u| u.ends_with("missing.png")).count(),
                1
            );
            assert!(
                !urls
                    .iter()
                    .any(|u| u.ends_with("older.png") || u.ends_with("#local"))
            );
            assert_eq!(
                page.resources
                    .as_ref()
                    .unwrap()
                    .failed
                    .iter()
                    .filter(|(kind, _)| *kind == ResourceKind::Image)
                    .count(),
                1
            );
        },
    );
}

#[test]
fn selection_omits_inert_hidden_and_closed_details_subtrees() {
    with_page(
        "<style>div,span,summary{height:4px}</style><template><div style='background-image:url(template.png)'></div></template><div style='display:none'><span style='background-image:url(hidden.png)'></span></div><input type=hidden style='background-image:url(input.png)'><details><summary style='background-image:url(summary.png)'>show</summary><div style='background-image:url(closed.png)'></div></details><div style='background-image:url(visible.png)'></div>",
        |page| {
            let fonts = Fonts::new();
            page.prepare_layout(80.0, 60.0, &fonts);
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/final/summary.png",
                    "https://page.test/final/visible.png"
                ]
            );
        },
    );
}

#[test]
fn internal_keys_are_collision_checked_and_reauthenticated() {
    with_page(
        "<div style='height:4px;background-image:url(tile.png)'></div>",
        |page| {
            let fonts = Fonts::new();
            let stranger = Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 255, 255],
            });
            page.images
                .insert("eris-css-background:0".into(), stranger.clone());
            let result = page.prepare_layout(80.0, 60.0, &fonts);
            let key = keys(&result)[0].to_owned();
            assert_eq!(key, "eris-css-background:1");
            assert!(Arc::ptr_eq(
                &page.images["eris-css-background:0"],
                &stranger
            ));
            page.images.insert(key.clone(), stranger.clone());
            assert!(keys(&page.layout(80.0, 60.0, &fonts)).is_empty());
            let repaired = page.prepare_layout(80.0, 60.0, &fonts);
            assert_eq!(keys(&repaired), ["eris-css-background:2"]);
            assert!(Arc::ptr_eq(&page.images[&key], &stranger));
            assert_eq!(requests().len(), 2);
        },
    );
}

#[test]
fn post_load_live_pixels_and_external_debits_both_limit_new_backgrounds() {
    with_page(
        "<div style='height:4px;background-image:url(tile.png)'></div>",
        |page| {
            let held = Arc::new(RasterImage {
                width: 4095,
                height: 4096,
                rgba: vec![0; 67_092_480],
            });
            let resources = page.resources.as_mut().unwrap();
            resources
                .image_cache
                .insert("https://page.test/held".into(), (held, false));
            resources.decoded_image_bytes = 67_092_480;
            page.images.insert(
                "inline-witness".into(),
                Arc::new(RasterImage {
                    width: 64,
                    height: 64,
                    rgba: vec![0; 16_384],
                }),
            );
            assert_eq!(page.retained_image_bytes(), 67_108_864);
            assert!(keys(&page.prepare_layout(80.0, 60.0, &Fonts::new())).is_empty());
            assert_eq!(
                page.resources.as_ref().unwrap().decoded_image_bytes,
                67_092_480
            );
            assert_eq!(page.retained_image_bytes(), 67_108_864);
            page.images.remove("inline-witness");
            page.prepare_layout(80.0, 60.0, &Fonts::new());
            assert_eq!(requests().len(), 2); // Negative cache is retained after refusal.
        },
    );
}

#[test]
fn honest_bridge_request_49_is_refused_locally_with_original_session() {
    let mut html = String::new();
    for i in 0..47 {
        html.push_str(&format!("<img src='/{i}.png'>"));
    }
    html.push_str("<div style='height:4px;background-image:url(last.png)'></div>");
    with_page(&html, |page| {
        assert_eq!(requests().len(), 48);
        let before = page
            .resources
            .as_ref()
            .unwrap()
            .fetcher
            .accounting_snapshot();
        assert!(keys(&page.prepare_layout(80.0, 60.0, &Fonts::new())).is_empty());
        assert_eq!(requests().len(), 48);
        let after = page
            .resources
            .as_ref()
            .unwrap()
            .fetcher
            .accounting_snapshot();
        assert_eq!(before, after);
        assert_eq!(after.bridge_attempts, 48);
    });
}

#[test]
fn constructors_and_conservative_style_policy_do_not_gain_fetch_authority() {
    std::thread::spawn(|| {
        net::use_page_fetch_bridge(bridge);
        let mut page = Page::from_html(
            Url::parse("https://page.test/page").unwrap(),
            "<div style='height:4px;background-image:url(tile.png)'></div>",
            false,
        );
        assert!(keys(&page.prepare_layout(80.0, 60.0, &Fonts::new())).is_empty());
        assert!(requests().is_empty());
        assert!(page.resources.is_none());
    })
    .join()
    .unwrap();
    with_page(
        "<meta http-equiv=content-security-policy content=\"default-src 'none'\"><div style='height:4px;background-image:url(tile.png)'></div>",
        |page| {
            assert!(keys(&page.prepare_layout(80.0, 60.0, &Fonts::new())).is_empty());
            assert_eq!(requests(), ["https://page.test/start"]);
        },
    );
}

#[test]
fn full_fragment_clone_and_key_publication_have_distinct_metadata_boundaries() {
    // Request32 bytes, full URL153 bytes, generated key21 bytes. Full URL
    // reservation971 + candidate-key64 + key publication660 =1695 bytes.
    let source = format!(
        "<div style='height:4px;background-image:url(tile.png#{})'></div>",
        "a".repeat(120)
    );
    for (allowance, expected_requests, expected_keys, expected_decoded) in
        [(970, 1, 0, 0), (1694, 2, 0, 8), (1695, 2, 1, 8)]
    {
        with_page(&source, move |page| {
            let resources = page.resources.as_mut().unwrap();
            resources.background_metadata_left = allowance;
            resources.resource_url_work_left = 2268;
            let result = page.prepare_layout(80.0, 60.0, &Fonts::new());
            assert_eq!(requests().len(), expected_requests);
            assert_eq!(keys(&result).len(), expected_keys);
            let resources = page.resources.as_ref().unwrap();
            assert_eq!(resources.decoded_image_bytes, expected_decoded);
            assert_eq!(resources.background_metadata_left, 0);
        });
    }
}

#[test]
fn key_work_cut_keeps_decoded_admission_but_does_not_publish_partial_alias() {
    // Request32 and key21 bytes:384 URL work +256 collision probe +660
    // publication =1300. The paint key is absent when publication is refused.
    for (work, expected_keys) in [(1299, 0), (1300, 1)] {
        with_page(
            "<div style='height:4px;background-image:url(tile.png)'></div>",
            move |page| {
                page.resources.as_mut().unwrap().resource_url_work_left = work;
                let result = page.prepare_layout(80.0, 60.0, &Fonts::new());
                assert_eq!(keys(&result).len(), expected_keys);
                let resources = page.resources.as_ref().unwrap();
                assert_eq!(resources.resource_url_work_left, 0);
                assert_eq!(resources.decoded_image_bytes, 8);
                assert_eq!(resources.image_cache.len(), 1);
                assert_eq!(resources.background_keys.len(), expected_keys);
            },
        );
    }
}

#[path = "background_resource_layers_tests.rs"]
mod layers;

// Initial suppression tests selection before any image can enter the cache.
// Later hiding retains that cache; this is not an eviction/cancellation test.
fn canvas_display_resource_reveal_sequence(source: &str) {
    with_page(source, |page| {
        let fonts = Fonts::new();
        assert_eq!(requests(), ["https://page.test/start"]);
        let hidden = page.prepare_layout(8.0, 6.0, &fonts);
        assert_eq!(requests(), ["https://page.test/start"]);
        assert!(keys(&hidden).is_empty());
        assert!(page.images.is_empty());
        let resources = page.resources.as_ref().unwrap();
        assert!(resources.image_cache.is_empty());
        assert!(resources.background_keys.is_empty());
        assert_eq!(resources.decoded_image_bytes, 0);

        page.runtime
            .execute(
                r#"document.getElementById('root').style.display = 'block';"#,
                &mut page.document,
            )
            .unwrap();
        let shown = page.prepare_layout(8.0, 6.0, &fonts);
        let selected = keys(&shown);
        assert_eq!(selected.len(), 1);
        let key = selected[0].to_owned();
        assert_eq!(
            requests(),
            [
                "https://page.test/start",
                "https://page.test/final/tile.png"
            ]
        );
        assert_eq!(page.images.len(), 1);
        let resources = page.resources.as_ref().unwrap();
        assert_eq!(resources.image_cache.len(), 1);
        assert_eq!(resources.background_keys.len(), 1);
        assert_eq!(resources.decoded_image_bytes, 8);

        page.runtime
            .execute(
                r#"document.getElementById('root').style.display = 'none';"#,
                &mut page.document,
            )
            .unwrap();
        assert!(keys(&page.prepare_layout(8.0, 6.0, &fonts)).is_empty());
        assert_eq!(
            requests(),
            [
                "https://page.test/start",
                "https://page.test/final/tile.png"
            ]
        );
        assert_eq!(page.images.len(), 1);
        let resources = page.resources.as_ref().unwrap();
        assert_eq!(resources.image_cache.len(), 1);
        assert_eq!(resources.background_keys.len(), 1);
        assert_eq!(resources.decoded_image_bytes, 8);

        page.runtime
            .execute(
                r#"document.getElementById('root').style.display = 'block';"#,
                &mut page.document,
            )
            .unwrap();
        let again = page.prepare_layout(8.0, 6.0, &fonts);
        assert_eq!(keys(&again), [key.as_str()]);
        assert_eq!(
            requests(),
            [
                "https://page.test/start",
                "https://page.test/final/tile.png"
            ]
        );
        assert_eq!(page.images.len(), 1);
        let resources = page.resources.as_ref().unwrap();
        assert_eq!(resources.image_cache.len(), 1);
        assert_eq!(resources.background_keys.len(), 1);
        assert_eq!(resources.decoded_image_bytes, 8);
    });
}

#[test]
fn canvas_display_root_none_root_url_waits_for_reveal() {
    canvas_display_resource_reveal_sequence(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:none;width:8px;height:6px;background:transparent;background-image:url(tile.png);background-repeat:no-repeat}body{width:8px;height:6px;background:transparent}</style></head><body id=body></body></html>"#,
    );
}

#[test]
fn canvas_display_root_none_body_url_waits_for_reveal() {
    canvas_display_resource_reveal_sequence(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:none;width:8px;height:6px;background:transparent}body{width:8px;height:6px;background:transparent;background-image:url(tile.png);background-repeat:no-repeat}</style></head><body id=body></body></html>"#,
    );
}

#[test]
fn canvas_display_displayed_root_and_body_urls_prepare_once() {
    for source in [
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;width:8px;height:6px;background:transparent;background-image:url(tile.png);background-repeat:no-repeat}body{width:8px;height:6px;background:transparent}</style></head><body id=body></body></html>"#,
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;width:8px;height:6px;background:transparent}body{width:8px;height:6px;background:transparent;background-image:url(tile.png);background-repeat:no-repeat}</style></head><body id=body></body></html>"#,
    ] {
        with_page(source, |page| {
            let fonts = Fonts::new();
            assert_eq!(requests(), ["https://page.test/start"]);
            let result = page.prepare_layout(8.0, 6.0, &fonts);
            let selected = keys(&result);
            assert_eq!(selected.len(), 1);
            let key = selected[0].to_owned();
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/final/tile.png"
                ]
            );
            assert_eq!(keys(&page.layout(8.0, 6.0, &fonts)), [key.as_str()]);
            assert_eq!(keys(&page.prepare_layout(8.0, 6.0, &fonts)), [key.as_str()]);
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/final/tile.png"
                ]
            );
        });
    }
}
