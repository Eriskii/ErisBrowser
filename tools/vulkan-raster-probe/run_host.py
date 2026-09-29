#!/usr/bin/env python3
"""Run the standalone custom Vulkan raster prototype with bounded POSIX child-process lifetimes."""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import hashlib
import math
import os
from pathlib import Path
import re
import selectors
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
DEFAULT_TIMEOUT = 25.0
OUTPUT_LIMIT = 256 * 1024
MAX_ADAPTERS = 16
TAIL_BYTES = 4096
_QUOTED = r'"(?:[^"\\\r\n]|\\.)*"'
ADAPTER = re.compile(
    rf"ADAPTER (0|[1-9][0-9]*) vendor=0x[0-9a-f]+ device=0x[0-9a-f]+ "
    rf"type=(?:Other|IntegratedGpu|DiscreteGpu|VirtualGpu|Cpu) backend=Vulkan "
    rf"name={_QUOTED} driver={_QUOTED} info={_QUOTED}"
)
# Independent protocol expectations: name,width,height,draws,invocations,buffers,bytes.
EXPECTED_PASSES = [
    ("overlapping-order",8,6,5,320,1664,192),
    ("fractional-clips",8,6,4,256,1408,192),
    ("fixed-resets-and-restores",10,7,6,448,2096,280),
    ("out-of-bounds-and-empty",7,5,3,192,1048,140),
    ("319x239-dispatch-edges",319,239,4,156160,610952,304964),
    ("forty-ordered-full-writes",320,240,41,3148800,624896,307200),
    ("gpu-clear-only",7,3,1,64,424,84),
    ("image-upscale-2x2",6,6,2,128,848,144),
    ("image-three-columns-to-five",7,3,2,128,716,84),
    ("image-downscale-distinct-4x4",4,4,2,128,720,64),
    ("image-clip-preserves-original-origin",8,3,2,128,736,96),
    ("image-negative-document-offset",5,3,2,128,680,60),
    ("image-negative-fractional-origin",4,4,2,128,680,64),
    ("image-tiny-and-fractional-clip",6,4,3,192,1044,96),
    ("image-exact-and-one-ulp-translated-edges",5,3,3,192,928,60),
    ("image-f32-endpoint-addition-rounds-back",4,2,2,128,592,32),
    ("image-rectangle-order-and-source-reuse",6,4,5,320,1568,96),
    ("image-fixed-escape-and-nested-restoration",8,5,6,384,1956,160),
    ("image-dispatch-tail-and-clear-border",320,240,2,153600,617160,307200),
    ("image-valid-empty-and-hidden",4,3,1,64,352,48),
    ("image-subnormal-origin-and-lost-extent",2,2,2,128,564,16),
    ('alpha-rectangle-six-boundaries', 6, 2, 12, 768, 3168, 48),
    ('alpha-image-six-boundaries', 6, 2, 3, 192, 944, 48),
    ('alpha-ordered-mixed-draws', 4, 1, 10, 640, 2644, 16),
    ('alpha-one-repeated-rounding', 7, 1, 65, 4160, 16696, 28),
    ('alpha-subpixel-and-fractional-clip', 4, 3, 4, 256, 1132, 48),
    ('alpha-image-negative-fractional-original-origin', 5, 3, 2, 128, 672, 60),
    ('alpha-fixed-reset-and-restoration', 5, 3, 6, 384, 1680, 60),
    ('alpha-transparent-hidden-rgb', 2, 2, 2, 128, 576, 16),
    ('alpha-empty-clip-still-clears-opaque-target', 3, 2, 1, 64, 304, 24),
]
def expected_lines():
    return [f"PASS {name} {w}x{h} draws={draws} invocations={work} gpu_buffers={buffers} compared_bytes={size} exact=true"
            for name,w,h,draws,work,buffers,size in EXPECTED_PASSES]



@dataclass
class RunResult:
    record: dict
    stdout: bytes
    stderr: bytes


def child_environment(loader_directory: Path | None) -> dict[str, str]:
    env = os.environ.copy()
    if loader_directory is not None:
        if not sys.platform.startswith("linux"):
            raise ValueError("--loader-directory is supported only on Linux")
        if not loader_directory.is_dir():
            raise ValueError("loader directory does not exist or is not a directory")
        value = str(loader_directory.resolve())
        if env.get("LD_LIBRARY_PATH"):
            value += ":" + env["LD_LIBRARY_PATH"]
        env["LD_LIBRARY_PATH"] = value
    return env


def _stop_group(proc: subprocess.Popen) -> str | None:
    """Also kill descendants whose original parent exited successfully."""
    error = None
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    except OSError as exc:
        error = f"process-group cleanup: {type(exc).__name__}: {exc}"
    try:
        proc.wait(timeout=2.0)
    except (OSError, subprocess.TimeoutExpired) as exc:
        error = f"child reap: {type(exc).__name__}: {exc}"
    return error


