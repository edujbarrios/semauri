# 11 · Structured data and JSON Schema

**Goal:** Define the shape of structured data.

## Concept

The Structured Data domain offers `create a schema`, `add a field` and typed properties. `semauri build` produces JSON Schema. A schema describes valid fields; it does not store records or write to the filesystem.

## Example program

Open [`main.suri`](./main.suri):

```text
create a schema called Student.
add a field called Name.
set the datatype of the field called Name to "string".
set the required of the field called Name to true.
add a field called Age.
set the datatype of the field called Age to "integer".
```

## Try it

Run these commands from the repository root (or the extracted package root):

```sh
semauri check learn_to_code/en/11_schemas/main.suri
semauri build learn_to_code/en/11_schemas/main.suri
```

**Expected result:** JSON Schema with required Name string and Age integer. The generated content appears in the output of `build`.

## Your challenge

Add a field called Course, give it the `string` datatype, mark it required and inspect the `required` array.

Run `check` again after making changes. If you get a diagnostic, use its source location to investigate one change at a time.

[← Back to the English course](../README.md) · [Versión en español](../../es/11_esquemas/README.md)
