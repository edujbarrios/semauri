# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  class SourceSpan
    attr_reader :start_line, :start_column, :end_line, :end_column

    def self.point(line, column)
      new(start_line: line, start_column: column, end_line: line, end_column: column + 1)
    end

    def initialize(start_line:, start_column:, end_line:, end_column:)
      @start_line = Integer(start_line)
      @start_column = Integer(start_column)
      @end_line = Integer(end_line)
      @end_column = Integer(end_column)
      validate!
      freeze
    end

    def single_line?
      start_line == end_line
    end

    def to_h
      {
        start: { line: start_line, column: start_column },
        end: { line: end_line, column: end_column }
      }
    end

    private

    def validate!
      raise ArgumentError, "Source span coordinates must be positive" if [start_line, start_column, end_line, end_column].any? { |value| value < 1 }
      return if end_line > start_line
      return if end_line == start_line && end_column >= start_column

      raise ArgumentError, "Source span end must not precede its start"
    end
  end
end
