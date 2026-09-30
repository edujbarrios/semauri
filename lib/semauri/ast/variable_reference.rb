# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class VariableReference < Node
      attr_reader :name

      def initialize(name:, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @name = name.downcase.freeze
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_variable_reference(self)
      end
    end
  end
end
