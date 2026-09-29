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
REVISION_TREE = '91b2052adad1f066ae031e2ff3a1e9bd6d732886'
ARRAY_DESCRIPTOR_DIRECTORIES = {'Object/defineProperty': 1131, 'Object/defineProperties': 632,
                                'Array/length': 30}
ARRAY_PREDICATE_DIRECTORIES = {'Array/prototype/every': 218, 'Array/prototype/some': 219}
OBJECT_INTEGRITY_DIRECTORIES = {'Object/seal': 94, 'Object/freeze': 53,
                                'Object/isSealed': 33, 'Object/isFrozen': 59}
TREE_PROFILES = {'array-descriptors', 'array-predicates', 'object-integrity'}
DIRECTORIES = {
    'JSON/parse': 77, 'JSON/stringify': 66,
    'String/prototype/charAt': 30, 'String/prototype/charCodeAt': 25,
    'String/prototype/codePointAt': 16, 'String/prototype/slice': 38,
    'String/prototype/substring': 46, 'String/fromCharCode': 17,
    'String/fromCodePoint': 11,
}
STRING_CONCAT_DIRECTORIES = {'String/prototype/concat': 22}
ARRAY_LAST_INDEX_OF_DIRECTORIES = {'Array/prototype/lastIndexOf': 198}
STRING_LAST_INDEX_OF_DIRECTORIES = {'String/prototype/lastIndexOf': 25}
STRING_SEARCH_DIRECTORIES = {'String/prototype/includes': 27, 'String/prototype/indexOf': 47,
                             'String/prototype/startsWith': 21, 'String/prototype/endsWith': 27,
                             'String/prototype/split': 120}
REFLECT_CONSTRUCTION_DIRECTORIES = {'Reflect/apply': 9, 'Reflect/construct': 10}
NEW_TARGET_DIRECTORIES = {'expressions/new.target': 14}
FUNCTION_CONSTRUCTOR_DIRECTORIES = {'Function': 179, 'Function/length': 13,
                                    'Function/internals/Construct': 6, 'Function/internals/Call': 2}
REGEXP_CONSTRUCTOR_DIRECTORIES = {'RegExp': 488}
REGEXP_MATCH_SEARCH_DIRECTORIES = {'String/prototype/match': 51, 'String/prototype/search': 43,
                                   'RegExp/prototype/Symbol.match': 53, 'RegExp/prototype/Symbol.search': 23}
