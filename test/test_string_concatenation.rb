# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class StringConcatenationTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_joined_with_concatenates_string_values
    source = <<~SEMA
      Let prefix be "Hello, ".
      Let name be "Semauri".
      Create a web called Greeting.
      If prefix joined with name is equal to "Hello, Semauri":
        Add a button called Continue.
      End.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    assert_equal ["Continue"], semantic.program.elements.map(&:label)
  end

  def test_parser_emits_concat_operator
    ast = @compiler.parse(
      'Let greeting be "Hello" joined with " world". Create a web.'
    )

    expression = ast.statements.first.value
    assert_instance_of Semauri::AST::BinaryExpression, expression
    assert_equal :concat, expression.operator
  end

  def test_hir_types_concat_as_string
    hir = @compiler.hir(
      'Let greeting be "Hello" joined with " world". Create a web.'
    )

    expression = hir.program.fields.fetch(:statements).first.fields.fetch(:value)
    assert_equal :binary, expression.kind
    assert_equal :concat, expression.fields.fetch(:operator)
    assert_equal :string, expression.type
  end

  def test_joined_with_requires_string_operands
    source = 'Let broken be 1 joined with " item". Create a web.'

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }
    assert_equal "S343", error.code
  end

  def test_compile_pipeline_constant_folds_concat
    source = <<~SEMA
      Create a web called Greeting.
      If "Hello" joined with "!" is equal to "Hello!":
        Add a button called Continue.
      End.
    SEMA

    result = @compiler.compile(source)
    assert_includes result.output, "Continue"
  end
end
