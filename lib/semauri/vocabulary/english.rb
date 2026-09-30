# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Vocabulary
    class English
      KEYWORDS = {
        "create" => :CREATE,
        "make" => :MAKE,
        "a" => :ARTICLE,
        "an" => :ARTICLE,
        "the" => :ARTICLE,
        "web" => :WEB,
        "website" => :WEB,
        "webpage" => :WEB,
        "page" => :WEB,
        "for" => :FOR,
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
