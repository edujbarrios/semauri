# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class Program < Node
      attr_reader :statements

      def initialize(statements:, line: 1, column: 1)
        @statements = statements.freeze
        super(line: line, column: column)
      end

      def accept(visitor)
        visitor.visit_program(self)
      end
    end
  end
end
