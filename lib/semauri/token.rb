# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "source_span"

module Semauri
  Token = Struct.new(:type, :lexeme, :literal, :line, :column, :end_line, :end_column, keyword_init: true) do
    def span
      SourceSpan.new(
        start_line: line,
        start_column: column,
        end_line: end_line || line,
        end_column: end_column || column + [lexeme.to_s.length, 1].max
      )
    end

    def to_h
      {
        type: type,
        lexeme: lexeme,
        literal: literal,
        line: line,
        column: column,
        end_line: span.end_line,
        end_column: span.end_column,
        span: span.to_h
      }
    end
  end
end
