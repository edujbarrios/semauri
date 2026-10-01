# Semantic domains

Semantic domains are Semauri's extension boundary for application-specific meaning.

A domain may contribute vocabulary, property type contracts, semantic IR construction, mutation rules, explanations and a default backend. The lexer and parser consume generic domain categories and must not be modified just to add new nouns.

## Contract

A domain subclasses `Semauri::Domains::Definition` and declares surface vocabulary:

```ruby
class ExampleDomain < Semauri::Domains::Definition
  def initialize
    super(
      name: :example,
      default_backend: "example",
      artifacts: { "board" => :board },
      elements: { "item" => :item },
      properties: { "enabled" => :enabled }
    )
  end
end
```

The declaration produces generic lexer categories:

```text
board   → DOMAIN_ARTIFACT(domain=example, kind=board)
item    → DOMAIN_ELEMENT(domain=example, kind=item)
enabled → DOMAIN_PROPERTY(domain=example, kind=enabled)
```

## Property typing

Domains define the semantic type accepted by each property:

```ruby
def property_type(property)
  :boolean if property.to_sym == :enabled
end
```

The Typed HIR builder validates this contract before lowering. A mismatched value is `S313`.

Domains may additionally validate values during lowering when type information alone is insufficient. The built-in Structured Data domain does this for JSON Schema datatype names.

## Domain IR

A domain owns the semantic structures produced by its operations. It does not have to reuse Web IR.

Required artifact hook:

```ruby
def create_artifact(kind:, subject:, title:)
  MyArtifact.new(...)
end
```

Element domains implement:

```ruby
def add_element(artifact:, kind:, label:, entities:)
  entity = entities.register(kind: kind) { |id| MyEntity.new(id: id, ...) }
  [artifact.add(entity), entity]
end
```

If the domain supports properties, either implement compatible immutable methods (`with_property`, `replace_element`) and use the default hook, or override:

```ruby
def set_property(artifact:, target:, property:, value:, source_span:)
  ...
end
```

Domain IR should be immutable where practical. Source provenance should be retained when a mutation originates from source code.

## Default backend

A domain may declare a default backend:

```ruby
def initialize
  super(name: :example, default_backend: "example", ...)
end
```

`Compiler#compile` and `semauri build` infer this backend unless the caller explicitly supplies `backend:` / `--backend`.

A domain without a default backend can still be used, but compilation must explicitly select one. Otherwise Semauri reports `S404`.

## Registration

Create a registry and inject it into the compiler:

```ruby
registry = Semauri::Domains::Registry.new
  .register(MyDomain.new)

compiler = Semauri::Compiler.new(domains: registry)
```

One registry instance is propagated through vocabulary classification, parsing, Typed HIR construction and HIR lowering.

Registration is transactional. A failed registration does not leave partially registered words behind.

## Vocabulary rules

Within one compiler instance:

- a surface term may belong to only one semantic domain;
- domains may not override core English grammar words such as `if`, `let`, `true`, or primitive color literals;
- domain operations carry explicit domain identity through AST and HIR;
- a domain operation cannot be applied to an artifact from another domain (`S327`).

These rules intentionally prefer errors over contextual guessing.

## Implicit properties

Natural shorthand such as:

```text
Make it blue.
```

has no explicit property word. The registry accepts shorthand only when exactly one loaded domain property is compatible with the value type.

If multiple properties could accept the value, parsing fails with `S236` and the program must use explicit `Set ...` syntax.

## Backend contract

Backends are separate from semantic domains. A domain produces semantic IR; a backend renders that IR.

```ruby
class MyBackend < Semauri::Backends::Base
  def render(program)
    ...
  end
end
```

Register it independently:

```ruby
backends = Semauri::Backends::Registry.new
  .register("example") { MyBackend.new }
```

This separation allows one domain to support multiple target formats without changing language semantics.

## Built-in domains

### Web

- artifact: `web` (`website`, `webpage`, `page`)
- elements: `button`, `image` (`picture`)
- property: `color : color`
- domain IR: `IR::WebDocument`, `IR::Element`
- default backend: `html`

### Structured Data

- artifact: `schema`
- element: `field`
- properties: `datatype : string`, `required : boolean`
- domain IR: `IR::SchemaDocument`, `IR::SchemaField`
- default backend: `json-schema`

## Contributor checklist

A domain PR should include:

1. a domain definition with non-conflicting vocabulary;
2. immutable domain IR or an explicit rationale otherwise;
3. property type contracts;
4. domain-specific semantic validation where needed;
5. at least one backend or an explicit reason why none is supplied;
6. parser-free/lexer-free extension tests;
7. negative tests for invalid types/values and ambiguity;
8. language/architecture documentation updates;
9. no direct target-markup logic in parser, AST or HIR.

Run `semauri domains` to inspect the vocabulary exported by the active compiler configuration.
