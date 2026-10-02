"""Bounded validation of same-snapshot worker/CPU/Vulkan differential records.

Pixel references are produced by Canvas from the captured commands. This is
not an independent web-layout or font-correctness oracle.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re

from browser_protocol import _decode, validate_listing

ROOT = Path(__file__).resolve().parent
DEFAULT_FIXTURES = ROOT / 'worker-text-fixtures'
FREEZE_SHA256 = 'bae728bb77043263235ab8ee6eda966c27f1edb0740abbd7f6312c8822542504'
NUM = r'(?:0|[1-9][0-9]{0,9})'
OPTION = rf'(?:none|{NUM})'
NAME = r'[a-z0-9-]{1,128}'
BASE_FIELDS = ('width', 'height', 'commands', 'image_entries', 'text_runs', 'bytes', 'scalars')
PLAN_FIELDS = ('lowered_commands', 'unique_sources', 'referenced_sources', 'total_rgba_bytes',
               'referenced_rgba_bytes', 'missing_images', 'occurrences', 'masks', 'coverage',
               'cold_work', 'scratch', 'rows', 'cpu_upper', 'gpu_upper', 'draws',
               'invocations', 'gpu_buffers')
READY = re.compile(
    rf'TEXT_READY (?P<name>{NAME}) path=(?P<path>gpu|cpu-fallback) '
    rf'reason=(?P<reason>[a-z0-9-]+) command_index=(?P<command_index>{OPTION}) '
    + ' '.join(rf'{name}=(?P<{name}>{NUM})' for name in BASE_FIELDS) + ' '
    + ' '.join(rf'{name}=(?P<{name}>{OPTION})' for name in PLAN_FIELDS)
    + rf' reference_bytes=(?P<reference_bytes>{NUM}) cpu_passes=2 cold_warm=true '
      r'reference=canvas-snapshot pixels_le_hex=(?P<pixels>[0-9a-f]+)')
SNAPSHOT = re.compile(rf'SNAPSHOT ({NAME}) bytes=({NUM}) hex=([0-9a-f]+)')
CAPTURE = re.compile(rf'TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=({NUM}) owned_children=0')
CPU = re.compile(rf'TEXT_CPU_COMPLETE cases=7 gpu_admitted=6 fallback=1 cpu_passes=14 reference_bytes=({NUM}) gpu_comparisons=0 differential=true')
PASS = re.compile(rf'TEXT_PASS ({NAME}) width=({NUM}) height=({NUM}) compared_bytes=({NUM}) exact=true')
COMPLETE = re.compile(rf'WORKER_TEXT_COMPLETE adapter=({NUM}) cases=7 gpu=6 fallback=1 gpu_compared_bytes=({NUM}) fallback_reference_bytes=({NUM}) differential=true custom_wgsl=true')


def require(condition: bool, message: str):
    if not condition:
        raise ValueError(message)


def _read(path: Path, cap: int) -> bytes:
    with path.open('rb') as source:
        data = source.read(cap + 1)
    require(len(data) <= cap, f'fixture size bound: {path.name}')
    return data


def load_fixtures(directory: Path = DEFAULT_FIXTURES) -> dict:
    directory = directory.resolve()
    raw = _read(directory / 'freeze.json', 65536)
    require(hashlib.sha256(raw).hexdigest() == FREEZE_SHA256, 'worker fixture freeze identity')
    ledger = json.loads(raw)
    require(isinstance(ledger['files'], list) and len(ledger['files']) <= 32, 'fixture file count')
    seen = set()
    for row in ledger['files']:
        name = row['path']
        path = (directory / name).resolve()
        require(path.is_relative_to(directory) and name not in seen, 'fixture path or duplicate')
        seen.add(name)
        data = _read(path, 262144)
        require(len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'],
                f'worker fixture changed: {name}')
    require('inventory.json' in seen, 'inventory missing from fixture binding')
    inventory = json.loads(_read(directory / 'inventory.json', 262144))
    require(len(inventory['cases']) == 7, 'worker fixture count')
    return {'cases': inventory['cases'], 'binding': {'sha256': FREEZE_SHA256, 'files': len(seen)}}


def _match(pattern, line: str, label: str):
    result = pattern.fullmatch(line)
    require(result is not None, f'invalid {label} record')
    return result


def _lines(stdout: bytes) -> list[str]:
    require(isinstance(stdout, bytes) and 0 < len(stdout) <= 2 * 1024 * 1024, 'stdout byte bound')
    require(stdout.endswith(b'\n') and b'\r' not in stdout and b'\0' not in stdout, 'stdout framing')
    try:
        lines = stdout[:-1].decode('utf-8', errors='strict').split('\n')
    except UnicodeDecodeError as exc:
        raise ValueError('stdout encoding') from exc
    require(len(lines) <= 64 and all(0 < len(line) <= 150000 for line in lines), 'stdout record bound')
    return lines


def _snapshot_observations(snapshot: dict, case: dict, directory: Path):
    """Fixture-specific semantic checks are shared with the Rust fixture contract."""
    request, frame = case['request'], case['frame']
    require((snapshot['width'], snapshot['height']) == (frame['width'], frame['height']), 'snapshot frame')
    require(snapshot['generation'] == request['generation'] and
            snapshot['processed_edit_sequence'] == request['expected_edit_sequence'] and
            snapshot['task_state'] == 0, 'snapshot request identity')
    require(snapshot['title'] == request['expected_title'] and
            snapshot['url'] == (directory / case['html_file']).resolve().as_uri(), 'snapshot title/URL')
    # The bounded EWB1 decoder retains diagnostics verbatim. Successful confined
    # workers also report their normal isolation status; empty is not required.
    caller = tuple(frame['caller_clip'])
    clip, fixed, stack = caller, False, []
    texts, drawing_order = [], []
    for command in snapshot['commands']:
        kind = command['kind']
        if kind in ('PushClip', 'PushFixed', 'PushOpacity'):
            require(len(stack) < 32, 'snapshot scope bound')
            stack.append((kind, clip, fixed))
            if kind == 'PushFixed':
                clip, fixed = caller, True
            elif kind == 'PushClip':
                x, y, w, h = command['rect']
                dx, dy = frame['viewport_offset'] if fixed else frame['document_offset']
                x, y = x + dx, y + dy
                left, top = max(clip[0], x), max(clip[1], y)
                right, bottom = min(clip[0] + clip[2], x + w), min(clip[1] + clip[3], y + h)
                clip = (left, top, max(0, right - left), max(0, bottom - top))
            continue
        if kind in ('PopClip', 'PopFixed', 'PopOpacity'):
            require(bool(stack) and stack[-1][0] == 'Push' + kind[3:], 'snapshot scope balance')
            _, clip, fixed = stack.pop()
            continue
        if kind == 'Text':
            texts.append({**{key: command[key] for key in ('text', 'bold', 'italic', 'monospace', 'size')},
                          'rgba': command['color'], 'fixed': fixed, 'clipped': clip != caller})
        if (kind == 'Rect' and command['radius'] == 0 and
                (command['color'][3] == 0 or 0 in command['rect'][2:])):
            continue
        drawing_order.append(kind)
    require(not stack, 'unclosed snapshot scope')
    require(texts == case['texts'], 'snapshot text content/style/scope differs from fixture')
    require(drawing_order == case['drawing_order'], 'snapshot paint order differs from fixture')
    expected_images = sorted(case['images'], key=lambda item: item['key'])
    require(len(snapshot['image_keys']) == len(expected_images) and
            len(snapshot['sources']) == len(expected_images), 'snapshot source inventory')
    for key, expected in zip(snapshot['image_keys'], expected_images):
        source = snapshot['sources'][key['source_id']]
        require(key['key'] == expected['key'] and source['width'] == expected['width'] and
                source['height'] == expected['height'] and
                source['rgba_hex'] == bytes(expected['rgba']).hex(), 'snapshot decoded image identity')
        references = sum(c['kind'] == 'Image' and c['key'] == key['key'] for c in snapshot['commands'])
        require(references == expected['references'], 'snapshot image reference count')


def _ready(line: str, case: dict, snapshot: dict) -> dict:
    match = _match(READY, line, 'TEXT_READY')
    row = match.groupdict()
    for key in (*BASE_FIELDS, *PLAN_FIELDS, 'command_index', 'reference_bytes'):
        row[key] = None if row[key] == 'none' else int(row[key])
    require(row['name'] == case['name'], 'worker ready order')
    require(row['path'] == case['expected_route'] and
            row['reason'] == (case['expected_fallback'] or 'none'), 'worker expected route/reason')
    require((row['width'], row['height']) == (snapshot['width'], snapshot['height']), 'ready frame size')
    require(0 < row['width'] <= 160 and 0 < row['height'] <= 80, 'suite viewport bound')
    commands, keys, sources = snapshot['commands'], snapshot['image_keys'], snapshot['sources']
    text = [c for c in commands if c['kind'] == 'Text']
    require(row['commands'] == len(commands) and row['image_entries'] == len(keys), 'ready snapshot counts')
    require(row['text_runs'] == len(text) and 0 < len(text) <= 256, 'ready text run count')
    require(row['bytes'] == sum(len(c['text'].encode('utf-8')) for c in text) <= 65536,
            'ready UTF-8 bytes')
    require(row['scalars'] == sum(len(c['text']) for c in text) <= 4096, 'ready scalar count')
    area = row['width'] * row['height']
    require(row['reference_bytes'] == area * 4 and len(row['pixels']) == area * 8, 'CPU reference size')
    pixels = bytes.fromhex(row.pop('pixels'))
    require(not any(pixels[3::4]), 'CPU reference high byte')
    row['reference_sha256'] = hashlib.sha256(pixels).hexdigest()
    row['reference_kind'] = 'same-snapshot Canvas differential; not independent golden'
    if row['path'] == 'cpu-fallback':
        require(row['reason'] != 'none' and row['command_index'] is not None and
                row['command_index'] < len(commands), 'fallback reason/index')
        require(all(row[k] is None for k in PLAN_FIELDS), 'fallback must have no partial GPU plan')
        rejected = commands[row['command_index']]
        require(rejected['kind'] == 'Rect' and rejected['radius'] > 0,
                'fallback index must identify the unsupported rounded rectangle')
        return row
    require(row['reason'] == 'none' and row['command_index'] is None, 'GPU route reason/index')
    require(all(row[k] is not None for k in PLAN_FIELDS), 'GPU plan metadata missing')
    nontext = len(commands) - len(text)
    require(row['lowered_commands'] == nontext + row['occurrences'] <= 256, 'lowered command count')
    require(row['unique_sources'] == len(sources) and
            0 <= row['referenced_sources'] <= row['unique_sources'], 'source counts')
    require(row['total_rgba_bytes'] == sum(s['width'] * s['height'] * 4 for s in sources),
            'source bytes')
    by_key = {key['key']: key['source_id'] for key in keys}
    image_commands = [c for c in commands if c['kind'] == 'Image']
    referenced = {by_key[c['key']] for c in image_commands if c['key'] in by_key}
    require(row['referenced_sources'] == len(referenced) and
            row['referenced_rgba_bytes'] == sum(sources[i]['width'] * sources[i]['height'] * 4
                                              for i in referenced) and
            row['missing_images'] == sum(c['key'] not in by_key for c in image_commands),
            'referenced and missing image accounting')
    require(0 <= row['referenced_rgba_bytes'] <= row['total_rgba_bytes'] <= 8 * 1024 * 1024,
            'source byte bound')
    require(row['missing_images'] <= sum(c['kind'] == 'Image' for c in commands), 'missing image count')
    require(0 <= row['masks'] <= row['occurrences'] <= row['scalars'] and
            row['masks'] + row['unique_sources'] <= 256, 'mask inventory bound')
    require(0 <= row['coverage'] <= 262144 and
            row['coverage'] <= row['cold_work'] <= row['coverage'] + row['masks'], 'mask coverage work')
    require(row['scratch'] % 4 == 0 and row['scratch'] <= (262144 + 4) * 4, 'mask scratch bound')
    require(row['masks'] != 0 or (row['coverage'] == row['cold_work'] == row['scratch'] == 0),
            'empty mask inventory storage')
    require(row['rows'] <= min(65536, row['occurrences'] * 1024), 'row table bound')
    require(nontext * area + row['occurrences'] <= row['cpu_upper'] <=
            min(32000000, max(1000000, area * 16)), 'CPU paint allowance')
    require(row['cold_work'] <= row['cpu_upper'] - nontext * area, 'cold work exceeds occurrence work')
    gpu_upper = (8 * area + row['total_rgba_bytes'] +
                 4 * (row['coverage'] + row['rows'] +
                      len(image_commands) * (row['width'] + row['height'])) +
                 256 * (row['lowered_commands'] + 1))
    require(row['gpu_upper'] == gpu_upper, 'GPU upper-bound accounting')
    require(area * 8 + 256 <= row['gpu_buffers'] <= row['gpu_upper'] <= 1048576,
            'GPU storage bounds')
    require(1 <= row['draws'] <= 257 and row['gpu_buffers'] % 4 == 0 and
            row['gpu_buffers'] >= area * 8 + row['draws'] * 256, 'GPU draw storage')
    clear_work = ((row['width'] + 7) // 8) * ((row['height'] + 7) // 8) * 64
    require(max(clear_work, row['draws'] * 64) <= row['invocations'] <= 4000000 and
            row['invocations'] % 64 == 0, 'GPU invocation bound')
    return row


def _prepare(lines: list[str], directory: Path) -> tuple[list[dict], dict]:
    inputs = load_fixtures(directory)
    require(len(lines) >= 16, 'truncated worker preparation')
    ready = []
    for index, case in enumerate(inputs['cases']):
        name, size, encoded = _match(SNAPSHOT, lines[index * 2], 'SNAPSHOT').groups()
        require(name == case['name'] and 0 < int(size) <= 65536 and len(encoded) == int(size) * 2,
                'snapshot identity or size')
        data = bytes.fromhex(encoded)
        snapshot = _decode(data)
        _snapshot_observations(snapshot, case, directory)
        row = _ready(lines[index * 2 + 1], case, snapshot)
        row['snapshot_sha256'] = hashlib.sha256(data).hexdigest()
        row['snapshot_bytes'] = len(data)
        row['diagnostics'] = snapshot['diagnostics']
        ready.append(row)
    tasks = int(_match(CAPTURE, lines[14], 'TEXT_CAPTURE_COMPLETE')[1])
    require(1 <= tasks <= 64, 'own task bound')
    require(sum(r['path'] == 'gpu' for r in ready) == 6, 'suite route counts')
    return ready, {'cases': 7, 'worker_cases': 7, 'gpu_admitted': 6, 'fallback': 1,
                   'cpu_passes': 14, 'cpu_compared_bytes': 2 * sum(r['reference_bytes'] for r in ready),
                   'reference_bytes': sum(r['reference_bytes'] for r in ready),
                   'retained_gpu_bytes': sum(r['gpu_buffers'] or 0 for r in ready),
                   'fixture_binding': inputs['binding'], 'frames': ready,
                   'reference_kind': 'same-snapshot Canvas differential; not independent golden'}


def validate_cpu(stdout: bytes, fixtures: Path = DEFAULT_FIXTURES) -> dict:
    lines = _lines(stdout)
    require(len(lines) == 16, 'CPU phase record count')
    _, result = _prepare(lines, fixtures)
    count = int(_match(CPU, lines[15], 'TEXT_CPU_COMPLETE')[1])
    require(count == result['reference_bytes'], 'CPU reference total')
    result.update(gpu_comparisons=0, gpu_compared_bytes=0)
    return result


def validate_run(stdout: bytes, adapters: list[str], index: int,
                 fixtures: Path = DEFAULT_FIXTURES) -> dict:
    require(0 <= index < len(adapters) <= 16, 'selected adapter bound')
    require(validate_listing(('\n'.join(adapters) + '\n').encode()) == adapters, 'adapter inventory')
    lines = _lines(stdout)
    require(len(lines) == 22 + len(adapters), 'GPU phase record count')
    ready, result = _prepare(lines, fixtures)
    require(lines[15:15 + len(adapters)] == adapters, 'adapter inventory changed')
    gpu_rows = [r for r in ready if r['path'] == 'gpu']
    for line, row in zip(lines[15 + len(adapters):-1], gpu_rows):
        name, width, height, size = _match(PASS, line, 'TEXT_PASS').groups()
        require(name == row['name'] and (int(width), int(height), int(size)) ==
                (row['width'], row['height'], row['reference_bytes']), 'GPU pass identity/size')
    adapter, gpu_bytes, fallback_bytes = map(int, _match(COMPLETE, lines[-1], 'WORKER_TEXT_COMPLETE').groups())
    require(adapter == index and gpu_bytes == sum(r['reference_bytes'] for r in gpu_rows) and
            fallback_bytes == sum(r['reference_bytes'] for r in ready if r['path'] == 'cpu-fallback'),
            'GPU and fallback totals')
    result.update(adapter=index, gpu_comparisons=6, gpu_compared_bytes=gpu_bytes,
                  fallback_reference_bytes=fallback_bytes)
    return result
