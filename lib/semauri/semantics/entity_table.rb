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
        if @entities.empty?
          raise SemanticError.new(
            "Pronoun '#{reference.pronoun}' has no object to refer to",
            code: "S304",
            line: reference.line,
            column: reference.column,
            end_line: reference.end_line,
            end_column: reference.end_column,
            hint: "Add an element before referring to it."
          )
        end

        if @entities.length > 1
          candidates = @entities.map { |entity| "#{entity.kind} '#{entity.label}'" }.join(", ")
          raise SemanticError.new(
            "Pronoun '#{reference.pronoun}' is ambiguous",
            code: "S305",
            line: reference.line,
            column: reference.column,
            end_line: reference.end_line,
            end_column: reference.end_column,
            hint: "Possible references: #{candidates}. Use an explicit reference such as 'the button called Buy'."
          )
        end

        @entities.first
      end

      def resolve_named(reference)
        candidates = @entities.select do |entity|
          entity.kind == reference.kind && entity.label.casecmp?(reference.label)
        end

        if candidates.empty?
          raise SemanticError.new(
            "No #{reference.kind} called '#{reference.label}' exists",
            code: "S309",
            line: reference.line,
            column: reference.column,
            end_line: reference.end_line,
            end_column: reference.end_column
          )
        end

        if candidates.length > 1
          raise SemanticError.new(
            "Reference to #{reference.kind} '#{reference.label}' is ambiguous",
            code: "S310",
            line: reference.line,
            column: reference.column,
            end_line: reference.end_line,
            end_column: reference.end_column,
            hint: "Give elements unique names before referring to them explicitly."
          )
        end

        candidates.first
      end
    end
  end
end
