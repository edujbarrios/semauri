# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class Element
      attr_reader :id, :element_type, :name, :properties

      def initialize(id:, element_type:, name: nil, properties: {})
        @id = id
        @element_type = element_type
        @name = name
        @properties = properties.dup.freeze
        freeze
      end

      def with_property(key, value)
        self.class.new(
          id: id,
          element_type: element_type,
          name: name,
          properties: properties.merge(key => value)
        )
      end

      def to_h
        { type: "element", id: id, element_type: element_type, name: name, properties: properties }
      end
    end
  end
end
