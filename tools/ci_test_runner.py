#!/usr/bin/env python3
"""Launch Cargo's CI test binaries without inherited build/runner descriptors.

The production worker still rejects unexpected open descriptors. This wrapper
only sanitizes the test process's initial environment; descriptors deliberately
opened by tests retain their normal inheritance behavior.
"""
import os
import subprocess
import sys


def main(argv):
    if not argv:
        print('usage: ci_test_runner.py PROGRAM [ARG ...]', file=sys.stderr)
        return 2
    env = os.environ.copy()
    for key in ('CARGO_MAKEFLAGS', 'MAKEFLAGS', 'MFLAGS'):
        env.pop(key, None)
    try:
        result = subprocess.run(argv, close_fds=True, env=env)
    except OSError as error:
        print(f'ci_test_runner: cannot start test program: {error}', file=sys.stderr)
        return 127
    # Popen represents signal termination as a negative signal number; expose
    # the conventional shell status without accidentally turning it into success.
    return result.returncode if result.returncode >= 0 else 128 - result.returncode


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