def run_process(
    binary: Path,
    args: list[str],
    name: str,
    output_dir: Path,
    env: dict[str, str],
    timeout: float = DEFAULT_TIMEOUT,
    output_limit: int = OUTPUT_LIMIT,
) -> RunResult:
    if not sys.platform.startswith("linux") or not hasattr(os, "WNOWAIT"):
        raise RuntimeError("Linux waitid WNOWAIT is required to retain leader ownership until group cleanup")
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 120:
        raise ValueError("timeout must be finite and between 0.05 and 120 seconds")
    if not 128 <= output_limit <= 4 * 1024 * 1024:
        raise ValueError("output limit must be between 128 bytes and 4 MiB")
    if not re.fullmatch(r"[a-z0-9-]+", name):
        raise ValueError("invalid result name")
    started = time.monotonic()
    proc = None
    poller = None
    buffers = {"stdout": bytearray(), "stderr": bytearray()}
    total = 0
    observed = 0
    status = "error"
    error = None
    try:
        proc = subprocess.Popen(
            [str(binary), *args],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            close_fds=True,
            start_new_session=True,
        )
        poller = selectors.DefaultSelector()
        for stream, label in [(proc.stdout, "stdout"), (proc.stderr, "stderr")]:
            os.set_blocking(stream.fileno(), False)
            poller.register(stream, selectors.EVENT_READ, label)
        while True:
            remaining = timeout - (time.monotonic() - started)
            if remaining <= 0:
                status = "timeout"
                break
            # Observe exit without reaping. The leader retains its PID/group
            # identity until _stop_group sends SIGKILL before proc.wait().
            exited = os.waitid(os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT) if not poller.get_map() else None
            if exited is not None:
                status = "exited"
                break
            for key, _ in poller.select(min(remaining, 0.05)):
                try:
                    chunk = os.read(key.fd, 16 * 1024)
                except BlockingIOError:
                    continue
                if not chunk:
                    poller.unregister(key.fileobj)
                    continue
                observed += len(chunk)
                retained = min(len(chunk), output_limit - total)
                buffers[key.data].extend(chunk[:retained])
                total += retained
                if retained != len(chunk):
                    status = "output_limit"
                    break
            if status == "output_limit":
                break
    except KeyboardInterrupt:
        status = "interrupted"
        error = "KeyboardInterrupt"
    except Exception as exc:
        status = "error"
        error = f"{type(exc).__name__}: {exc}"
    finally:
        if proc is not None:
            cleanup = _stop_group(proc)
            if cleanup:
                status = "cleanup_error"
                error = cleanup
            for stream in (proc.stdout, proc.stderr):
                if stream is not None:
                    stream.close()
        if poller is not None:
            poller.close()
    stdout, stderr = bytes(buffers["stdout"]), bytes(buffers["stderr"])
    stdout_log, stderr_log = f"{name}.stdout.log", f"{name}.stderr.log"
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / stdout_log).write_bytes(stdout)
    (output_dir / stderr_log).write_bytes(stderr)
    record = {
        "name": name,
        "args": args,
        "status": status,
        "returncode": proc.returncode if proc else None,
        "timeout_seconds": timeout,
        "elapsed_seconds": round(time.monotonic() - started, 6),
        "captured_bytes": total,
        "observed_bytes": observed,
        "output_limit_bytes": output_limit,
        "stdout_log": stdout_log,
        "stderr_log": stderr_log,
        "stdout_tail": stdout[-TAIL_BYTES:].decode("utf-8", errors="replace"),
        "stderr_tail": stderr[-TAIL_BYTES:].decode("utf-8", errors="replace"),
    }
    if error:
        record["error"] = error[:TAIL_BYTES]
    return RunResult(record, stdout, stderr)


def protocol_lines(result: RunResult) -> list[str]:
    if result.record["status"] != "exited" or result.record["returncode"] != 0:
        raise ValueError("process did not exit successfully within its limits")
    try:
        return [line for line in result.stdout.decode("utf-8").splitlines() if line]
    except UnicodeDecodeError as exc:
        raise ValueError("protocol output is not UTF-8") from exc


def inventory(lines: list[str]) -> list[str]:
    records = []
    seen = set()
    for line in lines:
        if not line.startswith("ADAPTER"):
            continue
        if len(line) > 4096:
            raise ValueError("adapter inventory record exceeds limits")
        match = ADAPTER.fullmatch(line)
        if not match:
            raise ValueError("malformed adapter inventory record")
        # A record cannot contain arbitrarily large numeric or identity fields.
        if len(match[1]) > 2:
            raise ValueError("adapter inventory record exceeds limits")
        index = int(match[1])
        if index in seen:
            raise ValueError("duplicate adapter index")
        seen.add(index)
        records.append(line)
        if len(records) > MAX_ADAPTERS:
            raise ValueError("adapter inventory exceeds limit")
    if not records or seen != set(range(len(records))):
        raise ValueError("inventory must contain 1 to 16 contiguous unique adapter indices")
    if [int(ADAPTER.fullmatch(line)[1]) for line in records] != list(range(len(records))):
        raise ValueError("adapter inventory is out of order")
    return records


