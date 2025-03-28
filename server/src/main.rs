use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use core::{
    column::Column,
    request_types::{
        Condition, CreateRequests, CreateTableRequests, DropTableRequest, InsertColumnRequest,
        InsertRowRequest, RenameTableRequest, SelectRequest, UpdateRequest,
    },
    row::Row,
    table::Table,
    value::Value,
};
use log::{error, info, LevelFilter};
use std::io::Error;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::{signal::ctrl_c, spawn};

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format_timestamp_millis()
        .init();

    // Load application state from file or create new state
    let app_state: Arc<AppState> =
        Arc::new(AppState::load().await.unwrap_or_else(|_| AppState::new()));

    // Define routes and handlers
    let app = Router::new()
        .route("/", get(root))
        .route("/tables", get(get_tables))
        .route("/create", post(create))
        .route("/create_table", post(create_table))
        .route("/drop_table", post(drop_table))
        .route("/rename_table", post(rename_table))
        .route("/update_table", post(update_table))
        .route("/insert_column", post(insert_column))
        .route("/insert_row", post(insert_row))
        .route("/select", post(select))
        .with_state(Arc::clone(&app_state));

    // Start HTTP server
    let address = "0.0.0.0:3000";
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => {
            info!("Http service started running on https://{}", address);
            listener
        }
        Err(err) => {
            error!("Error: {}", err);
            return;
        }
    };

    let server_task = spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            error!("Server error: {}", err);
        }
    });

    // Handle Ctrl+C (SIGINT) to gracefully shut down the server
    let _ = spawn({
        let app_state = Arc::clone(&app_state);
        async move {
            ctrl_c().await.expect("Failed to listen for Ctrl+C");
            if let Err(err) = app_state.save().await {
                error!("Failed to save state: {}", err);
            }
        }
    })
    .await;

    // Wait for server task to finish (though it should run indefinitely until SIGINT)
    if let Err(err) = server_task.await {
        error!("Server task error: {}", err);
    }
}

/// Handler for root endpoint
async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tables = state.get_all().await;
    Html(format_tables_html(tables))
}

