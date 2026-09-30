# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Backends
    class Base
      def render(_program)
        raise NotImplementedError, "Backends must implement #render"
      end
    end
  end
end
