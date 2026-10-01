# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require "stringio"
require "tempfile"
require_relative "test_helper"
require "semauri/cli"

class CLIEffectsTest < Minitest::Test
  def with_source(source)
    Tempfile.create(["semauri-effects", ".sema"]) do |file|
      file.write(source)
      file.flush
      yield file.path
    end
  end

  def cli
    stdout = StringIO.new
    stderr = StringIO.new
    instance = Semauri::CLI.new(stdout: stdout, stderr: stderr)
    [instance, stdout, stderr]
  end

  def test_effects_command_prints_manifest
    with_source('Copy "a.txt" to "b.txt".') do |path|
      instance, stdout, stderr = cli

      status = instance.run(["effects", path])
      manifest = JSON.parse(stdout.string)

      assert_equal 0, status
      assert_empty stderr.string
      assert_equal %w[filesystem_read filesystem_write], manifest.fetch("effects")
      assert_equal "filesystem", manifest.fetch("uses").first.fetch("domain")
    end
  end

  def test_check_can_reject_all_effects
    with_source('Write "hello" to "notes.txt".') do |path|
      instance, _stdout, stderr = cli

      status = instance.run(["check", "--allow-none", path])

      assert_equal Semauri::CLI::EXIT_COMPILE_ERROR, status
      assert_includes stderr.string, "S334"
      assert_includes stderr.string, "filesystem_write"
    end
  end

  def test_check_accepts_explicit_capability_allow_list
    with_source('Copy "a.txt" to "b.txt".') do |path|
      instance, stdout, stderr = cli

      status = instance.run([
        "check",
        "--allow", "filesystem_read",
        "--allow", "filesystem_write",
        path
      ])

      assert_equal 0, status
      assert_equal "OK\n", stdout.string
      assert_empty stderr.string
    end
  end
end
