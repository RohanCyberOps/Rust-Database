use crate::column::Column;
use crate::row::Row;
use serde::{Deserialize, Serialize};

/// Represents a database table.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

impl Table {
    /// Creates a new `Table` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the table.
    pub fn new(name: String) -> Self {
        Table {
            name,
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Adds a column to the table.
    ///
    /// # Arguments
    ///
    /// * `column` - The column to add.
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Adds a row to the table.
    ///
    /// # Arguments
    ///
    /// * `row` - The row to add.
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }
}
