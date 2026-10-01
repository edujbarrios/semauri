# Semauri distribution

Semauri is a programming language whose reference compiler is implemented in Ruby. Official Semauri distributions keep that implementation detail private: users install **Semauri**, not a system Ruby environment.

## Distribution model

An official release is a versioned directory containing the compiler, a private relocatable Ruby runtime and a small launcher:

```text
semauri-0.7.3-linux-x86_64/
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

The public command is always:

```bash
semauri ...
```

The launcher resolves the distribution directory and starts the reference compiler with `runtime/ruby/bin/ruby`. It never relies on `ruby` from the user's `PATH`.

## Install layout

The default installer uses user-owned paths and does not require root:

```text
~/.semauri/
├── current -> versions/0.7.3
└── versions/
    └── 0.7.3/
        ├── bin/
        ├── app/
        └── runtime/

~/.local/bin/semauri -> ~/.semauri/current/bin/semauri
```

Override these locations with:

```bash
SEMAURI_INSTALL_ROOT=/custom/root
SEMAURI_BIN_DIR=/custom/bin
```

## Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Install an exact version:

```bash
SEMAURI_VERSION=0.7.3 \
  curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

The installer:

1. detects Linux/macOS and x86_64/arm64;
2. resolves a Semauri GitHub Release;
3. downloads the matching distribution archive;
4. downloads `SHA256SUMS` and verifies the archive;
5. installs it under `~/.semauri/versions/<version>`;
6. switches `~/.semauri/current` atomically through a symlink;
7. exposes `~/.local/bin/semauri`.

Re-running the installer updates `current` to the requested/latest version while leaving older version directories available for rollback.

## Supported official targets

The first distribution pipeline publishes:

```text
linux-x86_64
linux-arm64
macos-x86_64
macos-arm64
```

Windows packaging remains planned rather than being advertised before it is tested end to end.

## Private Ruby runtime

Semauri 0.7.3 pins Ruby 3.4.11 from the immutable `bazel-contrib/portable-ruby` release `3.4.11-1`.

The release workflow pins every upstream archive by SHA-256 before it is accepted. The portable runtime is copied into the Semauri archive, including its upstream licensing files, and the final Semauri archive receives another checksum in `SHA256SUMS`.

This gives two useful boundaries:

```text
Semauri language/compiler source
        ↓
versioned Semauri distribution
        ↓
private portable Ruby runtime
```

Updating the compiler does not require users to manage Ruby. Updating the private Ruby runtime is a distribution concern and can be reviewed independently.

## Release lifecycle

`lib/semauri/version.rb` is the source of truth for the Semauri release version.

When a new version reaches `main`, `.github/workflows/release.yml`:

1. checks whether that version is already published;
2. downloads the pinned portable Ruby runtime for each supported target;
3. verifies the upstream runtime SHA-256;
4. builds a Semauri distribution with `scripts/package-distribution.sh`;
5. smoke-tests the relocated distribution with `semauri version` and `semauri check`;
6. uploads all four archives as workflow artifacts;
7. creates `SHA256SUMS`;
8. creates the version tag/GitHub Release and uploads the archives.

The workflow can also be dispatched manually to rebuild/replace release assets for the current version.

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

- Windows packages and installer support;
- Homebrew/Scoop/Winget integrations where maintainable;
- signed release provenance and stronger supply-chain metadata;
- project-local version selection such as `.semauri-version`;
- a future execution runtime that can evolve independently from the Ruby compiler.
