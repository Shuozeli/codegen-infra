//! Message (struct/table) definitions.

use super::field::FieldDef;

/// Definition of a message type (struct or table in FlatBuffers, message in Protobuf).
#[derive(Debug, Clone)]
pub struct MessageDef {
    /// The message name (e.g., `"Monster"`).
    pub name: String,
    /// Fields in this message.
    pub fields: Vec<FieldDef>,
    /// Whether this is a struct (FlatBuffers) vs table/proto message.
    pub is_struct: bool,
    /// Namespace this message belongs to.
    pub namespace: Option<String>,
    /// Doc comments.
    pub comments: Vec<String>,
}

impl MessageDef {
    /// Validate this message definition.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("message name is empty".into());
        }
        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for (i, f) in self.fields.iter().enumerate() {
            if !field_names.insert(f.name.clone()) {
                errors.push(format!("duplicate field name '{}' at index {}", f.name, i));
            }
        }
        errors
    }

    /// Get a field by name.
    pub fn field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }
}
