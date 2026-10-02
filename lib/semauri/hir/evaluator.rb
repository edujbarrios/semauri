# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "../semantics/value"
require_relative "../semantics/type_system"

module Semauri
  module HIR
    class Evaluator
      LOGICAL_OPERATORS = %i[and or].freeze

      def initialize(environment:, symbols_by_id:, on_symbol_resolution: nil)
        @environment = environment
        @symbols_by_id = symbols_by_id
        @on_symbol_resolution = on_symbol_resolution
      end

      def evaluate(node)
        case node.kind
        when :literal then literal(node)
        when :list then list(node)
        when :symbol_ref then symbol_ref(node)
        when :promote then promote(node)
        when :unary then unary(node)
        when :binary then binary(node)
        else
          raise semantic_error(node, "HIR node '#{node.kind}' is not an expression", "S325")
        end
      end

      private

      def literal(node)
        Semantics::Value.new(type: node.type, value: node.fields[:value], definition_span: node.span)
      end

      def list(node)
        values = node.fields.fetch(:items).map { |item| evaluate(item) }
        Semantics::Value.new(type: node.type, value: values.map(&:value).freeze, definition_span: node.span)
      end

      def symbol_ref(node)
        symbol_id = node.fields.fetch(:symbol_id)
        value = @environment.resolve(symbol_id, node: node)
        symbol = @symbols_by_id.fetch(symbol_id)
        @on_symbol_resolution&.call(node, value, symbol)
        value
      end

      def promote(node)
        value = evaluate(node.fields.fetch(:value))
        expected = node.type
        kind = Semantics::TypeSystem.ensure_assignable!(value.type, expected, node: node)
        unless kind == :promote || kind == :exact
          raise semantic_error(node, "Invalid nominal promotion", "S333")
        end

        Semantics::Value.new(type: expected, value: value.value, definition_span: node.span)
      end

      def unary(node)
        operand = evaluate(node.fields.fetch(:operand))
        Semantics::TypeSystem.unary_type(node.fields.fetch(:operator), operand.type, node: node)
        Semantics::Value.new(type: node.type, value: !operand.value, definition_span: node.span)
      end

      def binary(node)
        operator = node.fields.fetch(:operator)
        return logical(node, operator) if LOGICAL_OPERATORS.include?(operator)

        left = evaluate(node.fields.fetch(:left))
        right = evaluate(node.fields.fetch(:right))
        Semantics::TypeSystem.binary_type(operator, left.type, right.type, node: node)

        value = case operator
                when :add then left.value + right.value
                when :subtract then left.value - right.value
                when :multiply then left.value * right.value
                when :divide
                  raise semantic_error(node.fields.fetch(:right), "Division by zero", "S317") if right.value.zero?
                  left.value.fdiv(right.value)
                when :greater_than then left.value > right.value
                when :less_than then left.value < right.value
                when :greater_than_or_equal then left.value >= right.value
                when :less_than_or_equal then left.value <= right.value
                when :equal then left.value == right.value
                else raise semantic_error(node, "Unsupported HIR operator '#{operator}'", "S325")
                end

        Semantics::Value.new(type: node.type, value: value, definition_span: node.span)
      end

      def logical(node, operator)
        left = evaluate(node.fields.fetch(:left))
        Semantics::TypeSystem.ensure_boolean!(left.type, node: node.fields.fetch(:left),
                                              message: "Logical operators require boolean operands", code: "S319")

        if operator == :and && !left.value
          return Semantics::Value.new(type: :boolean, value: false, definition_span: node.span)
        end
        if operator == :or && left.value
          return Semantics::Value.new(type: :boolean, value: true, definition_span: node.span)
        end

        right = evaluate(node.fields.fetch(:right))
        Semantics::TypeSystem.binary_type(operator, left.type, right.type, node: node)
        result = operator == :and ? left.value && right.value : left.value || right.value
        Semantics::Value.new(type: :boolean, value: result, definition_span: node.span)
      end

      def semantic_error(node, message, code)
        SemanticError.new(message, code: code,
                          line: node.line, column: node.column,
                          end_line: node.end_line, end_column: node.end_column)
      end
    end
  end
end
