# Developing Semauri from source

Use this guide when you want to change the compiler, grammar, semantics, domains, runtime planning or backends.

## Prerequisites

Install Git and Rust **1.83 or newer** with Cargo.

Check the toolchain:

~~~bash
git --version
rustc --version
cargo --version
~~~

## Download the source code

Clone the repository:

~~~bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
~~~

If you only want a source snapshot without Git history, GitHub also exposes **Code → Download ZIP**, but cloning is recommended for development because it makes branches, diffs and pull requests straightforward.

## Build and test the compiler

Run the complete test suite first:

~~~bash
cargo test --all-targets
~~~

Run the CLI directly from source:

~~~bash
cargo run -- version
cargo run -- check examples/hello.sema
cargo run -- build examples/shop.sema
~~~

Create an optimized native binary:

~~~bash
cargo build --release
./target/release/semauri version
~~~

On Windows:

~~~powershell
cargo build --release
.\target\release\semauri.exe version
~~~

Generated build output lives under <code>target/</code> and release packages under <code>dist/</code>; neither should be committed.

## Compiler source layout

~~~text
src/
├── lib.rs                         public crate facade
├── main.rs                        executable entry point
└── compiler/
    ├── core/
    │   ├── diagnostics.rs         spans, diagnostics and errors
    │   └── types.rs               values and type system
    ├── domains/
    │   ├── model.rs               domain specifications
    │   ├── registry.rs            registration and vocabulary lookup
    │   └── builtins/
    │       ├── web.rs
    │       ├── structured_data.rs
    │       ├── filesystem.rs
    │       └── ml.rs
    ├── frontend/
    │   ├── lexer.rs
    │   ├── ast.rs
    │   └── parser.rs
    ├── semantic/
    │   ├── hir_model.rs
    │   ├── hir_builder.rs
    │   ├── optimizer.rs
    │   └── effects.rs
    ├── runtime/
    │   ├── model.rs
    │   └── lowering.rs
    ├── backends/
    │   ├── api.rs
    │   ├── compiler.rs
    │   ├── html.rs
    │   ├── json_schema.rs
    │   ├── shell.rs
    │   └── filesystem_executor.rs
    ├── cli.rs
    └── tests.rs
~~~

The current staged modularization keeps these source slices in one internal compiler module so private helper contracts and the public API stay stable. Physical ownership is already separated, making later conversion into independent Rust modules incremental rather than a single risky rewrite.

## Where to make a language change

### Change syntax or grammar

Usually inspect:

- <code>src/compiler/frontend/lexer.rs</code>
- <code>src/compiler/frontend/ast.rs</code>
- <code>src/compiler/frontend/parser.rs</code>
- <code>src/compiler/semantic/hir_model.rs</code>
- <code>src/compiler/semantic/hir_builder.rs</code>

Add positive and negative tests under <code>tests/</code>. If the syntax changes user-facing behavior, update <code>docs/01_LANGUAGE.md</code>.

### Add or change a semantic domain

Built-in domain declarations live under <code>src/compiler/domains/builtins/</code>. Domain contracts and registration live in <code>model.rs</code> and <code>registry.rs</code>.

Prefer extending the domain specification instead of hard-coding domain nouns or verbs into the lexer/parser.

### Change type checking or semantic analysis

Use <code>src/compiler/semantic/hir_builder.rs</code> and the shared types in <code>src/compiler/core/types.rs</code>. Preserve diagnostic codes unless the change intentionally revises the public diagnostic contract.

### Add an optimization

Use <code>src/compiler/semantic/optimizer.rs</code>. Optimizations must preserve observable behavior, effects and trapping/error behavior.

### Change runtime planning or execution

Runtime IR and lowering are under <code>src/compiler/runtime/</code>. Explicit filesystem execution is isolated in <code>src/compiler/backends/filesystem_executor.rs</code>.

Compilation must remain effect-free.

### Add or change a backend

Backends live under <code>src/compiler/backends/</code>. A backend consumes resolved semantic IR; it should not parse source text or reimplement name/type resolution.

## Recommended change workflow

~~~bash
git checkout -b feature/my-language-change
cargo test --all-targets
# edit code and docs
cargo fmt --check
cargo test --all-targets
git diff
~~~

For a grammar or semantics change, also run representative CLI commands against examples and add regression tests for invalid or ambiguous input.

## Before opening a pull request

Make sure tests cover the intended behavior and important failure cases, documentation describes user-visible changes, no <code>target/</code> or <code>dist/</code> artifacts are included, domain changes respect the extension boundary, effectful behavior still requires explicit capabilities, and public diagnostics/CLI behavior are intentionally preserved or explicitly documented when changed.

See [../CONTRIBUTING.md](../CONTRIBUTING.md) for contribution policy and style.
