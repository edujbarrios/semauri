# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "domains/registry"
require_relative "lexer"
require_relative "parser"
require_relative "hir/builder"
require_relative "hir/lowerer"
require_relative "hir/optimization/pass_manager"
require_relative "backends/registry"

module Semauri
  CompilationResult = Struct.new(
    :output, :ast, :hir, :optimized_hir, :ir, :explanations, :symbols,
    keyword_init: true
  )

  class Compiler
    attr_reader :domains

    def initialize(domains: Domains::Registry.default, vocabulary: nil, hir_builder: nil,
                   optimizer: HIR::Optimization::PassManager.default,
                   lowerer: nil, backends: Backends::Registry.default)
      @domains = domains
      @vocabulary = vocabulary || Vocabulary::English.new(domains: domains)
      @hir_builder = hir_builder || HIR::Builder.new(domains: domains)
      @optimizer = optimizer
      @lowerer = lowerer || HIR::Lowerer.new(domains: domains)
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

    # Analysis intentionally lowers unoptimized HIR so `explain` describes the
    # source program rather than compiler rewrites.
    def analyze(source)
      ast = parse(source)
      hir_result = @hir_builder.build(ast)
      [ast, @lowerer.lower(hir_result)]
    end

    def compile(source, backend: "html")
      ast = parse(source)
      hir_result = @hir_builder.build(ast)
      optimized = @optimizer.run(hir_result)

      # Keep a source-oriented trace while lowering the optimized program for
      # the generated artifact. This deliberately separates observability from
      # optimization implementation details.
      source_semantic = @lowerer.lower(hir_result)
      semantic = @lowerer.lower(optimized)
      output = @backends.fetch(backend).render(semantic.program)

      CompilationResult.new(
        output: output,
        ast: ast,
        hir: hir_result.program,
        optimized_hir: optimized.program,
        ir: semantic.program,
        explanations: source_semantic.explanations,
        symbols: semantic.symbols
      )
    end
  end
end
