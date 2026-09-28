#!/usr/bin/env python3
"""Import a complete pinned selection of Test262 data, without an external engine."""
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import re
import textwrap
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = 'tc39/test262'
REVISION = '7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd'
DIRECTORIES = {
    'JSON/parse': 77, 'JSON/stringify': 66,
    'String/prototype/charAt': 30, 'String/prototype/charCodeAt': 25,
    'String/prototype/codePointAt': 16, 'String/prototype/slice': 38,
    'String/prototype/substring': 46, 'String/fromCharCode': 17,
    'String/fromCodePoint': 11,
}
REGEXP_DIRECTORIES = {
    'RegExp/prototype/exec': 79, 'RegExp/prototype/test': 45,
    'RegExp/prototype/toString': 9, 'RegExp/prototype/source': 12,
}
TEMPLATE_DIRECTORIES = {'expressions/template-literal': 57}
FUNCTION_DIRECTORIES = {
    'expressions/function': 69, 'statements/function': 256,
    'expressions/arrow-function': 55, 'expressions/object/method-definition': 283,
}
REST_PARAMETER_DIRECTORIES = {'rest-parameters': 11}
PROFILES = {'string-json': DIRECTORIES, 'regexp': REGEXP_DIRECTORIES,
            'template-literal': TEMPLATE_DIRECTORIES, 'functions': FUNCTION_DIRECTORIES,
            'rest-parameters': REST_PARAMETER_DIRECTORIES}
PROFILE_ROOTS = {'string-json': 'test/built-ins', 'regexp': 'test/built-ins',
                 'template-literal': 'test/language', 'functions': 'test/language',
                 'rest-parameters': 'test/language'}


def corpus_name(profile):
    return 'test262' if profile == 'string-json' else f'test262-{profile}'


MAX_FILE = 2 * 1024 * 1024
MAX_TOTAL = 16 * 1024 * 1024
METADATA_KEYS = {'description', 'esid', 'es5id', 'es6id', 'info', 'author',
                 'negative', 'includes', 'flags', 'features', 'locale', 'timeout'}


def metadata_list(value):
    """Parse only Test262's simple string-list forms; reject other YAML forms."""
    value = value.strip()
    if not value:
        return []
    if value.startswith('[') and value.endswith(']'):
        entries = value[1:-1].split(',')
        if entries and not entries[-1].strip():
            entries.pop()
    else:
        entries = []
        for line in value.split('\n'):
            line = line.strip()
            if not line:
                continue
            if not line.startswith('- '):
                raise ValueError(f'unsupported Test262 metadata list: {value!r}')
            entries.append(line[2:])
    result = []
    for entry in entries:
        entry = entry.strip()
        if len(entry) >= 2 and entry[0] == entry[-1] and entry[0] in '\"\'':
            entry = entry[1:-1]
        if not re.fullmatch(r'[A-Za-z0-9_.-]+', entry):
            raise ValueError(f'unsupported Test262 metadata list item: {entry!r}')
        result.append(entry)
    if len(result) != len(set(result)):
        raise ValueError('duplicate Test262 metadata list item')
    return result


def parse_metadata(source):
    """Read execution metadata without changing source bytes or accepting YAML tags."""
    if source.count('/*---') != 1 or source.count('---*/') != 1:
        raise ValueError('test must contain exactly one Test262 frontmatter block')
    start = source.index('/*---') + 5
    end = source.index('---*/')
    if end < start:
        raise ValueError('invalid Test262 frontmatter delimiters')
    fields = {}
    key = None
    # YAML allows a common indentation on the entire top-level mapping. Only
    # remove that shared prefix; nested execution fields keep their structure.
    for line in textwrap.dedent(source[start:end]).split('\n'):
        match = re.fullmatch(r'([A-Za-z][A-Za-z0-9_-]*):[ \t]*(.*)\r?', line)
        if match:
            key, value = match.groups()
            if key in fields or key not in METADATA_KEYS:
                raise ValueError(f'duplicate or unknown Test262 metadata key: {key}')
            fields[key] = value.rstrip('\r')
        elif not line.strip() or line.lstrip().startswith('#'):
            continue
        elif key is not None and line[:1].isspace():
            fields[key] += '\n' + line.rstrip('\r')
        else:
            raise ValueError(f'unsupported Test262 frontmatter: {line!r}')
    result = {name: metadata_list(fields.get(name, ''))
              for name in ('flags', 'includes', 'features', 'locale')}
    result['negative'] = None
    if 'negative' in fields:
        values = {}
        for line in fields['negative'].split('\n'):
            if not line.strip():
                continue
            match = re.fullmatch(r'\s*(phase|type):\s*([A-Za-z][A-Za-z0-9]*)\s*', line)
            if not match or match[1] in values:
                raise ValueError('invalid negative metadata')
            values[match[1]] = match[2]
        if set(values) != {'phase', 'type'} or values['phase'] not in {'parse', 'resolution', 'runtime'}:
            raise ValueError('negative metadata requires a valid phase and error type')
        result['negative'] = values
    if {'noStrict', 'onlyStrict'} <= set(result['flags']):
        raise ValueError('conflicting strictness flags')
    if 'raw' in result['flags'] and set(result['flags']) & {'onlyStrict', 'module'}:
        raise ValueError('raw conflicts with strict/module mode')
    if fields.get('timeout'):
        result['timeout'] = fields['timeout'].strip()
    return result


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'Eris-Test262-data-import'})
    with urllib.request.urlopen(request, timeout=30) as response:
        data = response.read(MAX_FILE + 1)
    if len(data) > MAX_FILE:
        raise ValueError(f'upstream file exceeds import limit: {url}')
    return data


