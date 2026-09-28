//! One cancellable pipe channel and one owned child process. Renderer,
//! resource broker and image decoders use this framing/deadline/reaping boundary.
use super::codec;
use std::{
    io::{self, Read, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};

pub(super) struct Channel {
    child: Child,
    input: ChildStdin,
    output: ChildStdout,
    failed: bool,
}
impl Channel {
    pub(super) fn spawn(executable: &Path, mode: &str) -> Result<Self, String> {
        let mut command = Command::new(executable);
        #[cfg(all(target_os = "linux", feature = "vulkan-presenter"))]
        command.arg("--clean-worker-launch");
        command
            .arg(mode)
            .env_clear()
            .current_dir("/")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self::from_child(command.spawn().map_err(|e| format!("start {mode}: {e}"))?)
    }
    fn from_child(mut child: Child) -> Result<Self, String> {
        let input = child.stdin.take().ok_or("child stdin unavailable")?;
        let output = child.stdout.take().ok_or("child stdout unavailable")?;
        if let Err(e) = nonblocking(&input).and_then(|()| nonblocking(&output)) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
        Ok(Self {
            child,
            input,
            output,
            failed: false,
        })
    }
    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }
    pub(super) fn terminate(&mut self) {
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
    pub(super) fn exchange(
        &mut self,
        bytes: Vec<u8>,
        timeout: Duration,
        cancel: impl Fn() -> bool,
        service: impl FnMut(&[u8]) -> Result<Option<Vec<u8>>, String>,
    ) -> Result<Vec<u8>, String> {
        self.exchange_bounded(bytes, timeout, codec::MAX_FRAME, cancel, service)
    }
    pub(super) fn exchange_bounded(
        &mut self,
        bytes: Vec<u8>,
        timeout: Duration,
        response_limit: usize,
        cancel: impl Fn() -> bool,
        mut service: impl FnMut(&[u8]) -> Result<Option<Vec<u8>>, String>,
    ) -> Result<Vec<u8>, String> {
        if self.failed {
            return Err("child process is unavailable".into());
        }
        let started = Instant::now();
        let mut request = frame(bytes)?;
        let mut written = 0;
        let mut header = [0u8; 4];
        let mut header_read = 0;
        let mut response = Vec::new();
        let mut received = 0;
        let mut calls = 0;
        let result = (|| loop {
            if cancel() || started.elapsed() > timeout {
                return Err("child process cancelled or exceeded its time budget".into());
            }
            if written < request.len() {
                match self.input.write(&request[written..]) {
                    Ok(0) => return Err("child process closed its input".into()),
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
                let target = if header_read < 4 {
                    &mut header[header_read..]
                } else {
                    &mut response[received..]
                };
                match self.output.read(target) {
                    Ok(0) => return Err("child process closed its output".into()),
                    Ok(n) => {
                        if header_read < 4 {
                            header_read += n;
                            if header_read == 4 {
                                let length = u32::from_le_bytes(header) as usize;
                                if !(5..=response_limit.min(codec::MAX_FRAME)).contains(&length) {
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
                                if let Some(reply) = service(&response)? {
                                    calls += 1;
                                    if calls > crate::net::MAX_RESOURCES {
                                        return Err("too many broker calls in a transaction".into());
                                    }
                                    request = frame(reply)?;
                                    written = 0;
                                    header_read = 0;
                                    received = 0;
                                    response = Vec::new();
                                } else {
                                    return Ok(response);
                                }
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
        })();
        if result.is_err() {
            self.terminate();
        }
        result
    }
}
impl Drop for Channel {
    fn drop(&mut self) {
        self.terminate();
    }
}
fn frame(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if bytes.len() > codec::MAX_FRAME {
        return Err("IPC frame exceeds limit".into());
    }
    let mut frame = (bytes.len() as u32).to_le_bytes().to_vec();
    frame.extend_from_slice(&bytes);
    Ok(frame)
}
#[cfg(target_os = "linux")]
fn nonblocking(fd: &impl std::os::fd::AsFd) -> Result<(), String> {
    let flags = rustix::fs::fcntl_getfl(fd).map_err(|e| e.to_string())?;
    rustix::fs::fcntl_setfl(fd, flags | rustix::fs::OFlags::NONBLOCK).map_err(|e| e.to_string())
}
#[cfg(not(target_os = "linux"))]
fn nonblocking<T>(_: &T) -> Result<(), String> {
    Err("native processes currently require Linux".into())
}
#[cfg(target_os = "linux")]
fn wait_pipe(fd: &impl std::os::fd::AsFd, writable: bool) -> Result<(), String> {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    let interest = if writable {
        PollFlags::OUT
    } else {
        PollFlags::IN
    };
    match poll(
        &mut [PollFd::new(fd, interest)],
        Some(&Timespec {
            tv_sec: 0,
            tv_nsec: 20_000_000,
        }),
    ) {
        Ok(_) | Err(rustix::io::Errno::INTR) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
#[cfg(not(target_os = "linux"))]
fn wait_pipe<T>(_: &T, _: bool) -> Result<(), String> {
    Err("native processes currently require Linux".into())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn response_budget_rejects_advertised_length_before_reading_payload() {
        use std::os::unix::process::CommandExt;
        let child = Command::new("/bin/sh")
            .args(["-c", r"printf '\145\000\000\000'; sleep 60"])
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut channel = Channel::from_child(child).unwrap();
        let pid = channel.pid();
        let error = channel
            .exchange_bounded(
                vec![0; 5],
                Duration::from_secs(1),
                100,
                || false,
                |_| Ok(None),
            )
            .unwrap_err();
        assert_eq!(error, "IPC frame length outside limit");
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
    }
    #[test]
    fn deadlines_cover_stalled_reads_and_full_input_pipes() {
        use std::os::unix::process::CommandExt;
        for length in [1, 1024 * 1024] {
            let child = Command::new("/bin/sh")
                .args(["-c", "sleep 60"])
                .process_group(0)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let mut channel = Channel::from_child(child).unwrap();
            let pid = channel.pid();
            let started = Instant::now();
            let error = channel
                .exchange(
                    vec![0; length],
                    Duration::from_millis(80),
                    || false,
                    |_| Ok(None),
                )
                .unwrap_err();
            assert!(error.contains("time budget"));
            assert!(started.elapsed() < Duration::from_secs(3));
            assert!(channel.failed);
            assert!(!Path::new(&format!("/proc/{pid}")).exists());
            channel.terminate();
        }
    }
}
