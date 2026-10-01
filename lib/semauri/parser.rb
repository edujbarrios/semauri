# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "errors"
require_relative "source_span"
require_relative "domains/registry"
require_relative "ast/program"
require_relative "ast/create_artifact"
require_relative "ast/domain_operation"
require_relative "ast/domain_scope"
require_relative "ast/set_title"
require_relative "ast/add_element"
require_relative "ast/pronoun_reference"
require_relative "ast/named_reference"
require_relative "ast/set_property"
require_relative "ast/literal"
require_relative "ast/list_literal"
require_relative "ast/variable_reference"
require_relative "ast/let_binding"
require_relative "ast/block"
require_relative "ast/binary_expression"
require_relative "ast/if_statement"
require_relative "ast/for_each"
require_relative "ast/unary_expression"

module Semauri
  class Parser
    def initialize(tokens, domains: Domains::Registry.default)
      @tokens = tokens
      @domains = domains
      @current = 0
      @domain_scope_stack = []
    end

    def parse
      statements = []
      statements << statement until check?(:EOF)
      AST::Program.new(statements: statements, span: program_span(statements))
    end

    private

    def statement
      return create_statement(previous) if match?(:CREATE)
      return make_statement(previous) if match?(:MAKE)
      return add_statement if match?(:ADD)
      return let_statement(previous) if match?(:LET)
      return set_statement(previous) if match?(:SET)
      return if_statement(previous) if match?(:IF)
      return for_statement(previous) if match?(:FOR)
      return domain_scope_statement(previous) if match?(:WITHIN)
      return domain_operation_statement(previous) if match?(:DOMAIN_ACTION)

      error!(peek, "Expected a core statement or registered domain action", "S201")
    end

    def create_statement(start)
      match?(:ARTICLE)
      artifact = consume(:DOMAIN_ARTIFACT, "Expected a registered artifact after '#{start.lexeme}'", "S202")
      term = domain_term(artifact)

      subject = nil
      title = nil

      if match?(:CALLED)
        title = phrase_until(:DOT, :EOF)
      elsif match?(:FOR)
        match?(:ARTICLE)
        subject = phrase_until(:DOT, :EOF)
      end

      consume_optional_dot
      AST::CreateArtifact.new(domain: term.fetch(:domain), kind: term.fetch(:kind),
                              subject: normalize_phrase(subject), title: normalize_phrase(title), span: span_from(start))
    end

    def domain_scope_statement(start)
      domain_token = peek
      domain_name = domain_token.lexeme.to_s.downcase.to_sym
      unless @domains.names.include?(domain_name)
        error!(domain_token, "Unknown semantic domain '#{domain_token.lexeme}'", "S240",
               hint: "Available domains: #{@domains.names.join(', ')}.")
      end
      advance
      consume(:COLON, "Expected ':' after the semantic domain name", "S240")

      @domain_scope_stack << domain_name
      body = block_until(:END)
      @domain_scope_stack.pop

      consume(:END, "Expected 'End' to close the semantic domain scope", "S240")
      consume_optional_dot
      AST::DomainScope.new(domain: domain_name, body: body, span: span_from(start))
    end

    def domain_operation_statement(start)
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

      consume_optional_dot
      AST::DomainOperation.new(
        domain: term.fetch(:domain),
        operation: operation.name,
        arguments: arguments,
        span: span_from(start)
      )
    end

    def action_term(token)
      metadata = domain_term(token)
      candidates = metadata[:candidates] || metadata["candidates"]
      candidates = [metadata] unless candidates
      candidates = candidates.map { |candidate| candidate.transform_keys(&:to_sym) }

      if (scope = @domain_scope_stack.last)
        selected = candidates.find { |candidate| candidate.fetch(:domain).to_sym == scope }
        return selected if selected

        available = candidates.map { |candidate| candidate.fetch(:domain) }.uniq
        error!(token, "Action '#{token.lexeme}' is not available in semantic domain '#{scope}'", "S240",
               hint: "This action is available in: #{available.join(', ')}.")
      end

      return candidates.first if candidates.one?

      domains = candidates.map { |candidate| candidate.fetch(:domain) }.uniq
      error!(token, "Ambiguous domain action '#{token.lexeme}'", "S240",
             hint: "Qualify it with 'Within <domain>:'; candidates: #{domains.join(', ')}.")
    end

    def parse_operation_slot(slot)
      case slot.kind
      when :expression then expression
      else error!(peek, "Unsupported operation slot kind '#{slot.kind}'", "S239")
      end
    end

    def make_statement(start)
      return pronoun_property_statement(start) if check?(:PRONOUN)
      return named_property_statement(start) if named_reference_ahead?

      create_statement(start)
    end

    def pronoun_property_statement(start)
      pronoun = advance
      value_token = consume(:COLOR, "Expected a supported value after '#{pronoun.lexeme}'", "S206")
      domain, property = implicit_property_for(value_token, :color)
      consume_optional_dot

      target = AST::PronounReference.new(pronoun: pronoun.lexeme, span: pronoun.span)
      value = AST::Literal.new(value_type: :color, value: value_token.literal, span: value_token.span)
      AST::SetProperty.new(domain: domain, target: target, property: property, value: value, span: span_from(start))
    end

    def named_property_statement(start)
      match?(:ARTICLE)
      element = consume(:DOMAIN_ELEMENT, "Expected a registered element", "S207")
      element_term = domain_term(element)
      consume(:CALLED, "Expected 'called' or 'named' in an explicit reference", "S207")

      reference_tokens = tokens_until(:DOT, :EOF)
      error!(peek, "Expected a name and a value", "S208") if reference_tokens.length < 2

      value_token = reference_tokens.last
      error!(value_token, "Expected a supported value after the referenced element", "S208") unless value_token.type == :COLOR
      domain, property = implicit_property_for(value_token, :color)
      unless domain.to_sym == element_term.fetch(:domain).to_sym
        error!(value_token, "Implicit property belongs to domain '#{domain}', but the target belongs to '#{element_term.fetch(:domain)}'", "S235")
      end

      label_tokens = reference_tokens[0...-1]
      label = phrase_from_tokens(label_tokens)
      consume_optional_dot

      target = AST::NamedReference.new(domain: domain, kind: element_term.fetch(:kind),
                                       label: normalize_phrase(label), span: span_between(element, label_tokens.last))
      value = AST::Literal.new(value_type: :color, value: value_token.literal, span: value_token.span)
      AST::SetProperty.new(domain: domain, target: target, property: property, value: value, span: span_from(start))
    end

    def let_statement(start)
      name = consume(:WORD, "Expected a variable name after 'Let'", "S209")
      consume(:BE, "Expected 'be' after the variable name", "S210")
      value = expression
      consume_optional_dot
      AST::LetBinding.new(name: name.lexeme, value: value, span: span_from(start))
    end

    def set_statement(start)
      match?(:ARTICLE)
      property_token = consume(:DOMAIN_PROPERTY, "Expected a registered property after 'Set'", "S211")
      property_term = domain_term(property_token)
      consume(:OF, "Expected 'of' after the property name", "S212")
      target = canonical_reference(expected_domain: property_term.fetch(:domain))
      consume(:TO, "Expected 'to' before the new value", "S213")
      value = expression
      consume_optional_dot
      AST::SetProperty.new(domain: property_term.fetch(:domain), target: target,
                           property: property_term.fetch(:kind), value: value, span: span_from(start))
    end

    def if_statement(start)
      condition = expression
      consume(:COLON, "Expected ':' after the if condition", "S218")
      consequence = block_until(:OTHERWISE, :END)
      alternative = nil

      if match?(:OTHERWISE)
        consume(:COLON, "Expected ':' after 'Otherwise'", "S219")
        alternative = block_until(:END)
      end

      consume(:END, "Expected 'End' to close the If block", "S220")
      consume_optional_dot
      AST::IfStatement.new(condition: condition, consequence: consequence, alternative: alternative, span: span_from(start))
    end

    def for_statement(start)
      consume(:EVERY, "Expected 'every' after 'For'", "S228")
      variable = consume(:WORD, "Expected an iteration variable after 'For every'", "S229")
      consume(:IN, "Expected 'in' after the iteration variable", "S230")
      iterable = expression
      consume(:COLON, "Expected ':' after the iterable expression", "S231")
      body = block_until(:END)
      consume(:END, "Expected 'End' to close the For block", "S232")
      consume_optional_dot

      AST::ForEach.new(variable_name: variable.lexeme, iterable: iterable, body: body,
                       binding_span: variable.span, span: span_from(start))
    end

    def block_until(*terminators)
      statements = []
      statements << statement until terminators.include?(peek.type) || check?(:EOF)
      error!(peek, "Expected at least one statement in the block", "S221") if statements.empty?
      AST::Block.new(statements: statements, span: program_span(statements))
    end

    def expression = logical_or

    def logical_or
      expression = logical_and
      while match?(:OR)
        right = logical_and
        expression = AST::BinaryExpression.new(left: expression, operator: :or, right: right,
                                               span: span_between(expression, right))
      end
      expression
    end

    def logical_and
      expression = logical_not
      while match?(:AND)
        right = logical_not
        expression = AST::BinaryExpression.new(left: expression, operator: :and, right: right,
                                               span: span_between(expression, right))
      end
      expression
    end

    def logical_not
      if match?(:NOT)
        operator = previous
        operand = logical_not
        return AST::UnaryExpression.new(operator: :not, operand: operand, span: span_between(operator, operand))
      end
      comparison
    end

    def comparison
      left = additive
      return left unless match?(:IS)

      operator = if match?(:GREATER)
                   consume(:THAN, "Expected 'than' after 'greater'", "S222")
                   :greater_than
                 elsif match?(:LESS)
                   consume(:THAN, "Expected 'than' after 'less'", "S223")
                   :less_than
                 elsif match?(:EQUAL)
                   consume(:TO, "Expected 'to' after 'equal'", "S224")
                   :equal
                 else
                   error!(peek, "Expected 'greater than', 'less than', or 'equal to' after 'is'", "S225")
                 end

      right = additive
      AST::BinaryExpression.new(left: left, operator: operator, right: right, span: span_between(left, right))
    end

    def additive
      expression = multiplicative
      while match?(:PLUS, :MINUS)
        operator_token = previous
        right = multiplicative
        operator = operator_token.type == :PLUS ? :add : :subtract
        expression = AST::BinaryExpression.new(left: expression, operator: operator, right: right,
                                               span: span_between(expression, right))
      end
      expression
    end

    def multiplicative
      expression = primary
      loop do
        if match?(:TIMES)
          right = primary
          expression = AST::BinaryExpression.new(left: expression, operator: :multiply, right: right,
                                                 span: span_between(expression, right))
        elsif match?(:DIVIDED)
          consume(:BY, "Expected 'by' after 'divided'", "S226")
          right = primary
          expression = AST::BinaryExpression.new(left: expression, operator: :divide, right: right,
                                                 span: span_between(expression, right))
        else
          break
        end
      end
      expression
    end

    def primary
      token = peek
      case token.type
      when :COLOR
        advance
        AST::Literal.new(value_type: :color, value: token.literal, span: token.span)
      when :STRING
        advance
        AST::Literal.new(value_type: :string, value: token.literal, span: token.span)
      when :NUMBER
        advance
        AST::Literal.new(value_type: :number, value: token.literal, span: token.span)
      when :BOOLEAN
        advance
        AST::Literal.new(value_type: :boolean, value: token.lexeme.casecmp?("true"), span: token.span)
      when :ARTICLE
        start = advance
        list_literal(start)
      when :LIST
        start = advance
        list_literal(start, list_consumed: true)
      when :WORD
        advance
        AST::VariableReference.new(name: token.lexeme, span: token.span)
      when :LPAREN
        opening = advance
        inner = expression
        closing = consume(:RPAREN, "Expected ')' after expression", "S227")
        re_span_expression(inner, span_between(opening, closing))
      else
        error!(token, "Expected a color, string, number, boolean, list, variable or parenthesized expression", "S217")
      end
    end

    def list_literal(start, list_consumed: false)
      consume(:LIST, "Expected 'list' after the article in a list literal", "S233") unless list_consumed
      consume(:OF, "Expected 'of' after 'list'", "S234")

      items = [expression]
      items << expression while match?(:COMMA)

      AST::ListLiteral.new(items: items, span: span_between(start, items.last))
    end

    def re_span_expression(expression, span)
      case expression
      when AST::BinaryExpression
        AST::BinaryExpression.new(left: expression.left, operator: expression.operator, right: expression.right, span: span)
      when AST::Literal
        AST::Literal.new(value_type: expression.value_type, value: expression.value, span: span)
      when AST::ListLiteral
        AST::ListLiteral.new(items: expression.items, span: span)
      when AST::VariableReference
        AST::VariableReference.new(name: expression.name, span: span)
      when AST::UnaryExpression
        AST::UnaryExpression.new(operator: expression.operator, operand: expression.operand, span: span)
      else
        expression
      end
    end

    def canonical_reference(expected_domain:)
      if check?(:PRONOUN)
        token = advance
        return AST::PronounReference.new(pronoun: token.lexeme, span: token.span)
      end

      match?(:ARTICLE)
      element = consume(:DOMAIN_ELEMENT, "Expected a registered element or 'it' after 'of'", "S214")
      term = domain_term(element)
      unless term.fetch(:domain).to_sym == expected_domain.to_sym
        error!(element, "Element belongs to domain '#{term.fetch(:domain)}', expected '#{expected_domain}'", "S235")
      end
      consume(:CALLED, "Canonical references must name the target with 'called' or 'named'", "S215")
      label_tokens = tokens_until(:TO, :EOF)
      error!(peek, "Expected the referenced element name", "S216") if label_tokens.empty?
      AST::NamedReference.new(domain: term.fetch(:domain), kind: term.fetch(:kind),
                              label: normalize_phrase(phrase_from_tokens(label_tokens)),
                              span: span_between(element, label_tokens.last))
    end

    def named_reference_ahead?
      offset = check?(:ARTICLE) ? 1 : 0
      @tokens[@current + offset]&.type == :DOMAIN_ELEMENT
    end

    def add_statement
      start = previous
      match?(:ARTICLE)
      return add_title(start) if match?(:TITLE)

      if check?(:DOMAIN_ELEMENT)
        element = advance
        return add_element(start, domain_term(element))
      end

      error!(peek, "Expected 'title' or a registered domain element after 'Add'", "S203")
    end

    def add_title(start)
      consume(:CALLED, "Expected 'called' or 'named' after 'title'", "S204")
      title = phrase_until(:DOT, :EOF)
      consume_optional_dot
      AST::SetTitle.new(title: normalize_phrase(title), span: span_from(start))
    end

    def add_element(start, term)
      label = nil
      label = phrase_until(:DOT, :EOF) if match?(:CALLED)
      consume_optional_dot
      AST::AddElement.new(domain: term.fetch(:domain), kind: term.fetch(:kind),
                          label: normalize_phrase(label), span: span_from(start))
    end

    def implicit_property_for(token, value_type)
      inferred = @domains.infer_property_for_type(value_type)
      return inferred if inferred

      error!(token, "Cannot infer a unique property for #{value_type}; use 'Set <property> of ... to ...'", "S236")
    end

    def domain_term(token)
      token.literal || error!(token, "Domain token is missing semantic metadata", "S237")
    end

    def phrase_until(*terminators)
      tokens = tokens_until(*terminators)
      error!(peek, "Expected a name or description", "S205") if tokens.empty?
      phrase_from_tokens(tokens)
    end

    def tokens_until(*terminators)
      result = []
      result << advance until terminators.include?(peek.type)
      result
    end

    def phrase_from_tokens(tokens)
      tokens.map { |token| token.literal.is_a?(String) ? token.literal : token.lexeme }.join(" ")
    end

    def normalize_phrase(value)
      return nil unless value
      value.strip.gsub(/\s+/, " ").split.map { |word| word.match?(/\A[A-Z0-9]+\z/) ? word : word.capitalize }.join(" ")
    end

    def program_span(statements)
      return SourceSpan.point(1, 1) if statements.empty?
      SourceSpan.new(start_line: statements.first.line, start_column: statements.first.column,
                     end_line: statements.last.end_line, end_column: statements.last.end_column)
    end

    def span_from(start) = span_between(start, previous)

    def span_between(first, last)
      SourceSpan.new(start_line: first.line, start_column: first.column,
                     end_line: last.end_line, end_column: last.end_column)
    end

    def consume_optional_dot
      advance if check?(:DOT)
    end

    def consume(type, message, code)
      return advance if check?(type)
      error!(peek, message, code)
    end

    def consume_surface(word, message, code)
      return advance if peek.lexeme.casecmp?(word.to_s)
      error!(peek, message, code)
    end

    def match?(*types)
      return false unless types.any? { |type| check?(type) }
      advance
      true
    end

    def check?(type) = peek.type == type

    def advance
      @current += 1 unless at_end?
      previous
    end

    def at_end? = peek.type == :EOF
    def peek = @tokens[@current]
    def previous = @tokens[@current - 1]

    def error!(token, message, code, hint: nil)
      raise ParseError.new(message, code: code, line: token.line, column: token.column,
                           end_line: token.end_line, end_column: token.end_column, hint: hint)
    end
  end
end
