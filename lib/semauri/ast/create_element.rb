# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class CreateElement < Node
      attr_reader :element_type, :name

      def initialize(element_type:, name: nil, line:, column:)
        @element_type = element_type
        @name = name
        super(line: line, column: column)
      end

      def accept(visitor)
        visitor.visit_create_element(self)
      end
    end
  end
end
