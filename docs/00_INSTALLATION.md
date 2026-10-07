# Installation

This document covers installing and updating Semauri for normal use, plus the source-based workflow for compiler contributors.

> Current release documented here: **0.14.0**

Official Semauri distributions contain a native executable. **You do not need Rust, Cargo, or another language runtime installed to use Semauri.**

## Linux and macOS

```bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Verify the installation:

```text
semauri version
semauri help
```

The installer stores versioned distributions under `~/.semauri/versions/`, switches `~/.semauri/current`, and exposes the command through `~/.local/bin/semauri`.

### Install an exact version

```bash
SEMAURI_VERSION=0.14.0 \
  curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Custom install locations can be selected with `SEMAURI_INSTALL_ROOT` and `SEMAURI_BIN_DIR`.

## Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

Verify the installation in a new terminal if necessary:

```text
semauri version
semauri help
```

The installer stores versions under `$HOME\.semauri\versions\`, maintains the active version through `$HOME\.semauri\current.txt`, installs a stable command shim at `$HOME\.semauri\bin\semauri.cmd`, and launches the versioned native `semauri.exe`.

### Install an exact version

```powershell
$env:SEMAURI_VERSION = '0.14.0'
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
Remove-Item Env:SEMAURI_VERSION
```

The install root can be changed with `SEMAURI_INSTALL_ROOT` or the `-InstallRoot` PowerShell parameter.

## Supported official targets

Semauri publishes native packages for Linux x86_64/arm64, macOS x86_64/arm64, and Windows x86_64/arm64. Linux/macOS packages use `.tar.gz`; Windows packages use `.zip`.

## Updating Semauri

Re-run the installer to resolve and activate the latest release. Older version directories are kept locally so the distribution layout still supports manual rollback.

## Development from source

Compiler contributors need a Rust toolchain (Rust 1.83+):

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
cargo run -- help
cargo test --all-targets
```

A release build is produced with:

```bash
cargo build --release
./target/release/semauri version
```

For package layouts, checksums and the release process, continue with [`09_DISTRIBUTION.md`](09_DISTRIBUTION.md).

## Next

Continue with [`01_LANGUAGE.md`](01_LANGUAGE.md) for the language grammar and implemented semantics.
