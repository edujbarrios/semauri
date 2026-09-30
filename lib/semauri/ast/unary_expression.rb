# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class UnaryExpression < Node
      attr_reader :operator, :operand

      def initialize(operator:, operand:, span:)
        @operator = operator.to_sym
        @operand = operand
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_unary_expression(self)
      end
    end
  end
end
