# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class LogicalExpressionsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_and_binds_tighter_than_or
    source = "Create a web. If true or false and false: Add a button called Buy. End."
    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Buy"], semantic.program.elements.map(&:label)
  end

  def test_not_has_logical_precedence
    source = "Create a web. If not false and true: Add a button called Buy. End."
    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Buy"], semantic.program.elements.map(&:label)
  end

  def test_not_applies_to_a_comparison_in_natural_order
    source = "Let price be 10. Create a web. If not price is greater than 20: Add a button called Buy. End."
    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Buy"], semantic.program.elements.map(&:label)
  end

  def test_and_short_circuits_rhs_value_evaluation
    source = "Create a web. If false and 1 divided by 0 is equal to 1: Add a button called Never. End."
    _ast, semantic = @compiler.analyze(source)
    assert_empty semantic.program.elements
  end

  def test_or_short_circuits_rhs_value_evaluation
    source = "Create a web. If true or 1 divided by 0 is equal to 1: Add a button called Buy. End."
    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Buy"], semantic.program.elements.map(&:label)
  end

  def test_short_circuit_does_not_skip_name_resolution
    source = "Create a web. If true or missing is equal to true: Add a button called Invalid. End."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S312", error.code
  end

  def test_logical_operators_require_booleans
    source = "Create a web. If 1 and true: Add a button called Buy. End."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S319", error.code
  end

  def test_not_requires_boolean
    source = "Create a web. If not 1: Add a button called Buy. End."
    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S319", error.code
  end

  def test_ast_contains_unary_expression
    ast = @compiler.parse("Create a web. If not false: Add a button called Buy. End.")
    condition = ast.statements.last.condition
    assert_instance_of Semauri::AST::UnaryExpression, condition
    assert_equal :not, condition.operator
  end
end
