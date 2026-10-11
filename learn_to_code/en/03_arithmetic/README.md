# 03 · Arithmetic and precedence

**Goal:** Use expressions, arithmetic operators and parentheses.

## Concept

Semauri supports `plus`, `minus`, `times`, `divided by` and `modulo` (remainder). Multiplication has higher precedence than addition; parentheses change the order. Division by zero is rejected.

## Example program

Open [`main.sema`](./main.sema):

```text
Let subtotal be 10 plus 5 times 2.
Let grouped be (10 plus 5) times 2.
Let remainder be 17 modulo 3.

Print subtotal.
Print grouped.
Print remainder.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/03_arithmetic/main.sema
semauri run --allow console_write learn_to_code/en/03_arithmetic/main.sema
```

**Expected console output:**

```text
20
30
2
```

## Your challenge

Calculate the cost of three items priced at 8 each plus a shipping fee of 5.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/03_aritmetica/README.md)
