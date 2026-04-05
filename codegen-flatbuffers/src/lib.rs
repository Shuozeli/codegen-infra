//! FlatBuffers schema adapter for codegen-core IR.
//!
//! Converts from [`flatc_rs_schema::ResolvedSchema`] to the common
//! intermediate representation types.

use codegen_core::ir::{
    EnumDef, EnumValue, FieldDef, MessageDef, MethodDef, SchemaProvider, ServiceDef, Type,
};

use flatc_rs_schema::resolved::{ResolvedSchema, ResolvedType};
use flatc_rs_schema::BaseType;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum FlatbuffersAdapterError {
    #[error("unsupported base type: {0:?}")]
    UnsupportedBaseType(BaseType),

    #[error("object index out of bounds: {0}")]
    ObjectIndexOutOfBounds(usize),
}

// ---------------------------------------------------------------------------
// BaseType -> ScalarType
// ---------------------------------------------------------------------------

fn base_type_to_scalar(bt: BaseType) -> Option<codegen_core::ir::ScalarType> {
    match bt {
        BaseType::BASE_TYPE_BOOL => Some(codegen_core::ir::ScalarType::Bool),
        BaseType::BASE_TYPE_BYTE => Some(codegen_core::ir::ScalarType::Int8),
        BaseType::BASE_TYPE_U_BYTE => Some(codegen_core::ir::ScalarType::Uint8),
        BaseType::BASE_TYPE_SHORT => Some(codegen_core::ir::ScalarType::Int16),
        BaseType::BASE_TYPE_U_SHORT => Some(codegen_core::ir::ScalarType::Uint16),
        BaseType::BASE_TYPE_INT => Some(codegen_core::ir::ScalarType::Int32),
        BaseType::BASE_TYPE_U_INT => Some(codegen_core::ir::ScalarType::Uint32),
        BaseType::BASE_TYPE_LONG => Some(codegen_core::ir::ScalarType::Int64),
        BaseType::BASE_TYPE_U_LONG => Some(codegen_core::ir::ScalarType::Uint64),
        BaseType::BASE_TYPE_FLOAT => Some(codegen_core::ir::ScalarType::Float32),
        BaseType::BASE_TYPE_DOUBLE => Some(codegen_core::ir::ScalarType::Float64),
        // BASE_TYPE_STRING is NOT a scalar in FlatBuffers (it's an offset type)
        // but we handle it here for convenience
        BaseType::BASE_TYPE_STRING => Some(codegen_core::ir::ScalarType::String),
        // These are handled elsewhere
        BaseType::BASE_TYPE_VECTOR | BaseType::BASE_TYPE_VECTOR64 | BaseType::BASE_TYPE_ARRAY => {
            None
        }
        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_STRUCT => None,
        BaseType::BASE_TYPE_UNION => None,
        BaseType::BASE_TYPE_NONE | BaseType::BASE_TYPE_U_TYPE => None,
    }
}

// ---------------------------------------------------------------------------
// ResolvedType -> IR Type (with schema access for type resolution)
// ---------------------------------------------------------------------------

fn convert_type(
    rt: &ResolvedType,
    schema: &ResolvedSchema,
) -> Result<Type, FlatbuffersAdapterError> {
    use flatc_rs_schema::BaseType as B;

    // Handle scalar types
    if rt.base_type.is_scalar() {
        if let Some(scalar) = base_type_to_scalar(rt.base_type) {
            return Ok(Type::Scalar(scalar));
        }
    }

    // Handle string (not a scalar in FlatBuffers but we treat it as one)
    if rt.base_type == B::BASE_TYPE_STRING {
        return Ok(Type::Scalar(codegen_core::ir::ScalarType::String));
    }

    // Handle vectors
    if matches!(rt.base_type, B::BASE_TYPE_VECTOR | B::BASE_TYPE_VECTOR64) {
        let element_type = rt.element_type.unwrap_or(B::BASE_TYPE_NONE);
        let element = convert_type(
            &ResolvedType {
                base_type: element_type,
                base_size: rt.base_size,
                element_size: rt.element_size,
                element_type: None,
                index: rt.index,
                fixed_length: rt.fixed_length,
            },
            schema,
        )?;
        return Ok(Type::Vector(Box::new(element)));
    }

    // Handle arrays
    if matches!(rt.base_type, B::BASE_TYPE_ARRAY) {
        let element_type = rt.element_type.unwrap_or(B::BASE_TYPE_NONE);
        let element = convert_type(
            &ResolvedType {
                base_type: element_type,
                base_size: rt.base_size,
                element_size: rt.element_size,
                element_type: None,
                index: rt.index,
                fixed_length: rt.fixed_length,
            },
            schema,
        )?;
        return Ok(Type::Vector(Box::new(element)));
    }

    // Handle tables/structs by index - resolve actual type name
    if let Some(idx) = rt.index {
        let idx_usize = usize::try_from(idx)
            .map_err(|_| FlatbuffersAdapterError::ObjectIndexOutOfBounds(idx as usize))?;

        if idx_usize >= schema.objects.len() {
            return Err(FlatbuffersAdapterError::ObjectIndexOutOfBounds(idx_usize));
        }

        let obj = &schema.objects[idx_usize];
        let namespace = obj.namespace.as_ref().and_then(|n| n.namespace.clone());

        return Ok(Type::Message {
            name: obj.name.clone(),
            package: namespace,
        });
    }

    Err(FlatbuffersAdapterError::UnsupportedBaseType(rt.base_type))
}

