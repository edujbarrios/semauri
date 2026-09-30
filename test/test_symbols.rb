# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class SymbolsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_bindings_receive_stable_symbol_ids
    source = <<~SEMA
      Let price be 20.
      Let accent be blue.
      Create a web called Shop.
    SEMA

    _ast, semantic = @compiler.analyze(source)

    assert_equal [1, 2], semantic.symbols.map(&:id)
    assert_equal %w[price accent], semantic.symbols.map(&:name)
    assert_equal %i[number color], semantic.symbols.map(&:type)
    assert semantic.symbols.all? { |symbol| symbol.kind == :variable }
  end

  def test_shadowing_creates_a_distinct_symbol_identity
    source = <<~SEMA
      Let accent be blue.
      Create a web called Shop.
      If true:
        Let accent be red.
      End.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    accents = semantic.symbols.select { |symbol| symbol.name == "accent" }

    assert_equal 2, accents.length
    refute_equal accents[0].id, accents[1].id
    assert_equal [1, 2], accents.map(&:id)
  end

  def test_variable_resolution_reports_symbol_identity
    source = <<~SEMA
      Let accent be blue.
      Create a web called Shop.
      Add a button called Buy.
      Set the color of the button called Buy to accent.
    SEMA

    _ast, semantic = @compiler.analyze(source)

    assert semantic.explanations.any? { |line| line.include?("symbol #1") }
  end

  def test_symbols_are_serializable_for_tooling
    source = "Let count be 3. Create a web."
    _ast, semantic = @compiler.analyze(source)
    data = semantic.symbols.first.to_h

    assert_equal 1, data[:id]
    assert_equal "count", data[:name]
    assert_equal :number, data[:type]
    assert_equal :variable, data[:kind]
    assert data[:definition_span]
  end
end
