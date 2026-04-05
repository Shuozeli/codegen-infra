//! Integration tests for codegen-writers.
//!
//! These tests verify the full workflow:
//! 1. SchemaProvider converts schema -> IR (Intermediate Representation)
//! 2. CodeWriter consumes IR -> generated code
//!
//! Note: These tests build IR structures directly rather than parsing from
//! schema files, since the raw schema types (flatc_rs_schema, prost_types) are
//! not re-exported by the adapter crates.

use codegen_core::ir::{
    EnumDef, EnumValue, FieldDef, MessageDef, MethodDef, ScalarType, ServiceDef, Type,
};
use codegen_writers::{CodeWriter, DartCodeWriter, RustCodeWriter, TypeScriptCodeWriter};

// ---------------------------------------------------------------------------
// Test Fixtures: IR Definitions
// ---------------------------------------------------------------------------

/// Create a simple Monster message with various field types.
fn make_monster_message() -> MessageDef {
    MessageDef {
        name: "Monster".to_string(),
        fields: vec![
            FieldDef {
                name: "hp".to_string(),
                ty: Type::Scalar(ScalarType::Int16),
                is_optional: false,
                default_value: Some("100".to_string()),
                id: Some(0),
                comments: vec!["Monster HP".to_string()],
            },
            FieldDef {
                name: "name".to_string(),
                ty: Type::Scalar(ScalarType::String),
                is_optional: true,
                default_value: None,
                id: Some(1),
                comments: vec![],
            },
            FieldDef {
                name: "inventory".to_string(),
                ty: Type::Vector(Box::new(Type::Scalar(ScalarType::Uint8))),
                is_optional: false,
                default_value: None,
                id: Some(2),
                comments: vec![],
            },
            FieldDef {
                name: "pos".to_string(),
                ty: Type::Message {
                    name: "Vec3".to_string(),
                    package: Some("MyGame".to_string()),
                },
                is_optional: false,
                default_value: None,
                id: Some(3),
                comments: vec![],
            },
        ],
        is_struct: false,
        namespace: Some("MyGame".to_string()),
        comments: vec!["A monster entity".to_string()],
    }
}

/// Create a Color enum.
fn make_color_enum() -> EnumDef {
    EnumDef {
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
        is_struct: false,
        namespace: Some("MyGame".to_string()),
        comments: vec![],
    }
}

/// Create a Monster service.
fn make_monster_service() -> ServiceDef {
    ServiceDef {
        name: "MonsterService".to_string(),
        package: "MyGame".to_string(),
        proto_name: "MonsterService".to_string(),
        methods: vec![
            MethodDef {
                name: "create_monster".to_string(),
                proto_name: "CreateMonster".to_string(),
                input_type: "CreateMonsterRequest".to_string(),
                output_type: "Monster".to_string(),
                client_streaming: false,
                server_streaming: false,
                codec_path: "crate::codec::Codec".to_string(),
                comments: vec![],
            },
            MethodDef {
                name: "get_monster".to_string(),
                proto_name: "GetMonster".to_string(),
                input_type: "GetMonsterRequest".to_string(),
                output_type: "Monster".to_string(),
                client_streaming: false,
                server_streaming: false,
                codec_path: "crate::codec::Codec".to_string(),
                comments: vec![],
            },
        ],
        comments: vec!["Service for monster operations".to_string()],
    }
}

/// Create a Person message for Protobuf tests.
fn make_person_message() -> MessageDef {
    MessageDef {
        name: "Person".to_string(),
        fields: vec![
            FieldDef {
                name: "id".to_string(),
                ty: Type::Scalar(ScalarType::Int32),
                is_optional: false,
                default_value: None,
                id: Some(1),
                comments: vec![],
            },
            FieldDef {
                name: "name".to_string(),
                ty: Type::Scalar(ScalarType::String),
                is_optional: true,
                default_value: None,
                id: Some(2),
                comments: vec![],
            },
            FieldDef {
                name: "emails".to_string(),
                ty: Type::Vector(Box::new(Type::Scalar(ScalarType::String))),
                is_optional: false,
                default_value: None,
                id: Some(3),
                comments: vec![],
            },
        ],
        is_struct: false,
        namespace: Some("mygame".to_string()),
        comments: vec![],
    }
}

