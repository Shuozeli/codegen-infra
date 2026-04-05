//! TypeScript code writer.

use super::{to_pascal_case, CodeWriter, CodeWriterError};
use codegen_core::{escape_reserved_word, Language};
use codegen_schema::{EnumDef, MessageDef, ScalarType, ServiceDef, StreamingType, Type};

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
                ScalarType::Bool => "boolean".to_string(),
                ScalarType::Int8
                | ScalarType::Int16
                | ScalarType::Int32
                | ScalarType::Uint8
                | ScalarType::Uint16
                | ScalarType::Uint32
                | ScalarType::Float32
                | ScalarType::Float64 => "number".to_string(),
                ScalarType::Int64 => "bigint".to_string(),
                ScalarType::Uint64 => "bigint".to_string(),
                ScalarType::String => "string".to_string(),
                ScalarType::Bytes => "Uint8Array".to_string(),
            },
            Type::Message { name, .. } => {
                escape_reserved_word(&to_pascal_case(name), Language::TypeScript)
            }
            Type::Enum { name, .. } => {
                escape_reserved_word(&to_pascal_case(name), Language::TypeScript)
            }
            Type::Vector(inner) => format!("{}[]", Self::write_field_type(inner)),
            Type::Optional(inner) => format!("{} | null", Self::write_field_type(inner)),
            Type::Map { key, value } => {
                format!(
                    "Record<{}, {}>",
                    Self::write_field_type(key),
                    Self::write_field_type(value)
                )
            }
            Type::OneOf { name, .. } => {
                escape_reserved_word(&to_pascal_case(name), Language::TypeScript)
            }
            Type::ForeignKey(_) => "number".to_string(), // Foreign keys are typically integer IDs
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

        let interface_name = escape_reserved_word(&to_pascal_case(&msg.name), Language::TypeScript);
        output.push_str(&format!("export interface {} {{\n", interface_name));

        // Write fields
        for field in &msg.fields {
            for comment in &field.comments {
                output.push_str(&format!("  /** {} */\n", comment));
            }
            let ty_str = Self::write_field_type(&field.ty);
            let optional = if field.is_optional { "?" } else { "" };
            let field_name = escape_reserved_word(&field.name, Language::TypeScript);
            output.push_str(&format!("  {}{}: {};\n", field_name, optional, ty_str));
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

        let enum_name = escape_reserved_word(&to_pascal_case(&enm.name), Language::TypeScript);
        output.push_str(&format!("export enum {} {{\n", enum_name));

        // Write values
        for (i, value) in enm.values.iter().enumerate() {
            let suffix = if i < enm.values.len() - 1 { "," } else { "" };
            let variant_name =
                escape_reserved_word(&to_pascal_case(&value.name), Language::TypeScript);
            output.push_str(&format!("  {} = {}{}\n", variant_name, value.value, suffix));
        }

        output.push_str("}\n");
        output
    }

    fn write_service_impl(&self, svc: &ServiceDef) -> String {
        let mut output = String::new();

        // Write package as comment if present
        if let Some(ref pkg) = svc.package {
            if !pkg.is_empty() {
                output.push_str(&format!("// Package: {}\n", pkg));
            }
        }

        // Write doc comments
        for comment in &svc.comments {
            output.push_str(&format!("/** {} */\n", comment));
        }

        let class_name = escape_reserved_word(&to_pascal_case(&svc.name), Language::TypeScript);
        output.push_str(&format!("export class {} {{\n", class_name));
        output.push_str("  private client: any;\n\n");
        output.push_str("  constructor(client: any) {\n");
        output.push_str("    this.client = client;\n");
        output.push_str("  }\n\n");

        // Write methods
        for method in &svc.methods {
            let request_type =
                escape_reserved_word(&to_pascal_case(&method.input_type), Language::TypeScript);
            let response_type =
                escape_reserved_word(&to_pascal_case(&method.output_type), Language::TypeScript);
            let method_name =
                escape_reserved_word(&to_pascal_case(&method.name), Language::TypeScript);

            match method.streaming {
                StreamingType::None => {
                    // Unary: async method(request: Type): Promise<Response>
                    output.push_str(&format!(
                        "  async {}(request: {}): Promise<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output.push_str("    return this.client.invoke(this.path, request);\n");
                    output.push_str("  }\n");
                }
                StreamingType::Server => {
                    // Server streaming: async *method(request: Type): AsyncGenerator<Response>
                    output.push_str(&format!(
                        "  async *{}(request: {}): AsyncGenerator<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output
                        .push_str("    yield* this.client.serverStreaming(this.path, request);\n");
                    output.push_str("  }\n");
                }
                StreamingType::Client => {
                    // Client streaming: async method(request: AsyncGenerator<Type>): Promise<Response>
                    output.push_str(&format!(
                        "  async {}(request: AsyncGenerator<{}>): Promise<{}> {{\n",
                        method_name, request_type, response_type
                    ));
                    output
                        .push_str("    return this.client.clientStreaming(this.path, request);\n");
                    output.push_str("  }\n");
                }
                StreamingType::BiDi => {
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
    use codegen_schema::{EnumValue, FieldDef, ScalarType, Type};

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

    #[test]
    fn test_reserved_word_field_escaped() {
        // Field named "type" should be escaped to "type_"
        let writer = TypeScriptCodeWriter::new();
        let msg = MessageDef {
            name: "Data".to_string(),
            fields: vec![FieldDef {
                name: "type".to_string(),
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

        let output = writer.write_interface(&msg);
        assert!(output.contains("type_: string"));
        assert!(!output.contains("type: string"));
    }

    #[test]
    fn test_reserved_word_type_escaped() {
        // Type named "Type" should be escaped to "Type_"
        let writer = TypeScriptCodeWriter::new();
        let msg = MessageDef {
            name: "Type".to_string(),
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

        let output = writer.write_interface(&msg);
        // "Type" -> case-insensitive escape -> "Type_"
        assert!(output.contains("export interface Type_"));
        // The unescaped "export interface Type" should NOT appear
        assert!(!output.contains("export interface Type {"));
    }

    #[test]
    fn test_reserved_word_enum_escaped() {
        // Enum named "Enum" should be escaped to "Enum_"
        let mut writer = TypeScriptCodeWriter::new();
        let enm = EnumDef {
            name: "Enum".to_string(),
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
        // "Enum" -> case-insensitive escape -> "Enum_"
        assert!(output.contains("export enum Enum_"));
        // The unescaped "export enum Enum" should NOT appear
        assert!(!output.contains("export enum Enum {"));
    }
}
