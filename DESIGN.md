# codegen-infra Design Doc

<!-- agent-updated: 2026-04-04T00:00:00Z -->

## Overview

Unified code generation infrastructure for Rust. A low-level library providing shared primitives for code generation across multiple schema formats.

## Goals

1. **Zero external dependencies** in core (except `thiserror` for derive)
2. **Feature-gated modules** for different schema formats
3. **Schema-agnostic IR** that adapters can convert to
4. **Composable codegen** - adapters produce IR, generators consume IR

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        codegen-infra                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │   Adapter    │    │   Adapter    │    │   Adapter    │     │
│  │ (FlatBuffers)│    │ (Protobuf)   │    │ (Quiver-ORM) │     │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘     │
│         │                   │                   │              │
│         ▼                   ▼                   ▼              │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                    IR (SchemaProvider)                 │  │
│  │  ServiceDef, MethodDef, MessageDef, FieldDef, Type      │  │
│  └─────────────────────────────────────────────────────────┘  │
│                           │                                    │
│                           ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Code Writers (Rust, Dart, TS)              │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Modules

### codegen-core

Shared primitives with **zero external dependencies** (except `thiserror`).

```
codegen-core/
├── src/
│   ├── lib.rs          # Public exports
│   ├── code_writer.rs  # Indentation-aware code generation
│   ├── error.rs        # CodeGenError enum
│   └── ir/             # Intermediate Representation types
│       ├── mod.rs
│       ├── service.rs  # ServiceDef, MethodDef
│       ├── message.rs  # MessageDef
│       ├── field.rs    # FieldDef
│       ├── type.rs     # ScalarType, Type
│       └── schema.rs   # SchemaProvider trait, EnumDef
```

### codegen-flatbuffers

FlatBuffers schema adapter - converts `ResolvedSchema` to IR.

### codegen-protobuf

Protobuf schema adapter - converts `FileDescriptorProto` to IR.

### codegen-quiver

Quiver-ORM schema adapter - converts Quiver model definitions to IR.

## IR Types

### ServiceDef

```rust
pub struct ServiceDef {
    pub name: String,           // e.g., "Greeter"
    pub package: String,        // e.g., "helloworld"
    pub proto_name: String,     // e.g., "Greeter"
    pub methods: Vec<MethodDef>,
    pub comments: Vec<String>,
}
```

### MethodDef

```rust
pub struct MethodDef {
    pub name: String,            // snake_case: "say_hello"
    pub proto_name: String,      // PascalCase: "SayHello"
    pub input_type: String,      // e.g., "crate::HelloRequest"
    pub output_type: String,      // e.g., "crate::HelloReply"
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub codec_path: String,      // e.g., "crate::codec::Codec"
    pub comments: Vec<String>,
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
    pub default_value: Option<String>,
    pub id: Option<u32>,         // FlatBuffers field ID
    pub comments: Vec<String>,
}
```

### Type

```rust
pub enum Type {
    Scalar(ScalarType),
    Message { name: String, package: Option<String> },
    Enum { name: String, package: Option<String> },
    Vector(Box<Type>),
    InlineStruct(Box<Type>),
}

pub enum ScalarType {
    Bool, Int8, Uint8, Int16, Uint16,
    Int32, Uint32, Int64, Uint64,
    Float32, Float64, String, Bytes,
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

## SchemaProvider Trait

```rust
pub trait SchemaProvider {
    fn messages(&self) -> Vec<MessageDef>;
    fn services(&self) -> Vec<ServiceDef>;
    fn enums(&self) -> Vec<EnumDef>;
    fn file_ident(&self) -> Option<&str>;
    fn root_table(&self) -> Option<&str>;
}
```

## CodeWriter

Indentation-aware code generation:

```rust
let mut writer = CodeWriter::new();
writer.block("struct MyService", |w| {
    w.line("fn new() -> Self {");
    w.indent();
    w.line("Self {}");
    w.dedent();
    w.line("}");
});
let code = writer.finish();
```

## Feature Flags

- `codegen-core`: `grpc` - enables gRPC-specific IR types
- `codegen-flatbuffers`: `grpc` - enables gRPC service support

## Dependencies

### codegen-core
- `thiserror` (dev/derive only)

### codegen-flatbuffers
- `codegen-core`
- `flatc-rs-schema` (git dependency)

## Design Principles

1. **Adapter Pattern**: Each schema format has its own adapter that produces IR
2. **Trait-Based**: `SchemaProvider` trait allows any schema format to plug in
3. **Owned Data**: `SchemaProvider` returns owned `Vec<T>` to avoid lifetime issues
4. **Zero-Copy IR**: IR types own their data (String, Vec) for simplicity
5. **Error Handling**: `CodeGenError` enum with `thiserror` derive

## TODO

- [ ] Add more scalar type mappings (bytes, etc.)
- [ ] Implement protobuf adapter
- [ ] Implement quiver-orm adapter
- [ ] Add Rust code writer
- [ ] Add Dart code writer
- [ ] Add TypeScript code writer
- [ ] Add gRPC-specific IR types behind feature flag
