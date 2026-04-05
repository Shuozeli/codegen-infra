# codegen-infra Tasks

<!-- agent-updated: 2026-04-04T00:00:00Z -->

## Overview

Unified code generation infrastructure for Rust. A low-level library providing shared primitives for code generation across multiple schema formats.

## Phases

### Phase 1: Foundation (Current)

- [x] Create `codegen-core` crate with basic IR types
- [x] Create `codegen-flatbuffers` adapter
- [x] Write DESIGN.md
- [x] Write CLAUDE.md
- [x] **Task**: Add unit tests for `SchemaProvider` implementation
- [x] **Task**: Add `Bytes` scalar type support to adapter

### Phase 2: Protobuf Adapter

- [x] Create `codegen-protobuf` crate
- [x] **Task**: Define protobuf-specific IR extensions (if needed)
- [x] **Task**: Implement `SchemaProvider` for protobuf `FileDescriptorProto`
- [x] **Task**: Add unit tests for SchemaProvider implementation

### Phase 3: Quiver-ORM Adapter

- [x] Create `codegen-quiver` crate
- [x] **Task**: Define Quiver schema types
- [x] **Task**: Implement `SchemaProvider` for Quiver schema
- [x] **Task**: Add unit tests

### Phase 4: Code Writers

- [ ] Create `codegen-writers` workspace member or separate crates
- [ ] **Task**: Implement Rust code writer (`CodeWriter` trait + implementations)
- [ ] **Task**: Implement Dart code writer
- [ ] **Task**: Implement TypeScript code writer

### Phase 5: gRPC Support

- [ ] **Task**: Add `grpc` feature flag to `codegen-core`
- [ ] **Task**: Implement gRPC service generation in Rust writer
- [ ] **Task**: Implement gRPC service generation in Dart writer
- [ ] **Task**: Implement gRPC service generation in TypeScript writer

### Phase 6: Integration & Polish

- [ ] **Task**: End-to-end integration tests (schema → IR → generated code → compile)
- [ ] **Task**: Add benchmarks for code generation speed
- [ ] **Task**: Add CLI tool for schema → code generation
- [ ] **Task**: Publish to crates.io

## Task Details

### Phase 1 Tasks

#### T1.1: Add unit tests for SchemaProvider implementation

**Context**: The `FlatbuffersSchema` wrapper needs tests to verify correct conversion.

**Acceptance Criteria**:
- [ ] Test `messages()` returns only tables (not structs)
- [ ] Test `services()` correctly maps request/response indices
- [ ] Test `enums()` correctly converts union flag
- [ ] Test `root_table()` returns correct table name

#### T1.2: Add `Bytes` scalar type support

**Context**: FlatBuffers `BASE_TYPE_STRING` maps to String, but `bytes` fields need proper handling.

**Acceptance Criteria**:
- [ ] Map `BaseType::BASE_TYPE_STRING` → `ScalarType::String` (done)
- [ ] Handle byte arrays in vectors correctly
- [ ] Add test with schema containing byte fields

---

## Status

| Phase | Status |
|-------|--------|
| Phase 1: Foundation | ✅ Complete |
| Phase 2: Protobuf Adapter | ✅ Complete |
| Phase 3: Quiver-ORM Adapter | ✅ Complete |
| Phase 4: Code Writers | Pending |
| Phase 5: gRPC Support | Pending |
| Phase 6: Integration & Polish | Pending |

## Progress

- **Completed**: 15 items
- **In Progress**: 0 items
- **Pending**: 9 items
