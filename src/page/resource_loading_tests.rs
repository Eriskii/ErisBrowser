//! Independent HTTP/session fixtures. No new production resource selector.
use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const DOCUMENT: &[u8] = include_bytes!("../../tests/fixtures/page-resource-session/document.html");
const CSS: &[u8] = include_bytes!("../../tests/fixtures/page-resource-session/sheet.css");
const SCRIPT: &[u8] = include_bytes!("../../tests/fixtures/page-resource-session/ready.js");
const NEGATIVE: &[u8] = include_bytes!("../../tests/fixtures/page-resource-session/negative.html");
const CAP: &[u8] = include_bytes!("../../tests/fixtures/page-resource-session/cap.html");
const REDIRECTS: &[u8] =
    include_bytes!("../../tests/fixtures/page-resource-session/redirects.html");
const TWO_PIXELS: &[u8] =
    include_bytes!("../../tests/fixtures/page-resource-session/two-pixels.png");
const ONE_PIXEL: &[u8] = include_bytes!("../../tests/fixtures/page-resource-session/one-pixel.png");

struct Reply {
    status: &'static str,
    mime: &'static str,
    location: Option<String>,
    body: Vec<u8>,
}

impl Reply {
    fn ok(mime: &'static str, body: &[u8]) -> Self {
        Self {
            status: "200 OK",
            mime,
            location: None,
            body: body.to_vec(),
        }
    }

    fn redirect(location: String) -> Self {
        Self {
            status: "302 Found",
            mime: "text/plain",
            location: Some(location),
            body: Vec::new(),
        }
    }
}

struct Server {
    origin: String,
    requests: Arc<Mutex<Vec<(String, Instant)>>>,
    stop: Arc<AtomicBool>,
    task: Option<thread::JoinHandle<()>>,
}

impl Server {
    fn new(routes: HashMap<String, Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&requests);
        let stopped = Arc::clone(&stop);
        let task = thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("accept: {error}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut request = Vec::new();
                let mut buffer = [0; 1024];
                while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    let count = stream.read(&mut buffer).unwrap();
                    assert!(count > 0 && request.len() + count <= 8192);
                    request.extend_from_slice(&buffer[..count]);
                }
                let header = std::str::from_utf8(&request).unwrap();
                let mut words = header.split_ascii_whitespace();
                assert_eq!(words.next(), Some("GET"));
                let path = words.next().unwrap();
                {
                    let mut requests = seen.lock().unwrap();
                    requests.push((path.to_string(), Instant::now()));
                    assert!(requests.len() <= 128);
                }
                let fallback = Reply {
                    status: "404 Not Found",
                    mime: "text/plain",
                    location: None,
                    body: b"unexpected route".to_vec(),
                };
                let reply = routes.get(path).unwrap_or(&fallback);
                let mut headers = format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
                    reply.status,
                    reply.mime,
                    reply.body.len(),
                );
                if let Some(location) = &reply.location {
                    headers.push_str(&format!("Location: {location}\r\n"));
                }
                headers.push_str("\r\n");
                stream.write_all(headers.as_bytes()).unwrap();
                stream.write_all(&reply.body).unwrap();
            }
        });
        Self {
            origin,
            requests,
            stop,
            task: Some(task),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.origin)
    }

    fn paths(&self) -> Vec<String> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|(path, _)| path.clone())
            .collect()
    }

    fn first_request_at(&self) -> Instant {
        self.requests.lock().unwrap()[0].1
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(task) = self.task.take() {
            let result = task.join();
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }
}

fn accounting(page: &Page) -> net::FetcherAccounting {
    page.resources
        .as_ref()
        .unwrap()
        .fetcher
        .accounting_snapshot()
}

fn natural(page: &Page, id: &str, width: &str, height: &str) {
    let node = page.document.query_selector(id).unwrap();
    assert_eq!(
        page.document.attr(node, "data-eris-natural-width"),
        Some(width)
    );
    assert_eq!(
        page.document.attr(node, "data-eris-natural-height"),
        Some(height)
    );
}

