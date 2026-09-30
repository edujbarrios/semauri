# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class Literal < Node
      attr_reader :value_type, :value

      def initialize(value_type:, value:, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @value_type = value_type.to_sym
        @value = value
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_literal(self)
      end
    end
  end
end
