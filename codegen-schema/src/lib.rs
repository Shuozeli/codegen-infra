//! Unified schema representation for code generation.
//!
//! This crate provides a format-agnostic representation of schemas
//! that can be adapted from FlatBuffers, Protobuf, Quiver, etc.

pub mod schema;
pub mod r#type;

pub use r#type::{ForeignKey, OneOfVariant, ScalarType, Type};
pub use schema::{
    EnumDef, EnumValue, FieldDef, MessageDef, MethodDef, SchemaDef, SchemaProvider, ServiceDef,
    StreamingType,
};

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // ScalarType Tests
    // =====================================================================

    #[test]
    fn scalar_type_all_variants_exist() {
        // Verify all scalar type variants can be constructed
        let _ = ScalarType::Bool;
        let _ = ScalarType::Int8;
        let _ = ScalarType::Uint8;
        let _ = ScalarType::Int16;
        let _ = ScalarType::Uint16;
        let _ = ScalarType::Int32;
        let _ = ScalarType::Uint32;
        let _ = ScalarType::Int64;
        let _ = ScalarType::Uint64;
        let _ = ScalarType::Float32;
        let _ = ScalarType::Float64;
        let _ = ScalarType::String;
        let _ = ScalarType::Bytes;
    }

    #[test]
    fn scalar_type_debug_formatting() {
        assert_eq!(format!("{:?}", ScalarType::Bool), "Bool");
        assert_eq!(format!("{:?}", ScalarType::Int32), "Int32");
        assert_eq!(format!("{:?}", ScalarType::String), "String");
        assert_eq!(format!("{:?}", ScalarType::Bytes), "Bytes");
    }

    #[test]
    fn scalar_type_display() {
        assert_eq!(ScalarType::Bool.to_string(), "bool");
        assert_eq!(ScalarType::Int8.to_string(), "int8");
        assert_eq!(ScalarType::Uint8.to_string(), "uint8");
        assert_eq!(ScalarType::Int16.to_string(), "int16");
        assert_eq!(ScalarType::Uint16.to_string(), "uint16");
        assert_eq!(ScalarType::Int32.to_string(), "int32");
        assert_eq!(ScalarType::Uint32.to_string(), "uint32");
        assert_eq!(ScalarType::Int64.to_string(), "int64");
        assert_eq!(ScalarType::Uint64.to_string(), "uint64");
        assert_eq!(ScalarType::Float32.to_string(), "float32");
        assert_eq!(ScalarType::Float64.to_string(), "float64");
        assert_eq!(ScalarType::String.to_string(), "String");
        assert_eq!(ScalarType::Bytes.to_string(), "bytes");
    }

    // =====================================================================
    // Type Tests
    // =====================================================================

    #[test]
    fn type_create_scalar_variant() {
        let ty = Type::Scalar(ScalarType::Int32);
        assert!(ty.is_scalar());
    }

    #[test]
    fn type_create_message_variant() {
        let ty = Type::Message {
            name: "MyMessage".to_string(),
            package: None,
        };
        assert!(!ty.is_scalar());
        assert!(ty.element_type().is_none());
    }

    #[test]
    fn type_create_message_with_package() {
        let ty = Type::Message {
            name: "MyMessage".to_string(),
            package: Some("my.package".to_string()),
        };
        assert!(!ty.is_scalar());
        let display = format!("{}", ty);
        assert_eq!(display, "MyMessage");
    }

    #[test]
    fn type_create_enum_variant() {
        let ty = Type::Enum {
            name: "Status".to_string(),
            package: None,
        };
        assert!(!ty.is_scalar());
        assert!(ty.element_type().is_none());
    }

    #[test]
    fn type_create_enum_with_package() {
        let ty = Type::Enum {
            name: "Status".to_string(),
            package: Some("my.package".to_string()),
        };
        assert!(!ty.is_scalar());
    }

    #[test]
    fn type_create_vector() {
        let inner = Type::Scalar(ScalarType::Int32);
        let ty = Type::Vector(Box::new(inner));
        assert!(!ty.is_scalar());
        assert!(ty.element_type().is_some());
        if let Some(inner) = ty.element_type() {
            assert!(matches!(inner, Type::Scalar(ScalarType::Int32)));
        }
    }

    #[test]
    fn type_create_optional() {
        let inner = Type::Scalar(ScalarType::String);
        let ty = Type::Optional(Box::new(inner));
        assert!(!ty.is_scalar());
        assert!(ty.element_type().is_some());
        assert!(matches!(
            ty.element_type(),
            Some(Type::Scalar(ScalarType::String))
        ));
    }

    #[test]
    fn type_create_oneof() {
        let variants = vec![
            OneOfVariant {
                name: "int_val".to_string(),
                ty: Type::Scalar(ScalarType::Int32),
            },
            OneOfVariant {
                name: "str_val".to_string(),
                ty: Type::Scalar(ScalarType::String),
            },
        ];
        let ty = Type::OneOf {
            name: "choice".to_string(),
            variants,
        };
        assert!(!ty.is_scalar());
        assert!(ty.element_type().is_none());
    }

    #[test]
    fn type_is_scalar_returns_true_for_scalar() {
        let ty = Type::Scalar(ScalarType::Bool);
        assert!(ty.is_scalar());
    }

    #[test]
    fn type_is_scalar_returns_false_for_message() {
        let ty = Type::Message {
            name: "MyMessage".to_string(),
            package: None,
        };
        assert!(!ty.is_scalar());
    }

    #[test]
    fn type_is_scalar_returns_false_for_vector() {
        let ty = Type::Vector(Box::new(Type::Scalar(ScalarType::Int32)));
        assert!(!ty.is_scalar());
    }

    #[test]
    fn type_element_type_returns_some_for_vector() {
        let ty = Type::Vector(Box::new(Type::Scalar(ScalarType::Int64)));
        assert!(ty.element_type().is_some());
        if let Some(inner) = ty.element_type() {
            assert!(matches!(inner, Type::Scalar(ScalarType::Int64)));
        }
    }

    #[test]
    fn type_element_type_returns_none_for_scalar() {
        let ty = Type::Scalar(ScalarType::Int32);
        assert!(ty.element_type().is_none());
    }

    #[test]
    fn type_element_type_returns_none_for_message() {
        let ty = Type::Message {
            name: "MyMessage".to_string(),
            package: None,
        };
        assert!(ty.element_type().is_none());
    }

    // =====================================================================
    // SchemaDef Tests
    // =====================================================================

    #[test]
    fn schema_def_empty_is_valid() {
        let schema = SchemaDef {
            name: "EmptySchema".to_string(),
            messages: vec![],
            enums: vec![],
            services: vec![],
            file_ident: None,
            root_table: None,
        };
        assert!(schema.name == "EmptySchema");
        assert!(schema.messages.is_empty());
        assert!(schema.enums.is_empty());
        assert!(schema.services.is_empty());
    }

    #[test]
    fn schema_def_with_messages() {
        let schema = SchemaDef {
            name: "TestSchema".to_string(),
            messages: vec![MessageDef {
                name: "MyMessage".to_string(),
                fields: vec![],
                is_struct: false,
                namespace: None,
                comments: vec![],
            }],
            enums: vec![],
            services: vec![],
            file_ident: None,
            root_table: None,
        };
        assert_eq!(schema.messages.len(), 1);
        assert_eq!(schema.messages[0].name, "MyMessage");
    }

    #[test]
    fn schema_def_with_enums() {
        let schema = SchemaDef {
            name: "TestSchema".to_string(),
            messages: vec![],
            enums: vec![EnumDef {
                name: "Status".to_string(),
                values: vec![EnumValue {
                    name: "Active".to_string(),
                    value: 0,
                    comments: vec![],
                }],
                is_union: false,
                namespace: None,
                comments: vec![],
            }],
            services: vec![],
            file_ident: None,
            root_table: None,
        };
        assert_eq!(schema.enums.len(), 1);
        assert_eq!(schema.enums[0].name, "Status");
    }

    #[test]
    fn schema_def_with_services() {
        let schema = SchemaDef {
            name: "TestSchema".to_string(),
            messages: vec![],
            enums: vec![],
            services: vec![ServiceDef {
                name: "MyService".to_string(),
                methods: vec![],
                package: None,
                comments: vec![],
            }],
            file_ident: None,
            root_table: None,
        };
        assert_eq!(schema.services.len(), 1);
        assert_eq!(schema.services[0].name, "MyService");
    }

    #[test]
    fn schema_def_with_all_components() {
        let message = MessageDef {
            name: "Request".to_string(),
            fields: vec![],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        let enum_def = EnumDef {
            name: "Kind".to_string(),
            values: vec![EnumValue {
                name: "A".to_string(),
                value: 0,
                comments: vec![],
            }],
            is_union: false,
            namespace: None,
            comments: vec![],
        };
        let service = ServiceDef {
            name: "MyService".to_string(),
            methods: vec![],
            package: Some("my.package".to_string()),
            comments: vec![],
        };
        let schema = SchemaDef {
            name: "FullSchema".to_string(),
            messages: vec![message],
            enums: vec![enum_def],
            services: vec![service],
            file_ident: Some("file_id".to_string()),
            root_table: Some("Request".to_string()),
        };
        assert_eq!(schema.name, "FullSchema");
        assert_eq!(schema.messages.len(), 1);
        assert_eq!(schema.enums.len(), 1);
        assert_eq!(schema.services.len(), 1);
        assert_eq!(schema.file_ident, Some("file_id".to_string()));
        assert_eq!(schema.root_table, Some("Request".to_string()));
    }

    // =====================================================================
    // MessageDef Tests
    // =====================================================================

    #[test]
    fn message_def_create_with_fields() {
        let field = FieldDef {
            name: "id".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: false,
            default_value: None,
            id: Some(1),
            comments: vec![],
        };
        let message = MessageDef {
            name: "MyMessage".to_string(),
            fields: vec![field],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        assert_eq!(message.name, "MyMessage");
        assert_eq!(message.fields.len(), 1);
        assert!(!message.is_struct);
    }

    #[test]
    fn message_def_is_struct_flag() {
        let struct_message = MessageDef {
            name: "MyStruct".to_string(),
            fields: vec![],
            is_struct: true,
            namespace: None,
            comments: vec![],
        };
        let table_message = MessageDef {
            name: "MyTable".to_string(),
            fields: vec![],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        assert!(struct_message.is_struct);
        assert!(!table_message.is_struct);
    }

    #[test]
    fn message_def_validate_no_errors() {
        let message = MessageDef {
            name: "ValidMessage".to_string(),
            fields: vec![FieldDef {
                name: "field1".to_string(),
                ty: Type::Scalar(ScalarType::Int32),
                is_optional: false,
                default_value: None,
                id: None,
                comments: vec![],
            }],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        let errors = message.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn message_def_validate_empty_name() {
        let message = MessageDef {
            name: "".to_string(),
            fields: vec![],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        let errors = message.validate();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("empty"));
    }

    #[test]
    fn message_def_validate_duplicate_fields() {
        let message = MessageDef {
            name: "DuplicateFields".to_string(),
            fields: vec![
                FieldDef {
                    name: "same_name".to_string(),
                    ty: Type::Scalar(ScalarType::Int32),
                    is_optional: false,
                    default_value: None,
                    id: None,
                    comments: vec![],
                },
                FieldDef {
                    name: "same_name".to_string(),
                    ty: Type::Scalar(ScalarType::String),
                    is_optional: true,
                    default_value: None,
                    id: None,
                    comments: vec![],
                },
            ],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        let errors = message.validate();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("duplicate"));
    }

    #[test]
    fn message_def_field_lookup() {
        let message = MessageDef {
            name: "MyMessage".to_string(),
            fields: vec![
                FieldDef {
                    name: "id".to_string(),
                    ty: Type::Scalar(ScalarType::Int32),
                    is_optional: false,
                    default_value: None,
                    id: None,
                    comments: vec![],
                },
                FieldDef {
                    name: "name".to_string(),
                    ty: Type::Scalar(ScalarType::String),
                    is_optional: true,
                    default_value: None,
                    id: None,
                    comments: vec![],
                },
            ],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };
        assert!(message.field("id").is_some());
        assert!(message.field("name").is_some());
        assert!(message.field("nonexistent").is_none());
    }

    // =====================================================================
    // FieldDef Tests
    // =====================================================================

    #[test]
    fn field_def_create_with_type() {
        let field = FieldDef {
            name: "count".to_string(),
            ty: Type::Scalar(ScalarType::Uint64),
            is_optional: false,
            default_value: None,
            id: None,
            comments: vec![],
        };
        assert_eq!(field.name, "count");
        assert!(matches!(field.ty, Type::Scalar(ScalarType::Uint64)));
        assert!(!field.is_optional);
    }

    #[test]
    fn field_def_is_optional_flag() {
        let required = FieldDef {
            name: "required".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: false,
            default_value: None,
            id: None,
            comments: vec![],
        };
        let optional = FieldDef {
            name: "optional".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: true,
            default_value: None,
            id: None,
            comments: vec![],
        };
        assert!(!required.is_optional);
        assert!(optional.is_optional);
    }

    #[test]
    fn field_def_default_value() {
        let no_default = FieldDef {
            name: "no_default".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: false,
            default_value: None,
            id: None,
            comments: vec![],
        };
        let with_default = FieldDef {
            name: "with_default".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: false,
            default_value: Some("42".to_string()),
            id: None,
            comments: vec![],
        };
        assert_eq!(no_default.default_value, None);
        assert_eq!(with_default.default_value, Some("42".to_string()));
    }

    #[test]
    fn field_def_type_name_non_optional() {
        let field = FieldDef {
            name: "count".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: false,
            default_value: None,
            id: None,
            comments: vec![],
        };
        assert_eq!(field.type_name(), "int32");
    }

    #[test]
    fn field_def_type_name_optional() {
        let field = FieldDef {
            name: "count".to_string(),
            ty: Type::Scalar(ScalarType::Int32),
            is_optional: true,
            default_value: None,
            id: None,
            comments: vec![],
        };
        assert_eq!(field.type_name(), "Option<int32>");
    }

    #[test]
    fn field_def_type_name_complex() {
        let field = FieldDef {
            name: "items".to_string(),
            ty: Type::Vector(Box::new(Type::Scalar(ScalarType::String))),
            is_optional: false,
            default_value: None,
            id: None,
            comments: vec![],
        };
        assert_eq!(field.type_name(), "Vec<String>");
    }

    // =====================================================================
    // EnumDef Tests
    // =====================================================================

    #[test]
    fn enum_def_create_with_values() {
        let enum_def = EnumDef {
            name: "Color".to_string(),
            values: vec![
                EnumValue {
                    name: "Red".to_string(),
                    value: 0,
                    comments: vec![],
                },
                EnumValue {
                    name: "Green".to_string(),
                    value: 1,
                    comments: vec![],
                },
                EnumValue {
                    name: "Blue".to_string(),
                    value: 2,
                    comments: vec![],
                },
            ],
            is_union: false,
            namespace: None,
            comments: vec![],
        };
        assert_eq!(enum_def.name, "Color");
        assert_eq!(enum_def.values.len(), 3);
        assert!(!enum_def.is_union);
    }

    #[test]
    fn enum_def_is_union_flag() {
        let regular_enum = EnumDef {
            name: "Regular".to_string(),
            values: vec![],
            is_union: false,
            namespace: None,
            comments: vec![],
        };
        let union_enum = EnumDef {
            name: "Union".to_string(),
            values: vec![],
            is_union: true,
            namespace: None,
            comments: vec![],
        };
        assert!(!regular_enum.is_union);
        assert!(union_enum.is_union);
    }

    // =====================================================================
    // EnumValue Tests
    // =====================================================================

    #[test]
    fn enum_value_create_with_name_and_value() {
        let value = EnumValue {
            name: "Active".to_string(),
            value: 1,
            comments: vec![],
        };
        assert_eq!(value.name, "Active");
        assert_eq!(value.value, 1);
        assert!(value.comments.is_empty());
    }

    #[test]
    fn enum_value_create_with_comments() {
        let value = EnumValue {
            name: "Active".to_string(),
            value: 1,
            comments: vec!["Indicates active status".to_string()],
        };
        assert_eq!(value.name, "Active");
        assert_eq!(value.value, 1);
        assert_eq!(value.comments.len(), 1);
        assert_eq!(value.comments[0], "Indicates active status");
    }

    #[test]
    fn enum_value_negative_value() {
        let value = EnumValue {
            name: "Negative".to_string(),
            value: -10,
            comments: vec![],
        };
        assert_eq!(value.value, -10);
    }

    // =====================================================================
    // ServiceDef Tests
    // =====================================================================

    #[test]
    fn service_def_create_with_methods() {
        let method = MethodDef {
            name: "GetItem".to_string(),
            rust_name: None,
            input_type: "GetItemRequest".to_string(),
            output_type: "GetItemResponse".to_string(),
            streaming: StreamingType::None,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        };
        let service = ServiceDef {
            name: "MyService".to_string(),
            methods: vec![method],
            package: None,
            comments: vec![],
        };
        assert_eq!(service.name, "MyService");
        assert_eq!(service.methods.len(), 1);
        assert!(service.package.is_none());
    }

    #[test]
    fn service_def_with_package() {
        let service = ServiceDef {
            name: "MyService".to_string(),
            methods: vec![],
            package: Some("my.package".to_string()),
            comments: vec![],
        };
        assert_eq!(service.package, Some("my.package".to_string()));
    }

    #[test]
    fn service_def_with_comments() {
        let service = ServiceDef {
            name: "MyService".to_string(),
            methods: vec![],
            package: None,
            comments: vec![
                "This is a service".to_string(),
                "With multiple comments".to_string(),
            ],
        };
        assert_eq!(service.comments.len(), 2);
    }

    // =====================================================================
    // MethodDef Tests
    // =====================================================================

    #[test]
    fn method_def_create_with_input_output_types() {
        let method = MethodDef {
            name: "GetItem".to_string(),
            rust_name: None,
            input_type: "GetItemRequest".to_string(),
            output_type: "GetItemResponse".to_string(),
            streaming: StreamingType::None,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        };
        assert_eq!(method.name, "GetItem");
        assert_eq!(method.input_type, "GetItemRequest");
        assert_eq!(method.output_type, "GetItemResponse");
    }

    #[test]
    fn method_def_streaming_type_none() {
        let method = MethodDef {
            name: "Unary".to_string(),
            rust_name: None,
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            streaming: StreamingType::None,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        };
        assert_eq!(method.streaming, StreamingType::None);
        assert_eq!(method.streaming_mode(), "unary");
    }

    #[test]
    fn method_def_streaming_type_server() {
        let method = MethodDef {
            name: "ServerStream".to_string(),
            rust_name: None,
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            streaming: StreamingType::Server,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        };
        assert_eq!(method.streaming, StreamingType::Server);
        assert_eq!(method.streaming_mode(), "server streaming");
    }

    #[test]
    fn method_def_streaming_type_client() {
        let method = MethodDef {
            name: "ClientStream".to_string(),
            rust_name: None,
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            streaming: StreamingType::Client,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        };
        assert_eq!(method.streaming, StreamingType::Client);
        assert_eq!(method.streaming_mode(), "client streaming");
    }

    #[test]
    fn method_def_streaming_type_bidi() {
        let method = MethodDef {
            name: "BiDi".to_string(),
            rust_name: None,
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            streaming: StreamingType::BiDi,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        };
        assert_eq!(method.streaming, StreamingType::BiDi);
        assert_eq!(method.streaming_mode(), "bidi streaming");
    }

    // =====================================================================
    // StreamingType Tests
    // =====================================================================

    #[test]
    fn streaming_type_from_bool_tuple_none() {
        let st = StreamingType::from((false, false));
        assert_eq!(st, StreamingType::None);
    }

    #[test]
    fn streaming_type_from_bool_tuple_server() {
        let st = StreamingType::from((false, true));
        assert_eq!(st, StreamingType::Server);
    }

    #[test]
    fn streaming_type_from_bool_tuple_client() {
        let st = StreamingType::from((true, false));
        assert_eq!(st, StreamingType::Client);
    }

    #[test]
    fn streaming_type_from_bool_tuple_bidi() {
        let st = StreamingType::from((true, true));
        assert_eq!(st, StreamingType::BiDi);
    }

    #[test]
    fn streaming_type_debug() {
        assert_eq!(format!("{:?}", StreamingType::None), "None");
        assert_eq!(format!("{:?}", StreamingType::Server), "Server");
        assert_eq!(format!("{:?}", StreamingType::Client), "Client");
        assert_eq!(format!("{:?}", StreamingType::BiDi), "BiDi");
    }

    #[test]
    fn streaming_type_copy() {
        let st = StreamingType::Server;
        let _ = st; // Should not cause warning about unused
        let _st2 = st;
        let _st3 = st;
    }

    #[test]
    fn streaming_type_partial_eq() {
        assert_eq!(StreamingType::None, StreamingType::None);
        assert_eq!(StreamingType::Server, StreamingType::Server);
        assert_ne!(StreamingType::None, StreamingType::Server);
    }
}
