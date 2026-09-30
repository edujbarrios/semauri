# Semauri language specification — draft 0.1

This document describes the currently implemented language, not aspirational syntax.

## Philosophy

Semauri uses **controlled natural language**. Natural-looking syntax does not mean unrestricted prose. Accepted programs have explicit grammar and deterministic semantics.

## Source files

Recommended extension: `.sema`.

Semauri 0.1 is case-insensitive for recognized keywords. Names are normalized to title case by the current English surface implementation.

## Program

A program is a sequence of statements.

Approximate grammar:

```ebnf
program       = statement* EOF ;
statement     = create_web | set_title ;

create_web    = CREATE [ARTICLE] WEB [web_qualifier] ["."] ;
web_qualifier = CALLED phrase | FOR [ARTICLE] phrase ;

set_title     = ADD [ARTICLE] TITLE CALLED phrase ["."] ;
phrase        = token+ ;
```

The EBNF is descriptive; the handwritten parser is the executable implementation during the 0.x period.

## Vocabulary aliases

Current English aliases:

```text
create  := create | make
web     := web | website | webpage | page
called  := called | named
```

Articles `a`, `an` and `the` are syntactic noise in positions where an article is accepted.

## Web semantics

### Explicit title

```text
Create a web called Hello World.
```

means:

```text
WebDocument(title="Hello World", title_origin=explicit)
```

### Subject-derived title

```text
Create a web for a pet store.
```

means:

```text
WebDocument(
  subject="Pet Store",
  title="Pet Store",
  title_origin=subject_default
)
```

The default is deterministic and inspectable with `semauri explain`.

### Fallback title

```text
Create a web.
```

currently produces title `Untitled`. This fallback may become stricter before 1.0.

### Title override

```text
Create a web for a pet store.
Add a title called Happy Paws.
```

results in title `Happy Paws` with an explicit origin.

## Errors

Compiler errors have stable-ish category codes during 0.x:

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

Exact codes may still evolve before 1.0.

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic IR. A compiler component must never randomly choose between ambiguous meanings.
