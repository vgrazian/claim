use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::{Result, anyhow};
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
    #[serde(deserialize_with = "deserialize_string_id")]
    pub id: String,
    pub name: String,
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

impl MondayClient {
    pub fn new(api_key: String) -> Self {
        MondayClient {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_current_user(&self) -> Result<MondayUser> {
        self.get_current_user_verbose(false).await
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
            println!("User API response: {}", &response[..200.min(response.len())]);
        }

        let monday_response: MondayResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse Monday.com user response: {}", e))?;

        // Check for API errors
        if !monday_response.errors.is_empty() {
            let error_messages: Vec<String> = monday_response.errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!("Monday.com API errors: {}", error_messages.join(", ")));
        }

        monday_response.data
            .and_then(|data| data.me)
            .ok_or_else(|| anyhow!("No user data found in response"))
    }

    pub async fn query_board(
        &self,
        board_id: &str,
        group_name: &str,
        user_id: i64,
        limit: usize,
    ) -> Result<Board> {
        self.query_board_verbose(board_id, group_name, user_id, limit, false).await
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
            let error_messages: Vec<String> = monday_response.errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!("Monday.com API errors: {}", error_messages.join(", ")));
        }

        let board = monday_response.data
            .and_then(|data| data.boards)
            .and_then(|mut boards| boards.pop())
            .ok_or_else(|| anyhow!("No board found with ID {}", board_id))?;

        // Find the group with the matching name
        let group_id = board.groups
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
            board_id, group_id, 50
        );

        if verbose {
            println!("Sending items query:\n{}", items_query);
        }

        let items_request_body = MondayRequest { query: items_query };
        let items_response = self.send_request(items_request_body, verbose).await?;

        if verbose {
            println!("Items response: {}", &items_response[..500.min(items_response.len())]);
        }

        // Parse the response with better error handling
        let items_monday_response: Result<MondayResponse, _> = serde_json::from_str(&items_response);
        
        let items_monday_response = match items_monday_response {
            Ok(response) => response,
            Err(e) => {
                if verbose {
                    println!("Standard parsing failed: {}, trying manual extraction...", e);
                }
                manually_parse_response(&items_response).unwrap_or_else(|_| {
                    MondayResponse {
                        data: None,
                        errors: vec![MondayError {
                            message: format!("Failed to parse response: {}", e),
                            error_code: "PARSE_ERROR".to_string(),
                        }],
                    }
                })
            }
        };

        // Check for API errors
        if !items_monday_response.errors.is_empty() {
            let error_messages: Vec<String> = items_monday_response.errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!("Monday.com API errors: {}", error_messages.join(", ")));
        }

        // Extract the board with items from the response
        let mut items_board = items_monday_response.data
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
                        println!("Filtered {} items down to {} items for user {}", 
                                original_count, items_page.items.len(), user_id);
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
                        if let Some(filtered_group) = filtered_groups.iter().find(|g| g.id == result_group.id) {
                            result_group.items_page = filtered_group.items_page.clone();
                        }
                        break;
                    }
                }
            }
        }

        Ok(result_board)
    }

    pub async fn create_item(
        &self,
        board_id: &str,
        group_id: &str,
        item_name: &str,
        column_values: &serde_json::Value,
    ) -> Result<String> {
        self.create_item_verbose(board_id, group_id, item_name, column_values, false).await
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
            let error_messages: Vec<String> = monday_response.errors
                .iter()
                .map(|e| format!("{} (code: {})", e.message, e.error_code))
                .collect();
            return Err(anyhow!("Monday.com API errors: {}", error_messages.join(", ")));
        }

        // Parse the response to get the created item ID
        if let Some(data) = monday_response.data {
            // The response structure would need to be properly parsed based on the actual API response
            // For now, we'll return a success message
            Ok("success".to_string())
        } else {
            Err(anyhow!("No data returned from create item mutation"))
        }
    }

    async fn send_request(&self, request_body: MondayRequest, verbose: bool) -> Result<String> {
        if verbose {
            println!("Sending request to Monday.com API...");
        }
        
        let response = self.client
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Monday.com API error ({}): {}", status, error_text));
        }

        response.text()
            .await
            .map_err(|e| anyhow!("Failed to read response text: {}", e))
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.get_current_user().await?;
        Ok(())
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
                let board_id = board_val.get("id").and_then(|id| id.as_str()).unwrap_or("unknown").to_string();
                let board_name = board_val.get("name").and_then(|name| name.as_str()).unwrap_or("unknown").to_string();
                
                let mut board = Board {
                    id: board_id,
                    name: board_name,
                    groups: None,
                };
                
                if let Some(groups_array) = board_val.get("groups").and_then(|g| g.as_array()) {
                    let mut groups = Vec::new();
                    for group_val in groups_array {
                        let group_id = group_val.get("id").and_then(|id| id.as_str()).unwrap_or("unknown").to_string();
                        let group_title = group_val.get("title").and_then(|title| title.as_str()).unwrap_or("unknown").to_string();
                        
                        let mut group = Group {
                            id: group_id,
                            title: group_title,
                            items_page: None,
                        };
                        
                        if let Some(items_page_val) = group_val.get("items_page") {
                            let mut items_page = ItemsPage { items: Vec::new() };
                            
                            if let Some(items_array) = items_page_val.get("items").and_then(|i| i.as_array()) {
                                for item_val in items_array {
                                    let item_id = item_val.get("id").and_then(|id| id.as_str()).map(|s| s.to_string());
                                    let item_name = item_val.get("name").and_then(|name| name.as_str()).map(|s| s.to_string());
                                    
                                    let mut item = Item {
                                        id: item_id,
                                        name: item_name,
                                        column_values: Vec::new(),
                                    };
                                    
                                    if let Some(columns_array) = item_val.get("column_values").and_then(|c| c.as_array()) {
                                        for col_val in columns_array {
                                            let col_id = col_val.get("id").and_then(|id| id.as_str()).map(|s| s.to_string());
                                            let col_value = col_val.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
                                            let col_text = col_val.get("text").and_then(|t| t.as_str()).map(|s| s.to_string());
                                            
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
                                    if let Some(person_id) = person.get("id").and_then(|id| id.as_i64()) {
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