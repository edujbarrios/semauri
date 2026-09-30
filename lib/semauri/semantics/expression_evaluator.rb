# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "value"

module Semauri
  module Semantics
    class ExpressionEvaluator
      NUMERIC_OPERATORS = %i[add subtract multiply divide greater_than less_than].freeze
      LOGICAL_OPERATORS = %i[and or].freeze

      def initialize(scope:, on_variable_resolution: nil)
        @scope = scope
        @on_variable_resolution = on_variable_resolution
      end

      def evaluate(expression)
        expression.accept(self)
      end

      def visit_literal(node)
        Value.new(type: node.value_type, value: node.value, definition_span: node.span)
      end

      def visit_variable_reference(node)
        binding = @scope.resolve_binding(node)
        @on_variable_resolution&.call(node, binding.value, binding.symbol)
        binding.value
      end

      def visit_unary_expression(node)
        value = evaluate(node.operand)

        unless node.operator == :not
          raise semantic_error(node, "Unsupported unary operator '#{node.operator}'", "S318")
        end

        ensure_boolean!(value, node.operand)
        Value.new(type: :boolean, value: !value.value, definition_span: node.span)
      end

      def visit_binary_expression(node)
        return evaluate_logical(node) if LOGICAL_OPERATORS.include?(node.operator)

        left = evaluate(node.left)
        right = evaluate(node.right)

        return evaluate_numeric(node, left, right) if NUMERIC_OPERATORS.include?(node.operator)
        return evaluate_equality(node, left, right) if node.operator == :equal

        raise semantic_error(node, "Unsupported operator '#{node.operator}'", "S318")
      end

      private

      def evaluate_logical(node)
        left = evaluate(node.left)
        ensure_boolean!(left, node.left)

        if node.operator == :and && !left.value
          return Value.new(type: :boolean, value: false, definition_span: node.span)
        end

        if node.operator == :or && left.value
          return Value.new(type: :boolean, value: true, definition_span: node.span)
        end

        right = evaluate(node.right)
        ensure_boolean!(right, node.right)

        result = node.operator == :and ? left.value && right.value : left.value || right.value
        Value.new(type: :boolean, value: result, definition_span: node.span)
      end

      def evaluate_numeric(node, left, right)
        unless left.type == :number && right.type == :number
          raise semantic_error(
            node,
            "Operator '#{operator_name(node.operator)}' requires number operands, got #{left.type} and #{right.type}",
            "S314"
          )
        end

        result = case node.operator
                 when :add then left.value + right.value
                 when :subtract then left.value - right.value
                 when :multiply then left.value * right.value
                 when :divide
                   raise semantic_error(node.right, "Division by zero", "S317") if right.value.zero?
                   left.value.fdiv(right.value)
                 when :greater_than then left.value > right.value
                 when :less_than then left.value < right.value
                 end

        type = %i[greater_than less_than].include?(node.operator) ? :boolean : :number
        Value.new(type: type, value: result, definition_span: node.span)
      end

      def evaluate_equality(node, left, right)
        unless left.type == right.type
          raise semantic_error(
            node,
            "Equality requires operands of the same type, got #{left.type} and #{right.type}",
            "S315"
          )
        end

        Value.new(type: :boolean, value: left.value == right.value, definition_span: node.span)
      end

      def ensure_boolean!(value, node)
        return if value.type == :boolean

        raise semantic_error(node, "Logical operators require boolean operands, received #{value.type}", "S319")
      end

      def operator_name(operator)
        {
          add: "plus",
          subtract: "minus",
          multiply: "times",
          divide: "divided by",
          greater_than: "is greater than",
          less_than: "is less than"
        }.fetch(operator, operator.to_s)
      end

      def semantic_error(node, message, code)
        SemanticError.new(message, code: code,
                          line: node.line, column: node.column,
                          end_line: node.end_line, end_column: node.end_column)
      end
    end
  end
end
