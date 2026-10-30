# Semauri

**A readable, statically typed language for people and AI agents.** Write natural-language-inspired programs in `.suri`; compile with Rust, inspect effects, and run with explicit permissions. Current release: **v0.55.0**.

## Quick Start

Install the [latest binary for Windows, macOS or Linux](https://github.com/edujbarrios/semauri/releases/latest), or build from source with Rust:

```sh
cargo install --path . --locked
semauri version
```

Create `hello.suri` (also available as [`examples/quickstart.suri`](examples/quickstart.suri)):

```text
let numbers be a list of 1, 2, 3.
let doubled be map every n in numbers to (n times 2).
assert sum of doubled is equal to 12.
print doubled.
```

```sh
semauri check hello.suri
semauri run --allow console_write hello.suri
```

Output: `[2, 4, 6]`.

**Rules for humans and coding agents:** write lowercase keywords; finish each statement with `.`; declare immutable values before use; use `number`, `string`, `boolean`, and typed lists; declare pure functions with `define function ...`; use `if ... otherwise ... end.` for branches. Check generated code with `fmt` and `check` before running it. **No effects run unless explicitly allowed.** See the [complete language guide](docs/language-guide.md) for exact grammar and examples.

## Language at a glance

```text
define function square taking n as number to (n times n).
let value be invoke square to 7.
if value is equal to 49:
  print "Correct".
otherwise:
  print "Try again".
end.
```

Lists support `map every`, `filter every`, `reduce every`, `zip every`, `sum of`, `average of`, sorting, indexing, and ranges. Files can import typed functions. ML/AI workflows can be **planned and inspected**, but general model training is not yet an executable backend.

## Commands

| Command | Purpose |
| --- | --- |
| `semauri init PROJECT` | Scaffold a project |
| `semauri check FILE` | Validate syntax and types |
| `semauri fmt FILE --check` / `--write` | Check or apply formatting |
| `semauri run --allow console_write FILE` | Run and print (effect explicitly allowed) |
| `semauri test PATH [--allow-console]` | Execute assertion tests |
| `semauri verify PROJECT` | Check project formatting and tests |
| `semauri explain FILE` | Explain compiler decisions |
| `semauri effects FILE` / `plan FILE` / `graph FILE` | Inspect permissions and execution plan |
| `semauri build FILE [-o PATH]` | Generate build-time artifacts |
| `semauri config [PATH]` / `domains` / `version` | Inspect configuration, operations, or version |

Machine-readable output: `--json` on supported commands. Other inspection commands: `tokens`, `ast`, `hir`, `optimize`, `symbols`, `audit`. Run `semauri` for CLI help. [Reference and advanced operations](docs/language-guide.md).

## Real ML demo: linear regression

[`examples/linear_regression.suri`](examples/linear_regression.suri) implements **ordinary least squares from scratch**, adapted from [edujbarrios-ai](https://github.com/edujbarrios/edujbarrios-ai/tree/main/algorithms/02_classical_ml/linear_regression). Given `y = 2x + 1`, it learns the coefficients and predicts `y = 13` for `x = 6`.

```console
$ semauri run --allow console_write examples/linear_regression.suri
Linear regression (ordinary least squares)
Training observations:
5
Learned slope:
2.0
Learned intercept:
1
Prediction for x=6:
13
```

**Measured CLI wall time:** **1.802 ms median** (101 executions, 3 warmups); **2.159 ms** on the first run. GitHub Actions Ubuntu 24.04 / Linux x86_64, optimized binary, 2026-10-30. Includes startup, checking and execution; **excludes compilation**. Hardware and runner load affect results; this is *not* a cross-language performance comparison. [Verified benchmark and raw output](https://github.com/edujbarrios/semauri/actions/runs/41132112869). Reproduce with `cargo build --release --locked` and `python3 scripts/benchmark_linear_regression.py --samples 101`.

## Learn, extend, cite

[Full language guide](docs/language-guide.md) · [Examples](examples/) · [English course](learn_to_code/en/) · [Curso en español](learn_to_code/es/) · [Jupyter kernel](integrations/jupyter/README.md) · [VS Code](editors/vscode/) · [Contributing](CONTRIBUTING.md)

## Citation

Use GitHub's **Cite this repository** feature or [CITATION.cff](CITATION.cff). Suggested citation:

> Barrios, E. J. (2026). *Semauri: A natural-language-inspired programming language for humans and AI agents* (Version 0.55.0) [Computer software]. https://github.com/edujbarrios/semauri

Apache-2.0 licensed. See [LICENSE](LICENSE).
