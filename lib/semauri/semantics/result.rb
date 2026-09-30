# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module Semantics
    Result = Struct.new(:program, :explanations, :symbols, keyword_init: true)
  end
end
