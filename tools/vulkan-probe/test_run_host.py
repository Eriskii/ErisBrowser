#!/usr/bin/env python3
"""Bounded fake-process tests; these do not require Vulkan or a GPU."""
import contextlib
import io
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import time
import unittest
from unittest import mock

import run_host as runner


def adapter(index=0, name="Test GPU"):
    return (f'ADAPTER {index} vendor=0x10de device=0x1234 type=DiscreteGpu '
            f'backend=Vulkan name="{name}" driver="fake" info="test"')


def passes():
    return [
        f"PASS {w}x{h} frame={frame} compared_bytes={w*h*4} packed_row={w*4} "
        f"readback_row=1280 readback_buffer={1280*h}"
        for w, h in [(320, 240), (319, 239)] for frame in range(3)
    ]


def output(lines, **record):
    return runner.RunResult({"status": "exited", "returncode": 0, **record},
                            ("\n".join(lines) + "\n").encode(), b"")


class ProtocolTests(unittest.TestCase):
    def test_complete_protocol_and_identity(self):
        devices = [adapter(), adapter(1, "second GPU")]
        self.assertEqual(runner.validate_listing(output(devices)), devices)
        lines = devices + ["TEST adapter=1"] + passes() + [
            "COMPLETE adapter=1 comparisons=6 exact=true"]
        runner.validate_adapter(output(lines), devices, 1)
        with self.assertRaisesRegex(ValueError, "inventory changed"):
            runner.validate_adapter(output([adapter(name="other"), *lines[1:]]), devices, 1)

    def test_malformed_inventory_is_rejected(self):
        cases = [[], [adapter(1)], [adapter(), adapter()],
                 [adapter(1), adapter()], [adapter(i) for i in range(17)],
                 [adapter().replace("Vulkan", "Gl")],
                 [adapter(name="x" * 4096)], [adapter(), "unexpected output"],
                 [adapter().replace("ADAPTER 0", "ADAPTER 00")]]
        for lines in cases:
            with self.subTest(lines=str(lines)[:100]), self.assertRaises(ValueError):
                runner.validate_listing(output(lines))

    def test_exit_zero_does_not_substitute_for_exact_results(self):
        good = [adapter(), "TEST adapter=0", *passes(),
                "COMPLETE adapter=0 comparisons=6 exact=true"]
        variants = [good[:2], good[:-1], good + [good[-1]],
                    [*good[:2], *passes()[:5], passes()[0], good[-1]],
                    [*good[:-1], "COMPLETE adapter=1 comparisons=6 exact=true"],
                    [line.replace("compared_bytes=307200", "compared_bytes=307199")
                     for line in good],
                    [good[1], good[0], *good[2:]],
                    [*good[:2], "PASS " + "9" * 10000, *good[3:]]]
        for lines in variants:
            with self.subTest(lines=str(lines)[:100]), self.assertRaises(ValueError):
                runner.validate_adapter(output(lines), [adapter()], 0)
        for status, code in [("timeout", 0), ("exited", 1), ("output_limit", 0)]:
            with self.assertRaises(ValueError):
                runner.validate_listing(output([adapter()], status=status, returncode=code))
        invalid_utf8 = output([adapter()])
        invalid_utf8.stdout += b"\xff"
        with self.assertRaisesRegex(ValueError, "UTF-8"):
            runner.validate_listing(invalid_utf8)


