# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "lexer"
require_relative "parser"
require_relative "hir/builder"
require_relative "hir/lowerer"
require_relative "backends/registry"

module Semauri
  CompilationResult = Struct.new(:output, :ast, :hir, :ir, :explanations, :symbols, keyword_init: true)

  class Compiler
    def initialize(vocabulary: Vocabulary::English.new, hir_builder: HIR::Builder.new,
                   lowerer: HIR::Lowerer.new, backends: Backends::Registry.default)
      @vocabulary = vocabulary
      @hir_builder = hir_builder
      @lowerer = lowerer
      @backends = backends
    end

    def tokenize(source)
      Lexer.new(source, vocabulary: @vocabulary).tokens
    end

    def parse(source)
      Parser.new(tokenize(source)).parse
    end

    def hir(source)
      @hir_builder.build(parse(source))
    end

    def analyze(source)
      ast = parse(source)
      hir_result = @hir_builder.build(ast)
      [ast, @lowerer.lower(hir_result)]
    end

    def compile(source, backend: "html")
      ast = parse(source)
      hir_result = @hir_builder.build(ast)
      semantic = @lowerer.lower(hir_result)
      output = @backends.fetch(backend).render(semantic.program)

      CompilationResult.new(
        output: output,
        ast: ast,
        hir: hir_result.program,
        ir: semantic.program,
        explanations: semantic.explanations,
        symbols: semantic.symbols
      )
    end
  end
end