/// Create a PhoneType enum for Protobuf tests.
/// Note: Enum values use SCREAMING_SNAKE_CASE as is conventional for Protobuf enums.
fn make_phone_type_enum() -> EnumDef {
    EnumDef {
        name: "PhoneType".to_string(),
        values: vec![
            EnumValue {
                name: "MOBILE".to_string(),
                value: 0,
                comments: vec![],
            },
            EnumValue {
                name: "HOME".to_string(),
                value: 1,
                comments: vec![],
            },
            EnumValue {
                name: "WORK".to_string(),
                value: 2,
                comments: vec![],
            },
        ],
        is_union: false,
        is_struct: false,
        namespace: Some("mygame".to_string()),
        comments: vec![],
    }
}

/// Create a TestService for Protobuf tests.
fn make_test_service() -> ServiceDef {
    ServiceDef {
        name: "TestService".to_string(),
        package: "mygame".to_string(),
        proto_name: "TestService".to_string(),
        methods: vec![MethodDef {
            name: "get_person".to_string(),
            proto_name: "GetPerson".to_string(),
            input_type: "Person".to_string(),
            output_type: "Person".to_string(),
            client_streaming: false,
            server_streaming: false,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        }],
        comments: vec![],
    }
}

// ---------------------------------------------------------------------------
// Integration Tests: Rust Code Generation
// ---------------------------------------------------------------------------

#[test]
fn test_rust_message_generation() {
    let messages = vec![make_monster_message()];
    let enums = vec![make_color_enum()];
    let services = vec![make_monster_service()];

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify message is generated
    assert!(
        output.contains("pub struct Monster"),
        "Should contain Monster struct"
    );
    assert!(
        output.contains("pub hp: i16"),
        "Should contain hp field with correct Rust type"
    );
    assert!(
        output.contains("pub name: String"),
        "Should contain name field with correct Rust type"
    );
    assert!(
        output.contains("pub inventory: Vec<u8>"),
        "Should contain inventory field as Vec<u8>"
    );
    assert!(
        output.contains("pub pos: Vec3"),
        "Should contain pos field as Vec3 message type"
    );
    assert!(
        output.contains("/// A monster entity"),
        "Should contain doc comment"
    );
    assert!(
        output.contains("/// Monster HP"),
        "Should contain field doc comment"
    );
}

#[test]
fn test_rust_enum_generation() {
    let messages = vec![];
    let enums = vec![make_color_enum()];
    let services = vec![];

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify enum is generated
    assert!(
        output.contains("pub enum Color"),
        "Should contain Color enum"
    );
    assert!(output.contains("Red"), "Should contain Red variant");
    assert!(output.contains("Green"), "Should contain Green variant");
    assert!(output.contains("Blue"), "Should contain Blue variant");
    assert!(
        output.contains("#[derive(Debug, Clone)]"),
        "Should have derive attribute"
    );
}

#[test]
fn test_rust_service_generation() {
    let messages = vec![];
    let enums = vec![];
    let services = vec![make_monster_service()];

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify service is generated
    assert!(
        output.contains("pub struct MonsterService"),
        "Should contain MonsterService struct"
    );
    assert!(
        output.contains("pub fn create_monster"),
        "Should contain create_monster method"
    );
    assert!(
        output.contains("pub fn get_monster"),
        "Should contain get_monster method"
    );
    assert!(
        output.contains("Result<Monster, Box<dyn std::error::Error>>"),
        "Should have correct return type"
    );
}

#[test]
fn test_rust_vector_type_generation() {
    let msg = MessageDef {
        name: "VectorTest".to_string(),
        fields: vec![FieldDef {
            name: "items".to_string(),
            ty: Type::Vector(Box::new(Type::Scalar(ScalarType::String))),
            is_optional: false,
            default_value: None,
            id: Some(1),
            comments: vec![],
        }],
        is_struct: false,
        namespace: None,
        comments: vec![],
    };

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_message(&msg)
        .expect("write_message should succeed");

    assert!(output.contains("pub items: Vec<String>"));
}

// ---------------------------------------------------------------------------
// Integration Tests: Dart Code Generation
// ---------------------------------------------------------------------------