#[test]
fn actual_http_load_retains_navigation_css_script_and_shared_image_accounting() {
    let server = Server::new(HashMap::from([
        ("/document.html".into(), Reply::ok("text/html", DOCUMENT)),
        ("/sheet.css".into(), Reply::ok("text/css", CSS)),
        ("/ready.js".into(), Reply::ok("text/javascript", SCRIPT)),
        ("/tile.png".into(), Reply::ok("image/png", TWO_PIXELS)),
    ]));
    let mut page = Page::load(&server.url("/document.html"), true).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert_eq!(
        server.paths(),
        ["/document.html", "/sheet.css", "/ready.js", "/tile.png"]
    );
    let observed = accounting(&page);
    assert_eq!(
        (observed.count, observed.total, observed.bridge_present),
        (4, 620, false)
    );
    assert!(observed.started <= server.first_request_at());
    assert_eq!(page.document.title(), "Resource ready");
    let status = page.document.query_selector("#status").unwrap();
    assert_eq!(page.document.text_content(status), "ready");
    let styles = css::compute_styles_from_sources(&page.document, &page.stylesheets(), 100.0, 60.0);
    assert_eq!(styles[status].color, crate::graphics::Color::rgb(1, 2, 3));
    let resources = page.resources.as_ref().unwrap();
    assert_eq!(resources.decoded_image_bytes, 8);
    assert!(resources.failed.is_empty());
    assert_eq!(resources.image_cache.len(), 1);
    let key = server.url("/tile.png");
    let (cached, clean) = &resources.image_cache[&key];
    assert!(*clean);
    assert_eq!((cached.width, cached.height), (2, 1));
    assert_eq!(cached.rgba, [255, 0, 0, 255, 0, 0, 255, 255]);
    assert_eq!(page.images.len(), 2);
    for alias in ["/tile.png", "./tile.png#copy"] {
        assert!(Arc::ptr_eq(cached, &page.images[alias]));
        assert_eq!(page.image_origin_clean(alias), Some(true));
    }
    natural(&page, "#first", "2", "1");
    natural(&page, "#second", "2", "1");
    assert_eq!(page.retained_image_bytes(), 8);
    let fonts = Fonts::new();
    let _ = page.layout(100.0, 60.0, &fonts);
    let button = page.document.query_selector("#mutate").unwrap();
    assert!(page.click(button).is_none());
    assert_eq!(page.document.title(), "Resource clicked");
    let first = page.document.query_selector("#first").unwrap();
    assert_eq!(page.document.attr(first, "src"), Some("/later.png"));
    natural(&page, "#first", "2", "1");
    assert!(!page.images.contains_key("/later.png"));
    page.images.clear();
    page.refresh_inline_svg();
    assert_eq!(page.retained_image_bytes(), 8);
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
    assert_eq!(accounting(&page), observed);
    assert_eq!(
        server.paths(),
        ["/document.html", "/sheet.css", "/ready.js", "/tile.png"]
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}

#[test]
fn failed_status_and_decode_keys_survive_without_later_layout_or_mutation_fetches() {
    let server = Server::new(HashMap::from([
        ("/negative.html".into(), Reply::ok("text/html", NEGATIVE)),
        (
            "/missing.png".into(),
            Reply {
                status: "404 Not Found",
                mime: "text/plain",
                location: None,
                body: b"missing".to_vec(),
            },
        ),
        ("/bad.png".into(), Reply::ok("image/png", b"not a PNG")),
    ]));
    let mut page = Page::load(&server.url("/negative.html"), true).unwrap();
    let observed = accounting(&page);
    assert_eq!(
        (observed.count, observed.total, observed.bridge_present),
        (3, 437, false)
    );
    assert_eq!(
        server.paths(),
        ["/negative.html", "/missing.png", "/bad.png"]
    );
    assert_eq!(page.diagnostics.len(), 2);
    assert!(page.diagnostics[0].contains("subresource returned HTTP 404"));
    assert!(page.diagnostics[1].starts_with("image /bad.png:"));
    let diagnostics = page.diagnostics.clone();
    let resources = page.resources.as_ref().unwrap();
    assert_eq!(resources.failed.len(), 2);
    for path in ["/missing.png", "/bad.png"] {
        assert!(
            resources
                .failed
                .contains(&(ResourceKind::Image, server.url(path)))
        );
    }
    assert!(resources.image_cache.is_empty());
    assert_eq!(resources.decoded_image_bytes, 0);
    let url_work = resources.resource_url_work_left;
    let fonts = Fonts::new();
    for _ in 0..2 {
        let _ = page.layout(120.0, 80.0, &fonts);
    }
    let button = page.document.query_selector("#mutate").unwrap();
    assert!(page.click(button).is_none());
    assert_eq!(page.document.title(), "Failures clicked");
    let missing = page.document.query_selector("#missing").unwrap();
    page.document.set_attr(missing, "class", "changed");
    page.document.set_attr(
        missing,
        "style",
        "width:2px;background-image:url(/never.png)",
    );
    let bad = page.document.query_selector("#bad").unwrap();
    page.document.set_attr(bad, "src", "/another.png");
    let base = page.document.query_selector("#base").unwrap();
    page.document.set_attr(base, "href", "/changed/");
    page.run_pending_tasks();
    page.refresh_inline_svg();
    let _ = page.layout(150.0, 90.0, &fonts);
    assert_eq!(page.document.attr(missing, "src"), Some("/later.png"));
    assert_eq!(page.document.attr(missing, "data-eris-natural-width"), None);
    assert!(page.images.is_empty());
    assert_eq!(page.retained_image_bytes(), 0);
    assert_eq!(accounting(&page), observed);
    assert_eq!(
        page.resources.as_ref().unwrap().resource_url_work_left,
        url_work
    );
    assert_eq!(page.resources.as_ref().unwrap().failed.len(), 2);
    assert_eq!(page.diagnostics, diagnostics);
    assert_eq!(
        server.paths(),
        ["/negative.html", "/missing.png", "/bad.png"]
    );
}

#[test]
fn document_and_forty_seven_images_keep_the_original_request_cap_after_load() {
    let mut routes = HashMap::from([("/cap.html".into(), Reply::ok("text/html", CAP))]);
    for index in 0..47 {
        routes.insert(
            format!("/image/{index:02}.png"),
            Reply::ok("image/png", ONE_PIXEL),
        );
    }
    let server = Server::new(routes);
    let mut page = Page::load(&server.url("/cap.html"), false).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let expected: Vec<_> = std::iter::once("/cap.html".to_string())
        .chain((0..47).map(|index| format!("/image/{index:02}.png")))
        .collect();
    assert_eq!(server.paths(), expected);
    let observed = accounting(&page);
    assert_eq!(
        (observed.count, observed.total, observed.bridge_present),
        (48, 4921, false)
    );
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 188);
    assert_eq!(page.resources.as_ref().unwrap().image_cache.len(), 47);
    assert_eq!(page.retained_image_bytes(), 188);
    let cached = page.resources.as_ref().unwrap().image_cache[&server.url("/image/00.png")].clone();
    assert_eq!(cached.0.rgba, [0, 255, 0, 255]);
    let extra = page.document.create_element("img");
    page.attach_image(
        extra,
        "cached-after-load".into(),
        Arc::clone(&cached.0),
        cached.1,
    );
    assert!(Arc::ptr_eq(&page.images["cached-after-load"], &cached.0));
    assert_eq!(accounting(&page), observed);
    let next = Url::parse(&server.url("/after.png")).unwrap();
    let initiator = page.url.clone();
    let result = page.resources.as_mut().unwrap().fetcher.fetch(
        &next,
        Some(&initiator),
        ResourceKind::Image,
    );
    assert_eq!(result.unwrap_err(), "page resource or time budget exceeded");
    let after = accounting(&page);
    assert_eq!(after.count, 49);
    assert_eq!(
        (after.total, after.started, after.bridge_present),
        (observed.total, observed.started, false)
    );
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 188);
    assert_eq!(page.retained_image_bytes(), 188);
    assert_eq!(server.paths(), expected);
}

