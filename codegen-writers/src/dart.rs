//! Dart code writer.

use super::{to_camel_case, to_pascal_case, CodeWriter, CodeWriterError};
use codegen_core::ir::{EnumDef, MessageDef, ServiceDef, Type};

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
                codegen_core::ir::ScalarType::Bool => "bool".to_string(),
                codegen_core::ir::ScalarType::Int8
                | codegen_core::ir::ScalarType::Int16
                | codegen_core::ir::ScalarType::Int32
                | codegen_core::ir::ScalarType::Uint8
                | codegen_core::ir::ScalarType::Uint16
                | codegen_core::ir::ScalarType::Uint32
                | codegen_core::ir::ScalarType::Uint64
                | codegen_core::ir::ScalarType::Int64 => "int".to_string(),
                codegen_core::ir::ScalarType::Float32 | codegen_core::ir::ScalarType::Float64 => {
                    "double".to_string()
                }
                codegen_core::ir::ScalarType::String => "String".to_string(),
                codegen_core::ir::ScalarType::Bytes => "Uint8List".to_string(),
                _ => "Opaque".to_string(),
            },
            Type::Message { name, .. } => to_pascal_case(name),
            Type::Enum { name, .. } => to_pascal_case(name),
            Type::Vector(inner) => {
                format!("List<{}>", Self::write_field_type(inner))
            }
            Type::InlineStruct(inner) => Self::write_field_type(inner),
        }
    }

    fn write_class(&self, msg: &MessageDef) -> String {
        let mut output = String::new();
        let class_name = to_pascal_case(&msg.name);

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
            let field_name = to_camel_case(&field.name);
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
        let enum_name = to_pascal_case(&enm.name);

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
            output.push_str(&format!("  {}{}\n", to_pascal_case(&value.name), suffix));
        }

        output.push_str("}\n");
        output
    }

    fn write_service_impl(&self, svc: &ServiceDef) -> String {
        let mut output = String::new();
        let class_name = to_pascal_case(&svc.name);

        // Write doc comments
        for comment in &svc.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        output.push_str(&format!("class {} {{\n", class_name));
        output.push_str("  final pbGrpc.GrpcClient client;\n");
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

            match (method.client_streaming, method.server_streaming) {
                (false, false) => {
                    // Unary: Future<Response> method(Request request) async
                    output.push_str(&format!(
                        "  Future<{}> {}({} request) async {{\n",
                        response_type, method.name, request_type
                    ));
                    output.push_str("    // TODO: implement\n");
                    output.push_str("  }\n");
                }
                (false, true) => {
                    // Server streaming: Stream<Response> method(Request request)
                    output.push_str(&format!(
                        "  Stream<{}> {}({} request) {{\n",
                        response_type, method.name, request_type
                    ));
                    output.push_str("    // TODO: implement server streaming\n");
                    output.push_str("  }\n");
                }
                (true, false) => {
                    // Client streaming: Future<Response> method(Stream<Request> request) async
                    output.push_str(&format!(
                        "  Future<{}> {}(Stream<{}> request) async {{\n",
                        response_type, method.name, request_type
                    ));
                    output.push_str("    // TODO: implement client streaming\n");
                    output.push_str("  }\n");
                }
                (true, true) => {
                    // Bidirectional streaming: Stream<Response> method(Stream<Request> request)
                    output.push_str(&format!(
                        "  Stream<{}> {}(Stream<{}> request) {{\n",
                        response_type, method.name, request_type
                    ));
                    output.push_str("    // TODO: implement bidi streaming\n");
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
        output.push_str("import 'package:grpc/grpc.dart';\n\n");

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
    use codegen_core::ir::{FieldDef, ScalarType, Type};

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
                codegen_core::ir::EnumValue {
                    name: "red".to_string(),
                    value: 0,
                    comments: vec![],
                },
                codegen_core::ir::EnumValue {
                    name: "green".to_string(),
                    value: 1,
                    comments: vec![],
                },
            ],
            is_union: false,
            is_struct: false,
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
}