#[test]
fn test_dart_message_generation() {
    let messages = vec![make_monster_message()];
    let enums = vec![make_color_enum()];
    let services = vec![make_monster_service()];

    let mut writer = DartCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify message is generated as Dart class
    assert!(
        output.contains("class Monster"),
        "Should contain Monster class"
    );
    // Dart uses int for integer types
    assert!(output.contains("final int hp"), "Should contain hp field");
    assert!(
        output.contains("final String? name"),
        "Should contain optional name field as String?"
    );
    assert!(
        output.contains("final List<int> inventory"),
        "Should contain inventory field as List<int>"
    );
}

#[test]
fn test_dart_enum_generation() {
    let messages = vec![];
    let enums = vec![make_color_enum()];
    let services = vec![];

    let mut writer = DartCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify enum is generated
    assert!(output.contains("enum Color"), "Should contain Color enum");
    assert!(output.contains("Red"), "Should contain Red variant");
    assert!(output.contains("Green"), "Should contain Green variant");
    assert!(output.contains("Blue"), "Should contain Blue variant");
}

#[test]
fn test_dart_service_generation() {
    let messages = vec![];
    let enums = vec![];
    let services = vec![make_monster_service()];

    let mut writer = DartCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify service is generated
    assert!(
        output.contains("class MonsterService"),
        "Should contain MonsterService class"
    );
    // Dart uses method.name directly (no case conversion)
    assert!(
        output.contains("Future<Monster> create_monster"),
        "Should contain create_monster method"
    );
    assert!(
        output.contains("Future<Monster> get_monster"),
        "Should contain get_monster method"
    );
    assert!(
        output.contains("final pbGrpc.GrpcClient client"),
        "Should have grpc client field"
    );
}

// ---------------------------------------------------------------------------
// Integration Tests: TypeScript Code Generation
// ---------------------------------------------------------------------------

#[test]
fn test_typescript_message_generation() {
    let messages = vec![make_monster_message()];
    let enums = vec![make_color_enum()];
    let services = vec![make_monster_service()];

    let mut writer = TypeScriptCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify message is generated as TypeScript interface
    assert!(
        output.contains("export interface Monster"),
        "Should contain Monster interface"
    );
    assert!(
        output.contains("hp: number"),
        "Should contain hp field as number (TypeScript)"
    );
    assert!(
        output.contains("name?: string"),
        "Should contain optional name field as string"
    );
    assert!(
        output.contains("inventory: number[]"),
        "Should contain inventory field as number[]"
    );
    assert!(
        output.contains("/** A monster entity */"),
        "Should contain doc comment"
    );
}

#[test]
fn test_typescript_enum_generation() {
    let messages = vec![];
    let enums = vec![make_color_enum()];
    let services = vec![];

    let mut writer = TypeScriptCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify enum is generated
    assert!(
        output.contains("export enum Color"),
        "Should contain Color enum"
    );
    assert!(output.contains("Red = 0"), "Should contain Red = 0");
    assert!(output.contains("Green = 1"), "Should contain Green = 1");
    assert!(output.contains("Blue = 2"), "Should contain Blue = 2");
}

#[test]
fn test_typescript_service_generation() {
    let messages = vec![];
    let enums = vec![];
    let services = vec![make_monster_service()];

    let mut writer = TypeScriptCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify service is generated
    assert!(
        output.contains("export class MonsterService"),
        "Should contain MonsterService class"
    );
    // TypeScript converts method names to PascalCase
    assert!(
        output.contains("async CreateMonster"),
        "Should contain CreateMonster async method"
    );
    assert!(
        output.contains("async GetMonster"),
        "Should contain GetMonster async method"
    );
    assert!(
        output.contains("Promise<Monster>"),
        "Should have correct return type"
    );
}

// ---------------------------------------------------------------------------
// Integration Tests: Protobuf IR (via direct IR construction)
// ---------------------------------------------------------------------------

#[test]
fn test_protobuf_ir_to_rust_message() {
    let messages = vec![make_person_message()];
    let enums = vec![make_phone_type_enum()];
    let services = vec![make_test_service()];

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify message is generated
    assert!(
        output.contains("pub struct Person"),
        "Should contain Person struct"
    );
    assert!(
        output.contains("pub id: i32"),
        "Should contain id field with correct Rust type"
    );
    assert!(
        output.contains("pub name: String"),
        "Should contain name field with correct Rust type"
    );
    assert!(
        output.contains("pub emails: Vec<String>"),
        "Should contain emails field as Vec<String>"
    );
}

