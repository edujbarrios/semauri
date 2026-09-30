# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"

module Semauri
  module Semantics
    class Scope
      Binding = Struct.new(:name, :value, :span, keyword_init: true)

      attr_reader :parent

      def initialize(parent: nil)
        @parent = parent
        @bindings = {}
      end

      def child
        self.class.new(parent: self)
      end

      def define(name, value, node:)
        key = normalize(name)
        if @bindings.key?(key)
          raise SemanticError.new(
            "Variable '#{name}' is already defined in this scope",
            code: "S311",
            line: node.line,
            column: node.column,
            end_line: node.end_line,
            end_column: node.end_column,
            hint: "Choose a different name or reuse the existing variable."
          )
        end

        @bindings[key] = Binding.new(name: key, value: value, span: node.span)
        value
      end

      def resolve(reference)
        key = normalize(reference.name)
        binding = @bindings[key]
        return binding.value if binding
        return parent.resolve(reference) if parent

        raise SemanticError.new(
          "Unknown variable '#{reference.name}'",
          code: "S312",
          line: reference.line,
          column: reference.column,
          end_line: reference.end_line,
          end_column: reference.end_column,
          hint: "Declare it first with 'Let #{reference.name} be ...'."
        )
      end

      private

      def normalize(name)
        name.to_s.downcase
      end
    end
  end
end
