# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class ForEach < Node
      attr_reader :variable_name, :iterable, :body, :binding_span

      def initialize(variable_name:, iterable:, body:, binding_span:, span:)
        @variable_name = variable_name
        @iterable = iterable
        @body = body
        @binding_span = binding_span
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_for_each(self)
      end
    end
  end
end
