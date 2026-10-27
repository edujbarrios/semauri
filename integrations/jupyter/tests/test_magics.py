import unittest
from pathlib import Path
from semauri_kernel.kernel import SemauriKernel

class MagicTests(unittest.TestCase):
    def test_parse_and_reject_unsupported_commands(self):
        parse = SemauriKernel.parse_cell
        self.assertEqual(parse("%%check\nassert true."), ("check", False, "assert true."))
        self.assertEqual(parse("%%explain --json\nprint 1."), ("explain", True, "print 1."))
        self.assertEqual(parse("%%effects\nprint 1."), ("effects", False, "print 1."))
        self.assertEqual(parse("print 1."), ("run", False, "print 1."))
        for text in ("%%sh\necho unsafe", "%%explain --unsafe\nprint 1."):
            with self.assertRaises(ValueError):
                parse(text)

    def test_magics_have_no_capability_grants(self):
        self.assertEqual(SemauriKernel.command_for(Path("x.suri"), command="check"),
                         ["semauri", "check", "x.suri"])
        self.assertEqual(SemauriKernel.command_for(Path("x.suri"), command="explain", json_output=True),
                         ["semauri", "explain", "--json", "x.suri"])

    def test_completion(self):
        result = SemauriKernel().do_complete("pri", 3)
        self.assertIn("print", result["matches"])
        self.assertEqual(result["cursor_start"], 0)
