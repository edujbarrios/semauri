# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "../ir/web_document"
require_relative "../ir/element"
require_relative "result"
require_relative "context"

module Semauri
  module Semantics
    class Resolver
      def resolve(ast)
        @document = nil
        @explanations = []
        @context = Context.new
        ast.accept(self)
        raise SemanticError.new("Program does not create an artifact", code: "S301") unless @document

        Result.new(program: @document, explanations: @explanations.freeze)
      ensure
        @document = nil
        @explanations = nil
        @context = nil
      end

      def visit_program(node)
        node.statements.each { |statement| statement.accept(self) }
      end

      def visit_create_web(node)
        if @document
          raise SemanticError.new(
            "Semauri currently supports one web document per source file",
            code: "S302",
            line: node.line,
            column: node.column,
            hint: "Split independent web documents into separate .sema files."
          )
        end

        title, origin = infer_title(node)
        @document = IR::WebDocument.new(title: title, subject: node.subject, title_origin: origin)
        @explanations << "'web' resolved to an HTML web document (default web backend)."
        @explanations << "Subject resolved to '#{node.subject}'." if node.subject
        @explanations << title_explanation(title, origin)
      end

      def visit_set_title(node)
        require_document!(node, "Cannot add a title before creating a web document", "S303")
        @document = @document.with_title(node.title)
        @explanations << "Title explicitly set to '#{node.title}'."
      end

      def visit_create_element(node)
        require_document!(node, "Cannot add an element before creating a web document", "S304")

        id = @context.next_id(node.element_type)
        element = IR::Element.new(id: id, element_type: node.element_type, name: node.name)
        @document = @document.add_element(element)
        @context.register(id: id, entity_type: node.element_type, name: node.name)

        description = node.name ? "#{node.element_type} '#{node.name}'" : node.element_type.to_s
        @explanations << "Created #{description} as #{id}."
      end

      def visit_set_property(node)
        require_document!(node, "Cannot modify an element before creating a web document", "S307")

        target = @context.resolve(node.target, line: node.line, column: node.column)
        @document = @document.update_element(target.id) do |element|
          element.with_property(node.property, node.value)
        end

        @explanations << "Reference #{reference_text(node.target)} resolved to #{target.name || target.id}."
        @explanations << "Set #{node.property} of #{target.name || target.id} to '#{node.value}'."
      end

      private

      def require_document!(node, message, code)
        return if @document

        raise SemanticError.new(
          message,
          code: code,
          line: node.line,
          column: node.column,
          hint: "Create a web first."
        )
      end

      def infer_title(node)
        return [node.title, :explicit] if node.title
        return [node.subject, :subject_default] if node.subject

        ["Untitled", :fallback]
      end

      def title_explanation(title, origin)
        case origin
        when :explicit
          "Title explicitly set to '#{title}'."
        when :subject_default
          "No title was provided, so the web title defaults to its subject: '#{title}'."
        else
          "No title or subject was provided, so the web title defaults to 'Untitled'."
        end
      end

      def reference_text(reference)
        return "'it'" if reference.mode == :pronoun
        return "'#{reference.entity_type} called #{reference.name}'" if reference.name

        "'#{reference.entity_type}'"
      end
    end
  end
end
