# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../ir/web_document"
require_relative "../ir/element"

module Semauri
  module Domains
    class Web < Definition
      def initialize
        super(
          name: :web,
          artifacts: {
            "web" => :web,
            "website" => :web,
            "webpage" => :web,
            "page" => :web
          },
          elements: {
            "button" => :button,
            "image" => :image,
            "picture" => :image
          },
          properties: {
            "color" => :color
          }
        )
      end

      def property_type(property)
        { color: :color }[property.to_sym]
      end

      def create_artifact(kind:, subject:, title:)
        raise ArgumentError, "Unsupported web artifact '#{kind}'" unless kind.to_sym == :web

        resolved_title, origin = if title
                                   [title, :explicit]
                                 elsif subject
                                   [subject, :subject_default]
                                 else
                                   ["Untitled", :fallback]
                                 end

        IR::WebDocument.new(title: resolved_title, subject: subject, title_origin: origin)
      end

      def add_element(artifact:, kind:, label:, entities:)
        element = entities.register(kind: kind) do |id|
          IR::Element.new(id: id, kind: kind, label: label || kind.to_s.capitalize)
        end
        [artifact.add_element(element), element]
      end
    end
  end
end