def validate_listing(result: RunResult) -> list[str]:
    lines = protocol_lines(result)
    devices = inventory(lines)
    if lines != devices:
        raise ValueError("unexpected enumeration output")
    return devices


def validate_adapter(result: RunResult, expected: list[str], index: int) -> None:
    lines = protocol_lines(result)
    if inventory(lines) != expected:
        raise ValueError("adapter inventory changed between processes")
    if lines[:len(expected)] != expected:
        raise ValueError("inventory records must precede result records")
    wanted = expected_lines() + [f"COMPLETE adapter={index} fixtures={len(EXPECTED_PASSES)} exact=true custom_wgsl=true"]
    if lines[len(expected):] != wanted:
        raise ValueError("missing, duplicate, reordered or mismatched exact raster results")


def _write_summary(output_dir: Path, summary: dict) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    temp = output_dir / "host-results.json.tmp"
    temp.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    temp.replace(output_dir / "host-results.json")


def run_host(binary: Path, output_dir: Path, loader_directory: Path | None = None,
             timeout: float = DEFAULT_TIMEOUT, output_limit: int = OUTPUT_LIMIT) -> dict:
    if output_dir.exists():
        raise ValueError("output directory must be new; evidence is never overwritten")
    if not binary.is_file() or binary.stat().st_size > 128 * 1024 * 1024:
        raise ValueError("binary must be a regular file no larger than 128 MiB")
    with binary.open("rb") as stream:
        binary_hash = hashlib.file_digest(stream, "sha256").hexdigest()
    summary = {
        "schema": 1,
        "binary_sha256": binary_hash,
        "scope": "offscreen custom WGSL source-over rectangle and nearest-neighbor image prototype; no native surface or performance claim",
        "binary": os.path.relpath(binary.resolve(), ROOT),
        "loader_override": loader_directory is not None,
        "timeout_seconds": timeout,
        "success": False,
        "adapter_count": 0,
        "runs": [],
    }
    try:
        env = child_environment(loader_directory)
        listing = run_process(binary.resolve(), ["--list"], "enumeration", output_dir,
                              env, timeout, output_limit)
        summary["runs"].append(listing.record)
        devices = validate_listing(listing)
        summary["adapter_count"] = len(devices)
        _write_summary(output_dir, summary)
        failed = False
        for index in range(len(devices)):
            result = run_process(binary.resolve(), ["--adapter", str(index)],
                                 f"adapter-{index}", output_dir, env, timeout, output_limit)
            summary["runs"].append(result.record)
            try:
                validate_adapter(result, devices, index)
                result.record["validation"] = "passed"
            except ValueError as exc:
                result.record["validation"] = "failed"
                result.record["validation_error"] = str(exc)
                failed = True
            _write_summary(output_dir, summary)
            if result.record["status"] in {"interrupted", "cleanup_error"}:
                break
        summary["success"] = not failed and len(summary["runs"]) == len(devices) + 1
    except KeyboardInterrupt:
        summary["error"] = "KeyboardInterrupt"
    except (OSError, ValueError, RuntimeError) as exc:
        summary["error"] = f"{type(exc).__name__}: {exc}"[:TAIL_BYTES]
    finally:
        _write_summary(output_dir, summary)
    return summary


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/eris-vulkan-raster-prototype")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "output")
    parser.add_argument("--loader-directory", type=Path)
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT,
                        help="per-process deadline, 0.05 to 120 seconds (default: 25)")
    parser.add_argument("--allow-experimental-gpu", action="store_true", help="explicit opt-in; this performs actual Vulkan execution")
    args = parser.parse_args(argv)
    if not args.allow_experimental_gpu:
        parser.error("actual Vulkan execution requires --allow-experimental-gpu")
    if not sys.platform.startswith("linux") or not hasattr(os, "WNOWAIT"):
        parser.error("Linux waitid WNOWAIT is required")
    if not math.isfinite(args.timeout) or not 0.05 <= args.timeout <= 120:
        parser.error("--timeout must be finite and between 0.05 and 120 seconds")
    try:
        summary = run_host(args.binary, args.output_dir, args.loader_directory, args.timeout)
    except OSError as exc:
        parser.exit(1, f"could not write bounded probe evidence: {exc}\n")
    print(json.dumps({"success": summary["success"], "adapter_count": summary["adapter_count"],
                      "results": str(args.output_dir / "host-results.json")}, separators=(",", ":")))
    if summary.get("error") == "KeyboardInterrupt" or any(r["status"] == "interrupted" for r in summary["runs"]):
        return 130
    return 0 if summary["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
