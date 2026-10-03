# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "stringio"
require "tempfile"
require "tmpdir"
require_relative "test_helper"
require "semauri/cli"

class CLIRunTest < Minitest::Test
  def with_source(source)
    Tempfile.create(["semauri-run", ".sema"]) do |file|
      file.write(source)
      file.flush
      yield file.path
    end
  end

  def cli
    stdout = StringIO.new
    stderr = StringIO.new
    [Semauri::CLI.new(stdout: stdout, stderr: stderr), stdout, stderr]
  end

  def test_run_requires_explicit_capability_authorization
    with_source('Write "hello" to "notes.txt".') do |path|
      instance, _stdout, stderr = cli
      status = instance.run(["run", path])

      assert_equal Semauri::CLI::EXIT_COMPILE_ERROR, status
      assert_includes stderr.string, "S334"
      assert_includes stderr.string, "filesystem_write"
    end
  end

  def test_run_executes_filesystem_plan_in_requested_directory
    Dir.mktmpdir("semauri-run") do |dir|
      source = <<~SEMA
        Mkdir "build".
        Write "hello" to "build/notes.txt".
        Append " world" to "build/notes.txt".
      SEMA

      with_source(source) do |path|
        instance, _stdout, stderr = cli
        status = instance.run(["run", "--allow", "filesystem_write", "--cwd", dir, path])

        assert_equal 0, status
        assert_empty stderr.string
        assert_equal "hello world", File.read(File.join(dir, "build", "notes.txt"))
      end
    end
  end

  def test_run_dry_run_never_performs_effects
    Dir.mktmpdir("semauri-run") do |dir|
      with_source('Write "hello" to "notes.txt".') do |path|
        instance, stdout, stderr = cli
        status = instance.run(["run", "--dry-run", "--cwd", dir, path])

        assert_equal 0, status
        assert_empty stderr.string
        assert_includes stdout.string, "printf"
        refute File.exist?(File.join(dir, "notes.txt"))
      end
    end
  end
end
