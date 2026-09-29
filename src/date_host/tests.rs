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
