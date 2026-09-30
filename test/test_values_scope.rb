# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class ValuesAndScopeTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_variable_can_drive_a_property_value
    source = <<~SEMA
      Let accent be blue.
      Create a web called Shop.
      Add a button called Buy.
      Set the color of the button called Buy to accent.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    button = semantic.program.elements.first

    assert_equal "blue", button.properties[:color]
    assert semantic.explanations.any? { |line| line.include?("Bound 'accent'") }
    assert semantic.explanations.any? { |line| line.include?("Variable 'accent' resolved") }
  end

  def test_canonical_set_accepts_a_direct_color_literal
    source = "Create a web. Add a button called Buy. Set the color of the button called Buy to red."
    _ast, semantic = @compiler.analyze(source)

    assert_equal "red", semantic.program.elements.first.properties[:color]
  end

  def test_unknown_variable_is_a_semantic_error
    source = "Create a web. Add a button called Buy. Set the color of the button called Buy to accent."

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S312", error.code
    assert_includes error.hint, "Let accent be"
  end

  def test_duplicate_binding_is_rejected
    source = "Let accent be blue. Let accent be red. Create a web."

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S311", error.code
  end

  def test_property_types_are_checked
    source = <<~SEMA
      Let accent be "blue".
      Create a web.
      Add a button called Buy.
      Set the color of the button called Buy to accent.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S313", error.code
    assert_includes error.message, "expects color"
    assert_includes error.message, "received string"
  end

  def test_string_and_number_literals_can_be_bound
    source = <<~SEMA
      Let greeting be "Hello".
      Let count be 3.
      Let ratio be 2.5.
      Create a web.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    assert semantic.explanations.any? { |line| line.include?("string \"Hello\"") }
    assert semantic.explanations.any? { |line| line.include?("number 3") }
    assert semantic.explanations.any? { |line| line.include?("number 2.5") }
  end

  def test_legacy_make_syntax_uses_expression_pipeline
    source = "Create a web. Add a button called Buy. Make it green."
    ast, semantic = @compiler.analyze(source)
    mutation = ast.statements.last

    assert_instance_of Semauri::AST::Literal, mutation.value
    assert_equal :color, mutation.value.value_type
    assert_equal "green", semantic.program.elements.first.properties[:color]
  end
end