REGEXP_SPLIT_DIRECTORIES = {'RegExp/prototype/Symbol.split': 44, 'RegExp/Symbol.species': 4}
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
IS_PROTOTYPE_OF_DIRECTORIES = {'Object/prototype/isPrototypeOf': 10}
GLOBAL_VALUE_DIRECTORIES = {'global': 29, 'undefined': 8, 'NaN': 6, 'Infinity': 6}
ARRAY_SORT_DIRECTORIES = {'Array/prototype/sort': 54}
IDENTIFIER_DIRECTORIES = {'identifiers': 268, 'white-space': 67}
ARRAY_REDUCE_DIRECTORIES = {'Array/prototype/reduce': 260, 'Array/prototype/reduceRight': 260}
NUMBER_STATIC_DIRECTORIES = {
    'Number': 120, 'Number/MAX_VALUE': 4, 'Number/MIN_VALUE': 4,
    'Number/NEGATIVE_INFINITY': 4, 'Number/POSITIVE_INFINITY': 4,
    'Number/isFinite': 8, 'Number/isInteger': 9, 'Number/isNaN': 7, 'Number/isSafeInteger': 10,
}
NUMERIC_CONVERSION_DIRECTORIES = {'isFinite': 15, 'isNaN': 15}
NUMERIC_PARSING_DIRECTORIES = {'parseInt': 55, 'parseFloat': 54}
COMPOUND_ASSIGNMENT_DIRECTORIES = {'expressions/compound-assignment': 454}
ADDITION_DIRECTORIES = {'expressions/addition': 48}
LOGICAL_ASSIGNMENT_DIRECTORIES = {'expressions/logical-assignment': 78}
URI_DIRECTORIES = {'encodeURI':31, 'encodeURIComponent':31, 'decodeURI':55, 'decodeURIComponent':56}
RELATIONAL_DIRECTORIES = {'expressions/less-than':45, 'expressions/greater-than':49, 'expressions/less-than-or-equal':47, 'expressions/greater-than-or-equal':43}
EQUALITY_DIRECTORIES = {'expressions/equals':47, 'expressions/does-not-equals':38, 'expressions/strict-equals':30, 'expressions/strict-does-not-equals':30}
LABELS_DIRECTORIES = {'statements/labeled':24, 'statements/break':20, 'statements/continue':24}
SYMBOL_DIRECTORIES = {
    'Object/getOwnPropertySymbols': 12,
    'Reflect/ownKeys': 13,
    'Symbol': 12,
    'Symbol/asyncDispose': 3,
    'Symbol/asyncIterator': 2,
    'Symbol/dispose': 3,
    'Symbol/for': 9,
    'Symbol/hasInstance': 2,
    'Symbol/isConcatSpreadable': 2,
    'Symbol/iterator': 2,
    'Symbol/keyFor': 8,
    'Symbol/match': 2,
    'Symbol/matchAll': 2,
    'Symbol/prototype': 3,
    'Symbol/prototype/Symbol.toPrimitive': 9,
    'Symbol/prototype/description': 7,
    'Symbol/prototype/toString': 8,
    'Symbol/prototype/valueOf': 8,
    'Symbol/replace': 2,
    'Symbol/search': 2,
    'Symbol/species': 4,
    'Symbol/split': 2,
    'Symbol/toPrimitive': 2,
    'Symbol/toStringTag': 2,
    'Symbol/unscopables': 2,
}
PROFILES = {'object-integrity': OBJECT_INTEGRITY_DIRECTORIES, 'array-predicates': ARRAY_PREDICATE_DIRECTORIES, 'array-descriptors': ARRAY_DESCRIPTOR_DIRECTORIES, 'array-last-index-of': ARRAY_LAST_INDEX_OF_DIRECTORIES, 'string-last-index-of': STRING_LAST_INDEX_OF_DIRECTORIES, 'regexp-match-search': REGEXP_MATCH_SEARCH_DIRECTORIES, 'regexp-constructor': REGEXP_CONSTRUCTOR_DIRECTORIES, 'regexp-split': REGEXP_SPLIT_DIRECTORIES, 'string-search': STRING_SEARCH_DIRECTORIES, 'string-concat': STRING_CONCAT_DIRECTORIES, 'symbols': SYMBOL_DIRECTORIES, 'string-json': DIRECTORIES, 'regexp': REGEXP_DIRECTORIES,
            'function-constructor': FUNCTION_CONSTRUCTOR_DIRECTORIES,
            'reflect-construction': REFLECT_CONSTRUCTION_DIRECTORIES, 'new-target': NEW_TARGET_DIRECTORIES,
            'template-literal': TEMPLATE_DIRECTORIES, 'functions': FUNCTION_DIRECTORIES,
            'rest-parameters': REST_PARAMETER_DIRECTORIES,
            'is-prototype-of': IS_PROTOTYPE_OF_DIRECTORIES,
            'global-values': GLOBAL_VALUE_DIRECTORIES, 'array-sort': ARRAY_SORT_DIRECTORIES,
            'identifiers': IDENTIFIER_DIRECTORIES, 'array-reduce': ARRAY_REDUCE_DIRECTORIES,
            'number-statics': NUMBER_STATIC_DIRECTORIES,
            'numeric-conversion': NUMERIC_CONVERSION_DIRECTORIES,
            'numeric-parsing': NUMERIC_PARSING_DIRECTORIES,
            'compound-assignment': COMPOUND_ASSIGNMENT_DIRECTORIES,
            'addition': ADDITION_DIRECTORIES, 'logical-assignment': LOGICAL_ASSIGNMENT_DIRECTORIES, 'uri': URI_DIRECTORIES, 'relational': RELATIONAL_DIRECTORIES, 'equality': EQUALITY_DIRECTORIES, 'labels': LABELS_DIRECTORIES}
