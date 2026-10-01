# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module HIR
    module DomainOperationValidation
      def visit_domain_operation(node)
        hir = super
        domain = @domains.fetch(node.domain)
        operation = domain.operation(node.operation)
        domain.validate_operation_arguments!(
          operation: operation,
          arguments: hir.fields.fetch(:arguments),
          node: hir
        )
        hir
      end
    end

    Builder.prepend(DomainOperationValidation)
  end
end
