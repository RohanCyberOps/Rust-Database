use crate::value::Value;
use serde::{Deserialize, Serialize};

/// Represents a row in a database table.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Row {
    pub values: Vec<Value>,
}

impl Row {
    /// Creates a new `Row` instance.
    ///
    /// # Arguments
    ///
    /// * `values` - The values of the row.
    pub fn new(values: Vec<Value>) -> Self {
        Row { values }
    }

    /// Adds a value to the row.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to add.
    pub fn add_value<T>(&mut self, value: T)
    where
        T: Into<Value>,
    {
        self.values.push(value.into());
    }
}
