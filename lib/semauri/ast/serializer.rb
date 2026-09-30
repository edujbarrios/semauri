# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module AST
    class Serializer
      def serialize(node) = node.accept(self)

      def visit_program(node)
        { type: "program", statements: node.statements.map { |statement| statement.accept(self) }, span: node.span.to_h }
      end

      def visit_block(node)
        { type: "block", statements: node.statements.map { |statement| statement.accept(self) } }.merge(location(node))
      end

      def visit_if_statement(node)
        { type: "if_statement", condition: node.condition.accept(self), consequence: node.consequence.accept(self),
          alternative: node.alternative&.accept(self) }.merge(location(node))
      end

      def visit_unary_expression(node)
        { type: "unary_expression", operator: node.operator, operand: node.operand.accept(self) }.merge(location(node))
      end

      def visit_binary_expression(node)
        { type: "binary_expression", operator: node.operator, left: node.left.accept(self),
          right: node.right.accept(self) }.merge(location(node))
      end

      def visit_create_web(node) = { type: "create_web", subject: node.subject, title: node.title }.merge(location(node))
      def visit_set_title(node) = { type: "set_title", title: node.title }.merge(location(node))
      def visit_add_element(node) = { type: "add_element", kind: node.kind, label: node.label }.merge(location(node))
      def visit_pronoun_reference(node) = { type: "pronoun_reference", pronoun: node.pronoun }.merge(location(node))
      def visit_named_reference(node) = { type: "named_reference", kind: node.kind, label: node.label }.merge(location(node))
      def visit_literal(node) = { type: "literal", value_type: node.value_type, value: node.value }.merge(location(node))
      def visit_variable_reference(node) = { type: "variable_reference", name: node.name }.merge(location(node))
      def visit_let_binding(node) = { type: "let_binding", name: node.name, value: node.value.accept(self) }.merge(location(node))

      def visit_set_property(node)
        { type: "set_property", target: node.target.accept(self), property: node.property,
          value: node.value.accept(self) }.merge(location(node))
      end

      private

      def location(node)
        { line: node.line, column: node.column, end_line: node.end_line, end_column: node.end_column, span: node.span.to_h }
      end
    end
  end
end
