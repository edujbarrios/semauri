# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class RuntimeOperation
      attr_reader :id, :domain, :name, :arguments, :return_type, :result, :effects, :source_span

      def initialize(id:, domain:, name:, arguments:, return_type:, result:, effects: [], source_span: nil)
        @id = Integer(id)
        @domain = domain.to_sym
        @name = name.to_sym
        @arguments = arguments.transform_keys(&:to_sym).freeze
        @return_type = return_type
        @result = result
        @effects = Array(effects).map(&:to_sym).uniq.freeze
        @source_span = source_span
        freeze
      end

      def to_h
        {
          id: id,
          domain: domain,
          name: name,
          arguments: serialize(arguments),
          return_type: serialize_type(return_type),
          result: result&.to_h,
          effects: effects,
          source_span: source_span&.to_h
        }.freeze
      end

      private

      def serialize(value)
        case value
        when Array then value.map { |item| serialize(item) }
        when Hash then value.transform_values { |item| serialize(item) }
        else value.respond_to?(:to_h) ? value.to_h : value
        end
      end

      def serialize_type(type)
        type.respond_to?(:to_h) ? type.to_h : type
      end
    end
  end
end
