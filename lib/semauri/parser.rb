# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "errors"
require_relative "ast/program"
require_relative "ast/create_web"
require_relative "ast/set_title"

module Semauri
  class Parser
    def initialize(tokens)
      @tokens = tokens
      @current = 0
    end

    def parse
      statements = []
      statements << statement until check?(:EOF)
      AST::Program.new(statements: statements)
    end

    private

    def statement
      return create_statement if match?(:CREATE)
      return add_statement if match?(:ADD)

      error!(peek, "Expected a statement beginning with 'Create' or 'Add'", "S201")
    end

    def create_statement
      start = previous
      match?(:ARTICLE)
      consume(:WEB, "Expected 'web', 'website' or 'page' after 'Create'", "S202")

      subject = nil
      title = nil

      if match?(:CALLED)
        title = phrase_until(:DOT, :EOF)
      elsif match?(:FOR)
        match?(:ARTICLE)
        subject = phrase_until(:DOT, :EOF)
      end

      consume_optional_dot
      AST::CreateWeb.new(subject: normalize_phrase(subject), title: normalize_phrase(title),
                         line: start.line, column: start.column)
    end

    def add_statement
      start = previous
      match?(:ARTICLE)
      consume(:TITLE, "Semauri 0.1 currently supports 'Add a title called ...'", "S203")
      consume(:CALLED, "Expected 'called' or 'named' after 'title'", "S204")
      title = phrase_until(:DOT, :EOF)
      consume_optional_dot

      AST::SetTitle.new(title: normalize_phrase(title), line: start.line, column: start.column)
    end

    def phrase_until(*terminators)
      parts = []
      until terminators.include?(peek.type)
        token = advance
        parts << (token.literal || token.lexeme)
      end

      error!(peek, "Expected a name or description", "S205") if parts.empty?
      parts.join(" ")
    end

    def normalize_phrase(value)
      return nil unless value

      value.strip.gsub(/\s+/, " ").split.map { |word| word.match?(/\A[A-Z0-9]+\z/) ? word : word.capitalize }.join(" ")
    end

    def consume_optional_dot
      advance if check?(:DOT)
    end

    def consume(type, message, code)
      return advance if check?(type)
      error!(peek, message, code)
    end

    def match?(*types)
      return false unless types.any? { |type| check?(type) }
      advance
      true
    end

    def check?(type)
      peek.type == type
    end

    def advance
      @current += 1 unless at_end?
      previous
    end

    def at_end?
      peek.type == :EOF
    end

    def peek
      @tokens[@current]
    end

    def previous
      @tokens[@current - 1]
    end

    def error!(token, message, code)
      raise ParseError.new(message, code: code, line: token.line, column: token.column)
    end
  end
end
