//! Quiver-ORM schema adapter for codegen-schema.
//!
//! Converts from Quiver-ORM schema types to the common schema definition.
//!
//! Quiver-ORM uses a code-first schema definition approach. This adapter expects
//! a simplified schema structure that can be derived from Quiver model definitions.
//!
//! **Note:** Quiver-ORM does not have an equivalent to FlatBuffers unions or
//! Protobuf oneofs. Quiver enums always have `is_union: false` because Quiver
//! does not support union types. `Type::OneOf` is not produced by this adapter.

use codegen_schema::{
    EnumDef, EnumValue, FieldDef, ForeignKey, MessageDef, ScalarType, SchemaDef, ServiceDef, Type,
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
    fn to_schema_type(&self) -> Result<Type, QuiverAdapterError> {
        match self {
            QuiverColumnType::Int32 => Ok(Type::Scalar(ScalarType::Int32)),
            QuiverColumnType::Int64 => Ok(Type::Scalar(ScalarType::Int64)),
            QuiverColumnType::UInt32 => Ok(Type::Scalar(ScalarType::Uint32)),
            QuiverColumnType::UInt64 => Ok(Type::Scalar(ScalarType::Uint64)),
            QuiverColumnType::Float32 => Ok(Type::Scalar(ScalarType::Float32)),
            QuiverColumnType::Float64 => Ok(Type::Scalar(ScalarType::Float64)),
            QuiverColumnType::String => Ok(Type::Scalar(ScalarType::String)),
            QuiverColumnType::Bytes => Ok(Type::Scalar(ScalarType::Bytes)),
            QuiverColumnType::Bool => Ok(Type::Scalar(ScalarType::Bool)),
            QuiverColumnType::DateTime
            | QuiverColumnType::Date
            | QuiverColumnType::Time
            | QuiverColumnType::Uuid
            | QuiverColumnType::Json => Ok(Type::Scalar(ScalarType::String)),
            QuiverColumnType::Serial => Ok(Type::Scalar(ScalarType::Int64)),
            QuiverColumnType::ForeignKey { model, column } => Ok(Type::ForeignKey(ForeignKey {
                referenced_table: model.clone(),
                referenced_column: column.clone(),
            })),
        }
    }
}

// ---------------------------------------------------------------------------
// QuiverSchema -> SchemaDef
// ---------------------------------------------------------------------------

