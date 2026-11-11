# Semauri Jupyter kernel (native Rust, experimental)

The Semauri Jupyter adapter, kernel protocol, installer, completion engine and
cell executor are now implemented in **Rust**. There is no Python source or
`ipykernel` dependency in this repository. JupyterLab itself is an external
application with its own installation and runtime requirements.

Install Semauri's `semauri` executable, a Rust toolchain and ZeroMQ development
libraries (for example `libzmq3-dev` and `pkg-config` on Ubuntu). Then:

```sh
cargo build --release --manifest-path integrations/jupyter/Cargo.toml
./integrations/jupyter/target/release/semauri-jupyter install --user
jupyter lab
```

On Windows, the kernel binary is `semauri-jupyter.exe`. The installer also
accepts `--prefix=/path/to/environment` and `--sys-prefix` when
`VIRTUAL_ENV` or `CONDA_PREFIX` is set. To remove the kernel, delete its
`kernels/semauri` directory from your Jupyter data directory.

Select **Semauri** when creating a notebook. Each cell is an independent
`.suri` program, for example:

```text
let numbers be range from 1 to 5.
let squares be map every n in numbers to (square of n).
print squares.
```

Output: `[1, 4, 9, 16, 25]`.

**Execution isolation and permissions:** Cells do not reuse bindings from
earlier cells or replay side effects. Temporary sources reside in the current
project directory for local imports and are deleted after execution.
`console_write` is the only permission granted by default. Additional
effects must be explicitly authorized using
`SEMAURI_ALLOW_EFFECTS=console_write,filesystem_write`. The executor invokes
the Semauri compiler without a shell; `SEMAURI_BINARY` selects the compiler,
and `SEMAURI_CELL_TIMEOUT=60` sets a positive finite timeout in seconds.
Timed-out subprocesses are killed. The kernel authenticates Jupyter messages
with HMAC-SHA256 and supports the standard TCP/ZeroMQ transport.

## Read-only notebook directives

The first nonblank line can be `%%check`, `%%explain`, `%%effects`,
`%%plan`, `%%tokens`, `%%ast`, `%%hir`, `%%optimize`, `%%symbols`,
`%%graph`, `%%audit`, or `%%fmt`. `%%check --json` and
`%%explain --json` emit structured reports. These directives are kernel
commands, **not Semauri source syntax**, and never authorize side effects.

```text
%%check
assert true.
```

Completions cover common natural-language keywords and names declared in the
current cell; hover inspection provides offline contextual help. Completions
do not execute code, read another cell's state, or grant permissions.

To test the kernel without Python:

```sh
cargo test --manifest-path integrations/jupyter/Cargo.toml
```

The native kernel has its own Cargo project so ordinary Semauri builds do
not require ZeroMQ. Notebook server implementations may still need external
software, but Semauri's kernel internals and tests are Rust-only.
