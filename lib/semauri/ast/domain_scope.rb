# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class DomainScope < Node
      attr_reader :domain, :body

      def initialize(domain:, body:, span:)
        @domain = domain.to_sym
        @body = body
        super(span: span)
      end

      def accept(visitor)
        visitor.visit_domain_scope(self)
      end
    end
  end
end
