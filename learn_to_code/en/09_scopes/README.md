# 09 · Scopes and names

**Goal:** Understand where a variable is visible and how shadowing works.

## Concept

Blocks create lexical scopes. A `Let` inside `If` exists only inside that block. A local binding can **shadow** an outer one without changing it. Using an inner-only binding outside the block is a compile-time error.

## Example program

Open [`main.sema`](./main.sema):

```text
let message be "outside".

if true:
  let message be "inside".
  print message.
end.

print message.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/09_scopes/main.sema
semauri run --allow console_write learn_to_code/en/09_scopes/main.sema
```

**Expected console output:**

```text
inside
outside
```

## Your challenge

Declare `let secret be 7.` inside the block, try printing it after `End.` and fix the scope error.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/09_ambitos/README.md)
