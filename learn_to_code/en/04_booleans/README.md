# 04 · Booleans and comparisons

**Goal:** Make comparisons that return true or false.

## Concept

`is greater than`, `is less than`, `is equal to`, `and`, `or` and `not` construct boolean expressions. `and` requires both sides to be true; `or` needs one; `not` reverses the result.

## Example program

Open [`main.sema`](./main.sema):

```text
Let age be 20.
Let adult be age is greater than or equal to 18.
Let allowed be adult and not false.

Print adult.
Print allowed.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/04_booleans/main.sema
semauri run --allow console_write learn_to_code/en/04_booleans/main.sema
```

**Expected console output:**

```text
true
true
```

## Your challenge

Change the age to 16, predict the output and write an additional comparison using `is less than`.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/04_booleanos/README.md)
