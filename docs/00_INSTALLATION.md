# Install Semauri on Windows, macOS or Linux

Semauri is a compiled programming language with its own native command-line tool. This guide covers the official installers, manual downloads, troubleshooting and version pinning.

**New to terminals or programming?** Begin with [your first Semauri program](23_FIRST_PROGRAM.md), which explains opening PowerShell or Terminal, creating a project folder, saving a `.sema` file, checking it and running it.

> Current stable release documented here: **0.30.0**

Official Semauri distributions contain a native executable. **Rust and Cargo are not required to use a published Semauri binary.**

## Quick install

### Linux and macOS

~~~bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
~~~

The installer detects Linux/macOS and x86_64/arm64, downloads the matching GitHub Release asset, verifies its SHA-256 checksum and exposes <code>semauri</code> through <code>~/.local/bin</code> by default.

Verify:

~~~bash
semauri version
semauri help
~~~

If <code>~/.local/bin</code> is not in <code>PATH</code>, add it to your shell configuration.

### Windows PowerShell

~~~powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
~~~

The installer selects Windows x86_64 or arm64, verifies the release checksum, stores versioned binaries under <code>$HOME\.semauri\versions\</code> and adds <code>$HOME\.semauri\bin</code> to the user <code>PATH</code>.

Open a new terminal if necessary, then verify:

~~~powershell
semauri version
semauri help
~~~

## Manual download

All native archives are published at:

https://github.com/edujbarrios/semauri/releases/latest

Choose the asset matching your machine:

| Platform | Asset pattern |
| --- | --- |
| Linux x86_64 | <code>semauri-&lt;VERSION&gt;-linux-x86_64.tar.gz</code> |
| Linux arm64 | <code>semauri-&lt;VERSION&gt;-linux-arm64.tar.gz</code> |
| macOS Intel | <code>semauri-&lt;VERSION&gt;-macos-x86_64.tar.gz</code> |
| macOS Apple Silicon | <code>semauri-&lt;VERSION&gt;-macos-arm64.tar.gz</code> |
| Windows x86_64 | <code>semauri-&lt;VERSION&gt;-windows-x86_64.zip</code> |
| Windows arm64 | <code>semauri-&lt;VERSION&gt;-windows-arm64.zip</code> |

Download <code>SHA256SUMS</code> from the same release and verify the archive before extracting it. The executable is inside <code>bin/</code> in the extracted directory.

On Linux/macOS:

~~~bash
./semauri-<VERSION>-<PLATFORM>/bin/semauri version
~~~

On Windows PowerShell:

~~~powershell
.\semauri-<VERSION>-windows-x86_64\bin\semauri.exe version
~~~

## Install an exact version

Linux/macOS:

~~~bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | SEMAURI_VERSION=0.30.0 sh
~~~

Windows PowerShell:

~~~powershell
$env:SEMAURI_VERSION = '0.30.0'
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
Remove-Item Env:SEMAURI_VERSION
~~~

## Learn programming with the bundled course

All six official native archives include the full [bilingual Learn to Code curriculum](../learn_to_code/README.md): **11 lessons in English** ([`en/`](../learn_to_code/en/README.md)) and **11 in Spanish** ([`es/`](../learn_to_code/es/README.md)), each with a runnable `.sema` example. From the extracted archive root, try:

```sh
semauri check learn_to_code/en/01_hello_world/main.sema
semauri run --allow console_write learn_to_code/en/01_hello_world/main.sema
semauri check learn_to_code/es/01_hola_mundo/main.sema
semauri run --allow console_write learn_to_code/es/01_hola_mundo/main.sema
```

Use the executable under `bin/` if `semauri` is not yet on your PATH.

## Running Semauri source files

Semauri source files use the <code>.sema</code> extension.

Validate a file without producing an output:

~~~bash
semauri check examples/hello.sema
~~~

Compile a file to its domain-specific output:

~~~bash
semauri build examples/shop.sema
~~~

Inspect tokens, AST, HIR, effects or runtime planning:

~~~bash
semauri tokens examples/shop.sema
semauri ast examples/shop.sema
semauri hir examples/shop.sema
semauri effects examples/filesystem.sema
semauri plan examples/filesystem.sema
~~~

Effectful execution is explicit. The current execution path supports authorized filesystem workflows:

~~~bash
semauri run --allow filesystem_read --allow filesystem_write examples/filesystem.sema
~~~

Inspect the plan without performing effects:

~~~bash
semauri run --dry-run examples/filesystem.sema
~~~

Use <code>--cwd PATH</code> to choose the working directory for relative filesystem paths. This is a convenience option, **not** a security sandbox; execute filesystem programs only with appropriate trust and permissions.

## Troubleshooting

### `semauri` is not found after installation

1. **Close and reopen the terminal** first. Windows `PATH` changes do not automatically propagate to terminals already open.
2. Verify that the executable exists at the installer's default location:

   On **Windows PowerShell**:

   ```powershell
   Test-Path "$HOME\.semauri\bin\semauri.exe"
   & "$HOME\.semauri\bin\semauri.exe" version
   ```

   On **Linux/macOS**:

   ```sh
   ls -l "$HOME/.local/bin/semauri"
   "$HOME/.local/bin/semauri" version
   ```

3. If the absolute executable works but `semauri version` does not, the problem is the terminal's `PATH`, **not the compiler**.

   In **PowerShell**, restart the terminal first; the Windows installer attempts to add `$HOME\.semauri\bin` to the current user's `PATH`.

   On **Linux/macOS**, add the install directory to `PATH` temporarily:

   ```sh
   export PATH="$HOME/.local/bin:$PATH"
   semauri version
   ```

   To persist it, add the `export PATH=...` line to your shell startup file, such as `~/.bashrc` (Bash) or `~/.zshrc` (Zsh). Reopen the terminal.

### The program file cannot be found

Ensure the file exists in your **current** directory. Use `pwd` and `ls` on Linux/macOS; use `Get-Location` and `dir` in PowerShell. You can also pass the full path to `hello.sema`.

On Windows, verify Notepad did not append `.txt` to your file name.

### Permission denied or checksum mismatch

Do not disable checksum verification. Use the official [GitHub Releases](https://github.com/edujbarrios/semauri/releases/latest) page, choose the archive for your platform and verify `SHA256SUMS`. For executable permissions on Unix, ensure the extracted binary is executable. If installation fails, see [manual download](#manual-download).

### `semauri run` reports a missing capability

Running effectful operations requires explicit permission. For printing, use `semauri run --allow console_write hello.sema`. For file operations, see [runtime capabilities](05_RUNTIME.md). `check` and `--dry-run` let you inspect programs before granting permissions.

## Updating

Re-run the installer without <code>SEMAURI_VERSION</code> to resolve and activate the latest GitHub Release.

## Building from source

If you want to **change the language**, do not use the binary installation as your development environment. Clone the source repository and follow [DEVELOPMENT.md](DEVELOPMENT.md).

For package layout, checksums and release automation, see [09_DISTRIBUTION.md](09_DISTRIBUTION.md).
