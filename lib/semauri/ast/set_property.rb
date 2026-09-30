# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class SetProperty < Node
      attr_reader :target, :property, :value

      def initialize(target:, property:, value:, line:, column:)
        @target = target
        @property = property
        @value = value
        super(line: line, column: column)
      end

      def accept(visitor)
        visitor.visit_set_property(self)
      end
    end
  end
end
