//! TypeScript code writer.

use super::{to_pascal_case, CodeWriter, CodeWriterError};
use codegen_core::ir::{EnumDef, MessageDef, ServiceDef, Type};

/// Code writer for TypeScript.
#[derive(Default)]
pub struct TypeScriptCodeWriter;

impl TypeScriptCodeWriter {
    pub fn new() -> Self {
        Self
    }

    fn write_field_type(ty: &Type) -> String {
        match ty {
            Type::Scalar(scalar) => match scalar {
                codegen_core::ir::ScalarType::Bool => "boolean".to_string(),
                codegen_core::ir::ScalarType::Int8
                | codegen_core::ir::ScalarType::Int16
                | codegen_core::ir::ScalarType::Int32
                | codegen_core::ir::ScalarType::Uint8
                | codegen_core::ir::ScalarType::Uint16
                | codegen_core::ir::ScalarType::Uint32
                | codegen_core::ir::ScalarType::Float32
                | codegen_core::ir::ScalarType::Float64 => "number".to_string(),
                codegen_core::ir::ScalarType::Int64 => "bigint".to_string(),
                codegen_core::ir::ScalarType::Uint64 => "bigint".to_string(),
                codegen_core::ir::ScalarType::String => "string".to_string(),
                codegen_core::ir::ScalarType::Bytes => "Uint8Array".to_string(),
                _ => "unknown".to_string(),
            },
            Type::Message { name, .. } => to_pascal_case(name),
            Type::Enum { name, .. } => to_pascal_case(name),
            Type::Vector(inner) => format!("{}[]", Self::write_field_type(inner)),
            Type::InlineStruct(inner) => Self::write_field_type(inner),
        }
    }

    fn write_interface(&self, msg: &MessageDef) -> String {
        let mut output = String::new();

        // Write namespace as comment if present
        if let Some(ns) = &msg.namespace {
            output.push_str(&format!("// Namespace: {}\n", ns));
        }

        // Write doc comments
        for comment in &msg.comments {
            output.push_str(&format!("/** {} */\n", comment));
        }

        output.push_str(&format!(
            "export interface {} {{\n",
            to_pascal_case(&msg.name)
        ));

        // Write fields
        for field in &msg.fields {
            for comment in &field.comments {
                output.push_str(&format!("  /** {} */\n", comment));
            }
            let ty_str = Self::write_field_type(&field.ty);
            let optional = if field.is_optional { "?" } else { "" };
            output.push_str(&format!("  {}{}: {};\n", field.name, optional, ty_str));
        }

        output.push_str("}\n");
        output
    }

    fn write_enum_impl(&self, enm: &EnumDef) -> String {
        let mut output = String::new();

        // Write namespace as comment if present
        if let Some(ns) = &enm.namespace {
            output.push_str(&format!("// Namespace: {}\n", ns));
        }

        // Write doc comments
        for comment in &enm.comments {
            output.push_str(&format!("/** {} */\n", comment));
        }

        output.push_str(&format!("export enum {} {{\n", to_pascal_case(&enm.name)));

        // Write values
        for (i, value) in enm.values.iter().enumerate() {
            let suffix = if i < enm.values.len() - 1 { "," } else { "" };
            output.push_str(&format!(
                "  {} = {}{}\n",
                to_pascal_case(&value.name),
                value.value,
                suffix
            ));
        }

        output.push_str("}\n");
        output
    }

    fn write_service_impl(&self, svc: &ServiceDef) -> String {
        let mut output = String::new();

        // Write package as comment if present
        if !svc.package.is_empty() {
            output.push_str(&format!("// Package: {}\n", svc.package));
        }

        // Write doc comments
        for comment in &svc.comments {
            output.push_str(&format!("/** {} */\n", comment));
        }

        output.push_str(&format!("export class {} {{\n", to_pascal_case(&svc.name)));
        output.push_str("  private client: any;\n\n");
        output.push_str("  constructor(client: any) {\n");
        output.push_str("    this.client = client;\n");
        output.push_str("  }\n\n");

        // Write methods
        for method in &svc.methods {
            let request_type = to_pascal_case(&method.input_type);
            let response_type = to_pascal_case(&method.output_type);
            let method_name = to_pascal_case(&method.name);

            match (method.client_streaming, method.server_streaming) {
                (false, false) => {
                    // Unary: async method(request: Type): Promise<Response>
                    output.push_str(&format!(
                        "  async {}(request: {}): Promise<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output.push_str("    return this.client.invoke(this.path, request);\n");
                    output.push_str("  }\n");
                }
                (false, true) => {
                    // Server streaming: async *method(request: Type): AsyncGenerator<Response>
                    output.push_str(&format!(
                        "  async *{}(request: {}): AsyncGenerator<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output
                        .push_str("    yield* this.client.serverStreaming(this.path, request);\n");
                    output.push_str("  }\n");
                }
                (true, false) => {
                    // Client streaming: async method(request: AsyncGenerator<Type>): Promise<Response>
                    output.push_str(&format!(
                        "  async {}(request: AsyncGenerator<{}>): Promise<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output
                        .push_str("    return this.client.clientStreaming(this.path, request);\n");
                    output.push_str("  }\n");
                }
                (true, true) => {
                    // Bidirectional streaming: async *method(request: AsyncGenerator<Type>): AsyncGenerator<Response>
                    output.push_str(&format!(
                        "  async *{}(request: AsyncGenerator<{}>): AsyncGenerator<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output.push_str("    yield* this.client.bidiStreaming(this.path, request);\n");
                    output.push_str("  }\n");
                }
            }
        }

        output.push_str("}\n");
        output
    }
}

impl CodeWriter for TypeScriptCodeWriter {
    fn write_message(&mut self, msg: &MessageDef) -> Result<String, CodeWriterError> {
        Ok(self.write_interface(msg))
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

        output.push_str("// Generated by codegen-infra\n\n");

        // Write enums first
        for enm in enums {
            output.push_str(&self.write_enum_impl(enm));
            output.push('\n');
        }

        // Write messages
        for msg in messages {
            output.push_str(&self.write_interface(msg));
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
    fn test_write_simple_interface() {
        let writer = TypeScriptCodeWriter::new();
        let msg = MessageDef {
            name: "person".to_string(),
            fields: vec![FieldDef {
                name: "name".to_string(),
                ty: Type::Scalar(ScalarType::String),
                is_optional: false,
                default_value: None,
                id: Some(1),
                comments: vec![],
            }],
            is_struct: false,
            namespace: None,
            comments: vec!["A person object".to_string()],
        };

        let output = writer.write_interface(&msg);
        assert!(output.contains("export interface Person"));
        assert!(output.contains("name: string"));
    }

    #[test]
    fn test_write_enum() {
        let mut writer = TypeScriptCodeWriter::new();
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
        assert!(output.contains("export enum Color"));
        assert!(output.contains("Red = 0"));
        assert!(output.contains("Green = 1"));
    }

    #[test]
    fn test_field_type_mapping() {
        assert_eq!(
            TypeScriptCodeWriter::write_field_type(&Type::Scalar(ScalarType::Int32)),
            "number"
        );
        assert_eq!(
            TypeScriptCodeWriter::write_field_type(&Type::Scalar(ScalarType::String)),
            "string"
        );
        assert_eq!(
            TypeScriptCodeWriter::write_field_type(&Type::Vector(Box::new(Type::Scalar(
                ScalarType::String
            )))),
            "string[]"
        );
    }
}
