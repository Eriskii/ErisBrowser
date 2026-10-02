"""Synthetic transcript tests only: never launch workers, renderers or Vulkan.

The fake counters below satisfy bounds; they are deliberately not observations
or an exact GPU counter oracle. Independent literal RGB targets remain frozen.
"""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest

import browser_protocol as protocol

ORACLE = protocol.DEFAULT_ORACLE
ADAPTERS = [
    'ADAPTER 0 vendor=0x1234 device=0x1 type=DiscreteGpu backend=Vulkan name="synthetic" driver="fake" info="protocol-only"',
    'ADAPTER 1 vendor=0x0 device=0x2 type=Cpu backend=Vulkan name="synthetic-cpu" driver="fake" info="protocol-only"',
]


def u32(value):
    return struct.pack("<I", value)


def string(value):
    encoded = value.encode("utf-8")
    return u32(len(encoded)) + encoded


def rect(box, color, radius=0):
    return {"kind": "Rect", "rect": box, "color": color, "radius": radius}


def image(box, key):
    return {"kind": "Image", "rect": box, "key": key}


def command_bytes(command):
    kind = command["kind"]
    if kind == "PushClip":
        return b"\0" + struct.pack("<4f", *command["rect"])
    if kind in ("PopClip", "PushFixed", "PopFixed", "PopOpacity"):
        return bytes([{"PopClip": 1, "PushFixed": 2, "PopFixed": 3, "PopOpacity": 5}[kind]])
    if kind == "PushOpacity":
        return b"\4" + struct.pack("<f", command["opacity"])
    if kind == "Rect":
        return b"\6" + struct.pack("<4f", *command["rect"]) + bytes(command["color"]) + struct.pack("<f", command["radius"])
    if kind == "Image":
        return b"\10" + struct.pack("<4f", *command["rect"]) + string(command["key"])
    if kind == "Text":
        return (b"\7" + struct.pack("<3f", command["x"], command["y"], command["size"])
                + bytes(command["color"]) + bytes(command["flags"]) + string(command["text"]))
    raise AssertionError("test encoder only needs specified synthetic kinds")


def snapshot_bytes(snapshot):
    result = (b"EWB1" + struct.pack("<IIQQBfd", snapshot["width"], snapshot["height"],
              snapshot["generation"], snapshot["edit"], snapshot["task"], snapshot["content_height"], snapshot["load_ms"])
              + string(snapshot["title"]) + string(snapshot["url"]))
    result += u32(len(snapshot["commands"])) + b"".join(map(command_bytes, snapshot["commands"]))
    result += u32(len(snapshot["keys"]))
    for key, source_id in snapshot["keys"]:
        result += string(key) + u32(source_id)
    result += u32(len(snapshot["sources"]))
    for width, height, rgba in snapshot["sources"]:
        result += u32(width) + u32(height) + u32(len(rgba)) + bytes(rgba)
    result += u32(len(snapshot["diagnostics"])) + b"".join(map(string, snapshot["diagnostics"]))
    return result


