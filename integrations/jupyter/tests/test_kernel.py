"""Fast, offline unit tests: no Jupyter server and no Semauri build required."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from semauri_kernel.kernel import SemauriKernel


class KernelTests(unittest.TestCase):
    def setUp(self):
        self.kernel = SemauriKernel()
        self.sent = []
        self.kernel.send_response = lambda socket, typ, value: self.sent.append((typ, value))

    def test_command_has_no_implicit_external_capabilities(self):
        argv = self.kernel.command_for(Path("cell.suri"))
        self.assertEqual(argv, ["semauri", "run", "--allow", "console_write", "cell.suri"])

    def test_command_supports_explicit_permissions_and_rejects_injection(self):
        argv = self.kernel.command_for(Path("cell.suri"), binary="/opt/semauri",
                                       effects="console_write,filesystem_write")
        self.assertEqual(argv[:6], ["/opt/semauri", "run", "--allow",
                                    "console_write", "--allow", "filesystem_write"])
        with self.assertRaises(ValueError):
            self.kernel.command_for(Path("cell.suri"), effects="--help")

    def test_success_publishes_stdout_and_deletes_source(self):
        seen = []
        def execute(argv, **kwargs):
            path = Path(argv[-1])
            seen.append((path, path.read_text(encoding="utf-8"), kwargs))
            return subprocess.CompletedProcess(argv, 0, "[1, 4, 9]\n", "")
        with patch("semauri_kernel.kernel.subprocess.run", side_effect=execute):
            reply = self.kernel.do_execute('print "hello".', silent=False)
        self.assertEqual(reply["status"], "ok")
        self.assertEqual(self.sent, [("stream", {"name": "stdout", "text": "[1, 4, 9]\n"})])
        self.assertEqual(seen[0][1], 'print "hello".')
        self.assertFalse(seen[0][0].exists())
        self.assertIsNone(seen[0][2].get("shell"))

    def test_failure_publishes_original_diagnostic_and_cleans_up(self):
        with patch("semauri_kernel.kernel.subprocess.run",
                   return_value=subprocess.CompletedProcess([], 65, "", "S417: bad input\n")):
            reply = self.kernel.do_execute("bad code.", silent=False)
        self.assertEqual(reply["status"], "error")
        self.assertIn("S417", reply["evalue"])
        self.assertEqual(self.sent[0][0], "error")

    def test_json_failure_uses_stdout_diagnostic_without_duplicate_stream(self):
        diagnostic = '{"ok":false,"diagnostics":[{"code":"S417"}]}\n'
        with patch("semauri_kernel.kernel.subprocess.run",
                   return_value=subprocess.CompletedProcess([], 1, diagnostic, "")):
            reply = self.kernel.do_execute("%%check --json\nassert false.", silent=False)
        self.assertEqual(reply["status"], "error")
        self.assertIn("S417", reply["evalue"])
        self.assertEqual([kind for kind, _ in self.sent], ["error"])
        self.assertEqual(self.sent[0][1]["evalue"], diagnostic.strip())

    def test_failure_preserves_stdout_when_stderr_has_diagnostics(self):
        with patch("semauri_kernel.kernel.subprocess.run",
                   return_value=subprocess.CompletedProcess([], 65, "before error\n", "S417: invalid\n")):
            reply = self.kernel.do_execute("print missing.", silent=False)
        self.assertEqual(reply["status"], "error")
        self.assertEqual([kind for kind, _ in self.sent], ["stream", "error"])
        self.assertEqual(self.sent[0][1]["text"], "before error\n")
        self.assertEqual(reply["evalue"], "S417: invalid")

    def test_invalid_timeout_is_rejected_before_launch(self):
        for value in ("0", "-1", "nan", "inf", "-inf"):
            with self.subTest(value=value):
                with patch.dict(os.environ, {"SEMAURI_CELL_TIMEOUT": value}):
                    with patch("semauri_kernel.kernel.subprocess.run") as execute:
                        reply = self.kernel.do_execute("assert true.", silent=True)
                        self.assertEqual(reply["status"], "error")
                        self.assertIn("finite positive", reply["evalue"])
                        execute.assert_not_called()

    def test_silent_mode_does_not_publish_messages(self):
        with patch("semauri_kernel.kernel.subprocess.run",
                   return_value=subprocess.CompletedProcess([], 0, "ok\n", "")):
            reply = self.kernel.do_execute("assert true.", silent=True)
        self.assertEqual(reply["status"], "ok")
        self.assertFalse(self.sent)

    def test_timeout_fails_cleanly_without_replaying_code(self):
        with patch("semauri_kernel.kernel.subprocess.run",
                   side_effect=subprocess.TimeoutExpired("semauri", 1)):
            reply = self.kernel.do_execute("assert true.", silent=False)
        self.assertEqual(reply["status"], "error")
        self.assertIn("timeout", reply["evalue"])
        self.assertEqual(self.sent[0][0], "error")


if __name__ == "__main__":
    unittest.main()
