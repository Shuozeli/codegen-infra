# codegen-infra

Unified code generation infrastructure for Rust.

## Overview

A **pure framework** for code generation. This repo provides the core primitives and traits; schema-specific adapters live in their respective repositories.

## Structure

```
codegen-infra/
├── codegen-core/      # Shared primitives (zero external deps)
├── codegen-schema/    # Format-agnostic schema types
└── codegen-writers/    # Language code writers
```

**Schema adapters are in their respective repos:**
- `codegen-flatbuffers` → [flatbuffers-rs](https://github.com/Shuozeli/flatbuffers-rs)
- `codegen-protobuf` → [protobuf-rs](https://github.com/Shuozeli/protobuf-rs)
- `codegen-quiver` → [quiver-orm](https://github.com/Shuozeli/quiver-orm)

## Quick Start

```rust
use codegen_core::CodeWriter;
use codegen_schema::{SchemaDef, SchemaProvider};
use codegen_writers::RustCodeWriter;

// Schema-specific adapter (from flatbuffers-rs, protobuf-rs, etc.)
use codegen_flatbuffers::from_resolved_schema;

// Convert schema to generic representation
let schema_def: SchemaDef = from_resolved_schema(&resolved)?;

// Generate code
let mut writer = CodeWriter::new(Indent::FourSpaces);
let mut code_writer = RustCodeWriter::new(&mut writer);
let output = code_writer.write_file(&schema_def.messages, &schema_def.enums, &schema_def.services)?;
```

## Design Principles

1. **Zero external dependencies** in core (except `thiserror` for derive)
2. **Schema-agnostic IR** that adapters can convert to
3. **Adapter Pattern**: Each schema format has its own adapter in its repo
4. **Language writers**: Language-specific output in codegen-writers

## Core Types

- `SchemaDef` - complete schema with messages, enums, services
- `MessageDef` - message/struct/table definition
- `FieldDef` - field within a message
- `EnumDef` - enum/union definition
- `ServiceDef` - gRPC service definition
- `MethodDef` - single RPC method
- `Type` - scalar, message, enum, vector, optional, oneof types

## Building

```bash
cargo build
cargo test
cargo fmt
cargo clippy
```

## Design

See [DESIGN.md](DESIGN.md) for detailed architecture documentation.
