use core::client_functions::*;
use core::request_types::{
    Condition, CreateRequests, CreateTableRequests, DropTableRequest, InsertColumnRequest,
    InsertRowRequest, RenameTableRequest, SelectRequest, UpdateColumnRequest, UpdateRequest,
};
use core::row::Row;
use core::value::Value;
use log::{error, LevelFilter};
use reqwest::Client;

/// This main function demonstrates the usage of various client functions with example values.
///
/// The following client functions are demonstrated:
///
/// - `drop_table`: Drops a table from the database.
/// - `create`: Creates a new table in the database.
/// - `insert_column`: Inserts a new column into an existing table.
/// - `insert_row`: Inserts a new row into an existing table.
/// - `select`: Selects rows from a table based on a condition.
/// - `update_table`: Updates rows in a table based on a condition.
///
/// The example values used in this function are:
///
/// - `test_table`: The name of the table to be created, dropped, and updated.
/// - `test_key`, `test_key2`, `test_key3`: The names of the columns to be inserted into the table.
/// - `test_value`, `test_value_3`: The values to be inserted into the table.
/// - `13`, `27.55`, `128`, `17.78`: The values to be inserted into the table.
///
/// The following curl commands do exactly the same as the rust code in this function
/// ```sh
/// curl -X POST http://localhost:3000/drop_table -H "Content-Type: application/json" -d '{"name":"test_table"}'
/// curl -X POST http://localhost:3000/drop_table -H "Content-Type: application/json" -d '{"name":"test_table2"}'
/// curl -X POST http://localhost:3000/drop_table -H "Content-Type: application/json" -d '{"name":"test_drop_table"}'
/// curl -X POST http://localhost:3000/create -H "Content-Type: application/json" -d '{"name":"test_table"}'
/// curl -X POST http://localhost:3000/insert_column -H "Content-Type: application/json" -d '{"table_name":"test_table","key":"test_key","primary_key":true,"non_null":true,"unique":true,"foreign_key":null}'
/// curl -X POST http://localhost:3000/insert_column -H "Content-Type: application/json" -d '{"table_name":"test_table","key":"test_key2","primary_key":true,"non_null":true,"unique":true,"foreign_key":null}'
/// curl -X POST http://localhost:3000/insert_column -H "Content-Type: application/json" -d '{"table_name":"test_table","key":"test_key3","primary_key":true,"non_null":false,"unique":true,"foreign_key":null}'
/// curl -X POST http://localhost:3000/create_table -H "Content-Type: application/json" -d '{"name":"test_table2","insert_column_requests":[{"table_name":"test_table","key":"test_key3","primary_key":true,"non_null":false,"unique":true,"foreign_key":null}]}'
/// curl -X POST http://localhost:3000/rename_table -H "Content-Type: application/json" -d '{"current_name":"test_table2","new_name":"test_drop_table"}'
/// curl -X POST http://localhost:3000/drop_table -H "Content-Type: application/json" -d '{"name":"test_drop_table"}'
/// curl -X POST http://localhost:3000/insert_row -H "Content-Type: application/json" -d '{"table_name":"test_table","row":{"values":[{"Str":"test_value"},{"Int":13}]}}'
/// curl -X POST http://localhost:3000/insert_row -H "Content-Type: application/json" -d '{"table_name":"test_table","row":{"values":[{"Bool":true},{"Float":27.55},{"Int":128}]}}'
/// curl -X POST http://localhost:3000/insert_row -H "Content-Type: application/json" -d '{"table_name":"test_table","row":{"values":[{"Str":"test_value_3"},{"Float":17.78}]}}'
/// curl -X POST http://localhost:3000/select -H "Content-Type: application/json" -d '{"table_name":"test_table","columns":["test_key","test_key3"],"condition":null}'
/// curl -X POST http://localhost:3000/select -H "Content-Type: application/json" -d '{"table_name":"test_table","columns":["test_key","test_key3"],"condition":{"column":"test_key","value":"true"}}'
/// curl -X POST http://localhost:3000/update_table -H "Content-Type: application/json" -d '{"table_name":"test_table","condition":{"column":"test_key","value":"true"},"updates":[{"column":"test_key3","value":"updated_value"},{"column":"test_key2","value":"17.78"}]}'
/// ```
#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format_timestamp_millis()
        .init();

    let client = Client::new();

    if let Err(e) = client.post("http://localhost:3000").send().await {
        error!("Error, is the server on? :{}", e);
        return;
    }

    // Drop previous tables
    drop_table(
        &client,
        &DropTableRequest {
            name: "test_table".to_string(),
        },
    )
    .await
    .unwrap();
    drop_table(
        &client,
        &DropTableRequest {
            name: "test_table2".to_string(),
        },
    )
    .await
    .unwrap();
    drop_table(
        &client,
        &DropTableRequest {
            name: "test_drop_table".to_string(),
        },
    )
    .await
    .unwrap();

    // Create a table
    create(
        &client,
        &CreateRequests {
            name: "test_table".to_string(),
        },
    )
    .await
    .unwrap();

    // Insert columns
    let insert_column_request = InsertColumnRequest {
        table_name: "test_table".to_string(),
        key: "test_key".to_string(),
        primary_key: true,
        non_null: true,
        unique: true,
        foreign_key: None,
    };

    let insert_column_request2 = InsertColumnRequest {
        table_name: "test_table".to_string(),
        key: "test_key2".to_string(),
        primary_key: true,
        non_null: true,
        unique: true,
        foreign_key: None,
    };

    let insert_column_request3 = InsertColumnRequest {
        table_name: "test_table".to_string(),
        key: "test_key3".to_string(),
        primary_key: true,
        non_null: false,
        unique: true,
        foreign_key: None,
    };

    insert_column(&client, &insert_column_request)
        .await
        .unwrap();
    insert_column(&client, &insert_column_request2)
        .await
        .unwrap();
    insert_column(&client, &insert_column_request3)
        .await
        .unwrap();

    // Create new table to be dropped
    create_table(
        &client,
        &CreateTableRequests {
            name: "test_table2".to_string(),
            insert_column_requests: vec![insert_column_request3],
        },
    )
    .await
    .unwrap();

    rename_table(
        &client,
        &RenameTableRequest {
            current_name: "test_table2".to_string(),
            new_name: "test_drop_table".to_string(),
        },
    )
    .await
    .unwrap();

    // Drop the table
    drop_table(
        &client,
        &DropTableRequest {
            name: "test_drop_table".to_string(),
        },
    )
    .await
    .unwrap();

    // Insert a row
    let insert_row_request = InsertRowRequest {
        table_name: "test_table".to_string(),
        row: Row::new(vec![Value::from("test_value".to_string()), Value::from(13)]),
    };

    insert_row(&client, &insert_row_request).await.unwrap();

    // Insert a row
    let insert_row_request = InsertRowRequest {
        table_name: "test_table".to_string(),
        row: Row::new(vec![
            Value::from(true),
            Value::from(27.55),
            Value::from(128),
        ]),
    };

    insert_row(&client, &insert_row_request).await.unwrap();

    // Insert a row
    let insert_row_request = InsertRowRequest {
        table_name: "test_table".to_string(),
        row: Row::new(vec![
            Value::from("test_value_3".to_string()),
            Value::from(17.78),
        ]),
    };

    insert_row(&client, &insert_row_request).await.unwrap();

    // Select from the table without a condition
    let select_request = SelectRequest {
        table_name: "test_table".to_string(),
        columns: Option::from(vec!["test_key".to_string(), "test_key3".to_string()]), // Empty vec would mean *
        condition: None, // Add conditions if needed
    };

    select(&client, &select_request).await.unwrap();

    // Select from the table with a condition
    let select_request = SelectRequest {
        table_name: "test_table".to_string(),
        columns: Option::from(vec!["test_key".to_string(), "test_key3".to_string()]), // Empty vec would mean *
        condition: Option::from(Condition {
            column: "test_key".to_string(),
            value: "true".to_string(),
        }),
    };

    select(&client, &select_request).await.unwrap();

    // Update rows in the table
    let update_request = UpdateRequest {
        table_name: "test_table".to_string(),
        condition: Option::from(Condition {
            column: "test_key".to_string(),
            value: "true".to_string(),
        }),
        updates: vec![
            UpdateColumnRequest {
                column: "test_key3".to_string(),
                value: "updated_value".to_string(),
            },
            UpdateColumnRequest {
                column: "test_key2".to_string(),
                value: "17.78".to_string(),
            },
        ],
    };

    update_table(&client, &update_request).await.unwrap();
}