class Transcript:
    def __init__(self, directory=ORACLE):
        self.directory = Path(directory).resolve()
        self.cases = json.loads((self.directory / "fixtures.json").read_bytes())["cases"]
        self.snapshots = {}
        red, green, blue = [255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]
        # Explicit source-level fixtures, authored without the production
        # parser's worker-assumption helper and without any browser outcomes.
        bodies = [
            [rect([1, 1, 4, 2], red), rect([3, 0, 2, 3], blue)],
            [{"kind": "PushClip", "rect": [1, 1, 2, 2]}, rect([1, 1, 2, 2], red),
             rect([2, 2, 2, 2], green), {"kind": "PushFixed"}, rect([5, 0, 2, 1], blue),
             {"kind": "PopFixed"}, {"kind": "PopClip"}],
            [image([0, 0, 4, 1], "two-pixels.png"), image([2, 1, 4, 1], "two-pixels.png")],
            [rect([1, 0, 2, 1], [236, 238, 242, 255]), image([1, 0, 2, 1], "deliberately-absent.png"),
             rect([3, 1, 1, 1], blue)],
            [rect([1, 0, 2, 2], blue, 1), rect([0, 2, 1, 1], red)],
        ]
        for i, case in enumerate(self.cases[9:14]):
            width, height = case["frame"]["width"], case["frame"]["height"]
            address = (self.directory / case["html_file"]).as_uri()
            self.snapshots[case["name"]] = {
                "width": width, "height": height, "generation": 2009 + i, "edit": 0, "task": 0,
                "content_height": height, "load_ms": 0.25, "title": address, "url": address,
                "commands": [rect([0, 0, width, height], [255] * 4)] + bodies[i],
                "keys": [("two-pixels.png", 0)] if i == 2 else [],
                "sources": [(2, 1, [255, 0, 0, 255, 0, 0, 255, 128])] if i == 2 else [],
                "diagnostics": ["synthetic worker diagnostic"] + (["missing deliberately-absent.png"] if i == 3 else []),
            }

    def snapshot_line(self, name):
        data = snapshot_bytes(self.snapshots[name])
        return f"SNAPSHOT {name} bytes={len(data)} hex={data.hex()}"

    def case_line(self, case):
        name = case["name"]
        frame = case["frame"]
        if case["input_kind"] == "worker-html":
            snapshot = self.snapshots[name]
            commands = snapshot["commands"]
            entries = len(snapshot["keys"])
        else:
            commands = case["commands"]
            entries = sum(len(source["keys"]) for source in case["images"])
        count = len(commands)
        fallback = case["expected_path"] == "cpu-fallback"
        indices = {"direct-hidden-text-refuses-whole-frame": "2",
                   "direct-rounded-refuses-whole-frame": "0", "direct-unit-opacity-refuses-whole-frame": "1",
                   "direct-transparent-hidden-rounded-still-refuses": "2",
                   "html-rounded-page-requires-full-cpu-fallback": "1",
                   "direct-scope-cap-refuses-balanced-whole-frame": "32"}
        index = indices.get(name, "none")
        if fallback:
            stats = ["none"] * 6
            draws = work = storage = 0
        else:
            # The only sources are one aliased 8-byte direct image and one
            # repeated 8-byte worker image. Each missing fixture has one miss.
            has_source = name in ("direct-repeated-key-alias-and-alpha", "html-repeated-decoded-image-key")
            missing = int(name in ("direct-missing-image-is-a-legal-noop", "html-unavailable-image-keeps-placeholder"))
            stats = [count - missing, int(has_source), int(has_source), 8 * has_source, 8 * has_source, missing]
            draws, work = 1, 64
            storage = frame["width"] * frame["height"] * 8 + 256
        stat_fields = " ".join(f"{key}={value}" for key, value in zip(protocol.STAT_FIELDS, stats))
        packed = ",".join("00" + value for value in case["expected_rgb_hex_row_major"])
        return (f"CASE {name} path={case['expected_path']} reason={case['expected_fallback_reason'] or 'none'} "
                f"command_index={index} width={frame['width']} height={frame['height']} commands={count} image_entries={entries} "
                f"{stat_fields} draws={draws} invocations={work} gpu_buffers={storage} "
                f"compared_bytes={frame['width'] * frame['height'] * 4} packed={packed} exact=true")

    def lines(self):
        return ([self.snapshot_line(case["name"]) for case in self.cases[9:14]]
                + ["CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=1 owned_children=0"]
                + [self.case_line(case) for case in self.cases if case["expected_path"] == "cpu-fallback"]
                + ADAPTERS
                + [self.case_line(case) for case in self.cases if case["expected_path"] == "gpu"]
                + ["BROWSER_COMPLETE adapter=0 cases=16 gpu=9 fallback=7 pixels=197 compared_bytes=788 exact=true custom_wgsl=true"])


