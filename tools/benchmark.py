#!/usr/bin/env python3
"""Record reproducible local phase timings; never imply cross-engine parity."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/eris-browser')
    parser.add_argument('--iterations', type=int, default=100)
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/benchmark.json')
    args = parser.parse_args()
    if not 1 <= args.iterations <= 10000:
        parser.error('iterations must be between 1 and 10000')
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    sources = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock']
    sources += list((ROOT / 'src').rglob('*.rs'))
    sources += [path for directory in ('assets', 'examples')
                for path in (ROOT / directory).rglob('*') if path.is_file()]
    for source in sorted(sources):
        digest.update(str(source.relative_to(ROOT)).encode())
        digest.update(b'\0')
        digest.update(source.read_bytes())
        digest.update(b'\0')
    cases = []
    for name, address in [('home', 'eris:home'), ('gallery', 'examples/gallery.html'), ('forms', 'examples/forms.html')]:
        command = [str(args.binary.resolve()), address, '--benchmark', str(args.iterations),
                   '--width', '1180', '--height', '880', '--output', str(output.parent / f'benchmark-{name}.png')]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
        if result.returncode != 0 or '[page]' in result.stderr:
            raise RuntimeError(f'{name}: {result.stdout}\n{result.stderr}')
        match = re.search(r'median ([\d.]+) ms, p95 ([\d.]+) ms', result.stdout)
        if not match:
            raise RuntimeError(f'Could not parse benchmark output: {result.stdout}')
        cases.append(dict(name=name, address=address, median_ms=float(match[1]), p95_ms=float(match[2]), log=result.stdout.strip()))
    cpu = None
    cpuinfo = Path('/proc/cpuinfo')
    if cpuinfo.exists():
        cpu = next((line.split(':', 1)[1].strip() for line in cpuinfo.read_text().splitlines() if line.startswith('model name')), None)
    report = dict(timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  system=platform.platform(), cpu=cpu, rustc=subprocess.check_output(['rustc', '--version'], text=True).strip(),
                  input_sha256=digest.hexdigest(), input_hash_includes=['Cargo.toml', 'Cargo.lock', 'src/**/*.rs', 'assets/**', 'examples/**'],
                  binary_sha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(),
                  viewport=[1180, 880], iterations=args.iterations,
                  measured='warm-cache style computation + layout + software paint',
                  excluded=['network', 'HTML parsing', 'script execution', 'image decoding', 'PNG encoding', 'native presentation'],
                  chromium_comparison=False, controlled_environment=False, cases=cases)
    output.write_text(json.dumps(report, indent=2) + '\n')
    for case in cases:
        print(f"{case['name']}: median {case['median_ms']:.3f} ms; p95 {case['p95_ms']:.3f} ms")
    print(f'Report: {output}')


if __name__ == '__main__':
    main()
