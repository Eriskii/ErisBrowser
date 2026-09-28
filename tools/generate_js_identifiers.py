#!/usr/bin/env python3
"""Generate private ECMAScript identifier tables from pinned local Unicode data.

Always offline. Updating the Unicode pin is an explicit source/provenance review;
this program never downloads or silently refreshes input files.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
VERSION = '18.0.0'
MANIFEST_SHA256 = 'c48363e393828c39a657940ab4fcc80cefbceb08fa1f6885a3d0cd415ec1ae58'
DATA = ROOT / 'tests/upstream/unicode' / VERSION
OUTPUT = ROOT / 'src/js_identifier.rs'
MAX_FILE = 2 * 1024 * 1024
MAX_MANIFEST = 16 * 1024
MAX_LINES = 20000
MAX_LINE = 1024
MAX_RECORDS = 4096
MAX_RANGES = 2048
SCALAR_LIMIT = 0x110000
PAGE_SCALARS = 256
PAGE_BYTES = 64
PAGE_COUNT = SCALAR_LIMIT // PAGE_SCALARS
MAX_UNIQUE_PAGES = 256
MAX_OUTPUT = 256 * 1024
EXPECTED_COUNTS = {'ID_Start': 158739, 'ID_Continue': 162100}
PINNED = {
    'DerivedCoreProperties.txt': (
        'https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt', 1159889,
        '09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9'),
    'ReadMe.txt': (
        'https://www.unicode.org/Public/18.0.0/ucd/ReadMe.txt', 729,
        'b0ea442f29dee90584aacd3665a6e1ce87ffc917a4d31f34e1916d2d6f7f9205'),
    'LICENSE.txt': (
        'https://www.unicode.org/license.txt', 1995,
        'e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96'),
    'Unicode18.0.0.html': (
        'https://www.unicode.org/versions/Unicode18.0.0/', 62838,
        '238585432d40b86c63666334939f5ab691340b8c7a67ea63327d12471950fb63'),
}
HEADER = '# DerivedCoreProperties-18.0.0.txt'
DATA_DATE = '# Date: 2026-08-07, 16:19:42 GMT'
FINAL_STATEMENT = ('This directory contains final data files for version 18.0.0 of the\n'
                   'Unicode Character Database.')
CAVEAT = ('The official landing page still carries a preliminary banner; final-data status '
          'is taken from the versioned ReadMe and matching data header, not inferred from that page.')


def read_bounded(path, limit):
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'input is not a regular non-symlink file: {path.name}')
    with path.open('rb') as source:
        data = source.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f'input exceeds byte limit: {path.name}')
    return data


def load_inputs(directory):
    """Verify provenance against independent code pins, not a mutable hash alone."""
    manifest_bytes = read_bounded(directory / 'manifest.json', MAX_MANIFEST)
    if hashlib.sha256(manifest_bytes).hexdigest() != MANIFEST_SHA256:
        raise ValueError('Unicode manifest byte integrity mismatch')
    manifest = json.loads(manifest_bytes)
    if manifest.get('format') != 1 or manifest.get('unicode_version') != VERSION:
        raise ValueError('unexpected Unicode manifest format or version')
    retrieved = datetime.datetime.fromisoformat(manifest['retrieved_at_utc'])
    if retrieved.utcoffset() != datetime.timedelta(0):
        raise ValueError('retrieval timestamp must carry UTC timezone')
    files = manifest['files']
    if not isinstance(files, list) or len(files) != len(PINNED):
        raise ValueError('Unicode manifest inventory differs from pin')
    retained = {}
    for entry in files:
        name = entry['path']
        if name not in PINNED or name in retained:
            raise ValueError('unexpected or duplicate Unicode manifest path')
        url, size, digest = PINNED[name]
        if (entry['url'], entry['resolved_url'], entry['bytes'], entry['sha256']) != (url, url, size, digest):
            raise ValueError(f'Unicode provenance differs from pin: {name}')
        data = read_bounded(directory / name, MAX_FILE)
        if len(data) != size or hashlib.sha256(data).hexdigest() != digest:
            raise ValueError(f'Unicode byte integrity mismatch: {name}')
        retained[name] = data
    expected_evidence = {'data_header': HEADER, 'data_date': DATA_DATE,
                         'readme_statement': FINAL_STATEMENT, 'landing_page_caveat': CAVEAT,
                         'landing_page_release_date': 'September 16, 2026'}
    if manifest['final_version_evidence'] != expected_evidence:
        raise ValueError('Unicode final-version provenance differs from retained evidence')
    if FINAL_STATEMENT.encode() not in retained['ReadMe.txt']:
        raise ValueError('versioned ReadMe does not identify final data')
    landing = retained['Unicode18.0.0.html']
    if b'preliminary draft page' not in landing or b'2026 September 16' not in landing:
        raise ValueError('retained landing-page caveat/date is missing')
    if not retained['LICENSE.txt'].startswith(b'UNICODE LICENSE V3\n'):
        raise ValueError('unexpected Unicode license')
    return retained


def parse_properties(data):
    """Validate exact ID properties, section totals and scalar ranges before merging."""
    if len(data) > MAX_FILE:
        raise ValueError('Unicode data exceeds byte limit')
    text = data.decode('utf-8')
    if not text.startswith(HEADER + '\n' + DATA_DATE + '\n'):
        raise ValueError('Unicode data header/version/date mismatch')
    lines = text.splitlines()
    if len(lines) > MAX_LINES:
        raise ValueError('Unicode data exceeds line limit')
    rows = {name: [] for name in EXPECTED_COUNTS}
    totals = {}
    sections = set()
    active = None
    for line in lines:
        if len(line) > MAX_LINE:
            raise ValueError('Unicode data exceeds line length limit')
        if line.startswith('# Derived Property: '):
            if active is not None and active not in totals:
                raise ValueError(f'missing property total: {active}')
            name = line.removeprefix('# Derived Property: ').strip()
            active = name if name in rows else None
            if active is not None:
                if active in sections:
                    raise ValueError(f'duplicate property section: {active}')
                sections.add(active)
            continue
        if active is not None and line.startswith('# Total code points:'):
            match = re.fullmatch(r'# Total code points: ([0-9]{1,7})', line)
            if not match or active in totals:
                raise ValueError(f'invalid or duplicate property total: {active}')
            totals[active] = int(match[1])
            continue
        record, _, comment = line.partition('#')
        record = record.strip()
        if not record:
            continue
        mentions_selected = re.search(r'\bID_(?:Start|Continue)\b', record)
        if active is None:
            if mentions_selected:
                raise ValueError('selected property record outside its section')
            continue
        if active in totals:
            raise ValueError(f'property record follows its total: {active}')
        match = re.fullmatch(r'([0-9A-F]{4,6})(?:\.\.([0-9A-F]{4,6}))?\s*;\s*(ID_Start|ID_Continue)', record)
        if not match or match[3] != active:
            raise ValueError(f'invalid selected property record: {record[:80]}')
        first = int(match[1], 16)
        last = int(match[2], 16) if match[2] else first
        if first > last or last > 0x10FFFF or (first <= 0xDFFF and last >= 0xD800):
            raise ValueError('selected property range is not Unicode scalars')
        previous = rows[active]
        if previous and first <= previous[-1][1]:
            raise ValueError('selected property ranges overlap or are unordered')
        if len(previous) >= MAX_RECORDS:
            raise ValueError('selected property record limit exceeded')
        declared = re.search(r'\[([0-9]+)\]', comment)
        if declared and int(declared[1]) != last - first + 1:
            raise ValueError('selected property record count mismatch')
        previous.append((first, last))
    if sections != rows.keys() or totals != EXPECTED_COUNTS:
        raise ValueError('missing or unexpected selected property sections/totals')
    merged = {}
    for name, ranges in rows.items():
        count = sum(last - first + 1 for first, last in ranges)
        if count != totals[name]:
            raise ValueError(f'selected property code-point count mismatch: {name}')
        compact = []
        for first, last in ranges:
            if compact and first == compact[-1][1] + 1:
                compact[-1] = (compact[-1][0], last)
            else:
                compact.append((first, last))
        if not compact or len(compact) > MAX_RANGES:
            raise ValueError('merged identifier range limit exceeded')
        merged[name] = compact
    part = iter(merged['ID_Continue'])
    current = next(part)
    for first, last in merged['ID_Start']:
        while current[1] < first:
            current = next(part, (0x110000, 0x110000))
        if current[0] > first or current[1] < last:
            raise ValueError('ID_Start is not a subset of ID_Continue')
    return rows, merged


RUST_API = r'''
// One bounded page-index read and one bounded bitmap-word read per lookup.
// ASCII takes its separate fast path. The parser charges this real lookup bound.
pub(crate) const IDENTIFIER_LOOKUP_WORK: usize = 2;

#[inline]
pub(crate) fn is_identifier_start(value: char) -> bool {
    if value.is_ascii() {
        return matches!(value, '$' | '_' | 'a'..='z' | 'A'..='Z');
    }
    membership(value, 0)
}

#[inline]
pub(crate) fn is_identifier_part(value: char) -> bool {
    if value.is_ascii() {
        return matches!(value, '$' | '_' | 'a'..='z' | 'A'..='Z' | '0'..='9');
    }
    matches!(value, '\u{200c}' | '\u{200d}') || membership(value, 1)
}

// Fixed scalar-domain index; page IDs and eight-word pages are generator-checked.
// Bits alternate ID_Start / ID_Continue. No loops, allocation or normalization.
#[inline]
fn membership(value: char, property: u32) -> bool {
    let scalar = value as u32;
    let page = PAGE_INDEX[(scalar >> 8) as usize] as usize;
    let bit = ((scalar & 0xff) << 1) | property;
    (IDENTIFIER_PAGES[page][(bit >> 6) as usize] >> (bit & 63)) & 1 != 0
}
'''

RUST_TESTS = r'''
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecmascript_additions_and_id_not_xid_examples() {
        assert_eq!(IDENTIFIER_LOOKUP_WORK, 2);
        for value in ['$', '_', 'A', 'z', '\u{37a}', '\u{2118}', '\u{10400}'] {
            assert!(is_identifier_start(value), "{value:?}");
            assert!(is_identifier_part(value), "{value:?}");
        }
        for value in [
            '0', '\u{300}', '\u{b7}', '\u{203f}', '\u{660}', '\u{200c}', '\u{200d}',
        ] {
            assert!(!is_identifier_start(value), "{value:?}");
            assert!(is_identifier_part(value), "{value:?}");
        }
        for value in ['\0', '\u{b2}', '\u{feff}', '\u{1f600}', '\u{10ffff}'] {
            assert!(!is_identifier_start(value), "{value:?}");
            assert!(!is_identifier_part(value), "{value:?}");
        }
    }

    #[test]
    fn every_scalar_matches_independently_read_pinned_property_records() {
        // Independent from the Python parser/generator: expand the original
        // property records directly into bitmaps, then exercise the real API.
        let data = include_str!("../tests/upstream/unicode/18.0.0/DerivedCoreProperties.txt");
        let mut start = vec![false; 0x110000];
        let mut part = vec![false; 0x110000];
        for line in data.lines() {
            let record = line.split('#').next().unwrap().trim();
            let mut fields = record.split(';').map(str::trim);
            let Some(range) = fields.next() else { continue };
            let bitmap = match fields.next() {
                Some("ID_Start") => &mut start,
                Some("ID_Continue") => &mut part,
                _ => continue,
            };
            let mut ends = range.split("..");
            let first = u32::from_str_radix(ends.next().unwrap(), 16).unwrap();
            let last = ends
                .next()
                .map_or(first, |end| u32::from_str_radix(end, 16).unwrap());
            for value in first..=last {
                assert!(char::from_u32(value).is_some());
                assert!(!bitmap[value as usize]);
                bitmap[value as usize] = true;
            }
        }
        assert_eq!(start.iter().filter(|&&value| value).count(), 158739);
        assert_eq!(part.iter().filter(|&&value| value).count(), 162100);
        for raw in 0..=0x10ffff {
            let Some(value) = char::from_u32(raw) else {
                continue;
            };
            assert_eq!(
                is_identifier_start(value),
                start[raw as usize] || matches!(value, '$' | '_'),
                "start U+{raw:04X}"
            );
            assert_eq!(
                is_identifier_part(value),
                part[raw as usize] || matches!(value, '$' | '\u{200c}' | '\u{200d}'),
                "part U+{raw:04X}"
            );
        }
    }
}
'''


def pack_pages(properties):
    """Pack two exact property bits per scalar and deduplicate in first-use order."""
    packed = bytearray(SCALAR_LIMIT // 4)
    for property_bit, name in enumerate(('ID_Start', 'ID_Continue')):
        for first, last in properties[name]:
            for scalar in range(first, last + 1):
                bit = scalar * 2 + property_bit
                packed[bit >> 3] |= 1 << (bit & 7)
    indices, pages, known = [], [], {}
    for number in range(PAGE_COUNT):
        page = bytes(packed[number * PAGE_BYTES:(number + 1) * PAGE_BYTES])
        if page not in known:
            if len(pages) >= MAX_UNIQUE_PAGES:
                raise ValueError('unique identifier bitmap page limit exceeded')
            known[page] = len(pages)
            pages.append(page)
        indices.append(known[page])
    if len(indices) != 4352 or any(len(page) != 64 for page in pages):
        raise ValueError('identifier bitmap layout does not cover the fixed scalar domain')
    if any(index > 255 or index >= len(pages) for index in indices):
        raise ValueError('identifier bitmap page index exceeds its bounded pool')
    return indices, pages


def render(inputs):
    rows, merged = parse_properties(inputs['DerivedCoreProperties.txt'])
    license_text = inputs['LICENSE.txt'].decode('utf-8').rstrip('\n')
    output = ['// @generated by tools/generate_js_identifiers.py; do not edit.',
              '// Unicode 18.0.0 ID_Start and ID_Continue; no normalization or XID approximation.',
              '// Source: https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt',
              '// Source SHA-256: ' + PINNED['DerivedCoreProperties.txt'][2],
              '// Provenance: tests/upstream/unicode/18.0.0/manifest.json',
              '// License copied verbatim as line comments below:', '//']
    output.extend('// ' + line if line else '//' for line in license_text.splitlines())
    output.append(RUST_API.rstrip())
    indices, pages = pack_pages(merged)
    for name in ('ID_Start', 'ID_Continue'):
        output.append(f'// {name}: {EXPECTED_COUNTS[name]:,} points; {len(rows[name])} records; {len(merged[name])} validated merged ranges.')
    output.extend(['', '// 256 scalar slots per page; interleaved ID_Start and ID_Continue bits.',
                   f'// {len(indices)} one-byte indices + {len(pages)} unique 64-byte pages = {len(indices) + len(pages) * PAGE_BYTES:,} bytes.',
                   '#[rustfmt::skip]', f'const PAGE_INDEX: [u8; {len(indices)}] = ['])
    for offset in range(0, len(indices), 32):
        output.append('    ' + ', '.join(str(value) for value in indices[offset:offset + 32]) + ',')
    output.extend(['];', '', '#[rustfmt::skip]', f'const IDENTIFIER_PAGES: [[u64; 8]; {len(pages)}] = ['])
    for page in pages:
        words = [int.from_bytes(page[offset:offset + 8], 'little') for offset in range(0, PAGE_BYTES, 8)]
        output.append('    [' + ', '.join(f'0x{word:016x}' for word in words) + '],')
    output.append('];')
    output.append(RUST_TESTS.rstrip())
    result = ('\n'.join(output) + '\n').encode()
    if len(result) > MAX_OUTPUT:
        raise ValueError('generated module exceeds byte limit')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--data-dir', type=Path, default=DATA)
    parser.add_argument('--output', type=Path, default=OUTPUT)
    parser.add_argument('--check', action='store_true', help='verify generated bytes without writing')
    args = parser.parse_args()
    try:
        if args.output.resolve() in { (args.data_dir / name).resolve() for name in (*PINNED, 'manifest.json') }:
            raise ValueError('output aliases a pinned input')
        generated = render(load_inputs(args.data_dir))
        if args.check:
            if read_bounded(args.output, MAX_OUTPUT) != generated:
                raise ValueError('generated module is stale; rerun tools/generate_js_identifiers.py')
        else:
            args.output.write_bytes(generated)
        print(f'Unicode {VERSION} identifier module {"verified" if args.check else "generated"}: '
              f'{len(generated)} bytes, SHA-256 {hashlib.sha256(generated).hexdigest()}')
        return 0
    except (OSError, ValueError, KeyError, TypeError, StopIteration) as error:
        print(f'Identifier generator: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
