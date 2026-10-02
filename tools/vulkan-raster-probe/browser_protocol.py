"""Strict, inert protocol validation for the frozen offscreen browser bridge.

This module never launches a process or renders an image. Literal pixels and
input identities come from the independent oracle. GPU work/storage counters
are checked against bounds; they are not represented as exact frozen oracles.
"""
from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
import re
import struct

from run_host import inventory

DEFAULT_ORACLE = Path(__file__).resolve().parent / "browser-fixtures" / "oracle"
ORACLE_SHA256 = "d85574775ec6a2eb3537b498fe24678a888b930fe0e06efc52b410c8f15c0130"
FREEZE_SHA256 = "ec652b547096f37db93433d5347c78873cef2eae63a03ced622013aa746b88e1"
MAX_STDOUT = 2 * 1024 * 1024
MAX_DUMP = 65_536
STAT_FIELDS = ("lowered_commands", "unique_sources", "referenced_sources",
               "total_rgba_bytes", "referenced_rgba_bytes", "missing_images")
_N = r"(?:0|[1-9][0-9]{0,9})"
_OPTION = rf"(?:none|{_N})"
_CASE = re.compile(
    rf"CASE (?P<name>[a-z0-9-]+) path=(?P<path>gpu|cpu-fallback) "
    rf"reason=(?P<reason>[a-z0-9-]+) command_index=(?P<command_index>{_OPTION}) "
    + " ".join(rf"{name}=(?P<{name}>{_N})" for name in
               ("width", "height", "commands", "image_entries")) + " "
    + " ".join(rf"{name}=(?P<{name}>{_OPTION})" for name in STAT_FIELDS) + " "
    + " ".join(rf"{name}=(?P<{name}>{_N})" for name in
               ("draws", "invocations", "gpu_buffers", "compared_bytes"))
    + r" packed=(?P<packed>[0-9a-f]{8}(?:,[0-9a-f]{8})*) exact=true"
)
_SNAPSHOT = re.compile(rf"SNAPSHOT ([a-z0-9-]+) bytes=({_N}) hex=([0-9a-f]+)")
_CAPTURE = re.compile(rf"CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=({_N}) owned_children=0")
_INDEX = {"direct-hidden-text-refuses-whole-frame": 2,
          "direct-rounded-refuses-whole-frame": 0,
          "direct-unit-opacity-refuses-whole-frame": 1,
          "direct-transparent-hidden-rounded-still-refuses": 2}


