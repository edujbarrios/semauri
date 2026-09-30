# Semauri language specification — draft 0.4

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
                 | let_binding | set_property | if_statement ;

create_web       = (CREATE | MAKE) [ARTICLE] WEB [web_qualifier] ["."] ;
web_qualifier    = CALLED phrase | FOR [ARTICLE] phrase ;
add_title        = ADD [ARTICLE] TITLE CALLED phrase ["."] ;
add_element      = ADD [ARTICLE] (BUTTON | IMAGE) [CALLED phrase] ["."] ;
make_property    = MAKE (PRONOUN | named_reference) COLOR ["."] ;

let_binding      = LET identifier BE expression ["."] ;
set_property     = SET [ARTICLE] COLOR_PROPERTY OF reference TO expression ["."] ;

if_statement     = IF expression ":" block
                   [ OTHERWISE ":" block ]
                   END ["."] ;
block            = statement+ ;

expression       = logical_or ;
logical_or       = logical_and { OR logical_and } ;
logical_and      = logical_not { AND logical_not } ;
logical_not      = [ NOT ] (logical_not | comparison) ;
comparison       = additive [ IS comparison_operator additive ] ;
comparison_operator = GREATER THAN | LESS THAN | EQUAL TO ;
additive         = multiplicative { (PLUS | MINUS) multiplicative } ;
multiplicative   = primary { TIMES primary | DIVIDED BY primary } ;
primary          = COLOR | STRING | NUMBER | BOOLEAN
                 | variable_reference | "(" expression ")" ;

reference        = PRONOUN | named_reference ;
named_reference  = [ARTICLE] (BUTTON | IMAGE) CALLED phrase ;
variable_reference = identifier ;
identifier       = WORD ;
phrase           = token+ ;
```

The EBNF is descriptive; the handwritten recursive-descent parser remains the executable grammar during 0.x.

## Values and expressions

Current semantic value types are `color`, `string`, `number`, and `boolean`.

Expressions are typed. Arithmetic is restricted to numbers:

```text
Let subtotal be 10 plus 5 times 2.
Let total be (10 plus 5) times 2.
```

Multiplication and division bind more tightly than addition and subtraction. Division by zero is semantic error `S317`.

## Comparisons

Numeric ordering:

```text
price is greater than 20
price is less than 20
```

Strict equality:

```text
price is equal to 20
accent is equal to blue
```

Equality requires operands of the same semantic type. Semauri does not silently coerce `1` into `"1"`.

## Logical expressions

Boolean expressions support `and`, `or`, and `not` with deterministic precedence:

```text
not       # applies to the following logical/comparison expression
and       # binds tighter than or
or
```

For example:

```text
If not price is greater than 20 and inStock:
  Add a button called Buy.
End.
```

`and` and `or` use **short-circuit evaluation**. The right-hand expression is not evaluated when the left side already determines the result. This matters for name resolution and, later, side-effect-free function calls. Logical operands must be boolean (`S319`).

## Variables and lexical scope

Declare immutable bindings with `Let`:

```text
Let accent be blue.
Let price be 25.
```

Blocks create child lexical scopes. Parent bindings are visible inside a block; a child block may shadow a parent name without mutating it.

```text
Let accent be blue.

If true:
  Let accent be red.
End.
```

The inner `accent` is `red`; after the block the outer `accent` is still `blue`. Bindings declared inside a block do not escape that block.

## Conditional control flow

```text
Let price be 25.
Create a web called Shop.

If price is greater than 20:
  Add a button called Premium.
Otherwise:
  Add a button called Standard.
End.
```

The condition must have type `boolean`; otherwise Semauri emits `S316`.

### Current evaluation model

In 0.4, all available values are immutable and known during semantic analysis. Therefore `If` branches are selected **during semantic resolution** and only the selected branch contributes to the resolved IR.

This is deliberately not presented as runtime control flow. When Semauri gains external/runtime values, conditions that cannot be resolved statically will require a dedicated control-flow IR rather than being guessed or prematurely evaluated.

## Canonical property assignment

```text
Let accent be blue.
Set the color of the button called Buy to accent.
```

Legacy shorthand remains valid:

```text
Make it blue.
Make the button called Buy blue.
```

Both forms lower through the same expression-oriented semantic pipeline.

## Type checking

Properties declare an expected semantic value type. `color` currently expects a `color` value.

```text
Let accent be "blue".
Set the color of the button called Buy to accent.
```

fails with `S313`, because a string is not a color.

## References

`it` resolves only when exactly one addressable element exists. Explicit references such as `the button called Buy` are required when a pronoun would be ambiguous.

Semauri never selects a referent probabilistically.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic errors
- `S4xx`: backend errors

Relevant semantic diagnostics:

- `S311`: duplicate variable binding
- `S312`: unknown variable
- `S313`: property/value type mismatch
- `S314`: arithmetic/order operator received non-number operands
- `S315`: equality operands have incompatible types
- `S316`: non-boolean `If` condition
- `S317`: division by zero
- `S318`: unsupported expression operator
- `S319`: logical operator received a non-boolean operand

## Determinism rule

Given the same Semauri version, source program, selected backend and backend version, valid source must resolve to the same semantic IR. No compiler component may randomly choose between ambiguous meanings.
