# Scoped Boolean quantifiers — `any every`, `all every`

`any every ITEM in LIST if PREDICATE` and `all every ITEM in LIST if PREDICATE` perform **pure, typed Boolean reductions** on a concrete homogeneous list. They return `boolean`, without allocating a filtered list. The predicate is type-checked as `boolean` in a fresh lexical iterator scope and evaluated in input order.

```sema
let values be a list of 2, 4, 6.
let any_large be any every n in values if (n is greater than 5).
let all_even be all every n in values if ((n modulo 2) is equal to 0).
```

**Short circuit:** `any every` returns true on the first match and evaluates no later elements. `all every` returns false on the first failure and evaluates no later elements. On an empty typed list, `any every` returns false and `all every` returns true (vacuous truth). This differs from `any of LIST` / `all of LIST` which require a `list<boolean>` but have no per-element predicate.

Non-list input raises located `S365`; non-Boolean predicate `S366`. Function calls, outer captured immutable bindings, list operations, and conditional expressions are supported inside predicates. No new effects or runtime ML execution are introduced.

```sh
semauri check examples/quantified_predicates.sema
semauri run --allow console_write examples/quantified_predicates.sema
```
