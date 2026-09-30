# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class IfStatement < Node
      attr_reader :condition, :consequence, :alternative

      def initialize(condition:, consequence:, alternative: nil, span:)
        @condition = condition
        @consequence = consequence
        @alternative = alternative
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_if_statement(self)
      end
    end
  end
end
