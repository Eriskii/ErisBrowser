//! An exec-only stage, entered before UI, graphics or page initialization.
//! Driver descriptors live in the parent. Mark the child's copied non-stdio
//! descriptors close-on-exec, then replace this image with the requested worker.
//! The worker still performs its independent fail-closed descriptor inspection.
use std::{convert::Infallible, os::unix::process::CommandExt, process::Command};

pub fn launch_worker(role: &str) -> Result<Infallible, String> {
    if !matches!(
        role,
        "--page-worker" | "--resource-broker" | "--image-decoder" | "--timezone-discovery"
    ) {
        return Err("invalid clean worker role".into());
    }
    // No driver is initialized in this stage. Refuse an unexpected runtime
    // thread instead of assuming descriptor iteration is atomic with it.
    let threads = std::fs::read_dir("/proc/self/task")
        .map_err(|e| format!("inspect launch threads: {e}"))?
        .take(2)
        .try_fold(0, |count, entry| entry.map(|_| count + 1))
        .map_err(|e| format!("inspect launch thread: {e}"))?;
    if threads != 1 {
        return Err("clean worker launch requires one thread".into());
    }
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    // This safe API changes flags only; it does not close an FD behind a Rust
    // owner's back. Linux uses close_range(CLOEXEC) with an iteration fallback.
    // It returns no per-FD result: the worker's existing inspection after exec
    // is essential and rejects any descriptor that survived this operation.
    close_fds::set_fds_cloexec(3, &[]);
    let error = Command::new(executable)
        .arg(role)
        .env_clear()
        .current_dir("/")
        .exec();
    Err(format!("exec clean worker: {error}"))
}
