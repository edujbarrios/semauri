# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "definition"
require_relative "../semantics/nominal_type"

module Semauri
  module Domains
    class ML < Definition
      DATASET = Semantics::NominalType.new(domain: :ml, name: :dataset, base_type: :string)
      MODEL = Semantics::NominalType.new(domain: :ml, name: :model, base_type: :string)
      DEVICE = Semantics::NominalType.new(domain: :ml, name: :device, base_type: :string)
      TRAINING_RUN = Semantics::NominalType.new(domain: :ml, name: :training_run, base_type: :string)
      INFERENCE_RUN = Semantics::NominalType.new(domain: :ml, name: :inference_run, base_type: :string)

      def initialize
        super(
          name: :ml,
          types: [DATASET, MODEL, DEVICE, TRAINING_RUN, INFERENCE_RUN],
          operations: [
            Operation.new(
              name: :open_dataset,
              verbs: ["open"],
              pattern: [
                Operation.literal("dataset"),
                Operation.expression(:source, type: :string)
              ],
              returns: DATASET,
              effects: [:filesystem_read]
            ),
            Operation.new(
              name: :load_model,
              verbs: ["load"],
              pattern: [
                Operation.literal("model"),
                Operation.expression(:identifier, type: :string)
              ],
              returns: MODEL,
              effects: [:model_load]
            ),
            Operation.new(
              name: :select_device,
              verbs: ["select"],
              pattern: [
                Operation.literal("device"),
                Operation.expression(:name, type: :string)
              ],
              returns: DEVICE,
              effects: []
            ),
            Operation.new(
              name: :train,
              verbs: ["train"],
              pattern: [
                Operation.expression(:model, type: MODEL),
                Operation.literal("using"),
                Operation.expression(:dataset, type: DATASET),
                Operation.literal("on"),
                Operation.expression(:device, type: DEVICE),
                Operation.literal("for"),
                Operation.expression(:epochs, type: :number),
                Operation.literal("epochs")
              ],
              returns: TRAINING_RUN,
              effects: [:compute, :model_training]
            ),
            Operation.new(
              name: :infer,
              verbs: ["infer"],
              pattern: [
                Operation.expression(:model, type: MODEL),
                Operation.literal("on"),
                Operation.expression(:dataset, type: DATASET),
                Operation.literal("using"),
                Operation.expression(:device, type: DEVICE)
              ],
              returns: INFERENCE_RUN,
              effects: [:compute, :model_inference]
            )
          ]
        )
      end
    end
  end
end
