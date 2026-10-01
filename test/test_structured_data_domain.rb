# Copyright 2026 Eduardo J. Barrios
# SPDX-License-Identifier: Apache-2.0

require "json"
require_relative "test_helper"

class StructuredDataDomainTest < Minitest::Test
  def setup
    @compiler = Semauri::Compiler.new
  end

  def test_compiles_schema_to_json_schema
    source = <<~SEMA
      Create a schema called Pet.
      Add a field called Name.
      Set the datatype of the field called Name to "string".
      Set the required of the field called Name to true.
      Add a field called Age.
      Set the datatype of the field called Age to "integer".
    SEMA

    result = @compiler.compile(source, backend: "json-schema")
    json = JSON.parse(result.output)

    assert_equal "https://json-schema.org/draft/2020-12/schema", json.fetch("$schema")
    assert_equal "Pet", json.fetch("title")
    assert_equal "object", json.fetch("type")
    assert_equal({ "type" => "string" }, json.fetch("properties").fetch("Name"))
    assert_equal({ "type" => "integer" }, json.fetch("properties").fetch("Age"))
    assert_equal ["Name"], json.fetch("required")
  end

  def test_structured_data_uses_distinct_domain_ir
    _ast, semantic = @compiler.analyze("Create a schema called Pet. Add a field called Name.")

    assert_instance_of Semauri::IR::SchemaDocument, semantic.program
    assert_instance_of Semauri::IR::SchemaField, semantic.program.fields.first
    refute_instance_of Semauri::IR::WebDocument, semantic.program
  end

  def test_property_types_are_checked_by_domain_contract
    source = <<~SEMA
      Create a schema called Pet.
      Add a field called Name.
      Set the required of the field called Name to "yes".
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.hir(source) }
    assert_equal "S313", error.code
  end

  def test_invalid_schema_datatype_is_rejected_during_domain_lowering
    source = <<~SEMA
      Create a schema called Pet.
      Add a field called Name.
      Set the datatype of the field called Name to "uuid".
    SEMA

    error = assert_raises(Semauri::SemanticError) { @compiler.analyze(source) }
    assert_equal "S328", error.code
    assert_includes error.hint, "string"
  end

  def test_json_schema_backend_rejects_web_ir
    error = assert_raises(Semauri::BackendError) do
      @compiler.compile("Create a web called Shop.", backend: "json-schema")
    end

    assert_equal "S401", error.code
  end

  def test_default_domain_registry_exposes_web_and_structured_data
    assert_equal %i[structured_data web], @compiler.domains.names
  end
end
