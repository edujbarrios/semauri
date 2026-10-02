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
      SUPPORTED_PRECISIONS = %w[fp32 fp16 bf16].freeze

      def initialize
        super(
          name: :ml,
          types: [DATASET, MODEL, DEVICE, TRAINING_CONFIG, TRAINING_RUN, INFERENCE_RUN],
          operations: [
            Operation.new(name: :open_dataset, verbs: ["open"], pattern: [Operation.literal("dataset"), Operation.expression(:source, type: :string)], returns: DATASET, effects: [:filesystem_read]),
            Operation.new(name: :load_model, verbs: ["load"], pattern: [Operation.literal("model"), Operation.expression(:identifier, type: :string)], returns: MODEL, effects: [:model_load]),
            Operation.new(name: :build_cnn, verbs: ["build"], pattern: [Operation.literal("cnn"), Operation.literal("for"), Operation.expression(:classes, type: :number), Operation.literal("classes"), Operation.literal("input"), Operation.literal("channels"), Operation.expression(:input_channels, type: :number)], returns: MODEL, effects: []),
            Operation.new(name: :freeze_component, verbs: ["freeze"], pattern: [Operation.expression(:model, type: MODEL), Operation.literal("component"), Operation.expression(:component, type: :string)], returns: MODEL, effects: []),
            Operation.new(name: :apply_lora, verbs: ["apply"], pattern: [Operation.literal("lora"), Operation.literal("to"), Operation.expression(:model, type: MODEL), Operation.literal("rank"), Operation.expression(:rank, type: :number), Operation.literal("alpha"), Operation.expression(:alpha, type: :number)], returns: MODEL, effects: []),
            Operation.new(name: :select_device, verbs: ["select"], pattern: [Operation.literal("device"), Operation.expression(:name, type: :string)], returns: DEVICE, effects: []),
            Operation.new(
              name: :configure_training,
              verbs: ["configure"],
              pattern: [Operation.literal("training"), Operation.literal("for"), Operation.expression(:epochs, type: :number), Operation.literal("epochs"), Operation.literal("using"), Operation.literal("optimizer"), Operation.expression(:optimizer, type: :string), Operation.literal("learning"), Operation.literal("rate"), Operation.expression(:learning_rate, type: :number), Operation.literal("batch"), Operation.literal("size"), Operation.expression(:batch_size, type: :number), Operation.literal("seed"), Operation.expression(:seed, type: :number)],
              returns: TRAINING_CONFIG,
              effects: []
            ),
            Operation.new(
              name: :configure_training_advanced,
              verbs: ["configure"],
              pattern: [Operation.literal("training"), Operation.literal("for"), Operation.expression(:epochs, type: :number), Operation.literal("epochs"), Operation.literal("using"), Operation.literal("optimizer"), Operation.expression(:optimizer, type: :string), Operation.literal("learning"), Operation.literal("rate"), Operation.expression(:learning_rate, type: :number), Operation.literal("batch"), Operation.literal("size"), Operation.expression(:batch_size, type: :number), Operation.literal("seed"), Operation.expression(:seed, type: :number), Operation.literal("precision"), Operation.expression(:precision, type: :string), Operation.literal("accumulate"), Operation.expression(:gradient_accumulation, type: :number), Operation.literal("steps"), Operation.literal("checkpoint"), Operation.literal("every"), Operation.expression(:checkpoint_every, type: :number), Operation.literal("steps")],
              returns: TRAINING_CONFIG,
              effects: []
            ),
            Operation.new(name: :train, verbs: ["train"], pattern: [Operation.expression(:model, type: MODEL), Operation.literal("using"), Operation.expression(:dataset, type: DATASET), Operation.literal("on"), Operation.expression(:device, type: DEVICE), Operation.literal("for"), Operation.expression(:epochs, type: :number), Operation.literal("epochs")], returns: TRAINING_RUN, effects: [:compute, :model_training]),
            Operation.new(name: :fit, verbs: ["fit"], pattern: [Operation.expression(:model, type: MODEL), Operation.literal("using"), Operation.expression(:dataset, type: DATASET), Operation.literal("on"), Operation.expression(:device, type: DEVICE), Operation.literal("with"), Operation.expression(:config, type: TRAINING_CONFIG)], returns: TRAINING_RUN, effects: [:compute, :model_training]),
            Operation.new(name: :infer, verbs: ["infer"], pattern: [Operation.expression(:model, type: MODEL), Operation.literal("on"), Operation.expression(:dataset, type: DATASET), Operation.literal("using"), Operation.expression(:device, type: DEVICE)], returns: INFERENCE_RUN, effects: [:compute, :model_inference])
          ]
        )
      end

      def validate_operation_arguments!(operation:, arguments:, node:)
        case operation.name
        when :configure_training, :configure_training_advanced
          validate_positive_integer!(arguments.fetch(:epochs), "epochs", node, code: "S336")
          validate_optimizer!(arguments.fetch(:optimizer), node)
          validate_positive_number!(arguments.fetch(:learning_rate), "learning rate", node, code: "S336")
          validate_positive_integer!(arguments.fetch(:batch_size), "batch size", node, code: "S336")
          validate_non_negative_integer!(arguments.fetch(:seed), "seed", node, code: "S336")
          if operation.name == :configure_training_advanced
            validate_precision!(arguments.fetch(:precision), node)
            validate_positive_integer!(arguments.fetch(:gradient_accumulation), "gradient accumulation", node, code: "S338")
            validate_positive_integer!(arguments.fetch(:checkpoint_every), "checkpoint interval", node, code: "S338")
          end
        when :build_cnn
          validate_positive_integer!(arguments.fetch(:classes), "class count", node, code: "S337")
          validate_positive_integer!(arguments.fetch(:input_channels), "input channel count", node, code: "S337")
        when :freeze_component
          validate_non_empty_string!(arguments.fetch(:component), "component", node, code: "S337")
        when :apply_lora
          validate_positive_integer!(arguments.fetch(:rank), "LoRA rank", node, code: "S337")
          validate_positive_number!(arguments.fetch(:alpha), "LoRA alpha", node, code: "S337")
        end
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
        raise_ml_error(node, "Unsupported optimizer '#{value}'", code: "S336", hint: "Supported optimizers: #{SUPPORTED_OPTIMIZERS.join(', ')}.")
      end

      def validate_precision!(argument, node)
        value = literal_value(argument)
        return unless value
        return if SUPPORTED_PRECISIONS.include?(value.to_s.downcase)
        raise_ml_error(node, "Unsupported training precision '#{value}'", code: "S338", hint: "Supported precisions: #{SUPPORTED_PRECISIONS.join(', ')}.")
      end

      def validate_positive_number!(argument, name, node, code:)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(Numeric) && value.positive?
        raise_ml_error(node, "#{name} must be greater than zero", code: code)
      end

      def validate_positive_integer!(argument, name, node, code:)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(Numeric) && value.positive? && value.to_i == value
        raise_ml_error(node, "#{name} must be a positive integer", code: code)
      end

      def validate_non_negative_integer!(argument, name, node, code:)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(Numeric) && !value.negative? && value.to_i == value
        raise_ml_error(node, "#{name} must be a non-negative integer", code: code)
      end

      def validate_non_empty_string!(argument, name, node, code:)
        value = literal_value(argument)
        return unless value
        return if value.is_a?(String) && !value.strip.empty?
        raise_ml_error(node, "#{name} must be a non-empty string", code: code)
      end

      def raise_ml_error(node, message, code:, hint: nil)
        raise SemanticError.new(message, code: code, line: node.line, column: node.column, end_line: node.end_line, end_column: node.end_column, hint: hint)
      end
    end
  end
end
