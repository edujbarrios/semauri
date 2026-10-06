# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../../node"

module Semauri
  module HIR
    module Optimization
      module Passes
        class ConstantFolding
          LOGICAL = %i[and or].freeze

          def name = "constant_folding"

          def run(program, symbols:)
            @constants = {}
            @changes = 0
            [transform(program), @changes]
          ensure
            @constants = @changes = nil
          end

          private

          def transform(node)
            case node.kind
            when :program, :block
              rebuild(node, statements: node.fields.fetch(:statements).map { |statement| transform(statement) })
            when :domain_scope
              rebuild(node, body: transform(node.fields.fetch(:body)))
            when :let
              value = transform(node.fields.fetch(:value))
              @constants[node.fields.fetch(:symbol_id)] = value if constant?(value)
              rebuild(node, value: value)
            when :symbol_ref
              replace_symbol(node)
            when :promote
              rebuild(node, value: transform(node.fields.fetch(:value)))
            when :list
              rebuild(node, items: node.fields.fetch(:items).map { |item| transform(item) })
            when :unary
              fold_unary(node)
            when :binary
              fold_binary(node)
            when :if
              rebuild(node,
                      condition: transform(node.fields.fetch(:condition)),
                      consequence: transform(node.fields.fetch(:consequence)),
                      alternative: node.fields[:alternative] && transform(node.fields[:alternative]))
            when :for_each
              iterator_id = node.fields.fetch(:iterator_symbol_id)
              @constants.delete(iterator_id)
              rebuild(node,
                      iterable: transform(node.fields.fetch(:iterable)),
                      body: transform(node.fields.fetch(:body)))
            when :set_property
              rebuild(node, value: transform(node.fields.fetch(:value)))
            when :domain_operation
              rebuild(node, arguments: node.fields.fetch(:arguments).transform_values { |argument| transform(argument) })
            else
              node
            end
          end

          def replace_symbol(node)
            replacement = @constants[node.fields.fetch(:symbol_id)]
            return node unless replacement

            @changes += 1
            clone_constant(replacement, node.span)
          end

          def fold_unary(node)
            operand = transform(node.fields.fetch(:operand))
            rewritten = rebuild(node, operand: operand)
            return rewritten unless node.fields.fetch(:operator) == :not && literal?(operand)

            @changes += 1
            literal(:boolean, !operand.fields.fetch(:value), node.span)
          end

          def fold_binary(node)
            operator = node.fields.fetch(:operator)
            left = transform(node.fields.fetch(:left))

            if LOGICAL.include?(operator) && literal?(left)
              left_value = left.fields.fetch(:value)
              if operator == :and && left_value == false
                @changes += 1
                return literal(:boolean, false, node.span)
              end
              if operator == :or && left_value == true
                @changes += 1
                return literal(:boolean, true, node.span)
              end
            end

            right = transform(node.fields.fetch(:right))
            rewritten = rebuild(node, left: left, right: right)
            return rewritten unless literal?(left) && literal?(right)
            return rewritten if operator == :divide && right.fields.fetch(:value).zero?

            value = evaluate_binary(operator, left.fields.fetch(:value), right.fields.fetch(:value))
            return rewritten if value == :unsupported

            @changes += 1
            literal(node.type, value, node.span)
          end

          def evaluate_binary(operator, left, right)
            case operator
            when :add then left + right
            when :subtract then left - right
            when :multiply then left * right
            when :concat then left + right
            when :divide then left.fdiv(right)
            when :greater_than then left > right
            when :less_than then left < right
            when :greater_than_or_equal then left >= right
            when :less_than_or_equal then left <= right
            when :equal then left == right
            when :not_equal then left != right
            when :and then left && right
            when :or then left || right
            else :unsupported
            end
          end

          def constant?(node)
            literal?(node) || (node.kind == :list && node.fields.fetch(:items).all? { |item| constant?(item) })
          end

          def literal?(node) = node.kind == :literal

          def clone_constant(node, span)
            return literal(node.type, node.fields.fetch(:value), span) if literal?(node)

            HIR::Node.new(
              kind: :list,
              type: node.type,
              fields: { items: node.fields.fetch(:items).map { |item| clone_constant(item, item.span) } },
              span: span
            )
          end

          def literal(type, value, span)
            HIR::Node.new(kind: :literal, type: type, fields: { value: value }, span: span)
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
