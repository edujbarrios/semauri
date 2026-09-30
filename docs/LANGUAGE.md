# Semauri language specification — draft 0.3

This document describes the currently implemented language, not aspirational syntax.

## Philosophy

Semauri uses **controlled natural language**. Natural-looking syntax does not mean unrestricted prose. Accepted programs have explicit grammar and deterministic semantics. Ambiguity is an error rather than an invitation to guess.

## Source files

Recommended extension: `.sema`.

Recognized English keywords are case-insensitive. Entity labels are normalized by the current English surface implementation; variable names are normalized to lowercase.

## Program

Approximate grammar:

```ebnf
program          = statement* EOF ;
statement        = create_web | add_title | add_element | make_property
                 | let_binding | set_property ;

create_web       = (CREATE | MAKE) [ARTICLE] WEB [web_qualifier] ["."] ;
web_qualifier    = CALLED phrase | FOR [ARTICLE] phrase ;
add_title        = ADD [ARTICLE] TITLE CALLED phrase ["."] ;
add_element      = ADD [ARTICLE] (BUTTON | IMAGE) [CALLED phrase] ["."] ;

make_property    = MAKE (PRONOUN | named_reference) COLOR ["."] ;

let_binding      = LET identifier BE value ["."] ;
set_property     = SET [ARTICLE] COLOR_PROPERTY OF reference TO value ["."] ;

reference        = PRONOUN | named_reference ;
named_reference  = [ARTICLE] (BUTTON | IMAGE) CALLED phrase ;

value            = COLOR | STRING | NUMBER | variable_reference ;
variable_reference = identifier ;
identifier       = WORD ;
phrase           = token+ ;
```

The EBNF is descriptive; the handwritten recursive-descent parser remains the executable grammar during 0.x.

## Values

0.3 introduces typed values. Current value types are:

- `color`: controlled color words such as `blue` or `red`
- `string`: quoted text such as `"Hello"`
- `number`: integer or decimal numbers such as `3` or `2.5`

A color literal and a string containing a color name are intentionally different:

```text
blue       # color
"blue"     # string
```

This distinction allows semantic type checking without relying on target-specific behavior such as CSS parsing.

## Variables

Declare a variable with `Let`:

```text
Let accent be blue.
```

The current 0.3 surface requires a single-word variable name. Variable names are case-insensitive and normalized to lowercase.

Bindings are immutable within a scope. Redeclaring the same name in one scope is semantic error `S311`.

Using an undeclared variable is semantic error `S312`.

## Canonical property assignment

The preferred extensible assignment syntax is:

```text
Set the color of the button called Buy to blue.
```

or with a variable:

```text
Let accent be blue.
Set the color of the button called Buy to accent.
```

The existing shorthand remains valid:

```text
Make it blue.
Make the button called Buy blue.
```

Shorthand is parsed into the same expression-oriented AST and semantic pipeline.

## Type checking

Properties declare an expected semantic value type. `color` currently expects a `color` value.

Therefore this is valid:

```text
Let accent be blue.
Set the color of the button called Buy to accent.
```

but this is not:

```text
Let accent be "blue".
Set the color of the button called Buy to accent.
```

The second program fails with `S313` because a `string` is not a `color`.

## References

`it` resolves only when exactly one addressable element exists. Explicit references such as `the button called Buy` are required when a pronoun would be ambiguous.

Semauri never selects a referent probabilistically.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

New 0.3 semantic diagnostics:

- `S311`: duplicate variable binding
- `S312`: unknown variable
- `S313`: property/value type mismatch

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic IR. No compiler component may randomly choose between ambiguous meanings.
