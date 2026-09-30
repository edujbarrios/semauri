# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "symbol"

module Semauri
  module Semantics
    class SymbolTable
      def initialize
        @next_id = 1
        @symbols = []
      end

      def create(name:, kind:, type:, definition_span: nil)
        symbol = Symbol.new(
          id: @next_id,
          name: name,
          kind: kind,
          type: type,
          definition_span: definition_span
        )
        @next_id += 1
        @symbols << symbol
        symbol
      end

      def symbols
        @symbols.dup.freeze
      end

      def fetch(id)
        @symbols.fetch(Integer(id) - 1)
      rescue IndexError
        nil
      end
    end
  end
end
