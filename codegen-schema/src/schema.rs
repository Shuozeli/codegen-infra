//! Unified schema definitions.

use super::Type;

/// Trait for schema providers.
///
/// Adapters implement this trait to convert from format-specific
/// schema types to the unified [`SchemaDef`].
pub trait SchemaProvider {
    /// Get all message definitions.
    fn messages(&self) -> Vec<MessageDef>;

    /// Get all enum definitions.
    fn enums(&self) -> Vec<EnumDef>;

    /// Get all service definitions.
    fn services(&self) -> Vec<ServiceDef>;

    /// Get the file identifier, if any.
    fn file_ident(&self) -> Option<&str>;

    /// Get the root table name, if any.
    fn root_table(&self) -> Option<&str>;
}

/// Schema definition containing all types.
#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub name: String,
    pub messages: Vec<MessageDef>,
    pub enums: Vec<EnumDef>,
    pub services: Vec<ServiceDef>,
    /// File identifier (e.g., FlatBuffers file identifier).
    pub file_ident: Option<String>,
    /// Root table name (e.g., for FlatBuffers).
    pub root_table: Option<String>,
}

/// Message/table/struct definition.
#[derive(Debug, Clone)]
pub struct MessageDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub is_struct: bool,
    pub namespace: Option<String>,
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

/// Field definition.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: Type,
    pub is_optional: bool,
    pub default_value: Option<String>,
    /// Field ID (for FlatBuffers).
    pub id: Option<u32>,
    pub comments: Vec<String>,
}

impl FieldDef {
    /// Get the type name for code generation.
    pub fn type_name(&self) -> String {
        let base = self.ty.to_string();
        if self.is_optional {
            format!("Option<{base}>")
        } else {
            base
        }
    }
}

/// Enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub values: Vec<EnumValue>,
    pub is_union: bool,
    pub namespace: Option<String>,
    pub comments: Vec<String>,
}

/// Enum value.
#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub value: i64,
    pub comments: Vec<String>,
}

/// Service definition (RPC).
#[derive(Debug, Clone)]
pub struct ServiceDef {
    pub name: String,
    pub methods: Vec<MethodDef>,
    pub package: Option<String>,
    pub comments: Vec<String>,
}

/// Method definition.
#[derive(Debug, Clone)]
pub struct MethodDef {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    pub streaming: StreamingType,
    /// The codec path to use (e.g., `"crate::codec::Codec"`).
    pub codec_path: String,
    pub comments: Vec<String>,
}

/// Streaming type for RPC methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingType {
    None,   // Unary
    Server, // Server streaming
    Client, // Client streaming
    BiDi,   // Bidirectional streaming
}

impl MethodDef {
    /// Streaming mode as a human-readable string.
    pub fn streaming_mode(&self) -> &'static str {
        match self.streaming {
            StreamingType::None => "unary",
            StreamingType::Server => "server streaming",
            StreamingType::Client => "client streaming",
            StreamingType::BiDi => "bidi streaming",
        }
    }
}

impl From<(bool, bool)> for StreamingType {
    fn from((client, server): (bool, bool)) -> Self {
        match (client, server) {
            (false, false) => StreamingType::None,
            (false, true) => StreamingType::Server,
            (true, false) => StreamingType::Client,
            (true, true) => StreamingType::BiDi,
        }
    }
}
