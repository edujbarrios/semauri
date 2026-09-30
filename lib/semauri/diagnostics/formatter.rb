# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Diagnostics
    module Formatter
      module_function

      def format(error, source)
        header = header_for(error)
        return append_hint(header, error) unless error.line

        lines = source.lines(chomp: true)
        source_line = lines[error.line - 1]
        return append_hint(header, error) unless source_line

        gutter_width = error.line.to_s.length
        marker = marker_for(error, source_line)
        excerpt = [
          header,
          "#{' ' * gutter_width} |",
          "#{error.line.to_s.rjust(gutter_width)} | #{source_line}",
          "#{' ' * gutter_width} | #{marker}"
        ].join("\n")

        append_hint(excerpt, error)
      end

      def header_for(error)
        location = error.line ? " at #{error.line}:#{error.column || 1}" : ""
        "#{error.code}#{location}: #{error.message}"
      end
      private_class_method :header_for

      def marker_for(error, source_line)
        column = [error.column || 1, 1].max
        end_column = if error.end_line == error.line && error.end_column
                       error.end_column
                     else
                       source_line.length + 1
                     end
        width = [end_column - column, 1].max
        (" " * (column - 1)) + "^" + ("~" * (width - 1))
      end
      private_class_method :marker_for

      def append_hint(text, error)
        error.hint ? "#{text}\nHint: #{error.hint}" : text
      end
      private_class_method :append_hint
    end
  end
end
