# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class ContextSemanticsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_make_remains_a_create_synonym_when_followed_by_an_artifact
    statement = @compiler.parse("Make a web called Hello.").statements.first

    assert_instance_of Semauri::AST::CreateWeb, statement
    assert_equal "Hello", statement.title
  end

  def test_adds_a_button_and_resolves_it
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Make it blue.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    button = semantic.program.elements.first

    assert_equal :button, button.kind
    assert_equal "Buy", button.label
    assert_equal "blue", button.properties[:color]
    assert semantic.explanations.any? { |line| line.include?("'it' resolved") }
  end

  def test_rejects_an_ambiguous_pronoun
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Add an image called Logo.
      Make it blue.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }

    assert_equal "S305", error.code
    assert_includes error.hint, "button 'Buy'"
    assert_includes error.hint, "image 'Logo'"
  end

  def test_explicit_reference_resolves_ambiguity
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Add an image called Logo.
      Make the button called Buy blue.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    button = semantic.program.elements.find { |element| element.kind == :button }
    image = semantic.program.elements.find { |element| element.kind == :image }

    assert_equal "blue", button.properties[:color]
    assert_nil image.properties[:color]
    assert semantic.explanations.any? { |line| line.include?("Explicit reference resolved") }
  end

  def test_explicit_reference_reports_missing_entity
    source = <<~SEMA
      Create a web called Shop.
      Add an image called Logo.
      Make the button called Buy blue.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S309", error.code
  end

  def test_rejects_a_pronoun_without_a_referent
    source = <<~SEMA
      Create a web called Shop.
      Make it blue.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S304", error.code
  end

  def test_html_backend_renders_resolved_elements
    source = <<~SEMA
      Create a web called Shop.
      Add a button called Buy.
      Make it red.
    SEMA

    output = @compiler.compile(source).output
    assert_includes output, '<button style="color: red">Buy</button>'
  end
end
