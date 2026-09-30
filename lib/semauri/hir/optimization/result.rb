# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module HIR
    module Optimization
      Result = Struct.new(:program, :symbols, :changes, :passes, keyword_init: true) do
        def to_h
          {
            program: program.to_h,
            symbols: symbols.map(&:to_h),
            optimization: {
              total_changes: changes,
              passes: passes
            }
          }
        end
      end
    end
  end
end
