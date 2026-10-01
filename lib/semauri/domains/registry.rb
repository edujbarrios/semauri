# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "web"

module Semauri
  module Domains
    class Registry
      def self.default
        new.register(Web.new)
      end

      def initialize
        @domains = {}
        @terms = {}
      end

      def register(domain)
        key = domain.name.to_sym
        raise ArgumentError, "Domain '#{key}' is already registered" if @domains.key?(key)

        conflicts = domain.words.filter_map do |word|
          next unless @terms.key?(word)
          [word, @terms.fetch(word).domain]
        end

        unless conflicts.empty?
          word, owner = conflicts.first
          raise ArgumentError, "Domain term '#{word}' conflicts between '#{owner}' and '#{key}'"
        end

        domain.words.each { |word| @terms[word] = domain.classify(word) }
        @domains[key] = domain
        self
      end

      def classify(word)
        @terms[word.to_s.downcase]
      end

      def fetch(name)
        @domains.fetch(name.to_sym)
      rescue KeyError
        raise SemanticError.new("Unknown semantic domain '#{name}'", code: "S326")
      end

      def names = @domains.keys.sort.freeze
      def words = @terms.keys.sort.freeze

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
