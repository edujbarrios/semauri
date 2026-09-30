# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class Element
      attr_reader :id, :kind, :label, :properties

      def initialize(id:, kind:, label:, properties: {})
        @id = id.freeze
        @kind = kind.to_sym
        @label = label.freeze
        @properties = properties.transform_keys(&:to_sym).freeze
        freeze
      end

      def with_property(name, value)
        self.class.new(id: id, kind: kind, label: label, properties: properties.merge(name.to_sym => value))
      end

      def to_h
        { id: id, kind: kind, label: label, properties: properties }
      end
    end
  end
end
