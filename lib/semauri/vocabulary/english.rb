# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Vocabulary
    class English
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
        "color" => :COLOR_PROPERTY,
        "a" => :ARTICLE,
        "an" => :ARTICLE,
        "the" => :ARTICLE,
        "web" => :WEB,
        "website" => :WEB,
        "webpage" => :WEB,
        "page" => :WEB,
        "called" => :CALLED,
        "named" => :CALLED,
        "add" => :ADD,
        "title" => :TITLE,
        "button" => :BUTTON,
        "image" => :IMAGE,
        "picture" => :IMAGE,
        "it" => :PRONOUN,
        "black" => :COLOR,
        "white" => :COLOR,
        "red" => :COLOR,
        "green" => :COLOR,
        "blue" => :COLOR,
        "yellow" => :COLOR,
        "orange" => :COLOR,
        "purple" => :COLOR,
        "pink" => :COLOR,
        "gray" => :COLOR,
        "grey" => :COLOR,
        "brown" => :COLOR
      }.freeze

      def token_type(word)
        KEYWORDS.fetch(word.downcase, :WORD)
      end
    end
  end
end
