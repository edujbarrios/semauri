# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../errors"
require_relative "../ir/operation"
require_relative "../ir/operation_plan"

module Semauri
  module Domains
    class Filesystem < Definition
      PATH = Semantics::NominalType.new(domain: :filesystem, name: :path, base_type: :string)

      def initialize
        super(
          name: :filesystem,
          default_backend: "posix-sh",
          types: [PATH],
          operations: [
            Operation.new(name: :write, verbs: %w[write], pattern: [Operation.expression(:content, type: :string), Operation.literal("to"), Operation.expression(:path, type: PATH)], effects: [:filesystem_write]),
            Operation.new(name: :append, verbs: %w[append], pattern: [Operation.expression(:content, type: :string), Operation.literal("to"), Operation.expression(:path, type: PATH)], effects: [:filesystem_write]),
            Operation.new(name: :copy, verbs: %w[copy], pattern: [Operation.expression(:source, type: PATH), Operation.literal("to"), Operation.expression(:destination, type: PATH)], effects: [:filesystem_read, :filesystem_write]),
            Operation.new(name: :move, verbs: %w[move], pattern: [Operation.expression(:source, type: PATH), Operation.literal("to"), Operation.expression(:destination, type: PATH)], effects: [:filesystem_read, :filesystem_write]),
            Operation.new(name: :make_directory, verbs: %w[mkdir], pattern: [Operation.expression(:path, type: PATH)], effects: [:filesystem_write]),
            Operation.new(name: :touch, verbs: %w[touch], pattern: [Operation.expression(:path, type: PATH)], effects: [:filesystem_write]),
            Operation.new(name: :delete, verbs: %w[delete remove], pattern: [Operation.expression(:path, type: PATH)], effects: [:filesystem_write])
          ]
        )
      end

      def validate_operation_arguments!(operation:, arguments:, node:)
        path_arguments = case operation.name
                         when :copy, :move then %i[source destination]
                         when :write, :append, :make_directory, :touch, :delete then [:path]
                         else []
                         end
        path_arguments.each { |name| validate_path!(arguments.fetch(name), node) }
        arguments
      end

      def initial_artifact
        IR::OperationPlan.new(domain: :filesystem)
      end

      def execute_operation(operation:, artifact:, arguments:, source_span:)
        definition = self.operation(operation)
        plan = artifact || initial_artifact
        operation_ir = IR::Operation.new(name: operation, arguments: arguments, effects: definition.effects, source_span: source_span)
        updated = plan.add(operation_ir)
        OperationResult.new(artifact: updated, explanations: ["Planned filesystem operation '#{operation}' with effects #{definition.effects.join(', ')}."])
      end

      private

      def validate_path!(argument, node)
        candidate = argument
        candidate = candidate.fields[:value] if candidate.respond_to?(:kind) && candidate.kind == :promote
        return unless candidate.respond_to?(:kind) && candidate.kind == :literal
        value = candidate.fields.fetch(:value)
        return unless value.is_a?(String) && value.strip.empty?

        raise SemanticError.new(
          "filesystem path must not be empty",
          code: "S340",
          line: node.line, column: node.column, end_line: node.end_line, end_column: node.end_column,
          hint: "Use a non-empty filesystem path."
        )
      end
    end
  end
end
