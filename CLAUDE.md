# codegen-infra

Unified code generation infrastructure for Rust.

## Overview

A low-level Rust library providing shared primitives for code generation across multiple schema formats.

## Structure

```
codegen-infra/
├── codegen-core/        # Shared primitives (no external deps except thiserror)
├── codegen-flatbuffers/ # FlatBuffers schema adapter
├── codegen-protobuf/    # Protobuf schema adapter
└── codegen-quiver/      # Quiver-ORM schema adapter
```

## Quick Start

```rust
use codegen_core::{CodeWriter, CodeGenError, ir::ServiceDef};

// Create a code writer
let mut writer = CodeWriter::new();

// Generate code
writer.block("struct MyService", |w| {
    w.line("fn new() -> Self {");
    w.indent();
    w.line("Self {}");
    w.dedent();
    w.line("}");
});

let code = writer.finish();
```

## Design Principles

1. **Zero external dependencies** in core (except `thiserror` for derive)
2. **Feature-gated modules** for different schema formats
3. **Schema-agnostic IR** that adapters can convert to
4. **Adapter Pattern**: Each schema format has its own adapter

## IR Types

- `ServiceDef` - gRPC service definition
- `MethodDef` - single RPC method
- `MessageDef` - message/struct/table definition
- `FieldDef` - field within a message
- `Type` - scalar, message, enum, or vector types
- `EnumDef` - enum/union definition
- `EnumValue` - single enum value

## Building

```bash
cargo build
cargo test
cargo fmt
cargo clippy
```

## Design

See [DESIGN.md](DESIGN.md) for detailed architecture documentation.

## Features

- `codegen-core/grpc` - enables gRPC-specific IR types
- `codegen-flatbuffers/grpc` - enables gRPC service support in FlatBuffers adapter
