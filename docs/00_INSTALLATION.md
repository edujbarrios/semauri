# Installing and running Semauri

This guide covers binary installation, manual downloads, running <code>.sema</code> programs and installing an exact version.

> Current release documented here: **0.14.0**

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
SEMAURI_VERSION=0.14.0 \
  curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
~~~

Windows PowerShell:

~~~powershell
$env:SEMAURI_VERSION = '0.14.0'
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
Remove-Item Env:SEMAURI_VERSION
~~~

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

Use <code>--cwd PATH</code> to choose the working directory for relative filesystem paths.

## Updating

Re-run the installer without <code>SEMAURI_VERSION</code> to resolve and activate the latest GitHub Release.

## Building from source

If you want to **change the language**, do not use the binary installation as your development environment. Clone the source repository and follow [DEVELOPMENT.md](DEVELOPMENT.md).

For package layout, checksums and release automation, see [09_DISTRIBUTION.md](09_DISTRIBUTION.md).
