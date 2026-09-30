# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  class Error < StandardError
    attr_reader :code, :line, :column, :hint

    def initialize(message, code: "S000", line: nil, column: nil, hint: nil)
      super(message)
      @code = code
      @line = line
      @column = column
      @hint = hint
    end

    def diagnostic
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
