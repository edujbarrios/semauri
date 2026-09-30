# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class AddElement < Node
      attr_reader :kind, :label

      def initialize(kind:, label: nil, line:, column:)
        @kind = kind.to_sym
        @label = label
        super(line: line, column: column)
      end

      def accept(visitor)
        visitor.visit_add_element(self)
      end
    end
  end
end
