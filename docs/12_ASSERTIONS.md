# Assertions

Semauri supports a typed runtime assertion statement:

```sema
let expected be 6 times 7.
assert expected is equal to 42.
assert not false.
print "All checks passed".
```

Run it with permission for the **print** operation:

```sh
semauri run --allow console_write examples/assertions.suri
```

`Assert` accepts **boolean expressions only**. A non-boolean argument is rejected at compile time (`S329`). A false assertion is an execution failure (`S502`, exit status 70) with a source span. Successful assertions produce no output and require no external capability.

Assertions are not evaluated while compiling: `semauri check` validates the expression and type without executing it. `semauri run` evaluates assertions as part of the ordered console runtime plan. These assertions currently work with compile-time-resolvable values and conditions; dynamic results from external domain operations are not yet supported by the console executor.

This feature is a foundation for future test-runner conventions (#65) and explicit validation workflows (#62). It does not implement exceptions, recovery, or a dynamic control-flow runtime.
