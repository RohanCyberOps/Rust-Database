use core::client_functions::*;
use core::request_types::*;
use core::row::Row;
use core::value::Value;
use log::{debug, error, info, LevelFilter};
use reqwest::Client;
use std::io::{self, Write};

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

    greet_and_list_operations();

    loop {
        println!(" ");
        println!("Enter a command: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match parse_and_execute_command(&client, input).await {
            Ok(_) => {
                info!("Operation successful! You can view the results at http://localhost:3000/");
            }
            Err(e) => {
                error!("{}", e);
                print_syntax_example();
            }
        }
    }
}

/// Greets the user and lists available operations.
fn greet_and_list_operations() {
    info!("Welcome to the Interactive Database Client!");
    println!("Available operations:");
    println!("1. CREATE TABLE table_name");
    println!("2. INSERT INTO table_name (column1, column2) VALUES (value1, value2)");
    println!("3. SELECT column1, column2 FROM table_name WHERE condition");
    println!("4. UPDATE table_name SET column1 = value1 WHERE condition");
    println!("5. RENAME TABLE old_table_name TO new_table_name");
    println!("6. DROP TABLE table_name");
    println!("Type 'exit' to quit.");
}

/// Prints syntax examples for each operation.
fn print_syntax_example() {
    info!("\nExample Syntax:");

    // Example for CREATE TABLE
    println!("1. CREATE TABLE table_name (column1 TYPE, column2 TYPE, ...)");
    println!("   Example: CREATE TABLE users (id INT, name STRING, email STRING)");

    // Example for INSERT INTO
    println!("2. INSERT INTO table_name (column1, column2, ...) VALUES (value1, value2, ...)");
    println!(
        "   Example: INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com')"
    );

    // Example for SELECT
    println!("3. SELECT column1, column2, ... FROM table_name [WHERE condition]");
    println!("   Example: SELECT id, name FROM users WHERE email = 'alice@example.com'");

    // Example for UPDATE
    println!("4. UPDATE table_name SET column1 = value1, column2 = value2, ... [WHERE condition]");
    println!("   Example: UPDATE users SET name = 'Alice Smith' WHERE id = 1");

    // Example for RENAME TABLE
    println!("5. RENAME TABLE old_table_name TO new_table_name");
    println!("   Example: RENAME TABLE users TO customers");

    // Example for DROP TABLE
    println!("6. DROP TABLE table_name");
    println!("   Example: DROP TABLE customers");
}

/// Parses and executes a command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `command`: The command to parse and execute.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
async fn parse_and_execute_command(client: &Client, command: &str) -> Result<(), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Invalid command".into());
    }

    match parts[0].to_uppercase().as_str() {
        "CREATE" => create_table_command(client, parts, command).await,
        "INSERT" => insert_into_command(client, command).await,
        "SELECT" => select_command(client, command).await,
        "UPDATE" => update_command(client, command).await,
        "RENAME" => rename_table_command(client, parts).await,
        "DROP" => drop_table_command(client, parts).await,
        "EXIT" => exit_command(),
        _ => Err("Unknown command".into()),
    }
}

/// Handles the CREATE TABLE command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `parts`: The parts of the command.
/// - `command`: The full command.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
///
/// # Example
///
/// ```
/// CREATE TABLE users (id INT, name STRING, email STRING)
/// ```
async fn create_table_command(
    client: &Client,
    parts: Vec<&str>,
    command: &str,
) -> Result<(), String> {
    if parts.get(1).map(|s| *s) == Some("TABLE") {
        if let (Some(table_name), Some(columns_part)) = (
            parts.get(2),
            command.split('(').nth(1).and_then(|s| s.split(')').next()),
        ) {
            let columns: Vec<&str> = columns_part.split(',').map(|s| s.trim()).collect();
            let column_requests: Result<Vec<InsertColumnRequest>, String> = columns
                .iter()
                .map(|&col| {
                    let parts: Vec<&str> = col.split_whitespace().collect();
                    if parts.len() < 2 {
                        return Err("Syntax error in column definition".into());
                    }
                    let column_name = parts[0].to_string();
                    let column_type = parts[1].to_uppercase();
                    let (primary_key, non_null, unique) = (false, false, false);
                    let (_column_type, foreign_key) = match column_type.as_str() {
                        "INT" => (Value::Int(0), None),
                        "FLOAT" => (Value::Float(0.0), None),
                        "STRING" => (Value::Str("".to_string()), None),
                        "BOOL" => (Value::Bool(false), None),
                        _ => return Err("Unsupported column type".into()),
                    };
                    Ok(InsertColumnRequest {
                        table_name: table_name.to_string(),
                        key: column_name,
                        primary_key,
                        non_null,
                        unique,
                        foreign_key,
                    })
                })
                .collect();
            let column_requests = column_requests?;
            let request = CreateTableRequests {
                name: table_name.to_string(),
                insert_column_requests: column_requests,
            };
            create_table(client, &request)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("Syntax error: CREATE TABLE table_name (column_definitions)".into())
        }
    } else {
        Err("Syntax error: CREATE TABLE table_name (column_definitions)".into())
    }
}