PROFILE_ROOTS = {'object-integrity': 'test/built-ins', 'array-predicates': 'test/built-ins', 'array-descriptors': 'test/built-ins', 'array-last-index-of': 'test/built-ins', 'string-last-index-of': 'test/built-ins', 'regexp-match-search': 'test/built-ins', 'regexp-constructor': 'test/built-ins', 'regexp-split': 'test/built-ins', 'string-search': 'test/built-ins', 'string-concat': 'test/built-ins', 'symbols': 'test/built-ins', 'string-json': 'test/built-ins', 'regexp': 'test/built-ins',
                 'function-constructor': 'test/built-ins', 'reflect-construction': 'test/built-ins', 'new-target': 'test/language',
                 'template-literal': 'test/language', 'functions': 'test/language',
                 'rest-parameters': 'test/language', 'is-prototype-of': 'test/built-ins',
                 'global-values': 'test/built-ins', 'array-sort': 'test/built-ins',
                 'identifiers': 'test/language', 'array-reduce': 'test/built-ins',
                 'number-statics': 'test/built-ins', 'numeric-conversion': 'test/built-ins',
                 'numeric-parsing': 'test/built-ins', 'compound-assignment': 'test/language', 'addition': 'test/language', 'logical-assignment': 'test/language', 'uri': 'test/built-ins', 'relational': 'test/language', 'equality': 'test/language', 'labels': 'test/language'}


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


def tree_proof_path(route):
    if route == f'commits/{REVISION}':
        return 'inventory-proof/commit.json'
    if re.fullmatch(r'trees/[0-9a-f]{40}', route):
        return 'inventory-proof/tree-' + route[6:] + '.json'
    raise ValueError('invalid pinned Git proof route')


