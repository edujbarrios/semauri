# Semauri

Semauri is a statically typed language for deterministic programs and inspectable AI workflows. Source files use `.suri`. This README is the canonical language guide for humans and coding agents.

Current release: **0.31.0**. The compiler is written in Rust and runs on Windows, macOS, and Linux.

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

## Contribution contract

Every language change should include parser/type/runtime coverage as applicable, a runnable `.suri` example, updates to this README, and passing tests. Preserve stable diagnostic codes and source spans. Keep effects explicit and never make compilation perform network, filesystem, model, or process work.

```sh
cargo test
cargo fmt --check
```

Licensed under Apache-2.0. See [LICENSE](LICENSE).
