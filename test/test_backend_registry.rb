# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class BackendRegistryTest < Minitest::Test
  class DummyBackend
    def render(_program)
      "dummy"
    end
  end

  def test_backends_are_extensible_without_changing_compiler
    registry = Semauri::Backends::Registry.new.register("dummy") { DummyBackend.new }
    compiler = Semauri::Compiler.new(backends: registry)

    result = compiler.compile("Create a web called Hello.", backend: "dummy")
    assert_equal "dummy", result.output
  end
end
