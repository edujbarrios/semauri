# 07 · Lists of values

**Goal:** Keep multiple values of the same type in one list.

## Concept

`a list of ...` creates a homogeneous list: all elements share the same type. You can print the full list, but this experimental version has no index-based lookup.

## Example program

Open [`main.sema`](./main.sema):

```text
Let scores be a list of 10, 20, 30.
Let colors be a list of red, green, blue.

Print scores.
Print colors.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/07_lists/main.sema
semauri run --allow console_write learn_to_code/en/07_lists/main.sema
```

**Expected console output:**

```text
[10, 20, 30]
[red, green, blue]
```

## Your challenge

Make a list of three strings. Then mix strings and numbers intentionally and inspect the type error.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/07_listas/README.md)
