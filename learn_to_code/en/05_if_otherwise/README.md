# 05 · Decisions: If and Otherwise

**Goal:** Choose between different blocks based on a condition.

## Concept

`If condition:` starts a block; `Otherwise:` provides an alternative; `End.` closes the conditional. The condition must be boolean. In these examples, the compiler can determine the result from values known at compile time.

## Example program

Open [`main.sema`](./main.sema):

```text
Let grade be 7.

If grade is greater than or equal to 5:
  Print "Passed".
Otherwise:
  Print "Keep practicing".
End.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/05_if_otherwise/main.sema
semauri run --allow console_write learn_to_code/en/05_if_otherwise/main.sema
```

**Expected console output:**

```text
Passed
```

## Your challenge

Change the grade to 3. Then combine two comparisons with `and`.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/05_if_otherwise/README.md)
