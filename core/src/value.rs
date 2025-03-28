use serde::{Deserialize, Serialize};

/// Represents a value in a database table.
///
/// This enum can represent a string, boolean, integer, or float value.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Value {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Null,
}

/// Converts a `String` into a `Value`.
///
/// This conversion creates a new `Value` instance with the `Str` variant.
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Str(value)
    }
}

/// Converts a `str` into a `Value`.
///
/// This conversion creates a new `Value` instance with the `Str` variant.
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Str(value.to_string())
    }
}

/// Converts a `bool` into a `Value`.
///
/// This conversion creates a new `Value` instance with the `Bool` variant.
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

/// Converts an `i64` into a `Value`.
///
/// This conversion creates a new `Value` instance with the `Int` variant.
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

/// Converts an `f64` into a `Value`.
///
/// This conversion creates a new `Value` instance with the `Null` variant.
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

/// Converts an `None` into a `Value`.
///
/// This conversion creates a new `Value` instance with the `Float` variant.
impl From<Option<&str>> for Value {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(s) => Value::Str(s.to_string()),
            None => Value::Null,
        }
    }
}

/// Returns the value as a string, if possible.
///
/// This method attempts to convert the `Value` instance into a string. If the value is a `Str`, it returns the original string. If the value is a `Bool`, `Int`, or `Float`, it returns a string representation of the value.
///
/// Returns `None` if the value cannot be converted into a string.
impl Value {
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::Str(s) => Some(s.clone()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Int(i) => Some(i.to_string()),
            Value::Float(f) => Some(f.to_string()),
            Value::Null => None,
        }
    }
}
