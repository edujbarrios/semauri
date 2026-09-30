# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "element"

module Semauri
  module IR
    class WebDocument
      attr_reader :title, :subject, :title_origin, :elements

      def initialize(title:, subject: nil, title_origin:, elements: [])
        @title = title.freeze
        @subject = subject&.freeze
        @title_origin = title_origin
        @elements = elements.freeze
        freeze
      end

      def with_title(title, origin: :explicit)
        self.class.new(title: title, subject: subject, title_origin: origin, elements: elements)
      end

      def add_element(element)
        self.class.new(title: title, subject: subject, title_origin: title_origin, elements: elements + [element])
      end

      def replace_element(element)
        replacement = elements.map { |candidate| candidate.id == element.id ? element : candidate }
        self.class.new(title: title, subject: subject, title_origin: title_origin, elements: replacement)
      end

      def to_h
        {
          type: "web_document",
          title: title,
          subject: subject,
          title_origin: title_origin,
          elements: elements.map(&:to_h)
        }
      end
    end
  end
end
