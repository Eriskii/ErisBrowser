//! Renderer, image decoder and broker process restrictions: Landlock, resource caps and a seccomp denylist.
//! This reduces the exposed kernel surface; it is not a complete syscall allowlist.
use std::path::Path;

/// Landlock cannot revoke already-open files or sockets. Refuse unexpected
/// inherited descriptors before accepting any page. This avoids closing raw
/// descriptors behind Rust's ownership model.
#[cfg(target_os = "linux")]
pub fn check_inherited_descriptors() -> Result<(), String> {
    let directory = std::path::PathBuf::from(format!("/proc/{}/fd", std::process::id()));
    let entries =
        std::fs::read_dir(&directory).map_err(|e| format!("inspect inherited descriptors: {e}"))?;
    let mut scanner_descriptors = 0;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let fd: u32 = entry
            .file_name()
            .to_str()
            .ok_or("invalid descriptor name")?
            .parse()
            .map_err(|_| "invalid descriptor number")?;
        if fd < 3 {
            continue;
        }
        let target = std::fs::read_link(entry.path()).map_err(|e| e.to_string())?;
        if target != directory {
            return Err("page worker inherited an unexpected open descriptor".into());
        }
        scanner_descriptors += 1;
    }
    if scanner_descriptors != 1 {
        return Err("unexpected inherited descriptor directory".into());
    }
    Ok(())
}
#[cfg(not(target_os = "linux"))]
pub fn check_inherited_descriptors() -> Result<(), String> {
    Err("page descriptor inspection currently requires Linux procfs".into())
}

#[cfg(target_os = "linux")]
pub fn restrict_broker(root: Option<&Path>) -> Result<(), String> {
    restrict_with_role(root, false)
}
#[cfg(target_os = "linux")]
pub fn restrict_renderer() -> Result<(), String> {
    restrict_with_role(None, true)
}
#[cfg(target_os = "linux")]
fn restrict_with_role(root: Option<&Path>, renderer: bool) -> Result<(), String> {
    use landlock::{
        ABI, Access, AccessFs, AccessNet, CompatLevel, Compatible, PathBeneath, PathFd, Ruleset,
        RulesetAttr, RulesetCreatedAttr, RulesetStatus, Scope,
    };
    use rustix::process::{Resource, Rlimit, getrlimit, setrlimit};
    let abi = ABI::V6;
    let mut rules = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| e.to_string())?
        .handle_access(if renderer {
            AccessNet::from_all(abi)
        } else {
            AccessNet::BindTcp.into()
        })
        .map_err(|e| e.to_string())?
        .scope(Scope::from_all(abi))
        .map_err(|e| e.to_string())?
        .create()
        .map_err(|e| format!("Landlock ABI 6 sandbox required: {e}"))?;
    // Shared libraries are already mapped. DNS may lazily load libc NSS modules;
    // only the system library trees and exact resolver files are readable.
    let mut paths: Vec<&Path> = [
        "/etc/resolv.conf",
        "/etc/hosts",
        "/etc/nsswitch.conf",
        "/etc/gai.conf",
        "/etc/host.conf",
        "/lib",
        "/lib64",
        "/usr/lib",
        "/nix/store",
    ]
    .iter()
    .map(Path::new)
    .collect();
    if renderer {
        paths.clear();
    } else if let Some(root) = root {
        paths.push(root);
    }
    for path in paths {
        if !path.exists() {
            continue;
        }
        let access = if path.is_dir() {
            AccessFs::ReadFile | AccessFs::ReadDir
        } else {
            AccessFs::ReadFile.into()
        };
        rules = rules
            .add_rule(PathBeneath::new(
                PathFd::new(path).map_err(|e| e.to_string())?,
                access,
            ))
            .map_err(|e| e.to_string())?;
    }
    let status = rules.restrict_self().map_err(|e| e.to_string())?;
    if status.ruleset != RulesetStatus::FullyEnforced || !status.no_new_privs {
        return Err("required page sandbox was not fully enforced".into());
    }
    for (resource, cap) in [
        (Resource::As, 1536 * 1024 * 1024),
        (Resource::Nofile, 128),
        (Resource::Core, 0),
    ] {
        let old = getrlimit(resource);
        let maximum = old.maximum.unwrap_or(u64::MAX).min(cap);
        setrlimit(
            resource,
            Rlimit {
                current: Some(maximum),
                maximum: Some(maximum),
            },
        )
        .map_err(|e| e.to_string())?;
    }
    install_syscall_filter(renderer)
}

