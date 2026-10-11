# 01 · Your first program

**Goal:** Understand statements and print text or numbers in the console.

## Concept

Each statement ends with a period. `Print` displays a value. Programs are saved as `.sema` files, and a line starting with `#` is a comment.

## Example program

Open [`main.sema`](./main.sema):

```text
# A comment explains the program.
Print "Hello from Semauri".
Print 2026.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/01_hello_world/main.sema
semauri run --allow console_write learn_to_code/en/01_hello_world/main.sema
```

**Expected console output:**

```text
Hello from Semauri
2026
```

## Your challenge

Replace the greeting with your name and add another statement that prints your favourite number.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/01_hola_mundo/README.md)
