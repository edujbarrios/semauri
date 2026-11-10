# Semauri

<p align="center">
  <img src="assets/semauri_banner_dot.svg" alt="Semauri. — turquoise symbol and white wordmark ending with a period on a black background" width="850">
</p>

**Bringing human language closer to programming.**

Semauri is a programming language inspired by natural language, built for **humans and AI agents**. Write readable `.suri` code, catch mistakes with static type checking, and run it with explicit permissions.

[![Status: Active Development](https://img.shields.io/badge/status-active%20development-008080?labelColor=004d4d)](https://github.com/edujbarrios/semauri/commits/main)
[![Latest release](https://img.shields.io/github/v/release/edujbarrios/semauri?label=release&color=006666&labelColor=004d4d)](https://github.com/edujbarrios/semauri/releases/latest)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-004d4d?labelColor=006666)](LICENSE)

**Readable like language. Precise like code.** Start in minutes—no Rust installation needed when using a prebuilt binary.

## 1. Install

**No Rust toolchain is required** when using the prebuilt binaries. Choose your OS; each installer downloads the latest release and checks its SHA-256 checksum.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
semauri version
```

If `semauri` is not found, **open a new PowerShell window** and try again.

### macOS (Terminal)

```sh
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
semauri version
```

### Linux (Terminal)

```sh
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
semauri version
```

On macOS/Linux, the `export` updates `PATH` for this terminal session; add it to your shell profile to make it permanent. Requires `curl`, `tar`, and a SHA-256 utility (`shasum` or `sha256sum`).

**Alternatives:** [download a release manually](https://github.com/edujbarrios/semauri/releases/latest) or [build from source](docs/language-guide.md#install-and-verify) with Rust. Review [the install scripts](install.sh) ([Windows](install.ps1)) before executing downloaded code.

## 2. Your first program

Create a file called `hello.suri`:

```text
print "Hello, world!".
```

In the same folder, run:

```sh
semauri check hello.suri
semauri run --allow console_write hello.suri
```

Output: `Hello, world!`

**For humans and AI agents:** use lowercase keywords and `.suri` files; end statements with `.`; declare immutable values before use; follow documented syntax (do not guess). Run `semauri fmt FILE --check` and `semauri check FILE` before `run`. Inspect `semauri effects FILE` and grant **only** required permissions (printing needs `--allow console_write`). Use `--json` where supported for tool-friendly results. See the [language guide](docs/language-guide.md) for the grammar.

## 3. Learn by example

Start small and build up. Save any example as a `.suri` file, then run it with the same `check` and `run` commands above.

**Level 1 — Variables and arithmetic**

```text
let price be 10.
let quantity be 3.
print price times quantity.
```

**Level 2 — Conditions**

```text
let age be 20.
if age is greater than 17:
  print "Adult".
otherwise:
  print "Minor".
end.
```

**Level 3 — Typed functions**

```text
define function square taking n as number to (n times n).
let result be invoke square to 7.
assert result is equal to 49.
print result.
```

**Level 4 — Lists and transformations**

```text
let numbers be a list of 1, 2, 3.
let doubled be map every n in numbers to (n times 2).
assert sum of doubled is equal to 12.
print doubled.
```

Output: `[2, 4, 6]`. Also see [the quickstart file](examples/quickstart.suri).

**Level 5 — Real machine learning: linear regression**

The [complete example](examples/linear_regression.suri) implements ordinary least squares in Semauri, without a training service. For points following `y = 2x + 1`, it learns the slope and intercept, then predicts `13` for `x = 6`.

```sh
semauri run --allow console_write examples/linear_regression.suri
```

## 4. Command reference

| Command | What it does |
| --- | --- |
| `semauri init PROJECT` | Create a project with starter files and tests |
| `semauri check FILE` | Check syntax and types without executing |
| `semauri fmt FILE --check [--json]` | Check formatting; optionally emit structured results |
| `semauri fmt FILE --write` | Format the file |
| `semauri run --allow console_write FILE` | Run a program with permission to print |
| `semauri test PATH [--allow-console]` | Run assertion-based tests |
| `semauri verify PROJECT` | Check project formatting, types, and tests |
| `semauri explain FILE` | Explain compiler decisions |
| `semauri effects FILE` | Show required effects and permissions |
| `semauri plan FILE` | Inspect planned execution |
| `semauri graph FILE` | Visualize the execution plan |
| `semauri build FILE [-o PATH]` | Generate build artifacts |
| `semauri config [PATH]` | Inspect configuration |
| `semauri domains` | List supported domains |
| `semauri version` | Show the installed version |
| `semauri tokens FILE` | Inspect tokens |
| `semauri ast FILE` | Inspect the syntax tree |
| `semauri hir FILE` | Inspect typed intermediate representation |
| `semauri optimize FILE` | Inspect optimization |
| `semauri symbols FILE` | Inspect symbols |
| `semauri audit FILE` | Inspect the program audit |

Run `semauri` for CLI help. Supported commands also offer `--json` for machine-readable output. Other side effects require their own explicit permissions; see [the language guide](docs/language-guide.md).

## 5. Go further

[Language reference](docs/language-guide.md) · [More examples](examples/) · [English course](learn_to_code/en/) · [Curso en español](learn_to_code/es/) · [Jupyter](integrations/jupyter/README.md) · [VS Code](editors/vscode/) · [Release notes](docs/releases/0.69.0.md) · [Benchmarks](https://github.com/edujbarrios/semauri/actions/runs/41132112869) · [Contributing](CONTRIBUTING.md)

## Citation

To cite Semauri, use GitHub's **Cite this repository** option or the
[CITATION.cff](CITATION.cff) citation metadata.

Semauri is licensed under Apache-2.0 ([LICENSE](LICENSE)).
