# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"

module Semauri
  module Semantics
    class EntityTable
      def initialize
        @entities = []
        @kind_counts = Hash.new(0)
      end

      def register(kind:)
        kind = kind.to_sym
        @kind_counts[kind] += 1
        id = "#{kind}-#{@kind_counts[kind]}"
        entity = yield(id)
        @entities << entity
        entity
      end

      def resolve_pronoun(reference)
        resolve_pronoun_data(pronoun: reference.pronoun, node: reference)
      end

      def resolve_pronoun_data(pronoun:, node:)
        if @entities.empty?
          raise SemanticError.new(
            "Pronoun '#{pronoun}' has no object to refer to",
            code: "S304",
            line: node.line, column: node.column,
            end_line: node.end_line, end_column: node.end_column,
            hint: "Add an element before referring to it."
          )
        end

        if @entities.length > 1
          candidates = @entities.map { |entity| "#{entity.kind} '#{entity.label}'" }.join(", ")
          raise SemanticError.new(
            "Pronoun '#{pronoun}' is ambiguous",
            code: "S305",
            line: node.line, column: node.column,
            end_line: node.end_line, end_column: node.end_column,
            hint: "Possible references: #{candidates}. Use an explicit reference such as 'the button called Buy'."
          )
        end

        @entities.first
      end

      def resolve_named(reference)
        resolve_named_data(kind: reference.kind, label: reference.label, node: reference)
      end

      def resolve_named_data(kind:, label:, node:)
        kind = kind.to_sym
        candidates = @entities.select do |entity|
          entity.kind == kind && entity.label.casecmp?(label)
        end

        if candidates.empty?
          raise SemanticError.new(
            "No #{kind} called '#{label}' exists",
            code: "S309",
            line: node.line, column: node.column,
            end_line: node.end_line, end_column: node.end_column
          )
        end

        if candidates.length > 1
          raise SemanticError.new(
            "Reference to #{kind} '#{label}' is ambiguous",
            code: "S310",
            line: node.line, column: node.column,
            end_line: node.end_line, end_column: node.end_column,
            hint: "Give elements unique names before referring to them explicitly."
          )
        end

        candidates.first
      end
    end
  end
end
