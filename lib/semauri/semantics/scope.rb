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

      def define(name, value, node:, kind: :variable, definition_span: nil)
        key = normalize(name)
        ensure_available!(key, name, node)
        symbol = symbol_table.create(
          name: key,
          kind: kind,
          type: value.type,
          definition_span: definition_span || node.span
        )
        bind(symbol, value, node: node)
      end

      def bind(symbol, value, node:)
        key = normalize(symbol.name)
        ensure_available!(key, symbol.name, node)

        unless symbol.type == value.type
          raise SemanticError.new(
            "Internal binding type mismatch for '#{symbol.name}': #{symbol.type} vs #{value.type}",
            code: "S323",
            line: node.line, column: node.column,
            end_line: node.end_line, end_column: node.end_column
          )
        end

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

      def resolve(reference) = resolve_binding(reference).value
      def symbol_for(reference) = resolve_binding(reference).symbol
      def symbols = symbol_table.symbols

      private

      def ensure_available!(key, display_name, node)
        return unless @bindings.key?(key)
        raise SemanticError.new(
          "Variable '#{display_name}' is already defined in this scope",
          code: "S311",
          line: node.line, column: node.column,
          end_line: node.end_line, end_column: node.end_column,
          hint: "Choose a different name or reuse the existing variable."
        )
      end

      def normalize(name) = name.to_s.downcase
    end
  end
end
