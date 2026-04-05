# Code Review Issues

<!-- agent-updated: 2026-04-05T21:15:00Z -->

## HIGH Severity

| # | Issue | Status | Notes |
|---|-------|--------|-------|
| 1 | `Type::OneOf` can't represent protobuf oneofs - loses field names | TODO | |
| 2 | `to_snake_case` broken for consecutive capitals | TODO | |
| 3 | StreamingType hardcoded in adapters | TODO | |
| 4 | `element_type()` returns None for Optional/OneOf | TODO | |
| 5 | FlatBuffers `is_struct` flag always `false` | TODO | |

## MEDIUM Severity

| # | Issue | Status | Notes |
|---|-------|--------|-------|
| 6 | Soft errors via `eprintln` instead of propagating | TODO | |
| 7 | No adapter produces `Type::OneOf` | DONE | FlatBuffers unions and Protobuf oneofs now produce `Type::OneOf`. Quiver has no equivalent (not applicable). |
| 8 | `is_union` flag inconsistent across adapters | TODO | |
| 9 | Quiver `ForeignKey` loses referential info | TODO | |
| 10 | `InlineStruct` is dead code | DONE | Removed from `codegen-schema::Type` enum and all writer match arms. No adapter produced `InlineStruct`. |
| 11 | Protobuf `map<K,V>` becomes `Vector` | DONE | Added `Type::Map { key, value }` to codegen-schema, updated Protobuf adapter to detect map_entry messages and produce Map instead of Vector, updated all code writers (Rust: HashMap, Dart: Map, TypeScript: Record). |
| 12 | Rust `todo!()` in service generation | DONE | Rust service methods now properly handle StreamingType: Server returns `Stream<Output>`, Client accepts `Stream<Input>` and returns `Output`, BiDi accepts and returns `Stream<T>`. |

## LOW Severity

| # | Issue | Status | Notes |
|---|-------|--------|-------|
| 13 | Test expects lowercase `Vec<string>` | DONE | Fixed `ScalarType::String` Display impl to output `String` (uppercase), updated `field_def_type_name_complex` and `scalar_type_display` tests accordingly. |
| 14 | Dart writer hardcodes `pbGrpc` import | DONE | Derive client class from `ServiceDef.package` (e.g., `helloworld` -> `HelloworldClient`), add dynamic `package:{package}.pbrpc.dart` import in `write_file`. |
| 15 | No reserved word validation | DONE | Added reserved word detection and escaping for Rust, Dart, and TypeScript in codegen-core. Field names, type names, enum names, and method names that match reserved words are escaped with underscore suffix. Reserved word checking is case-insensitive to catch PascalCase variants. |
| 16 | `FieldDef::id` unused in writers | DONE | Retained in schema for FlatBuffers/Protobuf wire format compatibility and schema analysis. Not used by code writers (Rust/Dart/TypeScript) since field names define the API. Quiver sets `id: None` as it has no field IDs. |
| 17 | `ScalarType` non_exhaustive but fallbacks mask issues | DONE | Removed `#[non_exhaustive]` from ScalarType in codegen-schema and removed catch-all `_ =>` arms from codegen-writers (rust.rs, dart.rs, typescript.rs). Now adding a new ScalarType variant will cause a compile error in codegen-writers. |

---

## Issue Details

### HIGH #1: Type::OneOf loses field names

**Problem:** `Type::OneOf { name: String, variants: Vec<Type> }` only stores types, not field names.

**Protobuf oneof:**
```protobuf
oneof result {
  int32 i = 1;
  string s = 2;
}
```

**Current IR:** loses `i` and `s` field names.

**Fix:** Add named variants:
```rust
pub struct OneOfVariant {
    pub name: String,
    pub ty: Type,
}
pub enum Type {
    OneOf { name: String, variants: Vec<OneOfVariant> },
}
```

---

### HIGH #2: to_snake_case consecutive capitals

**Problem:** `"HTTPServer"` becomes `"h_t_t_p_server"` instead of `"http_server"`.

**Current impl:** inserts `_` between every capital letter.

**Fix:** Handle consecutive capitals as one word, lowercase first char only.

---

### HIGH #3: StreamingType hardcoded

**Problem:**
- FlatBuffers always `(false, false)` - no streaming support
- Protobuf `codec_path` hardcoded to `"crate::codec::Codec"`

**Fix:**
- FlatBuffers: detect streaming from IDL attributes
- Protobuf: adapt `codec_path` from actual proto

---

### HIGH #4: element_type() incomplete

**Problem:** Returns `None` for Optional, OneOf, InlineStruct - but they have inner types.

**Fix:** Return `Some` for all types with inner elements.

---

### HIGH #5: is_struct always false

**Problem:** `codegen-flatbuffers` always sets `is_struct: false` ignoring source.

**Fix:** Use `o.is_struct` from FlatBuffers schema.
