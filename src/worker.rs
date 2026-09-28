//! Isolated page execution and a bounded, validated binary IPC boundary.
mod codec;
mod sandbox;
use crate::{
    dom::{Document, NodeId},
    graphics::{Fonts, ImageStore},
    layout::LayoutResult,
    net,
    page::{Navigation, Page},
};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command as ProcessCommand, Stdio},
    time::{Duration, Instant},
};
use url::Url;

pub struct Snapshot {
    pub generation: u64,
    pub processed_edit_sequence: u64,
    pub layout: LayoutResult,
    pub images: ImageStore,
    pub document: Document,
    pub title: String,
    pub url: String,
    pub diagnostics: Vec<String>,
    pub load_ms: f64,
}
#[derive(Debug)]
pub enum Command {
    Load {
        navigation: Navigation,
    },
    Click {
        node: NodeId,
    },
    Edit {
        sequence: u64,
        node: NodeId,
        value: String,
    },
    Fragment {
        address: String,
    },
    Render {
        width: f32,
        height: f32,
    },
}
pub struct Reply {
    pub snapshot: Option<Snapshot>,
    pub navigation: Option<Navigation>,
}
impl Reply {
    fn empty() -> Self {
        Self {
            snapshot: None,
            navigation: None,
        }
    }
}
struct Init {
    scripts: bool,
    generation: u64,
    root: Option<PathBuf>,
}

