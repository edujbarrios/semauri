# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Effects
    EffectUse = Struct.new(:effect, :domain, :operation, :span, keyword_init: true) do
      def initialize(effect:, domain:, operation:, span: nil)
        super(effect: effect.to_sym, domain: domain.to_sym, operation: operation.to_sym, span: span)
        freeze
      end

      def to_h
        {
          effect: effect,
          domain: domain,
          operation: operation,
          span: span&.to_h
        }.freeze
      end
    end

    class Analysis
      attr_reader :uses

      def initialize(uses:)
        @uses = uses.freeze
        freeze
      end

      def effects
        uses.map(&:effect).uniq.sort.freeze
      end

      def pure? = effects.empty?

      def uses_for(effect)
        key = effect.to_sym
        uses.select { |use| use.effect == key }.freeze
      end

      def to_h
        {
          effects: effects,
          pure: pure?,
          uses: uses.map(&:to_h)
        }.freeze
      end
    end
  end
end
