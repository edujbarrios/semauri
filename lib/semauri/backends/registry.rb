# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "../errors"
require_relative "html"

module Semauri
  module Backends
    class Registry
      def self.default
        new.register("html") { HTML.new }
      end

      def initialize
        @factories = {}
      end

      def register(name, &factory)
        raise ArgumentError, "A backend factory block is required" unless factory

        @factories[name.to_s] = factory
        self
      end

      def fetch(name)
        factory = @factories[name.to_s]
        raise BackendError.new("Unknown backend '#{name}'", code: "S402") unless factory

        factory.call
      end

      def names
        @factories.keys.sort.freeze
      end
    end
  end
end