pub struct WorkerClient {
    child: Child,
    input: ChildStdin,
    output: ChildStdout,
    generation: u64,
    root: Option<PathBuf>,
    initial: Url,
    loaded: bool,
    failed: bool,
}
impl WorkerClient {
    pub fn spawn(scripts: bool, navigation: &Navigation, generation: u64) -> Result<Self, String> {
        Self::spawn_at(
            &std::env::current_exe().map_err(|e| e.to_string())?,
            scripts,
            navigation,
            generation,
        )
    }
    /// Explicit executable path also allows integration tests to launch the real binary.
    pub fn spawn_at(
        executable: &Path,
        scripts: bool,
        navigation: &Navigation,
        generation: u64,
    ) -> Result<Self, String> {
        let initial = net::parse_address(&navigation.address)?;
        validate_address(&initial)?;
        let root = if initial.scheme() == "file" {
            let path = initial
                .to_file_path()
                .map_err(|_| "invalid local document")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.is_file() {
                return Err("document must be a regular file".into());
            }
            Some(
                path.parent()
                    .ok_or("document has no parent directory")?
                    .to_owned(),
            )
        } else {
            None
        };
        let mut process = ProcessCommand::new(executable);
        process
            .arg("--page-worker")
            .env_clear()
            .current_dir("/")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            process.process_group(0);
        }
        let mut child = process
            .spawn()
            .map_err(|e| format!("start page worker: {e}"))?;
        let input = child.stdin.take().ok_or("worker stdin unavailable")?;
        let output = child.stdout.take().ok_or("worker stdout unavailable")?;
        if let Err(error) = nonblocking(&input).and_then(|()| nonblocking(&output)) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        let mut client = Self {
            child,
            input,
            output,
            generation,
            root: root.clone(),
            initial,
            loaded: false,
            failed: false,
        };
        let bytes = codec::encode_init(&Init {
            scripts,
            generation,
            root,
        })?;
        let response = client.transact(bytes, Duration::from_secs(5), || false)?;
        let reply = codec::decode_reply(&response)?;
        if reply.snapshot.is_some() || reply.navigation.is_some() {
            return Err("unexpected page worker startup reply".into());
        }
        Ok(client)
    }
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
    fn terminate(&mut self) {
        if self.failed {
            return;
        }
        self.failed = true;
        #[cfg(target_os = "linux")]
        if let Some(pid) = rustix::process::Pid::from_raw(self.child.id() as i32) {
            let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
    fn transact(
        &mut self,
        bytes: Vec<u8>,
        timeout: Duration,
        cancel: impl Fn() -> bool,
    ) -> Result<Vec<u8>, String> {
        if self.failed {
            return Err("page worker is unavailable".into());
        }
        let started = Instant::now();
        let mut request = (bytes.len() as u32).to_le_bytes().to_vec();
        request.extend_from_slice(&bytes);
        let mut written = 0;
        let mut header = [0u8; 4];
        let mut header_read = 0;
        let mut response = Vec::new();
        let mut received = 0;
        let result = (|| {
            loop {
                if cancel() || started.elapsed() > timeout {
                    return Err("page worker cancelled or exceeded its time budget".to_owned());
                }
                if written < request.len() {
                    match self.input.write(&request[written..]) {
                        Ok(0) => return Err("page worker closed its input".into()),
                        Ok(n) => {
                            written += n;
                            continue;
                        }
                        Err(e)
                            if matches!(
                                e.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                            ) => {}
                        Err(e) => return Err(e.to_string()),
                    }
                } else {
                    let target: &mut [u8] = if header_read < 4 {
                        &mut header[header_read..]
                    } else {
                        &mut response[received..]
                    };
                    match self.output.read(target) {
                        Ok(0) => return Err("page worker closed its output".into()),
                        Ok(n) => {
                            if header_read < 4 {
                                header_read += n;
                                if header_read == 4 {
                                    let length = u32::from_le_bytes(header) as usize;
                                    if !(5..=codec::MAX_FRAME).contains(&length) {
                                        return Err("IPC frame length outside limit".into());
                                    }
                                    response
                                        .try_reserve_exact(length)
                                        .map_err(|_| "IPC allocation failed")?;
                                    response.resize(length, 0);
                                }
                            } else {
                                received += n;
                                if received == response.len() {
                                    return Ok(response);
                                }
                            }
                            continue;
                        }
                        Err(e)
                            if matches!(
                                e.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                            ) => {}
                        Err(e) => return Err(e.to_string()),
                    }
                }
                if written < request.len() {
                    wait_pipe(&self.input, true)?;
                } else {
                    wait_pipe(&self.output, false)?;
                }
            }
        })();
        if result.is_err() {
            self.terminate();
        }
        result
    }
    pub fn exchange(
        &mut self,
        mut command: Command,
        cancel: impl Fn() -> bool,
    ) -> Result<Reply, String> {
        let timeout = if let Command::Load { navigation } = &mut command {
            if self.loaded {
                return Err("each document load requires a new page worker".into());
            }
            let url = net::parse_address(&navigation.address)?;
            if url != self.initial {
                return Err("worker load differs from authorized document".into());
            }
            navigation.address = url.into();
            self.loaded = true;
            Duration::from_secs(40)
        } else {
            Duration::from_secs(15)
        };
        let bytes = codec::encode_command(&command)?;
        let reply = self
            .transact(bytes, timeout, cancel)
            .and_then(|bytes| codec::decode_reply(&bytes));
        let reply = match reply {
            Ok(reply) => reply,
            Err(error) => {
                self.terminate();
                return Err(error);
            }
        };
        let validation = (|| {
            if let Some(snapshot) = &reply.snapshot {
                if snapshot.generation != self.generation
                    || !matches!(command, Command::Render { .. })
                {
                    return Err("unexpected page snapshot".to_owned());
                }
                self.validate_destination(&snapshot.url, true)?;
                if snapshot.url != "eris:error" {
                    let committed = Url::parse(&snapshot.url).map_err(|e| e.to_string())?;
                    if matches!(self.initial.scheme(), "http" | "https") {
                        if committed.origin() != self.initial.origin() {
                            return Err(
                                "cross-origin redirect requires parent-brokered response metadata"
                                    .into(),
                            );
                        }
                    } else {
                        let mut initial = self.initial.clone();
                        initial.set_fragment(None);
                        let mut committed = committed;
                        committed.set_fragment(None);
                        if committed != initial {
                            return Err("worker changed the authorized document URL".into());
                        }
                    }
                }
            }
            if let Some(navigation) = &reply.navigation {
                if !matches!(command, Command::Click { .. }) {
                    return Err("unexpected page navigation".to_owned());
                }
                self.validate_destination(&navigation.address, false)?;
            }
            Ok(())
        })();
        if let Err(error) = validation {
            self.terminate();
            return Err(error);
        }
        Ok(reply)
    }
    fn validate_destination(&self, address: &str, allow_error: bool) -> Result<(), String> {
        let url = Url::parse(address).map_err(|e| e.to_string())?;
        if allow_error && url.as_str() == "eris:error" {
            return Ok(());
        }
        validate_address(&url)?;
        if url.scheme() == "file" {
            let root = self
                .root
                .as_ref()
                .ok_or("remote worker requested a local file")?;
            let path = url
                .to_file_path()
                .map_err(|_| "invalid file URL")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.starts_with(root) || !path.is_file() {
                return Err("worker navigation escaped its file scope".into());
            }
        }
        if self.initial.scheme() == "https" && url.scheme() == "http" {
            return Err("worker requested an HTTPS downgrade".into());
        }
        Ok(())
    }
}
impl Drop for WorkerClient {
    fn drop(&mut self) {
        self.terminate();
    }
}
fn validate_address(url: &Url) -> Result<(), String> {
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URL credentials are not allowed".into());
    }
    match url.scheme() {
        "http" | "https" | "file" | "data" => Ok(()),
        "about" if url.path() == "blank" => Ok(()),
        "eris" if url.path() == "home" => Ok(()),
        _ => Err("unsupported worker navigation scheme".into()),
    }
}

