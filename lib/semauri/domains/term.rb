# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Domains
    class Term
      TOKEN_TYPES = {
        artifact: :DOMAIN_ARTIFACT,
        element: :DOMAIN_ELEMENT,
        property: :DOMAIN_PROPERTY
      }.freeze

      attr_reader :domain, :category, :kind

      def initialize(domain:, category:, kind:)
        @domain = domain.to_sym
        @category = category.to_sym
        @kind = kind.to_sym
        raise ArgumentError, "Unknown domain term category: #{category}" unless TOKEN_TYPES.key?(@category)
        freeze
      end

      def token_type = TOKEN_TYPES.fetch(category)

      def to_h
        { domain: domain, category: category, kind: kind }.freeze
      end
    end
  end
end
