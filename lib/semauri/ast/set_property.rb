# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class SetProperty < Node
      attr_reader :target, :property, :value

      def initialize(target:, property:, value:, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @target = target
        @property = property.to_sym
        @value = value
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_set_property(self)
      end
    end
  end
end
