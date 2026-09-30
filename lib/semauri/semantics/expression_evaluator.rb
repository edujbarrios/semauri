# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "value"
require_relative "type_system"

module Semauri
  module Semantics
    class ExpressionEvaluator
      LOGICAL_OPERATORS = %i[and or].freeze

      def initialize(scope:, on_variable_resolution: nil)
        @scope = scope
        @on_variable_resolution = on_variable_resolution
      end

      def evaluate(expression) = expression.accept(self)

      def visit_literal(node)
        Value.new(type: node.value_type, value: node.value, definition_span: node.span)
      end

      def visit_list_literal(node)
        values = node.items.map { |item| evaluate(item) }
        type = TypeSystem.list_type(values.map(&:type), node: node)
        Value.new(type: type, value: values.map(&:value).freeze, definition_span: node.span)
      end

      def visit_variable_reference(node)
        binding = @scope.resolve_binding(node)
        @on_variable_resolution&.call(node, binding.value, binding.symbol)
        binding.value
      end

      def visit_unary_expression(node)
        value = evaluate(node.operand)
        result_type = TypeSystem.unary_type(node.operator, value.type, node: node)
        Value.new(type: result_type, value: !value.value, definition_span: node.span)
      end

      def visit_binary_expression(node)
        return evaluate_logical(node) if LOGICAL_OPERATORS.include?(node.operator)

        left = evaluate(node.left)
        right = evaluate(node.right)
        result_type = TypeSystem.binary_type(node.operator, left.type, right.type, node: node)

        result = case node.operator
                 when :add then left.value + right.value
                 when :subtract then left.value - right.value
                 when :multiply then left.value * right.value
                 when :divide
                   raise semantic_error(node.right, "Division by zero", "S317") if right.value.zero?
                   left.value.fdiv(right.value)
                 when :greater_than then left.value > right.value
                 when :less_than then left.value < right.value
                 when :equal then left.value == right.value
                 else raise semantic_error(node, "Unsupported operator '#{node.operator}'", "S318")
                 end

        Value.new(type: result_type, value: result, definition_span: node.span)
      end

      private

      def evaluate_logical(node)
        left = evaluate(node.left)
        TypeSystem.ensure_boolean!(left.type, node: node.left,
                                   message: "Logical operators require boolean operands", code: "S319")

        if node.operator == :and && !left.value
          return Value.new(type: :boolean, value: false, definition_span: node.span)
        end
        if node.operator == :or && left.value
          return Value.new(type: :boolean, value: true, definition_span: node.span)
        end

        right = evaluate(node.right)
        TypeSystem.binary_type(node.operator, left.type, right.type, node: node)
        result = node.operator == :and ? left.value && right.value : left.value || right.value
        Value.new(type: :boolean, value: result, definition_span: node.span)
      end

      def semantic_error(node, message, code)
        SemanticError.new(message, code: code,
                          line: node.line, column: node.column,
                          end_line: node.end_line, end_column: node.end_column)
      end
    end
  end
end
