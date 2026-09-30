# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class Program < Node
      attr_reader :statements

      def initialize(statements:, line: 1, column: 1, end_line: nil, end_column: nil, span: nil)
        @statements = statements.freeze
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_program(self)
      end
    end
  end
end
