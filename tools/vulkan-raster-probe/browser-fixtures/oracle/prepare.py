#!/usr/bin/env python3
"""Data-only fixture preparation. Literal rows never derive from commands."""
import hashlib
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parent
PALETTE = {
    "W": "ffffff", "R": "ff0000", "G": "00ff00", "B": "0000ff",
    "C": "00ffff", "M": "ff00ff", "Y": "ffff00", "K": "000000",
    "P": "ff7f7f", "T": "204060", "A": "742a44", "D": "9e1f36",
    "F": "3535ff", "H": "7f7fff", "S": "eceef2",
}


def rect(x, y, w, h, rgba, radius=0):
    return dict(kind="Rect", rect=[x, y, w, h], color=rgba, radius=radius)


def image(x, y, w, h, key):
    return dict(kind="Image", rect=[x, y, w, h], key=key)


def push(rectangle):
    return dict(kind="PushClip", rect=rectangle)


def op(kind):
    return dict(kind=kind)


def frame(w, h, clear="ffffff", clip=None, document=(0, 0), fixed=(0, 0)):
    return dict(width=w, height=h, clear_rgb=clear,
                caller_clip=clip or [0, 0, w, h], document_offset=list(document),
                viewport_offset=list(fixed), zoom=1)


R, G, B = [255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]
cases = []


def direct(name, target, commands, rows, reason=None, **extra):
    cases.append(dict(name=name, input_kind="direct-draw-command", frame=target,
                      commands=commands, images=[], literal_rows=rows,
                      expected_path="cpu-fallback" if reason else "gpu",
                      expected_fallback_reason=reason, **extra))


direct("direct-fixed-escape-and-nested-restoration",
       frame(6, 4, clip=[1, 0, 4, 4], document=(0, -1), fixed=(1, 1)), [
           rect(1, 1, 4, 3, R), push([2, 1, 1, 2]), rect(0, 0, 6, 6, G),
           op("PushFixed"), rect(0, 0, 2, 1, B), push([1, 0, 1, 1]),
           op("PushFixed"), rect(2, 1, 2, 1, [255, 255, 0, 255]),
           op("PopFixed"), rect(0, 0, 4, 4, [255, 0, 255, 255]),
           op("PopClip"), op("PopFixed"), rect(2, 1, 1, 1, [0, 255, 255, 255]),
           op("PopClip"), rect(4, 3, 1, 1, [0, 0, 0, 255]),
       ], ["WRCRRW", "WBMRRW", "WRRYKW", "WWWWWW"])
direct("direct-translucent-fractional-clip-edges",
       frame(5, 3, clip=[0.5, 0.5, 3, 1.5]),
       [rect(-0.25, -0.25, 5, 3, [255, 0, 0, 128])],
       ["WWWWW", "WPPPW", "WWWWW"])
direct("direct-repeated-key-alias-and-alpha", frame(4, 2, clear="204060"), [
    image(0, 0, 4, 1, "shared.png"), image(0, 0, 4, 1, "shared.png"),
    image(0, 1, 4, 1, "alias.png"),
], ["DDTT", "AATT"])
cases[-1]["images"] = [dict(keys=["shared.png", "alias.png"], backing_id="shared-0",
                           width=2, height=1,
                           rgba=[200, 20, 40, 128, 99, 77, 55, 0])]
direct("direct-missing-image-is-a-legal-noop", frame(4, 2), [
    rect(0, 0, 2, 2, R), image(0, 0, 4, 2, "missing.png"), rect(3, 1, 1, 1, B),
], ["RRWW", "RRWB"], absent_image_keys=["missing.png"])
direct("direct-lines-are-bounding-rectangles", frame(5, 4), [
    dict(kind="Line", x1=4, y1=2, x2=1, y2=0, width=1, color=B),
    dict(kind="Line", x1=0, y1=3, x2=4, y2=3, width=1, color=R),
    dict(kind="Line", x1=0, y1=0, x2=0, y2=0, width=2, color=G),
], ["GGBBW", "GGBBW", "WWWWW", "RRRRW"])
direct("direct-hidden-text-refuses-whole-frame", frame(4, 2), [
    rect(0, 0, 2, 2, R), push([0, 0, 0, 0]),
    dict(kind="Text", x=0, y=0, text="hidden", size=12, color=[0, 0, 0, 255],
         bold=False, italic=False, monospace=False),
    op("PopClip"), rect(3, 1, 1, 1, B),
], ["RRWW", "RRWB"], "unsupported-text", first_reason_command_index=2)
direct("direct-rounded-refuses-whole-frame", frame(4, 3), [
    rect(1, 0, 2, 2, B, radius=1), rect(0, 2, 1, 1, R),
], ["WFFW", "WFFW", "RWWW"], "unsupported-rounded-rectangle",
       first_reason_command_index=0)
