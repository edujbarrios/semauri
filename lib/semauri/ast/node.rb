# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../source_span"

module Semauri
  module AST
    class Node
      attr_reader :span

      def initialize(line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @span = span || SourceSpan.new(
          start_line: line,
          start_column: column,
          end_line: end_line || line,
          end_column: end_column || column + 1
        )
        freeze
      end

      def line
        span.start_line
      end

      def column
        span.start_column
      end

      def end_line
        span.end_line
      end

      def end_column
        span.end_column
      end
    end
  end
end
