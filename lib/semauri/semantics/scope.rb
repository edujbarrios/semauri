# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "symbol_table"

module Semauri
  module Semantics
    class Scope
      Binding = Struct.new(:symbol, :value, :span, keyword_init: true) do
        def name = symbol.name
      end

      attr_reader :parent, :symbol_table

      def initialize(parent: nil, symbol_table: nil)
        @parent = parent
        @symbol_table = symbol_table || parent&.symbol_table || SymbolTable.new
        @bindings = {}
      end

      def child
        self.class.new(parent: self, symbol_table: symbol_table)
      end

      def define(name, value, node:, kind: :variable)
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

        symbol = symbol_table.create(
          name: key,
          kind: kind,
          type: value.type,
          definition_span: node.span
        )
        binding = Binding.new(symbol: symbol, value: value, span: node.span)
        @bindings[key] = binding
        binding
      end

      def resolve_binding(reference)
        key = normalize(reference.name)
        binding = @bindings[key]
        return binding if binding
        return parent.resolve_binding(reference) if parent

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

      def resolve(reference)
        resolve_binding(reference).value
      end

      def symbol_for(reference)
        resolve_binding(reference).symbol
      end

      def symbols
        symbol_table.symbols
      end

      private

      def normalize(name)
        name.to_s.downcase
      end
    end
  end
end
