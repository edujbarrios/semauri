# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "errors"
require_relative "source_span"
require_relative "ast/program"
require_relative "ast/create_web"
require_relative "ast/set_title"
require_relative "ast/add_element"
require_relative "ast/pronoun_reference"
require_relative "ast/named_reference"
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
      AST::Program.new(statements: statements, span: program_span(statements))
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
      AST::CreateWeb.new(subject: normalize_phrase(subject), title: normalize_phrase(title), span: span_from(start))
    end

    def make_statement(start)
      return pronoun_property_statement(start) if check?(:PRONOUN)
      return named_property_statement(start) if named_reference_ahead?

      create_statement(start)
    end

    def pronoun_property_statement(start)
      pronoun = advance
      color = consume(:COLOR, "Expected a supported color after '#{pronoun.lexeme}'", "S206")
      consume_optional_dot

      target = AST::PronounReference.new(pronoun: pronoun.lexeme, span: pronoun.span)
      AST::SetProperty.new(target: target, property: :color, value: color.lexeme.downcase, span: span_from(start))
    end

    def named_property_statement(start)
      match?(:ARTICLE)
      kind_token = advance
      kind = kind_token.type == :BUTTON ? :button : :image
      consume(:CALLED, "Expected 'called' or 'named' in an explicit reference", "S207")

      reference_tokens = tokens_until(:DOT, :EOF)
      error!(peek, "Expected a name and a color", "S208") if reference_tokens.length < 2

      color = reference_tokens.last
      unless color.type == :COLOR
        error!(color, "Expected a supported color after the referenced element", "S208")
      end

      label_tokens = reference_tokens[0...-1]
      label = phrase_from_tokens(label_tokens)
      consume_optional_dot

      target = AST::NamedReference.new(
        kind: kind,
        label: normalize_phrase(label),
        span: span_between(kind_token, label_tokens.last)
      )
      AST::SetProperty.new(target: target, property: :color, value: color.lexeme.downcase, span: span_from(start))
    end

    def named_reference_ahead?
      offset = check?(:ARTICLE) ? 1 : 0
      %i[BUTTON IMAGE].include?(@tokens[@current + offset]&.type)
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
      AST::SetTitle.new(title: normalize_phrase(title), span: span_from(start))
    end

    def add_element(start, kind)
      label = nil
      label = phrase_until(:DOT, :EOF) if match?(:CALLED)
      consume_optional_dot
      AST::AddElement.new(kind: kind, label: normalize_phrase(label), span: span_from(start))
    end

    def phrase_until(*terminators)
      tokens = tokens_until(*terminators)
      error!(peek, "Expected a name or description", "S205") if tokens.empty?
      phrase_from_tokens(tokens)
    end

    def tokens_until(*terminators)
      result = []
      result << advance until terminators.include?(peek.type)
      result
    end

    def phrase_from_tokens(tokens)
      tokens.map { |token| token.literal || token.lexeme }.join(" ")
    end

    def normalize_phrase(value)
      return nil unless value

      value.strip.gsub(/\s+/, " ").split.map do |word|
        word.match?(/\A[A-Z0-9]+\z/) ? word : word.capitalize
      end.join(" ")
    end

    def program_span(statements)
      return SourceSpan.point(1, 1) if statements.empty?

      SourceSpan.new(
        start_line: statements.first.line,
        start_column: statements.first.column,
        end_line: statements.last.end_line,
        end_column: statements.last.end_column
      )
    end

    def span_from(start)
      span_between(start, previous)
    end

    def span_between(first, last)
      SourceSpan.new(
        start_line: first.line,
        start_column: first.column,
        end_line: last.end_line,
        end_column: last.end_column
      )
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
      raise ParseError.new(message, code: code, line: token.line, column: token.column,
                           end_line: token.end_line, end_column: token.end_column)
    end
  end
end