// ---------------------------------------------------------------------------
// Newtype wrapper to work around orphan rule
// ---------------------------------------------------------------------------

/// Wrapper to implement SchemaProvider for ResolvedSchema.
pub struct FlatbuffersSchema<'a>(pub &'a ResolvedSchema);

impl<'a> SchemaProvider for FlatbuffersSchema<'a> {
    fn messages(&self) -> Vec<MessageDef> {
        let schema = self.0;
        // Only return non-struct objects (tables), not structs
        schema
            .objects
            .iter()
            .filter(|o| !o.is_struct)
            .map(|o| {
                let namespace = o.namespace.as_ref().and_then(|n| n.namespace.clone());
                let comments = o
                    .documentation
                    .as_ref()
                    .map(|d| d.lines.clone())
                    .unwrap_or_default();

                MessageDef {
                    name: o.name.clone(),
                    fields: o
                        .fields
                        .iter()
                        .map(|f| {
                            // Log conversion errors but use Int32 as fallback to avoid
                            // crashing on malformed schemas. In production, you might
                            // want to collect these errors instead.
                            let ty = convert_type(&f.type_, schema).unwrap_or_else(|e| {
                                eprintln!(
                                    "warning: failed to convert type for field '{}': {}",
                                    f.name, e
                                );
                                Type::Scalar(codegen_core::ir::ScalarType::Int32)
                            });
                            let default_value = f
                                .default_string
                                .clone()
                                .or_else(|| f.default_integer.map(|i| i.to_string()));
                            let comments = f
                                .documentation
                                .as_ref()
                                .map(|d| d.lines.clone())
                                .unwrap_or_default();

                            FieldDef {
                                name: f.name.clone(),
                                ty,
                                is_optional: f.is_optional,
                                default_value,
                                id: f.id,
                                comments,
                            }
                        })
                        .collect(),
                    is_struct: false,
                    namespace,
                    comments,
                }
            })
            .collect()
    }

    fn services(&self) -> Vec<ServiceDef> {
        let schema = self.0;
        schema
            .services
            .iter()
            .map(|s| {
                let comments = s
                    .documentation
                    .as_ref()
                    .map(|d| d.lines.clone())
                    .unwrap_or_default();
                let namespace = s.namespace.as_ref().and_then(|n| n.namespace.clone());

                ServiceDef {
                    name: s.name.clone(),
                    package: namespace.clone().unwrap_or_default(),
                    proto_name: s.name.clone(),
                    methods: s
                        .calls
                        .iter()
                        .map(|c| {
                            let request_idx = c.request_index;
                            let response_idx = c.response_index;

                            let (input_type, output_type) = if request_idx < schema.objects.len()
                                && response_idx < schema.objects.len()
                            {
                                let req = &schema.objects[request_idx];
                                let res = &schema.objects[response_idx];
                                (req.name.clone(), res.name.clone())
                            } else {
                                ("UnknownRequest".to_string(), "UnknownResponse".to_string())
                            };

                            MethodDef {
                                name: c.name.clone(),
                                proto_name: c.name.clone(),
                                input_type,
                                output_type,
                                client_streaming: false,
                                server_streaming: false,
                                codec_path: "crate::codec::Codec".to_string(),
                                comments: c
                                    .documentation
                                    .as_ref()
                                    .map(|d| d.lines.clone())
                                    .unwrap_or_default(),
                            }
                        })
                        .collect(),
                    comments,
                }
            })
            .collect()
    }

