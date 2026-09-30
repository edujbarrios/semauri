# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module AST
    class Reference
      MODES = %i[pronoun kind named].freeze

      attr_reader :mode, :entity_type, :name

      def self.pronoun
        new(mode: :pronoun)
      end

      def self.kind(entity_type)
        new(mode: :kind, entity_type: entity_type)
      end

      def self.named(entity_type, name)
        new(mode: :named, entity_type: entity_type, name: name)
      end

      def initialize(mode:, entity_type: nil, name: nil)
        raise ArgumentError, "Unknown reference mode: #{mode}" unless MODES.include?(mode)

        @mode = mode
        @entity_type = entity_type
        @name = name
        freeze
      end

      def to_h
        { mode: mode, entity_type: entity_type, name: name }
      end
    end
  end
end
