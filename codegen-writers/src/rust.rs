//! Rust code writer.

use super::{to_pascal_case, to_snake_case, CodeWriter, CodeWriterError};
use codegen_core::ir::{EnumDef, MessageDef, ServiceDef, Type};

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
                codegen_core::ir::ScalarType::Bool => "bool".to_string(),
                codegen_core::ir::ScalarType::Int8 => "i8".to_string(),
                codegen_core::ir::ScalarType::Uint8 => "u8".to_string(),
                codegen_core::ir::ScalarType::Int16 => "i16".to_string(),
                codegen_core::ir::ScalarType::Uint16 => "u16".to_string(),
                codegen_core::ir::ScalarType::Int32 => "i32".to_string(),
                codegen_core::ir::ScalarType::Uint32 => "u32".to_string(),
                codegen_core::ir::ScalarType::Int64 => "i64".to_string(),
                codegen_core::ir::ScalarType::Uint64 => "u64".to_string(),
                codegen_core::ir::ScalarType::Float32 => "f32".to_string(),
                codegen_core::ir::ScalarType::Float64 => "f64".to_string(),
                codegen_core::ir::ScalarType::String => "String".to_string(),
                codegen_core::ir::ScalarType::Bytes => "Vec<u8>".to_string(),
                _ => "Opaque".to_string(),
            },
            Type::Message { name, .. } => to_pascal_case(name),
            Type::Enum { name, .. } => to_pascal_case(name),
            Type::Vector(inner) => format!("Vec<{}>", Self::write_field_type(inner)),
            Type::InlineStruct(inner) => Self::write_field_type(inner),
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

        output.push_str(&format!("pub struct {} {{\n", to_pascal_case(&msg.name)));

        // Write fields
        for field in &msg.fields {
            for comment in &field.comments {
                output.push_str(&format!("    /// {}\n", comment));
            }
            let ty_str = Self::write_field_type(&field.ty);
            output.push_str(&format!("    pub {}: {},\n", field.name, ty_str));
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
        output.push_str(&format!("pub enum {} {{\n", to_pascal_case(&enm.name)));

        // Write variants
        for (i, value) in enm.values.iter().enumerate() {
            let suffix = if i < enm.values.len() - 1 { "," } else { "" };
            output.push_str(&format!("    {}{}\n", to_pascal_case(&value.name), suffix));
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

        output.push_str(&format!("pub struct {} {{\n", to_pascal_case(&svc.name)));
        output.push_str("    pub codec: (),\n");
        output.push_str("}\n\n");

        output.push_str(&format!("impl {} {{\n", to_pascal_case(&svc.name)));

        // Write methods
        for method in &svc.methods {
            let input_type = to_pascal_case(&method.input_type);
            let output_type = to_pascal_case(&method.output_type);
            output.push_str(&format!(
                "    pub fn {}(&self, req: {}) -> Result<{}, Box<dyn std::error::Error>> {{\n",
                to_snake_case(&method.name),
                input_type,
                output_type,
            ));
            output.push_str("        todo!(\"implement\")\n");
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
    use codegen_core::ir::{FieldDef, ScalarType, Type};

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
                codegen_core::ir::EnumValue {
                    name: "Red".to_string(),
                    value: 0,
                    comments: vec![],
                },
                codegen_core::ir::EnumValue {
                    name: "Green".to_string(),
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
}
