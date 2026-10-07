# Semauri distribution

Semauri's reference compiler is implemented in Rust. Official distributions ship one native Semauri executable per platform with no dependency on a system language runtime.

## Distribution model

Unix packages use this shape:

```text
semauri-0.13.0-linux-x86_64/
├── bin/
│   └── semauri
├── manifest.json
├── LICENSE
├── NOTICE
└── README.md
```

Windows packages use:

```text
semauri-0.13.0-windows-x86_64/
├── bin/
│   └── semauri.exe
├── manifest.json
├── LICENSE
├── NOTICE
└── README.md
```

The public command remains `semauri ...`. The package is self-contained at the application level and launches the native compiler directly.

## Quick install

Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/edujbarrios/semauri/main/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/edujbarrios/semauri/main/install.ps1 | iex
```

Both installers resolve the appropriate GitHub Release asset, verify it against `SHA256SUMS`, install it in a versioned user-owned directory and run `semauri version` as a smoke test.

## Supported official targets

```text
linux-x86_64
linux-arm64
macos-x86_64
macos-arm64
windows-x86_64
windows-arm64
```

Linux/macOS releases are `.tar.gz`; Windows releases are `.zip`.

## Native compiler boundary

The distribution boundary is now:

```text
Semauri source
      ↓
Rust reference compiler
      ↓
native platform executable
      ↓
versioned Semauri distribution
```

The native package layout keeps compiler startup and runtime dependencies explicit and compact.

## Release lifecycle

`Cargo.toml` is the source of truth for the Semauri version.

When a new version reaches `main`, `.github/workflows/release.yml`:

1. resolves the version from `Cargo.toml`;
2. builds the Rust compiler natively on each of the six supported runners;
3. packages and smoke-tests the relocated executable with `semauri version` and `semauri check`;
4. uploads all six packages as workflow artifacts;
5. generates one `SHA256SUMS` covering every package;
6. creates or updates the versioned GitHub Release.

Pull requests run the same native package smoke tests through CI before changes can reach `main`.

## Development from source

```bash
git clone https://github.com/edujbarrios/semauri.git
cd semauri
cargo test --all-targets
cargo run -- help
```

Contributors need Rust 1.83+; end users of published binaries do not.

## Future distribution work

Planned improvements include Homebrew/Scoop/Winget integrations where maintainable, signed release provenance, project-local version selection such as `.semauri-version`, and a separately evolvable execution runtime for runtime-dependent programs.
