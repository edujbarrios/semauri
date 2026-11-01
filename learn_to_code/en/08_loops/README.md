# 08 · Loops with For every

**Goal:** Repeat instructions for the elements of a list.

## Concept

`for every item in list:` introduces a loop. The variable `item` is in scope inside the block. These loops are **static** and expand over collections known at compile time, rather than arbitrary runtime input.

## Example program

Open [`main.suri`](./main.suri):

```text
let levels be a list of 1, 2, 3.

for every level in levels:
  print level.
end.

print "Done".
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/08_loops/main.suri
semauri run --allow console_write learn_to_code/en/08_loops/main.suri
```

**Expected console output:**

```text
1
2
3
Done
```

## Your challenge

Replace the list with other numbers and print `level times 2` inside the loop.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/08_bucles/README.md)