#[test]
fn image_mime_navigation_keeps_its_separate_synthesized_image_fetch() {
    let server = Server::new(HashMap::from([(
        "/direct.png".into(),
        Reply::ok("image/png", TWO_PIXELS),
    )]));
    let page = Page::load(&server.url("/direct.png"), false).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert_eq!(page.document.title(), "Image");
    assert_eq!(server.paths(), ["/direct.png", "/direct.png"]);
    let observed = accounting(&page);
    assert_eq!(
        (observed.count, observed.total, observed.bridge_present),
        (2, 142, false)
    );
    assert!(observed.started <= server.first_request_at());
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
    assert_eq!(page.resources.as_ref().unwrap().image_cache.len(), 1);
    let key = server.url("/direct.png");
    assert_eq!(page.images.len(), 1);
    assert_eq!(page.images[&key].rgba, [255, 0, 0, 255, 0, 0, 255, 255]);
    assert_eq!(page.image_origin_clean(&key), Some(true));
    natural(&page, "img", "2", "1");
}

#[test]
fn distinct_redirect_requests_share_no_cache_identity_and_keep_cross_origin_taint() {
    let foreign = Server::new(HashMap::from([(
        "/shared.png".into(),
        Reply::ok("image/png", TWO_PIXELS),
    )]));
    let server = Server::new(HashMap::from([
        ("/redirects.html".into(), Reply::ok("text/html", REDIRECTS)),
        (
            "/first.png".into(),
            Reply::redirect(foreign.url("/shared.png")),
        ),
        (
            "/second.png".into(),
            Reply::redirect(foreign.url("/shared.png")),
        ),
    ]));
    let page = Page::load(&server.url("/redirects.html"), false).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert_eq!(
        server.paths(),
        ["/redirects.html", "/first.png", "/second.png"]
    );
    assert_eq!(foreign.paths(), ["/shared.png", "/shared.png"]);
    let observed = accounting(&page);
    assert_eq!(
        (observed.count, observed.total, observed.bridge_present),
        (3, 294, false)
    );
    let resources = page.resources.as_ref().unwrap();
    assert_eq!(resources.image_cache.len(), 2);
    assert_eq!(resources.decoded_image_bytes, 16);
    assert!(
        !resources
            .image_cache
            .contains_key(&foreign.url("/shared.png"))
    );
    let (first, first_clean) = &resources.image_cache[&server.url("/first.png")];
    let (second, second_clean) = &resources.image_cache[&server.url("/second.png")];
    assert!(!*first_clean && !*second_clean);
    assert!(!Arc::ptr_eq(first, second));
    assert_eq!(first.rgba, [255, 0, 0, 255, 0, 0, 255, 255]);
    assert_eq!(first.rgba, second.rgba);
    assert_eq!(page.images.len(), 3);
    assert!(Arc::ptr_eq(first, &page.images["/first.png"]));
    assert!(Arc::ptr_eq(first, &page.images["./first.png#same"]));
    assert!(Arc::ptr_eq(second, &page.images["/second.png"]));
    for alias in ["/first.png", "./first.png#same", "/second.png"] {
        assert_eq!(page.image_origin_clean(alias), Some(false));
    }
    for id in ["#first", "#again", "#second"] {
        natural(&page, id, "2", "1");
    }
    assert_eq!(page.retained_image_bytes(), 16);
}
