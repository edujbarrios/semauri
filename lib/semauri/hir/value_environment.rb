# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"

module Semauri
  module HIR
    class ValueEnvironment
      def initialize(parent: nil)
        @parent = parent
        @values = {}
      end

      def child
        self.class.new(parent: self)
      end

      def bind(symbol_id, value, node:)
        id = Integer(symbol_id)
        if @values.key?(id)
          raise SemanticError.new(
            "HIR symbol ##{id} is already bound in this frame",
            code: "S324",
            line: node.line, column: node.column,
            end_line: node.end_line, end_column: node.end_column
          )
        end
        @values[id] = value
        value
      end

      def resolve(symbol_id, node:)
        id = Integer(symbol_id)
        return @values[id] if @values.key?(id)
        return @parent.resolve(id, node: node) if @parent

        raise SemanticError.new(
          "HIR symbol ##{id} has no value in the current scope",
          code: "S324",
          line: node.line, column: node.column,
          end_line: node.end_line, end_column: node.end_column
        )
      end
    end
  end
end
