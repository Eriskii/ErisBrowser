#!/usr/bin/env python3
"""Offline verification/rebuild of five pinned IANA Date test fixtures.

Default/--check verifies retained bytes and independently derived transition
anchors. --rebuild additionally builds the pinned upstream zic in a temporary
directory and compares its outputs. No network, host-zone lookup or Rust engine.
"""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import signal
import struct
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'tests/fixtures/date-zones'
MANIFEST_SHA256 = 'dd03e5575a4ed8a9713e3d02aea01241de14e3302b9131c51ab9a46c1423e4fc'
ARCHIVES = {
    'tzdata2026d.tar.gz': (479409, '0cb2aa8e333c3dc049badc42a0c61f21987b8cd44e107fa900bad764aacc7767'),
    'tzcode2026d.tar.gz': (328712, '2f5c9f7fe29e6b8cb863583667884b8ce17b0a485355a054b591c6bdfcd81791'),
}
ZONES = ('America/New_York', 'Australia/Lord_Howe', 'Pacific/Apia', 'Europe/Dublin', 'Asia/Kathmandu')
REGIONS = ('northamerica', 'australasia', 'europe', 'asia')
MAX_ARCHIVE = 1_048_576
MAX_EXPANDED = 4_194_304
MAX_MEMBER = 524_288
MAX_MEMBERS = 128
MAX_ZONE = 16_384
MAX_LOG = 65_536


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_bounded(path, limit):
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'not a regular non-symlink file: {path}')
    with path.open('rb') as source:
        data = source.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f'file exceeds bound: {path}')
    return data


def archive_members(data):
    """Expand at most 4 MiB, never extract paths or follow archive links."""
    if len(data) > MAX_ARCHIVE:
        raise ValueError('compressed archive exceeds bound')
    with gzip.GzipFile(fileobj=io.BytesIO(data)) as source:
        plain = source.read(MAX_EXPANDED + 1)
    if len(plain) > MAX_EXPANDED:
        raise ValueError('expanded archive exceeds bound')
    result = {}
    with tarfile.open(fileobj=io.BytesIO(plain), mode='r:') as source:
        for entry in source:
            if (len(result) >= MAX_MEMBERS or not entry.isfile()
                    or not entry.name or entry.name in ('.', '..')
                    or '/' in entry.name or '\\' in entry.name or entry.name in result
                    or entry.size > MAX_MEMBER):
                raise ValueError('invalid, duplicate or excessive archive member')
            stream = source.extractfile(entry)
            if stream is None:
                raise ValueError('missing archive member')
            content = stream.read(MAX_MEMBER + 1)
            if len(content) != entry.size:
                raise ValueError('archive member length mismatch')
            result[entry.name] = content
    return result


def tzif_records(data):
    """Small independent structural reader used only on fixture data."""
    if len(data) > MAX_ZONE:
        raise ValueError('TZif fixture exceeds bound')
    at = 0
    result = None
    for width in (4, 8):
        if data[at:at + 5] != b'TZif2' or at + 44 > len(data):
            raise ValueError('expected complete fat TZif2')
        utc, standard, leaps, count, kinds, chars = struct.unpack_from('>6I', data, at + 20)
        if (leaps != 0 or not 1 <= kinds <= 256 or count > 8192 or chars > 4096
                or utc not in (0, kinds) or standard not in (0, kinds)):
            raise ValueError('unexpected TZif fixture counts')
        at += 44
        end = at + count * (width + 1) + kinds * 6 + chars + standard + utc
        if end > len(data):
            raise ValueError('truncated TZif fixture')
        code = 'i' if width == 4 else 'q'
        transitions = list(struct.unpack_from('>' + code * count, data, at))
        at += count * width
        indices = data[at:at + count]
        at += count
        types = [struct.unpack_from('>iBB', data, at + n * 6) for n in range(kinds)]
        at = end
        if (any(a >= b for a, b in zip(transitions, transitions[1:]))
                or any(index >= kinds for index in indices)):
            raise ValueError('invalid TZif transition order/type')
        result = (transitions, indices, types)
    if len(data) < at + 2 or data[at] != 10 or data[-1] != 10:
        raise ValueError('missing TZif footer')
    footer = data[at + 1:-1].decode('ascii')
    if not footer or '\n' in footer or '\0' in footer:
        raise ValueError('invalid TZif footer')
    return (*result, footer)


def utc_second(civil):
    """March-era integer arithmetic, independent of acquisition's Jan formula."""
    year, month, day, hour, minute, second = civil
    year -= month <= 2
    era = year // 400
    yoe = year - era * 400
    mp = month + (-3 if month > 2 else 9)
    days = era * 146097 + 365 * yoe + yoe // 4 - yoe // 100 + (153 * mp + 2) // 5 + day - 1 - 719468
    return days * 86400 + hour * 3600 + minute * 60 + second


