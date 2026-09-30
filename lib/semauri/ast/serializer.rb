# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module AST
    class Serializer
      def serialize(node)
        node.accept(self)
      end

      def visit_program(node)
        { type: "program", statements: node.statements.map { |statement| statement.accept(self) } }
      end

      def visit_create_web(node)
        { type: "create_web", subject: node.subject, title: node.title, line: node.line, column: node.column }
      end

      def visit_set_title(node)
        { type: "set_title", title: node.title, line: node.line, column: node.column }
      end

      def visit_create_element(node)
        {
          type: "create_element",
          element_type: node.element_type,
          name: node.name,
          line: node.line,
          column: node.column
        }
      end

      def visit_set_property(node)
        {
          type: "set_property",
          target: node.target.to_h,
          property: node.property,
          value: node.value,
          line: node.line,
          column: node.column
        }
      end
    end
  end
end
