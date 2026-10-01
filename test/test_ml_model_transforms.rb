# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class MLModelTransformsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def source
    <<~SEMA
      Within ml:
        Let dataset be Open dataset "./images".
        Let model be Build cnn for 2 classes input channels 3.
        Let frozen be Freeze model component "features".
        Let adapted be Apply lora to frozen rank 16 alpha 32.
        Let device be Select device "cuda".
        Let config be Configure training for 5 epochs using optimizer "adamw" learning rate 0.0003 batch size 16 seed 42.
        Let run be Fit adapted using dataset on device with config.
      End.
    SEMA
  end

  def test_cnn_and_model_transforms_form_explicit_lineage
    plan = @compiler.runtime_plan(source)

    assert_equal %i[
      open_dataset build_cnn freeze_component apply_lora
      select_device configure_training fit
    ], plan.operations.map(&:name)

    build = plan.operations.find { |operation| operation.name == :build_cnn }
    freeze = plan.operations.find { |operation| operation.name == :freeze_component }
    lora = plan.operations.find { |operation| operation.name == :apply_lora }
    fit = plan.operations.find { |operation| operation.name == :fit }

    assert_equal "ml.model", build.result.type.to_s
    assert_equal build.result.id, freeze.arguments.fetch(:model).id
    assert_equal freeze.result.id, lora.arguments.fetch(:model).id
    assert_equal lora.result.id, fit.arguments.fetch(:model).id
  end

  def test_cnn_configuration_is_preserved_in_runtime_plan
    build = @compiler.runtime_plan(source).operations.find { |operation| operation.name == :build_cnn }

    assert_equal 2, build.arguments.fetch(:classes)
    assert_equal 3, build.arguments.fetch(:input_channels)
    assert_empty build.effects
  end

  def test_lora_configuration_is_preserved
    lora = @compiler.runtime_plan(source).operations.find { |operation| operation.name == :apply_lora }

    assert_equal 16, lora.arguments.fetch(:rank)
    assert_equal 32, lora.arguments.fetch(:alpha)
    assert_empty lora.effects
  end

  def test_model_transform_plan_only_declares_external_training_and_dataset_effects
    effects = @compiler.effect_analysis(source)

    assert_equal %i[compute filesystem_read model_training], effects.effects
  end

  def test_cnn_rejects_non_positive_class_count
    invalid = <<~SEMA
      Within ml:
        Let model be Build cnn for 0 classes input channels 3.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }

    assert_equal "S337", error.code
    assert_includes error.message, "class count"
  end

  def test_freeze_rejects_empty_component
    invalid = <<~SEMA
      Within ml:
        Let model be Build cnn for 2 classes input channels 3.
        Let frozen be Freeze model component "".
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }

    assert_equal "S337", error.code
    assert_includes error.message, "component"
  end

  def test_lora_rejects_invalid_rank
    invalid = <<~SEMA
      Within ml:
        Let model be Load model "some/model".
        Let adapted be Apply lora to model rank 0 alpha 16.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(invalid) }

    assert_equal "S337", error.code
    assert_includes error.message, "LoRA rank"
  end
end
