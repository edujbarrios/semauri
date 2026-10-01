# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class SemanticDomainsTest < Minitest::Test
  class TestElement
    attr_reader :id, :kind, :label, :properties

    def initialize(id:, kind:, label:, properties: {})
      @id = id
      @kind = kind.to_sym
      @label = label
      @properties = properties.freeze
      freeze
    end

    def with_property(name, value, source_span: nil)
      self.class.new(id: id, kind: kind, label: label, properties: properties.merge(name.to_sym => value))
    end
  end

  class TestArtifact
    attr_reader :title, :elements

    def initialize(title:, elements: [])
      @title = title
      @elements = elements.freeze
      freeze
    end

    def with_title(title)
      self.class.new(title: title, elements: elements)
    end

    def add_element(element)
      self.class.new(title: title, elements: elements + [element])
    end

    def replace_element(element)
      self.class.new(title: title, elements: elements.map { |candidate| candidate.id == element.id ? element : candidate })
    end
  end

  class CanvasDomain < Semauri::Domains::Definition
    def initialize
      super(
        name: :canvas,
        artifacts: { "canvas" => :canvas },
        elements: { "badge" => :badge },
        properties: { "tone" => :tone }
      )
    end

    def property_type(property)
      :color if property.to_sym == :tone
    end

    def create_artifact(kind:, subject:, title:)
      raise ArgumentError, "unexpected artifact" unless kind.to_sym == :canvas
      TestArtifact.new(title: title || subject || "Untitled Canvas")
    end

    def add_element(artifact:, kind:, label:, entities:)
      element = entities.register(kind: kind) do |id|
        TestElement.new(id: id, kind: kind, label: label || kind.to_s.capitalize)
      end
      [artifact.add_element(element), element]
    end
  end

  class CanvasBackend
    def render(program)
      element = program.elements.first
      "#{program.title}|#{element.kind}|#{element.label}|#{element.properties[:tone]}"
    end
  end

  def test_custom_domain_compiles_without_parser_or_lexer_changes
    domains = Semauri::Domains::Registry.new.register(CanvasDomain.new)
    backends = Semauri::Backends::Registry.new.register("canvas") { CanvasBackend.new }
    compiler = Semauri::Compiler.new(domains: domains, backends: backends)

    source = <<~SEMA
      Create a canvas called Status Board.
      Add a badge called Health.
      Set the tone of the badge called Health to green.
    SEMA

    result = compiler.compile(source, backend: "canvas")

    assert_equal "Status Board|badge|Health|green", result.output
    create = result.ast.statements.first
    property = result.ast.statements.last
    assert_equal :canvas, create.domain
    assert_equal :canvas, property.domain
    assert_equal :tone, property.property
  end

  def test_domain_terms_are_generic_lexer_categories
    domains = Semauri::Domains::Registry.new.register(CanvasDomain.new)
    compiler = Semauri::Compiler.new(domains: domains)

    tokens = compiler.tokenize("Create a canvas. Add a badge. Set the tone of it to red.")

    assert_includes tokens.map(&:type), :DOMAIN_ARTIFACT
    assert_includes tokens.map(&:type), :DOMAIN_ELEMENT
    assert_includes tokens.map(&:type), :DOMAIN_PROPERTY
  end

  def test_implicit_property_syntax_is_rejected_when_domains_make_it_ambiguous
    domains = Semauri::Domains::Registry.default.register(CanvasDomain.new)
    compiler = Semauri::Compiler.new(domains: domains)

    error = assert_raises(Semauri::ParseError) do
      compiler.parse("Create a web called Shop. Add a button called Buy. Make it blue.")
    end

    assert_equal "S236", error.code
    assert_includes error.message, "Cannot infer a unique property"
  end

  def test_domain_metadata_is_introspectable
    registry = Semauri::Domains::Registry.new.register(CanvasDomain.new)
    metadata = registry.to_h.fetch(:domains).first

    assert_equal :canvas, metadata.fetch(:name)
    assert_equal({ "canvas" => :canvas }, metadata.fetch(:artifacts))
    assert_equal({ "badge" => :badge }, metadata.fetch(:elements))
    assert_equal({ "tone" => :tone }, metadata.fetch(:properties))
  end

  def test_registry_rejects_surface_term_collisions_transactionally
    first = Class.new(Semauri::Domains::Definition) do
      def initialize = super(name: :one, artifacts: { "thing" => :thing })
    end
    second = Class.new(Semauri::Domains::Definition) do
      def initialize = super(name: :two, artifacts: { "other" => :other }, elements: { "thing" => :thing })
    end

    registry = Semauri::Domains::Registry.new.register(first.new)
    error = assert_raises(ArgumentError) { registry.register(second.new) }

    assert_includes error.message, "conflicts"
    assert_nil registry.classify("other")
    assert_equal [:one], registry.names
  end

  def test_english_vocabulary_rejects_domain_overrides_of_core_words
    invalid = Class.new(Semauri::Domains::Definition) do
      def initialize = super(name: :invalid, artifacts: { "if" => :artifact })
    end
    registry = Semauri::Domains::Registry.new.register(invalid.new)

    error = assert_raises(ArgumentError) { Semauri::Vocabulary::English.new(domains: registry) }
    assert_includes error.message, "reserved English vocabulary"
  end
end
