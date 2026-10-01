# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "evaluator"
require_relative "value_environment"
require_relative "../errors"
require_relative "../domains/registry"
require_relative "../semantics/entity_table"
require_relative "../semantics/result"
require_relative "../semantics/type_system"
require_relative "../semantics/value"

module Semauri
  module HIR
    class Lowerer
      def initialize(domains: Domains::Registry.default)
        @domains = domains
      end

      def lower(hir_result)
        @symbols = hir_result.symbols
        @symbols_by_id = @symbols.to_h { |symbol| [symbol.id, symbol] }
        @environment = ValueEnvironment.new
        @artifact = nil
        @active_domain = nil
        @entities = Semantics::EntityTable.new
        @explanations = []

        lower_statement(hir_result.program)
        raise SemanticError.new("Program does not create an artifact", code: "S301") unless @artifact

        Semantics::Result.new(program: @artifact, explanations: @explanations.freeze, symbols: @symbols)
      ensure
        @symbols = @symbols_by_id = @environment = @artifact = @active_domain = @entities = @explanations = nil
      end

      private

      def lower_statement(node)
        case node.kind
        when :program then lower_statements(node.fields.fetch(:statements))
        when :block then with_child_environment { lower_statements(node.fields.fetch(:statements)) }
        when :let then lower_let(node)
        when :if then lower_if(node)
        when :for_each then lower_for_each(node)
        when :create_artifact then lower_create_artifact(node)
        when :create_web then lower_legacy_create_web(node)
        when :set_title then lower_set_title(node)
        when :add_element then lower_add_element(node)
        when :set_property then lower_set_property(node)
        else raise semantic_error(node, "Unsupported HIR statement '#{node.kind}'", "S325")
        end
      end

      def lower_statements(statements)
        statements.each { |statement| lower_statement(statement) }
      end

      def lower_let(node)
        value = evaluate(node.fields.fetch(:value))
        symbol = symbol!(node.fields.fetch(:symbol_id), node)
        validate_symbol_type!(symbol, value, node)
        @environment.bind(symbol.id, value, node: node)
        display_name = node.fields[:source_name] || symbol.name
        @explanations << "Bound '#{display_name}' as symbol ##{symbol.id} to #{value.describe}."
      end

      def lower_if(node)
        condition = evaluate(node.fields.fetch(:condition))
        Semantics::TypeSystem.ensure_boolean!(condition.type, node: node.fields.fetch(:condition),
                                              message: "If condition must evaluate to boolean", code: "S316")
        branch = condition.value ? node.fields[:consequence] : node.fields[:alternative]
        @explanations << "If condition evaluated to #{condition.value}; selected #{condition.value ? 'consequence' : 'alternative'} branch."
        lower_statement(branch) if branch
      end

      def lower_for_each(node)
        iterable = evaluate(node.fields.fetch(:iterable))
        list_type = Semantics::TypeSystem.ensure_list!(iterable.type, node: node.fields.fetch(:iterable))
        iterator = symbol!(node.fields.fetch(:iterator_symbol_id), node)
        display_name = node.fields[:iterator_source_name] || iterator.name

        @explanations << "For every '#{display_name}' iterates over #{iterable.value.length} #{list_type.element_type} value(s) as symbol ##{iterator.id}."

        iterable.value.each do |item|
          parent = @environment
          begin
            @environment = parent.child
            value = Semantics::Value.new(type: list_type.element_type, value: item, definition_span: iterator.definition_span)
            validate_symbol_type!(iterator, value, node)
            @environment.bind(iterator.id, value, node: node)
            lower_statements(node.fields.fetch(:body).fields.fetch(:statements))
          ensure
            @environment = parent
          end
        end
      end

      def lower_create_artifact(node)
        if @artifact
          raise semantic_error(node, "Semauri currently supports one artifact per source file", "S302",
                               hint: "Split independent artifacts into separate .sema files.")
        end

        domain = @domains.fetch(node.fields.fetch(:domain))
        kind = node.fields.fetch(:artifact_kind)
        subject = node.fields[:subject]
        explicit_title = node.fields[:title]

        @artifact = domain.create_artifact(kind: kind, subject: subject, title: explicit_title)
        @active_domain = domain
        @explanations.concat(domain.creation_explanations(
                               artifact: @artifact,
                               kind: kind,
                               subject: subject,
                               explicit_title: explicit_title
                             ))
      end

      def lower_legacy_create_web(node)
        compatibility = HIR::Node.new(
          kind: :create_artifact,
          type: :unit,
          fields: { domain: :web, artifact_kind: :web, subject: node.fields[:subject], title: node.fields[:title] },
          span: node.span
        )
        lower_create_artifact(compatibility)
      end

      def lower_set_title(node)
        domain = require_artifact!(node, "Cannot add a title before creating an artifact", "S303")
        title = node.fields.fetch(:title)
        @artifact = domain.set_title(artifact: @artifact, title: title)
        @explanations << "Title explicitly set to '#{title}'."
      end

      def lower_add_element(node)
        domain = domain_for_node!(node)
        kind = node.fields.fetch(:element_kind)
        @artifact, element = domain.add_element(
          artifact: @artifact,
          kind: kind,
          label: node.fields[:label],
          entities: @entities
        )
        @explanations << "Added #{kind} '#{element.label}' as #{element.id}."
      end

      def lower_set_property(node)
        domain = domain_for_node!(node)
        target_node = node.fields.fetch(:target)
        target = resolve_reference(target_node, domain: domain)
        value = evaluate(node.fields.fetch(:value))
        property = node.fields.fetch(:property)
        @domains.validate_property!(domain: domain.name, property: property, actual_type: value.type,
                                    node: node.fields.fetch(:value))

        @artifact = domain.set_property(
          artifact: @artifact,
          target: target,
          property: property,
          value: value.value,
          source_span: node.span
        )
        @explanations << reference_explanation(target_node, target)
        @explanations << "Set #{target.id}.#{property} to #{value.value.inspect}."
      end

      def evaluate(node)
        Evaluator.new(
          environment: @environment,
          symbols_by_id: @symbols_by_id,
          on_symbol_resolution: lambda { |reference, value, symbol|
            name = reference.fields[:source_name] || reference.fields[:name] || symbol.name
            @explanations << "Variable '#{name}' resolved to symbol ##{symbol.id} (#{value.describe})."
          }
        ).evaluate(node)
      end

      def resolve_reference(node, domain:)
        case node.kind
        when :pronoun_reference
          @entities.resolve_pronoun_data(pronoun: node.fields.fetch(:pronoun), node: node)
        when :named_reference
          reference_domain = node.fields.fetch(:domain)
          ensure_same_domain!(reference_domain, domain.name, node)
          @entities.resolve_named_data(kind: node.fields.fetch(:entity_kind), label: node.fields.fetch(:label), node: node)
        else raise semantic_error(node, "Unsupported HIR reference '#{node.kind}'", "S325")
        end
      end

      def reference_explanation(reference, target)
        if reference.kind == :pronoun_reference
          "'#{reference.fields.fetch(:pronoun)}' resolved to #{target.kind} '#{target.label}' (#{target.id})."
        else
          "Explicit reference resolved to #{target.kind} '#{target.label}' (#{target.id})."
        end
      end

      def domain_for_node!(node)
        domain = require_artifact!(node, "Cannot use a domain operation before creating an artifact", "S307")
        requested = node.fields.fetch(:domain)
        ensure_same_domain!(requested, domain.name, node)
        domain
      end

      def ensure_same_domain!(requested, active, node)
        return if requested.to_sym == active.to_sym

        raise semantic_error(node,
                             "Operation belongs to domain '#{requested}', but the active artifact belongs to '#{active}'",
                             "S327")
      end

      def require_artifact!(node, message, code)
        return @active_domain if @artifact && @active_domain
        raise semantic_error(node, message, code, hint: "Create an artifact first.")
      end

      def with_child_environment
        parent = @environment
        @environment = parent.child
        yield
      ensure
        @environment = parent
      end

      def symbol!(id, node)
        @symbols_by_id.fetch(id)
      rescue KeyError
        raise semantic_error(node, "Unknown HIR symbol ##{id}", "S324")
      end

      def validate_symbol_type!(symbol, value, node)
        return if symbol.type == value.type
        raise semantic_error(node, "HIR symbol ##{symbol.id} expects #{symbol.type}, received #{value.type}", "S324")
      end

      def semantic_error(node, message, code, hint: nil)
        SemanticError.new(message, code: code,
                          line: node.line, column: node.column,
                          end_line: node.end_line, end_column: node.end_column,
                          hint: hint)
      end
    end
  end
end
