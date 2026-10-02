"""Synthetic timing protocol/ownership tests: no network, browser or GPU."""
from contextlib import ExitStack
import copy
import io
import json
import os
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import native_raster_timing_host as timing
import native_raster_host as base
import native_raster_wide_host as wide
import bridge_supervisor as supervisor
import run_browser_host
import test_native_raster_wide_host as wide_tests

URL = 'http://127.0.0.1:12345/admitted.html'


def u32(value):
    return struct.pack('<I', value)


def f32(value):
    return struct.pack('<f', value)


def blob(value):
    value = value.encode() if isinstance(value, str) else value
    return u32(len(value)) + value


def scene(url=URL, title='N', status='Native timing scene', text='A'):
    rect = f32(0) + f32(0) + f32(8) + f32(4)
    def run(value):
        return b'\x07' + f32(0) + f32(0) + blob(value) + f32(12) + bytes([0, 0, 0, 255]) + b'\0\0\1'
    key = url.replace('admitted.html', 'two-pixels.png')
    commands = (b'\x06' + rect + bytes([255] * 4) + f32(0) + run(text)
                + b'\x08' + rect + blob(key) + b'\x02' + run('F') + b'\x03')
    return (b'EBNI\x01' + u32(1280) + u32(880) + f32(0) + f32(1) + blob(title) + blob(url)
            + b'\0' + blob('') + u32(0) + u32(0) + b'\0' + f32(1000) + blob(status)
            + u32(6) + commands + u32(1) + blob(key) + u32(0) + u32(2) + u32(1)
            + blob(bytes([255, 0, 0, 255, 0, 255, 0, 255])))


def report(route='cpu', mode='measure', frames=3, url=URL):
    result = dict(schema=1, kind='native-benchmark', mode=mode, route=timing.ROUTES[route],
                  requested_frames=frames if mode == 'measure' else 1,
                  completed_frames=frames if mode == 'measure' else 1,
                  accepted_frames=frames if mode == 'measure' else 0,
                  scene_hex=scene(url).hex(), width=1280, height=880, generation=1,
                  viewport_revision=2, scale_factor=1.0, source_load_ms=17.25,
                  edit_sequence=0, diagnostics=0, checked=mode == 'check', failure=None,
                  owner_failure=None, owner_init_ns=123 if mode == 'measure' else None, samples=[])
    if mode == 'measure':
        for index in range(1, frames + 1):
            sample = dict(id=index, generation=1, viewport_revision=2, serial=index + 3,
                          width=1280, height=880, route=timing.ROUTES[route], outcome='presented',
                          submission='draws', configured=index == 1)
            sample.update({key: index * 100 for key in timing.NS})
            if route == 'cpu':
                sample['cleanup_ns'] = None
            result['samples'].append(sample)
    return result


def dump(value):
    return (json.dumps(value, separators=(',', ':')) + '\n').encode()


def release_fixture(directory, name):
    root = directory / f'{name}-source'
    for relative in ('Cargo.toml', 'Cargo.lock', 'crates/raster-core/Cargo.toml',
                     'crates/raster-core/Cargo.lock', 'crates/raster-core/src/lib.rs',
                     'src/main.rs', 'assets/font.ttf'):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(f'{name}: {relative}'.encode())
    binary = directory / f'{name}-eris'
    binary.write_bytes(f'{name} frozen release'.encode())
    record = dict(binary=str(binary), bytes=binary.stat().st_size, sha256=wide.sha(binary.read_bytes()),
                  profile='release', toolchain='1.98.0', features=['vulkan-raster'],
                  source_files=timing.compiled_sources(root))
    manifest = directory / f'{name}-release.json'
    manifest.write_bytes(dump(record))
    return dict(source=root, binary=binary, manifest=manifest, pin=wide.sha(manifest.read_bytes()), record=record)


def bodies():
    return timing.fixture_data(base.FIXTURES)


def population(value, pid=71):
    raster = 'CPU upload' if value['route'] == 'cpu-upload' else 'native-shaders with complete CPU admission fallback'
    adapter = f'presenter: Vulkan adapter="Fake GPU" type=DiscreteGpu driver="test" format=Bgra8Unorm color_space=Srgb alpha=Opaque mode=Fifo; raster={raster}\n'
    prefix = (adapter + 'native benchmark waiting: size=1280x880 loaded=false\n').encode()
    event = dict(prefix_bytes=len(prefix), prefix_sha256=wide.sha(prefix),
                 html_sha256=wide.sha(bodies()['/admitted.html']), physical_size=[1280, 880])
    marker = (f'native timing gate: document released for owned pid={pid} size=1280x880 '
              f'prefix_bytes={len(prefix)} prefix_sha256={wide.sha(prefix)} html_sha256={event["html_sha256"]}\n').encode()
    suffix = b''
    if value['mode'] == 'check':
        if value['route'] == 'native-raster':
            suffix += b'presenter: native shader frame presented; size=1280x880 serial=4 CPU_upload=false reference=true\n'
            prefix_verified = 'presenter: native shader acquired-texture verified;'
        else:
            prefix_verified = 'presenter: Vulkan verified'
        suffix += (f'{prefix_verified} serial=4 generation=1 viewport_revision=2 size=1280x880 compared_bytes=4505600 exact=true (before compositor)\n'
                   f'presenter: Vulkan acquired-texture verification passed; route={value["route"]} frames=1 compared_bytes=4505600 last_serial=4 (before compositor)\n').encode()
    control = dict(success=True, control_complete=True, controlled_size=[1280, 880], browser_returncode=0,
                   browser_reaped=True, identity={'pid': pid}, release=event, port=12345,
                   accepted_connections=2, requests=[])
    for path, body in bodies().items():
        response = wide.response_for(path, bodies())
        header = f'GET {path} HTTP/1.1\r\nHost: 127.0.0.1:12345\r\n\r\n'.encode()
        control['requests'].append(dict(path=path, method='GET', header_bytes=len(header), header_sha256=wide.sha(header),
                                        body_bytes=len(body), body_sha256=wide.sha(body), response_complete=True,
                                        response_bytes=len(response), response_sha256=wide.sha(response)))
    return prefix + marker + suffix, control


