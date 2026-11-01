// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0
const CI: &str = include_str!("../.github/workflows/ci.yml");
const RELEASE: &str = include_str!("../.github/workflows/release.yml");

#[test]
fn apple_silicon_distribution_uses_supported_macos_runner() {
    for (name, workflow) in [("CI", CI), ("Release", RELEASE)] {
        assert!(workflow.contains("runner: macos-15"), "{name} must package on supported ARM64 macOS");
        assert!(!workflow.contains("runner: macos-14"), "{name} still targets retired macOS 14");
        assert!(workflow.contains("platform: macos-arm64"), "{name} must preserve its arm64 archive");
    }
}
