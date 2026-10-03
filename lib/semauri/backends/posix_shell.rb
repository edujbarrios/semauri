# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "shellwords"
require_relative "base"
require_relative "../errors"
require_relative "../ir/operation_plan"

module Semauri
  module Backends
    class PosixShell < Base
      def render(program)
        unless program.is_a?(IR::OperationPlan) && program.domain == :filesystem
          raise BackendError.new("posix-sh backend expects a filesystem operation plan", code: "S403")
        end

        lines = ["#!/usr/bin/env sh", "set -eu", ""]
        program.operations.each { |operation| lines << render_operation(operation) }
        lines.join("\n") + "\n"
      end

      private

      def render_operation(operation)
        arguments = operation.arguments
        case operation.name
        when :write
          "printf '%s' #{escape(arguments.fetch(:content))} > #{escape(arguments.fetch(:path))}"
        when :append
          "printf '%s' #{escape(arguments.fetch(:content))} >> #{escape(arguments.fetch(:path))}"
        when :copy
          "cp #{escape(arguments.fetch(:source))} #{escape(arguments.fetch(:destination))}"
        when :move
          "mv #{escape(arguments.fetch(:source))} #{escape(arguments.fetch(:destination))}"
        when :make_directory
          "mkdir -p #{escape(arguments.fetch(:path))}"
        when :touch
          "touch #{escape(arguments.fetch(:path))}"
        when :delete
          "rm -f #{escape(arguments.fetch(:path))}"
        else
          raise BackendError.new("Unsupported filesystem operation '#{operation.name}'", code: "S405")
        end
      end

      def escape(value)
        Shellwords.escape(value.to_s)
      end
    end
  end
end
