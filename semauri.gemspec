# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "lib/semauri/version"

Gem::Specification.new do |spec|
  spec.name = "semauri"
  spec.version = Semauri::VERSION
  spec.authors = ["Eduardo J. Barrios"]
  spec.summary = "A deterministic programming language built around controlled natural language."
  spec.description = "Semauri explores natural-to-write programming with deterministic, explainable compiler semantics."
  spec.license = "Apache-2.0"
  spec.homepage = "https://github.com/edujbarrios/semauri"
  spec.required_ruby_version = ">= 3.2"

  spec.files = Dir["lib/**/*.rb", "bin/*", "README.md", "LICENSE", "NOTICE", "docs/**/*"]
  spec.bindir = "bin"
  spec.executables = ["semauri"]
  spec.require_paths = ["lib"]
  spec.metadata["source_code_uri"] = "https://github.com/edujbarrios/semauri"
  spec.metadata["changelog_uri"] = "https://github.com/edujbarrios/semauri/releases"
end
