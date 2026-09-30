# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class SemanticContextTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_pronoun_resolves_when_there_is_one_candidate
    source = "Create a web for a pet store. Add a button called Buy. Make it blue."
    _ast, semantic = @compiler.analyze(source)

    button = semantic.program.elements.first
    assert_equal :button, button.element_type
    assert_equal "Buy", button.name
    assert_equal "blue", button.properties[:color]
    assert semantic.explanations.any? { |line| line.include?("Reference 'it' resolved") }
  end

  def test_ambiguous_pronoun_is_rejected
    source = "Create a web. Add a button called Buy. Add a button called Cancel. Make it blue."

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S306", error.code
    assert_includes error.hint, "Buy"
    assert_includes error.hint, "Cancel"
  end

  def test_named_reference_disambiguates
    source = <<~SEMA
      Create a web.
      Add a button called Buy.
      Add a button called Cancel.
      Make the button called Buy blue.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    buy, cancel = semantic.program.elements

    assert_equal "blue", buy.properties[:color]
    assert_nil cancel.properties[:color]
  end

  def test_kind_reference_resolves_single_candidate
    _ast, semantic = @compiler.analyze("Create a web. Add a button. Make the button red.")
    assert_equal "red", semantic.program.elements.first.properties[:color]
  end

  def test_missing_pronoun_target_is_rejected
    error = assert_raises(Semauri::SemanticError) do
      @compiler.analyze("Create a web. Make it blue.")
    end

    assert_equal "S305", error.code
  end

  def test_make_remains_valid_for_web_creation
    _ast, semantic = @compiler.analyze("Make a website named Hello.")
    assert_equal "Hello", semantic.program.title
  end

  def test_html_backend_renders_resolved_button_style
    output = @compiler.compile("Create a web called Shop. Add a button called Buy. Make it green.").output
    assert_includes output, '<button type="button" style="background-color: green;">Buy</button>'
  end
end
