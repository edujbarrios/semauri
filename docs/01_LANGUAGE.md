# Semauri language reference — v0.27.0

This document describes implemented language behavior, not aspirational syntax.

## Philosophy

Semauri uses **controlled natural language**. Natural-looking syntax does not mean unrestricted prose. Accepted programs have explicit grammar and deterministic semantics. Ambiguity is an error rather than an invitation to guess.

**Canonical source syntax is lowercase.** Core and registered-domain commands remain case-insensitive for backwards compatibility; use `semauri fmt FILE [--check | --write]` for canonical style. Quoted text, comments, user names, and Rust/JSON identifiers are not forced to lowercase. See [lowercase syntax](21_LOWERCASE_SYNTAX.md).

## Approximate grammar

```ebnf
program          = statement* EOF ;
statement        = create_artifact | add_title | add_element | make_property
                 | let_binding | set_property | if_statement | for_each
                 | domain_operation | function_definition ;

create_artifact  = ( CREATE | MAKE ) [ ARTICLE ] DOMAIN_ARTIFACT
                   [ CALLED phrase | FOR [ ARTICLE ] phrase ] ["."] ;
add_element      = ADD [ ARTICLE ] DOMAIN_ELEMENT [ CALLED phrase ] ["."] ;
set_property     = SET [ ARTICLE ] DOMAIN_PROPERTY OF reference TO expression ["."] ;
reference        = PRONOUN | [ ARTICLE ] DOMAIN_ELEMENT CALLED phrase ;

let_binding      = LET identifier BE expression ["."] ;
function_definition = DEFINE FUNCTION identifier TAKING parameter { "," parameter } TO expression ["."] ;
parameter        = identifier AS function_type ;
function_type    = NUMBER | STRING | BOOLEAN | COLOR | LIST OF function_type ;
if_statement     = ( IF | UNLESS ) expression ":" block { OTHERWISE IF expression ":" block }
                 [ OTHERWISE ":" block ] END ["."] ;
for_each         = FOR EVERY identifier IN expression ":" block END ["."] ;
block            = statement+ ;

expression       = logical_or ;
logical_or       = logical_and { OR logical_and } ;
logical_and      = logical_not { AND logical_not } ;
logical_not      = [ NOT ] (logical_not | comparison) ;
comparison       = additive [ IS comparison_operator additive
                 | CONTAINS additive | AT additive ] ;
comparison_operator = GREATER THAN [ OR EQUAL TO ]
                    | LESS THAN [ OR EQUAL TO ]
                    | EQUAL TO | NOT EQUAL TO ;
additive         = multiplicative { (PLUS | MINUS) multiplicative } ;
multiplicative   = numeric_unary { TIMES numeric_unary | DIVIDED BY numeric_unary | MODULO numeric_unary } ;
numeric_unary    = [ MINUS ] (numeric_unary | primary) ;
primary          = COLOR | STRING | NUMBER | BOOLEAN | list_literal
                  | IF expression THEN expression OTHERWISE expression
                  | REPEAT numeric_unary OF primary
                 | LENGTH OF primary | REVERSE OF primary | SUM OF primary
                 | AVERAGE OF primary | MINIMUM OF primary | MAXIMUM OF primary
                 | MEDIAN OF primary | SORTED OF primary | DISTINCT OF primary
                 | ANY OF primary | ALL OF primary
                 | FIRST OF primary | LAST OF primary
                 | TAKE numeric_unary FROM primary | DROP numeric_unary FROM primary
                 | MAP EVERY identifier IN primary TO expression
                  | COUNT EVERY identifier IN primary IF expression
                 | FILTER EVERY identifier IN primary IF expression
                 | RANGE FROM numeric_unary TO numeric_unary
                  | INVOKE identifier TO primary | INVOKE identifier TO "(" expression { "," expression } ")"
                 | UPPERCASE OF primary | LOWERCASE OF primary
                 | variable_reference | "(" expression ")" ;
list_literal     = [ ARTICLE ] LIST OF expression { "," expression } ;
```

The handwritten recursive-descent parser remains the executable grammar during 0.x.

