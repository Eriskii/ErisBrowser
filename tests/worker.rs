//! Real-binary IPC tests: `cargo test --test worker -- --include-ignored`.
//! Confined worker tests require Linux with Landlock ABI 6; framing rejection
//! happens before sandbox initialization and is independently testable.
#![cfg(target_os = "linux")]

use eris::{
    page::Navigation,
    worker::{Command as WorkerCommand, Snapshot, WorkerClient},
};
use std::{
    cell::Cell,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
use url::Url;

const BINARY: &str = env!("CARGO_BIN_EXE_eris-browser");
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    navigation: Navigation,
}
impl Fixture {
    fn new(source: &str) -> Self {
        let directory = loop {
            let candidate = std::env::temp_dir().join(format!(
                "eris-worker-integration-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture directory: {error}"),
            }
        };
        let path = directory.join("index.html");
        fs::write(&path, source).expect("write fixture");
        Self {
            directory,
            navigation: Navigation::get(Url::from_file_path(path).unwrap().to_string()),
        }
    }
    fn spawn(&self, scripts: bool, generation: u64) -> WorkerClient {
        WorkerClient::spawn_at(Path::new(BINARY), scripts, &self.navigation, generation)
            .expect("real page worker requires fully enforced Linux Landlock ABI 6")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn load(client: &mut WorkerClient, navigation: &Navigation) {
    let reply = client
        .exchange(
            WorkerCommand::Load {
                navigation: navigation.clone(),
            },
            || false,
        )
        .expect("load through IPC");
    assert!(reply.snapshot.is_none());
    assert!(reply.navigation.is_none());
}
fn render(client: &mut WorkerClient) -> Snapshot {
    let reply = client
        .exchange(
            WorkerCommand::Render {
                width: 320.0,
                height: 240.0,
            },
            || false,
        )
        .expect("render through IPC");
    assert!(reply.navigation.is_none());
    reply.snapshot.expect("render returns a snapshot")
}
fn exchange_empty(client: &mut WorkerClient, command: WorkerCommand) {
    let reply = client.exchange(command, || false).expect("IPC command");
    assert!(reply.snapshot.is_none());
    assert!(reply.navigation.is_none());
}
fn process_exists(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn isolated_load_render_returns_valid_snapshot_from_distinct_process() {
    let fixture = Fixture::new(
        "<!doctype html><title>Worker fixture</title><h1 id=heading>Hello worker</h1><svg width=12 height=12><rect width=12 height=12 fill=red /></svg>",
    );
    let mut client = fixture.spawn(false, 42);
    let pid = client.pid();
    assert_ne!(pid, std::process::id());
    assert!(process_exists(pid));
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    assert_eq!(snapshot.generation, 42);
    assert_eq!(snapshot.processed_edit_sequence, 0);
    assert_eq!(snapshot.title, "Worker fixture");
    assert_eq!(snapshot.url, fixture.navigation.address);
    assert_eq!(
        snapshot
            .document
            .text_content(snapshot.document.query_selector("#heading").unwrap()),
        "Hello worker"
    );
    assert!(!snapshot.layout.commands.is_empty());
    assert!(snapshot.layout.content_height.is_finite());
    assert!(!snapshot.images.is_empty());
    let broker_pid = client
        .broker_pid()
        .expect("local document uses the resource broker");
    assert_ne!(broker_pid, pid);
    assert!(process_exists(broker_pid));
    assert!(snapshot.diagnostics.iter().any(|message| {
        message.starts_with(&format!("Page process {pid}:")) && message.contains("Landlock ABI 6")
    }));
    drop(client);
    assert!(!process_exists(pid), "Drop must reap the worker process");
    assert!(
        !process_exists(broker_pid),
        "Drop must reap the resource broker"
    );
}

fn accept_request(listener: &TcpListener, expected_path: &str) -> std::net::TcpStream {
    accept_form_request(listener, "GET", expected_path, None)
}
fn accept_form_request(
    listener: &TcpListener,
    method: &str,
    expected_path: &str,
    expected_body: Option<&str>,
) -> std::net::TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "fixture received no request for {expected_path}"
                );
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept fixture request: {error}"),
        }
    };
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut request = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !request.ends_with(b"\r\n\r\n") {
        assert!(request.len() < 16_384);
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "fixture header deadline");
        stream.set_read_timeout(Some(remaining)).unwrap();
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        request.push(byte[0]);
    }
    let headers = String::from_utf8(request).unwrap();
    assert!(headers.starts_with(&format!("{method} {expected_path} HTTP/1.1\r\n")));
    if let Some(body) = expected_body {
        let lower = headers.to_ascii_lowercase();
        assert!(lower.contains("content-type: application/x-www-form-urlencoded\r\n"));
        assert!(lower.contains(&format!("content-length: {}\r\n", body.len())));
        let mut received = vec![0; body.len()];
        stream.read_exact(&mut received).unwrap();
        assert_eq!(received, body.as_bytes());
    }
    stream
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and a bounded HTTP form fixture"]
fn broker_submits_only_the_authorized_form_body_and_commits_redirect() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let body = "q=hello+world&symbol=%F0%9F%A6%80";
    let navigation = Navigation {
        address: format!("{base}/submit?fixed=1"),
        form_body: Some(body.into()),
    };
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), true, &navigation, 107).unwrap();
    let mut wrong = navigation.clone();
    wrong.form_body = Some("q=unauthorized".into());
    assert!(
        client
            .exchange(WorkerCommand::Load { navigation: wrong }, || false)
            .err()
            .expect("unauthorized form body must fail")
            .contains("authorized")
    );
    assert!(client.broker_pid().is_none());
    let server = thread::spawn(move || {
        let mut stream = accept_form_request(&listener, "POST", "/submit?fixed=1", Some(body));
        write!(stream, "HTTP/1.1 303 See Other\r\nLocation: /complete\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&listener, "/complete");
        let html = "<!doctype html><title>Authorized form</title><p>Received</p>";
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}", html.len()).unwrap();
    });
    load(&mut client, &navigation);
    let snapshot = render(&mut client);
    assert_eq!(snapshot.title, "Authorized form");
    assert_eq!(snapshot.url, format!("{base}/complete"));
    server.join().unwrap();
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and two bounded loopback HTTP origins"]
fn broker_commits_cross_origin_redirect_and_enforces_the_final_origin() {
    let initial = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = TcpListener::bind("127.0.0.1:0").unwrap();
    initial.set_nonblocking(true).unwrap();
    destination.set_nonblocking(true).unwrap();
    let initial_address = initial.local_addr().unwrap();
    let final_address = destination.local_addr().unwrap();
    let navigation = Navigation::get(format!("http://{initial_address}/start"));
    let final_url = format!("http://{final_address}/final/page");
    let redirect_url = final_url.clone();
    let server = thread::spawn(move || {
        let mut stream = accept_request(&initial, "/start");
        write!(stream, "HTTP/1.1 302 Found\r\nLocation: {redirect_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/final/page");
        let body = format!(
            "<!doctype html><title>Broker redirect</title><p id=result>initial</p><script src=after.js></script><script src='http://{initial_address}/forbidden.js'></script><img src='/redirect-image'><img src='http://{initial_address}/direct-image'>"
        );
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/final/after.js");
        let body = "document.getElementById('result').textContent='final origin script';";
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/redirect-image");
        write!(stream, "HTTP/1.1 302 Found\r\nLocation: http://{initial_address}/private.json\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        initial
    });
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), true, &navigation, 105).unwrap();
    load(&mut client, &navigation);
    let snapshot = render(&mut client);
    let initial = server.join().unwrap();
    assert_eq!(snapshot.url, final_url);
    assert_eq!(snapshot.title, "Broker redirect");
    assert_eq!(
        snapshot
            .document
            .text_content(snapshot.document.query_selector("#result").unwrap()),
        "final origin script"
    );
    assert!(
        snapshot
            .diagnostics
            .iter()
            .any(|d| d.contains("cross-origin")),
        "{:?}",
        snapshot.diagnostics
    );
    assert_eq!(
        initial.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "blocked scripts/images/redirects must not reach the previous origin"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and a bounded stalled HTTP fixture"]
fn cancellation_during_broker_io_reaps_both_processes() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let navigation = Navigation::get(format!("http://{}/stall", listener.local_addr().unwrap()));
    let (accepted_tx, accepted_rx) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let mut stream = accept_request(&listener, "/stall");
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        accepted_tx.send(()).unwrap();
        let mut byte = [0];
        assert_eq!(
            stream.read(&mut byte).unwrap(),
            0,
            "broker cancellation closes its HTTP socket"
        );
    });
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 106).unwrap();
    let renderer_pid = client.pid();
    let broker_pid = Cell::new(None);
    let started = Instant::now();
    let result = client.exchange(WorkerCommand::Load { navigation }, || {
        // Children belong to the task which spawned them, so this remains
        // deterministic even while other integration tests run in parallel.
        for pid in fs::read_to_string("/proc/thread-self/children")
            .unwrap()
            .split_whitespace()
        {
            if let Ok(arguments) = fs::read(format!("/proc/{pid}/cmdline"))
                && arguments
                    .split(|&b| b == 0)
                    .any(|arg| arg == b"--resource-broker")
            {
                broker_pid.set(Some(pid.parse::<u32>().unwrap()));
            }
        }
        // Cancellation is intentionally a one-shot notification. Observing it
        // inside a broker exchange must also cancel the outer renderer request.
        accepted_rx.try_recv().is_ok() || started.elapsed() > Duration::from_secs(4)
    });
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(3));
    let broker_pid = broker_pid
        .get()
        .expect("the request reached a resource broker");
    assert!(!process_exists(renderer_pid));
    assert!(!process_exists(broker_pid));
    assert!(client.broker_pid().is_none());
    server.join().unwrap();
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and the confined broker's permitted loopback TCP client"]
fn broker_http_load_resolves_and_fetches_without_spawning_threads() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let navigation = Navigation::get(format!("http://{address}/fixture"));
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 104)
        .expect("confined HTTP worker");
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "worker did not connect to HTTP fixture"
                    );
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("HTTP fixture accept failed: {error}"),
            }
        };
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let read_deadline = Instant::now() + Duration::from_secs(2);
        while !request.ends_with(b"\r\n\r\n") {
            assert!(request.len() < 16_384, "oversized fixture request");
            let remaining = read_deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "HTTP fixture request timed out");
            stream.set_read_timeout(Some(remaining)).unwrap();
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        let request = String::from_utf8(request).unwrap();
        assert!(
            request.starts_with("GET /fixture HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains(&format!("host: {address}\r\n"))
        );
        let body = "<!doctype html><title>Confined HTTP</title><h1>TCP client works</h1>";
        write!(stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ).unwrap();
    });
    load(&mut client, &navigation);
    let snapshot = render(&mut client);
    server.join().expect("bounded HTTP fixture server");
    assert_eq!(snapshot.title, "Confined HTTP");
    assert_eq!(snapshot.url, navigation.address);
    assert_eq!(
        snapshot
            .document
            .text_content(snapshot.document.query_selector("h1").unwrap()),
        "TCP client works"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn edits_events_clicks_forms_and_fragments_round_trip_without_reloading() {
    let fixture = Fixture::new(
        r##"<!doctype html><title>Interactive worker</title>
        <form id=form action="https://example.invalid/submit" method=post>
        <input id=field name=q value=initial>
        <textarea id=notes name=notes>old</textarea>
        <input id=locked readonly value=locked>
        <button id=submit name=submit value=send>Send</button></form>
        <button id=click type=button onclick="document.getElementById('status').textContent='clicked'">Click</button>
        <a id=jump href="#target">Jump</a><h2 id=target>Target</h2><p id=status>ready</p>
        <script>document.getElementById('field').addEventListener('input', function(event) {
          document.getElementById('status').textContent = event.target.value;
        });</script>"##,
    );
    let mut client = fixture.spawn(true, 99);
    load(&mut client, &fixture.navigation);
    let first = render(&mut client);
    let node = |selector| first.document.query_selector(selector).unwrap();
    let field = node("#field");
    let status = node("#status");
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 2,
            node: field,
            value: "edited 🦀".into(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 1,
            node: field,
            value: "stale edit".into(),
        },
    );
    let edited = render(&mut client);
    assert_eq!(edited.processed_edit_sequence, 2);
    assert_eq!(edited.document.attr(field, "value"), Some("edited 🦀"));
    assert_eq!(edited.document.text_content(status), "edited 🦀");
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 3,
            node: node("#locked"),
            value: "blocked".into(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 4,
            node: node("#notes"),
            value: "line1\nline2".into(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Click {
            node: node("#click"),
        },
    );
    let clicked = render(&mut client);
    assert_eq!(clicked.document.text_content(status), "clicked");
    assert_eq!(
        clicked.document.attr(node("#locked"), "value"),
        Some("locked")
    );
    assert_eq!(clicked.processed_edit_sequence, 4);
    let navigation = client
        .exchange(
            WorkerCommand::Click {
                node: node("#submit"),
            },
            || false,
        )
        .unwrap()
        .navigation
        .expect("form returns navigation without sending a public request");
    assert_eq!(navigation.address, "https://example.invalid/submit");
    let controls: Vec<_> =
        url::form_urlencoded::parse(navigation.form_body.as_ref().unwrap().as_bytes())
            .into_owned()
            .collect();
    assert_eq!(
        controls,
        [
            ("q".into(), "edited 🦀".into()),
            ("notes".into(), "line1\r\nline2".into()),
            ("submit".into(), "send".into())
        ]
    );
    let fragment = client
        .exchange(
            WorkerCommand::Click {
                node: node("#jump"),
            },
            || false,
        )
        .unwrap()
        .navigation
        .unwrap();
    assert_eq!(
        fragment.address,
        format!("{}#target", fixture.navigation.address)
    );
    assert!(fragment.form_body.is_none());
    exchange_empty(
        &mut client,
        WorkerCommand::Fragment {
            address: fragment.address.clone(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Fragment {
            address: "https://example.invalid/other".into(),
        },
    );
    let final_snapshot = render(&mut client);
    assert_eq!(final_snapshot.url, fragment.address);
    assert_eq!(final_snapshot.document.text_content(status), "clicked");
    assert_eq!(final_snapshot.generation, 99);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn cancellation_reaps_child_and_makes_old_client_unavailable() {
    let navigation = Navigation::get("about:blank");
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 7).unwrap();
    let pid = client.pid();
    assert!(process_exists(pid));
    let started = Instant::now();
    let cancellation_checks = Cell::new(0);
    let error = client
        .exchange(WorkerCommand::Load { navigation }, || {
            let checks = cancellation_checks.get() + 1;
            cancellation_checks.set(checks);
            // Let the first transaction iteration send the request, then cancel
            // while the reply is still pending.
            checks >= 2
        })
        .err()
        .expect("cancelled exchange must fail");
    assert!(error.contains("cancelled"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!process_exists(pid), "cancellation must reap the worker");
    let error = client
        .exchange(
            WorkerCommand::Render {
                width: 320.0,
                height: 240.0,
            },
            || false,
        )
        .err()
        .expect("cancelled clients cannot be reused");
    assert!(error.contains("unavailable"), "{error}");
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn dropping_an_idle_client_leaves_no_child_process() {
    let navigation = Navigation::get("about:blank");
    let client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 8).unwrap();
    let pid = client.pid();
    assert!(process_exists(pid));
    drop(client);
    assert!(
        !process_exists(pid),
        "Drop must kill and reap an idle worker"
    );
}

#[test]
#[ignore = "launches a single-test subprocess and the real worker; Linux procfs required"]
fn unexpected_inherited_descriptor_is_rejected() {
    const HELPER: &str = "ERIS_WORKER_INHERITED_FD_HELPER";
    if std::env::var_os(HELPER).is_some() {
        // The dedicated, single-test subprocess avoids racing parallel tests
        // while the otherwise close-on-exec file temporarily permits inheritance.
        let file = fs::File::open("/dev/null").unwrap();
        let original = rustix::io::fcntl_getfd(&file).unwrap();
        rustix::io::fcntl_setfd(&file, rustix::io::FdFlags::empty()).unwrap();
        let result = WorkerClient::spawn_at(
            Path::new(BINARY),
            false,
            &Navigation::get("about:blank"),
            103,
        );
        rustix::io::fcntl_setfd(&file, original).unwrap();
        let error = result
            .err()
            .expect("unexpected inherited descriptors must fail closed");
        assert!(error.contains("unexpected open descriptor"), "{error}");
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "unexpected_inherited_descriptor_is_rejected",
            "--include-ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env(HELPER, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn malformed_initial_frames_and_eof_exit_without_waiting_for_payload() {
    for (input, keep_input_open) in [
        (u32::MAX.to_le_bytes().to_vec(), true),
        ([5u32.to_le_bytes().as_slice(), b"ERW"].concat(), false),
        (Vec::new(), false),
    ] {
        let mut child = Command::new(BINARY)
            .arg("--page-worker")
            .env_clear()
            .current_dir("/")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&input).unwrap();
        stdin.flush().unwrap();
        let _open_input = if keep_input_open {
            Some(stdin)
        } else {
            drop(stdin);
            None
        };
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > Duration::from_secs(2) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("malformed initial IPC did not exit promptly");
            }
            thread::sleep(Duration::from_millis(5));
        };
        assert!(!status.success(), "invalid frame must fail closed");
        let mut output = Vec::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut output)
            .unwrap();
        assert!(
            output.is_empty(),
            "invalid initialization cannot return a ready handshake"
        );
        assert!(!process_exists(child.id()));
    }
}