def git_tree_inventory(directories, prefix, read=None):
    """Walk complete nonrecursive Git trees; the Contents API caps at 1,000.

    Verify each binary Git tree hash, including every entry, against its parent.
    The separately pinned root binds the chain to the selected revision. The
    same reader can use retained proof bytes without any network in the runner.
    """
    if read is None:
        read = lambda route: fetch(f'https://api.github.com/repos/{REPOSITORY}/git/{route}')
    proof, trees = {}, {}
    proof_bytes = 0

    def document(route):
        nonlocal proof_bytes
        if len(proof) >= 32:
            raise ValueError('Git inventory proof document limit')
        data = read(route)
        proof_bytes += len(data)
        if len(data) > MAX_FILE or proof_bytes > 8 * 1024 * 1024:
            raise ValueError('Git inventory proof byte limit')
        value = json.loads(data)
        if not isinstance(value, dict):
            raise ValueError('invalid Git inventory proof object')
        proof[tree_proof_path(route)] = data
        return value

    commit = document(f'commits/{REVISION}')
    if (commit.get('sha') != REVISION or not isinstance(commit.get('tree'), dict)
            or commit['tree'].get('sha') != REVISION_TREE):
        raise ValueError('Git commit differs from pinned revision/root tree')

    def tree(sha):
        if sha in trees:
            return trees[sha]
        value = document('trees/' + sha)
        entries = value.get('tree')
        if (value.get('sha') != sha or value.get('truncated') is not False
                or not isinstance(entries, list) or len(entries) > 16384):
            raise ValueError('incomplete, oversized or mismatched Git tree')
        names, encoded = {}, []
        modes = {'040000': 'tree', '100644': 'blob', '100755': 'blob',
                 '120000': 'blob', '160000': 'commit'}
        for entry in entries:
            if not isinstance(entry, dict):
                raise ValueError('invalid Git tree entry')
            name, mode, kind, child = (entry.get(k) for k in ('path', 'mode', 'type', 'sha'))
            if (not isinstance(name, str) or not name or name in {'.', '..'}
                    or '/' in name or '\0' in name or len(name.encode()) > 512
                    or name in names or mode not in modes or modes[mode] != kind
                    or not isinstance(child, str) or not re.fullmatch(r'[0-9a-f]{40}', child)):
                raise ValueError('unsafe, duplicate or invalid Git tree entry')
            names[name] = entry
            raw_name = name.encode()
            order = raw_name + (b'/' if kind == 'tree' else b'')
            encoded.append((order, mode.lstrip('0').encode() + b' ' + raw_name
                            + b'\0' + bytes.fromhex(child)))
        body = b''.join(raw for _, raw in sorted(encoded))
        actual = hashlib.sha1(b'tree ' + str(len(body)).encode() + b'\0' + body).hexdigest()
        if actual != sha:
            raise ValueError('Git tree hash mismatch; inventory may be incomplete')
        trees[sha] = names
        return names

    listings, directory_trees = {}, {}
    for directory in directories:
        sha = REVISION_TREE
        path = f'{prefix}/{directory}'
        parts = path.split('/')
        if len(parts) > 16 or any(not re.fullmatch(r'[A-Za-z0-9_.-]+', part) for part in parts):
            raise ValueError('invalid or overdeep Git inventory path')
        for part in parts:
            entry = tree(sha).get(part)
            if not entry or entry['type'] != 'tree':
                raise ValueError('missing pinned Git directory: ' + path)
            sha = entry['sha']
        directory_trees[directory] = sha
        listing = []
        for name, entry in tree(sha).items():
            if entry['type'] == 'blob' and name.endswith('.js'):
                if entry['mode'] not in {'100644', '100755'}:
                    raise ValueError('Git test entry is not a regular file')
                listing.append(dict(name=name, type='file', sha=entry['sha']))
        listings[directory] = listing
    description = dict(method='complete-nonrecursive-git-trees', revision=REVISION,
                       root_tree=REVISION_TREE, directory_trees=directory_trees)
    return listings, proof, description