/// Handles the INSERT INTO command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `command`: The full command.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
///
/// # Example
///
/// ```
/// INSERT INTO users (id, name, email) VALUES (1, "Alice", "alice@example.com")
/// ```
async fn insert_into_command(client: &Client, command: &str) -> Result<(), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.get(1).map(|s| *s) == Some("INTO") {
        if let (Some(table_name), Some(values_part)) =
            (parts.get(2), command.split("VALUES").nth(1))
        {
            let columns_start = command
                .find('(')
                .ok_or("Syntax error: Missing opening parenthesis")?;
            let columns_end = command
                .find(')')
                .ok_or("Syntax error: Missing closing parenthesis")?;
            let columns_str = &command[columns_start + 1..columns_end].trim();
            let columns: Vec<&str> = columns_str.split(',').map(|s| s.trim()).collect();

            let values_start = values_part
                .find('(')
                .ok_or("Syntax error: Missing opening parenthesis for values")?;
            let values_end = values_part
                .find(')')
                .ok_or("Syntax error: Missing closing parenthesis for values")?;
            let values_str = &values_part[values_start + 1..values_end].trim();
            let values: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();

            if columns.len() != values.len() {
                return Err("Column count does not match value count".into());
            }

            let row_values: Vec<Value> = values
                .iter()
                .map(|&v| {
                    if v == "NULL" {
                        Value::Null
                    } else if let Ok(val) = v.parse::<i64>() {
                        Value::Int(val)
                    } else if let Ok(val) = v.parse::<f64>() {
                        Value::Float(val)
                    } else if v == "true" {
                        Value::Bool(true)
                    } else if v == "false" {
                        Value::Bool(false)
                    } else {
                        Value::Str(v.trim_matches(|c| c == '"' || c == '\'').to_string())
                    }
                })
                .collect();

            let row = Row::new(row_values);
            let request = InsertRowRequest {
                table_name: table_name.to_string(),
                row,
            };
            insert_row(client, &request)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("Syntax error: INSERT INTO table_name (columns) VALUES (values)".into())
        }
    } else {
        Err("Syntax error: INSERT INTO table_name (columns) VALUES (values)".into())
    }
}

/// Handles the SELECT command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `command`: The full command.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
///
/// # Example
///
/// ```
/// SELECT id, name FROM users WHERE email = "alice@example.com"
/// ```
async fn select_command(client: &Client, command: &str) -> Result<(), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut parts_iter = parts.iter().skip(1); // Skip the "SELECT" keyword

    let columns_part_vec: Vec<&str> = parts_iter
        .by_ref()
        .take_while(|&&s| s.to_uppercase() != "FROM")
        .map(|&s| s)
        .collect();
    let columns_part = columns_part_vec.join(" ");
    if columns_part.is_empty() {
        return Err("Syntax error: Missing columns".into());
    }
    let columns: Vec<&str> = columns_part.split(',').map(|s| s.trim()).collect();

    let from_keyword_index = parts
        .iter()
        .position(|&s| s.to_uppercase() == "FROM")
        .ok_or("Syntax error: Missing FROM keyword")?;
    let table_name = parts
        .get(from_keyword_index + 1)
        .ok_or("Syntax error: Missing table name")?;

    let mut condition = None;

    if let Some(where_index) = parts.iter().position(|&s| s.to_uppercase() == "WHERE") {
        let where_parts: Vec<&str> = parts.iter().skip(where_index + 1).map(|s| *s).collect();

        if where_parts.len() < 3 {
            return Err("Syntax error: Incomplete WHERE clause".into());
        }

        let condition_column = where_parts[0].to_string();
        let _operator = where_parts[1].to_string();
        let condition_value = where_parts[2..].join(" ");

        let condition_value = condition_value
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string();

        condition = Some(Condition {
            column: condition_column,
            value: condition_value,
        });
    }

    let request = SelectRequest {
        table_name: table_name.to_string(),
        columns: Some(columns.iter().map(|&col| col.to_string()).collect()),
        condition,
    };

    debug!("SelectRequest: {:?}", request);

    select(client, &request).await.map_err(|e| e.to_string())
}

