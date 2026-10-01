# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "domains/registry"
require_relative "lexer"
require_relative "parser"
require_relative "hir/builder"
require_relative "hir/lowerer"
require_relative "hir/optimization/pass_manager"
require_relative "effects/analyzer"
require_relative "effects/capability_policy"
require_relative "backends/registry"

module Semauri
  DomainOutput = Struct.new(:domain, :backend, :content, keyword_init: true) do
    def initialize(domain:, backend:, content:)
      super(domain: domain.to_sym, backend: backend.to_s, content: content.to_s)
      freeze
    end

    def filename
      "#{domain}.#{backend.gsub(/[^a-zA-Z0-9._-]+/, '-')}"
    end
  end

  CompilationResult = Struct.new(
    :output, :backend, :outputs, :ast, :hir, :optimized_hir, :ir, :explanations, :symbols, :effects,
    keyword_init: true
  ) do
    def multi_domain? = outputs.length > 1
  end

  class Compiler
    attr_reader :domains

    def initialize(domains: Domains::Registry.default, vocabulary: nil, hir_builder: nil,
                   optimizer: HIR::Optimization::PassManager.default,
                   lowerer: nil, effect_analyzer: Effects::Analyzer.new,
                   backends: Backends::Registry.default)
      @domains = domains
      @vocabulary = vocabulary || Vocabulary::English.new(domains: domains)
      @hir_builder = hir_builder || HIR::Builder.new(domains: domains)
      @optimizer = optimizer
      @lowerer = lowerer || HIR::Lowerer.new(domains: domains)
      @effect_analyzer = effect_analyzer
      @backends = backends
    end

    def tokenize(source)
      Lexer.new(source, vocabulary: @vocabulary).tokens
    end

    def parse(source)
      Parser.new(tokenize(source), domains: domains).parse
    end

    def hir(source)
      @hir_builder.build(parse(source))
    end

    def optimized_hir(source)
      @optimizer.run(hir(source))
    end

    def effect_analysis(source)
      @effect_analyzer.analyze(hir(source))
    end

    def validate_capabilities(source, policy:)
      policy.validate!(effect_analysis(source))
    end

    # Analysis intentionally lowers unoptimized HIR so `explain` describes the
    # source program rather than compiler rewrites.
    def analyze(source)
      ast = parse(source)
      hir_result = @hir_builder.build(ast)
      [ast, @lowerer.lower(hir_result)]
    end

    def compile(source, backend: nil, capability_policy: nil)
      ast = parse(source)
      hir_result = @hir_builder.build(ast)
      effects = @effect_analyzer.analyze(hir_result)
      capability_policy&.validate!(effects)
      optimized = @optimizer.run(hir_result)

      # Keep a source-oriented trace while lowering the optimized program for
      # generated outputs. This deliberately separates observability from
      # optimization implementation details.
      source_semantic = @lowerer.lower(hir_result)
      semantic = @lowerer.lower(optimized)
      outputs = render_program(semantic.program_ir, backend: backend).freeze
      single = outputs.one? ? outputs.first : nil

      CompilationResult.new(
        output: single&.content,
        backend: single&.backend,
        outputs: outputs,
        ast: ast,
        hir: hir_result.program,
        optimized_hir: optimized.program,
        ir: semantic.program,
        explanations: source_semantic.explanations,
        symbols: semantic.symbols,
        effects: effects
      )
    end

    private

    def render_program(program_ir, backend:)
      if backend && !program_ir.single?
        raise BackendError.new(
          "A single backend override cannot render a multi-domain program",
          code: "S405",
          hint: "Build without --backend so each semantic domain uses its own default backend."
        )
      end

      program_ir.units.map do |unit|
        backend_name = backend&.to_s || default_backend_for(unit.domain)
        content = @backends.fetch(backend_name).render(unit.artifact)
        DomainOutput.new(domain: unit.domain, backend: backend_name, content: content)
      end
    end

    def default_backend_for(domain_name)
      backend = domains.fetch(domain_name).default_backend
      return backend if backend

      raise BackendError.new(
        "Semantic domain '#{domain_name}' has no default backend",
        code: "S404"
      )
    end
  end
end
