"""A strictly capability-scoped, per-cell Semauri kernel.

Semauri currently evaluates entire programs while lowering. Individual
cells run separately; persisting bindings by replaying old cells would
silently repeat filesystem/network/AI effects, so this kernel deliberately
does NOT pretend to offer persistent mutable sessions.
"""
import math
import os
import re
from pathlib import Path
import subprocess
import tempfile

from ipykernel.kernelbase import Kernel


class SemauriKernel(Kernel):
    implementation = "semauri"
    implementation_version = "0.10.0"
    language = "semauri"
    language_version = "0.65"
    language_info = {
        "name": "semauri",
        "mimetype": "text/x-semauri",
        "file_extension": ".suri",
        "pygments_lexer": "text",
        "codemirror_mode": "text",
    }
    banner = "Semauri Jupyter kernel — standalone, capability-checked .suri cells"

    # Completes names from this cell only; it never executes previous cells.
    KEYWORDS = (
        "let", "be", "then", "do", "print", "assert", "range", "from", "to", "of", "number",
        "string", "boolean", "map", "filter", "count", "reduce", "every",
        "if", "when", "otherwise", "elif", "unless", "end", "within", "define", "function",
        "taking", "invoke", "import", "square", "sqrt", "round", "truncate",
        "factorial", "even", "odd", "true", "false", "cube", "cbrt",
        "reciprocal", "digits", "binary", "octal", "hex", "hexadecimal", "prime", "capitalize", "swapcase",
        "uncapitalize", "titlecase", "sum", "average", "variance", "stdev",
        "power", "exp", "ln", "log2", "log10", "sin", "cos",
        "asin", "acos", "atan", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "log1p", "expm1", "minimum", "maximum", "median",
        "sorted", "distinct", "flatten", "partition", "split", "join", "joined", "different",
        "floor", "ceiling", "length", "reverse", "take", "drop", "top", "bottom", "json", "parse",
        "len", "abs", "mean", "min", "max", "strip", "upper", "lower",
        "avg", "var", "stddev", "total", "size", "unique", "cardinality", "head", "final", "med", "prod",
        "rev", "sort", "flat", "ascending", "descending",
        "empty", "blank", "zero", "positive", "negative", "nonzero", "nonnegative", "nonpositive", "even", "odd",
        "least", "most", "above", "below", "one", "divisible", "multiple",
        "includes", "excludes", "does", "contain", "include",
        "begins", "finishes", "begin", "finish",
    )

    # Offline help for frequently used Semauri expressions and notebook magics.
    # This is deliberately descriptive, never a runtime evaluation.
    KEYWORD_HELP = {
        "let": "let name be EXPRESSION. Create an immutable binding.",
        "avg": "avg of NUMBERS. Alias for checked average of numeric list.",
        "var": "var of NUMBERS. Alias for variance of numeric list.",
        "stddev": "stddev of NUMBERS. Alias for stdev of numeric list.",
        "total": "total of NUMBERS. Alias for sum of numeric list.",
        "size": "size of VALUE. Alias for length of typed list or Unicode string.",
        "unique": "unique of LIST. Alias for distinct of typed list.",
        "cardinality": "cardinality of LIST. Count distinct values using checked list equality.",
        "count": "count distinct of LIST or count unique of LIST. Number of distinct typed values.",
        "head": "head of LIST. Alias for first of typed list.",
        "final": "final of LIST. Alias for last of typed list.",
        "med": "med of NUMBERS. Alias for checked median of numeric list.",
        "prod": "prod of NUMBERS. Alias for checked product of numeric list.",
        "rev": "rev of VALUE. Reverse a typed list or Unicode text; alias for reverse of.",
        "sort": "sort of LIST. Return a sorted typed list; alias for sorted of. Also sort ascending/descending of LIST.",
        "top": "top COUNT from LIST. Highest COUNT items, returned in descending order. Checked integral count.",
        "bottom": "bottom COUNT from LIST. Lowest COUNT items, returned in ascending order. Checked integral count.",
        "ascending": "sorted ascending of LIST or sort ascending of LIST. Checked ascending typed-list order.",
        "descending": "sorted descending of LIST or sort descending of LIST. Checked descending typed-list order.",
        "flat": "flat of NESTED. Flatten one level of nested typed lists; alias for flatten of.",
        "prime": "prime of NUMBER, or NUMBER is prime / is not prime. Checked integral primality.",
        "len": "len of VALUE. Alias for length of text or list (Unicode characters).",
        "abs": "abs of NUMBER. Alias for checked absolute of NUMBER.",
        "mean": "mean of LIST. Alias for average of a number list.",
        "min": "min of LIST. Alias for minimum of a typed list.",
        "max": "max of LIST. Alias for maximum of a typed list.",
        "strip": "strip of TEXT. Alias for trim of TEXT.",
        "upper": "upper of TEXT. Alias for uppercase of TEXT.",
        "lower": "lower of TEXT. Alias for lowercase of TEXT.",
        "when": "when CONDITION: ... otherwise: ... end. Synonym for if.",
        "elif": "elif CONDITION: ... end. Shorthand for otherwise if.",
        "then": "Optional delimiter in if/when/elif CONDITION then: ... end.",
        "do": "Optional loop delimiter: for item in items do: ... end.",
        "in": "ITEM in COLLECTION. Typed list membership or text substring.",
        "binary": "binary of INTEGER. Exact signed binary digits as text, e.g. 42 -> 101010.",
        "octal": "octal of INTEGER. Exact signed octal digits as text.",
        "hex": "hex of INTEGER. Exact lowercase hexadecimal text.",
        "hexadecimal": "hexadecimal of INTEGER. Alias for hex of INTEGER.",
        "one": "ITEM is one of COLLECTION / is not one of COLLECTION. Strict typed membership.",
        "includes": "COLLECTION includes VALUE. Typed alias for contains.",
        "excludes": "COLLECTION excludes VALUE. Typed alias for not-contains.",
        "does": "TEXT does not start/end/begin/finish with X; COLLECTION does not contain/include X.",
        "begins": "TEXT begins with PREFIX. Checked Unicode, case-sensitive starts-with predicate.",
        "finishes": "TEXT finishes with SUFFIX. Checked Unicode, case-sensitive ends-with predicate.",
        "begin": "TEXT does not begin with PREFIX. Negated checked starts-with predicate.",
        "finish": "TEXT does not finish with SUFFIX. Negated checked ends-with predicate.",
        "divisible": "X is divisible by Y / is not divisible by Y. Typed numeric remainder check.",
        "multiple": "X is a multiple of Y / is not a multiple of Y. Typed numeric remainder check.",
        "true": "FLAG is true / is not true. Strict Boolean comparison.",
        "false": "FLAG is false / is not false. Strict Boolean comparison.",
        "empty": "VALUE is empty / is not empty. Typed empty list or string predicate.",
        "blank": "TEXT is blank / is not blank. Text is whitespace-only or empty.",
        "zero": "NUMBER is zero / is not zero. Checked numeric zero predicate.",
        "positive": "NUMBER is positive / is not positive. Strictly greater than zero.",
        "negative": "NUMBER is negative / is not negative. Strictly less than zero.",
        "nonzero": "NUMBER is nonzero / is not nonzero. Checked nonzero or zero predicate.",
        "nonnegative": "NUMBER is nonnegative / is not nonnegative. Checked >= 0 or < 0.",
        "nonpositive": "NUMBER is nonpositive / is not nonpositive. Checked <= 0 or > 0.",
        "even": "INTEGER is even / is not even. Checked parity predicate.",
        "odd": "INTEGER is odd / is not odd. Checked parity predicate.",
        "least": "NUMBER is at least LIMIT (>=), or is not at least LIMIT (<). Checked numeric comparison.",
        "most": "NUMBER is at most LIMIT (<=), or is not at most LIMIT (>). Checked numeric comparison.",
        "above": "NUMBER is above LIMIT (>), or is not above LIMIT (<=). Checked numeric comparison.",
        "below": "NUMBER is below LIMIT (<), or is not below LIMIT (>=). Checked numeric comparison.",
        "joined": "LEFT joined with RIGHT. Concatenate two strings only.",
        "different": "LEFT is different from RIGHT. Strict inequality.",
        "print": "print EXPRESSION. Requires the console_write capability.",
        "assert": "assert CONDITION. Stop when a condition is false.",
        "define": "define function NAME taking x as number to EXPRESSION.",
        "invoke": "invoke FUNCTION to ARGUMENT. Call a defined function.",
        "import": 'import "lib/module.suri". Load functions from a local module.',
        "map": "map every item in ITEMS to EXPRESSION. Transform a list.",
        "filter": "filter every item in ITEMS where CONDITION. Filter a list.",
        "variance": "variance of VALUES. Population variance of numeric values.",
        "stdev": "stdev of VALUES. Population standard deviation.",
        "power": "power of BASE to EXPONENT. Checked numeric exponentiation.",
        "log2": "log2 of X. Base-2 logarithm; X must be positive.",
        "log10": "log10 of X. Base-10 logarithm; X must be positive.",
        "asin": "asin of X. Inverse sine in radians; X must be between -1 and 1.",
        "acos": "acos of X. Inverse cosine in radians; X must be between -1 and 1.",
        "atan": "atan of X. Inverse tangent in radians.",
        "sinh": "sinh of X. Hyperbolic sine.",
        "cosh": "cosh of X. Hyperbolic cosine.",
        "tanh": "tanh of X. Hyperbolic tangent.",
        "asinh": "asinh of X. Inverse hyperbolic sine (finite number).",
        "acosh": "acosh of X. Inverse hyperbolic cosine, requires X >= 1.",
        "atanh": "atanh of X. Inverse hyperbolic tangent, requires -1 < X < 1.",
        "log1p": "log1p of X. Accurate natural logarithm of 1 + X; requires X > -1.",
        "expm1": "expm1 of X. Accurate exp(X) - 1; requires finite result.",
        "%%check": "%%check\nSOURCE. Check source without executing effects.",
        "%%audit": "%%audit\nSOURCE. Show the compiler's ML audit report.",
        "%%fmt": "%%fmt\nSOURCE. Print formatting without changing the cell.",
        "%%effects": "%%effects\nSOURCE. Inspect required capabilities.",
    }

    @staticmethod
    def parse_cell(code):
        """Kernel-only directives are not part of Semauri syntax."""
        lines = code.splitlines(keepends=True)
        first = next((i for i, line in enumerate(lines) if line.strip()), None)
        if first is None or not lines[first].strip().startswith("%%"):
            return "run", False, code
        words = lines[first].strip().split()
        commands = {"%%check": "check", "%%explain": "explain",
                    "%%effects": "effects", "%%plan": "plan",
                    "%%tokens": "tokens", "%%ast": "ast", "%%hir": "hir",
                    "%%optimize": "optimize", "%%symbols": "symbols",
                    "%%graph": "graph", "%%fmt": "fmt",
                    "%%audit": "audit"}
        if words[0] not in commands:
            raise ValueError(f"Unknown Semauri cell directive: {words[0]}")
        json_mode = (len(words) == 2 and words[1] == "--json"
                     and commands[words[0]] in ("check", "explain"))
        if len(words) != 1 and not json_mode:
            raise ValueError("Unsupported Semauri cell directive arguments")
        source = "".join(lines[first + 1:])
        if not source.strip():
            raise ValueError("Semauri cell directive requires source code")
        return commands[words[0]], json_mode, source

    @staticmethod
    def command_for(code, *, binary=None, effects=None, command="run", json_output=False):
        """Build argv without shell interpolation or implicit extra grants."""
        binary = binary or os.environ.get("SEMAURI_BINARY", "semauri")
        if effects is None:
            # Notebook text output is the only default authorized effect.
            effects = os.environ.get("SEMAURI_ALLOW_EFFECTS", "console_write")
        tokens = [entry.strip() for entry in effects.split(",") if entry.strip()]
        # No user-controlled command-line switches may be smuggled in via effects.
        if any(not part.replace("_", "").isalnum() for part in tokens):
            raise ValueError("SEMAURI_ALLOW_EFFECTS contains an invalid effect name")
        if command not in ("run", "check", "explain", "effects", "plan",
                           "tokens", "ast", "hir", "optimize", "symbols",
                           "graph", "fmt", "audit"):
            raise ValueError("Unsupported Semauri command")
        if json_output and command not in ("check", "explain"):
            raise ValueError("JSON diagnostics are not supported for this command")
        argv = [binary, command]
        if command == "run":
            for effect in tokens:
                argv += ["--allow", effect]
        if json_output:
            argv.append("--json")
        return argv + [str(code)]

    def do_execute(self, code, silent, store_history=True, user_expressions=None,
                   allow_stdin=False, *, cell_meta=None, cell_id=None):
        if not code.strip():
            return {"status": "ok", "execution_count": self.execution_count,
                    "payload": [], "user_expressions": {}}
        # The source lives in notebook cwd so normal project-local imports
        # retain their expected resolution and .semauri-root boundary.
        path = None
        try:
            command, json_mode, source = self.parse_cell(code)
            with tempfile.NamedTemporaryFile(
                mode="w", encoding="utf-8", suffix=".suri", prefix=".semauri-cell-",
                dir=Path.cwd(), delete=False,
            ) as stream:
                path = Path(stream.name)
                stream.write(source)
            timeout = float(os.environ.get("SEMAURI_CELL_TIMEOUT", "60"))
            if not math.isfinite(timeout) or timeout <= 0:
                raise ValueError("SEMAURI_CELL_TIMEOUT must be a finite positive number")
            proc = subprocess.run(
                self.command_for(path, command=command, json_output=json_mode),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, encoding="utf-8", errors="replace", timeout=timeout,
                check=False,
            )
            if proc.returncode:
                # --json diagnostics may be written to stdout instead of
                # stderr. Surface them as the cell error, not just an exit code.
                # Preserve separate stdout when stderr carries the diagnostic.
                if not silent and proc.stdout and proc.stderr:
                    self.send_response(self.iopub_socket, "stream",
                                       {"name": "stdout", "text": proc.stdout})
                diagnostic = (proc.stderr.strip() or proc.stdout.strip()
                              or f"semauri exited with status {proc.returncode}")
                if not silent:
                    self.send_response(self.iopub_socket, "error", {
                        "ename": "SemauriError", "evalue": diagnostic,
                        "traceback": diagnostic.splitlines(),
                    })
                return {"status": "error", "execution_count": self.execution_count,
                        "ename": "SemauriError", "evalue": diagnostic,
                        "traceback": diagnostic.splitlines()}
            if not silent and proc.stdout:
                self.send_response(self.iopub_socket, "stream",
                                   {"name": "stdout", "text": proc.stdout})
            if not silent and proc.stderr:
                self.send_response(self.iopub_socket, "stream",
                                   {"name": "stderr", "text": proc.stderr})
            return {"status": "ok", "execution_count": self.execution_count,
                    "payload": [], "user_expressions": {}}
        except (OSError, ValueError, subprocess.TimeoutExpired) as exc:
            if isinstance(exc, subprocess.TimeoutExpired):
                detail = f"Semauri cell exceeded its timeout: {exc.timeout} seconds"
            else:
                detail = str(exc)
            if not silent:
                self.send_response(self.iopub_socket, "error", {
                    "ename": "SemauriKernelError", "evalue": detail,
                    "traceback": [detail],
                })
            return {"status": "error", "execution_count": self.execution_count,
                    "ename": "SemauriKernelError", "evalue": detail,
                    "traceback": [detail]}
        finally:
            if path is not None:
                path.unlink(missing_ok=True)

    def do_complete(self, code, cursor_pos):
        before = code[:cursor_pos]
        match = re.search(r"[A-Za-z_][A-Za-z0-9_]*$", before)
        if match is None:
            return {"matches": [], "cursor_start": cursor_pos,
                    "cursor_end": cursor_pos, "metadata": {}, "status": "ok"}
        prefix = match.group().lower()
        candidates = set(self.KEYWORDS)
        # Scope suggestions to this cell. No replay of previous cells, imports,
        # external process, permissions or background compiler execution.
        for line in before.splitlines():
            if line.lstrip().startswith(("#", "//", "%%")):
                continue
            declarations = re.finditer(
                r"\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s+be\b"
                r"|\bdefine\s+function\s+([A-Za-z_][A-Za-z0-9_]*)\b",
                line, flags=re.IGNORECASE,
            )
            for declaration in declarations:
                candidates.add(declaration.group(1) or declaration.group(2))
        return {"matches": sorted(
                    (name for name in candidates if name.lower().startswith(prefix)),
                    key=lambda name: (name.lower(), name)),
                "cursor_start": match.start(), "cursor_end": cursor_pos,
                "metadata": {}, "status": "ok"}

    def do_inspect(self, code, cursor_pos, detail_level=0, omit_sections=()):
        # Jupyter requests contextual help at the cursor. Never execute code
        # just to display help or request capabilities on a hover.
        word = re.search(r"(?:%%)?[A-Za-z_][A-Za-z0-9_]*$", code[:cursor_pos])
        description = (self.KEYWORD_HELP.get(word.group().lower())
                       if word else None)
        if not description:
            return {"status": "ok", "found": False, "data": {}, "metadata": {}}
        return {"status": "ok", "found": True,
                "data": {"text/plain": description}, "metadata": {}}
