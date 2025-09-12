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
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Item {
    #[serde(deserialize_with = "deserialize_string_id")]
    pub id: String,
    pub name: String,
    pub column_values: Vec<ColumnValue>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ColumnValue {
    #[serde(deserialize_with = "deserialize_string_id")]
    pub id: String,
    pub value: Option<String>,
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
        let query = r#"
        {
            me {
                id
                name
                email
            }
        }
        "#;

        let request_body = MondayRequest {
            query: query.to_string(),
        };

        let response = self.send_request(request_body).await?;
        
        // Debug: print raw response
        let preview = if response.len() > 200 {
            format!("{}...", &response[..200])
        } else {
            response.clone()
        };
        println!("User API response: {}", preview);
        
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
        // First, let's get all groups to find the right one
        println!("Getting board structure to find groups...");
        
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

        println!("Sending board query: {}", query);

        let request_body = MondayRequest { query };
        let response = self.send_request(request_body).await?;
        
        // Print the response for debugging
        let preview = if response.len() > 500 {
            format!("{}...", &response[..500])
        } else {
            response.clone()
        };
        println!("Board response: {}", preview);

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

        println!("Found group '{}' with ID: {}", group_name, group_id);

        // Now query items for this group with correct GraphQL syntax
        // Based on the working example you provided
        let items_query = format!(
            r#"
        {{
            boards(ids: ["{}"]) {{
                groups(ids: ["{}"]) {{
                    items_page(limit: {}) {{
                        items {{
                            id
                            name
                            column_values {{
                                id
                                value
                            }}
                        }}
                    }}
                }}
            }}
        }}
        "#,
            board_id, group_id, limit
        );

        println!("Sending items query: {}", items_query);

        let items_request_body = MondayRequest { query: items_query };
        let items_response = self.send_request(items_request_body).await?;
        
        let items_preview = if items_response.len() > 500 {
            format!("{}...", &items_response[..500])
        } else {
            items_response.clone()
        };
        println!("Items response: {}", items_preview);

        let items_monday_response: MondayResponse = serde_json::from_str(&items_response)
            .map_err(|e| anyhow!("Failed to parse items response: {}", e))?;

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

        // Combine the board info with groups from the first query
        items_board.groups = board.groups;

        Ok(items_board)
    }

    async fn send_request(&self, request_body: MondayRequest) -> Result<String> {
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
}