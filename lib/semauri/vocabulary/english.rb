# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Vocabulary
    class English
      KEYWORDS = {
        "create" => :CREATE,
        "make" => :CREATE,
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
        "title" => :TITLE
      }.freeze

      def token_type(word)
        KEYWORDS.fetch(word.downcase, :WORD)
      end
    end
  end
end
