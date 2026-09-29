use super::*;
#[test]
fn host_unconfigured_has_no_implicit_timezone_and_clock_floor_is_explicit() {
    assert_eq!(
        DateHost::unconfigured().zone().unwrap_err(),
        HostClockError::Unconfigured
    );
    for (n, ms) in [
        (-1_000_001, -2),
        (-1, -1),
        (0, 0),
        (1, 0),
        (999_999, 0),
        (1_000_000, 1),
    ] {
        let mut h = DateHost::fixed_for_test(None, n);
        assert_eq!(h.now_unix_millis(&mut 1).unwrap(), ms);
    }
    let mut h = DateHost::sequence_for_test(None, Arc::from([2_000_000, -1, 3_000_000]));
    assert_eq!(h.now_unix_millis(&mut 0), Err(HostClockError::WorkLimit));
    assert_eq!(h.now_unix_millis(&mut 1).unwrap(), 2);
    assert_eq!(h.now_unix_millis(&mut 1).unwrap(), -1);
    assert_eq!(h.now_unix_millis(&mut 1).unwrap(), 3);
    assert_eq!(h.now_unix_millis(&mut 1), Err(HostClockError::Exhausted));
    assert_eq!(
        DateHost::fixed_for_test(None, i128::MAX).now_unix_millis(&mut 1),
        Err(HostClockError::OutOfRange)
    );
}
#[test]
fn host_system_clock_is_bracketed_by_real_realtime_reads() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let actual = DateHost::unconfigured().now_unix_millis(&mut 1).unwrap();
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    assert!((before..=after).contains(&actual));
}
#[cfg(target_os = "linux")]
fn config() -> ZoneDiscoveryConfig {
    ZoneDiscoveryConfig {
        tz: None,
        tzdir: None,
        default_zone_file: "/etc/localtime".into(),
        default_zone_root: "/usr/share/zoneinfo".into(),
    }
}
#[test]
#[cfg(target_os = "linux")]
fn host_configuration_transport_preserves_raw_paths_and_rejects_bounds() {
    use std::os::unix::ffi::OsStringExt;
    let mut c = config();
    c.tz = Some(vec![b'X', 255]);
    c.tzdir = Some(b"/tmp/zones".to_vec());
    c.default_zone_file = std::ffi::OsString::from_vec(b"/tmp/\xffzone".to_vec()).into();
    let bytes = c.encode().unwrap();
    let d = ZoneDiscoveryConfig::decode(&bytes).unwrap();
    assert_eq!(c.tz, d.tz);
    assert_eq!(c.default_zone_file, d.default_zone_file);
    for n in 0..bytes.len() {
        assert!(ZoneDiscoveryConfig::decode(&bytes[..n]).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(ZoneDiscoveryConfig::decode(&trailing).is_err());
    c.tz = Some(vec![b'x'; MAX_PATH_BYTES + 1]);
    assert!(c.encode().is_err());
    c.tz = Some(vec![0]);
    assert!(c.encode().is_err());
}
#[test]
#[cfg(target_os = "linux")]
fn host_explicit_empty_and_posix_are_settings_not_error_fallbacks() {
    let mut c = config();
    c.tz = Some(Vec::new());
    let (p, s) = capture_selected(&c).unwrap();
    assert_eq!(s, ZoneSelection::ExplicitUtc);
    assert_eq!(p.kind, ZonePayloadKind::ExplicitUtc);
    c.tz = Some(b"EST5EDT,M3.2.0,M11.1.0".to_vec());
    let (p, s) = capture_selected(&c).unwrap();
    assert_eq!(s, ZoneSelection::Posix);
    assert_eq!(p.kind, ZonePayloadKind::Posix2024);
    c.default_zone_root = "/nonexistent-eris-zone-root".into();
    for value in ["EST5EDT", "Missing/Zone", "../escape", "A//B", ":"] {
        c.tz = Some(value.as_bytes().to_vec());
        assert!(capture_selected(&c).is_err(), "{value}");
    }
    c.tz = None;
    c.default_zone_file = "/nonexistent-eris-localtime".into();
    assert!(capture_selected(&c).is_err());
}
#[test]
#[cfg(target_os = "linux")]
fn host_pinned_file_refuses_special_files_traversal_and_magic_links() {
    use std::{fs, os::unix::fs::symlink};
    let dir = std::env::temp_dir().join(format!("eris-zone-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("zone"), b"exact bytes").unwrap();
    symlink("zone", dir.join("link")).unwrap();
    assert_eq!(
        read_pinned_file(Path::new("link"), Some(&dir)).unwrap(),
        b"exact bytes"
    );
    assert!(read_pinned_file(Path::new("../etc/localtime"), Some(&dir)).is_err());
    assert!(matches!(
        read_pinned_file(Path::new("/dev/null"), None),
        Err(HostZoneError::UnsafeFile)
    ));
    assert!(matches!(
        read_pinned_file(&dir, None),
        Err(HostZoneError::UnsafeFile)
    ));
    let file = std::fs::File::open(dir.join("zone")).unwrap();
    use std::os::fd::AsRawFd;
    assert!(
        read_pinned_file(
            Path::new(&format!("/proc/self/fd/{}", file.as_raw_fd())),
            None
        )
        .is_err()
    );
    use rustix::fs::{CWD, FileType, Mode, mknodat};
    mknodat(
        CWD,
        dir.join("fifo"),
        FileType::Fifo,
        Mode::RUSR | Mode::WUSR,
        0,
    )
    .unwrap();
    assert!(matches!(
        read_pinned_file(&dir.join("fifo"), None),
        Err(HostZoneError::UnsafeFile)
    ));
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn host_response_payload_is_revalidated_and_errors_never_become_utc() {
    let response = b"ETZ1\x01\x02EST5EDT,M3.2.0,M11.1.0";
    let captured = decode_response(response).unwrap();
    assert_eq!(
        captured.zone.offset_at_utc_ms(0, &mut 1000).unwrap(),
        -18_000
    );
    assert!(decode_response(b"ETZ1\x02\x01garbage").is_err());
    assert!(decode_response(b"ETZ1\xff\x00").is_err());
}

#[test]
#[cfg(target_os = "linux")]
fn host_supervisor_retains_pending_owner_after_cancellation_and_drop_never_joins() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let mut supervisor = ZoneDiscoverySupervisor::with_service(move |_| {
        entered_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        Err(HostZoneError::Cancelled)
    });
    let start = Instant::now();
    let result = supervisor.capture(Path::new("/test-trusted-helper"), config(), || {
        // Interior-mutability-free cancellation signal, driven by the service.
        entered_rx.try_recv().is_ok()
    });
    assert!(matches!(result, Err(HostZoneError::Cancelled)));
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(supervisor.cleanup_pending());
    assert!(matches!(
        supervisor.capture(Path::new("/test-trusted-helper"), config(), || false),
        Err(HostZoneError::CleanupPending)
    ));
    let start = Instant::now();
    drop(supervisor);
    assert!(start.elapsed() < Duration::from_millis(100));
    release_tx.send(()).unwrap();
}

#[test]
#[cfg(target_os = "linux")]
fn host_supervisor_deadline_covers_pending_spawn_and_refuses_accumulation() {
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let mut supervisor = ZoneDiscoverySupervisor::with_service(move |_| {
        release_rx.recv().unwrap();
        Err(HostZoneError::Cancelled)
    });
    let started = Instant::now();
    assert!(matches!(
        supervisor.capture(Path::new("/test-trusted-helper"), config(), || false),
        Err(HostZoneError::TimedOut)
    ));
    assert!(started.elapsed() >= CAPTURE_TIMEOUT);
    assert!(started.elapsed() < CAPTURE_TIMEOUT + Duration::from_secs(1));
    assert!(supervisor.cleanup_pending());
    assert!(matches!(
        supervisor.capture(Path::new("/test-trusted-helper"), config(), || false),
        Err(HostZoneError::CleanupPending)
    ));
    release_tx.send(()).unwrap();
}

#[cfg(target_os = "linux")]
fn shell_fixture(label: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path =
        std::env::temp_dir().join(format!("eris-zone-helper-{}-{label}", std::process::id()));
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
#[test]
#[cfg(target_os = "linux")]
fn host_actual_helper_success_nonzero_malformed_and_flood_are_distinguished() {
    let mut supervisor = ZoneDiscoverySupervisor::new();
    for (name, script, expected) in [
        (
            "success",
            "while read line; do :; done; printf 'ETZ1\\001\\002EST5EDT,M3.2.0,M11.1.0'",
            0,
        ),
        (
            "nonzero",
            "while read line; do :; done; printf 'ETZ1\\001\\002EST5EDT,M3.2.0,M11.1.0'; exit 2",
            1,
        ),
        (
            "malformed",
            "while read line; do :; done; printf 'ETZ1\\002\\001trailing'",
            1,
        ),
        (
            "flood",
            "while read line; do :; done; while :; do printf '1234567890123456789012345678901234567890123456789012345678901234567890'; done",
            1,
        ),
    ] {
        let path = shell_fixture(name, script);
        let result = supervisor.capture(&path, config(), || false);
        if expected == 0 {
            let captured = result.unwrap();
            assert_eq!(
                captured.zone.offset_at_utc_ms(0, &mut 1000).unwrap(),
                -18_000
            );
        } else if name == "flood" {
            assert!(
                matches!(result, Err(HostZoneError::Zone(ZoneError::TooLarge))),
                "{result:?}"
            );
        } else {
            assert!(result.is_err(), "{name}");
        }
        assert!(!supervisor.cleanup_pending());
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
#[cfg(target_os = "linux")]
fn host_actual_blocked_helper_cancelled_and_reaped_before_reuse() {
    let marker = std::env::temp_dir().join(format!("eris-zone-pid-{}", std::process::id()));
    let path = shell_fixture(
        "cancel",
        &format!("echo $$ > '{}'; while :; do :; done", marker.display()),
    );
    let mut supervisor = ZoneDiscoverySupervisor::new();
    let start = Instant::now();
    let result = supervisor.capture(&path, config(), || {
        start.elapsed() > Duration::from_millis(40)
    });
    assert!(matches!(result, Err(HostZoneError::Cancelled)));
    let deadline = Instant::now() + Duration::from_secs(1);
    while supervisor.cleanup_pending() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!supervisor.cleanup_pending());
    let pid = std::fs::read_to_string(&marker)
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    assert!(!Path::new(&format!("/proc/{pid}")).exists());
    std::fs::remove_file(marker).unwrap();
    std::fs::remove_file(path).unwrap();
}

#[test]
#[cfg(target_os = "linux")]
fn host_non_utf8_file_names_and_source_size_cap_are_exact() {
    use std::{fs, os::unix::ffi::OsStringExt};
    let dir = std::env::temp_dir().join(format!("eris-zone-byte-path-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let name = std::ffi::OsString::from_vec(b"named-\xff-zone".to_vec());
    let bytes = include_bytes!("../../tests/fixtures/date-zones/Asia/Kathmandu");
    fs::write(dir.join(&name), bytes).unwrap();
    let mut config = config();
    config.default_zone_root = dir.clone();
    config.tz = Some(b"named-\xff-zone".to_vec());
    let (payload, selection) = capture_selected(&config).unwrap();
    assert_eq!(selection, ZoneSelection::NamedFile);
    assert_eq!(&*payload.bytes, bytes);
    assert_eq!(
        DateHost::from_payload(&payload)
            .unwrap()
            .zone()
            .unwrap()
            .offset_at_utc_ms(0, &mut 1000)
            .unwrap(),
        19_800
    );
    let oversized = dir.join("large");
    let file = fs::File::create(&oversized).unwrap();
    file.set_len((MAX_ZONE_SOURCE_BYTES + 1) as u64).unwrap();
    assert!(matches!(
        read_pinned_file(&oversized, None),
        Err(HostZoneError::Zone(ZoneError::TooLarge))
    ));
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(target_os = "linux")]
fn poll_request() -> CaptureRequest {
    let (result, _) = mpsc::sync_channel(1);
    CaptureRequest {
        executable: PathBuf::new(),
        request: Vec::new(),
        deadline: Instant::now() + Duration::from_secs(1),
        cancel: Arc::new(AtomicBool::new(false)),
        result,
    }
}

#[test]
#[cfg(target_os = "linux")]
fn host_poll_services_output_while_request_writes_are_blocked_and_drains_hup() {
    use rustix::event::{PollFd, PollFlags, poll};
    use std::io::{Read, Write};
    let (mut output, mut producer) = std::io::pipe().unwrap();
    let (consumer, mut input) = std::io::pipe().unwrap();
    nonblocking(&output).unwrap();
    nonblocking(&input).unwrap();
    let chunk = [b'x'; 4096];
    let mut blocked = false;
    // Only a fixed stack chunk is retained; the byte and iteration ceiling also
    // bounds this test on hosts with unusually large pipe buffers.
    for _ in 0..1024 {
        match input.write(&chunk) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                blocked = true;
                break;
            }
            Err(e) => panic!("fill request direction: {e}"),
        }
    }
    assert!(blocked, "request direction must be demonstrably blocked");
    producer.write_all(b"ET").unwrap();
    let request = poll_request();
    {
        let mut fds = [
            PollFd::new(&output, PollFlags::IN),
            PollFd::new(&input, PollFlags::OUT),
        ];
        poll_zone_fds(&request, &mut fds, poll).unwrap();
        assert!(fds[0].revents().intersects(PollFlags::IN | PollFlags::HUP));
        assert!(!fds[1].revents().contains(PollFlags::OUT));
    }
    let mut first = [0; 2];
    output.read_exact(&mut first).unwrap();
    assert_eq!(&first, b"ET");
    assert_eq!(
        output.read(&mut first).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    producer.write_all(b"Z1\x01\x02UTC0").unwrap();
    drop(producer);
    {
        let mut fds = [
            PollFd::new(&output, PollFlags::IN),
            PollFd::new(&input, PollFlags::OUT),
        ];
        poll_zone_fds(&request, &mut fds, poll).unwrap();
        assert!(fds[0].revents().contains(PollFlags::HUP));
    }
    let mut response = b"ET".to_vec();
    output.read_to_end(&mut response).unwrap();
    assert_eq!(
        decode_response(&response).unwrap().payload.bytes.as_ref(),
        b"UTC0"
    );
    drop(consumer);
    let mut fds = [
        PollFd::new(&output, PollFlags::IN),
        PollFd::new(&input, PollFlags::OUT),
    ];
    assert_eq!(
        poll_zone_fds(&request, &mut fds, poll),
        Err(HostZoneError::Unavailable)
    );
}

#[test]
#[cfg(target_os = "linux")]
fn host_poll_interrupts_and_timeouts_recheck_the_original_deadline_and_cancel() {
    use rustix::event::{PollFd, PollFlags};
    use std::cell::Cell;
    let (output, _producer) = std::io::pipe().unwrap();
    let mut request = poll_request();
    let mut fds = [PollFd::new(&output, PollFlags::IN)];
    let calls = Cell::new(0);
    poll_zone_fds(&request, &mut fds, |fds, timeout| {
        calls.set(calls.get() + 1);
        assert_eq!(fds.len(), 1);
        let timeout = timeout.unwrap();
        assert_eq!(timeout.tv_sec, 0);
        assert!(timeout.tv_nsec > 0 && timeout.tv_nsec <= POLL_SLICE.as_nanos() as i64);
        Err(rustix::io::Errno::INTR)
    })
    .unwrap();
    assert_eq!(calls.get(), 1, "interruption must return to the outer loop");
    assert_eq!(
        poll_zone_fds(&request, &mut fds, |_, _| {
            request.cancel.store(true, Ordering::Release);
            Err(rustix::io::Errno::INTR)
        }),
        Err(HostZoneError::Cancelled)
    );
    request.cancel.store(false, Ordering::Release);
    request.deadline = Instant::now() - Duration::from_millis(1);
    assert_eq!(
        poll_zone_fds(&request, &mut fds, |_, _| panic!(
            "expired request must not poll"
        )),
        Err(HostZoneError::TimedOut)
    );
    assert_eq!(zone_wait_duration(&request), Err(HostZoneError::TimedOut));
    request.deadline = Instant::now() + POLL_SLICE / 2;
    let original_remaining = request.deadline.saturating_duration_since(Instant::now());
    // Scheduling may consume this tiny remaining interval. Either refusal or a
    // timeout no larger than that remainder is correct; no sub-10ms speed claim.
    let result = poll_zone_fds(&request, &mut fds, |_, timeout| {
        assert!(timeout.unwrap().tv_nsec <= original_remaining.as_nanos() as i64);
        Ok(0)
    });
    assert!(matches!(result, Ok(()) | Err(HostZoneError::TimedOut)));
}

#[cfg(target_os = "linux")]
fn wait_for_capture_cleanup(supervisor: &ZoneDiscoverySupervisor) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while supervisor.cleanup_pending() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        !supervisor.cleanup_pending(),
        "owned helper must be reaped before reuse"
    );
}
#[cfg(target_os = "linux")]
fn marker_pid(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

#[test]
#[cfg(target_os = "linux")]
fn host_poll_fragmented_response_and_final_hangup_preserve_all_bytes() {
    use std::cell::Cell;
    let base = std::env::temp_dir().join(format!("eris-zone-fragments-{}", std::process::id()));
    let marker = base.with_extension("marker");
    let gate = base.with_extension("gate");
    let _ = std::fs::remove_file(&marker);
    let _ = std::fs::remove_file(&gate);
    let script = format!(
        "while read line; do :; done; printf 'ET'; printf '%s' $$ > '{}'; while [ ! -f '{}' ]; do :; done; printf 'Z1\\001\\002EST5EDT,M3.2.0,M11.1.0'",
        marker.display(),
        gate.display()
    );
    let path = shell_fixture("fragments", &script);
    let released = Cell::new(false);
    let mut supervisor = ZoneDiscoverySupervisor::new();
    let captured = supervisor
        .capture(&path, config(), || {
            if marker_pid(&marker).is_some() && !released.get() {
                std::fs::write(&gate, b"continue").unwrap();
                released.set(true);
            }
            false
        })
        .unwrap();
    assert!(released.get());
    assert_eq!(captured.payload.bytes.as_ref(), b"EST5EDT,M3.2.0,M11.1.0");
    assert_eq!(
        captured.zone.offset_at_utc_ms(0, &mut 1000).unwrap(),
        -18_000
    );
    wait_for_capture_cleanup(&supervisor);
    // Completion is reported only after run_capture has reaped its owned Child.
    for path in [path, marker, gate] {
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
#[cfg(target_os = "linux")]
fn host_eof_before_exit_waits_for_exit_status_or_cancellation_and_deadline() {
    use std::cell::Cell;
    for action in ["success", "nonzero", "cancel", "timeout"] {
        let base =
            std::env::temp_dir().join(format!("eris-zone-eof-{}-{action}", std::process::id()));
        let marker = base.with_extension("marker");
        let gate = base.with_extension("gate");
        let _ = std::fs::remove_file(&marker);
        let _ = std::fs::remove_file(&gate);
        let status = if action == "nonzero" { 7 } else { 0 };
        // Explicitly close stdout while still alive. A valid response plus EOF
        // must not succeed until the separate non-reaping exit observation.
        let script = format!(
            "while read line; do :; done; printf 'ETZ1\\001\\002UTC0'; exec 1>&-; printf '%s' $$ > '{}'; while [ ! -f '{}' ]; do :; done; exit {status}",
            marker.display(),
            gate.display()
        );
        let path = shell_fixture(&format!("eof-{action}"), &script);
        let mut supervisor = ZoneDiscoverySupervisor::new();
        let observed_alive = Cell::new(false);
        let result = supervisor.capture(&path, config(), || {
            if let Some(pid) = marker_pid(&marker) {
                observed_alive
                    .set(observed_alive.get() || Path::new(&format!("/proc/{pid}")).exists());
                match action {
                    "success" | "nonzero" => {
                        std::fs::write(&gate, b"exit").unwrap();
                    }
                    "cancel" => return true,
                    _ => {}
                }
            }
            false
        });
        assert!(
            observed_alive.get(),
            "test must reach the still-live EOF state: {action}"
        );
        match action {
            "success" => assert!(result.is_ok()),
            "nonzero" => assert!(matches!(result, Err(HostZoneError::Unavailable))),
            "cancel" => assert!(matches!(result, Err(HostZoneError::Cancelled))),
            "timeout" => assert!(matches!(result, Err(HostZoneError::TimedOut))),
            _ => unreachable!(),
        }
        wait_for_capture_cleanup(&supervisor);
        // The retained supervisor reaches idle only after its owned Child is reaped;
        // a post-reap numeric PID lookup would not prove ownership (PID reuse).
        for path in [path, marker, gate] {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[test]
#[cfg(target_os = "linux")]
fn host_run_capture_drains_output_during_a_large_private_partial_request() {
    let base = std::env::temp_dir().join(format!("eris-zone-large-request-{}", std::process::id()));
    let marker = base.with_extension("marker");
    let gate = base.with_extension("gate");
    let _ = std::fs::remove_file(&marker);
    let _ = std::fs::remove_file(&gate);
    // Deliberate private stress input, not an accepted ZoneDiscoveryConfig:
    // its 1 MiB+4 KiB exceeds the public 12 KiB cap and ordinary pipe capacity.
    // The real run_capture write/read loop still has to deliver every line while
    // a response prefix is already available and the helper initially won't read.
    const LINES: usize = 1028;
    let script = format!(
        "printf 'ET'; printf '%s' $$ > '{}'; while [ ! -f '{}' ]; do :; done; n=0; while read line; do n=$((n+1)); done; [ \"$n\" -eq {LINES} ] || exit 9; printf 'Z1\\001\\002UTC0'",
        marker.display(),
        gate.display()
    );
    let path = shell_fixture("large-private-request", &script);
    let mut request = poll_request();
    request.executable = path.clone();
    request.request = vec![b'x'; LINES * 1024];
    for byte in request.request.iter_mut().skip(1023).step_by(1024) {
        *byte = b'\n';
    }
    assert!(request.request.len() > MAX_ZONE_CONFIG_BYTES);
    let mut invalid = config();
    invalid.tz = Some(vec![b'x'; MAX_ZONE_CONFIG_BYTES]);
    assert!(invalid.encode().is_err());
    let cancel = request.cancel.clone();
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let result = run_capture(&request);
        let _ = done_tx.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(1);
    while marker_pid(&marker).is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    if marker_pid(&marker).is_some() {
        std::fs::write(&gate, b"drain").unwrap();
    } else {
        cancel.store(true, Ordering::Release);
    }
    let result = done_rx.recv_timeout(Duration::from_secs(2));
    cancel.store(true, Ordering::Release);
    let captured = result.unwrap().unwrap();
    assert_eq!(captured.payload.bytes.as_ref(), b"UTC0");
    worker.join().unwrap(); // result is sent only after owned-child cleanup.
    for path in [path, marker, gate] {
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
#[cfg(target_os = "linux")]
fn host_run_capture_silent_blocked_helper_obeys_cancel_and_deadline() {
    use std::cell::Cell;
    for action in ["cancel", "timeout"] {
        let base =
            std::env::temp_dir().join(format!("eris-zone-silent-{}-{action}", std::process::id()));
        let marker = base.with_extension("marker");
        let fifo = base.with_extension("fifo");
        let _ = std::fs::remove_file(&marker);
        let _ = std::fs::remove_file(&fifo);
        use rustix::fs::{CWD, FileType, Mode, mknodat};
        mknodat(CWD, &fifo, FileType::Fifo, Mode::RUSR | Mode::WUSR, 0).unwrap();
        let script = format!(
            "while read line; do :; done; printf '%s' $$ > '{}'; read never < '{}'",
            marker.display(),
            fifo.display()
        );
        let path = shell_fixture(&format!("silent-{action}"), &script);
        let observed = Cell::new(false);
        let mut supervisor = ZoneDiscoverySupervisor::new();
        let result = supervisor.capture(&path, config(), || {
            observed.set(observed.get() || marker_pid(&marker).is_some());
            action == "cancel" && observed.get()
        });
        assert!(observed.get(), "helper must reach the silent blocking read");
        if action == "cancel" {
            assert!(matches!(result, Err(HostZoneError::Cancelled)));
        } else {
            assert!(matches!(result, Err(HostZoneError::TimedOut)));
        }
        wait_for_capture_cleanup(&supervisor);
        for path in [path, marker, fifo] {
            std::fs::remove_file(path).unwrap();
        }
    }
}
