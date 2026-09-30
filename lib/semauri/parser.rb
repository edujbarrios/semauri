# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "errors"
require_relative "ast/program"
require_relative "ast/create_web"
require_relative "ast/set_title"
require_relative "ast/create_element"
require_relative "ast/set_property"
require_relative "ast/reference"

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
      return create_statement(previous) if match?(:CREATE)
      return make_statement(previous) if match?(:MAKE)
      return add_statement(previous) if match?(:ADD)

      error!(peek, "Expected a statement beginning with 'Create', 'Add' or 'Make'", "S201")
    end

    def create_statement(start)
      match?(:ARTICLE)
      return create_web_statement(start) if match?(:WEB)
      return create_element_statement(start, :button) if match?(:BUTTON)

      error!(peek, "Expected 'web' or 'button' after 'Create'", "S202")
    end

    def make_statement(start)
      if web_creation_ahead?
        match?(:ARTICLE)
        consume(:WEB, "Expected a web artifact", "S202")
        return create_web_statement(start)
      end

      target = reference
      color = consume(:COLOR, "Expected a supported color after the target", "S206")
      consume_optional_dot

      AST::SetProperty.new(
        target: target,
        property: :color,
        value: color.literal.downcase,
        line: start.line,
        column: start.column
      )
    end

    def add_statement(start)
      match?(:ARTICLE)

      if match?(:TITLE)
        consume(:CALLED, "Expected 'called' or 'named' after 'title'", "S204")
        title = phrase_until(:DOT, :EOF)
        consume_optional_dot
        return AST::SetTitle.new(title: normalize_phrase(title), line: start.line, column: start.column)
      end

      return create_element_statement(start, :button) if match?(:BUTTON)

      error!(peek, "Expected 'title' or 'button' after 'Add'", "S203")
    end

    def create_web_statement(start)
      subject = nil
      title = nil

      if match?(:CALLED)
        title = phrase_until(:DOT, :EOF)
      elsif match?(:FOR)
        match?(:ARTICLE)
        subject = phrase_until(:DOT, :EOF)
      end

      consume_optional_dot
      AST::CreateWeb.new(
        subject: normalize_phrase(subject),
        title: normalize_phrase(title),
        line: start.line,
        column: start.column
      )
    end

    def create_element_statement(start, element_type)
      name = match?(:CALLED) ? normalize_phrase(phrase_until(:DOT, :EOF)) : nil
      consume_optional_dot
      AST::CreateElement.new(element_type: element_type, name: name, line: start.line, column: start.column)
    end

    def reference
      return AST::Reference.pronoun if match?(:PRONOUN)

      match?(:ARTICLE)
      consume(:BUTTON, "Expected 'it' or a supported element such as 'button'", "S207")

      if match?(:CALLED)
        name = normalize_phrase(phrase_until(:COLOR, :DOT, :EOF))
        return AST::Reference.named(:button, name)
      end

      AST::Reference.kind(:button)
    end

    def web_creation_ahead?
      return true if check?(:WEB)

      check?(:ARTICLE) && peek_next.type == :WEB
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

    def peek_next
      @tokens[[@current + 1, @tokens.length - 1].min]
    end

    def previous
      @tokens[@current - 1]
    end

    def error!(token, message, code)
      raise ParseError.new(message, code: code, line: token.line, column: token.column)
    end
  end
end
