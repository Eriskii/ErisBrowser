//! Source URL identity through actual loading and stylesheet recollection.
use super::*;
use crate::css::{SourceBase, StyleSource, url as css_url};
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

const DOCUMENT: &str = include_str!("../../tests/fixtures/css-url-provenance/document.html");
const MAIN: &str = include_str!("../../tests/fixtures/css-url-provenance/main.css");
const IMPORT: &str = include_str!("../../tests/fixtures/css-url-provenance/import.css");

struct Server {
    origin: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    task: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&requests);
        let stopped = Arc::clone(&stop);
        let task = thread::spawn(move || {
            let start = Instant::now();
            while !stopped.load(Ordering::Acquire) && start.elapsed() < Duration::from_secs(10) {
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(e) => panic!("accept: {e}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut input = Vec::new();
                let mut buffer = [0u8; 1024];
                while !input.windows(4).any(|x| x == b"\r\n\r\n") {
                    let n = stream.read(&mut buffer).unwrap();
                    assert!(n > 0 && input.len() + n <= 8192);
                    input.extend_from_slice(&buffer[..n]);
                }
                let header = std::str::from_utf8(&input).unwrap();
                let path = header.split_ascii_whitespace().nth(1).unwrap();
                seen.lock().unwrap().push(path.into());
                let (status, metadata, body) = match path {
                    "/page.html" => ("200 OK", "Content-Type: text/html\r\n", DOCUMENT),
                    "/start.css" => ("302 Found", "Location: /assets/v1/main.css\r\n", ""),
                    "/assets/v1/main.css" => ("200 OK", "Content-Type: text/css\r\n", MAIN),
                    "/theme/import.css" => ("302 Found", "Location: /cdn/theme/final.css\r\n", ""),
                    "/cdn/theme/final.css" => ("200 OK", "Content-Type: text/css\r\n", IMPORT),
                    _ => (
                        "404 Not Found",
                        "Content-Type: text/plain\r\n",
                        "unexpected request",
                    ),
                };
                write!(stream, "HTTP/1.1 {status}\r\n{metadata}Content-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        Self {
            origin,
            requests,
            stop,
            task: Some(task),
        }
    }
    fn paths(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(task) = self.task.take() {
            task.join().unwrap();
        }
    }
}

fn absolute(source: &StyleSource, document: &Document, value: &str) -> String {
    let mut budget = css_url::UrlBudget {
        work: 1_000_000,
        bytes: 1_000_000,
    };
    let value = css_url::decode(value, &mut budget).unwrap().unwrap();
    match css_url::resolve(&value, &source.base, document.base_url(), &mut budget).unwrap() {
        css_url::ResolvedUrl::Absolute(url) => url.to_string(),
        other => panic!("expected absolute URL: {other:?}"),
    }
}
fn source_with<'a>(sources: &'a [StyleSource], needle: &str) -> &'a StyleSource {
    sources.iter().find(|s| s.source.contains(needle)).unwrap()
}

