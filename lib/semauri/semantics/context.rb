# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"

module Semauri
  module Semantics
    class Context
      Entry = Struct.new(:id, :entity_type, :name, keyword_init: true)

      def initialize
        @entries = []
        @counters = Hash.new(0)
      end

      def next_id(entity_type)
        @counters[entity_type] += 1
        "#{entity_type}-#{@counters[entity_type]}"
      end

      def register(id:, entity_type:, name: nil)
        @entries << Entry.new(id: id, entity_type: entity_type, name: name)
      end

      def resolve(reference, line:, column:)
        candidates = candidates_for(reference)
        return candidates.first if candidates.one?

        if candidates.empty?
          raise SemanticError.new(
            "Reference has no matching target",
            code: "S305",
            line: line,
            column: column,
            hint: missing_hint(reference)
          )
        end

        raise SemanticError.new(
          "Reference is ambiguous: #{describe(reference)} matches #{candidates.length} elements",
          code: "S306",
          line: line,
          column: column,
          hint: ambiguity_hint(candidates)
        )
      end

      private

      def candidates_for(reference)
        case reference.mode
        when :pronoun
          @entries
        when :kind
          @entries.select { |entry| entry.entity_type == reference.entity_type }
        when :named
          @entries.select do |entry|
            entry.entity_type == reference.entity_type && same_name?(entry.name, reference.name)
          end
        else
          []
        end
      end

      def same_name?(left, right)
        left && right && left.casecmp?(right)
      end

      def describe(reference)
        return "'it'" if reference.mode == :pronoun
        return "#{reference.entity_type} called '#{reference.name}'" if reference.name

        reference.entity_type.to_s
      end

      def missing_hint(reference)
        if reference.mode == :pronoun
          "Create exactly one referable element before using 'it'."
        else
          "Check the element type or name."
        end
      end

      def ambiguity_hint(candidates)
        names = candidates.map { |entry| entry.name || entry.id }
        "Name the target explicitly. Candidates: #{names.join(', ')}."
      end
    end
  end
end
