# codegen-infra Design Doc

<!-- agent-updated: 2026-04-05T05:15:00Z -->

## Overview

Unified code generation framework providing **M schema formats × N target languages** via a shared Intermediate Representation (IR).

**Current status:** All adapters and code writers implemented. 55 tests passing.

## Architecture

```
                         Schema Formats
        ┌─────────────┬─────────────┬─────────────┐
        │ FlatBuffers │  Protobuf  │  Quiver-ORM │
        │   (.fbs)    │  (.proto)  │  (.quiver)  │
        └──────┬──────┴──────┬─────┴──────┬──────┘
               │             │            │
               ▼             ▼            ▼
        ┌─────────────────────────────────────────┐
        │              Adapters (M)               │
        │  FlatBuffersSchema  ProtobufSchema  ...  │
        └────────────────────┬────────────────────┘
                             │
                             ▼
        ┌─────────────────────────────────────────┐
        │            codegen-core                 │
        │                                         │
        │  ┌─────────────────────────────────┐   │
        │  │   Intermediate Representation    │   │
        │  │   ServiceDef, MessageDef,       │   │
        │  │   FieldDef, EnumDef, Type       │   │
        │  └─────────────────────────────────┘   │
        │                                         │
        │  ┌─────────────────────────────────┐   │
        │  │   SchemaProvider trait           │   │
        │  └─────────────────────────────────┘   │
        │                                         │
        │  ┌─────────────────────────────────┐   │
        │  │   CodeWriter trait              │   │
        │  └─────────────────────────────────┘   │
        └────────────────────┬────────────────────┘
                             │
                             ▼
        ┌─────────────────────────────────────────┐
        │           Code Writers (N)               │
        │   RustCodeWriter  DartCodeWriter  TS    │
        └─────────────────────────────────────────┘
```

## Modules

```
codegen-infra/
├── codegen-core/          # Shared IR + traits (zero external deps)
│   └── src/
│       ├── lib.rs
│       ├── code_writer.rs # Indentation-aware code generation
│       ├── error.rs       # CodeWriterError
│       └── ir/           # Intermediate Representation
│           ├── mod.rs     # Re-exports
│           ├── type.rs    # ScalarType, Type
│           ├── field.rs   # FieldDef
│           ├── message.rs # MessageDef
│           ├── service.rs # ServiceDef, MethodDef
│           └── schema.rs # SchemaProvider trait, EnumDef
│
├── codegen-flatbuffers/   # Adapter: FlatBuffers → IR
├── codegen-protobuf/      # Adapter: Protobuf → IR
├── codegen-quiver/        # Adapter: Quiver-ORM → IR
│
└── codegen-writers/       # Code generators: IR → target language
    └── src/
        ├── lib.rs         # to_snake_case, to_pascal_case, to_camel_case
        ├── rust.rs        # RustCodeWriter
        ├── dart.rs        # DartCodeWriter
        └── typescript.rs  # TypeScriptCodeWriter
```

## Design Decisions

### 1. Raw Strings over syn

**Decision:** Use raw strings via `CodeWriter` instead of `syn` AST manipulation.

**Rationale:**
- Simpler mental model - no AST complexity
- No external dependencies in codegen-core
- Sufficient for generating clean, formatted code
- `syn` can be added later if needed for complex Rust-specific generation

**CodeWriter API:**
```rust
let mut w = CodeWriter::new();
w.line("// Generated code");
w.block("struct Foo {", |w| {
    w.line("field: i32,");
});
```

### 2. Naming Convention Normalization

**Decision:** Adapters normalize schema-specific names to IR strings. Code writers handle target-language casing.

**Approach:**
- IR stores original names as-is
- `to_snake_case()`, `to_pascal_case()`, `to_camel_case()` helpers in code writers
- Each code writer applies appropriate casing for target language

**Example mappings:**

| Schema | Type Name | IR | Rust | Dart | TypeScript |
|--------|-----------|-----|------|------|------------|
| FlatBuffers | MyTable | "MyTable" | `struct MyTable` | `class MyTable` | `interface MyTable` |
| Protobuf | MyMessage | "MyMessage" | `struct MyMessage` | `class MyMessage` | `interface MyMessage` |
| Quiver | my_table | "my_table" | `struct MyTable` | `class MyTable` | `interface MyTable` |

