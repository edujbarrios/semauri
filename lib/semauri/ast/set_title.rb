# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class SetTitle < Node
      attr_reader :title

      def initialize(title:, line:, column:)
        @title = title
        super(line: line, column: column)
      end

      def accept(visitor)
        visitor.visit_set_title(self)
      end
    end
  end
end
