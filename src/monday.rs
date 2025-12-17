use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
struct MondayRequest {
    query: String,
}

#[derive(Debug, Deserialize)]
struct MondayResponse {
    data: Option<MondayData>,
    #[serde(default)]
    errors: Vec<MondayError>,
}

#[derive(Debug, Deserialize)]
struct MondayData {
    me: Option<MondayUser>,
    boards: Option<Vec<Board>>,
    items: Option<Vec<Item>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MondayUser {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Board {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    #[serde(skip_serializing)]
    pub name: Option<String>,
    pub groups: Option<Vec<Group>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Group {
    #[serde(deserialize_with = "deserialize_string_id")]
    pub id: String,
    pub title: String,
    pub items_page: Option<ItemsPage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemsPage {
    #[serde(default)]
    pub items: Vec<Item>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Item {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub column_values: Vec<ColumnValue>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ColumnValue {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MondayError {
    message: String,
    #[serde(default)]
    error_code: String,
}

pub struct MondayClient {
    client: Client,
    api_key: String,
}

// Custom deserializer to handle both string and integer IDs
fn deserialize_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Number(num) => num
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("Invalid integer ID")),
        Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| serde::de::Error::custom("Invalid string ID")),
        _ => Err(serde::de::Error::custom("ID must be a string or integer")),
    }
}

// Custom deserializer for string IDs that might come as numbers
fn deserialize_string_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Number(num) => Ok(num.to_string()),
        Value::String(s) => Ok(s),
        Value::Null => Ok("null".to_string()),
        _ => Err(serde::de::Error::custom("ID must be a string or number")),
    }
}

// Helper function to extract items from the JSON response
fn extract_items_from_response(value: &Value) -> Result<(Vec<Item>, Option<String>)> {
    let mut items = Vec::new();
    let mut cursor = None;

    // Navigate through the nested structure: data -> boards -> groups -> items_page -> items
    if let Some(data) = value.get("data") {
        if let Some(boards) = data.get("boards").and_then(|b| b.as_array()) {
            for board in boards {
                if let Some(groups) = board.get("groups").and_then(|g| g.as_array()) {
                    for group in groups {
                        if let Some(items_page) = group.get("items_page") {
                            // Extract cursor
                            if let Some(cursor_val) =
                                items_page.get("cursor").and_then(|c| c.as_str())
                            {
                                if !cursor_val.is_empty() {
                                    cursor = Some(cursor_val.to_string());
                                }
                            }

                            // Extract items
                            if let Some(items_array) =
                                items_page.get("items").and_then(|i| i.as_array())
                            {
                                for item_val in items_array {
                                    let item = parse_item(item_val)?;
                                    items.push(item);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((items, cursor))
}

// Helper function to parse an individual item from JSON
fn parse_item(item_val: &Value) -> Result<Item> {
    let mut item = Item::default();

    // Extract item ID
    if let Some(id) = item_val.get("id").and_then(|id| id.as_str()) {
        item.id = Some(id.to_string());
    }

    // Extract item name
    if let Some(name) = item_val.get("name").and_then(|name| name.as_str()) {
        item.name = Some(name.to_string());
    }

    // Extract column values
    if let Some(columns_array) = item_val.get("column_values").and_then(|c| c.as_array()) {
        for col_val in columns_array {
            let mut column = ColumnValue::default();

            // Extract column ID
            if let Some(col_id) = col_val.get("id").and_then(|id| id.as_str()) {
                column.id = Some(col_id.to_string());
            }

            // Extract column value
            if let Some(value) = col_val.get("value") {
                if value.is_string() {
                    if let Some(value_str) = value.as_str() {
                        column.value = Some(value_str.to_string());
                    }
                } else if value.is_null() {
                    column.value = Some("null".to_string());
                } else {
                    column.value = Some(value.to_string());
                }
            }

            // Extract column text
            if let Some(text) = col_val.get("text") {
                if text.is_string() {
                    if let Some(text_str) = text.as_str() {
                        column.text = Some(text_str.to_string());
                    }
                } else if text.is_null() {
                    column.text = Some("null".to_string());
                } else {
                    column.text = Some(text.to_string());
                }
            }

            item.column_values.push(column);
        }
    }

    Ok(item)
}

impl MondayClient {
    pub fn new(api_key: String) -> Self {
        MondayClient {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_current_user_verbose(&self, verbose: bool) -> Result<MondayUser> {
        let query = r#"
        {
            me {
                id
                name
                email
            }
        }
        "#;

        if verbose {
            println!("Sending user query:\n{}", query);
        }

        let request_body = MondayRequest {
            query: query.to_string(),
        };

        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!(
                "User API response: {}",
                &response[..200.min(response.len())]
            );
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse Monday.com user response: {}", e))?;

        // Check for API errors
        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        monday_response
            .data
            .and_then(|data| data.me)
            .ok_or_else(|| anyhow!("No user data found in response"))
    }

    pub async fn query_board_verbose(
        &self,
        board_id: &str,
        group_name: &str,
        user_id: i64,
        limit: usize,
        verbose: bool,
    ) -> Result<Board> {
        // First, let's get all groups to find the right one
        if verbose {
            println!("Getting board structure to find groups...");
        }

        let query = format!(
            r#"
        {{
            boards(ids: ["{}"]) {{
                id
                name
                groups {{
                    id
                    title
                }}
            }}
        }}
        "#,
            board_id
        );

        if verbose {
            println!("Sending board query:\n{}", query);
        }

        let request_body = MondayRequest { query };
        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!("Board response: {}", &response[..500.min(response.len())]);
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse board response: {}", e))?;

        // Check for API errors
        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        let board = monday_response
            .data
            .and_then(|data| data.boards)
            .and_then(|mut boards| boards.pop())
            .ok_or_else(|| anyhow!("No board found with ID {}", board_id))?;

        // Find the group with the matching name
        let group_id = board
            .groups
            .as_ref()
            .ok_or_else(|| anyhow!("No groups found in board"))?
            .iter()
            .find(|group| group.title == group_name)
            .map(|group| group.id.clone())
            .ok_or_else(|| anyhow!("Group '{}' not found in board {}", group_name, board_id))?;

        if verbose {
            println!("Found group '{}' with ID: {}", group_name, group_id);
        }

        // Now get items with ALL column values (not just person column)
        if verbose {
            println!("\nGetting items with all column values...");
        }
        let items_query = format!(
            r#"
        {{
            boards(ids: ["{}"]) {{
                groups(ids: ["{}"]) {{
                    id
                    title
                    items_page(limit: {}) {{
                        cursor
                        items {{
                            id
                            name
                            column_values {{
                                id
                                value
                                text
                            }}
                        }}
                    }}
                }}
            }}
        }}
        "#,
            board_id, group_id, 500
        );

        if verbose {
            println!("Sending items query:\n{}", items_query);
        }

        let items_request_body = MondayRequest { query: items_query };
        let items_response = self.send_request(items_request_body, verbose).await?;

        if verbose {
            println!(
                "Items response: {}",
                &items_response[..500.min(items_response.len())]
            );
        }

        // Parse the response with better error handling
        let items_monday_response: Result<MondayResponse, _> =
            serde_json::from_str(&items_response);

        let items_monday_response = match items_monday_response {
            Ok(response) => response,
            Err(e) => {
                if verbose {
                    println!(
                        "Standard parsing failed: {}, trying manual extraction...",
                        e
                    );
                }
                manually_parse_response(&items_response).unwrap_or_else(|_| MondayResponse {
                    data: None,
                    errors: vec![MondayError {
                        message: format!("Failed to parse response: {}", e),
                        error_code: "PARSE_ERROR".to_string(),
                    }],
                })
            }
        };

        // Check for API errors
        if !items_monday_response.errors.is_empty() {
            let error_messages: Vec<String> = items_monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        // Extract the board with items from the response
        let mut items_board = items_monday_response
            .data
            .and_then(|data| data.boards)
            .and_then(|mut boards| boards.pop())
            .ok_or_else(|| anyhow!("No board data found in items response"))?;

        // Filter items locally by user
        if let Some(groups) = &mut items_board.groups {
            for group in groups {
                if let Some(items_page) = &mut group.items_page {
                    let original_count = items_page.items.len();
                    items_page.items.retain(|item| is_user_item(item, user_id));
                    if verbose {
                        println!(
                            "Filtered {} items down to {} items for user {}",
                            original_count,
                            items_page.items.len(),
                            user_id
                        );
                    }

                    // Limit to the requested number of items
                    if items_page.items.len() > limit {
                        items_page.items.truncate(limit);
                    }
                }
            }
        }

        // Merge the filtered results back into the original board structure
        let mut result_board = board.clone();

        // Replace just the items in the target group with the filtered items
        if let Some(result_groups) = &mut result_board.groups {
            if let Some(filtered_groups) = &items_board.groups {
                for result_group in result_groups {
                    if result_group.title == group_name {
                        if let Some(filtered_group) =
                            filtered_groups.iter().find(|g| g.id == result_group.id)
                        {
                            result_group.items_page = filtered_group.items_page.clone();
                        }
                        break;
                    }
                }
            }
        }

        Ok(result_board)
    }

    // NEW METHOD: Get board with all groups (without items)
    pub async fn get_board_with_groups(&self, board_id: &str, verbose: bool) -> Result<Board> {
        let query = format!(
            r#"
        {{
            boards(ids: ["{}"]) {{
                id
                name
                groups {{
                    id
                    title
                }}
            }}
        }}
        "#,
            board_id
        );

        if verbose {
            println!("Sending board groups query:\n{}", query);
        }

        let request_body = MondayRequest { query };
        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!(
                "Board groups response: {}",
                &response[..500.min(response.len())]
            );
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse board groups response: {}", e))?;

        // Check for API errors
        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        let board = monday_response
            .data
            .and_then(|data| data.boards)
            .and_then(|mut boards| boards.pop())
            .ok_or_else(|| anyhow!("No board found with ID {}", board_id))?;

        Ok(board)
    }

    pub async fn create_item_verbose(
        &self,
        board_id: &str,
        group_id: &str,
        item_name: &str,
        column_values: &serde_json::Value,
        verbose: bool,
    ) -> Result<String> {
        let query = format!(
            r#"
        mutation {{
            create_item(
                board_id: "{}",
                group_id: "{}",
                item_name: "{}",
                column_values: "{}"
            ) {{
                id
            }}
        }}
        "#,
            board_id,
            group_id,
            item_name,
            column_values.to_string().replace('"', "\\\"")
        );

        if verbose {
            println!("Sending create item mutation:\n{}", query);
        }

        let request_body = MondayRequest {
            query: query.to_string(),
        };

        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!("Create item response: {}", response);
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse create item response: {}", e))?;

        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        // Parse the raw response to get the created item ID
        let json_response: Value = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse response as JSON: {}", e))?;

        if let Some(data) = json_response.get("data") {
            if let Some(create_item) = data.get("create_item") {
                if let Some(id) = create_item.get("id") {
                    if let Some(id_str) = id.as_str() {
                        return Ok(id_str.to_string());
                    }
                }
            }
            Err(anyhow!(
                "Failed to extract item ID from create item response"
            ))
        } else {
            Err(anyhow!("No data returned from create item mutation"))
        }
    }
    pub async fn update_item_verbose(
        &self,
        item_id: &str,
        column_values: &serde_json::Value,
        verbose: bool,
    ) -> Result<()> {
        let query = format!(
            r#"
        mutation {{
            change_multiple_column_values(
                item_id: {},
                board_id: "6500270039",
                column_values: "{}"
            ) {{
                id
            }}
        }}
        "#,
            item_id,
            column_values.to_string().replace('"', "\\\"")
        );

        if verbose {
            println!("Sending update item mutation:\n{}", query);
        }

        let request_body = MondayRequest {
            query: query.to_string(),
        };

        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!("Update item response: {}", response);
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse update item response: {}", e))?;

        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        Ok(())
    }

    // Method to query ALL items in a group (without user filtering)
    pub async fn query_all_items_in_group(
        &self,
        board_id: &str,
        group_id: &str,
        limit: usize,
        verbose: bool,
    ) -> Result<Vec<Item>> {
        let mut all_items = Vec::new();
        let mut cursor: Option<String> = None;
        let page_size = 500;
        let mut total_pages = 0;

        if verbose {
            println!(
                "Starting paginated query for all items in group {}",
                group_id
            );
        }

        loop {
            total_pages += 1;

            // Build the query with proper cursor handling
            let query = if let Some(cursor_str) = &cursor {
                format!(
                    r#"
                    {{
                        boards(ids: ["{}"]) {{
                            groups(ids: ["{}"]) {{
                                items_page(limit: {}, cursor: "{}") {{
                                    cursor
                                    items {{
                                        id
                                        name
                                        column_values {{
                                            id
                                            value
                                            text
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                    "#,
                    board_id, group_id, page_size, cursor_str
                )
            } else {
                format!(
                    r#"
                    {{
                        boards(ids: ["{}"]) {{
                            groups(ids: ["{}"]) {{
                                items_page(limit: {}) {{
                                    cursor
                                    items {{
                                        id
                                        name
                                        column_values {{
                                            id
                                            value
                                            text
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                    "#,
                    board_id, group_id, page_size
                )
            };

            if verbose {
                println!(
                    "Sending paginated query (page {}, cursor: {:?})",
                    total_pages, cursor
                );
            }

            let request_body = MondayRequest { query };
            let response = self.send_request(request_body, verbose).await?;

            // Parse the response
            let value: Value = serde_json::from_str(&response)
                .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))?;

            // Extract items and cursor
            let (page_items, next_cursor) = extract_items_from_response(&value)
                .map_err(|e| anyhow!("Failed to extract items from response: {}", e))?;

            if verbose {
                println!("Page {}: Extracted {} items", total_pages, page_items.len());
            }

            all_items.extend(page_items);

            // Check if we have more pages or reached the limit
            if let Some(next_cursor_val) = next_cursor {
                if !next_cursor_val.is_empty() && all_items.len() < limit {
                    cursor = Some(next_cursor_val);

                    // Add a small delay to avoid rate limiting
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                } else {
                    break;
                }
            } else {
                break;
            }

            // Safety limit
            if total_pages > 100 {
                if verbose {
                    println!("Reached safety limit of 100 pages");
                }
                break;
            }
        }

        if verbose {
            println!("Total items collected: {}", all_items.len());
        }

        // Limit the final result
        if all_items.len() > limit {
            all_items.truncate(limit);
        }

        Ok(all_items)
    }

    // Method to query items with server-side filtering by user and dates
    pub async fn query_items_with_filters(
        &self,
        board_id: &str,
        group_id: &str,
        user_id: i64,
        dates: &[String],
        limit: usize,
        verbose: bool,
    ) -> Result<Vec<Item>> {
        if verbose {
            println!(
                "Querying items with server-side filters: user_id={}, dates={:?}",
                user_id, dates
            );
        }

        // Build the date compare_value array
        let date_values: Vec<String> = dates
            .iter()
            .flat_map(|date| vec!["EXACT".to_string(), date.clone()])
            .collect();
        let date_values_json = serde_json::to_string(&date_values)?;

        // Build the query with server-side filtering
        let query = format!(
            r#"
            {{
                boards(ids: ["{}"]) {{
                    groups(ids: ["{}"]) {{
                        items_page(
                            limit: {}
                            query_params: {{
                                rules: [
                                    {{
                                        column_id: "person"
                                        compare_value: ["person-{}"]
                                        operator: any_of
                                    }},
                                    {{
                                        column_id: "date4"
                                        compare_value: {}
                                        operator: any_of
                                    }}
                                ]
                                operator: and
                            }}
                        ) {{
                            cursor
                            items {{
                                id
                                name
                            column_values {{
                                id
                                value
                                text
                                }}
                            }}
                        }}
                    }}
                }}
            }}
            "#,
            board_id, group_id, limit, user_id, date_values_json
        );

        if verbose {
            println!("Sending server-side filtered query:\n{}", query);
        }

        let request_body = MondayRequest { query };
        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!("Response: {}", &response[..500.min(response.len())]);
        }

        // Parse the response
        let value: Value = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))?;

        // Extract items
        let (items, _cursor) = extract_items_from_response(&value)
            .map_err(|e| anyhow!("Failed to extract items from response: {}", e))?;

        if verbose {
            println!("Extracted {} items with server-side filtering", items.len());
        }

        Ok(items)
    }

    // NEW METHOD: Get an item by its ID
    pub async fn get_item_by_id(&self, item_id: &str, verbose: bool) -> Result<Option<Item>> {
        let query = format!(
            r#"
        {{
            items(ids: ["{}"]) {{
                id
                name
                column_values {{
                    id
                    value
                    text
                }}
            }}
        }}
        "#,
            item_id
        );

        if verbose {
            println!("Sending get item query:\n{}", query);
        }

        let request_body = MondayRequest { query };
        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!(
                "Get item response: {}",
                &response[..500.min(response.len())]
            );
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse get item response: {}", e))?;

        // Check for API errors
        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        // Extract the item from the response
        if let Some(data) = monday_response.data {
            // Try to get items from the items field first
            if let Some(items) = data.items {
                if !items.is_empty() {
                    return Ok(Some(items[0].clone()));
                }
            }

            // Alternative parsing for nested structures
            if let Some(boards) = data.boards {
                for board in boards {
                    if let Some(groups) = board.groups {
                        for group in groups {
                            if let Some(items_page) = group.items_page {
                                for item in items_page.items {
                                    if item.id.as_deref() == Some(item_id) {
                                        return Ok(Some(item));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    // NEW METHOD: Delete an item by its ID
    pub async fn delete_item(&self, item_id: &str, verbose: bool) -> Result<String> {
        let query = format!(
            r#"
        mutation {{
            delete_item (item_id: {}) {{
                id
            }}
        }}
        "#,
            item_id
        );

        if verbose {
            println!("Sending delete item mutation:\n{}", query);
        }

        let request_body = MondayRequest {
            query: query.to_string(),
        };

        let response = self.send_request(request_body, verbose).await?;

        if verbose {
            println!("Delete item response: {}", response);
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse delete item response: {}", e))?;

        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response
                .errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!(
                "Monday.com API errors: {}",
                error_messages.join(", ")
            ));
        }

        // Parse the response to get the deleted item ID
        if monday_response.data.is_some() {
            Ok(format!("Item {} deleted successfully", item_id))
        } else {
            Err(anyhow!("No data returned from delete item mutation"))
        }
    }

    async fn send_request(&self, request_body: MondayRequest, verbose: bool) -> Result<String> {
        if verbose {
            println!("Sending request to Monday.com API...");
        }

        let response = self
            .client
            .post("https://api.monday.com/v2")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .header("API-Version", "2023-10")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Monday.com: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Monday.com API error ({}): {}", status, error_text));
        }

        response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response text: {}", e))
    }

    pub async fn test_connection_verbose(&self, verbose: bool) -> Result<()> {
        self.get_current_user_verbose(verbose).await?;
        Ok(())
    }
}

// Helper function to manually parse response if standard parsing fails
fn manually_parse_response(response: &str) -> Result<MondayResponse, anyhow::Error> {
    let value: Value = serde_json::from_str(response)?;

    let mut boards = Vec::new();
    if let Some(data) = value.get("data") {
        if let Some(boards_array) = data.get("boards").and_then(|b| b.as_array()) {
            for board_val in boards_array {
                let mut board = Board {
                    id: None,
                    name: None,
                    groups: None,
                };

                if let Some(groups_array) = board_val.get("groups").and_then(|g| g.as_array()) {
                    let mut groups = Vec::new();
                    for group_val in groups_array {
                        let group_id = group_val
                            .get("id")
                            .and_then(|id| id.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let group_title = group_val
                            .get("title")
                            .and_then(|title| title.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let mut group = Group {
                            id: group_id,
                            title: group_title,
                            items_page: None,
                        };

                        if let Some(items_page_val) = group_val.get("items_page") {
                            let mut items_page = ItemsPage {
                                items: Vec::new(),
                                cursor: None,
                            };

                            if let Some(cursor_val) =
                                items_page_val.get("cursor").and_then(|c| c.as_str())
                            {
                                items_page.cursor = Some(cursor_val.to_string());
                            }

                            if let Some(items_array) =
                                items_page_val.get("items").and_then(|i| i.as_array())
                            {
                                for item_val in items_array {
                                    let item_id = item_val
                                        .get("id")
                                        .and_then(|id| id.as_str())
                                        .map(|s| s.to_string());
                                    let item_name = item_val
                                        .get("name")
                                        .and_then(|name| name.as_str())
                                        .map(|s| s.to_string());

                                    let mut item = Item {
                                        id: item_id,
                                        name: item_name,
                                        column_values: Vec::new(),
                                    };

                                    if let Some(columns_array) =
                                        item_val.get("column_values").and_then(|c| c.as_array())
                                    {
                                        for col_val in columns_array {
                                            let col_id = col_val
                                                .get("id")
                                                .and_then(|id| id.as_str())
                                                .map(|s| s.to_string());
                                            let col_value = col_val
                                                .get("value")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            let col_text = col_val
                                                .get("text")
                                                .and_then(|t| t.as_str())
                                                .map(|s| s.to_string());

                                            let column = ColumnValue {
                                                id: col_id,
                                                value: col_value,
                                                text: col_text,
                                            };
                                            item.column_values.push(column);
                                        }
                                    }

                                    items_page.items.push(item);
                                }
                            }

                            group.items_page = Some(items_page);
                        }

                        groups.push(group);
                    }

                    board.groups = Some(groups);
                }

                boards.push(board);
            }
        }
    }

    Ok(MondayResponse {
        data: Some(MondayData {
            me: None,
            boards: Some(boards),
            items: None,
        }),
        errors: Vec::new(),
    })
}

// Helper function to filter items by user
fn is_user_item(item: &Item, user_id: i64) -> bool {
    for col in &item.column_values {
        if let Some(value) = &col.value {
            if let Some(col_id) = &col.id {
                if col_id == "person" {
                    // Parse the JSON value to extract user IDs
                    if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                        if let Some(persons) = parsed_value.get("personsAndTeams") {
                            if let Some(persons_array) = persons.as_array() {
                                for person in persons_array {
                                    if let Some(person_id) =
                                        person.get("id").and_then(|id| id.as_i64())
                                    {
                                        if person_id == user_id {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monday_client_new() {
        let client = MondayClient::new("test-key".to_string());
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_deserialize_id_from_string() {
        let json_string = r#"{"id": "123"}"#;
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(deserialize_with = "deserialize_id")]
            id: i64,
        }

        let result: Result<TestStruct, _> = serde_json::from_str(json_string);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 123);
    }

    #[test]
    fn test_deserialize_id_from_number() {
        let json_number = r#"{"id": 123}"#;
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(deserialize_with = "deserialize_id")]
            id: i64,
        }

        let result: Result<TestStruct, _> = serde_json::from_str(json_number);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 123);
    }

    #[test]
    fn test_deserialize_string_id_from_string() {
        let json_string = r#"{"id": "test_id"}"#;
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(deserialize_with = "deserialize_string_id")]
            id: String,
        }

        let result: Result<TestStruct, _> = serde_json::from_str(json_string);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "test_id");
    }

    #[test]
    fn test_deserialize_string_id_from_number() {
        let json_number = r#"{"id": 123}"#;
        #[derive(Deserialize)]
        struct TestStruct {
            #[serde(deserialize_with = "deserialize_string_id")]
            id: String,
        }

        let result: Result<TestStruct, _> = serde_json::from_str(json_number);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "123");
    }
}