def import_corpus(output, profile='string-json'):
    raw = f'https://raw.githubusercontent.com/{REPOSITORY}/{REVISION}/'
    inventory = {}
    entries = {}
    for directory, expected in PROFILES[profile].items():
        remote = f'{PROFILE_ROOTS[profile]}/{directory}'
        listing = json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/contents/{remote}?ref={REVISION}'))
        names = []
        for entry in listing:
            name = entry.get('name', '')
            if entry.get('type') == 'file' and name.endswith('.js'):
                if not re.fullmatch(r'[A-Za-z0-9_.-]+\.js', name):
                    raise ValueError(f'unsafe upstream test filename: {name}')
                names.append(name)
                entries[f'{remote}/{name}'] = entry['sha']
        if len(names) != expected or len(set(names)) != expected:
            raise ValueError(f'pinned directory inventory mismatch: {directory}')
        inventory[directory] = sorted(names)
    paths = sorted(entries)
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        sources = dict(zip(paths, pool.map(lambda path: fetch(raw + path), paths)))
    harness = {'assert.js', 'sta.js'}
    if profile in {'template-literal', 'functions', 'rest-parameters'}:
        # These unchanged helpers support the assertion-integrity preflight,
        # even when no selected test requests them directly.
        harness.update({'propertyHelper.js', 'compareArray.js'})
    for path, data in sources.items():
        blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        if blob != entries[path]:
            raise ValueError(f'raw source differs from pinned Git blob: {path}')
        if '_FIXTURE' not in Path(path).name:
            try:
                metadata = parse_metadata(data.decode('utf-8'))
            except ValueError as error:
                raise ValueError(f'{path}: {error}') from error
            harness.update(metadata['includes'])
            if 'async' in metadata['flags']:
                harness.add('doneprintHandle.js')
    additional = ['LICENSE', 'INTERPRETING.md'] + [f'harness/{name}' for name in sorted(harness)]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        sources.update(zip(additional, pool.map(lambda path: fetch(raw + path), additional)))
    if sum(map(len, sources.values())) > MAX_TOTAL:
        raise ValueError('Test262 selection exceeds aggregate import limit')
    scope = {
        'string-json': 'all direct .js files in nine built-ins directories; no implementation',
        'regexp': 'all direct .js files in four RegExp prototype directories; no implementation',
        'template-literal': 'all direct .js files in language/expressions/template-literal; no implementation',
        'functions': 'all direct .js files in four language function/arrow/object-method directories; no implementation',
        'rest-parameters': 'all direct .js files in language/rest-parameters; no implementation',
    }[profile]
    manifest = dict(format=1, repository=f'https://github.com/{REPOSITORY}', revision=REVISION,
                    scope=scope,
                    directories=inventory, test_files=len(paths), files=[])
    for path, data in sorted(sources.items()):
        target = output / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        manifest['files'].append(dict(path=path, upstream_path=path, bytes=len(data),
                                     sha256=hashlib.sha256(data).hexdigest()))
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(f'Imported {len(paths)} Test262 files and {len(harness)} harness files at {REVISION}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', choices=PROFILES, default='string-json')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    output = args.output or ROOT / 'tests/upstream' / corpus_name(args.profile)
    import_corpus(output, args.profile)


if __name__ == '__main__':
    main()
