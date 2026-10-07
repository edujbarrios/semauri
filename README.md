# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

> Status: **0.14.0 / experimental**

## What is Semauri?

Semauri is an experimental open-source programming language based on **controlled natural language**. It combines an explicit grammar, typed HIR, lexical scopes, deterministic resolution, effects and semantic domains. It is not unrestricted English and it is not an LLM wrapper.

## What can it do today?

| Area | Current capability |
| --- | --- |
| Core | immutable bindings, arithmetic, booleans, comparisons, lists, lexical scopes, `If`, static `For every`, `#` comments, escaped strings |
| Console | print typed values from executable programs with explicit authorization |
| Web | build typed web documents and render HTML |
| Structured data | build typed schemas and render JSON Schema |
| Filesystem | plan and explicitly execute write/append/copy/move/mkdir/touch/delete workflows |
| ML | plan typed datasets, models, devices, training, inference, LoRA/QLoRA and training configuration |
| Tooling | inspect HIR, symbols, effects, optimization and runtime plans; capability-gated `run` |

Compilation remains effect-free. Execution is a separate opt-in step and effectful programs must be authorized explicitly.

## Write and run a `.sema` program

Semauri source files use the `.sema` extension. Create a file such as `hello.sema`:

```text
Let answer be 6 times 7.
Print "Hello from Semauri".
Print answer.
```

First validate it without executing effects:

```sh
semauri check hello.sema
```

Inspect what the runtime would execute:

```sh
semauri run --dry-run hello.sema
```

Then run it after explicitly authorizing console output:

```sh
semauri run --allow console_write hello.sema
```

The program prints:

```text
Hello from Semauri
42
```

The authorization is intentional: Semauri discovers required effects statically and refuses to execute them unless they are allowed. A missing capability produces a diagnostic instead of silently performing the effect.

## Install Semauri

The easiest way to use Semauri is to install the native binary published in GitHub Releases. End users do **not** need Rust.

Linux or macOS:

~~~bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
~~~

Windows PowerShell:

~~~powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
~~~

Verify the installation:

~~~text
semauri version
semauri help
~~~

