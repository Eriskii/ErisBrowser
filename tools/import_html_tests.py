#!/usr/bin/env python3
"""Import pinned WPT HTML tree-construction data; never import a parser engine."""
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = 'web-platform-tests/wpt'
DEFAULT_REVISION = 'f085a1efc1f58fbe263d384b1e335d656fe58e66'
UPSTREAM_DIRECTORY = 'html/syntax/parsing/resources'
MAX_FILE = 4 * 1024 * 1024
MAX_TOTAL = 32 * 1024 * 1024


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'Eris-conformance-data-import'})
    with urllib.request.urlopen(request, timeout=30) as response:
        data = response.read(MAX_FILE + 1)
    if len(data) > MAX_FILE:
        raise ValueError(f'upstream file exceeds import limit: {url}')
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default=DEFAULT_REVISION)
    parser.add_argument('--output', type=Path, default=ROOT / 'tests/upstream/wpt-html')
    args = parser.parse_args()
    if not re.fullmatch('[0-9a-f]{40}', args.revision):
        parser.error('--revision must be an exact 40-digit commit SHA')
    raw_base = f'https://raw.githubusercontent.com/{REPOSITORY}/{args.revision}/'
    listing = json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/contents/{UPSTREAM_DIRECTORY}?ref={args.revision}'))
    files = [('LICENSE.md', 'LICENSE.md'), ('README.upstream.md', f'{UPSTREAM_DIRECTORY}/README.md')]
    for entry in listing:
        name = entry.get('name', '')
        if entry.get('type') == 'file' and name.endswith('.dat'):
            if not re.fullmatch(r'[A-Za-z0-9_-]+\.dat', name):
                raise ValueError(f'unsupported upstream data filename: {name}')
            files.append((f'resources/{name}', f'{UPSTREAM_DIRECTORY}/{name}'))
    if len(files) < 3 or len(files) > 200:
        raise ValueError('unexpected upstream file count')
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        payloads = list(pool.map(lambda file: fetch(raw_base + file[1]), files))
    if sum(map(len, payloads)) > MAX_TOTAL:
        raise ValueError('upstream corpus exceeds aggregate import limit')
    manifest = dict(repository=f'https://github.com/{REPOSITORY}', revision=args.revision,
                    upstream_directory=UPSTREAM_DIRECTORY,
                    imported_scope='all .dat files from this directory; no parser implementation', files=[])
    for (local, remote), data in zip(files, payloads):
        target = args.output / local
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        manifest['files'].append(dict(path=local, upstream_path=remote, bytes=len(data), sha256=hashlib.sha256(data).hexdigest()))
    (args.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Imported {len(files) - 2} tree-construction data files at {args.revision} to {args.output}')


if __name__ == '__main__':
    main()
