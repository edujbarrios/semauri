# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class MLEvaluationTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_evaluation_is_typed_and_declares_compute_effects
    source = <<~SEMA
      Within ml:
        Let dataset be Open dataset "./validation".
        Let model be Load model "resnet18".
        Let device be Select device "cuda".
        Let evaluation be Evaluate model on dataset using device metric "accuracy".
      End.
    SEMA

    plan = @compiler.runtime_plan(source)
    evaluate = plan.operations.last
    assert_equal :evaluate, evaluate.name
    assert_equal "accuracy", evaluate.arguments.fetch(:metric)
    assert_equal "ml.evaluation_run", evaluate.result.type.to_s
    assert_equal [:compute, :model_evaluation], evaluate.effects
  end

  def test_evaluation_rejects_unknown_literal_metric
    source = <<~SEMA
      Within ml:
        Let dataset be Open dataset "./validation".
        Let model be Load model "resnet18".
        Let device be Select device "cpu".
        Let evaluation be Evaluate model on dataset using device metric "bleu".
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }
    assert_equal "S341", error.code
    assert_includes error.message, "Unsupported evaluation metric"
  end
end
