//! Explicit Date embedding context and bounded Linux host-zone discovery.
//! The renderer receives validated bytes, never a pathname or open file.
//! Unconfigured contexts permit a real UTC wall clock but refuse local rules.
pub use crate::js_date::time_zone::{
    MAX_ABS_OFFSET_SECONDS, MAX_POSIX_BYTES, MAX_ZONE_SOURCE_BYTES, TimeZoneSnapshot, ZoneBudget,
    ZoneError, ZonePayload, ZonePayloadKind,
};
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug)]
pub struct DateHost {
    zone: Option<Arc<TimeZoneSnapshot>>,
    clock: WallClock,
}
#[derive(Clone, Debug)]
enum WallClock {
    System,
    #[cfg(test)]
    Fixed(i128),
    #[cfg(test)]
    Sequence {
        nanos: Arc<[i128]>,
        next: usize,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostClockError {
    Unconfigured,
    WorkLimit,
    OutOfRange,
    Exhausted,
}
impl HostClockError {
    pub fn is_work_limit(self) -> bool {
        self == Self::WorkLimit
    }
}
impl fmt::Display for HostClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Date host {self:?}")
    }
}
impl std::error::Error for HostClockError {}
impl Default for DateHost {
    fn default() -> Self {
        Self::unconfigured()
    }
}
impl DateHost {
    pub fn unconfigured() -> Self {
        Self {
            zone: None,
            clock: WallClock::System,
        }
    }
    pub fn system(zone: Arc<TimeZoneSnapshot>) -> Self {
        Self {
            zone: Some(zone),
            clock: WallClock::System,
        }
    }
    pub fn from_payload(payload: &ZonePayload) -> Result<Self, ZoneError> {
        Ok(Self::system(Arc::new(TimeZoneSnapshot::parse(
            payload,
            &mut ZoneBudget::default(),
        )?)))
    }
    pub fn zone(&self) -> Result<&TimeZoneSnapshot, HostClockError> {
        self.zone.as_deref().ok_or(HostClockError::Unconfigured)
    }
    pub fn now_unix_millis(&mut self, work: &mut usize) -> Result<i64, HostClockError> {
        *work = work.checked_sub(1).ok_or(HostClockError::WorkLimit)?;
        let nanos = match &mut self.clock {
            WallClock::System => match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(v) => i128::try_from(v.as_nanos()).map_err(|_| HostClockError::OutOfRange)?,
                Err(e) => -i128::try_from(e.duration().as_nanos())
                    .map_err(|_| HostClockError::OutOfRange)?,
            },
            #[cfg(test)]
            WallClock::Fixed(n) => *n,
            #[cfg(test)]
            WallClock::Sequence { nanos, next } => {
                let n = *nanos.get(*next).ok_or(HostClockError::Exhausted)?;
                *next += 1;
                n
            }
        };
        let ms = nanos.div_euclid(1_000_000);
        if ms.unsigned_abs() > 8_640_000_000_000_000 {
            return Err(HostClockError::OutOfRange);
        }
        i64::try_from(ms).map_err(|_| HostClockError::OutOfRange)
    }
    #[cfg(test)]
    pub(crate) fn fixed_for_test(zone: Option<Arc<TimeZoneSnapshot>>, nanos: i128) -> Self {
        Self {
            zone,
            clock: WallClock::Fixed(nanos),
        }
    }
    #[cfg(test)]
    pub(crate) fn sequence_for_test(
        zone: Option<Arc<TimeZoneSnapshot>>,
        nanos: Arc<[i128]>,
    ) -> Self {
        Self {
            zone,
            clock: WallClock::Sequence { nanos, next: 0 },
        }
    }
}

