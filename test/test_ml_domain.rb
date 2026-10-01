# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class MLDomainTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def source
    <<~SEMA
      Within ml:
        Let dataset be Open dataset "./images".
        Let model be Load model "resnet18".
        Let device be Select device "cuda".
        Let run be Train model using dataset on device for 10 epochs.
        Let inference be Infer model on dataset using device.
      End.
    SEMA
  end

  def test_ml_domain_exposes_nominal_types
    domain = @compiler.domains.fetch(:ml)

    assert_equal "ml.dataset", domain.type(:dataset).to_s
    assert_equal "ml.model", domain.type(:model).to_s
    assert_equal "ml.device", domain.type(:device).to_s
    assert_equal "ml.training_run", domain.type(:training_run).to_s
    assert_equal "ml.inference_run", domain.type(:inference_run).to_s
  end

  def test_ml_program_lowers_to_runtime_plan_without_execution
    result = @compiler.compile(source)
    plan = result.runtime_plan

    assert result.runtime?
    assert_empty result.outputs
    assert_equal 5, plan.size
    assert_equal %i[open_dataset load_model select_device train infer], plan.operations.map(&:name)
  end

  def test_ml_runtime_values_flow_between_operations
    plan = @compiler.runtime_plan(source)
    dataset, model, device, train, infer_op = plan.operations

    assert_equal "ml.dataset", dataset.result.type.to_s
    assert_equal "ml.model", model.result.type.to_s
    assert_equal "ml.device", device.result.type.to_s
    assert_equal "ml.training_run", train.result.type.to_s
    assert_equal "ml.inference_run", infer_op.result.type.to_s

    assert_equal model.result.id, train.arguments.fetch(:model).id
    assert_equal dataset.result.id, train.arguments.fetch(:dataset).id
    assert_equal device.result.id, train.arguments.fetch(:device).id
    assert_equal model.result.id, infer_op.arguments.fetch(:model).id
  end

  def test_ml_effect_manifest_is_explicit
    effects = @compiler.effect_analysis(source)

    assert_equal %i[compute filesystem_read model_inference model_load model_training], effects.effects
    refute effects.pure?
  end

  def test_ml_plan_preserves_training_configuration
    train = @compiler.runtime_plan(source).operations.find { |operation| operation.name == :train }

    assert_equal 10, train.arguments.fetch(:epochs)
    assert_equal [:compute, :model_training], train.effects
  end
end