/// Internal entry point: this executes before any native UI initialization.
pub fn serve() -> Result<(), String> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let init = codec::decode_init(&codec::read_frame(&mut input, codec::MAX_REQUEST)?)?;
    if let Err(error) = sandbox::check_inherited_descriptors()
        .and_then(|()| sandbox::restrict(init.root.as_deref()))
    {
        codec::write_frame(&mut output, &codec::encode_error(&error)?)?;
        return Err(error);
    }
    net::use_synchronous_dns_for_page_process();
    codec::write_frame(&mut output, &codec::encode_reply(&Reply::empty())?)?;
    let fonts = Fonts::new();
    let mut page: Option<Page> = None;
    let mut processed_edit_sequence = 0;
    loop {
        let bytes = codec::read_frame(&mut input, codec::MAX_REQUEST)?;
        let command = codec::decode_command(&bytes)?;
        let mut reply = Reply::empty();
        match command {
            Command::Load { navigation } => {
                if page.is_some() {
                    return Err("duplicate worker load".into());
                }
                page = Some(
                    Page::load_navigation(&navigation, init.scripts)
                        .unwrap_or_else(|error| Page::error(&navigation.address, &error)),
                );
            }
            Command::Click { node } => {
                reply.navigation = page.as_mut().ok_or("no page")?.click(node)
            }
            Command::Edit {
                sequence,
                node,
                value,
            } => {
                if sequence > processed_edit_sequence {
                    apply_edit(page.as_mut().ok_or("no page")?, node, &value);
                    processed_edit_sequence = sequence;
                }
            }
            Command::Fragment { address } => {
                let page = page.as_mut().ok_or("no page")?;
                if let Ok(target) = Url::parse(&address) {
                    let mut before = page.url.clone();
                    before.set_fragment(None);
                    let mut after = target.clone();
                    after.set_fragment(None);
                    if before == after {
                        page.url = target;
                    }
                }
            }
            Command::Render { width, height } => {
                if !width.is_finite()
                    || !height.is_finite()
                    || !(1.0..=8192.0).contains(&width)
                    || !(1.0..=8192.0).contains(&height)
                {
                    return Err("invalid viewport".into());
                }
                let page = page.as_mut().ok_or("no page")?;
                page.diagnostics.truncate(255);
                let mut diagnostics: Vec<String> = page
                    .diagnostics
                    .iter()
                    .map(|s| s.chars().take(2048).collect())
                    .collect();
                diagnostics.push(format!(
                    "Page process {}: Landlock ABI 6 and seccomp syscall restrictions",
                    std::process::id()
                ));
                reply.snapshot = Some(Snapshot {
                    generation: init.generation,
                    processed_edit_sequence,
                    layout: page.layout(width, height, &fonts),
                    images: page.images.clone(),
                    document: page.document.clone(),
                    title: page.title().chars().take(512).collect(),
                    url: page.url.to_string(),
                    diagnostics,
                    load_ms: page.load_ms,
                });
            }
        }
        codec::write_frame(&mut output, &codec::encode_reply(&reply)?)?;
    }
}
pub fn apply_edit(page: &mut Page, node: NodeId, value: &str) {
    if value.len() > 65_536 || !page.can_edit_control(node) {
        return;
    }
    if page.document.tag(node) == Some("textarea") {
        page.document.set_text_content(node, value);
    } else {
        page.document.set_attr(node, "value", value);
    }
    if page.scripts_enabled
        && let Err(error) = page
            .runtime
            .dispatch_event(node, "input", &mut page.document)
    {
        page.diagnostics.push(format!("input: {error}"));
    }
}

