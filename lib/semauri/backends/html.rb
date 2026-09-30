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
        elements = program.elements.map { |element| render_element(element) }.join("\n")
        elements = elements.empty? ? "" : "\n#{indent(elements, 4)}"

        <<~HTML
          <!doctype html>
          <html lang="en">
          <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>#{title}</title>
          </head>
          <body>
            <h1>#{title}</h1>#{elements}
          </body>
          </html>
        HTML
      end

      private

      def render_element(element)
        case element.element_type
        when :button
          render_button(element)
        else
          raise BackendError.new("HTML backend does not support #{element.element_type}", code: "S402")
        end
      end

      def render_button(element)
        label = CGI.escapeHTML(element.name || "Button")
        style = if element.properties[:color]
                  %( style="background-color: #{CGI.escapeHTML(element.properties[:color])};")
                else
                  ""
                end
        %(<button type="button"#{style}>#{label}</button>)
      end

      def indent(text, spaces)
        prefix = " " * spaces
        text.lines.map { |line| prefix + line }.join.rstrip
      end
    end
  end
end