direct("direct-unit-opacity-refuses-whole-frame", frame(3, 2), [
    rect(0, 0, 3, 2, B), dict(kind="PushOpacity", opacity=1),
    rect(0, 0, 2, 1, R), op("PopOpacity"), rect(2, 1, 1, 1, G),
], ["RRB", "BBG"], "unsupported-opacity", first_reason_command_index=1)
direct("direct-transparent-hidden-rounded-still-refuses", frame(3, 2), [
    rect(0, 0, 1, 1, R), push([0, 0, 0, 0]),
    rect(0, 0, 1, 1, [99, 77, 55, 0], radius=3), op("PopClip"),
    rect(2, 1, 1, 1, B),
], ["RWW", "WWB"], "unsupported-rounded-rectangle", first_reason_command_index=2)


def html(name, target, css, body, rows, assumptions, reason=None):
    source = ("<!doctype html><html><head><meta charset=utf-8><style>"
              "html,body{margin:0;padding:0;border:0}body{background:white;"
              f"width:{target['width']}px;height:{target['height']}px;position:relative}}"
              + css + "</style></head><body>" + body + "</body></html>\n")
    path = ROOT / "html" / (name + ".html")
    path.parent.mkdir(exist_ok=True)
    with path.open("x") as output:
        output.write(source)
    cases.append(dict(name=name, input_kind="worker-html", html_file=str(path.relative_to(ROOT)),
                      frame=target, literal_rows=rows,
                      expected_path="cpu-fallback" if reason else "gpu",
                      expected_fallback_reason=reason,
                      worker_command_assumptions=assumptions,
                      scripts_enabled=False, network_allowed=False))


html("html-ordered-rectangles", frame(6, 4),
     "#red{position:absolute;left:1px;top:1px;width:4px;height:2px;background:red}"
     "#blue{position:absolute;left:3px;top:0;width:2px;height:3px;background:blue}",
     "<div id=red></div><div id=blue></div>",
     ["WWWBBW", "WRRBBW", "WRRBBW", "WWWWWW"],
     ["White root/body canvas background covers the viewport.",
      "Visible rectangles are red [1,1,4,2], then blue [3,0,2,3], radius zero.",
      "No Text, Image, opacity or rounded commands; balanced allowed scopes only."])
html("html-fixed-child-escapes-scrolling-clip", frame(8, 4, document=(0, -1)),
     "#clip{position:absolute;left:1px;top:1px;width:2px;height:2px;overflow:hidden;background:red}"
     "#green{position:absolute;left:1px;top:1px;width:2px;height:2px;background:lime}"
     "#fixed{position:fixed;left:5px;top:0;width:2px;height:1px;background:blue}",
     "<div id=clip><div id=green></div><div id=fixed></div></div>",
     ["WRRWWBBW", "WRGWWWWW", "WWWWWWWW", "WWWWWWWW"],
     ["Red box is [1,1,2,2] in document coordinates; green child is [2,2,2,2].",
      "A document clip [1,1,2,2] clips the green child.",
      "Blue fixed child is [5,0,2,1] in viewport coordinates inside a typed fixed scope.",
      "Fixed rendering escapes the ancestor clip; scope restoration remains balanced.",
      "No Text, Image, opacity or rounded commands."])
html("html-repeated-decoded-image-key", frame(6, 2),
     "img{position:absolute;display:block;width:4px;height:1px;border:0;padding:0}"
     "#a{left:0;top:0}#b{left:2px;top:1px}",
     "<img id=a src=two-pixels.png alt=''><img id=b src=two-pixels.png alt=''>",
     ["RRHHWW", "WWRRHH"],
     ["Both image commands use the same key two-pixels.png and one decoded 2x1 RGBA source.",
      "Decoded source bytes are [255,0,0,255,0,0,255,128] with no color transformation.",
      "Image rectangles are [0,0,4,1] then [2,1,4,1].",
      "Successful image loading suppresses the missing-image placeholder and alt text.",
      "No Text, opacity or rounded commands; background is white."])
