# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "set"
require_relative "../../node"

module Semauri
  module HIR
    module Optimization
      module Passes
        class DeadBindingElimination
          def name = "dead_binding_elimination"

          def run(program, symbols:)
            @changes = 0
            optimized, = optimize_scope(program, Set.new)
            [optimized, @changes]
          ensure
            @changes = nil
          end

          private

          def optimize_scope(node, live_out)
            case node.kind
            when :program, :block
              optimize_statement_list(node, live_out)
            else
              [node, live_out | symbol_refs(node)]
            end
          end

          def optimize_statement_list(node, live_out)
            live = live_out.dup
            kept = []

            node.fields.fetch(:statements).reverse_each do |statement|
              optimized, live = optimize_statement(statement, live)
              kept << optimized if optimized
            end

            [rebuild(node, statements: kept.reverse), live]
          end

          def optimize_statement(node, live_after)
            case node.kind
            when :let
              optimize_let(node, live_after)
            when :block
              optimize_scope(node, live_after)
            when :domain_scope
              optimize_domain_scope(node, live_after)
            when :if
              optimize_if(node, live_after)
            when :for_each
              optimize_for_each(node, live_after)
            else
              [node, live_after | symbol_refs(node)]
            end
          end

          def optimize_let(node, live_after)
            symbol_id = node.fields.fetch(:symbol_id)
            value = node.fields.fetch(:value)

            unless live_after.include?(symbol_id)
              return remove_dead_let(live_after) if removable_expression?(value)

              # A dead binding whose initializer can trap is observable. Keep it
              # and keep any symbols required to evaluate the initializer.
              return [node, live_after | symbol_refs(value)]
            end

            live_before = live_after.dup
            live_before.delete(symbol_id)
            live_before.merge(symbol_refs(value))
            [node, live_before]
          end

          def remove_dead_let(live_after)
            @changes += 1
            [nil, live_after]
          end

          def optimize_domain_scope(node, live_after)
            # Domain scopes are also lexical variable scopes: declarations inside
            # do not escape, but references to outer symbols must remain live.
            body, body_live = optimize_scope(node.fields.fetch(:body), Set.new)
            [rebuild(node, body: body), live_after | body_live]
          end

          def optimize_if(node, live_after)
            consequence, consequence_live = optimize_scope(node.fields.fetch(:consequence), live_after)

            if node.fields[:alternative]
              alternative, alternative_live = optimize_scope(node.fields.fetch(:alternative), live_after)
            else
              alternative = nil
              alternative_live = live_after.dup
            end

            live_before = consequence_live | alternative_live | symbol_refs(node.fields.fetch(:condition))
            [rebuild(node, consequence: consequence, alternative: alternative), live_before]
          end

          def optimize_for_each(node, live_after)
            iterator_id = node.fields.fetch(:iterator_symbol_id)
            body, body_live = optimize_scope(node.fields.fetch(:body), Set.new)
            body_live.delete(iterator_id)

            live_before = live_after | body_live | symbol_refs(node.fields.fetch(:iterable))
            [rebuild(node, body: body), live_before]
          end

          # Expressions are pure, but some can still fail during evaluation.
          # Dead-code elimination may only remove an initializer when evaluation
          # is guaranteed not to produce an observable compiler/runtime error.
          def removable_expression?(node)
            case node.kind
            when :literal, :symbol_ref
              true
            when :list
              node.fields.fetch(:items).all? { |item| removable_expression?(item) }
            when :unary
              removable_expression?(node.fields.fetch(:operand))
            when :binary
              left = node.fields.fetch(:left)
              right = node.fields.fetch(:right)
              return false unless removable_expression?(left) && removable_expression?(right)

              if node.fields.fetch(:operator) == :divide
                return right.kind == :literal && !right.fields.fetch(:value).zero?
              end

              true
            else
              false
            end
          end

          def symbol_refs(node)
            refs = Set.new
            collect_refs(node, refs)
            refs
          end

          def collect_refs(value, refs)
            case value
            when HIR::Node
              if value.kind == :symbol_ref
                refs << value.fields.fetch(:symbol_id)
                return
              end
              value.fields.each_value { |field| collect_refs(field, refs) }
            when Array
              value.each { |item| collect_refs(item, refs) }
            when Hash
              value.each_value { |item| collect_refs(item, refs) }
            end
          end

          def rebuild(node, **updates)
            fields = node.fields.merge(updates)
            return node if fields == node.fields

            HIR::Node.new(kind: node.kind, type: node.type, fields: fields, span: node.span)
          end
        end
      end
    end
  end
end