def import_corpus(output, profile='string-json'):
    raw = f'https://raw.githubusercontent.com/{REPOSITORY}/{REVISION}/'
    inventory = {}
    entries = {}
    tree_listings, proof, proof_description = {}, {}, None
    if profile in TREE_PROFILES:
        tree_listings, proof, proof_description = git_tree_inventory(PROFILES[profile], PROFILE_ROOTS[profile])
    for directory, expected in PROFILES[profile].items():
        remote = f'{PROFILE_ROOTS[profile]}/{directory}'
        listing = (tree_listings[directory] if profile in TREE_PROFILES else
                   json.loads(fetch(f'https://api.github.com/repos/{REPOSITORY}/contents/{remote}?ref={REVISION}')))
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
    if profile in {'array-last-index-of', 'string-last-index-of', 'regexp-match-search', 'regexp-constructor', 'regexp-split', 'string-search', 'function-constructor', 'reflect-construction', 'new-target', 'string-concat', 'symbols', 'template-literal', 'functions', 'rest-parameters', 'is-prototype-of', 'identifiers', 'compound-assignment', 'addition', 'logical-assignment', 'uri', 'relational', 'equality', 'labels'}:
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
    if sum(map(len, sources.values())) + sum(map(len, proof.values())) > MAX_TOTAL:
        raise ValueError('Test262 selection exceeds aggregate import limit')
    scope = {
        'object-integrity': 'all direct .js files in Object/seal, Object/freeze, Object/isSealed and Object/isFrozen; complete pinned Git trees; no implementation',
        'array-predicates': 'all direct .js files in Array/prototype/every and Array/prototype/some; complete pinned Git trees; no implementation',
        'array-descriptors': 'all direct .js files in Object/defineProperty, Object/defineProperties and Array/length; complete pinned Git trees; no implementation',
        'array-last-index-of': 'all direct .js files in built-ins/Array/prototype/lastIndexOf; no implementation',
        'string-last-index-of': 'all direct .js files in built-ins/String/prototype/lastIndexOf; no implementation',
        'regexp-match-search': 'all direct .js files in built-ins/String/prototype/match, String/prototype/search, RegExp/prototype/Symbol.match and RegExp/prototype/Symbol.search; no implementation',
        'regexp-constructor': 'all direct .js files in built-ins/RegExp; no implementation',
        'regexp-split': 'all direct .js files in built-ins/RegExp/prototype/Symbol.split and RegExp/Symbol.species; no implementation',
        'string-search': 'all direct .js files in built-ins/String/prototype/includes, indexOf, startsWith, endsWith and split; no implementation',
        'function-constructor': 'all direct .js files in built-ins/Function, Function/length, Function/internals/Construct and Function/internals/Call; no implementation',
        'reflect-construction': 'all direct .js files in built-ins/Reflect/apply and Reflect/construct; no implementation',
        'new-target': 'all direct .js files in language/expressions/new.target; no implementation',
        'string-concat': 'all direct .js files in built-ins/String/prototype/concat; no implementation',
        'symbols': 'all .js files in the complete built-ins/Symbol tree and direct files in Object/getOwnPropertySymbols and Reflect/ownKeys; no implementation',
        'string-json': 'all direct .js files in nine built-ins directories; no implementation',
        'regexp': 'all direct .js files in four RegExp prototype directories; no implementation',
        'template-literal': 'all direct .js files in language/expressions/template-literal; no implementation',
        'functions': 'all direct .js files in four language function/arrow/object-method directories; no implementation',
        'rest-parameters': 'all direct .js files in language/rest-parameters; no implementation',
        'is-prototype-of': 'all direct .js files in built-ins/Object/prototype/isPrototypeOf; no implementation',
        'global-values': 'all direct .js files in built-ins/global, undefined, NaN and Infinity; no implementation',
        'array-sort': 'all direct .js files in built-ins/Array/prototype/sort; no implementation',
        'identifiers': 'all direct .js files in language/identifiers and language/white-space; no implementation',
        'array-reduce': 'all direct .js files in built-ins/Array/prototype/reduce and reduceRight; no implementation',
        'number-statics': 'all direct .js files in built-ins/Number and eight Number constant/predicate directories; no implementation',
        'numeric-conversion': 'all direct .js files in built-ins/isFinite and isNaN; no implementation',
        'numeric-parsing': 'all direct .js files in built-ins/parseInt and parseFloat; no implementation',
        'compound-assignment': 'all direct .js files in language/expressions/compound-assignment; no implementation',
        'addition': 'all direct .js files in language/expressions/addition; no implementation',
        'logical-assignment': 'all direct .js files in language/expressions/logical-assignment; no implementation',
        'uri': 'all direct .js files in built-ins/encodeURI, encodeURIComponent, decodeURI and decodeURIComponent; no implementation',
        'labels': 'all direct .js files in language/statements/labeled, break and continue; no implementation',
        'equality': 'all direct .js files in language/expressions/equals, does-not-equals, strict-equals and strict-does-not-equals; no implementation',
        'relational': 'all direct .js files in language/expressions/less-than, greater-than, less-than-or-equal and greater-than-or-equal; no implementation',
    }[profile]
    manifest = dict(format=1, repository=f'https://github.com/{REPOSITORY}', revision=REVISION,
                    scope=scope,
                    directories=inventory, test_files=len(paths), files=[])
    if proof_description is not None:
        manifest['inventory_proof'] = dict(proof_description, files=[])
        for path, data in sorted(proof.items()):
            route = (f'commits/{REVISION}' if path.endswith('/commit.json') else
                     'trees/' + Path(path).name.removeprefix('tree-').removesuffix('.json'))
            target = output / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            manifest['inventory_proof']['files'].append(dict(
                path=path, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
                url=f'https://api.github.com/repos/{REPOSITORY}/git/{route}'))
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
