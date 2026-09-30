# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Vocabulary
    class English
      COLORS = %w[black blue brown gray green orange pink purple red white yellow].freeze

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
        "it" => :PRONOUN
      }.freeze

      def token_type(word)
        normalized = word.downcase
        return :COLOR if COLORS.include?(normalized)

        KEYWORDS.fetch(normalized, :WORD)
      end
    end
  end
end
