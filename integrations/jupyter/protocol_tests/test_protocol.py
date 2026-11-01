import os
import time
import unittest
from jupyter_client import KernelManager

@unittest.skipUnless(os.environ.get("SEMAURI_BINARY"), "SEMAURI_BINARY not configured")
class ProtocolTests(unittest.TestCase):
    def test_jupyter_executes_real_compiler(self):
        manager = KernelManager(kernel_name="semauri")
        manager.start_kernel(env={**os.environ, "SEMAURI_CELL_TIMEOUT": "30"})
        client = manager.client()
        try:
            client.start_channels()
            client.wait_for_ready(timeout=30)
            for source, fragment in (
                ('print "Jupyter".', "Jupyter"),
                ("%%check\nassert true.", "OK"),
                ("%%explain\nwithin web:\n  create a web called Example.\n  add a button called Start.\nend.", "Entered semantic domain scope 'web'."),
            ):
                mid = client.execute(source)
                output = []
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    message = client.get_iopub_msg(timeout=10)
                    if message.get("parent_header", {}).get("msg_id") != mid:
                        continue
                    if message["msg_type"] == "stream":
                        output.append(message["content"]["text"])
                    if message["msg_type"] == "error":
                        self.fail(message["content"]["evalue"])
                    if message["msg_type"] == "status" and message["content"].get("execution_state") == "idle":
                        break
                reply = client.get_shell_msg(timeout=10)
                self.assertEqual(reply["content"]["status"], "ok", reply["content"])
                self.assertIn(fragment, "".join(output))
        finally:
            client.stop_channels()
            manager.shutdown_kernel(now=True)
