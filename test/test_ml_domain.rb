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

  def configured_source
    <<~SEMA
      Within ml:
        Let dataset be Open dataset "./images".
        Let model be Load model "resnet18".
        Let device be Select device "cuda".
        Let config be Configure training for 12 epochs using optimizer "adamw" learning rate 0.0003 batch size 32 seed 42.
        Let run be Fit model using dataset on device with config.
      End.
    SEMA
  end

  def test_ml_domain_exposes_nominal_types
    domain = @compiler.domains.fetch(:ml)
    assert_equal "ml.dataset", domain.type(:dataset).to_s
    assert_equal "ml.model", domain.type(:model).to_s
    assert_equal "ml.device", domain.type(:device).to_s
    assert_equal "ml.training_config", domain.type(:training_config).to_s
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

  def test_simple_train_preserves_epochs
    train = @compiler.runtime_plan(source).operations.find { |operation| operation.name == :train }
    assert_equal 10, train.arguments.fetch(:epochs)
    assert_equal [:compute, :model_training], train.effects
  end

  def test_typed_training_config_flows_into_fit
    plan = @compiler.runtime_plan(configured_source)
    configure = plan.operations.find { |operation| operation.name == :configure_training }
    fit = plan.operations.find { |operation| operation.name == :fit }
    assert_equal "ml.training_config", configure.result.type.to_s
    assert_equal 12, configure.arguments.fetch(:epochs)
    assert_equal "adamw", configure.arguments.fetch(:optimizer)
    assert_in_delta 0.0003, configure.arguments.fetch(:learning_rate), 0.0000001
    assert_equal 32, configure.arguments.fetch(:batch_size)
    assert_equal 42, configure.arguments.fetch(:seed)
    assert_equal configure.result.id, fit.arguments.fetch(:config).id
    assert_equal [:compute, :model_training], fit.effects
  end

  def test_qlora_produces_derived_model_with_explicit_configuration
    source = <<~SEMA
      Within ml:
        Let model be Load model "llama".
        Let adapted be Adapt model with qlora rank 16 alpha 32 quantization 4 bits targets "q_proj,v_proj".
      End.
    SEMA
    plan = @compiler.runtime_plan(source)
    load, adapt = plan.operations
    assert_equal :apply_qlora, adapt.name
    assert_equal load.result.id, adapt.arguments.fetch(:model).id
    assert_equal 16, adapt.arguments.fetch(:rank)
    assert_equal 32, adapt.arguments.fetch(:alpha)
    assert_equal 4, adapt.arguments.fetch(:quantization_bits)
    assert_equal "q_proj,v_proj", adapt.arguments.fetch(:targets)
    assert_equal "ml.model", adapt.result.type.to_s
    assert_empty adapt.effects
  end

  def test_qlora_rejects_unsupported_quantization
    invalid = <<~SEMA
      Within ml:
        Let model be Load model "llama".
        Let adapted be Adapt model with qlora rank 8 alpha 16 quantization 3 bits targets "q_proj".
      End.
    SEMA
    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }
    assert_equal "S339", error.code
    assert_includes error.message, "quantization"
  end

  def test_training_config_rejects_unsupported_optimizer
    invalid = <<~SEMA
      Within ml:
        Let config be Configure training for 2 epochs using optimizer "rmsprop" learning rate 0.001 batch size 8 seed 1.
      End.
    SEMA
    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }
    assert_equal "S336", error.code
    assert_includes error.message, "Unsupported optimizer"
  end

  def test_training_config_rejects_invalid_numeric_constraints
    invalid = <<~SEMA
      Within ml:
        Let config be Configure training for 0 epochs using optimizer "adam" learning rate 0.001 batch size 8 seed 1.
      End.
    SEMA
    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }
    assert_equal "S336", error.code
    assert_includes error.message, "positive integer"
  end

  def test_opaque_training_config_cannot_be_forged_from_string
    invalid = <<~SEMA
      Within ml:
        Let dataset be Open dataset "./images".
        Let model be Load model "resnet18".
        Let device be Select device "cuda".
        Fit model using dataset on device with "not-a-config".
      End.
    SEMA
    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }
    assert_equal "S329", error.code
    assert_includes error.message, "ml.training_config"
  end
end