pub const MAX_ZONE_CONFIG_BYTES: usize = 12_288;
const MAX_PATH_BYTES: usize = 4096;
const MAX_RESPONSE_BYTES: usize = MAX_ZONE_SOURCE_BYTES + 16;
const CAPTURE_TIMEOUT: Duration = Duration::from_millis(1000);
const CLEANUP_WINDOW: Duration = Duration::from_millis(100);
const POLL_SLICE: Duration = Duration::from_millis(10);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostZoneError {
    InvalidConfiguration,
    Unavailable,
    UnsafeFile,
    TimedOut,
    Cancelled,
    CleanupPending,
    ChangedDuringRead,
    Zone(ZoneError),
}
impl fmt::Display for HostZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "host timezone {self:?}")
    }
}
impl std::error::Error for HostZoneError {}
impl From<ZoneError> for HostZoneError {
    fn from(e: ZoneError) -> Self {
        Self::Zone(e)
    }
}
#[derive(Clone, Debug)]
pub struct ZoneDiscoveryConfig {
    pub tz: Option<Vec<u8>>,
    pub tzdir: Option<Vec<u8>>,
    pub default_zone_file: PathBuf,
    pub default_zone_root: PathBuf,
}
impl ZoneDiscoveryConfig {
    pub fn from_environment() -> Result<Self, HostZoneError> {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            fn env(name: &str) -> Result<Option<Vec<u8>>, HostZoneError> {
                std::env::var_os(name)
                    .map(|s| {
                        let bytes = s.as_bytes();
                        if bytes.len() > MAX_PATH_BYTES {
                            return Err(HostZoneError::InvalidConfiguration);
                        }
                        Ok(bytes.to_vec())
                    })
                    .transpose()
            }
            let out = Self {
                tz: env("TZ")?,
                tzdir: env("TZDIR")?,
                default_zone_file: "/etc/localtime".into(),
                default_zone_root: "/usr/share/zoneinfo".into(),
            };
            out.encode()?;
            Ok(out)
        }
        #[cfg(not(unix))]
        {
            Err(HostZoneError::Unavailable)
        }
    }
    fn encode(&self) -> Result<Vec<u8>, HostZoneError> {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let fields = [
                self.tz.as_deref(),
                self.tzdir.as_deref(),
                Some(self.default_zone_file.as_os_str().as_bytes()),
                Some(self.default_zone_root.as_os_str().as_bytes()),
            ];
            let mut size = 4;
            for bytes in fields {
                size += 4;
                if let Some(bytes) = bytes {
                    if bytes.len() > MAX_PATH_BYTES || bytes.contains(&0) {
                        return Err(HostZoneError::InvalidConfiguration);
                    }
                    size += bytes.len();
                }
            }
            if size > MAX_ZONE_CONFIG_BYTES {
                return Err(HostZoneError::InvalidConfiguration);
            }
            let mut out = Vec::new();
            out.try_reserve_exact(size)
                .map_err(|_| HostZoneError::Zone(ZoneError::Allocation))?;
            out.extend_from_slice(b"ETZ1");
            for bytes in fields {
                match bytes {
                    Some(bytes) => {
                        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        out.extend_from_slice(bytes)
                    }
                    None => out.extend_from_slice(&u32::MAX.to_le_bytes()),
                }
            }
            Ok(out)
        }
        #[cfg(not(unix))]
        {
            Err(HostZoneError::Unavailable)
        }
    }
    #[cfg(target_os = "linux")]
    fn decode(bytes: &[u8]) -> Result<Self, HostZoneError> {
        use std::os::unix::ffi::OsStringExt;
        if bytes.len() > MAX_ZONE_CONFIG_BYTES || bytes.get(..4) != Some(b"ETZ1") {
            return Err(HostZoneError::InvalidConfiguration);
        }
        let mut at = 4;
        let mut field = || -> Result<Option<Vec<u8>>, HostZoneError> {
            let end = at + 4;
            let raw = bytes
                .get(at..end)
                .ok_or(HostZoneError::InvalidConfiguration)?;
            at = end;
            let n = u32::from_le_bytes(raw.try_into().unwrap());
            if n == u32::MAX {
                return Ok(None);
            }
            let n = n as usize;
            if n > MAX_PATH_BYTES {
                return Err(HostZoneError::InvalidConfiguration);
            }
            let value = bytes
                .get(at..at + n)
                .ok_or(HostZoneError::InvalidConfiguration)?;
            at += n;
            if value.contains(&0) {
                return Err(HostZoneError::InvalidConfiguration);
            }
            Ok(Some(value.to_vec()))
        };
        let tz = field()?;
        let tzdir = field()?;
        let file = field()?.ok_or(HostZoneError::InvalidConfiguration)?;
        let root = field()?.ok_or(HostZoneError::InvalidConfiguration)?;
        if at != bytes.len() {
            return Err(HostZoneError::InvalidConfiguration);
        }
        Ok(Self {
            tz,
            tzdir,
            default_zone_file: std::ffi::OsString::from_vec(file).into(),
            default_zone_root: std::ffi::OsString::from_vec(root).into(),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneSelection {
    DefaultFile,
    ExplicitUtc,
    Posix,
    AbsoluteFile,
    NamedFile,
}
#[derive(Clone, Debug)]
pub struct HostZoneProvenance {
    pub selection: ZoneSelection,
    pub source_bytes: usize,
}
#[derive(Clone, Debug)]
pub struct CapturedTimeZone {
    pub payload: ZonePayload,
    pub zone: Arc<TimeZoneSnapshot>,
    pub provenance: HostZoneProvenance,
}
impl CapturedTimeZone {
    pub fn date_host(&self) -> DateHost {
        DateHost::system(self.zone.clone())
    }
}

struct CaptureRequest {
    executable: PathBuf,
    request: Vec<u8>,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    result: mpsc::SyncSender<Result<CapturedTimeZone, HostZoneError>>,
}
/// One service thread owns spawn and the only outstanding child. A blocked OS
/// spawn/read/termination can outlive capture's deadline; new capture is refused
/// until ownership is actually resolved. Drop never joins or blocking-waits.
pub struct ZoneDiscoverySupervisor {
    sender: Option<mpsc::SyncSender<CaptureRequest>>,
    busy: Arc<AtomicBool>,
    cancel: Option<Arc<AtomicBool>>,
}
impl Default for ZoneDiscoverySupervisor {
    fn default() -> Self {
        Self::new()
    }
}
impl ZoneDiscoverySupervisor {
    pub fn new() -> Self {
        Self::with_service(run_capture)
    }
    fn with_service(
        service: impl Fn(&CaptureRequest) -> Result<CapturedTimeZone, HostZoneError> + Send + 'static,
    ) -> Self {
        let (tx, rx) = mpsc::sync_channel::<CaptureRequest>(1);
        let busy = Arc::new(AtomicBool::new(false));
        let thread_busy = busy.clone();
        let spawned = std::thread::Builder::new()
            .name("eris-timezone".into())
            .spawn(move || {
                while let Ok(request) = rx.recv() {
                    let result = service(&request);
                    thread_busy.store(false, Ordering::Release);
                    let _ = request.result.try_send(result);
                }
            });
        Self {
            sender: spawned.ok().map(|_| tx),
            busy,
            cancel: None,
        }
    }
    pub fn cleanup_pending(&self) -> bool {
        self.busy.load(Ordering::Acquire)
    }
    pub fn capture(
        &mut self,
        executable: &Path,
        config: ZoneDiscoveryConfig,
        cancelled: impl Fn() -> bool,
    ) -> Result<CapturedTimeZone, HostZoneError> {
        let deadline = Instant::now() + CAPTURE_TIMEOUT;
        if cancelled() {
            return Err(HostZoneError::Cancelled);
        }
        if executable.as_os_str().len() > MAX_PATH_BYTES || !executable.is_absolute() {
            return Err(HostZoneError::InvalidConfiguration);
        }
        let request = config.encode()?;
        let sender = self.sender.as_ref().ok_or(HostZoneError::Unavailable)?;
        if self
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(HostZoneError::CleanupPending);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        let (tx, rx) = mpsc::sync_channel(1);
        let send = sender.try_send(CaptureRequest {
            executable: executable.to_owned(),
            request,
            deadline,
            cancel: cancel.clone(),
            result: tx,
        });
        if send.is_err() {
            self.busy.store(false, Ordering::Release);
            return Err(HostZoneError::Unavailable);
        }
        loop {
            if cancelled() {
                cancel.store(true, Ordering::Release);
                return Err(HostZoneError::Cancelled);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                cancel.store(true, Ordering::Release);
                return Err(HostZoneError::TimedOut);
            }
            match rx.recv_timeout(remaining.min(POLL_SLICE)) {
                Ok(result) => {
                    self.cancel = None;
                    return result;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(HostZoneError::Unavailable);
                }
            }
        }
    }
}
impl Drop for ZoneDiscoverySupervisor {
    fn drop(&mut self) {
        if let Some(cancel) = &self.cancel {
            cancel.store(true, Ordering::Release);
        }
        self.sender.take();
    }
}
fn stopped(request: &CaptureRequest) -> Result<(), HostZoneError> {
    if request.cancel.load(Ordering::Acquire) {
        Err(HostZoneError::Cancelled)
    } else if Instant::now() >= request.deadline {
        Err(HostZoneError::TimedOut)
    } else {
        Ok(())
    }
}
#[cfg(target_os = "linux")]
fn run_capture(request: &CaptureRequest) -> Result<CapturedTimeZone, HostZoneError> {
    use std::{
        io::{Read, Write},
        os::unix::process::CommandExt,
        process::{Command, Stdio},
    };
    stopped(request)?;
    let mut command = Command::new(&request.executable);
    #[cfg(feature = "vulkan-presenter")]
    command.arg("--clean-worker-launch");
    command
        .arg("--timezone-discovery")
        .env_clear()
        .current_dir("/")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0);
    // Command::spawn may block. This service thread retains its busy state; the
    // caller deadline does not imply the underlying kernel operation completed.
    let mut child = command.spawn().map_err(|_| HostZoneError::Unavailable)?;
    let result = (|| {
        stopped(request)?;
        let input = child.stdin.take().ok_or(HostZoneError::Unavailable)?;
        let mut output = child.stdout.take().ok_or(HostZoneError::Unavailable)?;
        nonblocking(&input)?;
        nonblocking(&output)?;
        let mut written = 0;
        let mut response = Vec::new();
        response
            .try_reserve_exact(MAX_RESPONSE_BYTES + 1)
            .map_err(|_| HostZoneError::Zone(ZoneError::Allocation))?;
        let mut input = Some(input);
        loop {
            stopped(request)?;
            if let Some(pipe) = input.as_mut() {
                match pipe.write(&request.request[written..]) {
                    Ok(0) => return Err(HostZoneError::Unavailable),
                    Ok(n) => written += n,
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                        ) => {}
                    Err(_) => return Err(HostZoneError::Unavailable),
                }
                if written == request.request.len() {
                    input = None;
                }
            }
            let mut chunk = [0u8; 4096];
            match output.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if response.len() + n > MAX_RESPONSE_BYTES {
                        return Err(HostZoneError::Zone(ZoneError::TooLarge));
                    }
                    response.extend_from_slice(&chunk[..n]);
                    continue;
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err(HostZoneError::Unavailable),
            }
            wait_zone_io(request, input.as_ref(), &output)?;
        }
        stopped(request)?;
        let captured = decode_response(&response)?;
        // Observe exit without reaping: retain PID identity until group cleanup.
        let pid =
            rustix::process::Pid::from_raw(child.id() as i32).ok_or(HostZoneError::Unavailable)?;
        loop {
            stopped(request)?;
            use rustix::process::{WaitIdOptions, waitid};
            match waitid(
                rustix::process::WaitId::Pid(pid),
                WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
            )
            .map_err(|_| HostZoneError::Unavailable)?
            {
                Some(status) if status.exit_status() == Some(0) => break,
                Some(_) => return Err(HostZoneError::Unavailable),
                // EOF is no longer polled: its persistent HUP would spin. A
                // race-dependent exit wait remains until a future pidfd change.
                None => std::thread::sleep(zone_wait_duration(request)?),
            }
        }
        stopped(request)?;
        Ok(captured)
    })();
    // Kill while PID ownership is retained, BEFORE try_wait can release identity.
    kill_owned(&mut child);
    let cleanup_start = Instant::now();
    let mut pending_result = result;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return pending_result,
            Ok(None) | Err(_) => {
                if cleanup_start.elapsed() >= CLEANUP_WINDOW {
                    // Do not publish success while teardown is unresolved. The outer
                    // deadline remains finite; this owner refuses accumulation.
                    pending_result = Err(HostZoneError::CleanupPending);
                }
                std::thread::sleep(POLL_SLICE);
            }
        }
    }
}
#[cfg(target_os = "linux")]
fn zone_wait_duration(request: &CaptureRequest) -> Result<Duration, HostZoneError> {
    stopped(request)?;
    let remaining = request.deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(HostZoneError::TimedOut);
    }
    Ok(remaining.min(POLL_SLICE))
}
#[cfg(target_os = "linux")]
fn wait_zone_io(
    request: &CaptureRequest,
    input: Option<&std::process::ChildStdin>,
    output: &std::process::ChildStdout,
) -> Result<(), HostZoneError> {
    use rustix::event::{PollFd, PollFlags, poll};
    if let Some(input) = input {
        poll_zone_fds(
            request,
            &mut [
                PollFd::new(output, PollFlags::IN),
                PollFd::new(input, PollFlags::OUT),
            ],
            poll,
        )
    } else {
        poll_zone_fds(request, &mut [PollFd::new(output, PollFlags::IN)], poll)
    }
}
#[cfg(target_os = "linux")]
fn poll_zone_fds(
    request: &CaptureRequest,
    fds: &mut [rustix::event::PollFd<'_>],
    poll: impl FnOnce(
        &mut [rustix::event::PollFd<'_>],
        Option<&rustix::event::Timespec>,
    ) -> rustix::io::Result<usize>,
) -> Result<(), HostZoneError> {
    use rustix::event::{PollFlags, Timespec};
    let duration = zone_wait_duration(request)?;
    let timeout = Timespec {
        tv_sec: 0,
        tv_nsec: duration.subsec_nanos().into(),
    };
    let result = poll(fds, Some(&timeout));
    // EINTR returns through the outer loop; neither interruption nor spurious
    // readiness gets an unchecked fresh timeout or bypasses cancellation.
    stopped(request)?;
    match result {
        Ok(_) => {
            if fds[0]
                .revents()
                .intersects(PollFlags::ERR | PollFlags::NVAL)
                || fds.get(1).is_some_and(|fd| {
                    fd.revents()
                        .intersects(PollFlags::ERR | PollFlags::NVAL | PollFlags::HUP)
                })
            {
                return Err(HostZoneError::Unavailable);
            }
            // Output HUP is deliberately retained as read readiness. Buffered
            // final bytes must be consumed before read() reports actual EOF.
            Ok(())
        }
        Err(rustix::io::Errno::INTR) => Ok(()),
        Err(_) => Err(HostZoneError::Unavailable),
    }
}

#[cfg(not(target_os = "linux"))]
fn run_capture(_request: &CaptureRequest) -> Result<CapturedTimeZone, HostZoneError> {
    Err(HostZoneError::Unavailable)
}
#[cfg(target_os = "linux")]
fn nonblocking(fd: &impl std::os::fd::AsFd) -> Result<(), HostZoneError> {
    let flags = rustix::fs::fcntl_getfl(fd).map_err(|_| HostZoneError::Unavailable)?;
    rustix::fs::fcntl_setfl(fd, flags | rustix::fs::OFlags::NONBLOCK)
        .map_err(|_| HostZoneError::Unavailable)
}
#[cfg(target_os = "linux")]
fn kill_owned(child: &mut std::process::Child) {
    if let Some(pid) = rustix::process::Pid::from_raw(child.id() as i32) {
        let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
    }
    let _ = child.kill();
}
fn decode_response(response: &[u8]) -> Result<CapturedTimeZone, HostZoneError> {
    if response.len() < 6 || &response[..4] != b"ETZ1" {
        return Err(HostZoneError::Unavailable);
    }
    if response[4] == 255 {
        if response.len() != 6 {
            return Err(HostZoneError::InvalidConfiguration);
        }
        return Err(match response[5] {
            1 => HostZoneError::InvalidConfiguration,
            2 => HostZoneError::UnsafeFile,
            3 => HostZoneError::ChangedDuringRead,
            4 => HostZoneError::Zone(ZoneError::MissingRules),
            _ => HostZoneError::Unavailable,
        });
    }
    let kind = match response[4] {
        0 => ZonePayloadKind::Tzif,
        1 => ZonePayloadKind::Posix2024,
        2 => ZonePayloadKind::ExplicitUtc,
        _ => return Err(HostZoneError::InvalidConfiguration),
    };
    let selection = match response[5] {
        0 => ZoneSelection::DefaultFile,
        1 => ZoneSelection::ExplicitUtc,
        2 => ZoneSelection::Posix,
        3 => ZoneSelection::AbsoluteFile,
        4 => ZoneSelection::NamedFile,
        _ => return Err(HostZoneError::InvalidConfiguration),
    };
    let compatible = matches!(
        (kind, selection),
        (
            ZonePayloadKind::Tzif,
            ZoneSelection::DefaultFile | ZoneSelection::AbsoluteFile | ZoneSelection::NamedFile
        ) | (ZonePayloadKind::Posix2024, ZoneSelection::Posix)
            | (ZonePayloadKind::ExplicitUtc, ZoneSelection::ExplicitUtc)
    );
    if !compatible {
        return Err(HostZoneError::InvalidConfiguration);
    }
    let payload = ZonePayload::new(kind, &response[6..])?;
    let zone = Arc::new(TimeZoneSnapshot::parse(
        &payload,
        &mut ZoneBudget::default(),
    )?);
    Ok(CapturedTimeZone {
        provenance: HostZoneProvenance {
            selection,
            source_bytes: payload.bytes.len(),
        },
        payload,
        zone,
    })
}

/// Early internal executable role. Only this helper performs zone filesystem I/O.
pub fn serve_timezone_discovery() -> Result<(), HostZoneError> {
    #[cfg(target_os = "linux")]
    {
        use std::io::{Read, Write};
        check_descriptors()?;
        use rustix::process::{Resource, Rlimit, setrlimit};
        for (resource, value) in [
            (Resource::Cpu, 1),
            (Resource::As, 536_870_912),
            (Resource::Nofile, 32),
            (Resource::Core, 0),
        ] {
            setrlimit(
                resource,
                Rlimit {
                    current: Some(value),
                    maximum: Some(value),
                },
            )
            .map_err(|_| HostZoneError::Unavailable)?;
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAX_ZONE_CONFIG_BYTES + 1)
            .map_err(|_| HostZoneError::Zone(ZoneError::Allocation))?;
        std::io::stdin()
            .lock()
            .take((MAX_ZONE_CONFIG_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| HostZoneError::Unavailable)?;
        let result =
            ZoneDiscoveryConfig::decode(&bytes).and_then(|config| capture_selected(&config));
        let mut out = std::io::stdout().lock();
        out.write_all(b"ETZ1")
            .map_err(|_| HostZoneError::Unavailable)?;
        match result {
            Ok((payload, selection)) => {
                let kind = match payload.kind {
                    ZonePayloadKind::Tzif => 0,
                    ZonePayloadKind::Posix2024 => 1,
                    ZonePayloadKind::ExplicitUtc => 2,
                };
                let selection = match selection {
                    ZoneSelection::DefaultFile => 0,
                    ZoneSelection::ExplicitUtc => 1,
                    ZoneSelection::Posix => 2,
                    ZoneSelection::AbsoluteFile => 3,
                    ZoneSelection::NamedFile => 4,
                };
                out.write_all(&[kind, selection])
                    .and_then(|()| out.write_all(&payload.bytes))
                    .map_err(|_| HostZoneError::Unavailable)?;
            }
            Err(e) => {
                let code = match e {
                    HostZoneError::InvalidConfiguration => 1,
                    HostZoneError::UnsafeFile => 2,
                    HostZoneError::ChangedDuringRead => 3,
                    HostZoneError::Zone(ZoneError::MissingRules) => 4,
                    _ => 0,
                };
                out.write_all(&[255, code])
                    .map_err(|_| HostZoneError::Unavailable)?;
            }
        }
        out.flush().map_err(|_| HostZoneError::Unavailable)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(HostZoneError::Unavailable)
    }
}
#[cfg(target_os = "linux")]
fn check_descriptors() -> Result<(), HostZoneError> {
    let directory = PathBuf::from(format!("/proc/{}/fd", std::process::id()));
    let mut scanner = 0;
    for entry in std::fs::read_dir(&directory).map_err(|_| HostZoneError::Unavailable)? {
        let entry = entry.map_err(|_| HostZoneError::Unavailable)?;
        let fd: u32 = entry
            .file_name()
            .to_str()
            .ok_or(HostZoneError::UnsafeFile)?
            .parse()
            .map_err(|_| HostZoneError::UnsafeFile)?;
        if fd < 3 {
            continue;
        }
        if std::fs::read_link(entry.path()).map_err(|_| HostZoneError::Unavailable)? != directory {
            return Err(HostZoneError::UnsafeFile);
        }
        scanner += 1;
    }
    if scanner != 1 {
        Err(HostZoneError::UnsafeFile)
    } else {
        Ok(())
    }
}
#[cfg(target_os = "linux")]
fn capture_selected(
    config: &ZoneDiscoveryConfig,
) -> Result<(ZonePayload, ZoneSelection), HostZoneError> {
    use std::os::unix::ffi::OsStrExt;
    let file = |path: &Path, root: Option<&Path>, selection| -> Result<_, HostZoneError> {
        Ok((
            ZonePayload::new(ZonePayloadKind::Tzif, &read_pinned_file(path, root)?)?,
            selection,
        ))
    };
    let Some(tz) = &config.tz else {
        return file(&config.default_zone_file, None, ZoneSelection::DefaultFile);
    };
    if tz.is_empty() {
        return Ok((
            ZonePayload::new(ZonePayloadKind::ExplicitUtc, &[])?,
            ZoneSelection::ExplicitUtc,
        ));
    }
    let value = tz.strip_prefix(b":").unwrap_or(tz);
    if value.is_empty() {
        return Err(HostZoneError::InvalidConfiguration);
    }
    let path = Path::new(std::ffi::OsStr::from_bytes(value));
    if path.is_absolute() {
        return file(path, None, ZoneSelection::AbsoluteFile);
    }
    let mut missing_rules = false;
    if value.is_ascii() && value.len() <= MAX_POSIX_BYTES {
        let payload = ZonePayload::new(ZonePayloadKind::Posix2024, value)?;
        match TimeZoneSnapshot::parse(&payload, &mut ZoneBudget::default()) {
            Ok(_) => return Ok((payload, ZoneSelection::Posix)),
            Err(ZoneError::MissingRules) => missing_rules = true,
            Err(ZoneError::Malformed) => {}
            Err(e) => return Err(HostZoneError::Zone(e)),
        }
    }
    if value
        .split(|&b| b == b'/')
        .any(|p| p.is_empty() || p == b"." || p == b"..")
    {
        return Err(HostZoneError::InvalidConfiguration);
    }
    let root = match &config.tzdir {
        Some(root) => Path::new(std::ffi::OsStr::from_bytes(root)),
        None => &config.default_zone_root,
    };
    if !root.is_absolute() {
        return Err(HostZoneError::InvalidConfiguration);
    }
    match file(path, Some(root), ZoneSelection::NamedFile) {
        Err(HostZoneError::Unavailable) if missing_rules => {
            Err(HostZoneError::Zone(ZoneError::MissingRules))
        }
        other => other,
    }
}
#[cfg(target_os = "linux")]
fn read_pinned_file(path: &Path, root: Option<&Path>) -> Result<Vec<u8>, HostZoneError> {
    use rustix::fs::{CWD, FileType, Mode, OFlags, ResolveFlags, fstat, open, openat2};
    use std::{io::Read, os::fd::AsRawFd};
    let pin = if let Some(root) = root {
        let root = openat2(
            CWD,
            root,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_MAGICLINKS,
        )
        .map_err(|_| HostZoneError::Unavailable)?;
        openat2(
            &root,
            path,
            OFlags::PATH | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::BENEATH | ResolveFlags::NO_MAGICLINKS,
        )
    } else {
        if !path.is_absolute() {
            return Err(HostZoneError::InvalidConfiguration);
        }
        openat2(
            CWD,
            path,
            OFlags::PATH | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_MAGICLINKS,
        )
    }
    .map_err(|_| HostZoneError::Unavailable)?;
    let before = fstat(&pin).map_err(|_| HostZoneError::Unavailable)?;
    if FileType::from_raw_mode(before.st_mode) != FileType::RegularFile {
        return Err(HostZoneError::UnsafeFile);
    }
    let n = usize::try_from(before.st_size).map_err(|_| HostZoneError::UnsafeFile)?;
    if n > MAX_ZONE_SOURCE_BYTES {
        return Err(HostZoneError::Zone(ZoneError::TooLarge));
    }
    let fd = open(
        format!("/proc/self/fd/{}", pin.as_raw_fd()),
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOCTTY,
        Mode::empty(),
    )
    .map_err(|_| HostZoneError::Unavailable)?;
    let opened = fstat(&fd).map_err(|_| HostZoneError::Unavailable)?;
    if !same_stat(&before, &opened) {
        return Err(HostZoneError::ChangedDuringRead);
    }
    let mut file = std::fs::File::from(fd);
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(n + 1)
        .map_err(|_| HostZoneError::Zone(ZoneError::Allocation))?;
    (&mut file)
        .take((n + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| HostZoneError::Unavailable)?;
    let after = fstat(&file).map_err(|_| HostZoneError::Unavailable)?;
    if bytes.len() != n || !same_stat(&before, &after) {
        return Err(HostZoneError::ChangedDuringRead);
    }
    Ok(bytes)
}
#[cfg(target_os = "linux")]
fn same_stat(a: &rustix::fs::Stat, b: &rustix::fs::Stat) -> bool {
    a.st_dev == b.st_dev
        && a.st_ino == b.st_ino
        && a.st_mode == b.st_mode
        && a.st_size == b.st_size
        && a.st_mtime == b.st_mtime
        && a.st_mtime_nsec == b.st_mtime_nsec
        && a.st_ctime == b.st_ctime
        && a.st_ctime_nsec == b.st_ctime_nsec
}
#[cfg(test)]
#[path = "date_host/tests.rs"]
mod tests;
