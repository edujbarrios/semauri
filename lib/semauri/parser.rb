# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "errors"
require_relative "ast/program"
require_relative "ast/create_web"
require_relative "ast/set_title"
require_relative "ast/add_element"
require_relative "ast/pronoun_reference"
require_relative "ast/set_property"

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
      return add_statement if match?(:ADD)

      error!(peek, "Expected a statement beginning with 'Create', 'Make' or 'Add'", "S201")
    end

    def create_statement(start)
      match?(:ARTICLE)
      consume(:WEB, "Expected 'web', 'website' or 'page' after '#{start.lexeme}'", "S202")

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

    def make_statement(start)
      return create_statement(start) unless check?(:PRONOUN)

      pronoun = advance
      color = consume(:COLOR, "Expected a supported color after '#{pronoun.lexeme}'", "S206")
      consume_optional_dot

      target = AST::PronounReference.new(pronoun: pronoun.lexeme, line: pronoun.line, column: pronoun.column)
      AST::SetProperty.new(target: target, property: :color, value: color.lexeme.downcase,
                           line: start.line, column: start.column)
    end

    def add_statement
      start = previous
      match?(:ARTICLE)

      return add_title(start) if match?(:TITLE)
      return add_element(start, :button) if match?(:BUTTON)
      return add_element(start, :image) if match?(:IMAGE)

      error!(peek, "Expected 'title', 'button' or 'image' after 'Add'", "S203")
    end

    def add_title(start)
      consume(:CALLED, "Expected 'called' or 'named' after 'title'", "S204")
      title = phrase_until(:DOT, :EOF)
      consume_optional_dot
      AST::SetTitle.new(title: normalize_phrase(title), line: start.line, column: start.column)
    end

    def add_element(start, kind)
      label = nil
      label = phrase_until(:DOT, :EOF) if match?(:CALLED)
      consume_optional_dot
      AST::AddElement.new(kind: kind, label: normalize_phrase(label), line: start.line, column: start.column)
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

      value.strip.gsub(/\s+/, " ").split.map do |word|
        word.match?(/\A[A-Z0-9]+\z/) ? word : word.capitalize
      end.join(" ")
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
