# 10 · Build a web page

**Goal:** Generate HTML using a declarative semantic domain.

## Concept

`semauri build` compiles Web-domain statements into HTML. `Create a web`, `Add a button` and `Set the color ...` describe the document. Use **`build` instead of `run`**: generating the output has no external effects.

## Example program

Open [`main.sema`](./main.sema):

```text
Create a web called My First Page.
Add a button called Start.
Set the color of the button called Start to blue.
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/10_build_a_website/main.sema
semauri build learn_to_code/en/10_build_a_website/main.sema
```

**Expected result:** HTML with a blue Start button. The generated content appears in the output of `build`.

## Your challenge

Add another button called Next in green and inspect the generated `<button>` tags.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/10_crea_una_web/README.md)
