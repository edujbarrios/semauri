# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Domains
    class ActionCandidates
      attr_reader :terms

      def initialize(terms)
        @terms = Array(terms).freeze
        raise ArgumentError, "ActionCandidates requires at least one term" if @terms.empty?
        raise ArgumentError, "ActionCandidates only accepts action terms" unless @terms.all? { |term| term.category == :action }
        freeze
      end

      def token_type = :DOMAIN_ACTION

      def to_h
        { category: :action, candidates: terms.map(&:to_h) }.freeze
      end
    end
  end
end
