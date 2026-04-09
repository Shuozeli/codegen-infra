# Unified Code Generation Infrastructure

[![CI](https://github.com/Shuozeli/codegen-infra/actions/workflows/ci.yml/badge.svg)](https://github.com/Shuozeli/codegen-infra/actions/workflows/ci.yml)

<!-- agent-updated: 2026-04-07T23:30:00Z -->

## Overview

A low-level Rust framework for unified code generation across multiple schema formats.

**codegen-infra is a pure framework** — it provides the core primitives and traits that schema repositories use to build their own code generators. It does NOT contain schema-specific adapters; those live in their respective repositories.

## Motivation

Current projects have separate codegen implementations:

| Project | Schema Format | Codegen Targets |
|---------|---------------|----------------|
| [flatbuffers-rs](https://github.com/Shuozeli/flatbuffers-rs) | FlatBuffers | Rust, TypeScript, Dart |
| [protobuf-rs](https://github.com/Shuozeli/protobuf-rs) | Protobuf | Rust |
| [quiver-orm](https://github.com/Shuozeli/quiver-orm) | SQL Schema | Rust, SQL, TypeScript |

Each had duplicated:
- Code writer utilities
- Error handling (`CodeGenError`)
- Service/IR types
- Name casing utilities

## Architecture

```
codegen-infra (this repo)
├── codegen-core/       # Shared primitives (zero external deps)
│   ├── code_writer.rs  # Indentation-aware string building
│   ├── error.rs       # CodeGenError
│   └── reserved_words.rs
│
├── codegen-schema/     # Format-agnostic schema types
│   ├── schema.rs       # SchemaDef, SchemaProvider trait
│   ├── type.rs         # Type, ScalarType, etc.
│   └── validate.rs     # Schema validation
│
└── codegen-writers/    # Language-specific code writers
    ├── rust.rs         # RustCodeWriter
    ├── typescript.rs   # TypeScriptCodeWriter
    └── dart.rs         # DartCodeWriter
```

## Schema Adapters (in their respective repos)

Each schema format provides an adapter that implements `SchemaProvider`:

| Repository | Adapter Crate | Description |
|------------|---------------|-------------|
| [flatbuffers-rs](https://github.com/Shuozeli/flatbuffers-rs) | `codegen-flatbuffers` | Converts FlatBuffers `ResolvedSchema` → `SchemaDef` |
| [protobuf-rs](https://github.com/Shuozeli/protobuf-rs) | `codegen-protobuf` | Converts Protobuf `FileDescriptorProto` → `SchemaDef` |
| [quiver-orm](https://github.com/Shuozeli/quiver-orm) | `codegen-quiver` | Converts Quiver schema → `SchemaDef` |

## Core Traits

### SchemaProvider

```rust
pub trait SchemaProvider {
    fn messages(&self) -> Vec<MessageDef>;
    fn services(&self) -> Vec<ServiceDef>;
    fn enums(&self) -> Vec<EnumDef>;
    fn file_ident(&self) -> Option<&str>;
    fn root_table(&self) -> Option<&str>;
}
```

### CodeWriter

```rust
pub struct CodeWriter {
    // Indentation-aware string building
}

impl CodeWriter {
    pub fn new(indentation: Indent) -> Self;
    pub fn write_line(&mut self, line: &str);
    pub fn write_block(&mut self, block: impl FnOnce(&mut Self));
}
```

## Key Design Decisions

### 1. Zero External Dependencies in Core

`codegen-core` has zero external dependencies:
- `std`
- `thiserror` (for derive macro only in codegen-schema and codegen-writers)

### 2. Schema-Agnostic

The framework knows nothing about FlatBuffers, Protobuf, or any other schema format. It only deals with the generic `SchemaDef` types.

### 3. Separation of Concerns

- **codegen-core**: Pure primitives, no schema knowledge
- **codegen-schema**: Generic schema types and traits
- **codegen-writers**: Language-specific output formatting
- **Adapters**: Schema-specific conversion logic (in their respective repos)

## Usage

```rust
use codegen_core::CodeWriter;
use codegen_schema::{SchemaDef, SchemaProvider};
use codegen_writers::RustCodeWriter;

// Schema-specific adapter (e.g., from flatbuffers-rs)
use codegen_flatbuffers::from_resolved_schema;

// Convert your schema to the generic representation
let schema_def: SchemaDef = from_resolved_schema(&resolved)?;

// Generate code using the generic schema
let mut writer = CodeWriter::new(Indent::FourSpaces);
let mut code_writer = RustCodeWriter::new(&mut writer);
let output = code_writer.write_file(
    &schema_def.messages,
    &schema_def.enums,
    &schema_def.services,
)?;
```

## Status

- [x] Core primitives (CodeWriter, CodeGenError, reserved words)
- [x] Format-agnostic schema types (SchemaDef, MessageDef, etc.)
- [x] Language code writers (Rust, TypeScript, Dart)
- [x] FlatBuffers adapter moved to [flatbuffers-rs](https://github.com/Shuozeli/flatbuffers-rs)
- [ ] Protobuf adapter to move to protobuf-rs
- [ ] Quiver adapter to move to quiver-orm
