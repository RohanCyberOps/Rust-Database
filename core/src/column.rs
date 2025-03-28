use serde::{Deserialize, Serialize};

/// Represents a column in a database table.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Column {
    pub key: String,
    pub primary_key: bool,
    pub non_null: bool,
    pub unique: bool,
    pub foreign_key: Option<Vec<Box<Column>>>,
}

impl Column {
    /// Creates a new `Column` instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key or name of the column.
    /// * `primary_key` - Indicates if the column is a primary key.
    /// * `non_null` - Indicates if the column does not allow NULL values.
    /// * `unique` - Indicates if the column values must be unique.
    /// * `foreign_key` - Optional foreign key reference to another column.
    pub fn new(
        key: String,
        primary_key: bool,
        non_null: bool,
        unique: bool,
        foreign_key: Option<Vec<Box<Column>>>,
    ) -> Self {
        Self {
            key,
            primary_key,
            non_null,
            unique,
            foreign_key,
        }
    }
}
