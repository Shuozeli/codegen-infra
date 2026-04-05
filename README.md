# Unified Code Generation Infrastructure

[![CI](https://github.com/Shuozeli/codegen-infra/actions/workflows/ci.yml/badge.svg)](https://github.com/Shuozeli/codegen-infra/actions/workflows/ci.yml)

<!-- agent-updated: 2026-04-05T04:40:00Z -->

## Overview

A low-level Rust library for unified code generation across multiple schema formats (FlatBuffers, Protobuf, Quiver-ORM).

## Motivation

Current projects have separate codegen implementations:

| Project | Schema Format | Codegen Targets |
|---------|---------------|-----------------|
| flatbuffers-rs | FlatBuffers | Rust, TypeScript, Dart |
| protobuf-rs | Protobuf | Rust |
| quiver-orm | SQL Schema | Rust, SQL, TypeScript |

Each has duplicated:
- Code writer utilities
- Error handling (`CodeGenError`)
- Service/IR types
- Name casing utilities

## Goals

1. **Shared primitives**: `CodeWriter`, `CodeGenError`, casing helpers
2. **Schema trait**: Unified interface for schema access
3. **IR types**: Common `ServiceDef`, `MessageDef`, `FieldDef`
4. **Generator trait**: `Generator<Schema, Output>` interface
5. **Feature-gated modules**: Only compile what you need

## Architecture

```
codegen-core/          # Shared primitives (no external deps)
├── code_writer.rs     # Indented code generation
├── error.rs           # CodeGenError
├── ir/                # Intermediate Representation
│   ├── mod.rs
│   ├── service.rs     # ServiceDef, MethodDef
│   ├── message.rs     # MessageDef, FieldDef, Type
│   └── schema.rs      # SchemaProvider trait
└── case.rs           # SnakeCase, UpperCamel, etc.

codegen-flatbuffers/   # FlatBuffers schema adapter
codegen-protobuf/      # Protobuf schema adapter
codegen-quiver/        # Quiver-ORM schema adapter
```

## Core Traits

### SchemaProvider

```rust
pub trait SchemaProvider {
    fn messages(&self) -> Vec<&dyn MessageDef>;
    fn services(&self) -> Vec<&dyn ServiceDef>;
    fn enums(&self) -> Vec<&dyn EnumDef>;
    fn has_namespace(&self) -> bool;
}
```

### Generator

```rust
pub trait Generator<S: SchemaProvider> {
    fn generate(&self, schema: &S) -> Result<String, CodeGenError>;
}
```

## Key Design Decisions

### 1. No External Dependencies in Core

`codegen-core` has zero external dependencies. It only uses:
- `std`
- `thiserror` (for derive macro only)

### 2. Error Aggregation

Collect all errors before failing for better IDE support:
```rust
pub struct CodeGenResult<T> {
    pub errors: Vec<CodeGenError>,
    pub value: Option<T>,
}
```

### 3. Incremental Generation

Generator can be called multiple times, appending to the same `CodeWriter`:
```rust
pub trait IncrementalGenerator<S: SchemaProvider> {
    fn generate_incremental(&self, schema: &S, writer: &mut CodeWriter) -> Result<(), CodeGenError>;
}
```

### 4. Feature Flags

| Feature | Enables |
|---------|---------|
| `flatbuffers` | FlatBuffers schema adapter |
| `protobuf` | Protobuf schema adapter |
| `quiver` | Quiver-ORM schema adapter |
| `grpc` | gRPC service generation |
| `dart` | Dart code generation |
| `typescript` | TypeScript code generation |

## Files to Create

```
codegen-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs
    ├── code_writer.rs
    ├── case.rs
    └── ir/
        ├── mod.rs
        ├── service.rs
        ├── message.rs
        ├── field.rs
        ├── type.rs
        └── schema.rs
```

## Migration Plan

1. Extract `CodeWriter` from flatbuffers-rs → codegen-core
2. Extract `CodeGenError` from flatbuffers-rs → codegen-core
3. Extract IR types (ServiceDef, MethodDef) from pure-grpc-rs → codegen-core
4. Create schema traits for FlatBuffers and Protobuf
5. Wire up to existing generators

## Open Questions

- [ ] Should we use `syn` for code generation or raw strings?
- [ ] How to handle different naming conventions across schemas?
- [ ] Should we generate directly to files or to `CodeWriter` buffer?