class ProtocolTests(unittest.TestCase):
    def test_both_routes_check_and_measure_exact_schema(self):
        for route in timing.ROUTES:
            for mode in ('check', 'measure'):
                value = report(route, mode)
                accepted = timing.validate_report(dump(value), route, mode, 3, URL)
                self.assertEqual(accepted['identity']['commands'], 6)
                stderr, control = population(value)
                verified = timing.validate_stderr(stderr, control, value, 12345, bodies())
                self.assertEqual(verified['verified_frames'], int(mode == 'check'))

    def test_report_duplicates_constants_trailing_and_unknown_fields_rejected(self):
        raw = dump(report())
        for bad in (raw.replace(b'"schema":1', b'"schema":1,"schema":1'), raw + b'{}\n',
                    raw.replace(b'17.25', b'NaN'), raw[:-1], raw.replace(b'"schema":1', b'"extra":0,"schema":1')):
            with self.subTest(bad=bad[:40]), self.assertRaises(ValueError):
                timing.validate_report(bad, 'cpu', 'measure', 3, URL)

    def test_every_report_count_type_failure_and_route_is_strict(self):
        changes = dict(schema=True, mode='check', route='native-raster', requested_frames=2,
                       completed_frames=2, accepted_frames=2, width=1279, height=879,
                       generation=0, viewport_revision=False, scale_factor=True, source_load_ms=-1,
                       failure='late shutdown', owner_failure='scope failed', checked=True, owner_init_ns=None)
        for key, value in changes.items():
            changed = report(); changed[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                timing.validate_report(dump(changed), 'cpu', 'measure', 3, URL)

    def test_sample_ordinal_serial_phase_identity_and_outcome_rejections(self):
        changes = dict(id=2, generation=2, viewport_revision=3, serial=0, width=1279,
                       height=879, route='native-raster', outcome='aborted', submission='flush', configured=1)
        changes.update({key: None for key in timing.NS if key != 'cleanup_ns'})
        changes['cleanup_ns'] = 0
        for key, value in changes.items():
            changed = report(); changed['samples'][0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                timing.validate_report(dump(changed), 'cpu', 'measure', 3, URL)
        for mutate in (lambda v: v['samples'].pop(), lambda v: v['samples'][1].update(serial=4),
                       lambda v: v['samples'][0].update(ui_prepare_ns=-1),
                       lambda v: v['samples'][0].update(owner_total_ns=1 << 64)):
            changed = report(); mutate(changed)
            with self.assertRaises(ValueError):
                timing.validate_report(dump(changed), 'cpu', 'measure', 3, URL)

    def test_check_cannot_hide_failure_or_contain_timings(self):
        for key, value in dict(checked=False, failure='shutdown', owner_failure='owner', accepted_frames=1,
                               owner_init_ns=0, samples=[report()['samples'][0]]).items():
            changed = report(mode='check'); changed[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                timing.validate_report(dump(changed), 'cpu', 'check', 3, URL)

    def test_identity_complete_structure_not_only_hash_or_counts(self):
        raw = scene()
        for data in (raw[:-1], raw + b'\0', b'EBNI\x02' + raw[5:],
                     raw[:5] + u32(1279) + raw[9:],
                     raw.replace(blob('Native timing scene'), blob('other status')),
                     raw.replace(blob(URL), blob('http://127.0.0.1:12346/admitted.html')),
                     raw.replace(blob('N'), blob('X'), 1)):
            with self.subTest(size=len(data)), self.assertRaises(ValueError):
                timing.parse_identity(data.hex(), URL)
        for text in (raw.hex().upper(), raw.hex() + 'a', '00' * (timing.MAX_SCENE + 1)):
            with self.assertRaises(ValueError):
                timing.parse_identity(text, URL)
        # Equal inventory counts with different text still retain distinct bytes.
        self.assertNotEqual(timing.parse_identity(scene(text='B').hex(), URL)['sha256'],
                            timing.parse_identity(raw.hex(), URL)['sha256'])

    def test_identity_float_scope_alias_and_payload_checks(self):
        raw = scene()
        key = URL.replace('admitted.html', 'two-pixels.png')
        marker = u32(1) + blob(key) + u32(0) + u32(2) + u32(1)
        changes = [raw[:13] + f32(float('nan')) + raw[17:],
                   raw.replace(b'\x03' + marker, b'\x01' + marker),
                   raw.replace(marker, u32(1) + blob(key) + u32(1) + u32(2) + u32(1)),
                   raw.replace(marker, u32(1) + blob(key) + u32(0) + u32(3) + u32(1))]
        for changed in changes:
            with self.assertRaises(ValueError):
                timing.parse_identity(changed.hex(), URL)

    def test_gate_before_check_exact_prefix_and_latest_physical_size(self):
        value = report(mode='check'); data, control = population(value)
        lines = data.splitlines(keepends=True)
        for bad in (b''.join(lines[:2] + [lines[3], lines[2]] + lines[4:]),
                    data.replace(b'waiting: size=1280x880', b'waiting: size=1279x880'),
                    data.replace(b'prefix_sha256=', b'prefix_sha256=0')):
            with self.assertRaises(ValueError):
                timing.validate_stderr(bad, control, value, 12345, bodies())
        wrong = b'native benchmark waiting: size=1279x880 loaded=false\n'
        prefix = b''.join(lines[:2]) + wrong
        event = control['release']
        marker = lines[2].replace(str(event['prefix_bytes']).encode(), str(len(prefix)).encode(), 1)
        marker = marker.replace(event['prefix_sha256'].encode(), wide.sha(prefix).encode())
        event.update(prefix_bytes=len(prefix), prefix_sha256=wide.sha(prefix))
        with self.assertRaises(ValueError):
            timing.validate_stderr(prefix + marker + b''.join(lines[3:]), control, value, 12345, bodies())

    def test_measure_verification_and_check_route_quota_mutations(self):
        value = report('gpu', 'check'); data, control = population(value)
        for before, after in [(b'compared_bytes=4505600', b'compared_bytes=4505599'), (b'frames=1', b'frames=2'),
                              (b'last_serial=4', b'last_serial=5'), (b'reference=true', b'reference=false'),
                              (b'generation=1', b'generation=2'), (b'route=native-raster', b'route=cpu-upload')]:
            with self.subTest(before=before), self.assertRaises(ValueError):
                timing.validate_stderr(data.replace(before, after), control, value, 12345, bodies())
        with self.assertRaises(ValueError):
            timing.validate_stderr(data, control, report('gpu'), 12345, bodies())
        for bad in (data + data.splitlines(keepends=True)[-1], data + b'presenter: warning unparsed\n'):
            with self.assertRaises(ValueError):
                timing.validate_stderr(bad, control, value, 12345, bodies())

    def test_adapter_initialization_cannot_move_after_terminal_check_summary(self):
        value = report(mode='check'); data, control = population(value)
        lines = data.splitlines(keepends=True)
        adapter = lines.pop(0)
        prefix = lines[0]
        event = control['release']
        lines[1] = lines[1].replace(str(event['prefix_bytes']).encode(), str(len(prefix)).encode(), 1)
        lines[1] = lines[1].replace(event['prefix_sha256'].encode(), wide.sha(prefix).encode())
        event.update(prefix_bytes=len(prefix), prefix_sha256=wide.sha(prefix))
        with self.assertRaises(ValueError):
            timing.validate_stderr(b''.join(lines) + adapter, control, value, 12345, bodies())

    def test_owned_http_cleanup_and_payload_proof_fail_closed(self):
        value = report(); data, original = population(value)
        mutations = [lambda x: x.update(success=False), lambda x: x.update(browser_reaped=False),
                     lambda x: x.update(port=12346), lambda x: x.update(stderr_drain_error='timeout'),
                     lambda x: x.update(controlled_size=[1180, 880]),
                     lambda x: x['requests'][0].update(response_complete=False),
                     lambda x: x['requests'][1].update(body_sha256='0' * 64),
                     lambda x: x['requests'].reverse()]
        for mutate in mutations:
            control = copy.deepcopy(original); mutate(control)
            with self.assertRaises(ValueError):
                timing.validate_stderr(data, control, value, 12345, bodies())

    def test_direct_first_and_steady_quantiles_not_phase_sums(self):
        a, b = report(frames=3), report('gpu', frames=3)
        for run in (a, b):
            for sample, direct in zip(run['samples'], (1000, 10, 20)):
                sample['prepare_to_present_ns'] = direct
                sample['owner_total_ns'] = direct + 3
                sample['ui_prepare_ns'] = 100000
        got = timing.statistics_for([a, b])
        self.assertEqual(got['cpu-upload']['prepare_to_present_ns']['first_frame']['median_ns'], 1000)
        self.assertEqual(got['cpu-upload']['prepare_to_present_ns']['steady_frames'],
                         {'count': 2, 'median_ns': 15.0, 'p95_nearest_rank_ns': 20})
        self.assertEqual(got['native-raster']['owner_total_ns']['steady_frames']['median_ns'], 18.0)

    def test_sequence_is_separate_checks_then_counterbalanced_processes(self):
        self.assertEqual([row[0] for row in timing.sequence(3)],
                         ['check-cpu', 'check-gpu', 'measure-1-cpu', 'measure-1-gpu',
                          'measure-2-gpu', 'measure-2-cpu', 'measure-3-cpu', 'measure-3-gpu'])
        for n in (0, 9, True):
            with self.assertRaises(ValueError):
                timing.sequence(n)


class FakeListener:
    def __init__(self, *args, **kwargs):
        self.closed = False
    def bind(self, address):
        if address != ('127.0.0.1', 0):
            raise AssertionError(address)
    def listen(self, count):
        if count != wide.MAX_ACTIVE_CONNECTIONS:
            raise AssertionError(count)
    def setblocking(self, _):
        pass
    def getsockname(self):
        return ('127.0.0.1', 12345)
    def fileno(self):
        return 70
    def accept(self):
        raise BlockingIOError
    def close(self):
        self.closed = True


class HostTests(unittest.TestCase):
    def execute(self, directory, mutation=None, *, comparison='cpu-native', ab_mutation=None):
        binary, controller = directory / 'eris', directory / 'hyprctl'
        binary.write_bytes(b'frozen release'); controller.write_bytes(b'controller')
        variants, options = {}, {}
        if comparison == 'native-ab':
            variants = {name: release_fixture(directory, name) for name in ('baseline', 'candidate')}
            binary = variants['candidate']['binary']
            options = dict(comparison=comparison, candidate_manifest=variants['candidate']['manifest'],
                           candidate_manifest_sha256=variants['candidate']['pin'],
                           baseline_binary=variants['baseline']['binary'], baseline_manifest=variants['baseline']['manifest'],
                           baseline_manifest_sha256=variants['baseline']['pin'], baseline_source=variants['baseline']['source'])
        output = directory / 'output'
        calls, listener = [], FakeListener()
        def run(command, destination, name, env, timeout, **kwargs):
            calls.append((command, kwargs))
            self.assertEqual(kwargs, {'pass_fds': (70,)})
            get = lambda name: command[command.index(name) + 1]
            route, mode = get('--route'), get('--mode')
            value = report(route, mode, int(get('--frames')))
            data, control = population(value)
            control.update(binary_sha256=wide.sha(Path(get('--binary')).read_bytes()), controller_sha256=wide.sha(controller.read_bytes()),
                           fixture_freeze=base.FIXTURE_SHA256, route=route, mode=mode)
            state = {'report': value, 'stderr': data, 'control': control, 'status': 'exited', 'returncode': 0,
                     'cleanup': {'complete': True, 'descendants': 0}, 'supervisor_stderr': b''}
            if mutation:
                mutation(state, len(calls), binary)
            if ab_mutation:
                ab_mutation(state, len(calls), variants)
            stdout = dump(state['report'])
            (destination / f'{name}.stderr.log').write_bytes(state['stderr'])
            (destination / f'{name}.stdout.log').write_bytes(stdout)
            (destination / f'{name}.supervisor.stderr.log').write_bytes(state['supervisor_stderr'])
            Path(get('--receipt')).write_bytes(dump(state['control']))
            record = {key: state[key] for key in ('status', 'returncode', 'cleanup')}
            record.update(stdout_sha256=wide.sha(stdout), stderr_sha256=wide.sha(state['stderr']),
                          capture_gate_required=False, capture_gate_granted=False)
            return record, stdout
        with ExitStack() as stack:
            stack.enter_context(patch.object(timing.socket, 'socket', return_value=listener))
            stack.enter_context(patch.object(base, 'environment', return_value={}))
            stack.enter_context(patch.object(base, 'loader_binding', return_value={'synthetic': True}))
            stack.enter_context(patch.object(timing, 'source_binding', return_value=[{'synthetic': 'source'}]))
            stack.enter_context(patch.object(timing.shutil, 'which', return_value=str(controller)))
            stack.enter_context(patch.object(base, 'run_supervised', side_effect=run))
            if variants:
                stack.enter_context(patch.object(timing, 'ROOT', variants['candidate']['source']))
            result = timing.run_host(binary, output, directory, 30, 20, 16 if variants else 3,
                                     3 if variants else 2, wide.sha(binary.read_bytes()), **options)
        return result, calls, listener, output

    def test_all_processes_share_one_listener_url_binary_and_exact_scene(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, listener, output = self.execute(Path(tmp))
            self.assertTrue(result['success'], result.get('error'))
            self.assertEqual(len(calls), 6)
            self.assertTrue(listener.closed)
            self.assertEqual((output / 'scene.ebni').read_bytes(), scene())
            self.assertEqual(result['statistics']['cpu-upload']['owner_total_ns']['steady_frames']['count'], 4)
            for command, _ in calls:
                self.assertEqual(command[command.index('--port') + 1], '12345')
                self.assertEqual(command[command.index('--listener-fd') + 1], '70')

    def test_scene_mismatch_after_first_check_stops_and_retains_both_raw_reports(self):
        def mutate(state, count, _):
            if count == 2:
                state['report']['scene_hex'] = scene(text='B').hex()
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, _, output = self.execute(Path(tmp), mutate)
            self.assertFalse(result['success']); self.assertEqual(len(calls), 2)
            self.assertIn('scene bytes', result['error'])
            self.assertTrue((output / 'check-cpu.stdout.log').exists())
            self.assertTrue((output / 'check-gpu.stdout.log').exists())
            self.assertEqual(len(result['runs']), 2)
            self.assertNotIn('statistics', result)

    def test_failed_cleanup_nonzero_exit_and_late_json_failure_stop_first(self):
        for mutate in (lambda s, n, b: s['cleanup'].update(complete=False),
                       lambda s, n, b: s.update(returncode=1),
                       lambda s, n, b: s['report'].update(failure='shutdown'),
                       lambda s, n, b: s.update(supervisor_stderr=b'failed'),
                       lambda s, n, b: s['control'].update(browser_reaped=False)):
            with tempfile.TemporaryDirectory() as tmp:
                result, calls, _, _ = self.execute(Path(tmp), mutate)
                self.assertFalse(result['success']); self.assertEqual(len(calls), 1)

    def test_binary_mutation_and_changed_adapter_reject_before_next_case(self):
        for mutate in (lambda s, n, b: b.write_bytes(b'changed release'),
                       lambda s, n, b: s['report'].update(scale_factor=2) if n == 2 else None,
                       lambda s, n, b: s.update(stderr=s['stderr'].replace(b'Fake GPU', b'New GPU')) if n == 2 else None):
            with tempfile.TemporaryDirectory() as tmp:
                result, calls, _, _ = self.execute(Path(tmp), mutate)
                self.assertFalse(result['success']); self.assertLessEqual(len(calls), 2)

    def test_existing_output_and_invalid_limits_refused_without_listener(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(timing.socket, 'socket') as sock:
            path = Path(tmp)
            with self.assertRaises(ValueError):
                timing.run_host(path / 'missing', path, path, 30, 20, 16, 3, '0' * 64)
            with self.assertRaises(ValueError):
                timing.run_host(path / 'missing', path / 'new', path, 10, 20, 16, 3, '0' * 64)
            sock.assert_not_called()


class CompiledBindingTests(unittest.TestCase):
    def bind(self, item):
        return timing.compiled_binding(item['binary'], item['manifest'], item['pin'], item['source'])

    def test_two_independent_release_trees_and_copied_binary_are_bound(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            baseline, candidate = [release_fixture(directory, name) for name in ('baseline', 'candidate')]
            self.assertNotEqual(self.bind(baseline)['release']['source_files'], self.bind(candidate)['release']['source_files'])
            copied = directory / 'copied-release'
            copied.write_bytes(baseline['binary'].read_bytes())
            baseline['binary'] = copied
            self.assertEqual(self.bind(baseline)['binary'], str(copied))
            baseline['source'] = candidate['source']
            with self.assertRaisesRegex(ValueError, 'source inventory'):
                self.bind(baseline)

    def test_release_profile_features_binary_size_hash_and_schema_rejected(self):
        mutations = [lambda r: r.update(profile='dev'), lambda r: r.update(features=[]),
                     lambda r: r.update(features=['vulkan-raster', 'extra']), lambda r: r.update(toolchain=1.98),
                     lambda r: r.update(bytes=True), lambda r: r.update(bytes=r['bytes'] + 1),
                     lambda r: r.update(sha256='0' * 64), lambda r: r.update(extra='unbound'),
                     lambda r: r.update(source_files=[]), lambda r: r.update(source_files=r['source_files'] * 150)]
        with tempfile.TemporaryDirectory() as tmp:
            item = release_fixture(Path(tmp), 'baseline')
            for mutate in mutations:
                record = copy.deepcopy(item['record']); mutate(record)
                item['manifest'].write_bytes(dump(record)); item['pin'] = wide.sha(item['manifest'].read_bytes())
                with self.subTest(record=record.keys()), self.assertRaises(ValueError):
                    self.bind(item)

    def test_source_paths_duplicates_counts_and_bytes_cannot_be_excused(self):
        mutations = [lambda r: r['source_files'].pop(),
                     lambda r: r['source_files'].append(r['source_files'][0]),
                     lambda r: r['source_files'][0].update(bytes=0),
                     lambda r: r['source_files'][0].update(sha256='0' * 64)]
        mutations += [lambda r, path=path: r['source_files'][0].update(path=path)
                      for path in ('../Cargo.toml', '/Cargo.toml', './Cargo.toml', 'src//main.rs',
                                   'src\\main.rs', 'src/../Cargo.toml', '', 'src/\0')]
        with tempfile.TemporaryDirectory() as tmp:
            item = release_fixture(Path(tmp), 'baseline')
            for mutate in mutations:
                record = copy.deepcopy(item['record']); mutate(record)
                item['manifest'].write_bytes(dump(record)); item['pin'] = wide.sha(item['manifest'].read_bytes())
                with self.assertRaises(ValueError):
                    self.bind(item)

    def test_added_changed_missing_sources_and_symlinks_fail(self):
        for mode in ('added', 'changed', 'missing', 'file-symlink', 'directory-symlink', 'root-symlink', 'fifo'):
            with tempfile.TemporaryDirectory() as tmp:
                directory = Path(tmp); item = release_fixture(directory, 'baseline')
                source = item['source'] / 'src/main.rs'
                if mode == 'added':
                    (source.parent / 'extra.rs').write_bytes(b'new')
                elif mode == 'changed':
                    source.write_bytes(b'new')
                elif mode == 'missing':
                    source.unlink()
                elif mode == 'file-symlink':
                    source.unlink(); source.symlink_to(item['source'] / 'Cargo.toml')
                elif mode == 'directory-symlink':
                    assets = item['source'] / 'assets'
                    assets.rename(directory / 'assets-real'); assets.symlink_to(directory / 'assets-real')
                elif mode == 'root-symlink':
                    link = directory / 'source-link'; link.symlink_to(item['source']); item['source'] = link
                else:
                    source.unlink(); os.mkfifo(source)
                with self.subTest(mode=mode), self.assertRaises(ValueError):
                    self.bind(item)

    def test_manifest_pin_duplicate_json_and_bound_are_checked_before_contents(self):
        with tempfile.TemporaryDirectory() as tmp:
            item = release_fixture(Path(tmp), 'baseline')
            item['pin'] = '0' * 64
            with self.assertRaisesRegex(ValueError, 'pin mismatch'):
                self.bind(item)
            for raw in (b'{"profile":"release","profile":"release"}', b' ' * (timing.MAX_REPORT + 1)):
                item['manifest'].write_bytes(raw); item['pin'] = wide.sha(raw)
                with self.assertRaises(ValueError):
                    self.bind(item)


class NativeABTests(unittest.TestCase):
    def execute(self, directory, mutate=None):
        return HostTests.execute(self, directory, comparison='native-ab', ab_mutation=mutate)

    def test_exact_eight_native_cases_distinct_binaries_manifests_and_full_statistics(self):
        def mutate(state, _count, variants):
            baseline = state['control']['binary_sha256'] == variants['baseline']['record']['sha256']
            if state['report']['mode'] == 'measure':
                state['report']['owner_init_ns'] = 200 if baseline else 400
                state['report']['samples'][1].update({name: 1_000_000 if baseline else 2_000_000 for name in timing.NS})
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, listener, output = self.execute(Path(tmp), mutate)
            self.assertTrue(result['success'], result.get('error')); self.assertTrue(listener.closed)
            self.assertEqual(len(calls), 8)
            expected = ['check-baseline', 'check-candidate', 'measure-1-baseline', 'measure-1-candidate',
                        'measure-2-candidate', 'measure-2-baseline', 'measure-3-baseline', 'measure-3-candidate']
            self.assertEqual([r['name'] for r in result['runs']], expected)
            self.assertEqual(sum(len(r['report']['samples']) for r in result['runs']), 96)
            self.assertEqual(sum(r['validation']['diagnostics']['compared_bytes'] for r in result['runs']), 9_011_200)
            for run, (command, _) in zip(result['runs'], calls):
                self.assertEqual(run['report']['route'], 'native-raster')
                self.assertEqual(command[command.index('--route') + 1], 'gpu')
                self.assertEqual(command[command.index('--port') + 1], '12345')
                self.assertEqual(command[command.index('--binary') + 1], result['bindings']['variants'][run['variant']]['binary'])
                self.assertNotIn('variant', run['report'])
            for variant, tail in (('baseline', 1_000_000), ('candidate', 2_000_000)):
                self.assertEqual(result['initialization_statistics'][variant]['count'], 3)
                self.assertEqual(set(result['phase_statistics'][variant]), set(timing.NS[:-2]))
                for metric in ('owner_total_ns', 'prepare_to_present_ns'):
                    summary = result['statistics'][variant][metric]
                    self.assertEqual(summary['first_frame']['count'], 3)
                    self.assertEqual(summary['steady_frames']['count'], 45)
                    self.assertEqual(summary['steady_frames']['p95_nearest_rank_ns'], tail)
                self.assertEqual(wide.sha((output / f'{variant}-release.json').read_bytes()),
                                 result['bindings']['variants'][variant]['manifest_sha256'])

    def test_each_variant_source_manifest_binary_and_manifest_copy_mutation_stops(self):
        for variant in ('baseline', 'candidate'):
            for field in ('source', 'manifest', 'binary', 'copied-manifest'):
                def mutate(_state, count, variants):
                    if count == 1:
                        item = variants[variant]
                        path = item[field] if field in ('manifest', 'binary') else (
                            item['source'] / 'src/main.rs' if field == 'source' else
                            item['manifest'].parent / 'output' / f'{variant}-release.json')
                        path.write_bytes(b'changed after first case')
                with tempfile.TemporaryDirectory() as tmp, self.subTest(variant=variant, field=field):
                    result, calls, listener, output = self.execute(Path(tmp), mutate)
                    self.assertFalse(result['success']); self.assertEqual(len(calls), 1); self.assertTrue(listener.closed)
                    self.assertTrue((output / 'check-baseline.stdout.log').exists())
                    self.assertNotIn('statistics', result)

    def test_wrong_selected_binary_cpu_route_and_changed_scene_scale_adapter_fail(self):
        mutations = [lambda s, v: s['control'].update(binary_sha256=v['candidate']['record']['sha256']),
                     lambda s, v: s['report'].update(route='cpu-upload'),
                     lambda s, v: s['report'].update(scene_hex=scene(text='changed').hex()),
                     lambda s, v: s['report'].update(scale_factor=2),
                     lambda s, v: s.update(stderr=s['stderr'].replace(b'Fake GPU', b'Different GPU'))]
        for index, change in enumerate(mutations):
            target = 1 if index < 2 else 2
            def mutate(state, count, variants):
                if count == target:
                    change(state, variants)
            with tempfile.TemporaryDirectory() as tmp, self.subTest(index=index):
                result, calls, _, _ = self.execute(Path(tmp), mutate)
                self.assertFalse(result['success']); self.assertEqual(len(calls), target)
                self.assertNotIn('statistics', result)

    def test_later_measure_failure_preserves_attempt_without_aggregate_or_retry(self):
        for kind in ('cleanup', 'phase', 'exit'):
            def mutate(state, count, _variants):
                if count == 5:
                    if kind == 'cleanup':
                        state['cleanup']['complete'] = False
                    elif kind == 'phase':
                        state['report']['samples'][1]['submit_ns'] = None
                    else:
                        state['returncode'] = 1
            with tempfile.TemporaryDirectory() as tmp, self.subTest(kind=kind):
                result, calls, _, output = self.execute(Path(tmp), mutate)
                self.assertFalse(result['success']); self.assertEqual(len(calls), 5)
                self.assertEqual(len(result['runs']), 5)
                self.assertEqual(sum(r['success'] for r in result['runs']), 4)
                self.assertTrue((output / 'measure-2-candidate.stdout.log').exists())
                self.assertNotIn('statistics', result)

    def test_comparison_population_and_missing_or_default_manifest_arguments_refused_early(self):
        for frames, repeats in ((15, 3), (16, 2), (16, 4), (128, 3)):
            with self.assertRaises(ValueError):
                timing.comparison_sequence('native-ab', frames, repeats)
        with tempfile.TemporaryDirectory() as tmp, patch.object(timing.socket, 'socket') as sock:
            root = Path(tmp)
            for options in ({'comparison': 'native-ab'}, {'candidate_manifest': root / 'manifest'}):
                with self.assertRaises(ValueError):
                    timing.run_host(root / 'binary', root / 'output', root, 30, 20,
                                    expected_binary_sha256='0' * 64, **options)
            sock.assert_not_called()


class GateAndWrapperTests(unittest.TestCase):
    def test_common_waiting_gate_preserves_prefix_and_exact_responses(self):
        stream, listener, pairs, record, emitted, context, startup = wide_tests.HttpReleaseTests.scenario(self)
        real_read = os.read
        waiting = b'native benchmark waiting: size=1280x880 loaded=false\n'
        def read(fd, size):
            data = real_read(fd, size)
            return waiting if data == startup else data
        with context, patch.object(wide.os, 'read', side_effect=read):
            wide.serve_and_relay(SimpleNamespace(pid=71, stderr=stream), listener, record, bodies(), lambda: None,
                                 wide.time.monotonic() + 2, emitted.append, waiting_pattern=timing.WAITING,
                                 marker_prefix='native timing gate', forbidden_patterns=(timing.CPU_VERIFIED, timing.SUMMARY))
        self.assertEqual(record['release']['prefix_sha256'], wide.sha(waiting))
        self.assertIn(b'native timing gate:', b''.join(emitted))
        for (_, client), path in zip(pairs, wide.ROUTES):
            self.assertEqual(client.recv(4096), wide.response_for(path, bodies()))

    def test_common_gate_wrong_size_early_cpu_verification_and_partial_line_never_release(self):
        for mode in ('wrong-size', 'early-cpu', 'partial-native'):
            setup = {'partial_native': True} if mode == 'partial-native' else {}
            stream, listener, _, record, emitted, context, startup = wide_tests.HttpReleaseTests.scenario(self, **setup)
            real_read = os.read
            waiting = b'native benchmark waiting: size=1280x880 loaded=false\n'
            if mode == 'wrong-size':
                waiting = waiting.replace(b'1280', b'1279')
            elif mode == 'early-cpu':
                waiting += (b'presenter: Vulkan verified serial=4 generation=1 viewport_revision=2 '
                            b'size=1280x880 compared_bytes=4505600 exact=true (before compositor)\n')
            else:
                waiting += b'presenter: native shader frame pres'
            def read(fd, size):
                data = real_read(fd, size)
                return waiting if data == startup else data
            with context, patch.object(wide.os, 'read', side_effect=read), self.assertRaises(ValueError):
                wide.serve_and_relay(SimpleNamespace(pid=71, stderr=stream), listener, record, bodies(), lambda: None,
                                     wide.time.monotonic() + 2, emitted.append, waiting_pattern=timing.WAITING,
                                     marker_prefix='native timing gate', forbidden_patterns=(timing.CPU_VERIFIED, timing.SUMMARY))
            self.assertNotIn('release', record)

    def test_wrapper_setup_failure_drains_raw_tail_and_kills_unreaped_child_before_wait(self):
        for mode in ('check', 'measure'):
            with tempfile.TemporaryDirectory() as tmp:
                directory = Path(tmp)
                binary, controller = directory / 'browser', directory / 'controller'
                binary.write_bytes(b'bin'); controller.write_bytes(b'ctl')
                read_fd, write_fd = os.pipe()
                os.write(write_fd, b'original timing startup diagnostic\n'); os.close(write_fd)
                stream = os.fdopen(read_fd, 'rb')
                self.addCleanup(stream.close)
                child = SimpleNamespace(pid=71, stderr=stream)
                events = []
                child.wait = lambda timeout: events.append('reap') or -9
                captured, listener = io.BytesIO(), FakeListener()
                args = SimpleNamespace(binary=binary, controller=controller, binary_sha256=wide.sha(b'bin'),
                                       controller_sha256=wide.sha(b'ctl'), window_seconds=20, receipt=directory / 'receipt.json',
                                       route='cpu', mode=mode, frames=16, port=12345, listener_fd=70, fixtures=base.FIXTURES)
                with patch.object(timing, 'validate_listener_fds'), \
                        patch.object(timing.socket, 'socket', return_value=listener), \
                        patch.object(timing.subprocess, 'Popen', return_value=child) as spawn, \
                        patch.object(base, 'child_identity', return_value={'pid': 71}), \
                        patch.object(base, 'configure_owned', side_effect=ValueError('synthetic controller failure')), \
                        patch.object(timing.os, 'waitid', side_effect=lambda *_: events.append('owned')), \
                        patch.object(timing.os, 'kill', side_effect=lambda *_: events.append('kill')), \
                        patch.object(timing.sys, 'stderr', SimpleNamespace(buffer=captured)):
                    self.assertEqual(timing.launch_owned(args), 1)
                self.assertEqual(events, ['owned', 'kill', 'reap'])
                self.assertEqual(captured.getvalue(), b'original timing startup diagnostic\n')
                self.assertTrue(listener.closed)
                self.assertTrue(spawn.call_args.kwargs['close_fds'])
                self.assertNotIn('pass_fds', spawn.call_args.kwargs)
                command = spawn.call_args.args[0]
                self.assertIn('--raster=cpu', command)
                self.assertEqual('--benchmark-native-check' in command, mode == 'check')
                self.assertEqual('--benchmark-native' in command, mode == 'measure')
                record = json.loads(args.receipt.read_bytes())
                self.assertFalse(record['success']); self.assertTrue(record['browser_reaped'])



class DescriptorTests(unittest.TestCase):
    def test_descriptor_validator_rejects_nonlistener_family_port_and_stdio(self):
        for descriptors in ((0,), (True,), (7, 8), [7]):
            with self.assertRaises(ValueError):
                supervisor.validate_listener_fds(descriptors)
        supervisor.validate_listener_fds(())
        for address, family, kind, listening in [(('0.0.0.0', 12), 2, 1, 1), (('127.0.0.1', 0), 2, 1, 1),
                                                (('127.0.0.1', 12), 1, 1, 1), (('127.0.0.1', 12), 2, 2, 1),
                                                (('127.0.0.1', 12), 2, 1, 0)]:
            sock = SimpleNamespace(family=family, getsockname=lambda: address,
                                   getsockopt=lambda level, key: {timing.socket.SO_TYPE: kind, timing.socket.SO_ACCEPTCONN: listening,
                                                                 timing.socket.SO_PROTOCOL: timing.socket.IPPROTO_TCP}[key],
                                   detach=lambda: None)
            with patch.object(supervisor.socket, 'socket', return_value=sock), self.assertRaises(ValueError):
                supervisor.validate_listener_fds((7,))

    def test_listener_protocol_check_accepts_tcp_and_rejects_other_stream_protocols(self):
        detached = []
        for protocol in (timing.socket.IPPROTO_TCP, 132):
            sock = SimpleNamespace(family=timing.socket.AF_INET, getsockname=lambda: ('127.0.0.1', 12345),
                                   getsockopt=lambda level, key: {
                                       timing.socket.SO_TYPE: timing.socket.SOCK_STREAM,
                                       timing.socket.SO_ACCEPTCONN: 1,
                                       timing.socket.SO_PROTOCOL: protocol}[key],
                                   detach=lambda: detached.append(True))
            with patch.object(supervisor.socket, 'socket', return_value=sock):
                if protocol == timing.socket.IPPROTO_TCP:
                    supervisor.validate_listener_fds((7,))
                else:
                    with self.assertRaises(ValueError):
                        supervisor.validate_listener_fds((7,))
        self.assertEqual(detached, [True, True])

    def test_run_supervised_explicit_fd_forwarding_and_default_isolation(self):
        for descriptors in ((), (71,)):
            with tempfile.TemporaryDirectory() as tmp:
                output = Path(tmp)
                def spawn(command, **kwargs):
                    self.assertEqual(kwargs['pass_fds'], descriptors)
                    self.assertTrue(kwargs['close_fds'])
                    self.assertEqual('--pass-listener-fd' in command, bool(descriptors))
                    if descriptors:
                        self.assertEqual(command[command.index('--pass-listener-fd') + 1], '71')
                    (output / 'case.stdout.log').write_bytes(b'')
                    (output / 'case.stderr.log').write_bytes(b'')
                    record = dict(status='exited', returncode=0, stdout_sha256=wide.sha(b''),
                                  stderr_sha256=wide.sha(b''), captured_bytes=0)
                    (output / 'case.json').write_bytes(dump(record))
                    return SimpleNamespace(returncode=0, wait=lambda timeout: 0)
                with patch.object(supervisor, 'validate_listener_fds') as validate, patch.object(run_browser_host.subprocess, 'Popen', side_effect=spawn):
                    record, raw = run_browser_host.run_supervised(['fake'], output, 'case', {}, 1, pass_fds=descriptors)
                    self.assertEqual(record['status'], 'exited'); self.assertEqual(raw, b'')
                    validate.assert_called_once_with(descriptors)

    def test_supervisor_forwards_only_requested_descriptor_to_direct_child(self):
        # Stop immediately at Popen: no subreaper mutation, real process, socket or signal.
        for descriptors in ((), (71,)):
            with patch.object(supervisor, 'validate_listener_fds') as validate, \
                    patch.object(supervisor, 'enable_subreaper'), patch.object(supervisor.signal, 'signal'), \
                    patch.object(supervisor.subprocess, 'Popen', side_effect=OSError('synthetic stop')) as spawn:
                record, _, _ = supervisor.supervise(['fake'], 1, 128, pass_fds=descriptors)
                self.assertEqual(spawn.call_args.kwargs['pass_fds'], descriptors)
                self.assertTrue(spawn.call_args.kwargs['close_fds'])
                validate.assert_called_once_with(descriptors)
                self.assertNotEqual(record['status'], 'exited')


if __name__ == '__main__':
    unittest.main()
