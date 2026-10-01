# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class LexerTest < Minitest::Test
  def test_tokenizes_basic_web_sentence
    tokens = Semauri::Lexer.new("Create a web for a pet store.").tokens
    assert_equal %i[CREATE ARTICLE DOMAIN_ARTIFACT FOR ARTICLE WORD WORD DOT EOF], tokens.map(&:type)
    assert_equal({ domain: :web, category: :artifact, kind: :web }, tokens[2].literal)
    assert_equal "pet", tokens[5].lexeme
    assert_equal "store", tokens[6].lexeme
  end

  def test_supports_surface_synonyms
    tokens = Semauri::Lexer.new("Make a website named Hello.").tokens
    assert_equal %i[MAKE ARTICLE DOMAIN_ARTIFACT CALLED WORD DOT EOF], tokens.map(&:type)
    assert_equal :web, tokens[2].literal[:domain]
    assert_equal :web, tokens[2].literal[:kind]
  end

  def test_tokenizes_variable_declaration_and_number
    tokens = Semauri::Lexer.new("Let count be 2.5.").tokens
    assert_equal %i[LET WORD BE NUMBER DOT EOF], tokens.map(&:type)
    assert_equal 2.5, tokens[3].literal
  end
end
