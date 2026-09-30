# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require "optparse"
require_relative "compiler"
require_relative "ast/serializer"
require_relative "version"

module Semauri
  class CLI
    EXIT_SUCCESS = 0
    EXIT_USAGE = 64
    EXIT_COMPILE_ERROR = 65

    def initialize(stdout: $stdout, stderr: $stderr, compiler: Compiler.new)
      @stdout = stdout
      @stderr = stderr
      @compiler = compiler
      @current_source = nil
    end

    def run(argv)
      command = argv.shift
      return help if command.nil? || %w[help --help -h].include?(command)
      return version if %w[version --version -v].include?(command)

      case command
      when "tokens" then tokens(argv)
      when "ast" then ast(argv)
      when "hir" then hir(argv)
      when "symbols" then symbols(argv)
      when "explain" then explain(argv)
      when "check" then check(argv)
      when "build" then build(argv)
      else
        @stderr.puts "Unknown command: #{command}"
        @stderr.puts usage
        EXIT_USAGE
      end
    rescue ArgumentError => e
      @stderr.puts e.message
      EXIT_USAGE
    rescue Semauri::Error => e
      @stderr.puts e.diagnostic(source: @current_source)
      EXIT_COMPILE_ERROR
    rescue Errno::ENOENT => e
      @stderr.puts "S001: #{e.message}"
      EXIT_USAGE
    end

    private

    def tokens(argv)
      source = read_source!(argv)
      @stdout.puts JSON.pretty_generate(@compiler.tokenize(source).map(&:to_h))
      EXIT_SUCCESS
    end

    def ast(argv)
      source = read_source!(argv)
      tree = @compiler.parse(source)
      @stdout.puts JSON.pretty_generate(AST::Serializer.new.serialize(tree))
      EXIT_SUCCESS
    end

    def hir(argv)
      source = read_source!(argv)
      @stdout.puts JSON.pretty_generate(@compiler.hir(source).to_h)
      EXIT_SUCCESS
    end

    def symbols(argv)
      source = read_source!(argv)
      _ast, semantic = @compiler.analyze(source)
      @stdout.puts JSON.pretty_generate(semantic.symbols.map(&:to_h))
      EXIT_SUCCESS
    end

    def explain(argv)
      source = read_source!(argv)
      _ast, semantic = @compiler.analyze(source)
      semantic.explanations.each_with_index { |line, index| @stdout.puts "#{index + 1}. #{line}" }
      EXIT_SUCCESS
    end

    def check(argv)
      source = read_source!(argv)
      @compiler.analyze(source)
      @stdout.puts "OK"
      EXIT_SUCCESS
    end

    def build(argv)
      options = { backend: "html", output: nil }
      parser = OptionParser.new do |opts|
        opts.on("-o", "--output PATH", "Write output to PATH") { |value| options[:output] = value }
        opts.on("--backend NAME", "Select a backend (default: html)") { |value| options[:backend] = value }
      end
      parser.parse!(argv)

      source = read_source!(argv)
      result = @compiler.compile(source, backend: options[:backend])

      if options[:output]
        File.write(options[:output], result.output)
        @stdout.puts "Built #{options[:output]}"
      else
        @stdout.write result.output
      end
      EXIT_SUCCESS
    end

    def read_source!(argv)
      path = argv.shift
      raise ArgumentError, usage unless path
      raise ArgumentError, "Unexpected arguments: #{argv.join(' ')}" unless argv.empty?

      @current_source = File.read(path, encoding: "UTF-8")
    end

    def version
      @stdout.puts "Semauri #{VERSION}"
      EXIT_SUCCESS
    end

    def help
      @stdout.puts usage
      EXIT_SUCCESS
    end

    def usage
      <<~TEXT
        Usage: semauri <command> [options] FILE

        Commands:
          tokens FILE              Print lexer tokens as JSON
          ast FILE                 Print the parsed syntax AST as JSON
          hir FILE                 Print typed HIR and symbol references as JSON
          symbols FILE             Print semantic symbols as JSON
          explain FILE             Explain semantic decisions
          check FILE               Validate source without generating output
          build FILE [-o PATH]     Compile source (HTML by default)
          version                  Print version
      TEXT
    end
  end
end
