//! Quiver-ORM schema adapter for codegen-core IR.
//!
//! Converts from Quiver-ORM schema types to the common intermediate representation.
//!
//! Quiver-ORM uses a code-first schema definition approach. This adapter expects
//! a simplified schema structure that can be derived from Quiver model definitions.

use codegen_core::ir::{
    EnumDef, EnumValue, FieldDef, MessageDef, SchemaProvider, ServiceDef, Type,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum QuiverAdapterError {
    #[error("unsupported field type: {0}")]
    UnsupportedFieldType(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),
}

// ---------------------------------------------------------------------------
// Quiver schema types (simplified representation)
// ---------------------------------------------------------------------------

/// Represents a Quiver model field.
#[derive(Debug, Clone)]
pub struct QuiverField {
    pub name: String,
    pub column_type: QuiverColumnType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub is_unique: bool,
}

/// Quiver column types.
#[derive(Debug, Clone)]
pub enum QuiverColumnType {
    /// Integer types
    Int32,
    Int64,
    UInt32,
    UInt64,
    /// Floating point
    Float32,
    Float64,
    /// String/Bytes
    String,
    Bytes,
    /// Boolean
    Bool,
    /// Date/Time
    DateTime,
    Date,
    Time,
    /// UUID
    Uuid,
    /// JSON
    Json,
    /// Serial (auto-increment)
    Serial,
    /// Foreign key reference
    ForeignKey {
        model: String,
        column: String,
    },
}

/// Represents a Quiver model (table).
#[derive(Debug, Clone)]
pub struct QuiverModel {
    pub name: String,
    pub table_name: String,
    pub fields: Vec<QuiverField>,
    pub primary_key: Option<String>,
    pub indices: Vec<QuiverIndex>,
    pub foreign_keys: Vec<QuiverForeignKey>,
}

/// Represents an index on a model.
#[derive(Debug, Clone)]
pub struct QuiverIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

/// Represents a foreign key relationship.
#[derive(Debug, Clone)]
pub struct QuiverForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    pub referenced_model: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: String,
    pub on_update: String,
}

/// Represents a Quiver enum (similar to Rust enum).
#[derive(Debug, Clone)]
pub struct QuiverEnum {
    pub name: String,
    pub variants: Vec<QuiverEnumVariant>,
    pub is_exhaustive: bool,
}

/// Single enum variant.
#[derive(Debug, Clone)]
pub struct QuiverEnumVariant {
    pub name: String,
    pub discriminant: i64,
    pub fields: Vec<String>,
}

/// Represents a Quiver schema.
#[derive(Debug, Clone, Default)]
pub struct QuiverSchema {
    pub models: Vec<QuiverModel>,
    pub enums: Vec<QuiverEnum>,
    pub file_ident: Option<String>,
}

// ---------------------------------------------------------------------------
// Type mapping
// ---------------------------------------------------------------------------