## Typed lazy selections and bounded list generation

Version 0.27.0 introduces `if CONDITION then VALUE otherwise VALUE` as an expression, distinct from existing statement-level `if CONDITION: ... end.`. Both branches must have the same statically inferred type (`S376`) and the condition must be Boolean (`S375`). The compiler evaluates **only the selected branch**, including under constant folding; branches can return lists and be used inside user-defined pure functions. See [typed conditional expressions](39_CONDITIONAL_EXPRESSIONS.md).

`repeat COUNT of VALUE` constructs a `list<T>` with COUNT copies of a single evaluated value. COUNT must be an integer between 0 and 10,000 inclusive; zero produces a typed empty list. Static nonnumeric counts produce `S377`, while fractional, negative or excess counts produce `S378`. This operation is pure and keeps the `repeat` word contextual. See [typed repetition](40_REPEAT_LISTS.md).

## Typed pure user-defined functions

Version 0.26.0 extends top-level `define function NAME taking PARAM as TYPE to EXPRESSION.` declarations to 1–16 comma-separated typed parameters. Calls accept `invoke NAME to PRIMARY` for one argument, or `invoke NAME to (ARG1, ARG2, ...)` for several. Parameter types are `number`, `string`, `boolean`, `color` or nested `list of TYPE`; return types are inferred by HIR. Functions support 1–16 typed arguments, lexical shadowing and earlier immutable outer bindings, and may invoke earlier definitions. Parenthesize complex arguments such as `invoke square to (2 plus 3)`.

```sema
define function square taking n as number to (n times n).
let numbers be range from 1 to 4.
let squares be map every x in numbers to (invoke square to x).
assert last of squares is equal to 16.
```

Typed `count every element in LIST if PREDICATE` produces an exact integer number of matches without materializing a filtered list. The iterator uses lexical scope and may call a previously declared pure function. Empty lists return zero; invalid inputs and predicates produce `S365`/`S366`, while count overflow would produce `S373`. See [predicate counting](36_COUNT_EVERY.md).

Function bodies are restricted to pure core-language expressions. Calls are type checked before evaluation, receive their own parameter environment and work only on concrete compiler-known values. Recursive/forward calls, more than 16 parameters and arbitrary runtime functions are not supported. `define`, `function` and `invoke` are contextual spellings. For complete semantics and diagnostics see [pure functions](35_PURE_FUNCTIONS.md) and [multi-parameter functions](37_MULTI_PARAMETER_FUNCTIONS.md). `minimum of` and `maximum of` also share the exact mixed i64/f64 comparator used for sorting and median, avoiding large-integer rounding errors; see [precision fix](38_EXACT_MIXED_EXTREMA.md).

## Typed sequence transformations and numeric reductions

Version 0.24.0 adds `map every element in LIST to BODY` and `filter every element in LIST if PREDICATE` as pure, type-checked list expressions. Their iterator variables use lexical scopes and can access immutable enclosing bindings. Mapping infers `list<T> -> list<U>`; filtering requires a boolean predicate and retains `list<T>`. The contextual syntax does not prevent variables named `map` or `filter`. `range from START to END` produces up to 10,000 signed integer elements inclusively, ascending or descending; `range` is also contextual. Parenthesize nontrivial operands. See [map/filter](33_MAP_FILTER.md) and [integer ranges](34_INTEGER_RANGE.md).



Semauri includes pure operations for working with concrete strings and homogeneous lists. `sorted of` returns a new ascending list of numbers, strings or booleans (with precise mixed integer/decimal comparisons), while `distinct of` returns a new homogeneous list without repeated values, keeping the first occurrence. Neither modifies its source. `first of` and `last of` select typed list elements without mutation; `take COUNT from LIST` and `drop COUNT from LIST` return typed prefixes/suffixes with clamped non-negative integer counts. `reverse of` returns a new string or list with its elements in reverse order; original list bindings remain unchanged. String reversal operates on Unicode scalar values rather than grapheme clusters.