#[cfg(target_os = "linux")]
fn nonblocking(fd: &impl std::os::fd::AsFd) -> Result<(), String> {
    let flags = rustix::fs::fcntl_getfl(fd).map_err(|e| e.to_string())?;
    rustix::fs::fcntl_setfl(fd, flags | rustix::fs::OFlags::NONBLOCK).map_err(|e| e.to_string())
}
#[cfg(not(target_os = "linux"))]
fn nonblocking<T>(_: &T) -> Result<(), String> {
    Err("native page process currently requires Linux".into())
}

#[cfg(target_os = "linux")]
fn wait_pipe(fd: &impl std::os::fd::AsFd, writable: bool) -> Result<(), String> {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    let interest = if writable {
        PollFlags::OUT
    } else {
        PollFlags::IN
    };
    let timeout = Timespec {
        tv_sec: 0,
        tv_nsec: 20_000_000,
    };
    match poll(&mut [PollFd::new(fd, interest)], Some(&timeout)) {
        Ok(_) | Err(rustix::io::Errno::INTR) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
#[cfg(not(target_os = "linux"))]
fn wait_pipe<T>(_: &T, _: bool) -> Result<(), String> {
    Err("native page process currently requires Linux".into())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    fn unresponsive_child() -> WorkerClient {
        use std::os::unix::process::CommandExt;
        let mut child = ProcessCommand::new("/bin/sh")
            .args(["-c", "sleep 60"])
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        nonblocking(&input).unwrap();
        nonblocking(&output).unwrap();
        WorkerClient {
            child,
            input,
            output,
            generation: 0,
            root: None,
            initial: Url::parse("about:blank").unwrap(),
            loaded: true,
            failed: false,
        }
    }
    #[test]
    fn worker_deadlines_cover_stalled_reads_and_full_input_pipes() {
        for length in [1, 1024 * 1024] {
            let mut client = unresponsive_child();
            let pid = client.pid();
            let started = Instant::now();
            let error = client
                .transact(vec![0; length], Duration::from_millis(80), || false)
                .unwrap_err();
            assert!(error.contains("time budget"));
            assert!(started.elapsed() < Duration::from_secs(3));
            assert!(client.failed);
            assert!(!Path::new(&format!("/proc/{pid}")).exists());
            // Already reaped; cleanup must not signal the numerical PID again.
            client.terminate();
        }
    }
    #[test]
    fn worker_navigation_cannot_gain_files_or_active_schemes() {
        let mut client = unresponsive_child();
        client.initial = Url::parse("https://example.com/").unwrap();
        for destination in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "http://example.com/",
            "https://user:password@example.com/",
        ] {
            assert!(client.validate_destination(destination, false).is_err());
        }
        assert!(
            client
                .validate_destination("https://other.example/", false)
                .is_ok()
        );
        assert!(client.validate_destination("eris:error", true).is_ok());
        assert!(client.validate_destination("eris:error", false).is_err());
    }
}
