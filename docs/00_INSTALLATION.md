# Installation

This document covers installing and updating Semauri for normal use, plus the source-based workflow for compiler contributors.

> Current release documented here: **0.7.4**

Official Semauri distributions include the private Ruby runtime used by the reference compiler. **You do not need Ruby installed on your system to use Semauri.**

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
SEMAURI_VERSION=0.7.4 \
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

The installer stores versions under `$HOME\.semauri\versions\`, maintains the active version through `$HOME\.semauri\current.txt`, installs the stable launcher at `$HOME\.semauri\bin\semauri.cmd`, and adds that directory to the user `PATH` when needed.

### Install an exact version

```powershell
$env:SEMAURI_VERSION = '0.7.4'
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
Remove-Item Env:SEMAURI_VERSION
```

The install root can be changed with `SEMAURI_INSTALL_ROOT` or the `-InstallRoot` PowerShell parameter.

## Supported official targets

Semauri 0.7.4 provides packages for:

- Linux x86_64
- Linux arm64
- macOS x86_64
- macOS arm64
- Windows x86_64
- Windows arm64

Linux/macOS packages use `.tar.gz`; Windows packages use `.zip`.

## Updating Semauri

Re-run the installer to resolve and activate the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

or on Windows:

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

Older version directories are kept locally, so the distribution layout supports manual rollback when necessary.

## Development from source

Only contributors working directly on the reference compiler need a system Ruby installation.

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby bin/semauri help
```

The reference compiler supports Ruby 3.2+ and its core has no runtime gem dependencies.

Run the complete test suite with:

```bash
ruby -Ilib -e 'Dir["test/test_*.rb"].sort.each { |file| require_relative file }'
```

For package layouts, private runtime details, checksums and the release process, continue with [`09_DISTRIBUTION.md`](09_DISTRIBUTION.md).

## Next

Continue with [`01_LANGUAGE.md`](01_LANGUAGE.md) for the language grammar and implemented semantics.
