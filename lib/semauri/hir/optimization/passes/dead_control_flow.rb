# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../../node"

module Semauri
  module HIR
    module Optimization
      module Passes
        class DeadControlFlow
          def name = "dead_control_flow"

          def run(program, symbols:)
            @changes = 0
            [transform(program), @changes]
          ensure
            @changes = nil
          end

          private

          def transform(node)
            case node.kind
            when :program, :block
              statements = node.fields.fetch(:statements).filter_map { |statement| transform(statement) }
              rebuild(node, statements: statements)
            when :if
              transform_if(node)
            when :for_each
              transform_for_each(node)
            else
              node
            end
          end

          def transform_if(node)
            condition = node.fields.fetch(:condition)
            consequence = transform(node.fields.fetch(:consequence))
            alternative = node.fields[:alternative] && transform(node.fields[:alternative])

            unless condition.kind == :literal && condition.type == :boolean
              return rebuild(node, consequence: consequence, alternative: alternative)
            end

            @changes += 1
            condition.fields.fetch(:value) ? consequence : alternative
          end

          def transform_for_each(node)
            iterable = node.fields.fetch(:iterable)
            body = transform(node.fields.fetch(:body))

            if iterable.kind == :list && iterable.fields.fetch(:items).empty?
              @changes += 1
              return nil
            end

            rebuild(node, body: body)
          end

          def rebuild(node, **updates)
            fields = node.fields.merge(updates)
            return node if fields == node.fields

            HIR::Node.new(kind: node.kind, type: node.type, fields: fields, span: node.span)
          end
        end
      end
    end
  end
end
