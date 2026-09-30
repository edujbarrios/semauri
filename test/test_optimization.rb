# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class OptimizationTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_constant_propagation_and_dead_branch_elimination
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

    optimized = @compiler.optimized_hir(source)
    kinds = collect_kinds(optimized.program)

    refute_includes kinds, :if
    refute_includes kinds, :binary
    assert_operator optimized.changes, :>, 0
    assert_equal ["constant_folding", "dead_control_flow"], optimized.passes.map { |pass| pass[:name] }
  end

  def test_optimization_preserves_generated_output
    source = File.read(File.expand_path("../examples/shop.sema", __dir__))
    optimized_output = @compiler.compile(source).output

    no_optimization = Semauri::HIR::Optimization::PassManager.new(passes: [])
    baseline = Semauri::Compiler.new(optimizer: no_optimization).compile(source).output

    assert_equal baseline, optimized_output
  end

  def test_short_circuit_folding_does_not_evaluate_dead_rhs
    source = <<~SEMA
      Create a web called Safe.
      If true or (1 divided by 0 is equal to 1):
        Add a button called Works.
      End.
    SEMA

    result = @compiler.compile(source)

    assert_includes result.output, "Works"
  end

  def test_symbol_table_remains_full_program_after_dead_branch_elimination
    source = <<~SEMA
      Create a web called Symbols.
      If true:
        Let chosen be 1.
      Otherwise:
        Let discarded be 2.
      End.
    SEMA

    optimized = @compiler.optimized_hir(source)

    assert_equal %w[chosen discarded], optimized.symbols.map(&:name)
    refute_includes collect_kinds(optimized.program), :if
  end

  def test_compile_exposes_raw_and_optimized_hir
    result = @compiler.compile("Create a web. If true: Add a button called Buy. End.")

    assert_equal :program, result.hir.kind
    assert_equal :program, result.optimized_hir.kind
    assert_includes collect_kinds(result.hir), :if
    refute_includes collect_kinds(result.optimized_hir), :if
  end

  private

  def collect_kinds(node)
    kinds = [node.kind]
    node.fields.each_value do |value|
      case value
      when Semauri::HIR::Node
        kinds.concat collect_kinds(value)
      when Array
        value.grep(Semauri::HIR::Node).each { |child| kinds.concat collect_kinds(child) }
      end
    end
    kinds
  end
end
