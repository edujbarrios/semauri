# Python-friendly function aliases (unreleased)

Semauri retains its natural-language expression form `FUNCTION of OPERAND`.
These shorter names reuse the same statically typed operators and optimizer
paths as their existing counterparts. They do **not** import Python, permit
Python-style function calls with parentheses, or relax typing or permissions.

| Alias | Existing Semauri operator | Input and result |
| --- | --- | --- |
| `len of VALUE` | `length of VALUE` | string Unicode scalar count or typed list length → number |
| `abs of X` | `absolute of X` | numeric absolute value → number; checked overflow |
| `mean of NUMBERS` | `average of NUMBERS` | numeric list mean → number |
| `min of LIST` | `minimum of LIST` | typed list minimum → item |
| `max of LIST` | `maximum of LIST` | typed list maximum → item |
| `strip of TEXT` | `trim of TEXT` | surrounding whitespace removed → string |
| `upper of TEXT` | `uppercase of TEXT` | Unicode uppercase → string |
| `lower of TEXT` | `lowercase of TEXT` | Unicode lowercase → string |

```text
let numbers be a list of 2, 4, 6.
assert mean of numbers is equal to 4.
assert abs of (minus 5) is equal to 5.
assert strip of " hi " is equal to "hi".
```

All eight names are contextual: ordinary variables named `len`, `mean`,
and so on still work when not immediately followed by `of`.
As with existing prefix functions, parenthesize a compound operand.
See [the executable example](../examples/python_function_aliases.suri).

This capability is currently on the `main` development branch, not in an
independently tagged release; the latest published release remains v0.61.0.
