# 06 · More decisions: Unless and Otherwise if

**Goal:** Use a chain of conditions without nested If blocks.

## Concept

`unless condition:` is short for `if not condition:`. `otherwise if` (also `Else if`) evaluates the next condition if earlier branches did not match. The whole chain has one `End.`.

## Example program

Open [`main.suri`](./main.suri):

```text
let score be 82.

unless score is greater than or equal to 50:
  print "Try again".
otherwise if score is greater than or equal to 90:
  print "Excellent".
otherwise if score is greater than or equal to 70:
  print "Great".
otherwise:
  print "Passed".
end.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/06_unless_else_if/main.suri
semauri run --allow console_write learn_to_code/en/06_unless_else_if/main.suri
```

**Expected console output:**

```text
Great
```

## Your challenge

Try the values 30, 65, 75 and 95 and predict the message for each.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/06_unless_y_else_if/README.md)
