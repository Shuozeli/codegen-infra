//! Rust code writer.

use super::{to_pascal_case, to_snake_case, CodeWriter, CodeWriterError};
use codegen_core::{escape_reserved_word, Language};
use codegen_schema::{EnumDef, MessageDef, ScalarType, ServiceDef, StreamingType, Type};

/// Code writer for Rust.
#[derive(Default)]
pub struct RustCodeWriter;

impl RustCodeWriter {
    pub fn new() -> Self {
        Self
    }

    fn write_field_type(ty: &Type) -> String {
        match ty {
            Type::Scalar(scalar) => match scalar {
                ScalarType::Bool => "bool".to_string(),
                ScalarType::Int8 => "i8".to_string(),
                ScalarType::Uint8 => "u8".to_string(),
                ScalarType::Int16 => "i16".to_string(),
                ScalarType::Uint16 => "u16".to_string(),
                ScalarType::Int32 => "i32".to_string(),
                ScalarType::Uint32 => "u32".to_string(),
                ScalarType::Int64 => "i64".to_string(),
                ScalarType::Uint64 => "u64".to_string(),
                ScalarType::Float32 => "f32".to_string(),
                ScalarType::Float64 => "f64".to_string(),
                ScalarType::String => "String".to_string(),
                ScalarType::Bytes => "Vec<u8>".to_string(),
            },
            Type::Message { name, .. } => {
                escape_reserved_word(&to_pascal_case(name), Language::Rust)
            }
            Type::Enum { name, .. } => escape_reserved_word(&to_pascal_case(name), Language::Rust),
            Type::Vector(inner) => format!("Vec<{}>", Self::write_field_type(inner)),
            Type::Optional(inner) => format!("Option<{}>", Self::write_field_type(inner)),
            Type::Map { key, value } => {
                format!(
                    "std::collections::HashMap<{}, {}>",
                    Self::write_field_type(key),
                    Self::write_field_type(value)
                )
            }
            Type::OneOf { name, .. } => escape_reserved_word(&to_pascal_case(name), Language::Rust),
            Type::ForeignKey(_) => "i64".to_string(), // Foreign keys are typically integer IDs
        }
    }

    fn write_struct(&self, msg: &MessageDef) -> String {
        let mut output = String::new();

        // Write struct indicator if applicable
        if msg.is_struct {
            output.push_str("/// FlatBuffers struct\n");
        }

        // Write doc comments
        for comment in &msg.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        let struct_name = escape_reserved_word(&to_pascal_case(&msg.name), Language::Rust);
        output.push_str(&format!("pub struct {} {{\n", struct_name));

        // Write fields
        for field in &msg.fields {
            for comment in &field.comments {
                output.push_str(&format!("    /// {}\n", comment));
            }
            let ty_str = Self::write_field_type(&field.ty);
            // Field names are snake_case in IR, escape before any transformation
            let field_name = escape_reserved_word(&field.name, Language::Rust);
            output.push_str(&format!("    pub {}: {},\n", field_name, ty_str));
        }

        output.push_str("}\n");
        output
    }

    fn write_enum_impl(&self, enm: &EnumDef) -> String {
        let mut output = String::new();

        // Write doc comments
        for comment in &enm.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        output.push_str("#[derive(Debug, Clone)]\n");
        let enum_name = escape_reserved_word(&to_pascal_case(&enm.name), Language::Rust);
        output.push_str(&format!("pub enum {} {{\n", enum_name));

        // Write variants
        for (i, value) in enm.values.iter().enumerate() {
            let suffix = if i < enm.values.len() - 1 { "," } else { "" };
            let variant_name = escape_reserved_word(&to_pascal_case(&value.name), Language::Rust);
            output.push_str(&format!("    {}{}\n", variant_name, suffix));
        }

        output.push_str("}\n");
        output
    }

