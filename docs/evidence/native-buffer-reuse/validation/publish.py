from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser')
RUN = Path(__file__).resolve().parent
OUT = ROOT / 'docs/evidence/native-buffer-reuse'
assert not OUT.exists()
host = json.loads((RUN / 'comparison-1/timing-host-results.json').read_text())
probe = json.loads((RUN / 'reuse-gpu-1/host-results.json').read_text())
assert host['success'] and len(host['runs']) == 8
assert probe['success'] and probe['adapter_count'] == 3
for row in host['bindings']['source_files']:
    data = (ROOT / row['path']).read_bytes()
    assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], row['path']
for variant in host['bindings']['variants'].values():
    for row in variant['release']['source_files']:
        data = (Path(variant['source_directory']) / row['path']).read_bytes()
        assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], row['path']
for row in json.loads((RUN / 'probe-build-1.json').read_text())['source_files']:
    data = (ROOT / row['path']).read_bytes()
    assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], row['path']

files = {}
def add(path, name):
    assert path.is_file() and not path.is_symlink() and name not in files, name
    files[name] = path

for parent in ['comparison-1', 'reuse-gpu-1']:
    for path in sorted((RUN / parent).rglob('*')):
        if path.is_file(): add(path, parent + '/' + path.relative_to(RUN / parent).as_posix())
for path in sorted(RUN.iterdir()):
    if path.is_file() and path.suffix in ('.json', '.log', '.py', '.md', '.rs'):
        add(path, 'validation/' + path.name)
for directory, prefix in [('/tmp/eris-native-buffer-reuse-core-1', 'core-validation'),
                          ('/tmp/eris-native-buffer-reuse-host-checks', 'host-validation')]:
    for path in sorted(Path(directory).iterdir()):
        if path.is_file(): add(path, prefix + '/' + path.name)
for name in [
    'eris-native-buffer-reuse-core-source-ready.json',
    'eris-native-buffer-reuse-presenter-source-ready.json',
    'eris-native-buffer-reuse-host-source-ready.json',
    'eris-native-buffer-reuse-host-script-review.json',
    'eris-native-buffer-reuse-presenter-script-review.json',
    'eris-native-buffer-reuse-presenter-review.json',
    'eris-native-buffer-reuse-presenter-review-timing-addendum.json',
    'eris-native-buffer-reuse-gpu-review.json',
    'eris-native-buffer-reuse-actual-review.json',
]:
    add(Path('/tmp') / name, 'review/' + name)

# Runtime/host overlays plus the immutable base commit reconstruct source inputs
# without repeating the entire 120-file old release source tree in this package.
names = set(subprocess.check_output(['git', 'diff', '--name-only'], cwd=ROOT, text=True).splitlines())
names.update(subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard'], cwd=ROOT, text=True).splitlines())
for name in sorted(names):
    path = ROOT / name
    if name.startswith(('src/', 'crates/', 'tools/')) and path.suffix in ('.rs', '.wgsl', '.toml', '.py'):
        add(path, 'source/' + name)

rows = []
for name, path in sorted(files.items()):
    data = path.read_bytes()
    destination = OUT / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)
    rows.append(dict(path=name, bytes=len(data), sha256=hashlib.sha256(data).hexdigest()))
assert len(rows) <= 256 and sum(row['bytes'] for row in rows) < 8 * 1024 * 1024
index = dict(schema=1, encoding='identity', files=rows)
data = (json.dumps(index, indent=2) + '\n').encode()
(OUT / 'index.json').write_bytes(data)

statistics = host['statistics']
percentages = {}
for population in ['first_frame', 'steady_frames']:
    percentages[population] = {}
    for field in ['median_ns', 'p95_nearest_rank_ns']:
        baseline = statistics['baseline']['prepare_to_present_ns'][population][field]
        candidate = statistics['candidate']['prepare_to_present_ns'][population][field]
        percentages[population][field] = round(100 * (candidate / baseline - 1), 4)
