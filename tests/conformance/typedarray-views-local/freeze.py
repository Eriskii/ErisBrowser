"""Inert authoring: freeze source identities and expectations; no project imports or JS parsing."""
from pathlib import Path
import hashlib
import json
import re
import shutil

ROOT = Path('/tmp/eris-typedarray-views-1')
DEST = ROOT / 'local-fixtures'
WORK = ROOT / 'worktree'

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def binding(path):
    raw = path.read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': digest(raw)}

def write(name, value):
    with (DEST / name).open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')

fixture = (DEST / 'cases.js').read_bytes()
names = re.findall(rb'^  ([a-z][a-z0-9_]*): function \(\)', fixture, re.M)
assert len(names) == len(set(names)) == 23
names = [name.decode() for name in names]
controls = json.loads((DEST / 'controls.json').read_text())
assert len(controls['controls']) == 12 and controls['pairs'] == 6
complete = {'status': 'complete', 'phase': 'runtime', 'error_type': '', 'error_identity': ''}
wrong = {'status': 'exception', 'phase': 'runtime', 'error_type': 'Error', 'error_identity': 'Error'}
metadata = dict(flags=[], features=[], includes=[], locale=[], negative=None)

refs = [ROOT / 'published-base.json', ROOT / 'source-ready.json', ROOT / 'proposal.md',
        ROOT / 'primary-sources.json', ROOT / 'join-expansion/source-ready.json',
        ROOT / 'join-expansion/proposal.md',
        WORK / 'tests/conformance/typedarray-foundation-local/cases.js',
        WORK / 'tests/conformance/typedarray-foundation-local/controls.json',
        WORK / 'tests/conformance/typedarray-foundation-local/manifest.json',
        WORK / 'tests/conformance/reflect-properties-local/cases.js',
        WORK / 'tests/conformance/reflect-properties-local/controls.json',
        WORK / 'tests/conformance/reflect-properties-local/manifest.json',
        WORK / 'tools/test262_conformance.py']
(DEST / 'inputs').mkdir(exist_ok=True)
inputs = []
for index, path in enumerate(refs):
    row = binding(path)
    held = DEST / 'inputs' / (f'{index:02d}-' + path.name)
    assert not held.exists()
    shutil.copyfile(path, held)
    assert held.read_bytes() == path.read_bytes()
    row['held_copy'] = str(held)
    inputs.append(row)
base = json.loads((ROOT / 'published-base.json').read_text())
assert base['commit'] == 'a3c0bb1d5167236cdf676247f17aea3720dcb866'
write('context.json', {'schema': 1, 'published_base': base['commit'],
      'before_adapter_sha256': 'c21074ad37ed121547e6386dd529c67c6c438a6a0b2a6583d453118c49ef78e8',
      'status': 'Independent expectations only. No local baseline or candidate execution by this author.',
      'selected_scope': 'Additive join-expansion supersedes only the smaller scope prediction, preserving the original proposal and hold.',
      'existing_populations': 'Original foundation and Reflect local case/control/manifest bytes retained without modification. New local population is separate from 103 upstream source bodies / 206 modes.',
      'inputs': inputs})
write('inventory.json', {'schema': 1, 'collection': 'typedArrayViewCases', 'status': 'source-only frozen expectations; execution pending',
      'cases': names, 'named_bodies': 23, 'modes': ['sloppy', 'strict'], 'mode_cases': 46,
      'case_invocation_arguments': [],
      'case_suffix_template': '\nif (typedArrayViewCases.CASE_NAME() !== true) throw new Error("case did not return true");\n',
      'ordinary_observation': dict(complete, return_value=True),
      'controls': {'pairs': 6, 'modes': 24}, 'total_mode_observations': 70,
      'number_kinds': ['Int8Array', 'Uint8Array', 'Uint8ClampedArray', 'Int16Array', 'Uint16Array', 'Int32Array', 'Uint32Array', 'Float16Array', 'Float32Array', 'Float64Array'],
      'excluded_scope': ['BigInt views', 'shared or immutable buffers', 'Proxy', 'foreign realms', 'class-based subclasses', 'unrelated TypedArray methods', 'implementation-specific resource tariffs'],
      'published_base': base['commit']})
urls = {'subarray': 'https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.subarray',
        'join': 'https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.join',
        'alias': 'https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.tostring',
        'species': 'https://tc39.es/ecma262/multipage/indexed-collections.html#sec-typedarrayspeciescreate',
        'result': 'https://tc39.es/ecma262/multipage/indexed-collections.html#sec-typedarraycreatefromconstructor',
        'array_to_string': 'https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.tostring'}
