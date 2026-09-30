# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "result"
require_relative "passes/constant_folding"
require_relative "passes/dead_control_flow"

module Semauri
  module HIR
    module Optimization
      class PassManager
        def self.default
          new(passes: [Passes::ConstantFolding.new, Passes::DeadControlFlow.new])
        end

        def initialize(passes:)
          @passes = passes.freeze
        end

        def run(hir_result)
          program = hir_result.program
          reports = []
          total_changes = 0

          @passes.each do |pass|
            program, changes = pass.run(program, symbols: hir_result.symbols)
            total_changes += changes
            reports << { name: pass.name, changes: changes }
          end

          Result.new(
            program: program,
            symbols: hir_result.symbols,
            changes: total_changes,
            passes: reports.freeze
          )
        end
      end
    end
  end
end
