# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

module Semauri
  module IR
    class WebDocument
      attr_reader :title, :subject, :title_origin

      def initialize(title:, subject: nil, title_origin:)
        @title = title
        @subject = subject
        @title_origin = title_origin
        freeze
      end

      def with_title(title, origin: :explicit)
        self.class.new(title: title, subject: subject, title_origin: origin)
      end

      def to_h
        { type: "web_document", title: title, subject: subject, title_origin: title_origin }
      end
    end
  end
end
