# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../ir/runtime_plan"
require_relative "../ir/runtime_value_ref"
require_relative "../semantics/result"

module Semauri
  module HIR
    module RuntimeLowering
      private

      def lower_let(node)
        value = lower_runtime_value(node.fields.fetch(:value))
        symbol = symbol!(node.fields.fetch(:symbol_id), node)
        validate_symbol_type!(symbol, value, node)
        @environment.bind(symbol.id, value, node: node)
        display_name = node.fields[:source_name] || symbol.name
        @explanations << "Bound '#{display_name}' as symbol ##{symbol.id} to #{value.describe}."
      end

      def lower_domain_operation(node)
        domain = @domains.fetch(node.fields.fetch(:domain))
        if (scope = @domain_scope_stack.last) && scope != domain.name
          raise semantic_error(
            node,
            "Operation '#{node.fields.fetch(:operation)}' belongs to domain '#{domain.name}', but the active semantic scope is '#{scope}'",
            "S332"
          )
        end

        operation = domain.operation(node.fields.fetch(:operation))
        unless operation.return_type == :unit && !runtime_dependent?(node.fields.fetch(:arguments))
          return append_runtime_operation(node, domain, operation)
        end

        super
      end

      def evaluate(node)
        if runtime_dependent?(node)
          raise semantic_error(
            node,
            "Runtime values cannot be evaluated during compilation",
            "S335",
            hint: "Use runtime values only as direct operation inputs for now. Runtime arithmetic and control flow require the planned CFG/SSA lowering stage."
          )
        end
        super
      end

      def lower_runtime_value(node)
        case node.kind
        when :domain_operation
          append_runtime_operation(node, @domains.fetch(node.fields.fetch(:domain)),
                                   @domains.fetch(node.fields.fetch(:domain)).operation(node.fields.fetch(:operation)))
        when :symbol_ref
          bound = @environment.resolve(node.fields.fetch(:symbol_id), node: node)
          return bound if bound.is_a?(IR::RuntimeValueRef)
          evaluate(node)
        when :promote
          value = lower_runtime_value(node.fields.fetch(:value))
          return value unless value.is_a?(IR::RuntimeValueRef)

          IR::RuntimeValueRef.new(
            id: value.id,
            type: node.type,
            producer_id: value.producer_id,
            source_span: node.span
          )
        else
          evaluate(node)
        end
      end

      def append_runtime_operation(node, domain, operation)
        arguments = node.fields.fetch(:arguments).transform_values do |argument|
          value = lower_runtime_value(argument)
          value.is_a?(Semantics::Value) ? value.value : value
        end.freeze

        @runtime_plan, result = @runtime_plan.append(
          domain: domain.name,
          name: operation.name,
          arguments: arguments,
          return_type: operation.return_type,
          effects: operation.effects,
          source_span: node.span
        )

        if result
          @explanations << "Planned runtime operation '#{domain.name}.#{operation.name}' as operation ##{@runtime_plan.size}; result is #{result.id} (#{result.type})."
        else
          @explanations << "Planned runtime operation '#{domain.name}.#{operation.name}' as operation ##{@runtime_plan.size}."
        end
        result || Semantics::Value.new(type: :unit, value: nil, definition_span: node.span)
      end

      def runtime_dependent?(value)
        case value
        when HIR::Node
          return true if value.kind == :domain_operation
          if value.kind == :symbol_ref
            bound = @environment.resolve(value.fields.fetch(:symbol_id), node: value)
            return bound.is_a?(IR::RuntimeValueRef)
          end
          value.fields.any? { |_name, field| runtime_dependent?(field) }
        when Array
          value.any? { |item| runtime_dependent?(item) }
        when Hash
          value.any? { |_name, item| runtime_dependent?(item) }
        else
          false
        end
      end
    end
  end
end
