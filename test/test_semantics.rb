# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class SemanticsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_subject_becomes_default_title
    _ast, semantic = @compiler.analyze("Create a web for a pet store.")

    assert_equal "Pet Store", semantic.program.title
    assert_equal :subject_default, semantic.program.title_origin
    assert semantic.explanations.any? { |line| line.include?("defaults to its subject") }
  end

  def test_explicit_title_overrides_inferred_title
    source = <<~SEMA
      Create a web for a pet store.
      Add a title called Happy Paws.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    assert_equal "Happy Paws", semantic.program.title
    assert_equal :explicit, semantic.program.title_origin
  end

  def test_rejects_title_before_document
    error = assert_raises(Semauri::SemanticError) do
      @compiler.analyze("Add a title called Hello.")
    end

    assert_equal "S303", error.code
  end
end
