# Semauri

**A programming language for readable, typed and predictable software.**

**Write clearly. Compile confidently. Build with intent.**

Semauri is a **statically typed programming language** with its own compiler, executable programs and domain-specific backends. Write programs in approachable, structured English-like syntax, compile them deterministically and inspect the effects they require before execution.

Implemented in Rust, the compiler includes a lexer, parser, typed intermediate representation, lexical scopes, optimization, source-aware diagnostics and effect-aware execution planning. Semauri programs can perform authorized console and filesystem operations, generate web pages and structured schemas, and describe typed machine-learning workflows.

**Latest release:** [Semauri v0.20.0](https://github.com/edujbarrios/semauri/releases/latest) · **Source extension:** `.sema` · **Platforms:** Windows, macOS, Linux

## Get started

**1. Install Semauri.** No Rust installation is needed to run published native binaries.

On **Windows**, open **PowerShell** from the Start menu:

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

On **macOS or Linux**, open **Terminal**:

```sh
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Then open a new terminal if needed and verify the installation using `semauri version`.

**2. Create a file named `hello.sema` in a plain-text editor.** Add:

```sema
let answer be 6 times 7.
assert answer is equal to 42.
print "Hello from Semauri".
print answer.
```

**3. Open a terminal in the directory containing the file, then check and run it.**

```sh
semauri check hello.sema
semauri run --allow console_write hello.sema
```

For guidance on opening your terminal, creating a project folder, saving the right extension and fixing installation issues, see **[Your first Semauri program](docs/23_FIRST_PROGRAM.md)**. Prefer to inspect installers before running them? See the [manual, checksum-verified installation guide](docs/00_INSTALLATION.md).

## Your first program's output

To inspect what would happen before running it, use `semauri run --dry-run hello.sema`. Lowercase commands are the canonical style; earlier uppercase or mixed-case programs remain supported.

Expected console output:

```text
Hello from Semauri
42
```

An `assert` with a false condition reports a source-located runtime error (`S502`). Compilation, checking, planning, graph export and audits do not perform the program's declared effects.

## Lowercase syntax and formatter

The lexer recognizes language keywords and registered domain commands case-insensitively. `LET`, `Let`, and `let` have the same meaning, but all new official source files use lowercase.

```sh
semauri fmt examples/hello_console.sema           # print formatted source
semauri fmt examples/hello_console.sema --check   # exit 1 when formatting differs
semauri fmt examples/hello_console.sema --write   # rewrite this file
```

`fmt` uses real lexer tokens and source spans. It lowercases **recognized syntax**, not quoted strings, comments, identifiers, names or source formatting. It does not change Rust APIs, JSON field names or filesystem path case. All official examples and **22 bilingual course programs** now follow this convention, and an automated test enforces it. See [lowercase syntax](docs/21_LOWERCASE_SYNTAX.md).

## Language capabilities

| Area | Supported behavior |
| --- | --- |
| Typed core | Immutable bindings, numbers, strings, booleans, colors, homogeneous lists, comparisons, arithmetic and short-circuit logic |
| Control flow | `if`, `unless`, `otherwise if`, lexical scopes and **static** `for every` iteration |
| Collections | `length of`, typed `contains`, zero-based indexing, immutable list concatenation and `reverse of` |
| Numeric data | Checked `sum of` and typed `average of` numeric lists |
| String transformations | Unicode-aware `uppercase of`, `lowercase of` and `reverse of` |
| Assertions and tests | Pure typed `assert` statements, source-located failures and `semauri test` suites |
| Web and structured data | Typed declarations compiled to HTML and JSON Schema |
| Filesystem | Build shell plans or explicitly execute authorized filesystem operations; deletion is file-only |
| ML semantic domain | Typed dataset/model/device operations, training configs, LoRA/QLoRA, inference and evaluation **plans** |
| Analysis | Typed HIR, optimizer, symbols, declared effects, JSON diagnostics, SSA dependency graphs and ML reproducibility preflight |

**Current language scope:** User-defined typed functions and general runtime-dependent branching/loops are on the roadmap. Domain-specific ML operations currently produce typed workflow plans; executing real model training requires a framework adapter. See the [architecture](docs/08_ARCHITECTURE.md) and [roadmap](docs/10_ROADMAP.md).

### Typed collections

```sema
let numbers be a list of 5, 9, 13.
let more be a list of 21, 34.
let combined be numbers plus more.
assert combined contains 21.
assert (numbers at 1) is equal to 9.
assert length of combined is equal to 5.
```

List elements have one compatible type. List indexing is zero-based and checked for invalid or out-of-range positions; concatenation creates a new list without mutating its inputs. `reverse of` creates a new reversed list or string, while `sum of`, `average of`, `minimum of` and `maximum of` compute typed numeric aggregates. For [sequence reversal](docs/24_REVERSE.md), [numeric aggregates](docs/25_NUMERIC_AGGREGATIONS.md) and [numeric extrema](docs/26_NUMERIC_EXTREMA.md), see the new language reference sections. For string transformation, see [case expressions](docs/22_STRING_CASE.md): `uppercase of "hello"` returns `"HELLO"` and `lowercase of "HELLO"` returns `"hello"` without changing the case of Semauri syntax. See the [language reference](docs/01_LANGUAGE.md), [indexing](docs/16_LIST_INDEXING.md), and [list concatenation](docs/17_LIST_CONCATENATION.md).

### First-class assertion tests

```sh
semauri test examples/test_suite
semauri test examples/test_suite --allow-console
```

Test discovery is deterministic and recursive. Every test file must execute at least one `assert`; file/network/ML effects are rejected. Console printing requires explicit `--allow-console` authorization. A failed suite returns a nonzero exit code. See the [test runner contract](docs/15_TEST_RUNNER.md).

## ML engineering: typed, inspectable plans

ML syntax expresses **intended work** while retaining typed dependencies and declared effects. It is not a substitute for executing and evaluating experiments in PyTorch or another framework.

```sema
within ml:
  let dataset be source dataset "./images" revision "dataset-v3".
  let model be resolve model "resnet18" revision "weights-v2".
  let device be select device "cpu".
  let config be configure training for 3 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42.
  let run be fit model using dataset on device with config.
end.
```

Save as `train.sema` and inspect the compiler's output:

```sh
semauri check train.sema
semauri effects train.sema
semauri plan train.sema
semauri graph train.sema
semauri audit train.sema --strict
```

**`graph`** exports a versioned JSON SSA dependency graph with producer/consumer edges, declared effects, source spans and validated reference types. **`audit`** checks for unversioned dataset/model declarations and direct unseeded training forms; `--strict` exits with status `2` when warnings exist. **Declared revisions are unverified metadata** until an external execution adapter checks the artifacts. A warning-free audit is *not* proof of actual reproducibility, deterministic accelerators or successful training.

Read [ML semantics](docs/06_ML.md), [dataflow graphs](docs/18_ML_DATAFLOW.md), [version-qualified inputs](docs/19_ML_VERSIONED_INPUTS.md) and [audit limitations](docs/20_ML_AUDIT.md).

## Command reference

| Command | Purpose |
| --- | --- |
| `semauri help`, `semauri version`, `semauri domains` | CLI help, version and registered domains |
| `semauri check FILE [--json]` | Validate and type-check without executing; structured diagnostics with `--json` |
| `semauri fmt FILE [--check \| --write]` | Canonicalize syntax or enforce it in CI |
| `semauri test PATH [--allow-console]` | Run assertion-based tests in a file/directory |
| `semauri tokens FILE`, `semauri ast FILE` | Inspect token stream and syntax tree |
| `semauri hir FILE`, `semauri optimize FILE`, `semauri symbols FILE` | Inspect typed IR, optimization and symbol bindings |
| `semauri effects FILE`, `semauri plan FILE`, `semauri graph FILE` | Inspect effects, runtime operations and SSA dependencies |
| `semauri audit FILE [--strict]` | Analyze ML provenance/seed declarations for CI |
| `semauri explain FILE` | Explain compiler resolution and lowering |
| `semauri build FILE [-o PATH]` | Generate supported build-time artifacts |
| `semauri run FILE --allow EFFECT` | Run supported console or filesystem effects with explicit capabilities |

Use `--dry-run` with `semauri run` to inspect work without performing it. For more detail, see [runtime and permissions](docs/05_RUNTIME.md) and [capabilities](docs/04_EFFECTS.md).

## Two additional application examples

**Web generation:**

```sema
let price be 18.
let tax be 4.
let total be price plus tax.
create a web called Pet Shop.
add a button called Buy.
if total is greater than 20:
  set the color of the button called Buy to red.
otherwise:
  set the color of the button called Buy to green.
end.
```

```sh
semauri build examples/shop.sema
```

**Filesystem operations:**

```sema
within filesystem:
  mkdir "build".
  write "hello" to "build/notes.txt".
  copy "build/notes.txt" to "build/backup.txt".
  delete "build/old.tmp".
end.
```

```sh
semauri run --dry-run examples/filesystem.sema
semauri run --allow filesystem_read --allow filesystem_write examples/filesystem.sema
```

Execution requires explicit authorization and is **not sandboxed by `--cwd`**. File deletion refuses directories instead of recursively deleting their contents. Never execute untrusted programs with sensitive filesystem permissions.

## Learn and explore

Official Semauri releases include native executables for **Linux, macOS and Windows** on **x86_64 and ARM64**. Read the [installation guide](docs/00_INSTALLATION.md) for troubleshooting, version pinning and checksum-verified manual downloads.

**Learn to Code:** an 11-lesson course is available in [English](learn_to_code/en/README.md) and [Spanish](learn_to_code/es/README.md). The code commands in both tracks are lowercase; localized prose and string outputs remain unchanged.

## Architecture and development

```text
.sema source
    → lexer and domain vocabulary
    → recursive-descent parser and AST
    → name resolution and typed HIR
    → optimization and effect analysis
    → semantic IR / typed runtime plan
    → backend or authorized executor
```

Compiler source: [`src/compiler/`](src/compiler/). Public Rust extension APIs support registering additional domains and backends. The core avoids guessing ambiguous natural language.

```sh
git clone https://github.com/edujbarrios/semauri.git
cd semauri
cargo fmt --check
cargo test --all-targets
cargo run -- check examples/hello_console.sema
cargo run -- fmt examples/hello_console.sema --check
```

See [development](docs/DEVELOPMENT.md), [contributing](CONTRIBUTING.md), [documentation index](docs/README.md), [changelog](CHANGELOG.md), and the [v0.20.0 release notes](docs/releases/0.20.0.md).

## License

Copyright 2026 Eduardo J. Barrios. Licensed under the **Apache License 2.0**; see [LICENSE](LICENSE) and [NOTICE](NOTICE).
