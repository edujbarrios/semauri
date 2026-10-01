# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../ir/program"
require_relative "../ir/runtime_plan"

module Semauri
  module Semantics
    class Result
      attr_reader :program_ir, :runtime_plan, :explanations, :symbols

      def initialize(program_ir:, explanations:, symbols:, runtime_plan: IR::RuntimePlan.new)
        raise ArgumentError, "program_ir must be an IR::Program" unless program_ir.is_a?(IR::Program)
        raise ArgumentError, "runtime_plan must be an IR::RuntimePlan" unless runtime_plan.is_a?(IR::RuntimePlan)

        @program_ir = program_ir
        @runtime_plan = runtime_plan
        @explanations = explanations.freeze
        @symbols = symbols.freeze
        freeze
      end

      def domains = program_ir.domains
      def multi_domain? = program_ir.size > 1
      def runtime? = !runtime_plan.empty?

      # Compatibility API: single-domain callers keep receiving the concrete
      # domain IR they received before ProgramIR existed.
      def program
        program_ir.single? ? program_ir.single_unit.artifact : program_ir
      end

      # Compatibility API for existing backends/tests. A multi-domain program
      # intentionally has no single semantic domain.
      def domain
        program_ir.single? ? program_ir.single_unit.domain : nil
      end
    end
  end
end
