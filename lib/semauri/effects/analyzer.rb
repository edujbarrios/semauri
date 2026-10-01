# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "analysis"
require_relative "../hir/node"

module Semauri
  module Effects
    class Analyzer
      def analyze(hir_result_or_node)
        root = hir_result_or_node.respond_to?(:program) ? hir_result_or_node.program : hir_result_or_node
        uses = []
        collect(root, uses)
        Analysis.new(uses: uses)
      end

      private

      def collect(value, uses)
        case value
        when HIR::Node
          collect_node(value, uses)
        when Array
          value.each { |item| collect(item, uses) }
        when Hash
          value.each_value { |item| collect(item, uses) }
        end
      end

      def collect_node(node, uses)
        if node.kind == :domain_operation
          domain = node.fields.fetch(:domain)
          operation = node.fields.fetch(:operation)
          Array(node.fields[:effects]).each do |effect|
            uses << EffectUse.new(effect: effect, domain: domain, operation: operation, span: node.span)
          end
        end

        node.fields.each_value { |field| collect(field, uses) }
      end
    end
  end
end
