//! Dart code writer.

use super::{to_camel_case, to_pascal_case, CodeWriter, CodeWriterError};
use codegen_core::{escape_reserved_word, Language};
use codegen_schema::{EnumDef, MessageDef, ScalarType, ServiceDef, StreamingType, Type};

/// Code writer for Dart.
#[derive(Default)]
pub struct DartCodeWriter;

impl DartCodeWriter {
    pub fn new() -> Self {
        Self
    }

    fn write_field_type(ty: &Type) -> String {
        match ty {
            Type::Scalar(scalar) => match scalar {
                ScalarType::Bool => "bool".to_string(),
                ScalarType::Int8
                | ScalarType::Int16
                | ScalarType::Int32
                | ScalarType::Uint8
                | ScalarType::Uint16
                | ScalarType::Uint32
                | ScalarType::Uint64
                | ScalarType::Int64 => "int".to_string(),
                ScalarType::Float32 | ScalarType::Float64 => "double".to_string(),
                ScalarType::String => "String".to_string(),
                ScalarType::Bytes => "Uint8List".to_string(),
            },
            Type::Message { name, .. } => {
                escape_reserved_word(&to_pascal_case(name), Language::Dart)
            }
            Type::Enum { name, .. } => escape_reserved_word(&to_pascal_case(name), Language::Dart),
            Type::Vector(inner) => {
                format!("List<{}>", Self::write_field_type(inner))
            }
            Type::Optional(inner) => format!("{}?", Self::write_field_type(inner)),
            Type::Map { key, value } => {
                format!(
                    "Map<{}, {}>",
                    Self::write_field_type(key),
                    Self::write_field_type(value)
                )
            }
            Type::OneOf { name, .. } => escape_reserved_word(&to_pascal_case(name), Language::Dart),
            Type::ForeignKey(_) => "int".to_string(), // Foreign keys are typically integer IDs
        }
    }

    fn write_class(&self, msg: &MessageDef) -> String {
        let mut output = String::new();
        // Case conversion first, then escape if reserved
        let class_name = escape_reserved_word(&to_pascal_case(&msg.name), Language::Dart);

        // Write struct indicator if applicable
        if msg.is_struct {
            output.push_str("/// FlatBuffers struct\n");
        }

        // Write doc comments
        for comment in &msg.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        output.push_str(&format!("class {} {{\n", class_name));

        // Write fields
        for field in &msg.fields {
            for comment in &field.comments {
                output.push_str(&format!("  /// {}\n", comment));
            }
            let ty_str = Self::write_field_type(&field.ty);
            // Case conversion first, then escape if reserved
            let field_name = escape_reserved_word(&to_camel_case(&field.name), Language::Dart);
            if field.is_optional {
                output.push_str(&format!("  final {}? {};\n", ty_str, field_name));
            } else {
                output.push_str(&format!("  final {} {};\n", ty_str, field_name));
            }
        }

        output.push_str("}\n");
        output
    }

    fn write_enum_impl(&self, enm: &EnumDef) -> String {
        let mut output = String::new();
        // Case conversion first, then escape if reserved
        let enum_name = escape_reserved_word(&to_pascal_case(&enm.name), Language::Dart);

        // Write doc comments
        for comment in &enm.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        output.push_str(&format!("enum {} {{\n", enum_name));

        // Write values
        for (i, value) in enm.values.iter().enumerate() {
            for comment in &value.comments {
                output.push_str(&format!("  /// {}\n", comment));
            }
            let suffix = if i < enm.values.len() - 1 { "," } else { "" };
            // Case conversion first, then escape if reserved
            let variant_name = escape_reserved_word(&to_pascal_case(&value.name), Language::Dart);
            output.push_str(&format!("  {}{}\n", variant_name, suffix));
        }

        output.push_str("}\n");
        output
    }

