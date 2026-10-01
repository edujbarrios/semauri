# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class DomainScopesTest < Minitest::Test
  class ArchiveDomain < Semauri::Domains::Definition
    def initialize
      super(
        name: :archive,
        operations: [
          Semauri::Domains::Operation.new(
            name: :delete,
            verbs: ["delete"],
            pattern: [Semauri::Domains::Operation.expression(:entry, type: :string)],
            effects: [:archive_write]
          )
        ]
      )
    end
  end

  def registry_with_collision
    Semauri::Domains::Registry.new
      .register(Semauri::Domains::Filesystem.new)
      .register(ArchiveDomain.new)
  end

  def test_unique_action_remains_available_without_domain_scope
    compiler = Semauri::Compiler.new
    ast = compiler.parse('Write "hello" to "notes.txt".')

    operation = ast.statements.first
    assert_instance_of Semauri::AST::DomainOperation, operation
    assert_equal :filesystem, operation.domain
    assert_equal :write, operation.operation
  end

  def test_shared_action_requires_explicit_domain_scope
    compiler = Semauri::Compiler.new(domains: registry_with_collision)

    error = assert_raises(Semauri::ParseError) do
      compiler.parse('Delete "tmp.log".')
    end

    assert_equal "S240", error.code
    assert_includes error.message, "Ambiguous domain action"
    assert_includes error.hint, "filesystem"
    assert_includes error.hint, "archive"
  end

  def test_within_scope_selects_filesystem_action_deterministically
    compiler = Semauri::Compiler.new(domains: registry_with_collision)
    result = compiler.hir(<<~SEMA)
      Within filesystem:
        Delete "tmp.log".
      End.
    SEMA

    scope = result.program.fields.fetch(:statements).first
    operation = scope.fields.fetch(:body).fields.fetch(:statements).first

    assert_equal :domain_scope, scope.kind
    assert_equal :filesystem, scope.fields.fetch(:domain)
    assert_equal :domain_operation, operation.kind
    assert_equal :filesystem, operation.fields.fetch(:domain)
    assert_equal :delete, operation.fields.fetch(:operation)
  end

  def test_same_surface_action_can_resolve_to_another_domain
    compiler = Semauri::Compiler.new(domains: registry_with_collision)
    result = compiler.hir(<<~SEMA)
      Within archive:
        Delete "old-entry".
      End.
    SEMA

    scope = result.program.fields.fetch(:statements).first
    operation = scope.fields.fetch(:body).fields.fetch(:statements).first

    assert_equal :archive, scope.fields.fetch(:domain)
    assert_equal :archive, operation.fields.fetch(:domain)
    assert_equal :delete, operation.fields.fetch(:operation)
  end

  def test_scope_rejects_action_that_is_not_available_in_that_domain
    compiler = Semauri::Compiler.new(domains: registry_with_collision)

    error = assert_raises(Semauri::ParseError) do
      compiler.parse(<<~SEMA)
        Within archive:
          Write "hello" to "notes.txt".
        End.
      SEMA
    end

    assert_equal "S240", error.code
    assert_includes error.message, "not available"
    assert_includes error.hint, "filesystem"
  end

  def test_domain_scope_is_a_lexical_variable_scope
    compiler = Semauri::Compiler.new

    error = assert_raises(Semauri::SemanticError) do
      compiler.hir(<<~SEMA)
        Within filesystem:
          Let local be "hello".
          Write local to "notes.txt".
        End.
        Write local to "outside.txt".
      SEMA
    end

    assert_equal "S312", error.code
  end
end
