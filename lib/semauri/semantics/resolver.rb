# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "../ir/web_document"
require_relative "result"

module Semauri
  module Semantics
    class Resolver
      def resolve(ast)
        @document = nil
        @explanations = []
        ast.accept(self)
        raise SemanticError.new("Program does not create an artifact", code: "S301") unless @document

        Result.new(program: @document, explanations: @explanations.freeze)
      ensure
        @document = nil
        @explanations = nil
      end

      def visit_program(node)
        node.statements.each { |statement| statement.accept(self) }
      end

      def visit_create_web(node)
        if @document
          raise SemanticError.new(
            "Semauri 0.1 supports one web document per source file",
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
        unless @document
          raise SemanticError.new(
            "Cannot add a title before creating a web document",
            code: "S303",
            line: node.line,
            column: node.column,
            hint: "Create a web first, then add its title."
          )
        end

        @document = @document.with_title(node.title)
        @explanations << "Title explicitly set to '#{node.title}'."
      end

      private

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
