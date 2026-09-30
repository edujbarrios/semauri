# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class NamedReference < Node
      attr_reader :kind, :label

      def initialize(kind:, label:, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @kind = kind.to_sym
        @label = label
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_named_reference(self)
      end
    end
  end
end
