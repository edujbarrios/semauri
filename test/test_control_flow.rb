# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class ControlFlowTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_arithmetic_precedence_and_comparison_select_a_branch
    source = <<~SEMA
      Let price be 10 plus 5 times 2.
      Create a web called Shop.
      If price is equal to 20:
        Add a button called Buy.
      End.
    SEMA
    _ast, semantic = @compiler.analyze(source)
    assert_equal 1, semantic.program.elements.length
    assert_equal "Buy", semantic.program.elements.first.label
    assert semantic.explanations.any? { |line| line.include?("If condition evaluated to true") }
  end

  def test_parentheses_override_arithmetic_precedence
    source = <<~SEMA
      Let total be (10 plus 5) times 2.
      Create a web called Shop.
      If total is greater than 20:
        Add a button called Buy.
      End.
    SEMA
    _ast, semantic = @compiler.analyze(source)
    assert_equal "Buy", semantic.program.elements.first.label
  end

  def test_otherwise_branch_is_selected_deterministically
    source = <<~SEMA
      Let price be 5.
      Create a web called Shop.
      If price is greater than 20:
        Add a button called Premium.
      Otherwise:
        Add a button called Standard.
      End.
    SEMA
    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Standard"], semantic.program.elements.map(&:label)
    assert semantic.explanations.any? { |line| line.include?("alternative branch") }
  end

  def test_block_scope_can_shadow_parent_without_mutating_it
    source = <<~SEMA
      Let accent be blue.
      Create a web called Shop.
      Add a button called Buy.
      If true:
        Let accent be red.
        Set the color of the button called Buy to accent.
      End.
      Set the color of the button called Buy to accent.
    SEMA
    _ast, semantic = @compiler.analyze(source)
    assert_equal "blue", semantic.program.elements.first.properties[:color]
  end

  def test_block_local_binding_does_not_leak
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      If true:
        Let localAccent be red.
        Set the color of the button called Buy to localAccent.
      End.
      Set the color of the button called Buy to localAccent.
    SEMA
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S312", error.code
  end

  def test_nested_if_blocks
    source = <<~SEMA
      Let price be 30.
      Let stock be 2.
      Create a web called Shop.
      If price is greater than 20:
        If stock is greater than 0:
          Add a button called Buy.
        End.
      End.
    SEMA
    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Buy"], semantic.program.elements.map(&:label)
  end

  def test_if_requires_boolean_condition
    source = "Create a web. If 42: Add a button called Buy. End."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S316", error.code
  end

  def test_arithmetic_requires_numbers
    source = "Let broken be \"hello\" plus 2. Create a web."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S314", error.code
  end

  def test_equality_is_strictly_typed
    source = "Create a web. If 1 is equal to \"1\": Add a button called Buy. End."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S315", error.code
  end

  def test_division_by_zero_is_a_semantic_error
    source = "Let ratio be 10 divided by 0. Create a web."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S317", error.code
  end

  def test_ast_contains_explicit_block_and_binary_expression_nodes
    source = "Let price be 30. Create a web. If price is greater than 20: Add a button called Buy. End."
    ast = @compiler.parse(source)
    conditional = ast.statements.last
    assert_instance_of Semauri::AST::IfStatement, conditional
    assert_instance_of Semauri::AST::BinaryExpression, conditional.condition
    assert_instance_of Semauri::AST::Block, conditional.consequence
  end
end
