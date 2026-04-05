//! Error types for code generation.

use thiserror::Error;

/// Errors that can occur during code generation.
#[derive(Debug, Error)]
pub enum CodeGenError {
    /// Internal error indicating a bug in the code generator.
    #[error("internal codegen error: {0}")]
    Internal(String),

    /// User-facing error for invalid schema input.
    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    /// Error when accessing schema fields that should be present after validation.
    #[error("field '{0}' is missing")]
    MissingField(&'static str),

    /// Error for unsupported schema features.
    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

/// Result type for code generation.
pub type Result<T> = std::result::Result<T, CodeGenError>;
