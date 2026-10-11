# 02 · Variables and types

**Goal:** Store data under names and identify strings, numbers and booleans.

## Concept

`Let name be value.` creates an immutable binding. Strings are enclosed in quotes, numbers are not, and `true` / `false` are booleans. Reassigning an immutable binding is not allowed.

## Example program

Open [`main.sema`](./main.sema):

```text
Let name be "Alex".
Let age be 18.
Let learning be true.

Print name.
Print age.
Print learning.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/02_variables_and_types/main.sema
semauri run --allow console_write learn_to_code/en/02_variables_and_types/main.sema
```

**Expected console output:**

```text
Alex
18
true
```

## Your challenge

Add `Let city be "London".` and print the value. What is the difference between `18` and `"18"`?

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/02_variables_y_tipos/README.md)