html("html-unavailable-image-keeps-placeholder", frame(4, 2),
     "img{position:absolute;display:block;left:1px;top:0;width:2px;height:1px;border:0;padding:0}"
     "#blue{position:absolute;left:3px;top:1px;width:1px;height:1px;background:blue}",
     "<img src=deliberately-absent.png alt=''><div id=blue></div>",
     ["WSSW", "WWWB"],
     ["deliberately-absent.png is absent; no raster exists for its Image command key.",
      "Layout emits opaque placeholder [1,0,2,1] RGB(236,238,242), radius zero, followed by the legal missing Image command.",
      "The blue rectangle [3,1,1,1] remains visible after the missing image.",
      "Empty alt suppresses text; missing-load diagnostics are retained, not turned into a GPU fallback.",
      "The bridge itself performs no image retrieval or file opening."])
html("html-rounded-page-requires-full-cpu-fallback", frame(4, 3),
     "#blue{position:absolute;left:1px;top:0;width:2px;height:2px;background:blue;border-radius:1px}"
     "#red{position:absolute;left:0;top:2px;width:1px;height:1px;background:red}",
     "<div id=blue></div><div id=red></div>",
     ["WFFW", "WFFW", "RWWW"],
     ["Blue background rectangle is [1,0,2,2], radius one, with no border.",
      "Red rectangle is [0,2,1,1], radius zero; white canvas background covers the viewport.",
      "No earlier unsupported command changes the expected rounded-rectangle rejection reason."],
     "unsupported-rounded-rectangle")

direct("direct-command-cap-refuses-whole-frame", frame(1, 1),
       [rect(0, 0, 0, 0, [0, 0, 0, 0]) for _ in range(257)], ["W"],
       "command-budget", limit=256, supplied=257)
direct("direct-scope-cap-refuses-balanced-whole-frame", frame(1, 1),
       [op("PushFixed") for _ in range(33)] + [op("PopFixed") for _ in range(33)],
       ["W"], "scope-budget", limit=32, supplied_depth=33)


def chunk(kind, value):
    return struct.pack(">I", len(value)) + kind + value + struct.pack(">I", zlib.crc32(kind + value))


# Encode the authored source pixels directly. This is not an image decoder or renderer.
source = bytes([255, 0, 0, 255, 0, 0, 255, 128])
png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 1, 8, 6, 0, 0, 0))
       + chunk(b"IDAT", zlib.compress(b"\0" + source)) + chunk(b"IEND", b""))
(ROOT / "html" / "two-pixels.png").write_bytes(png)

for case in cases:
    w, h = case["frame"]["width"], case["frame"]["height"]
    assert len(case["literal_rows"]) == h and all(len(row) == w for row in case["literal_rows"])
    pixels = [PALETTE[letter] for row in case["literal_rows"] for letter in row]
    rgb = b"".join(bytes.fromhex(pixel) for pixel in pixels)
    words = b"".join(int(pixel, 16).to_bytes(4, "little") for pixel in pixels)
    case["expected_rgb_hex_row_major"] = pixels
    case["expected_rgb_bytes_sha256"] = hashlib.sha256(rgb).hexdigest()
    case["expected_packed_rgb_le_sha256"] = hashlib.sha256(words).hexdigest()
    case["expected_cpu_exhausted"] = False
    path = ROOT / "expected" / (case["name"] + ".rgb")
    path.parent.mkdir(exist_ok=True)
    with path.open("xb") as out:
        out.write(rgb)
    case["expected_rgb_file"] = str(path.relative_to(ROOT))

assert len(cases) == 16
assert sum(case["expected_path"] == "gpu" for case in cases) == 9
assert sum(case["input_kind"] == "worker-html" for case in cases) == 5
data = dict(schema=1, status="independent draft; no engine outcomes", palette=PALETTE,
            cases=cases, counts=dict(cases=16, direct=11, worker_html=5, gpu=9, cpu_fallback=7,
                                    rgb_bytes=sum(len(c["expected_rgb_hex_row_major"]) * 3 for c in cases),
                                    packed_rgb_bytes=sum(len(c["expected_rgb_hex_row_major"]) * 4 for c in cases)))
with (ROOT / "fixtures.json").open("x") as output:
    json.dump(data, output, indent=2)
    output.write("\n")
print(json.dumps(data["counts"]))
