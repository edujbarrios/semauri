# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class HIRTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_variable_references_lower_to_symbol_ids
    result = @compiler.hir(<<~SEMA)
      Let price be 20.
      Let tax be 4.
      Let total be price plus tax.
      Create a web called Shop.
    SEMA

    statements = result.program.fields[:statements]
    total = statements[2]
    expression = total.fields[:value]

    assert_equal :binary, expression.kind
    assert_equal :number, expression.type
    assert_equal 1, expression.fields[:left].fields[:symbol_id]
    assert_equal 2, expression.fields[:right].fields[:symbol_id]
    assert_equal [1, 2, 3], result.symbols.map(&:id)
  end

  def test_hir_preserves_both_if_branches
    result = @compiler.hir(<<~SEMA)
      Let price be 25.
      Create a web called Shop.
      If price is greater than 20:
        Let accent be red.
      Otherwise:
        Let accent be green.
      End.
    SEMA

    if_node = result.program.fields[:statements][2]

    assert_equal :if, if_node.kind
    assert_equal :boolean, if_node.fields[:condition].type
    assert_equal :block, if_node.fields[:consequence].kind
    assert_equal :block, if_node.fields[:alternative].kind
    assert_equal 3, result.symbols.length
    refute_equal result.symbols[1].id, result.symbols[2].id
  end

  def test_shadowed_names_resolve_to_different_symbols
    result = @compiler.hir(<<~SEMA)
      Let accent be blue.
      Create a web called Shop.
      If true:
        Let accent be red.
        Add a button called Buy.
        Set the color of the button called Buy to accent.
      End.
    SEMA

    if_node = result.program.fields[:statements][2]
    inner_statements = if_node.fields[:consequence].fields[:statements]
    set_property = inner_statements[2]
    symbol_ref = set_property.fields[:value]

    assert_equal :symbol_ref, symbol_ref.kind
    assert_equal 2, symbol_ref.fields[:symbol_id]
    assert_equal %w[accent accent], result.symbols.map(&:name)
  end

  def test_hir_type_checks_expressions_without_evaluating_control_flow
    result = @compiler.hir(<<~SEMA)
      Let price be 10 plus 5 times 2.
      Create a web called Shop.
      If price is equal to 20:
        Add a button called Exact.
      Otherwise:
        Add a button called Other.
      End.
    SEMA

    price = result.program.fields[:statements].first.fields[:value]
    condition = result.program.fields[:statements][2].fields[:condition]

    assert_equal :number, price.type
    assert_equal :boolean, condition.type
  end

  def test_hir_rejects_type_invalid_property_values
    error = assert_raises(Semauri::SemanticError) do
      @compiler.hir(<<~SEMA)
        Let accent be "blue".
        Create a web called Shop.
        Add a button called Buy.
        Set the color of the button called Buy to accent.
      SEMA
    end

    assert_equal "S313", error.code
  end
end