    fn enums(&self) -> Vec<EnumDef> {
        let schema = self.0;
        schema
            .enums
            .iter()
            .map(|e| {
                let namespace = e.namespace.as_ref().and_then(|n| n.namespace.clone());

                EnumDef {
                    name: e.name.clone(),
                    values: e
                        .values
                        .iter()
                        .map(|v| EnumValue {
                            name: v.name.clone(),
                            value: v.value,
                            comments: v
                                .documentation
                                .as_ref()
                                .map(|d| d.lines.clone())
                                .unwrap_or_default(),
                        })
                        .collect(),
                    is_union: e.is_union,
                    is_struct: false,
                    namespace,
                    comments: e
                        .documentation
                        .as_ref()
                        .map(|d| d.lines.clone())
                        .unwrap_or_default(),
                }
            })
            .collect()
    }

    fn file_ident(&self) -> Option<&str> {
        self.0.file_ident.as_deref()
    }

    fn root_table(&self) -> Option<&str> {
        self.0
            .root_table_index
            .and_then(|idx| self.0.objects.get(idx).map(|o| o.name.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatc_rs_schema::resolved::{
        ResolvedEnum, ResolvedEnumVal, ResolvedField, ResolvedObject, ResolvedType as RT,
    };
    use flatc_rs_schema::BaseType as B;
    use flatc_rs_schema::Namespace;

    fn make_test_schema() -> ResolvedSchema {
        let monster_fields = vec![
            ResolvedField {
                name: "hp".to_string(),
                type_: RT {
                    base_type: B::BASE_TYPE_SHORT,
                    base_size: None,
                    element_size: None,
                    element_type: None,
                    index: None,
                    fixed_length: None,
                },
                id: Some(0),
                offset: Some(4),
                default_integer: Some(100),
                default_real: None,
                default_string: None,
                is_deprecated: false,
                is_required: false,
                is_key: false,
                is_optional: false,
                attributes: None,
                documentation: Some(flatc_rs_schema::Documentation {
                    lines: vec!["Monster HP".to_string()],
                }),
                padding: None,
                is_offset_64: false,
                span: None,
            },
            ResolvedField {
                name: "name".to_string(),
                type_: RT {
                    base_type: B::BASE_TYPE_STRING,
                    base_size: None,
                    element_size: None,
                    element_type: None,
                    index: None,
                    fixed_length: None,
                },
                id: Some(1),
                offset: Some(6),
                default_integer: None,
                default_real: None,
                default_string: None,
                is_deprecated: false,
                is_required: false,
                is_key: false,
                is_optional: true,
                attributes: None,
                documentation: None,
                padding: None,
                is_offset_64: false,
                span: None,
            },
        ];

        let weapon_fields = vec![ResolvedField {
            name: "damage".to_string(),
            type_: RT {
                base_type: B::BASE_TYPE_SHORT,
                base_size: None,
                element_size: None,
                element_type: None,
                index: None,
                fixed_length: None,
            },
            id: Some(0),
            offset: Some(4),
            default_integer: None,
            default_real: None,
            default_string: None,
            is_deprecated: false,
            is_required: false,
            is_key: false,
            is_optional: false,
            attributes: None,
            documentation: None,
            padding: None,
            is_offset_64: false,
            span: None,
        }];

        let color_enum = ResolvedEnum {
            name: "Color".to_string(),
            values: vec![
                ResolvedEnumVal {
                    name: "Red".to_string(),
                    value: 0,
                    union_type: None,
                    documentation: None,
                    attributes: None,
                    span: None,
                },
                ResolvedEnumVal {
                    name: "Green".to_string(),
                    value: 1,
                    union_type: None,
                    documentation: None,
                    attributes: None,
                    span: None,
                },
            ],
            is_union: false,
            underlying_type: RT {
                base_type: B::BASE_TYPE_BYTE,
                base_size: None,
                element_size: None,
                element_type: None,
                index: None,
                fixed_length: None,
            },
            attributes: None,
            documentation: None,
            declaration_file: None,
            namespace: Some(Namespace {
                namespace: Some("MyGame".to_string()),
            }),
            span: None,
        };

        ResolvedSchema {
            objects: vec![
                ResolvedObject {
                    name: "Monster".to_string(),
                    fields: monster_fields,
                    is_struct: false,
                    min_align: Some(2),
                    byte_size: Some(10),
                    attributes: None,
                    documentation: None,
                    declaration_file: None,
                    namespace: Some(Namespace {
                        namespace: Some("MyGame".to_string()),
                    }),
                    span: None,
                },
                ResolvedObject {
                    name: "Weapon".to_string(),
                    fields: weapon_fields,
                    is_struct: true,
                    min_align: Some(2),
                    byte_size: Some(4),
                    attributes: None,
                    documentation: None,
                    declaration_file: None,
                    namespace: Some(Namespace {
                        namespace: Some("MyGame".to_string()),
                    }),
                    span: None,
                },
            ],
            enums: vec![color_enum],
            file_ident: Some("MyGame".to_string()),
            file_ext: Some("bin".to_string()),
            root_table_index: Some(0),
            services: vec![],
            advanced_features: flatc_rs_schema::AdvancedFeatures::default(),
            fbs_files: vec![],
        }
    }

    #[test]
    fn test_messages_excludes_structs() {
        let schema = make_test_schema();
        let provider = FlatbuffersSchema(&schema);

        let messages = provider.messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].name, "Monster");
    }

    #[test]
    fn test_messages_contain_correct_fields() {
        let schema = make_test_schema();
        let provider = FlatbuffersSchema(&schema);

        let messages = provider.messages();
        let monster = &messages[0];

        assert_eq!(monster.name, "Monster");
        assert_eq!(monster.fields.len(), 2);
        assert_eq!(monster.fields[0].name, "hp");
        assert_eq!(monster.fields[1].name, "name");
        assert!(monster.fields[1].is_optional); // name is optional in our test
    }

    #[test]
    fn test_enums() {
        let schema = make_test_schema();
        let provider = FlatbuffersSchema(&schema);

        let enums = provider.enums();
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "Color");
        assert_eq!(enums[0].values.len(), 2);
        assert_eq!(enums[0].values[0].name, "Red");
        assert_eq!(enums[0].values[1].name, "Green");
    }

