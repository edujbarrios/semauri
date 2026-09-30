# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"
require_relative "result"
require_relative "../errors"
require_relative "../semantics/symbol_table"
require_relative "../semantics/type_system"

module Semauri
  module HIR
    class Builder
      class Environment
        def initialize(parent: nil)
          @parent = parent
          @symbols = {}
        end

        def child = self.class.new(parent: self)

        def define(symbol, node:)
          key = symbol.name
          if @symbols.key?(key)
            raise SemanticError.new(
              "Variable '#{symbol.name}' is already defined in this scope",
              code: "S311",
              line: node.line, column: node.column,
              end_line: node.end_line, end_column: node.end_column,
              hint: "Choose a different name or reuse the existing variable."
            )
          end
          @symbols[key] = symbol
        end

        def resolve(reference)
          key = reference.name.to_s.downcase
          return @symbols[key] if @symbols.key?(key)
          return @parent.resolve(reference) if @parent

          raise SemanticError.new(
            "Unknown variable '#{reference.name}'",
            code: "S312",
            line: reference.line, column: reference.column,
            end_line: reference.end_line, end_column: reference.end_column,
            hint: "Declare it first with 'Let #{reference.name} be ...'."
          )
        end
      end

      def build(ast)
        @symbols = Semantics::SymbolTable.new
        @environment = Environment.new
        program = ast.accept(self)
        Result.new(program: program, symbols: @symbols.symbols)
      ensure
        @symbols = @environment = nil
      end

      def visit_program(node)
        HIR::Node.new(kind: :program, type: :unit,
                      fields: { statements: node.statements.map { |statement| statement.accept(self) } },
                      span: node.span)
      end

      def visit_block(node)
        with_child_environment do
          HIR::Node.new(kind: :block, type: :unit,
                        fields: { statements: node.statements.map { |statement| statement.accept(self) } },
                        span: node.span)
        end
      end

      def visit_let_binding(node)
        value = node.value.accept(self)
        symbol = @symbols.create(name: node.name, kind: :variable, type: value.type, definition_span: node.span)
        @environment.define(symbol, node: node)

        HIR::Node.new(kind: :let, type: :unit,
                      fields: { symbol_id: symbol.id, name: symbol.name, value: value },
                      span: node.span)
      end

      def visit_literal(node)
        HIR::Node.new(kind: :literal, type: node.value_type,
                      fields: { value: node.value }, span: node.span)
      end

      def visit_variable_reference(node)
        symbol = @environment.resolve(node)
        HIR::Node.new(kind: :symbol_ref, type: symbol.type,
                      fields: { symbol_id: symbol.id, name: symbol.name }, span: node.span)
      end

      def visit_unary_expression(node)
        operand = node.operand.accept(self)
        result_type = Semantics::TypeSystem.unary_type(node.operator, operand.type, node: node)
        HIR::Node.new(kind: :unary, type: result_type,
                      fields: { operator: node.operator, operand: operand }, span: node.span)
      end

      def visit_binary_expression(node)
        left = node.left.accept(self)
        right = node.right.accept(self)
        result_type = Semantics::TypeSystem.binary_type(node.operator, left.type, right.type, node: node)
        HIR::Node.new(kind: :binary, type: result_type,
                      fields: { operator: node.operator, left: left, right: right }, span: node.span)
      end

      def visit_if_statement(node)
        condition = node.condition.accept(self)
        Semantics::TypeSystem.ensure_boolean!(condition.type, node: node.condition,
                                               message: "If condition must evaluate to boolean")
        consequence = node.consequence.accept(self)
        alternative = node.alternative&.accept(self)

        HIR::Node.new(kind: :if, type: :unit,
                      fields: { condition: condition, consequence: consequence, alternative: alternative },
                      span: node.span)
      end

      def visit_create_web(node)
        HIR::Node.new(kind: :create_web, type: :unit,
                      fields: { subject: node.subject, title: node.title }, span: node.span)
      end

      def visit_set_title(node)
        HIR::Node.new(kind: :set_title, type: :unit,
                      fields: { title: node.title }, span: node.span)
      end

      def visit_add_element(node)
        HIR::Node.new(kind: :add_element, type: :unit,
                      fields: { element_kind: node.kind, label: node.label }, span: node.span)
      end

      def visit_pronoun_reference(node)
        HIR::Node.new(kind: :pronoun_reference, type: :entity_ref,
                      fields: { pronoun: node.pronoun }, span: node.span)
      end

      def visit_named_reference(node)
        HIR::Node.new(kind: :named_reference, type: :entity_ref,
                      fields: { entity_kind: node.kind, label: node.label }, span: node.span)
      end

      def visit_set_property(node)
        target = node.target.accept(self)
        value = node.value.accept(self)
        Semantics::TypeSystem.validate_property!(node.property, value.type, node: node.value)

        HIR::Node.new(kind: :set_property, type: :unit,
                      fields: { target: target, property: node.property, value: value }, span: node.span)
      end

      private

      def with_child_environment
        parent = @environment
        @environment = parent.child
        yield
      ensure
        @environment = parent
      end
    end
  end
end
