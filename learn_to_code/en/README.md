# Learn to Code with Semauri

**Language:** English · [Español](../es/README.md) · [Choose a language](../README.md)

A **free, beginner-friendly course** for learning to program with Semauri **0.16.1**. No previous programming experience is needed. Semauri uses a **controlled natural-language grammar with English keywords**, not unrestricted English or pseudocode.

## Get started

1. [Install Semauri](../../docs/00_INSTALLATION.md) and confirm with `semauri version`. With Rust installed, contributors can also use `cargo run --` from the repository root.
2. Clone the repo or extract the official release archive. Both include `learn_to_code/en/` and `learn_to_code/es/`.
3. Run the following commands from the **repository or extracted package root**:

```sh
semauri check learn_to_code/en/01_hello_world/main.suri
semauri run --dry-run learn_to_code/en/01_hello_world/main.suri
semauri run --allow console_write learn_to_code/en/01_hello_world/main.suri
```

**Important:** `check` validates without execution; `run --dry-run` shows the plan without side effects; `run --allow console_write` explicitly permits printing. Lessons 10 and 11 use `build` to generate documents instead of executing operations.

## Eleven lessons, in order

| # | Lesson | What you learn | Command |
| --- | --- | --- | --- |
| 1 | [Your first program](./01_hello_world/README.md) | Understand statements and print text or numbers in the console. | `run` |
| 2 | [Variables and types](./02_variables_and_types/README.md) | Store data under names and identify strings, numbers and booleans. | `run` |
| 3 | [Arithmetic and precedence](./03_arithmetic/README.md) | Use expressions, arithmetic operators and parentheses. | `run` |
| 4 | [Booleans and comparisons](./04_booleans/README.md) | Make comparisons that return true or false. | `run` |
| 5 | [Decisions: If and Otherwise](./05_if_otherwise/README.md) | Choose between different blocks based on a condition. | `run` |
| 6 | [More decisions: Unless and Otherwise if](./06_unless_else_if/README.md) | Use a chain of conditions without nested If blocks. | `run` |
| 7 | [Lists of values](./07_lists/README.md) | Keep multiple values of the same type in one list. | `run` |
| 8 | [Loops with For every](./08_loops/README.md) | Repeat instructions for the elements of a list. | `run` |
| 9 | [Scopes and names](./09_scopes/README.md) | Understand where a variable is visible and how shadowing works. | `run` |
| 10 | [Build a web page](./10_build_a_website/README.md) | Generate HTML using a declarative semantic domain. | `build` |
| 11 | [Structured data and JSON Schema](./11_schemas/README.md) | Define the shape of structured data. | `build` |

Each folder includes an introductory README with a concept, an actual `main.suri` program, its expected result and an exercise. No network connection is needed to try the lessons.

## Useful commands

```sh
# Run a console lesson
semauri check learn_to_code/en/05_if_otherwise/main.suri
semauri run --allow console_write learn_to_code/en/05_if_otherwise/main.suri

# Generate HTML and JSON Schema
semauri build learn_to_code/en/10_build_a_website/main.suri
semauri build learn_to_code/en/11_schemas/main.suri

# Inspect the compiler's explanation
semauri explain learn_to_code/en/05_if_otherwise/main.suri
```

Even without enabling effects, you can learn by using `check`, `tokens`, `ast`, `hir`, `explain` and `build`. Diagnostic `S312` means an unknown variable; `S316` means a non-boolean condition. See the [language reference](../../docs/01_LANGUAGE.md).

## What this course does not cover (yet)

Semauri is **experimental**. It does not yet provide user-defined functions, interactive input, or loops over runtime-dependent collections. You cannot paste arbitrary Python or JavaScript into Semauri.

## Keep learning

Explore the [reference examples](../../examples/), [types](../../docs/02_TYPES.md), [developer guide](../../docs/DEVELOPMENT.md) and [contributing instructions](../../CONTRIBUTING.md).

**Quality assurance:** all 22 `main.suri` examples are covered by `tests/learning_course.rs` and included in the six official release packages.
