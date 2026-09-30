# Semauri language specification — draft 0.2

This document describes the currently implemented language, not aspirational syntax.

## Philosophy

Semauri uses **controlled natural language**. Natural-looking syntax does not mean unrestricted prose. Accepted programs have explicit grammar and deterministic semantics.

## Source files

Recommended extension: `.sema`.

Semauri is case-insensitive for recognized keywords. Names are normalized to title case by the current English surface implementation.

## Program

A program is a sequence of statements.

Approximate grammar:

```ebnf
program       = statement* EOF ;
statement     = create_web | add_title | add_element | set_color ;

create_web    = (CREATE | MAKE) [ARTICLE] WEB [web_qualifier] ["."] ;
web_qualifier = CALLED phrase | FOR [ARTICLE] phrase ;

add_title     = ADD [ARTICLE] TITLE CALLED phrase ["."] ;
add_element   = ADD [ARTICLE] (BUTTON | IMAGE) [CALLED phrase] ["."] ;
set_color     = MAKE PRONOUN COLOR ["."] ;
phrase        = token+ ;
```

The EBNF is descriptive; the handwritten parser is the executable implementation during the 0.x period.

## Contextual `make`

`make` is intentionally contextual:

```text
Make a web called Hello.
```

means creation, while:

```text
Make it blue.
```

means property mutation. The parser represents the second form as a `SetProperty` AST node with a `PronounReference`; it does not decide which entity `it` means.

## Entity semantics

Elements introduced by the program become semantic entities with deterministic IDs such as `button-1` and `image-1`.

```text
Add a button called Buy.
```

creates an entity equivalent to:

```text
Element(id="button-1", kind=button, label="Buy")
```

The IR stores entities independently of HTML.

## Pronoun resolution

`it` is currently the only supported pronoun.

If exactly one addressable element exists:

```text
Create a web called Shop.
Add a button called Buy.
Make it blue.
```

`it` resolves deterministically to `button-1`.

If no addressable element exists, semantic error `S304` is emitted.

If multiple elements could be referenced, semantic error `S305` is emitted. Semauri does **not** silently select the most recent object:

```text
Create a web called Shop.
Add a button called Buy.
Add an image called Logo.
Make it blue.
```

is intentionally ambiguous and therefore invalid.

This strict rule gives future versions room to add explicit named references without changing the meaning of existing source.

## Web semantics

### Subject-derived title

```text
Create a web for a pet store.
```

produces a `WebDocument` whose title defaults to `Pet Store`.

### Title override

```text
Create a web for a pet store.
Add a title called Happy Paws.
```

sets the title explicitly to `Happy Paws`.

## Supported colors

The current English vocabulary recognizes a constrained set of named colors: black, white, red, green, blue, yellow, orange, purple, pink, gray/grey and brown.

This is intentionally narrower than CSS. Language semantics should not accidentally depend on browser-specific parsing rules.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

Relevant 0.2 semantic diagnostics:

- `S304`: pronoun has no referent
- `S305`: pronoun is ambiguous
- `S306`: element added before document creation
- `S307`: mutation before document creation
- `S308`: unsupported reference kind

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic IR. A compiler component must never randomly choose between ambiguous meanings.