To download without the installer, open the [latest GitHub Release](https://github.com/edujbarrios/semauri/releases/latest), choose the archive for your platform, verify it with <code>SHA256SUMS</code>, extract it and run the executable under <code>bin/</code>.

## CLI workflow

Use the following commands when working with a program:

| Command | Purpose |
| --- | --- |
| `semauri check FILE` | Parse, resolve and type-check without producing output or performing effects |
| `semauri run --dry-run FILE` | Inspect the executable runtime plan without performing effects |
| `semauri run --allow EFFECT FILE` | Execute a supported program with explicit capabilities |
| `semauri build FILE` | Compile build-time domains to HTML, JSON Schema or a shell plan |
| `semauri plan FILE` | Print the typed runtime operation plan as JSON |
| `semauri effects FILE` | List the capabilities required by the source |
| `semauri tokens/ast/hir FILE` | Inspect compiler stages while debugging language behavior |

Representative commands:

~~~bash
semauri check examples/hello_console.sema
semauri run --dry-run examples/hello_console.sema
semauri run --allow console_write examples/hello_console.sema
semauri build examples/shop.sema
semauri plan examples/filesystem.sema
~~~

`build` compiles source into its domain output and remains effect-free. Filesystem execution requires every capability used by the program:

~~~bash
semauri run --allow filesystem_read --allow filesystem_write examples/filesystem.sema
~~~

Use <code>--dry-run</code> to inspect the executable plan without performing effects. See [Installation](docs/00_INSTALLATION.md) for platform details and manual downloads.

### Source usability

Programs can now contain line comments and escaped text without preprocessing:

```text
# Strings support newline, tab, quote and backslash escapes.
Let message be "hello\nfrom Semauri".
```

Supported escapes are `\n`, `\t`, `\r`, `\"` and `\\`. Unknown escapes are rejected with a source diagnostic instead of being silently accepted.

## Example: Semauri to HTML

The repository includes this example as `examples/shop.sema`:

```text
Let price be 18.
Let tax be 4.
Let total be price plus tax.

Create a web called Pet Shop.
Add a button called Buy.

If total is greater than 20:
  Set the color of the button called Buy to red.
Otherwise:
  Set the color of the button called Buy to green.
End.
```

Build it with:

```sh
semauri build examples/shop.sema
```

Semauri evaluates the deterministic control flow (`18 + 4 > 20`) and generates this HTML:

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Pet Shop</title>
</head>
<body>
  <h1>Pet Shop</h1>
    <button style="color: red">Buy</button>
</body>
</html>
```

## Example: practical filesystem automation

```text
Within filesystem:
  Mkdir "build".
  Write "hello from Semauri" to "build/notes.txt".
  Append " - deterministic automation" to "build/notes.txt".
  Copy "build/notes.txt" to "build/backup.txt".
  Move "build/backup.txt" to "build/archive.txt".
  Touch "build/complete.marker".
  Delete "build/old.tmp".
End.
```

Compile without effects:

```sh
semauri build examples/filesystem.sema
```

Or explicitly execute it after authorizing capabilities:

```sh
semauri run --allow filesystem_read --allow filesystem_write examples/filesystem.sema
```

Use `--dry-run` to inspect the executable plan without performing effects, and `--cwd PATH` to choose the execution directory.

## Direction

Semauri is not yet a complete general-purpose runtime language. The next major milestones are user-defined typed procedures/functions, runtime-dependent control flow, structured record/map values, HTTP/process domains, richer filesystem reads/queries, modules and a small standard library.

The design rule remains: add useful power without giving up deterministic semantics, inspectable effects or the compiler/runtime boundary.

## Architecture

```text
Semauri source
    ↓
vocabulary + lexer
    ↓
recursive-descent parser
    ↓
syntax AST
    ↓
name resolution + type checking
    ↓
typed HIR
    ↓
optimization
    ↓
semantic/domain IR or RuntimePlan
    ↓
backend / explicit capability-gated runtime
```

## Documentation

Start with [docs/README.md](docs/README.md) for the documentation map. Installation and execution are covered in [docs/00_INSTALLATION.md](docs/00_INSTALLATION.md); language semantics start in [docs/01_LANGUAGE.md](docs/01_LANGUAGE.md).

## Develop and contribute to the language

The reference compiler is written in Rust. Contributors need Git and Rust 1.83 or newer; users of published binaries do not.

Clone the repository and establish a clean baseline:

~~~bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
cargo fmt --check
cargo test --all-targets
cargo run -- check examples/hello_console.sema
cargo run -- run --allow console_write examples/hello_console.sema
~~~

When changing the language, follow the complete pipeline rather than patching only the parser:

```text
source vocabulary and lexer
    -> syntax AST and parser
    -> name resolution and type checking
    -> typed HIR
    -> optimization and effect analysis
    -> domain IR or RuntimePlan
    -> backend or capability-gated executor
```

The main source areas are:

| Change | Start here |
| --- | --- |
| Syntax or grammar | `src/compiler/frontend/` and `src/compiler/semantic/` |
| Types, symbols or diagnostics | `src/compiler/core/` and `src/compiler/semantic/hir_builder.rs` |
| Semantic domain or vocabulary | `src/compiler/domains/` |
| Runtime planning or execution | `src/compiler/runtime/` and the relevant executor under `src/compiler/backends/` |
| Generated output | `src/compiler/backends/` |
| Optimizations | `src/compiler/semantic/optimizer.rs` |
| CLI behavior | `src/compiler/cli.rs` |

For every language change:

1. Add a motivating `.sema` example or test case.
2. Add positive tests and negative tests for invalid or ambiguous input.
3. Preserve source spans, diagnostic codes, type safety and explicit effects.
4. Update the relevant user documentation.
5. Run the complete validation before opening a pull request.

~~~bash
cargo fmt --check
cargo test --all-targets
cargo run -- check examples/hello_console.sema
cargo run -- run --allow console_write examples/hello_console.sema
git diff --check
~~~

Compilation must remain effect-free. New executable behavior must declare its effects and validate capabilities before execution. Semantic domains should extend the registry rather than introduce domain-specific lexer or parser special cases, and backends should consume resolved IR rather than parse source text.

Read [Developing Semauri](docs/DEVELOPMENT.md) for the compiler map and step-by-step workflows, [Contributing](CONTRIBUTING.md) for PR requirements, and the [documentation index](docs/README.md) for language, type, domain, runtime and architecture references.

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See `LICENSE` and `NOTICE`.