/// Format tables data into HTML
fn format_tables_html(tables: Vec<Table>) -> String {
    let mut html = String::new();

    html.push_str(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Database Tables</title>
            <style>
                body { font-family: Arial, sans-serif; }
                table { width: 100%; border-collapse: collapse; }
                th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
                th { background-color: #f2f2f2; }
             .label { font-size: 10px; color: #666; }
            </style>
        </head>
        <body>
            <h1>Database Tables</h1>
    "#,
    );

    for table in tables {
        html.push_str(&format!(
            r#"
            <h2>{}</h2>
            <table>
                <tr>
        "#,
            table.name
        ));

        for column in &table.columns {
            let mut labels = Vec::new();

            if column.primary_key {
                labels.push("Primary");
            }
            if column.unique {
                labels.push("Unique");
            }
            if column.non_null {
                labels.push("Non-Null");
            }

            let labels_str = labels.join(", ");

            html.push_str(&format!(r#"
                    <th style="border-right: 1px solid #ddd;"><span style="float: left;">{}</span><span class="label" style="float: right;">{}</span></th>
            "#, column.key, labels_str));
        }

        html.push_str(
            r#"
                </tr>
        "#,
        );

        for row in &table.rows {
            html.push_str(
                r#"
                <tr>
            "#,
            );

            for value in &row.values {
                html.push_str(&format!(
                    r#"
                    <td style="border-right: 1px solid #ddd;">{}</td>
                "#,
                    value.as_string().unwrap_or_default()
                ));
            }

            html.push_str(
                r#"
                </tr>
            "#,
            );
        }

        html.push_str(
            r#"
            </table>
        "#,
        );
    }

    html.push_str(
        r#"
            </body>
        </html>
    "#,
    );

    html
}

/// Handler to get all tables
async fn get_tables(State(state): State<Arc<AppState>>) -> Json<Vec<Table>> {
    let tables = state.get_all().await;
    let json = Json(tables);
    info!("Tables: {:?}", json);
    json
}

/// Handler to create a new table
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/create -H "Content-Type: application/json" -d '{"name":"test_table"}'
/// ```
///
/// Creates a new table in the database.
///
/// ## Parameters
///
/// - `name`: The name of the table to be created.
///
/// ## Returns
///
/// Returns a JSON object representing the newly created table.
///
/// ## Errors
///
/// - Returns an error if a table with the same name already exists.
async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRequests>,
) -> Response {
    let table_name = payload.name;

    if state.get(&table_name).await.is_some() {
        let error = format!("Table '{}' already exists", table_name);
        error!("{}", error);
        return (StatusCode::BAD_REQUEST, Json(error)).into_response();
    }

    let new_table = Table {
        name: table_name,
        columns: Vec::new(),
        rows: Vec::new(),
    };

    state.create(new_table.clone()).await;
    match state.save().await {
        Ok(_) => {
            info!("Created table: {:?}", &new_table);
            (StatusCode::OK, Json(new_table)).into_response()
        }
        Err(err) => {
            let error = format!("Failed to save state: {}", err);
            error!("{}", error);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// Handler to drop a table
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/drop_table -H "Content-Type: application/json" -d '{"name":"test_table"}'
/// ```
///
/// Drops a table from the database.
///
/// ## Parameters
///
/// - `name`: The name of the table to be dropped.
///
/// ## Returns
///
/// Returns a success message if the table is dropped successfully.
///
/// ## Errors
///
/// - Returns an error if the table does not exist.
async fn drop_table(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DropTableRequest>,
) -> Response {
    let table_name = payload.name;

    if state.drop_table(&table_name).await {
        match state.save().await {
            Ok(_) => {
                info!("Dropped table: {}", table_name);
                (
                    StatusCode::OK,
                    Json(format!("Dropped table '{}'", table_name)),
                )
                    .into_response()
            }
            Err(err) => {
                let error = format!("Failed to save state: {}", err);
                error!("{}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    } else {
        let error = format!("Table '{}' does not exist", table_name);
        error!("{}", error);
        (StatusCode::NOT_FOUND, Json(error)).into_response()
    }
}

/// Handler to rename a table's name
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/rename_table -H "Content-Type: application/json" -d '{"current_name":"test_table2","new_name":"test_drop_table"}'
/// ```
///
/// Renames a table in the database.
///
/// ## Parameters
///
/// - `current_name`: The current name of the table to be renamed.
/// - `new_name`: The new name of the table.
///
/// ## Returns
///
/// Returns a success message if the table is renamed successfully.
///
/// ## Errors
///
/// - Returns an error if the table does not exist.
async fn rename_table(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RenameTableRequest>,
) -> Response {
    let current_name = payload.current_name;
    let new_name = payload.new_name;

    if let Some(mut table) = state.get(&current_name).await {
        table.name = new_name;
        state.drop_table(&current_name).await;
        state.create(table.clone()).await;
        match state.save().await {
            Ok(_) => {
                info!(
                    "Rename table name from '{}' to '{}'",
                    current_name, table.name
                );
                (
                    StatusCode::OK,
                    Json(format!(
                        "Renamed table name from '{}' to '{}'",
                        current_name, table.name
                    )),
                )
                    .into_response()
            }
            Err(err) => {
                let error = format!("Failed to save state: {}", err);
                error!("{}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    } else {
        let error = format!("Table '{}' does not exist", current_name);
        error!("{}", error);
        (StatusCode::NOT_FOUND, Json(error)).into_response()
    }
}

/// Handler to insert a new column into a table
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/insert_column -H "Content-Type: application/json" -d '{"table_name":"test_table","key":"test_key","primary_key":true,"non_null":true,"unique":true,"foreign_key":null}'
/// ```
///
/// Inserts a new column into a table in the database.
///
/// ## Parameters
///
/// - `table_name`: The name of the table into which the column is to be inserted.
/// - `key`: The name of the column to be inserted.
/// - `primary_key`: Whether the column is a primary key.
/// - `non_null`: Whether the column is non-null.
/// - `unique`: Whether the column is unique.
/// - `foreign_key`: The foreign key constraint for the column.
///
/// ## Returns
///
/// Returns a JSON object representing the newly inserted column.
///
/// ## Errors
///
/// - Returns an error if the table does not exist.
async fn insert_column(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InsertColumnRequest>,
) -> Response {
    let table_name = payload.table_name;

    if let Some(mut table) = state.get(&table_name).await {
        let column = Column::new(
            payload.key,
            payload.primary_key,
            payload.non_null,
            payload.unique,
            payload
                .foreign_key
                .map(|fk| fk.into_iter().map(Box::new).collect()),
        );
        table.add_column(column.clone());
        state.drop_table(&table_name).await;
        state.create(table).await;
        match state.save().await {
            Ok(_) => {
                info!("Inserted column into table '{}': {:?}", table_name, column);
                (StatusCode::OK, Json(column)).into_response()
            }
            Err(err) => {
                let error = format!("Failed to save state: {}", err);
                error!("{}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    } else {
        let error = format!("Table '{}' does not exist", table_name);
        error!("{}", error);
        (StatusCode::NOT_FOUND, Json(error)).into_response()
    }
}

/// Handler to create a new table with specified columns
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/create_table -H "Content-Type: application/json" -d '{"name":"test_table2","insert_column_requests":[{"table_name":"test_table","key":"test_key3","primary_key":true,"non_null":false,"unique":true,"foreign_key":null}]}'
/// ```
///
/// Creates a new table with specified columns in the database.
///
/// ## Parameters
///
/// - `name`: The name of the table to be created.
/// - `insert_column_requests`: A list of column insertion requests.
///
/// ## Returns
///
/// Returns a JSON object representing the newly created table.
///
/// ## Errors
///
/// - Returns an error if a table with the same name already exists.
async fn create_table(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTableRequests>,
) -> impl IntoResponse {
    let table_name = payload.name;

    if state.get(&table_name).await.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(format!("Table '{}' already exists", table_name)),
        )
            .into_response();
    }

    let new_table = Table {
        name: table_name.clone(),
        columns: Vec::new(),
        rows: Vec::new(),
    };

    state.create(new_table.clone()).await;

    for insert_column_request in payload.insert_column_requests {
        let mut request = insert_column_request;
        request.table_name = table_name.clone();
        let json_payload = Json(request);

        let response = insert_column(State(state.clone()), json_payload).await;

        // Return immediately if there's an error
        if response.status() != StatusCode::OK {
            return response;
        }
    }

    match state.save().await {
        Ok(_) => {
            info!("Created table: {:?}", new_table);
            (StatusCode::OK, Json(new_table)).into_response()
        }
        Err(err) => {
            let error_message = format!("Failed to save state: {}", err);
            error!("{}", error_message);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message)).into_response()
        }
    }
}

/// Handler to insert a new row into a table
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/insert_row -H "Content-Type: application/json" -d '{"table_name":"test_table","row":{"values":[{"Str":"test_value"},{"Int":13}]}}'
/// ```
///
/// Inserts a new row into a table in the database.
///
/// ## Parameters
///
/// - `table_name`: The name of the table into which the row is to be inserted.
/// - `row`: The row to be inserted.
///
/// ## Returns
///
/// Returns a JSON object representing the newly inserted row.
///
/// ## Errors
///
/// - Returns an error if the table does not exist.
async fn insert_row(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InsertRowRequest>,
) -> Response {
    let table_name = payload.table_name;
    info!("Received insert request for table '{}'", table_name);

    if let Some(mut table) = state.get(&table_name).await {
        let mut row = payload.row;
        info!("Inserting row: {:?}", row);

        let columns_len = table.columns.len();
        info!("Table '{}' expects {} columns", table_name, columns_len);

        if row.values.len() > columns_len {
            let error = format!(
                "Row has {} values, but table expects {} values consider adding more columns",
                row.values.len(),
                columns_len
            );
            error!("{}", error);
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }

        if row.values.len() < columns_len {
            // Check if column allows Non-Null
            let additional_rows = columns_len - row.values.len();
            // if any additional columns are non_null return with an error
            if table
                .columns
                .iter()
                .rev()
                .take(additional_rows)
                .any(|col| col.non_null)
            {
                let error = format!("Row has {} values, but table expects {} values. This fails out because at least one additional column is Non-Null", row.values.len(), columns_len);
                error!("{}", error);
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            } else {
                for _ in 0..additional_rows {
                    row.add_value(None)
                }
            }
        }

        let row_values = row
            .values
            .iter()
            .map(|value| value.as_string().unwrap_or_default())
            .collect::<Vec<String>>();
        table.add_row(row.clone());
        state.drop_table(&table_name).await;
        state.create(table).await;

        // Handle the Result from state.save() manually
        match state.save().await {
            Ok(_) => {
                info!("Inserted row into table '{}': {:?}", table_name, row);
                (StatusCode::OK, Json(row_values)).into_response()
            }
            Err(err) => {
                let error_message = format!("Failed to save state: {}", err);
                error!("{}", error_message);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message)).into_response()
            }
        }
    } else {
        let error = format!("Table '{}' does not exist", table_name);
        error!("{}", error);
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }
}

/// Handler to select rows from a table based on specified conditions or retrieve all rows if no conditions are provided.
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/select -H "Content-Type: application/json" -d '{"table_name":"test_table","columns":["test_key","test_key3"],"condition":{"column":"test_key","value":"true"}}'
/// ```
///
/// Retrieves rows from the specified table (`table_name`) optionally filtered by columns (`columns`) and a conditional (`condition`).
/// If `columns` is not provided, all columns are selected (`SELECT *`).
///
/// ## Parameters
///
/// - `table_name`: Name of the table from which rows are selected.
/// - `columns`: Optional. List of columns to select. If not provided, all columns are selected.
/// - `condition`: Optional. Specifies a condition to filter rows. Only rows matching this condition are returned.
///
/// ## Returns
///
/// Returns a JSON array of rows, where each row is represented as an array of strings (values of selected columns).
///
/// ## Errors
///
/// - Returns an error if the specified `table_name` does not exist in the application state.
/// - Returns an error if the specified `condition.column` does not exist in the table.
///
/// ## Notes
///
/// - This handler supports flexible column selection and row filtering based on conditions.
///
async fn select(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SelectRequest>,
) -> Response {
    if let Some(table) = state.get(payload.table_name.as_str()).await {
        let rows = select_rows(&table, payload.columns, payload.condition.as_ref()).await;

        match rows {
            Ok(rows) => (StatusCode::OK, Json(rows)).into_response(),
            Err(error) => {
                error!("{}", error);
                (StatusCode::BAD_REQUEST, Json(error)).into_response()
            }
        }
    } else {
        let error = format!("Table '{}' does not exist", payload.table_name);
        error!("{}", error);
        (StatusCode::NOT_FOUND, Json(error)).into_response()
    }
}

/// Helper function to select rows from a table based on specified conditions
/// ## Parameters
///
/// - `table_name`: Name of the table from which rows are selected.
/// - `columns`: Optional. List of columns to select. If not provided, all columns are selected.
/// - `condition`: Optional. Specifies a condition to filter rows. Only rows matching this condition are returned.
///
/// ## Returns
///
/// Returns a JSON array of rows, where each row is represented as an array of strings (values of selected columns).
///
/// ## Errors
///
/// - Returns an error if the specified `table_name` does not exist in the application state.
/// - Returns an error if the specified `condition.column` does not exist in the table.
async fn select_rows(
    table: &Table,
    columns: Option<Vec<String>>,
    condition: Option<&Condition>,
) -> Result<Vec<Row>, String> {
    let mut rows = vec![];

    for row in &table.rows {
        if let Some(cond) = condition {
            if let Some(col_index) = table.columns.iter().position(|col| col.key == cond.column) {
                if row.values[col_index].as_string().unwrap_or_default() != cond.value {
                    continue;
                }
            } else {
                return Err(format!("Column '{}' not found", cond.column));
            }
        }

        let mut selected_row = Row::new(vec![]);

        if let Some(ref cols) = columns {
            for col in cols {
                if let Some(col_index) = table.columns.iter().position(|c| c.key == *col) {
                    selected_row.add_value(row.values[col_index].clone());
                } else {
                    return Err(format!("Column '{}' not found", col));
                }
            }
        } else {
            // SELECT *
            for value in &row.values {
                selected_row.add_value(value.clone());
            }
        }

        rows.push(selected_row);
    }

    info!("Selected Rows: {:?}", rows);
    Ok(rows)
}

/// Handler to update rows in a table based on specified conditions
///
/// # Example
///
/// ```
/// curl -X POST http://localhost:3000/update_table -H "Content-Type: application/json" -d '{"table_name":"test_table","condition":{"column":"test_key","value":"true"},"updates":[{"column":"test_key3","value":"updated_value"},{"column":"test_key2","value":"17.78"}]}'
/// ```
///
/// Updates rows in the specified table (`table_name`) optionally filtered by a condition (`condition`).
///
/// ## Parameters
///
/// - `table_name`: Name of the table from which rows are updated.
/// - `condition`: Optional. Specifies a condition to filter rows. Only rows matching this condition are updated.
/// - `updates`: List of updates to apply to the filtered rows. Each update specifies a column and a new value.
///
/// ## Returns
///
/// Returns a success message if the update is successful.
///
/// ## Errors
///
/// - Returns an error if the specified `table_name` does not exist in the application state.
/// - Returns an error if the specified `condition.column` does not exist in the table.
/// - Returns an error if any of the `updates` specify a column that does not exist in the table.
///
/// ## Notes
///
/// - This handler supports flexible row filtering based on conditions and updates multiple columns at once.
async fn update_table(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateRequest>,
) -> Response {
    if let Some(mut table) = state.get(payload.table_name.as_str()).await {
        // Fetch rows that match the condition
        let rows = select_rows(&table, None, payload.condition.as_ref()).await;

        match rows {
            Ok(mut selected_rows) => {
                // Update logic for rows that match the condition
                for row in &mut selected_rows {
                    for update in &payload.updates {
                        if let Some(col_index) = table
                            .columns
                            .iter()
                            .position(|col| col.key == update.column)
                        {
                            row.values[col_index] = Value::from(update.value.clone());
                        } else {
                            let error = format!("Column '{}' not found", update.column);
                            error!("{}", error);
                            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
                        }
                    }
                }

                // Apply the updates back to the original table rows
                for row in &mut table.rows {
                    if let Some(condition) = &payload.condition {
                        if let Some(col_index) = table
                            .columns
                            .iter()
                            .position(|col| col.key == condition.column)
                        {
                            if row.values[col_index].as_string().unwrap_or_default()
                                == condition.value
                            {
                                for update in &payload.updates {
                                    if let Some(update_col_index) = table
                                        .columns
                                        .iter()
                                        .position(|col| col.key == update.column)
                                    {
                                        row.values[update_col_index] =
                                            Value::from(update.value.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                // Drop the current table
                state.drop_table(&*payload.table_name.clone()).await;

                // Create the updated table
                let updated_table = Table {
                    name: payload.table_name.clone(),
                    columns: table.columns.clone(),
                    rows: table.rows.clone(),
                };

                // Create the updated table in the state
                state.create(updated_table).await;

                info!("Updated Rows: {:?}", selected_rows);
                (StatusCode::OK, Json("Rows updated successfully")).into_response()
            }
            Err(error) => {
                error!("{}", error);
                (StatusCode::BAD_REQUEST, Json(error)).into_response()
            }
        }
    } else {
        let error = format!("Table '{}' does not exist", payload.table_name);
        error!("{}", error);
        (StatusCode::NOT_FOUND, Json(error)).into_response()
    }
}

/// Application state holding tables
#[derive(Clone)]
struct AppState {
    tables: Arc<Mutex<Vec<Table>>>,
}

impl AppState {
    /// Create a new instance of AppState
    pub fn new() -> Self {
        AppState {
            tables: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Load application state from file
    pub async fn load() -> Result<Self, Error> {
        let file = File::open("db.json")
            .await
            .map_err(|_| Error::new(io::ErrorKind::NotFound, "File not found"))?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).await?;
        let tables: Vec<Table> = serde_json::from_str(&contents)?;
        Ok(AppState {
            tables: Arc::new(Mutex::new(tables)),
        })
    }

    /// Save application state to file
    pub async fn save(&self) -> Result<(), Error> {
        let tables = self.get_all().await;
        let contents = serde_json::to_string(&tables)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("db.json")
            .await?;
        let mut writer = io::BufWriter::new(file);
        writer.write_all(contents.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    /// Add a new table to the application state
    pub async fn create(&self, table: Table) {
        let mut lock = self.tables.lock().await;
        lock.push(table);
    }

    /// Get all tables from the application state
    pub async fn get_all(&self) -> Vec<Table> {
        let lock = self.tables.lock().await;
        lock.iter().cloned().collect()
    }

    /// Get a specific table from the application state by name
    pub async fn get(&self, table_name: &str) -> Option<Table> {
        let lock = self.tables.lock().await;
        lock.iter().find(|table| table.name == table_name).cloned()
    }

    /// Drop a table from the application state by name
    pub async fn drop_table(&self, table_name: &str) -> bool {
        let mut lock = self.tables.lock().await;
        if let Some(index) = lock.iter().position(|table| table.name == table_name) {
            lock.remove(index);
            true
        } else {
            false
        }
    }
}
