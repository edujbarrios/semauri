# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class AddElement < Node
      attr_reader :domain, :kind, :label

      def initialize(domain:, kind:, label: nil, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @domain = domain.to_sym
        @kind = kind.to_sym
        @label = label
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_add_element(self)
      end
    end
  end
end
