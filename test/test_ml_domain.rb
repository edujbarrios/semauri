# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require_relative "test_helper"

class MLDomainTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_cnn_workflow_compiles_to_an_ml_plan
    source = <<~SEMA
      Within ml:
        Import dataset "./cats-vs-dogs" as "training".
        Initialize cnn as "classifier" with 2 classes.
        Train "classifier" using "training" for 10 epochs.
        Evaluate "classifier" using "training".
        Save "classifier" to "./checkpoints/classifier".
        Explain "classifier" using "gradcam".
      End.
    SEMA

    result = @compiler.compile(source)
    json = JSON.parse(result.output)

    assert_equal "ml-plan", result.backend
    assert_equal :ml, result.ir.domain
    assert_instance_of Semauri::IR::MLPlan, result.ir
    assert_equal %i[import_dataset initialize_cnn train evaluate save explain_model], result.ir.operations.map(&:name)
    assert_equal ["training"], json.fetch("datasets")
    assert_equal ["classifier"], json.fetch("models")
  end

  def test_pretrained_fine_tuning_plan_supports_freezing
    source = <<~SEMA
      Within ml:
        Import dataset "./birds" as "training".
        Use model "google/vit-base-patch16-224" as "classifier".
        Freeze "classifier" component "backbone".
        Train "classifier" using "training" for 8 epochs.
        Evaluate "classifier" using "training".
      End.
    SEMA

    result = @compiler.compile(source)

    assert_equal %i[import_dataset use_pretrained_model freeze_component train evaluate], result.ir.operations.map(&:name)
  end

  def test_ml_effect_manifest_exposes_training_and_model_lifecycle_capabilities
    source = <<~SEMA
      Within ml:
        Import dataset "./birds" as "training".
        Use model "google/vit-base-patch16-224" as "classifier".
        Train "classifier" using "training" for 3 epochs.
        Save "classifier" to "./checkpoint".
        Explain "classifier" using "integrated-gradients".
      End.
    SEMA

    effects = @compiler.effect_analysis(source).effects

    assert_includes effects, :filesystem_read
    assert_includes effects, :filesystem_write
    assert_includes effects, :network
    assert_includes effects, :model_download
    assert_includes effects, :gpu_compute
    assert_includes effects, :model_training
    assert_includes effects, :checkpoint_write
    assert_includes effects, :model_explanation
  end

  def test_unknown_model_is_rejected
    source = <<~SEMA
      Within ml:
        Import dataset "./data" as "training".
        Train "missing" using "training" for 2 epochs.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.compile(source) }

    assert_equal "S336", error.code
    assert_includes error.message, "Unknown ML model"
  end

  def test_unknown_dataset_is_rejected
    source = <<~SEMA
      Within ml:
        Initialize cnn as "classifier" with 2 classes.
        Train "classifier" using "missing" for 2 epochs.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.compile(source) }

    assert_equal "S336", error.code
    assert_includes error.message, "Unknown ML dataset"
  end

  def test_epochs_and_classes_must_be_positive_integers
    classes_error = assert_raises(Semauri::SemanticError) do
      @compiler.compile('Within ml: Initialize cnn as "classifier" with 0 classes. End.')
    end
    epochs_error = assert_raises(Semauri::SemanticError) do
      @compiler.compile(<<~SEMA)
        Within ml:
          Import dataset "./data" as "training".
          Initialize cnn as "classifier" with 2 classes.
          Train "classifier" using "training" for 1.5 epochs.
        End.
      SEMA
    end

    assert_equal "S336", classes_error.code
    assert_equal "S336", epochs_error.code
  end

  def test_duplicate_aliases_are_rejected
    source = <<~SEMA
      Within ml:
        Initialize cnn as "classifier" with 2 classes.
        Initialize cnn as "classifier" with 3 classes.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.compile(source) }

    assert_equal "S336", error.code
    assert_includes error.message, "already defined"
  end

  def test_explain_reports_ml_semantic_decisions
    source = <<~SEMA
      Within ml:
        Import dataset "./data" as "training".
        Initialize cnn as "classifier" with 2 classes.
        Train "classifier" using "training" for 2 epochs.
      End.
    SEMA

    _ast, semantic = @compiler.analyze(source)
    explanation = semantic.explanations.join("\n")

    assert_includes explanation, "ML dataset 'training' planned"
    assert_includes explanation, "CNN 'classifier' planned"
    assert_includes explanation, "Training planned"
  end
end
