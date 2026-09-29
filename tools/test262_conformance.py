#!/usr/bin/env python3
"""Execute unchanged pinned Test262 scripts with Eris; retain every mode/result."""
import argparse
import concurrent.futures
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys
import time

from html_conformance import bounded_process, paths_alias
from import_test262 import (DIRECTORIES, PROFILES, PROFILE_ROOTS, MAX_FILE, MAX_TOTAL,
                            REPOSITORY, REVISION, TREE_PROFILES, corpus_name, parse_metadata,
                            git_tree_inventory, profile_tree_inventory, tree_proof_path)

ROOT = Path(__file__).resolve().parents[1]
CONSTRUCTOR_FEATURES = {'Reflect', 'Reflect.apply', 'Reflect.construct', 'new.target'}
SUPPORTED_FEATURES = {'arrow-function', 'String.fromCodePoint', 'well-formed-json-stringify', 'for-in-order'} | CONSTRUCTOR_FEATURES
STRING_CONCAT_FEATURES = SUPPORTED_FEATURES.copy()
REGEXP_FEATURES = SUPPORTED_FEATURES | {'regexp-dotall', 'regexp-match-indices', 'regexp-named-groups', 'regexp-sticky'}
TEMPLATE_FEATURES = SUPPORTED_FEATURES | {'template', 'u180e'}
FUNCTION_FEATURES = SUPPORTED_FEATURES | {'default-parameters', 'object-methods',
                                          'computed-property-names', 'trailing-function-commas'}
REST_PARAMETER_FEATURES = FUNCTION_FEATURES | {'rest-parameters'}
IS_PROTOTYPE_OF_FEATURES = SUPPORTED_FEATURES.copy()
GLOBAL_VALUE_FEATURES = SUPPORTED_FEATURES | {'globalThis'}
ARRAY_SORT_FEATURES = SUPPORTED_FEATURES | {'stable-array-sort'}
IDENTIFIER_FEATURES = SUPPORTED_FEATURES | {'u180e'}
ARRAY_REDUCE_FEATURES = SUPPORTED_FEATURES.copy()
NUMBER_STATIC_FEATURES = SUPPORTED_FEATURES.copy()
NUMERIC_CONVERSION_FEATURES = SUPPORTED_FEATURES.copy()
NUMERIC_PARSING_FEATURES = SUPPORTED_FEATURES.copy()
COMPOUND_ASSIGNMENT_FEATURES = SUPPORTED_FEATURES.copy()
ADDITION_FEATURES = SUPPORTED_FEATURES.copy()
LOGICAL_ASSIGNMENT_FEATURES = SUPPORTED_FEATURES | {'logical-assignment-operators'}
URI_FEATURES = SUPPORTED_FEATURES.copy()
RELATIONAL_FEATURES = SUPPORTED_FEATURES.copy()
EQUALITY_FEATURES = SUPPORTED_FEATURES.copy()
LABELS_FEATURES = SUPPORTED_FEATURES.copy()
SYMBOL_FEATURES = SUPPORTED_FEATURES | {
    'Symbol', 'Symbol.asyncIterator', 'Symbol.hasInstance', 'Symbol.isConcatSpreadable',
    'Symbol.iterator', 'Symbol.match', 'Symbol.matchAll', 'Symbol.replace', 'Symbol.search',
    'Symbol.species', 'Symbol.split', 'Symbol.toPrimitive', 'Symbol.toStringTag',
    'Symbol.unscopables', 'Symbol.prototype.description', 'Reflect', 'Reflect.ownKeys',
    'computed-property-names', 'object-methods',
}
CONSTRUCTION_FEATURES = SYMBOL_FEATURES | FUNCTION_FEATURES | {'new.target', 'Reflect.apply', 'Reflect.construct', 'template'}
FUNCTION_CONSTRUCTOR_FEATURES = CONSTRUCTION_FEATURES | REST_PARAMETER_FEATURES
STRING_SEARCH_FEATURES = SYMBOL_FEATURES | FUNCTION_FEATURES | {'String.prototype.includes', 'String.prototype.endsWith'}
REGEXP_MATCH_SEARCH_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
ARRAY_DESCRIPTOR_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
ARRAY_PREDICATE_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
OBJECT_INTEGRITY_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
DATE_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
FOR_OF_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'for-of', 'let', 'const'}
CORE_ITERATOR_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'Array.prototype.values'}
ARRAY_FROM_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'for-of', 'let', 'const', 'Array.prototype.values'}
ARRAY_SPLICE_FEATURES = ARRAY_FROM_FEATURES.copy()
ARRAY_CONCAT_FEATURES = ARRAY_SPLICE_FEATURES.copy()
ARRAY_FIND_FEATURES = ARRAY_PREDICATE_FEATURES | {'array-find-from-last'}
ARRAY_LAST_INDEX_OF_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
STRING_LAST_INDEX_OF_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES
REGEXP_SPLIT_FEATURES = CONSTRUCTION_FEATURES
REGEXP_CONSTRUCTOR_FEATURES = CONSTRUCTION_FEATURES | REGEXP_FEATURES | {'u180e'}
PROFILE_FEATURES = {'array-concat': ARRAY_CONCAT_FEATURES, 'array-splice': ARRAY_SPLICE_FEATURES, 'array-from': ARRAY_FROM_FEATURES, 'for-of': FOR_OF_FEATURES, 'core-iterators': CORE_ITERATOR_FEATURES, 'date': DATE_FEATURES, 'array-find': ARRAY_FIND_FEATURES, 'object-integrity': OBJECT_INTEGRITY_FEATURES, 'array-predicates': ARRAY_PREDICATE_FEATURES, 'array-descriptors': ARRAY_DESCRIPTOR_FEATURES, 'array-last-index-of': ARRAY_LAST_INDEX_OF_FEATURES, 'string-last-index-of': STRING_LAST_INDEX_OF_FEATURES, 'regexp-match-search': REGEXP_MATCH_SEARCH_FEATURES, 'regexp-constructor': REGEXP_CONSTRUCTOR_FEATURES, 'regexp-split': REGEXP_SPLIT_FEATURES, 'string-search': STRING_SEARCH_FEATURES, 'function-constructor': FUNCTION_CONSTRUCTOR_FEATURES, 'string-concat': STRING_CONCAT_FEATURES, 'symbols': SYMBOL_FEATURES, 'string-json': SUPPORTED_FEATURES, 'regexp': REGEXP_FEATURES,
                    'reflect-construction': CONSTRUCTION_FEATURES, 'new-target': CONSTRUCTION_FEATURES,
                    'template-literal': TEMPLATE_FEATURES, 'functions': FUNCTION_FEATURES,
                    'rest-parameters': REST_PARAMETER_FEATURES,
                    'is-prototype-of': IS_PROTOTYPE_OF_FEATURES,
                    'global-values': GLOBAL_VALUE_FEATURES, 'array-sort': ARRAY_SORT_FEATURES,
                    'identifiers': IDENTIFIER_FEATURES, 'array-reduce': ARRAY_REDUCE_FEATURES,
                    'number-statics': NUMBER_STATIC_FEATURES,
                    'numeric-conversion': NUMERIC_CONVERSION_FEATURES,
                    'numeric-parsing': NUMERIC_PARSING_FEATURES,
                    'compound-assignment': COMPOUND_ASSIGNMENT_FEATURES,
                    'addition': ADDITION_FEATURES, 'logical-assignment': LOGICAL_ASSIGNMENT_FEATURES, 'uri': URI_FEATURES, 'relational': RELATIONAL_FEATURES, 'equality': EQUALITY_FEATURES, 'labels': LABELS_FEATURES}
INTRINSIC_ERRORS = {'Error', 'TypeError', 'RangeError', 'SyntaxError', 'ReferenceError', 'EvalError', 'URIError'}
KNOWN_FLAGS = {'onlyStrict', 'noStrict', 'module', 'raw', 'async', 'generated',
               'CanBlockIsFalse', 'CanBlockIsTrue', 'non-deterministic'}
BAD_RUN_STATUSES = {'adapter-error', 'timeout', 'resource'}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def modes(metadata):
    flags = set(metadata['flags'])
    if 'module' in flags:
        return ['module']
    if 'raw' in flags:
        return ['raw']
    if 'onlyStrict' in flags:
        return ['strict']
    if 'noStrict' in flags:
        return ['sloppy']
    return ['sloppy', 'strict']


def harness_names(metadata):
    if 'raw' in metadata['flags']:
        return []
    names = ['assert.js', 'sta.js']
    if 'async' in metadata['flags']:
        names.append('doneprintHandle.js')
    return names + metadata['includes']


def load_corpus(directory, profile='string-json'):
    directories = PROFILES[profile]
    manifest_bytes = (directory / 'manifest.json').read_bytes()
    if len(manifest_bytes) > MAX_FILE:
        raise ValueError('manifest exceeds size limit')
    manifest = json.loads(manifest_bytes)
    if (manifest.get('format') != 1 or manifest.get('revision') != REVISION
            or manifest.get('repository') != f'https://github.com/{REPOSITORY}'):
        raise ValueError('unexpected Test262 corpus format, repository or pinned revision')
    inventory = manifest.get('directories', {})
    if inventory.keys() != directories.keys():
        raise ValueError('Test262 directory inventory differs from pinned selection')
    expected_tests = set()
    for name, count in directories.items():
        filenames = inventory[name]
        if (not isinstance(filenames, list) or len(filenames) != count
                or len(set(filenames)) != count
                or any(not re.fullmatch(r'[A-Za-z0-9_.-]+\.js', value) for value in filenames)):
            raise ValueError(f'pinned Test262 directory inventory mismatch: {name}')
        expected_tests.update(f'{PROFILE_ROOTS[profile]}/{name}/{value}' for value in filenames)
    if manifest.get('test_files') != len(expected_tests):
        raise ValueError('Test262 test file count differs from pinned inventory')
    files = {}
    total = 0
    for entry in manifest['files']:
        path = Path(entry['path'])
        if (path.is_absolute() or '..' in path.parts or str(path) != entry['path']
                or entry['path'] != entry.get('upstream_path') or str(path) in files):
            raise ValueError('unsafe, duplicate or remapped path in Test262 manifest')
        target = directory / path
        if target.is_symlink() or directory.resolve() not in target.resolve().parents:
            raise ValueError('Test262 corpus symlink escapes or aliases pinned bytes')
        if not isinstance(entry['bytes'], int) or not 0 <= entry['bytes'] <= MAX_FILE:
            raise ValueError('invalid Test262 file size')
        with target.open('rb') as handle:
            data = handle.read(MAX_FILE + 1)
        total += len(data)
        if total > MAX_TOTAL or len(data) != entry['bytes'] or digest(data) != entry['sha256']:
            raise ValueError(f'Test262 corpus integrity mismatch: {path}')
        files[str(path)] = data
    if profile in TREE_PROFILES:
        description = manifest.get('inventory_proof', {})
        if not isinstance(description, dict):
            raise ValueError('invalid Git inventory proof description')
        proof_entries = description.get('files', [])
        if (not isinstance(proof_entries, list) or len(proof_entries) > 32
                or any(not isinstance(entry, dict) or not isinstance(entry.get('path'), str)
                       for entry in proof_entries)):
            raise ValueError('invalid Git inventory proof file list')
        proof_by_path = {entry['path']: entry for entry in proof_entries}
        if len(proof_by_path) != len(proof_entries):
            raise ValueError('duplicate Git inventory proof file')

        def read_proof(route):
            nonlocal total
            path = tree_proof_path(route)
            entry = proof_by_path.get(path, {})
            target = directory / path
            if (target.is_symlink() or directory.resolve() not in target.resolve().parents
                    or entry.get('url') != f'https://api.github.com/repos/{REPOSITORY}/git/{route}'
                    or not isinstance(entry.get('bytes'), int) or not 0 <= entry['bytes'] <= MAX_FILE):
                raise ValueError('invalid, missing or aliased Git inventory proof')
            with target.open('rb') as handle:
                data = handle.read(MAX_FILE + 1)
            total += len(data)
            if (total > MAX_TOTAL or len(data) != entry['bytes']
                    or digest(data) != entry.get('sha256')):
                raise ValueError('Git inventory proof integrity mismatch')
            return data

        listings, proof, expected_proof = profile_tree_inventory(profile, read_proof)
        if ({key: value for key, value in description.items() if key != 'files'} != expected_proof
                or set(proof) != set(proof_by_path)):
            raise ValueError('Git inventory proof description differs from pinned trees')
        for name, listing in listings.items():
            if sorted(entry['name'] for entry in listing) != sorted(inventory[name]):
                raise ValueError('test inventory differs from complete pinned Git tree')
            for entry in listing:
                path = f'{PROFILE_ROOTS[profile]}/{name}/{entry["name"]}'
                data = files.get(path, b'')
                blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
                if blob != entry['sha']:
                    raise ValueError('test source differs from pinned Git tree blob')
    if profile in {'array-concat', 'array-splice', 'array-from', 'date', 'for-of', 'core-iterators'}:
        for path, expected_blob in expected_proof['auxiliary_blobs'].items():
            data = files.get(path, b'')
            blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            if blob != expected_blob:
                raise ValueError('helper or legal bytes differ from pinned Git blob')
        if profile in {'array-concat', 'array-splice', 'array-from', 'for-of', 'core-iterators'} and files.keys() != (
                expected_tests | expected_proof['auxiliary_blobs'].keys()):
            raise ValueError('iteration auxiliary inventory differs from pinned selection')
    actual_tests = {path for path in files if path.startswith('test/')}
    if actual_tests != expected_tests:
        raise ValueError('Test262 manifest test inventory is incomplete or contains extras')
    disk_scripts = {str(path.relative_to(directory)) for path in directory.rglob('*.js')}
    if disk_scripts != {path for path in files if path.endswith('.js')}:
        raise ValueError('unlisted or missing Test262 JavaScript files on disk')
    if not {'LICENSE', 'INTERPRETING.md', 'harness/assert.js', 'harness/sta.js'} <= files.keys():
        raise ValueError('missing Test262 license, interpretation rules or default harness')
    cases = []
    fixtures = []
    for path in sorted(expected_tests):
        if '_FIXTURE' in Path(path).name:
            fixtures.append(path)
            continue
        source = files[path].decode('utf-8')
        metadata = parse_metadata(source)
        names = harness_names(metadata)
        harness = []
        for name in names:
            key = f'harness/{name}'
            if key not in files:
                raise ValueError(f'missing requested Test262 harness file: {key}')
            harness.append((name, files[key]))
        for mode in modes(metadata):
            case = dict(id=f'{path}:{mode}', file=path, mode=mode,
                        source=files[path], metadata=metadata, harness=harness)
            case['case_sha256'] = case_fingerprint(case)
            cases.append(case)
    if not cases or len({case['id'] for case in cases}) != len(cases):
        raise ValueError('empty or duplicate Test262 case inventory')
    return manifest, files, cases, fixtures, digest(manifest_bytes)


def case_fingerprint(case):
    identity = dict(source_sha256=digest(case['source']), mode=case['mode'],
                    metadata=case['metadata'],
                    harness=[dict(name=name, sha256=digest(source)) for name, source in case['harness']])
    return digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode())


def unsupported_reason(case, supported_features=SUPPORTED_FEATURES):
    metadata = case['metadata']
    flags = set(metadata['flags'])
    if case['mode'] == 'module' or (metadata['negative'] or {}).get('phase') == 'resolution':
        return 'module parsing/resolution is not implemented'
    if metadata['negative'] and metadata['negative']['type'] not in INTRINSIC_ERRORS:
        return 'negative error constructor identity is not implemented: ' + metadata['negative']['type']
    if 'async' in flags:
        return 'asynchronous completion and jobs are not implemented'
    if flags - KNOWN_FLAGS:
        return 'unknown execution flags: ' + ', '.join(sorted(flags - KNOWN_FLAGS))
    if flags & {'CanBlockIsFalse', 'CanBlockIsTrue'}:
        return 'Test262 agent blocking policy is not implemented'
    if metadata['locale']:
        return 'locale-dependent execution is not implemented'
    if metadata.get('timeout'):
        return 'metadata-specific timeout policy is not implemented'
    unavailable = set(metadata['features']) - supported_features
    if unavailable:
        return 'unimplemented declared features: ' + ', '.join(sorted(unavailable))
    sources = [case['source']] + [source for _, source in case['harness']]
    if any(re.search(rb'\$262\b|\$DONE\b|\bprint\s*\(', source) for source in sources):
        return 'Test262 host hooks are not implemented (conservative source check)'
    return None


def encode_request(case):
    output = bytearray(b'ERJS1')
    output.append({'sloppy': 0, 'strict': 1, 'raw': 2, 'module': 3}[case['mode']])
    output.append(int((case['metadata']['negative'] or {}).get('phase') == 'parse'))
    output.extend(struct.pack('<I', len(case['harness'])))
    def string(value):
        output.extend(struct.pack('<I', len(value)))
        output.extend(value)
    for name, source in case['harness']:
        string(name.encode())
        string(source)
    string(case['source'])
    if len(output) > 4 * 1024 * 1024:
        raise ValueError('test request exceeds adapter budget')
    return bytes(output)


def decode_response(output):
    if not output.startswith(b'ERJR2'):
        raise ValueError('invalid JS adapter response magic')
    offset = 5
    fields = {}
    for key in ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness'):
        if len(output) - offset < 4:
            raise ValueError('truncated JS adapter response length')
        length, = struct.unpack_from('<I', output, offset)
        offset += 4
        if length > 16384 or length > len(output) - offset:
            raise ValueError('invalid JS adapter response field length')
        fields[key] = output[offset:offset + length].decode('utf-8')
        offset += length
    if offset != len(output):
        raise ValueError('trailing JS adapter response data')
    status, phase = fields['status'], fields['phase']
    if status not in {'complete', 'exception', 'unsupported', 'resource', 'harness-error'}:
        raise ValueError('unknown JS adapter response status')
    if phase not in {'parse', 'runtime', 'harness', 'mode'}:
        raise ValueError('unknown JS adapter response phase')
    if fields['error_identity'] and fields['error_identity'] not in INTRINSIC_ERRORS:
        raise ValueError('unknown intrinsic error identity')
    if ((status == 'complete' and (phase not in {'parse', 'runtime'} or fields['error_type'] or fields['error_identity'] or fields['harness']))
            or (status == 'exception' and (phase not in {'parse', 'runtime'} or not fields['error_type'] or fields['harness']))
            or (status == 'harness-error' and (phase != 'harness' or not fields['harness']))):
        raise ValueError('inconsistent JS adapter response')
    return fields


def classify(case, observation):
    status = observation['status']
    expected = case['metadata']['negative']
    if status in {'unsupported', 'resource', 'harness-error'}:
        return status
    if expected is None:
        return 'passed' if status == 'complete' and observation['phase'] == 'runtime' else 'failed'
    return ('passed' if status == 'exception' and observation['phase'] == expected['phase']
            and observation['error_identity'] == expected['type'] else 'failed')


def run_case(case, binary, timeout, supported_features=SUPPORTED_FEATURES):
    result = {key: case[key] for key in ('id', 'file', 'mode', 'case_sha256')}
    result['source_sha256'] = digest(case['source'])
    result['expected_negative'] = case['metadata']['negative']
    reason = unsupported_reason(case, supported_features)
    if reason:
        return dict(result, status='unsupported', reason=reason)
    try:
        exit_code, output, stderr = bounded_process([str(binary)], encode_request(case), timeout,
                                                   stdout_limit=96 * 1024, stderr_limit=64 * 1024)
        if exit_code:
            return dict(result, status='adapter-error', reason=f'adapter exit {exit_code}',
                        stderr=stderr.decode('utf-8', 'replace')[:4096])
        observation = decode_response(output)
        return dict(result, status=classify(case, observation), observation=observation)
    except subprocess.TimeoutExpired:
        return dict(result, status='timeout', reason=f'adapter exceeded {timeout:g} seconds')
    except (OSError, ValueError) as error:
        return dict(result, status='adapter-error', reason=str(error))


