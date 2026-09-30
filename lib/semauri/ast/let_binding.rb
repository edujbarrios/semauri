# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class LetBinding < Node
      attr_reader :name, :value

      def initialize(name:, value:, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @name = name.downcase.freeze
        @value = value
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_let_binding(self)
      end
    end
  end
end
