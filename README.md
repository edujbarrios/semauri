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

## Example: execute a Semauri program

```text
Let answer be 6 times 7.
Print "Hello from Semauri".
Print answer.
```

Run it after authorizing console output:

```sh
semauri run --allow console_write examples/hello_console.sema
```

The program prints:

```text
Hello from Semauri
42
```

## Install, download and run Semauri

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

A Semauri source file uses the <code>.sema</code> extension. Common commands are:

~~~bash
semauri check examples/hello.sema
semauri build examples/shop.sema
semauri plan examples/filesystem.sema
~~~

<code>build</code> compiles source into the domain output (for example HTML, JSON Schema or a shell plan). <code>run</code> is reserved for explicitly supported effectful execution. Filesystem programs require the capabilities they use:

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

## Developing the language

To modify Semauri itself, clone the repository and build it with Rust:

~~~bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
cargo test --all-targets
cargo run -- check examples/hello.sema
~~~

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for the compiler layout and a step-by-step guide to changing syntax, semantics, domains, optimizations, runtimes or backends. Contribution requirements live in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See `LICENSE` and `NOTICE`.
