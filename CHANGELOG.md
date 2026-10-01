# Changelog

All notable changes to Semauri are documented here.

## Unreleased

### Added
- Stable semantic symbols, typed HIR and composable optimization passes.
- Homogeneous `List<T>` collections and static `For every` iteration.
- Semantic domain extension API with generic artifact/element/property/action categories.
- Explicit `Within <domain>: ... End.` semantic scopes.
- Nominal semantic domain types and explicit promotion boundaries.
- Static effect analysis and immutable capability allow-list policies.
- `semauri effects FILE`, `semauri optimize FILE` and `semauri domains` introspection.
- Immutable `IR::Program` for multi-domain programs.
- Immutable runtime planning IR through `IR::RuntimePlan`, `IR::RuntimeOperation` and SSA-like `IR::RuntimeValueRef`.
- Value-returning semantic-domain operations and `semauri plan FILE`.
- Web, Structured Data and Filesystem built-in semantic domains.
- HTML, JSON Schema and POSIX shell backends.
- Built-in ML semantic domain with `ml.dataset`, `ml.model`, `ml.device`, `ml.training_config`, `ml.training_run` and `ml.inference_run`.
- Typed ML dataset/model/device, training and inference planning operations.
- Typed `Configure training` / `Fit` workflow.
- ML model construction through `Build cnn`.
- Immutable model derivation through component freezing and LoRA adaptation.
- Model lineage preserved through RuntimePlan references.
- ML training diagnostic `S336` and model-configuration diagnostic `S337`.
- Versioned Semauri distribution layout with a private Ruby runtime.
- `install.sh` quick installer for Linux/macOS x86_64 and arm64.
- SHA-256 verification for downloaded Semauri releases.
- Automated GitHub Release packaging for all supported distribution targets.
- Pinned portable Ruby 3.4.11 runtime with upstream archive checksums.
- Distribution manifest recording Semauri/platform/private-runtime versions.

### Changed
- The reference compiler remains implemented in Ruby while official distributions no longer require users to install or invoke Ruby.
- The public CLI in installation documentation is now `semauri ...`; direct `ruby bin/semauri ...` usage is reserved for compiler development.
- Primitive-to-nominal promotion is opt-in per nominal type.
- ML runtime-produced nominal values must be produced by ML semantic operations rather than forged from primitives.
- Compilation/planning never performs declared external effects.
- ML model configuration is represented as immutable plan derivation rather than hidden framework mutation.
- ML source semantics remain framework-independent; compilation does not import PyTorch, Transformers or execute model workloads.
- Release publication is driven by the version in `lib/semauri/version.rb` reaching `main`; published archives are smoke-tested before release.
