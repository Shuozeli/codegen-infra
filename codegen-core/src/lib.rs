//! Low-level code generation primitives.
//!
//! This crate provides shared infrastructure for code generation:
//! - [`CodeWriter`] - indentation-aware string builder
//! - [`CodeGenError`] - error type for code generation failures
//!
//! ## Design Principles
//!
//! 1. **Zero external dependencies** (except `thiserror` for derive)
//! 2. **Feature-gated modules** for different schema formats
//! 3. **Schema-agnostic** - schema types live in codegen-schema

pub mod code_writer;
pub mod error;
pub mod reserved_words;

pub use code_writer::CodeWriter;
pub use error::CodeGenError;
pub use reserved_words::{escape_reserved_word, is_reserved_word, Language};
