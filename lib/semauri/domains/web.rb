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
          default_backend: "html",
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

      def creation_explanations(artifact:, kind:, subject:, explicit_title:)
        explanations = ["'web' resolved to an HTML web document (default web backend)."]
        explanations << "Subject resolved to '#{subject}'." if subject
        explanations << case artifact.title_origin
                        when :explicit
                          "Title explicitly set to '#{artifact.title}'."
                        when :subject_default
                          "No title was provided, so the web title defaults to its subject: '#{artifact.title}'."
                        else
                          "No title or subject was provided, so the web title defaults to 'Untitled'."
                        end
        explanations
      end
    end
  end
end
