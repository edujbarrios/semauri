# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require_relative "base"
require_relative "../errors"
require_relative "../ir/schema_document"

module Semauri
  module Backends
    class JSONSchema < Base
      def render(program)
        unless program.is_a?(IR::SchemaDocument)
          raise BackendError.new("JSON Schema backend cannot render #{program.class}", code: "S401")
        end

        properties = {}
        required = []

        program.fields.each do |field|
          properties[field.label] = { type: field.properties.fetch(:datatype, "string") }
          required << field.label if field.properties[:required]
        end

        document = {
          "$schema" => "https://json-schema.org/draft/2020-12/schema",
          "title" => program.title,
          "type" => "object",
          "properties" => properties
        }
        document["required"] = required unless required.empty?

        JSON.pretty_generate(document) + "\n"
      end
    end
  end
end
