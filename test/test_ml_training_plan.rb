# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class MLTrainingPlanTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_advanced_training_plan_preserves_precision_accumulation_and_checkpoint_policy
    source = <<~SEMA
      Within ml:
        Let config be Plan training for 12 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42 precision "bf16" accumulate 4 steps checkpoint every 250 steps.
      End.
    SEMA

    operation = @compiler.runtime_plan(source).operations.fetch(0)

    assert_equal :plan_training, operation.name
    assert_equal "ml.training_config", operation.result.type.to_s
    assert_equal "bf16", operation.arguments.fetch(:precision)
    assert_equal 4, operation.arguments.fetch(:gradient_accumulation)
    assert_equal 250, operation.arguments.fetch(:checkpoint_every)
  end

  def test_advanced_training_plan_rejects_unknown_precision
    source = <<~SEMA
      Within ml:
        Let config be Plan training for 2 epochs using optimizer "adam" learning rate 0.001 batch size 8 seed 1 precision "int8" accumulate 1 steps checkpoint every 100 steps.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }

    assert_equal "S338", error.code
    assert_includes error.message, "Unsupported training precision"
  end

  def test_advanced_training_plan_rejects_zero_accumulation
    source = <<~SEMA
      Within ml:
        Let config be Plan training for 2 epochs using optimizer "adam" learning rate 0.001 batch size 8 seed 1 precision "fp16" accumulate 0 steps checkpoint every 100 steps.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }

    assert_equal "S338", error.code
    assert_includes error.message, "gradient accumulation"
  end
end
