//! Reserved word validation and escaping for target languages.
//!
//! This module provides utilities to detect and escape reserved words
//! in various programming languages to prevent generating invalid code.

use std::collections::HashSet;

/// Language for reserved word checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Dart,
    TypeScript,
}

// ---------------------------------------------------------------------------
// Reserved word sets
// ---------------------------------------------------------------------------

/// Rust reserved words.
/// Reference: <https://doc.rust-lang.org/reference/keywords.html>
fn rust_reserved_words() -> HashSet<&'static str> {
    [
        // Keywords
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", // Strict keywords
        "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
        "unsized", "virtual", "yield",
    ]
    .into_iter()
    .collect()
}

/// Dart reserved words and built-in identifiers.
/// Reference: <https://dart.dev/language/keywords>
fn dart_reserved_words() -> HashSet<&'static str> {
    [
        // Keywords
        "abstract",
        "as",
        "assert",
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "covariant",
        "default",
        "deferred",
        "do",
        "dynamic",
        "else",
        "enum",
        "export",
        "extends",
        "extension",
        "external",
        "factory",
        "false",
        "final",
        "finally",
        "for",
        "Function",
        "get",
        "hide",
        "if",
        "implements",
        "import",
        "in",
        "interface",
        "is",
        "late",
        "library",
        "mixin",
        "new",
        "null",
        "on",
        "operator",
        "part",
        "required",
        "rethrow",
        "return",
        "set",
        "show",
        "static",
        "super",
        "switch",
        "sync",
        "this",
        "throw",
        "true",
        "try",
        "typedef",
        "var",
        "void",
        "while",
        "with",
        "yield",
        // Built-in identifiers (contextual keywords)
        "abstract",
        "as",
        "cast",
        "class",
        "enum",
        "extension",
        "external",
        "factory",
        "get",
        "implements",
        "interface",
        "late",
        "mixin",
        "on",
        "operator",
        "required",
        "set",
        "static",
        "typedef",
    ]
    .into_iter()
    .collect()
}

/// TypeScript reserved words and keywords.
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar>
fn typescript_reserved_words() -> HashSet<&'static str> {
    [
        // Keywords
        "any",
        "as",
        "async",
        "await",
        "boolean",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "declare",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "from",
        "function",
        "get",
        "if",
        "implements",
        "import",
        "in",
        "instanceof",
        "interface",
        "is",
        "keyof",
        "let",
        "module",
        "namespace",
        "never",
        "new",
        "null",
        "number",
        "object",
        "of",
        "package",
        "private",
        "protected",
        "public",
        "readonly",
        "require",
        "return",
        "set",
        "static",
        "string",
        "super",
        "switch",
        "symbol",
        "this",
        "throw",
        "true",
        "try",
        "type",
        "typeof",
        "undefined",
        "var",
        "void",
        "while",
        "with",
        "yield",
        // Strict mode keywords
        "any",
        "number",
        "string",
        "boolean",
        "symbol",
        "bigint",
        // Contextual keywords
        "abstract",
        "asserts",
        "constructor",
        "declare",
        "get",
        "set",
        "type",
    ]
    .into_iter()
    .collect()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns true if the given identifier is a reserved word in the target language.
/// The check is case-insensitive since schema names may be in any case.
pub fn is_reserved_word(identifier: &str, language: Language) -> bool {
    let lower = identifier.to_lowercase();
    match language {
        Language::Rust => rust_reserved_words().contains(lower.as_str()),
        Language::Dart => dart_reserved_words().contains(lower.as_str()),
        Language::TypeScript => typescript_reserved_words().contains(lower.as_str()),
    }
}

