# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class InclusiveComparisonsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_greater_than_or_equal_includes_boundary
    source = <<~SEMA
      Let stock be 2.
      Create a web called Shop.
      If stock is greater than or equal to 2:
        Add a button called Available.
      End.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Available"], semantic.program.elements.map(&:label)
  end

  def test_less_than_or_equal_includes_boundary
    source = <<~SEMA
      Let retries be 3.
      Create a web called App.
      If retries is less than or equal to 3:
        Add a button called Retry.
      End.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Retry"], semantic.program.elements.map(&:label)
  end

  def test_inclusive_comparison_selects_alternative_outside_boundary
    source = <<~SEMA
      Let value be 4.
      Create a web called Range.
      If value is greater than or equal to 5:
        Add a button called High.
      Otherwise:
        Add a button called Low.
      End.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Low"], semantic.program.elements.map(&:label)
  end

  def test_parser_emits_distinct_inclusive_operators
    greater = @compiler.parse(
      "Create a web. If 1 is greater than or equal to 1: Add a button called Yes. End."
    )
    less = @compiler.parse(
      "Create a web. If 1 is less than or equal to 1: Add a button called Yes. End."
    )

    assert_equal :greater_than_or_equal, greater.statements.last.condition.operator
    assert_equal :less_than_or_equal, less.statements.last.condition.operator
  end

  def test_compile_pipeline_constant_folds_inclusive_comparison
    source = <<~SEMA
      Create a web called Shop.
      If 10 is less than or equal to 10:
        Add a button called Buy.
      End.
    SEMA

    result = @compiler.compile(source)
    assert_includes result.output, "Buy"
  end
end
