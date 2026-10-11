//! Literal URL-background pixels and explicit preparation/resource provenance.
use eris::{
    graphics::{Canvas, Fonts},
    layout::LayoutResult,
    page::Page,
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const DOCUMENT: &str = r##"<!doctype html><base id=b href="/document-base/"><link rel=stylesheet href="/entry.css"><style>html,body{margin:0;background:white}div{position:absolute;width:4px;height:2px;background-size:4px 2px;background-repeat:no-repeat}#external{left:0;top:0}#imported{left:6px;top:0}#inline{left:0;top:4px;background-image:url(Inline.PNG)}#variable{left:6px;top:4px;background-image:var(--asset)}</style><div id=external></div><div id=imported></div><div id=inline></div><div id=variable></div>"##;
const MAIN: &str = "@import '../imports/theme.css'; #external{background-image:url(Pattern.PNG)}";
const IMPORTED: &str = ":root{--asset:url(Var.PNG)} #imported{background-image:url(Imported.PNG)}";
const RED: &[u8] = include_bytes!("reftests/url-background-assets/red.png");
const GREEN: &[u8] = include_bytes!("reftests/url-background-assets/green.png");
const BLUE: &[u8] = include_bytes!("reftests/url-background-assets/blue.png");
const YELLOW: &[u8] = include_bytes!("reftests/url-background-assets/yellow.png");
const CYAN: &[u8] = include_bytes!("reftests/url-background-assets/cyan.png");
const MAGENTA: &[u8] = include_bytes!("reftests/url-background-assets/magenta.png");

struct Reply {
    status: &'static str,
    mime: &'static str,
    location: Option<&'static str>,
    body: &'static [u8],
}
impl Reply {
    fn ok(mime: &'static str, body: &'static [u8]) -> Self {
        Self {
            status: "200 OK",
            mime,
            location: None,
            body,
        }
    }
    fn redirect(location: &'static str) -> Self {
        Self {
            status: "302 Found",
            mime: "text/plain",
            location: Some(location),
            body: b"",
        }
    }
}
struct Server {
    origin: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    task: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new() -> Self {
        let routes = HashMap::from([
            (
                "/document.html",
                Reply::ok("text/html", DOCUMENT.as_bytes()),
            ),
            ("/entry.css", Reply::redirect("/sheet/main.css")),
            ("/sheet/main.css", Reply::ok("text/css", MAIN.as_bytes())),
            ("/imports/theme.css", Reply::redirect("/final/theme.css")),
            (
                "/final/theme.css",
                Reply::ok("text/css", IMPORTED.as_bytes()),
            ),
            ("/sheet/Pattern.PNG", Reply::ok("image/png", RED)),
            ("/final/Imported.PNG", Reply::ok("image/png", GREEN)),
            ("/document-base/Inline.PNG", Reply::ok("image/png", BLUE)),
            ("/document-base/Var.PNG", Reply::ok("image/png", YELLOW)),
            ("/changed/Inline.PNG", Reply::ok("image/png", CYAN)),
            ("/changed/Var.PNG", Reply::ok("image/png", MAGENTA)),
        ]);
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
                    let mut paths = seen.lock().unwrap();
                    paths.push(path.to_string());
                    assert!(paths.len() <= 32);
                }
                let fallback = Reply {
                    status: "404 Not Found",
                    mime: "text/plain",
                    location: None,
                    body: b"unexpected route",
                };
                let reply = routes.get(path).unwrap_or(&fallback);
                let mut headers = format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
                    reply.status,
                    reply.mime,
                    reply.body.len()
                );
                if let Some(location) = reply.location {
                    headers.push_str(&format!("Location: {location}\r\n"));
                }
                headers.push_str("\r\n");
                stream.write_all(headers.as_bytes()).unwrap();
                stream.write_all(reply.body).unwrap();
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
            let result = task.join();
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }
}
fn color(value: u8) -> u32 {
    match value {
        b'W' => 0xffffff,
        b'R' => 0xff0000,
        b'G' => 0x00ff00,
        b'B' => 0x0000ff,
        b'C' => 0x00ffff,
        b'M' => 0xff00ff,
        b'Y' => 0xffff00,
        b'K' => 0x000000,
        b'O' => 0xff8000,
        _ => panic!("literal palette"),
    }
}
fn pixels(page: &Page, layout: &LayoutResult, fonts: &Fonts, width: u32, height: u32) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas.pixels
}
fn expected_rows(rows: &[&str]) -> Vec<u32> {
    rows.iter().flat_map(|row| row.bytes().map(color)).collect()
}
type Hit = (usize, eris::layout::HitAction, f32, f32, f32, f32, bool);
fn hits(layout: &LayoutResult) -> Vec<Hit> {
    layout
        .hit_regions
        .iter()
        .map(|hit| {
            (
                hit.node,
                hit.action,
                hit.rect.x,
                hit.rect.y,
                hit.rect.width,
                hit.rect.height,
                hit.fixed,
            )
        })
        .collect()
}
#[test]
fn explicit_prepare_preserves_redirect_import_and_consuming_variable_bases() {
    let server = Server::new();
    let mut page = Page::load(&format!("{}/document.html", server.origin), false).unwrap();
    let fonts = Fonts::new();
    let initial = [
        "/document.html",
        "/entry.css",
        "/sheet/main.css",
        "/imports/theme.css",
        "/final/theme.css",
    ];
    assert_eq!(server.paths(), initial);
    let graph: Vec<_> = page
        .document
        .nodes
        .iter()
        .map(|node| (node.parent, node.children.clone()))
        .collect();
    let before = page.layout(12.0, 8.0, &fonts);
    let baseline_hits = hits(&before);
    for (selector, x, y) in [
        ("#external", 0.0, 0.0),
        ("#imported", 6.0, 0.0),
        ("#inline", 0.0, 4.0),
        ("#variable", 6.0, 4.0),
    ] {
        let id = page.document.query_selector(selector).unwrap();
        assert!(before.hit_regions.iter().any(|hit| hit.node == id
            && !hit.fixed
            && (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height) == (x, y, 4.0, 2.0)));
    }
    assert_eq!(pixels(&page, &before, &fonts, 12, 8), vec![0xffffff; 96]);
    assert_eq!(server.paths(), initial, "read-only layout must not fetch");
    let first = page.prepare_layout(12.0, 8.0, &fonts);
    let prepared_paths = [
        "/document.html",
        "/entry.css",
        "/sheet/main.css",
        "/imports/theme.css",
        "/final/theme.css",
        "/sheet/Pattern.PNG",
        "/final/Imported.PNG",
        "/document-base/Inline.PNG",
        "/document-base/Var.PNG",
    ];
    assert_eq!(server.paths(), prepared_paths);
    let first_pixels = expected_rows(&[
        "RRRRWWGGGGWW",
        "RRRRWWGGGGWW",
        "WWWWWWWWWWWW",
        "WWWWWWWWWWWW",
        "BBBBWWYYYYWW",
        "BBBBWWYYYYWW",
        "WWWWWWWWWWWW",
        "WWWWWWWWWWWW",
    ]);
    assert_eq!(pixels(&page, &first, &fonts, 12, 8), first_pixels);
    assert_eq!(hits(&first), baseline_hits);
    let again = page.prepare_layout(12.0, 8.0, &fonts);
    assert_eq!(pixels(&page, &again, &fonts, 12, 8), first_pixels);
    assert_eq!(hits(&again), baseline_hits);
    let cached = page.layout(12.0, 8.0, &fonts);
    assert_eq!(pixels(&page, &cached, &fonts, 12, 8), first_pixels);
    assert_eq!(server.paths(), prepared_paths, "cache reuse must not fetch");
    let base = page.document.query_selector("#b").unwrap();
    page.document.set_attr(base, "href", "/changed/");
    let changed_cached = page.layout(12.0, 8.0, &fonts);
    assert_eq!(
        pixels(&page, &changed_cached, &fonts, 12, 8),
        expected_rows(&[
            "RRRRWWGGGGWW",
            "RRRRWWGGGGWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW"
        ])
    );
    assert_eq!(
        server.paths(),
        prepared_paths,
        "base mutation/read-only layout must not fetch"
    );
    let changed = page.prepare_layout(12.0, 8.0, &fonts);
    assert_eq!(
        pixels(&page, &changed, &fonts, 12, 8),
        expected_rows(&[
            "RRRRWWGGGGWW",
            "RRRRWWGGGGWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW",
            "CCCCWWMMMMWW",
            "CCCCWWMMMMWW",
            "WWWWWWWWWWWW",
            "WWWWWWWWWWWW"
        ])
    );
    assert_eq!(hits(&changed), baseline_hits);
    assert_eq!(
        server.paths(),
        [
            "/document.html",
            "/entry.css",
            "/sheet/main.css",
            "/imports/theme.css",
            "/final/theme.css",
            "/sheet/Pattern.PNG",
            "/final/Imported.PNG",
            "/document-base/Inline.PNG",
            "/document-base/Var.PNG",
            "/changed/Inline.PNG",
            "/changed/Var.PNG"
        ]
    );
    assert_eq!(
        page.document
            .nodes
            .iter()
            .map(|node| (node.parent, node.children.clone()))
            .collect::<Vec<_>>(),
        graph
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn url_backgrounds_confined_worker_prepares_natural_sizing_before_snapshot() {
    use eris::{
        page::Navigation,
        worker::{Command, WorkerClient},
    };
    use std::path::Path;
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reftests/url-background-sizing.html");
    let navigation = Navigation::get(url::Url::from_file_path(path).unwrap().as_str());
    let mut client = WorkerClient::spawn_at(
        Path::new(env!("CARGO_BIN_EXE_eris-browser")),
        false,
        &navigation,
        263,
    )
    .unwrap();
    let loaded = client
        .exchange(Command::Load { navigation }, || false)
        .unwrap();
    assert!(loaded.snapshot.is_none() && loaded.navigation.is_none());
    let reply = client
        .exchange(
            Command::Render {
                width: 320.0,
                height: 240.0,
            },
            || false,
        )
        .unwrap();
    assert!(reply.navigation.is_none());
    let snapshot = reply.snapshot.unwrap();
    assert_eq!(snapshot.generation, 263);
    assert_eq!(snapshot.processed_edit_sequence, 0);
    assert!(
        snapshot
            .diagnostics
            .iter()
            .all(|message| message.starts_with("Page process ")
                || message.starts_with("Resource broker ")),
        "{:?}",
        snapshot.diagnostics
    );
    let mut expected = vec![0xffffff; 320 * 240];
    for (x, rows) in [
        (
            20,
            [
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
                "WWWWRGBCWWWW",
                "WWWWMYKOWWWW",
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
            ],
        ),
        (
            50,
            [
                "WWWWWWWWWWWW",
                "RRRGGGBBBCCC",
                "RRRGGGBBBCCC",
                "RRRGGGBBBCCC",
                "MMMYYYKKKOOO",
                "MMMYYYKKKOOO",
                "MMMYYYKKKOOO",
                "WWWWWWWWWWWW",
            ],
        ),
        (
            80,
            [
                "RRGGGGBBBBCC",
                "RRGGGGBBBBCC",
                "RRGGGGBBBBCC",
                "RRGGGGBBBBCC",
                "MMYYYYKKKKOO",
                "MMYYYYKKKKOO",
                "MMYYYYKKKKOO",
                "MMYYYYKKKKOO",
            ],
        ),
        (
            110,
            [
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
                "WWRRGGBBCCWW",
                "WWRRGGBBCCWW",
                "WWMMYYKKOOWW",
                "WWMMYYKKOOWW",
                "WWWWWWWWWWWW",
                "WWWWWWWWWWWW",
            ],
        ),
        (
            140,
            [
                "WWWWWWWWWWWW",
                "WWRRGGBBCCWW",
                "WWRRGGBBCCWW",
                "WWRRGGBBCCWW",
                "WWMMYYKKOOWW",
                "WWMMYYKKOOWW",
                "WWMMYYKKOOWW",
                "WWWWWWWWWWWW",
            ],
        ),
        (
            170,
            [
                "WWWWWWWWWWWW",
                "RRRGGGBBBCCC",
                "RRRGGGBBBCCC",
                "RRRGGGBBBCCC",
                "MMMYYYKKKOOO",
                "MMMYYYKKKOOO",
                "MMMYYYKKKOOO",
                "WWWWWWWWWWWW",
            ],
        ),
    ] {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, value) in row.bytes().enumerate() {
                expected[(20 + dy) * 320 + x + dx] = color(value);
            }
        }
    }
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.paint(
        &snapshot.layout.commands,
        &Fonts::new(),
        &snapshot.images,
        0.0,
        0.0,
    );
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
}
