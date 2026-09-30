# Semauri

**SEMAURI — Semantic Expression Mapping and Unified Runtime Interpretation**

**Natural to write. Deterministic to run.**

## About

Semauri is an experimental programming language that lets people write deterministic programs using controlled natural language.

Its name reflects the compiler's goal: map human-readable expressions into a unified semantic representation that can be interpreted and lowered deterministically.

It is not an LLM wrapper and it does not ask AI to guess what the user meant. Semauri parses a defined language, builds compiler structures, resolves meaning, checks types and lowers the program to a target backend.

Created by **Eduardo J. Barrios** and open sourced from the beginning under the **Apache License 2.0**.

> Status: **0.5.x / experimental**

## Example

Create `shop.sema`:

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

### Compile it

```bash
ruby bin/semauri build shop.sema
```

Output:

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

`total` evaluates to `22`, so Semauri deterministically selects the first branch and the generated button is red.

### Explain the same program

```bash
ruby bin/semauri explain shop.sema
```

Output:

```text
1. Bound 'price' as symbol #1 to number 18.
2. Bound 'tax' as symbol #2 to number 4.
3. Variable 'price' resolved to symbol #1 (number 18).
4. Variable 'tax' resolved to symbol #2 (number 4).
5. Bound 'total' as symbol #3 to number 22.
6. 'web' resolved to an HTML web document (default web backend).
7. Title explicitly set to 'Pet Shop'.
8. Added button 'Buy' as button-1.
9. Variable 'total' resolved to symbol #3 (number 22).
10. If condition evaluated to true; selected consequence branch.
11. Explicit reference resolved to button 'Buy' (button-1).
12. Set button-1.color to "red".
```

The important part is that `explain` reports the compiler's actual semantic decisions; it is not an AI-generated explanation.

## How Semauri was built

The reference compiler is written in **Ruby** and intentionally avoids runtime dependencies in its core.

It started from a small handwritten compiler and has grown layer by layer:

```text
source
  ↓
lexer
  ↓
tokens
  ↓
recursive-descent parser
  ↓
AST
  ↓
scopes + symbols + type checking
  ↓
typed HIR
  ↓
optimization passes
  ↓
semantic IR
  ↓
backend
```

Important implementation choices:

- handwritten recursive-descent parser
- immutable AST/semantic objects where practical
- lexical scopes and stable semantic symbol IDs
- typed expressions and strict comparisons
- deterministic reference resolution
- composable HIR optimization passes
- explicit ambiguity errors
- source spans and compiler diagnostics
- visitor, strategy, registry and dependency-injection patterns
- tests used as executable language specification

The compiler is developed incrementally so each stage remains understandable, testable and replaceable.

## Current features

- numbers, strings, booleans and colors
- immutable `Let` bindings
- homogeneous `List<T>` values and `For every` iteration
- arithmetic and parentheses
- typed comparisons
- `and`, `or`, `not` with short-circuit evaluation
- lexical scopes and shadowing
- stable semantic symbols
- typed HIR
- constant propagation/folding and dead-branch elimination
- `If / Otherwise / End`
- web documents, buttons and images
- explicit references and constrained `it` resolution
- HTML backend
- source-aware diagnostics

## Development

Requires Ruby 3.2+.

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

CI tests Ruby 3.2, 3.3 and 3.4.

Other compiler inspection commands are available through `semauri help`, including `semauri hir` and `semauri optimize`.

## Project principles

- natural syntax does not mean ambiguous semantics
- the compiler must fail instead of guessing
- no LLM is required by the deterministic compiler core
- semantic work happens before code generation
- new language features require tests and documented semantics
- extension points should be explicit and maintainable

See [docs/LANGUAGE.md](docs/LANGUAGE.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/ROADMAP.md](docs/ROADMAP.md).

## Contributing

Pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Copyright 2026 Eduardo J. Barrios.

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
