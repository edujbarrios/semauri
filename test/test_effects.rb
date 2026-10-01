# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class EffectsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_pure_declarative_program_has_no_effects
    analysis = @compiler.effect_analysis("Create a web called Hello.")

    assert analysis.pure?
    assert_empty analysis.effects
  end

  def test_filesystem_program_reports_unique_effects_and_provenance
    source = <<~SEMA
      Write "hello" to "notes.txt".
      Copy "notes.txt" to "backup.txt".
    SEMA

    analysis = @compiler.effect_analysis(source)

    assert_equal %i[filesystem_read filesystem_write], analysis.effects
    refute analysis.pure?
    write_use = analysis.uses_for(:filesystem_write).first
    assert_equal :filesystem, write_use.domain
    assert_equal :write, write_use.operation
    assert_equal 1, write_use.span.start_line
  end

  def test_effect_analysis_is_conservative_before_optimization
    source = <<~SEMA
      If false:
        Delete "unused.tmp".
      End.
      Create a web called Hello.
    SEMA

    analysis = @compiler.effect_analysis(source)

    assert_equal [:filesystem_write], analysis.effects
  end

  def test_capability_policy_rejects_missing_effect
    analysis = @compiler.effect_analysis('Write "hello" to "notes.txt".')
    policy = Semauri::Effects::CapabilityPolicy.allow_none

    error = assert_raises(Semauri::SemanticError) { policy.validate!(analysis) }

    assert_equal "S333", error.code
    assert_includes error.message, "filesystem_write"
  end

  def test_capability_policy_accepts_explicitly_allowed_effects
    analysis = @compiler.effect_analysis('Copy "a.txt" to "b.txt".')
    policy = Semauri::Effects::CapabilityPolicy.allow(:filesystem_read, :filesystem_write)

    assert_same analysis, policy.validate!(analysis)
  end

  def test_compile_can_enforce_a_capability_policy_without_executing_effects
    policy = Semauri::Effects::CapabilityPolicy.allow(:filesystem_write)
    result = @compiler.compile('Write "hello" to "notes.txt".', capability_policy: policy)

    assert_equal [:filesystem_write], result.effects.effects
    assert_includes result.output, "notes.txt"
  end
end
