# Semauri language specification — draft 0.4

This document describes implemented language behavior, not aspirational syntax.

## Philosophy

Semauri uses **controlled natural language**. Natural-looking syntax does not mean unrestricted prose. Accepted programs have explicit grammar and deterministic semantics. Ambiguity is an error rather than an invitation to guess.

## Approximate grammar

```ebnf
program          = statement* EOF ;
statement        = create_web | add_title | add_element | make_property
                 | let_binding | set_property | if_statement | for_each ;

let_binding      = LET identifier BE expression ["."] ;
if_statement     = IF expression ":" block [ OTHERWISE ":" block ] END ["."] ;
for_each         = FOR EVERY identifier IN expression ":" block END ["."] ;
block            = statement+ ;

expression       = logical_or ;
logical_or       = logical_and { OR logical_and } ;
logical_and      = logical_not { AND logical_not } ;
logical_not      = [ NOT ] (logical_not | comparison) ;
comparison       = additive [ IS comparison_operator additive ] ;
additive         = multiplicative { (PLUS | MINUS) multiplicative } ;
multiplicative   = primary { TIMES primary | DIVIDED BY primary } ;
primary          = COLOR | STRING | NUMBER | BOOLEAN | list_literal
                 | variable_reference | "(" expression ")" ;
list_literal     = [ ARTICLE ] LIST OF expression { "," expression } ;
```

The handwritten recursive-descent parser remains the executable grammar during 0.x.

## Values and expressions

Primitive semantic value types are `color`, `string`, `number`, and `boolean`.

Arithmetic is numeric and strictly typed. Multiplication/division bind more tightly than addition/subtraction. Equality requires both operands to have the same semantic type. `and` / `or` short-circuit and logical operators require booleans.

## Collections

Semauri supports homogeneous list literals:

```text
Let prices be a list of 10, 20, 30.
Let colors be a list of red, green, blue.
```

Their types are `list<number>` and `list<color>` respectively. Mixed lists are rejected with `S321`:

```text
Let invalid be a list of 1, "two".
```

List literals currently require at least one item so the element type can be inferred. There is no implicit union/coercion rule.

## Variables and lexical scope

`Let` creates immutable bindings. Blocks create child lexical scopes, parent bindings are visible inside child scopes, and child bindings can shadow parent names without mutating them.

Bindings have stable semantic symbol identities independent from source spelling. This supports HIR references and future tooling such as safe rename and go-to-definition.

## Conditional control flow

```text
If price is greater than 20:
  Add a button called Premium.
Otherwise:
  Add a button called Standard.
End.
```

The condition must be boolean (`S316`). The typed HIR preserves both branches. The current executable semantic lowering selects a branch statically because all current values are compile-time-known.

## Static iteration

```text
Let colors be a list of red, green, blue.

For every accent in colors:
  Set the color of the button called Buy to accent.
End.
```

`For every` requires a list (`S322`). The iteration variable has the list's element type and is visible only inside the loop body.

The iterator is one semantic declaration with one symbol ID. Static execution may bind different values to that same symbol across iterations; it does not create a new symbol per iteration.

The HIR preserves `for_each` structurally. The current executable lowering iterates compile-time-known lists during semantic analysis. Runtime/external collections will require a later control-flow IR.

## References and properties

`it` resolves only when exactly one addressable entity exists. Explicit references such as `the button called Buy` are required when a pronoun is ambiguous. Semauri never chooses a referent probabilistically.

`color` currently expects a `color` value. Property/value type mismatches produce `S313`.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

Relevant semantic diagnostics include:

- `S311`: duplicate binding
- `S312`: unknown variable
- `S313`: property/value type mismatch
- `S314`: invalid numeric operands
- `S315`: incompatible equality operands
- `S316`: non-boolean `If` condition
- `S317`: division by zero
- `S318`: unsupported expression operator
- `S319`: invalid logical operands
- `S320`: list element type cannot be inferred
- `S321`: heterogeneous list
- `S322`: iteration over a non-list value
- `S323`: internal symbol/value binding type mismatch

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic result. No compiler component may randomly choose between ambiguous meanings.
