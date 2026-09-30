# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"
require_relative "../lib/semauri/semantics/resolver"

class HIRLoweringTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_compiler_exposes_hir_used_for_lowering
    result = @compiler.compile(<<~SEMA)
      Let price be 20.
      Create a web called Shop.
    SEMA

    assert_equal :program, result.hir.kind
    assert_equal "Shop", result.ir.title
    assert_equal [1], result.symbols.map(&:id)
  end

  def test_hir_lowering_matches_legacy_resolver_for_existing_program
    source = <<~SEMA
      Let price be 18.
      Let tax be 4.
      Let total be price plus tax.
      Create a web called Pet Shop.
      Add a button called Buy.
      If total is greater than 20:
        Set the color of the button called Buy to red.
      Otherwise:
        Set the color of the button called Buy to green.
      End.
    SEMA

    ast = @compiler.parse(source)
    legacy = Semauri::Semantics::Resolver.new.resolve(ast)
    hir = Semauri::HIR::Builder.new.build(ast)
    lowered = Semauri::HIR::Lowerer.new.lower(hir)

    assert_equal legacy.program.to_h, lowered.program.to_h
    assert_equal legacy.explanations, lowered.explanations
  end

  def test_symbol_table_covers_unselected_branches
    _ast, semantic = @compiler.analyze(<<~SEMA)
      Create a web called Shop.
      If true:
        Let selected be red.
      Otherwise:
        Let unselected be green.
      End.
    SEMA

    assert_equal %w[selected unselected], semantic.symbols.map(&:name)
    assert_equal [1, 2], semantic.symbols.map(&:id)
  end

  def test_static_loop_lowering_reuses_hir_iterator_symbol
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

  def test_short_circuit_skips_runtime_evaluation_of_rhs
    result = @compiler.compile(<<~SEMA)
      Create a web called Shop.
      If true or 1 divided by 0 is equal to 1:
        Add a button called Safe.
      End.
    SEMA

    assert_includes result.output, ">Safe</button>"
  end

  def test_name_resolution_still_checks_short_circuited_rhs
    error = assert_raises(Semauri::SemanticError) do
      @compiler.compile(<<~SEMA)
        Create a web called Shop.
        If true or missingVariable is equal to true:
          Add a button called Invalid.
        End.
      SEMA
    end

    assert_equal "S312", error.code
  end
end
