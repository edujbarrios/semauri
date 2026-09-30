# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "diagnostics/formatter"

module Semauri
  class Error < StandardError
    attr_reader :code, :line, :column, :end_line, :end_column, :hint

    def initialize(message, code: "S000", line: nil, column: nil, end_line: nil, end_column: nil, hint: nil)
      super(message)
      @code = code
      @line = line
      @column = column
      @end_line = end_line
      @end_column = end_column
      @hint = hint
    end

    def diagnostic(source: nil)
      return Diagnostics::Formatter.format(self, source) if source

      location = line ? " at #{line}:#{column || 1}" : ""
      text = "#{code}#{location}: #{message}"
      hint ? "#{text}\nHint: #{hint}" : text
    end
  end

  class LexError < Error; end
  class ParseError < Error; end
  class SemanticError < Error; end
  class BackendError < Error; end
end
