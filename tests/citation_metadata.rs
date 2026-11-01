// Copyright 2026 Eduardo J. Barrios
// SPDX-License-Identifier: Apache-2.0

const CITATION: &str = include_str!("../CITATION.cff");
const README: &str = include_str!("../README.md");

#[test]
fn citation_metadata_tracks_crate_version_and_attribution() {
    assert!(CITATION.contains("cff-version: 1.2.0"));
    assert!(CITATION.contains("type: software"));
    assert!(CITATION.contains("family-names: Barrios"));
    assert!(CITATION.contains("given-names: \"Eduardo J.\""));
    assert!(CITATION.contains("license: Apache-2.0"));
    assert!(CITATION.contains("repository-code: \"https://github.com/edujbarrios/semauri\""));
    let actual_version = format!("version: {}", env!("CARGO_PKG_VERSION"));
    assert!(CITATION.lines().any(|line| line == actual_version), "{actual_version}");
}

#[test]
fn citation_is_discoverable_from_readme() {
    assert!(README.contains("## Citation"));
    assert!(README.contains("[CITATION.cff](CITATION.cff)"));
    assert!(README.contains("Cite this repository"));
}
