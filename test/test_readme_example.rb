# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class ReadmeExampleTest < Minitest::Test
  SOURCE = <<~SEMA.freeze
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

  def test_canonical_readme_example_compiles_to_red_button
    result = Semauri::Compiler.new.compile(SOURCE)

    assert_includes result.output, "<title>Pet Shop</title>"
    assert_includes result.output, '<button style="color: red">Buy</button>'
    refute_includes result.output, "color: green"
  end

  def test_canonical_readme_explanation_is_stable
    result = Semauri::Compiler.new.compile(SOURCE)

    assert_equal "Bound 'price' as symbol #1 to number 18.", result.explanations[0]
    assert_equal "Bound 'tax' as symbol #2 to number 4.", result.explanations[1]
    assert_equal "Bound 'total' as symbol #3 to number 22.", result.explanations[4]
    assert_equal "If condition evaluated to true; selected consequence branch.", result.explanations[9]
    assert_equal %q{Set button-1.color to "red".}, result.explanations[11]
  end
end
