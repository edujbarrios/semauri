# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class DomainOperation < Node
      attr_reader :domain, :operation, :arguments

      def initialize(domain:, operation:, arguments:, **location)
        @domain = domain.to_sym
        @operation = operation.to_sym
        @arguments = arguments.transform_keys(&:to_sym).freeze
        super(**location)
      end

      def accept(visitor)
        visitor.visit_domain_operation(self)
      end
    end
  end
end
