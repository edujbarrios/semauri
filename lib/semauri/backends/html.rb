# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "cgi"
require_relative "base"
require_relative "../errors"
require_relative "../ir/web_document"

module Semauri
  module Backends
    class HTML < Base
      def render(program)
        unless program.is_a?(IR::WebDocument)
          raise BackendError.new("HTML backend cannot render #{program.class}", code: "S401")
        end

        title = CGI.escapeHTML(program.title)
        body = program.elements.map { |element| render_element(element) }.join("\n")
        body = indent(body, 4) unless body.empty?

        <<~HTML
          <!doctype html>
          <html lang="en">
          <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>#{title}</title>
          </head>
          <body>
            <h1>#{title}</h1>#{body.empty? ? "" : "\n#{body}"}
          </body>
          </html>
        HTML
      end

      private

      def render_element(element)
        style = style_attribute(element)
        label = CGI.escapeHTML(element.label)

        case element.kind
        when :button
          %(<button#{style}>#{label}</button>)
        when :image
          %(<figure#{style} data-semauri-kind="image" aria-label="#{label}">#{label}</figure>)
        else
          raise BackendError.new("HTML backend cannot render element kind '#{element.kind}'", code: "S403")
        end
      end

      def style_attribute(element)
        styles = []
        styles << "color: #{CGI.escapeHTML(element.properties[:color])}" if element.properties[:color]
        styles.empty? ? "" : %( style="#{styles.join('; ')}")
      end

      def indent(text, count)
        prefix = " " * count
        text.lines.map { |line| prefix + line }.join.chomp
      end
    end
  end
end
