# Semauri language specification — draft 0.2

This document describes the currently implemented language, not aspirational syntax.

## Philosophy

Semauri uses **controlled natural language**. Natural-looking syntax does not mean unrestricted prose. Accepted programs have explicit grammar and deterministic semantics.

The compiler may infer meaning only when the rule is deterministic and documented. Ambiguity is an error.

## Source files

Recommended extension: `.sema`.

Semauri is case-insensitive for recognized keywords. Names are normalized to title case by the current English surface implementation.

## Program

A program is a sequence of statements.

Approximate grammar:

```ebnf
program         = statement* EOF ;
statement       = create_statement | add_statement | make_statement ;

create_statement = CREATE [ARTICLE] (create_web | create_button) ;
add_statement    = ADD [ARTICLE] (set_title | create_button) ;
make_statement   = MAKE (create_web_compat | set_color) ;

create_web      = WEB [web_qualifier] ["."] ;
web_qualifier   = CALLED phrase | FOR [ARTICLE] phrase ;
set_title       = TITLE CALLED phrase ["."] ;
create_button   = BUTTON [CALLED phrase] ["."] ;

set_color       = reference COLOR ["."] ;
reference       = PRONOUN
                | [ARTICLE] BUTTON
                | [ARTICLE] BUTTON CALLED phrase ;

phrase          = token+ ;
```

The EBNF is descriptive; the handwritten parser is the executable implementation during the 0.x period.

## Contextual `make`

`make` is intentionally contextual in 0.2.

```text
Make a website named Hello.
```

remains compatible with 0.1 and creates a web document.

```text
Make it blue.
```

is a property mutation. The lexer does not decide which meaning applies; the parser does.

## Web semantics

### Subject-derived title

```text
Create a web for a pet store.
```

resolves `title := subject`, producing `Pet Store`.

### Buttons

```text
Create a web for a pet store.
Add a button called Buy.
```

creates a referable button element in the web document IR.

Buttons receive stable semantic IDs such as `button-1`. IDs are compiler-internal and are not part of source syntax.

## References and pronouns

Semauri 0.2 introduces deterministic reference resolution.

### Pronoun

```text
Create a web.
Add a button called Buy.
Make it blue.
```

`it` resolves because there is exactly one referable element.

### Ambiguity

```text
Create a web.
Add a button called Buy.
Add a button called Cancel.
Make it blue.
```

is invalid. Semauri does not guess which button `it` means.

Use an explicit reference instead:

```text
Make the button called Buy blue.
```

### Kind reference

```text
Create a web.
Add a button.
Make the button red.
```

is valid only when exactly one button exists.

## Colors

0.2 recognizes a deliberately small built-in color vocabulary:

`black`, `blue`, `brown`, `gray`, `green`, `orange`, `pink`, `purple`, `red`, `white`, `yellow`.

This list is intentionally constrained while the property/type system is still being designed.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

Reference resolution currently uses:

- `S305`: no matching referent
- `S306`: ambiguous referent

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic IR. A compiler component must never randomly choose between ambiguous meanings.
