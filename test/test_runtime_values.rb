# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class RuntimeValuesTest < Minitest::Test
  def setup
    response_type = Semauri::Semantics::NominalType.new(domain: :remote, name: :response, base_type: :string)

    remote_domain = Class.new(Semauri::Domains::Definition) do
      define_method(:initialize) do
        super(
          name: :remote,
          types: [response_type],
          operations: [
            Semauri::Domains::Operation.new(
              name: :fetch,
              verbs: ["fetch"],
              pattern: [Semauri::Domains::Operation.expression(:url, type: :string)],
              returns: response_type,
              effects: [:network]
            ),
            Semauri::Domains::Operation.new(
              name: :consume,
              verbs: ["consume"],
              pattern: [Semauri::Domains::Operation.expression(:response, type: response_type)],
              effects: [:storage_write]
            )
          ]
        )
      end
    end

    registry = Semauri::Domains::Registry.new.register(remote_domain.new)
    @compiler = Semauri::Compiler.new(domains: registry)
  end

  def source
    <<~SEMA
      Within remote:
        Let response be Fetch "https://example.test".
        Consume response.
      End.
    SEMA
  end

  def test_value_returning_operation_can_be_used_in_let_binding
    hir = @compiler.hir(source)
    scope = hir.program.fields.fetch(:statements).first
    binding = scope.fields.fetch(:body).fields.fetch(:statements).first
    operation = binding.fields.fetch(:value)

    assert_equal :domain_operation, operation.kind
    assert_equal :fetch, operation.fields.fetch(:operation)
    assert_equal "remote.response", operation.type.to_s
  end

  def test_runtime_plan_connects_operation_results_by_reference
    _ast, semantic = @compiler.analyze(source)
    plan = semantic.runtime_plan

    assert semantic.runtime?
    assert_equal 2, plan.size

    fetch = plan.operations.first
    consume = plan.operations.last
    assert_equal :fetch, fetch.name
    assert_equal "%1", fetch.result.id
    assert_equal "remote.response", fetch.result.type.to_s

    argument = consume.arguments.fetch(:response)
    assert_instance_of Semauri::IR::RuntimeValueRef, argument
    assert_equal "%1", argument.id
    assert_equal fetch.id, argument.producer_id
  end

  def test_compile_exposes_runtime_plan_without_executing_it
    result = @compiler.compile(source)

    assert result.runtime?
    assert_empty result.outputs
    assert_equal 2, result.runtime_plan.size
    assert_equal %i[network storage_write], result.effects.effects
  end

  def test_runtime_value_in_compile_time_control_flow_fails_explicitly
    program = <<~SEMA
      Within remote:
        Let response be Fetch "https://example.test".
        If response is equal to response:
          Consume response.
        End.
      End.
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(program) }

    assert_equal "S335", error.code
    assert_includes error.message, "Runtime values cannot be evaluated during compilation"
  end

  def test_runtime_plan_is_optimized_but_preserves_effectful_operations
    plan = @compiler.runtime_plan(source)

    assert_equal %i[fetch consume], plan.operations.map(&:name)
    assert_equal [:network], plan.operations.first.effects
    assert_equal [:storage_write], plan.operations.last.effects
  end
end
