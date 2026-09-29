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
ARRAY_FIND_DIRECTORIES = {'Array/prototype/find': 23, 'Array/prototype/findIndex': 23,
                          'Array/prototype/findLast': 24, 'Array/prototype/findLastIndex': 24}
ARRAY_FROM_DIRECTORIES = {'Array/from': 47}
DATE_TREE = '6ad4fab73be4a87e6bfb58793b5fa9c82335ce5e'
DATE_DIRECTORIES = {'Date': 78, 'Date/UTC': 17, 'Date/now': 6, 'Date/parse': 8, 'Date/prototype': 44, 'Date/prototype/Symbol.toPrimitive': 18, 'Date/prototype/constructor': 1, 'Date/prototype/getDate': 8, 'Date/prototype/getDay': 8, 'Date/prototype/getFullYear': 8, 'Date/prototype/getHours': 8, 'Date/prototype/getMilliseconds': 8, 'Date/prototype/getMinutes': 8, 'Date/prototype/getMonth': 8, 'Date/prototype/getSeconds': 8, 'Date/prototype/getTime': 8, 'Date/prototype/getTimezoneOffset': 8, 'Date/prototype/getUTCDate': 8, 'Date/prototype/getUTCDay': 8, 'Date/prototype/getUTCFullYear': 8, 'Date/prototype/getUTCHours': 8, 'Date/prototype/getUTCMilliseconds': 8, 'Date/prototype/getUTCMinutes': 8, 'Date/prototype/getUTCMonth': 8, 'Date/prototype/getUTCSeconds': 8, 'Date/prototype/setDate': 14, 'Date/prototype/setFullYear': 20, 'Date/prototype/setHours': 23, 'Date/prototype/setMilliseconds': 14, 'Date/prototype/setMinutes': 18, 'Date/prototype/setMonth': 17, 'Date/prototype/setSeconds': 17, 'Date/prototype/setTime': 11, 'Date/prototype/setUTCDate': 7, 'Date/prototype/setUTCFullYear': 6, 'Date/prototype/setUTCHours': 11, 'Date/prototype/setUTCMilliseconds': 8, 'Date/prototype/setUTCMinutes': 8, 'Date/prototype/setUTCMonth': 9, 'Date/prototype/setUTCSeconds': 9, 'Date/prototype/toDateString': 7, 'Date/prototype/toISOString': 17, 'Date/prototype/toJSON': 13, 'Date/prototype/toLocaleDateString': 4, 'Date/prototype/toLocaleString': 4, 'Date/prototype/toLocaleTimeString': 4, 'Date/prototype/toString': 8, 'Date/prototype/toTemporalInstant': 8, 'Date/prototype/toTimeString': 6, 'Date/prototype/toUTCString': 9, 'Date/prototype/valueOf': 6}
FOR_OF_DIRECTORIES = {'statements/for-of': 182, 'statements/for-of/dstr': 569}
CORE_ITERATOR_DIRECTORIES = {
    'Array/prototype/values': 12, 'Array/prototype/keys': 12, 'Array/prototype/entries': 12,
    'String/prototype/Symbol.iterator': 6, 'ArrayIteratorPrototype': 0,
    'ArrayIteratorPrototype/Symbol.toStringTag': 3, 'ArrayIteratorPrototype/next': 24,
    'StringIteratorPrototype': 2, 'StringIteratorPrototype/next': 5,
}
ITERATION_SUBTREES = {
    'array-from': {'Array/from': 'e3a97b42c65283fc23a15d65610c2461e6f02715'},
    'for-of': {'statements/for-of': '592792d58aaf7752ef1b74e0edba13157f698a38'},
    'core-iterators': {
        'Array/prototype/values': 'd29084de7fdb7555ea16f25ba77f7a763acf67b9',
        'Array/prototype/keys': '6aa30e99a0abd72649ac41d7b7ac35ac71fbe781',
        'Array/prototype/entries': 'd5bad750535fe75bb48ce3433e14daaf427e7ecf',
        'String/prototype/Symbol.iterator': '2581ea6296933ab9a73383941619e0ef397ee8fb',
        'ArrayIteratorPrototype': 'aa50fb6e097adf8d3c71a210bd71578758f58e9d',
        'StringIteratorPrototype': 'c539b1d32923aa79d63fe6eb99c0660e31c0fa17',
    },
}
ITERATION_HELPERS = {
    'array-from': {'assert.js', 'sta.js', 'compareArray.js', 'propertyHelper.js', 'isConstructor.js'},
    'for-of': {'assert.js', 'sta.js', 'asyncHelpers.js', 'compareArray.js',
               'doneprintHandle.js', 'propertyHelper.js', 'resizableArrayBufferUtils.js'},
    'core-iterators': {'assert.js', 'sta.js', 'compareArray.js', 'isConstructor.js',
                       'propertyHelper.js', 'resizableArrayBufferUtils.js',
                       'detachArrayBuffer.js', 'testTypedArray.js'},
}
TREE_PROFILES = {'array-from', 'for-of', 'core-iterators', 'date', 'array-descriptors', 'array-predicates', 'object-integrity', 'array-find'}
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
PROFILES = {'array-from': ARRAY_FROM_DIRECTORIES, 'for-of': FOR_OF_DIRECTORIES, 'core-iterators': CORE_ITERATOR_DIRECTORIES, 'date': DATE_DIRECTORIES, 'array-find': ARRAY_FIND_DIRECTORIES, 'object-integrity': OBJECT_INTEGRITY_DIRECTORIES, 'array-predicates': ARRAY_PREDICATE_DIRECTORIES, 'array-descriptors': ARRAY_DESCRIPTOR_DIRECTORIES, 'array-last-index-of': ARRAY_LAST_INDEX_OF_DIRECTORIES, 'string-last-index-of': STRING_LAST_INDEX_OF_DIRECTORIES, 'regexp-match-search': REGEXP_MATCH_SEARCH_DIRECTORIES, 'regexp-constructor': REGEXP_CONSTRUCTOR_DIRECTORIES, 'regexp-split': REGEXP_SPLIT_DIRECTORIES, 'string-search': STRING_SEARCH_DIRECTORIES, 'string-concat': STRING_CONCAT_DIRECTORIES, 'symbols': SYMBOL_DIRECTORIES, 'string-json': DIRECTORIES, 'regexp': REGEXP_DIRECTORIES,
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
PROFILE_ROOTS = {'array-from': 'test/built-ins', 'for-of': 'test/language', 'core-iterators': 'test/built-ins', 'date': 'test/built-ins', 'array-find': 'test/built-ins', 'object-integrity': 'test/built-ins', 'array-predicates': 'test/built-ins', 'array-descriptors': 'test/built-ins', 'array-last-index-of': 'test/built-ins', 'string-last-index-of': 'test/built-ins', 'regexp-match-search': 'test/built-ins', 'regexp-constructor': 'test/built-ins', 'regexp-split': 'test/built-ins', 'string-search': 'test/built-ins', 'string-concat': 'test/built-ins', 'symbols': 'test/built-ins', 'string-json': 'test/built-ins', 'regexp': 'test/built-ins',
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
    recursive_trees = {DATE_TREE} | {sha for roots in ITERATION_SUBTREES.values() for sha in roots.values()}
    if route in {f'trees/{sha}?recursive=1' for sha in recursive_trees}:
        return 'inventory-proof/recursive-' + route[6:].removesuffix('?recursive=1') + '.json'
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


def date_tree_inventory(read=None):
    """Authenticate the entire pinned Date subtree, including all 51 directories.

    Ancestors and the harness use original nonrecursive responses. The original
    recursive Date response is retained, and every descendant binary tree hash
    is reconstructed; no projected response is presented as an API response.
    This separate bounded path does not relax existing direct-tree limits.
    """
    if read is None:
        read = lambda route: fetch(f'https://api.github.com/repos/{REPOSITORY}/git/{route}')
    proof, used_bytes = {}, 0

    def document(route):
        nonlocal used_bytes
        if len(proof) >= 8:
            raise ValueError('Date proof document limit')
        data = read(route)
        used_bytes += len(data)
        if len(data) > MAX_FILE or used_bytes > 4 * 1024 * 1024:
            raise ValueError('Date proof byte limit')
        value = json.loads(data)
        if not isinstance(value, dict):
            raise ValueError('invalid Date proof object')
        proof[tree_proof_path(route)] = data
        return value

    def verify_tree(entries, expected):
        if not isinstance(entries, list) or len(entries) > 16384:
            raise ValueError('Date tree entry limit')
        names, encoded = {}, []
        modes = {'040000': 'tree', '100644': 'blob', '100755': 'blob',
                 '120000': 'blob', '160000': 'commit'}
        for entry in entries:
            if not isinstance(entry, dict):
                raise ValueError('invalid Date tree entry')
            name, mode, kind, child = (entry.get(k) for k in ('path', 'mode', 'type', 'sha'))
            if (not isinstance(name, str) or not name or name in {'.', '..'}
                    or any(c in name for c in '/\\\0') or len(name.encode()) > 512
                    or name in names or mode not in modes or modes[mode] != kind
                    or not isinstance(child, str) or not re.fullmatch(r'[0-9a-f]{40}', child)):
                raise ValueError('invalid Date tree name, mode or hash')
            names[name] = entry
            raw = name.encode()
            encoded.append((raw + (b'/' if kind == 'tree' else b''),
                            mode.lstrip('0').encode() + b' ' + raw + b'\0' + bytes.fromhex(child)))
        body = b''.join(part for _, part in sorted(encoded))
        if hashlib.sha1(b'tree ' + str(len(body)).encode() + b'\0' + body).hexdigest() != expected:
            raise ValueError('Date Git tree hash mismatch; incomplete inventory')
        return names

    def tree(sha):
        value = document('trees/' + sha)
        if value.get('sha') != sha or value.get('truncated') is not False:
            raise ValueError('incomplete or mismatched Date ancestor tree')
        return verify_tree(value.get('tree'), sha)

    commit = document(f'commits/{REVISION}')
    if (commit.get('sha') != REVISION or not isinstance(commit.get('tree'), dict)
            or commit['tree'].get('sha') != REVISION_TREE):
        raise ValueError('Date commit differs from pinned root')
    root = tree(REVISION_TREE)
    entries = root
    for part in ('test', 'built-ins'):
        selected = entries.get(part)
        if not selected or selected['type'] != 'tree':
            raise ValueError('missing Date ancestor')
        entries = tree(selected['sha'])
    selected = entries.get('Date')
    if not selected or selected['type'] != 'tree' or selected['sha'] != DATE_TREE:
        raise ValueError('Date subtree differs from pinned root')
    value = document(f'trees/{DATE_TREE}?recursive=1')
    rows = value.get('tree')
    if (value.get('sha') != DATE_TREE or value.get('truncated') is not False
            or not isinstance(rows, list) or len(rows) > 4096):
        raise ValueError('incomplete or oversized recursive Date proof')
    by_path, directories, children = {}, {'': DATE_TREE}, {'': []}
    for entry in rows:
        if not isinstance(entry, dict) or not isinstance(entry.get('path'), str):
            raise ValueError('invalid recursive Date entry')
        path = entry['path']
        parts = path.split('/')
        if (len(parts) > 8 or len(path.encode()) > 1024 or path in by_path
                or any(not re.fullmatch(r'[A-Za-z0-9_.-]+', p) or p in {'.', '..'} for p in parts)):
            raise ValueError('unsafe, duplicate or overdeep Date path')
        by_path[path] = entry
        if entry.get('type') == 'tree':
            directories[path] = entry.get('sha')
            children[path] = []
    if len(directories) > 64:
        raise ValueError('Date directory limit')
    for path, entry in by_path.items():
        parent, _, name = path.rpartition('/')
        if parent not in children:
            raise ValueError('missing recursive Date parent')
        children[parent].append(dict(entry, path=name))
    for path, sha in directories.items():
        verify_tree(children[path], sha)
    mapped = {'Date' + ('/' + name if name else ''): sha for name, sha in directories.items()}
    if mapped.keys() != DATE_DIRECTORIES.keys():
        raise ValueError('recursive Date directory set differs from complete selection')
    listings = {name: [] for name in mapped}
    total = 0
    for path, entry in by_path.items():
        if entry['type'] == 'tree':
            continue
        size = entry.get('size')
        if (entry['type'] != 'blob' or entry['mode'] not in {'100644', '100755'}
                or not path.endswith('.js') or type(size) is not int or not 0 <= size <= MAX_FILE):
            raise ValueError('Date subtree contains nonregular, non-JavaScript or oversized source')
        total += size
        if total > 2 * 1024 * 1024:
            raise ValueError('Date source aggregate limit')
        parent, _, name = path.rpartition('/')
        directory = 'Date' + ('/' + parent if parent else '')
        listings[directory].append(dict(name=name, type='file', sha=entry['sha']))
    if any(len(listings[name]) != count for name, count in DATE_DIRECTORIES.items()):
        raise ValueError('Date source directory counts differ')
    if sum(map(len, listings.values())) != 594:
        raise ValueError('Date source count differs')
    harness_entry = root.get('harness')
    if not harness_entry or harness_entry['type'] != 'tree':
        raise ValueError('missing root-linked Date harness')
    harness = tree(harness_entry['sha'])
    auxiliary = {}
    for name in ('assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js',
                 'isConstructor.js', 'assertRelativeDateMs.js', 'dateConstants.js'):
        entry = harness.get(name)
        if not entry or entry['type'] != 'blob' or entry['mode'] not in {'100644', '100755'}:
            raise ValueError('missing pinned Date helper')
        auxiliary['harness/' + name] = entry['sha']
    for name in ('LICENSE', 'INTERPRETING.md'):
        entry = root.get(name)
        if not entry or entry['type'] != 'blob' or entry['mode'] not in {'100644', '100755'}:
            raise ValueError('missing pinned Date legal file')
        auxiliary[name] = entry['sha']
    description = dict(method='complete-root-linked-recursive-git-subtree', revision=REVISION,
                       root_tree=REVISION_TREE, subtree='test/built-ins/Date', subtree_tree=DATE_TREE,
                       directory_trees=mapped, auxiliary_blobs=auxiliary)
    return listings, proof, description


def iteration_tree_inventory(profile, read=None):
    """Verify complete selected recursive subtrees and unchanged auxiliary blobs.

    Retain original API responses, reconstruct every descendant Git tree, and
    authenticate each selected root through the pinned commit's ancestor chain.
    The fixed profile roots and direct-directory counts reject added/omitted
    directories as well as source substitutions; no execution informs selection.
    """
    if profile not in ITERATION_SUBTREES:
        raise ValueError('unknown iteration inventory profile')
    if read is None:
        read = lambda route: fetch(f'https://api.github.com/repos/{REPOSITORY}/git/{route}')
    proof, trees, used_bytes = {}, {}, 0

    def document(route):
        nonlocal used_bytes
        if len(proof) >= 24:
            raise ValueError('iteration proof document limit')
        data = read(route)
        used_bytes += len(data)
        if len(data) > MAX_FILE or used_bytes > 4 * 1024 * 1024:
            raise ValueError('iteration proof byte limit')
        value = json.loads(data)
        if not isinstance(value, dict):
            raise ValueError('invalid iteration proof object')
        proof[tree_proof_path(route)] = data
        return value

    def verify_tree(entries, expected):
        if not isinstance(entries, list) or len(entries) > 16384:
            raise ValueError('iteration tree entry limit')
        names, encoded = {}, []
        modes = {'040000': 'tree', '100644': 'blob', '100755': 'blob',
                 '120000': 'blob', '160000': 'commit'}
        for entry in entries:
            if not isinstance(entry, dict):
                raise ValueError('invalid iteration tree entry')
            name, mode, kind, child = (entry.get(k) for k in ('path', 'mode', 'type', 'sha'))
            if (not isinstance(name, str) or not name or name in {'.', '..'}
                    or any(c in name for c in '/\\\0') or len(name.encode()) > 512
                    or name in names or not isinstance(mode, str) or mode not in modes
                    or modes[mode] != kind or not isinstance(child, str)
                    or not re.fullmatch(r'[0-9a-f]{40}', child)):
                raise ValueError('invalid iteration tree name, mode or hash')
            names[name] = entry
            raw = name.encode()
            encoded.append((raw + (b'/' if kind == 'tree' else b''),
                            mode.lstrip('0').encode() + b' ' + raw + b'\0' + bytes.fromhex(child)))
        body = b''.join(part for _, part in sorted(encoded))
        if hashlib.sha1(b'tree ' + str(len(body)).encode() + b'\0' + body).hexdigest() != expected:
            raise ValueError('iteration Git tree hash mismatch; incomplete inventory')
        return names

    def tree(sha):
        if sha not in trees:
            value = document('trees/' + sha)
            if value.get('sha') != sha or value.get('truncated') is not False:
                raise ValueError('incomplete or mismatched iteration ancestor tree')
            trees[sha] = verify_tree(value.get('tree'), sha)
        return trees[sha]

    commit = document(f'commits/{REVISION}')
    if (commit.get('sha') != REVISION or not isinstance(commit.get('tree'), dict)
            or commit['tree'].get('sha') != REVISION_TREE):
        raise ValueError('iteration commit differs from pinned root')
    root = tree(REVISION_TREE)
    listings, mapped, total = {}, {}, 0
    for selected_root, pinned_tree in ITERATION_SUBTREES[profile].items():
        sha = REVISION_TREE
        for part in (PROFILE_ROOTS[profile] + '/' + selected_root).split('/'):
            entry = tree(sha).get(part)
            if not entry or entry['type'] != 'tree':
                raise ValueError('missing iteration ancestor')
            sha = entry['sha']
        if sha != pinned_tree:
            raise ValueError('iteration subtree differs from pinned root')
        value = document(f'trees/{sha}?recursive=1')
        rows = value.get('tree')
        if (value.get('sha') != sha or value.get('truncated') is not False
                or not isinstance(rows, list) or len(rows) > 4096):
            raise ValueError('incomplete or oversized recursive iteration proof')
        by_path, directories, children = {}, {'': sha}, {'': []}
        for entry in rows:
            if not isinstance(entry, dict) or not isinstance(entry.get('path'), str):
                raise ValueError('invalid recursive iteration entry')
            path = entry['path']
            parts = path.split('/')
            if (len(parts) > 8 or len(path.encode()) > 1024 or path in by_path
                    or any(not re.fullmatch(r'[A-Za-z0-9_.-]+', p) or p in {'.', '..'} for p in parts)):
                raise ValueError('unsafe, duplicate or overdeep iteration path')
            by_path[path] = entry
            if entry.get('type') == 'tree':
                directories[path] = entry.get('sha')
                children[path] = []
        if len(directories) > 64:
            raise ValueError('iteration directory limit')
        for path, entry in by_path.items():
            parent, _, name = path.rpartition('/')
            if parent not in children:
                raise ValueError('missing recursive iteration parent')
            children[parent].append(dict(entry, path=name))
        for path, child_sha in directories.items():
            verify_tree(children[path], child_sha)
            name = selected_root + ('/' + path if path else '')
            if name in mapped:
                raise ValueError('overlapping iteration subtree selection')
            mapped[name] = child_sha
            listings[name] = []
        for path, entry in by_path.items():
            if entry['type'] == 'tree':
                continue
            size = entry.get('size')
            if (entry['type'] != 'blob' or entry['mode'] not in {'100644', '100755'}
                    or not path.endswith('.js') or type(size) is not int or not 0 <= size <= MAX_FILE):
                raise ValueError('iteration source is nonregular, non-JavaScript or oversized')
            total += size
            if total > 2 * 1024 * 1024:
                raise ValueError('iteration source aggregate limit')
            parent, _, name = path.rpartition('/')
            directory = selected_root + ('/' + parent if parent else '')
            listings[directory].append(dict(name=name, type='file', sha=entry['sha']))
    if (mapped.keys() != PROFILES[profile].keys()
            or any(len(listings[name]) != count for name, count in PROFILES[profile].items())):
        raise ValueError('iteration directory/source inventory differs from complete selection')
    harness_entry = root.get('harness')
    if not harness_entry or harness_entry['type'] != 'tree':
        raise ValueError('missing root-linked iteration harness')
    harness = tree(harness_entry['sha'])
    auxiliary = {}
    for name in sorted(ITERATION_HELPERS[profile]):
        entry = harness.get(name)
        if not entry or entry['type'] != 'blob' or entry['mode'] not in {'100644', '100755'}:
            raise ValueError('missing pinned iteration helper')
        auxiliary['harness/' + name] = entry['sha']
    for name in ('LICENSE', 'INTERPRETING.md'):
        entry = root.get(name)
        if not entry or entry['type'] != 'blob' or entry['mode'] not in {'100644', '100755'}:
            raise ValueError('missing pinned iteration legal file')
        auxiliary[name] = entry['sha']
    description = dict(method='complete-root-linked-recursive-git-subtrees', revision=REVISION,
                       root_tree=REVISION_TREE, prefix=PROFILE_ROOTS[profile],
                       subtree_trees=ITERATION_SUBTREES[profile], directory_trees=mapped,
                       auxiliary_blobs=auxiliary)
    return listings, proof, description


def profile_tree_inventory(profile, read=None):
    if profile == 'date':
        return date_tree_inventory(read)
    if profile in ITERATION_SUBTREES:
        return iteration_tree_inventory(profile, read)
    return git_tree_inventory(PROFILES[profile], PROFILE_ROOTS[profile], read)


def import_corpus(output, profile='string-json'):
    raw = f'https://raw.githubusercontent.com/{REPOSITORY}/{REVISION}/'
    inventory = {}
    entries = {}
    tree_listings, proof, proof_description = {}, {}, None
    if profile in TREE_PROFILES:
        tree_listings, proof, proof_description = profile_tree_inventory(profile)
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
    if profile == 'date' or profile in ITERATION_SUBTREES:
        for path, expected_blob in proof_description['auxiliary_blobs'].items():
            data = sources.get(path, b'')
            blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            if blob != expected_blob:
                raise ValueError('helper or legal file differs from pinned Git blob: ' + path)
    if sum(map(len, sources.values())) + sum(map(len, proof.values())) > MAX_TOTAL:
        raise ValueError('Test262 selection exceeds aggregate import limit')
    scope = {
        'array-from': 'all .js files in the complete recursive built-ins/Array/from subtree; original root-linked Git proofs; no implementation',
        'for-of': 'all .js files in the complete recursive language/statements/for-of subtree, including dstr; original root-linked Git proofs; no implementation',
        'core-iterators': 'all .js files recursively in Array/prototype/{values,keys,entries}, String/prototype/Symbol.iterator, ArrayIteratorPrototype and StringIteratorPrototype; original root-linked Git proofs; no implementation',
        'date': 'all .js files in the complete recursive built-ins/Date subtree; all 51 directories and blobs authenticated to the pinned Git root; no implementation',
        'array-find': 'all direct .js files in Array/prototype/find, findIndex, findLast and findLastIndex; complete pinned Git trees; no implementation',
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
                     ('trees/' + Path(path).name.removeprefix('recursive-').removesuffix('.json') + '?recursive=1' if Path(path).name.startswith('recursive-') else
                      'trees/' + Path(path).name.removeprefix('tree-').removesuffix('.json')))
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
