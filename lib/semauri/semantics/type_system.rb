# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "list_type"
require_relative "nominal_type"

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
                        "Equality requires operands of the same type, got #{type_name(left_type)} and #{type_name(right_type)}",
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

      def list_type(item_types, node:)
        raise error(node, "A list needs at least one item so its type can be inferred", "S320") if item_types.empty?

        element_type = item_types.first
        mismatched = item_types.find { |type| type != element_type }
        if mismatched
          raise error(node,
                      "List items must have one type, got #{type_name(element_type)} and #{type_name(mismatched)}",
                      "S321")
        end

        ListType.new(element_type)
      end

      def ensure_list!(type, node:)
        return type if type.is_a?(ListType)
        raise error(node, "For every requires a list, received #{type_name(type)}", "S322")
      end

      # Nominal types are intentionally not aliases. Exact nominal identity is
      # required once a value is nominal. A primitive may be promoted into a
      # nominal type only when that nominal type explicitly declares it as its
      # base representation.
      def assignment_kind(actual, expected)
        return :exact if actual == expected
        return :promote if expected.is_a?(NominalType) && actual == expected.base_type
        nil
      end

      def assignable?(actual, expected)
        !assignment_kind(actual, expected).nil?
      end

      def ensure_assignable!(actual, expected, node:, message: nil, code: "S333")
        kind = assignment_kind(actual, expected)
        return kind if kind

        raise error(
          node,
          message || "Expected #{type_name(expected)}, but received #{type_name(actual)}",
          code,
          hint: nominal_mismatch_hint(actual, expected)
        )
      end

      def property_type(property)
        { color: :color }[property]
      end

      def validate_property!(property, actual_type, node:)
        expected = property_type(property)
        return actual_type unless expected
        return actual_type if actual_type == expected

        raise error(node,
                    "Property '#{property}' expects #{type_name(expected)}, but received #{type_name(actual_type)}",
                    "S313",
                    hint: "Use a #{type_name(expected)} literal or a variable containing a #{type_name(expected)}.")
      end

      def ensure_boolean!(actual_type, node:, message: "Expected a boolean expression", code: "S316")
        ensure_type!(actual_type, :boolean, node, message, code)
      end

      def ensure_type!(actual, expected, node, message, code)
        return actual if actual == expected
        raise error(node, "#{message}, received #{type_name(actual)}", code)
      end

      def ensure_pair!(left, right, expected, node, message, code)
        return expected if left == expected && right == expected
        raise error(node, "#{message}, got #{type_name(left)} and #{type_name(right)}", code)
      end

      def type_name(type)
        type.to_s
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

      def nominal_mismatch_hint(actual, expected)
        return nil unless expected.is_a?(NominalType)
        return "A #{expected.base_type} value can be promoted to #{expected}." if actual == expected.base_type
        return "#{actual} and #{expected} are distinct nominal types." if actual.is_a?(NominalType)
        "Provide a #{expected} value."
      end
      private_class_method :nominal_mismatch_hint
    end
  end
end