Numeric reductions use `sum of`, `average of`, `minimum of`, `maximum of` and `median of` with `list<number>` inputs and return `number`. Non-numeric inputs fail static validation (`S353`); integer sums check overflow and report `S354` rather than silently wrapping.

```sema
let readings be a list of 2, 4, 6.
let reversed be reverse of readings.
let total be sum of readings.
let mean be average of readings.
assert (reversed at 0) is equal to 6.
assert total is equal to 12.
assert mean is equal to 4.
```

These prefix expressions bind to the next primary. Parenthesize larger expressions, for example `sum of (a plus b)`. See [reversal](24_REVERSE.md), [numeric aggregations](25_NUMERIC_AGGREGATIONS.md) and [numeric extrema](26_NUMERIC_EXTREMA.md) for full semantics, including floating-point precision and error cases. See [median](28_MEDIAN.md) for middle-element semantics and precise integer handling, and [Boolean reductions](27_BOOLEAN_REDUCTIONS.md) for `any of` and `all of` on typed `list<boolean>`. See [sorted lists](29_SORTED.md) and [distinct lists](30_DISTINCT.md) for typed transformations, stable ordering and representation-aware equality. See [first/last](31_LIST_BOUNDARIES.md) and [take/drop](32_LIST_WINDOWS.md) for static element selection and immutable list slicing.

## Unicode string transformations

Semauri supports **pure, typed** prefix expressions for case mapping. Both return `string` and require a `string` operand; other input types are rejected during type checking (`S351`).

```sema
let name be "Semauri".
let shouted be uppercase of name.
assert lowercase of shouted is equal to "semauri".
print shouted.
```

These operations follow Rust Unicode case mapping rather than language-specific locale rules. Use parentheses to transform larger expressions, for example `uppercase of ("hello" plus " world")`. See [typed string case expressions](22_STRING_CASE.md).

## Comments

Comments are ignored outside string literals:

```text
# Original line comments remain supported.
// A second line-comment style is available.
let amount be 10. /* a block comment
   /* may contain nested comments */
   and continue across lines. */
print amount.
```

Block comments can nest. An unclosed block comment is a lexical error (`S105`) at the opening comment location. Comment markers inside quoted strings remain literal text.

## Semantic domains

`DOMAIN_ARTIFACT`, `DOMAIN_ELEMENT` and `DOMAIN_PROPERTY` are generic lexical categories. Their vocabulary comes from the compiler's registered semantic domains rather than from hardcoded parser productions.

The built-in Web domain currently contributes:

```text
artifacts: web, website, webpage, page → web
elements:  button → button
           image, picture → image
property:  color → color : color
```

A domain term carries explicit semantic metadata through AST and HIR:

```text
button
  ↓
DOMAIN_ELEMENT(domain=web, kind=button)
```

A domain defines the semantic type accepted by each property. For example, Web's `color` property accepts a `color` value. A mismatch is `S313`.

Operations from different domains cannot be silently mixed. Applying a property/reference from one domain to an artifact from another is `S327`.

Surface terms are globally unambiguous within one compiler instance. Registering two domains that claim the same term is rejected when constructing the domain registry.

### Implicit property syntax

The shorthand:

```text
make it blue.
```

omits the property name. Semauri accepts this only when the loaded domain registry can infer exactly one property compatible with the literal type. With the default Web domain, `blue : color` uniquely implies the `color` property.

If multiple loaded domains could interpret the same value type, the shorthand is rejected and explicit `set ...` syntax is required.

## Values and expressions

Primitive semantic value types are `color`, `string`, `number`, and `boolean`.

### Numeric literals

Numbers may use a decimal fraction, a scientific exponent (`e` or `E` with an optional `+` or `-` sign) and underscores between digits for readability:

```text
let population be 1_000_000.
let rate be 2.5e-3.
let distance be 1_200.5E+2.
```

Integers must fit into signed 64-bit range; decimal and scientific literals must parse to finite floating-point values. Invalid separators, incomplete exponents and out-of-range literals are lexical errors (`S106`), never compiler panics. A trailing period still terminates a statement unless followed by a fractional digit.

