# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "schema_field"

module Semauri
  module IR
    class SchemaDocument
      attr_reader :title, :fields

      def initialize(title:, fields: [])
        @title = title.freeze
        @fields = fields.freeze
        freeze
      end

      def with_title(title)
        self.class.new(title: title, fields: fields)
      end

      def add_field(field)
        self.class.new(title: title, fields: fields + [field])
      end

      def replace_element(field)
        replacement = fields.map { |candidate| candidate.id == field.id ? field : candidate }
        self.class.new(title: title, fields: replacement)
      end

      def to_h
        {
          type: "schema_document",
          title: title,
          fields: fields.map(&:to_h)
        }
      end
    end
  end
end
