# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class RuntimeValueRef
      attr_reader :id, :type, :producer_id, :source_span

      def initialize(id:, type:, producer_id:, source_span: nil)
        @id = id.to_s.freeze
        @type = type
        @producer_id = Integer(producer_id)
        @source_span = source_span
        freeze
      end

      def runtime_reference? = true
      def describe = "runtime #{type} #{id}"
      def to_s = id

      def to_h
        {
          kind: :runtime_value_ref,
          id: id,
          type: type.respond_to?(:to_h) ? type.to_h : type,
          producer_id: producer_id,
          source_span: source_span&.to_h
        }.freeze
      end
    end
  end
end