`plus` is overloaded deliberately: it adds two numbers, concatenates two strings, or creates a new list when both operands are lists with the same element type. It never coerces between strings, numbers, and lists. The remaining arithmetic operators are numeric and strictly typed. Prefix `minus` negates a number and binds more tightly than multiplication/division/modulo, which bind more tightly than addition/subtraction. `modulo` returns the numeric remainder and, like division, rejects a zero divisor with `S317`. Equality and inequality (`is equal to` / `is not equal to`) require both operands to have the same semantic type.

Ordering comparisons are numeric and strictly typed. Semauri supports `is greater than`, `is less than`, `is greater than or equal to`, and `is less than or equal to`. The inclusive forms include the boundary value and lower to distinct typed comparison operators rather than being rewritten as boolean combinations.

### Logical expressions and short-circuiting

`and`, `or` and `not` operate on booleans. `and` / `or` use runtime/constant-evaluation short-circuiting: the right-hand expression is not **evaluated** when the left-hand value already determines the result.

However, name resolution and type checking happen earlier while building Typed HIR. Therefore every referenced name must exist and every subexpression must type-check, even if its runtime value would be short-circuited.

Valid short-circuit example:

```text
if true or 1 divided by 0 is equal to 1:
  add a button called Safe.
end.
```

The division expression is well-formed and typed, but it is never evaluated, so no division-by-zero error occurs.

This is still invalid:

```text
if true or missingVariable is equal to true:
  add a button called Invalid.
end.
```

`missingVariable` fails name resolution with `S312` before evaluation begins.

### Unicode string escapes

Double-quoted strings support `\n`, `\t`, `\r`, `\"` and `\\`, plus a Unicode scalar escape `\u{HEX}` where `HEX` contains 1 to 6 hexadecimal digits. For example:

```text
let greeting be "Bonjour \u{1F30D}".
```

The lexer decodes the escape before type checking. Invalid syntax, out-of-range values (above U+10FFFF) and surrogate code points (U+D800–U+DFFF) are rejected with `S104`. Unescaped carriage returns and line feeds are rejected with `S102`. This keeps source parsing deterministic on all platforms.

## Collections

Semauri supports homogeneous list literals:

```text
let prices be a list of 10, 20, 30.
let colors be a list of red, green, blue.
```

Their types are `list<number>` and `list<color>`. Mixed lists are rejected with `S321`. There is no implicit union/coercion rule.

The typed collection operators in v0.17.0 include `length of` for strings and lists, `contains` for string substrings and list membership, zero-based `list at index`, and immutable same-type list concatenation using `plus`. They do not enable iteration over arbitrary values produced at runtime. Details: [length](13_LENGTH_EXPRESSIONS.md), [membership](14_CONTAINS.md), [indexing](16_LIST_INDEXING.md), and [concatenation](17_LIST_CONCATENATION.md).

## Assertions and tests

The registered console operation `assert boolean_expression.` checks a Boolean at execution; false raises a source-located `S502` runtime error. The pure assertion itself requires no effect permission. `semauri test PATH` discovers assertion-based `.sema` test files and refuses non-console effects. A program requiring `print` needs `--allow-console` to run under the test command. This is not a general-purpose testing runtime; see [assertions](12_ASSERTIONS.md) and [test runner](15_TEST_RUNNER.md).

## Variables and lexical scope

`Let` creates immutable bindings. Blocks create child lexical scopes, parent bindings are visible inside child scopes, and child bindings can shadow parent names without mutating them.

Bindings have stable semantic symbol identities independent from source spelling. Typed HIR references symbols by ID rather than repeating string lookup.

## Conditional control flow

```text
if price is greater than or equal to 20:
  add a button called Premium.
otherwise:
  add a button called Standard.
end.
```

The condition must be boolean (`S316`). Typed HIR preserves both branches. The current HIR lowerer selects a branch statically because all current values are compile-time-known.

### Unless

`unless condition:` is shorthand for `if not condition:`. Its condition is type-checked as a boolean and lowered into the existing unary `not` expression, so it works with the same `Otherwise` / `otherwise if` clauses and closes with one `End.`:

