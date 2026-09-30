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
            hint: "Possible references: #{candidates}. Explicit references will be added in a future release."
          )
        end

        @entities.first
      end
    end
  end
end