    fn write_service_impl(&self, svc: &ServiceDef) -> String {
        let mut output = String::new();
        let class_name = escape_reserved_word(&to_pascal_case(&svc.name), Language::Dart);

        // Derive the client class name from the package
        let client_class = svc
            .package
            .as_ref()
            .map(|p| to_pascal_case(p))
            .unwrap_or_else(|| "GrpcClient".to_string())
            + "Client";

        // Write doc comments
        for comment in &svc.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        output.push_str(&format!("class {} {{\n", class_name));
        output.push_str(&format!("  final {} client;\n", client_class));
        output.push_str(&format!("  {}(this.client);\n\n", class_name));

        // Write methods
        for method in &svc.methods {
            let request_type = Self::write_field_type(&Type::Message {
                name: method.input_type.clone(),
                package: None,
            });
            let response_type = Self::write_field_type(&Type::Message {
                name: method.output_type.clone(),
                package: None,
            });
            let method_name = escape_reserved_word(&to_camel_case(&method.name), Language::Dart);

            match method.streaming {
                StreamingType::None => {
                    // Unary: Future<Response> method(Request request) async
                    output.push_str(&format!(
                        "  Future<{}> {}({} request) async {{\n",
                        response_type, method_name, request_type
                    ));
                    output.push_str("    throw UnimplementedError();\n");
                    output.push_str("  }\n");
                }
                StreamingType::Server => {
                    // Server streaming: Stream<Response> method(Request request)
                    output.push_str(&format!(
                        "  Stream<{}> {}({} request) {{\n",
                        response_type, method_name, request_type
                    ));
                    output.push_str("    throw UnimplementedError();\n");
                    output.push_str("  }\n");
                }
                StreamingType::Client => {
                    // Client streaming: Future<Response> method(Stream<Request> request) async
                    output.push_str(&format!(
                        "  Future<{}> {}(Stream<{}> request) async {{\n",
                        response_type, method_name, request_type
                    ));
                    output.push_str("    throw UnimplementedError();\n");
                    output.push_str("  }\n");
                }
                StreamingType::BiDi => {
                    // Bidirectional streaming: Stream<Response> method(Stream<Request> request)
                    output.push_str(&format!(
                        "  Stream<{}> {}(Stream<{}> request) {{\n",
                        response_type, method_name, request_type
                    ));
                    output.push_str("    throw UnimplementedError();\n");
                    output.push_str("  }\n");
                }
            }
        }

        output.push_str("}\n");
        output
    }
}

impl CodeWriter for DartCodeWriter {
    fn write_message(&mut self, msg: &MessageDef) -> Result<String, CodeWriterError> {
        Ok(self.write_class(msg))
    }

    fn write_enum(&mut self, enm: &EnumDef) -> Result<String, CodeWriterError> {
        Ok(self.write_enum_impl(enm))
    }

    fn write_service(&mut self, svc: &ServiceDef) -> Result<String, CodeWriterError> {
        Ok(self.write_service_impl(svc))
    }

