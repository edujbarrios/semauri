# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class OperationPlan
      attr_reader :domain, :operations

      def initialize(domain:, operations: [])
        @domain = domain.to_sym
        @operations = operations.freeze
        freeze
      end

      def add(operation)
        self.class.new(domain: domain, operations: operations + [operation])
      end

      def to_h
        { domain: domain, operations: operations.map(&:to_h) }
      end
    end
  end
end
