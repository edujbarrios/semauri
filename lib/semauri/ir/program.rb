# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class Program
      Unit = Struct.new(:domain, :artifact, keyword_init: true) do
        def initialize(domain:, artifact:)
          super(domain: domain.to_sym, artifact: artifact)
          freeze
        end

        def to_h
          { domain: domain, artifact: artifact.respond_to?(:to_h) ? artifact.to_h : artifact.class.name }.freeze
        end
      end

      attr_reader :units

      def initialize(units: [])
        normalized = units.map do |unit|
          unit.is_a?(Unit) ? unit : Unit.new(domain: unit.fetch(:domain), artifact: unit.fetch(:artifact))
        end

        domains = normalized.map(&:domain)
        duplicates = domains.group_by(&:itself).select { |_domain, values| values.length > 1 }.keys
        raise ArgumentError, "Duplicate ProgramIR domains: #{duplicates.join(', ')}" unless duplicates.empty?

        @units = normalized.freeze
        @index = @units.to_h { |unit| [unit.domain, unit] }.freeze
        freeze
      end

      def domains = units.map(&:domain).freeze
      def empty? = units.empty?
      def single? = units.length == 1
      def size = units.length

      def include?(domain)
        @index.key?(domain.to_sym)
      end

      def fetch(domain)
        @index.fetch(domain.to_sym)
      end

      def artifact(domain)
        @index[domain.to_sym]&.artifact
      end

      def put(domain:, artifact:)
        key = domain.to_sym
        replacement = Unit.new(domain: key, artifact: artifact)

        if include?(key)
          self.class.new(units: units.map { |unit| unit.domain == key ? replacement : unit })
        else
          self.class.new(units: units + [replacement])
        end
      end

      def single_unit
        raise ArgumentError, "ProgramIR contains #{size} domains" unless single?
        units.first
      end

      def to_h
        { kind: :program_ir, domains: domains, units: units.map(&:to_h) }.freeze
      end
    end
  end
end
