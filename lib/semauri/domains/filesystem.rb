# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../ir/operation"
require_relative "../ir/operation_plan"

module Semauri
  module Domains
    class Filesystem < Definition
      def initialize
        super(
          name: :filesystem,
          default_backend: "posix-sh",
          operations: [
            Operation.new(
              name: :write,
              verbs: %w[write],
              pattern: [
                Operation.expression(:content, type: :string),
                Operation.literal("to"),
                Operation.expression(:path, type: :string)
              ],
              effects: [:filesystem_write]
            ),
            Operation.new(
              name: :copy,
              verbs: %w[copy],
              pattern: [
                Operation.expression(:source, type: :string),
                Operation.literal("to"),
                Operation.expression(:destination, type: :string)
              ],
              effects: [:filesystem_read, :filesystem_write]
            ),
            Operation.new(
              name: :delete,
              verbs: %w[delete remove],
              pattern: [Operation.expression(:path, type: :string)],
              effects: [:filesystem_write]
            )
          ]
        )
      end

      def initial_artifact
        IR::OperationPlan.new(domain: :filesystem)
      end

      def execute_operation(operation:, artifact:, arguments:, source_span:)
        definition = self.operation(operation)
        plan = artifact || initial_artifact
        operation_ir = IR::Operation.new(
          name: operation,
          arguments: arguments,
          effects: definition.effects,
          source_span: source_span
        )
        updated = plan.add(operation_ir)

        OperationResult.new(
          artifact: updated,
          explanations: ["Planned filesystem operation '#{operation}' with effects #{definition.effects.join(', ')}."]
        )
      end
    end
  end
end