primary = {
    'schema': 1,
    'base_commit': 'f26a63f4e5eb3e4fb1722f76c25fa814c4fc804a',
    'baseline_runtime_commit': '51ca9edcc35e3705404607e60cb88afefcd85efd',
    'scope': 'Exact-size complete retired native buffer reuse; changed offscreen frames and one controlled native browser release comparison before compositor',
    'implementation': {
        'native_only': True, 'idle_leases_maximum': 1,
        'identity': 'Private Arc context and exact checked dimensions, format, lengths, alignment and actual buffer descriptors',
        'overwrite': 'All input and uniform bytes rewritten; full clear and all ordered draws and conversion encoded every frame',
        'success_boundary': 'Specific submission retired, all scopes successful, requested verification and presentation complete, final locked current/deadline checks',
        'unchanged_caps': {'native_plan_bytes': 16777216, 'active_plus_readback_bytes': 25165824,
                           'application_ledger_bytes': 134217728, 'next_ui_reservation_bytes': 16777216},
        'excluded': 'Driver, allocator, queue staging and binding overhead remain outside explicit-buffer ledger; this is not RSS/VRAM measurement',
        'probe_and_one_shot_api_preserved': True,
    },
    'sources': {
        'overlays': 'native-buffer-reuse/source; apply to base_commit for changed source files',
        'host_bound_files': len(host['bindings']['source_files']),
        'candidate_compiled_files': len(host['bindings']['variants']['candidate']['release']['source_files']),
        'baseline_compiled_files': len(host['bindings']['variants']['baseline']['release']['source_files']),
        'probe_compiled_scope_files': len(json.loads((RUN / 'probe-build-1.json').read_text())['source_files']),
        'baseline_snapshot': 'Every manifest blob was recovered from baseline_runtime_commit and verified before execution; public manifest retained, executable excluded',
    },
    'tests': {
        'root_native_each_toolchain': 1397, 'toolchains': ['1.88.0', '1.98.0'],
        'new_presenter_groups': 10, 'core_gpu_each_toolchain': 107,
        'core_default_minimum_toolchain': 84, 'new_core_groups': 12,
        'all_python': 450, 'root_python': 357, 'raster_probe_python': 93,
        'new_native_ab_host_groups': 10, 'new_reuse_protocol_groups': 3,
        'strict_clippy': 'Root default, vulkan-presenter and vulkan-raster configurations on both toolchains; core GPU and probe with browser bridge on both toolchains',
        'formatting': 'Root, core and raster probe pass Rust1.88 fmt checks; core log retained, root/probe checks observed in an empty-output successful shell execution without separately retained logs',
    },
    'offscreen': {
        'attempts': 1, 'adapters': probe['runs'][0]['validation']['adapters'],
        'frames': 84, 'successful_frame_allocations': 39, 'successful_frame_reuses': 45,
        'successful_frame_evictions': 39, 'cancellation_guard_cases': 18,
        'disposable_cancellation_allocations': 18, 'total_lease_allocations': 57,
        'compared_active_bytes': 4680, 'formats': ['Bgra8Unorm', 'Rgba8Unorm'],
        'reference': 'Independent literal geometry/color pixels; no reference enters encoders',
        'checks': 'Equal-size changed images, alpha, masks, row origins, uniforms, clear colors; input absence, hidden work, sizes, format and context changes; six guarded cancellations per adapter',
        'cleanup': 'Four successful processes, four reaps, zero surviving descendants, empty checker/supervisor stderr',
        'acquired_surface': False,
    },
    'comparison': {
        'attempts': 1, 'successful_processes': 8, 'check_processes': 2,
        'checked_bytes_each': 4505600, 'total_checked_bytes': 9011200,
        'timed_processes': 6, 'frames_each': 16, 'total_timed_frames': 96,
        'first_frames_each_variant': 3, 'subsequent_frames_each_variant': 45,
        'measured_reference_or_readback': False, 'size': [1280, 880],
        'scene_bytes': len(bytes.fromhex(host['runs'][0]['report']['scene_hex'])),
        'scene_sha256': host['scene_sha256'], 'url': host['url'],
        'scale_factor': host['runs'][0]['report']['scale_factor'],
        'adapter': host['runs'][0]['validation']['diagnostics']['adapter']['line'],
        'releases': {name: {k: v for k, v in item['release'].items() if k != 'source_files'}
                     for name, item in host['bindings']['variants'].items()},
        'cleanup': 'All eight browser children and wrappers exited0 and were reaped; each supervisor reports complete cleanup, reaped1 and zero descendants',
    },
    'statistics': statistics,
    'phase_statistics': host['phase_statistics'],
    'initialization_statistics': host['initialization_statistics'],
    'environment': {label: json.loads((RUN / f'environment-{label}-1.json').read_text())
                    for label in ['before', 'after']},
    'candidate_percent_change_in_direct_prepare_to_present': percentages,
    'retained_preexecution_corrections': [
        'Initial checker compile rejected Result<(),String>::as_deref; error log retained and corrected before actual execution',
        'Source review corrected one handwritten alpha-blend reference from0x0819aa to0x091aab using integer round-to-nearest; original fixture draft retained',
        'A/B source binding canonical path sorting corrected before actual execution; initial host source and both focused runs retained',
    ],
    'limits': [
        'One tiny static fixture on one NVIDIA window surface/format; three fresh processes per variant',
        'Sample1 kept separately; all samples2..16 retained, including elevated early submit intervals; no failed actual attempt or outlier removed',
        'Host intervals through prepare/present/retirement, not GPU timestamps, display latency, throughput or Chromium-relative results',
        'GPU clocks and power not locked; before snapshot P5/360MHz, after snapshot P0/2505MHz; these do not measure frequencies during frames or isolate the cause of every observed difference',
        'Cache-return bookkeeping is after the direct prepare-to-present endpoint and inside owner_total; final retained-cache destruction occurs during owner release outside frame timing',
        'Fresh native plans and coverage rebuilt each frame; queue write staging still allocates; no retained plan/pixel/reference cache',
        'Complete web compatibility, production security and Chromium performance target remain unfulfilled',
    ],
    'package': dict(index='native-buffer-reuse/index.json', sha256=hashlib.sha256(data).hexdigest(),
                    files=len(rows), payload_bytes=sum(row['bytes'] for row in rows)),
}
(ROOT / 'docs/evidence/vulkan-native-buffer-reuse.json').write_text(json.dumps(primary, indent=2) + '\n')
(ROOT / 'docs/evidence/vulkan-native-timing-host-ci.json').write_bytes((RUN / 'parent-ci.json').read_bytes())
print(json.dumps(primary['package']))
