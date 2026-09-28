//! Loopback tests: `cargo test --test network -- --include-ignored`.
use eris::{
    net::{Fetcher, ResourceKind},
    page::{Navigation, Page},
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
use url::Url;
#[derive(Clone, Debug)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
fn read_request(stream: &mut impl Read) -> std::io::Result<Request> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let boundary = loop {
        if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            break end + 4;
        }
        if bytes.len() > 64 * 1024 {
            return Err(std::io::Error::other("request header limit"));
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        bytes.extend_from_slice(&chunk[..n]);
    };
    let header = String::from_utf8(bytes[..boundary].to_vec()).map_err(std::io::Error::other)?;
    let mut lines = header.split("\r\n");
    let request_line = lines
        .next()
        .unwrap_or("")
        .split_ascii_whitespace()
        .collect::<Vec<_>>();
    if request_line.len() != 3 {
        return Err(std::io::Error::other("invalid request line"));
    }
    let headers: HashMap<String, String> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    if headers.contains_key("transfer-encoding") {
        return Err(std::io::Error::other("expected a known Content-Length"));
    }
    let length = headers
        .get("content-length")
        .map(|length| length.parse::<usize>())
        .transpose()
        .map_err(std::io::Error::other)?
        .unwrap_or(0);
    if length > eris::net::MAX_FORM_BODY_BYTES {
        return Err(std::io::Error::other("request body limit"));
    }
    while bytes.len() - boundary < length {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    Ok(Request {
        method: request_line[0].into(),
        path: request_line[1].into(),
        headers,
        body: bytes[boundary..boundary + length].to_vec(),
    })
}
struct Server {
    base: Url,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(routes: Vec<(&str, String)>) -> Self {
        Self::new_bytes(
            routes
                .into_iter()
                .map(|(path, response)| (path, response.into_bytes()))
                .collect(),
        )
    }
    fn new_bytes(routes: Vec<(&str, Vec<u8>)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
        let routes: HashMap<String, Vec<u8>> =
            routes.into_iter().map(|(k, v)| (k.into(), v)).collect();
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let worker = thread::spawn(move || {
            while !signal.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let request = match read_request(&mut stream) {
                            Ok(request) => request,
                            Err(_) => {
                                let _ = stream.write_all(
                                    response("400 Bad Request", "text/plain", "", "bad request")
                                        .as_bytes(),
                                );
                                continue;
                            }
                        };
                        let response = routes.get(&request.path).cloned().unwrap_or_else(|| {
                            response("404 Not Found", "text/plain", "", "missing").into_bytes()
                        });
                        recorded.lock().unwrap().push(request);
                        let _ = stream.write_all(&response);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            base,
            requests,
            stop,
            worker: Some(worker),
        }
    }
    fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
fn response(status: &str, mime: &str, headers: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
        body.len()
    )
}

fn byte_response(mime: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
    response.extend_from_slice(body);
    response
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn layered_imports_keep_network_order_shared_layers_and_unsupported_condition_policy() {
    use eris::{css, graphics::Color};
    let server = Server::new(vec![
        (
            "/",
            response(
                "200 OK",
                "text/html",
                "",
                r#"<!doctype html><style>
          @layer base, theme;
          @import '/theme.css' layer(th\65me);
          @import '/base.css' layer(base);
          @import '/must-not-fetch.css' layer(unused) supports(display:bogus);
          @layer theme { #x{background:blue} }
          </style><p id=x>sample</p>"#,
            ),
        ),
        (
            "/theme.css",
            response(
                "200 OK",
                "text/css",
                "",
                "@import '/tokens.css' layer(tokens);#x{color:blue;border-color:blue!important}",
            ),
        ),
        (
            "/tokens.css",
            response("200 OK", "text/css", "", "#x{background:red!important}"),
        ),
        (
            "/base.css",
            response(
                "200 OK",
                "text/css",
                "",
                "#x{color:red;border-color:red!important}",
            ),
        ),
    ]);
    let page = Page::load(server.base.as_str(), false).unwrap();
    assert_eq!(page.diagnostics.len(), 1, "{:?}", page.diagnostics);
    assert!(page.diagnostics[0].contains("supports conditions are unsupported"));
    let x = page.document.query_selector("#x").unwrap();
    let styles =
        css::compute_styles_from_sources(&page.document, &page.stylesheets(), 400.0, 300.0);
    assert_eq!(styles[x].color, Color::rgb(0, 0, 255));
    assert_eq!(styles[x].border_color, Color::rgb(255, 0, 0));
    assert_eq!(styles[x].background_color, Color::rgb(255, 0, 0));
    let paths = server
        .requests()
        .into_iter()
        .map(|r| r.path)
        .collect::<Vec<_>>();
    assert_eq!(paths, ["/", "/theme.css", "/tokens.css", "/base.css"]);
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn base_urls_and_recursive_imports_preserve_redirect_bases_encoding_order_and_media() {
    use eris::{css, graphics::Color};
    let (child, _, errors) =
        encoding_rs::SHIFT_JIS.encode("@import 'parent.css'; .日本{color:blue;background:red}");
    assert!(!errors);
    let (parent, _, errors) = encoding_rs::SHIFT_JIS
        .encode("@import url('child.css?name=日本'); #x{color:green} @import 'late.css';");
    assert!(!errors);
    let s = Server::new_bytes(vec![
        (
            "/docs/page",
            byte_response(
                "text/html; charset=utf-8",
                br#"<!doctype html>
            <base href='/assets/'><base href='/ignored/'>
            <link rel=stylesheet href='entry.css'>
            <style>@import 'inline.css' screen and (min-width: 400px);</style>
            <div id=x class='&#26085;&#26412;'>test</div><a id=link href='#section'>link</a>
            <script src='code.js'></script>"#,
            ),
        ),
        (
            "/assets/entry.css",
            response(
                "302 Found",
                "text/css",
                "Location: /styles/parent.css\r\n",
                "",
            )
            .into_bytes(),
        ),
        (
            "/styles/parent.css",
            byte_response("text/css; charset=shift_jis", &parent),
        ),
        (
            "/styles/child.css?name=%E6%97%A5%E6%9C%AC",
            byte_response("text/css", &child),
        ),
        (
            "/assets/inline.css",
            byte_response("text/css", b"#x{background:blue}"),
        ),
        (
            "/assets/code.js",
            byte_response(
                "text/javascript",
                b"document.getElementById('x').setAttribute('loaded','yes');",
            ),
        ),
    ]);
    let p = Page::load(s.base.join("docs/page").unwrap().as_str(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let x = p.document.query_selector("#x").unwrap();
    assert_eq!(p.document.attr(x, "loaded"), Some("yes"));
    let narrow = css::compute_styles_from_sources(&p.document, &p.stylesheets(), 300.0, 200.0);
    let wide = css::compute_styles_from_sources(&p.document, &p.stylesheets(), 600.0, 200.0);
    assert_eq!(narrow[x].color, Color::rgb(0, 128, 0));
    assert_eq!(narrow[x].background_color, Color::rgb(255, 0, 0));
    assert_eq!(wide[x].background_color, Color::rgb(0, 0, 255));
    assert_eq!(
        p.resolve_navigation("#section").unwrap(),
        s.base.join("assets/#section").unwrap().as_str()
    );
    assert_eq!(p.document.url(), &p.url);
    let paths: Vec<_> = s.requests().into_iter().map(|r| r.path).collect();
    assert_eq!(
        paths,
        [
            "/docs/page",
            "/assets/entry.css",
            "/styles/parent.css",
            "/styles/child.css?name=%E6%97%A5%E6%9C%AC",
            "/assets/inline.css",
            "/assets/code.js"
        ]
    );
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn base_and_import_urls_cannot_broaden_document_fetch_authority() {
    let other = Server::new(vec![]);
    let html = format!(
        "<!doctype html><base href='{}'><link rel=stylesheet href='secret.css'><script src='secret.js'></script>",
        other.base
    );
    let s = Server::new(vec![
        ("/base", response("200 OK", "text/html", "", &html)),
        (
            "/import",
            response(
                "200 OK",
                "text/html",
                "",
                &format!(
                    "<!doctype html><style>@import '{}secret.css'; @import 'never.css' supports(display:grid); p{{color:green}}</style><style type='text/plain'>@import 'inert.css';</style><p>x",
                    other.base
                ),
            ),
        ),
    ]);
    let base = Page::load(s.base.join("base").unwrap().as_str(), true).unwrap();
    assert_eq!(base.diagnostics.len(), 2);
    let imported = Page::load(s.base.join("import").unwrap().as_str(), false).unwrap();
    assert_eq!(imported.diagnostics.len(), 2, "{:?}", imported.diagnostics);
    assert!(other.requests().is_empty());
    assert_eq!(s.requests().len(), 2);
    for base in ["file:///etc/", "eris:home"] {
        let p = Page::from_html(s.base.clone(), &format!("<base href='{base}'>"), false);
        assert!(p.resolve_navigation("#fragment").is_err());
    }
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn late_encoding_reparse_does_not_repeat_post_and_subresources_inherit_encoding() {
    use eris::graphics::{Canvas, Fonts};
    let source = format!(
        "<!doctype html><head><!--{}--><meta charset=shift_jis><link rel=stylesheet href=/style></head><body><div id=x class=日本>初</div><script src=/code></script>",
        " ".repeat(1100)
    );
    let (html, _, errors) = encoding_rs::SHIFT_JIS.encode(&source);
    assert!(!errors);
    let (style, _, errors) = encoding_rs::SHIFT_JIS
        .encode("body{margin:0}.日本{width:80px;height:40px;background:blue}");
    assert!(!errors);
    let (script, _, errors) =
        encoding_rs::SHIFT_JIS.encode("document.getElementById('x').textContent='日本';");
    assert!(!errors);
    let s = Server::new_bytes(vec![
        ("/post", byte_response("text/html", &html)),
        ("/style", byte_response("text/css", &style)),
        ("/code", byte_response("text/javascript", &script)),
    ]);
    let p = Page::load_navigation(
        &Navigation {
            address: s.base.join("post").unwrap().to_string(),
            form_body: Some("word=one".into()),
        },
        true,
    )
    .unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.document.character_set(), "Shift_JIS");
    let x = p.document.query_selector("#x").unwrap();
    assert_eq!(p.document.text_content(x), "日本");
    assert_eq!(p.document.attr(x, "class"), Some("日本"));
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(150, 100).unwrap();
    canvas.paint(
        &p.layout(150.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(canvas.pixels[30 * 150 + 60], 0x0000ff);
    let requests = s.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].body, b"word=one");
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.path == "/post")
            .count(),
        1
    );
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn script_charset_cache_and_stylesheet_charset_choose_their_own_encodings() {
    let html = "<!doctype html><meta charset=utf-8><link rel=stylesheet href=/style><div id=x class=café></div><script>var decoded='';</script><script src=/code charset=windows-1252></script><script src=/code charset=utf-8></script><script>document.getElementById('x').textContent=decoded;</script>";
    let s = Server::new_bytes(vec![
        ("/page", byte_response("text/html", html.as_bytes())),
        (
            "/style",
            byte_response(
                "text/css",
                b"@charset \"windows-1252\";.caf\xe9{background:red}",
            ),
        ),
        (
            "/code",
            byte_response("text/javascript", b"decoded += '\xc3\xa9';"),
        ),
    ]);
    let p = Page::load(s.base.join("page").unwrap().as_str(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "Ã©é"
    );
    assert!(
        p.stylesheets()
            .iter()
            .any(|sheet| sheet.source.contains(".café{background:red}"))
    );
    let fonts = eris::graphics::Fonts::new();
    let layout = p.layout(200.0, 100.0, &fonts);
    let mut canvas = eris::graphics::Canvas::new(200, 100).unwrap();
    canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
    assert!(
        canvas.pixels.contains(&0xff0000),
        "decoded CSS selector must reach painting"
    );
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn redirects_revalidate_file_policy_and_cycles_stop() {
    let s = Server::new(vec![
        (
            "/file",
            response(
                "302 Found",
                "text/html",
                "Location: file:///etc/passwd\r\n",
                "",
            ),
        ),
        (
            "/loop",
            response("302 Found", "text/html", "Location: /loop\r\n", ""),
        ),
        (
            "/next",
            response("302 Found", "text/html", "Location: /done\r\n", ""),
        ),
        (
            "/done",
            response("200 OK", "text/html", "", "<h1>Done</h1>"),
        ),
    ]);
    let mut f = Fetcher::default();
    assert!(
        f.fetch(&s.base.join("file").unwrap(), None, ResourceKind::Document)
            .is_err()
    );
    assert!(
        f.fetch(&s.base.join("loop").unwrap(), None, ResourceKind::Document)
            .is_err()
    );
    let r = f
        .fetch(&s.base.join("next").unwrap(), None, ResourceKind::Document)
        .unwrap();
    assert_eq!(r.url.path(), "/done");
    assert_eq!(r.text(), "<h1>Done</h1>");
}
#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn csp_and_script_mime_fail_closed() {
    let s = Server::new(vec![
        (
            "/csp",
            response(
                "200 OK",
                "text/html",
                "Content-Security-Policy: script-src 'none'\r\n",
                "<p id=x>original</p><script>document.getElementById('x').textContent='unsafe';</script>",
            ),
        ),
        (
            "/mime",
            response(
                "200 OK",
                "text/html",
                "",
                "<p id=x>original</p><script src=/wrong.js></script>",
            ),
        ),
        (
            "/wrong.js",
            response(
                "200 OK",
                "text/plain",
                "",
                "document.getElementById('x').textContent='unsafe';",
            ),
        ),
    ]);
    for path in ["csp", "mime"] {
        let p = Page::load(s.base.join(path).unwrap().as_str(), true).unwrap();
        assert_eq!(
            p.document
                .text_content(p.document.query_selector("#x").unwrap()),
            "original"
        );
        assert!(!p.diagnostics.is_empty());
    }
}
#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn external_css_and_scripts_are_loaded_over_http() {
    let s = Server::new(vec![
        (
            "/",
            response(
                "200 OK",
                "text/html",
                "",
                "<link rel=stylesheet href=/a.css><p id=x>original</p><script src=/a.js></script>",
            ),
        ),
        (
            "/a.css",
            response("200 OK", "text/css", "", "p{color:rebeccapurple}"),
        ),
        (
            "/a.js",
            response(
                "200 OK",
                "text/javascript",
                "",
                "document.getElementById('x').textContent='loaded';",
            ),
        ),
    ]);
    let p = Page::load(s.base.as_str(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "loaded"
    );
    assert_eq!(
        p.stylesheets()
            .iter()
            .map(|s| s.source.as_ref())
            .collect::<Vec<_>>(),
        vec!["p{color:rebeccapurple}"]
    );
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn failed_status_scripts_never_execute_and_svg_mime_is_case_insensitive() {
    let s = Server::new(vec![
        (
            "/",
            response(
                "200 OK",
                "text/html",
                "",
                "<style>body{margin:0}</style><p id=x>original</p><script src=/missing.js></script><img src=/image.svg>",
            ),
        ),
        (
            "/missing.js",
            response(
                "404 Not Found",
                "text/javascript",
                "",
                "document.getElementById('x').textContent='unsafe';",
            ),
        ),
        (
            "/image.svg",
            response(
                "200 OK",
                "Image/Svg+Xml; charset=utf-8",
                "",
                "<svg width='8' height='8'><rect width='8' height='8' fill='#123456'/></svg>",
            ),
        ),
    ]);
    let p = Page::load(s.base.as_str(), true).unwrap();
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "original"
    );
    assert!(p.diagnostics.iter().any(|d| d.contains("HTTP 404")));
    let image = p.images.get("/image.svg").unwrap();
    assert_eq!((image.width, image.height), (8, 8));
    let center = (4 * 8 + 4) * 4;
    assert_eq!(&image.rgba[center..center + 4], &[0x12, 0x34, 0x56, 255]);
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn script_redirect_cannot_escape_initiating_origin() {
    let other = Server::new(vec![(
        "/foreign.js",
        response(
            "200 OK",
            "text/javascript",
            "",
            "document.getElementById('x').textContent='unsafe';",
        ),
    )]);
    let s = Server::new(vec![
        (
            "/",
            response(
                "200 OK",
                "text/html",
                "",
                "<p id=x>original</p><script src=/redirect.js></script>",
            ),
        ),
        (
            "/redirect.js",
            response(
                "302 Found",
                "text/javascript",
                &format!("Location: {}\r\n", other.base.join("foreign.js").unwrap()),
                "",
            ),
        ),
    ]);
    let p = Page::load(s.base.as_str(), true).unwrap();
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "original"
    );
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.contains("cross-origin active subresource blocked"))
    );
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn post_form_sends_encoded_body_content_type_and_preserves_action_query() {
    let s = Server::new(vec![(
        "/submit?existing=1",
        response(
            "200 OK",
            "text/html",
            "Set-Cookie: ignored=1; Path=/\r\n",
            "<p id=result>posted</p>",
        ),
    )]);
    let mut source = Page::from_html(
        s.base.clone(),
        "<form method=post action='/submit?existing=1'><input name=q value='hello world &amp; α'><input type=checkbox name=flag checked><input name=disabled disabled value=secret><button name=send value=yes>Send</button></form>",
        false,
    );
    let navigation = source
        .click(source.document.query_selector("button").unwrap())
        .unwrap();
    assert_eq!(
        navigation.form_body.as_deref(),
        Some("q=hello+world+%26+%CE%B1&flag=on&send=yes")
    );
    assert_eq!(
        Url::parse(&navigation.address).unwrap().query(),
        Some("existing=1")
    );
    let loaded = Page::load_navigation(&navigation, false).unwrap();
    assert_eq!(
        loaded
            .document
            .text_content(loaded.document.query_selector("#result").unwrap()),
        "posted"
    );
    let request = &s.requests()[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/submit?existing=1");
    assert_eq!(
        request.headers.get("content-type").map(String::as_str),
        Some("application/x-www-form-urlencoded")
    );
    assert_eq!(
        request.body,
        navigation.form_body.as_ref().unwrap().as_bytes()
    );
    assert_eq!(
        request
            .headers
            .get("content-length")
            .unwrap()
            .parse::<usize>()
            .unwrap(),
        request.body.len()
    );
    assert!(!request.headers.contains_key("cookie"));
    assert!(!request.headers.contains_key("authorization"));
    // The GET wrapper has no saved body or cookie state to replay on reload/history.
    Page::load(&navigation.address, false).unwrap();
    let reloaded = &s.requests()[1];
    assert_eq!(reloaded.method, "GET");
    assert!(reloaded.body.is_empty());
    assert!(!reloaded.headers.contains_key("cookie"));
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn post_redirects_rewrite_301_302_303_and_preserve_307_308() {
    for status in [301, 302, 303, 307, 308] {
        let s = Server::new(vec![
            (
                "/start",
                response(
                    &format!("{status} Redirect"),
                    "text/html",
                    "Location: /finished?keep=1#anchor\r\n",
                    "",
                ),
            ),
            (
                "/finished?keep=1",
                response("200 OK", "text/html", "", "<p>done</p>"),
            ),
        ]);
        let navigation = Navigation {
            address: s.base.join("start").unwrap().to_string(),
            form_body: Some("name=one+two&encoded=%CE%B1".into()),
        };
        let page = Page::load_navigation(&navigation, false).unwrap();
        assert_eq!(page.url.path(), "/finished");
        assert_eq!(page.url.fragment(), Some("anchor"));
        let requests = s.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(
            requests[0].body,
            navigation.form_body.as_ref().unwrap().as_bytes()
        );
        let preserved = matches!(status, 307 | 308);
        assert_eq!(requests[1].method, if preserved { "POST" } else { "GET" });
        if preserved {
            assert_eq!(requests[1].body, requests[0].body);
            assert_eq!(
                requests[1].headers.get("content-type"),
                requests[0].headers.get("content-type")
            );
        } else {
            assert!(requests[1].body.is_empty());
            assert!(!requests[1].headers.contains_key("content-type"));
        }
    }
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn post_rejects_non_http_redirects_and_oversized_bodies_before_sending() {
    let s = Server::new(vec![(
        "/escape",
        response(
            "307 Temporary Redirect",
            "text/html",
            "Location: data:text/html,unsafe\r\n",
            "",
        ),
    )]);
    let mut fetcher = Fetcher::default();
    let url = s.base.join("escape").unwrap();
    assert!(fetcher.fetch_document(&url, Some("private=value")).is_err());
    assert_eq!(s.requests().len(), 1);
    assert!(
        fetcher
            .fetch_document(&url, Some(&"a".repeat(eris::net::MAX_FORM_BODY_BYTES + 1)))
            .is_err()
    );
    assert_eq!(s.requests().len(), 1);
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn http_text_is_decoded_once_and_bom_precedes_charset_header() {
    fn encoded_response(body: &[u8]) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=windows-1252\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
        response.extend_from_slice(body);
        response
    }
    let s = Server::new_bytes(vec![
        ("/latin", encoded_response(b"caf\xe9")),
        ("/bom", encoded_response(b"\xef\xbb\xbfcaf\xc3\xa9")),
    ]);
    for path in ["latin", "bom"] {
        let mut fetcher = Fetcher::default();
        let resource = fetcher
            .fetch_document(&s.base.join(path).unwrap(), None)
            .unwrap();
        assert_eq!(resource.text(), "café", "{path}");
    }
}

#[test]
#[ignore = "requires permission to bind a loopback test server"]
fn encoded_gzip_header_is_bounded_before_it_produces_decoded_bytes() {
    // A valid gzip FNAME field can be arbitrarily long while producing no body output.
    // Chunked transfer omits Content-Length, so the raw-stream limit must stop it.
    let mut gzip = vec![0x1f, 0x8b, 8, 8, 0, 0, 0, 0, 0, 255];
    gzip.extend(std::iter::repeat_n(
        b'a',
        eris::net::MAX_RESOURCE_BYTES + 64,
    ));
    gzip.push(0);
    gzip.extend_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let mut encoded = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Encoding: gzip\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
    encoded.extend_from_slice(format!("{:x}\r\n", gzip.len()).as_bytes());
    encoded.extend_from_slice(&gzip);
    encoded.extend_from_slice(b"\r\n0\r\n\r\n");
    let s = Server::new_bytes(vec![("/gzip", encoded)]);
    let mut fetcher = Fetcher::default();
    assert!(
        fetcher
            .fetch_document(&s.base.join("gzip").unwrap(), None)
            .is_err()
    );
}

#[test]
#[ignore = "requires permission to bind two loopback test servers"]
fn image_origin_taint_survives_redirects_and_cached_aliases() {
    let svg = "<svg width='2' height='2'><rect width='2' height='2' fill='red'/></svg>";
    let home = Server::new(vec![(
        "/image",
        response("200 OK", "image/svg+xml", "", svg),
    )]);
    let other = Server::new(vec![(
        "/back",
        response(
            "302 Found",
            "text/plain",
            &format!("Location: {}\r\n", home.base.join("image").unwrap()),
            "",
        ),
    )]);
    let mut f = Fetcher::default();
    let clean = f
        .fetch(
            &home.base.join("image").unwrap(),
            Some(&home.base),
            ResourceKind::Image,
        )
        .unwrap();
    assert!(clean.origin_clean);
    let tainted = f
        .fetch(
            &other.base.join("back").unwrap(),
            Some(&home.base),
            ResourceKind::Image,
        )
        .unwrap();
    assert_eq!(tainted.url, clean.url);
    assert!(
        !tainted.origin_clean,
        "a final same-origin URL cannot remove earlier redirect taint"
    );
    let document = Server::new(vec![(
        "/page",
        response(
            "200 OK",
            "text/html",
            "",
            &format!(
                "<img src='{0}'><img src='{0}'><img src='{0}#alias'>",
                other.base.join("back").unwrap()
            ),
        ),
    )]);
    let page = Page::load(document.base.join("page").unwrap().as_str(), false).unwrap();
    let key = other.base.join("back").unwrap().to_string();
    assert_eq!(page.image_origin_clean(&key), Some(false));
    assert_eq!(
        page.image_origin_clean(&format!("{key}#alias")),
        Some(false)
    );
    assert!(Arc::ptr_eq(
        page.images.get(&key).unwrap(),
        page.images.get(&format!("{key}#alias")).unwrap()
    ));
    assert_eq!(
        other.requests().len(),
        2,
        "page aliases share a single image fetch"
    );
}
