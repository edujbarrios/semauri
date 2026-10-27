# Semauri Jupyter kernel (experimental)

Install Semauri's Rust executable on `PATH` first, then run:

```sh
python3 -m pip install -e ./integrations/jupyter
semauri-jupyter-install --user
python3 -m pip install jupyterlab
jupyter lab
```

Select **Semauri** as the kernel when opening a notebook. Each code
cell contains a standalone Semauri `.suri` program:

```text
let numbers be range from 1 to 5.
let squares be map every n in numbers to (square of n).
print squares.
```

Expected text output: `[1, 4, 9, 16, 25]`.

**Cells are deliberately independent.** The compiler evaluates entire
programs during lowering; concatenating and replaying prior cells would
repeat side effects (including filesystem/AI work) and silently change
semantics. For now, declare reusable functions or imports inside the same
cell; notebook execution does not preserve previous cell bindings. This
limitation will only be lifted after a true compiler-backed incremental
session protocol exists.

Only the `console_write` effect is granted by default for notebook
output. To explicitly add additional capabilities, set
`SEMAURI_ALLOW_EFFECTS=console_write,filesystem_write` before launching
Jupyter; treat every additional permission as a security decision.
`SEMAURI_BINARY=/path/to/semauri` chooses the compiler executable,
`SEMAURI_CELL_TIMEOUT=60` configures the per-cell timeout (seconds).
No shell interpretation is performed. Errors use Semauri's native diagnostic
text, and temporary `.suri` source files are removed after each cell.

Use `python3 -m unittest discover -s integrations/jupyter/tests` after
installing this package to test the adapter offline.