#[test]
fn test_protobuf_ir_to_rust_enum() {
    let messages = vec![];
    let enums = vec![make_phone_type_enum()];
    let services = vec![];

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify enum is generated
    assert!(
        output.contains("pub enum PhoneType"),
        "Should contain PhoneType enum"
    );
    // Rust uses PascalCase for enum variants (to_pascal_case converts MOBILE -> MOBILE since no underscores)
    assert!(
        output.contains("MOBILE"),
        "Should contain MOBILE variant (all caps - no underscores to trigger case change)"
    );
    assert!(output.contains("HOME"), "Should contain HOME variant");
    assert!(output.contains("WORK"), "Should contain WORK variant");
}

#[test]
fn test_protobuf_ir_to_rust_service() {
    let messages = vec![];
    let enums = vec![];
    let services = vec![make_test_service()];

    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify service is generated
    assert!(
        output.contains("pub struct TestService"),
        "Should contain TestService struct"
    );
    assert!(
        output.contains("pub fn get_person"),
        "Should contain get_person method (snake_case)"
    );
}

// ---------------------------------------------------------------------------
// Integration Tests: Full Round-Trip Verification
// ---------------------------------------------------------------------------

#[test]
fn test_round_trip_all_languages() {
    let messages = vec![make_monster_message()];
    let enums = vec![make_color_enum()];
    let services = vec![make_monster_service()];

    // Generate code for all three languages
    let mut rust_writer = RustCodeWriter::new();
    let mut dart_writer = DartCodeWriter::new();
    let mut ts_writer = TypeScriptCodeWriter::new();

    let rust_output = rust_writer
        .write_file(&messages, &enums, &services)
        .expect("Rust write_file should succeed");
    let dart_output = dart_writer
        .write_file(&messages, &enums, &services)
        .expect("Dart write_file should succeed");
    let ts_output = ts_writer
        .write_file(&messages, &enums, &services)
        .expect("TypeScript write_file should succeed");

    // Verify all outputs contain the same semantic elements
    // Monster message
    assert!(rust_output.contains("pub struct Monster"));
    assert!(dart_output.contains("class Monster"));
    assert!(ts_output.contains("export interface Monster"));

    // Color enum
    assert!(rust_output.contains("pub enum Color"));
    assert!(dart_output.contains("enum Color"));
    assert!(ts_output.contains("export enum Color"));

    // MonsterService
    assert!(rust_output.contains("pub struct MonsterService"));
    assert!(dart_output.contains("class MonsterService"));
    assert!(ts_output.contains("export class MonsterService"));
}

#[test]
fn test_round_trip_rust_message_enum_service() {
    let messages = vec![make_monster_message()];
    let enums = vec![make_color_enum()];
    let services = vec![make_monster_service()];

    // Verify IR is correct
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].name, "Monster");
    assert_eq!(messages[0].fields.len(), 4);

    assert_eq!(enums.len(), 1);
    assert_eq!(enums[0].name, "Color");
    assert_eq!(enums[0].values.len(), 3);

    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "MonsterService");
    assert_eq!(services[0].methods.len(), 2);

    // Generate Rust code
    let mut writer = RustCodeWriter::new();
    let output = writer
        .write_file(&messages, &enums, &services)
        .expect("write_file should succeed");

    // Verify generated code contains expected elements
    assert!(output.contains("pub struct Monster"));
    assert!(output.contains("pub enum Color"));
    assert!(output.contains("pub struct MonsterService"));
    assert!(output.contains("pub fn create_monster"));
    assert!(output.contains("pub fn get_monster"));
}

// ---------------------------------------------------------------------------
// Integration Tests: Type Mapping Verification
// ---------------------------------------------------------------------------

