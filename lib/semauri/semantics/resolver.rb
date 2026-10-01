# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "../ast/pronoun_reference"
require_relative "../ast/named_reference"
require_relative "../ir/web_document"
require_relative "../ir/element"
require_relative "../ir/program"
require_relative "result"
require_relative "entity_table"
require_relative "scope"
require_relative "expression_evaluator"
require_relative "type_system"

module Semauri
  module Semantics
    # Legacy AST executor kept only as a regression oracle while the production
    # compiler uses Typed HIR. It intentionally supports only the built-in Web
    # domain and should not be used as the semantic-domain extension surface.
    class Resolver
      def resolve(ast)
        @document = nil
        @explanations = []
        @entities = EntityTable.new
        @scope = Scope.new
        ast.accept(self)
        raise SemanticError.new("Program does not create an artifact", code: "S301") unless @document

        program_ir = IR::Program.new(units: [{ domain: :web, artifact: @document }])
        Result.new(program_ir: program_ir, explanations: @explanations.freeze, symbols: @scope.symbols)
      ensure
        @document = @explanations = @entities = @scope = nil
      end

      def visit_program(node)
        node.statements.each { |statement| statement.accept(self) }
      end

      def visit_block(node)
        with_child_scope { execute_statements(node) }
      end

      def visit_if_statement(node)
        condition = evaluate(node.condition)
        TypeSystem.ensure_boolean!(condition.type, node: node.condition,
                                   message: "If condition must evaluate to boolean", code: "S316")
        branch = condition.value ? node.consequence : node.alternative
        @explanations << "If condition evaluated to #{condition.value}; selected #{condition.value ? 'consequence' : 'alternative'} branch."
        branch&.accept(self)
      end

      def visit_for_each(node)
        iterable = evaluate(node.iterable)
        list_type = TypeSystem.ensure_list!(iterable.type, node: node.iterable)
        iterator_symbol = @scope.symbol_table.create(
          name: node.variable_name,
          kind: :iterator,
          type: list_type.element_type,
          definition_span: node.binding_span
        )

        @explanations << "For every '#{node.variable_name}' iterates over #{iterable.value.length} #{list_type.element_type} value(s) as symbol ##{iterator_symbol.id}."

        iterable.value.each do |item|
          parent = @scope
          begin
            @scope = parent.child
            value = Value.new(type: list_type.element_type, value: item, definition_span: node.binding_span)
            @scope.bind(iterator_symbol, value, node: node)
            execute_statements(node.body)
          ensure
            @scope = parent
          end
        end
      end

      def visit_create_artifact(node)
        ensure_web_domain!(node)
        create_web_document(node)
      end

      def visit_create_web(node)
        create_web_document(node)
      end

      def visit_set_title(node)
        require_document!(node, "Cannot add a title before creating a web document", "S303")
        @document = @document.with_title(node.title)
        @explanations << "Title explicitly set to '#{node.title}'."
      end

      def visit_add_element(node)
        ensure_web_domain!(node) if node.respond_to?(:domain)
        require_document!(node, "Cannot add an element before creating a web document", "S306")
        label = node.label || node.kind.to_s.capitalize
        element = @entities.register(kind: node.kind) { |id| IR::Element.new(id: id, kind: node.kind, label: label) }
        @document = @document.add_element(element)
        @explanations << "Added #{node.kind} '#{label}' as #{element.id}."
      end

      def visit_let_binding(node)
        value = evaluate(node.value)
        binding = @scope.define(node.name, value, node: node)
        @explanations << "Bound '#{node.name}' as symbol ##{binding.symbol.id} to #{value.describe}."
      end

      def visit_set_property(node)
        ensure_web_domain!(node) if node.respond_to?(:domain)
        require_document!(node, "Cannot modify an element before creating a web document", "S307")
        target = resolve_reference(node.target)
        value = evaluate(node.value)
        TypeSystem.validate_property!(node.property, value.type, node: node.value)
        updated = target.with_property(node.property, value.value, source_span: node.span)
        @document = @document.replace_element(updated)
        @explanations << reference_explanation(node.target, target)
        @explanations << "Set #{target.id}.#{node.property} to #{value.value.inspect}."
      end

      private

      def create_web_document(node)
        if @document
          raise semantic_error("Semauri currently supports one web document per source file", code: "S302", node: node,
                               hint: "Split independent web documents into separate .sema files.")
        end
        title, origin = infer_title(node)
        @document = IR::WebDocument.new(title: title, subject: node.subject, title_origin: origin)
        @explanations << "'web' resolved to an HTML web document (default web backend)."
        @explanations << "Subject resolved to '#{node.subject}'." if node.subject
        @explanations << title_explanation(title, origin)
      end

      def ensure_web_domain!(node)
        return if node.domain.to_sym == :web

        raise semantic_error("Legacy resolver only supports the Web domain", code: "S326", node: node)
      end

      def execute_statements(block)
        block.statements.each { |statement| statement.accept(self) }
      end

      def evaluate(expression)
        ExpressionEvaluator.new(scope: @scope, on_variable_resolution: lambda { |reference, value, symbol|
          @explanations << "Variable '#{reference.name}' resolved to symbol ##{symbol.id} (#{value.describe})."
        }).evaluate(expression)
      end

      def with_child_scope
        parent = @scope
        @scope = parent.child
        yield
      ensure
        @scope = parent
      end

      def resolve_reference(reference)
        case reference
        when AST::PronounReference then @entities.resolve_pronoun(reference)
        when AST::NamedReference then @entities.resolve_named(reference)
        else raise SemanticError.new("Unsupported reference #{reference.class}", code: "S308")
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
        SemanticError.new(message, code: code, line: node.line, column: node.column,
                          end_line: node.end_line, end_column: node.end_column, hint: hint)
      end

      def infer_title(node)
        return [node.title, :explicit] if node.title
        return [node.subject, :subject_default] if node.subject
        ["Untitled", :fallback]
      end

      def title_explanation(title, origin)
        case origin
        when :explicit then "Title explicitly set to '#{title}'."
        when :subject_default then "No title was provided, so the web title defaults to its subject: '#{title}'."
        else "No title or subject was provided, so the web title defaults to 'Untitled'."
        end
      end
    end
  end
end
