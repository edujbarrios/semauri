# Typed `contains` comparisons

Use `contains` to test whether a string includes a substring, or whether a typed list includes an item:

```sema
Let names be a list of "Ada", "Grace".
Assert names contains "Ada".
If "Semauri" contains "mau":
  Print "found".
End.
```

`contains` is a binary comparison operator, binds more tightly than `and` and `or`, and returns `boolean`. For `string contains string`, matches are case-sensitive literal Unicode substrings (empty needle matches). For `list<T> contains T`, the element type must match exactly and membership uses value equality. No implicit string-to-number coercion takes place. Invalid type combinations fail with `S344` during type checking.

Known literal string comparisons may be constant-folded. List comparisons evaluate through the regular compile-time interpreter, and runtime-produced lists are not yet supported in core expressions. `contains` has no external effects and does not bypass capability policies.