    #[test]
    fn test_file_ident() {
        let schema = make_test_schema();
        let provider = FlatbuffersSchema(&schema);

        assert_eq!(provider.file_ident(), Some("MyGame"));
    }

    #[test]
    fn test_root_table() {
        let schema = make_test_schema();
        let provider = FlatbuffersSchema(&schema);

        assert_eq!(provider.root_table(), Some("Monster"));
    }

    #[test]
    fn test_convert_scalar_type() {
        let schema = make_test_schema();
        let rt = RT {
            base_type: B::BASE_TYPE_INT,
            base_size: None,
            element_size: None,
            element_type: None,
            index: None,
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            Type::Scalar(codegen_core::ir::ScalarType::Int32)
        ));
    }

    #[test]
    fn test_convert_string_type() {
        let schema = make_test_schema();
        let rt = RT {
            base_type: B::BASE_TYPE_STRING,
            base_size: None,
            element_size: None,
            element_type: None,
            index: None,
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            Type::Scalar(codegen_core::ir::ScalarType::String)
        ));
    }

    #[test]
    fn test_convert_message_type() {
        let schema = make_test_schema();
        // Reference to Weapon (index 1)
        let rt = RT {
            base_type: B::BASE_TYPE_TABLE,
            base_size: None,
            element_size: None,
            element_type: None,
            index: Some(1),
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Message { name, package } => {
                assert_eq!(name, "Weapon");
                assert_eq!(package, Some("MyGame".to_string()));
            }
            _ => panic!("Expected Message type"),
        }
    }

    #[test]
    fn test_convert_vector_type() {
        let schema = make_test_schema();
        // Vector of strings
        let rt = RT {
            base_type: B::BASE_TYPE_VECTOR,
            base_size: None,
            element_size: None,
            element_type: Some(B::BASE_TYPE_STRING),
            index: None,
            fixed_length: None,
        };

        let result = convert_type(&rt, &schema);
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Vector(inner) => {
                assert!(matches!(
                    *inner,
                    Type::Scalar(codegen_core::ir::ScalarType::String)
                ));
            }
            _ => panic!("Expected Vector type"),
        }
    }
}