    fn write_service_impl(&self, svc: &ServiceDef) -> String {
        let mut output = String::new();

        // Write doc comments
        for comment in &svc.comments {
            output.push_str(&format!("/// {}\n", comment));
        }

        let service_name = escape_reserved_word(&to_pascal_case(&svc.name), Language::Rust);
        output.push_str(&format!("pub struct {} {{\n", service_name));
        output.push_str("    pub codec: (),\n");
        output.push_str("}\n\n");

        output.push_str(&format!("impl {} {{\n", service_name));

        // Write methods
        for method in &svc.methods {
            let input_type =
                escape_reserved_word(&to_pascal_case(&method.input_type), Language::Rust);
            let output_type =
                escape_reserved_word(&to_pascal_case(&method.output_type), Language::Rust);
            let method_name = escape_reserved_word(&to_snake_case(&method.name), Language::Rust);

            match method.streaming {
                StreamingType::None => {
                    // Unary: fn method(&self, req: Input) -> Result<Output, Error>
                    output.push_str(&format!(
                        "    pub fn {}(&self, req: {}) -> Result<{}, Box<dyn std::error::Error>> {{\n",
                        method_name,
                        input_type,
                        output_type,
                    ));
                    output.push_str("        Err(\"unimplemented\".into())\n");
                }
                StreamingType::Server => {
                    // Server streaming: fn method(&self, req: Input) -> Result<Stream<Output>, Error>
                    output.push_str(&format!(
                        "    pub fn {}(&self, req: {}) -> Result<futures::Stream<Item = {}> + Send, Box<dyn std::error::Error>> {{\n",
                        method_name,
                        input_type,
                        output_type,
                    ));
                    output.push_str("        Ok(futures::stream::pending())\n");
                }
                StreamingType::Client => {
                    // Client streaming: fn method(&self, req: Stream<Input>) -> Result<Output, Error>
                    output.push_str(&format!(
                        "    pub fn {}(&self, req: futures::Stream<Item = {}> + Send) -> Result<{}, Box<dyn std::error::Error>> {{\n",
                        method_name,
                        input_type,
                        output_type,
                    ));
                    output.push_str("        Err(\"unimplemented\".into())\n");
                }
                StreamingType::BiDi => {
                    // Bidirectional streaming: fn method(&self, req: Stream<Input>) -> Result<Stream<Output>, Error>
                    output.push_str(&format!(
                        "    pub fn {}(&self, req: futures::Stream<Item = {}> + Send) -> Result<futures::Stream<Item = {}> + Send, Box<dyn std::error::Error>> {{\n",
                        method_name,
                        input_type,
                        output_type,
                    ));
                    output.push_str("        Ok(futures::stream::pending())\n");
                }
            }
            output.push_str("    }\n");
        }

        output.push_str("}\n");
        output
    }
}

impl CodeWriter for RustCodeWriter {
    fn write_message(&mut self, msg: &MessageDef) -> Result<String, CodeWriterError> {
        Ok(self.write_struct(msg))
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
            output.push_str(&self.write_struct(msg));
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
    fn test_write_simple_struct() {
        let writer = RustCodeWriter::new();
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

        let output = writer.write_struct(&msg);
        assert!(output.contains("pub struct Person"));
        assert!(output.contains("pub name: String"));
    }

    #[test]
    fn test_write_enum() {
        let mut writer = RustCodeWriter::new();
        let enm = EnumDef {
            name: "color".to_string(),
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
            ],
            is_union: false,
            namespace: None,
            comments: vec![],
        };

        let output = writer.write_enum(&enm).unwrap();
        assert!(output.contains("pub enum Color"));
        assert!(output.contains("Red"));
        assert!(output.contains("Green"));
    }

    #[test]
    fn test_field_type_mapping() {
        assert_eq!(
            RustCodeWriter::write_field_type(&Type::Scalar(ScalarType::Int32)),
            "i32"
        );
        assert_eq!(
            RustCodeWriter::write_field_type(&Type::Scalar(ScalarType::String)),
            "String"
        );
        assert_eq!(
            RustCodeWriter::write_field_type(&Type::Vector(Box::new(Type::Scalar(
                ScalarType::String
            )))),
            "Vec<String>"
        );
    }

    #[test]
    fn test_reserved_word_field_escaped() {
        // Field named "type" should be escaped to "type_"
        let writer = RustCodeWriter::new();
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

        let output = writer.write_struct(&msg);
        assert!(output.contains("pub type_: String"));
        assert!(!output.contains("pub type: String"));
    }

    #[test]
    fn test_reserved_word_type_escaped() {
        // Type named "type" (lowercase) should be escaped to "type_"
        // since "type" is a Rust keyword
        let writer = RustCodeWriter::new();
        let msg = MessageDef {
            name: "type".to_string(),
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

        let output = writer.write_struct(&msg);
        // "type" -> to_pascal_case -> "Type", then case-insensitive escape -> "Type_"
        assert!(output.contains("pub struct Type_ {\n"));
        // The unescaped "pub struct Type {" should NOT appear
        assert!(!output.contains("pub struct Type {"));
    }

    #[test]
    fn test_reserved_word_enum_escaped() {
        // Enum named "enum" (lowercase) should be escaped to "Enum_"
        let mut writer = RustCodeWriter::new();
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
        // "enum" -> to_pascal_case -> "Enum", then case-insensitive escape -> "Enum_"
        assert!(output.contains("pub enum Enum_"));
        // The unescaped "pub enum Enum" should NOT appear
        assert!(!output.contains("pub enum Enum {"));
    }
}
