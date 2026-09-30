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
        <<~HTML
          <!doctype html>
          <html lang="en">
          <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>#{title}</title>
          </head>
          <body>
            <h1>#{title}</h1>
          </body>
          </html>
        HTML
      end
    end
  end
end
