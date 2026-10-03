# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class FilesystemWorkflowsTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def source
    <<~SEMA
      Within filesystem:
        Mkdir "build".
        Write "hello" to "build/notes.txt".
        Append " world" to "build/notes.txt".
        Copy "build/notes.txt" to "build/backup.txt".
        Move "build/backup.txt" to "build/archive.txt".
        Touch "build/complete.marker".
        Delete "build/old.tmp".
      End.
    SEMA
  end

  def test_practical_filesystem_operations_lower_in_order
    result = @compiler.compile(source)
    plan = result.output.program

    assert_equal :filesystem, plan.domain
    assert_equal %i[make_directory write append copy move touch delete], plan.operations.map(&:name)
  end

  def test_practical_filesystem_effects_are_explicit
    effects = @compiler.effect_analysis(source)
    assert_equal %i[filesystem_read filesystem_write], effects.effects
  end

  def test_posix_backend_renders_practical_workflow
    output = @compiler.build(source)

    assert_includes output, "mkdir -p build"
    assert_includes output, "printf '%s' hello > build/notes.txt"
    assert_includes output, "printf '%s' \\ world >> build/notes.txt"
    assert_includes output, "cp build/notes.txt build/backup.txt"
    assert_includes output, "mv build/backup.txt build/archive.txt"
    assert_includes output, "touch build/complete.marker"
    assert_includes output, "rm -f build/old.tmp"
  end
end
