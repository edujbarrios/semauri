# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Domains
    class OperationResult
      attr_reader :artifact, :value, :explanations

      def initialize(artifact:, value: nil, explanations: [])
        @artifact = artifact
        @value = value
        @explanations = Array(explanations).map(&:to_s).freeze
        freeze
      end
    end
  end
end
