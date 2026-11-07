import unittest
from pathlib import Path
from semauri_kernel.kernel import SemauriKernel

class MagicTests(unittest.TestCase):
    def test_parse_and_reject_unsupported_commands(self):
        parse = SemauriKernel.parse_cell
        self.assertEqual(parse("%%check\nassert true."), ("check", False, "assert true."))
        self.assertEqual(parse("%%explain --json\nprint 1."), ("explain", True, "print 1."))
        self.assertEqual(parse("%%effects\nprint 1."), ("effects", False, "print 1."))
        self.assertEqual(parse("%%audit\nassert true."), ("audit", False, "assert true."))
        self.assertEqual(parse("print 1."), ("run", False, "print 1."))
        for text in ("%%sh\necho unsafe", "%%explain --unsafe\nprint 1."):
            with self.assertRaises(ValueError):
                parse(text)

    def test_magics_have_no_capability_grants(self):
        self.assertEqual(SemauriKernel.command_for(Path("x.suri"), command="check"),
                         ["semauri", "check", "x.suri"])
        self.assertEqual(SemauriKernel.command_for(Path("x.suri"), command="explain", json_output=True),
                         ["semauri", "explain", "--json", "x.suri"])

    def test_new_read_only_inspection_directives(self):
        parse = SemauriKernel.parse_cell
        inspection = ("tokens", "ast", "hir", "optimize", "symbols", "graph", "fmt", "audit")
        for command in inspection:
            with self.subTest(command=command):
                self.assertEqual(parse("%%" + command + "\nprint 1."),
                                 (command, False, "print 1."))
                argv = SemauriKernel.command_for(Path("x.suri"), command=command,
                                                  effects="filesystem_write,console_write")
                self.assertEqual(argv, ["semauri", command, "x.suri"])
                with self.assertRaises(ValueError):
                    parse("%%" + command + " --write\nprint 1.")
                with self.assertRaises(ValueError):
                    parse("%%" + command + " --json\nprint 1.")
        with self.assertRaises(ValueError):
            SemauriKernel.command_for(Path("x.suri"), command="fmt", json_output=True)
        with self.assertRaises(ValueError):
            SemauriKernel.command_for(Path("x.suri"), command="unknown")

    def test_extended_language_completions(self):
        kernel = SemauriKernel()
        for prefix, expected in (("capit", "capitalize"), ("swap", "swapcase"),
                                 ("recip", "reciprocal"), ("dig", "digits"),
                                 ("uncap", "uncapitalize"), ("title", "titlecase"),
                                 ("log10", "log10"), ("log2", "log2"),
                                 ("asin", "asin"), ("acos", "acos"), ("atan", "atan"),
                                 ("asin", "asinh"), ("acos", "acosh"), ("atan", "atanh"),
                                 ("log1", "log1p"), ("expm", "expm1"),
                                 ("whe", "when"), ("join", "joined"), ("differ", "different"),
                                 ("eli", "elif"), ("the", "then"), ("do", "do"),
                                 ("len", "len"), ("abs", "abs"), ("mea", "mean"),
                                 ("min", "min"), ("max", "max"), ("stri", "strip"),
                                 ("upp", "upper"), ("low", "lower"),
                                 ("empt", "empty"), ("blan", "blank"),
                                 ("zer", "zero"), ("posit", "positive"),
                                 ("negat", "negative"), ("nonz", "nonzero"),
                                 ("nonn", "nonnegative"), ("nonp", "nonpositive"),
                                 ("eve", "even"), ("od", "odd"),
                                 ("lea", "least"), ("mos", "most"),
                                 ("one", "one"), ("divis", "divisible"), ("mult", "multiple"),
                                 ("incl", "includes"), ("excl", "excludes"),
                                 ("doe", "does"),
                                 ("avg", "avg"), ("var", "var"), ("stdde", "stddev"),
                                 ("tota", "total"), ("size", "size"),
                                 ("uniqu", "unique"), ("head", "head"), ("fina", "final"),
                                 ("med", "med"), ("prod", "prod"),
                                 ("rev", "rev"), ("sor", "sort"), ("fla", "flat"),
                                 ("bina", "binary"), ("octa", "octal"),
                                 ("hexa", "hexadecimal"), ("hex", "hex"),
                                 ("abov", "above"), ("belo", "below")):
            result = kernel.do_complete(prefix, len(prefix))
            self.assertIn(expected, result["matches"])

    def test_negated_numerical_bound_help(self):
        help_entries = SemauriKernel.KEYWORD_HELP
        for word, expected in (
            ("least", "is not at least"),
            ("most", "is not at most"),
            ("above", "is not above"),
            ("below", "is not below"),
        ):
            self.assertIn(expected, help_entries[word])

    def test_non_sign_predicate_help_is_offline(self):
        entries = SemauriKernel.KEYWORD_HELP
        for word in ("nonzero", "nonnegative", "nonpositive"):
            self.assertIn("is " + word, entries[word])
            self.assertIn("is not " + word, entries[word])

    def test_concise_collection_function_help(self):
        for word, original in (("rev", "reverse"), ("sort", "sorted"),
                               ("flat", "flatten")):
            entry = SemauriKernel.KEYWORD_HELP[word]
            self.assertIn(word + " of", entry)
            self.assertIn(original + " of", entry)

    def test_directional_sort_help_and_completion(self):
        help_entries = SemauriKernel.KEYWORD_HELP
        for direction in ("ascending", "descending"):
            self.assertIn("sorted " + direction + " of", help_entries[direction])
            self.assertIn("sort " + direction + " of", help_entries[direction])
            result = SemauriKernel().do_complete(direction[:4], 4)
            self.assertIn(direction, result["matches"])

    def test_natural_text_boundary_help_and_completion(self):
        entries = SemauriKernel.KEYWORD_HELP
        for positive, negative in (("begins", "begin"), ("finishes", "finish")):
            self.assertIn(positive + " with", entries[positive])
            self.assertIn("does not " + negative + " with", entries[negative])
            completion = SemauriKernel().do_complete(positive[:4], 4)
            self.assertIn(positive, completion["matches"])

    def test_top_and_bottom_collection_help(self):
        help_entries = SemauriKernel.KEYWORD_HELP
        for keyword, label in (("top", "Highest"), ("bottom", "Lowest")):
            self.assertIn(keyword + " COUNT from LIST", help_entries[keyword])
            self.assertIn(label, help_entries[keyword])
            completion = SemauriKernel().do_complete(keyword[:2], 2)
            self.assertIn(keyword, completion["matches"])

    def test_completion(self):
        result = SemauriKernel().do_complete("pri", 3)
        self.assertIn("print", result["matches"])
        self.assertEqual(result["cursor_start"], 0)
