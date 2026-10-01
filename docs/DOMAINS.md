# Semantic domains

Semantic domains are Semauri's extension boundary for application-specific meaning.

A domain may contribute vocabulary, property type contracts, declarative operations, effect metadata, semantic IR construction, mutation rules, explanations and a default backend. The lexer and parser consume generic domain categories and must not be modified just to add new nouns or verbs.

## Contract

A domain subclasses `Semauri::Domains::Definition` and may declare artifact-oriented vocabulary:

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

Artifact/element/property is a convenience model, not a requirement. Operation-oriented domains can expose actions instead.

## Declarative operations

A domain operation contributes a verb and a deterministic phrase pattern. Domain verbs become `DOMAIN_ACTION` tokens; they do not become core Semauri keywords.

```ruby
write = Semauri::Domains::Operation.new(
  name: :write,
  verbs: ["write"],
  pattern: [
    Semauri::Domains::Operation.expression(:content, type: :string),
    Semauri::Domains::Operation.literal("to"),
    Semauri::Domains::Operation.expression(:path, type: :string)
  ],
  returns: :unit,
  effects: [:filesystem_write]
)

class ExampleDomain < Semauri::Domains::Definition
  def initialize
    super(name: :example, operations: [write])
  end
end
```

The generic parser can then parse:

```text
Write "hello" to "notes.txt".
```

without a `write` branch in `Parser`.

Typed HIR preserves the operation contract:

```text
domain_operation example.write : unit
├── content : string
├── path    : string
└── effects = [filesystem_write]
```

Expression slot types are validated while building HIR. A mismatch is `S329`.

### Operation lowering

Operation domains implement:

```ruby
def execute_operation(operation:, artifact:, arguments:, source_span:)
  ...
  Semauri::Domains::OperationResult.new(
    artifact: updated_plan,
    explanations: ["..."]
  )
end
```

An operation-only domain may also implement `initial_artifact` to lazily create a semantic plan when its first operation is encountered. This allows source programs that do not need a synthetic `Create ...` statement.

The compiler records operation effects but does not perform those effects merely because source code is compiled.

## Effects

Effects are semantic metadata used to distinguish observable operations from pure expressions. Current domains may declare arbitrary stable effect identifiers such as:

```text
filesystem_read
filesystem_write
network
storage_read
storage_write
model_inference
```

Effects are preserved in Typed HIR. Optimizers may rewrite operation arguments, but must not silently remove or reorder observable operations as though they were pure arithmetic.

A future capability/runtime layer will use this information for execution policy and effect validation.

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

Artifact-oriented domains may implement:

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

Operation-oriented domains may reuse `IR::OperationPlan` / `IR::Operation` or provide a more specialized immutable IR.

Domain IR should be immutable where practical. Source provenance should be retained when a mutation or operation originates from source code.

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
- a domain operation cannot be applied while a different domain is active (`S327`).

These rules intentionally prefer errors over contextual guessing. Global term uniqueness is deliberately conservative; lexical domain scopes/qualification are planned for composing many domains with common verbs.

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

### Filesystem

- operations: `write`, `copy`, `delete` / `remove`
- typed string arguments for content and paths
- effects: `filesystem_read`, `filesystem_write`
- domain IR: generic `IR::OperationPlan`
- default backend: `posix-sh`
- compilation generates a script; it never mutates the compiler host filesystem

## Contributor checklist

A domain PR should include:

1. a domain definition with non-conflicting vocabulary;
2. declarative operation signatures when procedural behavior is needed;
3. typed arguments/properties and explicit effect metadata;
4. immutable domain IR or an explicit rationale otherwise;
5. domain-specific semantic validation where needed;
6. at least one backend or an explicit reason why none is supplied;
7. parser-free/lexer-free extension tests proving concrete domain words/verbs do not require core cases;
8. negative tests for invalid types/values, malformed patterns and ambiguity;
9. language/architecture documentation updates;
10. no direct target-markup or external-effect execution in parser, AST or HIR.

Run `semauri domains` to inspect the vocabulary and operations exported by the active compiler configuration.

See [`UNIVERSAL_DOMAINS.md`](UNIVERSAL_DOMAINS.md) for the longer-term multi-domain architecture.
