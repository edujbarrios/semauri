# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "set"
require_relative "../errors"

module Semauri
  module Effects
    class CapabilityPolicy
      attr_reader :allowed

      def self.allow_none = new(allowed: [])
      def self.allow(*effects) = new(allowed: effects)

      def initialize(allowed:)
        @allowed = Set.new(Array(allowed).map(&:to_sym)).freeze
        freeze
      end

      def allows?(effect) = allowed.include?(effect.to_sym)

      def denied(analysis)
        analysis.effects.reject { |effect| allows?(effect) }.freeze
      end

      def validate!(analysis)
        missing = denied(analysis)
        return analysis if missing.empty?

        first_use = missing.lazy.map { |effect| analysis.uses_for(effect).first }.find(&:itself)
        raise SemanticError.new(
          "Program requires capabilities not allowed by policy: #{missing.join(', ')}",
          code: "S333",
          line: first_use&.span&.start_line,
          column: first_use&.span&.start_column,
          end_line: first_use&.span&.end_line,
          end_column: first_use&.span&.end_column,
          hint: "Inspect required effects and explicitly allow only the capabilities this execution environment should grant."
        )
      end

      def to_h
        { allowed: allowed.to_a.sort.freeze }.freeze
      end
    end
  end
end