#[test]
fn test_type_mapping_scalar_types() {
    // Test that scalar types are mapped correctly across languages
    // Note: Field names are used as-is from the IR (snake_case)
    let messages = vec![MessageDef {
        name: "ScalarTypes".to_string(),
        fields: vec![
            FieldDef {
                name: "bool_field".to_string(),
                ty: Type::Scalar(ScalarType::Bool),
                is_optional: false,
                default_value: None,
                id: Some(1),
                comments: vec![],
            },
            FieldDef {
                name: "i32_field".to_string(),
                ty: Type::Scalar(ScalarType::Int32),
                is_optional: false,
                default_value: None,
                id: Some(2),
                comments: vec![],
            },
            FieldDef {
                name: "i64_field".to_string(),
                ty: Type::Scalar(ScalarType::Int64),
                is_optional: false,
                default_value: None,
                id: Some(3),
                comments: vec![],
            },
            FieldDef {
                name: "string_field".to_string(),
                ty: Type::Scalar(ScalarType::String),
                is_optional: false,
                default_value: None,
                id: Some(4),
                comments: vec![],
            },
            FieldDef {
                name: "bytes_field".to_string(),
                ty: Type::Scalar(ScalarType::Bytes),
                is_optional: false,
                default_value: None,
                id: Some(5),
                comments: vec![],
            },
        ],
        is_struct: false,
        namespace: None,
        comments: vec![],
    }];

    let mut rust_writer = RustCodeWriter::new();
    let mut dart_writer = DartCodeWriter::new();
    let mut ts_writer = TypeScriptCodeWriter::new();

    let rust_output = rust_writer
        .write_file(&messages, &[], &[])
        .expect("Rust write_file should succeed");
    let dart_output = dart_writer
        .write_file(&messages, &[], &[])
        .expect("Dart write_file should succeed");
    let ts_output = ts_writer
        .write_file(&messages, &[], &[])
        .expect("TypeScript write_file should succeed");

    // Rust type mappings (field names used as-is)
    assert!(rust_output.contains("bool_field: bool"));
    assert!(rust_output.contains("i32_field: i32"));
    assert!(rust_output.contains("i64_field: i64"));
    assert!(rust_output.contains("string_field: String"));
    assert!(rust_output.contains("bytes_field: Vec<u8>"));

    // Dart type mappings (field names used as-is)
    assert!(dart_output.contains("bool boolField"));
    assert!(dart_output.contains("int i32Field"));
    assert!(dart_output.contains("int i64Field"));
    assert!(dart_output.contains("String stringField"));
    assert!(dart_output.contains("Uint8List bytesField"));

    // TypeScript type mappings (field names used as-is)
    assert!(ts_output.contains("bool_field: boolean"));
    assert!(ts_output.contains("i32_field: number"));
    assert!(ts_output.contains("i64_field: bigint"));
    assert!(ts_output.contains("string_field: string"));
    assert!(ts_output.contains("bytes_field: Uint8Array"));
}

#[test]
fn test_type_mapping_vector_types() {
    let messages = vec![MessageDef {
        name: "VectorTypes".to_string(),
        fields: vec![FieldDef {
            name: "strings".to_string(),
            ty: Type::Vector(Box::new(Type::Scalar(ScalarType::String))),
            is_optional: false,
            default_value: None,
            id: Some(1),
            comments: vec![],
        }],
        is_struct: false,
        namespace: None,
        comments: vec![],
    }];

    let mut rust_writer = RustCodeWriter::new();
    let mut dart_writer = DartCodeWriter::new();
    let mut ts_writer = TypeScriptCodeWriter::new();

    let rust_output = rust_writer
        .write_file(&messages, &[], &[])
        .expect("Rust write_file should succeed");
    let dart_output = dart_writer
        .write_file(&messages, &[], &[])
        .expect("Dart write_file should succeed");
    let ts_output = ts_writer
        .write_file(&messages, &[], &[])
        .expect("TypeScript write_file should succeed");

    // Verify vector type mappings
    assert!(rust_output.contains("pub strings: Vec<String>"));
    assert!(dart_output.contains("final List<String> strings"));
    assert!(ts_output.contains("strings: string[]"));
}

#[test]
fn test_type_mapping_message_reference() {
    let messages = vec![MessageDef {
        name: "Container".to_string(),
        fields: vec![FieldDef {
            name: "monster".to_string(),
            ty: Type::Message {
                name: "Monster".to_string(),
                package: Some("MyGame".to_string()),
            },
            is_optional: true,
            default_value: None,
            id: Some(1),
            comments: vec![],
        }],
        is_struct: false,
        namespace: None,
        comments: vec![],
    }];

    let mut rust_writer = RustCodeWriter::new();
    let mut dart_writer = DartCodeWriter::new();
    let mut ts_writer = TypeScriptCodeWriter::new();

    let rust_output = rust_writer
        .write_file(&messages, &[], &[])
        .expect("Rust write_file should succeed");
    let dart_output = dart_writer
        .write_file(&messages, &[], &[])
        .expect("Dart write_file should succeed");
    let ts_output = ts_writer
        .write_file(&messages, &[], &[])
        .expect("TypeScript write_file should succeed");

    // Verify message reference type mappings
    // Note: The Rust code writer does NOT wrap message references in Option
    // even for optional fields - it just uses the message type directly
    assert!(rust_output.contains("pub monster: Monster"));
    assert!(dart_output.contains("final Monster? monster"));
    assert!(ts_output.contains("monster?: Monster"));
}