def encoded(lines):
    return ("\n".join(lines) + "\n").encode()


class BrowserProtocolTests(unittest.TestCase):
    def setUp(self):
        self.transcript = Transcript()

    def validate(self, lines):
        return protocol.validate_run(encoded(lines), ADAPTERS, 0, self.transcript.directory)

    def test_complete_synthetic_transcript_retains_all_capture_fields(self):
        lines = self.transcript.lines()
        result = self.validate(lines)
        self.assertEqual(result["counts"], {"cases": 16, "gpu": 9, "fallback": 7, "pixels": 197, "compared_bytes": 788})
        self.assertEqual(len(result["snapshots"]), 5)
        self.assertEqual(len(result["cases"]), 16)
        for name, record in result["snapshots"].items():
            data = snapshot_bytes(self.transcript.snapshots[name])
            self.assertEqual(record["sha256"], hashlib.sha256(data).hexdigest())
            self.assertEqual(bytes.fromhex(record["hex"]), data)
            self.assertEqual(record["decoded"]["diagnostics"], self.transcript.snapshots[name]["diagnostics"])
        self.assertEqual(result["stdout_sha256"], hashlib.sha256(encoded(lines)).hexdigest())

    def test_listing_rejects_extras_duplicates_order_gaps_and_bad_backend(self):
        self.assertEqual(protocol.validate_listing(encoded(ADAPTERS)), ADAPTERS)
        for bad in ([*ADAPTERS, "extra"], ADAPTERS[::-1], [ADAPTERS[0]] * 2,
                    [ADAPTERS[1]], [ADAPTERS[0].replace("Vulkan", "Gl")], [], ["ADAPTER 00 nope"]):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                protocol.validate_listing(encoded(bad))

    def test_every_case_omission_duplicate_reordering_and_changed_pixels_rejected(self):
        original = self.transcript.lines()
        for i, line in enumerate(original):
            if not line.startswith("CASE "):
                continue
            variants = [original[:i] + original[i + 1:], original[:i] + [line] + original[i:],
                        original[:i] + [line.replace("packed=00", "packed=ff", 1)] + original[i + 1:]]
            j = next(j for j, other in enumerate(original) if j != i and other.startswith("CASE "))
            swapped = original.copy()
            swapped[i], swapped[j] = swapped[j], swapped[i]
            variants.append(swapped)
            for variant in variants:
                with self.subTest(case=line.split()[1]), self.assertRaises(ValueError):
                    self.validate(variant)

    def test_phase_boundaries_inventory_and_footer_are_mandatory(self):
        original = self.transcript.lines()
        variants = [original[1:], original[:-1], original + [original[-1]],
                    original[:5] + original[6:], [ADAPTERS[0]] + original,
                    [line.replace("owned_children=0", "owned_children=1") for line in original],
                    [line.replace("own_tasks=1", "own_tasks=0") for line in original],
                    [line.replace("own_tasks=1", "own_tasks=65") for line in original],
                    [line.replace("adapter=0 cases=16", "adapter=1 cases=16") for line in original],
                    [line.replace("pixels=197", "pixels=196") for line in original]]
        for variant in variants:
            with self.subTest(variant=variants.index(variant)), self.assertRaises(ValueError):
                self.validate(variant)
        with self.assertRaises(ValueError):
            protocol.validate_run(encoded(original), ADAPTERS, True, ORACLE)

    def test_capture_hex_length_magic_truncation_tail_and_unknown_tags(self):
        original = self.transcript.lines()
        name = next(iter(self.transcript.snapshots))
        raw = snapshot_bytes(self.transcript.snapshots[name])
        for data in (b"NOPE" + raw[4:], raw[:-1], raw + b"\0", b"EWB1", raw[:32]):
            lines = original.copy()
            lines[0] = f"SNAPSHOT {name} bytes={len(data)} hex={data.hex()}"
            with self.subTest(size=len(data)), self.assertRaises(ValueError):
                self.validate(lines)
        lines = original.copy()
        lines[0] = lines[0].replace(f"bytes={len(raw)}", f"bytes={len(raw) + 1}")
        with self.assertRaises(ValueError):
            self.validate(lines)
        with self.assertRaises(ValueError):
            protocol._Decoder(b"\xff").command()

    def test_worker_request_identity_and_source_assumptions_cannot_drift(self):
        base = self.transcript
        name = next(iter(base.snapshots))
        for key, value in (("generation", 0), ("edit", 1), ("task", 1), ("width", 7),
                           ("url", "file:///other"), ("title", "changed"), ("content_height", float("nan")),
                           ("load_ms", float("inf"))):
            self.transcript = copy.deepcopy(base)
            self.transcript.snapshots[name][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(self.transcript.lines())
        self.transcript = copy.deepcopy(base)
        self.transcript.snapshots[name]["commands"][1]["rect"][0] = 2
        with self.assertRaises(ValueError):
            self.validate(self.transcript.lines())

    def test_worker_fixed_clip_balance_and_hidden_unsupported_are_checked(self):
        base = self.transcript
        fixed = "html-fixed-child-escapes-scrolling-clip"
        for edit in ("remove-fixed", "change-clip", "mismatched-pop", "hidden-text", "hidden-opacity"):
            self.transcript = copy.deepcopy(base)
            commands = self.transcript.snapshots[fixed]["commands"]
            if edit == "remove-fixed":
                commands[:] = [c for c in commands if c["kind"] not in ("PushFixed", "PopFixed")]
            elif edit == "change-clip":
                commands[1]["rect"] = [0, 0, 0, 0]
            elif edit == "mismatched-pop":
                commands[-1] = {"kind": "PopFixed"}
            elif edit == "hidden-text":
                commands.insert(0, {"kind": "Text", "x": 0, "y": 0, "size": 0, "color": [0] * 4,
                                    "flags": [0, 0, 0], "text": ""})
            else:
                commands.insert(0, {"kind": "PushOpacity", "opacity": 1})
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.validate(self.transcript.lines())

    def test_image_bytes_dimensions_alias_ids_keys_and_missing_diagnostics(self):
        base = self.transcript
        name = "html-repeated-decoded-image-key"
        for edit in ("bytes", "dimensions", "bad-id", "duplicate", "unused-source", "unsorted"):
            self.transcript = copy.deepcopy(base)
            snap = self.transcript.snapshots[name]
            if edit == "bytes":
                snap["sources"][0][2][0] = 0
            elif edit == "dimensions":
                snap["sources"][0] = (1, 1, [255] * 8)
            elif edit == "bad-id":
                snap["keys"][0] = ("two-pixels.png", 1)
            elif edit == "duplicate":
                snap["keys"] *= 2
            elif edit == "unused-source":
                snap["sources"].append((1, 1, [255] * 4))
            else:
                snap["keys"].append(("a.png", 0))
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.validate(self.transcript.lines())
        self.transcript = copy.deepcopy(base)
        self.transcript.snapshots["html-unavailable-image-keeps-placeholder"]["diagnostics"] = []
        with self.assertRaises(ValueError):
            self.validate(self.transcript.lines())

    def test_all_six_stats_are_exact_and_fallback_does_not_invent_stats(self):
        original = self.transcript.lines()
        for i, line in enumerate(original):
            if not line.startswith("CASE "):
                continue
            for key in protocol.STAT_FIELDS:
                old = line.split(key + "=", 1)[1].split()[0]
                replacement = "0" if old == "none" else str(int(old) + 1)
                lines = original.copy()
                lines[i] = line.replace(f"{key}={old} ", f"{key}={replacement} ", 1)
                with self.subTest(case=line.split()[1], key=key), self.assertRaises(ValueError):
                    self.validate(lines)

    def test_case_counts_routes_indices_and_resource_bounds(self):
        original = self.transcript.lines()
        gpu_index = next(i for i, line in enumerate(original) if line.startswith("CASE ") and "path=gpu " in line)
        fallback_index = next(i for i, line in enumerate(original) if "path=cpu-fallback " in line)
        for old, new in (("draws=1 ", "draws=258 "), ("invocations=64 ", "invocations=65 "),
                         ("invocations=64 ", "invocations=4000064 "), ("gpu_buffers=448 ", "gpu_buffers=4 "),
                         ("width=6 ", "width=5 "), ("commands=15 ", "commands=14 "),
                         ("image_entries=0 ", "image_entries=1 "), ("command_index=none ", "command_index=0 "),
                         ("compared_bytes=96 ", "compared_bytes=95 "), ("exact=true", "exact=false")):
            self.assertIn(old, original[gpu_index])
            lines = original.copy()
            lines[gpu_index] = lines[gpu_index].replace(old, new, 1)
            with self.subTest(field=old), self.assertRaises(ValueError):
                self.validate(lines)
        for old, new in (("command_index=2 ", "command_index=1 "),
                         ("reason=unsupported-text ", "reason=unsupported-opacity "), ("draws=0 ", "draws=1 ")):
            lines = original.copy()
            lines[fallback_index] = lines[fallback_index].replace(old, new, 1)
            with self.subTest(field=old), self.assertRaises(ValueError):
                self.validate(lines)

    def test_record_sizes_utf8_and_noncanonical_numbers(self):
        data = encoded(self.transcript.lines())
        for bad in (data[:-1], data + b"\n", data.replace(b"\n", b"\r\n"), data + b"\0",
                    b"\xff\n", b"x" * (protocol.MAX_STDOUT + 1), data.replace(b"width=6 ", b"width=06 ", 1)):
            with self.subTest(size=len(bad)), self.assertRaises(ValueError):
                protocol.validate_run(bad, ADAPTERS, 0, ORACLE)
        with self.assertRaises(ValueError):
            protocol._Decoder(u32(0xffffffff)).string()
        with self.assertRaises(ValueError):
            protocol._Decoder(u32(257)).count()
        with self.assertRaises(ValueError):
            protocol._Decoder(u32(1) + b"\xff").string()

    def test_binary_decoder_preserves_float_bits_and_rejects_invalid_bool(self):
        command = {"kind": "Rect", "rect": [-0.0, 0.0, 0.0, 0.0], "color": [0] * 4, "radius": 0}
        raw = command_bytes(command)
        parsed = protocol._Decoder(raw).command()
        self.assertEqual(struct.pack("<f", parsed["rect"][0]), b"\0\0\0\x80")
        text = {"kind": "Text", "x": 0, "y": 0, "size": 1, "color": [0] * 4,
                "flags": [2, 0, 0], "text": ""}
        with self.assertRaises(ValueError):
            protocol._Decoder(command_bytes(text)).command()

    def test_benign_worker_entries_are_retained_and_counted_not_pruned(self):
        name = "html-ordered-rectangles"
        self.transcript.snapshots[name]["commands"].insert(0, rect([-0.0, 0, 0, 0], [0] * 4))
        result = self.validate(self.transcript.lines())
        commands = result["snapshots"][name]["decoded"]["commands"]
        self.assertEqual(len(commands), 4)
        case = next(case for case in result["cases"] if case["name"] == name)
        self.assertEqual(case["commands"], 4)
        self.assertEqual(case["lowered_commands"], 4)

    def test_frozen_oracle_assets_cannot_be_changed_to_accept_a_transcript(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "oracle"
            shutil.copytree(ORACLE, target)
            self.transcript = Transcript(target)
            original = self.transcript.lines()
            asset = target / "expected" / "html-ordered-rectangles.rgb"
            asset.write_bytes(b"\0" * asset.stat().st_size)
            with self.assertRaises(ValueError):
                self.validate(original)


if __name__ == "__main__":
    unittest.main()
