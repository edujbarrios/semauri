# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "token"
require_relative "errors"
require_relative "vocabulary/english"

module Semauri
  class Lexer
    PUNCTUATION = { "." => :DOT, "," => :COMMA, ":" => :COLON, "(" => :LPAREN, ")" => :RPAREN }.freeze

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
                         line: start_line, column: start_column, end_line: @line, end_column: @column)
      end

      return string_token(start_line, start_column) if char == '"'
      return number_token(start_line, start_column) if digit?(char)
      return word_token(start_line, start_column) if word_start?(char)

      raise LexError.new("Unexpected character #{char.inspect}", code: "S101",
                         line: start_line, column: start_column,
                         end_line: start_line, end_column: start_column + 1)
    end

    def word_token(line, column)
      text = +""
      text << advance while !eof? && word_char?(current)
      classification = @vocabulary.classify(text)
      Token.new(type: classification.type, lexeme: text, literal: classification.literal,
                line: line, column: column, end_line: @line, end_column: @column)
    end

    def number_token(line, column)
      text = +""
      text << advance while !eof? && digit?(current)
      if !eof? && current == "." && digit?(peek_char)
        text << advance
        text << advance while !eof? && digit?(current)
      end
      literal = text.include?(".") ? Float(text) : Integer(text)
      Token.new(type: :NUMBER, lexeme: text, literal: literal, line: line, column: column,
                end_line: @line, end_column: @column)
    end

    def string_token(line, column)
      advance
      value = +""
      until eof? || current == '"'
        if current == "\n"
          raise LexError.new("Strings cannot span multiple lines yet", code: "S102",
                             line: line, column: column, end_line: @line, end_column: @column)
        end
        value << advance
      end
      if eof?
        raise LexError.new("Unterminated string", code: "S103", line: line, column: column,
                           end_line: @line, end_column: @column)
      end
      advance
      Token.new(type: :STRING, lexeme: value, literal: value, line: line, column: column,
                end_line: @line, end_column: @column)
    end

    def skip_whitespace
      advance while !eof? && current.match?(/\s/)
    end

    def digit?(char) = char && char.match?(/[0-9]/)
    def word_start?(char) = char.match?(/[[:alpha:]]/)
    def word_char?(char) = char.match?(/[[:alnum:]_'-]/)
    def current = @source[@index]
    def peek_char = @source[@index + 1]
    def eof? = @index >= @source.length

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
      Token.new(type: type, lexeme: lexeme, literal: nil, line: @line, column: @column,
                end_line: @line, end_column: @column)
    end
  end
end
