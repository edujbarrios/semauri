# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require "optparse"
require "fileutils"
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
      when "domains" then domains(argv)
      when "tokens" then tokens(argv)
      when "ast" then ast(argv)
      when "hir" then hir(argv)
      when "optimize" then optimize(argv)
      when "symbols" then symbols(argv)
      when "effects" then effects(argv)
      when "plan" then plan(argv)
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

    def domains(argv)
      raise ArgumentError, "Unexpected arguments: #{argv.join(' ')}" unless argv.empty?
      @stdout.puts JSON.pretty_generate(@compiler.domains.to_h)
      EXIT_SUCCESS
    end

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

    def optimize(argv)
      source = read_source!(argv)
      @stdout.puts JSON.pretty_generate(@compiler.optimized_hir(source).to_h)
      EXIT_SUCCESS
    end

    def symbols(argv)
      source = read_source!(argv)
      _ast, semantic = @compiler.analyze(source)
      @stdout.puts JSON.pretty_generate(semantic.symbols.map(&:to_h))
      EXIT_SUCCESS
    end

    def effects(argv)
      source = read_source!(argv)
      @stdout.puts JSON.pretty_generate(@compiler.effect_analysis(source).to_h)
      EXIT_SUCCESS
    end

    def plan(argv)
      source = read_source!(argv)
      @stdout.puts JSON.pretty_generate(@compiler.runtime_plan(source).to_h)
      EXIT_SUCCESS
    end

    def explain(argv)
      source = read_source!(argv)
      _ast, semantic = @compiler.analyze(source)
      semantic.explanations.each_with_index { |line, index| @stdout.puts "#{index + 1}. #{line}" }
      EXIT_SUCCESS
    end

    def check(argv)
      options = { allowed: nil }
      parser = OptionParser.new do |opts|
        opts.on("--allow EFFECT", "Allow one capability; repeat for multiple capabilities") do |value|
          (options[:allowed] ||= []) << value
        end
        opts.on("--allow-none", "Reject any program that declares effects") { options[:allowed] = [] }
      end
      parser.parse!(argv)

      source = read_source!(argv)
      @compiler.analyze(source)
      if options[:allowed]
        policy = Effects::CapabilityPolicy.new(allowed: options[:allowed])
        @compiler.validate_capabilities(source, policy: policy)
      end
      @stdout.puts "OK"
      EXIT_SUCCESS
    end

    def build(argv)
      options = { backend: nil, output: nil }
      parser = OptionParser.new do |opts|
        opts.on("-o", "--output PATH", "Write one output file, or a directory for multi-domain programs") { |value| options[:output] = value }
        opts.on("--backend NAME", "Override the backend for a single-domain program") { |value| options[:backend] = value }
      end
      parser.parse!(argv)

      source = read_source!(argv)
      result = @compiler.compile(source, backend: options[:backend])
      if result.outputs.empty? && result.runtime?
        raise BackendError.new(
          "Program produces a runtime plan but no build-time backend output",
          code: "S406",
          hint: "Inspect it with 'semauri plan FILE'. Execution support will be provided by the future runtime."
        )
      end

      if result.multi_domain?
        write_multi_domain(result, options[:output])
      elsif options[:output]
        File.write(options[:output], result.output)
        @stdout.puts "Built #{options[:output]} with #{result.backend}"
      else
        @stdout.write result.output
      end
      EXIT_SUCCESS
    end

    def write_multi_domain(result, output_path)
      if output_path
        FileUtils.mkdir_p(output_path)
        result.outputs.each do |output|
          path = File.join(output_path, output.filename)
          File.write(path, output.content)
          @stdout.puts "Built #{path} for #{output.domain} with #{output.backend}"
        end
        return
      end

      result.outputs.each_with_index do |output, index|
        @stdout.puts if index.positive?
        @stdout.puts "=== #{output.domain} [#{output.backend}] ==="
        @stdout.write output.content
        @stdout.puts unless output.content.end_with?("\n")
      end
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
        Usage: semauri <command> [options] [FILE]

        Commands:
          domains                  Print registered semantic domains as JSON
          tokens FILE              Print lexer tokens as JSON
          ast FILE                 Print the parsed syntax AST as JSON
          hir FILE                 Print typed HIR before optimization
          optimize FILE            Print optimized HIR and pass statistics
          symbols FILE             Print semantic symbols as JSON
          effects FILE             Print statically required effects/capabilities
          plan FILE                Print the lowered runtime operation plan as JSON
          explain FILE             Explain semantic decisions
          check FILE [--allow ...] Validate source and optionally enforce capabilities
          build FILE [-o PATH]     Compile one or more build-time domain outputs
          version                  Print version
      TEXT
    end
  end
end
