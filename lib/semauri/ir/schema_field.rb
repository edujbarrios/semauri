# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class SchemaField
      ALLOWED_DATATYPES = %w[string number integer boolean object array].freeze

      attr_reader :id, :kind, :label, :properties, :property_sources

      def initialize(id:, label:, properties: nil, property_sources: {})
        @id = id.freeze
        @kind = :field
        @label = label.freeze
        @properties = (properties || { datatype: "string", required: false }).freeze
        @property_sources = property_sources.freeze
        freeze
      end

      def with_property(name, value, source_span: nil)
        key = name.to_sym
        self.class.new(
          id: id,
          label: label,
          properties: properties.merge(key => value),
          property_sources: property_sources.merge(key => source_span)
        )
      end

      def to_h
        {
          id: id,
          kind: kind,
          label: label,
          properties: properties,
          property_sources: property_sources.transform_values { |span| span&.to_h }
        }
      end
    end
  end
end
