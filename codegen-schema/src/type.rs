//! Type representations.

use std::fmt;

/// Scalar types supported across all schema formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Bool,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float32,
    Float64,
    String,
    Bytes,
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarType::Bool => write!(f, "bool"),
            ScalarType::Int8 => write!(f, "int8"),
            ScalarType::Uint8 => write!(f, "uint8"),
            ScalarType::Int16 => write!(f, "int16"),
            ScalarType::Uint16 => write!(f, "uint16"),
            ScalarType::Int32 => write!(f, "int32"),
            ScalarType::Uint32 => write!(f, "uint32"),
            ScalarType::Int64 => write!(f, "int64"),
            ScalarType::Uint64 => write!(f, "uint64"),
            ScalarType::Float32 => write!(f, "float32"),
            ScalarType::Float64 => write!(f, "float64"),
            ScalarType::String => write!(f, "String"),
            ScalarType::Bytes => write!(f, "bytes"),
        }
    }
}

/// A single variant within a oneof group.
#[derive(Debug, Clone)]
pub struct OneOfVariant {
    /// The field name of this variant (e.g., "i", "s" in `oneof result { int32 i = 1; string s = 2; }`).
    pub name: String,
    /// The type of this variant.
    pub ty: Type,
}

/// A foreign key reference type.
#[derive(Debug, Clone)]
pub struct ForeignKey {
    /// The table being referenced.
    pub referenced_table: String,
    /// The column being referenced.
    pub referenced_column: String,
}

/// Type in the unified schema.
#[derive(Debug, Clone)]
pub enum Type {
    /// A scalar type.
    Scalar(ScalarType),
    /// A named message/struct type.
    Message {
        name: String,
        package: Option<String>,
    },
    /// An enum type.
    Enum {
        name: String,
        package: Option<String>,
    },
    /// A vector/array of elements.
    Vector(Box<Type>),
    /// An optional/nullable type.
    Optional(Box<Type>),
    /// A map type (key-value pairs).
    Map { key: Box<Type>, value: Box<Type> },
    /// A oneof group.
    OneOf {
        name: String,
        variants: Vec<OneOfVariant>,
    },
    /// A foreign key reference.
    ForeignKey(ForeignKey),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Scalar(s) => write!(f, "{}", s),
            Type::Message { name, .. } => write!(f, "{}", name),
            Type::Enum { name, .. } => write!(f, "{}", name),
            Type::Vector(inner) => write!(f, "Vec<{}>", inner),
            Type::Optional(inner) => write!(f, "Option<{}>", inner),
            Type::Map { key, value } => write!(f, "Map<{}, {}>", key, value),
            Type::OneOf { name, .. } => write!(f, "{}", name),
            Type::ForeignKey(fk) => {
                write!(f, "FK<{}.{}>", fk.referenced_table, fk.referenced_column)
            }
        }
    }
}

impl Type {
    /// Returns true if this is a scalar type.
    pub fn is_scalar(&self) -> bool {
        matches!(self, Type::Scalar(_))
    }

    /// Get the inner type for container types (Vector, Optional).
    ///
    /// Returns `Some(&inner)` for types that have a single inner type.
    /// Returns `None` for `OneOf` (has multiple variants, not a single element).
    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::Vector(inner) => Some(inner),
            Type::Optional(inner) => Some(inner),
            _ => None,
        }
    }
}
