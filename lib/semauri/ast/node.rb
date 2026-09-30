# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module AST
    class Node
      attr_reader :line, :column

      def initialize(line:, column:)
        @line = line
        @column = column
        freeze
      end
    end
  end
end
