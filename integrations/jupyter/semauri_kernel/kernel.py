"""A strictly capability-scoped, per-cell Semauri kernel.

Semauri currently evaluates entire programs while lowering. Individual
cells run separately; persisting bindings by replaying old cells would
silently repeat filesystem/network/AI effects, so this kernel deliberately
does NOT pretend to offer persistent mutable sessions.
"""
import os
from pathlib import Path
import subprocess
import tempfile

from ipykernel.kernelbase import Kernel


class SemauriKernel(Kernel):
    implementation = "semauri"
    implementation_version = "0.1.0"
    language = "semauri"
    language_version = "0.48"
    language_info = {
        "name": "semauri",
        "mimetype": "text/x-semauri",
        "file_extension": ".suri",
        "pygments_lexer": "text",
        "codemirror_mode": "text",
    }
    banner = "Semauri Jupyter kernel — standalone, capability-checked .suri cells"

    @staticmethod
    def command_for(code, *, binary=None, effects=None):
        """Build argv without shell interpolation or implicit extra grants."""
        binary = binary or os.environ.get("SEMAURI_BINARY", "semauri")
        if effects is None:
            # Notebook text output is the only default authorized effect.
            effects = os.environ.get("SEMAURI_ALLOW_EFFECTS", "console_write")
        tokens = [entry.strip() for entry in effects.split(",") if entry.strip()]
        # No user-controlled command-line switches may be smuggled in via effects.
        if any(not part.replace("_", "").isalnum() for part in tokens):
            raise ValueError("SEMAURI_ALLOW_EFFECTS contains an invalid effect name")
        argv = [binary, "run"]
        for effect in tokens:
            argv += ["--allow", effect]
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
            with tempfile.NamedTemporaryFile(
                mode="w", encoding="utf-8", suffix=".suri", prefix=".semauri-cell-",
                dir=Path.cwd(), delete=False,
            ) as stream:
                path = Path(stream.name)
                stream.write(code)
            timeout = float(os.environ.get("SEMAURI_CELL_TIMEOUT", "60"))
            if timeout <= 0:
                raise ValueError("SEMAURI_CELL_TIMEOUT must be positive")
            proc = subprocess.run(
                self.command_for(path), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, encoding="utf-8", errors="replace", timeout=timeout,
                check=False,
            )
            if not silent and proc.stdout:
                self.send_response(self.iopub_socket, "stream",
                                   {"name": "stdout", "text": proc.stdout})
            if proc.returncode:
                diagnostic = proc.stderr.strip() or f"semauri exited with status {proc.returncode}"
                if not silent:
                    self.send_response(self.iopub_socket, "error", {
                        "ename": "SemauriError", "evalue": diagnostic,
                        "traceback": diagnostic.splitlines(),
                    })
                return {"status": "error", "execution_count": self.execution_count,
                        "ename": "SemauriError", "evalue": diagnostic,
                        "traceback": diagnostic.splitlines()}
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
