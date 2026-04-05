//! Low-level code generation primitives.
//!
//! This crate provides shared infrastructure for code generation:
//! - [`CodeWriter`] - indentation-aware string builder
//! - [`CodeGenError`] - error type for code generation failures
//! - [`ir`] - intermediate representation for schema types
//!
//! ## Design Principles
//!
//! 1. **Zero external dependencies** (except `thiserror` for derive)
//! 2. **Feature-gated modules** for different schema formats
//! 3. **Schema-agnostic IR** that adapters can convert to

pub mod code_writer;
pub mod error;
pub mod ir;

pub use code_writer::CodeWriter;
pub use error::CodeGenError;
