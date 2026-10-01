# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../domains/registry"

module Semauri
  module Vocabulary
    class English
      Classification = Struct.new(:type, :literal, keyword_init: true) do
        def initialize(type:, literal: nil)
          super(type: type.to_sym, literal: literal)
          freeze
        end
      end

      KEYWORDS = {
        "create" => :CREATE,
        "make" => :MAKE,
        "let" => :LET,
        "be" => :BE,
        "set" => :SET,
        "if" => :IF,
        "otherwise" => :OTHERWISE,
        "else" => :OTHERWISE,
        "end" => :END,
        "for" => :FOR,
        "every" => :EVERY,
        "in" => :IN,
        "list" => :LIST,
        "is" => :IS,
        "greater" => :GREATER,
        "less" => :LESS,
        "than" => :THAN,
        "equal" => :EQUAL,
        "and" => :AND,
        "or" => :OR,
        "not" => :NOT,
        "plus" => :PLUS,
        "minus" => :MINUS,
        "times" => :TIMES,
        "divided" => :DIVIDED,
        "by" => :BY,
        "true" => :BOOLEAN,
        "false" => :BOOLEAN,
        "of" => :OF,
        "to" => :TO,
        "a" => :ARTICLE,
        "an" => :ARTICLE,
        "the" => :ARTICLE,
        "called" => :CALLED,
        "named" => :CALLED,
        "add" => :ADD,
        "title" => :TITLE,
        "it" => :PRONOUN
      }.freeze

      COLORS = %w[black white red green blue yellow orange purple pink gray grey brown].freeze

      def initialize(domains: Domains::Registry.default)
        @domains = domains
      end

      def classify(word)
        normalized = word.to_s.downcase

        if (term = @domains.classify(normalized))
          return Classification.new(type: term.token_type, literal: term.to_h)
        end

        if COLORS.include?(normalized)
          return Classification.new(type: :COLOR, literal: normalized)
        end

        if (type = KEYWORDS[normalized])
          return Classification.new(type: type)
        end

        Classification.new(type: :WORD, literal: word)
      end

      # Compatibility API for callers that only need the lexical category.
      def token_type(word) = classify(word).type
    end
  end
end
