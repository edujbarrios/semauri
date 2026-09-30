# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Semantics
    class Symbol
      attr_reader :id, :name, :kind, :type, :definition_span

      def initialize(id:, name:, kind:, type:, definition_span: nil)
        @id = Integer(id)
        @name = name.to_s.downcase.freeze
        @kind = kind.to_sym
        @type = type
        @definition_span = definition_span
        freeze
      end

      def to_h
        {
          id: id,
          name: name,
          kind: kind,
          type: type.respond_to?(:to_h) ? type.to_h : type,
          definition_span: definition_span&.to_h
        }
      end

      def inspect
        "#<#{self.class} ##{id} #{kind} #{name}: #{type}>"
      end
    end
  end
end
