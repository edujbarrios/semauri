# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class BinaryExpression < Node
      attr_reader :left, :operator, :right

      def initialize(left:, operator:, right:, span:)
        @left = left
        @operator = operator.to_sym
        @right = right
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_binary_expression(self)
      end
    end
  end
end
