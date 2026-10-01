# Semauri distribution

Semauri is a programming language whose reference compiler is implemented in Ruby. Official Semauri distributions keep that implementation detail private: users install **Semauri**, not a system Ruby environment.

## Distribution model

An official release is a versioned directory containing the compiler, a private Ruby runtime and a small platform launcher.

Unix packages use this shape:

```text
semauri-0.7.4-linux-x86_64/
├── bin/
│   └── semauri
├── app/
│   ├── bin/semauri
│   └── lib/semauri/...
├── runtime/
│   └── ruby/...
├── manifest.json
├── LICENSE
├── NOTICE
└── README.md
```

Windows packages use the same semantic layout with a Windows launcher:

```text
semauri-0.7.4-windows-x86_64/
├── bin/
│   └── semauri.cmd
├── app/
│   ├── bin/semauri
│   └── lib/semauri/...
├── runtime/
│   └── ruby/
│       └── bin/ruby.exe
├── manifest.json
├── LICENSE
├── NOTICE
└── README.md
```

The public command is always:

```text
semauri ...
```

The launcher resolves its distribution directory and starts the reference compiler with the private Ruby runtime. It never relies on `ruby` from the user's `PATH`.

## Quick install

### Linux and macOS

```bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Install an exact version:

```bash
SEMAURI_VERSION=0.7.4 \
  curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

The Unix installer:

1. detects Linux/macOS and x86_64/arm64;
2. resolves a Semauri GitHub Release;
3. downloads the matching `.tar.gz` distribution;
4. downloads `SHA256SUMS` and verifies the archive;
5. installs it under `~/.semauri/versions/<version>`;
6. switches `~/.semauri/current` through a symlink;
7. exposes `~/.local/bin/semauri`.

### Windows — PowerShell

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

Install an exact version:

```powershell
$env:SEMAURI_VERSION = '0.7.4'
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
Remove-Item Env:SEMAURI_VERSION
```

The Windows installer:

1. detects native x86_64 or ARM64 Windows;
2. resolves a Semauri GitHub Release;
3. downloads the matching `.zip` distribution;
4. verifies it against `SHA256SUMS` with `Get-FileHash`;
5. installs the version under `$HOME\.semauri\versions\<version>`;
6. writes `$HOME\.semauri\current.txt` with the active version;
7. creates `$HOME\.semauri\bin\semauri.cmd` as a stable version-selecting shim;
8. adds `$HOME\.semauri\bin` to the user `PATH` when necessary;
9. runs `semauri version` as an installation smoke test.

No administrator privileges or system Ruby installation are required.

## Install layouts

### Linux/macOS

```text
~/.semauri/
├── current -> versions/0.7.4
└── versions/
    └── 0.7.4/
        ├── bin/
        ├── app/
        └── runtime/

~/.local/bin/semauri -> ~/.semauri/current/bin/semauri
```

Override the Unix locations with:

```bash
SEMAURI_INSTALL_ROOT=/custom/root
SEMAURI_BIN_DIR=/custom/bin
```

### Windows

```text
%USERPROFILE%\.semauri\
├── current.txt
├── bin\
│   └── semauri.cmd
└── versions\
    └── 0.7.4\
        ├── bin\semauri.cmd
        ├── app\
        └── runtime\ruby\
```

Override the Windows root with the `SEMAURI_INSTALL_ROOT` environment variable or the `-InstallRoot` PowerShell parameter.

Re-running either installer switches the active version while keeping older version directories available for rollback.

## Supported official targets

Semauri 0.7.4 publishes:

```text
linux-x86_64
linux-arm64
macos-x86_64
macos-arm64
windows-x86_64
windows-arm64
```

Linux/macOS releases are `.tar.gz`; Windows releases are `.zip`.

## Private Ruby runtimes

Semauri 0.7.4 pins Ruby 3.4.11 for every official target.

Linux/macOS use the immutable `bazel-contrib/portable-ruby` release `3.4.11-1`. Windows uses the archive distribution from RubyInstaller `RubyInstaller-3.4.11-1`.

Every upstream runtime archive is pinned by its SHA-256 digest in the release workflow. The runtime is copied inside the Semauri package, and the final Semauri archive receives another checksum in the release-level `SHA256SUMS` file.

This gives two useful boundaries:

```text
Semauri language/compiler source
        ↓
versioned Semauri distribution
        ↓
private platform Ruby runtime
```

Updating the compiler does not require users to manage Ruby. Updating the private Ruby runtime is a distribution concern that can be reviewed independently.

## Release lifecycle

`lib/semauri/version.rb` is the source of truth for the Semauri release version.

When a new version reaches `main`, `.github/workflows/release.yml`:

1. resolves the Semauri version;
2. downloads the checksum-pinned private Ruby runtime for each target;
3. builds four Unix and two Windows distributions;
4. smoke-tests each relocated package with `semauri version` and `semauri check`;
5. uploads all six packages as workflow artifacts;
6. generates one `SHA256SUMS` covering every package;
7. creates the version tag/GitHub Release and uploads all release assets.

Windows packaging is additionally smoke-tested on pull requests on native x64 and ARM64 GitHub runners before it can reach `main`.

The release workflow can also be dispatched manually to rebuild/replace release assets for the current version.

## Development from source

Contributors can still run the reference compiler directly:

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
ruby bin/semauri help
```

That workflow requires Ruby 3.2+ because it is a compiler-development environment, not the end-user distribution.

The distinction is intentional:

```text
users        → install Semauri
contributors → may run the Ruby reference compiler directly
```

## Future distribution work

Planned improvements include:

- Homebrew/Scoop/Winget integrations where maintainable;
- signed release provenance and stronger supply-chain metadata;
- project-local version selection such as `.semauri-version`;
- a future execution runtime that can evolve independently from the Ruby compiler.
