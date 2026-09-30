# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class SourceSpansTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_tokens_use_exclusive_end_positions
    tokens = @compiler.tokenize("Make it blue.")

    assert_equal({ start: { line: 1, column: 1 }, end: { line: 1, column: 5 } }, tokens[0].span.to_h)
    assert_equal({ start: { line: 1, column: 6 }, end: { line: 1, column: 8 } }, tokens[1].span.to_h)
    assert_equal({ start: { line: 1, column: 9 }, end: { line: 1, column: 13 } }, tokens[2].span.to_h)
  end

  def test_statement_span_covers_complete_sentence
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Make it blue.
    SEMA

    statement = @compiler.parse(source).statements.last

    assert_equal 3, statement.line
    assert_equal 1, statement.column
    assert_equal 3, statement.end_line
    assert_equal 14, statement.end_column
  end

  def test_property_mutation_keeps_source_provenance
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Make it blue.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    button = semantic.program.elements.first

    assert_equal "blue", button.properties[:color]
    assert_equal(
      { start: { line: 3, column: 1 }, end: { line: 3, column: 14 } },
      button.property_provenance[:color]
    )
  end

  def test_ambiguity_diagnostic_highlights_exact_pronoun
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Add an image called Logo.
      Make it blue.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    diagnostic = error.diagnostic(source: source)

    assert_equal "S305", error.code
    assert_includes diagnostic, "S305 at 4:6"
    assert_includes diagnostic, "4 | Make it blue."
    assert_includes diagnostic, "^~"
  end
end
