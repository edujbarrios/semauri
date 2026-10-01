# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../errors"
require_relative "../ir/schema_document"
require_relative "../ir/schema_field"

module Semauri
  module Domains
    class StructuredData < Definition
      def initialize
        super(
          name: :structured_data,
          default_backend: "json-schema",
          artifacts: {
            "schema" => :schema
          },
          elements: {
            "field" => :field
          },
          properties: {
            "datatype" => :datatype,
            "required" => :required
          }
        )
      end

      def property_type(property)
        {
          datatype: :string,
          required: :boolean
        }[property.to_sym]
      end

      def create_artifact(kind:, subject:, title:)
        raise ArgumentError, "Unsupported structured-data artifact '#{kind}'" unless kind.to_sym == :schema

        IR::SchemaDocument.new(title: title || subject || "Untitled Schema")
      end

      def add_element(artifact:, kind:, label:, entities:)
        raise ArgumentError, "Unsupported structured-data element '#{kind}'" unless kind.to_sym == :field

        field = entities.register(kind: :field) do |id|
          IR::SchemaField.new(id: id, label: label || "Field")
        end
        [artifact.add_field(field), field]
      end

      def set_property(artifact:, target:, property:, value:, source_span:)
        validate_datatype!(value, source_span) if property.to_sym == :datatype
        super
      end

      def creation_explanations(artifact:, kind:, subject:, explicit_title:)
        ["'schema' resolved to a structured-data schema.", "Schema title resolved to '#{artifact.title}'."]
      end

      private

      def validate_datatype!(value, source_span)
        return if IR::SchemaField::ALLOWED_DATATYPES.include?(value)

        span = source_span
        raise SemanticError.new(
          "Unsupported schema datatype '#{value}'",
          code: "S328",
          line: span&.start_line || 1,
          column: span&.start_column || 1,
          end_line: span&.end_line || 1,
          end_column: span&.end_column || 2,
          hint: "Use one of: #{IR::SchemaField::ALLOWED_DATATYPES.join(', ')}."
        )
      end
    end
  end
end
