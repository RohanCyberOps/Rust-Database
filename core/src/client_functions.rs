//! Client Functions to interact with the server's API.
use crate::request_types::{
    CreateRequests, CreateTableRequests, DropTableRequest, InsertColumnRequest, InsertRowRequest,
    RenameTableRequest, SelectRequest, UpdateRequest,
};
use log::{debug, error, info};
use reqwest::Client;
use serde_json::json;
use std::error;

/// Creates a new table on the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `create_table_request` - The request object containing the name of the table to create.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{CreateRequests};
/// use core::client_functions::create;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
///     let create_request = CreateRequests { name: "test_table".to_string() };
///     create(&client, &create_request).await.unwrap();
/// }
/// ```
pub async fn create(
    client: &Client,
    create_table_request: &CreateRequests,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/create".to_string();
    let body = json!({
        "name": create_table_request.name,
    });

    let resp = client.post(&url).json(&body).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Create Response: {:?}", resp);
            info!("Created Table {:?}", create_table_request.name);
            Ok(())
        }
        false => {
            debug!("Create Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

/// Creates a new table with specified columns on the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `create_table_request` - The request object containing the table name and columns.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{CreateTableRequests, InsertColumnRequest};
/// use core::client_functions::create_table;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
/// // Insert columns
///     let insert_column_request = InsertColumnRequest {
///         table_name: "test_table".to_string(),
///         key: "test_key".to_string(),
///         primary_key: true,
///         non_null: true,
///         unique: true,
///         foreign_key: None,
///     };
///
/// // Create new table to be dropped
///     create_table(&client, &CreateTableRequests {
///        name: "test_table2".to_string(),
///         insert_column_requests: vec![insert_column_request],
///     }).await.unwrap();
/// }
/// ```
pub async fn create_table(
    client: &Client,
    create_table_request: &CreateTableRequests,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/create_table".to_string();

    let resp = client.post(&url).json(create_table_request).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Create Table Response: {:?}", resp);
            info!("Created Table {:?}", create_table_request.name);
            Ok(())
        }
        false => {
            debug!("Create Table Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

/// Drops a table on the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `drop_table_request` - The request object containing the name of the table to drop.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{DropTableRequest};
/// use core::client_functions::drop_table;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
///     let drop_table_request = DropTableRequest { name: "test_table".to_string() };
///     drop_table(&client, &drop_table_request).await.unwrap();
/// }
/// ```
pub async fn drop_table(
    client: &Client,
    drop_table_request: &DropTableRequest,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/drop_table".to_string();

    let resp = client.post(&url).json(drop_table_request).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Drop Table Response: {:?}", resp);
            info!("Dropped Table {:?}", drop_table_request.name);
        }
        false => {
            debug!("Drop Table Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            error!("Drop Table Response: {}", error_message);
        }
    }
    Ok(())
}

/// Renames a table's name on the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `rename_table_request` - The request object containing the current and new names of the table.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{RenameTableRequest};
/// use core::client_functions::rename_table;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
///     let rename_table_request = RenameTableRequest { current_name: "test_table2".to_string(), new_name: "test_drop_table".to_string() };
///     rename_table(&client, &rename_table_request).await.unwrap();
/// }
/// ```
pub async fn rename_table(
    client: &Client,
    rename_table_request: &RenameTableRequest,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/rename_table".to_string();

    let resp = client.post(&url).json(rename_table_request).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Rename Table Response: {:?}", resp);
            info!("Renamed Table {:?}", rename_table_request);
            Ok(())
        }
        false => {
            debug!("Rename Table Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

/// Inserts a new column into a table on the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `insert_column_request` - The request object containing the table name and column details.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{InsertColumnRequest};
/// use core::client_functions::insert_column;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
///     // Insert columns
///     let insert_column_request = InsertColumnRequest {
///         table_name: "test_table".to_string(),
///         key: "test_key".to_string(),
///         primary_key: true,
///         non_null: true,
///         unique: true,
///         foreign_key: None,
///     };
///     insert_column(&client, &insert_column_request).await.unwrap();
/// }
/// ```
pub async fn insert_column(
    client: &Client,
    insert_column_request: &InsertColumnRequest,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/insert_column".to_string();

    let resp = client.post(&url).json(insert_column_request).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Insert Column Response: {:?}", resp);
            info!("Inserted Column {:?}", insert_column_request);
            Ok(())
        }
        false => {
            debug!("Insert Column Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

/// Inserts a new row into a table on the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `insert_row_request` - The request object containing the table name and row data.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{InsertRowRequest};
/// use core::client_functions::insert_row;
/// use core::row::Row;
/// use core::value::Value;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
///  // Insert a row
///     let insert_row_request = InsertRowRequest {
///         table_name: "test_table".to_string(),
///         row: Row::new(vec![Value::from("test_value".to_string()), Value::from(13)])
///     };
///
///     insert_row(&client, &insert_row_request).await.unwrap();
/// }
/// ```
pub async fn insert_row(
    client: &Client,
    insert_row_request: &InsertRowRequest,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/insert_row".to_string();

    let resp = client.post(&url).json(insert_row_request).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Insert Row Response: {:?}", resp);
            info!("Inserted Row {:?}", insert_row_request);
            Ok(())
        }
        false => {
            debug!("Insert Row Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

/// Sends a select query to the server.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `select_request` - The request object containing the select query details.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{SelectRequest, Condition};
/// use core::client_functions::select;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
/// // Select from the table with a condition
///     let select_request = SelectRequest {
///         table_name: "test_table".to_string(),
///         columns: Option::from(vec!["test_key".to_string(), "test_key3".to_string()]), // Empty vec would mean *
///         condition: Option::from(Condition {
///             column: "test_key".to_string(),
///             value: "true".to_string(),
///         }),
///     };
///
///     select(&client, &select_request).await.unwrap();
/// }
/// ```
pub async fn select(
    client: &Client,
    select_request: &SelectRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://localhost:3000/select".to_string();

    let resp = client.post(&url).json(select_request).send().await?;

    // Extract the status code before consuming `resp`
    let status = resp.status();
    match status.is_success() {
        true => {
            let body = resp.text().await?;
            debug!("Select Response: {}", body); // Log the body content
            info!("Select result from 'test_create_table': {}", body);
            Ok(())
        }
        false => {
            debug!("Select Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

/// Updates rows in a table on the server based on specified conditions.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `update_request` - The request object containing the table name, condition, and updates.
///
/// # Examples
///
/// ```
/// use log::LevelFilter;
/// use reqwest::Client;
/// use core::request_types::{UpdateRequest, Condition, UpdateColumnRequest};
/// use core::client_functions::update_table;
///
/// #[tokio::main]
/// async fn main() {
///
///     env_logger::builder()
///         .filter_level(LevelFilter::Info)
///         .format_timestamp_millis()
///         .init();
///
///     let client = Client::new();
///
/// // Update rows in the table
///     let update_request = UpdateRequest {
///         table_name: "test_table".to_string(),
///         condition: Option::from(Condition {
///             column: "test_key".to_string(),
///             value: "true".to_string(),
///         }),
///         updates: vec![
///             UpdateColumnRequest {
///                 column: "test_key3".to_string(),
///                 value: "updated_value".to_string(),
///             },
///             UpdateColumnRequest {
///                 column: "test_key2".to_string(),
///                 value: "17.78".to_string(),
///             },
///         ],
///     };
///
///     update_table(&client, &update_request).await.unwrap();
/// }
/// ```
pub async fn update_table(
    client: &Client,
    update_request: &UpdateRequest,
) -> Result<(), Box<dyn error::Error>> {
    let url = "http://localhost:3000/update_table".to_string();

    let resp = client.post(&url).json(update_request).send().await?;

    match resp.status().is_success() {
        true => {
            debug!("Update Table Response: {:?}", resp);
            info!("Updated Table {:?}", update_request.table_name);
            Ok(())
        }
        false => {
            debug!("Update Table Response: {:?}", resp);
            let error_body = resp.json::<serde_json::Value>().await?;
            let error_message = error_body.as_str().unwrap_or("Unknown error");
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}
