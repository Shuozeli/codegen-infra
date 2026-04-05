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

impl ServiceDef {
    /// Fully-qualified name (e.g., `"helloworld.Greeter"`).
    pub fn fully_qualified_name(&self) -> String {
        if let Some(ref pkg) = self.package {
            format!("{}.{}", pkg, self.name)
        } else {
            self.name.clone()
        }
    }

    /// Validate this service definition.
    ///
    /// Returns a list of problems. An empty vec means the definition is valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("service name is empty".into());
        }
        for (i, m) in self.methods.iter().enumerate() {
            if m.name.is_empty() {
                errors.push(format!("method[{i}] name is empty"));
            }
            if m.input_type.is_empty() {
                errors.push(format!("method[{i}] `{}` input_type is empty", m.name));
            }
            if m.output_type.is_empty() {
                errors.push(format!("method[{i}] `{}` output_type is empty", m.name));
            }
        }
        errors
    }
}

/// Method definition.
#[derive(Debug, Clone)]
pub struct MethodDef {
    /// Proto-style method name (e.g., `"SayHello"`).
    pub name: String,
    /// Optional snake_case name for Rust (e.g., `"say_hello"`).
    /// If None, code generators should derive it via `name.to_snake_case()`.
    pub rust_name: Option<String>,
    /// Fully-qualified input type path (e.g., `"crate::HelloRequest"`).
    pub input_type: String,
    /// Fully-qualified output type path (e.g., `"crate::HelloReply"`).
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

    /// gRPC path for this method (e.g., `"/helloworld.Greeter/SayHello"`).
    pub fn grpc_path(&self, service_fqn: &str) -> String {
        format!("/{service_fqn}/{}", self.name)
    }

    /// Get the Rust method name (snake_case).
    ///
    /// If `rust_name` is set, use it; otherwise derive from `name`.
    pub fn rust_name(&self) -> String {
        self.rust_name
            .clone()
            .unwrap_or_else(|| heck::ToSnakeCase::to_snake_case(self.name.as_str()))
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
