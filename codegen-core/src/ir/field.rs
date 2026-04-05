//! Field definitions.

use super::r#type::Type;

/// Definition of a field within a message.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// The field name (e.g., `"hp"`, `"name"`).
    pub name: String,
    /// The field type.
    pub ty: Type,
    /// Whether this field is optional/nullable.
    pub is_optional: bool,
    /// Default value (if any).
    pub default_value: Option<String>,
    /// Field ID (for FlatBuffers).
    pub id: Option<u32>,
    /// Doc comments.
    pub comments: Vec<String>,
}

impl FieldDef {
    /// Get the type name for code generation.
    #[allow(dead_code)]
    pub fn type_name(&self) -> String {
        let base = self.ty.to_string();
        if self.is_optional {
            format!("Option<{base}>")
        } else {
            base
        }
    }
}
