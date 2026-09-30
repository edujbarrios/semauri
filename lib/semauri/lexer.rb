# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "token"
require_relative "errors"
require_relative "vocabulary/english"

module Semauri
  class Lexer
    PUNCTUATION = {
      "." => :DOT,
      "," => :COMMA,
      ":" => :COLON
    }.freeze

    def initialize(source, vocabulary: Vocabulary::English.new)
      @source = source
      @vocabulary = vocabulary
      @index = 0
      @line = 1
      @column = 1
    end

    def tokens
      result = []
      result << next_token until result.last&.type == :EOF
      result
    end

    private

    def next_token
      skip_whitespace
      return token(:EOF, "") if eof?

      start_line = @line
      start_column = @column
      char = current

      if PUNCTUATION.key?(char)
        advance
        return Token.new(type: PUNCTUATION.fetch(char), lexeme: char, literal: nil,
                         line: start_line, column: start_column)
      end

      return string_token(start_line, start_column) if char == '"'
      return word_token(start_line, start_column) if word_start?(char)

      raise LexError.new(
        "Unexpected character #{char.inspect}",
        code: "S101",
        line: start_line,
        column: start_column,
        hint: "Semauri 0.1 currently accepts words, quoted strings, '.', ',' and ':'."
      )
    end

    def word_token(line, column)
      text = +""
      text << advance while !eof? && word_char?(current)
      type = @vocabulary.token_type(text)
      Token.new(type: type, lexeme: text, literal: type == :WORD ? text : nil, line: line, column: column)
    end

    def string_token(line, column)
      advance
      value = +""

      until eof? || current == '"'
        if current == "\n"
          raise LexError.new("Strings cannot span multiple lines yet", code: "S102", line: line, column: column)
        end
        value << advance
      end

      raise LexError.new("Unterminated string", code: "S103", line: line, column: column) if eof?

      advance
      Token.new(type: :STRING, lexeme: value, literal: value, line: line, column: column)
    end

    def skip_whitespace
      while !eof? && current.match?(/\s/)
        advance
      end
    end

    def word_start?(char)
      char.match?(/[[:alpha:]]/)
    end

    def word_char?(char)
      char.match?(/[[:alnum:]_'-]/)
    end

    def current
      @source[@index]
    end

    def eof?
      @index >= @source.length
    end

    def advance
      char = @source[@index]
      @index += 1
      if char == "\n"
        @line += 1
        @column = 1
      else
        @column += 1
      end
      char
    end

    def token(type, lexeme)
      Token.new(type: type, lexeme: lexeme, literal: nil, line: @line, column: @column)
    end
  end
end
