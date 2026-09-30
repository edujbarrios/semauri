# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "../ast/pronoun_reference"
require_relative "../ast/named_reference"
require_relative "../ast/literal"
require_relative "../ast/variable_reference"
require_relative "../ir/web_document"
require_relative "../ir/element"
require_relative "result"
require_relative "entity_table"
require_relative "scope"
require_relative "value"

module Semauri
  module Semantics
    class Resolver
      PROPERTY_TYPES = { color: :color }.freeze

      def resolve(ast)
        @document = nil
        @explanations = []
        @entities = EntityTable.new
        @scope = Scope.new
        ast.accept(self)
        raise SemanticError.new("Program does not create an artifact", code: "S301") unless @document

        Result.new(program: @document, explanations: @explanations.freeze)
      ensure
        @document = nil
        @explanations = nil
        @entities = nil
        @scope = nil
      end

      def visit_program(node)
        node.statements.each { |statement| statement.accept(self) }
      end

      def visit_create_web(node)
        if @document
          raise semantic_error(
            "Semauri currently supports one web document per source file",
            code: "S302",
            node: node,
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

      def visit_add_element(node)
        require_document!(node, "Cannot add an element before creating a web document", "S306")
        label = node.label || node.kind.to_s.capitalize

        element = @entities.register(kind: node.kind) do |id|
          IR::Element.new(id: id, kind: node.kind, label: label)
        end

        @document = @document.add_element(element)
        @explanations << "Added #{node.kind} '#{label}' as #{element.id}."
      end

      def visit_let_binding(node)
        value = node.value.accept(self)
        @scope.define(node.name, value, node: node)
        @explanations << "Bound '#{node.name}' to #{value.describe}."
      end

      def visit_literal(node)
        Value.new(type: node.value_type, value: node.value, definition_span: node.span)
      end

      def visit_variable_reference(node)
        value = @scope.resolve(node)
        @explanations << "Variable '#{node.name}' resolved to #{value.describe}."
        value
      end

      def visit_set_property(node)
        require_document!(node, "Cannot modify an element before creating a web document", "S307")
        target = resolve_reference(node.target)
        value = node.value.accept(self)
        validate_property_type!(node.property, value, node.value)

        updated = target.with_property(node.property, value.value, source_span: node.span)
        @document = @document.replace_element(updated)
        @explanations << reference_explanation(node.target, target)
        @explanations << "Set #{target.id}.#{node.property} to #{value.value.inspect}."
      end

      private

      def validate_property_type!(property, value, expression)
        expected = PROPERTY_TYPES[property]
        return unless expected
        return if value.type == expected

        raise semantic_error(
          "Property '#{property}' expects #{expected}, but received #{value.type}",
          code: "S313",
          node: expression,
          hint: "Use a #{expected} literal or a variable containing a #{expected}."
        )
      end

      def resolve_reference(reference)
        case reference
        when AST::PronounReference
          @entities.resolve_pronoun(reference)
        when AST::NamedReference
          @entities.resolve_named(reference)
        else
          raise SemanticError.new("Unsupported reference #{reference.class}", code: "S308")
        end
      end

      def reference_explanation(reference, target)
        if reference.is_a?(AST::PronounReference)
          "'#{reference.pronoun}' resolved to #{target.kind} '#{target.label}' (#{target.id})."
        else
          "Explicit reference resolved to #{target.kind} '#{target.label}' (#{target.id})."
        end
      end

      def require_document!(node, message, code)
        return if @document

        raise semantic_error(message, code: code, node: node, hint: "Create a web first.")
      end

      def semantic_error(message, code:, node:, hint: nil)
        SemanticError.new(
          message,
          code: code,
          line: node.line,
          column: node.column,
          end_line: node.end_line,
          end_column: node.end_column,
          hint: hint
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
    end
  end
end
