# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class ProgramIRTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def multi_domain_source
    <<~SEMA
      Create a web called Pet Shop.
      Add a button called Buy.
      Write "hello" to "notes.txt".
      Set the color of the button called Buy to red.
    SEMA
  end

  def test_analyze_builds_program_ir_for_multiple_domains
    _ast, semantic = @compiler.analyze(multi_domain_source)

    assert semantic.multi_domain?
    assert_instance_of Semauri::IR::Program, semantic.program
    assert_equal %i[web filesystem], semantic.domains
    assert_instance_of Semauri::IR::WebDocument, semantic.program_ir.fetch(:web).artifact
    assert_instance_of Semauri::IR::OperationPlan, semantic.program_ir.fetch(:filesystem).artifact
  end

  def test_compile_renders_each_domain_with_its_default_backend
    result = @compiler.compile(multi_domain_source)

    assert result.multi_domain?
    assert_nil result.output
    assert_nil result.backend
    assert_equal %i[web filesystem], result.outputs.map(&:domain)
    assert_equal %w[html posix-sh], result.outputs.map(&:backend)
    assert_includes result.outputs.find { |output| output.domain == :web }.content, "color: red"
    assert_includes result.outputs.find { |output| output.domain == :filesystem }.content, "notes.txt"
  end

  def test_single_domain_compilation_remains_backwards_compatible
    result = @compiler.compile("Create a web called Shop.")
    _ast, semantic = @compiler.analyze("Create a web called Shop.")

    refute result.multi_domain?
    assert_equal "html", result.backend
    assert_includes result.output, "<title>Shop</title>"
    assert_instance_of Semauri::IR::WebDocument, semantic.program
    assert_equal :web, semantic.domain
  end

  def test_backend_override_is_rejected_for_multi_domain_program
    error = assert_raises(Semauri::BackendError) do
      @compiler.compile(multi_domain_source, backend: "html")
    end

    assert_equal "S405", error.code
  end

  def test_procedural_domain_does_not_steal_declarative_title_focus
    source = <<~SEMA
      Create a web called Initial.
      Write "hello" to "notes.txt".
      Add a title called Final.
    SEMA

    result = @compiler.compile(source)
    web = result.outputs.find { |output| output.domain == :web }

    assert_includes web.content, "<title>Final</title>"
  end
end
