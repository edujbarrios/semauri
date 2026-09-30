# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "node"

module Semauri
  module AST
    class CreateWeb < Node
      attr_reader :subject, :title

      def initialize(subject: nil, title: nil, line: nil, column: nil, end_line: nil, end_column: nil, span: nil)
        @subject = subject
        @title = title
        super(line: line, column: column, end_line: end_line, end_column: end_column, span: span)
      end

      def accept(visitor)
        visitor.visit_create_web(self)
      end
    end
  end
end