/// Handles the UPDATE command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `command`: The full command.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
///
/// # Example
///
/// ```
/// UPDATE users SET name = "Alicia" WHERE email = "alice@example.com"
/// ```
async fn update_command(client: &Client, command: &str) -> Result<(), String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if let (Some(table_name), Some(set_index)) = (
        parts.get(1),
        parts.iter().position(|&s| s.to_uppercase() == "SET"),
    ) {
        let condition_index = parts
            .iter()
            .position(|&s| s.to_uppercase() == "WHERE")
            .unwrap_or(parts.len());
        let updates_str = parts[set_index + 1..condition_index].join(" ");
        let updates: Vec<&str> = updates_str.split(',').map(|s| s.trim()).collect();

        let updates: Result<Vec<UpdateColumnRequest>, String> = updates
            .iter()
            .map(|&update| {
                let key_value: Vec<&str> = update.split('=').map(|s| s.trim()).collect();
                if key_value.len() != 2 {
                    return Err("Syntax error in UPDATE clause".into());
                }
                Ok(UpdateColumnRequest {
                    column: key_value[0].to_string(),
                    value: key_value[1]
                        .trim_matches(|c| c == '"' || c == '\'')
                        .to_string(),
                })
            })
            .collect();

        let condition = if condition_index < parts.len() {
            // Collect condition parts after the WHERE clause
            let where_parts: Vec<&str> =
                parts.iter().skip(condition_index + 1).map(|s| *s).collect();
            if where_parts.len() < 3 {
                return Err("Syntax error: Incomplete WHERE clause".into());
            }

            let condition_column = where_parts[0].to_string();
            let _operator = where_parts[1].to_string(); // The operator is not used here, but could be parsed
            let condition_value = where_parts[2..].join(" ").trim_matches('"').to_string();

            Some(Condition {
                column: condition_column,
                value: condition_value,
            })
        } else {
            None
        };

        let request = UpdateRequest {
            table_name: table_name.to_string(),
            condition,
            updates: updates?,
        };

        // Print request for debugging
        debug!("UpdateRequest: {:?}", request);

        // Execute the request
        update_table(client, &request)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(
            "Syntax error: UPDATE table_name SET column=value[,column2=value2] [WHERE condition]"
                .into(),
        )
    }
}

/// Handles the RENAME TABLE command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `parts`: The parts of the command.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
///
/// # Example
///
/// ```
/// RENAME TABLE users TO customers
/// ```
async fn rename_table_command(client: &Client, parts: Vec<&str>) -> Result<(), String> {
    if let (Some(old_name), Some(new_name)) = (parts.get(2), parts.get(4)) {
        let request = RenameTableRequest {
            current_name: old_name.to_string(),
            new_name: new_name.to_string(),
        };
        rename_table(client, &request)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Syntax error: RENAME TABLE old_table_name TO new_table_name".into())
    }
}

/// Handles the DROP TABLE command.
///
/// # Parameters
///
/// - `client`: The HTTP client.
/// - `parts`: The parts of the command.
///
/// # Returns
///
/// Returns a `Result` indicating whether the command was executed successfully.
///
/// # Example
///
/// ```
/// DROP TABLE customers
/// ```
async fn drop_table_command(client: &Client, parts: Vec<&str>) -> Result<(), String> {
    if parts.get(1).map(|s| *s) == Some("TABLE") {
        if let Some(table_name) = parts.get(2) {
            let request = DropTableRequest {
                name: table_name.to_string(),
            };
            drop_table(client, &request)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("Syntax error: DROP TABLE table_name".into())
        }
    } else {
        Err("Syntax error: DROP TABLE table_name".into())
    }
}

/// Exits the program.
///
/// # Returns
///
/// Returns a `Result` indicating whether the exit was successful.
///
/// # Example
///
/// ```
/// EXIT
/// ```
fn exit_command() -> Result<(), String> {
    std::process::exit(0);
}
