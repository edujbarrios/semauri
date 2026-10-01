# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "term"

module Semauri
  module Domains
    class Definition
      attr_reader :name, :default_backend

      def initialize(name:, artifacts: {}, elements: {}, properties: {}, default_backend: nil)
        @name = name.to_sym
        @default_backend = default_backend&.to_s&.freeze
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

      def property_kinds
        @terms.values.select { |term| term.category == :property }.map(&:kind).uniq.freeze
      end

      def to_h
        grouped = @terms.group_by { |_surface, term| term.category }
        {
          name: name,
          default_backend: default_backend,
          artifacts: serialize_terms(grouped[:artifact]),
          elements: serialize_terms(grouped[:element]),
          properties: serialize_terms(grouped[:property])
        }
      end

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

      def creation_explanations(artifact:, kind:, subject:, explicit_title:)
        ["Created #{name} artifact '#{kind}'."]
      end

      private

      def serialize_terms(entries)
        Array(entries).to_h { |surface, term| [surface, term.kind] }
      end

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
