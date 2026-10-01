# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "runtime_operation"
require_relative "runtime_value_ref"

module Semauri
  module IR
    class RuntimePlan
      attr_reader :operations

      def initialize(operations: [])
        @operations = operations.freeze
        freeze
      end

      def empty? = operations.empty?
      def size = operations.length

      def append(domain:, name:, arguments:, return_type:, effects:, source_span: nil)
        id = operations.length + 1
        result = if return_type == :unit
                   nil
                 else
                   RuntimeValueRef.new(
                     id: "%#{id}",
                     type: return_type,
                     producer_id: id,
                     source_span: source_span
                   )
                 end

        operation = RuntimeOperation.new(
          id: id,
          domain: domain,
          name: name,
          arguments: arguments,
          return_type: return_type,
          result: result,
          effects: effects,
          source_span: source_span
        )

        [self.class.new(operations: operations + [operation]), result]
      end

      def to_h
        {
          kind: :runtime_plan,
          operations: operations.map(&:to_h)
        }.freeze
      end
    end
  end
end
