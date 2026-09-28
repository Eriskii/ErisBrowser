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
                            REPOSITORY, REVISION, corpus_name, parse_metadata)

ROOT = Path(__file__).resolve().parents[1]
SUPPORTED_FEATURES = {'arrow-function', 'String.fromCodePoint', 'well-formed-json-stringify', 'for-in-order'}
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
PROFILE_FEATURES = {'symbols': SYMBOL_FEATURES, 'string-json': SUPPORTED_FEATURES, 'regexp': REGEXP_FEATURES,
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