write('spec-notes.json', {'schema': 1, 'sources': urls,
      'review': 'Primary algorithm page read directly with browser tool during source-only authoring. No text is copied into the JS fixture.',
      'oracle_rules': [
          'Subarray records initial extent and stored offset. Initially invalid source extent is zero; index coercion and species still occur.',
          'Tracking source plus undefined end supplies two constructor arguments; other cases supply three. End objects are coerced even when they yield undefined.',
          'Species resolution follows index coercion. Constructed result must be a currently valid Number typed view; a shorter/different-kind result is allowed.',
          'Join validates initially, then converts separator once. Its iteration count is captured, but elements reflect later resize/detach.',
          'Empty valid join still converts separator; absent elements contribute empty fields. Output preserves UTF-16 units.',
          'The toString property stores the existing Array function; its callable-join path is generic and returns the direct call result.'
      ], 'limitations': 'Examples are bounded independent witnesses, not a full conformance claim. Parent executes unchanged sources only after source review.'})

matrix = []
for kind, items in [('case', [{'name': name} for name in names]), ('control', controls['controls'])]:
    for row in items:
        if kind == 'control':
            assert row['source_sha256'] == digest(row['source'].encode())
            assert row['expected_observation'] == (complete if row['positive'] else wrong)
            assert (row['asserted_observation'] == row['literal_observation']) == row['positive']
            positive = next(item for item in controls['controls'] if item['name'] == row['pair'] + '-positive')
            tail = 'if(actual!==' + json.dumps(row['asserted_observation']) + ')throw new Error(' + json.dumps(row['pair'] + ' expectation') + ');'
            positive_tail = 'if(actual!==' + json.dumps(positive['asserted_observation']) + ')throw new Error(' + json.dumps(row['pair'] + ' expectation') + ');'
            assert row['source'].endswith(tail) and positive['source'].endswith(positive_tail)
            assert row['source'][:-len(tail)] == positive['source'][:-len(positive_tail)]
        for mode in ['sloppy', 'strict']:
            body = (fixture + b'\nif (typedArrayViewCases.' + row['name'].encode() + b'() !== true) throw new Error("case did not return true");\n') if kind == 'case' else row['source'].encode()
            identity = {'source_sha256': digest(body), 'mode': mode, 'metadata': metadata, 'harness': []}
            item = {'id': row['name'] + '#' + mode, 'mode': mode, 'kind': kind, 'source_sha256': digest(body),
                    'case_sha256': digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode()),
                    'expected_observation': complete if kind == 'case' else row['expected_observation']}
            if kind == 'control':
                item.update(pair=row['pair'], positive=row['positive'], partner=row['pair'] + '-positive#' + mode)
            matrix.append(item)
assert len(matrix) == len({row['id'] for row in matrix}) == 70
write('matrix.json', matrix)
files = ['cases.js', 'controls.json', 'README.md', 'design.md', 'inventory.json', 'spec-notes.json', 'context.json', 'matrix.json', 'freeze.py']
write('manifest.json', {'schema': 1, 'collection': 'typedArrayViewCases', 'case_count': 23, 'case_modes': 46, 'control_pairs': 6,
      'control_modes': 24, 'case_invocation_arguments': [], 'ordinary_observation': dict(complete, return_value=True),
      'files': [dict(path=name, bytes=(DEST/name).stat().st_size, sha256=digest((DEST/name).read_bytes())) for name in files]})
files.append('manifest.json')
(DEST / 'held').mkdir(exist_ok=True)
held_rows=[]
for name in files:
    path=DEST/name;held=DEST/'held'/name
    assert not held.exists();shutil.copyfile(path,held)
    row=binding(path);row['held_copy']=str(held);held_rows.append(row)
write('source-ready.json', {'schema':1, 'status':'Frozen independent source expectations; no parser, compiler, adapter, browser or project-test execution',
      'published_base':base['commit'], 'cases':23,'case_modes':46,'control_pairs':6,'control_modes':24,'total_observations':70,
      'files':held_rows, 'context_copies':[binding(Path(row['held_copy'])) for row in inputs],
      'validation':'Inert byte/JSON checks only: 23 unique textual named declarations, 12 exact paired sources, 70 unique mode/fingerprint rows. Future mode handling and positive-partner health must follow README contract.',
      'review_status':'Script incremental source read clear; final hash-bound peer pending.'})
print(json.dumps(binding(DEST/'source-ready.json')))
print('Fixture',json.dumps(binding(DEST/'cases.js')))
print('Controls',json.dumps(binding(DEST/'controls.json')))
print('Matrix',json.dumps(binding(DEST/'matrix.json')))