def verify(directory):
    raw = read_bounded(directory / 'manifest.json', MAX_LOG)
    if digest(raw) != MANIFEST_SHA256:
        raise ValueError('pinned manifest hash mismatch')
    manifest = json.loads(raw)
    archives = {}
    for name, (size, sha) in ARCHIVES.items():
        content = read_bounded(directory / 'upstream' / name, MAX_ARCHIVE)
        if len(content) != size or digest(content) != sha:
            raise ValueError(f'archive pin mismatch: {name}')
        archives[name] = archive_members(content)
        if archives[name]['version'] != b'2026d\n':
            raise ValueError('archive version mismatch')
    for record in manifest['members']:
        content = archives[record['archive']][record['member']]
        if len(content) != record['bytes'] or digest(content) != record['sha256']:
            raise ValueError('selected source member mismatch')
    license_bytes = read_bounded(directory / 'LICENSE', MAX_LOG)
    if any(archive['LICENSE'] != license_bytes for archive in archives.values()):
        raise ValueError('unchanged license mismatch')
    records = {}
    if tuple(item['path'] for item in manifest['fixtures']) != ZONES:
        raise ValueError('zone inventory mismatch')
    for record in manifest['fixtures']:
        content = read_bounded(directory / record['path'], MAX_ZONE)
        if len(content) != record['bytes'] or digest(content) != record['sha256']:
            raise ValueError('compiled zone pin mismatch')
        decoded = tzif_records(content)
        if decoded[-1] != record['footer']:
            raise ValueError('footer mismatch')
        records[record['path']] = decoded
    expectation = manifest['expectations']
    raw = read_bounded(directory / expectation['path'], MAX_LOG)
    if len(raw) != expectation['bytes'] or digest(raw) != expectation['sha256']:
        raise ValueError('independent expectation pin mismatch')
    for anchor in json.loads(raw)['transitions']:
        second = utc_second(anchor['utc_civil'])
        if second != anchor['utc_second']:
            raise ValueError('independent calendar disagreement')
        transitions, indices, types, _ = records[anchor['zone']]
        index = transitions.index(second)
        before = types[indices[index - 1] if index else 0][0]
        after = types[indices[index]][0]
        if (before, after) != (anchor['before_offset_seconds'], anchor['after_offset_seconds']):
            raise ValueError('compiled transition differs from source-derived anchor')
    return manifest, archives


def run_bounded(args, cwd):
    """Finite compiler deadline, disk-backed diagnostics, whole-group cleanup."""
    if os.name != 'posix':
        raise ValueError('rebuild requires POSIX process-group cleanup')
    with tempfile.TemporaryFile() as log:
        env = dict(os.environ, LC_ALL='C')
        process = subprocess.Popen(args, cwd=cwd, env=env, stdout=log, stderr=log, start_new_session=True)
        try:
            status = process.wait(timeout=25)
        finally:
            # Also clean descendants if the group leader exited before them.
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=5)
        log.seek(0)
        output = log.read(MAX_LOG + 1)
        if len(output) > MAX_LOG:
            raise ValueError('compiler diagnostic output exceeds bound')
        if status:
            raise ValueError(f'compiler exited {status}: {output.decode(errors="replace")}')
        return output.decode('utf-8')


def rebuild(directory, compiler='cc'):
    manifest, archives = verify(directory)
    with tempfile.TemporaryDirectory(prefix='eris-date-zones-') as temporary:
        work = Path(temporary)
        for name in ('zic.c', 'private.h', 'tzfile.h'):
            (work / name).write_bytes(archives['tzcode2026d.tar.gz'][name])
        for name in REGIONS:
            (work / name).write_bytes(archives['tzdata2026d.tar.gz'][name])
        for name, content in manifest['build']['generated_headers'].items():
            (work / name).write_text(content)
        run_bounded([compiler, '-std=c17', '-O2', '-o', 'zic', 'zic.c'], work)
        version = run_bounded([str(work / 'zic'), '--version'], work).strip()
        if version != manifest['build']['zic_version']:
            raise ValueError('rebuilt zic version mismatch')
        run_bounded([str(work / 'zic'), '-b', 'fat', '-d', 'out', *REGIONS], work)
        for name in ZONES:
            if read_bounded(work / 'out' / name, MAX_ZONE) != read_bounded(directory / name, MAX_ZONE):
                raise ValueError(f'TZif rebuild differs: {name}')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group()
    action.add_argument('--check', action='store_true', help='offline retained-byte verification (default)')
    action.add_argument('--rebuild', action='store_true', help='compile pinned zic and compare fixture bytes')
    parser.add_argument('--compiler', default='cc', help='C17 compiler executable for --rebuild')
    parser.add_argument('--directory', type=Path, default=DIRECTORY)
    args = parser.parse_args()
    if args.rebuild:
        rebuild(args.directory, args.compiler)
    else:
        verify(args.directory)
    print('IANA 2026d: 2 release archives, 5 exact TZif fixtures, 10 independent transition anchors verified')


if __name__ == '__main__':
    main()
