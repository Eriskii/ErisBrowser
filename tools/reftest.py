#!/usr/bin/env python3
"""Exact pixel comparisons for the self-authored Eris reference fixtures.

This is intentionally not a WPT runner or a claim of web standards conformance.
Only Python's standard library and the Eris binary are required.
"""
from __future__ import annotations

import argparse
import binascii
import json
from pathlib import Path
import struct
import subprocess
import sys
import time
import zlib

ROOT = Path(__file__).resolve().parents[1]
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def decode_png(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if not data.startswith(PNG_SIGNATURE):
        raise ValueError("invalid PNG signature")
    cursor = len(PNG_SIGNATURE)
    width = height = channels = 0
    compressed = bytearray()
    ended = False
    while cursor + 12 <= len(data):
        length = struct.unpack_from(">I", data, cursor)[0]
        kind = data[cursor + 4:cursor + 8]
        start = cursor + 8
        end = start + length
        if end + 4 > len(data):
            raise ValueError("truncated PNG chunk")
        chunk = data[start:end]
        expected_crc = struct.unpack_from(">I", data, end)[0]
        if binascii.crc32(kind + chunk) & 0xFFFFFFFF != expected_crc:
            raise ValueError("PNG chunk checksum mismatch")
        if kind == b"IHDR":
            if len(chunk) != 13 or width:
                raise ValueError("invalid PNG header")
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", chunk)
            if depth != 8 or color not in (2, 6) or compression or filtering or interlace:
                raise ValueError("expected a noninterlaced RGB/RGBA 8-bit PNG")
            if not width or not height or width * height > 16_777_216:
                raise ValueError("PNG dimensions exceed decoder limit")
            channels = 3 if color == 2 else 4
        elif kind == b"IDAT":
            compressed.extend(chunk)
        elif kind == b"IEND":
            ended = True
            break
        cursor = end + 4
    if not ended or not width or not compressed:
        raise ValueError("PNG missing required chunks")
    stride = width * channels
    expected_size = height * (stride + 1)
    decoder = zlib.decompressobj()
    raw = decoder.decompress(compressed, expected_size + 1)
    if len(raw) != expected_size or not decoder.eof:
        raise ValueError("PNG decompressed size mismatch")
    rgba = bytearray()
    previous = bytearray(stride)
    cursor = 0
    for _ in range(height):
        filter_kind = raw[cursor]
        row = bytearray(raw[cursor + 1:cursor + stride + 1])
        cursor += stride + 1
        for x in range(stride):
            left = row[x - channels] if x >= channels else 0
            above = previous[x]
            upper_left = previous[x - channels] if x >= channels else 0
            if filter_kind == 0:
                predictor = 0
            elif filter_kind == 1:
                predictor = left
            elif filter_kind == 2:
                predictor = above
            elif filter_kind == 3:
                predictor = (left + above) // 2
            elif filter_kind == 4:
                base = left + above - upper_left
                dl, da, du = abs(base - left), abs(base - above), abs(base - upper_left)
                predictor = left if dl <= da and dl <= du else above if da <= du else upper_left
            else:
                raise ValueError(f"invalid PNG filter {filter_kind}")
            row[x] = (row[x] + predictor) & 255
        if channels == 4:
            rgba.extend(row)
        else:
            for x in range(0, stride, 3):
                rgba.extend(row[x:x + 3])
                rgba.append(255)
        previous = row
    return width, height, bytes(rgba)


def encode_difference(path: Path, width: int, height: int, pixels: bytes) -> None:
    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF)
    raw = b"".join(b"\0" + pixels[y * width * 4:(y + 1) * width * 4] for y in range(height))
    path.write_bytes(PNG_SIGNATURE + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def render(binary: Path, fixture: Path, output: Path, width: int, height: int, timeout: float) -> None:
    command = [str(binary), "--render", str(fixture.resolve()), "--output", str(output), "--width", str(width), "--height", str(height), "--no-scripts"]
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    if result.returncode:
        raise RuntimeError(f"render exited {result.returncode}: {(result.stderr or result.stdout)[-4000:]}")
    if not output.is_file():
        raise RuntimeError("renderer reported success without producing the PNG")
    if "[page]" in result.stderr:
        raise RuntimeError(f"page reported a diagnostic: {result.stderr[-4000:]}")


def run_case(binary: Path, base: Path, directory: Path, case: dict, width: int, height: int, timeout: float) -> dict:
    name = case["name"]
    if not isinstance(name, str) or not name or any(c not in "abcdefghijklmnopqrstuvwxyz0123456789-_" for c in name):
        raise ValueError("case names must contain only lowercase letters, digits, hyphens or underscores")
    test_png = directory / f"{name}.png"
    reference_png = directory / f"{name}-ref.png"
    render(binary, base / case["test"], test_png, width, height, timeout)
    render(binary, base / case["reference"], reference_png, width, height, timeout)
    tw, th, actual = decode_png(test_png)
    rw, rh, reference = decode_png(reference_png)
    if (tw, th) != (width, height) or (rw, rh) != (width, height):
        raise ValueError(f"unexpected output dimensions: test={tw}x{th}, reference={rw}x{rh}")
    different = 0
    first_difference = None
    maximum_error = 0
    diff = bytearray()
    for offset in range(0, len(actual), 4):
        a, b = actual[offset:offset + 4], reference[offset:offset + 4]
        if a != b:
            different += 1
            if first_difference is None:
                index = offset // 4
                first_difference = dict(x=index % width, y=index // width, actual=list(a), reference=list(b))
            maximum_error = max(maximum_error, max(abs(x - y) for x, y in zip(a, b)))
            diff.extend(b"\xff\0\0\xff")
        else:
            diff.extend(b"\xff\xff\xff\xff")
    result = dict(name=name, status="pass" if not different else "fail", different_pixels=different, maximum_channel_error=maximum_error, test_image=str(test_png), reference_image=str(reference_png))
    if different:
        diff_path = directory / f"{name}-diff.png"
        encode_difference(diff_path, width, height, diff)
        result.update(first_difference=first_difference, difference_image=str(diff_path))
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/eris-browser")
    parser.add_argument("--manifest", type=Path, default=ROOT / "tests/reftests/manifest.json")
    parser.add_argument("--output-dir", type=Path, default=ROOT / "target/reftests")
    parser.add_argument("--timeout", type=float, default=15.0)
    args = parser.parse_args()
    args.output_dir = args.output_dir.resolve()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    try:
        manifest = json.loads(args.manifest.read_text())
        cases = manifest["cases"]
        if not cases:
            raise ValueError("manifest contains no cases")
        width, height = int(manifest["width"]), int(manifest["height"])
        results = []
        for case in cases:
            case_started = time.monotonic()
            try:
                result = run_case(args.binary.resolve(), args.manifest.resolve().parent, args.output_dir, case, width, height, args.timeout)
            except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired, KeyError, zlib.error) as error:
                result = dict(name=case.get("name", "invalid-case"), status="error", error=str(error))
            result["seconds"] = round(time.monotonic() - case_started, 4)
            results.append(result)
        passed = sum(case["status"] == "pass" for case in results)
        report = dict(suite="eris-self-authored-reftests", conformance_claim=False, binary=str(args.binary.resolve()), viewport=dict(width=width, height=height), passed=passed, failed=len(results)-passed, total=len(results), seconds=round(time.monotonic()-started, 4), cases=results)
    except (OSError, ValueError, KeyError, TypeError) as error:
        report = dict(suite="eris-self-authored-reftests", conformance_claim=False, passed=0, failed=1, total=0, error=str(error))
    rendered = json.dumps(report, indent=2) + "\n"
    (args.output_dir / "report.json").write_text(rendered)
    sys.stdout.write(rendered)
    return 0 if report["failed"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