impl QuiverColumnType {
    fn to_ir_type(&self) -> Type {
        match self {
            QuiverColumnType::Int32 => Type::Scalar(codegen_core::ir::ScalarType::Int32),
            QuiverColumnType::Int64 => Type::Scalar(codegen_core::ir::ScalarType::Int64),
            QuiverColumnType::UInt32 => Type::Scalar(codegen_core::ir::ScalarType::Uint32),
            QuiverColumnType::UInt64 => Type::Scalar(codegen_core::ir::ScalarType::Uint64),
            QuiverColumnType::Float32 => Type::Scalar(codegen_core::ir::ScalarType::Float32),
            QuiverColumnType::Float64 => Type::Scalar(codegen_core::ir::ScalarType::Float64),
            QuiverColumnType::String => Type::Scalar(codegen_core::ir::ScalarType::String),
            QuiverColumnType::Bytes => Type::Scalar(codegen_core::ir::ScalarType::Bytes),
            QuiverColumnType::Bool => Type::Scalar(codegen_core::ir::ScalarType::Bool),
            QuiverColumnType::DateTime
            | QuiverColumnType::Date
            | QuiverColumnType::Time
            | QuiverColumnType::Uuid
            | QuiverColumnType::Json
            | QuiverColumnType::Serial
            | QuiverColumnType::ForeignKey { .. } => {
                // These don't map directly to IR scalars, use String as fallback
                Type::Scalar(codegen_core::ir::ScalarType::String)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SchemaProvider implementation
// ---------------------------------------------------------------------------

impl SchemaProvider for QuiverSchema {
    fn messages(&self) -> Vec<MessageDef> {
        self.models
            .iter()
            .map(|model| MessageDef {
                name: model.name.clone(),
                fields: model
                    .fields
                    .iter()
                    .map(|f| FieldDef {
                        name: f.name.clone(),
                        ty: f.column_type.to_ir_type(),
                        is_optional: f.is_nullable && !f.is_primary_key,
                        default_value: f.default_value.clone(),
                        id: None, // Quiver doesn't use field IDs
                        comments: vec![],
                    })
                    .collect(),
                is_struct: false, // Quiver models are tables, not structs
                namespace: None,
                comments: vec![],
            })
            .collect()
    }

    fn services(&self) -> Vec<ServiceDef> {
        // Quiver-ORM doesn't have gRPC services
        vec![]
    }

    fn enums(&self) -> Vec<EnumDef> {
        self.enums
            .iter()
            .map(|e| EnumDef {
                name: e.name.clone(),
                values: e
                    .variants
                    .iter()
                    .map(|v| EnumValue {
                        name: v.name.clone(),
                        value: v.discriminant,
                        comments: vec![],
                    })
                    .collect(),
                is_union: false,
                is_struct: false,
                namespace: None,
                comments: vec![],
            })
            .collect()
    }

    fn file_ident(&self) -> Option<&str> {
        self.file_ident.as_deref()
    }

    fn root_table(&self) -> Option<&str> {
        // Return the first model's name as root table convention
        self.models.first().map(|m| m.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_schema() -> QuiverSchema {
        QuiverSchema {
            models: vec![
                QuiverModel {
                    name: "User".to_string(),
                    table_name: "users".to_string(),
                    fields: vec![
                        QuiverField {
                            name: "id".to_string(),
                            column_type: QuiverColumnType::Serial,
                            is_nullable: false,
                            is_primary_key: true,
                            default_value: None,
                            is_unique: true,
                        },
                        QuiverField {
                            name: "email".to_string(),
                            column_type: QuiverColumnType::String,
                            is_nullable: false,
                            is_primary_key: false,
                            default_value: None,
                            is_unique: true,
                        },
                        QuiverField {
                            name: "created_at".to_string(),
                            column_type: QuiverColumnType::DateTime,
                            is_nullable: false,
                            is_primary_key: false,
                            default_value: Some("NOW()".to_string()),
                            is_unique: false,
                        },
                    ],
                    primary_key: Some("id".to_string()),
                    indices: vec![],
                    foreign_keys: vec![],
                },
                QuiverModel {
                    name: "Post".to_string(),
                    table_name: "posts".to_string(),
                    fields: vec![
                        QuiverField {
                            name: "id".to_string(),
                            column_type: QuiverColumnType::Serial,
                            is_nullable: false,
                            is_primary_key: true,
                            default_value: None,
                            is_unique: true,
                        },
                        QuiverField {
                            name: "title".to_string(),
                            column_type: QuiverColumnType::String,
                            is_nullable: false,
                            is_primary_key: false,
                            default_value: None,
                            is_unique: false,
                        },
                        QuiverField {
                            name: "author_id".to_string(),
                            column_type: QuiverColumnType::ForeignKey {
                                model: "User".to_string(),
                                column: "id".to_string(),
                            },
                            is_nullable: false,
                            is_primary_key: false,
                            default_value: None,
                            is_unique: false,
                        },
                    ],
                    primary_key: Some("id".to_string()),
                    indices: vec![],
                    foreign_keys: vec![QuiverForeignKey {
                        name: "fk_author".to_string(),
                        columns: vec!["author_id".to_string()],
                        referenced_model: "User".to_string(),
                        referenced_columns: vec!["id".to_string()],
                        on_delete: "CASCADE".to_string(),
                        on_update: "CASCADE".to_string(),
                    }],
                },
            ],
            enums: vec![QuiverEnum {
                name: "UserRole".to_string(),
                variants: vec![
                    QuiverEnumVariant {
                        name: "Admin".to_string(),
                        discriminant: 0,
                        fields: vec![],
                    },
                    QuiverEnumVariant {
                        name: "User".to_string(),
                        discriminant: 1,
                        fields: vec![],
                    },
                    QuiverEnumVariant {
                        name: "Guest".to_string(),
                        discriminant: 2,
                        fields: vec![],
                    },
                ],
                is_exhaustive: true,
            }],
            file_ident: Some("quiver_schema".to_string()),
        }
    }

    #[test]
    fn test_messages() {
        let schema = make_test_schema();
        let messages = schema.messages();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].name, "User");
        assert_eq!(messages[1].name, "Post");
    }

    #[test]
    fn test_user_fields() {
        let schema = make_test_schema();
        let messages = schema.messages();
        let user = &messages[0];

        assert_eq!(user.fields.len(), 3);
        assert_eq!(user.fields[0].name, "id");
        assert!(matches!(
            user.fields[0].ty,
            Type::Scalar(codegen_core::ir::ScalarType::String)
        )); // Serial maps to String
        assert!(!user.fields[0].is_optional);
        assert!(user.fields[0].default_value.is_none());

        assert_eq!(user.fields[1].name, "email");
        assert!(matches!(
            user.fields[1].ty,
            Type::Scalar(codegen_core::ir::ScalarType::String)
        ));
        assert!(!user.fields[1].is_optional);

        assert_eq!(user.fields[2].name, "created_at");
        assert!(matches!(
            user.fields[2].ty,
            Type::Scalar(codegen_core::ir::ScalarType::String)
        )); // DateTime maps to String
        assert!(!user.fields[2].is_optional);
        assert_eq!(user.fields[2].default_value, Some("NOW()".to_string()));
    }

    #[test]
    fn test_enums() {
        let schema = make_test_schema();
        let enums = schema.enums();

        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "UserRole");
        assert_eq!(enums[0].values.len(), 3);
        assert_eq!(enums[0].values[0].name, "Admin");
        assert_eq!(enums[0].values[0].value, 0);
        assert_eq!(enums[0].values[1].name, "User");
        assert_eq!(enums[0].values[1].value, 1);
        assert_eq!(enums[0].values[2].name, "Guest");
        assert_eq!(enums[0].values[2].value, 2);
    }

    #[test]
    fn test_services_empty() {
        let schema = make_test_schema();
        let services = schema.services();
        assert!(services.is_empty());
    }

    #[test]
    fn test_file_ident() {
        let schema = make_test_schema();
        assert_eq!(schema.file_ident(), Some("quiver_schema"));
    }

    #[test]
    fn test_root_table() {
        let schema = make_test_schema();
        assert_eq!(schema.root_table(), Some("User"));
    }

    #[test]
    fn test_foreign_key_type() {
        let schema = make_test_schema();
        let messages = schema.messages();
        let post = messages.iter().find(|m| m.name == "Post").unwrap();
        let author_id = post.fields.iter().find(|f| f.name == "author_id").unwrap();

        // ForeignKey maps to String in IR
        assert!(matches!(
            author_id.ty,
            Type::Scalar(codegen_core::ir::ScalarType::String)
        ));
    }
}
