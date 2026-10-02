#!/usr/bin/env python3
"""Isolated Linux subreaper for one checker.

Only validate_listener_fds is imported by hosts; supervise runs in a fresh process.

Only this process waits for its children. A PID is signalled only while a
non-reaping waitid proves it is our child. No process lookup after reap is used
as evidence of ownership. Cleanup failure is an explicit failed result, not a
claim that SIGKILL always completes within a deadline.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import math
import os
from pathlib import Path
import re
import selectors
import signal
import socket
import subprocess
import sys
import time

MAX_CHILDREN = 256
MAX_REAPS = 4096
PROC_BYTES = 4096
READ_BYTES = 16384
OUTPUT_LIMIT = 2 * 1024 * 1024
CLEANUP_SECONDS = 5.0
PR_SET_CHILD_SUBREAPER = 36
PR_GET_CHILD_SUBREAPER = 37
WAIT_FLAGS = os.WEXITED | os.WNOHANG | os.WNOWAIT
CAPTURE = re.compile(rb'CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=([1-9][0-9]?) owned_children=0')
TEXT_CAPTURE = re.compile(rb'TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=([1-9][0-9]?) owned_children=0')


def enable_subreaper() -> None:
    if sys.platform != "linux" or len(list(Path('/proc/self/task').iterdir())) != 1:
        raise RuntimeError("a fresh single-threaded Linux supervisor is required")
    if signal.getsignal(signal.SIGCHLD) != signal.SIG_DFL:
        raise RuntimeError("SIGCHLD must use the default non-autoreaping disposition")
    libc = ctypes.CDLL(None, use_errno=True)
    libc.prctl.restype = ctypes.c_int
    if libc.prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "PR_SET_CHILD_SUBREAPER")
    enabled = ctypes.c_int()
    if libc.prctl(PR_GET_CHILD_SUBREAPER, ctypes.byref(enabled), 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "PR_GET_CHILD_SUBREAPER")
    if enabled.value != 1:
        raise RuntimeError("subreaper setting was not retained")
    if own_children():
        raise RuntimeError("supervisor must start without children")


def own_children() -> list[int]:
    # This process remains single-threaded; no global /proc PID sweep.
    with open(f'/proc/self/task/{os.getpid()}/children', 'rb') as source:
        data = source.read(PROC_BYTES + 1)
    if len(data) > PROC_BYTES:
        raise RuntimeError("own-child list exceeds its byte bound")
    fields = data.split()
    if len(fields) > MAX_CHILDREN or any(not p.isdigit() for p in fields):
        raise RuntimeError("own-child list exceeds its count bound or is invalid")
    children = [int(p) for p in fields]
    if len(set(children)) != len(children) or any(p <= 1 for p in children):
        raise RuntimeError("invalid own-child identity")
    return children


def observe(pid: int):
    # None means a live child, not absence. ECHILD is never ignored here.
    return os.waitid(os.P_PID, pid, WAIT_FLAGS)


def signal_owned(pid: int) -> None:
    observe(pid)
    # No reaper or SIGCHLD handler runs between this proof and these signals.
    # An exiting child remains a zombie, reserving its numeric PID.
    try:
        if os.getpgid(pid) == pid:
            os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass  # An unreaped zombie may no longer have a process group.
    try:
        os.kill(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def cleanup(checker_pid: int, drain, seconds: float) -> dict:
    deadline = time.monotonic() + seconds
    reaped = 0
    descendants = 0
    checker_exit = None
    checker_reaped = False
    # Do not retain numeric PIDs for later signalling after reaping them.
    try:
        # The known unreaped checker can be stopped even if bounded procfs
        # enumeration subsequently fails. Ownership is still checked by waitid.
        signal_owned(checker_pid)
        while time.monotonic() < deadline:
            children = own_children()
            if not children:
                try:
                    os.waitid(os.P_ALL, 0, WAIT_FLAGS)
                except ChildProcessError:
                    return dict(complete=True, reaped=reaped, descendants=descendants,
                                checker_returncode=checker_exit, error=None)
                raise RuntimeError("empty proc child list disagrees with waitid")
            for pid in children:
                signal_owned(pid)
            for pid in children:
                state = observe(pid)
                if state is None:
                    continue
                result = os.waitid(os.P_PID, pid, os.WEXITED | os.WNOHANG)
                if result is None or result.si_pid != pid:
                    raise RuntimeError("child ownership changed during sole-waiter reap")
                reaped += 1
                if not checker_reaped and pid == checker_pid:
                    checker_reaped = True
                    checker_exit = (result.si_status if result.si_code == os.CLD_EXITED
                                    else -result.si_status)
                else:
                    descendants += 1
                if reaped > MAX_REAPS:
                    raise RuntimeError("cleanup reap bound exceeded")
            drain(0.01)
        raise TimeoutError("descendant cleanup deadline expired")
    except (OSError, RuntimeError, TimeoutError) as exc:
        return dict(complete=False, reaped=reaped, descendants=descendants,
                    checker_returncode=checker_exit,
                    error=f'{type(exc).__name__}: {exc}')


def validate_listener_fds(pass_fds: tuple[int, ...]) -> None:
    """Forward at most one owned loopback listening socket, never arbitrary fds."""
    if type(pass_fds) is not tuple or len(pass_fds) > 1:
        raise ValueError('at most one listener descriptor may be inherited')
    for fd in pass_fds:
        if type(fd) is not int or fd < 3:
            raise ValueError('listener descriptor must exclude standard streams')
        stream = socket.socket(fileno=fd)
        try:
            address = stream.getsockname()
            if (stream.family != socket.AF_INET
                    or stream.getsockopt(socket.SOL_SOCKET, socket.SO_TYPE) != socket.SOCK_STREAM
                    or stream.getsockopt(socket.SOL_SOCKET, socket.SO_PROTOCOL) != socket.IPPROTO_TCP
                    or stream.getsockopt(socket.SOL_SOCKET, socket.SO_ACCEPTCONN) != 1
                    or address[0] != '127.0.0.1' or not 1 <= address[1] <= 65535):
                raise ValueError('inherited descriptor must be a loopback TCP listener')
        finally:
            stream.detach()


def supervise(command: list[str], timeout: float, output_limit: int,
              cleanup_seconds: float = CLEANUP_SECONDS,
              capture_gate: bool = False,
              worker_text_gate: bool = False,
              pass_fds: tuple[int, ...] = ()) -> tuple[dict, bytes, bytes]:
    validate_listener_fds(pass_fds)
    if capture_gate and worker_text_gate:
        raise ValueError('capture gate modes are mutually exclusive')
    gate_pattern = TEXT_CAPTURE if worker_text_gate else CAPTURE
    capture_gate = capture_gate or worker_text_gate
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 120:
        raise ValueError("timeout must be finite and in 0.05..120 seconds")
    if not 128 <= output_limit <= 4 * 1024 * 1024:
        raise ValueError("output limit must be in 128 bytes..4 MiB")
    if not math.isfinite(cleanup_seconds) or not 0.05 <= cleanup_seconds <= 10:
        raise ValueError("cleanup deadline must be finite and in 0.05..10 seconds")
    enable_subreaper()
    cancelled = []

    def cancel(signum, _frame):
        if not cancelled:
            cancelled.append(signum)

    for signum in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(signum, cancel)
    started = time.monotonic()
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    observed = retained = 0
    overflow = False
    gate_granted = False
    gate_allowed = True
    parsed_stdout = 0
    poller = selectors.DefaultSelector()
    proc = None
    status = 'error'
    error = None
    cleanup_result = dict(complete=True, reaped=0, descendants=0,
                          checker_returncode=None, error=None)

    def drain(wait):
        nonlocal observed, retained, overflow, gate_granted, parsed_stdout
        for key, _ in poller.select(wait):
            try:
                chunk = os.read(key.fd, READ_BYTES)
            except BlockingIOError:
                continue
            if not chunk:
                poller.unregister(key.fileobj)
                continue
            observed += len(chunk)
            size = min(len(chunk), output_limit - retained)
            buffers[key.data].extend(chunk[:size])
            retained += size
            overflow |= size != len(chunk)
            if capture_gate and gate_allowed and not overflow and key.data == 'stdout':
                data = buffers['stdout']
                while (end := data.find(b'\n', parsed_stdout)) != -1:
                    line = bytes(data[parsed_stdout:end])
                    parsed_stdout = end + 1
                    if line.startswith((b'CAPTURE_COMPLETE', b'TEXT_CAPTURE_COMPLETE')):
                        match = gate_pattern.fullmatch(line)
                        if gate_granted or not match or int(match[1]) > 64:
                            raise RuntimeError('invalid or repeated capture gate record')
                        # Checker has already dropped every worker, checked all
                        # its own task child lists, then blocked on our stdin
                        # grant. Reparenting on child exit occurs before reap;
                        # adopted survivors therefore appear here before grant.
                        if own_children() != [proc.pid]:
                            raise RuntimeError('adopted descendants prevent Vulkan grant')
                        if (observe(proc.pid) is not None or cancelled or
                                time.monotonic() - started >= timeout):
                            raise RuntimeError('checker exited, cancelled or timed out before grant')
                        proc.stdin.write(b'GPU_READY\n')
                        proc.stdin.flush()
                        proc.stdin.close()
                        gate_granted = True
                if len(data) - parsed_stdout > 150_000:
                    raise RuntimeError('capture protocol line exceeds bound')

    try:
        proc = subprocess.Popen(command, stdin=subprocess.PIPE if capture_gate else subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                close_fds=True, start_new_session=True, pass_fds=pass_fds)
        for stream, label in ((proc.stdout, 'stdout'), (proc.stderr, 'stderr')):
            os.set_blocking(stream.fileno(), False)
            poller.register(stream, selectors.EVENT_READ, label)
        while True:
            if cancelled:
                status = 'cancelled'
                break
            if overflow:
                status = 'output_limit'
                break
            if observe(proc.pid) is not None:
                status = 'exited'
                break
            remaining = timeout - (time.monotonic() - started)
            if remaining <= 0:
                status = 'timeout'
                break
            drain(min(0.05, remaining))
    except (OSError, RuntimeError, ValueError) as exc:
        error = f'{type(exc).__name__}: {exc}'
    finally:
        gate_allowed = False
        if proc is not None:
            # No poll(), wait(), context-manager exit or destructor before cleanup.
            cleanup_result = cleanup(proc.pid, drain, cleanup_seconds)
            # Mark Popen as accounted for, preventing its destructor from becoming
            # an additional waiter. On failure the receipt explicitly says that
            # cleanup remains unresolved; returning does not assert process death.
            proc.returncode = cleanup_result['checker_returncode']
            if proc.returncode is None:
                proc.returncode = -signal.SIGKILL
            if cleanup_result['complete']:
                # Reaping all descendants closes inherited pipe writers. Drain the
                # remaining bounded pipe content; an output cap is still a failure.
                for _ in range(512):
                    if not poller.get_map():
                        break
                    drain(0)
                if poller.get_map():
                    cleanup_result['complete'] = False
                    cleanup_result['error'] = 'pipes remain open after all children reaped'
            for stream in (proc.stdin, proc.stdout, proc.stderr):
                if stream is not None:
                    stream.close()
        poller.close()
    if not cleanup_result['complete']:
        status = 'cleanup_error'
    elif cancelled:
        status = 'cancelled'
    elif status == 'exited' and cleanup_result['descendants']:
        status = 'surviving_descendants'
    elif overflow:
        status = 'output_limit'
    elif status == 'exited' and capture_gate and not gate_granted:
        status = 'missing_capture_gate'
    stdout, stderr = bytes(buffers['stdout']), bytes(buffers['stderr'])
    record = dict(status=status, returncode=cleanup_result['checker_returncode'],
                  command=command, timeout_seconds=timeout,
                  cleanup_seconds=cleanup_seconds, cleanup=cleanup_result,
                  elapsed_seconds=time.monotonic() - started,
                  captured_bytes=retained, observed_bytes=observed,
                  output_limit_bytes=output_limit, cancellation_signal=cancelled or None,
                  capture_gate_required=capture_gate, capture_gate_granted=gate_granted,
                  stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                  stderr_sha256=hashlib.sha256(stderr).hexdigest(), error=error)
    return record, stdout, stderr


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=60)
    parser.add_argument('--output-limit', type=int, default=OUTPUT_LIMIT)
    gates = parser.add_mutually_exclusive_group()
    gates.add_argument('--capture-gate', action='store_true')
    gates.add_argument('--worker-text-gate', action='store_true')
    parser.add_argument('--result', type=Path, required=True)
    parser.add_argument('--pass-listener-fd', type=int, action='append', default=[])
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('a checker command is required')
    try:
        record, stdout, stderr = supervise(command, args.timeout, args.output_limit,
                                          capture_gate=args.capture_gate,
                                          worker_text_gate=args.worker_text_gate,
                                          pass_fds=tuple(args.pass_listener_fd))
    except (OSError, RuntimeError, ValueError) as exc:
        record, stdout, stderr = dict(status='setup_error', error=str(exc)), b'', b''
    args.result.parent.mkdir(parents=True, exist_ok=True)
    args.result.with_suffix('.stdout.log').write_bytes(stdout)
    args.result.with_suffix('.stderr.log').write_bytes(stderr)
    args.result.write_text(json.dumps(record, indent=2) + '\n')
    return 0 if record['status'] == 'exited' and record['returncode'] == 0 else 1


if __name__ == '__main__':
    raise SystemExit(main())
