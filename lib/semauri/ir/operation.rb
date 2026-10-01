# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class Operation
      attr_reader :name, :arguments, :effects, :source_span

      def initialize(name:, arguments:, effects: [], source_span: nil)
        @name = name.to_sym
        @arguments = arguments.transform_keys(&:to_sym).freeze
        @effects = Array(effects).map(&:to_sym).uniq.freeze
        @source_span = source_span
        freeze
      end

      def to_h
        {
          name: name,
          arguments: arguments,
          effects: effects,
          source_span: source_span&.to_h
        }
      end
    end
  end
end
