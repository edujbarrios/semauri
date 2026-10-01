# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require_relative "test_helper"

class NominalTypesTest < Minitest::Test
  def test_nominal_types_use_domain_and_name_identity
    path = Semauri::Semantics::NominalType.new(domain: :filesystem, name: :path, base_type: :string)
    url = Semauri::Semantics::NominalType.new(domain: :http, name: :url, base_type: :string)

    assert_equal "filesystem.path", path.to_s
    refute_equal path, url
    assert_equal path, Semauri::Semantics::NominalType.new(domain: :filesystem, name: :path, base_type: :string)
  end

  def test_primitive_base_type_can_be_promoted_when_nominal_type_allows_it
    path = Semauri::Domains::Filesystem::PATH

    assert path.promote_from_base?
    assert_equal :promote, Semauri::Semantics::TypeSystem.assignment_kind(:string, path)
    assert Semauri::Semantics::TypeSystem.assignable?(:string, path)
    assert_equal :exact, Semauri::Semantics::TypeSystem.assignment_kind(path, path)
  end

  def test_opaque_nominal_type_rejects_base_promotion
    config = Semauri::Semantics::NominalType.new(
      domain: :ml,
      name: :training_config,
      base_type: :opaque,
      promote_from_base: false
    )

    refute config.promote_from_base?
    refute Semauri::Semantics::TypeSystem.assignable?(:opaque, config)
    assert_equal :exact, Semauri::Semantics::TypeSystem.assignment_kind(config, config)
  end

  def test_one_nominal_type_cannot_be_reinterpreted_as_another_with_same_base
    path = Semauri::Domains::Filesystem::PATH
    url = Semauri::Semantics::NominalType.new(domain: :http, name: :url, base_type: :string)

    refute Semauri::Semantics::TypeSystem.assignable?(url, path)
    refute Semauri::Semantics::TypeSystem.assignable?(path, url)
  end

  def test_filesystem_domain_exposes_nominal_type_metadata
    domain = Semauri::Domains::Filesystem.new
    path = domain.type(:path)

    assert_equal Semauri::Domains::Filesystem::PATH, path
    assert_equal :filesystem, path.domain
    assert_equal :path, path.name
    assert_equal :string, path.base_type
    assert path.promote_from_base?

    metadata = domain.to_h
    assert_includes metadata.fetch(:types), path.to_h
  end

  def test_operation_signature_serializes_nominal_path_type
    domain = Semauri::Domains::Filesystem.new
    write = domain.operation(:write)
    path_slot = write.slots.find { |slot| slot.name == :path }

    assert_equal Semauri::Domains::Filesystem::PATH, path_slot.type
    serialized_slot = write.to_h.fetch(:pattern).find { |segment| segment[:slot] == :path }
    assert_equal Semauri::Domains::Filesystem::PATH.to_h, serialized_slot.fetch(:type)
  end
end
