# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class MLPlan
      attr_reader :operations

      def initialize(operations: [])
        @operations = operations.freeze
        freeze
      end

      def domain = :ml

      def add(operation)
        self.class.new(operations: operations + [operation])
      end

      def dataset_names
        operations.filter_map do |operation|
          operation.arguments[:name] if operation.name == :import_dataset
        end.freeze
      end

      def model_names
        operations.filter_map do |operation|
          operation.arguments[:name] if %i[initialize_cnn use_pretrained_model].include?(operation.name)
        end.freeze
      end

      def dataset?(name) = dataset_names.include?(name.to_s)
      def model?(name) = model_names.include?(name.to_s)

      def to_h
        {
          kind: :ml_plan,
          domain: :ml,
          datasets: dataset_names,
          models: model_names,
          operations: operations.map(&:to_h)
        }.freeze
      end
    end
  end
end
