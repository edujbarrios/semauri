# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Semantics
    class NominalType
      attr_reader :domain, :name, :base_type

      def initialize(domain:, name:, base_type:)
        @domain = domain.to_sym
        @name = name.to_sym
        @base_type = normalize_base_type(base_type)
        freeze
      end

      def ==(other)
        other.is_a?(self.class) &&
          domain == other.domain &&
          name == other.name &&
          base_type == other.base_type
      end
      alias eql? ==

      def hash = [self.class, domain, name, base_type].hash
      def to_s = "#{domain}.#{name}"

      def to_h
        { kind: :nominal, domain: domain, name: name, base: serialize_type(base_type) }.freeze
      end

      private

      def normalize_base_type(type)
        type.is_a?(String) ? type.to_sym : type
      end

      def serialize_type(type)
        type.respond_to?(:to_h) ? type.to_h : type
      end
    end
  end
end
