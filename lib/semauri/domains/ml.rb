# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../errors"
require_relative "../semantics/nominal_type"

module Semauri
  module Domains
    class ML < Definition
      DATASET = Semantics::NominalType.new(domain: :ml, name: :dataset, base_type: :string, promote_from_base: false)
      MODEL = Semantics::NominalType.new(domain: :ml, name: :model, base_type: :string, promote_from_base: false)
      DEVICE = Semantics::NominalType.new(domain: :ml, name: :device, base_type: :string, promote_from_base: false)
      TRAINING_CONFIG = Semantics::NominalType.new(domain: :ml, name: :training_config, base_type: :opaque, promote_from_base: false)
      TRAINING_RUN = Semantics::NominalType.new(domain: :ml, name: :training_run, base_type: :opaque, promote_from_base: false)
      INFERENCE_RUN = Semantics::NominalType.new(domain: :ml, name: :inference_run, base_type: :opaque, promote_from_base: false)

      SUPPORTED_OPTIMIZERS = %w[adam adamw sgd].freeze

      def initialize
        super(
          name: :ml,
          types: [DATASET, MODEL, DEVICE, TRAINING_CONFIG, TRAINING_RUN, INFERENCE_RUN],
          operations: [
            Operation.new(
              name: :open_dataset,
              verbs: ["open"],
              pattern: [
                Operation.literal("dataset"),
                Operation.expression(:source, type: :string)
              ],
              returns: DATASET,
              effects: [:filesystem_read]
            ),
            Operation.new(
              name: :load_model,
              verbs: ["load"],
              pattern: [
                Operation.literal("model"),
                Operation.expression(:identifier, type: :string)
              ],
              returns: MODEL,
              effects: [:model_load]
            ),
            Operation.new(
              name: :select_device,
              verbs: ["select"],
              pattern: [
                Operation.literal("device"),
                Operation.expression(:name, type: :string)
              ],
              returns: DEVICE,
              effects: []
            ),
            Operation.new(
              name: :configure_training,
              verbs: ["configure"],
              pattern: [
                Operation.literal("training"),
                Operation.literal("for"),
                Operation.expression(:epochs, type: :number),
                Operation.literal("epochs"),
                Operation.literal("using"),
                Operation.literal("optimizer"),
                Operation.expression(:optimizer, type: :string),
                Operation.literal("learning"),
                Operation.literal("rate"),
                Operation.expression(:learning_rate, type: :number),
                Operation.literal("batch"),
                Operation.literal("size"),
                Operation.expression(:batch_size, type: :number),
                Operation.literal("seed"),
                Operation.expression(:seed, type: :number)
              ],
              returns: TRAINING_CONFIG,
              effects: []
            ),
            Operation.new(
              name: :train,
              verbs: ["train"],
              pattern: [
                Operation.expression(:model, type: MODEL),
                Operation.literal("using"),
                Operation.expression(:dataset, type: DATASET),
                Operation.literal("on"),
                Operation.expression(:device, type: DEVICE),
                Operation.literal("for"),
                Operation.expression(:epochs, type: :number),
                Operation.literal("epochs")
              ],
              returns: TRAINING_RUN,
              effects: [:compute, :model_training]
            ),
            Operation.new(
              name: :fit,
              verbs: ["fit"],
              pattern: [
                Operation.expression(:model, type: MODEL),
                Operation.literal("using"),
                Operation.expression(:dataset, type: DATASET),
                Operation.literal("on"),
                Operation.expression(:device, type: DEVICE),
                Operation.literal("with"),
                Operation.expression(:config, type: TRAINING_CONFIG)
              ],
              returns: TRAINING_RUN,
              effects: [:compute, :model_training]
            ),
            Operation.new(
              name: :infer,
              verbs: ["infer"],
              pattern: [
                Operation.expression(:model, type: MODEL),
                Operation.literal("on"),
                Operation.expression(:dataset, type: DATASET),
                Operation.literal("using"),
                Operation.expression(:device, type: DEVICE)
              ],
              returns: INFERENCE_RUN,
              effects: [:compute, :model_inference]
            )
          ]
        )
      end

      def validate_operation_arguments!(operation:, arguments:, node:)
        return arguments unless operation.name == :configure_training

        validate_positive_integer!(arguments.fetch(:epochs), "epochs", node)
        validate_optimizer!(arguments.fetch(:optimizer), node)
        validate_positive_number!(arguments.fetch(:learning_rate), "learning rate", node)
        validate_positive_integer!(arguments.fetch(:batch_size), "batch size", node)
        validate_non_negative_integer!(arguments.fetch(:seed), "seed", node)
        arguments
      end

      private

      def literal_value(node)
        node.fields.fetch(:value) if node.kind == :literal
      end

      def validate_optimizer!(argument, node)
        value = literal_value(argument)
        return unless value
        return if SUPPORTED_OPTIMIZERS.include?(value.to_s.downcase)

        raise_ml_config_error(node, "Unsupported optimizer '#{value}'",
                              "Supported optimizers: #{SUPPORTED_OPTIMIZERS.join(', ')}.")
      end

      def validate_positive_number!(argument, name, node)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(Numeric) && value.positive?

        raise_ml_config_error(node, "Training #{name} must be greater than zero")
      end

      def validate_positive_integer!(argument, name, node)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(Numeric) && value.positive? && value.to_i == value

        raise_ml_config_error(node, "Training #{name} must be a positive integer")
      end

      def validate_non_negative_integer!(argument, name, node)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(Numeric) && !value.negative? && value.to_i == value

        raise_ml_config_error(node, "Training #{name} must be a non-negative integer")
      end

      def raise_ml_config_error(node, message, hint = nil)
        raise SemanticError.new(
          message,
          code: "S336",
          line: node.line,
          column: node.column,
          end_line: node.end_line,
          end_column: node.end_column,
          hint: hint
        )
      end
    end
  end
end
