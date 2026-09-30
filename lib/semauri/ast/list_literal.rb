# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class ListLiteral < Node
      attr_reader :items

      def initialize(items:, span:)
        @items = items.freeze
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_list_literal(self)
      end
    end
  end
end