def date_preflight_variants():
    """Independent guarded pairs; only the final expected value differs."""
    guard = ("assert.sameValue(typeof Date,'function');assert.sameValue(typeof Date.UTC,'function');"
             "assert.sameValue(Date.UTC(1970,0,1),0);assert.sameValue(new Date(0).getTime(),0);"
             "assert.sameValue(new Date(0).toISOString(),'1970-01-01T00:00:00.000Z');")
    pairs = [
        ('epoch', 'var actual=new Date(123).valueOf();', 'actual', '123', '124'),
        ('clip', 'assert.sameValue(new Date(8640000000000000).getTime(),8640000000000000);var actual=new Date(8640000000000001).getTime();', 'actual', 'NaN', '0'),
        ('negative-fraction', 'var actual=new Date(-1.9).getTime();', 'actual', '-1', '-2'),
        ('UTC-defaults', 'assert.sameValue(Date.UTC(),NaN);assert.sameValue(Date.UTC(1970),0);var actual=Date.UTC(99,0,1);', 'actual', '915148800000', '0'),
        ('leap-arithmetic', 'assert.sameValue(Date.UTC(2000,1,29),951782400000);var actual=Date.UTC(2001,1,29);', 'actual', '983404800000', '0'),
        ('constructor-coercion', "var count=0,actual=new Date({valueOf:function(){count++;return 7;}}).getTime();assert.sameValue(count,1);", 'actual', '7', '8'),
        ('date-copy-no-coercion', "var d=new Date(42),reads=0;d.valueOf=function(){reads++;throw 'valueOf';};d.toString=function(){reads++;throw 'string';};var actual=new Date(d).getTime();assert.sameValue(reads,0);", 'actual', '42', '0'),
        ('constructor-default-hint', "var hints='',o={};o[Symbol.toPrimitive]=function(h){hints+=h;return 9;};assert.sameValue(new Date(o).getTime(),9);", 'hints', "'default'", "'number'"),
        ('parse-string-hint', "var hints='',o={};o[Symbol.toPrimitive]=function(h){hints+=h;return '1970-01-01';};assert.sameValue(Date.parse(o),0);", 'hints', "'string'", "'number'"),
        ('parse-offset', "var actual=Date.parse('2000-01-01T05:30:00+05:30');", 'actual', '946684800000', '946704600000'),
        ('expanded-ISO', "var actual=new Date(253402300800000).toISOString();", 'actual', "'+010000-01-01T00:00:00.000Z'", "'10000-01-01T00:00:00.000Z'"),
        ('invalid-ISO', "assert.throws(RangeError,function(){new Date(NaN).toISOString();});var actual=new Date(NaN).toString();", 'actual', "'Invalid Date'", "''"),
        ('brand-before-conversion', "var reads=0,p={valueOf:function(){reads++;throw 'coercion';}};assert.throws(TypeError,function(){Date.prototype.setTime.call({},p);});", 'reads', '0', '1'),
        ('setter-abrupt', "var d=new Date(42),reason={},caught;try{d.setTime({valueOf:function(){d.setTime(7);throw reason;}});}catch(e){caught=e;}assert.sameValue(caught,reason);var actual=d.getTime();", 'actual', '7', '42'),
        ('setter-initial-time', "var d=new Date(1577934245006);var actual=d.setUTCSeconds({valueOf:function(){d.setTime(0);return 9;}});", 'actual', '1577934249006', '9000'),
        ('setter-invalid-year-recovery', 'var d=new Date(NaN),actual=d.setUTCFullYear(2000);', 'actual', '946684800000', 'NaN'),
        ('setter-undefined', 'var d=new Date(0),actual=d.setUTCFullYear(2000,undefined);', 'actual', 'NaN', '946684800000'),
        ('local-roundtrip', 'var d=new Date(2020,0,15,12,34,56,789);assert.sameValue(d.getFullYear(),2020);assert.sameValue(d.getMonth(),0);assert.sameValue(d.getDate(),15);assert.sameValue(d.getHours(),12);var actual=d.getMilliseconds();', 'actual', '789', '0'),
        ('local-relative', 'var d=new Date(1970,0,1);var actual=d.valueOf()-d.getTimezoneOffset()*60000;', 'actual', '0', '1'),
        ('JSON-hooks', "var trace='',o={valueOf:function(){trace+='V';return 0;},get toISOString(){trace+='G';return function(){trace+='I';assert.sameValue(this,o);assert.sameValue(arguments.length,0);return 9;};}};assert.sameValue(Date.prototype.toJSON.call(o),9);", 'trace', "'VGI'", "'GVI'"),
        ('JSON-nonfinite', "var reads=0,o={valueOf:function(){return Infinity;},get toISOString(){reads++;throw 'late';}};assert.sameValue(Date.prototype.toJSON.call(o),null);", 'reads', '0', '1'),
        ('symbol-ordinary', "var trace='',o={toString:function(){trace+='S';return 's';},valueOf:function(){trace+='V';return 7;}};var m=Date.prototype[Symbol.toPrimitive];assert.sameValue(m.call(o,'default'),'s');assert.sameValue(m.call(o,'number'),7);", 'trace', "'SV'", "'VS'"),
        ('annex-alias', "assert.sameValue(Date.prototype.toGMTString,Date.prototype.toUTCString);var actual=Date.prototype.toGMTString.name;", 'actual', "'toUTCString'", "'toGMTString'"),
        ('prototype-slot-absent', "assert.throws(TypeError,function(){Date.prototype.getTime.call(Date.prototype);});var actual=Object.prototype.toString.call(Date.prototype);", 'actual', "'[object Object]'", "'[object Date]'"),
        ('construction-order', "var trace='',target=(function(){}).bind(null),p={};Object.defineProperty(target,'prototype',{get:function(){trace+='P';return p;}});var d=Reflect.construct(Date,[{valueOf:function(){trace+='V';return 7;}}],target);assert.sameValue(Date.prototype.getTime.call(d),7);assert.sameValue(Object.getPrototypeOf(d),p);", 'trace', "'VP'", "'PV'"),
        ('UTC-order', "var trace='';function n(c,v){return {valueOf:function(){trace+=c;return v;}};}assert.sameValue(Date.UTC(n('Y',1970),n('M',0),n('D',1)),0);", 'trace', "'YMD'", "'DMY'"),
    ]
    lengths = {
        'getDate': 0, 'getDay': 0, 'getFullYear': 0, 'getHours': 0,
        'getMilliseconds': 0, 'getMinutes': 0, 'getMonth': 0, 'getSeconds': 0,
        'getUTCDate': 0, 'getUTCDay': 0, 'getUTCFullYear': 0, 'getUTCHours': 0,
        'getUTCMilliseconds': 0, 'getUTCMinutes': 0, 'getUTCMonth': 0, 'getUTCSeconds': 0,
        'getTime': 0, 'getTimezoneOffset': 0, 'valueOf': 0,
        'toDateString': 0, 'toISOString': 0, 'toLocaleDateString': 0,
        'toLocaleString': 0, 'toLocaleTimeString': 0, 'toString': 0,
        'toTimeString': 0, 'toUTCString': 0, 'setDate': 1, 'setFullYear': 3,
        'setHours': 4, 'setMilliseconds': 1, 'setMinutes': 3, 'setMonth': 2,
        'setSeconds': 2, 'setTime': 1, 'setUTCDate': 1, 'setUTCFullYear': 3,
        'setUTCHours': 4, 'setUTCMilliseconds': 1, 'setUTCMinutes': 3,
        'setUTCMonth': 2, 'setUTCSeconds': 2, 'toJSON': 1,
        'getYear': 0, 'setYear': 1, 'toGMTString': 0,
    }
    for name, length in lengths.items():
        intrinsic_name = 'toUTCString' if name == 'toGMTString' else name
        result_type = 'string' if name.startswith('to') else 'number'
        argument = ',0' if name.startswith('set') else ''
        setup = (f"var m=Date.prototype.{name};assert.sameValue(typeof m,'function');"
                 f"assert.sameValue(typeof m.call(new Date(0){argument}),'{result_type}');"
                 f"verifyProperty(Date.prototype,'{name}',{{value:m,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'name',{{value:'{intrinsic_name}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'length',{{value:{length},writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new m();});")
        pairs.append(('property-' + name, setup, 'm.length', str(length), str(length + 1)))
    for name, length, argument in [('now', 0, ''), ('parse', 1, "'1970-01-01'"), ('UTC', 7, '1970')]:
        setup = (f"var m=Date.{name};assert.sameValue(typeof m({argument}),'number');"
                 f"verifyProperty(Date,'{name}',{{value:m,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'name',{{value:'{name}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'length',{{value:{length},writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new m();});")
        pairs.append(('property-static-' + name, setup, 'm.length', str(length), str(length + 1)))
    pairs.append(('property-symbol-toPrimitive',
                  "var m=Date.prototype[Symbol.toPrimitive];assert.sameValue(m.call(new Date(0),'number'),0);"
                  "verifyProperty(Date.prototype,Symbol.toPrimitive,{value:m,writable:false,enumerable:false,configurable:true},{restore:true});"
                  "verifyProperty(m,'name',{value:'[Symbol.toPrimitive]',writable:false,enumerable:false,configurable:true},{restore:true});"
                  "verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                  "assert.throws(TypeError,function(){new m();});", 'm.length', '1', '2'))
    pairs.append(('property-constructor',
                  "verifyProperty(globalThis,'Date',{value:Date,writable:true,enumerable:false,configurable:true},{restore:true});"
                  "verifyProperty(Date,'name',{value:'Date',writable:false,enumerable:false,configurable:true},{restore:true});"
                  "verifyProperty(Date,'length',{value:7,writable:false,enumerable:false,configurable:true},{restore:true});"
                  "verifyProperty(Date,'prototype',{value:Date.prototype,writable:false,enumerable:false,configurable:false},{restore:true});",
                  'Date.length', '7', '8'))
    return [('date-' + name + suffix, guard + setup + f'assert.sameValue({actual},{value});', expected, mode)
            for mode in ('sloppy', 'strict') for name, setup, actual, good, bad in pairs
            for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed'))]


def for_of_preflight_variants():
    """Byte-exact controls frozen independently before implementation or runs."""
    guard = "assert.sameValue(typeof Symbol,'function','Symbol prerequisite');\nassert.sameValue(typeof Symbol.iterator,'symbol','well-known iterator key prerequisite');\nvar $foGuardSource={},$foGuardCount=0,$foGuardSum=0,$foGuardSteps=0;\n$foGuardSource[Symbol.iterator]=function(){return {next:function(){var n=$foGuardSteps++;return n<2?{done:false,value:n+2}:{done:true};}};};\nfor(var $foGuardValue of $foGuardSource){$foGuardCount++;$foGuardSum+=$foGuardValue;}\nassert.sameValue($foGuardCount,2,'successful protocol prerequisite');assert.sameValue($foGuardSum,5);assert.sameValue($foGuardSteps,3);\nfunction sequence(values,trace){var index=0,source={},it={next:function(){if(trace)trace.push('next'+index);return index<values.length?{value:values[index++],done:false}:{done:true};}};source[Symbol.iterator]=function(){return it;};return {source:source,it:it};}\nfunction text(a){var s='';for(var i=0;i<a.length;i++){if(i)s+=',';s+=a[i];}return s;}\n"
    pairs = [
        ('protocol-values', 'var out=[];for(var x of sequence([2,3]).source)out.push(x);', 'text(out)', '"2,3"', '"3,2"'),
        ('next-cache', 'var n=0,reads=0,o={},it={};Object.defineProperty(it,"next",{get:function(){reads++;return function(){return n++?{done:true}:{value:4};};}});o[Symbol.iterator]=function(){return it;};for(var x of o){}', 'reads', '1', '2'),
        ('close-break', 'var count=0,s=sequence([1]);s.it.return=function(){count++;return {};};for(var x of s.source)break;', 'count', '1', '0'),
        ('no-close-step-error', 'var count=0,reason={},caught,s=sequence([]);s.it.next=function(){throw reason;};s.it.return=function(){count++;return {};};try{for(var x of s.source){}}catch(e){caught=e;}assert.sameValue(caught,reason);', 'count', '0', '1'),
        ('throw-precedence', 'var original={},closing={},caught,s=sequence([1]);s.it.return=function(){throw closing;};try{for(var x of s.source)throw original;}catch(e){caught=e;}', 'caught', 'original', 'closing'),
        ('lexical-closures', 'var f=[];for(let x of sequence([1,2]).source)f.push(function(){return x;});', 'f[0]()', '1', '2'),
        ('tdz', 'var caught=false;try{for(let x of x){}}catch(e){assert.sameValue(e.constructor,ReferenceError);caught=true;}', 'caught', 'true', 'false'),
        ('assignment-order', 'var n=0,target={},s=sequence([1]);function base(){n++;return target;}for(base().x of s.source){}', 'n', '1', '2'),
        ('unicode', 'var out=[];for(var x of "A\\uD83D\\uDE00B")out.push(x);', 'out.length', '3', '4'),
        ('array-live-growth', 'var a=[1],out=[];for(var x of a){out.push(x);if(x===1)a.push(2);}', 'text(out)', '"1,2"', '"1"'),
        ('same-loop-continue', 'var closes=0,s=sequence([1,2]);s.it.return=function(){closes++;return {};};for(var x of s.source)continue;', 'closes', '0', '2'),
        ('outward-label-close', 'var count=0,a=sequence([1]),b=sequence([2]);b.it.return=function(){count++;return {};};outer:for(var x of a.source){for(var y of b.source)continue outer;}', 'count', '1', '0'),
    ]
    return [('for-of-' + name + suffix, guard + setup + f'\nassert.sameValue({actual},{value});\n', expected, mode)
            for mode in ('sloppy', 'strict') for name, setup, actual, good, bad in pairs
            for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed'))]


def core_iterator_preflight_variants():
    """Native iterator controls avoid for-of syntax and prove successful calls first."""
    guard = ("assert.sameValue(typeof Symbol.iterator,'symbol');"
             "assert.sameValue(typeof Array.prototype.values,'function');"
             "assert.sameValue(typeof Array.prototype.keys,'function');"
             "assert.sameValue(typeof Array.prototype.entries,'function');"
             "assert.sameValue(typeof String.prototype[Symbol.iterator],'function');"
             "assert.sameValue(Array.prototype.values.call([7]).next().value,7);"
             "assert.sameValue(Array.prototype.keys.call([7]).next().value,0);"
             "assert.sameValue(Array.prototype.entries.call([7]).next().value[1],7);"
             "assert.sameValue(String.prototype[Symbol.iterator].call('a').next().value,'a');")
    pairs = [
        ('values-holes', "var it=Array.prototype.values.call(new Array(1)),a=it.next();assert.sameValue(a.value,undefined);assert.sameValue(a.done,false);", 'it.next().done', 'true', 'false'),
        ('keys-no-element-get', "var reads=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;throw 'element';}});var it=Array.prototype.keys.call(o);assert.sameValue(it.next().value,0);", 'reads', '0', '1'),
        ('entries-fresh', "var it=[4,5].entries(),a=it.next().value,b=it.next().value;assert.sameValue(a[0],0);assert.sameValue(a[1],4);assert.sameValue(b[0],1);assert.sameValue(b[1],5);assert.sameValue(Object.getPrototypeOf(a),Array.prototype);", 'a===b', 'false', 'true'),
        ('live-growth-and-shrink', "var a=[1],it=a.values();assert.sameValue(it.next().value,1);a.push(2);assert.sameValue(it.next().value,2);a.length=0;assert.sameValue(it.next().done,true);a.push(3);", 'it.next().done', 'true', 'false'),
        ('length-read-order', "var reads=0,o={0:9,get length(){reads++;return 1;}},it=Array.prototype.values.call(o);assert.sameValue(reads,0);assert.sameValue(it.next().value,9);assert.sameValue(it.next().done,true);assert.sameValue(it.next().done,true);", 'reads', '2', '3'),
        ('element-get-reentrancy', "var nested,a=[0,2,3],it=a.values();Object.defineProperty(a,'0',{get:function(){nested=it.next();return 1;}});assert.sameValue(it.next().value,1);assert.sameValue(nested.value,2);", 'it.next().value', '3', '2'),
        ('element-abrupt-advances', "var reason={},caught,a=[0,2],it=a.values();Object.defineProperty(a,'0',{get:function(){throw reason;}});try{it.next();}catch(e){caught=e;}assert.sameValue(caught,reason);", 'it.next().value', '2', 'undefined'),
        ('length-abrupt-retries', "var reason={},caught,n=0,o={0:8,get length(){if(n++===0)throw reason;return 1;}},it=Array.prototype.values.call(o);try{it.next();}catch(e){caught=e;}assert.sameValue(caught,reason);", 'it.next().value', '8', 'undefined'),
        ('completed-fresh-result', "var it=[].values(),a=it.next(),b=it.next();assert.sameValue(a.done,true);assert.sameValue(b.done,true);assert.sameValue(a.value,undefined);", 'a===b', 'false', 'true'),
        ('generic-full-length', "var o={length:Infinity,0:9},it=Array.prototype.values.call(o);", 'it.next().value', '9', 'undefined'),
        ('arguments-alias', "function f(){assert.sameValue(arguments[Symbol.iterator],Array.prototype.values);return arguments[Symbol.iterator]().next().value;}", 'f(8)', '8', 'undefined'),
        ('string-utf16', r"var it='\uD800\uD801\uDC00e\u0301'[Symbol.iterator]();assert.sameValue(it.next().value,'\uD800');assert.sameValue(it.next().value,'\uD801\uDC00');assert.sameValue(it.next().value,'e');assert.sameValue(it.next().value,'\u0301');", 'it.next().done', 'true', 'false'),
        ('string-converts-once', "var reads=0,o={toString:function(){reads++;return 'ab';}},it=String.prototype[Symbol.iterator].call(o);assert.sameValue(reads,1);o.toString=function(){throw 'late';};assert.sameValue(it.next().value,'a');assert.sameValue(it.next().value,'b');", 'reads', '1', '2'),
        ('string-completed-fresh-result', "var it=''[Symbol.iterator](),a=it.next(),b=it.next();assert.sameValue(a.value,undefined);assert.sameValue(a.done,true);assert.sameValue(b.done,true);", 'a===b', 'false', 'true'),
        ('private-brands', "var a=[1].values(),s='x'[Symbol.iterator]();assert.throws(TypeError,function(){a.next.call(Object.create(Object.getPrototypeOf(a)));});assert.throws(TypeError,function(){s.next.call(a);});assert.throws(TypeError,function(){a.next.call(s);});assert.sameValue(a.next().value,1);", 's.next().value', "'x'", 'undefined'),
        ('shared-prototype', "var a=[1].values(),s='a'[Symbol.iterator](),p=Object.getPrototypeOf(Object.getPrototypeOf(a));assert.sameValue(p,Object.getPrototypeOf(Object.getPrototypeOf(s)));var method=p[Symbol.iterator];assert.sameValue(method.call(null),null);assert.sameValue(method.call(3),3);", 'method.call(a)===a', 'true', 'false'),
        ('frozen-private-progress', "var a=[1,2].values(),s='ab'[Symbol.iterator]();Object.freeze(a);Object.freeze(s);assert.sameValue(a.next().value,1);assert.sameValue(a.next().value,2);assert.sameValue(s.next().value,'a');", 's.next().value', "'b'", 'undefined'),
        ('nullish-receivers', "assert.throws(TypeError,function(){Array.prototype.values.call(null);});assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(undefined);});assert.throws(TypeError,function(){String.prototype[Symbol.iterator].call(Symbol('x'));});", "Array.prototype.values.call('a').next().value", "'a'", 'undefined'),
        ('property-result', "var a=[7].values().next();verifyProperty(a,'value',{value:7,writable:true,enumerable:true,configurable:true},{restore:true});verifyProperty(a,'done',{value:false,writable:true,enumerable:true,configurable:true},{restore:true});", 'Object.getPrototypeOf(a)===Object.prototype', 'true', 'false'),
    ]
    for method in ('values', 'keys', 'entries', 'string'):
        owner, key = ('String.prototype', 'Symbol.iterator') if method == 'string' else ('Array.prototype', repr(method))
        name = '[Symbol.iterator]' if method == 'string' else method
        setup = (f"var m={owner}[{key}];"
                 f"verifyProperty({owner},{key},{{value:m,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'name',{{value:'{name}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});"
                 "assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new m();});")
        if method == 'values':
            setup += 'assert.sameValue(Array.prototype[Symbol.iterator],m);'
        pairs.append(('property-' + method, setup, 'm.length', '0', '1'))
    for kind, factory in [('Array', '[1].values()'), ('String', "'a'[Symbol.iterator]()")]:
        setup = (f"var it={factory},p=Object.getPrototypeOf(it),m=it.next;assert.sameValue(m.call(it).done,false);"
                 "verifyProperty(p,'next',{value:m,writable:true,enumerable:false,configurable:true},{restore:true});"
                 "verifyProperty(m,'name',{value:'next',writable:false,enumerable:false,configurable:true},{restore:true});"
                 "verifyProperty(m,'length',{value:0,writable:false,enumerable:false,configurable:true},{restore:true});"
                 f"verifyProperty(p,Symbol.toStringTag,{{value:'{kind} Iterator',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new m();});")
        pairs.append(('property-' + kind.lower() + '-next', setup, 'm.length', '0', '1'))
    return [('core-iterators-' + name + suffix, guard + setup + f'assert.sameValue({actual},{value});', expected, mode)
            for mode in ('sloppy', 'strict') for name, setup, actual, good, bad in pairs
            for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed'))]


def array_concat_preflight_variants():
    """Independent standard-derived pairs; two successful calls precede errors."""
    guard = (
        "assert.sameValue(typeof Array.prototype.concat,'function','concat availability');\n"
        "var m=Array.prototype.concat,$acArray=[1],$acResult=m.call($acArray,[2],3);"
        "assert.sameValue($acResult.length,3);assert.sameValue($acResult[0],1);"
        "assert.sameValue($acResult[1],2);assert.sameValue($acResult[2],3);"
        "assert.sameValue($acArray.length,1);\n"
        "var $acLike={0:4,length:1},$acGeneric=m.call($acLike,5);"
        "assert.sameValue($acGeneric.length,2);assert.sameValue($acGeneric[0],$acLike);"
        "assert.sameValue($acGeneric[1],5);\n"
    )
    pairs = [
        ('property-metadata',
         "verifyProperty(Array.prototype,'concat',{value:m,writable:true,enumerable:false,configurable:true},{restore:true});"
         "verifyProperty(m,'name',{value:'concat',writable:false,enumerable:false,configurable:true},{restore:true});"
         "verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
         "assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
         "assert.throws(TypeError,function(){new m();});", 'm.length', '1', '2'),
        ('alias-and-shallow',
         "var a=[1],nested=[2],token={};delete Array.prototype.concat;var r=m.call(a,[nested],token);"
         "assert.sameValue(r[0],1);assert.sameValue(r[1],nested);assert.sameValue(r[2],token);"
         "assert.sameValue(a.length,1);assert.sameValue(r===a,false);nested[0]=7;", 'r[1][0]', '7', '2'),
        ('spreadability-to-boolean',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var touched=0,o={0:7,length:1};"
         "o[Symbol.isConcatSpreadable]={valueOf:function(){touched++;throw 1;},toString:function(){touched++;throw 2;}};"
         "var a=[8];a[Symbol.isConcatSpreadable]=0;var r=m.call([],o,a);"
         "assert.sameValue(r[0],7);assert.sameValue(r[1],a);", 'touched', '0', '1'),
        ('nonspread-no-length-or-coercion',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var touched=0,o={};o[Symbol.isConcatSpreadable]=false;"
         "Object.defineProperty(o,'length',{get:function(){touched++;throw 1;}});"
         "o.valueOf=function(){touched++;throw 2;};o.toString=function(){touched++;throw 3;};"
         "var token=Symbol('item'),r=m.call([],o,token,null,undefined);"
         "assert.sameValue(r[0],o);assert.sameValue(r[1],token);assert.sameValue(r[2],null);assert.sameValue(r[3],undefined);", 'touched', '0', '1'),
        ('spread-length-order',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var trace='',o={};"
         "Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){trace+='S';return true;}});"
         "Object.defineProperty(o,'length',{get:function(){trace+='L';return {valueOf:function(){trace+='V';return 1;}};}});"
         "Object.defineProperty(o,'0',{get:function(){trace+='G';return 9;}});"
         "var r=m.call([],o);assert.sameValue(r[0],9);", 'trace', "'SLVG'", "'LSVG'"),
        ('holes-inherited-and-live',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var p={1:6},o=Object.create(p);o.length=3;"
         "o[1]=2;o[Symbol.isConcatSpreadable]=true;Object.defineProperty(o,'0',{get:function(){assert.sameValue(this,o);delete o[1];o.length=1;return 4;}});"
         "var r=m.call([],o),d=Object.getOwnPropertyDescriptor(r,'1');"
         "assert.sameValue(r.length,3);assert.sameValue(r[0],4);assert.sameValue(r[1],6);assert.sameValue(2 in r,false);",
         'd.value===6&&d.writable&&d.enumerable&&d.configurable', 'true', 'false'),
        ('live-later-argument',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var b=[2],a=[1];"
         "Object.defineProperty(a,'0',{get:function(){b[0]=8;b[Symbol.isConcatSpreadable]=false;return 3;}});"
         "var r=m.call(a,b);assert.sameValue(r[0],3);assert.sameValue(r[1],b);", 'r[1][0]', '8', '2'),
        ('species-order',
         "assert.sameValue(typeof Symbol.species,'symbol');assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');"
         "var trace='',a=[4],ctor={},out={};function C(n){trace+='N';assert.sameValue(arguments.length,1);assert.sameValue(n,0);assert.sameValue(new.target,C);return out;}"
         "Object.defineProperty(a,'constructor',{get:function(){trace+='C';return ctor;}});"
         "Object.defineProperty(ctor,Symbol.species,{get:function(){trace+='S';return C;}});"
         "Object.defineProperty(a,Symbol.isConcatSpreadable,{get:function(){trace+='B';return true;}});"
         "Object.defineProperty(a,'0',{get:function(){trace+='G';return 4;}});"
         "Object.defineProperty(out,'length',{set:function(n){trace+='L';assert.sameValue(n,1);}});"
         "var r=m.call(a);assert.sameValue(r,out);assert.sameValue(out[0],4);", 'trace', "'CSNBGL'", "'BCSNGL'"),
        ('species-default-and-invalid',
         "assert.sameValue(typeof Symbol.species,'symbol');var A=Array,a=[];a.constructor=undefined;"
         "Array=function(){throw new Test262Error('global replacement');};assert.sameValue(Object.getPrototypeOf(m.call(a)),A.prototype);"
         "a.constructor={};a.constructor[Symbol.species]=null;assert.sameValue(Object.getPrototypeOf(m.call(a)),A.prototype);"
         "a.constructor[Symbol.species]={};assert.throws(TypeError,function(){m.call(a);});"
         "a.constructor=null;assert.throws(TypeError,function(){m.call(a);});"
         "var o={};Object.defineProperty(o,'constructor',{get:function(){throw new Test262Error('generic constructor');}});",
         'm.call(o)[0]===o', 'true', 'false'),
        ('species-alias-later-source',
         "assert.sameValue(typeof Symbol.species,'symbol');var a=[1,2],b=[7,8];a.constructor={};"
         "a.constructor[Symbol.species]=function(n){assert.sameValue(n,0);return b;};"
         "var r=m.call(a,b);assert.sameValue(r,b);", "b.join(',')", "'1,2,1,2'", "'1,2,7,8'"),
        ('create-data-bypasses-setter',
         "assert.sameValue(typeof Symbol.species,'symbol');var called=0,p={};Object.defineProperty(p,'0',{set:function(){called++;}});"
         "var out=Object.create(p),a=[5];a.constructor={};a.constructor[Symbol.species]=function(){return out;};"
         "var r=m.call(a),d=Object.getOwnPropertyDescriptor(out,'0');assert.sameValue(r,out);assert.sameValue(out.length,1);",
         'called===0&&d.value===5&&d.writable&&d.enumerable&&d.configurable', 'true', 'false'),
        ('partial-definition-abrupt',
         "assert.sameValue(typeof Symbol.species,'symbol');var out={},a=[3,4,5],later=0;"
         "Object.defineProperty(out,'1',{value:9,writable:false,configurable:false});"
         "Object.defineProperty(a,'2',{get:function(){later++;return 5;}});"
         "a.constructor={};a.constructor[Symbol.species]=function(){return out;};"
         "assert.throws(TypeError,function(){m.call(a);});",
         'out[0]===3&&out[1]===9&&later===0&&!out.hasOwnProperty("length")', 'true', 'false'),
        ('late-length-failure',
         "assert.sameValue(typeof Symbol.species,'symbol');var out={},a=[3];Object.defineProperty(out,'length',{value:0,writable:false});"
         "a.constructor={};a.constructor[Symbol.species]=function(){return out;};"
         "assert.throws(TypeError,function(){m.call(a);});", 'out[0]===3&&out.length===0', 'true', 'false'),
        ('abrupt-identity',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var reason={},caught,later=0,o={};"
         "Object.defineProperty(o,Symbol.isConcatSpreadable,{get:function(){throw reason;}});"
         "var b={};Object.defineProperty(b,Symbol.isConcatSpreadable,{get:function(){later++;return false;}});"
         "try{m.call([],o,b);}catch(e){caught=e;}assert.sameValue(caught,reason);"
         "assert.throws(TypeError,function(){m.call(null);});assert.throws(TypeError,function(){m.call(undefined);});", 'later', '0', '1'),
        ('boxed-string-and-mapped-arguments',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var boxed=new String('ab'),r=m.call(boxed);assert.sameValue(r[0],boxed);"
         "boxed[Symbol.isConcatSpreadable]=true;r=m.call(boxed);assert.sameValue(r.join(','),'a,b');"
         "var ok=Function('m','return (function(a,b){var o=arguments;o[Symbol.isConcatSpreadable]=true;Object.defineProperty(o,0,{get:function(){b=7;return 1;}});var r=m.call([],o);return r[0]===1&&r[1]===7&&b===7;})(1,2);')(m);", 'ok', 'true', 'false'),
        ('safe-length-before-indexed-effects',
         "assert.sameValue(typeof Symbol.isConcatSpreadable,'symbol');var touched=0,o={length:Infinity};o[Symbol.isConcatSpreadable]=true;"
         "Object.defineProperty(o,'0',{get:function(){touched++;throw 1;}});assert.throws(TypeError,function(){m.call({},o);});"
         "var reason={},caught,n={length:9007199254740991};n[Symbol.isConcatSpreadable]=true;"
         "Object.defineProperty(n,'0',{get:function(){throw reason;}});try{m.call([],n);}catch(e){caught=e;}assert.sameValue(caught,reason);", 'touched', '0', '1'),
    ]
    return [('array-concat-' + name + suffix, guard + setup + '\n' + f'assert.sameValue({actual},{value});\n', expected, mode)
            for mode in ('sloppy', 'strict') for name, setup, actual, good, bad in pairs
            for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed'))]


def array_splice_preflight_variants():
    """Independent spec-derived pairs; real array and generic successes guard errors."""
    guard = (
        "assert.sameValue(typeof Array.prototype.splice,'function','splice availability');\n"
        "var m=Array.prototype.splice,$asArray=[1,2,3],$asDeleted=m.call($asArray,1,1,8);"
        "assert.sameValue($asDeleted.length,1);assert.sameValue($asDeleted[0],2);"
        "assert.sameValue($asArray[1],8);assert.sameValue($asArray.length,3);\n"
        "var $asLike={0:4,1:5,length:2},$asResult=m.call($asLike,0,1);"
        "assert.sameValue($asResult[0],4);assert.sameValue($asLike[0],5);"
        "assert.sameValue($asLike.length,1);\n"
    )
    pairs = [
        ('property-metadata',
         "verifyProperty(Array.prototype,'splice',{value:m,writable:true,enumerable:false,configurable:true},{restore:true});"
         "verifyProperty(m,'name',{value:'splice',writable:false,enumerable:false,configurable:true},{restore:true});"
         "verifyProperty(m,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});"
         "assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
         "assert.throws(TypeError,function(){new m();});", 'm.length', '2', '3'),
        ('arity-and-alias',
         "var a=[1,2,3];delete Array.prototype.splice;var r=m.call(a);"
         "assert.sameValue(r.length,0);assert.sameValue(a.length,3);"
         "r=m.call(a,1,undefined,7);assert.sameValue(r.length,0);"
         "r=m.call(a,2);assert.sameValue(r[0],2);assert.sameValue(r[1],3);"
         "var token={valueOf:function(){throw new Test262Error('inserted coercion');}};"
         "m.call(a,0,0,token);assert.sameValue(a[0],token);", "a.length+':'+a[1]+':'+a[2]", "'3:1:7'", "'3:7:1'"),
        ('coercion-order',
         "var trace='',o={};Object.defineProperty(o,'length',{configurable:true,get:function(){trace+='L';return {valueOf:function(){trace+='V';return 5;}};},set:function(v){trace+='W';assert.sameValue(v,4);}});"
         "Object.defineProperty(o,'constructor',{get:function(){throw new Test262Error('generic constructor');}});"
         "var start={valueOf:function(){trace+='S';return 1;}},count={valueOf:function(){trace+='D';return 1;}};"
         "var r=m.call(o,start,count);assert.sameValue(r.length,1);", 'trace', "'LVSDW'", "'LSDVW'"),
        ('clamping-and-errors',
         "var a=[1,2,3],r=m.call(a,-1,Infinity);assert.sameValue(r[0],3);"
         "r=m.call(a,Infinity,9);assert.sameValue(r.length,0);r=m.call(a,-Infinity,1);assert.sameValue(r[0],1);"
         "var calls=0,start={valueOf:function(){calls++;return 0;}};"
         "assert.throws(TypeError,function(){m.call(null,start);});"
         "assert.throws(TypeError,function(){m.call({length:Symbol()},start);});"
         "var huge={length:9007199254740991};assert.throws(TypeError,function(){m.call(huge,9007199254740991,0,9);});"
         "assert.sameValue(huge.length,9007199254740991);", 'calls===0&&a.length===1&&a[0]===2', 'true', 'false'),
        ('sparse-and-inherited',
         "var o=Object.create({1:4});o[0]=2;o.length=3;var r=m.call(o,0,2);"
         "assert.sameValue(r[0],2);assert.sameValue(r[1],4);assert.sameValue(r.hasOwnProperty('1'),true);"
         "assert.sameValue(o.hasOwnProperty('0'),false);assert.sameValue(o.length,1);"
         "var a=[,undefined,3],d=m.call(a,0,2);", "!(0 in d)&&d.hasOwnProperty('1')&&d[1]===undefined&&a[0]===3", 'true', 'false'),
        ('live-deleted-get',
         "var o={1:7,2:3,length:3},trace='';Object.defineProperty(o,'0',{configurable:true,get:function(){trace+='G';delete o[1];o[2]=9;return 5;}});"
         "var r=m.call(o,0,3);assert.sameValue(r[0],5);assert.sameValue(r[2],9);assert.sameValue(o.length,0);", "trace==='G'&&!(1 in r)&&r.length===3", 'true', 'false'),
        ('shrink-direction',
         "var o={0:0,length:4},trace='';"
         "Object.defineProperty(o,'1',{configurable:true,get:function(){trace+='g1;';return 11;},set:function(v){trace+='s1:'+v+';';}});"
         "Object.defineProperty(o,'2',{configurable:true,get:function(){trace+='g2;';return 22;},set:function(v){trace+='s2:'+v+';';}});"
         "Object.defineProperty(o,'3',{configurable:true,get:function(){trace+='g3;';return 33;}});"
         "var r=m.call(o,0,1);assert.sameValue(r[0],0);assert.sameValue(o[0],11);assert.sameValue(o.length,3);assert.sameValue(3 in o,false);",
         'trace', "'g1;g2;s1:22;g3;s2:33;'", "'g3;s2:33;g2;s1:22;g1;'"),
        ('grow-direction',
         "var o={length:3},trace='';"
         "Object.defineProperty(o,'0',{configurable:true,get:function(){trace+='g0;';return 10;},set:function(v){trace+='s0:'+v+';';}});"
         "Object.defineProperty(o,'1',{configurable:true,get:function(){trace+='g1;';return 20;},set:function(v){trace+='s1:'+v+';';}});"
         "Object.defineProperty(o,'2',{configurable:true,get:function(){trace+='g2;';return 30;},set:function(v){trace+='s2:'+v+';';}});"
         "var r=m.call(o,0,0,7);assert.sameValue(r.length,0);assert.sameValue(o[3],30);assert.sameValue(o.length,4);",
         'trace', "'g2;g1;s2:20;g0;s1:10;s0:7;'", "'g0;s1:10;g1;s2:20;g2;s0:7;'"),
        ('species-order',
         "assert.sameValue(typeof Symbol.species,'symbol');var a=[4,5],trace='',out={},ctor={};"
         "function C(n){trace+='N';assert.sameValue(arguments.length,1);assert.sameValue(n,1);assert.sameValue(new.target,C);return out;}"
         "Object.defineProperty(ctor,Symbol.species,{get:function(){trace+='S';return C;}});"
         "Object.defineProperty(a,'constructor',{get:function(){trace+='C';return ctor;}});"
         "Object.defineProperty(a,'0',{configurable:true,get:function(){trace+='G';return 4;},set:function(v){assert.sameValue(v,5);}});"
         "Object.defineProperty(out,'length',{set:function(n){trace+='L';assert.sameValue(n,1);}});"
         "var r=m.call(a,0,1);assert.sameValue(r,out);assert.sameValue(out[0],4);assert.sameValue(a.length,1);",
         'trace', "'CSNGL'", "'CNSGL'"),
        ('species-default-and-invalid',
         "assert.sameValue(typeof Symbol.species,'symbol');var A=Array,a=[1];a.constructor=undefined;"
         "Array=function(){throw new Test262Error('global Array replacement');};var r=m.call(a,0,0);assert.sameValue(Object.getPrototypeOf(r),A.prototype);"
         "var b=[];b.constructor={};b.constructor[Symbol.species]=null;assert.sameValue(Object.getPrototypeOf(m.call(b)),A.prototype);"
         "b.constructor[Symbol.species]={};assert.throws(TypeError,function(){m.call(b);});"
         "b.constructor=null;assert.throws(TypeError,function(){m.call(b);});"
         "var thrown={},caught;b.constructor={};Object.defineProperty(b.constructor,Symbol.species,{get:function(){throw thrown;}});"
         "try{m.call(b);}catch(e){caught=e;}", 'caught===thrown&&a.length===1', 'true', 'false'),
        ('species-alias-result',
         "assert.sameValue(typeof Symbol.species,'symbol');var a=[1,2,3];a.constructor={};a.constructor[Symbol.species]=function(n){assert.sameValue(n,1);return a;};"
         "var r=m.call(a,1,1,9);assert.sameValue(r,a);", "a[0]+':'+a[1]+':'+a.length+':'+(2 in a)", "'2:9:3:false'", "'1:9:3:true'"),
        ('result-descriptors-and-length-order',
         "assert.sameValue(typeof Symbol.species,'symbol');var a=[1,2],called=0,p={};Object.defineProperty(p,'0',{set:function(){called++;}});"
         "var out=Object.create(p);Object.defineProperty(out,'length',{set:function(n){assert.sameValue(n,1);a[1]=8;}});"
         "a.constructor={};a.constructor[Symbol.species]=function(){return out;};var r=m.call(a,0,1),d=Object.getOwnPropertyDescriptor(r,'0');"
         "assert.sameValue(r,out);assert.sameValue(a.length,1);", 'd.value===1&&d.writable&&d.enumerable&&d.configurable&&called===0&&a[0]===8', 'true', 'false'),
        ('partial-strict-writes',
         "var a=[1,2,3];Object.defineProperty(a,'1',{writable:false});"
         "assert.throws(TypeError,function(){m.call(a,0,0,9);});", "a.join(',')+':'+a.length", "'1,2,2,3:4'", "'1,2,3:3'"),
        ('partial-strict-deletion',
         "var a=[1,2,3,4];Object.defineProperty(a,'2',{configurable:false});"
         "assert.throws(TypeError,function(){m.call(a,0,3);});", "a[0]+':'+a[1]+':'+a[2]+':'+(3 in a)+':'+a.length", "'4:2:3:false:4'", "'1:2:3:true:4'"),
        ('mapped-arguments',
         "var mapped=Function('m','return (function(a,b){var r=m.call(arguments,0,1);return a===2&&b===2&&arguments.length===1&&arguments[0]===2&&arguments[1]===undefined&&r[0]===1;})(1,2);')(m);"
         "var unmapped=(function(a,b){'use strict';var r=m.call(arguments,0,1);return a===1&&b===2&&arguments.length===1&&arguments[0]===2&&arguments[1]===undefined&&r[0]===1;})(1,2);", 'mapped&&unmapped', 'true', 'false'),
        ('huge-and-late-array-length',
         "var o={length:9007199254740991},r=m.call(o,9007199254740991,0);assert.sameValue(r.length,0);assert.sameValue(o.length,9007199254740991);"
         "var touched=0,big={length:4294967296};Object.defineProperty(big,'0',{get:function(){touched++;return 1;}});"
         "assert.throws(RangeError,function(){m.call(big,0);});assert.sameValue(touched,0);"
         "var a=new Array(4294967295);assert.throws(RangeError,function(){m.call(a,4294967295,0,9);});",
         "a.length===4294967295&&a['4294967295']===9", 'true', 'false'),
    ]
    return [('array-splice-' + name + suffix, guard + setup + '\n' + f'assert.sameValue({actual},{value});\n', expected, mode)
            for mode in ('sloppy', 'strict') for name, setup, actual, good, bad in pairs
            for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed'))]


def array_from_preflight_variants():
    """Exact independently frozen source/mode pairs; availability guards precede errors."""
    guard = "assert.sameValue(typeof Array.from,'function','Array.from availability');\nvar $afLike=Array.from({0:3,length:1});assert.sameValue($afLike.length,1);assert.sameValue($afLike[0],3);\nvar $afSource={},$afNext=0;$afSource[Symbol.iterator]=function(){return {next:function(){return $afNext++===0?{value:5,done:false}:{done:true};}};};\nvar $afResult=Array.from($afSource);assert.sameValue($afResult.length,1);assert.sameValue($afResult[0],5);assert.sameValue($afNext,2);\nfunction sequence(values){var n=0,it={next:function(){return n<values.length?{value:values[n++],done:false}:{done:true};}},o={};o[Symbol.iterator]=function(){return it;};return {source:o,it:it};}\nfunction sameList(a,b){assert.sameValue(a.length,b.length);for(var j=0;j<b.length;j++)assert.sameValue(a[j],b[j]);}\nfunction traceText(a){var s='';for(var j=0;j<a.length;j++){if(j)s+=',';s+=a[j];}return s;}\n"
    pairs = [
        ('smoke', 'var result=Array.from({0:2,length:1});', 'result[0]', '2', '3'),
        ('custom-protocol', 'var result=Array.from(sequence([3,5]).source);', 'result.length+result[1]', '7', '8'),
        ('iterator-precedence', 'var s=sequence([9]);Object.defineProperty(s.source,"length",{get:function(){throw "length";}});var result=Array.from(s.source);', 'result[0]', '9', '8'),
        ('null-fallback', 'var o={0:7,length:1};o[Symbol.iterator]=null;var result=Array.from(o);', 'result[0]', '7', '8'),
        ('mapper-validation', 'var reads=0,o={};Object.defineProperty(o,Symbol.iterator,{get:function(){reads++;throw "read";}});assert.throws(TypeError,function(){Array.from(o,null);});', 'reads', '0', '1'),
        ('mapper-arguments', 'var ctx={},n=0;var result=Array.from(sequence([4]).source,function(v,k){"use strict";assert.sameValue(this,ctx);assert.sameValue(arguments.length,2);n++;return v+k;},ctx);', 'result[0]+n', '5', '6'),
        ('constructor-iterable', 'var count;function C(){count=arguments.length;assert.sameValue(new.target,C);}var result=Array.from.call(C,sequence([1]).source);', 'count', '0', '1'),
        ('constructor-arraylike', 'var count;function C(n){count=arguments.length;assert.sameValue(n,1);}var result=Array.from.call(C,{0:1,length:1});', 'count', '1', '0'),
        ('create-data', 'var calls=0,p={};Object.defineProperty(p,"0",{set:function(){calls++;}});function C(){}C.prototype=p;var result=Array.from.call(C,sequence([8]).source);assert.sameValue(result[0],8);', 'calls', '0', '1'),
        ('mapper-close', 'var token={},caught,n=0,s=sequence([1]);s.it.return=function(){n++;throw 9;};try{Array.from(s.source,function(){throw token;});}catch(e){caught=e;}assert.sameValue(caught,token);', 'n', '1', '0'),
        ('step-no-close', 'var token={},caught,n=0,s=sequence([]);s.it.next=function(){throw token;};s.it.return=function(){n++;return {};};try{Array.from(s.source);}catch(e){caught=e;}assert.sameValue(caught,token);', 'n', '0', '1'),
        ('final-length-no-close', 'var out={},n=0,s=sequence([]);Object.defineProperty(out,"length",{value:0,writable:false});function C(){return out;}s.it.return=function(){n++;return {};};assert.throws(TypeError,function(){Array.from.call(C,s.source);});', 'n', '0', '1'),
        ('live-indices', 'var o={0:1,1:2,length:2};var result=Array.from(o,function(v,k){if(k===0)o[1]=9;return v;});', 'result[1]', '9', '2'),
        ('unicode', 'var result=Array.from("A\\uD83D\\uDE00B");', 'result.length', '3', '4'),
        ('property-metadata', 'verifyProperty(Array.from,"name",{value:"from",writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(Array.from,"length",{value:1,writable:false,enumerable:false,configurable:true},{restore:true});assert.throws(TypeError,function(){new Array.from({length:0});});', 'Array.from.length', '1', '2'),
        ('native-range', 'var reads=0;assert.throws(RangeError,function(){Array.from({length:4294967296,get 0(){reads++;}});});', 'reads', '0', '1'),
    ]
    return [('array-from-' + name + suffix, guard + setup + f'\nassert.sameValue({actual},{value});\n', expected, mode)
            for name, setup, actual, good, bad in pairs for mode in ('sloppy', 'strict')
            for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed'))]


def harness_preflight(files, binary, timeout, profile='string-json'):
    """Fail closed if the upstream assertions no longer enforce basic failures."""
    scripts = [
        ('success', 'assert.sameValue(1, 1); assert.sameValue(NaN, NaN);', 'passed'),
        ('assert-false', 'assert(false);', 'failed'),
        ('same-value', 'assert.sameValue(1, 2);', 'failed'),
        ('signed-zero', 'assert.sameValue(0, -0);', 'failed'),
        ('not-same-value', 'assert.notSameValue(1, 1);', 'failed'),
        ('throws-success', 'assert.throws(TypeError, function () { throw new TypeError(); });', 'passed'),
        ('throws-wrong-type', 'assert.throws(TypeError, function () { throw new RangeError(); });', 'failed'),
        ('throws-missing', 'assert.throws(TypeError, function () {});', 'failed'),
        ('array-mismatch', 'assert.compareArray([1], [2]);', 'failed'),
        ('property-success', "verifyProperty({x: 1}, 'x', {value: 1, writable: true, enumerable: true, configurable: true});", 'passed'),
        ('property-writable', "verifyProperty({x: 1}, 'x', {writable: false});", 'failed'),
        ('property-enumerable', "verifyProperty({x: 1}, 'x', {enumerable: false});", 'failed'),
        ('property-configurable', "verifyProperty({x: 1}, 'x', {configurable: false});", 'failed'),
    ]
    variants = [(name, source, expected, 'sloppy') for name, source, expected in scripts]
    variants += [('strict-' + name, source, expected, 'strict') for name, source, expected in scripts]
    variants += [
        ('strict-receiver', "function f(){return this;} assert.sameValue(f(),undefined); assert.sameValue(f.call(null),null); assert.sameValue(f.call(3),3);", 'passed', 'strict'),
        ('strict-unbound-write', "assert.throws(ReferenceError,function(){missing=1;});", 'passed', 'strict'),
        ('strict-readonly-write', "var o={};Object.defineProperty(o,'x',{value:1});assert.throws(TypeError,function(){o.x=2;});", 'passed', 'strict'),
        ('strict-delete', "var o={};Object.defineProperty(o,'x',{value:1});assert.throws(TypeError,function(){delete o.x;});", 'passed', 'strict'),
        ('strict-arguments', "function f(a){arguments[0]=2;assert.sameValue(a,1);assert.throws(TypeError,()=>arguments.callee);}f(1);", 'passed', 'strict'),
        ('strict-tdz', "assert.throws(ReferenceError,function(){typeof x;let x;});", 'passed', 'strict'),
    ]
    if profile == 'regexp':
        checks = [
            ('regexp-success', "var r=/(?<x>a)(b)?/dg;var m=r.exec('xa');assert.sameValue(m.index,1);assert.sameValue(m.groups.x,'a');assert.sameValue(m[2],undefined);assert.sameValue(m.indices[0][1],2);assert.sameValue(r.lastIndex,2);", 'passed'),
            ('regexp-no-match', "assert(/x/.test('y'));", 'failed'),
            ('regexp-capture-mismatch', "assert.sameValue(/(a)/.exec('a')[1],'b');", 'failed'),
            ('regexp-last-index-mismatch', "var r=/a/g;r.test('a');assert.sameValue(r.lastIndex,0);", 'failed'),
            ('regexp-string-methods', "assert.sameValue('a1b2'.replace(/(\\d)/g,'[$1]'),'a[1]b[2]');assert.sameValue('ab'.match(/(?:)/g).length,3);assert.sameValue('a,b'.split(/(,)/)[1],',');", 'passed'),
            ('regexp-unsupported-unicode', "new RegExp('a','u');", 'unsupported'),
        ]
        variants += [(name, source, expected, mode)
                     for mode in ('sloppy', 'strict') for name, source, expected in checks]
    if profile == 'template-literal':
        checks = [
            ('template-success', "assert.sameValue(`a${1 + 2}b${`c${4}`}d`, 'a3bc4d');", 'passed'),
            ('template-cooked-mismatch', r"assert.sameValue(`\uD800${'x'}`, 'wrong');", 'failed'),
            ('template-order', "var log='';var value={toString:function(){log+='s';return 'v';}};function next(){log+='n';return 2;}assert.sameValue(`${value}${next()}`, 'v2');assert.sameValue(log,'sn');", 'passed'),
            ('template-order-mismatch', "var log='';var value={toString:function(){log+='s';return 'v';}};function next(){log+='n';return 2;}var text=`${value}${next()}`;assert.sameValue(log,'ns');", 'failed'),
            ('template-line-normalization', "assert.sameValue(`a\r\nb\rc`, 'a\\nb\\nc');", 'passed'),
            ('template-tag-unsupported', "function tag(x){return x;} tag`a${1}`;", 'unsupported'),
        ]
        variants += [(name, source, expected, mode)
                     for mode in ('sloppy', 'strict') for name, source, expected in checks]
    if profile in {'functions', 'rest-parameters'}:
        checks = [
            ('default-success', "function f(a=3,b=a+1){return a+b;}assert.sameValue(f(),7);assert.sameValue(f(0),1);", 'passed'),
            ('default-supplied-mismatch', "function f(a=3){return a;}assert.sameValue(f(0),3);", 'failed'),
            ('default-tdz', "function f(a=b,b=2){}assert.throws(ReferenceError,function(){f();});", 'passed'),
            ('default-tdz-wrong-type', "function f(a=b,b=2){}assert.throws(TypeError,function(){f();});", 'failed'),
            ('default-arguments', "function f(a=3){arguments[0]=9;assert.sameValue(a,3);var args=arguments;assert.throws(TypeError,function(){return args.callee;});}f(undefined);", 'passed'),
            ('default-arguments-mismatch', "function f(a=3){arguments[0]=9;assert.sameValue(a,9);}f(undefined);", 'failed'),
            ('default-scope', "function f(a=3,read=()=>a){var a=9;assert.sameValue(read(),3);assert.sameValue(a,9);}f();", 'passed'),
            ('default-scope-mismatch', "function f(a=3,read=()=>a){var a=9;assert.sameValue(read(),9);}f();", 'failed'),
        ]
        variants += [(name, source, expected, mode)
                     for mode in ('sloppy', 'strict') for name, source, expected in checks]
    if profile == 'rest-parameters':
        checks = [
            ('rest-array', "function f(...r){return r;}var a=f(1,2);var b=f(1,2);assert(Array.isArray(a));assert.compareArray(a,[1,2]);assert.sameValue(f().length,0);a[0]=9;assert.sameValue(b[0],1);", 'passed'),
            ('rest-array-mismatch', "function f(...r){return r;}assert.compareArray(f(1,2),[1,3]);", 'failed'),
            ('rest-index', "function f(a,b,...r){assert.compareArray(r,[3,undefined,5]);assert.sameValue(r.length,3);}f(1,2,3,undefined,5);", 'passed'),
            ('rest-index-mismatch', "function f(a,b,...r){assert.compareArray(r,[2,3]);}f(1,2,3);", 'failed'),
            ('rest-unmapped', "function f(a,...r){arguments[0]=9;assert.sameValue(a,1);r[0]=7;assert.sameValue(arguments[1],2);arguments[1]=8;assert.sameValue(r[0],7);}f(1,2);", 'passed'),
            ('rest-unmapped-mismatch', "function f(a,...r){arguments[0]=9;assert.sameValue(a,9);}f(1,2);", 'failed'),
            ('rest-length', "function f(a,b,...r){}function g(a=1,...r){}assert.sameValue(f.length,2);assert.sameValue(g.length,0);", 'passed'),
            ('rest-length-mismatch', "function f(a,b,...r){}assert.sameValue(f.length,3);", 'failed'),
        ]
        variants += [(name, source, expected, mode)
                     for mode in ('sloppy', 'strict') for name, source, expected in checks]
    if profile == 'is-prototype-of':
        checks = [
            ('prototype-chain', "var p={};var q=Object.create(p);var o=Object.create(q);var m=Object.prototype.isPrototypeOf;assert.sameValue(m.call(p,o),true);assert.sameValue(m.call(q,o),true);assert.sameValue(m.call({},o),false);", 'passed'),
            ('prototype-chain-mismatch', "var p={};var o=Object.create(p);assert.sameValue(p.isPrototypeOf(o),false);", 'failed'),
            ('prototype-self', "var p={};assert.sameValue(p.isPrototypeOf(p),false);assert.sameValue(Object.prototype.isPrototypeOf(Object.create(null)),false);", 'passed'),
            ('prototype-self-mismatch', "var p={};assert.sameValue(p.isPrototypeOf(p),true);", 'failed'),
            ('prototype-conversion-order', "var m=Object.prototype.isPrototypeOf;assert.sameValue(m.call(null,undefined),false);assert.sameValue(m.call(undefined,null),false);assert.sameValue(m.call(null,0),false);assert.sameValue(m.call(undefined,''),false);assert.sameValue(m.call(null,false),false);", 'passed'),
            ('prototype-conversion-order-mismatch', "assert.throws(TypeError,function(){Object.prototype.isPrototypeOf.call(null,1);});", 'failed'),
            ('prototype-object-receiver', "var m=Object.prototype.isPrototypeOf;assert.throws(TypeError,function(){m.call(null,{});});assert.throws(TypeError,function(){m.call(undefined,function(){});});", 'passed'),
            ('prototype-object-receiver-mismatch', "assert.throws(RangeError,function(){Object.prototype.isPrototypeOf.call(null,{});});", 'failed'),
            ('prototype-no-getters', "var calls=0;var p={toString:function(){calls++;throw 1;},valueOf:function(){calls++;throw 2;}};var o=Object.create(p);Object.defineProperty(o,'__proto__',{get:function(){calls++;throw 3;}});Object.defineProperty(o,'prototype',{get:function(){calls++;throw 4;}});assert.sameValue(p.isPrototypeOf(o),true);assert.sameValue(calls,0);", 'passed'),
            ('prototype-no-getters-mismatch', "var calls=0;var p={valueOf:function(){calls++;return 1;}};p.isPrototypeOf(Object.create(p));assert.sameValue(calls,1);", 'failed'),
            ('prototype-boxed-identity', "var m=Object.prototype.isPrototypeOf;var box=Object(1);var o=Object.create(box);assert.sameValue(m.call(box,o),true);assert.sameValue(m.call(1,o),false);assert.sameValue(m.call('x',{}),false);assert.sameValue(m.call(false,{}),false);", 'passed'),
            ('prototype-boxed-identity-mismatch', "var box=Object(1);assert.sameValue(Object.prototype.isPrototypeOf.call(1,Object.create(box)),true);", 'failed'),
            ('prototype-intrinsics', "var m=Object.prototype.isPrototypeOf;assert.sameValue(m.call(Array.prototype,[]),true);assert.sameValue(m.call(Function.prototype,function(){}),true);assert.sameValue(m.call(Function.prototype,m),true);assert.sameValue(m.call(Object.prototype,[]),true);", 'passed'),
            ('prototype-intrinsics-mismatch', "assert.sameValue(Object.prototype.isPrototypeOf.call(Array.prototype,[]),false);", 'failed'),
            ('prototype-property-metadata', "var m=Object.prototype.isPrototypeOf;verifyProperty(Object.prototype,'isPrototypeOf',{value:m,writable:true,enumerable:false,configurable:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true});verifyProperty(m,'name',{value:'isPrototypeOf',writable:false,enumerable:false,configurable:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.prototype.hasOwnProperty.call(m,'prototype'),false);assert.throws(TypeError,function(){new m({});});", 'passed'),
            ('prototype-property-metadata-mismatch', "verifyProperty(Object.prototype.isPrototypeOf,'length',{value:2});", 'failed'),
        ]
        variants += [(name, source, expected, mode)
                     for mode in ('sloppy', 'strict') for name, source, expected in checks]
    if profile == 'global-values':
        for mode in ('sloppy', 'strict'):
            write = ("assert.throws(TypeError,function(){realm[key]=replacement;});"
                     if mode == 'strict' else "assert.sameValue(realm[key]=replacement,replacement);")
            wrong_write = ("assert.throws(RangeError,function(){Infinity=3;});"
                           if mode == 'strict' else "Infinity=3;assert.sameValue(Infinity,3);")
            delete = ("assert.throws(TypeError,function(){delete realm[key];});"
                      if mode == 'strict' else "assert.sameValue(delete realm[key],false);")
            wrong_delete = ("var realm=this;assert.throws(RangeError,function(){delete realm.Infinity;});"
                            if mode == 'strict' else "assert.sameValue(delete this.Infinity,true);")
            checks = [
                ('global-values-identity', "assert.sameValue(globalThis,this);assert.sameValue(undefined,void 0);assert.sameValue(NaN,0/0);assert.sameValue(Infinity,1/0);assert.notSameValue(Infinity,-1/0);", 'passed'),
                ('global-values-identity-mismatch', "assert.sameValue(Infinity,-1/0);", 'failed'),
                ('global-values-immutable-descriptors', "var names=['undefined','NaN','Infinity'],values=[void 0,0/0,1/0];for(var i=0;i<names.length;i++){var d=Object.getOwnPropertyDescriptor(this,names[i]);assert.sameValue(d.value,values[i]);assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);assert.sameValue(d.get,undefined);assert.sameValue(d.set,undefined);}", 'passed'),
                ('global-values-immutable-descriptors-mismatch', "assert.sameValue(Object.getOwnPropertyDescriptor(this,'NaN').enumerable,true);", 'failed'),
                ('global-values-this-descriptor', "var d=Object.getOwnPropertyDescriptor(this,'globalThis');assert.sameValue(d.value,this);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,true);", 'passed'),
                ('global-values-this-descriptor-mismatch', "assert.sameValue(Object.getOwnPropertyDescriptor(this,'globalThis').writable,false);", 'failed'),
                ('global-values-immutable-write', "var realm=this,names=['undefined','NaN','Infinity'],values=[void 0,0/0,1/0],calls=0,replacement={toString:function(){calls++;throw 1;},valueOf:function(){calls++;throw 2;}};for(var i=0;i<names.length;i++){var key=names[i];" + write + "assert.sameValue(realm[key],values[i]);}assert.sameValue(calls,0);", 'passed'),
                ('global-values-immutable-write-mismatch', wrong_write, 'failed'),
                ('global-values-immutable-delete', "var realm=this,names=['undefined','NaN','Infinity'],values=[void 0,0/0,1/0];for(var i=0;i<names.length;i++){var key=names[i];" + delete + "assert.sameValue(realm[key],values[i]);}", 'passed'),
                ('global-values-immutable-delete-mismatch', wrong_delete, 'failed'),
                ('global-values-this-replace', "var realm=this,replacement={};assert.sameValue(globalThis=replacement,replacement);assert.sameValue(globalThis,replacement);assert.sameValue(realm.globalThis,replacement);assert.sameValue(this,realm);function readThis(){return this;}assert.sameValue(readThis.call(realm),realm);globalThis=realm;assert.sameValue(globalThis,realm);", 'passed'),
                ('global-values-this-replace-mismatch', "var realm=this;globalThis={};assert.sameValue(realm.globalThis,realm);", 'failed'),
                ('global-values-this-recreate', "var realm=this;assert.sameValue(delete realm.globalThis,true);assert.sameValue(Object.getOwnPropertyDescriptor(realm,'globalThis'),undefined);assert.sameValue(typeof globalThis,'undefined');assert.sameValue(this,realm);var replacement={};realm.globalThis=replacement;assert.sameValue(globalThis,replacement);var d=Object.getOwnPropertyDescriptor(realm,'globalThis');assert.sameValue(d.value,replacement);assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,true);assert.sameValue(d.configurable,true);assert.sameValue(this,realm);", 'passed'),
                ('global-values-this-recreate-mismatch', "assert.sameValue(delete this.globalThis,false);", 'failed'),
                ('global-values-this-lexical', "let globalThis='lexical';assert.sameValue(globalThis,'lexical');assert.sameValue(this.globalThis,this);this.globalThis=7;assert.sameValue(globalThis,'lexical');assert.sameValue(this.globalThis,7);", 'passed'),
                ('global-values-this-lexical-mismatch', "let globalThis='lexical';assert.sameValue(this.globalThis,globalThis);", 'failed'),
            ]
            variants += [(name, source, expected, mode) for name, source, expected in checks]
    if profile == 'array-sort':
        # A missing .sort/.call can throw the very TypeError an error-focused
        # check expects. Assert availability and a successful call outside each
        # assert.throws callback; a wrong partner alone never proves health.
        guard = ("assert.sameValue(typeof Array.prototype.sort,'function');"
                 "var smoke=[2,1];assert.sameValue(smoke.sort(),smoke);"
                 "assert.sameValue(smoke[0],1);assert.sameValue(smoke[1],2);")
        pairs = [
            ('basic', "var a=[10,2,1];var result=a.sort();assert.sameValue(result,a);assert.compareArray(a,[1,10,2]);", "a[1]", '10', '2'),
            ('stability', "var a={key:2},b={key:1},c={key:2},d={key:1};var items=[a,b,c,d];items.sort(function(x,y){return x.key-y.key;});assert.sameValue(items[0],b);assert.sameValue(items[1],d);assert.sameValue(items[3],c);", 'items[2]', 'a', 'c'),
            ('undefined-holes', "var a=[undefined,3,,1,,undefined],calls=0;a.sort(function(x,y){calls++;assert.notSameValue(x,undefined);assert.notSameValue(y,undefined);return x-y;});assert(calls>0);assert.sameValue(a.length,6);assert.compareArray(a.slice(0,4),[1,3,undefined,undefined]);assert.sameValue(3 in a,true);assert.sameValue(5 in a,false);", '4 in a', 'false', 'true'),
            ('validation-order', "var reads=0,o={};Object.defineProperty(o,'length',{get:function(){reads++;throw new RangeError();}});assert.throws(TypeError,function(){Array.prototype.sort.call(o,{});});assert.throws(TypeError,function(){Array.prototype.sort.call(null);});", 'reads', '0', '1'),
            ('callback-coercion', "var calls=0,conversions=0,a=[3,1,2];a.sort(function(x,y){'use strict';assert.sameValue(this,undefined);assert.sameValue(arguments.length,2);calls++;return {valueOf:function(){conversions++;return x-y;}};});assert(calls>0);assert.sameValue(conversions,calls);assert.compareArray(a,[1,2,3]);var equal=[2,1];equal.sort(function(){return NaN;});", 'equal[0]', '2', '1'),
            ('default-conversion', "var calls=0,x={toString:function(){calls++;return 'x';}};[x,x].sort();assert(calls>=2);var a=['\\uE000','\\uD800\\uDC00'];a.sort();", 'a[0]', "'\\uD800\\uDC00'", "'\\uE000'"),
            ('generic-length', "var reads=0,o={0:3,1:1,2:9};Object.defineProperty(o,'length',{get:function(){reads++;return 2.9;}});assert.sameValue(Array.prototype.sort.call(o),o);assert.sameValue(o[0],1);assert.sameValue(o[1],3);assert.sameValue(o[2],9);var negative={0:3,1:1,length:-1};Array.prototype.sort.call(negative);assert.sameValue(negative[0],3);assert.sameValue(negative.length,-1);", 'reads', '1', '2'),
            ('collection-order', "var log='',writes=[],o={length:2};Object.defineProperty(o,'0',{get:function(){log+='a';o[2]=0;o.length=3;return 3;},set:function(v){log+='x';writes.push(v);}});Object.defineProperty(o,'1',{get:function(){log+='b';return 1;},set:function(v){log+='y';writes.push(v);}});Array.prototype.sort.call(o,function(a,b){log+='c';return a-b;});assert.sameValue(log.slice(0,2),'ab');assert.sameValue(log.slice(-2),'xy');assert.compareArray(writes,[1,3]);assert.sameValue(o[2],0);", 'log.charAt(2)', "'c'", "'x'"),
            ('inherited-index', "var stored=0,p={};Object.defineProperty(p,'0',{get:function(){return 3;},set:function(v){stored=v;}});var o=Object.create(p);o[1]=1;o.length=2;assert.sameValue(Array.prototype.sort.call(o),o);assert.sameValue(stored,1);assert.sameValue(o[1],3);", "Object.prototype.hasOwnProperty.call(o,'0')", 'false', 'true'),
            ('abrupt-comparator', "var reason={},caught=false,calls=0,a=[3,2,1];try{a.sort(function(){calls++;throw reason;});}catch(e){caught=true;assert.sameValue(e,reason);}assert(caught);assert(calls>0);assert.compareArray(a,[3,2,1]);", 'a[0]', '3', '1'),
            ('throwing-write-delete', "var o={0:3,1:1,length:2};Object.defineProperty(o,'1',{writable:false});assert.throws(TypeError,function(){Array.prototype.sort.call(o);});assert.sameValue(o[0],1);var sparse={length:2};Object.defineProperty(sparse,'1',{value:7,writable:true,configurable:false});assert.throws(TypeError,function(){Array.prototype.sort.call(sparse);});assert.sameValue(sparse[0],7);", 'sparse[1]', '7', 'undefined'),
            ('property-metadata', "var m=Array.prototype.sort;verifyProperty(Array.prototype,'sort',{value:m,writable:true,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'name',{value:'sort',writable:false,enumerable:false,configurable:true},{restore:true});assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.prototype.hasOwnProperty.call(m,'prototype'),false);assert.throws(TypeError,function(){new m();});", 'm.length', '1', '2'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    source = guard + setup + f'assert.sameValue({actual},{value});'
                    variants.append(('array-sort-' + name + suffix, source, expected, mode))
    if profile == 'labels':
        guard = "guard:{break guard;throw new Test262Error('unreached');}"
        pairs = [
            ('block', "var trace='';outer:{trace+='A';inner:{trace+='B';break outer;}trace+='X';}trace+='C';", 'trace', "'ABC'", "'ABXC'"),
            ('while', "var n=0,trace='';outer:while(n<3){n++;inner:while(true){trace+=n;continue outer;}trace+='X';}", 'trace', "'123'", "'123X'"),
            ('for', "var n=0,trace='';outer:alias:for(var i=0;i<3;i++){for(var j=0;j<2;j++){trace+=i;continue alias;}trace+='X';}n=i;assert.sameValue(n,3);", 'trace', "'012'", "'012X'"),
            ('do', "var n=0,checks=0;outer:do{n++;if(n<3)continue outer;break outer;}while(++checks<5);assert.sameValue(n,3);", 'checks', '2', '3'),
            ('for-in', "var trace='';outer:for(var key in {a:1,b:2,c:3}){for(var j=0;j<2;j++){trace+=key;continue outer;}}", 'trace', "'abc'", "'aabbcc'"),
            ('switch', "var trace='';outer:for(var i=0;i<3;i++){switch(i){case 0:trace+='A';continue outer;case 1:trace+='B';break;default:break outer;}trace+='C';}", 'trace', "'ABC'", "'ABCC'"),
            ('finally', "var trace='';outer:for(var i=0;i<3;i++){try{trace+=i;continue outer;}finally{trace+='F';}}", 'trace', "'0F1F2F'", "'012'"),
            ('override', "var trace='';outer:for(var i=0;i<3;i++){try{trace+='A';continue outer;}finally{trace+='F';break outer;}}", 'trace', "'AF'", "'AFAFAF'"),
            ('abrupt', "var reason={},seen;function f(){outer:{try{break outer;}finally{return 7;}}return 9;}assert.sameValue(f(),7);try{outer:{try{break outer;}finally{throw reason;}}}catch(e){seen=e;}", 'seen', 'reason', 'undefined'),
            ('hoist', "assert.sameValue(x,undefined);outer:{break outer;var x=7;}assert.sameValue(x,undefined);var n=0;outer:var y=++n;", 'y', '1', '2'),
            ('unicode', "var n=0;\\u0061:{n++;break a;n++;}a:{n++;break \\u0061;n++;}π:{n++;break π;n++;}", 'n', '3', '6'),
            ('function-scope', "function f(){outer:{break outer;}return 2;}var n=0;outer:{n=f();break outer;}outer:{n++;break outer;}function g(){outer:{break outer;}return 4;}n+=g();", 'n', '7', '8'),
        ]
        for mode in ('sloppy','strict'):
            for name,setup,actual,good,bad in pairs:
                if name == 'unicode' and mode == 'sloppy':
                    setup += "if(false){L:let\nx=1;L:let\n{}}"
                for suffix,value,expected in (('',good,'passed'),('-mismatch',bad,'failed')):
                    variants.append(('labels-'+name+suffix,guard+setup+f'assert.sameValue({actual},{value});',expected,mode))
    if profile == 'equality':
        for operator, label, strict, equal, unequal in (
                ('==','eq',False,'true','false'), ('!=','ne',False,'false','true'),
                ('===','strict-eq',True,'true','false'), ('!==','strict-ne',True,'false','true')):
            guard = "assert.sameValue(new Number(1)==1,true);"
            if strict:
                pairs = [
                    ('types', f"assert.sameValue(false{operator}null,{unequal});"
                     f"assert.sameValue(new Number(1){operator}1,{unequal});"
                     f"assert.sameValue(null{operator}undefined,{unequal});",
                     f"1{operator}'1'", unequal, equal),
                    ('no-hooks', "var calls=0,object={get valueOf(){calls++;throw 'unused';},get toString(){calls++;throw 'unused';}};"
                     f"assert.sameValue(object{operator}1,{unequal});assert.sameValue(object{operator}object,{equal});"
                     f"assert.sameValue(null{operator}object,{unequal});",
                     'calls', '0', '1'),
                    ('identity', "var a={},b={},f=function(){},g=function(){};"
                     f"assert.sameValue(a{operator}b,{unequal});assert.sameValue(f{operator}g,{unequal});"
                     f"assert.sameValue(f{operator}f,{equal});", f'a{operator}a', equal, unequal),
                    ('utf16', f"assert.sameValue('\\uD800'{operator}'\\uDC00',{unequal});"
                     f"assert.sameValue('\\u00E9'{operator}'e\\u0301',{unequal});",
                     f"'\\uD800\\uDC00'{operator}'\\uD800\\uDC00'", equal, unequal),
                    ('numbers', f"assert.sameValue(NaN{operator}NaN,{unequal});"
                     f"assert.sameValue(Infinity{operator}Infinity,{equal});",
                     f'-0{operator}0', equal, unequal),
                    ('evaluation', "var trace='',object={valueOf:function(){throw 'unused';}};"
                     "function a(){trace+='A';return object;}function b(){trace+='B';return 1;}"
                     f"assert.sameValue(a(){operator}b(),{unequal});", 'trace', "'AB'", "'BA'"),
                ]
            else:
                pairs = [
                    ('types', f"assert.sameValue(false{operator}null,{unequal});"
                     f"assert.sameValue(null{operator}undefined,{equal});"
                     f"assert.sameValue(new Boolean(false){operator}'0',{equal});",
                     f"new String('1'){operator}1", equal, unequal),
                    ('live-order', "var trace='',object={get valueOf(){trace+='L';return function(){assert.sameValue(this,object);trace+='l';"
                     "Object.defineProperty(object,'toString',{get:function(){trace+='T';return function(){assert.sameValue(this,object);trace+='t';return '1';};}});return {};};}};"
                     "function a(){trace+='A';return object;}function b(){trace+='B';return 1;}"
                     f"assert.sameValue(a(){operator}b(),{equal});", 'trace', "'ABLlTt'", "'ALlTtB'"),
                    ('abrupt', "var trace='',reason={},seen,object={get valueOf(){trace+='L';throw reason;}};"
                     "function b(){trace+='B';return 1;}"
                     f"try{{object{operator}b();}}catch(e){{seen=e;}}assert.sameValue(seen,reason);",
                     'trace', "'BL'", "'LB'"),
                    ('no-hooks', "var calls=0,object={get valueOf(){calls++;throw 'unused';}};"
                     f"assert.sameValue(object{operator}object,{equal});assert.sameValue(object{operator}{{}},{unequal});"
                     f"assert.sameValue(object{operator}null,{unequal});assert.sameValue(undefined{operator}object,{unequal});",
                     'calls', '0', '1'),
                    ('utf16', f"assert.sameValue(new String('\\uD800'){operator}'\\uDC00',{unequal});"
                     f"assert.sameValue(new String('\\u00E9'){operator}'e\\u0301',{unequal});",
                     f"new String('\\uD800'){operator}'\\uD800'", equal, unequal),
                    ('fallback', f"assert.throws(TypeError,function(){{return Object.create(null){operator}1;}});"
                     f"assert.sameValue({{valueOf:null,toString:function(){{return '1';}}}}{operator}true,{equal});",
                     f"false{operator}''", equal, unequal),
                ]
            for mode in ('sloppy','strict'):
                for name,setup,actual,good,bad in pairs:
                    for suffix,value,expected in (('',good,'passed'),('-mismatch',bad,'failed')):
                        variants.append(('equality-'+label+'-'+name+suffix,
                                         guard+setup+f'assert.sameValue({actual},{value});',expected,mode))
    if profile == 'relational':
        for operator, label, lexical, numeric in (
                ('<','lt','false','true'), ('>','gt','true','false'),
                ('<=','le','false','true'), ('>=','ge','true','false')):
            guard = f"assert.sameValue(new String('2'){operator}new String('10'),{lexical});"
            pairs = [
                ('types', f"assert.sameValue(new Number(2){operator}new String('10'),{numeric});"
                 f"assert.sameValue({{valueOf:function(){{return '2';}}}}{operator}'10',{lexical});"
                 f"assert.sameValue(NaN{operator}1,false);assert.sameValue(1{operator}undefined,false);",
                 f"'2'{operator}'10'", lexical, numeric),
                ('live-order', "var trace='',right={valueOf:function(){throw 'stale';}},left={get valueOf(){trace+='L';"
                 "return function(){assert.sameValue(this,left);trace+='l';right.valueOf=function(){trace+='R';return '10';};return {};};},"
                 "get toString(){trace+='T';return function(){assert.sameValue(this,left);trace+='t';return '2';};}};"
                 "function a(){trace+='A';return left;}function b(){trace+='B';return right;}"
                 f"assert.sameValue(a(){operator}b(),{lexical});",
                 'trace', "'ABLlTtR'", "'ALlTtBR'"),
                ('abrupt', "var trace='',reason={},seen,left={get valueOf(){trace+='L';throw reason;}},"
                 "right={get valueOf(){trace+='R';throw 'unused';}};function b(){trace+='B';return right;}"
                 f'try{{left{operator}b();}}catch(e){{seen=e;}}assert.sameValue(seen,reason);',
                 'trace', "'BL'", "'BLR'"),
                ('utf16', f"assert.sameValue(new String('\\uD800\\uDC00'){operator}'\\uE000',{numeric});"
                 f"assert.sameValue('a'{operator}'aa',{numeric});",
                 f"new String('\\uD800'){operator}'\\uDC00'", numeric, lexical),
                ('fallback', f"assert.sameValue({{valueOf:null,toString:function(){{return '2';}}}}{operator}'10',{lexical});"
                 f"assert.throws(TypeError,function(){{return Object.create(null){operator}1;}});"
                 f"assert.throws(TypeError,function(){{return {{valueOf:function(){{return {{}};}},toString:function(){{return {{}};}}}}{operator}1;}});",
                 f"null{operator}1", numeric, lexical),
                ('chain', "var trace='',left={valueOf:function(){trace+='L';return '2';}},right={valueOf:function(){trace+='R';return '10';}};"
                 "function a(){trace+='A';return left;}function b(){trace+='B';return right;}"
                 "function c(){trace+='C';return {valueOf:function(){trace+='V';return 3;}};}"
                 f'assert.sameValue(a(){operator}b(){operator}c(),{numeric});',
                 'trace', "'ABLRCV'", "'ABCLRV'"),
            ]
            for mode in ('sloppy','strict'):
                for name,setup,actual,good,bad in pairs:
                    for suffix,value,expected in (('',good,'passed'),('-mismatch',bad,'failed')):
                        variants.append(('relational-'+label+'-'+name+suffix,
                                         guard+setup+f'assert.sameValue({actual},{value});',expected,mode))
    if profile == 'uri':
        for function in ('encodeURI', 'encodeURIComponent', 'decodeURI', 'decodeURIComponent'):
            encoding = function.startswith('encode')
            component = function.endswith('Component')
            guard = f"assert.sameValue({function}('a'),'a');"
            unicode_input = "'\\u00E9\\uD83E\\uDD80'" if encoding else "'%C3%A9%F0%9F%A6%80'"
            unicode_output = "'%C3%A9%F0%9F%A6%80'" if encoding else "'\\u00E9\\uD83E\\uDD80'"
            reserved_input = "'/+ #'" if encoding else "'%2f%2B%20%23'"
            reserved_output = ("'%2F%2B%20%23'" if component else "'/+%20#'") if encoding else ("'/+ #'" if component else "'%2f%2B %23'")
            invalid = "'\\uD800'" if encoding else "'%ED%A0%80'"
            pairs = [
                ('unicode', f'assert.sameValue({function}(),"undefined");assert.sameValue({function}(null),"null");',
                 f'{function}({unicode_input})', unicode_output, '"wrong"'),
                ('reserved', f'assert.sameValue({function}("AZaz09-_.!~*\'()"),"AZaz09-_.!~*\'()");',
                 f'{function}({reserved_input})', reserved_output, '"wrong"'),
                ('conversion', "var trace='',object={get toString(){trace+='T';return function(){assert.sameValue(this,object);trace+='t';return {};};},"
                 "get valueOf(){trace+='V';return function(){assert.sameValue(this,object);trace+='v';return 'a';};}};"
                 f"assert.sameValue({function}(object,{{toString:function(){{throw 'unused';}}}}),'a');",
                 'trace', "'TtVv'", "'VvTt'"),
                ('abrupt', "var reason={},seen,object={get toString(){throw reason;},get valueOf(){throw 'unused';}};"
                 f'try{{{function}(object);}}catch(e){{seen=e;}}',
                 'seen', 'reason', 'undefined'),
                ('malformed', f'assert.throws(URIError,function(){{{function}({invalid});}});'
                 f'var caught=false;try{{{function}({invalid});}}catch(e){{caught=e instanceof URIError;}}',
                 'caught', 'true', 'false'),
                ('metadata', f'var saved={function};assert.sameValue(saved.name,"{function}");assert.sameValue(saved.length,1);'
                 'var desc=Object.getOwnPropertyDescriptor(saved,"length");assert.sameValue(desc.writable,false);'
                 'assert.sameValue(desc.enumerable,false);assert.sameValue(desc.configurable,true);'
                 'assert.throws(TypeError,function(){new saved();});',
                 'saved.hasOwnProperty("prototype")', 'false', 'true'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('uri-' + function + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'logical-assignment':
        for operator, label, take, skip in (
                ('&&=', 'and', '1', '0'), ('||=', 'or', '0', '1'),
                ('??=', 'nullish', 'null', '0')):
            guard = f'var enabled={take};assert.sameValue(enabled{operator}2,2);'
            pairs = [
                ('values', f'var a={take},b={skip},calls=0;function rhs(){{calls++;return "value";}}'
                 f'assert.sameValue(a{operator}rhs(),"value");assert.sameValue(a,"value");'
                 f'assert.sameValue(b{operator}rhs(),{skip});assert.sameValue(b,{skip});',
                 'calls', '1', '2'),
                ('skip-reference', f'var trace="",object={{get x(){{trace+="G";return {skip};}},set x(v){{throw "unused";}}}};'
                 'function target(){trace+="O";return object;}function key(){trace+="K";return "x";}'
                 f'assert.sameValue(target()[key()]{operator}(function(){{throw "unused";}})(),{skip});',
                 'trace', '"OKG"', '"OKGS"'),
                ('write-reference', f'var trace="",stored,other={{x:99}},object={{get x(){{trace+="G";return {take};}},'
                 'set x(v){trace+="S";stored=v;}},selected=object;function target(){trace+="O";return selected;}'
                 'function key(){trace+="K";return "x";}function rhs(){trace+="R";selected=other;return 7;}'
                 f'assert.sameValue(target()[key()]{operator}rhs(),7);assert.sameValue(stored,7);assert.sameValue(other.x,99);',
                 'trace', '"OKGRS"', '"OKGORS"'),
                ('abrupt', f'var trace="",reason={{}},seen,object={{get x(){{trace+="G";return {take};}},'
                 'set x(v){trace+="S";}};function rhs(){trace+="R";throw reason;}'
                 f'try{{object.x{operator}rhs();}}catch(e){{seen=e;}}assert.sameValue(seen,reason);',
                 'trace', '"GR"', '"GRS"'),
                ('readonly', f'const held={skip};assert.sameValue(held{operator}(function(){{throw "unused";}})(),{skip});'
                 f'const locked={take};var calls=0;function rhs(){{calls++;return 9;}}'
                 f'assert.throws(TypeError,function(){{locked{operator}rhs();}});',
                 'calls', '1', '0'),
                ('name', f'var named={take};named{operator}function(){{}};'
                 'var desc=Object.getOwnPropertyDescriptor(named,"name");assert.sameValue(desc.writable,false);'
                 'assert.sameValue(desc.enumerable,false);assert.sameValue(desc.configurable,true);',
                 'named.name', '"named"', '""'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('logical-assignment-' + label + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'addition':
        guard = "assert.sameValue({valueOf:function(){return '1';}}+2,'12');"
        pairs = [
            ('primitives', "assert.sameValue(null+true,1);assert.sameValue(undefined+0,NaN);assert.sameValue(Infinity+-Infinity,NaN);"
             "assert.sameValue(-0+-0,-0);assert.sameValue(-0+0,0);assert.sameValue(''+(-0),'0');",
             'true+true', '2', '3'),
            ('boxed', "assert.sameValue(new String('1')+new String('2'),'12');assert.sameValue(new Number(1)+new String('2'),'12');"
             "assert.sameValue(new Boolean(false)+new Number(2),2);",
             "new String('a')+false", "'afalse'", "'a0'"),
            ('evaluation-order', "var trace='',left={get valueOf(){trace+='L';return function(){assert.sameValue(this,left);trace+='l';return {};};},"
             "get toString(){trace+='T';return function(){assert.sameValue(this,left);trace+='t';return 'x';};}},"
             "right={get valueOf(){trace+='R';return function(){assert.sameValue(this,right);trace+='r';return 'y';};}};"
             "function a(){trace+='A';return left;}function b(){trace+='B';return right;}assert.sameValue(a()+b(),'xy');",
             'trace', "'ABLlTtRr'", "'ALlTtBRr'"),
            ('live-conversion', "var trace='',right={valueOf:function(){throw 'stale';}},left={valueOf:function(){trace+='L';"
             "right.valueOf=function(){trace+='R';return 'b';};return 'a';}};assert.sameValue(left+right,'ab');"
             "assert.sameValue({valueOf:null,toString:function(){return 'x';}}+1,'x1');"
             "assert.throws(TypeError,function(){return {valueOf:function(){return {};},toString:function(){return {};}}+1;});",
             'trace', "'LR'", "'RL'"),
            ('abrupt', "var reason={},seen,trace='',left={valueOf:function(){trace+='L';return 'x';}},"
             "right={get valueOf(){trace+='R';throw reason;}};try{left+right;}catch(e){seen=e;}assert.sameValue(seen,reason);"
             "seen=undefined;try{({get valueOf(){throw reason;}})+right;}catch(e){seen=e;}assert.sameValue(seen,reason);",
             'trace', "'LR'", "'LRR'"),
            ('utf16', "var a='\\uD800',b='\\uDC00';var joined={valueOf:function(){return a;}}+{valueOf:function(){return b;}};"
             "assert.sameValue(joined.length,2);assert.sameValue(joined.charCodeAt(0),0xD800);assert.sameValue(joined.charCodeAt(1),0xDC00);"
             "assert.sameValue('x'+null,'xnull');assert.sameValue('x'+undefined,'xundefined');",
             'joined', "'\\uD800\\uDC00'", "'\\uFFFD\\uFFFD'"),
            ('compound-reference', "var trace='',stored,object={get x(){trace+='G';return {valueOf:function(){trace+='L';return 'a';}};},"
             "set x(value){trace+='S';stored=value;}};function target(){trace+='O';return object;}"
             "function key(){trace+='K';return 'x';}function rhs(){trace+='R';return {valueOf:function(){trace+='V';return 'b';}};}"
             "var returned=(target()[key()]+=rhs());assert.sameValue(returned,'ab');assert.sameValue(stored,returned);",
             'trace', "'OKGRLVS'", "'OKGLRVS'"),
            ('compound-abrupt', "var reason={},seen,wrote=false,trace='',object={get x(){trace+='G';return {valueOf:function(){trace+='L';return 'a';}};},"
             "set x(value){wrote=true;}};try{object.x+={valueOf:function(){trace+='R';throw reason;}};}catch(e){seen=e;}"
             "assert.sameValue(seen,reason);assert.sameValue(wrote,false);",
             'trace', "'GLR'", "'GRL'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('addition-' + name + suffix,
                                     guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'compound-assignment':
        for operator, label, result, negative in (
                ('<<=', 'left-shift', 18, -2), ('>>=', 'right-shift', 4, -1),
                ('>>>=', 'unsigned-shift', 4, 2147483647), ('&=', 'and', 1, 1),
                ('^=', 'xor', 8, -2), ('|=', 'or', 9, -1)):
            guard = f'var probe=9;probe{operator}1;assert.sameValue(probe,{result});'
            pairs = [
                ('values', f'var a=9,returned=(a{operator}1);assert.sameValue(a,{result});assert.sameValue(returned,a);'
                 f'var n=-1;n{operator}1;assert.sameValue(n,{negative});var z=-0;z{operator}0;',
                 'z', '0', '-0'),
                ('reference-order', "var trace='',stored,old={valueOf:function(){trace+='L';return 9;}},"
                 "rhsValue={valueOf:function(){trace+='V';return 1;}},object={get x(){trace+='G';return old;},set x(value){trace+='S';stored=value;}};"
                 "function target(){trace+='O';return object;}function key(){trace+='K';return {toString:function(){trace+='C';return 'x';}};}"
                 "function rhs(){trace+='R';return rhsValue;}"
                 f'var returned=(target()[key()]{operator}rhs());assert.sameValue(returned,{result});assert.sameValue(stored,returned);',
                 'trace', "'OKCGRLVS'", "'OKCRGLVS'"),
                ('abrupt-order', "var trace='',reason={},seen,wrote=false,old={valueOf:function(){trace+='L';throw reason;}},"
                 "object={get x(){trace+='G';return old;},set x(value){wrote=true;}};"
                 "function rhs(){trace+='R';return {valueOf:function(){throw 'unused';}};}"
                 f'try{{object.x{operator}rhs();}}catch(e){{seen=e;}}assert.sameValue(seen,reason);assert.sameValue(wrote,false);',
                 'trace', "'GRL'", "'GLR'"),
                ('write-failure', "var object={},trace='',seen,returned;Object.defineProperty(object,'x',{value:9,writable:false});"
                 "function rhs(){trace+='R';return 1;}"
                 f'try{{returned=(object.x{operator}rhs());}}catch(e){{seen=e;}}assert.sameValue(object.x,9);'
                 f'if(@STRICT@){{assert.sameValue(seen.constructor,TypeError);assert.sameValue(returned,undefined);}}'
                 f'else{{assert.sameValue(seen,undefined);assert.sameValue(returned,{result});}}',
                 'trace', "'R'", "''"),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    setup = setup.replace('@STRICT@', str(mode == 'strict').lower())
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('compound-assignment-' + label + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'numeric-parsing':
        for method in ('parseInt', 'parseFloat'):
            guard = (f"var m={method};assert.sameValue(typeof m,'function');"
                     "assert.sameValue(m({toString:function(){return '12';}},10),12);"
                     f"assert.sameValue(Number.{method},m);")
            grammar = ("assert.sameValue(m('  -0tail',10),-0);assert.sameValue(m('0x10'),16);"
                       "assert.sameValue(m('0b11'),0);assert.sameValue(m('11',4294967298),3);"
                       "assert.sameValue(m('11',-4294967294),3);assert.sameValue(m('11',Infinity),11);"
                       "assert.sameValue(m('11',1),NaN);assert.sameValue(m('z',36),35);"
                       if method == 'parseInt' else
                       "assert.sameValue(m('  -0tail'),-0);assert.sameValue(m('1.25e2x'),125);"
                       "assert.sameValue(m('1e+'),1);assert.sameValue(m('0x10'),0);"
                       "assert.sameValue(m('+Infinityx'),Infinity);assert.sameValue(m('.5x'),0.5);"
                       "assert.sameValue(m('.x'),NaN);assert.sameValue(m('inf'),NaN);")
            radix_order = ("var r={get valueOf(){trace+='R';return function(){trace+='r';return 10;};}};"
                           "assert.sameValue(m(o,r),12);" if method == 'parseInt' else
                           "var r={get valueOf(){throw 'ignored';}};assert.sameValue(m(o,r),12);")
            trace = "'TtvRr'" if method == 'parseInt' else "'Ttv'"
            length = '2' if method == 'parseInt' else '1'
            pairs = [
                ('grammar', grammar + "assert.sameValue(m(),NaN);assert.sameValue(m('\\uD80012'),NaN);"
                 "assert.sameValue(m('\\uFEFF12\\uD800'),12);",
                 "m('12tail')", '12', '13'),
                ('hook-radix-order', "var trace='',o={get toString(){trace+='T';return function(){assert.sameValue(this,o);trace+='t';return {};};},"
                 "valueOf:function(){assert.sameValue(this,o);trace+='v';return '12';}};" + radix_order,
                 'trace', trace, "''"),
                ('abrupt', "var reason={},seen,called=false,o={get toString(){throw reason;},valueOf:function(){called=true;return 1;}};"
                 "try{m(o,{valueOf:function(){called=true;return 10;}});}catch(e){seen=e;}"
                 "assert.sameValue(seen,reason);assert.throws(TypeError,function(){m({toString:null,valueOf:null});});",
                 'called', 'false', 'true'),
                ('property-metadata', f"verifyProperty(Number,'{method}',{{value:m,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'name',{{value:'{method}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 f"verifyProperty(m,'length',{{value:{length},writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new m('1');});",
                 'm.length', length, '3'),
                ('arguments-receiver', "var trace='',bomb={get toString(){throw 'receiver';},get valueOf(){throw 'receiver';}};"
                 "function first(){trace+='a';return {toString:function(){trace+='s';return '12';}};}"
                 "function extra(){trace+='b';return bomb;}assert.sameValue(m.call(bomb,first(),10,extra()),12);"
                 "assert.sameValue(m.apply(null,['12',10,bomb]),12);",
                 'trace', "'abs'", "'asb'"),
                ('alias-mutation', f"var owner=Number,alias=owner.{method};{method}=function(){{throw 'global';}};"
                 f"assert.sameValue(owner.{method},alias);owner.{method}=function(){{throw 'static';}};"
                 "Number={};Object.defineProperty(alias,'name',{value:'changed'});"
                 "assert.sameValue(alias.bind(null)({toString:function(){return '13';}},10),13);",
                 "m({toString:function(){return '14';}},10)", '14', '15'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('numeric-parsing-' + method + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'numeric-conversion':
        guard = ("var N=Number;assert.sameValue(typeof N,'function');"
                 "assert.sameValue(N({valueOf:function(){return 5;}}),5);")
        pairs = [
            ('number-primitives', "assert.sameValue(N(),0);assert.sameValue(N(undefined),NaN);"
             "assert.sameValue(N(null),0);assert.sameValue(N(false),0);assert.sameValue(N(true),1);"
             "assert.sameValue(N('-0'),-0);assert.sameValue(N(' 0x10 '),16);assert.sameValue(N('0b11'),3);"
             "assert.sameValue(N('Infinity'),Infinity);assert.sameValue(N('1x'),NaN);",
             "N('')", '0', '1'),
            ('number-hook-order', "var trace='',o={get valueOf(){trace+='V';return function(){assert.sameValue(this,o);trace+='v';return {};};},"
             "get toString(){trace+='T';return function(){assert.sameValue(this,o);trace+='t';return '7';};}};"
             "assert.sameValue(N(o),7);",
             'trace', "'VvTt'", "'TtVv'"),
            ('number-abrupt', "var reason={},seen,called=false;try{N({get valueOf(){throw reason;},toString:function(){called=true;return '1';}});}catch(e){seen=e;}"
             "assert.sameValue(seen,reason);assert.sameValue(called,false);seen=undefined;"
             "try{new N({valueOf:function(){throw reason;}});}catch(e){seen=e;}",
             'seen', 'reason', 'undefined'),
            ('number-noncallable', "var trace='',o={valueOf:7,toString:function(){trace+='t';return '8';}};"
             "assert.sameValue(N(o),8);assert.throws(TypeError,function(){N({valueOf:function(){return {};},toString:function(){return {};}});});",
             'trace', "'t'", "''"),
            ('number-boxing-alias', "var prototype=N.prototype;Number=function(){throw 1;};"
             "var box=new N({valueOf:function(){return -0;}});Number=N;"
             "assert.sameValue(typeof box,'object');assert.sameValue(Object.getPrototypeOf(box),prototype);"
             "assert.sameValue(box.valueOf(),-0);assert.sameValue(new N(undefined).valueOf(),NaN);",
             'N(box)', '-0', '0'),
            ('number-argument-order', "var trace='',bomb={get valueOf(){throw 9;}},o={valueOf:function(){trace+='v';return 4;}};"
             "function first(){trace+='a';return o;}function extra(){trace+='b';return bomb;}"
             "assert.sameValue(N.call(bomb,first(),extra()),4);assert.sameValue(N.apply(null,[o,bomb]),4);",
             'trace', "'abvv'", "'avbv'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('numeric-conversion-' + name + suffix,
                                     guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
        for method in ('isFinite', 'isNaN'):
            converted = 'true' if method == 'isFinite' else 'false'
            guard = (f"var m={method};assert.sameValue(typeof m,'function');"
                     f"assert.sameValue(m({{valueOf:function(){{return 1;}}}}),{converted});")
            pairs = [
                ('coercion', "var reads=0,o={valueOf:function(){reads++;return '2';}};"
                 f"assert.sameValue(m(o),{converted});assert.sameValue(Number.{method}(o),false);"
                 f"assert.sameValue(m.call({{}},null),{converted});assert.sameValue(m([]),{converted});",
                 'reads', '1', '0'),
                ('abrupt-order', "var trace='',reason={},seen;function first(){trace+='a';return {valueOf:function(){trace+='v';throw reason;}};}"
                 "function extra(){trace+='b';return {};}try{m(first(),extra());}catch(e){seen=e;}"
                 "assert.sameValue(seen,reason);assert.throws(TypeError,function(){m({valueOf:null,toString:null});});",
                 'trace', "'abv'", "'avb'"),
                ('property-metadata', f"verifyProperty(m,'name',{{value:'{method}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                 "assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new m(1);});"
                 f"{method}=function(){{throw 1;}};assert.sameValue(m.bind(null)({{valueOf:function(){{return 1;}}}}),{converted});",
                 'm.length', '1', '2'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('numeric-conversion-' + method + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'number-statics':
        # Validate actual callable execution before nonconstructor/descriptor
        # checks; an absent method's TypeError is never positive evidence.
        checks = {
            'isFinite': [('0', True), ('-0', True), ('1.5', True), ('5e-324', True),
                         ('1.7976931348623157e308', True), ('Infinity', False), ('-Infinity', False), ('NaN', False)],
            'isInteger': [('0', True), ('-0', True), ('1', True), ('-1', True),
                          ('9007199254740992', True), ('1.7976931348623157e308', True),
                          ('1.5', False), ('5e-324', False), ('Infinity', False), ('NaN', False)],
            'isNaN': [('NaN', True), ('0/0', True), ('0', False), ('-0', False),
                      ('1.5', False), ('Infinity', False), ('-Infinity', False)],
            'isSafeInteger': [('0', True), ('-0', True), ('9007199254740991', True),
                              ('-9007199254740991', True), ('9007199254740992', False),
                              ('-9007199254740992', False), ('1.5', False), ('5e-324', False),
                              ('1.7976931348623157e308', False), ('Infinity', False), ('NaN', False)],
        }
        for method, values in checks.items():
            canonical = 'NaN' if method == 'isNaN' else '1'
            guard = (f"var m=Number.{method};assert.sameValue(typeof m,'function');"
                     f"assert.sameValue(m({canonical}),true);")
            pairs = [
                ('classification', ''.join(f'assert.sameValue(m({value}),{str(expected).lower()});'
                                           for value, expected in values),
                 f'm({canonical})', 'true', 'false'),
                ('noncoercion', "var reads=0,bomb={};Object.defineProperty(bomb,'valueOf',{get:function(){reads++;throw 7;}});"
                 "Object.defineProperty(bomb,'toString',{get:function(){reads++;throw 8;}});"
                 "var inputs=[undefined,null,true,false,'','1','NaN',[],[1],{},new Number(1),function(){},bomb];"
                 "for(var i=0;i<inputs.length;i++){assert.sameValue(m(inputs[i]),false);}assert.sameValue(m(),false);"
                 f"assert.sameValue(m.call(bomb,{canonical}),true);assert.sameValue(m({canonical},bomb),true);",
                 'reads', '0', '1'),
                ('property-metadata', f"verifyProperty(Number,'{method}',{{value:m,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 "verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                 f"verifyProperty(m,'name',{{value:'{method}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.prototype.hasOwnProperty.call(m,'prototype'),false);"
                 "assert.throws(TypeError,function(){new m();});"
                 f"Number.{method}=function(){{return false;}};assert.sameValue(m.apply(null,[{canonical}]),true);"
                 f"delete Number.{method};assert.sameValue(m.bind({{}})({canonical}),true);",
                 'm.length', '1', '2'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('number-statics-' + method + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
        constants = [('EPSILON', '2.220446049250313e-16'), ('MAX_SAFE_INTEGER', '9007199254740991'),
                     ('MIN_SAFE_INTEGER', '-9007199254740991'), ('MAX_VALUE', '1.7976931348623157e308'),
                     ('MIN_VALUE', '5e-324'), ('NaN', 'NaN'),
                     ('NEGATIVE_INFINITY', '-Infinity'), ('POSITIVE_INFINITY', 'Infinity')]
        constant_guard = ''.join(f"assert.sameValue(typeof Number.{name},'number');assert.sameValue(Number.{name},{value});"
                                 for name, value in constants)
        constant_pairs = [
            ('special-values', "assert.sameValue(typeof Number.NaN,'number');assert.sameValue(Number.NaN,NaN);"
             "assert.sameValue(Number.NEGATIVE_INFINITY,-Infinity);assert.sameValue(Number.POSITIVE_INFINITY,Infinity);"
             "assert.sameValue(1/Number.NEGATIVE_INFINITY,-0);",
             '1/Number.POSITIVE_INFINITY', '0', '-0'),
            ('finite-extremes', "assert.sameValue(Number.MAX_VALUE,1.7976931348623157e308);assert.sameValue(Number.MIN_VALUE,5e-324);"
             "assert.sameValue(Number.MAX_VALUE*2,Infinity);assert.sameValue(Number.MIN_VALUE/2,0);assert.sameValue(Number.MIN_VALUE*2,1e-323);",
             'Number.MIN_VALUE', '5e-324', '2.2250738585072014e-308'),
            ('epsilon', "assert.sameValue(Number.EPSILON,2.220446049250313e-16);assert.notSameValue(1+Number.EPSILON,1);",
             '1+Number.EPSILON/2', '1', '2'),
            ('safe-limits', "assert.sameValue(Number.MAX_SAFE_INTEGER,9007199254740991);assert.sameValue(Number.MIN_SAFE_INTEGER,-9007199254740991);"
             "assert.sameValue(Number.MAX_SAFE_INTEGER+1,9007199254740992);",
             'Number.MIN_SAFE_INTEGER', '-9007199254740991', '-9007199254740992'),
            ('property-constants', constant_guard + ''.join(
                f"verifyProperty(Number,'{name}',{{value:{value},writable:false,enumerable:false,configurable:false}},{{restore:true}});"
                for name, value in constants),
             'Number.MAX_SAFE_INTEGER', '9007199254740991', '0'),
            ('constant-immutability', "assert.sameValue(Number.EPSILON,2.220446049250313e-16);@WRITES@"
             "assert.throws(TypeError,function(){Object.defineProperty(Number,'EPSILON',{value:0});});",
             'Number.EPSILON', '2.220446049250313e-16', '0'),
        ]
        for mode in ('sloppy', 'strict'):
            writes = ("assert.throws(TypeError,function(){Number.EPSILON=0;});assert.throws(TypeError,function(){delete Number.EPSILON;});"
                      if mode == 'strict' else "Number.EPSILON=0;assert.sameValue(delete Number.EPSILON,false);")
            for name, setup, actual, good, bad in constant_pairs:
                setup = setup.replace('@WRITES@', writes)
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('number-statics-' + name + suffix,
                                     setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'array-reduce':
        # Both directions prove successful method execution before any error
        # assertion: an absent method's TypeError is not validation evidence.
        for method, order, digits, first, last in (
                ('reduce', '012', '123', 0, 2), ('reduceRight', '210', '321', 2, 0)):
            guard = (f"var m=Array.prototype.{method};assert.sameValue(typeof m,'function');"
                     "assert.sameValue(m.call([1,2],function(a,b){return a+b;},0),3);")
            pairs = [
                ('direction', "var a=[1,2,3],trace='';var result=m.call(a,function(acc,v,k,o){assert.sameValue(o,a);trace+=k;return acc*10+v;},0);"
                 f"assert.sameValue(trace,'{order}');assert.sameValue(m.call(a,function(acc,v){{return acc*10+v;}}),{digits});",
                 'result', digits, '999'),
                ('initial', "var calls=0,fn=function(){calls++;return 8;};assert.throws(TypeError,function(){m.call([],fn);});"
                 "assert.sameValue(m.call([],fn,undefined),undefined);assert.sameValue(m.call([undefined],fn),undefined);"
                 "var token={valueOf:function(){throw 9;}};assert.sameValue(m.call([],fn,token),token);",
                 'calls', '0', '1'),
                ('holes-undefined', "var a=[,undefined,,4],trace='',undefineds=0;var result=m.call(a,function(acc,v,k){trace+=k;if(v===undefined){undefineds++;return acc;}return acc+v;},0);"
                 f"assert.sameValue(trace,'{'13' if method == 'reduce' else '31'}');assert.sameValue(undefineds,1);"
                 "assert.throws(TypeError,function(){m.call(new Array(4),function(){});});",
                 'result', '4', '5'),
                ('validation-order', "var trace='',reads=0,o={};Object.defineProperty(o,'length',{get:function(){trace+='L';return {valueOf:function(){trace+='V';return 1;}};}});"
                 "Object.defineProperty(o,'0',{get:function(){reads++;return 1;}});assert.throws(TypeError,function(){m.call(o,{},7);});"
                 "assert.throws(TypeError,function(){m.call(null,function(){});});assert.throws(TypeError,function(){m.call(undefined,function(){});});assert.sameValue(reads,0);",
                 'trace', "'LV'", "'VL'"),
                ('inherited', "var p={};Object.defineProperty(p,'0',{value:4,enumerable:false});Object.defineProperty(p,'2',{value:8,enumerable:false});"
                 "var o=Object.create(p);o[1]=7;o.length=3;var calls=0;var result=m.call(o,function(acc,v,k,receiver){assert.sameValue(receiver,o);calls++;return acc+v;});assert.sameValue(calls,2);",
                 'result', '19', '20'),
                ('live-get', f"var o={{length:3,1:7}},trace='';Object.defineProperty(o,'{first}',{{get:function(){{o[{last}]=9;delete o[1];return 1;}}}});"
                 "var result=m.call(o,function(acc,v,k){trace+=k;return acc+v;},0);"
                 f"assert.sameValue(trace,'{first}{last}');assert.sameValue(1 in o,false);",
                 'result', '10', '17'),
                ('captured-length', "var reads=0,o={0:1,1:2,2:3},calls=0;Object.defineProperty(o,'length',{configurable:true,get:function(){reads++;return 3;}});"
                 "var result=m.call(o,function(acc,v,k){if(calls===0){Object.defineProperty(o,'length',{value:0});o[3]=99;}calls++;return acc+v;},0);"
                 "assert.sameValue(reads,1);assert.sameValue(calls,3);assert.sameValue(o.length,0);assert.sameValue(o[3],99);",
                 'result', '6', '105'),
                ('abrupt', "var reason={},caught=0,calls=0,o={length:3};"
                 f"Object.defineProperty(o,'{first}',{{get:function(){{throw reason;}}}});"
                 "try{m.call(o,function(){calls++;},0);}catch(e){assert.sameValue(e,reason);caught++;}assert.sameValue(calls,0);"
                 "var a=[1,2,3];try{m.call(a,function(){calls++;a.changed=7;throw reason;},0);}catch(e){assert.sameValue(e,reason);caught++;}"
                 "assert.sameValue(caught,2);assert.sameValue(a.changed,7);",
                 'calls', '1', '2'),
                ('callback-this', "var realm=this,o=[1,2],calls=0;var result=m.call(o,function(acc,v,k,receiver){'use strict';assert.sameValue(this,undefined);"
                 "assert.sameValue(arguments.length,4);assert.sameValue(receiver,o);calls++;return acc+v;},0);assert.sameValue(result,3);assert.sameValue(calls,2);"
                 "m.call([1],function(){assert.sameValue(this,@THIS@);return 0;},0);var token={};var bound=function(acc,v){'use strict';assert.sameValue(this,token);return acc+v;}.bind(token);",
                 'm.call([2],bound,3)', '5', '6'),
                ('boxed-string', "var boxed,trace='';var result=m.call('ab',function(acc,v,k,o){if(boxed===undefined){boxed=o;}assert.sameValue(o,boxed);"
                 "assert.sameValue(typeof o,'object');assert.sameValue(Object.getPrototypeOf(o),String.prototype);trace+=k;return acc+v;},'');"
                 f"assert.sameValue(trace,'{'01' if method == 'reduce' else '10'}');",
                 'result', "'ab'" if method == 'reduce' else "'ba'", "'wrong'"),
                ('saved-alias', f"Array.prototype.{method}=function(){{return 99;}};var fn=function(a,b){{return a+b;}};"
                 "assert.sameValue(m.apply([1,2],[fn,0]),3);assert.sameValue(m.bind([1,2])(fn,0),3);"
                 f"delete Array.prototype.{method};",
                 'm.call([1,2],fn,0)', '3', '99'),
                ('property-metadata', f"verifyProperty(Array.prototype,'{method}',{{value:m,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 "verifyProperty(m,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                 f"verifyProperty(m,'name',{{value:'{method}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getPrototypeOf(m),Function.prototype);assert.sameValue(Object.prototype.hasOwnProperty.call(m,'prototype'),false);"
                 "assert.throws(TypeError,function(){new m();});",
                 'm.length', '1', '2'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    setup = setup.replace('@THIS@', 'undefined' if mode == 'strict' else 'realm')
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        source = guard + setup + f'assert.sameValue({actual},{value});'
                        variants.append(('array-reduce-' + method + '-' + name + suffix, source, expected, mode))
    if profile == 'identifiers':
        pairs = [
            ('identifier-alias', r"var \u0061=7;var \u{000000000061}lias=9;assert.sameValue(a,7);function f(\u0078){return x;}assert.sameValue(f(11),11);", 'alias', '9', '8'),
            ('identifier-id-properties', r"var ℘=1,ͺ=2;assert.sameValue(\u2118,1);", r'\u037A', '2', '3'),
            ('identifier-parts', r"var á=1,a·=2,a‿=3,a٠=4;assert.sameValue(a\u0301,1);assert.sameValue(a\u00B7,2);assert.sameValue(a\u203F,3);", r'a\u0660', '4', '5'),
            ('identifier-join', r"var a‌b=5,a‍b=6;assert.sameValue(a\u200Cb,5);", r'a\u200Db', '6', '5'),
            ('identifier-supplementary', r"var 𐐀=10;assert.sameValue(\u{10400},10);var \u{000000000010400}x=11;", '𐐀x', '11', '12'),
            ('identifier-normalization', r"var é=3,é=4;assert.sameValue(e\u0301,4);", r'\u00E9', '3', '4'),
            ('identifier-keyword-properties', r"var o={\u0069f:7,\u0074his:8,\u006eull:9};assert.sameValue(o.if,7);assert.sameValue(o.\u0074his,8);", 'o.null', '9', '8'),
            ('identifier-whitespace-literals', "\ufeffvar x\u00a0=\u202f1;if(true){x+=2;}assert.sameValue(false,false);assert.sameValue(null,null);", 'x', '3', '4'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'symbols':
        pairs = [
            ('symbol-identity', "var a=Symbol('x'),b=Symbol('x');", 'a===b', 'false', 'true'),
            ('symbol-registry', "var a=Symbol.for('x'),b=Symbol.for('x');", 'a===b', 'true', 'false'),
            ('symbol-key', "var a=Symbol('x'),b=Symbol('x'),o={};o[a]=3;o[b]=4;", 'o[a]', '3', '4'),
            ('symbol-reflection', "var a=Symbol(),o={x:1};o[a]=2;var keys=Reflect.ownKeys(o);", 'keys[1]===a', 'true', 'false'),
            ('symbol-primitive', "var o={};o[Symbol.toPrimitive]=function(hint){return hint==='number'?7:8;};", '+o', '7', '8'),
            ('symbol-tag', "var o={};o[Symbol.toStringTag]='Custom';", 'Object.prototype.toString.call(o)', "'[object Custom]'", "'[object Object]'"),
            ('symbol-instance', "var o={};o[Symbol.hasInstance]=function(v){return v===3;};", '3 instanceof o', 'true', 'false'),
            ('symbol-json', "var o={x:Symbol()};o[Symbol()]=1;", 'JSON.stringify(o)', "'{}'", "'{x:1}'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'string-concat':
        pairs = [
            ('concat-basic', '', "'a'.concat('b',3)", "'ab3'", "'a3b'"),
            ('concat-generic', '', "String.prototype.concat.call(12,'x')", "'12x'", "'x12'"),
            ('concat-order', "var log='';var r={toString:function(){log+='r';return 'R';}},a={toString:function(){log+='a';return 'A';}};String.prototype.concat.call(r,a);", 'log', "'ra'", "'ar'"),
            ('concat-utf16', r"var text='\ud800'.concat('\udfff');", 'text.charCodeAt(1)', '0xdfff', '0xfffd'),
            ('concat-null', "assert.sameValue(typeof String.prototype.concat,'function');var caught;try{String.prototype.concat.call(null);}catch(e){caught=e;}", 'caught instanceof TypeError', 'true', 'false'),
            ('concat-name', '', 'String.prototype.concat.name', "'concat'", "'wrong'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'regexp-match-search':
        guard = ("assert.sameValue(typeof RegExp.prototype[Symbol.match],'function');"
                 "assert.sameValue(typeof RegExp.prototype[Symbol.search],'function');"
                 "assert.sameValue(RegExp.prototype[Symbol.match].call(/a/,'a')[0],'a');"
                 "assert.sameValue(RegExp.prototype[Symbol.search].call(/a/,'ba'),1);")
        pairs = [
            ('match-hook', "var raw={toString:function(){throw 1;}},p={},out={};p[Symbol.match]=function(v){assert.sameValue(this,p);assert.sameValue(v,raw);return out;};", 'String.prototype.match.call(raw,p)===out', 'true', 'false'),
            ('search-hook', "var raw={toString:function(){throw 1;}},p={},out={};p[Symbol.search]=function(v){assert.sameValue(this,p);assert.sameValue(v,raw);return out;};", 'String.prototype.search.call(raw,p)===out', 'true', 'false'),
            ('match-fallback', "RegExp.prototype[Symbol.match]=function(s){assert.sameValue(s,'abc');assert.sameValue(this.source,'b');return 7;};", "'abc'.match('b')", '7', '8'),
            ('search-fallback', "RegExp.prototype[Symbol.search]=function(s){assert.sameValue(s,'abc');assert.sameValue(this.source,'b');return 7;};", "'abc'.search('b')", '7', '8'),
            ('match-property-descriptor', "var m=RegExp.prototype[Symbol.match];verifyProperty(m,'name',{value:'[Symbol.match]',writable:false,enumerable:false,configurable:true});verifyProperty(RegExp.prototype,Symbol.match,{value:m,writable:true,enumerable:false,configurable:true});assert.throws(TypeError,function(){new m();});", 'm.length', '1', '2'),
            ('search-property-descriptor', "var m=RegExp.prototype[Symbol.search];verifyProperty(m,'name',{value:'[Symbol.search]',writable:false,enumerable:false,configurable:true});verifyProperty(RegExp.prototype,Symbol.search,{value:m,writable:true,enumerable:false,configurable:true});assert.throws(TypeError,function(){new m();});", 'm.length', '1', '2'),
            ('match-unicode', r"var log='',r={flags:'gu',exec:function(){if(this.lastIndex>3)return null;log+=this.lastIndex+',';return {0:''};}};RegExp.prototype[Symbol.match].call(r,'\ud83d\ude00x');", 'log', "'0,2,3,'", "'0,1,2,3,'"),
            ('match-unicode-sets', r"var log='',r={flags:'gv',exec:function(){if(this.lastIndex>3)return null;log+=this.lastIndex+',';return {0:''};}};RegExp.prototype[Symbol.match].call(r,'\ud83d\ude00x');", 'log', "'0,2,3,'", "'0,1,2,3,'"),
            ('search-restore', "var token={},r={lastIndex:-0,exec:function(){assert.sameValue(this.lastIndex,0);this.lastIndex=9;return {index:token};}};assert.sameValue(RegExp.prototype[Symbol.search].call(r,'a'),token);", 'r.lastIndex', '-0', '0'),
            ('search-abrupt', "var token={},caught,r={lastIndex:9,exec:function(){this.lastIndex=3;throw token;}};try{RegExp.prototype[Symbol.search].call(r,'a');}catch(e){caught=e;}assert.sameValue(caught,token);", 'r.lastIndex', '3', '9'),
            ('match-order', "var log='',r={exec:function(s){log+='e';return null;}};Object.defineProperty(r,'flags',{get:function(){log+='f';return {toString:function(){log+='t';return '';}};}});RegExp.prototype[Symbol.match].call(r,{toString:function(){log+='s';return 'a';}});", 'log', "'sfte'", "'fste'"),
            ('match-result', "var n=0,r={flags:'g',exec:function(){n++;return n==1?{0:{toString:function(){return 'x';}}}:null;}};", "RegExp.prototype[Symbol.match].call(r,'x')[0]", "'x'", "'wrong'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('regexp-match-search-' + name + suffix,
                                     guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'array-last-index-of':
        guard = "assert.sameValue(typeof Array.prototype.lastIndexOf,'function');assert.sameValue([1,2,1].lastIndexOf(1),2);"
        pairs = [
            ('presence', '', "[1,2,1].lastIndexOf(1,undefined)", '0', '2'),
            ('negative', '', "[1,2,1].lastIndexOf(1,-2)", '0', '2'),
            ('holes', '', "[,,].lastIndexOf(undefined)", '-1', '1'),
            ('nan', '', "[NaN].lastIndexOf(NaN)", '-1', '0'),
            ('identity', "var a={},b={};", "[a,b,a].lastIndexOf(b)", '1', '2'),
            ('inherited', "var o=Object.create({1:'x'});o.length=3;", "Array.prototype.lastIndexOf.call(o,'x')", '1', '-1'),
            ('boxed-string', '', r"Array.prototype.lastIndexOf.call('a\ud800\udfffa','\udfff')", '2', '1'),
            ('empty', "var n=0,p={valueOf:function(){n++;return 0;}};Array.prototype.lastIndexOf.call({length:0},0,p);", 'n', '0', '1'),
            ('length-order', "var log='',o={0:1};Object.defineProperty(o,'length',{get:function(){log+='l';return {valueOf:function(){log+='n';return 1;}};}});Array.prototype.lastIndexOf.call(o,1,{valueOf:function(){log+='p';return 0;}});", 'log', "'lnp'", "'wrong'"),
            ('live', "var o={length:3,1:'x'};Object.defineProperty(o,'2',{get:function(){delete o[1];o[0]='x';o.length=0;return 'y';}});", "Array.prototype.lastIndexOf.call(o,'x')", '0', '1'),
            ('abrupt', "var marker={},caught,o={length:1};Object.defineProperty(o,'0',{get:function(){throw marker;}});try{Array.prototype.lastIndexOf.call(o,1);}catch(e){caught=e;}", 'caught===marker', 'true', 'false'),
            ('safe-integer', "var o={length:Infinity};o['9007199254740990']='x';", "Array.prototype.lastIndexOf.call(o,'x')", '9007199254740990', '-1'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('array-last-index-of-' + name + suffix,
                                     guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'string-last-index-of':
        guard = "assert.sameValue(typeof String.prototype.lastIndexOf,'function');assert.sameValue('aba'.lastIndexOf('a'),2);"
        pairs = [
            ('overlap', '', "'ababa'.lastIndexOf('aba')", '2', '0'),
            ('position', '', "'ababa'.lastIndexOf('aba',1.9)", '0', '2'),
            ('nan', '', "'aba'.lastIndexOf('a',NaN)", '2', '0'),
            ('empty', '', "'abc'.lastIndexOf('',Infinity)", '3', '0'),
            ('negative', '', "'aba'.lastIndexOf('a',-Infinity)", '0', '2'),
            ('utf16', '', r"'\ud800\udc00\ud800\udc00'.lastIndexOf('\udc00')", '3', '1'),
            ('order', "var log='',r={toString:function(){log+='r';return 'abcabc';}},s={toString:function(){log+='s';return 'bc';}},p={valueOf:function(){log+='p';return 3;}};String.prototype.lastIndexOf.call(r,s,p);", 'log', "'rsp'", "'wrong'"),
            ('position-abrupt', "var marker={},caught,p={valueOf:function(){throw marker;}};try{''.lastIndexOf('longer',p);}catch(e){caught=e;}", 'caught===marker', 'true', 'false'),
            ('nullish', "var caught;try{String.prototype.lastIndexOf.call(null,'x');}catch(e){caught=e;}", 'caught instanceof TypeError', 'true', 'false'),
            ('symbol', "var caught;try{''.lastIndexOf(Symbol());}catch(e){caught=e;}", 'caught instanceof TypeError', 'true', 'false'),
            ('regexp', "var r=/x/;Object.defineProperty(r,Symbol.match,{get:function(){throw 1;}});", "'/x/-/x/'.lastIndexOf(r)", '4', '0'),
            ('descriptor', "var d=Object.getOwnPropertyDescriptor(String.prototype.lastIndexOf,'length');", 'd.value===1&&!d.writable&&!d.enumerable&&d.configurable', 'true', 'false'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('string-last-index-of-' + name + suffix,
                                     guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'regexp-constructor':
        pairs = [
            ('regexp-ctor-identity', "var r={constructor:RegExp};r[Symbol.match]=true;", 'RegExp(r)===r', 'true', 'false'),
            ('regexp-ctor-disabled-match', "var r=/x/;r[Symbol.match]=false;", 'RegExp(r)===r', 'false', 'true'),
            ('regexp-ctor-like', "var r={source:'a',flags:'i'};r[Symbol.match]=true;", "new RegExp(r).test('A')", 'true', 'false'),
            ('regexp-ctor-order', "var log='',r={},nt=(function(){}).bind(null);Object.defineProperty(r,Symbol.match,{get:function(){log+='m';return true;}});Object.defineProperty(r,'source',{get:function(){log+='s';return {toString:function(){log+='t';return 'a';}};}});Object.defineProperty(r,'flags',{get:function(){log+='f';return 'g';}});Object.defineProperty(nt,'prototype',{get:function(){log+='p';return Object.prototype;}});Reflect.construct(RegExp,[r],nt);", 'log', "'msfpt'", "'wrong'"),
            ('regexp-ctor-match-abrupt', "var marker={},caught,r={};Object.defineProperty(r,Symbol.match,{get:function(){throw marker;}});try{new RegExp(r,'g');}catch(e){caught=e;}", 'caught===marker', 'true', 'false'),
            ('regexp-ctor-conversion', "var p=function(){},f=[];p.toString=function(){return 'a';};f.toString=function(){return 'g';};var r=new RegExp(p,f);", 'r.source+r.flags', "'ag'", "'wrong'"),
            ('regexp-ctor-classify-slot', "var n=0,r=/x/g;Object.defineProperty(r,Symbol.match,{get:function(){n++;return false;}});new RegExp(r);", 'n', '1', '0'),
            ('regexp-ctor-flags-error', "var marker={},caught,r={source:'a'};r[Symbol.match]=true;Object.defineProperty(r,'flags',{get:function(){throw marker;}});try{new RegExp(r);}catch(e){caught=e;}", 'caught===marker', 'true', 'false'),
            ('regexp-ctor-explicit-flags', "var r={source:'a'};r[Symbol.match]=true;Object.defineProperty(r,'flags',{get:function(){throw 1;}});", "new RegExp(r,'i').source", "'a'", "'wrong'"),
            ('regexp-ctor-split', "var r={source:'b',flags:''};r[Symbol.match]=true;", "RegExp.prototype[Symbol.split].call(r,'abc').join('|')", "'a|c'", "'wrong'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'regexp-split':
        pairs = [
            ('regexp-split-default', "var r=/(b)/;", "'abc'.split(r).join('|')", "'a|b|c'", "'wrong'"),
            ('regexp-split-receiver', "var r=/x/;r.lastIndex=9;'axb'.split(r);", 'r.lastIndex', '9', '0'),
            ('regexp-split-species-getter', "var get=Object.getOwnPropertyDescriptor(RegExp,Symbol.species).get;", 'get.call(null)', 'null', 'undefined'),
            ('regexp-split-species-args', "var r={flags:'g',constructor:{}},seen;r.constructor[Symbol.species]=function(p,f){assert.sameValue(p,r);seen=f;return {exec:function(){return null;}};};RegExp.prototype[Symbol.split].call(r,'x');", 'seen', "'gy'", "'g'"),
            ('regexp-split-order', "var log='',r={flags:'',constructor:{}};r.constructor[Symbol.species]=function(){log+='c';return {exec:function(){throw 1;}};};RegExp.prototype[Symbol.split].call(r,{toString:function(){log+='s';return 'x';}},{valueOf:function(){log+='l';return 0;}});", 'log', "'scl'", "'slc'"),
            ('regexp-split-empty', "var r={flags:'',constructor:{}},n=0;r.constructor[Symbol.species]=function(){var s={exec:function(){n++;return null;}};Object.defineProperty(s,'lastIndex',{set:function(){throw 1;}});return s;};var a=RegExp.prototype[Symbol.split].call(r,'');assert.sameValue(a[0],'');", 'n', '1', '0'),
            ('regexp-split-capture-limit', "var r={flags:'',constructor:{}},marker={};r.constructor[Symbol.species]=function(){return {exec:function(){this.lastIndex=1;return {length:Infinity,1:marker};}};};var a=RegExp.prototype[Symbol.split].call(r,'ab',2);", 'a[1]===marker', 'true', 'false'),
            ('regexp-split-index-clamp', "var r={flags:'',constructor:{}};r.constructor[Symbol.species]=function(){return {exec:function(){this.lastIndex=Infinity;return {length:0};}};};", "RegExp.prototype[Symbol.split].call(r,'abc').join('|')", "'|'", "'wrong'"),
            ('regexp-split-null-hook', "var r=/x/;r[Symbol.split]=null;", "'a/x/b'.split(r).join('|')", "'a|b'", "'wrong'"),
            ('regexp-split-unicode', r"var log='',r={flags:'v',constructor:{}};r.constructor[Symbol.species]=function(){return {exec:function(){log+=this.lastIndex;return null;}};};RegExp.prototype[Symbol.split].call(r,'\ud800\udc00x');", 'log', "'02'", "'012'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'string-search':
        pairs = [
            ('string-function-receiver', "var f=function(){};f.toString=function(){return 'abc';};", "String.prototype.slice.call(f,1)", "'bc'", "'wrong'"),
            ('string-array-receiver', "var a=[];a.toString=function(){return 'abc';};", "String.prototype.charCodeAt.call(a,1)", '98', '99'),
            ('string-function-needle', "var f=function(){};f.toString=function(){return 'bc';};", "'abcd'.indexOf(f)", '1', '2'),
            ('string-regexp-override', "var r=/x/;r[Symbol.match]=false;r.toString=function(){return 'x';};", "'x'.includes(r)", 'true', 'false'),
            ('string-match-abrupt', "var marker={},caught,o={};Object.defineProperty(o,Symbol.match,{get:function(){throw marker;}});try{'x'.endsWith(o);}catch(e){caught=e;}", 'caught===marker', 'true', 'false'),
            ('string-search-order', "var log='',r={toString:function(){log+='r';return 'abc';}},s={toString:function(){log+='s';return 'b';}},p={valueOf:function(){log+='p';return 1;}};Object.defineProperty(s,Symbol.match,{get:function(){log+='m';return false;}});String.prototype.includes.call(r,s,p);", 'log', "'rmsp'", "'wrong'"),
            ('string-split-order', "var log='',s={toString:function(){log+='s';return ',';}},l={valueOf:function(){log+='l';return 0;}};'a,b'.split(s,l);", 'log', "'ls'", "'wrong'"),
            ('string-split-hook', "var r={},l={},s={},out={};s[Symbol.split]=function(a,b){assert.sameValue(this,s);assert.sameValue(a,r);assert.sameValue(b,l);return out;};", 'String.prototype.split.call(r,s,l)===out', 'true', 'false'),
            ('string-split-abrupt', "var marker={},caught,s={};Object.defineProperty(s,Symbol.split,{get:function(){throw marker;}});try{''.split(s);}catch(e){caught=e;}", 'caught===marker', 'true', 'false'),
            ('string-split-utf16', r"var s=[];s.toString=function(){return '';};var parts='\ud800\udfff'.split(s);", 'parts[1].charCodeAt(0)', '0xdfff', '0xfffd'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'function-constructor':
        pairs = [
            ('dynamic-call', "var f=Function('a','return a+1');", 'f(3)', '4', '5'),
            ('dynamic-global', "var x=1;function outer(){var x=2;return Function('return x');}", 'outer()()', '1', '2'),
            ('dynamic-strict', "var f=Function('return this');", 'f()===globalThis', 'true', 'false'),
            ('dynamic-target', "var F=Function('return new.target');", '(new F())===F', 'true', 'false'),
            ('dynamic-prototype', "var B=(function(){}).bind(null);Object.defineProperty(B,'prototype',{value:1});var f=Reflect.construct(Function,[''],B);", 'Object.getPrototypeOf(f)===Function.prototype', 'true', 'false'),
            ('dynamic-name', "var anonymous=7;", "Function('return anonymous')()", '7', '8'),
            ('dynamic-boundary', "var caught;try{Function('/*','*/ ) {');}catch(e){caught=e;}", 'caught instanceof SyntaxError', 'true', 'false'),
            ('dynamic-order', "var log='',a={toString:function(){log+='a';return ')';}},b={toString:function(){log+='b';return '';}};try{Function(a,b);}catch(e){assert.sameValue(e instanceof SyntaxError,true);}", 'log', "'ab'", "'a'"),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile in {'reflect-construction', 'new-target'}:
        pairs = [
            ('target-call', 'function F(){return new.target;}', 'F()', 'undefined', 'F'),
            ('target-new', 'var t;function F(){t=new.target;}new F();', 't', 'F', 'undefined'),
            ('target-arrow', 'function F(a=()=>new.target){return a;}var f=new F();', 'f()', 'F', 'undefined'),
            ('target-alternate', 'var t;function F(){t=new.target;}function G(){}Reflect.construct(F,[],G);', 't', 'G', 'F'),
            ('target-bound', 'var t;function F(){t=new.target;}var B=F.bind(null);new B();', 't', 'F', 'B'),
            ('target-prototype', 'function F(){}function G(){}var x=Reflect.construct(F,[],G);', 'Object.getPrototypeOf(x)', 'G.prototype', 'F.prototype'),
            ('target-order', "var log='';var B=(function(){}).bind(null);Object.defineProperty(B,'prototype',{get:function(){log+='p';return {};}});Reflect.construct(function(){log+='f';},{get length(){log+='l';return 1;},get 0(){log+='0';}},B);", 'log', "'l0pf'", "'pl0f'"),
            ('target-object', 'function G(){}var original={},x=Reflect.construct(Object,[original],G);', 'x===original', 'false', 'true'),
            ('apply-result', 'function f(a,b){return a+b;}var result=Reflect.apply(f,null,{0:2,1:3,length:2});', 'result', '5', '6'),
            ('apply-receiver', 'var receiver={};function f(){return this;}var result=Reflect.apply(f,receiver,[]);', 'result', 'receiver', 'undefined'),
            ('apply-validation', "assert.sameValue(typeof Reflect.apply,'function');var count=0,caught;try{Reflect.apply({},null,{get length(){count++;}});}catch(e){caught=e;}assert.sameValue(count,0);", 'caught instanceof TypeError', 'true', 'false'),
            ('construct-validation', "assert.sameValue(typeof Reflect.construct,'function');var count=0,caught;try{Reflect.construct(function(){},{get length(){count++;}},undefined);}catch(e){caught=e;}assert.sameValue(count,0);", 'caught instanceof TypeError', 'true', 'false'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile in {'symbols', 'reflect-construction', 'new-target'}:
        pairs = [
            ('symbol-alternate-target', 'var x=Reflect.construct(Object,[],Symbol);', 'Object.getPrototypeOf(x)===Symbol.prototype', 'true', 'false'),
            ('symbol-construction-order', "var log='',caught;try{Reflect.construct(Symbol,{get length(){log+='l';return 1;},get 0(){log+='0';return {toString:function(){log+='s';return 'x';}};}});}catch(e){caught=e;}assert.sameValue(caught instanceof TypeError,true);", 'log', "'l0'", "''"),
            ('symbol-bound-target', 'var B=Symbol.bind(null);var x=Reflect.construct(Object,[],B);', 'Object.getPrototypeOf(x)===Object.prototype', 'true', 'false'),
            ('symbol-argument-abrupt', 'var marker={},caught;try{Reflect.construct(Symbol,{get length(){throw marker;}});}catch(e){caught=e;}', 'caught===marker', 'true', 'false'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append((name + suffix, setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'array-descriptors':
        guard = ("assert.sameValue(typeof Object.defineProperty,'function');assert.sameValue(typeof Object.defineProperties,'function');"
                 "var smoke=[];assert.sameValue(Object.defineProperty(smoke,'0',{value:7,writable:true,enumerable:true,configurable:true}),smoke);"
                 "assert.sameValue(smoke[0],7);assert.sameValue(Object.defineProperties(smoke,{'0':{value:8}}),smoke);assert.sameValue(smoke[0],8);")
        pairs = [
            ('index-defaults', "var a=[];Object.defineProperty(a,'0',{value:5});var d=Object.getOwnPropertyDescriptor(a,'0');"
             "assert.sameValue(d.value,5);assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);",
             'a.length', '1', '0'),
            ('accessor-receiver', "var a=[],stored=3;var get=function(){assert.sameValue(this,a);return stored;};var set=function(v){assert.sameValue(this,a);stored=v;};"
             "Object.defineProperty(a,'1',{get:get,set:set,enumerable:true,configurable:true});a[1]=11;var d=Object.getOwnPropertyDescriptor(a,'1');"
             "assert.sameValue(d.get,get);assert.sameValue(d.set,set);assert.sameValue(a.length,2);",
             'a[1]', '11', '3'),
            ('index-growth', "var a=[];Object.defineProperty(a,'3',{value:9,writable:true,enumerable:true,configurable:true});"
             "assert.sameValue(0 in a,false);assert.sameValue(2 in a,false);assert.sameValue(a[3],9);",
             'a.length', '4', '3'),
            ('length-shrink', "var a=[1,2,3];assert.sameValue(Object.defineProperty(a,'length',{value:1}),a);"
             "assert.sameValue(a[0],1);assert.sameValue(1 in a,false);assert.sameValue(2 in a,false);",
             'a.length', '1', '3'),
            ('shrink-rollback', "var a=[0,1,2,3];Object.defineProperty(a,'2',{configurable:false});"
             "assert.throws(TypeError,function(){Object.defineProperty(a,'length',{value:1,writable:false});});"
             "assert.sameValue(a.length,3);assert.sameValue(3 in a,false);assert.sameValue(a[2],2);",
             "Object.getOwnPropertyDescriptor(a,'length').writable", 'false', 'true'),
            ('nonwritable-length', "var a=[1];Object.defineProperty(a,'length',{writable:false});"
             "assert.throws(TypeError,function(){Object.defineProperty(a,'1',{value:2});});Object.defineProperty(a,'0',{value:8});"
             "assert.sameValue(a.length,1);assert.sameValue(1 in a,false);",
             'a[0]', '8', '1'),
            ('length-coercion-order', "var a=[1,2,3],trace='',value={valueOf:function(){trace+='N';return 2;}},d={};"
             "Object.defineProperty(d,'value',{get:function(){trace+='V';return value;}});Object.defineProperty(d,'writable',{get:function(){trace+='W';return true;}});"
             "Object.defineProperty(a,'length',d);assert.sameValue(a.length,2);",
             'trace', "'VWNN'", "'VWN'"),
            ('invalid-length', "var a=[1,2];assert.throws(RangeError,function(){Object.defineProperty(a,'length',{value:1.5,writable:false});});"
             "assert.sameValue(a.length,2);assert.sameValue(a[1],2);",
             "Object.getOwnPropertyDescriptor(a,'length').writable", 'true', 'false'),
            ('define-properties-staging', "var a=[1],map={},reason={},trace='',caught=false;"
             "Object.defineProperty(map,'0',{enumerable:true,get:function(){trace+='A';return {value:9};}});"
             "Object.defineProperty(map,'1',{enumerable:true,get:function(){trace+='B';throw reason;}});"
             "try{Object.defineProperties(a,map);}catch(e){assert.sameValue(e,reason);caught=true;}assert(caught);assert.sameValue(trace,'AB');assert.sameValue(a.length,1);",
             'a[0]', '1', '9'),
            ('define-properties-partial', "var a=[0,1];Object.defineProperty(a,'1',{writable:false,configurable:false});"
             "assert.throws(TypeError,function(){Object.defineProperties(a,{'0':{value:7},'1':{value:8}});});assert.sameValue(a[1],1);",
             'a[0]', '7', '0'),
            ('property-keys', "var a=[],calls=0,key={toString:function(){calls++;return '0';}},s=Symbol('x');"
             "Object.defineProperty(a,key,{value:4});Object.defineProperty(a,s,{value:5});Object.defineProperty(a,'01',{value:6});"
             "Object.defineProperty(a,'4294967295',{value:7});assert.sameValue(calls,1);assert.sameValue(a[s],5);assert.sameValue(a[Symbol('x')],undefined);assert.sameValue(a['01'],6);",
             'a.length', '1', '2'),
            ('reflect-false-order', "var reflect=Reflect.defineProperty;assert.sameValue(typeof reflect,'function');assert.sameValue(reflect(smoke,'0',{value:9}),true);"
             "var a=[1],trace='',key={toString:function(){trace+='K';return '1';}},d={};Object.defineProperty(a,'length',{writable:false});"
             "Object.defineProperty(d,'value',{get:function(){trace+='V';return 2;}});Object.defineProperty(d,'writable',{get:function(){trace+='W';return true;}});"
             "var result=reflect(a,key,d);assert.sameValue(trace,'KVW');assert.sameValue(a.length,1);assert.sameValue(1 in a,false);",
             'result', 'false', 'true'),
            ('property-metadata', "var one=Object.defineProperty,many=Object.defineProperties;"
             "verifyProperty(Object,'defineProperty',{value:one,writable:true,enumerable:false,configurable:true},{restore:true});"
             "verifyProperty(Object,'defineProperties',{value:many,writable:true,enumerable:false,configurable:true},{restore:true});"
             "verifyProperty(one,'length',{value:3,writable:false,enumerable:false,configurable:true},{restore:true});"
             "verifyProperty(many,'length',{value:2,writable:false,enumerable:false,configurable:true},{restore:true});"
             "verifyProperty(one,'name',{value:'defineProperty',writable:false,enumerable:false,configurable:true},{restore:true});"
             "verifyProperty(many,'name',{value:'defineProperties',writable:false,enumerable:false,configurable:true},{restore:true});"
             "assert.throws(TypeError,function(){new one();});assert.throws(TypeError,function(){new many();});",
             'many.length', '2', '3'),
        ]
        for mode in ('sloppy', 'strict'):
            for name, setup, actual, good, bad in pairs:
                for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                    variants.append(('array-descriptors-' + name + suffix,
                                     guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'array-predicates':
        for method, neutral in (('every', 'true'), ('some', 'false')):
            terminal = 'false' if method == 'every' else 'true'
            guard = (f"var predicate=Array.prototype.{method};assert.sameValue(typeof predicate,'function');"
                     "var smokeCalls=0;assert.sameValue(predicate.call([1],function(){smokeCalls++;return true;}),true);"
                     "assert.sameValue(predicate.call([1],function(){smokeCalls++;return false;}),false);assert.sameValue(smokeCalls,2);")
            pairs = [
                ('empty', "var calls=0;var result=predicate.call([],function(){calls++;throw new Error('empty');});assert.sameValue(calls,0);",
                 'result', neutral, terminal),
                ('callback-order', "var a=[4,5,6],trace='';var result=predicate.call(a,function(value,index,object){"
                 "assert.sameValue(arguments.length,3);assert.sameValue(object,a);trace+=index+':'+value+';';return " + neutral + ";});"
                 "assert.sameValue(result," + neutral + ");",
                 'trace', "'0:4;1:5;2:6;'", "'2:6;1:5;0:4;'"),
                ('live-inherited-indices', "var proto={1:2},a=Object.create(proto),trace='';a.length=4;a[0]=1;a[2]=3;"
                 "predicate.call(a,function(value,index,object){assert.sameValue(object,a);trace+=index+':'+value+';';"
                 "if(index===0){delete a[2];a[3]=4;a[4]=5;a.length=6;}return " + neutral + ";});",
                 'trace', "'0:1;1:2;3:4;'", "'0:1;1:2;2:3;3:4;4:5;'"),
                ('boolean-conversion', "var coerced=0,value={valueOf:function(){coerced++;throw new Error('coercion');},"
                 "toString:function(){coerced++;throw new Error('coercion');}};"
                 "var no=predicate.call([1],function(){return 0;}),yes=predicate.call([1],function(){return value;});"
                 "assert.sameValue(coerced,0);assert.sameValue(no,false);",
                 'yes', 'true', 'false'),
                ('short-circuit', "var a=[1,2,3],trace='';Object.defineProperty(a,'2',{get:function(){throw new Error('late read');}});"
                 "var result=predicate.call(a,function(value,index){trace+=index;return index===0?" + neutral + ':' + terminal + ";});"
                 "assert.sameValue(result," + terminal + ");",
                 'trace', "'01'", "'012'"),
                ('length-before-callback', "var trace='',a={};Object.defineProperty(a,'length',{get:function(){trace+='L';"
                 "return {valueOf:function(){trace+='N';return 0;}};}});"
                 "assert.throws(TypeError,function(){predicate.call(a,null);});",
                 'trace', "'LN'", "''"),
                ('abrupt-callback', "var reason={},caught=false,trace='';try{predicate.call([1,2],function(value,index){trace+=index;throw reason;});}"
                 "catch(error){assert.sameValue(error,reason);caught=true;}assert.sameValue(caught,true);",
                 'trace', "'0'", "'01'"),
                ('this-argument', "var calls=0,object={};predicate.call([1],function(){'use strict';calls++;assert.sameValue(this,undefined);return " + neutral + ";});"
                 "predicate.call([1],function(){'use strict';calls++;assert.sameValue(this,object);return " + neutral + ";},object);"
                 "predicate.call([1],function(){'use strict';calls++;assert.sameValue(this,7);return " + neutral + ";},7);",
                 'calls', '3', '2'),
                ('generic-string', "var trace='';predicate.call('ab',function(value,index,object){assert.sameValue(object.length,2);"
                 "trace+=index+':'+value+';';return " + neutral + ";});",
                 'trace', "'0:a;1:b;'", "'1:b;0:a;'"),
                ('no-species', "var a=[1],ctor={},calls=0;Object.defineProperty(ctor,Symbol.species,{get:function(){throw new Error('species');}});"
                 "a.constructor=ctor;assert.sameValue(predicate.call(a,function(){calls++;return " + neutral + ";})," + neutral + ");"
                 "Object.defineProperty(a,'constructor',{get:function(){throw new Error('constructor');}});"
                 "assert.sameValue(predicate.call(a,function(){calls++;return " + neutral + ";})," + neutral + ");",
                 'calls', '2', '1'),
                ('invalid-receiver-callback', "assert.throws(TypeError,function(){predicate.call(null,function(){});});"
                 "assert.throws(TypeError,function(){predicate.call(undefined,function(){});});"
                 "assert.throws(TypeError,function(){predicate.call([],{});});"
                 "var calls=0;predicate.call([1],function(){calls++;return " + neutral + ";});",
                 'calls', '1', '0'),
                ('property-metadata', "verifyProperty(Array.prototype,'" + method + "',{value:predicate,writable:true,enumerable:false,configurable:true},{restore:true});"
                 "verifyProperty(predicate,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                 "verifyProperty(predicate,'name',{value:'" + method + "',writable:false,enumerable:false,configurable:true},{restore:true});"
                 "assert.throws(TypeError,function(){new predicate(function(){});});",
                 'predicate.length', '1', '2'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('array-predicates-' + method + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'object-integrity':
        for method in ('seal', 'freeze', 'isSealed', 'isFrozen'):
            mutating = method in {'seal', 'freeze'}
            frozen = method in {'freeze', 'isFrozen'}
            writable = 'false' if frozen else 'true'
            guard = f"var integrity=Object.{method};assert.sameValue(typeof integrity,'function');var smoke={{x:1}};"
            if mutating:
                guard += ("assert.sameValue(integrity(smoke),smoke);assert.sameValue(Object.isExtensible(smoke),false);"
                          "assert.sameValue(Object.getOwnPropertyDescriptor(smoke,'x').configurable,false);"
                          "assert.sameValue(Object.getOwnPropertyDescriptor(smoke,'x').writable," + writable + ");")
            else:
                guard += ("assert.sameValue(integrity(smoke),false);var emptySmoke={};Object.preventExtensions(emptySmoke);"
                          "assert.sameValue(integrity(emptySmoke),true);")
            primitive = ("var symbol=Symbol('integrity');" + ''.join(
                f"assert.sameValue(integrity({value}),{value if mutating else 'true'});"
                for value in ('undefined', 'null', 'true', '1', '-0', 'NaN', "'text'")))
            metadata = (f"verifyProperty(Object,'{method}',{{value:integrity,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                        "verifyProperty(integrity,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                        f"verifyProperty(integrity,'name',{{value:'{method}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                        "assert.sameValue(Object.getOwnPropertyDescriptor(integrity,'prototype'),undefined);"
                        "assert.throws(TypeError,function(){new integrity({});});")
            pairs = [('primitives', primitive, 'integrity(symbol)', 'symbol' if mutating else 'true', 'false')]
            if mutating:
                pairs += [
                    ('data-flags', "var o={};Object.defineProperty(o,'x',{value:3,writable:true,enumerable:false,configurable:true});"
                     "integrity(o);var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.value,3);assert.sameValue(d.enumerable,false);"
                     "assert.sameValue(d.configurable,false);assert.sameValue(Object.isExtensible(o),false);",
                     'd.writable', writable, 'true' if frozen else 'false'),
                    ('accessor-preserved', "var o={},reads=0,stored=1;var get=function(){reads++;return stored;};var set=function(v){stored=v;};"
                     "Object.defineProperty(o,'x',{get:get,set:set,enumerable:true,configurable:true});integrity(o);"
                     "var d=Object.getOwnPropertyDescriptor(o,'x');assert.sameValue(d.get,get);assert.sameValue(d.set,set);"
                     "assert.sameValue(d.configurable,false);assert.sameValue(d.enumerable,true);assert.sameValue('writable' in d,false);"
                     "assert.sameValue(reads,0);o.x=7;",
                     'stored', '7', '1'),
                    ('array-holes-length', "var a=[1,,3];integrity(a);assert.sameValue(a.length,3);assert.sameValue(1 in a,false);"
                     "assert.sameValue(Object.getOwnPropertyDescriptor(a,'0').configurable,false);"
                     "assert.sameValue(Object.getOwnPropertyDescriptor(a,'0').writable," + writable + ");"
                     "assert.sameValue(Reflect.defineProperty(a,'1',{value:2}),false);",
                     "Object.getOwnPropertyDescriptor(a,'length').writable", writable, 'true' if frozen else 'false'),
                    ('symbol-nonenumerable', "var o={},key=Symbol('own');Object.defineProperty(o,key,{value:4,writable:true,configurable:true});"
                     "integrity(o);var d=Object.getOwnPropertyDescriptor(o,key);assert.sameValue(d.value,4);"
                     "assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);",
                     'd.writable', writable, 'true' if frozen else 'false'),
                    ('own-only-shallow', "var proto={inherited:1},o=Object.create(proto);o.child={n:2};integrity(o);o.child.n=9;"
                     "assert.sameValue(Object.getOwnPropertyDescriptor(proto,'inherited').configurable,true);"
                     "assert.sameValue(Object.isExtensible(proto),true);assert.sameValue(Object.getOwnPropertyDescriptor(o,'inherited'),undefined);",
                     'o.child.n', '9', '2'),
                    ('mapped-arguments', "var probe=Function('integrity','a','integrity(arguments);a=9;return arguments[1];');",
                     'probe(integrity,3)', '3' if frozen else '9', '9' if frozen else '3'),
                    ('unmapped-arguments', "function probe(a){'use strict';integrity(arguments);a=9;assert.sameValue(arguments[0],3);"
                     + ("var args=arguments;assert.throws(TypeError,function(){args[0]=7;});" if frozen else "arguments[0]=7;")
                     + "assert.sameValue(a,9);return arguments[0];}",
                     'probe(3)', '3' if frozen else '7', '7' if frozen else '3'),
                    ('write-delete-rules', "var o={x:1};integrity(o);assert.throws(TypeError,function(){'use strict';delete o.x;});"
                     "assert.throws(TypeError,function(){'use strict';o.newValue=3;});"
                     + ("assert.throws(TypeError,function(){'use strict';o.x=8;});assert.throws(TypeError,function(){Object.defineProperty(o,'x',{value:8});});"
                        if frozen else "o.x=7;Object.defineProperty(o,'x',{value:8});"),
                     'o.x', '1' if frozen else '8', '8' if frozen else '1'),
                    ('idempotent-nonextensible', "var o={x:1};Object.preventExtensions(o);assert.sameValue(integrity(o),o);"
                     "assert.sameValue(integrity(o),o);assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').configurable,false);",
                     'Object.isExtensible(o)', 'false', 'true'),
                    ('boxed-string', "var o=new String('ab');assert.sameValue(integrity(o),o);"
                     "var d=Object.getOwnPropertyDescriptor(o,'0');assert.sameValue(d.value,'a');assert.sameValue(d.writable,false);"
                     "assert.sameValue(d.configurable,false);assert.sameValue(Object.isExtensible(o),false);",
                     "o[1]", "'b'", "'a'"),
                ]
            else:
                pairs += [
                    ('empty-transition', "var o={};assert.sameValue(integrity(o),false);Object.preventExtensions(o);",
                     'integrity(o)', 'true', 'false'),
                    ('data-flags', "var o={x:1};Object.preventExtensions(o);assert.sameValue(integrity(o),false);"
                     "Object.defineProperty(o,'x',{configurable:false});var intermediate=integrity(o);"
                     "Object.defineProperty(o,'x',{writable:false});assert.sameValue(integrity(o),true);",
                     'intermediate', 'false' if frozen else 'true', 'true' if frozen else 'false'),
                    ('accessor-no-invocation', "var o={},reads=0,stored=1,get=function(){reads++;return stored;},set=function(v){stored=v;};"
                     "Object.defineProperty(o,'x',{get:get,set:set,configurable:false});Object.preventExtensions(o);"
                     "assert.sameValue(integrity(o),true);o.x=7;assert.sameValue(stored,7);"
                     "assert.sameValue(Object.getOwnPropertyDescriptor(o,'x').get,get);",
                     'reads', '0', '1'),
                    ('configurable-accessor', "var o={};Object.defineProperty(o,'x',{get:undefined,set:undefined,configurable:true});"
                     "Object.preventExtensions(o);var initial=integrity(o);Object.defineProperty(o,'x',{configurable:false});"
                     "assert.sameValue(integrity(o),true);",
                     'initial', 'false', 'true'),
                    ('array-length', "var a=[];Object.preventExtensions(a);var initial=integrity(a);"
                     "Object.defineProperty(a,'length',{writable:false});assert.sameValue(integrity(a),true);",
                     'initial', 'false' if frozen else 'true', 'true' if frozen else 'false'),
                    ('array-index-hole', "var a=[1,,3];Object.defineProperty(a,'length',{writable:false});"
                     "Object.defineProperty(a,'0',{configurable:false});Object.defineProperty(a,'2',{configurable:false});"
                     "Object.preventExtensions(a);assert.sameValue(integrity(a)," + ('false' if frozen else 'true') + ");"
                     "Object.defineProperty(a,'0',{writable:false});Object.defineProperty(a,'2',{writable:false});assert.sameValue(1 in a,false);",
                     'integrity(a)', 'true', 'false'),
                    ('symbol-nonenumerable', "var o={},key=Symbol('own');Object.defineProperty(o,key,{value:3,writable:false,configurable:true});"
                     "Object.preventExtensions(o);var initial=integrity(o);Object.defineProperty(o,key,{configurable:false});"
                     "assert.sameValue(integrity(o),true);",
                     'initial', 'false', 'true'),
                    ('own-only', "var proto={inherited:1},o=Object.create(proto);Object.preventExtensions(o);"
                     "assert.sameValue(Object.getOwnPropertyDescriptor(proto,'inherited').configurable,true);"
                     "assert.sameValue(Object.isExtensible(proto),true);",
                     'integrity(o)', 'true', 'false'),
                    ('mapped-arguments', "var probe=Function('query','a','Object.seal(arguments);var result=query(arguments);a=9;"
                     "if(arguments[1]!==9)throw new Error(\"lost mapping\");return result;');",
                     'probe(integrity,3)', 'false' if frozen else 'true', 'true' if frozen else 'false'),
                    ('unmapped-arguments', "function probe(a){'use strict';Object.freeze(arguments);a=9;"
                     "assert.sameValue(arguments[0],3);return integrity(arguments);}",
                     'probe(3)', 'true', 'false'),
                ]
            pairs.append(('property-metadata', metadata, 'integrity.length', '1', '2'))
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('object-integrity-' + method + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'array-find':
        for method in ('find', 'findIndex', 'findLast', 'findLastIndex'):
            reverse = method in ('findLast', 'findLastIndex')
            indexed = method in ('findIndex', 'findLastIndex')
            first, last = (2, 0) if reverse else (0, 2)
            absent = '-1' if indexed else 'undefined'
            guard = (f"var find=Array.prototype.{method};assert.sameValue(typeof find,'function');"
                     "var smokeCalls=0;assert.sameValue(find.call([7],function(){smokeCalls++;return true;}),"
                     + ('0' if indexed else '7') + ");"
                     "assert.sameValue(find.call([7],function(){smokeCalls++;return false;})," + absent + ");"
                     "assert.sameValue(smokeCalls,2);")
            pairs = [
                ('empty', "var calls=0;var result=find.call([],function(){calls++;throw new Error('empty');});assert.sameValue(calls,0);",
                 'result', absent, '0'),
                ('direction-arguments', "var a=['a','b','c'],trace='';var result=find.call(a,function(value,index,object){"
                 "assert.sameValue(arguments.length,3);assert.sameValue(object,a);trace+=index+':'+value+';';return false;});"
                 "assert.sameValue(result," + absent + ");",
                 'trace', "'2:c;1:b;0:a;'" if reverse else "'0:a;1:b;2:c;'", "''"),
                ('holes-inherited', "var a=Object.create({1:9}),trace='';a.length=3;find.call(a,function(value,index,object){"
                 "assert.sameValue(object,a);trace+=index+':'+value+';';return false;});assert.sameValue(0 in a,false);assert.sameValue(2 in a,false);",
                 'trace', "'2:undefined;1:9;0:undefined;'" if reverse else "'0:undefined;1:9;2:undefined;'", "'1:9;'"),
                ('captured-value', "var before={},after={},a=[before],seen;var result=find.call(a,function(value,index){"
                 "seen=value;assert.sameValue(index,0);a[0]=after;return true;});assert.sameValue(a[0],after);assert.sameValue(seen,before);",
                 'result', '0' if indexed else 'before', '1' if indexed else 'after'),
                ('boolean-short-circuit', "var a=['a','b','c'],trace='',coerced=0,truthy={valueOf:function(){coerced++;throw new Error('coercion');},"
                 "toString:function(){coerced++;throw new Error('coercion');}};"
                 f"Object.defineProperty(a,'{last}',{{get:function(){{throw new Error('late read');}}}});"
                 "var result=find.call(a,function(value,index){trace+=index;return index===1?truthy:0;});"
                 "assert.sameValue(coerced,0);assert.sameValue(trace," + ("'21'" if reverse else "'01'") + ");",
                 'result', '1' if indexed else "'b'", absent),
                ('length-snapshot-live', "var a=[1,2,3],trace='';var result=find.call(a,function(value,index){trace+=index+':'+value+';';"
                 f"if(index==={first}){{delete a[1];a[{last}]=9;a.length=5;a[3]=4;a[4]=5;}}return false;}});"
                 "assert.sameValue(result," + absent + ");",
                 'trace', "'2:3;1:undefined;0:9;'" if reverse else "'0:1;1:undefined;2:9;'", "''"),
                ('length-before-callback', "var a={},trace='';Object.defineProperty(a,'length',{get:function(){trace+='L';"
                 "return {valueOf:function(){trace+='N';return 0;}};}});"
                 "assert.throws(TypeError,function(){find.call(a,42);});",
                 'trace', "'LN'", "''"),
                ('receiver-validation', "var calls=0,callback=function(){calls++;return true;};"
                 "assert.throws(TypeError,function(){find.call(null,callback);});"
                 "assert.throws(TypeError,function(){find.call(undefined,callback);});",
                 'calls', '0', '1'),
                ('strict-thisarg', "var marker={},calls=0;find.call([1],function(){'use strict';assert.sameValue(this,marker);calls++;return false;},marker);"
                 "find.call([1],function(){'use strict';assert.sameValue(this,undefined);calls++;return false;});"
                 "find.call([1],function(){'use strict';assert.sameValue(this,3);calls++;return false;},3);",
                 'calls', '3', '0'),
                ('sloppy-thisarg', "var boxed=Function('return typeof this===\"object\" && this.valueOf()===3;');"
                 "var global=Function('return this===globalThis;');"
                 "assert.sameValue(find.call([1],boxed,3)," + ('0' if indexed else '1') + ");"
                 "var result=find.call([1],global);",
                 'result', '0' if indexed else '1', absent),
                ('callback-abrupt-identity', "var reason={},caught=false,trace='';try{find.call([1,2,3],function(value,index){trace+=index;throw reason;});}"
                 "catch(e){assert.sameValue(e,reason);caught=true;}assert.sameValue(caught,true);",
                 'trace', "'2'" if reverse else "'0'", "'012'"),
                ('getter-abrupt-identity', "var a={length:3},reason={},caught=false,calls=0;"
                 f"Object.defineProperty(a,'{first}',{{get:function(){{throw reason;}}}});"
                 "try{find.call(a,function(){calls++;return false;});}catch(e){assert.sameValue(e,reason);caught=true;}assert.sameValue(caught,true);",
                 'calls', '0', '1'),
                ('live-getters', "var a={length:3},trace='';"
                 f"Object.defineProperty(a,'{first}',{{get:function(){{trace+='G';return 7;}}}});"
                 "find.call(a,function(value,index){trace+='C'+index;"
                 f"if(index==={first}){{assert.sameValue(value,7);Object.defineProperty(a,'1',{{get:function(){{trace+='H';return 8;}}}});}}"
                 "if(index===1)assert.sameValue(value,8);return false;});",
                 'trace', "'GC2HC1C0'" if reverse else "'GC0HC1C2'", "''"),
                ('generic-string', "var trace='',result=find.call('abc',function(value,index,object){"
                 "assert.sameValue(typeof object,'object');assert.sameValue(object.length,3);trace+=index+value;return value==='b';});"
                 "assert.sameValue(trace," + ("'2c1b'" if reverse else "'0a1b'") + ");",
                 'result', '1' if indexed else "'b'", absent),
                ('no-species', "var a=[4],reads=0;Object.defineProperty(a,'constructor',{get:function(){reads++;throw new Error('constructor');}});"
                 "var result=find.call(a,function(){return true;});assert.sameValue(result," + ('0' if indexed else '4') + ");",
                 'reads', '0', '1'),
                ('property-metadata', f"verifyProperty(Array.prototype,'{method}',{{value:find,writable:true,enumerable:false,configurable:true}},{{restore:true}});"
                 "verifyProperty(find,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});"
                 f"verifyProperty(find,'name',{{value:'{method}',writable:false,enumerable:false,configurable:true}},{{restore:true}});"
                 "assert.sameValue(Object.getOwnPropertyDescriptor(find,'prototype'),undefined);"
                 "assert.throws(TypeError,function(){new find(function(){return true;});});",
                 'find.length', '1', '2'),
            ]
            for mode in ('sloppy', 'strict'):
                for name, setup, actual, good, bad in pairs:
                    for suffix, value, expected in (('', good, 'passed'), ('-mismatch', bad, 'failed')):
                        variants.append(('array-find-' + method + '-' + name + suffix,
                                         guard + setup + f'assert.sameValue({actual},{value});', expected, mode))
    if profile == 'date':
        variants += date_preflight_variants()
    if profile == 'for-of':
        variants += for_of_preflight_variants()
    if profile == 'core-iterators':
        variants += core_iterator_preflight_variants()
    if profile == 'array-from':
        variants += array_from_preflight_variants()
    if profile == 'array-concat':
        variants += array_concat_preflight_variants()
    if profile == 'array-splice':
        variants += array_splice_preflight_variants()
    outcomes = []
    identifier_controls = {}
    for name, source, expected, mode in variants:
        includes = ['propertyHelper.js'] if 'property-' in name else []
        case = dict(id=f'harness-preflight:{name}', file='<preflight>', mode=mode,
                    metadata=dict(flags=[], includes=includes, features=[], locale=[], negative=None),
                    source=source.encode(), harness=[(item, files[f'harness/{item}']) for item in ['assert.js', 'sta.js'] + includes])
        case['case_sha256'] = case_fingerprint(case)
        result = run_case(case, binary, timeout)
        correct = result['status'] == expected
        if expected == 'failed':
            correct = correct and result.get('observation', {}).get('error_type') == 'Test262Error'
        outcomes.append(dict(name=name, expected=expected, verified=correct, result=result))
        if profile == 'identifiers' and name.startswith('identifier-') and expected == 'passed':
            identifier_controls[(name, mode)] = dict(
                name=name, mode=mode, case_sha256=case['case_sha256'],
                source_sha256=digest(case['source']), expected=expected,
                verified=correct, result=result)
    if profile in {'array-concat', 'array-splice', 'array-from', 'date', 'for-of', 'core-iterators'}:
        # An unavailable prerequisite can throw the harness's Test262Error in both partners.
        # Preserve that raw observation, but do not verify the mismatch unless
        # its separately executed positive partner completes successfully.
        positives = {(item['name'], item['result']['mode']): item for item in outcomes
                     if item['name'].startswith(profile + '-') and item['expected'] == 'passed'}
        for item in outcomes:
            if item['name'].startswith(profile + '-') and item['name'].endswith('-mismatch'):
                partner = positives[(item['name'].removesuffix('-mismatch'), item['result']['mode'])]
                result = partner['result']
                item['prerequisite'] = dict(name=partner['name'], mode=result['mode'],
                    case_sha256=result['case_sha256'], source_sha256=result['source_sha256'],
                    expected=partner['expected'], verified=partner['verified'])
                item['verified'] = item['verified'] and partner['verified']
    if profile == 'identifiers':
        # Parse-negative observations alone can pass on an old lexer that
        # rejects every escape. Require a separately executed valid control,
        # retaining both the raw negative result and the control's identity.
        invalid = [
            ('escaped-keyword-binding', r'var \u0069f=1;', 'identifier-alias'),
            ('escaped-keyword-terminal', r'\u0069f(true){}', 'identifier-keyword-properties'),
            ('escaped-literal', r'var x=tr\u0075e;', 'identifier-keyword-properties'),
            ('escaped-this', r'var x=th\u0069s;', 'identifier-keyword-properties'),
            ('initial-mark', r'var \u0300x=1;', 'identifier-parts'),
            ('initial-digit', r'var \u0030x=1;', 'identifier-alias'),
            ('surrogate', r'var \uD800=1;', 'identifier-supplementary'),
            ('surrogate-pair', r'var \uD801\uDC00=1;', 'identifier-supplementary'),
            ('empty-braced', r'var \u{}=1;', 'identifier-alias'),
            ('outside-scalar', r'var \u{110000}=1;', 'identifier-supplementary'),
            ('numeric-adjacency', r'var x=1\u0061;', 'identifier-alias'),
            ('forbidden-nel', 'var\u0085x=1;', 'identifier-whitespace-literals'),
        ]
        for mode in ('sloppy', 'strict'):
            for label, source, control in invalid:
                name = 'identifier-syntax-' + label
                case = dict(id=f'harness-preflight:{name}', file='<preflight>', mode=mode,
                            metadata=dict(flags=[], includes=[], features=[], locale=[],
                                          negative=dict(phase='parse', type='SyntaxError')),
                            source=source.encode(),
                            harness=[(item, files[f'harness/{item}']) for item in ('assert.js', 'sta.js')])
                case['case_sha256'] = case_fingerprint(case)
                result = run_case(case, binary, timeout)
                prerequisite = identifier_controls[(control, mode)]
                outcomes.append(dict(name=name, expected='passed',
                                     verified=result['status'] == 'passed' and prerequisite['verified'],
                                     result=result, prerequisite=prerequisite))
    return outcomes


def check_baseline(previous, current):
    for key in ('revision', 'corpus_manifest_sha256', 'runner_policy_sha256'):
        if previous.get(key) != current[key]:
            raise ValueError(f'baseline {key} differs; explicitly review a new baseline')
    if previous['cases'].keys() != current['cases'].keys():
        raise ValueError('baseline Test262 mode inventory differs')
    regressions, improvements = [], []
    for name, old in previous['cases'].items():
        new = current['cases'][name]
        if old.get('case_sha256') != new['case_sha256']:
            raise ValueError(f'baseline source/metadata/mode/harness differs: {name}')
        if old['status'] == 'passed' and new['status'] != 'passed':
            regressions.append(name)
        if old['status'] != 'passed' and new['status'] == 'passed':
            improvements.append(name)
    return regressions, improvements


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/eris-js')
    parser.add_argument('--profile', choices=PROFILES, default='string-json')
    parser.add_argument('--corpus', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--record-baseline', type=Path)
    parser.add_argument('--jobs', type=int, default=4)
    parser.add_argument('--timeout', type=float, default=3.0)
    args = parser.parse_args()
    args.corpus = args.corpus or ROOT / 'tests/upstream' / corpus_name(args.profile)
    args.output = args.output or ROOT / 'artifacts' / f'{corpus_name(args.profile)}-report.json'
    if args.baseline and args.record_baseline:
        parser.error('baseline checking and recording are mutually exclusive')
    if any(path is not None and paths_alias(path, args.output)
           for path in (args.baseline, args.record_baseline)):
        parser.error('report output must be separate from the baseline path')
    if not 1 <= args.jobs <= 16 or not 0 < args.timeout <= 60:
        parser.error('jobs must be 1..16 and timeout must be (0,60] seconds')
    started = time.monotonic()
    try:
        manifest, files, cases, fixtures, corpus_hash = load_corpus(args.corpus, args.profile)
        binary = args.binary.resolve()
        binary_hash = digest(binary.read_bytes())
        preflight = harness_preflight(files, binary, args.timeout, args.profile)
        preflight_ok = all(item['verified'] for item in preflight)
        supported_features = PROFILE_FEATURES[args.profile]
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
            results = list(pool.map(lambda case: run_case(case, binary, args.timeout, supported_features), cases))
        counts = dict(Counter(result['status'] for result in results))
        policy = dict(format=2, supported_features=sorted(supported_features),
                      negative_intrinsic_errors=sorted(INTRINSIC_ERRORS), strict=True,
                      modules=False, async_completion=False, host_hooks=False,
                      timeout_seconds=args.timeout)
        policy_hash = digest(json.dumps(policy, sort_keys=True).encode())
        if digest(binary.read_bytes()) != binary_hash:
            raise ValueError('adapter binary changed during the run; repeat with a stable build')
        current = dict(revision=manifest['revision'], corpus_manifest_sha256=corpus_hash,
                       runner_policy_sha256=policy_hash, binary_sha256=binary_hash,
                       cases={item['id']: {key: item[key] for key in ('status', 'case_sha256', 'source_sha256')}
                              for item in results})
        regressions, improvements = [], []
        if args.baseline:
            regressions, improvements = check_baseline(json.loads(args.baseline.read_bytes()), current)
        report = dict(suite=f'pinned-test262-{args.profile}-selection', full_test262_conformance=False,
                      repository=manifest['repository'], revision=manifest['revision'],
                      corpus_manifest_sha256=corpus_hash, runner_policy=policy, runner_policy_sha256=policy_hash,
                      source_files=manifest['test_files'], source_tests=len({case['file'] for case in cases}),
                      negative_source_tests=len({case['file'] for case in cases if case['metadata']['negative']}),
                      fixture_files=fixtures, mode_cases=len(cases), counts=counts,
                      seconds=round(time.monotonic() - started, 3), binary=str(binary), binary_sha256=binary_hash,
                      harness_preflight=preflight, harness_preflight_passed=preflight_ok,
                      regressions=regressions, improvements=improvements, cases=results)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2, ensure_ascii=True) + '\n', encoding='utf-8')
        healthy = preflight_ok and not any(counts.get(status) for status in BAD_RUN_STATUSES)
        if args.record_baseline and healthy:
            args.record_baseline.parent.mkdir(parents=True, exist_ok=True)
            args.record_baseline.write_text(json.dumps(current, indent=2) + '\n', encoding='utf-8')
        print(f'{manifest["test_files"]} pinned files; {len(cases)} mode cases; {counts}')
        print(f'Upstream harness preflight: {"passed" if preflight_ok else "FAILED"}')
        if args.baseline:
            print(f'{len(regressions)} regressions; {len(improvements)} newly passing cases')
        print(f'Report: {args.output}')
        if not healthy or regressions:
            return 1
        if args.baseline or args.record_baseline:
            return 0
        return 0 if counts.get('passed', 0) == len(cases) else 1
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f'Test262 runner: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
