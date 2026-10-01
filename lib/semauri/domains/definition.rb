# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "term"

module Semauri
  module Domains
    class Definition
      attr_reader :name

      def initialize(name:, artifacts: {}, elements: {}, properties: {})
        @name = name.to_sym
        @terms = {}
        register_terms(:artifact, artifacts)
        register_terms(:element, elements)
        register_terms(:property, properties)
        freeze
      end

      def classify(word)
        @terms[word.to_s.downcase]
      end

      def words = @terms.keys.freeze

      def property_type(property)
        nil
      end

      def create_artifact(kind:, subject:, title:)
        raise NotImplementedError, "Domain '#{name}' does not implement artifact creation"
      end

      def set_title(artifact:, title:)
        artifact.with_title(title)
      end

      def add_element(artifact:, kind:, label:, entities:)
        raise NotImplementedError, "Domain '#{name}' does not implement elements"
      end

      def set_property(artifact:, target:, property:, value:, source_span:)
        updated = target.with_property(property, value, source_span: source_span)
        artifact.replace_element(updated)
      end

      private

      def register_terms(category, mapping)
        mapping.each do |surface, kind|
          key = surface.to_s.downcase
          raise ArgumentError, "Duplicate term '#{surface}' in domain '#{name}'" if @terms.key?(key)

          @terms[key] = Term.new(domain: name, category: category, kind: kind)
        end
      end
    end
  end
end