@unittest.skipUnless(os.name == "posix", "POSIX process groups are required")
class ProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="vulkan runner tests ")
        self.root = Path(self.temp.name)
        self.addCleanup(self.temp.cleanup)
        self.binary = self.root / "fake probe with spaces"
        self.logs = self.root / "output with spaces"

    def executable(self, body):
        self.binary.write_text(f"#!{sys.executable}\n" + body, encoding="utf-8")
        self.binary.chmod(0o755)
        return self.binary

    def run_child(self, body, **kwargs):
        self.executable(body)
        return runner.run_process(self.binary, [], "case", self.logs,
                                  os.environ.copy(), **kwargs)

    def assert_not_running(self, pid):
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            try:
                state = Path(f"/proc/{pid}/stat").read_text().split(") ", 1)[1][0]
            except FileNotFoundError:
                return
            if state == "Z":
                return  # Killed descendants can await their external reaper.
            time.sleep(0.01)
        os.kill(pid, signal.SIGKILL)
        self.fail(f"descendant {pid} remained alive after cleanup")

    def fork_body(self, leader_exit=False):
        path = self.root / "child.pid"
        return ("import os,time\n"
                "pid=os.fork()\n"
                "if pid == 0:\n"
                "    time.sleep(30)\n"
                "    os._exit(0)\n"
                f"with open({str(path)!r},'w') as file:\n"
                "    file.write(str(pid))\n"
                "print('ready', flush=True)\n" +
                ("os._exit(0)\n" if leader_exit else "time.sleep(30)\n")), path

    def test_success_spaces_and_portable_loader_environment(self):
        devices = [adapter(), adapter(1)]
        self.executable("import json,os,sys\n"
                        f"devices={devices!r}\nrecords={passes()!r}\n"
                        "print('\\n'.join(devices))\n"
                        "if sys.argv[1] == '--adapter':\n"
                        "    n=int(sys.argv[2])\n"
                        "    print(f'TEST adapter={n}')\n"
                        "    print('\\n'.join(records))\n"
                        "    print(f'COMPLETE adapter={n} comparisons=6 exact=true')\n"
                        "print(os.environ.get('LD_LIBRARY_PATH',''),file=sys.stderr)\n")
        loader = self.root / "loader with spaces"
        loader.mkdir()
        with mock.patch.dict(os.environ, {"LD_LIBRARY_PATH": "existing-loader"}):
            before = os.environ.copy()
            override = loader if sys.platform.startswith("linux") else None
            summary = runner.run_host(self.binary, self.logs, override, timeout=2)
            self.assertEqual(os.environ, before)
        self.assertTrue(summary["success"], summary)
        self.assertEqual(summary["adapter_count"], 2)
        self.assertEqual([r["args"] for r in summary["runs"]],
                         [["--list"], ["--adapter", "0"], ["--adapter", "1"]])
        self.assertFalse(Path(summary["binary"]).is_absolute())
        self.assertEqual(summary["runs"][0]["stderr_tail"].strip(),
                         (str(loader.resolve()) + ":" if override else "") + "existing-loader")
        self.assertEqual(json.loads((self.logs / "host-results.json").read_text()), summary)
        self.assertLess((self.logs / "host-results.json").stat().st_size, 20000)

    def test_argv_is_not_shell_text(self):
        self.executable("import json,sys\nprint(json.dumps(sys.argv[1:]))\n")
        args = ["two words", "$(not-a-command)", "semi;colon"]
        result = runner.run_process(self.binary, args, "argv", self.logs,
                                    os.environ.copy(), timeout=2)
        self.assertEqual(json.loads(result.stdout), args)

    def test_shared_output_cap_for_stdout_and_stderr(self):
        for fd in (1, 2):
            with self.subTest(fd=fd):
                result = self.run_child(f"import os\nwhile True: os.write({fd},b'x'*8192)\n",
                                        timeout=2, output_limit=4096)
                self.assertEqual(result.record["status"], "output_limit")
                self.assertEqual(len(result.stdout) + len(result.stderr), 4096)
                self.assertEqual(sum((self.logs / result.record[key]).stat().st_size
                                     for key in ("stdout_log", "stderr_log")), 4096)
                self.assertLessEqual(result.record["observed_bytes"], 4096 + 16384)
        result = self.run_child("import os\nwhile True:\n"
                                "    os.write(1,b'x'*1024)\n    os.write(2,b'y'*1024)\n",
                                timeout=2, output_limit=4096)
        self.assertEqual(result.record["status"], "output_limit")
        self.assertEqual(len(result.stdout) + len(result.stderr), 4096)

    @unittest.skipUnless(Path("/proc/self/stat").exists(), "Linux /proc proves child death")
    def test_timeout_kills_descendants_even_when_leader_exits(self):
        for leader_exit in (False, True):
            with self.subTest(leader_exit=leader_exit):
                body, pid_path = self.fork_body(leader_exit)
                started = time.monotonic()
                result = self.run_child(body, timeout=0.3)
                self.assertEqual(result.record["status"], "timeout")
                self.assertLess(time.monotonic() - started, 3)
                self.assert_not_running(int(pid_path.read_text()))

    @unittest.skipUnless(Path("/proc/self/stat").exists(), "Linux /proc proves child death")
    def test_successful_leader_cannot_leave_background_child(self):
        body, pid_path = self.fork_body(leader_exit=True)
        body = body.replace("    time.sleep(30)",
                            "    os.close(1)\n    os.close(2)\n    time.sleep(30)")
        result = self.run_child(body, timeout=2)
        self.assertEqual(result.record["status"], "exited")
        self.assertEqual(result.record["returncode"], 0)
        self.assert_not_running(int(pid_path.read_text()))

    @unittest.skipUnless(Path("/proc/self/stat").exists(), "Linux /proc proves child death")
    def test_interrupt_and_read_error_kill_process_group(self):
        real_selector = runner.selectors.DefaultSelector
        for exception, status in [(KeyboardInterrupt, "interrupted"), (OSError, "error")]:
            with self.subTest(exception=exception):
                body, pid_path = self.fork_body()

                class FailingSelector(real_selector):
                    def select(self, timeout=None):
                        events = super().select(timeout)
                        if events and pid_path.exists() and pid_path.read_text().strip():
                            raise exception("injected")
                        return events

                with mock.patch.object(runner.selectors, "DefaultSelector", FailingSelector):
                    result = self.run_child(body, timeout=2)
                self.assertEqual(result.record["status"], status)
                self.assert_not_running(int(pid_path.read_text()))

    def test_spawn_failure_and_false_pass_are_recorded(self):
        summary = runner.run_host(self.root / "missing", self.logs, timeout=1)
        self.assertFalse(summary["success"])
        self.assertEqual(summary["runs"][0]["status"], "error")
        self.executable(f"print({adapter()!r})\n")
        summary = runner.run_host(self.binary, self.logs, timeout=1)
        self.assertFalse(summary["success"])
        self.assertEqual(summary["runs"][1]["validation"], "failed")

    def test_invalid_limits_fail_before_spawn(self):
        for timeout in [float("nan"), float("inf"), 0, 121]:
            with self.subTest(timeout=timeout), self.assertRaises(ValueError):
                runner.run_process(self.binary, [], "test", self.logs, {}, timeout=timeout)
        with self.assertRaises(ValueError):
            runner.child_environment(self.root / "missing")
        with mock.patch.object(runner.sys, "platform", "darwin"), self.assertRaisesRegex(ValueError, "only on Linux"):
            runner.child_environment(self.root)
        with mock.patch.object(runner.os, "name", "nt"), self.assertRaisesRegex(RuntimeError, "POSIX"):
            runner.run_process(self.binary, [], "test", self.logs, {})

    def test_cli_returns_interrupted_status(self):
        fake = {"success": False, "adapter_count": 0,
                "runs": [{"status": "interrupted"}]}
        with mock.patch.object(runner, "run_host", return_value=fake), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.main(["--output-dir", str(self.logs)]), 130)


if __name__ == "__main__":
    unittest.main()
