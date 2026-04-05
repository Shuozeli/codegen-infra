# codegen-infra Plans

<!-- agent-updated: 2026-04-05T05:40:00Z -->

## Overview

This document outlines the roadmap and phases for codegen-infra development.

---

## Phase 1: IR Enhancements (High Priority)

### 1.1 Streaming RPC Support

**Status:** Partial (flags exist but ignored)

**Goal:** Full streaming support for all 4 gRPC method types.

**Changes to IR:**

```rust
pub struct MethodDef {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    pub streaming: StreamingType,
    pub comments: Vec<String>,
}

pub enum StreamingType {
    None,                    // Unary
    Server,                  // Server streaming
    Client,                  // Client streaming
    BiDi,                    // Bidirectional streaming
}
```

**Tasks:**
- [ ] Update `MethodDef` in `codegen-core/src/ir/service.rs`
- [ ] Update FlatBuffers adapter to detect streaming
- [ ] Update Protobuf adapter to detect streaming
- [ ] Update Quiver adapter (if applicable)
- [ ] Update RustCodeWriter to generate `Stream<T>` return types
- [ ] Update DartCodeWriter to generate `Stream<T>` return types
- [ ] Update TypeScriptCodeWriter to generate `AsyncGenerator<T>`
- [ ] Add integration tests for streaming methods

### 1.2 OneOf / Unions Support

**Status:** Not supported

**Goal:** Unified representation of discriminated unions across schema formats.

**Changes to IR:**

```rust
// In Type enum
pub enum Type {
    // ... existing variants
    OneOf {
        name: String,
        variants: Vec<Type>,
    },
}

// New top-level type
pub struct OneOfDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
    pub comments: Vec<String>,
}

pub struct VariantDef {
    pub name: String,
    pub type_name: Option<String>,  // None = void/empty
}
```

**Tasks:**
- [ ] Add `OneOf` variant to `Type` enum
- [ ] Add `OneOfDef` struct
- [ ] Update FlatBuffers adapter (unions → OneOf)
- [ ] Update Protobuf adapter (oneof → OneOf)
- [ ] Update Quiver adapter (if applicable)
- [ ] Update all code writers to generate union/oneof types
- [ ] Add integration tests

---

## Phase 2: Code Generation Improvements (High Priority)

### 2.1 Add syn Support

**Status:** Raw strings only

**Goal:** Enable AST-level code generation for Rust output.

**Approach:**

Create optional `codegen-core-syn` crate:

```
codegen-core/
codegen-core-syn/     # Optional, behind feature flag
```

**Features:**
- `syn::Item` generation from IR types
- `quote::Tokens` for template expansion
- Optional pretty-printing with `prettyplease`

**Tasks:**
- [ ] Create `codegen-core-syn` crate
- [ ] Add `syn`, `quote`, `proc-macro2` dependencies (behind feature)
- [ ] Implement `RustCodeGenerator` trait for syn-based output
- [ ] Add `#[cfg(feature = "syn")]` conditional compilation
- [ ] Add tests comparing syn output vs raw string output
- [ ] Document when to use syn vs raw strings

### 2.2 Go Code Writer

**Status:** Not started

**Goal:** Add Go as a target language.

**Tasks:**
- [ ] Create `codegen-writers/src/go.rs`
- [ ] Implement `CodeWriter` trait for Go
- [ ] Go-specific type mappings (int, string, []byte, etc.)
- [ ] Add integration tests

### 2.3 Python Code Writer

**Status:** Not started

**Goal:** Add Python as a target language.

**Tasks:**
- [ ] Create `codegen-writers/src/python.rs`
- [ ] Implement `CodeWriter` trait for Python
- [ ] Python-specific type mappings (int, str, bytes, List, etc.)
- [ ] Add integration tests

---

## Phase 3: Consumer Migrations (Medium Priority)

### Migration Status

| Project | Status | Notes |
|---------|--------|-------|
| flatbuffers-rs | Partial | Uses codegen_core::CodeWriter, but has local duplicate helpers |
| protobuf-rs | Partial | rust_gen.rs uses codegen_core, rust_gen_runtime.rs still old |
| quiver-orm | Not started | Uses raw strings, no codegen-core usage |

### Tasks

#### flatbuffers-rs
- [ ] Remove duplicate casing helpers in `type_map.rs`
- [ ] Use `codegen_core::to_snake_case`, etc.
- [ ] Verify all tests pass

#### protobuf-rs
- [ ] Migrate `rust_gen_runtime.rs` to use codegen_core
- [ ] Remove duplicate CodeWriter from runtime
- [ ] Verify all tests pass

#### quiver-orm
- [ ] Evaluate if codegen-core fits quiver-codegen architecture
- [ ] If yes: migrate to codegen_core::CodeWriter
- [ ] If no: document why (architectural decision)

---

## Phase 4: Documentation (Low Priority)

### 4.1 Quiver Adapter Scope

**Status:** Unclear

**Goal:** Document what Quiver features are supported.

**Tasks:**
- [ ] Audit quiver-orm schema types
- [ ] Document supported Quiver schema features
- [ ] Document unsupported features (with reasons)
- [ ] Add to DESIGN.md

### 4.2 Update DESIGN.md

- [ ] Add Phase 1-3 status
- [ ] Update architecture diagram if needed
- [ ] Document syn vs raw strings decision
- [ ] Add consumer migration status table

---

## Open Questions

| Question | Decision Needed |
|----------|-----------------|
| OneOf in Type enum vs standalone? | Standalone `OneOfDef` is cleaner |
| Quiver streaming support? | Check if Quiver supports streaming |
| Go/Python priority? | Go first, then Python |
| syn crate name? | `codegen-core-syn` or separate `codegen-syn`? |

---

## TODO Checklist

### Phase 1
- [ ] Streaming RPC: Update MethodDef
- [ ] Streaming RPC: Update adapters
- [ ] Streaming RPC: Update code writers
- [ ] Streaming RPC: Add tests
- [ ] OneOf: Add to Type enum
- [ ] OneOf: Add OneOfDef struct
- [ ] OneOf: Update adapters
- [ ] OneOf: Update code writers
- [ ] OneOf: Add tests

### Phase 2
- [ ] Create codegen-core-syn crate
- [ ] Implement syn-based Rust generation
- [ ] Create Go code writer
- [ ] Create Python code writer

### Phase 3
- [ ] Complete flatbuffers-rs migration
- [ ] Complete protobuf-rs migration
- [ ] Evaluate quiver-orm migration

### Phase 4
- [ ] Document Quiver adapter scope
- [ ] Update DESIGN.md
