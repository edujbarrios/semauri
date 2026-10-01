# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class ParserTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_parses_subject_based_web_as_domain_artifact
    program = @compiler.parse("Create a web for a pet store.")
    statement = program.statements.first

    assert_instance_of Semauri::AST::CreateArtifact, statement
    assert_equal :web, statement.domain
    assert_equal :web, statement.kind
    assert_equal "Pet Store", statement.subject
    assert_nil statement.title
  end

  def test_parses_explicit_title
    program = @compiler.parse("Create a web called Hello World.")
    statement = program.statements.first

    assert_equal :web, statement.domain
    assert_equal "Hello World", statement.title
  end
end