/// Escapes a reserved word by appending an underscore.
///
/// If the identifier is not a reserved word, returns it unchanged.
/// This approach works across all languages and produces valid identifiers.
/// The check is case-insensitive.
pub fn escape_reserved_word(identifier: &str, language: Language) -> String {
    if is_reserved_word(identifier, language) {
        format!("{}_", identifier)
    } else {
        identifier.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_reserved_words() {
        // Keywords that would commonly appear in schemas
        assert!(is_reserved_word("type", Language::Rust));
        assert!(is_reserved_word("fn", Language::Rust));
        assert!(is_reserved_word("let", Language::Rust));
        assert!(is_reserved_word("mut", Language::Rust));
        assert!(is_reserved_word("struct", Language::Rust));
        assert!(is_reserved_word("enum", Language::Rust));
        assert!(is_reserved_word("match", Language::Rust));
        assert!(is_reserved_word("pub", Language::Rust));
        assert!(is_reserved_word("use", Language::Rust));
        assert!(is_reserved_word("mod", Language::Rust));
        assert!(is_reserved_word("crate", Language::Rust));
        assert!(is_reserved_word("self", Language::Rust));
        assert!(is_reserved_word("Self", Language::Rust));
        assert!(is_reserved_word("trait", Language::Rust));
        assert!(is_reserved_word("impl", Language::Rust));
        assert!(is_reserved_word("where", Language::Rust));
        assert!(is_reserved_word("async", Language::Rust));
        assert!(is_reserved_word("await", Language::Rust));
        assert!(is_reserved_word("move", Language::Rust));
        assert!(is_reserved_word("dyn", Language::Rust));
        assert!(is_reserved_word("ref", Language::Rust));
        assert!(is_reserved_word("const", Language::Rust));
        assert!(is_reserved_word("static", Language::Rust));
        assert!(is_reserved_word("unsafe", Language::Rust));
        assert!(is_reserved_word("extern", Language::Rust));
        assert!(is_reserved_word("as", Language::Rust));
        assert!(is_reserved_word("break", Language::Rust));
        assert!(is_reserved_word("continue", Language::Rust));
        assert!(is_reserved_word("loop", Language::Rust));
        assert!(is_reserved_word("while", Language::Rust));
        assert!(is_reserved_word("for", Language::Rust));
        assert!(is_reserved_word("in", Language::Rust));
        assert!(is_reserved_word("if", Language::Rust));
        assert!(is_reserved_word("else", Language::Rust));
        assert!(is_reserved_word("return", Language::Rust));
        assert!(is_reserved_word("true", Language::Rust));
        assert!(is_reserved_word("false", Language::Rust));

        // Non-reserved words
        assert!(!is_reserved_word("name", Language::Rust));
        assert!(!is_reserved_word("monster", Language::Rust));
        assert!(!is_reserved_word("hp", Language::Rust));
        // Note: Case-insensitive matching means "Type" IS detected as reserved
        // because "type" is a Rust keyword
    }

    #[test]
    fn test_dart_reserved_words() {
        assert!(is_reserved_word("class", Language::Dart));
        assert!(is_reserved_word("void", Language::Dart));
        assert!(is_reserved_word("var", Language::Dart));
        assert!(is_reserved_word("final", Language::Dart));
        assert!(is_reserved_word("const", Language::Dart));
        assert!(is_reserved_word("if", Language::Dart));
        assert!(is_reserved_word("else", Language::Dart));
        assert!(is_reserved_word("for", Language::Dart));
        assert!(is_reserved_word("while", Language::Dart));
        assert!(is_reserved_word("return", Language::Dart));
        assert!(is_reserved_word("true", Language::Dart));
        assert!(is_reserved_word("false", Language::Dart));
        assert!(is_reserved_word("null", Language::Dart));
        assert!(is_reserved_word("this", Language::Dart));
        assert!(is_reserved_word("super", Language::Dart));
        assert!(is_reserved_word("extends", Language::Dart));
        assert!(is_reserved_word("implements", Language::Dart));
        assert!(is_reserved_word("import", Language::Dart));
        assert!(is_reserved_word("enum", Language::Dart));
        assert!(is_reserved_word("typedef", Language::Dart));
        assert!(is_reserved_word("async", Language::Dart));
        assert!(is_reserved_word("await", Language::Dart));
        assert!(is_reserved_word("yield", Language::Dart));
        assert!(is_reserved_word("static", Language::Dart));
        assert!(is_reserved_word("abstract", Language::Dart));
        assert!(is_reserved_word("interface", Language::Dart));
        assert!(is_reserved_word("mixin", Language::Dart));
        assert!(is_reserved_word("extension", Language::Dart));
        assert!(is_reserved_word("get", Language::Dart));
        assert!(is_reserved_word("set", Language::Dart));
        assert!(is_reserved_word("operator", Language::Dart));

        // Non-reserved words
        assert!(!is_reserved_word("name", Language::Dart));
        assert!(!is_reserved_word("monster", Language::Dart));
    }

    #[test]
    fn test_typescript_reserved_words() {
        assert!(is_reserved_word("function", Language::TypeScript));
        assert!(is_reserved_word("var", Language::TypeScript));
        assert!(is_reserved_word("let", Language::TypeScript));
        assert!(is_reserved_word("const", Language::TypeScript));
        assert!(is_reserved_word("if", Language::TypeScript));
        assert!(is_reserved_word("else", Language::TypeScript));
        assert!(is_reserved_word("for", Language::TypeScript));
        assert!(is_reserved_word("while", Language::TypeScript));
        assert!(is_reserved_word("class", Language::TypeScript));
        assert!(is_reserved_word("extends", Language::TypeScript));
        assert!(is_reserved_word("implements", Language::TypeScript));
        assert!(is_reserved_word("interface", Language::TypeScript));
        assert!(is_reserved_word("type", Language::TypeScript));
        assert!(is_reserved_word("enum", Language::TypeScript));
        assert!(is_reserved_word("import", Language::TypeScript));
        assert!(is_reserved_word("export", Language::TypeScript));
        assert!(is_reserved_word("async", Language::TypeScript));
        assert!(is_reserved_word("await", Language::TypeScript));
        assert!(is_reserved_word("yield", Language::TypeScript));
        assert!(is_reserved_word("public", Language::TypeScript));
        assert!(is_reserved_word("private", Language::TypeScript));
        assert!(is_reserved_word("protected", Language::TypeScript));
        assert!(is_reserved_word("readonly", Language::TypeScript));
        assert!(is_reserved_word("abstract", Language::TypeScript));
        assert!(is_reserved_word("new", Language::TypeScript));
        assert!(is_reserved_word("delete", Language::TypeScript));
        assert!(is_reserved_word("typeof", Language::TypeScript));
        assert!(is_reserved_word("string", Language::TypeScript));
        assert!(is_reserved_word("number", Language::TypeScript));
        assert!(is_reserved_word("boolean", Language::TypeScript));
        assert!(is_reserved_word("any", Language::TypeScript));
        assert!(is_reserved_word("void", Language::TypeScript));
        assert!(is_reserved_word("never", Language::TypeScript));
        assert!(is_reserved_word("null", Language::TypeScript));
        assert!(is_reserved_word("undefined", Language::TypeScript));
        assert!(is_reserved_word("true", Language::TypeScript));
        assert!(is_reserved_word("false", Language::TypeScript));
        assert!(is_reserved_word("in", Language::TypeScript));
        assert!(is_reserved_word("instanceof", Language::TypeScript));
        assert!(is_reserved_word("return", Language::TypeScript));
        assert!(is_reserved_word("from", Language::TypeScript));
        assert!(is_reserved_word("as", Language::TypeScript));
        assert!(is_reserved_word("default", Language::TypeScript));

        // Non-reserved words
        assert!(!is_reserved_word("name", Language::TypeScript));
        assert!(!is_reserved_word("monster", Language::TypeScript));
    }

    #[test]
    fn test_escape_reserved_word() {
        // Rust
        assert_eq!(escape_reserved_word("type", Language::Rust), "type_");
        assert_eq!(escape_reserved_word("fn", Language::Rust), "fn_");
        assert_eq!(escape_reserved_word("name", Language::Rust), "name"); // Not reserved

        // Dart
        assert_eq!(escape_reserved_word("class", Language::Dart), "class_");
        assert_eq!(escape_reserved_word("var", Language::Dart), "var_");
        assert_eq!(escape_reserved_word("name", Language::Dart), "name"); // Not reserved

        // TypeScript
        assert_eq!(escape_reserved_word("type", Language::TypeScript), "type_");
        assert_eq!(
            escape_reserved_word("function", Language::TypeScript),
            "function_"
        );
        assert_eq!(escape_reserved_word("name", Language::TypeScript), "name"); // Not reserved
    }
}