/// Convert a QuiverSchema to a SchemaDef.
///
/// Failing fields are skipped with a warning rather than aborting the entire model.
/// This allows partial schema generation when some field types are unsupported.
pub fn from_quiver_schema(schema: &QuiverSchema) -> Result<SchemaDef, QuiverAdapterError> {
    let messages: Vec<MessageDef> = schema
        .models
        .iter()
        .map(|model| {
            let fields: Vec<FieldDef> = model
                .fields
                .iter()
                .filter_map(|f| {
                    match f.column_type.to_schema_type() {
                        Ok(ty) => Some(FieldDef {
                            name: f.name.clone(),
                            ty,
                            is_optional: f.is_nullable && !f.is_primary_key,
                            default_value: f.default_value.clone(),
                            id: None, // Quiver doesn't use field IDs
                            comments: vec![],
                        }),
                        Err(e) => {
                            eprintln!("warning: skipping field '{}.{}': {}", model.name, f.name, e);
                            None
                        }
                    }
                })
                .collect();
            Ok(MessageDef {
                name: model.name.clone(),
                fields,
                is_struct: false, // Quiver models are tables, not structs
                namespace: None,
                comments: vec![],
            })
        })
        .collect::<Result<Vec<MessageDef>, QuiverAdapterError>>()?;

    let enums: Vec<EnumDef> = schema
        .enums
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
            namespace: None,
            comments: vec![],
        })
        .collect();

    // Quiver-ORM doesn't have gRPC services
    let services: Vec<ServiceDef> = vec![];

    Ok(SchemaDef {
        name: schema.file_ident.clone().unwrap_or_default(),
        messages,
        enums,
        services,
        file_ident: schema.file_ident.clone(),
        root_table: schema.models.first().map(|m| m.name.clone()),
    })
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
                            column_type: QuiverColumnType::Int64,
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
                            column_type: QuiverColumnType::String,
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
                            column_type: QuiverColumnType::Int64,
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
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");

        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].name, "User");
        assert_eq!(result.messages[1].name, "Post");
    }

    #[test]
    fn test_user_fields() {
        let schema = make_test_schema();
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");
        let user = &result.messages[0];

        assert_eq!(user.fields.len(), 3);
        assert_eq!(user.fields[0].name, "id");
        assert!(matches!(user.fields[0].ty, Type::Scalar(ScalarType::Int64)));
        assert!(!user.fields[0].is_optional);
        assert!(user.fields[0].default_value.is_none());

        assert_eq!(user.fields[1].name, "email");
        assert!(matches!(
            user.fields[1].ty,
            Type::Scalar(ScalarType::String)
        ));
        assert!(!user.fields[1].is_optional);

        assert_eq!(user.fields[2].name, "created_at");
        assert!(matches!(
            user.fields[2].ty,
            Type::Scalar(ScalarType::String)
        ));
        assert!(!user.fields[2].is_optional);
        assert_eq!(user.fields[2].default_value, Some("NOW()".to_string()));
    }

    #[test]
    fn test_enums() {
        let schema = make_test_schema();
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");

        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].name, "UserRole");
        assert_eq!(result.enums[0].values.len(), 3);
        assert_eq!(result.enums[0].values[0].name, "Admin");
        assert_eq!(result.enums[0].values[0].value, 0);
        assert_eq!(result.enums[0].values[1].name, "User");
        assert_eq!(result.enums[0].values[1].value, 1);
        assert_eq!(result.enums[0].values[2].name, "Guest");
        assert_eq!(result.enums[0].values[2].value, 2);
    }

    #[test]
    fn test_services_empty() {
        let schema = make_test_schema();
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");
        assert!(result.services.is_empty());
    }

    #[test]
    fn test_file_ident() {
        let schema = make_test_schema();
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");
        assert_eq!(result.file_ident, Some("quiver_schema".to_string()));
    }

    #[test]
    fn test_root_table() {
        let schema = make_test_schema();
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");
        assert_eq!(result.root_table, Some("User".to_string()));
    }

    #[test]
    fn test_foreign_key_type() {
        let schema = make_test_schema();
        let result = from_quiver_schema(&schema).expect("schema conversion should succeed");
        let post = result.messages.iter().find(|m| m.name == "Post").unwrap();
        let author_id = post.fields.iter().find(|f| f.name == "author_id").unwrap();

        // ForeignKey maps to Type::ForeignKey with correct references
        assert!(matches!(
            &author_id.ty,
            Type::ForeignKey(ForeignKey {
                referenced_table,
                referenced_column
            }) if referenced_table == "User" && referenced_column == "id"
        ));
    }

    #[test]
    fn test_datetime_type_maps_to_string() {
        let schema = QuiverSchema {
            models: vec![QuiverModel {
                name: "Event".to_string(),
                table_name: "events".to_string(),
                fields: vec![QuiverField {
                    name: "created_at".to_string(),
                    column_type: QuiverColumnType::DateTime,
                    is_nullable: false,
                    is_primary_key: false,
                    default_value: None,
                    is_unique: false,
                }],
                primary_key: None,
                indices: vec![],
                foreign_keys: vec![],
            }],
            enums: vec![],
            file_ident: None,
        };
        let result = from_quiver_schema(&schema);
        assert!(result.is_ok());
        let schema_def = result.unwrap();
        let event = schema_def
            .messages
            .iter()
            .find(|m| m.name == "Event")
            .unwrap();
        let field = event
            .fields
            .iter()
            .find(|f| f.name == "created_at")
            .unwrap();
        assert!(matches!(field.ty, Type::Scalar(ScalarType::String)));
    }

    #[test]
    fn test_unsupported_serial_type_skips_field() {
        let schema = QuiverSchema {
            models: vec![QuiverModel {
                name: "Item".to_string(),
                table_name: "items".to_string(),
                fields: vec![QuiverField {
                    name: "id".to_string(),
                    column_type: QuiverColumnType::Serial,
                    is_nullable: false,
                    is_primary_key: true,
                    default_value: None,
                    is_unique: true,
                }],
                primary_key: Some("id".to_string()),
                indices: vec![],
                foreign_keys: vec![],
            }],
            enums: vec![],
            file_ident: None,
        };
        // Conversion succeeds and the Serial field is mapped to Int64
        let result = from_quiver_schema(&schema);
        assert!(result.is_ok());
        let schema_def = result.unwrap();
        let item = schema_def
            .messages
            .iter()
            .find(|m| m.name == "Item")
            .unwrap();
        // The Serial field should be mapped to Int64, so Item has one field
        assert_eq!(
            item.fields.len(),
            1,
            "Serial field should be mapped to Int64"
        );
        let field = item.fields.iter().find(|f| f.name == "id").unwrap();
        assert!(matches!(field.ty, Type::Scalar(ScalarType::Int64)));
    }
}
