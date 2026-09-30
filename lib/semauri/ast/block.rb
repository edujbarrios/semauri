# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class Block < Node
      attr_reader :statements

      def initialize(statements:, span:)
        @statements = statements.freeze
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_block(self)
      end
    end
  end
end
