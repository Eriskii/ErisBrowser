#!/usr/bin/env python3
"""Build and launch Eris; expose installed dynamic desktop libraries on NixOS."""
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent


def launch_environment():
    env = os.environ.copy()
    store = Path('/nix/store')
    if store.is_dir():
        libraries = []
        # Only known library package directories; never include arbitrary application libs.
        for package in ('wayland', 'libxkbcommon', 'libx11', 'libX11', 'libxcb',
                        'libxcursor', 'libXcursor', 'libxrandr', 'libXrandr', 'libxi', 'libXi'):
            candidates = [p for p in store.glob(f'*-{package}-[0-9]*') if (p / 'lib').is_dir()]
            if candidates:
                def version(path):
                    tail = path.name.split(f'-{package}-', 1)[-1]
                    return tuple(int(n) for n in re.findall(r'\d+', tail))
                libraries.append(str(max(candidates, key=version) / 'lib'))
        if libraries:
            env['LD_LIBRARY_PATH'] = ':'.join(libraries + ([env['LD_LIBRARY_PATH']] if env.get('LD_LIBRARY_PATH') else []))
    return env


if __name__ == '__main__':
    subprocess.run(['cargo', 'build', '--locked', '--release'], cwd=ROOT, check=True)
    os.execve(ROOT / 'target/release/eris-browser', ['eris-browser', *sys.argv[1:]], launch_environment())
