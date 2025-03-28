//! # Core Module Documentation
//!
//! This module provides the fundamental building blocks for database operations,
//! including data structures and functions for tables, columns, rows, and values.
//!
//! ## Database Functionality
//!
//! The following modules are used by both the server and client to provide database functionality:
//!
//! - [`table`](table): Defines the `Table` structure representing a database table.
//! - [`column`](crate::column): Defines the `Column` structure representing a column in a table.
//! - [`request_types`](request_types): Defines various request types used in interacting with tables.
//! - [`value`](value): Defines the `Value` structure representing a value in a table.
//! - [`row`](row): Defines the `Row` structure representing a row in a table.
//!
//! These modules encapsulate related functionality and data structures essential for database operations.
//!
//! ## Client-Side Functionality
//!
//! The following module provides functions for building a client to interact with the server's API:
//!
//! - [`client_functions`](client_functions): Client Functions to interact with the server's API.
//!
//!
//! ## Examples
//!
//! For examples of using the client_functions, see the documentation of the client_functions module.

pub mod client_functions;
pub mod column;
pub mod request_types;
pub mod row;
pub mod table;
pub mod value;

#[cfg(feature = "doc_examples")]
extern crate reqwest;
