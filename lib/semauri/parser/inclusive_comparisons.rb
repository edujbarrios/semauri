# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module ParserInclusiveComparisons
    private

    def comparison
      left = additive
      return left unless match?(:IS)

      operator = if match?(:GREATER)
                   inclusive_ordering_operator(:greater_than, :greater_than_or_equal, "greater", "S222")
                 elsif match?(:LESS)
                   inclusive_ordering_operator(:less_than, :less_than_or_equal, "less", "S223")
                 elsif match?(:EQUAL)
                   consume(:TO, "Expected 'to' after 'equal'", "S224")
                   :equal
                 else
                   error!(
                     peek,
                     "Expected 'greater than', 'less than', or 'equal to' after 'is'",
                     "S225",
                     hint: "Ordering comparisons may add 'or equal to', for example 'is greater than or equal to'."
                   )
                 end

      right = additive
      AST::BinaryExpression.new(left: left, operator: operator, right: right, span: span_between(left, right))
    end

    def inclusive_ordering_operator(strict_operator, inclusive_operator, direction, code)
      consume(:THAN, "Expected 'than' after '#{direction}'", code)
      return strict_operator unless match?(:OR)

      consume(:EQUAL, "Expected 'equal' after 'or' in an inclusive comparison", code)
      consume(:TO, "Expected 'to' after 'equal' in an inclusive comparison", code)
      inclusive_operator
    end
  end

  Parser.prepend(ParserInclusiveComparisons)
end
