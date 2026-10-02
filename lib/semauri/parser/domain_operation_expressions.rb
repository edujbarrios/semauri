# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "inclusive_comparisons"

module Semauri
  module ParserDomainOperationExpressions
    private

    def domain_operation_statement(start)
      node, = parse_registered_domain_operation(start)
      consume_optional_dot
      node
    end

    def primary
      return super unless check?(:DOMAIN_ACTION)

      start = advance
      node, operation = parse_registered_domain_operation(start)
      if operation.return_type == :unit
        error!(
          start,
          "Operation '#{operation.name}' does not produce a value",
          "S241",
          hint: "Use '#{start.lexeme}' as a statement instead of inside an expression."
        )
      end
      node
    end

    def parse_registered_domain_operation(start)
      term = action_term(start)
      domain = @domains.fetch(term.fetch(:domain))
      operation = domain.operation(term.fetch(:kind))
      arguments = {}

      operation.pattern.each do |segment|
        case segment
        when Domains::Operation::Literal
          consume_surface(segment.word, "Expected '#{segment.word}' in '#{operation.name}' operation", "S238")
        when Domains::Operation::Slot
          arguments[segment.name] = parse_operation_slot(segment)
        else
          error!(peek, "Unsupported operation pattern segment", "S239")
        end
      end

      node = AST::DomainOperation.new(
        domain: term.fetch(:domain),
        operation: operation.name,
        arguments: arguments,
        span: span_from(start)
      )
      [node, operation]
    end
  end

  Parser.prepend(ParserDomainOperationExpressions)
end
