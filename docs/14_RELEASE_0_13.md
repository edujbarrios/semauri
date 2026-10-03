# Semauri 0.13.0

0.13 focuses on making ordinary Semauri source files more practical to maintain.

## Language changes

- `#` starts a line comment outside strings. Comments may occupy a whole line or follow a statement.
- String literals now decode `\n`, `\t`, `\r`, `\"` and `\\`.
- Unknown string escapes fail with diagnostic `S104` rather than producing ambiguous text.
- Literal physical newlines remain disallowed inside quoted strings; use `\n` so source layout stays deterministic.

## Quality

- Lexer regression coverage now includes comments, valid escapes and invalid escapes.
- README capability/status documentation is synchronized with 0.13.0.

## Repository hygiene

The 0.13 cleanup deliberately avoids deleting historical release notes, examples, packaging launchers or tests merely because they look duplicated: each is still part of documentation, distribution or regression coverage. Cleanup is limited to artifacts that can be proven generated/stale so repository history remains useful.
