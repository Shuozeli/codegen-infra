//! Schema provider trait.

use super::{MessageDef, ServiceDef};

/// Represents a scalar type.
pub use super::r#type::ScalarType;

/// Trait for accessing schema information.
///
/// Implementors convert from schema-specific types (e.g., FlatBuffers `ResolvedSchema`,
/// Protobuf `FileDescriptorProto`) to the common IR.
pub trait SchemaProvider {
    /// Get all message types in the schema.
    fn messages(&self) -> Vec<MessageDef>;

    /// Get all service definitions in the schema.
    fn services(&self) -> Vec<ServiceDef>;

    /// Get all enum definitions in the schema.
    fn enums(&self) -> Vec<EnumDef>;

    /// Get the file identifier (FlatBuffers) or None.
    fn file_ident(&self) -> Option<&str>;

    /// Get the root table name if defined.
    fn root_table(&self) -> Option<&str>;
}

/// Enum definition.
#[derive(Debug, Clone)]
pub struct EnumDef {
    /// The enum name.
    pub name: String,
    /// Values in the enum.
    pub values: Vec<EnumValue>,
    /// Whether this is a union (FlatBuffers) or bitflags.
    pub is_union: bool,
    /// Whether this is a struct-backed enum.
    pub is_struct: bool,
    /// Namespace.
    pub namespace: Option<String>,
    /// Doc comments.
    pub comments: Vec<String>,
}

/// A single value in an enum.
#[derive(Debug, Clone)]
pub struct EnumValue {
    /// The value name.
    pub name: String,
    /// The numeric value.
    pub value: i64,
    /// Doc comments.
    pub comments: Vec<String>,
}