    fn write_file(
        &mut self,
        messages: &[MessageDef],
        enums: &[EnumDef],
        services: &[ServiceDef],
    ) -> Result<String, CodeWriterError> {
        let mut output = String::new();

        output.push_str("// Generated by codegen-infra\n");
        output.push_str("import 'package:protobuf/protobuf.dart';\n");
        output.push_str("import 'package:grpc/grpc.dart';\n");

        // Add package-specific imports for services
        let packages: std::collections::HashSet<_> =
            services.iter().filter_map(|s| s.package.as_ref()).collect();
        for pkg in packages {
            output.push_str(&format!("import 'package:{}.pbrpc.dart';\n", pkg));
        }
        output.push('\n');

        // Write enums first
        for enm in enums {
            output.push_str(&self.write_enum_impl(enm));
            output.push('\n');
        }

        // Write messages
        for msg in messages {
            output.push_str(&self.write_class(msg));
            output.push('\n');
        }

        // Write services
        for svc in services {
            output.push_str(&self.write_service_impl(svc));
            output.push('\n');
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codegen_schema::{EnumValue, FieldDef, ScalarType, Type};

    #[test]
    fn test_write_simple_class() {
        let writer = DartCodeWriter::new();
        let msg = MessageDef {
            name: "person".to_string(),
            fields: vec![FieldDef {
                name: "name".to_string(),
                ty: Type::Scalar(ScalarType::String),
                is_optional: true,
                default_value: None,
                id: Some(1),
                comments: vec![],
            }],
            is_struct: false,
            namespace: None,
            comments: vec!["A person object".to_string()],
        };

        let output = writer.write_class(&msg);
        assert!(output.contains("class Person"));
        assert!(output.contains("final String? name"));
    }

    #[test]
    fn test_write_enum() {
        let mut writer = DartCodeWriter::new();
        let enm = EnumDef {
            name: "color".to_string(),
            values: vec![
                EnumValue {
                    name: "red".to_string(),
                    value: 0,
                    comments: vec![],
                },
                EnumValue {
                    name: "green".to_string(),
                    value: 1,
                    comments: vec![],
                },
            ],
            is_union: false,
            namespace: None,
            comments: vec![],
        };

        let output = writer.write_enum(&enm).unwrap();
        assert!(output.contains("enum Color"));
        assert!(output.contains("Red"));
        assert!(output.contains("Green"));
    }

    #[test]
    fn test_field_type_mapping() {
        assert_eq!(
            DartCodeWriter::write_field_type(&Type::Scalar(ScalarType::Int32)),
            "int"
        );
        assert_eq!(
            DartCodeWriter::write_field_type(&Type::Scalar(ScalarType::String)),
            "String"
        );
        assert_eq!(
            DartCodeWriter::write_field_type(&Type::Vector(Box::new(Type::Scalar(
                ScalarType::String
            )))),
            "List<String>"
        );
    }

    #[test]
    fn test_reserved_word_field_escaped() {
        // Field named "class" (reserved in Dart) should be escaped to "class_"
        let writer = DartCodeWriter::new();
        let msg = MessageDef {
            name: "Data".to_string(),
            fields: vec![FieldDef {
                name: "class".to_string(),
                ty: Type::Scalar(ScalarType::String),
                is_optional: false,
                default_value: None,
                id: Some(1),
                comments: vec![],
            }],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };

        let output = writer.write_class(&msg);
        // Field name should be escaped to class_ (with underscore suffix)
        assert!(output.contains("final String class_;"));
        // Ensure the unescaped version doesn't appear as a complete field name
        // Note: "final String class" is a substring of "final String class_", so we check for the full line
        assert!(output.contains("  final String class_;\n"));
    }

    #[test]
    fn test_reserved_word_type_escaped() {
        // Type named "class" (reserved in Dart) should be escaped to "class_"
        // Note: "class" is a reserved word in Dart; we escape to "class_"
        let writer = DartCodeWriter::new();
        let msg = MessageDef {
            name: "class".to_string(),
            fields: vec![FieldDef {
                name: "value".to_string(),
                ty: Type::Scalar(ScalarType::Int32),
                is_optional: false,
                default_value: None,
                id: Some(1),
                comments: vec![],
            }],
            is_struct: false,
            namespace: None,
            comments: vec![],
        };

        let output = writer.write_class(&msg);
        // "class" is reserved in Dart:
        // - to_pascal_case("class") = "Class"
        // - escape_reserved_word("Class", Dart) -> "Class_" (since "class" is reserved)
        assert!(output.contains("class Class_ {"));
        assert!(!output.contains("class Class {")); // The unescaped version
    }

    #[test]
    fn test_reserved_word_enum_escaped() {
        // Enum named "enum" should be escaped to "Enum_"
        let mut writer = DartCodeWriter::new();
        let enm = EnumDef {
            name: "enum".to_string(),
            values: vec![EnumValue {
                name: "A".to_string(),
                value: 0,
                comments: vec![],
            }],
            is_union: false,
            namespace: None,
            comments: vec![],
        };

        let output = writer.write_enum(&enm).unwrap();
        // "enum" is reserved in Dart:
        // - to_pascal_case("enum") = "Enum"
        // - escape_reserved_word("Enum", Dart) -> "Enum_" (since "enum" is reserved)
        assert!(output.contains("enum Enum_ {"));
        assert!(!output.contains("enum Enum {")); // The unescaped version
    }
}
