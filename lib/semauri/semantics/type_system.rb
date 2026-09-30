# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"

module Semauri
  module Semantics
    module TypeSystem
      module_function

      ARITHMETIC = %i[add subtract multiply divide].freeze
      ORDERING = %i[greater_than less_than].freeze
      LOGICAL = %i[and or].freeze

      def unary_type(operator, operand_type, node:)
        if operator == :not
          ensure_type!(operand_type, :boolean, node, "Logical 'not' requires a boolean operand", "S319")
          return :boolean
        end

        raise error(node, "Unsupported unary operator '#{operator}'", "S318")
      end

      def binary_type(operator, left_type, right_type, node:)
        if ARITHMETIC.include?(operator)
          ensure_pair!(left_type, right_type, :number, node,
                       "Operator '#{operator_name(operator)}' requires number operands", "S314")
          return :number
        end

        if ORDERING.include?(operator)
          ensure_pair!(left_type, right_type, :number, node,
                       "Operator '#{operator_name(operator)}' requires number operands", "S314")
          return :boolean
        end

        if operator == :equal
          unless left_type == right_type
            raise error(node,
                        "Equality requires operands of the same type, got #{left_type} and #{right_type}",
                        "S315")
          end
          return :boolean
        end

        if LOGICAL.include?(operator)
          ensure_pair!(left_type, right_type, :boolean, node,
                       "Logical operators require boolean operands", "S319")
          return :boolean
        end

        raise error(node, "Unsupported operator '#{operator}'", "S318")
      end

      def property_type(property)
        { color: :color }[property]
      end

      def validate_property!(property, actual_type, node:)
        expected = property_type(property)
        return actual_type unless expected
        return actual_type if actual_type == expected

        raise error(node,
                    "Property '#{property}' expects #{expected}, but received #{actual_type}",
                    "S313",
                    hint: "Use a #{expected} literal or a variable containing a #{expected}.")
      end

      def ensure_boolean!(actual_type, node:, message: "Expected a boolean expression", code: "S316")
        ensure_type!(actual_type, :boolean, node, message, code)
      end

      def ensure_type!(actual, expected, node, message, code)
        return actual if actual == expected
        raise error(node, "#{message}, received #{actual}", code)
      end

      def ensure_pair!(left, right, expected, node, message, code)
        return expected if left == expected && right == expected
        raise error(node, "#{message}, got #{left} and #{right}", code)
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

      def error(node, message, code, hint: nil)
        SemanticError.new(message, code: code,
                          line: node.line, column: node.column,
                          end_line: node.end_line, end_column: node.end_column,
                          hint: hint)
      end
    end
  end
end
