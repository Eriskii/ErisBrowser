from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

A = Path(__file__).resolve().parents[1]
W = A / 'worktree'
workflow = W / '.github/workflows/check.yml'
text = workflow.read_text()
start = text.index('          from pathlib import Path\n', text.index('python3 - "$listing"'))
end = text.index('          PY\n', start)
source = ''.join(line[10:] for line in text[start:end].splitlines(True))
records = {}
for label, tests in [('bridge-list-ci-fix-1', 'bridge-tests-ci-fix-1'),
                     ('bridge-presenter-list-ci-fix-1', 'bridge-presenter-tests-ci-fix-1')]:
    listing = A / 'validation' / (label + '.log')
    run = subprocess.run([sys.executable, '-', str(listing)], input=source, text=True,
                         capture_output=True, cwd=W, check=True)
    receipt = json.loads((A / 'validation' / (tests + '.json')).read_text())
    log = A / 'validation' / receipt['log']
    assert receipt['exit'] == 0 and hashlib.sha256(log.read_bytes()).hexdigest() == receipt['sha256']
    assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', log.read_text()) == [('192', '0', '0')]
    records[label] = {'validation': run.stdout.strip(), 'test_receipt': receipt,
                      'listing_sha256': hashlib.sha256(listing.read_bytes()).hexdigest()}
result = {'status': 'pass', 'workflow_sha256': hashlib.sha256(workflow.read_bytes()).hexdigest(),
          'tests_per_feature': 192, 'features': ['raster-bridge', 'raster-bridge,vulkan-presenter'],
          'checks': records, 'source_changes': 'Explicit CI roster adds the nine new box_core_tests; old183 names and executable Rust source unchanged.',
          'original_failure': {'url': 'https://github.com/Eriskii/ErisBrowser/actions/runs/38530546529',
                               'job': 115741118156, 'reason': 'found192, missing[], extra ninebox_core_tests'}}
with (A / 'root/ci-roster-validation.json').open('x') as out:
    json.dump(result, out, indent=2); out.write('\n')
print(json.dumps(result))
