# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Semantics
    class Value
      attr_reader :type, :value, :definition_span

      def initialize(type:, value:, definition_span: nil)
        @type = type.to_sym
        @value = value
        @definition_span = definition_span
        freeze
      end

      def describe
        "#{type} #{value.inspect}"
      end
    end
  end
end