#[test]
fn stylesheet_provenance_survives_redirects_imports_and_document_base_changes() {
    let server = Server::new();
    let mut page = Page::load(&format!("{}/page.html", server.origin), false).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let sources = page.stylesheets();
    assert_eq!(sources.len(), 3);
    let main = source_with(&sources, "--local");
    let imported = source_with(&sources, "--picture");
    let embedded = source_with(&sources, "--embedded");
    assert!(matches!(embedded.base, SourceBase::Document));
    for (source, path) in [
        (main, "/assets/v1/main.css"),
        (imported, "/cdn/theme/final.css"),
    ] {
        let SourceBase::Stylesheet(base) = &source.base else {
            panic!("external base")
        };
        assert_eq!(base.as_str(), format!("{}{path}", server.origin));
    }
    let probe = page.document.query_selector("#probe").unwrap();
    let fonts = Fonts::new();
    let mut first_pixels = None;
    for base_path in ["/first/", "/second/"] {
        let base = page.document.query_selector("#base").unwrap();
        page.document.set_attr(base, "href", base_path);
        let recollected = page.stylesheets();
        let next_main = source_with(&recollected, "--local");
        let (SourceBase::Stylesheet(before), SourceBase::Stylesheet(after)) =
            (&main.base, &next_main.base)
        else {
            panic!("external handles")
        };
        assert!(Arc::ptr_eq(before, after));
        assert_eq!(
            absolute(next_main, &page.document, "url(Tile.PNG)"),
            format!("{}/assets/v1/Tile.PNG", server.origin)
        );
        assert_eq!(
            absolute(
                source_with(&recollected, "--picture"),
                &page.document,
                "url(Icon.PNG)"
            ),
            format!("{}/cdn/theme/Icon.PNG", server.origin)
        );
        assert_eq!(
            absolute(
                source_with(&recollected, "--embedded"),
                &page.document,
                "url(Tile.PNG)"
            ),
            format!("{}{base_path}Tile.PNG", server.origin)
        );
        let styles = css::compute_styles_from_sources(&page.document, &recollected, 160.0, 60.0);
        assert_eq!(styles[probe].color, crate::graphics::Color::rgb(255, 0, 0));
        assert!(styles[probe].background_image.is_none());
        let layout = page.layout(160.0, 60.0, &fonts);
        let mut canvas = crate::graphics::Canvas::new(160, 60).unwrap();
        canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        if let Some(first) = &first_pixels {
            assert_eq!(&canvas.pixels, first);
        } else {
            first_pixels = Some(canvas.pixels);
        }
        assert!(page.images.is_empty());
    }
    assert_eq!(
        server.paths(),
        [
            "/page.html",
            "/start.css",
            "/assets/v1/main.css",
            "/theme/import.css",
            "/cdn/theme/final.css"
        ]
    );
    assert!(!css::supports_declaration(
        "background-image",
        "url(Tile.PNG)"
    ));
}

#[test]
fn cached_inline_segments_keep_separate_bases_until_the_style_text_changes() {
    let mut page = Page::from_html(
        Url::parse("https://example.test/docs/page").unwrap(),
        "<base id=b href='/first/'><style id=s media=screen>p{color:red}</style><p id=p>x</p>",
        false,
    );
    let style = page.document.query_selector("#s").unwrap();
    let original: Arc<str> = "p{color:red}".into();
    page.inline_styles.insert(
        style,
        (
            Arc::clone(&original),
            vec![
                StyleSource::with_base(
                    "p{--import:url(Icon.PNG)}",
                    SourceBase::Stylesheet(Arc::new(
                        Url::parse("https://cdn.test/css/final.css").unwrap(),
                    )),
                ),
                StyleSource::new(original),
            ],
        ),
    );
    let old = page.stylesheets();
    assert_eq!(old.len(), 2);
    assert!(
        old.iter()
            .all(|s| s.media.iter().map(AsRef::as_ref).collect::<Vec<&str>>() == ["screen"])
    );
    page.document.set_attr(
        page.document.query_selector("#b").unwrap(),
        "href",
        "/second/",
    );
    page.document.set_encoding(encoding_rs::WINDOWS_1252);
    assert_eq!(
        absolute(&old[0], &page.document, "url(Icon.PNG)"),
        "https://cdn.test/css/Icon.PNG"
    );
    assert_eq!(
        absolute(&old[1], &page.document, "url(é?q=é)"),
        "https://example.test/second/%C3%A9?q=%C3%A9"
    );
    assert_eq!(
        crate::document_url::parse(&page.document, page.document.base_url(), "é?q=é")
            .unwrap()
            .as_str(),
        "https://example.test/second/%C3%A9?q=%E9"
    );
    page.document.set_text_content(style, "p{color:blue}");
    let changed = page.stylesheets();
    assert_eq!(changed.len(), 1);
    assert!(matches!(changed[0].base, SourceBase::Document));
    assert_eq!(changed[0].source.as_ref(), "p{color:blue}");
    let styles = css::compute_styles_from_sources(&page.document, &changed, 160.0, 60.0);
    assert_eq!(
        styles[page.document.query_selector("#p").unwrap()].color,
        crate::graphics::Color::rgb(0, 0, 255)
    );
    assert!(page.images.is_empty());
}
