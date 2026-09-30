# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class CompilerTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_compiles_deterministic_html
    output = @compiler.compile("Create a web for a pet store.").output
    assert_includes output, "<title>Pet Store</title>"
    assert_includes output, "<h1>Pet Store</h1>"
  end

  def test_escapes_html
    output = @compiler.compile('Create a web called "Cats & Dogs".').output
    assert_includes output, "Cats &amp; Dogs"
  end
end
