# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class MLResourceBudgetsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_resource_budget_flows_into_budgeted_fit
    source = <<~SEMA
      Within ml:
        Let dataset be Open dataset "./images".
        Let model be Load model "resnet18".
        Let device be Select device "cuda".
        Let config be Configure training for 4 epochs using optimizer "adamw" learning rate 0.001 batch size 16 seed 42.
        Let resources be Budget resources memory 24 gb workers 4.
        Let run be Allocate resources to fit model using dataset on device with config.
      End.
    SEMA

    plan = @compiler.runtime_plan(source)
    budget = plan.operations.find { |operation| operation.name == :budget_resources }
    fit = plan.operations.find { |operation| operation.name == :fit_budgeted }

    assert_equal "ml.resource_budget", budget.result.type.to_s
    assert_equal 24, budget.arguments.fetch(:memory_gb)
    assert_equal 4, budget.arguments.fetch(:workers)
    assert_empty budget.effects
    assert_equal budget.result.id, fit.arguments.fetch(:budget).id
    assert_equal "ml.training_run", fit.result.type.to_s
    assert_equal [:compute, :model_training], fit.effects
  end

  def test_resource_budget_rejects_non_positive_memory
    source = <<~SEMA
      Within ml:
        Let resources be Budget resources memory 0 gb workers 2.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }
    assert_equal "S342", error.code
    assert_includes error.message, "memory budget"
  end

  def test_resource_budget_rejects_fractional_worker_count
    source = <<~SEMA
      Within ml:
        Let resources be Budget resources memory 8 gb workers 1.5.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }
    assert_equal "S342", error.code
    assert_includes error.message, "worker count"
  end
end
