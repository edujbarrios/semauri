# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class DomainOperationsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_filesystem_operations_compile_to_a_plan_and_default_backend
    source = <<~SEMA
      Let message be "hello from Semauri".
      Write message to "notes.txt".
      Copy "notes.txt" to "backup.txt".
      Delete "old.tmp".
    SEMA

    result = @compiler.compile(source)

    assert_equal "posix-sh", result.backend
    assert_equal :filesystem, result.ir.domain
    assert_equal %i[write copy delete], result.ir.operations.map(&:name)
    assert_includes result.output, "#!/usr/bin/env sh"
    assert_includes result.output, "notes.txt"
    assert_includes result.output, "backup.txt"
    assert_includes result.output, "rm -f old.tmp"
  end

  def test_domain_operation_hir_preserves_effect_and_nominal_type_metadata
    hir = @compiler.hir('Write "hello" to "notes.txt".')
    operation = hir.program.fields.fetch(:statements).first
    path = operation.fields.fetch(:arguments).fetch(:path)

    assert_equal :domain_operation, operation.kind
    assert_equal :filesystem, operation.fields.fetch(:domain)
    assert_equal :write, operation.fields.fetch(:operation)
    assert_equal %i[filesystem_write], operation.fields.fetch(:effects)
    assert_equal :promote, path.kind
    assert_equal Semauri::Domains::Filesystem::PATH, path.type
    assert_equal :string, path.fields.fetch(:from_type)
    assert_equal :string, path.fields.fetch(:value).type
    assert_equal "notes.txt", path.fields.fetch(:value).fields.fetch(:value)
  end

  def test_operation_argument_types_are_checked_before_lowering
    error = assert_raises(Semauri::SemanticError) do
      @compiler.hir('Write 42 to "notes.txt".')
    end

    assert_equal "S329", error.code
  end

  def test_operation_phrase_connectors_are_deterministic
    error = assert_raises(Semauri::ParseError) do
      @compiler.parse('Write "hello" into "notes.txt".')
    end

    assert_equal "S238", error.code
  end

  def test_external_domain_can_add_a_new_verb_without_parser_changes
    echo_domain = Class.new(Semauri::Domains::Definition) do
      def initialize
        super(
          name: :echo_test,
          operations: [
            Semauri::Domains::Operation.new(
              name: :emit,
              verbs: ["emit"],
              pattern: [Semauri::Domains::Operation.expression(:message, type: :string)],
              effects: []
            )
          ]
        )
      end
    end

    registry = Semauri::Domains::Registry.new.register(echo_domain.new)
    compiler = Semauri::Compiler.new(domains: registry)
    hir = compiler.hir('Emit "hello".')
    operation = hir.program.fields.fetch(:statements).first

    assert_equal :domain_operation, operation.kind
    assert_equal :echo_test, operation.fields.fetch(:domain)
    assert_equal :emit, operation.fields.fetch(:operation)
    assert_equal "hello", operation.fields.fetch(:arguments).fetch(:message).fields.fetch(:value)
  end

  def test_optimizer_propagates_constants_inside_nominal_promotion_without_removing_effectful_operation
    source = <<~SEMA
      Let path be "notes.txt".
      Write "hello" to path.
    SEMA

    optimized = @compiler.optimized_hir(source)
    statements = optimized.program.fields.fetch(:statements)
    operation = statements.find { |statement| statement.kind == :domain_operation }
    promoted_path = operation.fields.fetch(:arguments).fetch(:path)

    refute_nil operation
    assert_equal :promote, promoted_path.kind
    assert_equal Semauri::Domains::Filesystem::PATH, promoted_path.type
    assert_equal "notes.txt", promoted_path.fields.fetch(:value).fields.fetch(:value)
    assert_equal [:filesystem_write], operation.fields.fetch(:effects)
  end
end
