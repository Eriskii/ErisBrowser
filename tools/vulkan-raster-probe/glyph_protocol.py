"""Strict glyph-check protocol; literal inputs and parent-font references stay distinct.

Dynamic glyph/raster/planner metadata is checked for bounded consistency, never
reported as an independent exact counter oracle. No child process is launched.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re

from run_host import inventory

ROOT = Path(__file__).resolve().parent
DEFAULT_FIXTURES = ROOT / 'glyph-fixtures'
LITERAL_SHA256 = '1a23a33fc1e7e45b3bb825583316cee889eab46a4111af85485b717fb009b5aa'
FONT_SHA256 = 'ecb63ecb6ea172278eaed57acc4fe5f27ce6a8c4567aa35392be4fe7511cf7c1'
PARENT = '5ad2cfd3582f92f8368a0a0fa2fd3bb7dd7af2f0'
MAX_OUTPUT = 2 * 1024 * 1024
MAX_GPU_BYTES = 1_048_576
MAX_INVOCATIONS = 4_000_000
MAX_PIXELS = 320 * 240
NUM = r'(0|[1-9][0-9]{0,9})'
NAME = r'([a-z0-9-]{1,128})'
META_FIELDS = ('bytes', 'scalars', 'visited', 'occurrences', 'masks', 'coverage',
               'cold_work', 'scratch', 'rows', 'cpu_upper', 'gpu_upper')
META = re.compile('FONT_META ' + NAME + ''.join(' ' + k + '=' + NUM for k in META_FIELDS) + ' cold_warm=true')
READY_FIELDS = ('width', 'height', 'compared_bytes', 'draws', 'invocations', 'gpu_buffers')
READY = re.compile('GLYPH_READY ' + NAME + r' population=(literal-mask|font-reference)' +
                   ''.join(' ' + k + '=' + NUM for k in READY_FIELDS))
PREPARED = re.compile('GLYPH_PREPARED cases=26 literal=12 font=14 retained_gpu_bytes=' + NUM +
                      ' retained_reference_bytes=' + NUM + ' worker_cases=0')
PASS = re.compile('GLYPH_PASS ' + NAME + r' population=(literal-mask|font-reference)' +
                  ' width=' + NUM + ' height=' + NUM + ' compared_bytes=' + NUM + ' exact=true')


def _read(path: Path, cap: int) -> bytes:
    if not path.is_file() or path.stat().st_size > cap:
        raise ValueError(f'input missing or over bound: {path.name}')
    with path.open('rb') as stream:
        data = stream.read(cap + 1)
    if len(data) > cap:
        raise ValueError('input grew beyond bound')
    return data


def _json(data: bytes) -> dict:
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate JSON key')
            result[key] = value
        return result
    try:
        value = json.loads(data, object_pairs_hook=pairs,
                           parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError('invalid JSON input') from exc
    if not isinstance(value, dict):
        raise ValueError('JSON object required')
    return value


def load_fixtures(directory: Path = DEFAULT_FIXTURES) -> dict:
    directory = Path(directory)
    rows = []
    docs = []
    for filename, expected in [('literal-fixtures.json', LITERAL_SHA256), ('font-inventory.json', FONT_SHA256)]:
        data = _read(directory / filename, 2 * 1024 * 1024)
        if hashlib.sha256(data).hexdigest() != expected:
            raise ValueError(f'frozen fixture bytes changed: {filename}')
        docs.append(_json(data))
        rows.append({'path': filename, 'bytes': len(data), 'sha256': expected})
    literals, fonts = docs[0]['pixel_cases'], docs[1]['cases']
    if len(literals) != 12 or len(fonts) != 14 or docs[1]['parent'] != PARENT:
        raise ValueError('frozen fixture count/parent changed')
    cases = []
    for population, inputs in [('literal-mask', literals), ('font-reference', fonts)]:
        for source in inputs:
            f = source['frame']
            w, h = f['width'], f['height']
            if not 1 <= w <= 320 or not 1 <= h <= 240 or not re.fullmatch(NAME, source['name']):
                raise ValueError('invalid frozen frame/name')
            texts = [c['text'] for c in source['commands'] if c['kind'] == 'Text']
            cases.append({'name': source['name'], 'population': population, 'width': w, 'height': h,
                          'compared_bytes': w * h * 4,
                          'text_bytes': sum(len(t.encode('utf-8')) for t in texts),
                          'text_scalars': sum(len(t) for t in texts),
                          'nontext_commands': sum(c['kind'] != 'Text' for c in source['commands']),
                          'image_sources': len(source['images'])})
    if len({c['name'] for c in cases}) != 26:
        raise ValueError('duplicate frozen name')
    return {'cases': cases, 'fonts': cases[12:], 'bindings': rows,
            'compared_bytes': sum(c['compared_bytes'] for c in cases)}


def check_baselines(directory: Path, manifest_sha256: str, fixtures: Path = DEFAULT_FIXTURES) -> dict:
    """Require a separately pinned parent manifest and all exact packed frames."""
    if not re.fullmatch(r'[0-9a-f]{64}', manifest_sha256 or ''):
        raise ValueError('missing pinned parent-baseline manifest hash')
    directory = Path(directory).resolve()
    data = _read(directory / 'freeze.json', 256 * 1024)
    if hashlib.sha256(data).hexdigest() != manifest_sha256:
        raise ValueError('parent baseline manifest changed')
    doc = _json(data)
    if (doc.get('schema') != 1 or doc.get('parent') != PARENT or doc.get('cases') != 14 or
            doc.get('bytes') != 344064 or doc.get('encoding') != 'u32le-00RRGGBB'):
        raise ValueError('parent baseline identity or encoding changed')
    rows = doc.get('files')
    wanted = {c['name'] + '.rgb': c['compared_bytes'] for c in load_fixtures(fixtures)['fonts']}
    if not isinstance(rows, list) or len(rows) != len(wanted):
        raise ValueError('parent baseline inventory changed')
    seen = set()
    for row in rows:
        name = row.get('path')
        if name not in wanted or name in seen:
            raise ValueError('unknown, duplicate or escaping parent baseline path')
        seen.add(name)
        path = (directory / name).resolve()
        if not path.is_relative_to(directory):
            raise ValueError('parent baseline symlink escapes directory')
        pixels = _read(path, MAX_PIXELS * 4)
        if row.get('bytes') != wanted[name] or len(pixels) != wanted[name]:
            raise ValueError('parent baseline byte count changed')
        if hashlib.sha256(pixels).hexdigest() != row.get('sha256'):
            raise ValueError('parent baseline pixels changed')
        if any(pixels[3::4]):
            raise ValueError('parent baseline target high byte is not zero')
    if seen != set(wanted):
        raise ValueError('incomplete parent baseline inventory')
    return {'manifest_sha256': manifest_sha256, 'parent_commit': PARENT,
            'encoding': 'u32le-00RRGGBB', 'files': rows,
            'bytes': sum(wanted.values()), 'pixel_claim': 'reference-derived immutable-parent CPU'}


def _lines(stdout: bytes) -> list[str]:
    if not stdout or len(stdout) > MAX_OUTPUT or not stdout.endswith(b'\n'):
        raise ValueError('empty, truncated or oversized protocol')
    try:
        lines = stdout.decode('utf-8').split('\n')[:-1]
    except UnicodeError as exc:
        raise ValueError('invalid UTF-8 protocol') from exc
    if any(not line or '\r' in line or len(line) > 4096 for line in lines):
        raise ValueError('empty, oversized or noncanonical protocol line')
    return lines


def validate_listing(stdout: bytes) -> list[str]:
    lines = _lines(stdout)
    devices = inventory(lines)
    if lines != devices:
        raise ValueError('unexpected enumeration output')
    return devices


def _match(pattern, line, description):
    match = pattern.fullmatch(line)
    if not match:
        raise ValueError('invalid ' + description)
    return match.groups()


def _preparation(lines: list[str], fixtures: Path) -> tuple[int, dict]:
    inputs = load_fixtures(fixtures)
    if len(lines) < 41:
        raise ValueError('truncated preparation phases')
    metadata = []
    for line, case in zip(lines[:14], inputs['fonts']):
        name, *values = _match(META, line, 'FONT_META')
        m = dict(zip(META_FIELDS, map(int, values)))
        if name != case['name'] or m['bytes'] != case['text_bytes'] or m['scalars'] != case['text_scalars']:
            raise ValueError('font order or frozen input byte/scalar count changed')
        if not 0 <= m['masks'] <= m['occurrences'] <= m['visited'] <= m['scalars'] <= 4096:
            raise ValueError('font visit/occurrence/mask counts exceed bounds')
        if (m['masks'] + case['image_sources'] > 256 or
                m['occurrences'] + case['nontext_commands'] > 256 or
                m['bytes'] > 65536 or m['coverage'] > 262144 or m['rows'] > 65536):
            raise ValueError('font preparation table bound')
        if not m['coverage'] <= m['cold_work'] <= m['coverage'] + m['masks']:
            raise ValueError('font cold-work accounting inconsistent')
        if m['scratch'] > (262144 + 4) * 4 or m['scratch'] % 4:
            raise ValueError('font raster scratch bound')
        if m['rows'] > m['occurrences'] * 1024 or (m['masks'] == 0 and (m['coverage'] or m['scratch'])):
            raise ValueError('font row/storage consistency')
        area = case['width'] * case['height']
        cpu_min = area * case['nontext_commands'] + m['occurrences']
        cpu_cap = min(32000000, max(1000000, area * 16))
        if not cpu_min <= m['cpu_upper'] <= cpu_cap:
            raise ValueError('font CPU allowance bound')
        if not area * 8 + 256 <= m['gpu_upper'] <= MAX_GPU_BYTES:
            raise ValueError('font GPU allowance bound')
        metadata.append({'name': name, **m, 'cold_warm': True,
                         'claim': 'observed bounded metadata, not independent exact counter oracle'})
    ready = []
    for line, case in zip(lines[14:40], inputs['cases']):
        name, population, *values = _match(READY, line, 'GLYPH_READY')
        row = dict(zip(READY_FIELDS, map(int, values)))
        if name != case['name'] or population != case['population'] or any(row[k] != case[k] for k in ('width', 'height', 'compared_bytes')):
            raise ValueError('prepared frame identity or size changed')
        clear_work = ((row['width'] + 7) // 8) * ((row['height'] + 7) // 8) * 64
        if not 1 <= row['draws'] <= 257:
            raise ValueError('per-frame draw bound')
        if not max(clear_work, row['draws'] * 64) <= row['invocations'] <= MAX_INVOCATIONS or row['invocations'] % 64:
            raise ValueError('per-frame invocation bound')
        minimum_bytes = row['compared_bytes'] * 2 + row['draws'] * 256
        if not minimum_bytes <= row['gpu_buffers'] <= MAX_GPU_BYTES or row['gpu_buffers'] % 4:
            raise ValueError('per-frame GPU storage bound')
        if population == 'font-reference' and row['gpu_buffers'] > metadata[len(ready) - 12]['gpu_upper']:
            raise ValueError('actual GPU buffers exceed font preflight upper bound')
        ready.append({'name': name, 'population': population, **row})
    gpu, reference = map(int, _match(PREPARED, lines[40], 'GLYPH_PREPARED'))
    if gpu != sum(r['gpu_buffers'] for r in ready) or gpu > 26 * MAX_GPU_BYTES:
        raise ValueError('retained GPU summary differs from frame sums')
    if reference != inputs['compared_bytes'] or reference != sum(r['compared_bytes'] for r in ready) or reference > 26 * MAX_PIXELS * 4:
        raise ValueError('retained reference summary differs from frame sums')
    return 41, {'cases': 26, 'literal_cases': 12, 'font_cases': 14, 'worker_cases': 0,
                'compared_bytes': reference, 'retained_gpu_bytes': gpu,
                'metadata': metadata, 'ready': ready, 'fixture_bindings': inputs['bindings']}


def validate_cpu(stdout: bytes, fixtures: Path = DEFAULT_FIXTURES) -> dict:
    lines = _lines(stdout)
    pos, result = _preparation(lines, fixtures)
    if lines[pos:] != ['GLYPH_CPU_COMPLETE cases=26 literal=12 font=14 cpu_passes=28 exact=true']:
        raise ValueError('CPU completion differs or contains GPU/extra output')
    prepared_bytes = result.pop('compared_bytes')
    return {**result, 'mode': 'cpu-check', 'cpu_font_passes': 28, 'gpu_comparisons': 0,
            'retained_reference_bytes': prepared_bytes,
            'cpu_font_compared_bytes': 2 * sum(row['compared_bytes'] for row in result['ready'][12:]),
            'literal_cpu_comparisons': 0}


def validate_run(stdout: bytes, adapters: list[str], index: int, fixtures: Path = DEFAULT_FIXTURES) -> dict:
    if inventory(adapters) != adapters or not 0 <= index < len(adapters):
        raise ValueError('invalid expected adapter inventory/index')
    lines = _lines(stdout)
    pos, result = _preparation(lines, fixtures)
    if lines[pos:pos + len(adapters)] != adapters:
        raise ValueError('adapter enumeration changed or occurred before preparation')
    pos += len(adapters)
    if len(lines) != pos + 27:
        raise ValueError('missing or extra GPU completion rows')
    for line, expected in zip(lines[pos:pos + 26], result['ready']):
        name, population, width, height, compared = _match(PASS, line, 'GLYPH_PASS')
        if (name, population, int(width), int(height), int(compared)) != tuple(expected[k] for k in ('name', 'population', 'width', 'height', 'compared_bytes')):
            raise ValueError('GPU pass identity, order or byte count changed')
    if lines[-1] != f'GLYPH_COMPLETE adapter={index} cases=26 literal=12 font=14 exact=true custom_wgsl=true':
        raise ValueError('invalid final GPU summary')
    return {**result, 'mode': 'gpu', 'adapter': index, 'gpu_comparisons': 26}
