# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  Token = Struct.new(:type, :lexeme, :literal, :line, :column, keyword_init: true) do
    def to_h
      { type: type, lexeme: lexeme, literal: literal, line: line, column: column }
    end
  end
end
