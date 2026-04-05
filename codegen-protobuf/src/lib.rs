//! Protobuf schema adapter for codegen-core IR.
//!
//! Converts from protobuf `FileDescriptorProto` to the common
//! intermediate representation types.

use codegen_core::ir::{
    EnumDef, EnumValue, FieldDef, MessageDef, MethodDef, SchemaProvider, ServiceDef, Type,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ProtobufAdapterError {
    #[error("unsupported field type: {0}")]
    UnsupportedFieldType(i32),

    #[error("type not found: {0}")]
    TypeNotFound(String),
}

// ---------------------------------------------------------------------------
// Proto type mapping
// ---------------------------------------------------------------------------

/// Map protobuf field type to IR scalar type.
fn proto_type_to_scalar(type_str: &str) -> Option<codegen_core::ir::ScalarType> {
    match type_str {
        // Integer types
        "int32" | "sint32" | "sfixed32" => Some(codegen_core::ir::ScalarType::Int32),
        "int64" | "sint64" | "sfixed64" => Some(codegen_core::ir::ScalarType::Int64),
        "uint32" | "fixed32" => Some(codegen_core::ir::ScalarType::Uint32),
        "uint64" | "fixed64" => Some(codegen_core::ir::ScalarType::Uint64),

        // Floating point
        "float" => Some(codegen_core::ir::ScalarType::Float32),
        "double" => Some(codegen_core::ir::ScalarType::Float64),

        // Other
        "bool" => Some(codegen_core::ir::ScalarType::Bool),
        "string" => Some(codegen_core::ir::ScalarType::String),
        "bytes" => Some(codegen_core::ir::ScalarType::Bytes),

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Newtype wrapper to work around orphan rule
// ---------------------------------------------------------------------------

/// Wrapper to implement SchemaProvider for FileDescriptorProto.
pub struct ProtobufSchema<'a> {
    pub file: &'a prost_types::FileDescriptorProto,
}

impl<'a> ProtobufSchema<'a> {
    /// Create a new ProtobufSchema from a FileDescriptorProto.
    pub fn new(file: &'a prost_types::FileDescriptorProto) -> Self {
        ProtobufSchema { file }
    }

    /// Resolve a protobuf type name to IR Type.
    fn resolve_type_name(&self, type_name: &str) -> Type {
        // Check if it's a well-known type
        if let Some(scalar) = proto_type_to_scalar(type_name) {
            return Type::Scalar(scalar);
        }

        // It's a message or enum type
        let trimmed = type_name.trim_start_matches('.');
        if let Some((pkg, name)) = trimmed.rsplit_once('.') {
            // Has package prefix
            Type::Message {
                name: name.to_string(),
                package: Some(pkg.to_string()),
            }
        } else {
            // Local type - check if it's an enum
            let is_enum = self
                .file
                .enum_type
                .iter()
                .any(|e| e.name.as_deref() == Some(trimmed));

            if is_enum {
                Type::Enum {
                    name: trimmed.to_string(),
                    package: None,
                }
            } else {
                Type::Message {
                    name: trimmed.to_string(),
                    package: None,
                }
            }
        }
    }

    /// Convert a protobuf field type to IR Type.
    fn convert_field_type(&self, field: &prost_types::FieldDescriptorProto) -> Type {
        // If type_name is set, it's a named type (message or enum)
        if let Some(ref type_name_bytes) = field.type_name {
            let type_str = String::from_utf8_lossy(type_name_bytes.as_bytes());
            return self.resolve_type_name(&type_str);
        }

        // Otherwise, it's a scalar type
        let type_i32 = field.r#type.unwrap_or(0);
        let type_str = match prost_types::field_descriptor_proto::Type::try_from(type_i32) {
            Ok(prost_types::field_descriptor_proto::Type::Double) => "double",
            Ok(prost_types::field_descriptor_proto::Type::Float) => "float",
            Ok(prost_types::field_descriptor_proto::Type::Int64) => "int64",
            Ok(prost_types::field_descriptor_proto::Type::Uint64) => "uint64",
            Ok(prost_types::field_descriptor_proto::Type::Int32) => "int32",
            Ok(prost_types::field_descriptor_proto::Type::Fixed64) => "fixed64",
            Ok(prost_types::field_descriptor_proto::Type::Fixed32) => "fixed32",
            Ok(prost_types::field_descriptor_proto::Type::Bool) => "bool",
            Ok(prost_types::field_descriptor_proto::Type::String) => "string",
            Ok(prost_types::field_descriptor_proto::Type::Bytes) => "bytes",
            Ok(prost_types::field_descriptor_proto::Type::Uint32) => "uint32",
            Ok(prost_types::field_descriptor_proto::Type::Sfixed32) => "sfixed32",
            Ok(prost_types::field_descriptor_proto::Type::Sfixed64) => "sfixed64",
            Ok(prost_types::field_descriptor_proto::Type::Sint32) => "sint32",
            Ok(prost_types::field_descriptor_proto::Type::Sint64) => "sint64",
            _ => "bytes",
        };

        proto_type_to_scalar(type_str)
            .map(Type::Scalar)
            .unwrap_or(Type::Scalar(codegen_core::ir::ScalarType::Bytes))
    }

    /// Convert a protobuf field to IR FieldDef.
    fn convert_field(&self, field: &prost_types::FieldDescriptorProto) -> FieldDef {
        let ty = self.convert_field_type(field);

        // Handle repeated fields (they become vectors)
        let is_repeated =
            field.label == Some(prost_types::field_descriptor_proto::Label::Repeated as i32);
        let ty = if is_repeated {
            Type::Vector(Box::new(ty))
        } else {
            ty
        };

        let is_optional =
            field.label == Some(prost_types::field_descriptor_proto::Label::Optional as i32);

        FieldDef {
            name: field.name.clone().unwrap_or_default(),
            ty,
            is_optional,
            default_value: field.default_value.clone(),
            id: field.number.map(|n| n as u32),
            comments: vec![],
        }
    }
}

impl<'a> SchemaProvider for ProtobufSchema<'a> {
    fn messages(&self) -> Vec<MessageDef> {
        let package = self.file.package.clone().unwrap_or_default();

        self.file
            .message_type
            .iter()
            .filter(|m| m.options.as_ref().is_none_or(|o| o.map_entry != Some(true)))
            .filter_map(|msg| {
                let name = msg.name.clone()?;
                let namespace = package.clone();

                Some(MessageDef {
                    name,
                    fields: msg.field.iter().map(|f| self.convert_field(f)).collect(),
                    is_struct: false,
                    namespace: Some(namespace),
                    comments: vec![],
                })
            })
            .collect()
    }

    fn services(&self) -> Vec<ServiceDef> {
        let package = self.file.package.clone().unwrap_or_default();

        self.file
            .service
            .iter()
            .filter_map(|svc| {
                let svc_name = svc.name.clone()?;

                Some(ServiceDef {
                    name: svc_name.clone(),
                    package: package.clone(),
                    proto_name: svc_name,
                    methods: svc
                        .method
                        .iter()
                        .filter_map(|m| {
                            let method_name = m.name.clone()?;
                            let input_type = m
                                .input_type
                                .as_ref()
                                .and_then(|t| {
                                    String::from_utf8_lossy(t.as_bytes())
                                        .trim_start_matches('.')
                                        .rsplit_once('.')
                                        .map(|(_, n)| n.to_string())
                                })
                                .unwrap_or_else(|| "Unknown".to_string());
                            let output_type = m
                                .output_type
                                .as_ref()
                                .and_then(|t| {
                                    String::from_utf8_lossy(t.as_bytes())
                                        .trim_start_matches('.')
                                        .rsplit_once('.')
                                        .map(|(_, n)| n.to_string())
                                })
                                .unwrap_or_else(|| "Unknown".to_string());

                            Some(MethodDef {
                                name: to_snake_case(&method_name),
                                proto_name: method_name,
                                input_type,
                                output_type,
                                client_streaming: m.client_streaming.unwrap_or(false),
                                server_streaming: m.server_streaming.unwrap_or(false),
                                codec_path: "crate::codec::Codec".to_string(),
                                comments: vec![],
                            })
                        })
                        .collect(),
                    comments: vec![],
                })
            })
            .collect()
    }

    fn enums(&self) -> Vec<EnumDef> {
        let package = self.file.package.clone().unwrap_or_default();

        self.file
            .enum_type
            .iter()
            .filter_map(|e| {
                let name = e.name.clone()?;
                Some(EnumDef {
                    name,
                    values: e
                        .value
                        .iter()
                        .filter_map(|v| {
                            Some(EnumValue {
                                name: v.name.clone()?,
                                value: v.number.unwrap_or(0) as i64,
                                comments: vec![],
                            })
                        })
                        .collect(),
                    is_union: false,
                    is_struct: false,
                    namespace: Some(package.clone()),
                    comments: vec![],
                })
            })
            .collect()
    }

    fn file_ident(&self) -> Option<&str> {
        self.file.name.as_deref()
    }

    fn root_table(&self) -> Option<&str> {
        self.file
            .message_type
            .first()
            .and_then(|m| m.name.as_deref())
    }
}

/// Convert a name to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_proto() -> prost_types::FileDescriptorProto {
        use prost_types::field_descriptor_proto::Label;

        prost_types::FileDescriptorProto {
            name: Some("test.proto".to_string()),
            package: Some("test".to_string()),
            message_type: vec![prost_types::DescriptorProto {
                name: Some("Person".to_string()),
                field: vec![
                    prost_types::FieldDescriptorProto {
                        name: Some("id".to_string()),
                        number: Some(1),
                        label: Some(Label::Optional as i32),
                        r#type: Some(prost_types::field_descriptor_proto::Type::Int32 as i32),
                        ..Default::default()
                    },
                    prost_types::FieldDescriptorProto {
                        name: Some("name".to_string()),
                        number: Some(2),
                        label: Some(Label::Optional as i32),
                        r#type: Some(prost_types::field_descriptor_proto::Type::String as i32),
                        ..Default::default()
                    },
                    prost_types::FieldDescriptorProto {
                        name: Some("emails".to_string()),
                        number: Some(3),
                        label: Some(Label::Repeated as i32),
                        r#type: Some(prost_types::field_descriptor_proto::Type::String as i32),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            enum_type: vec![prost_types::EnumDescriptorProto {
                name: Some("PhoneType".to_string()),
                value: vec![
                    prost_types::EnumValueDescriptorProto {
                        name: Some("MOBILE".to_string()),
                        number: Some(0),
                        ..Default::default()
                    },
                    prost_types::EnumValueDescriptorProto {
                        name: Some("HOME".to_string()),
                        number: Some(1),
                        ..Default::default()
                    },
                    prost_types::EnumValueDescriptorProto {
                        name: Some("WORK".to_string()),
                        number: Some(2),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            service: vec![prost_types::ServiceDescriptorProto {
                name: Some("TestService".to_string()),
                method: vec![prost_types::MethodDescriptorProto {
                    name: Some("GetPerson".to_string()),
                    input_type: Some(".test.Person".to_string()),
                    output_type: Some(".test.Person".to_string()),
                    client_streaming: Some(false),
                    server_streaming: Some(false),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_messages() {
        let proto = make_test_proto();
        let schema = ProtobufSchema::new(&proto);

        let messages = schema.messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].name, "Person");
        assert_eq!(messages[0].fields.len(), 3);
    }

    #[test]
    fn test_enums() {
        let proto = make_test_proto();
        let schema = ProtobufSchema::new(&proto);

        let enums = schema.enums();
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].name, "PhoneType");
        assert_eq!(enums[0].values.len(), 3);
    }

    #[test]
    fn test_services() {
        let proto = make_test_proto();
        let schema = ProtobufSchema::new(&proto);

        let services = schema.services();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "TestService");
        assert_eq!(services[0].methods.len(), 1);
        assert_eq!(services[0].methods[0].name, "get_person");
    }

    #[test]
    fn test_field_types() {
        let proto = make_test_proto();
        let schema = ProtobufSchema::new(&proto);

        let messages = schema.messages();
        let person = &messages[0];

        // id is int32
        assert!(matches!(
            person.fields[0].ty,
            Type::Scalar(codegen_core::ir::ScalarType::Int32)
        ));

        // name is string
        assert!(matches!(
            person.fields[1].ty,
            Type::Scalar(codegen_core::ir::ScalarType::String)
        ));

        // emails is repeated string -> Vector<String>
        assert!(matches!(
            &person.fields[2].ty,
            Type::Vector(inner) if matches!(inner.as_ref(), Type::Scalar(codegen_core::ir::ScalarType::String))
        ));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("GetPerson"), "get_person");
        assert_eq!(to_snake_case("CreateUser"), "create_user");
        assert_eq!(to_snake_case("simple"), "simple");
    }
}