// ---------------------------------------------------------------------------
// Integration Tests: Streaming Method Variants
// ---------------------------------------------------------------------------

#[test]
fn test_rust_streaming_methods() {
    let unary_service = ServiceDef {
        name: "UnaryService".to_string(),
        package: "test".to_string(),
        proto_name: "UnaryService".to_string(),
        methods: vec![MethodDef {
            name: "get".to_string(),
            proto_name: "Get".to_string(),
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            client_streaming: false,
            server_streaming: false,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        }],
        comments: vec![],
    };

    let server_streaming_service = ServiceDef {
        name: "ServerStreamingService".to_string(),
        package: "test".to_string(),
        proto_name: "ServerStreamingService".to_string(),
        methods: vec![MethodDef {
            name: "subscribe".to_string(),
            proto_name: "Subscribe".to_string(),
            input_type: "Request".to_string(),
            output_type: "Response".to_string(),
            client_streaming: false,
            server_streaming: true,
            codec_path: "crate::codec::Codec".to_string(),
            comments: vec![],
        }],
        comments: vec![],
    };

    // Test Rust service generation for different streaming modes
    let mut rust_writer = RustCodeWriter::new();

    let unary_output = rust_writer
        .write_service(&unary_service)
        .expect("write_service should succeed");

    let server_streaming_output = rust_writer
        .write_service(&server_streaming_service)
        .expect("write_service should succeed");

    // Both should generate similar Rust signatures (streaming is handled at runtime)
    assert!(unary_output.contains("pub fn get"));
    assert!(server_streaming_output.contains("pub fn subscribe"));
}

#[test]
fn test_typescript_streaming_methods() {
    let streaming_service = ServiceDef {
        name: "StreamingService".to_string(),
        package: "test".to_string(),
        proto_name: "StreamingService".to_string(),
        methods: vec![
            MethodDef {
                name: "unary".to_string(),
                proto_name: "Unary".to_string(),
                input_type: "Request".to_string(),
                output_type: "Response".to_string(),
                client_streaming: false,
                server_streaming: false,
                codec_path: "crate::codec::Codec".to_string(),
                comments: vec![],
            },
            MethodDef {
                name: "server_stream".to_string(),
                proto_name: "ServerStream".to_string(),
                input_type: "Request".to_string(),
                output_type: "Response".to_string(),
                client_streaming: false,
                server_streaming: true,
                codec_path: "crate::codec::Codec".to_string(),
                comments: vec![],
            },
            MethodDef {
                name: "client_stream".to_string(),
                proto_name: "ClientStream".to_string(),
                input_type: "Request".to_string(),
                output_type: "Response".to_string(),
                client_streaming: true,
                server_streaming: false,
                codec_path: "crate::codec::Codec".to_string(),
                comments: vec![],
            },
            MethodDef {
                name: "bidi_stream".to_string(),
                proto_name: "BidiStream".to_string(),
                input_type: "Request".to_string(),
                output_type: "Response".to_string(),
                client_streaming: true,
                server_streaming: true,
                codec_path: "crate::codec::Codec".to_string(),
                comments: vec![],
            },
        ],
        comments: vec![],
    };

    let mut ts_writer = TypeScriptCodeWriter::new();
    let output = ts_writer
        .write_service(&streaming_service)
        .expect("write_service should succeed");

    // Verify different streaming method signatures
    // Unary: Promise<Response>
    assert!(output.contains("Promise<Response>"));
    // Server streaming: AsyncGenerator<Response>
    assert!(output.contains("AsyncGenerator<Response>"));
    // Client streaming: Promise<Response> with AsyncGenerator input
    assert!(output.contains("AsyncGenerator<Request>"));
}
