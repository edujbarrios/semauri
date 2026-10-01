# Semauri architecture

Semauri follows a compiler pipeline with strict boundaries so language evolution remains reviewable and contributors can work on one phase without understanding every backend.

## Pipeline

```text
Semantic Domain Registry
  ├── vocabulary terms
  ├── property type contracts
  └── domain-IR construction
            │
            ▼
Source → Vocabulary + Lexer → Tokens
                              ↓
                   Recursive-descent Parser
                              ↓
                         Syntax AST
                              ↓
                     Typed HIR Builder
                      ├── name resolution
                      ├── stable symbols
                      └── type checking
                              ↓
                    HIR Optimization Passes
                      ├── constant folding
                      ├── dead control flow
                      └── dead bindings
                              ↓
                         HIR Lowerer
                      ├── value evaluation
                      ├── static control flow
                      └── domain dispatch
                              ↓
                      Semantic / Domain IR
                              ↓
                        Backend Registry
                              ↓
                         Target Artifact
```

## Semantic domains

A semantic domain is the extension boundary between the language core and application-specific concepts.

A domain contributes:

- artifact words such as `web`
- element words such as `button` and `image`
- property words such as `color`
- semantic kinds for those surface forms
- property type contracts
- domain-IR construction and mutation hooks
- optional source-oriented explanations

The built-in `Domains::Web` is registered through the same public contract used by external domains.

The lexer does not contain tokens named `WEB`, `BUTTON` or `IMAGE`. Domain words are classified into generic categories:

```text
web     → DOMAIN_ARTIFACT { domain: web, kind: web }
button  → DOMAIN_ELEMENT  { domain: web, kind: button }
color   → DOMAIN_PROPERTY { domain: web, kind: color }
```

The parser consumes those generic categories, which means a new domain can add vocabulary without editing lexer or parser code.

`Domains::Registry` rejects conflicting surface terms and is dependency-injected through `Compiler`, `Vocabulary::English`, `Parser`, `HIR::Builder` and `HIR::Lowerer`.

A custom compiler can therefore be assembled explicitly:

```ruby
registry = Semauri::Domains::Registry.new.register(MyDomain.new)
compiler = Semauri::Compiler.new(domains: registry)
```

`semauri domains` exposes loaded domain metadata for tooling and debugging.

## Syntax AST

The AST represents source syntax and source spans. Domain operations carry explicit domain identity, but the AST does not construct domain IR or target markup.

For example:

```text
Create a web called Shop.
```

becomes conceptually:

```text
CreateArtifact(domain=web, kind=web, title="Shop")
```

## Typed HIR

HIR is the compiler's semantic structural representation. Names become stable symbol references, expressions carry types, and domain operations retain their domain identity.

A source variable such as `price` is no longer identified by spelling after this phase:

```text
price → symbol_ref(#3, number)
```

Property checking is delegated through the active semantic domain. The core type system still owns primitive/expression types; domains define what types their properties accept.

## HIR optimization

Optimization is an explicit compiler phase driven by `HIR::Optimization::PassManager`. Passes receive typed HIR and return typed HIR; they do not emit target code or call backend renderers.

The current pipeline includes:

- propagation of compile-time-known immutable bindings
- arithmetic/comparison/logical constant folding
- short-circuit-aware folding
- elimination of statically unreachable conditional branches
- conservative dead immutable-binding elimination with liveness/use analysis

`build` lowers optimized HIR. `explain` intentionally lowers unoptimized HIR so its trace describes the source program rather than optimizer rewrites.

## HIR lowering

The lowerer owns language-level execution of compile-time-known constructs, but delegates domain operations to the registered domain implementation.

For example, the lowerer does **not** construct `IR::WebDocument` directly. It requests:

```text
domain.create_artifact(...)
domain.add_element(...)
domain.set_property(...)
```

This keeps domain IR out of parser and HIR infrastructure.

Because all current values are compile-time-known, the lowerer can currently evaluate conditions and iterate static lists. HIR itself still preserves branch and loop structure for future runtime lowering.

## Symbols and value environments

Semantic symbol identity and current values are separate concerns:

```text
Symbol #4
  name: accent
  type: color
  kind: iterator

Value environment, iteration 1: #4 → red
Value environment, iteration 2: #4 → green
```

This prevents static loop execution from inventing a new declaration on every iteration and prepares the compiler for runtime frames.

## Entity table

`Semantics::EntityTable` resolves addressable entities within the active artifact. Domain identity is validated before named references are resolved, preventing a property or element from one domain being silently applied to another.

## Domain IR and backends

Domains own their semantic IR. The built-in Web domain currently produces `IR::WebDocument` and `IR::Element`; another domain may return entirely different immutable structures.

Backends consume domain IR and never parse natural language or resolve references.

## Dependency direction

```text
CLI → Compiler → Domains + Lexer/Parser → AST → HIR → Optimization → Lowering → Domain IR ← Backends
```

Key rules:

- lexer/parser must not hardcode domain nouns
- AST must not depend on a target backend
- HIR must not depend on target markup
- optimization passes must not perform domain rendering
- domain IR must not depend on parser tokens
- backends must not parse source text
- ambiguity resolution must not happen in a backend
- optional NLP/LLM support must sit before the deterministic compiler boundary

## Why no LLM in the compiler core?

An optional future free-form adapter may translate unrestricted prose into strict Semauri source:

```text
free-form language → optional adapter → strict Semauri → compiler
```

The resulting program is still parsed, typed and compiled deterministically.
