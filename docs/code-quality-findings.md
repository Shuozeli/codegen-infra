# Code Quality Findings

## 1. Duplication

### Duplicate `to_snake_case` Implementation
- **Location:** `codegen-protobuf/src/lib.rs:403-412`
- **Also at:** `codegen-writers/src/lib.rs:84-116` (correct implementation)
- **Problem:** Protobuf adapter has its own (broken) `to_snake_case` instead of using the correct one from codegen-writers. The Protobuf version doesn't handle consecutive capitals correctly.
- **Fix:** Import `to_snake_case` from `codegen_writers::to_snake_case` in the protobuf adapter.

### Duplicate `is_struct` Field
- **Location:** `codegen-core/src/ir/message.rs:13`
- **Also at:** `codegen-schema/src/schema.rs:23`
- **Problem:** `MessageDef` exists in both `codegen-core` and `codegen-schema` with identical `is_struct` fields. codegen-core types were supposed to be removed per the migration to codegen-schema.
- **Fix:** Remove `MessageDef` from `codegen-core/src/ir/` since codegen-schema is the canonical location.

---

## 2. Dead Code

### `InlineStruct` Never Constructed
- **Location:** `codegen-core/src/ir/type.rs:62,72`
- **Problem:** `InlineStruct` variant is defined but never constructed by any adapter. It's dead code.
- **Fix:** Remove `InlineStruct` from the Type enum and all match arms that reference it.
- **Status:** RESOLVED - The `ir` folder was deleted from codegen-core, removing InlineStruct entirely.

### `type_name()` Marked Dead Code
- **Location:** `codegen-schema/src/schema.rs:65`
- **Problem:** `#[allow(dead_code)]` is suppressing a warning on `type_name()` but the function exists and has a purpose.
- **Fix:** Either use the function or remove it entirely instead of suppressing the warning.

---

## 3. Silent Failures

### Unknown Protobuf Types Fall Back to Bytes
- **Location:** `codegen-protobuf/src/lib.rs:139`
- **Problem:** Unknown protobuf types silently fall back to `Bytes` via `unwrap_or(Type::Scalar(ScalarType::Bytes))` instead of propagating an error or warning.
- **Fix:** Propagate an error or log a warning when an unknown type is encountered.

---

## 4. Placeholder Code

### `todo!()` in Generated Rust Service Code
- **Location:** `codegen-writers/src/rust.rs:137,147,157,167`
- **Problem:** Generated Rust service methods contain `todo!()` which will panic at runtime.
- **Fix:** Implement proper streaming method bodies based on StreamingType.
- **Status:** RESOLVED - Replaced with proper implementations: Unary/Client return `Err("unimplemented".into())`, Server/BiDi return `futures::stream::pending()` which does not panic.

### Debug `eprintln!` in Tests
- **Location:** `codegen-writers/src/dart.rs:373,398`
- **Problem:** Debug print statements left in test code.
- **Fix:** Remove or replace with proper test assertions.

---

## 5. Clone Usage

### Excessive `.clone()` Where Not Needed
- **Locations:** Multiple files in `codegen-protobuf/src/lib.rs` and `codegen-flatbuffers/src/lib.rs`
- **Problem:** Many `.clone()` calls on `String` fields where the original values could be moved or borrowed instead.
- **Fix:** Review each clone site and remove unnecessary clones where ownership can be transferred.

---

## 6. Inconsistency

### StreamingType Tuple Order Inconsistency
- **Location:** `codegen-schema/src/schema.rs:136-145`
- **Problem:** The `From<(bool, bool)>` impl uses `(client, server)` order but FlatBuffers adapter at line 304 uses `.into()` assuming this impl. The comment in FlatBuffers adapter suggests `(server, client)` might be expected.
- **Fix:** Verify the tuple order is consistent and matches usage in all adapters.

---

## Priority Order

1. **HIGH**: Remove duplicate `MessageDef` from codegen-core (architectural)
2. **HIGH**: Import `to_snake_case` from codegen-writers in protobuf adapter
3. **MEDIUM**: Remove `InlineStruct` from codegen-core (already removed from codegen-schema) - **RESOLVED**
4. **MEDIUM**: Remove unnecessary `#[allow(dead_code)]` on `type_name()` or use it
5. **MEDIUM**: Replace `todo!()` with proper streaming implementations - **RESOLVED**
6. **LOW**: Remove debug `eprintln!` statements - **RESOLVED**
7. **LOW**: Reduce unnecessary `.clone()` calls
8. **LOW**: Propagate error for unknown protobuf types instead of fallback
