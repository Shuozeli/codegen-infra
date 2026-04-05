//! Code writers for multiple languages.
//!
//! This module provides language-specific code writers that consume the common
//! schema types and produce code in the target language.

mod dart;
mod rust;
mod typescript;

pub use dart::DartCodeWriter;
pub use rust::RustCodeWriter;
pub use typescript::TypeScriptCodeWriter;

use codegen_schema::{EnumDef, MessageDef, ServiceDef};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CodeWriterError {
    #[error("indent error: {0}")]
    IndentError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

// ---------------------------------------------------------------------------
// CodeWriter trait
// ---------------------------------------------------------------------------

/// Trait for language-specific code writers.
pub trait CodeWriter {
    /// Generate code for a message (struct/table/class).
    fn write_message(&mut self, msg: &MessageDef) -> Result<String, CodeWriterError>;

    /// Generate code for an enum.
    fn write_enum(&mut self, enm: &EnumDef) -> Result<String, CodeWriterError>;

    /// Generate code for a service.
    fn write_service(&mut self, svc: &ServiceDef) -> Result<String, CodeWriterError>;

    /// Generate a complete file with all messages, enums, and services.
    fn write_file(
        &mut self,
        messages: &[MessageDef],
        enums: &[EnumDef],
        services: &[ServiceDef],
    ) -> Result<String, CodeWriterError>;
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Convert a type name to the target language naming convention.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a type name to snake_case.
///
/// Handles consecutive uppercase letters (acronyms) correctly:
/// - "HTTPServer" -> "http_server"
/// - "MyHTTPServer" -> "my_http_server"
/// - "IOError" -> "io_error"
pub fn to_snake_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            // Check if this uppercase starts a new word: preceded by uppercase AND followed by lowercase
            if i > 0
                && chars[i - 1].is_uppercase()
                && i + 1 < chars.len()
                && chars[i + 1].is_lowercase()
            {
                // This uppercase is preceded by uppercase and followed by lowercase:
                // it's the LAST uppercase of the acronym, starting the next word.
                // Add underscore before it.
                result.push('_');
            }
            if !result.is_empty()
                && !result.ends_with('_')
                && !chars
                    .get(result.len().saturating_sub(1))
                    .is_some_and(|&x| x.is_uppercase())
            {
                // Previous char was lowercase: add underscore
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a field name to camelCase.
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    let mut is_first = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            if is_first {
                result.push(c.to_ascii_lowercase());
                is_first = false;
            } else {
                result.push(c);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("simple"), "Simple");
        assert_eq!(to_pascal_case("grpc_service"), "GrpcService");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("Simple"), "simple");
        assert_eq!(to_snake_case("GrpcService"), "grpc_service");
        // Consecutive uppercase (acronyms) - per issue description
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("MyHTTPServer"), "my_http_server");
        assert_eq!(to_snake_case("IOError"), "io_error");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("simple_name"), "simpleName");
        assert_eq!(to_camel_case("grpc_service_name"), "grpcServiceName");
        // PascalCase input (e.g., from protobuf) should be lowercased
        assert_eq!(to_camel_case("CreateMonster"), "createMonster");
        assert_eq!(to_camel_case("GetMonster"), "getMonster");
        // Consecutive capitals: only first char lowercased
        assert_eq!(to_camel_case("HTTPServer"), "hTTPServer");
    }
}
