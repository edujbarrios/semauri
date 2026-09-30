# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class CollectionsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_list_literal_has_a_parametric_type
    _ast, semantic = @compiler.analyze("Let prices be a list of 10, 20, 30. Create a web called Shop.")
    type = semantic.symbols.first.type

    assert_instance_of Semauri::Semantics::ListType, type
    assert_equal :number, type.element_type
    assert_equal "list<number>", type.to_s
  end

  def test_for_every_executes_static_list_in_order
    result = @compiler.compile(<<~SEMA)
      Let colors be a list of red, green.
      Create a web called Shop.
      Add a button called Buy.
      For every accent in colors:
        Set the color of the button called Buy to accent.
      End.
    SEMA

    assert_includes result.output, "color: green"
    assert_equal %w[colors accent], result.symbols.map(&:name)
    assert_equal %i[variable iterator], result.symbols.map(&:kind)
  end

  def test_iterator_does_not_escape_loop_scope
    error = assert_raises(Semauri::SemanticError) do
      @compiler.analyze(<<~SEMA)
        Let colors be a list of red, green.
        Create a web called Shop.
        Add a button called Buy.
        For every accent in colors:
          Set the color of the button called Buy to accent.
        End.
        Set the color of the button called Buy to accent.
      SEMA
    end

    assert_equal "S312", error.code
  end

  def test_lists_must_be_homogeneous
    error = assert_raises(Semauri::SemanticError) do
      @compiler.analyze('Let values be a list of 1, "two". Create a web called Shop.')
    end

    assert_equal "S321", error.code
  end

  def test_for_every_requires_a_list
    error = assert_raises(Semauri::SemanticError) do
      @compiler.analyze(<<~SEMA)
        Let price be 10.
        Create a web called Shop.
        For every item in price:
          Add a button called Invalid.
        End.
      SEMA
    end

    assert_equal "S322", error.code
  end

  def test_hir_preserves_for_each_and_iterator_symbol
    hir = @compiler.hir(<<~SEMA)
      Let prices be a list of 10, 20, 30.
      Create a web called Shop.
      For every price in prices:
        If price is greater than 15:
          Add a button called Premium.
        End.
      End.
    SEMA

    loop_node = hir.program.fields[:statements][2]
    assert_equal :for_each, loop_node.kind
    assert_equal 2, loop_node.fields[:iterator_symbol_id]
    assert_equal :number, hir.symbols[1].type

    if_node = loop_node.fields[:body].fields[:statements].first
    symbol_ref = if_node.fields[:condition].fields[:left]
    assert_equal :symbol_ref, symbol_ref.kind
    assert_equal 2, symbol_ref.fields[:symbol_id]
  end
end
