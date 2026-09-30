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
set_color     = MAKE (PRONOUN | named_reference) COLOR ["."] ;
named_reference = [ARTICLE] (BUTTON | IMAGE) CALLED phrase ;
phrase        = token+ ;
```

The EBNF is descriptive; the handwritten parser is the executable implementation during the 0.x period.

## Contextual `make`

`make` is contextual:

```text
Make a web called Hello.
```

means creation, while:

```text
Make it blue.
```

means property mutation.

## Entity semantics

Elements introduced by the program become semantic entities with deterministic IDs such as `button-1` and `image-1`.

```text
Add a button called Buy.
```

creates an entity equivalent to:

```text
Element(id="button-1", kind=button, label="Buy")
```

## Pronoun resolution

`it` is currently the only supported pronoun. It resolves only when exactly one addressable element exists. With zero candidates Semauri emits `S304`; with multiple candidates it emits `S305` rather than guessing.

## Explicit references

When a pronoun would be ambiguous, source can identify an entity by kind and label:

```text
Create a web called Shop.
Add a button called Buy.
Add an image called Logo.
Make the button called Buy blue.
```

The explicit reference resolves to `button-1`, leaving `image-1` unchanged.

If no matching entity exists, Semauri emits `S309`. If more than one entity has the same kind and label, it emits `S310`.

## Web semantics

```text
Create a web for a pet store.
```

produces a `WebDocument` whose title defaults to `Pet Store`.

```text
Add a title called Happy Paws.
```

sets the title explicitly.

## Supported colors

The current English vocabulary recognizes black, white, red, green, blue, yellow, orange, purple, pink, gray/grey and brown.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

Reference diagnostics:

- `S304`: pronoun has no referent
- `S305`: pronoun is ambiguous
- `S309`: explicit reference does not exist
- `S310`: explicit reference is ambiguous

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic IR. A compiler component must never randomly choose between ambiguous meanings.
