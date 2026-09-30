# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class Element
      attr_reader :id, :kind, :label, :properties, :property_provenance

      def initialize(id:, kind:, label:, properties: {}, property_provenance: {})
        @id = id.freeze
        @kind = kind.to_sym
        @label = label.freeze
        @properties = properties.transform_keys(&:to_sym).freeze
        @property_provenance = property_provenance.transform_keys(&:to_sym).freeze
        freeze
      end

      def with_property(name, value, source_span: nil)
        key = name.to_sym
        provenance = property_provenance
        provenance = provenance.merge(key => source_span.to_h) if source_span

        self.class.new(
          id: id,
          kind: kind,
          label: label,
          properties: properties.merge(key => value),
          property_provenance: provenance
        )
      end

      def to_h
        {
          id: id,
          kind: kind,
          label: label,
          properties: properties,
          property_provenance: property_provenance
        }
      end
    end
  end
end
