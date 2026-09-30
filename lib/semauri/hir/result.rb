# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module HIR
    Result = Struct.new(:program, :symbols, keyword_init: true) do
      def to_h
        {
          program: program.to_h,
          symbols: symbols.map(&:to_h)
        }
      end
    end
  end
end