### 3. Buffer over File Output

**Decision:** `CodeWriter` generates to `String` buffer. Caller handles filesystem.

**Rationale:**
- Separation of concerns - code generation is independent of I/O
- Easier to test - just check the string content
- Enables composable workflows (generate → transform → write)

**Output pattern:**
```rust
let code = writer.write_file(&messages, &enums, &services)?;
std::fs::write("output.rs", code)?;
```

## Intermediate Representation (IR)

### SchemaProvider Trait

```rust
pub trait SchemaProvider {
    fn messages(&self) -> Vec<MessageDef>;
    fn services(&self) -> Vec<ServiceDef>;
    fn enums(&self) -> Vec<EnumDef>;
    fn file_ident(&self) -> Option<&str>;
    fn root_table(&self) -> Option<&str>;
}
```

### Type

```rust
pub enum ScalarType {
    Bool, Int8, Uint8, Int16, Uint16,
    Int32, Uint32, Int64, Uint64,
    Float32, Float64, String, Bytes,
}

pub enum Type {
    Scalar(ScalarType),
    Message { name: String, package: Option<String> },
    Enum { name: String, package: Option<String> },
    Vector(Box<Type>),
    InlineStruct(Box<Type>),
}
```

### MessageDef

```rust
pub struct MessageDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub is_struct: bool,        // true = struct, false = table
    pub namespace: Option<String>,
    pub comments: Vec<String>,
}
```

### FieldDef

```rust
pub struct FieldDef {
    pub name: String,
    pub ty: Type,
    pub is_optional: bool,
    pub comments: Vec<String>,
}
```

### ServiceDef

```rust
pub struct ServiceDef {
    pub name: String,
    pub methods: Vec<MethodDef>,
    pub package: Option<String>,
    pub comments: Vec<String>,
}

pub struct MethodDef {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
}
```

### EnumDef

```rust
pub struct EnumDef {
    pub name: String,
    pub values: Vec<EnumValue>,
    pub is_union: bool,
    pub is_struct: bool,
    pub namespace: Option<String>,
    pub comments: Vec<String>,
}

pub struct EnumValue {
    pub name: String,
    pub value: i64,
    pub comments: Vec<String>,
}
```

## CodeWriter Trait

```rust
pub trait CodeWriter {
    fn write_message(&mut self, msg: &MessageDef) -> Result<String, CodeWriterError>;
    fn write_enum(&mut self, enm: &EnumDef) -> Result<String, CodeWriterError>;
    fn write_service(&mut self, svc: &ServiceDef) -> Result<String, CodeWriterError>;
    fn write_file(&mut self, messages: &[MessageDef], enums: &[EnumDef], services: &[ServiceDef]) -> Result<String, CodeWriterError>;
}
```

## Feature Flags

| Flag | Enables | Dependencies |
|------|---------|--------------|
| (default) | Core IR, no adapters | `thiserror` |
| `flatbuffers` | FlatBuffers adapter | `codegen-core`, `flatc-rs-schema` |
| `protobuf` | Protobuf adapter | `codegen-core`, `prost-types` |
| `quiver` | Quiver-ORM adapter | `codegen-core` |
| `writers` | All code writers (Rust, Dart, TS) | `codegen-core` |

## Consumer Projects

Projects using codegen-infra:

| Project | Status | Adapters Used |
|---------|--------|---------------|
| flatbuffers-rs | Using | flatbuffers |
| protobuf-rs | Using | protobuf |
| quiver-orm | Using | quiver |

## Dependencies

### codegen-core
- `thiserror` (dev/derive only)

### codegen-flatbuffers
- `codegen-core`
- `flatc-rs-schema` (git dependency)

### codegen-protobuf
- `codegen-core`
- `prost-types`

### codegen-quiver
- `codegen-core`

## TODO

- [x] SchemaProvider trait
- [x] IR types (ServiceDef, MethodDef, MessageDef, FieldDef, EnumDef, Type)
- [x] CodeWriter trait
- [x] FlatBuffers adapter
- [x] Protobuf adapter
- [x] Quiver-ORM adapter
- [x] Rust code writer
- [x] Dart code writer
- [x] TypeScript code writer
- [x] Integration tests
- [ ] Go code writer
- [ ] Python code writer
- [ ] Streaming RPC support in IR
- [ ] One-of/unions handling across languages
