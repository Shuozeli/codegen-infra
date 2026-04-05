//! Intermediate Representation (IR) for schema types.
//!
//! This module defines common types used across different schema formats
//! to represent services, messages, fields, and types.

pub mod field;
pub mod message;
pub mod schema;
pub mod service;
pub mod r#type;

pub use field::FieldDef;
pub use message::MessageDef;
pub use r#type::Type;
pub use schema::{EnumDef, EnumValue, ScalarType, SchemaProvider};
pub use service::{MethodDef, ServiceDef};
