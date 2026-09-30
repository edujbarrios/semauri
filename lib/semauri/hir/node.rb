# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module HIR
    class Node
      attr_reader :kind, :type, :fields, :span

      def initialize(kind:, type:, fields: {}, span: nil)
        @kind = kind.to_sym
        @type = type
        @fields = fields.freeze
        @span = span
        freeze
      end

      def line = span&.start_line || 1
      def column = span&.start_column || 1
      def end_line = span&.end_line || line
      def end_column = span&.end_column || column + 1

      def to_h
        {
          kind: kind,
          type: serialize_type(type),
          fields: serialize(fields),
          span: span&.to_h
        }
      end

      private

      def serialize(value)
        case value
        when Node then value.to_h
        when Array then value.map { |item| serialize(item) }
        when Hash then value.transform_values { |item| serialize(item) }
        else value
        end
      end

      def serialize_type(value)
        value.respond_to?(:to_h) ? value.to_h : value
      end
    end
  end
end
