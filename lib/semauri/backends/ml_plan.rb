# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require_relative "base"
require_relative "../errors"
require_relative "../ir/ml_plan"

module Semauri
  module Backends
    class MLPlan < Base
      def render(program)
        unless program.is_a?(IR::MLPlan)
          raise BackendError.new("ml-plan backend expects an ML semantic plan", code: "S403")
        end

        JSON.pretty_generate(program.to_h) + "\n"
      end
    end
  end
end
