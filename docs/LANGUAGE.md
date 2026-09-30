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

Arithmetic is numeric and strictly typed. Multiplication/division bind more tightly than addition/subtraction. Equality requires both operands to have the same semantic type.

### Logical expressions and short-circuiting

`and`, `or` and `not` operate on booleans. `and` / `or` use runtime/constant-evaluation short-circuiting: the right-hand expression is not **evaluated** when the left-hand value already determines the result.

However, name resolution and type checking happen earlier while building Typed HIR. Therefore every referenced name must exist and every subexpression must type-check, even if its runtime value would be short-circuited.

Valid short-circuit example:

```text
If true or 1 divided by 0 is equal to 1:
  Add a button called Safe.
End.
```

The division expression is well-formed and typed, but it is never evaluated, so no division-by-zero error occurs.

This is still invalid:

```text
If true or missingVariable is equal to true:
  Add a button called Invalid.
End.
```

`missingVariable` fails name resolution with `S312` before evaluation begins.

## Collections

Semauri supports homogeneous list literals:

```text
Let prices be a list of 10, 20, 30.
Let colors be a list of red, green, blue.
```

Their types are `list<number>` and `list<color>`. Mixed lists are rejected with `S321`. There is no implicit union/coercion rule.

## Variables and lexical scope

`Let` creates immutable bindings. Blocks create child lexical scopes, parent bindings are visible inside child scopes, and child bindings can shadow parent names without mutating them.

Bindings have stable semantic symbol identities independent from source spelling. Typed HIR references symbols by ID rather than repeating string lookup.

## Conditional control flow

```text
If price is greater than 20:
  Add a button called Premium.
Otherwise:
  Add a button called Standard.
End.
```

The condition must be boolean (`S316`). Typed HIR preserves both branches. The current HIR lowerer selects a branch statically because all current values are compile-time-known.

The symbol table describes the complete typed program, including declarations inside a statically unselected branch. Branch elimination belongs to a later optimization/lowering phase, not name resolution.

## Static iteration

```text
Let colors be a list of red, green, blue.

For every accent in colors:
  Set the color of the button called Buy to accent.
End.
```

`For every` requires a list (`S322`). The iteration variable has the list's element type and is visible only inside the loop body.

The iterator is one semantic declaration with one symbol ID. Static execution binds different values to that same symbol across iterations; it does not create a new declaration for each item.

HIR preserves `for_each` structurally. The current lowerer iterates compile-time-known lists. Runtime/external collections will require a later control-flow IR.

## References and properties

`it` resolves only when exactly one addressable entity exists. Explicit references such as `the button called Buy` are required when a pronoun is ambiguous. Semauri never chooses a referent probabilistically.

`color` currently expects a `color` value. Property/value type mismatches produce `S313`.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic/HIR errors
- `S4xx`: backend errors

Relevant diagnostics include:

- `S311`: duplicate binding
- `S312`: unknown variable during name resolution
- `S313`: property/value type mismatch
- `S314`: invalid numeric operands
- `S315`: incompatible equality operands
- `S316`: non-boolean `If` condition
- `S317`: division by zero during evaluation
- `S318`: unsupported expression operator
- `S319`: invalid logical operands
- `S320`: list element type cannot be inferred
- `S321`: heterogeneous list
- `S322`: iteration over a non-list value
- `S323`: internal symbol/value binding type mismatch
- `S324`: invalid HIR symbol binding/value environment
- `S325`: unsupported HIR node/operator

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic result. No compiler component may randomly choose between ambiguous meanings.
