# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class CreateWeb < Node
      attr_reader :subject, :title

      def initialize(subject: nil, title: nil, line:, column:)
        @subject = subject
        @title = title
        super(line: line, column: column)
      end

      def accept(visitor)
        visitor.visit_create_web(self)
      end
    end
  end
end
