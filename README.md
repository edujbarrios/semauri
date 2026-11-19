# Semauri

<p align="center">
  <img src="assets/semauri_banner_dot.svg" alt="Semauri — natural-language-inspired programming" width="850">
</p>

**Readable like language. Precise like code.**

Semauri is a statically typed programming language for humans and AI agents. Write
plain-English-inspired statements in `.suri` files, check them before execution,
and build programs without complicated syntax.

[![Latest release](https://img.shields.io/github/v/release/edujbarrios/semauri?label=release)](https://github.com/edujbarrios/semauri/releases/latest)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

## Install

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

**macOS / Linux:**

```sh
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

Installers download a platform-specific release and verify its SHA-256 checksum.
Review downloaded scripts before executing them. You can also
[download binaries manually](https://github.com/edujbarrios/semauri/releases/latest)
or [build from Rust source](docs/language-guide.md#install-and-verify).

## Your first Semauri program

Save as `hello.suri`:

```text
let numbers be a list of 1, 2, 3.
let doubled be map every n in numbers to (n times 2).
assert sum of doubled is equal to 12.
print doubled.
```

Run it **without permission flags**:

```sh
semauri hello.suri
# Equivalent: semauri run hello.suri
```

Output:

```text
[2, 4, 6]
```

## Language essentials

```text
# Immutable values and arithmetic
let price be 10.
let quantity be 3.
let total be price times quantity.

# Statically typed functions
define function square taking n as number to (n times n).
let result be invoke square to 7.

# Conditions
if total is greater than 20:
  print "Ready".
otherwise:
  print "Not ready".
end.

assert result is equal to 49.
```

**Composition rules:** end statements with `.`, use `:` to begin blocks and
`end.` to close them. Keywords are case-insensitive (lowercase is canonical).
Variables are immutable. Function parameters and expressions are statically
type-checked; unsupported combinations fail before execution.

## Main features

- **Types and data:** numbers, strings, booleans, typed lists, records,
  dictionaries, JSON and Unicode-aware text tools.
- **Collections and math:** `map every`, `filter every`, `count of`,
  `count every`, sorting, statistics and numeric functions.
- **Functions and flow:** reusable typed functions, `if`/`otherwise`,
  loops, assertions and local modules/libraries.
- **Tooling:** source formatter, static checker, built-in test runner,
  explanations, effect inspection and machine-readable JSON reports.
- **AI workflows:** typed AI/model primitives with inspectable plans and
  explicit capabilities; provider integration remains in development.

### Whole-word and whole-line matching (v0.79.0)

Use `TEXT has word VALUE` or `TEXT has line VALUE` to match whole
Unicode-aware words and exact lines; use `does not have` for the opposite.
These are distinct from substring `contains`. See the
[example](examples/whole_text_membership.suri).

### Text processing (v0.78.0)

`first word of text`, `last line of text`, `word count of text`,
`line count of text` and `normalize whitespace of text` make
Unicode-aware text processing easier. [Example](examples/text_composition.suri).

### Clear terminal output, safe additional effects

`print` works immediately with `semauri run FILE.suri` or
`semauri FILE.suri`. `test` and `verify` likewise allow prints in tests
by default. No `--allow-console` or `--allow console_write` is needed.

Other effects **still require permission**. For example, generating a secure
random password needs:

```text
let password be password of 20 characters.
assert count of password is equal to 20.
```

```sh
semauri run examples/secure_password.suri --allow secure_random
```

Use `semauri effects FILE.suri` to inspect required capabilities.
Use `--deny-console` when running or testing programs in a restricted
environment; printing is then disallowed. Avoid printing real secrets.

## Commands

| Command | Purpose |
| --- | --- |
| `semauri FILE.suri` / `semauri run FILE.suri` | Execute; console output enabled |
| `semauri run .` / `semauri check .` | Run or check the project's `main.suri` |
| `semauri check FILE.suri` | Validate syntax and types |
| `semauri fmt FILE.suri --write` | Format code |
| `semauri test PATH` | Run assertion tests |
| `semauri init PROJECT` | Create a starter project |
| `semauri verify PROJECT` | Check formatting, types and tests |
| `semauri effects FILE.suri` | Inspect required permissions |
| `semauri explain FILE.suri` / `semauri plan FILE.suri` | Inspect compilation and execution |
| `semauri version --json` | Machine-readable version |

For additional CLI and language syntax, see the
[language reference](docs/language-guide.md).

## Learn and extend

[Examples](examples/) · [Full language guide](docs/language-guide.md) ·
[Local libraries](docs/external-libraries.md) ·
[English course](learn_to_code/en/) · [Curso en español](learn_to_code/es/) ·
[VS Code](editors/vscode/) · [Jupyter](integrations/jupyter/README.md) ·
[Release notes](docs/releases/0.79.0.md) · [Benchmarks](benches/BASELINE.md) ·
[Contributing](CONTRIBUTING.md)

## Citation and license

Use GitHub's **Cite this repository** feature or [CITATION.cff](CITATION.cff).
Apache-2.0 license ([LICENSE](LICENSE)).
