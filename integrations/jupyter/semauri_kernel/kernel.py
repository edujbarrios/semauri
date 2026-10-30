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
    implementation_version = "0.1.0"
    language = "semauri"
    language_version = "0.54"
    language_info = {
        "name": "semauri",
        "mimetype": "text/x-semauri",
        "file_extension": ".suri",
        "pygments_lexer": "text",
        "codemirror_mode": "text",
    }
    banner = "Semauri Jupyter kernel — standalone, capability-checked .suri cells"

    KEYWORDS = ("let", "be", "print", "assert", "range", "from", "to", "of", "number", "string",
                "boolean", "map", "filter", "count", "reduce", "every", "if", "otherwise",
                "end", "within", "define", "function", "taking", "invoke", "square", "sqrt",
                "round", "truncate", "factorial", "even", "odd", "true", "false",
                "cube", "cbrt", "reciprocal", "digits", "prime", "capitalize", "swapcase", "uncapitalize", "titlecase")

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
                    "%%graph": "graph", "%%fmt": "fmt"}
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
                           "graph", "fmt"):
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
        match = re.search(r"[A-Za-z_]+$", code[:cursor_pos])
        if match is None:
            return {"matches": [], "cursor_start": cursor_pos,
                    "cursor_end": cursor_pos, "metadata": {}, "status": "ok"}
        return {"matches": sorted(k for k in self.KEYWORDS
                                  if k.startswith(match.group().lower())),
                "cursor_start": match.start(), "cursor_end": cursor_pos,
                "metadata": {}, "status": "ok"}
