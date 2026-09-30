# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "lexer"
require_relative "parser"
require_relative "semantics/resolver"
require_relative "backends/registry"

module Semauri
  CompilationResult = Struct.new(:output, :ast, :ir, :explanations, :symbols, keyword_init: true)

  class Compiler
    def initialize(vocabulary: Vocabulary::English.new, resolver: Semantics::Resolver.new,
                   backends: Backends::Registry.default)
      @vocabulary = vocabulary
      @resolver = resolver
      @backends = backends
    end

    def tokenize(source)
      Lexer.new(source, vocabulary: @vocabulary).tokens
    end

    def parse(source)
      Parser.new(tokenize(source)).parse
    end

    def analyze(source)
      ast = parse(source)
      [ast, @resolver.resolve(ast)]
    end

    def compile(source, backend: "html")
      ast, semantic = analyze(source)
      output = @backends.fetch(backend).render(semantic.program)
      CompilationResult.new(
        output: output,
        ast: ast,
        ir: semantic.program,
        explanations: semantic.explanations,
        symbols: semantic.symbols
      )
    end
  end
end