```text
unless score is greater than or equal to 70:
  add a button called Below.
otherwise:
  add a button called Passed.
end.
```

A non-boolean operand to `Unless` reports `S319`, the same diagnostic as `not` on a non-boolean value.

### Chained conditionals

Use `otherwise if` (or `Else if`) to test multiple conditions in order. The first true branch wins, `Otherwise:` is optional and must be last, and **the whole chain uses a single `End.`**:

```text
if score is greater than or equal to 90:
  add a button called Excellent.
otherwise if score is greater than or equal to 70:
  add a button called Passed.
otherwise:
  add a button called Retry.
end.
```

The parser represents subsequent clauses as nested `if_statement` nodes inside the prior branch's alternative block. Each branch preserves its source span and lexical scope. Conditions in *all* clauses are type-checked, including unreachable clauses, and invalid conditions still report `S316`. Missing colons and empty clauses retain `S218` and `S221`, respectively. See [`examples/otherwise_if.sema`](../examples/otherwise_if.sema).

The symbol table describes the complete typed program, including declarations inside a statically unselected branch. Branch elimination belongs to a later optimization/lowering phase, not name resolution.

## Static iteration

```text
let colors be a list of red, green, blue.

for every accent in colors:
  set the color of the button called Buy to accent.
end.
```

`for every` requires a list (`S322`). The iteration variable has the list's element type and is visible only inside the loop body.

The iterator is one semantic declaration with one symbol ID. Static execution binds different values to that same symbol across iterations; it does not create a new declaration for each item.

HIR preserves `for_each` structurally. The current lowerer iterates compile-time-known lists. Runtime/external collections will require a later control-flow IR.

## References

`it` resolves only when exactly one addressable entity exists. Explicit references such as `the button called Buy` are required when a pronoun is ambiguous. Semauri never chooses a referent probabilistically.

Named references carry domain identity. The active artifact's domain is checked before entity lookup.

## Errors

- `S1xx`: lexical errors
- `S2xx`: parse errors
- `S3xx`: semantic/HIR errors
- `S4xx`: backend errors

Relevant diagnostics include:

- `S311`: duplicate binding
- `S312`: unknown variable during name resolution
- `S313`: domain property/value type mismatch
- `S314`: invalid numeric operands
- `S315`: incompatible equality operands
- `S316`: non-boolean `If` condition
- `S317`: zero divisor during division or modulo evaluation
- `S318`: unsupported expression operator
- `S319`: invalid logical operands
- `S320`: list element type cannot be inferred
- `S321`: heterogeneous list
- `S322`: iteration over a non-list value
- `S323`: internal symbol/value binding type mismatch
- `S324`: invalid HIR symbol binding/value environment
- `S325`: unsupported HIR node/operator
- `S326`: unknown semantic domain
- `S327`: cross-domain operation on the active artifact
- `S359`: invalid input to `sorted of` or non-finite sorted numeric value
- `S360`: `distinct of` requires a typed list
- `S361`: `first of` and `last of` require a list
- `S362`: `first of` or `last of` evaluated on an empty typed list
- `S363`: `take` / `drop` require a number count and typed list
- `S364`: negative or fractional count for `take` / `drop`
- `S365`: map/filter requires a list input
- `S366`: invalid filter predicate or unit-valued map body
- `S367`: range endpoints are not numeric
- `S368`: fractional range endpoint or more than 10,000 generated elements
- `S369`: function not previously declared
- `S370`: function argument does not match parameter type
- `S371`: duplicate function name
- `S372`: invalid function scope or impure function body
- `S373`: `count every` matched element count exceeds signed 64-bit range
- `S374`: incorrect arity when invoking a typed user-defined function
- `S375`: conditional expression condition is not boolean
- `S376`: conditional expression branches have different types
- `S377`: repeat count is not numeric or repeated element has no value
- `S378`: negative, fractional, or greater-than-10,000 repeat count

## Determinism rule

Given the same Semauri version, source program, loaded semantic domains, selected backend and backend version, valid source must resolve to the same semantic result. No compiler component may randomly choose between ambiguous meanings.