def _require(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def _sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _read(path: Path, limit: int) -> bytes:
    with path.open("rb") as source:
        data = source.read(limit + 1)
    _require(len(data) <= limit, f"oracle file exceeds bound: {path.name}")
    return data


def _lines(stdout: bytes) -> list[str]:
    _require(isinstance(stdout, bytes) and 0 < len(stdout) <= MAX_STDOUT,
             "stdout size bound")
    _require(stdout.endswith(b"\n") and b"\r" not in stdout and b"\0" not in stdout,
             "stdout must use complete LF records")
    try:
        lines = stdout[:-1].decode("utf-8", errors="strict").split("\n")
    except UnicodeDecodeError as error:
        raise ValueError("stdout is not UTF-8") from error
    _require(len(lines) <= 64 and all(0 < len(line) <= 2 * MAX_DUMP + 256 for line in lines),
             "stdout record bound")
    return lines


def validate_listing(stdout: bytes) -> list[str]:
    """Accept only the ordered, complete Vulkan adapter inventory."""
    lines = _lines(stdout)
    devices = inventory(lines)
    _require(lines == devices, "unexpected listing output")
    return devices


def _oracle(directory: Path) -> list[dict]:
    freeze = _read(directory / "freeze.json", 65_536)
    _require(_sha(freeze) == FREEZE_SHA256, "oracle freeze identity")
    ledger = json.loads(freeze)
    # The pinned ledger contains only 29 bounded relative paths. Verify every
    # original input, including HTML, PNG, independent notes and peer review.
    _require(len(ledger["files"]) == 29, "oracle ledger inventory")
    for record in ledger["files"]:
        data = _read(directory / record["path"], 262_144)
        _require(len(data) == record["bytes"] and _sha(data) == record["sha256"],
                 f"oracle input identity: {record['path']}")
    data = _read(directory / "fixtures.json", 262_144)
    _require(_sha(data) == ORACLE_SHA256, "fixture identity")
    cases = json.loads(data)["cases"]
    _require(len(cases) == 16, "fixture count")
    return cases


class _Decoder:
    """Bounded little-endian EWB1 reader; no parser-controlled allocation sizes."""

    def __init__(self, data: bytes):
        _require(0 < len(data) <= MAX_DUMP, "EWB1 size")
        self.data = data
        self.offset = 0

    def raw(self, count: int) -> bytes:
        _require(0 <= count <= len(self.data) - self.offset, "truncated EWB1")
        value = self.data[self.offset:self.offset + count]
        self.offset += count
        return value

    def number(self, fmt: str):
        return struct.unpack("<" + fmt, self.raw(struct.calcsize("<" + fmt)))[0]

    def count(self, maximum: int = 256) -> int:
        count = self.number("I")
        _require(count <= maximum, "EWB1 count bound")
        return count

    def string(self, maximum: int = MAX_DUMP) -> str:
        try:
            return self.raw(self.count(maximum)).decode("utf-8", errors="strict")
        except UnicodeDecodeError as error:
            raise ValueError("EWB1 string is not UTF-8") from error

    def scalar(self, fmt: str = "f") -> float:
        value = self.number(fmt)
        _require(math.isfinite(value), "nonfinite EWB1 scalar")
        return value

    def rect(self) -> list[float]:
        rect = [self.scalar() for _ in range(4)]
        _require(all(abs(v) <= 1_000_000 for v in rect) and min(rect[2:]) >= 0,
                 "EWB1 rectangle bound")
        return rect

    def command(self) -> dict:
        tag = self.number("B")
        if tag == 0:
            return {"kind": "PushClip", "rect": self.rect()}
        if tag in (1, 2, 3, 5):
            return {"kind": {1: "PopClip", 2: "PushFixed", 3: "PopFixed", 5: "PopOpacity"}[tag]}
        if tag == 4:
            return {"kind": "PushOpacity", "opacity": self.scalar()}
        if tag == 6:
            result = {"kind": "Rect", "rect": self.rect(), "color": list(self.raw(4)),
                      "radius": self.scalar()}
            _require(0 <= result["radius"] <= 1_000_000, "EWB1 radius bound")
            return result
        if tag == 7:
            result = {"kind": "Text", "x": self.scalar(), "y": self.scalar(),
                      "size": self.scalar(), "color": list(self.raw(4))}
            flags = list(self.raw(3))
            _require(all(v in (0, 1) for v in flags), "EWB1 boolean")
            result.update(zip(("bold", "italic", "monospace"), flags))
            result["text"] = self.string()
            return result
        if tag == 8:
            return {"kind": "Image", "rect": self.rect(), "key": self.string(4096)}
        if tag == 9:
            result = {"kind": "Line"}
            result.update((key, self.scalar()) for key in ("x1", "y1", "x2", "y2", "width"))
            result["color"] = list(self.raw(4))
            return result
        raise ValueError("unknown EWB1 command tag")


def _decode(data: bytes) -> dict:
    decoder = _Decoder(data)
    _require(decoder.raw(4) == b"EWB1", "EWB1 magic")
    result = {"width": decoder.number("I"), "height": decoder.number("I"),
              "generation": decoder.number("Q"), "processed_edit_sequence": decoder.number("Q"),
              "task_state": decoder.number("B"), "content_height": decoder.scalar(),
              "load_ms": decoder.scalar("d"), "title": decoder.string(), "url": decoder.string()}
    _require(0 <= result["content_height"] <= 1_000_000 and result["load_ms"] >= 0,
             "EWB1 metadata bounds")
    result["commands"] = [decoder.command() for _ in range(decoder.count())]
    keys = []
    next_id = 0
    for _ in range(decoder.count()):
        key, source_id = decoder.string(4096), decoder.count(255)
        _require(not keys or keys[-1]["key"] < key, "EWB1 keys must be unique and sorted")
        _require(source_id <= next_id, "EWB1 source IDs must follow first sorted-key occurrence")
        if source_id == next_id:
            next_id += 1
        keys.append({"key": key, "source_id": source_id})
    source_count = decoder.count()
    _require(source_count == next_id, "EWB1 source/key inventory")
    sources = []
    for _ in range(source_count):
        width, height, size = decoder.number("I"), decoder.number("I"), decoder.count(MAX_DUMP)
        _require(width > 0 and height > 0 and width * height * 4 == size, "EWB1 RGBA dimensions")
        rgba = decoder.raw(size)
        sources.append({"width": width, "height": height, "rgba_hex": rgba.hex(), "sha256": _sha(rgba)})
    result.update(image_keys=keys, sources=sources,
                  diagnostics=[decoder.string() for _ in range(decoder.count())])
    _require(decoder.offset == len(data), "trailing EWB1 bytes")
    return result


def _worker_primitives(case: dict) -> list[dict]:
    """Transcription of the five frozen HTML source assumptions, not rendering."""
    frame = case["frame"]
    def rect(box, color, radius=0):
        return {"kind": "Rect", "rect": box, "color": color, "radius": radius}
    def image(box, key):
        return {"kind": "Image", "rect": box, "key": key}
    red, green, blue = [255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]
    white = rect([0, 0, frame["width"], frame["height"]], [255] * 4)
    return [white] + {
        "html-ordered-rectangles": [rect([1, 1, 4, 2], red), rect([3, 0, 2, 3], blue)],
        "html-fixed-child-escapes-scrolling-clip": [rect([1, 1, 2, 2], red),
            rect([2, 2, 2, 2], green), rect([5, 0, 2, 1], blue)],
        "html-repeated-decoded-image-key": [image([0, 0, 4, 1], "two-pixels.png"),
            image([2, 1, 4, 1], "two-pixels.png")],
        "html-unavailable-image-keeps-placeholder": [rect([1, 0, 2, 1], [236, 238, 242, 255]),
            image([1, 0, 2, 1], "deliberately-absent.png"), rect([3, 1, 1, 1], blue)],
        "html-rounded-page-requires-full-cpu-fallback": [rect([1, 0, 2, 2], blue, 1),
            rect([0, 2, 1, 1], red)],
    }[case["name"]]


def _verify_worker(snapshot: dict, case: dict, index: int, directory: Path) -> None:
    address = (directory / case["html_file"]).resolve().as_uri()
    frame = case["frame"]
    _require((snapshot["width"], snapshot["height"]) == (frame["width"], frame["height"]),
             "worker viewport differs")
    _require(snapshot["generation"] == 2000 + index and snapshot["processed_edit_sequence"] == 0
             and snapshot["task_state"] == 0 and snapshot["url"] == address
             and snapshot["title"] == address, "worker request metadata differs")
    wanted = _worker_primitives(case)
    seen, scopes = [], []
    fixed_case = case["name"] == "html-fixed-child-escapes-scrolling-clip"
    for command in snapshot["commands"]:
        kind = command["kind"]
        if kind in ("PushClip", "PushFixed"):
            scopes.append(command)
            _require(len(scopes) <= 32, "worker scope limit")
            continue
        if kind in ("PopClip", "PopFixed"):
            _require(bool(scopes) and scopes[-1]["kind"] == kind.replace("Pop", "Push"),
                     "worker scope balance")
            scopes.pop()
            continue
        if kind == "Rect" and command["radius"] == 0 and (
                command["color"][3] == 0 or min(command["rect"][2:]) == 0):
            continue  # Retained in complete dump and counts; never removed from evidence.
        _require(len(seen) < len(wanted) and command == wanted[len(seen)],
                 "worker primitive differs from frozen assumption")
        if fixed_case:
            fixed = any(scope["kind"] == "PushFixed" for scope in scopes)
            _require(fixed == (command["color"] == [0, 0, 255, 255]), "worker fixed scope differs")
            if command["color"] == [0, 255, 0, 255]:
                _require(any(scope == {"kind": "PushClip", "rect": [1, 1, 2, 2]} for scope in scopes),
                         "worker document clip differs")
        seen.append(command)
    _require(not scopes and seen == wanted, "worker command inventory differs")
    if case["name"] == "html-repeated-decoded-image-key":
        _require(snapshot["image_keys"] == [{"key": "two-pixels.png", "source_id": 0}]
                 and len(snapshot["sources"]) == 1, "worker image identity inventory differs")
        source = snapshot["sources"][0]
        _require((source["width"], source["height"], source["rgba_hex"]) ==
                 (2, 1, "ff0000ff0000ff80"), "worker decoded source bytes differ")
    else:
        _require(not snapshot["image_keys"] and not snapshot["sources"], "unexpected worker image")
    if case["name"] == "html-unavailable-image-keeps-placeholder":
        _require(any("deliberately-absent.png" in value for value in snapshot["diagnostics"]),
                 "missing-image diagnostic was not retained")


def _input(case: dict, snapshots: dict) -> tuple[list[dict], dict[str, int], list[int]]:
    if case["input_kind"] == "worker-html":
        snapshot = snapshots[case["name"]]["decoded"]
        return (snapshot["commands"], {v["key"]: v["source_id"] for v in snapshot["image_keys"]},
                [len(bytes.fromhex(v["rgba_hex"])) for v in snapshot["sources"]])
    keys = {key: i for i, source in enumerate(case["images"]) for key in source["keys"]}
    return case["commands"], keys, [len(v["rgba"]) for v in case["images"]]


def _case(line: str, case: dict, snapshots: dict) -> dict:
    match = _CASE.fullmatch(line)
    _require(match is not None, "malformed CASE record")
    row = match.groupdict()
    for key in ("command_index", "width", "height", "commands", "image_entries", *STAT_FIELDS,
                "draws", "invocations", "gpu_buffers", "compared_bytes"):
        row[key] = None if row[key] == "none" else int(row[key])
    frame = case["frame"]
    commands, keys, sizes = _input(case, snapshots)
    _require(row["name"] == case["name"] and row["path"] == case["expected_path"]
             and row["reason"] == (case["expected_fallback_reason"] or "none"), "CASE route/order differs")
    _require((row["width"], row["height"], row["commands"], row["image_entries"]) ==
             (frame["width"], frame["height"], len(commands), len(keys)), "CASE input counts differ")
    pixels = case["expected_rgb_hex_row_major"]
    _require(row["packed"] == ",".join("00" + pixel for pixel in pixels)
             and row["compared_bytes"] == len(pixels) * 4, "CASE literal pixels differ")
    if row["command_index"] is not None:
        _require(row["command_index"] < len(commands), "CASE rejection index bound")
    if case["name"] in _INDEX:
        _require(row["command_index"] == _INDEX[case["name"]], "CASE frozen rejection index differs")
    if row["path"] == "cpu-fallback":
        _require(all(row[key] is None for key in STAT_FIELDS), "fallback has no successful bridge stats")
        _require(row["draws"] == row["invocations"] == row["gpu_buffers"] == 0,
                 "fallback must not claim GPU work")
    else:
        _require(row["command_index"] is None, "GPU case has rejection index")
        missing, referenced = 0, set()
        for command in commands:
            if command["kind"] == "Image":
                if command["key"] in keys:
                    referenced.add(keys[command["key"]])
                else:
                    missing += 1
        stats = (len(commands) - missing, len(sizes), len(referenced), sum(sizes),
                 sum(sizes[i] for i in referenced), missing)
        _require(tuple(row[key] for key in STAT_FIELDS) == stats, "CASE bridge stats differ")
        _require(1 <= row["draws"] <= min(257, row["lowered_commands"] + 1), "GPU draw bound")
        _require(64 * row["draws"] <= row["invocations"] <= 4_000_000
                 and row["invocations"] % 64 == 0, "GPU invocation bound")
        _require(8 * len(pixels) + 256 * row["draws"] <= row["gpu_buffers"] <= 1_048_576
                 and row["gpu_buffers"] % 4 == 0, "GPU buffer bound")
    row.update(exact=True, expected_rgb_sha256=case["expected_rgb_bytes_sha256"],
               expected_packed_sha256=case["expected_packed_rgb_le_sha256"])
    return row


def validate_run(stdout: bytes, adapters: list[str], index: int,
                 oracle_dir: Path = DEFAULT_ORACLE) -> dict:
    """Validate a complete run. Process cleanup/exit/stderr remain host obligations."""
    _require(isinstance(adapters, list) and all(isinstance(v, str) for v in adapters), "adapter input")
    _require(validate_listing(("\n".join(adapters) + "\n").encode()) == adapters, "adapter inventory")
    _require(type(index) is int and 0 <= index < len(adapters), "selected adapter bound")
    directory = Path(oracle_dir).resolve()
    cases = _oracle(directory)
    lines = _lines(stdout)
    expected_lines = 5 + 1 + 7 + len(adapters) + 9 + 1
    _require(len(lines) == expected_lines, "run record inventory differs")
    snapshots, cursor = {}, 0
    for case_index, case in enumerate(cases):
        if case["input_kind"] != "worker-html":
            continue
        match = _SNAPSHOT.fullmatch(lines[cursor])
        _require(match is not None and match[1] == case["name"], "snapshot order/name differs")
        size = int(match[2])
        _require(0 < size <= MAX_DUMP and len(match[3]) == size * 2, "snapshot hex length")
        data = bytes.fromhex(match[3])
        decoded = _decode(data)
        _verify_worker(decoded, case, case_index, directory)
        snapshots[case["name"]] = {"bytes": size, "sha256": _sha(data), "hex": match[3], "decoded": decoded}
        cursor += 1
    capture = _CAPTURE.fullmatch(lines[cursor])
    _require(capture is not None and 1 <= int(capture[1]) <= 64, "capture footer differs")
    cursor += 1
    rows = []
    for case in cases:
        if case["expected_path"] == "cpu-fallback":
            rows.append(_case(lines[cursor], case, snapshots))
            cursor += 1
    _require(lines[cursor:cursor + len(adapters)] == adapters, "run adapter inventory differs")
    cursor += len(adapters)
    for case in cases:
        if case["expected_path"] == "gpu":
            rows.append(_case(lines[cursor], case, snapshots))
            cursor += 1
    footer = (f"BROWSER_COMPLETE adapter={index} cases=16 gpu=9 fallback=7 pixels=197 "
              "compared_bytes=788 exact=true custom_wgsl=true")
    _require(lines[cursor] == footer and cursor + 1 == len(lines), "completion footer differs")
    _require(sum(row["compared_bytes"] for row in rows) == 788, "compared-byte total differs")
    return {"schema": 1, "adapter": index, "adapters": adapters, "stdout_sha256": _sha(stdout),
            "oracle_sha256": ORACLE_SHA256, "oracle_freeze_sha256": FREEZE_SHA256,
            "snapshots": snapshots, "capture": {"cases": 16, "worker_cases": 5,
                "own_tasks": int(capture[1]), "owned_children": 0}, "cases": rows,
            "counts": {"cases": 16, "gpu": 9, "fallback": 7, "pixels": 197, "compared_bytes": 788},
            "gpu_counter_policy": "bounds only; bridge input statistics and literal pixels are exact",
            "exact": True, "custom_wgsl": True}
