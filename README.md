# Semauri

Semauri is a statically typed language for deterministic programs and inspectable AI workflows. Source files use `.suri`. This README is the canonical language guide for humans and coding agents.

Current release: **0.33.0**. The compiler is written in Rust and runs on Windows, macOS, and Linux.

## Agent quick start

When generating or changing a Semauri program:

1. Use a `.suri` filename and lowercase language keywords.
2. End every statement with a period.
3. Declare a value before using it. Values are immutable and lexically scoped.
4. Run `semauri fmt FILE --check`, then `semauri check FILE`.
5. Inspect `semauri effects FILE` and `semauri plan FILE` before execution.
6. Grant only the effects the program needs when using `run`.
7. Use `semauri test PATH` for assertion-based test files.

Never invent syntax. Prefer a small checked program over a plausible-looking unsupported construct.

## Install and verify

Download the archive for your platform from the [latest release](https://github.com/edujbarrios/semauri/releases/latest), extract it, and put the binary on `PATH`.

```sh
semauri version
```

From a source checkout:

```sh
cargo build --release
cargo test
cargo run -- check examples/hello.suri
```

## Smallest program

Create `hello.suri`:

```text
print "Hello from Semauri".
```

Then check and run it:

```sh
semauri check hello.suri
semauri run --allow console_write hello.suri
```

## Core syntax

```text
# immutable values and exact arithmetic
let name be "Semauri".
let answer be (6 times 7).
let values be a list of 1, 2, 3, 4.

# typed control flow
if answer is equal to 42:
  print name plus " works".
otherwise:
  print "unexpected".
end.

# pure typed function
define function double taking value as number to (value times 2).
let doubled be invoke double to 21.
assert doubled is equal to 42.
```

Comments begin with `#`, `//`, or a nestable `/* ... */` block. Keywords are case-insensitive, but lowercase is canonical. Strings preserve their case.

### Types

- `number`: checked signed integers and finite decimal values.
- `string`: UTF-8 text.
- `boolean`: `true` or `false`.
- `color`: named color values used by declarative programs.
- `list<T>`: immutable homogeneous lists.
- nominal runtime types: typed handles produced by operations.

Semauri does not silently coerce unrelated types.

### Expressions

Supported expression families include arithmetic, comparisons, Boolean operators, string concatenation, list indexing and concatenation, `length of`, `contains`, `uppercase of`, `lowercase of`, `reverse of`, numeric reductions, extrema, median, sorting, distinct values, bounded list operations, ranges, map/filter/count/reduce, pairwise zip, partition, flatten, repetition, conditional expressions, and pure function calls.

Use parentheses whenever generated code could be ambiguous.

### Control flow and scope

```text
let score be 91.

if score is greater than or equal to 90:
  print "excellent".
otherwise if score is greater than or equal to 70:
  print "pass".
otherwise:
  print "retry".
end.

unless false:
  print "runs".
end.
```

Branches and iteration bodies have lexical scope. Functions are pure, must be declared before use, accept 1–16 typed parameters, and have inferred return types.

## Effects and safe execution

Compilation is deterministic and does not grant capabilities. Inspect requirements explicitly:

```sh
semauri effects program.suri
semauri plan program.suri
semauri graph program.suri
```

Execution requires named grants:

```sh
semauri run --allow console_write program.suri
semauri run --allow filesystem_read --allow filesystem_write program.suri
```

Use `--dry-run` when you need the output plan without performing effects.

## `.config.suri`

Project configuration uses typed JSON parameters and named environment bindings:

```text
parameter model = "local/default".
parameter max_tokens = 1024.
parameter temperature = 0.2.
environment model_api_key = "SEMAURI_MODEL_API_KEY".
```

Validate it without exposing environment values:

```sh
semauri config
semauri config .config.suri --set temperature=0.0
semauri config .config.suri --check-env
```

`--set` may override only declared parameters and accepts a JSON value. Environment bindings are resolved lazily from the process environment; summaries report only the variable name and whether it is set, never its value. Do not commit credentials.

Programs read declared bindings at runtime with an explicit `config_read` effect:

```text
let model_identifier be string config "model".
let max_turns be number config "max_turns".
let safe_mode be boolean config "safe_mode".
```

`semauri run` discovers the nearest `.config.suri`, starting beside the source file and walking toward the filesystem root. Use `--config PATH` to select one explicitly or `--set NAME=JSON_VALUE` to override a declared parameter. Each binding is resolved only when the program reads it; unrelated environment secrets are never loaded.

## Assertions and tests

```text
let result be (6 times 7).
assert result is equal to 42.
```

```sh
semauri test examples/test_suite
semauri test path/to/tests --allow-console
```

The test runner recursively discovers `.suri` files, does not follow symbolic links, requires an executable assertion, and rejects non-console effects.

## LLM and VLM primitives

Semauri provides typed, provider-neutral primitives for language models, vision models, embeddings, tools, and bounded agents. They compile into an inspectable runtime plan. The built-in deterministic adapter executes `local/*` identifiers without network access; hosted identifiers require a provider adapter.

```text
let model_identifier be string config "language_model".
let request_budget be number config "request_budget".
cap ai requests to request_budget.
reattempt ai requests 2 attempts.
memoize ai response cache.
let model be connect language model model_identifier.
let assistant be compose agent using model system "Answer briefly." max 4 turns.
let answer be ask assistant with "Summarize the supplied context.".
print answer.
```

```text
let model_identifier be string config "vision_model".
let model be activate vision model model_identifier.
let description be inspect image "sample.png" using model with "Describe visible facts.".
print description.
```

Available primitives:

- `connect language model IDENTIFIER`
- `activate vision model IDENTIFIER`
- `cap ai requests to NUMBER`
- `reattempt ai requests NUMBER attempts`
- `memoize ai response cache`
- `prompt MODEL with TEXT`
- `inspect image PATH using MODEL with TEXT`
- `embed TEXT using MODEL`
- `declare tool NAME schema JSON_SCHEMA`
- `compose agent using MODEL system TEXT max NUMBER turns`
- `ask AGENT with TEXT`

Language-model and vision-model handles are distinct nominal types. Configuration reads, network, model loading, inference, image reads, console output, and tool execution remain explicit effects. Compilation never contacts a provider or reads credentials.

Request limits reject additional uncached inference once exhausted. The response cache is in-memory and scoped to one program run. Retry policy is stored by the runtime for provider adapters; the deterministic local adapter does not manufacture transient failures.

## Capability roadmap

Semauri is useful today for deterministic programs and inspectable local AI workflows. The larger runtime is intentionally being built in layers. Important capabilities not yet claimed as complete are:

- hosted-provider adapters with streaming, timeouts, rate limits, retry backoff, and fallback chains;
- structured records/maps, JSON decoding, typed extraction, classification, and schema validation;
- multi-turn conversation values, tool dispatch with validated arguments, execution traces, and cost accounting;
- HTTP, general file I/O, command-line input, persistent key-value storage, vector search, and text chunking;
- recoverable errors, modules/imports, packages, macros, pattern matching, and richer numeric types;
- formatter/editor services, debugger integration, a REPL, standalone builds, and WebAssembly tooling.

New runtime features should preserve the existing rules: typed inputs, immutable values, explicit capabilities, deterministic local tests, inspectable plans, and no secret values in diagnostics.

## Declarative programs

Semauri can build checked web, structured-data, and filesystem artifacts. Use a domain scope when operations need one:

```text
within web:
  create a web called Example.
  add a button called Start.
  set its color to blue.
end.
```

```sh
semauri build page.suri
semauri build page.suri -o page.html
```

## Compiler commands

| Command | Purpose |
| --- | --- |
| `config [PATH]` | Validate parameters and environment bindings |
| `check FILE` | Type-check and validate source |
| `fmt FILE [--check\|--write]` | Enforce canonical lowercase syntax |
| `tokens`, `ast`, `hir`, `optimize` | Inspect compiler stages |
| `symbols`, `explain` | Inspect names and semantic decisions |
| `effects`, `plan`, `graph` | Inspect capabilities and runtime dataflow |
| `build FILE [-o PATH]` | Render build-time artifacts |
| `run FILE --allow EFFECT` | Execute supported effects explicitly |
| `test PATH` | Run assertion suites |
| `domains` | Print registered operation metadata |
| `version` | Print the compiler version |

Add `--json` to supported inspection and validation commands when consuming output programmatically.

## Repository map

- `src/compiler/frontend/`: tokens, parser, AST, and formatting.
- `src/compiler/semantic/`: typed HIR and semantic analysis.
- `src/compiler/runtime/`: lowering, effects, dataflow, and execution plans.
- `src/compiler/domains/`: registered declarative and runtime operations.
- `src/compiler/backends/`: renderers, CLI execution, and test runner.
- `examples/`: checked runnable programs.
- `learn_to_code/`: English and Spanish courses.
- `tests/`: compiler and language contracts.
- `editors/vscode/`: installable VS Code metadata and the canonical `source.semauri` TextMate grammar.

## Editor and GitHub highlighting

The repository includes a TextMate grammar for `.suri` and `.config.suri` files under `editors/vscode/`. It provides local VS Code highlighting now and is the canonical public grammar for future GitHub Linguist integration.

GitHub only adds a new language after its public-usage threshold is met. Until `.suri` has sufficient independently distributed usage, GitHub will not provide official `semauri` fenced-code highlighting or a Semauri language bar entry. Do not mislabel Semauri as another language in `.gitattributes`; that would make repository statistics inaccurate.

## Contribution contract

Every language change should include parser/type/runtime coverage as applicable, a runnable `.suri` example, updates to this README, and passing tests. Preserve stable diagnostic codes and source spans. Keep effects explicit and never make compilation perform network, filesystem, model, or process work.

```sh
cargo test
cargo fmt --check
```

Licensed under Apache-2.0. See [LICENSE](LICENSE).
