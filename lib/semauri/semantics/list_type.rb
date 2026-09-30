# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Semantics
    class ListType
      attr_reader :element_type

      def initialize(element_type)
        @element_type = element_type
        freeze
      end

      def ==(other)
        other.is_a?(self.class) && other.element_type == element_type
      end
      alias eql? ==

      def hash
        [self.class, element_type].hash
      end

      def to_s
        "list<#{element_type}>"
      end

      def to_h
        {
          kind: "list",
          element_type: element_type.respond_to?(:to_h) ? element_type.to_h : element_type
        }
      end
    end
  end
end