#[cfg(not(target_os = "linux"))]
pub fn restrict_broker(_: Option<&Path>) -> Result<(), String> {
    Err("native page sandbox currently requires Linux with Landlock ABI 6".into())
}

#[cfg(not(target_os = "linux"))]
pub fn restrict_renderer() -> Result<(), String> {
    Err("native renderer sandbox requires Linux".into())
}

#[cfg(all(
    target_os = "linux",
    target_endian = "little",
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    )
))]
fn syscall_filter(renderer: bool) -> Result<seccompiler::BpfProgram, String> {
    use seccompiler::{
        SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter, SeccompRule,
    };
    use std::collections::BTreeMap;
    // All confined children run single-threaded. Reject process/thread creation, new
    // executables and namespace changes, including io_uring's alternative path
    // to socket operations. Only the broker retains sockets for DNS and HTTP.
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = [
        libc::SYS_clone,
        libc::SYS_clone3,
        libc::SYS_execve,
        libc::SYS_execveat,
        libc::SYS_unshare,
        libc::SYS_setns,
        libc::SYS_setsid,
        libc::SYS_setpgid,
        libc::SYS_io_uring_setup,
        libc::SYS_io_uring_enter,
        libc::SYS_io_uring_register,
        libc::SYS_ptrace,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_pidfd_open,
        libc::SYS_pidfd_getfd,
        libc::SYS_pidfd_send_signal,
        libc::SYS_process_madvise,
        libc::SYS_process_mrelease,
        libc::SYS_kcmp,
        libc::SYS_bpf,
        libc::SYS_perf_event_open,
        libc::SYS_userfaultfd,
        libc::SYS_add_key,
        libc::SYS_request_key,
        libc::SYS_keyctl,
        libc::SYS_mount,
        libc::SYS_umount2,
        libc::SYS_pivot_root,
        libc::SYS_chroot,
        libc::SYS_fsopen,
        libc::SYS_fsconfig,
        libc::SYS_fsmount,
        libc::SYS_fspick,
        libc::SYS_move_mount,
        libc::SYS_open_tree,
        libc::SYS_mount_setattr,
        libc::SYS_init_module,
        libc::SYS_finit_module,
        libc::SYS_delete_module,
        libc::SYS_kexec_load,
        libc::SYS_reboot,
        libc::SYS_swapon,
        libc::SYS_swapoff,
        libc::SYS_personality,
        libc::SYS_seccomp,
        // No page operation needs a new local IPC pair, regardless of family.
        libc::SYS_socketpair,
        // Landlock does not mediate System V IPC. No worker operation needs
        // shared memory, semaphores, or message queues shared with other apps.
        libc::SYS_shmget,
        libc::SYS_shmat,
        libc::SYS_shmdt,
        libc::SYS_shmctl,
        libc::SYS_msgget,
        libc::SYS_msgsnd,
        libc::SYS_msgrcv,
        libc::SYS_msgctl,
        libc::SYS_semget,
        libc::SYS_semop,
        libc::SYS_semtimedop,
        libc::SYS_semctl,
        libc::SYS_mq_open,
        libc::SYS_mq_unlink,
        libc::SYS_mq_timedsend,
        libc::SYS_mq_timedreceive,
        libc::SYS_mq_notify,
        libc::SYS_mq_getsetattr,
        // A same-user process may lower another process's limits/priority.
        libc::SYS_setpriority,
        libc::SYS_sched_setaffinity,
        libc::SYS_sched_setscheduler,
        libc::SYS_sched_setparam,
        libc::SYS_sched_setattr,
        // Landlock's read/write restrictions do not cover these metadata
        // mutations, including pathname operations outside readable roots.
        libc::SYS_fchmod,
        libc::SYS_fchmodat,
        libc::SYS_fchown,
        libc::SYS_fchownat,
        libc::SYS_utimensat,
        libc::SYS_setxattr,
        libc::SYS_lsetxattr,
        libc::SYS_fsetxattr,
        libc::SYS_removexattr,
        libc::SYS_lremovexattr,
        libc::SYS_fremovexattr,
    ]
    .into_iter()
    .map(|syscall| (syscall, Vec::new()))
    .collect();
    #[cfg(target_arch = "x86_64")]
    for syscall in [
        libc::SYS_fork,
        libc::SYS_vfork,
        libc::SYS_modify_ldt,
        libc::SYS_iopl,
        libc::SYS_ioperm,
        // Older pathname variants are absent on aarch64 and riscv64.
        libc::SYS_chmod,
        libc::SYS_chown,
        libc::SYS_lchown,
        libc::SYS_utime,
        libc::SYS_utimes,
        libc::SYS_futimesat,
    ] {
        rules.insert(syscall, Vec::new());
    }
    // libc does not yet expose every recent syscall on each supported target.
    // These numbers match Linux arch/x86/entry/syscalls/syscall_64.tbl and
    // scripts/syscall.tbl (the aarch64/riscv64 table). Keep this list in sync
    // when updating the policy; unsupported architectures fail closed below.
    for syscall in [
        452, // fchmodat2
        463, // setxattrat
        466, // removexattrat
        469, // file_setattr: filesystem flags formerly changed through ioctl
    ] {
        rules.insert(syscall, Vec::new());
    }
    let unix = SeccompCondition::new(
        0,
        SeccompCmpArgLen::Dword,
        SeccompCmpOp::Eq,
        libc::AF_UNIX as u64,
    )
    .map_err(|error| error.to_string())?;
    rules.insert(
        libc::SYS_socket,
        if renderer {
            Vec::new()
        } else {
            vec![SeccompRule::new(vec![unix]).map_err(|error| error.to_string())?]
        },
    );
    // ioctl can mutate filesystem metadata through read-only descriptors.
    // The worker only needs nonblocking mode and bytes-available queries.
    // Linux consumes request as an unsigned int, so ignore its upper 32 bits.
    let forbidden_ioctl = [libc::FIONBIO, libc::FIONREAD]
        .into_iter()
        .map(|request| {
            SeccompCondition::new(
                1,
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::Ne,
                u64::from(request as u32),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    rules.insert(
        libc::SYS_ioctl,
        vec![SeccompRule::new(forbidden_ioctl).map_err(|error| error.to_string())?],
    );
    // Querying limits is harmless and used by diagnostics. Reject a non-null
    // new_limit pointer, including requests targeting another same-user task.
    rules.insert(
        libc::SYS_prlimit64,
        vec![
            SeccompRule::new(vec![
                SeccompCondition::new(2, SeccompCmpArgLen::Qword, SeccompCmpOp::Ne, 0)
                    .map_err(|error| error.to_string())?,
            ])
            .map_err(|error| error.to_string())?,
        ],
    );
    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        std::env::consts::ARCH
            .try_into()
            .map_err(|error: seccompiler::BackendError| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let program: seccompiler::BpfProgram = filter
        .try_into()
        .map_err(|error: seccompiler::BackendError| error.to_string())?;
    #[cfg(target_arch = "x86_64")]
    {
        // seccompiler validates AUDIT_ARCH, but x32 shares the x86_64 audit
        // architecture and marks syscall numbers with bit 30. Reject that ABI
        // before the native denylist so flagged syscall numbers cannot bypass it.
        // The seccomp_data.nr field is the first 32 bits of the kernel ABI.
        let mut guarded = vec![
            seccompiler::sock_filter {
                code: (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16,
                jt: 0,
                jf: 0,
                k: 0,
            },
            seccompiler::sock_filter {
                code: (libc::BPF_JMP | libc::BPF_JGE | libc::BPF_K) as u16,
                jt: 0,
                jf: 1,
                k: 0x4000_0000,
            },
            seccompiler::sock_filter {
                code: (libc::BPF_RET | libc::BPF_K) as u16,
                jt: 0,
                jf: 0,
                k: libc::SECCOMP_RET_ERRNO | libc::EPERM as u32,
            },
        ];
        guarded.extend(program);
        Ok(guarded)
    }
    #[cfg(not(target_arch = "x86_64"))]
    Ok(program)
}

#[cfg(all(
    target_os = "linux",
    target_endian = "little",
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    )
))]
fn install_syscall_filter(renderer: bool) -> Result<(), String> {
    // The kernel copies the generated program; the safe wrapper installs it on
    // this thread before the worker loads any page or creates any other thread.
    let program = syscall_filter(renderer)?;
    seccompiler::apply_filter(&program)
        .map_err(|error| format!("required page syscall filter: {error}"))
}

#[cfg(all(
    target_os = "linux",
    not(all(
        target_endian = "little",
        any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )
    ))
))]
fn install_syscall_filter(_renderer: bool) -> Result<(), String> {
    Err("page syscall filtering requires little-endian x86_64, aarch64 or riscv64".into())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires Linux Landlock ABI 6; probes renderer access in a separate process"]
    fn renderer_has_no_direct_file_or_socket_access() {
        const CHILD: &str = "ERIS_RENDERER_SANDBOX_PROBE";
        if let Some(path) = std::env::var_os(CHILD) {
            let path = std::path::PathBuf::from(path);
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "document");
            let resolver = std::fs::read("/etc/resolv.conf").is_ok();
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            drop(std::net::TcpStream::connect(address).expect("TCP client preflight"));
            drop(listener);
            drop(std::net::UdpSocket::bind("127.0.0.1:0").expect("UDP preflight"));
            drop(std::os::unix::net::UnixStream::pair().expect("Unix IPC preflight"));
            restrict_renderer().expect("renderer confinement fully enforced");
            assert_eq!(
                std::fs::read(&path).unwrap_err().kind(),
                std::io::ErrorKind::PermissionDenied
            );
            assert!(std::fs::read_dir(path.parent().unwrap()).is_err());
            assert!(std::fs::write(&path, "forbidden").is_err());
            if resolver {
                assert_eq!(
                    std::fs::read("/etc/resolv.conf").unwrap_err().kind(),
                    std::io::ErrorKind::PermissionDenied
                );
            }
            assert_eq!(
                std::net::TcpStream::connect(address)
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                std::net::UdpSocket::bind("127.0.0.1:0")
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                std::os::unix::net::UnixStream::pair()
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            return;
        }
        let directory =
            std::env::temp_dir().join(format!("eris-renderer-probe-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("page.html");
        std::fs::write(&path, "document").unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "worker::sandbox::tests::renderer_has_no_direct_file_or_socket_access",
                "--include-ignored",
                "--nocapture",
            ])
            .env(CHILD, &path)
            .output()
            .unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "document");
        std::fs::remove_dir_all(directory).unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn renderer_filter_denies_every_socket_family() {
        let program = syscall_filter(true).unwrap();
        for family in [
            libc::AF_UNIX,
            libc::AF_INET,
            libc::AF_INET6,
            libc::AF_NETLINK,
            libc::AF_PACKET,
            0,
        ] {
            for argument in [family as u64, (family as u64) | (1u64 << 32)] {
                assert_eq!(
                    evaluate(&program, libc::SYS_socket as u32, 0xc000_003e, argument),
                    libc::SECCOMP_RET_ERRNO | libc::EPERM as u32
                );
            }
        }
    }
    #[cfg(target_arch = "x86_64")]
    #[test]
    fn both_roles_deny_system_ipc_and_mutating_other_process_limits() {
        let denied = libc::SECCOMP_RET_ERRNO | libc::EPERM as u32;
        for renderer in [false, true] {
            let program = syscall_filter(renderer).unwrap();
            for syscall in [
                libc::SYS_shmget,
                libc::SYS_shmat,
                libc::SYS_shmdt,
                libc::SYS_shmctl,
                libc::SYS_msgget,
                libc::SYS_msgsnd,
                libc::SYS_msgrcv,
                libc::SYS_msgctl,
                libc::SYS_semget,
                libc::SYS_semop,
                libc::SYS_semtimedop,
                libc::SYS_semctl,
                libc::SYS_mq_open,
                libc::SYS_mq_unlink,
                libc::SYS_mq_timedsend,
                libc::SYS_mq_timedreceive,
                libc::SYS_mq_notify,
                libc::SYS_mq_getsetattr,
                libc::SYS_setpriority,
                libc::SYS_sched_setaffinity,
                libc::SYS_sched_setscheduler,
                libc::SYS_sched_setparam,
                libc::SYS_sched_setattr,
            ] {
                assert_eq!(
                    evaluate(&program, syscall as u32, 0xc000_003e, 0),
                    denied,
                    "syscall {syscall}, renderer={renderer}"
                );
            }
            for new_limit in [0, 1, 1 << 32, u64::MAX] {
                assert_eq!(
                    evaluate_three_arguments(
                        &program,
                        libc::SYS_prlimit64 as u32,
                        0xc000_003e,
                        12345,
                        0,
                        new_limit
                    ),
                    if new_limit == 0 {
                        libc::SECCOMP_RET_ALLOW
                    } else {
                        denied
                    }
                );
            }
        }
    }
    #[test]
    #[ignore = "requires Linux Landlock ABI 6; launches a separately confined test process"]
    fn filesystem_restrictions_in_a_separate_process() {
        const CHILD: &str = "ERIS_SANDBOX_TEST_ROOT";
        if let Some(root) = std::env::var_os(CHILD) {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let root = std::path::PathBuf::from(root);
            let outside = root.parent().unwrap().join("secret");
            assert_eq!(std::fs::read_to_string(&outside).unwrap(), "outside");
            std::fs::set_permissions(&outside, std::fs::Permissions::from_mode(0o640))
                .expect("metadata mutation preflight");
            let outside_file = std::fs::File::open(&outside).unwrap();
            outside_file
                .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(123_456_789))
                .expect("timestamp mutation preflight");
            let original_metadata = outside_file.metadata().unwrap();
            // Establish that these facilities work before confinement, so an
            // outer test harness restriction cannot produce a false pass.
            drop(std::net::TcpListener::bind("127.0.0.1:0").expect("TCP preflight"));
            drop(std::os::unix::net::UnixStream::pair().expect("Unix socket preflight"));
            std::thread::Builder::new()
                .spawn(|| ())
                .expect("thread preflight")
                .join()
                .unwrap();
            let socket_path = root.join("service.sock");
            let listener = std::os::unix::net::UnixListener::bind(&socket_path)
                .expect("Unix listener preflight");
            drop(
                std::os::unix::net::UnixStream::connect(&socket_path)
                    .expect("Unix connection preflight"),
            );
            restrict_broker(Some(&root)).expect("sandbox must be fully enforced");
            assert_eq!(
                std::fs::read_to_string(root.join("page.html")).unwrap(),
                "allowed"
            );
            assert!(std::fs::read_to_string(&outside).is_err());
            assert!(std::fs::read_to_string(root.join("escape")).is_err());
            assert!(std::fs::write(root.join("created"), "no").is_err());
            assert!(std::fs::write(root.join("page.html"), "no").is_err());
            assert!(std::fs::remove_file(root.join("page.html")).is_err());
            assert_eq!(
                std::fs::set_permissions(&outside, std::fs::Permissions::from_mode(0o777))
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                outside_file
                    .set_permissions(std::fs::Permissions::from_mode(0o777))
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                outside_file
                    .set_modified(std::time::UNIX_EPOCH)
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                rustix::fs::fchown(&outside_file, None, None),
                Err(rustix::io::Errno::PERM)
            );
            assert_eq!(
                rustix::fs::setxattr(
                    &outside,
                    "user.eris-sandbox-probe",
                    b"forbidden",
                    rustix::fs::XattrFlags::empty(),
                ),
                Err(rustix::io::Errno::PERM)
            );
            assert_eq!(
                rustix::fs::removexattr(&outside, "user.eris-sandbox-probe"),
                Err(rustix::io::Errno::PERM)
            );
            assert_eq!(
                rustix::fs::ioctl_setflags(&outside_file, rustix::fs::IFlags::empty()),
                Err(rustix::io::Errno::PERM)
            );
            let after = outside_file.metadata().unwrap();
            assert_eq!(after.mode(), original_metadata.mode());
            assert_eq!(after.uid(), original_metadata.uid());
            assert_eq!(after.gid(), original_metadata.gid());
            assert_eq!(
                after.modified().unwrap(),
                original_metadata.modified().unwrap()
            );
            assert_eq!(after.ctime(), original_metadata.ctime());
            assert_eq!(after.ctime_nsec(), original_metadata.ctime_nsec());
            assert!(
                std::process::Command::new("/bin/sh")
                    .arg("-c")
                    .arg("true")
                    .status()
                    .is_err()
            );
            assert!(std::net::TcpListener::bind("127.0.0.1:0").is_err());
            assert_eq!(
                std::os::unix::net::UnixStream::connect(&socket_path)
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                std::os::unix::net::UnixStream::pair()
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            assert_eq!(
                std::thread::Builder::new()
                    .spawn(|| ())
                    .unwrap_err()
                    .raw_os_error(),
                Some(libc::EPERM)
            );
            // DNS needs a UDP socket and its automatically selected local port.
            let udp =
                std::net::UdpSocket::bind("127.0.0.1:0").expect("UDP remains available for DNS");
            rustix::io::ioctl_fionbio(&udp, true)
                .expect("nonblocking socket ioctl remains allowed");
            assert_eq!(rustix::io::ioctl_fionread(&udp).unwrap(), 0);
            drop(listener);
            assert_eq!(
                rustix::process::getrlimit(rustix::process::Resource::Core).maximum,
                Some(0)
            );
            return;
        }
        let temporary =
            std::env::temp_dir().join(format!("eris-sandbox-probe-{}", std::process::id()));
        let root = temporary.join("document");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("page.html"), "allowed").unwrap();
        std::fs::write(temporary.join("secret"), "outside").unwrap();
        std::os::unix::fs::symlink(temporary.join("secret"), root.join("escape")).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "worker::sandbox::tests::filesystem_restrictions_in_a_separate_process",
                "--include-ignored",
                "--nocapture",
            ])
            .env(CHILD, &root)
            .output()
            .unwrap();
        std::fs::remove_dir_all(temporary).unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn filter_checks_native_architecture_x32_and_truncated_socket_arguments() {
        let program = syscall_filter(false).unwrap();
        let native = 0xc000_003e; // AUDIT_ARCH_X86_64, Linux audit UAPI.
        let denied = libc::SECCOMP_RET_ERRNO | libc::EPERM as u32;
        for syscall in [
            libc::SYS_clone,
            libc::SYS_clone3,
            libc::SYS_fork,
            libc::SYS_vfork,
            libc::SYS_execve,
            libc::SYS_execveat,
            libc::SYS_io_uring_setup,
            libc::SYS_io_uring_enter,
            libc::SYS_io_uring_register,
            libc::SYS_ptrace,
            libc::SYS_process_vm_readv,
            libc::SYS_pidfd_getfd,
            libc::SYS_bpf,
            libc::SYS_perf_event_open,
            libc::SYS_socketpair,
        ] {
            assert_eq!(
                evaluate(&program, syscall as u32, native, 0),
                denied,
                "syscall {syscall}"
            );
        }
        for domain in [libc::AF_UNIX as u64, (1u64 << 32) | libc::AF_UNIX as u64] {
            assert_eq!(
                evaluate(&program, libc::SYS_socket as u32, native, domain),
                denied
            );
        }
        for domain in [libc::AF_INET, libc::AF_INET6, libc::AF_NETLINK] {
            assert_eq!(
                evaluate(&program, libc::SYS_socket as u32, native, domain as u64),
                libc::SECCOMP_RET_ALLOW
            );
        }
        assert_eq!(
            evaluate(&program, libc::SYS_read as u32, native, 0),
            libc::SECCOMP_RET_ALLOW
        );
        assert_eq!(
            evaluate(&program, libc::SYS_read as u32, 0x4000_0003, 0),
            libc::SECCOMP_RET_KILL_PROCESS
        );
        for syscall in [
            libc::SYS_read as u32,
            libc::SYS_socket as u32,
            libc::SYS_clone as u32,
        ] {
            assert_eq!(
                evaluate(
                    &program,
                    0x4000_0000 | syscall,
                    native,
                    libc::AF_INET as u64
                ),
                denied
            );
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn filter_blocks_metadata_mutation_and_limits_ioctl_requests() {
        let program = syscall_filter(false).unwrap();
        let native = 0xc000_003e;
        let denied = libc::SECCOMP_RET_ERRNO | libc::EPERM as u32;
        for syscall in [
            libc::SYS_chmod,
            libc::SYS_fchmod,
            libc::SYS_fchmodat,
            libc::SYS_fchmodat2,
            libc::SYS_chown,
            libc::SYS_fchown,
            libc::SYS_lchown,
            libc::SYS_fchownat,
            libc::SYS_utime,
            libc::SYS_utimes,
            libc::SYS_futimesat,
            libc::SYS_utimensat,
            libc::SYS_setxattr,
            libc::SYS_lsetxattr,
            libc::SYS_fsetxattr,
            libc::SYS_removexattr,
            libc::SYS_lremovexattr,
            libc::SYS_fremovexattr,
            463, // setxattrat
            466, // removexattrat
            469, // file_setattr
        ] {
            assert_eq!(
                evaluate(&program, syscall as u32, native, 0),
                denied,
                "metadata syscall {syscall}"
            );
        }
        for request in [libc::FIONBIO, libc::FIONREAD] {
            let request = u64::from(request as u32);
            for value in [request, request | (1u64 << 32)] {
                assert_eq!(
                    evaluate_arguments(&program, libc::SYS_ioctl as u32, native, 42, value),
                    libc::SECCOMP_RET_ALLOW,
                    "allowed ioctl {value:#x}"
                );
            }
        }
        for request in [
            u64::from(libc::FS_IOC_SETFLAGS as u32),
            u64::from(libc::FIOASYNC as u32),
            u64::from(libc::TIOCSTI as u32),
            0,
            u64::MAX,
        ] {
            for value in [request, request | (1u64 << 32)] {
                // Put an allowed request value in arg0 to check that the rule
                // examines arg1 (the request), not the descriptor argument.
                assert_eq!(
                    evaluate_arguments(
                        &program,
                        libc::SYS_ioctl as u32,
                        native,
                        u64::from(libc::FIONBIO as u32),
                        value,
                    ),
                    denied,
                    "forbidden ioctl {value:#x}"
                );
            }
        }
    }

    /// Interpret the small classic-BPF instruction subset emitted by these
    /// rules, independently checking ABI/argument edge cases without raw unsafe
    /// syscalls. Separate child probes above exercise the real kernel filter.
    #[cfg(target_arch = "x86_64")]
    fn evaluate(
        program: &[seccompiler::sock_filter],
        syscall: u32,
        architecture: u32,
        arg0: u64,
    ) -> u32 {
        evaluate_arguments(program, syscall, architecture, arg0, 0)
    }

    #[cfg(target_arch = "x86_64")]
    fn evaluate_arguments(
        program: &[seccompiler::sock_filter],
        syscall: u32,
        architecture: u32,
        arg0: u64,
        arg1: u64,
    ) -> u32 {
        evaluate_three_arguments(program, syscall, architecture, arg0, arg1, 0)
    }
    #[cfg(target_arch = "x86_64")]
    fn evaluate_three_arguments(
        program: &[seccompiler::sock_filter],
        syscall: u32,
        architecture: u32,
        arg0: u64,
        arg1: u64,
        arg2: u64,
    ) -> u32 {
        let mut data = [0u8; 64];
        data[..4].copy_from_slice(&syscall.to_le_bytes());
        data[4..8].copy_from_slice(&architecture.to_le_bytes());
        data[16..24].copy_from_slice(&arg0.to_le_bytes());
        data[24..32].copy_from_slice(&arg1.to_le_bytes());
        data[32..40].copy_from_slice(&arg2.to_le_bytes());
        let mut accumulator = 0;
        let mut pc = 0;
        for _ in 0..program.len() {
            let instruction = &program[pc];
            let code = u32::from(instruction.code);
            if code == libc::BPF_LD | libc::BPF_W | libc::BPF_ABS {
                accumulator = u32::from_le_bytes(
                    data[instruction.k as usize..instruction.k as usize + 4]
                        .try_into()
                        .unwrap(),
                );
            } else if code == libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K {
                pc += if accumulator == instruction.k {
                    instruction.jt
                } else {
                    instruction.jf
                } as usize;
            } else if code == libc::BPF_JMP | libc::BPF_JGE | libc::BPF_K {
                pc += if accumulator >= instruction.k {
                    instruction.jt
                } else {
                    instruction.jf
                } as usize;
            } else if code == libc::BPF_JMP | libc::BPF_JA {
                pc += instruction.k as usize;
            } else if code == libc::BPF_RET | libc::BPF_K {
                return instruction.k;
            } else {
                panic!("unexpected generated BPF instruction {code:#x}");
            }
            pc += 1;
        }
        panic!("filter did not return");
    }
}
