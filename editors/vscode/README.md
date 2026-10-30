# Semauri language support

This extension provides TextMate syntax highlighting, comments, brackets, and folding for `.suri` source and `.config.suri` configuration files.

For local development, copy this directory into your VS Code extensions directory and restart VS Code, or package it with `npx @vscode/vsce package`.

The grammar is Apache-2.0 licensed with the Semauri repository and uses the stable scope `source.semauri`. It is also the upstream grammar intended for GitHub Linguist once public usage meets Linguist's language-admission threshold.

For Semauri 0.52, the grammar highlights binary (`0b1010`), octal (`0o755`), and hexadecimal (`0xFF`) integer literals, as well as single- and double-quoted strings with escapes. The Rust compiler remains the source of truth for syntax validity.

For Semauri 0.53, the embedded grammar also recognizes raw strings (`r"..."` and `r'...'`) and the `\\0`, `\\b`, and `\\f` escape sequences in ordinary strings. Raw strings deliberately do not interpret escape sequences.
