# Universal semantic domains

Semauri's long-term goal is not to hard-code every programming domain into the language. The core should provide deterministic syntax, types, control flow, symbols and diagnostics; domains should provide application-specific meaning.

The 0.5 domain API proves that nouns can be extended without changing the lexer/parser, but `artifact -> element -> property` is not general enough for domains such as filesystem automation, HTTP, databases, ML pipelines, cloud infrastructure or robotics.

## Target architecture

```text
Semauri Core
├── syntax + deterministic parsing
├── primitive / parametric types
├── symbols + lexical scope
├── control flow
├── typed HIR
├── optimization passes
├── effects / capabilities
└── domain registry
     ├── vocabulary
     ├── operations
     ├── domain types
     ├── lowering hooks
     └── backend recommendations
```

A domain should be able to contribute **operations**, not only artifacts/elements/properties.

## Declarative domain operations

An operation is described by:

- one or more surface verbs;
- a stable semantic operation name;
- a deterministic phrase pattern;
- typed argument slots;
- a result type;
- an effect/capability set;
- a domain lowering hook.

Conceptually:

```ruby
operation :write,
  verbs: ["write"],
  pattern: [
    expression(:content, type: :string),
    literal("to"),
    expression(:path, type: :string)
  ],
  returns: :unit,
  effects: [:filesystem_write]
```

Source:

```text
Write "hello" to "notes.txt".
```

Typed HIR:

```text
domain_operation filesystem.write : unit
├── content : string = "hello"
├── path    : string = "notes.txt"
└── effects = [filesystem_write]
```

The parser does not contain a `write` special case. It sees a registered domain action and executes the declarative pattern.

## Why effects are part of HIR

Operations such as file writes, HTTP requests or database mutations are observable. Optimizers must not delete or reorder them as if they were pure arithmetic.

Examples of future effects:

```text
filesystem_read
filesystem_write
network
process
clock
random
storage_read
storage_write
model_inference
```

Effect metadata is compiler information, not permission to execute the effect during compilation.

## Compile, do not secretly execute

Domain operations should normally lower into semantic plans/IR. A backend can render those plans to a target such as POSIX shell, SQL, infrastructure configuration, Python or a future Semauri runtime.

For example, a filesystem domain can compile:

```text
Write "hello" to "notes.txt".
Copy "notes.txt" to "backup.txt".
```

into a filesystem plan and later a shell backend. The compiler itself must not write or delete user files merely because it compiled the source.

## Beyond one active domain

Single-domain programs are only an intermediate milestone. General programming requires values and operations to cross domain boundaries:

```text
HTTP -> structured data -> filesystem
Database -> analytics -> web
Filesystem -> model inference -> structured data
```

The future program-level representation should therefore own multiple domain plans rather than one active artifact:

```text
ProgramIR
├── operations
├── values
├── effects
└── domain-owned nodes
```

Domain identity remains explicit on every operation so ambiguity is never solved probabilistically.

## Domain types

Core types (`number`, `string`, `boolean`, `color`, `List<T>`) are not enough for all domains. The extension model should eventually support nominal domain types such as:

```text
filesystem.path
http.url
http.response
sql.query
sql.rowset
ml.model
ml.tensor
cloud.resource
```

Conversions between domains must be explicit or registered as typed conversions.

## Namespace and collision strategy

Global vocabulary collision rejection is safe but will become restrictive when many domains are loaded. Long term, Semauri should support domain qualification / lexical domain scopes so common verbs such as `get`, `run`, `delete` and `create` can coexist deterministically.

Possible surface forms include:

```text
Within filesystem:
  Delete "tmp.log".
End.
```

or explicit qualification when ambiguity exists. Unqualified syntax should only be accepted when resolution is unique.

## Runtime boundary

Today Semauri evaluates only compile-time-known values. Operations that produce runtime values require a later runtime/CFG layer:

```text
Typed HIR
  -> optimization
  -> control-flow IR / runtime operations
  -> backend or Semauri VM
```

That layer is required for programs such as:

```text
Let response be Fetch "https://example.com/data".
If response is successful:
  Write response to "data.json".
End.
```

The current operation API should be designed so adding runtime results later does not require changing surface parsing again.

## Extension rule

A domain is truly external only if adding it does **not** require editing:

- the lexer;
- the core parser's list of domain verbs;
- AST/HIR code for that specific domain;
- unrelated domains;
- existing backends.

New generic extension primitives may change the core. New concrete domains should only register against those primitives.
