# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../errors"
require_relative "../ir/operation"
require_relative "../ir/ml_plan"

module Semauri
  module Domains
    class ML < Definition
      def initialize
        super(
          name: :ml,
          default_backend: "ml-plan",
          operations: [
            Operation.new(
              name: :import_dataset,
              verbs: ["import"],
              pattern: [
                Operation.literal("dataset"),
                Operation.expression(:source, type: :string),
                Operation.literal("as"),
                Operation.expression(:name, type: :string)
              ],
              effects: [:filesystem_read]
            ),
            Operation.new(
              name: :initialize_cnn,
              verbs: ["initialize"],
              pattern: [
                Operation.literal("cnn"),
                Operation.literal("as"),
                Operation.expression(:name, type: :string),
                Operation.literal("with"),
                Operation.expression(:classes, type: :number),
                Operation.literal("classes")
              ]
            ),
            Operation.new(
              name: :use_pretrained_model,
              verbs: ["use"],
              pattern: [
                Operation.literal("model"),
                Operation.expression(:source, type: :string),
                Operation.literal("as"),
                Operation.expression(:name, type: :string)
              ],
              effects: [:network, :model_download]
            ),
            Operation.new(
              name: :freeze_component,
              verbs: ["freeze"],
              pattern: [
                Operation.expression(:model, type: :string),
                Operation.literal("component"),
                Operation.expression(:component, type: :string)
              ]
            ),
            Operation.new(
              name: :train,
              verbs: ["train"],
              pattern: [
                Operation.expression(:model, type: :string),
                Operation.literal("using"),
                Operation.expression(:dataset, type: :string),
                Operation.literal("for"),
                Operation.expression(:epochs, type: :number),
                Operation.literal("epochs")
              ],
              effects: [:filesystem_read, :gpu_compute, :model_training]
            ),
            Operation.new(
              name: :evaluate,
              verbs: ["evaluate"],
              pattern: [
                Operation.expression(:model, type: :string),
                Operation.literal("using"),
                Operation.expression(:dataset, type: :string)
              ],
              effects: [:filesystem_read, :gpu_compute, :model_inference]
            ),
            Operation.new(
              name: :save,
              verbs: ["save"],
              pattern: [
                Operation.expression(:model, type: :string),
                Operation.literal("to"),
                Operation.expression(:path, type: :string)
              ],
              effects: [:filesystem_write, :checkpoint_write]
            ),
            Operation.new(
              name: :explain_model,
              verbs: ["explain"],
              pattern: [
                Operation.expression(:model, type: :string),
                Operation.literal("using"),
                Operation.expression(:method, type: :string)
              ],
              effects: [:gpu_compute, :model_inference, :model_explanation]
            )
          ]
        )
      end

      def initial_artifact
        IR::MLPlan.new
      end

      def execute_operation(operation:, artifact:, arguments:, source_span:)
        plan = artifact || initial_artifact
        validate_operation!(operation, plan, arguments, source_span)
        definition = self.operation(operation)
        operation_ir = IR::Operation.new(
          name: operation,
          arguments: arguments,
          effects: definition.effects,
          source_span: source_span
        )
        updated = plan.add(operation_ir)

        OperationResult.new(
          artifact: updated,
          explanations: [explanation_for(operation, arguments)]
        )
      end

      private

      def validate_operation!(operation, plan, arguments, span)
        case operation.to_sym
        when :import_dataset
          ensure_unique_dataset!(plan, arguments.fetch(:name), span)
        when :initialize_cnn
          ensure_unique_model!(plan, arguments.fetch(:name), span)
          ensure_positive_integer!(arguments.fetch(:classes), "classes", span)
        when :use_pretrained_model
          ensure_unique_model!(plan, arguments.fetch(:name), span)
        when :freeze_component
          ensure_model!(plan, arguments.fetch(:model), span)
        when :train
          ensure_model!(plan, arguments.fetch(:model), span)
          ensure_dataset!(plan, arguments.fetch(:dataset), span)
          ensure_positive_integer!(arguments.fetch(:epochs), "epochs", span)
        when :evaluate
          ensure_model!(plan, arguments.fetch(:model), span)
          ensure_dataset!(plan, arguments.fetch(:dataset), span)
        when :save, :explain_model
          ensure_model!(plan, arguments.fetch(:model), span)
        end
      end

      def ensure_unique_dataset!(plan, name, span)
        return unless plan.dataset?(name)
        raise_ml_error(span, "Dataset '#{name}' is already defined in this ML plan")
      end

      def ensure_unique_model!(plan, name, span)
        return unless plan.model?(name)
        raise_ml_error(span, "Model '#{name}' is already defined in this ML plan")
      end

      def ensure_dataset!(plan, name, span)
        return if plan.dataset?(name)
        raise_ml_error(span, "Unknown ML dataset '#{name}'", hint: "Import the dataset before using it in training or evaluation.")
      end

      def ensure_model!(plan, name, span)
        return if plan.model?(name)
        raise_ml_error(span, "Unknown ML model '#{name}'", hint: "Initialize a CNN or select a pretrained model before using it.")
      end

      def ensure_positive_integer!(value, label, span)
        valid = value.is_a?(Integer) && value.positive?
        return if valid
        raise_ml_error(span, "ML #{label} must be a positive integer, received #{value.inspect}")
      end

      def raise_ml_error(span, message, hint: nil)
        raise SemanticError.new(
          message,
          code: "S336",
          line: span&.start_line,
          column: span&.start_column,
          end_line: span&.end_line,
          end_column: span&.end_column,
          hint: hint
        )
      end

      def explanation_for(operation, arguments)
        case operation.to_sym
        when :import_dataset
          "ML dataset '#{arguments.fetch(:name)}' planned from #{arguments.fetch(:source).inspect}."
        when :initialize_cnn
          "CNN '#{arguments.fetch(:name)}' planned with #{arguments.fetch(:classes)} output classes."
        when :use_pretrained_model
          "Pretrained model '#{arguments.fetch(:name)}' planned from #{arguments.fetch(:source).inspect}."
        when :freeze_component
          "Component #{arguments.fetch(:component).inspect} of model '#{arguments.fetch(:model)}' marked frozen."
        when :train
          "Training planned for model '#{arguments.fetch(:model)}' on dataset '#{arguments.fetch(:dataset)}' for #{arguments.fetch(:epochs)} epochs."
        when :evaluate
          "Evaluation planned for model '#{arguments.fetch(:model)}' on dataset '#{arguments.fetch(:dataset)}'."
        when :save
          "Checkpoint output for model '#{arguments.fetch(:model)}' planned at #{arguments.fetch(:path).inspect}."
        when :explain_model
          "Model explanation for '#{arguments.fetch(:model)}' planned with method #{arguments.fetch(:method).inspect}."
        else
          "ML operation '#{operation}' planned."
        end
      end
    end
  end
end
