# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "action_candidates"
require_relative "web"
require_relative "structured_data"
require_relative "filesystem"

module Semauri
  module Domains
    class Registry
      def self.default
        new.register(Web.new).register(StructuredData.new).register(Filesystem.new)
      end

      def initialize
        @domains = {}
        @terms = {}
        @actions = {}
      end

      def register(domain)
        key = domain.name.to_sym
        raise ArgumentError, "Domain '#{key}' is already registered" if @domains.key?(key)

        pending = domain.words.map { |word| [word, domain.classify(word)] }
        pending.each do |word, term|
          if term.category == :action
            if @terms.key?(word)
              raise ArgumentError, "Domain action '#{word}' conflicts with non-action term owned by '#{@terms.fetch(word).domain}'"
            end
          elsif @terms.key?(word) || @actions.key?(word)
            owner = @terms[word]&.domain || @actions.fetch(word).first.domain
            raise ArgumentError, "Domain term '#{word}' conflicts between '#{owner}' and '#{key}'"
          end
        end

        pending.each do |word, term|
          if term.category == :action
            @actions[word] = Array(@actions[word]) + [term]
            @actions[word].freeze
          else
            @terms[word] = term
          end
        end

        @domains[key] = domain
        self
      end

      def classify(word)
        key = word.to_s.downcase
        return @terms[key] if @terms.key?(key)

        candidates = @actions[key]
        candidates && ActionCandidates.new(candidates)
      end

      def action_candidates(word)
        Array(@actions[word.to_s.downcase]).freeze
      end

      def fetch(name)
        @domains.fetch(name.to_sym)
      rescue KeyError
        raise SemanticError.new("Unknown semantic domain '#{name}'", code: "S326")
      end

      def names = @domains.keys.sort.freeze
      def words = (@terms.keys | @actions.keys).sort.freeze

      def to_h
        { domains: names.map { |name| fetch(name).to_h } }
      end

      def infer_property_for_type(value_type)
        candidates = @domains.values.flat_map do |domain|
          domain.property_kinds.filter_map do |property|
            [domain.name, property] if domain.property_type(property) == value_type.to_sym
          end
        end
        candidates.one? ? candidates.first : nil
      end

      def validate_property!(domain:, property:, actual_type:, node:)
        expected = fetch(domain).property_type(property)
        return actual_type unless expected
        return actual_type if expected == actual_type

        raise SemanticError.new(
          "Property '#{property}' in domain '#{domain}' expects #{expected}, but received #{actual_type}",
          code: "S313",
          line: node.line, column: node.column, end_line: node.end_line, end_column: node.end_column,
          hint: "Use a #{expected} literal or a variable containing a #{expected}."
        )
      end
    end
  end
end
