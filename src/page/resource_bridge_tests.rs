//! Private lifetime witnesses. The fake bridge is not an isolation proof.
use super::*;
use crate::net::Resource;
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Debug, PartialEq, Eq)]
struct Request {
    url: String,
    initiator: Option<String>,
    kind: ResourceKind,
    form: Option<String>,
}

thread_local! {
    static REQUESTS: RefCell<Vec<Request>> = const { RefCell::new(Vec::new()) };
}

const FINAL_DOCUMENT: &str = "https://document.test/final/page.html";
const HTML: &str = r#"<!doctype html><base id=base href="https://images.test/a/">
<img id=first src="tile.png"><img id=second src="./tile.png">
<button id=go onclick="document.getElementById('first').setAttribute('src','later.png');document.getElementById('base').setAttribute('href','https://elsewhere.test/new/');this.textContent='changed'">change</button>"#;

fn original_bridge(
    url: &Url,
    initiator: Option<&Url>,
    kind: ResourceKind,
    form: Option<&str>,
) -> Result<Resource, String> {
    REQUESTS.with(|requests| {
        requests.borrow_mut().push(Request {
            url: url.to_string(),
            initiator: initiator.map(ToString::to_string),
            kind,
            form: form.map(str::to_owned),
        });
    });
    match kind {
        ResourceKind::Document => {
            assert_eq!(url.as_str(), "https://document.test/start");
            assert!(initiator.is_none());
            Ok(Resource {
                url: Url::parse(FINAL_DOCUMENT).unwrap(),
                bytes: HTML.as_bytes().to_vec(),
                content_type: "text/html; charset=utf-8".into(),
                headers: BTreeMap::new(),
                status: 200,
                origin_clean: true,
                decoded_image: None,
            })
        }
        ResourceKind::Image => {
            assert_eq!(initiator.unwrap().as_str(), FINAL_DOCUMENT);
            assert!(matches!(
                url.as_str(),
                "https://images.test/a/tile.png" | "https://images.test/a/probe.png"
            ));
            assert!(form.is_none());
            Ok(Resource {
                // Opaque replies expose request identity, not a final redirect URL.
                url: url.clone(),
                bytes: Vec::new(),
                content_type: String::new(),
                headers: BTreeMap::new(),
                status: 200,
                origin_clean: false,
                decoded_image: Some(RasterImage {
                    width: 2,
                    height: 1,
                    rgba: vec![255, 0, 0, 255, 0, 255, 0, 128],
                }),
            })
        }
        other => panic!("unexpected bridge request: {other:?}"),
    }
}

fn forbidden_bridge(
    _: &Url,
    _: Option<&Url>,
    _: ResourceKind,
    _: Option<&str>,
) -> Result<Resource, String> {
    panic!("unexpected request through a newly installed bridge")
}

#[test]
fn page_retains_opaque_cache_taint_and_the_original_bridge_after_callbacks() {
    // The bridge and request recorder are thread-local; disposal prevents leaks
    // into other tests, including the real confined-worker integration tests.
    std::thread::spawn(|| {
        net::use_page_fetch_bridge(original_bridge);
        let mut page = Page::load_navigation(
            &Navigation {
                address: "https://document.test/start".into(),
                form_body: Some("a=b".into()),
            },
            true,
        )
        .unwrap();
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        assert_eq!(page.url.as_str(), FINAL_DOCUMENT);
        let session = page.resources.as_ref().unwrap();
        assert_eq!(session.decoded_image_bytes, 8);
        assert_eq!(session.image_cache.len(), 1);
        assert!(session.failed.is_empty());
        let (cached, clean) = &session.image_cache["https://images.test/a/tile.png"];
        assert!(!clean);
        assert_eq!(cached.rgba, [255, 0, 0, 255, 0, 255, 0, 128]);
        assert!(Arc::ptr_eq(cached, &page.images["tile.png"]));
        assert!(Arc::ptr_eq(cached, &page.images["./tile.png"]));
        assert_eq!(page.retained_image_bytes(), 8);
        for key in ["tile.png", "./tile.png"] {
            assert_eq!(page.image_origin_clean(key), Some(false));
        }
        for name in ["#first", "#second"] {
            let id = page.document.query_selector(name).unwrap();
            assert_eq!(page.document.attr(id, "data-eris-natural-width"), Some("2"));
            assert_eq!(
                page.document.attr(id, "data-eris-natural-height"),
                Some("1")
            );
        }
        let url_work_left = session.resource_url_work_left;
        let button = page.document.query_selector("#go").unwrap();
        assert!(page.click(button).is_none());
        page.run_pending_tasks();
        let fonts = Fonts::new();
        let _ = page.layout(120.0, 80.0, &fonts);
        assert_eq!(page.document.text_content(button), "changed");
        assert_eq!(
            page.document.base_url().as_str(),
            "https://elsewhere.test/new/"
        );
        assert_eq!(
            page.document
                .attr(page.document.query_selector("#first").unwrap(), "src"),
            Some("later.png")
        );
        let session = page.resources.as_ref().unwrap();
        assert_eq!(session.decoded_image_bytes, 8);
        assert_eq!(session.resource_url_work_left, url_work_left);
        REQUESTS.with(|requests| {
            assert_eq!(
                *requests.borrow(),
                [
                    Request {
                        url: "https://document.test/start".into(),
                        initiator: None,
                        kind: ResourceKind::Document,
                        form: Some("a=b".into())
                    },
                    Request {
                        url: "https://images.test/a/tile.png".into(),
                        initiator: Some(FINAL_DOCUMENT.into()),
                        kind: ResourceKind::Image,
                        form: None
                    },
                ]
            );
        });
        net::use_page_fetch_bridge(forbidden_bridge);
        let probe = Url::parse("https://images.test/a/probe.png").unwrap();
        let response = page
            .resources
            .as_mut()
            .unwrap()
            .fetcher
            .fetch(&probe, Some(&page.url), ResourceKind::Image)
            .unwrap();
        assert!(response.bytes.is_empty() && response.headers.is_empty());
        assert!(!response.origin_clean);
        assert_eq!(response.decoded_image.unwrap().rgba.len(), 8);
        // This private fetch does not install a raster or refund any accounting.
        assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
        REQUESTS.with(|requests| assert_eq!(requests.borrow().len(), 3));
    })
    .join()
    .unwrap();
}

#[test]
fn text_builtins_and_error_constructors_never_create_a_resource_session() {
    std::thread::spawn(|| {
        net::use_page_fetch_bridge(forbidden_bridge);
        let mut page = Page::from_html(
            Url::parse("https://document.test/page").unwrap(),
            "<link rel=stylesheet href='https://remote.test/a.css'><img id=i src='https://remote.test/a.png'><svg width=1 height=1><rect width=1 height=1 fill=red></rect></svg>",
            false,
        );
        assert!(page.resources.is_none());
        assert!(page.images.values().any(|image| image.rgba == [255, 0, 0, 255]));
        let id = page.document.query_selector("#i").unwrap();
        page.document.set_attr(id, "src", "https://remote.test/b.png");
        let fonts = Fonts::new();
        let _ = page.layout(100.0, 60.0, &fonts);
        page.refresh_inline_svg();
        assert!(page.resources.is_none());
        for address in ["eris:home", "about:blank"] {
            let builtin = Page::load(address, false).unwrap();
            assert!(builtin.resources.is_none());
        }
        let error = Page::error("https://remote.test/failed", "failure");
        assert!(error.resources.is_none());
    }).join().unwrap();
}
