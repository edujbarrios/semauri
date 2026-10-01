# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "../semantics/type_system"

module Semauri
  module Domains
    class Operation
      class Literal
        attr_reader :word

        def initialize(word)
          @word = word.to_s.downcase.freeze
          raise ArgumentError, "Operation literal cannot be empty" if @word.empty?
          freeze
        end

        def to_h = { literal: word }.freeze
      end

      class Slot
        KINDS = %i[expression].freeze

        attr_reader :name, :kind, :type

        def initialize(name:, kind:, type: nil)
          @name = name.to_sym
          @kind = kind.to_sym
          @type = Operation.normalize_type(type)
          raise ArgumentError, "Unsupported operation slot kind '#{kind}'" unless KINDS.include?(@kind)
          freeze
        end

        def to_h = { slot: name, kind: kind, type: Operation.serialize_type(type) }.freeze
      end

      attr_reader :name, :verbs, :pattern, :return_type, :effects

      def self.literal(word) = Literal.new(word)
      def self.expression(name, type: nil) = Slot.new(name: name, kind: :expression, type: type)

      def self.normalize_type(type)
        return nil if type.nil?
        type.is_a?(String) ? type.to_sym : type
      end

      def self.serialize_type(type)
        type.respond_to?(:to_h) ? type.to_h : type
      end

      def initialize(name:, verbs:, pattern:, returns: :unit, effects: [])
        @name = name.to_sym
        @verbs = Array(verbs).map { |verb| verb.to_s.downcase.freeze }.uniq.freeze
        @pattern = Array(pattern).freeze
        @return_type = self.class.normalize_type(returns)
        @effects = Array(effects).map(&:to_sym).uniq.freeze

        raise ArgumentError, "Operation '#{name}' requires at least one verb" if @verbs.empty?
        raise ArgumentError, "Operation '#{name}' contains an empty verb" if @verbs.any?(&:empty?)
        unless @pattern.all? { |segment| segment.is_a?(Literal) || segment.is_a?(Slot) }
          raise ArgumentError, "Operation '#{name}' pattern contains an unsupported segment"
        end

        slot_names = slots.map(&:name)
        raise ArgumentError, "Operation '#{name}' contains duplicate argument slots" unless slot_names.uniq.length == slot_names.length

        freeze
      end

      def slots = pattern.grep(Slot).freeze

      def expected_type(argument)
        slots.find { |slot| slot.name == argument.to_sym }&.type
      end

      def validate_argument_types!(arguments, node:)
        slots.each do |slot|
          value = arguments.fetch(slot.name)
          next unless slot.type
          next if Semantics::TypeSystem.assignable?(value.type, slot.type)

          raise SemanticError.new(
            "Operation '#{name}' argument '#{slot.name}' expects #{Semantics::TypeSystem.type_name(slot.type)}, but received #{Semantics::TypeSystem.type_name(value.type)}",
            code: "S329",
            line: value.line, column: value.column,
            end_line: value.end_line, end_column: value.end_column,
            hint: "Provide a #{Semantics::TypeSystem.type_name(slot.type)} expression for '#{slot.name}'."
          )
        end
        arguments
      rescue KeyError => error
        raise SemanticError.new(
          "Operation '#{name}' is missing argument #{error.key.inspect}",
          code: "S330",
          line: node.line, column: node.column,
          end_line: node.end_line, end_column: node.end_column
        )
      end

      def to_h
        {
          name: name,
          verbs: verbs,
          pattern: pattern.map(&:to_h),
          returns: self.class.serialize_type(return_type),
          effects: effects
        }.freeze
      end
    end
  end
end
